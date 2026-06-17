%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(session_connection_guild_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

do_guild_connect_skips_session_connect_when_cached_unavailable_test() ->
    GuildId = 2002,
    Attempt = 3,
    Parent = self(),
    TestRef = make_ref(),
    GuildPid = spawn(fun() -> guild_stub_loop(Parent, TestRef) end),
    ManagerPid = spawn(fun() -> manager_stub_loop(GuildId, GuildPid) end),
    true = register(guild_manager, ManagerPid),
    Features = [<<"UNAVAILABLE_FOR_EVERYONE">>],
    GuildData = #{<<"guild">> => #{<<"features">> => Features}},
    CacheState = #{id => GuildId, data => GuildData},
    _ = guild_availability:update_unavailability_cache_for_state(CacheState),
    try
        ok = session_connection_guild:do_guild_connect(
            connect_context(Parent, GuildId, Attempt)
        ),
        {ok, Resp} = await_unavailable_result(GuildId, Attempt),
        ?assertEqual(true, maps:get(<<"unavailable">>, Resp)),
        ?assertNot(saw_stub_call(TestRef, 200))
    after
        safe_unregister(guild_manager, ManagerPid),
        ManagerPid ! stop,
        GuildPid ! stop,
        CleanupState = #{id => GuildId, data => #{<<"guild">> => #{<<"features">> => []}}},
        guild_availability:update_unavailability_cache_for_state(CleanupState)
    end.

do_guild_connect_uses_session_connect_async_cast_test() ->
    GuildId = 2004,
    Parent = self(),
    TestRef = make_ref(),
    GuildPid = spawn(fun() -> guild_stub_loop(Parent, TestRef) end),
    ManagerPid = spawn(fun() -> manager_stub_loop(GuildId, GuildPid) end),
    true = register(guild_manager, ManagerPid),
    SessionPid = spawn(fun capture_loop/0),
    try
        ok = session_connection_guild:do_guild_connect(connect_context(SessionPid, GuildId, 0)),
        ?assertMatch(
            {session_connect_async, _},
            await_stub_cast(TestRef, 1000)
        )
    after
        SessionPid ! stop,
        safe_unregister(guild_manager, ManagerPid),
        ManagerPid ! stop,
        GuildPid ! stop
    end.

do_guild_connect_start_or_lookup_after_lookup_miss_test() ->
    GuildId = 2005,
    Parent = self(),
    TestRef = make_ref(),
    GuildPid = spawn(fun() -> guild_stub_loop(Parent, TestRef) end),
    ManagerPid = spawn(fun() -> manager_lookup_miss_stub_loop(GuildId, GuildPid) end),
    true = register(guild_manager, ManagerPid),
    SessionPid = spawn(fun capture_loop/0),
    try
        ok = session_connection_guild:do_guild_connect(connect_context(SessionPid, GuildId, 0)),
        ?assertMatch(
            {session_connect_async, _},
            await_stub_cast(TestRef, 1000)
        )
    after
        SessionPid ! stop,
        safe_unregister(guild_manager, ManagerPid),
        ManagerPid ! stop,
        GuildPid ! stop
    end.

guild_connect_timeout_exhaustion_marks_unavailable_test() ->
    GuildId = 9001,
    Attempt = 25,
    State0 = exhausted_connect_state(<<"st1">>, GuildId, Attempt),
    {noreply, State1} = session_connection_guild:handle_guild_connect_timeout(
        GuildId, Attempt, State0
    ),
    [E] = maps:get(collected_guild_states, State1, []),
    ?assertEqual(true, maps:get(<<"unavailable">>, E)),
    ?assertEqual(unavailable, maps:get(GuildId, maps:get(guilds, State1))),
    ?assertNot(maps:is_key(GuildId, maps:get(guild_connect_inflight, State1, #{}))).

session_connect_failed_exhaustion_marks_unavailable_test() ->
    GuildId = 9002,
    Attempt = 25,
    State0 = (exhausted_connect_state(<<"st2">>, GuildId, undefined))#{
        presence_pid => undefined
    },
    {noreply, State1} = session_connection_guild:handle_result_internal(
        GuildId,
        Attempt,
        {error, {session_connect_failed, worker_crashed}},
        State0
    ),
    [E] = maps:get(collected_guild_states, State1, []),
    ?assertEqual(true, maps:get(<<"unavailable">>, E)),
    ?assertEqual(unavailable, maps:get(GuildId, maps:get(guilds, State1))).

guild_connect_worker_down_exhaustion_clears_inflight_and_marks_unavailable_test() ->
    GuildId = 9003,
    Attempt = 25,
    WorkerRef = make_ref(),
    State0 = (exhausted_connect_state(<<"st3">>, GuildId, Attempt))#{
        guild_connect_workers => #{WorkerRef => {GuildId, Attempt, self()}},
        presence_pid => undefined
    },
    {guild_connect_worker, {noreply, State1}} =
        session_connection_guild:handle_guild_connect_worker_down(WorkerRef, killed, State0),
    ?assertEqual(#{}, maps:get(guild_connect_inflight, State1)),
    ?assertEqual(#{}, maps:get(guild_connect_workers, State1)),
    ?assertEqual(unavailable, maps:get(GuildId, maps:get(guilds, State1))).

guild_connect_worker_normal_down_only_cleans_worker_monitor_test() ->
    GuildId = 9004,
    Attempt = 3,
    WorkerRef = make_ref(),
    State0 = (exhausted_connect_state(<<"st4">>, GuildId, Attempt))#{
        guild_connect_workers => #{WorkerRef => {GuildId, Attempt, self()}}
    },
    {guild_connect_worker, {noreply, State1}} =
        session_connection_guild:handle_guild_connect_worker_down(WorkerRef, normal, State0),
    ?assertEqual(#{GuildId => Attempt}, maps:get(guild_connect_inflight, State1)),
    ?assertEqual(#{}, maps:get(guild_connect_workers, State1)).

finalize_guild_connection_propagates_ready_stop_test() ->
    GuildId = 4242,
    GuildPid = spawn(fun wait_for_stop/0),
    try
        {stop, normal, S} = session_connection_guild:finalize_guild_connection(
            GuildId,
            GuildPid,
            #{guilds => #{}},
            fun(St) -> {stop, normal, St#{stopped => true}} end
        ),
        ?assert(maps:get(stopped, S))
    after
        GuildPid ! stop
    end.

repair_stalled_guild_connects_requeues_unresolved_guilds_test() ->
    State0 = #{
        id => <<"repair1">>,
        user_id => 100,
        guilds => #{
            1 => undefined,
            2 => unavailable,
            3 => cached_unavailable,
            4 => {self(), make_ref()}
        },
        guild_connect_inflight => #{2 => 1}
    },
    State1 = session_connection_guild:repair_stalled_guild_connects(State0),
    ?assert(is_integer(maps:get(guild_connect_last_repair_at, State1))),
    ?assertEqual([{guild_connect, 1, 0}], collect_guild_connect_messages([])).

repair_stalled_guild_connects_throttles_repair_test() ->
    Now = erlang:system_time(millisecond),
    State0 = #{
        id => <<"repair2">>,
        user_id => 100,
        guilds => #{1 => undefined},
        guild_connect_last_repair_at => Now
    },
    State1 = session_connection_guild:repair_stalled_guild_connects(State0),
    ?assertEqual(Now, maps:get(guild_connect_last_repair_at, State1)),
    ?assertEqual([], collect_guild_connect_messages([])).

repair_stalled_guild_connects_limits_batch_test() ->
    Guilds = maps:from_list([{Id, undefined} || Id <- lists:seq(1, 12)]),
    State0 = #{
        id => <<"repair3">>,
        user_id => 100,
        guilds => Guilds,
        guild_connect_last_repair_at => 0
    },
    _State1 = session_connection_guild:repair_stalled_guild_connects(State0),
    Expected = [{guild_connect, Id, 0} || Id <- lists:seq(1, 8)],
    ?assertEqual(Expected, collect_guild_connect_messages([])).

not_member_below_cap_schedules_retry_test() ->
    GuildId = 9201,
    Attempt = 0,
    State0 = #{
        id => <<"nm1">>,
        user_id => 100,
        guilds => #{GuildId => undefined},
        ready => undefined
    },
    {noreply, State1} = session_connection_guild:handle_result_internal(
        GuildId, Attempt, {error, not_member}, State0
    ),
    ?assert(maps:is_key(GuildId, maps:get(guilds, State1))),
    receive
        {guild_connect, GuildId, 1} -> ok
    after 5000 ->
        ?assert(false, not_member_retry_not_scheduled)
    end.

not_member_exhaustion_removes_guild_without_unavailable_marker_test() ->
    GuildId = 9202,
    OtherGuildId = 5550,
    Attempt = 3,
    State0 = (base_dispatch_state())#{
        guilds => #{
            GuildId => {self(), make_ref()},
            OtherGuildId => {self(), make_ref()}
        },
        guild_subscription_state => #{GuildId => #{}},
        active_guilds => sets:from_list([GuildId]),
        collected_guild_states => [],
        ready => undefined
    },
    {noreply, State1} = session_connection_guild:handle_result_internal(
        GuildId, Attempt, {error, not_member}, State0
    ),
    ?assertNot(maps:is_key(GuildId, maps:get(guilds, State1))),
    ?assertEqual([], maps:get(collected_guild_states, State1)),
    ?assert(maps:is_key(OtherGuildId, maps:get(guilds, State1))),
    ?assertNot(maps:is_key(GuildId, maps:get(guild_subscription_state, State1))),
    ?assertNot(sets:is_element(GuildId, maps:get(active_guilds, State1))),
    ?assertEqual([], collect_guild_connect_messages([])).

