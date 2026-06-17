%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(process_registry_integration_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

-define(REGISTRY_TABLE, process_registry_table).

integration_full_lifecycle_test() ->
    process_registry:init(),
    Id = 12345,
    Key = process_registry:build_process_key(guild, Id),
    ?assertEqual({guild, 12345}, Key),

    Pid = spawn_sleep(200),
    {_Ref, Map1} = register_and_assert_count(Key, Pid, 1, #{}),

    Map2 = maps:put(guild_67890, loading, Map1),
    ?assertEqual(1, process_registry:get_count(Map2)),

    OtherKey = process_registry:build_process_key(channel, 67890),
    OtherPid = spawn_sleep(200),
    ets:insert(?REGISTRY_TABLE, {OtherKey, OtherPid}),
    {ok, OtherPid, _OtherRef, Map3} = process_registry:lookup_or_monitor(
        OtherKey, channel_67890, Map2
    ),
    ?assertEqual(2, process_registry:get_count(Map3)),

    Map4 = process_registry:cleanup_on_down(Pid, Map3),
    ?assertEqual(1, process_registry:get_count(Map4)),
    ?assertEqual(loading, maps:get(guild_67890, Map4)),

    process_registry:safe_unregister(Key),
    process_registry:safe_unregister(OtherKey),

    ?assertEqual(undefined, process_registry:registry_whereis(Key)),
    ?assertEqual(undefined, process_registry:registry_whereis(OtherKey)).

integration_process_death_test_() ->
    {timeout, 10, fun() ->
        process_registry:init(),
        Key = {test_integration, 5001},

        Pid = spawn(fun death_probe_loop/0),

        {Ref, Map} = register_and_assert_count(Key, Pid, 1, #{}),

        Pid ! die,

        receive
            {'DOWN', Ref, process, Pid, _Reason} ->
                Map2 = process_registry:cleanup_on_down(Pid, Map),
                ?assertEqual(0, process_registry:get_count(Map2)),
                process_registry:safe_unregister(Key)
        after 500 ->
            ?assert(false)
        end
    end}.

integration_race_conditions_test_() ->
    {timeout, 10, fun() ->
        process_registry:init(),
        Key = {test_integration, 5002},

        Parent = self(),

        FirstPid = spawn_sleep(300),
        {_FirstRef, _Map1} = register_and_assert_count(Key, FirstPid, 1, #{}),

        Workers = [spawn_register_worker(Parent, Key) || _ <- lists:seq(1, 3)],

        Results = [
            receive
                {register_result, R} -> R
            after 1000 -> timeout
            end
         || _ <- Workers
        ],
        AllGotFirstPid = lists:all(
            fun
                ({ok, P, _, _}) -> P =:= FirstPid;
                (_) -> false
            end,
            Results
        ),
        ?assert(AllGotFirstPid),

        process_registry:safe_unregister(Key)
    end}.

integration_rapid_cycles_test_() ->
    {timeout, 10, fun() ->
        process_registry:init(),
        lists:foreach(fun rapid_cycle/1, lists:seq(1, 10))
    end}.

force_stop_process_normal_test_() ->
    {timeout, 15, fun() ->
        Pid = spawn(fun wait_for_stop_loop/0),
        ?assert(process_liveness:is_alive(Pid)),
        process_registry:force_stop_process(Pid),
        timer:sleep(50),
        ?assertEqual(false, process_liveness:is_alive(Pid))
    end}.

force_stop_process_already_dead_test() ->
    Pid = spawn(fun() -> ok end),
    timer:sleep(10),
    ?assertEqual(false, process_liveness:is_alive(Pid)),
    process_registry:force_stop_process(Pid).

force_stop_process_kills_unresponsive_test_() ->
    {timeout, 15, fun() ->
        Pid = spawn(fun trapped_wait_loop/0),
        ?assert(process_liveness:is_alive(Pid)),
        process_registry:force_stop_process(Pid),
        timer:sleep(100),
        ?assertEqual(false, process_liveness:is_alive(Pid))
    end}.

register_and_monitor_duplicate_stops_loser_test_() ->
    {timeout, 15, fun() ->
        process_registry:init(),
        Key = {test_reg, 1007},
        WinnerPid = spawn_sleep(5000),
        ets:insert(?REGISTRY_TABLE, {Key, WinnerPid}),
        LoserPid = spawn(fun trapped_wait_loop/0),
        ?assert(process_liveness:is_alive(LoserPid)),
        Result = process_registry:register_and_monitor(Key, LoserPid, #{}),
        ?assertMatch({ok, WinnerPid, _, _}, Result),
        timer:sleep(100),
        ?assertEqual(false, process_liveness:is_alive(LoserPid)),
        process_registry:registry_unregister(Key)
    end}.

spawn_sleep(Milliseconds) ->
    spawn(fun() -> timer:sleep(Milliseconds) end).

death_probe_loop() ->
    receive
        die -> exit(normal)
    after 100 ->
        exit(normal)
    end.

wait_for_stop_loop() ->
    receive
        stop -> ok
    after infinity ->
        ok
    end.

trapped_wait_loop() ->
    process_flag(trap_exit, true),
    receive
        never_arrives -> ok
    after infinity ->
        ok
    end.

spawn_register_worker(Parent, Key) ->
    spawn(fun() ->
        NewPid = spawn_sleep(100),
        Result = process_registry:register_and_monitor(Key, NewPid, #{}),
        Parent ! {register_result, Result}
    end).

rapid_cycle(N) ->
    Key = {test_rapid, N},
    Pid = spawn_sleep(50),
    {_Ref, _Map} = register_and_assert_count(Key, Pid, 1, #{}),
    process_registry:safe_unregister(Key),
    ?assertEqual(undefined, process_registry:registry_whereis(Key)).

register_and_assert_count(Key, Pid, Count, ProcessMap) ->
    {ok, Pid, Ref, Map} = process_registry:register_and_monitor(Key, Pid, ProcessMap),
    ?assertEqual(Count, process_registry:get_count(Map)),
    {Ref, Map}.
