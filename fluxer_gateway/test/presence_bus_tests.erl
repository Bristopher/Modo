%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(presence_bus_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

subscribe_publish_roundtrip_test() ->
    {ok, Pid} = maybe_start_for_test(),
    UserId = 99999,
    Payload = #{<<"status">> => <<"online">>},
    ?assertEqual(ok, presence_bus:subscribe(UserId)),
    ?assertEqual(ok, presence_bus:publish(UserId, Payload)),
    receive
        {presence, UserId, Payload} ->
            ok
    after 1000 ->
        ?assert(false)
    end,
    ?assertEqual(ok, presence_bus:unsubscribe(UserId)),
    ?assertEqual(ok, gen_server:stop(Pid)).

unsubscribe_stops_delivery_test() ->
    {ok, Pid} = maybe_start_for_test(),
    UserId = 88888,
    Payload = #{<<"status">> => <<"idle">>},
    presence_bus:subscribe(UserId),
    ?assertEqual(ok, presence_bus:unsubscribe(UserId)),
    ?assertEqual(ok, presence_bus:publish(UserId, Payload)),
    receive
        {presence, UserId, Payload} ->
            ?assert(false)
    after 300 ->
        ok
    end,
    ?assertEqual(ok, gen_server:stop(Pid)).

select_shard_test() ->
    ?assert(presence_bus:select_shard(100, 4) >= 0),
    ?assert(presence_bus:select_shard(100, 4) < 4).

find_shard_by_ref_test() ->
    Ref1 = make_ref(),
    Shards = #{0 => #{pid => self(), ref => Ref1}},
    ?assertEqual({ok, 0}, presence_bus:find_shard_by_ref(Ref1, Shards)),
    ?assertEqual(not_found, presence_bus:find_shard_by_ref(make_ref(), Shards)).

find_shard_by_pid_test() ->
    Shards = #{0 => #{pid => self(), ref => make_ref()}},
    ?assertEqual({ok, 0}, presence_bus:find_shard_by_pid(self(), Shards)),
    ?assertEqual(not_found, presence_bus:find_shard_by_pid(spawn(fun() -> ok end), Shards)).

diagnostic_info_returns_map_test() ->
    {ok, Pid} = maybe_start_for_test(),
    Info = presence_bus:diagnostic_info(),
    ?assert(is_map(Info)),
    ?assert(maps:is_key(shard_count, Info)),
    ?assertEqual(ok, gen_server:stop(Pid)).

publish_cross_node_does_not_crash_test() ->
    {ok, Pid} = maybe_start_for_test(),
    ?assertEqual(ok, presence_bus:publish_cross_node(12345, #{<<"status">> => <<"online">>})),
    ?assertEqual(ok, gen_server:stop(Pid)).

maybe_start_for_test() ->
    ensure_test_config(),
    case whereis(presence_bus) of
        undefined -> presence_bus:start_link();
        Existing when is_pid(Existing) -> {ok, Existing}
    end.

ensure_test_config() ->
    case persistent_term:get({fluxer_gateway, runtime_config}, undefined) of
        Config when is_map(Config) ->
            ok;
        _ ->
            persistent_term:put({fluxer_gateway, runtime_config}, #{presence_bus_shards => 1})
    end.
