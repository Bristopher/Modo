%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_runtime_probe).
-typing([eqwalizer]).

-export([
    snapshot/0,
    top_processes/2,
    sample_processes/2,
    top_guilds/1,
    sample_guilds/2,
    guild_probe/1,
    logger_status/0,
    gc_logger/0
]).

-define(MAX_LIMIT, 100).
-define(DEFAULT_LIMIT, 20).
-define(MAX_SAMPLE_MS, 5000).
-define(DEFAULT_SAMPLE_MS, 250).
-define(MIN_SAMPLE_MS, 10).
-define(STATE_TIMEOUT_MS, 50).
-define(MAX_PROCESS_SCAN, 5000).
-define(MAX_GUILD_PID_SCAN, 5000).
-define(LOGGER_HANDLER, logger_simple_h).
-define(LOGGER_GC_MAX_QUEUE, 1000).

-type metric() :: memory | message_queue_len | reductions | total_heap_size.
-type row() :: map().

-export_type([metric/0, row/0]).

-spec snapshot() -> map().
snapshot() ->
    #{
        node => node(),
        memory => erlang:memory(),
        process_count => erlang:system_info(process_count),
        run_queue => erlang:statistics(run_queue),
        scheduler_wall_time => safe_scheduler_wall_time(),
        reductions => erlang:statistics(reductions)
    }.

-spec top_processes(metric() | term(), pos_integer()) -> [row()].
top_processes(memory, Limit) ->
    top_by_info(memory, Limit);
top_processes(message_queue_len, Limit) ->
    top_by_info(message_queue_len, Limit);
top_processes(reductions, Limit) ->
    top_by_info(reductions, Limit);
top_processes(total_heap_size, Limit) ->
    top_by_info(total_heap_size, Limit);
top_processes(_, Limit) ->
    top_by_info(memory, Limit).

-spec sample_processes(integer(), pos_integer()) -> [row()].
sample_processes(Milliseconds, Limit) ->
    Pids = process_scan_pids(),
    Before = reductions_by_pid(Pids),
    ok = gateway_retry_timer:wait(clamp_ms(Milliseconds)),
    After = reductions_by_pid(Pids),
    Rows = [
        enrich_pid(Pid, #{reduction_delta => Delta})
     || {Pid, R1} <- maps:to_list(After),
        R0 <- [maps:get(Pid, Before, R1)],
        Delta <- [R1 - R0],
        Delta > 0
    ],
    take(Limit, lists:sort(fun row_reduction_delta_ge/2, Rows)).

-spec top_guilds(pos_integer()) -> [row()].
top_guilds(Limit) ->
    Rows = [guild_row(GuildId, Pid) || {GuildId, Pid} <- local_guild_pids()],
    take(Limit, lists:sort(fun row_memory_ge/2, Rows)).

-spec sample_guilds(integer(), pos_integer()) -> [row()].
sample_guilds(Milliseconds, Limit) ->
    Guilds = local_guild_pids(),
    Before = maps:from_list([{Pid, reductions(Pid)} || {_GuildId, Pid} <- Guilds]),
    ok = gateway_retry_timer:wait(clamp_ms(Milliseconds)),
    Rows = [
        (guild_row(GuildId, Pid))#{
            reduction_delta => reductions(Pid) - maps:get(Pid, Before, reductions(Pid))
        }
     || {GuildId, Pid} <- Guilds
    ],
    take(Limit, lists:sort(fun row_reduction_delta_ge/2, Rows)).

-spec guild_probe(integer()) -> row().
guild_probe(GuildId) when is_integer(GuildId) ->
    try guild_manager:lookup(GuildId) of
        {ok, Pid} when is_pid(Pid) ->
            guild_row(GuildId, Pid);
        Other ->
            #{guild_id => GuildId, lookup => Other}
    catch
        Class:Reason ->
            #{guild_id => GuildId, lookup => {error, {Class, Reason}}}
    end.