not_member_removal_unblocks_initial_ready_test() ->
    GuildId = 9203,
    Attempt = 3,
    State0 = (base_dispatch_state())#{
        guilds => #{GuildId => undefined},
        collected_guild_states => [],
        ready => #{<<"guilds">> => []},
        presence_pid => self(),
        socket_pid => undefined
    },
    ?assertMatch(
        {stop, normal, _},
        session_connection_guild:handle_result_internal(
            GuildId, Attempt, {error, not_member}, State0
        )
    ).

base_dispatch_state() ->
    #{
        id => <<"nm">>,
        user_id => 100,
        seq => 0,
        buffer => [],
        buffer_bytes => 0,
        socket_pid => undefined,
        channels => #{},
        relationships => #{},
        suppress_presence_updates => false,
        pending_presences => [],
        presence_pid => undefined,
        ignored_events => #{},
        debounce_reactions => false,
        reaction_buffer => [],
        reaction_buffer_timer => undefined
    }.

connect_context(SessionPid, GuildId, Attempt) ->
    #{
        session_pid => SessionPid,
        guild_id => GuildId,
        attempt => Attempt,
        session_id => <<"sa1">>,
        user_id => 77,
        bot => false,
        is_staff => false,
        initial_guild_id => undefined,
        peer_ip => undefined,
        user_data => #{<<"flags">> => <<"0">>}
    }.

