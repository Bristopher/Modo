%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_manager_shard_reload).
-typing([eqwalizer]).

-include_lib("fluxer_gateway/include/timeout_config.hrl").

-export([
    do_reload_guild/3,
    do_reload_all_guilds/3,
    select_guilds_to_reload/2
]).
-export_type([guild_id/0, guild_ref/0, state/0]).

-define(BATCH_SIZE, 10).
-define(BATCH_DELAY_MS, 100).

-type guild_id() :: integer().
-type guild_ref() :: {pid(), reference()}.
-type fetch_result() :: {ok, map()} | {error, term()}.
-type state() :: map().

-spec do_reload_guild(guild_id(), gen_server:from(), state()) ->
    {reply, {error, not_found}, state()} | {noreply, state()}.
do_reload_guild(GuildId, From, State) ->
    Guilds = maps:get(guilds, State),
    GuildKey = process_registry:build_process_key(guild, GuildId),
    case maps:get(GuildId, Guilds, undefined) of
        {Pid, Ref} -> reload_tracked_guild(GuildId, GuildKey, Pid, Ref, From, State);
        _Other -> reload_registered_guild(GuildId, GuildKey, From, State)
    end.

-spec do_reload_all_guilds([guild_id()], gen_server:from(), state()) -> {noreply, state()}.
do_reload_all_guilds(GuildIds, From, State) ->
    GuildsToReload = select_guilds_to_reload(GuildIds, maps:get(guilds, State)),
    spawn(fun() -> reload_all_async(From, GuildsToReload) end),
    {noreply, State}.

