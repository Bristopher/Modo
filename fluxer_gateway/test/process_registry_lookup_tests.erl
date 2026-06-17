%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(process_registry_lookup_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

-define(REGISTRY_TABLE, process_registry_table).

lookup_or_monitor_success_test() ->
    process_registry:init(),
    Key = {test_lookup, 3001},
    MapKey = test_key,
    ProcessMap = #{},

    Pid = spawn(fun() -> timer:sleep(200) end),
    ets:insert(?REGISTRY_TABLE, {Key, Pid}),

    Result = process_registry:lookup_or_monitor(Key, MapKey, ProcessMap),

    ?assertMatch({ok, Pid, _Ref, _NewMap}, Result),
    {ok, ReturnedPid, Ref, NewMap} = Result,

    ?assertEqual(Pid, ReturnedPid),
    ?assert(is_reference(Ref)),
    ?assertEqual({Pid, Ref}, maps:get(MapKey, NewMap)),

    process_registry:registry_unregister(Key).

lookup_or_monitor_not_found_test() ->
    process_registry:init(),
    Key = {test_lookup, 3002},
    MapKey = test_key,
    ProcessMap = #{},

    Result = process_registry:lookup_or_monitor(Key, MapKey, ProcessMap),
    ?assertEqual({error, not_found}, Result).

lookup_or_monitor_existing_map_test() ->
    process_registry:init(),
    Key = {test_lookup, 3003},
    MapKey = new_key,
    ExistingPid = list_to_pid("<0.100.0>"),
    ExistingRef = make_ref(),
    ProcessMap = #{existing_key => {ExistingPid, ExistingRef}},
    {_Pid, _Ref, NewMap} = lookup_registered(Key, MapKey, ProcessMap),

    ?assertEqual(2, maps:size(NewMap)),
    ?assert(maps:is_key(existing_key, NewMap)),
    ?assert(maps:is_key(MapKey, NewMap)),

    process_registry:registry_unregister(Key).

lookup_or_monitor_different_key_test() ->
    process_registry:init(),
    Key = {test_lookup, 3004},
    MapKey = different_key_name,
    ProcessMap = #{},
    {Pid, Ref, NewMap} = lookup_registered(Key, MapKey, ProcessMap),

    ?assertEqual({Pid, Ref}, maps:get(MapKey, NewMap)),
    ?assertEqual(false, maps:is_key(Key, NewMap)),

    process_registry:registry_unregister(Key).

lookup_or_monitor_dead_process_test() ->
    process_registry:init(),
    Key = {test_lookup, 3005},
    MapKey = test_key,
    ProcessMap = #{},

    Pid = spawn(fun() -> ok end),
    timer:sleep(10),
    ets:insert(?REGISTRY_TABLE, {Key, Pid}),

    Result = process_registry:lookup_or_monitor(Key, MapKey, ProcessMap),
    ?assertEqual({error, not_found}, Result).

safe_unregister_registered_test() ->
    process_registry:init(),
    Key = {test_unreg, 4001},

    Pid = spawn(fun() -> timer:sleep(100) end),
    ets:insert(?REGISTRY_TABLE, {Key, Pid}),

    ?assertEqual(Pid, process_registry:registry_whereis(Key)),
    ?assertEqual(ok, process_registry:safe_unregister(Key)),
    ?assertEqual(undefined, process_registry:registry_whereis(Key)).

safe_unregister_unregistered_test() ->
    process_registry:init(),
    ?assertEqual(ok, process_registry:safe_unregister({test_unreg, 4002})).

safe_unregister_multiple_test() ->
    process_registry:init(),
    Key = {test_unreg, 4003},

    Pid = spawn(fun() -> timer:sleep(100) end),
    ets:insert(?REGISTRY_TABLE, {Key, Pid}),

    ?assertEqual(ok, process_registry:safe_unregister(Key)),
    ?assertEqual(ok, process_registry:safe_unregister(Key)),
    ?assertEqual(ok, process_registry:safe_unregister(Key)).

safe_unregister_edge_cases_test() ->
    process_registry:init(),
    ?assertEqual(ok, process_registry:safe_unregister({test_unreg, 4004})),
    ?assertEqual(ok, process_registry:safe_unregister({test_unreg, 4005})),
    ?assertEqual(ok, process_registry:safe_unregister({test_unreg, 4006})).

lookup_or_monitor_reuses_existing_ref_for_same_pid_test() ->
    process_registry:init(),
    Key = {test_lookup, 3006},
    MapKey = same_pid_key,

    Pid = spawn(fun() -> timer:sleep(300) end),
    ets:insert(?REGISTRY_TABLE, {Key, Pid}),

    {ok, Pid, Ref1, Map1} = process_registry:lookup_or_monitor(Key, MapKey, #{}),
    {ok, Pid, Ref2, Map2} = process_registry:lookup_or_monitor(Key, MapKey, Map1),

    ?assertEqual(Ref1, Ref2),
    ?assertEqual({Pid, Ref1}, maps:get(MapKey, Map2)),

    process_registry:registry_unregister(Key).

safe_unregister_pid_conditional_keeps_other_owner_test() ->
    process_registry:init(),
    Key = {test_unreg, 4007},
    OldPid = list_to_pid("<0.100.0>"),
    NewPid = list_to_pid("<0.101.0>"),

    ets:insert(?REGISTRY_TABLE, {Key, NewPid}),

    ?assertEqual(ok, process_registry:safe_unregister(Key, OldPid)),
    ?assertEqual([{Key, NewPid}], ets:lookup(?REGISTRY_TABLE, Key)),

    ?assertEqual(ok, process_registry:safe_unregister(Key, NewPid)),
    ?assertEqual([], ets:lookup(?REGISTRY_TABLE, Key)).

lookup_registered(Key, MapKey, ProcessMap) ->
    Pid = spawn(fun() -> timer:sleep(200) end),
    ets:insert(?REGISTRY_TABLE, {Key, Pid}),
    {ok, Pid, Ref, NewMap} = process_registry:lookup_or_monitor(Key, MapKey, ProcessMap),
    {Pid, Ref, NewMap}.
