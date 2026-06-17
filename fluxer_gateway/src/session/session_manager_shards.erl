%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(session_manager_shards).
-typing([eqwalizer]).

-export([
    ensure_shard/2,
    restart_shard/2,
    start_shards/1,
    ensure_shard_table/0,
    sync_shard_table/1,
    delete_shard_table/0
]).

-export_type([session_id/0, shard/0, state/0]).

-define(SHARD_TABLE, session_manager_shard_table).

-type session_id() :: binary().
-type shard() :: #{pid := pid(), ref := reference()}.
-type state() :: #{shards := #{non_neg_integer() => shard()}, shard_count := pos_integer()}.

-spec ensure_shard(session_id(), state()) -> {non_neg_integer(), state()}.
ensure_shard(SessionId, State) ->
    Count = maps:get(shard_count, State),
    Shards = maps:get(shards, State),
    Index = session_manager_routing:select_shard(SessionId, Count),
    case maps:get(Index, Shards, undefined) of
        undefined ->
            {_Result, NewState} = restart_shard(Index, State),
            {Index, NewState};
        #{pid := Pid} ->
            ensure_live_shard(SessionId, Index, Pid, State)
    end.

-spec ensure_live_shard(session_id(), non_neg_integer(), pid(), state()) ->
    {non_neg_integer(), state()}.
ensure_live_shard(_SessionId, Index, Pid, State) ->
    case process_liveness:is_alive(Pid) of
        true ->
            {Index, State};
        false ->
            {_Result, NewState} = restart_shard(Index, State),
            {Index, NewState}
    end.

-spec restart_shard(non_neg_integer(), state()) -> {shard() | {error, term()}, state()}.
restart_shard(Index, State) ->
    case start_shard(Index) of
        {ok, Shard} ->
            Shards = maps:get(shards, State),
            NewState = State#{shards := Shards#{Index => Shard}},
            sync_shard_table(NewState),
            {Shard, NewState};
        {error, Reason} ->
            Shards = maps:get(shards, State),
            NewState = State#{shards := maps:remove(Index, Shards)},
            {{error, Reason}, NewState}
    end.

-spec start_shards(pos_integer()) -> #{non_neg_integer() => shard()}.
start_shards(Count) ->
    lists:foldl(
        fun maybe_add_shard/2,
        #{},
        lists:seq(0, Count - 1)
    ).

-spec maybe_add_shard(non_neg_integer(), #{non_neg_integer() => shard()}) ->
    #{non_neg_integer() => shard()}.
maybe_add_shard(Index, Acc) ->
    case start_shard(Index) of
        {ok, Shard} -> Acc#{Index => Shard};
        {error, _Reason} -> Acc
    end.

-spec ensure_shard_table() -> ok.
ensure_shard_table() ->
    case ets:whereis(?SHARD_TABLE) of
        undefined ->
            _ = ets:new(?SHARD_TABLE, [named_table, public, set, {read_concurrency, true}]),
            ok;
        _ ->
            ok
    end.

-spec sync_shard_table(state()) -> ok.
sync_shard_table(State) ->
    ensure_shard_table(),
    _ = ets:delete_all_objects(?SHARD_TABLE),
    ShardCount = maps:get(shard_count, State),
    ets:insert(?SHARD_TABLE, {shard_count, ShardCount}),
    Shards = maps:get(shards, State),
    maps:foreach(
        fun(Index, #{pid := Pid}) -> put_shard_pid(Index, Pid) end,
        Shards
    ),
    ok.

-spec delete_shard_table() -> ok.
delete_shard_table() ->
    try ets:delete(?SHARD_TABLE) of
        true -> ok
    catch
        error:badarg -> ok
    end.

-spec start_shard(non_neg_integer()) -> {ok, shard()} | {error, term()}.
start_shard(Index) ->
    case session_manager_shard:start_link(Index) of
        {ok, Pid} ->
            Ref = erlang:monitor(process, Pid),
            put_shard_pid(Index, Pid),
            {ok, #{pid => Pid, ref => Ref}};
        ignore ->
            {error, ignore};
        Error ->
            Error
    end.

-spec put_shard_pid(non_neg_integer(), pid()) -> ok.
put_shard_pid(Index, Pid) ->
    ets:insert(?SHARD_TABLE, {{shard_pid, Index}, Pid}),
    ok.