exhausted_connect_state(SessionId, GuildId, Attempt) ->
    #{
        id => SessionId,
        user_id => 100,
        guilds => #{GuildId => undefined},
        guild_connect_inflight => maybe_inflight(GuildId, Attempt),
        collected_guild_states => [],
        ready => #{<<"guilds">> => []}
    }.

maybe_inflight(_GuildId, undefined) ->
    #{};
maybe_inflight(GuildId, Attempt) ->
    #{GuildId => Attempt}.

wait_for_stop() ->
    receive
        stop -> ok
    after 30000 -> ok
    end.

capture_loop() ->
    receive
        stop -> ok;
        _ -> capture_loop()
    after 30000 -> ok
    end.

await_stub_cast(Ref, T) ->
    receive
        {guild_stub_cast, Ref, M} -> M;
        _ -> await_stub_cast(Ref, T)
    after T -> timeout
    end.

await_unavailable_result(G, A) ->
    receive
        {guild_connect_result, G, A, {ok_cached_unavailable, R}} -> {ok, R};
        _ -> await_unavailable_result(G, A)
    after 1000 -> timeout
    end.

saw_stub_call(Ref, T) ->
    receive
        {guild_stub_called, Ref, _} -> true;
        _ -> saw_stub_call(Ref, T)
    after T -> false
    end.

safe_unregister(N, P) ->
    case whereis(N) of
        P -> unregister(N);
        _ -> ok
    end,
    ok.

manager_stub_loop(G, P) ->
    receive
        stop ->
            ok;
        {'$gen_call', F, {with_timeout, Request, _Timeout}} ->
            reply_manager_stub_request(G, P, F, Request),
            manager_stub_loop(G, P);
        {'$gen_call', F, Request} ->
            reply_manager_stub_request(G, P, F, Request),
            manager_stub_loop(G, P);
        _ ->
            manager_stub_loop(G, P)
    after 30000 -> ok
    end.

reply_manager_stub_request(G, P, F, {lookup, G}) ->
    gen_server:reply(F, {ok, P});
reply_manager_stub_request(G, P, F, {start_or_lookup, G}) ->
    gen_server:reply(F, {ok, P});
reply_manager_stub_request(_G, _P, F, _Request) ->
    gen_server:reply(F, {error, unsupported}).

manager_lookup_miss_stub_loop(G, P) ->
    receive
        stop ->
            ok;
        {'$gen_call', F, {with_timeout, Request, _Timeout}} ->
            reply_lookup_miss_stub_request(G, P, F, Request),
            manager_lookup_miss_stub_loop(G, P);
        {'$gen_call', F, Request} ->
            reply_lookup_miss_stub_request(G, P, F, Request),
            manager_lookup_miss_stub_loop(G, P);
        _ ->
            manager_lookup_miss_stub_loop(G, P)
    after 30000 -> ok
    end.

reply_lookup_miss_stub_request(G, _P, F, {lookup, G}) ->
    gen_server:reply(F, {error, not_found});
reply_lookup_miss_stub_request(G, P, F, {start_or_lookup, G}) ->
    gen_server:reply(F, {ok, P});
reply_lookup_miss_stub_request(_G, _P, F, _Request) ->
    gen_server:reply(F, {error, unsupported}).

collect_guild_connect_messages(Acc) ->
    receive
        {guild_connect, _GuildId, _Attempt} = Msg ->
            collect_guild_connect_messages([Msg | Acc])
    after 0 ->
        lists:reverse(Acc)
    end.

guild_stub_loop(Parent, Ref) ->
    receive
        stop ->
            ok;
        {'$gen_cast', M} ->
            Parent ! {guild_stub_cast, Ref, M},
            guild_stub_loop(Parent, Ref);
        {'$gen_call', F, R} ->
            Parent ! {guild_stub_called, Ref, R},
            gen_server:reply(F, {ok, #{}}),
            guild_stub_loop(Parent, Ref);
        _ ->
            guild_stub_loop(Parent, Ref)
    after 30000 -> ok
    end.
