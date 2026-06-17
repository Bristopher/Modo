%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(presence_manager_shards_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

determine_count_configured_test() ->
    with_runtime_config(presence_shards, 5, fun() ->
        ?assertMatch({5, configured}, presence_manager_shards:determine_count())
    end).

determine_count_auto_test() ->
    with_runtime_config(presence_shards, undefined, fun() ->
        {Count, auto} = presence_manager_shards:determine_count(),
        ?assert(Count > 0)
    end).

select_test() ->
    ?assert(presence_manager_shards:select(100, 4) >= 0),
    ?assert(presence_manager_shards:select(100, 4) < 4).

find_by_ref_test() ->
    Ref1 = make_ref(),
    Ref2 = make_ref(),
    Shards = #{0 => #{pid => self(), ref => Ref1}, 1 => #{pid => self(), ref => Ref2}},
    ?assertEqual({ok, 0}, presence_manager_shards:find_by_ref(Ref1, Shards)),
    ?assertEqual({ok, 1}, presence_manager_shards:find_by_ref(Ref2, Shards)),
    ?assertEqual(not_found, presence_manager_shards:find_by_ref(make_ref(), Shards)).

default_count_test() ->
    ?assert(presence_manager_shards:default_count() >= 1).

with_runtime_config(Key, Value, Fun) ->
    ensure_runtime_config(),
    Original = fluxer_gateway_env:get(Key),
    fluxer_gateway_env:patch(#{Key => Value}),
    Result = Fun(),
    restore_runtime_config(Key, Original),
    Result.

ensure_runtime_config() ->
    case persistent_term:get({fluxer_gateway, runtime_config}, undefined) of
        undefined -> persistent_term:put({fluxer_gateway, runtime_config}, #{});
        _ -> ok
    end.

restore_runtime_config(Key, undefined) ->
    fluxer_gateway_env:update(fun(Map) -> maps:remove(Key, Map) end);
restore_runtime_config(Key, Value) ->
    fluxer_gateway_env:update(fun(Map) -> Map#{Key => Value} end).
