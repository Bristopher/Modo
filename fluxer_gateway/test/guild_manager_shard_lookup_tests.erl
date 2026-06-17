%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_manager_shard_lookup_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

collect_active_guild_ids_filters_loading_and_dead_entries_test() ->
    LivePid = spawn(fun() ->
        receive
            stop -> ok
        after infinity ->
            ok
        end
    end),
    DeadPid = spawn(fun() -> ok end),
    timer:sleep(10),
    Guilds = #{
        1 => {LivePid, make_ref()},
        2 => loading,
        3 => {DeadPid, make_ref()}
    },
    ?assertEqual([1], guild_manager_shard_lookup:collect_active_guild_ids(Guilds)),
    LivePid ! stop.

do_start_or_lookup_loading_deduplicates_requests_test() ->
    GuildId = 4444,
    From1 = {self(), make_ref()},
    From2 = {self(), make_ref()},
    State0 = #{guilds => #{GuildId => loading}, pending_requests => #{}, shard_index => 0},
    {noreply, State1} = guild_manager_shard_lookup:do_start_or_lookup(GuildId, From1, State0),
    Pending1 = maps:get(pending_requests, State1),
    ?assertEqual([From1], maps:get(GuildId, Pending1)),
    {noreply, State2} = guild_manager_shard_lookup:do_start_or_lookup(GuildId, From2, State1),
    Requests = maps:get(GuildId, maps:get(pending_requests, State2)),
    ?assertEqual(2, length(Requests)),
    ?assert(lists:member(From1, Requests)),
    ?assert(lists:member(From2, Requests)).

do_lookup_returns_existing_pid_from_state_test() ->
    GuildId = 5151,
    GuildPid = self(),
    State0 = #{
        guilds => #{GuildId => {GuildPid, make_ref()}},
        pending_requests => #{},
        shard_index => 0
    },
    {reply, {ok, GuildPid}, State1} = guild_manager_shard_lookup:do_lookup(GuildId, State0),
    ?assertEqual(State0, State1).

do_lookup_returns_not_found_when_loading_test() ->
    GuildId = 6161,
    State0 = #{guilds => #{GuildId => loading}, pending_requests => #{}, shard_index => 0},
    ?assertEqual(
        {reply, {error, not_found}, State0},
        guild_manager_shard_lookup:do_lookup(GuildId, State0)
    ).

reply_to_all_empty_list_test() ->
    ?assertEqual(ok, guild_manager_shard_lookup:reply_to_all([], ok)).

lookup_or_fetch_rejects_zero_guild_rollout_test() ->
    process_registry:init(),
    OldConfig = gateway_rollout_config:get(),
    persistent_term:put(gateway_rollout_config, OldConfig#{<<"guild_rollout_percentage">> => 0}),
    GuildId = 99999,
    From = {self(), make_ref()},
    State = #{guilds => #{}, pending_requests => #{}, shard_index => 0},
    try
        Result = guild_manager_shard_lookup:lookup_or_fetch(GuildId, From, State),
        ?assertMatch({reply, {error, not_eligible}, _}, Result)
    after
        persistent_term:put(gateway_rollout_config, OldConfig)
    end.

do_ensure_started_rejects_zero_guild_rollout_test() ->
    process_registry:init(),
    OldConfig = gateway_rollout_config:get(),
    persistent_term:put(gateway_rollout_config, OldConfig#{<<"guild_rollout_percentage">> => 0}),
    GuildId = 99998,
    State = #{guilds => #{}, pending_requests => #{}, shard_index => 0},
    try
        Result = guild_manager_shard_lookup:do_ensure_started(GuildId, State),
        ?assertMatch({reply, {error, not_eligible}, _}, Result)
    after
        persistent_term:put(gateway_rollout_config, OldConfig)
    end.

stale_fetch_success_uses_superseding_tracked_guild_test() ->
    GuildId = 7321,
    ReplyRef = make_ref(),
    WorkerRef = make_ref(),
    FetchToken = make_ref(),
    SupersedingPid = spawn(fun() ->
        receive
            stop -> ok
        after infinity ->
            ok
        end
    end),
    timer:sleep(10),
    State0 = #{
        guilds => #{GuildId => {SupersedingPid, make_ref()}},
        pending_requests => #{GuildId => [{self(), ReplyRef}]},
        fetch_workers => #{WorkerRef => {GuildId, self(), FetchToken}},
        shard_index => 0
    },
    try
        {noreply, State1} = guild_manager_shard_lookup:handle_guild_data_fetched(
            GuildId,
            FetchToken,
            {ok, #{<<"guild">> => #{<<"id">> => <<"7321">>, <<"features">> => []}}},
            State0
        ),
        {TrackedPid, _Ref} = maps:get(GuildId, maps:get(guilds, State1)),
        ?assertEqual(SupersedingPid, TrackedPid),
        ?assertEqual(#{}, maps:get(pending_requests, State1)),
        receive
            {ReplyRef, {ok, SupersedingPid}} -> ok
        after 100 ->
            ?assert(false)
        end
    after
        SupersedingPid ! stop
    end.