-spec select_guilds_to_reload([guild_id()], #{guild_id() => guild_ref() | loading}) ->
    [{guild_id(), pid()}].
select_guilds_to_reload([], Guilds) ->
    maps:fold(fun select_reload_target/3, [], Guilds);
select_guilds_to_reload(GuildIds, Guilds) ->
    lists:filtermap(fun(GuildId) -> maybe_reload_target(GuildId, Guilds) end, GuildIds).

-spec select_reload_target(guild_id(), guild_ref() | loading, [{guild_id(), pid()}]) ->
    [{guild_id(), pid()}].
select_reload_target(GuildId, {Pid, _Ref}, Acc) ->
    case maybe_live_reload_target(GuildId, Pid) of
        {true, Target} -> [Target | Acc];
        false -> Acc
    end;
select_reload_target(_GuildId, loading, Acc) ->
    Acc.

-spec reload_tracked_guild(
    guild_id(), process_registry:process_key(), pid(), reference(), gen_server:from(), state()
) ->
    {reply, {error, not_found}, state()} | {noreply, state()}.
reload_tracked_guild(GuildId, GuildKey, Pid, Ref, From, State) ->
    case process_liveness:is_alive(Pid) of
        true ->
            spawn_reload(GuildId, Pid, From),
            {noreply, State};
        false ->
            demonitor(Ref, [flush]),
            process_registry:safe_unregister(GuildKey),
            Guilds = maps:remove(GuildId, maps:get(guilds, State)),
            do_reload_guild(GuildId, From, State#{guilds => Guilds})
    end.

-spec reload_registered_guild(
    guild_id(), process_registry:process_key(), gen_server:from(), state()
) ->
    {reply, {error, not_found}, state()} | {noreply, state()}.
reload_registered_guild(GuildId, GuildKey, From, State) ->
    case process_registry:registry_whereis(GuildKey) of
        undefined -> {reply, {error, not_found}, State};
        _ExistingPid -> lookup_registered_for_reload(GuildId, GuildKey, From, State)
    end.

-spec lookup_registered_for_reload(
    guild_id(), process_registry:process_key(), gen_server:from(), state()
) ->
    {reply, {error, not_found}, state()} | {noreply, state()}.
lookup_registered_for_reload(GuildId, GuildKey, From, State) ->
    Guilds = maps:get(guilds, State),
    case process_registry:lookup_or_monitor(GuildKey, GuildId, Guilds) of
        {ok, Pid, _Ref, NewGuilds} ->
            NewState = State#{guilds => NewGuilds},
            spawn_reload(GuildId, Pid, From),
            {noreply, NewState};
        {error, not_found} ->
            {reply, {error, not_found}, State}
    end.

-spec spawn_reload(guild_id(), pid(), gen_server:from()) -> pid().
spawn_reload(GuildId, Pid, From) ->
    spawn(fun() -> reload_one_async(GuildId, Pid, From) end).

-spec reload_one_async(guild_id(), pid(), gen_server:from()) -> ok.
reload_one_async(GuildId, Pid, From) ->
    erlang:process_flag(fullsweep_after, 0),
    reply_reload_result(Pid, From, safe_fetch(GuildId)).

-spec reply_reload_result(pid(), gen_server:from(), fetch_result()) -> ok.
reply_reload_result(Pid, From, {ok, Data}) ->
    _ = shard_utils:safe_gen_call_detailed(Pid, {reload, Data}, ?GUILD_CALL_TIMEOUT),
    gen_server:reply(From, ok);
reply_reload_result(_Pid, From, {error, _Reason}) ->
    gen_server:reply(From, {error, fetch_failed}).

-spec reload_all_async(gen_server:from(), [{guild_id(), pid()}]) -> ok.
reload_all_async(From, GuildsToReload) ->
    erlang:process_flag(fullsweep_after, 0),
    reload_guilds_in_batches(GuildsToReload),
    gen_server:reply(From, #{count => length(GuildsToReload)}).

-spec reload_guilds_in_batches([{guild_id(), pid()}]) -> ok.
reload_guilds_in_batches([]) ->
    ok;
reload_guilds_in_batches(Guilds) ->
    {Batch, Remaining} = lists:split(min(?BATCH_SIZE, length(Guilds)), Guilds),
    reload_batch(Batch),
    maybe_reload_remaining(Remaining).

-spec maybe_reload_remaining([{guild_id(), pid()}]) -> ok.
maybe_reload_remaining([]) ->
    ok;
maybe_reload_remaining(Remaining) ->
    ok = gateway_retry_timer:wait(?BATCH_DELAY_MS),
    reload_guilds_in_batches(Remaining).

-spec reload_batch([{guild_id(), pid()}]) -> ok.
reload_batch(Batch) ->
    lists:foreach(fun spawn_batch_reload/1, Batch).

-spec spawn_batch_reload({guild_id(), pid()}) -> pid().
spawn_batch_reload({GuildId, Pid}) ->
    spawn(fun() ->
        erlang:process_flag(fullsweep_after, 0),
        maybe_reload_process(Pid, safe_fetch(GuildId))
    end).

-spec maybe_reload_process(pid(), fetch_result()) -> ok.
maybe_reload_process(Pid, {ok, Data}) ->
    _ = shard_utils:safe_gen_call_detailed(Pid, {reload, Data}, ?GUILD_CALL_TIMEOUT),
    ok;
maybe_reload_process(_Pid, {error, _Reason}) ->
    ok.

-spec maybe_reload_target(guild_id(), map()) -> false | {true, {guild_id(), pid()}}.
maybe_reload_target(GuildId, Guilds) ->
    case maps:get(GuildId, Guilds, undefined) of
        {Pid, _Ref} -> maybe_live_reload_target(GuildId, Pid);
        _Other -> false
    end.

-spec maybe_live_reload_target(guild_id(), pid()) -> false | {true, {guild_id(), pid()}}.
maybe_live_reload_target(GuildId, Pid) ->
    case process_liveness:is_alive(Pid) of
        true -> {true, {GuildId, Pid}};
        false -> false
    end.

-spec safe_fetch(guild_id()) -> fetch_result().
safe_fetch(GuildId) ->
    try guild_manager_shard_fetch:fetch_guild_data(GuildId) of
        Result -> Result
    catch
        error:Reason -> {error, {fetch_failed, Reason}};
        exit:Reason -> {error, {fetch_exited, Reason}}
    end.