-spec logger_status() -> row().
logger_status() ->
    case whereis(?LOGGER_HANDLER) of
        Pid when is_pid(Pid) -> enrich_pid(Pid, #{handler => ?LOGGER_HANDLER});
        undefined -> #{handler => ?LOGGER_HANDLER, status => not_found}
    end.

-spec gc_logger() -> row().
gc_logger() ->
    case whereis(?LOGGER_HANDLER) of
        Pid when is_pid(Pid) -> gc_logger_pid(Pid);
        undefined -> #{handler => ?LOGGER_HANDLER, status => not_found}
    end.

-spec gc_logger_pid(pid()) -> row().
gc_logger_pid(Pid) ->
    Before = enrich_pid(Pid, #{handler => ?LOGGER_HANDLER}),
    case maps:get(message_queue_len, Before, 0) of
        Len when Len =< ?LOGGER_GC_MAX_QUEUE ->
            Result = erlang:garbage_collect(Pid, [{type, major}]),
            #{
                handler => ?LOGGER_HANDLER,
                status => ok,
                result => Result,
                before => Before,
                after_info => enrich_pid(Pid, #{handler => ?LOGGER_HANDLER})
            };
        Len ->
            #{
                handler => ?LOGGER_HANDLER,
                status => skipped,
                reason => logger_queue_not_idle,
                message_queue_len => Len,
                max_queue => ?LOGGER_GC_MAX_QUEUE,
                before => Before
            }
    end.

-spec top_by_info(metric(), pos_integer()) -> [row()].
top_by_info(Key, Limit) ->
    Rows = [enrich_pid(Pid, #{}) || Pid <- process_scan_pids()],
    Sorted = lists:sort(fun(A, B) -> maps:get(Key, A, 0) >= maps:get(Key, B, 0) end, Rows),
    take(Limit, Sorted).

-spec local_guild_pids() -> [{integer(), pid()}].
local_guild_pids() ->
    try bounded_ets_rows(guild_pid_cache, ?MAX_GUILD_PID_SCAN) of
        Rows ->
            lists:usort([
                {GuildId, Pid}
             || {GuildId, Pid} <- Rows,
                is_integer(GuildId),
                is_pid(Pid),
                node(Pid) =:= node()
            ])
    catch
        error:badarg -> []
    end.

-spec process_scan_pids() -> [pid()].
process_scan_pids() ->
    lists:sublist(erlang:processes(), ?MAX_PROCESS_SCAN).

-spec bounded_ets_rows(ets:table(), pos_integer()) -> [term()].
bounded_ets_rows(Table, Limit) ->
    MatchSpec = [{{'$1', '$2'}, [], [{{'$1', '$2'}}]}],
    case ets:select(Table, MatchSpec, Limit) of
        {Rows, _Continuation} -> Rows;
        '$end_of_table' -> []
    end.

-spec guild_row(integer(), pid()) -> row().
guild_row(GuildId, Pid) ->
    Base = enrich_pid(Pid, #{guild_id => GuildId}),
    case safe_state(Pid) of
        State when is_map(State) ->
            Data = map_utils:ensure_map(maps:get(data, State, #{})),
            Guild = map_utils:ensure_map(maps:get(<<"guild">>, Data, #{})),
            Base#{
                guild_name => maps:get(<<"name">>, Guild, <<"Unknown">>),
                member_count => safe_member_count(Data),
                session_count => map_size(map_utils:ensure_map(maps:get(sessions, State, #{}))),
                presence_count => map_size(
                    map_utils:ensure_map(maps:get(presences, State, #{}))
                )
            };
        _ ->
            Base#{state => unavailable}
    end.

-spec enrich_pid(pid(), map()) -> row().
enrich_pid(Pid, Extra) ->
    Info = info_map(Pid),
    maps:merge(Info#{pid => pid_to_list(Pid)}, Extra).

-spec info_map(pid()) -> map().
info_map(Pid) ->
    case
        erlang:process_info(Pid, [
            memory,
            message_queue_len,
            reductions,
            current_function,
            initial_call,
            registered_name,
            total_heap_size,
            heap_size,
            stack_size,
            garbage_collection
        ])
    of
        undefined ->
            #{alive => false};
        Info when is_list(Info) ->
            maps:from_list(Info)
    end.

-spec safe_state(pid()) -> term().
safe_state(Pid) ->
    try sys:get_state(Pid, ?STATE_TIMEOUT_MS) of
        State -> State
    catch
        _:_ -> unavailable
    end.

-spec safe_member_count(term()) -> non_neg_integer().
safe_member_count(Data) ->
    try guild_data_index:member_count(Data) of
        Count when is_integer(Count), Count >= 0 -> Count;
        _ -> 0
    catch
        _:_ -> 0
    end.

-spec reductions_by_pid([pid()]) -> #{pid() => non_neg_integer()}.
reductions_by_pid(Pids) ->
    maps:from_list([{Pid, reductions(Pid)} || Pid <- Pids]).

-spec reductions(pid()) -> non_neg_integer().
reductions(Pid) ->
    case erlang:process_info(Pid, reductions) of
        {reductions, R} when is_integer(R), R >= 0 -> R;
        _ -> 0
    end.

-spec safe_scheduler_wall_time() -> term().
safe_scheduler_wall_time() ->
    try erlang:statistics(scheduler_wall_time) of
        Value -> Value
    catch
        _:_ -> unavailable
    end.

-spec clamp_ms(term()) -> pos_integer().
clamp_ms(Ms) when is_integer(Ms), Ms >= ?MIN_SAMPLE_MS, Ms =< ?MAX_SAMPLE_MS -> Ms;
clamp_ms(Ms) when is_integer(Ms), Ms > ?MAX_SAMPLE_MS -> ?MAX_SAMPLE_MS;
clamp_ms(_) -> ?DEFAULT_SAMPLE_MS.

-spec take(term(), [row()]) -> [row()].
take(Limit, Rows) when is_integer(Limit), Limit > 0 ->
    lists:sublist(Rows, min(Limit, ?MAX_LIMIT));
take(_, Rows) ->
    lists:sublist(Rows, ?DEFAULT_LIMIT).

-spec row_memory_ge(row(), row()) -> boolean().
row_memory_ge(A, B) ->
    maps:get(memory, A, 0) >= maps:get(memory, B, 0).

-spec row_reduction_delta_ge(row(), row()) -> boolean().
row_reduction_delta_ge(A, B) ->
    maps:get(reduction_delta, A, 0) >= maps:get(reduction_delta, B, 0).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

snapshot_returns_basic_runtime_info_test() ->
    Snapshot = snapshot(),
    ?assert(is_atom(maps:get(node, Snapshot))),
    ?assert(is_list(maps:get(memory, Snapshot))),
    ?assert(is_integer(maps:get(process_count, Snapshot))).

top_processes_is_bounded_test() ->
    Rows = top_processes(memory, 2),
    ?assert(length(Rows) =< 2),
    ?assert(lists:all(fun(Row) -> maps:is_key(pid, Row) end, Rows)).

sample_processes_is_bounded_test() ->
    Rows = sample_processes(10, 2),
    ?assert(length(Rows) =< 2).

process_scan_pids_is_bounded_test() ->
    ?assert(length(process_scan_pids()) =< ?MAX_PROCESS_SCAN).

bounded_ets_rows_respects_limit_test() ->
    Table = ets:new(?MODULE, [set]),
    try
        true = ets:insert(Table, [{1, self()}, {2, self()}, {3, self()}]),
        Rows = bounded_ets_rows(Table, 2),
        ?assertEqual(2, length(Rows))
    after
        ets:delete(Table)
    end.

guild_probe_without_manager_does_not_crash_test() ->
    Row = guild_probe(1),
    ?assertEqual(1, maps:get(guild_id, Row)).

-endif.
