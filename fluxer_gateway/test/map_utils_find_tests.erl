%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(map_utils_find_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

find_by_field_basic_test() ->
    List = [
        #{id => 1, type => a},
        #{id => 2, type => b},
        #{id => 3, type => a},
        #{id => 4, type => c}
    ],

    ?assertEqual({ok, #{id => 2, type => b}}, map_utils:find_by_field(List, id, 2)),
    ?assertEqual({ok, #{id => 4, type => c}}, map_utils:find_by_field(List, id, 4)),

    ?assertEqual(error, map_utils:find_by_field(List, id, 999)),
    ?assertEqual(error, map_utils:find_by_field(List, type, nonexistent)),

    ?assertEqual(error, map_utils:find_by_field([], id, 1)).

find_by_field_multiple_matches_test() ->
    List = [
        #{id => 1, type => a, order => first},
        #{id => 2, type => b, order => second},
        #{id => 3, type => a, order => third},
        #{id => 4, type => c, order => fourth},
        #{id => 5, type => a, order => fifth}
    ],

    {ok, First} = map_utils:find_by_field(List, type, a),
    ?assertEqual(1, maps:get(id, First)),
    ?assertEqual(first, maps:get(order, First)),

    ?assertNotEqual(third, maps:get(order, First)),
    ?assertNotEqual(fifth, maps:get(order, First)),

    List2 = [
        #{name => <<"Alice">>, age => 25},
        #{name => <<"Bob">>, age => 30},
        #{name => <<"Charlie">>, age => 25},
        #{name => <<"Diana">>, age => 25}
    ],

    {ok, FirstAge25} = map_utils:find_by_field(List2, age, 25),
    ?assertEqual(<<"Alice">>, maps:get(name, FirstAge25)).

find_by_field_no_matches_test() ->
    List = [
        #{id => 1, type => a},
        #{id => 2, type => b},
        #{id => 3, type => c}
    ],

    ?assertEqual(error, map_utils:find_by_field(List, id, 999)),
    ?assertEqual(error, map_utils:find_by_field(List, type, z)),
    ?assertEqual(error, map_utils:find_by_field(List, missing_field, value)),
    ?assertEqual(error, map_utils:find_by_field(List, id, <<"wrong_type">>)),

    ?assertEqual(error, map_utils:find_by_field([], any_field, any_value)).

find_by_field_with_non_maps_test() ->
    MixedList = [
        not_a_map,
        123,
        #{id => 1, type => a},
        <<"binary">>,
        undefined,
        #{id => 2, type => b},
        [],
        #{id => 3, type => a}
    ],

    {ok, Found1} = map_utils:find_by_field(MixedList, type, a),
    ?assertEqual(1, maps:get(id, Found1)),

    {ok, Found2} = map_utils:find_by_field(MixedList, id, 2),
    ?assertEqual(b, maps:get(type, Found2)),

    MixedList2 = [atom, 456, {tuple}, #{id => 5, type => z}],
    ?assertEqual({ok, #{id => 5, type => z}}, map_utils:find_by_field(MixedList2, id, 5)),

    OnlyNonMaps = [atom, 123, <<"binary">>, {tuple}, []],
    ?assertEqual(error, map_utils:find_by_field(OnlyNonMaps, field, value)).

find_by_field_invalid_input_test() ->
    ?assertEqual(error, map_utils:find_by_field(not_a_list, field, value)),
    ?assertEqual(error, map_utils:find_by_field(#{}, field, value)),
    ?assertEqual(error, map_utils:find_by_field(123, field, value)),
    ?assertEqual(error, map_utils:find_by_field(<<"binary">>, field, value)),
    ?assertEqual(error, map_utils:find_by_field(undefined, field, value)),
    ?assertEqual(error, map_utils:find_by_field(atom, field, value)),
    ?assertEqual(error, map_utils:find_by_field({tuple}, field, value)).

find_by_field_complex_values_test() ->
    List = [
        #{<<"id">> => <<"first">>, <<"data">> => <<"value1">>},
        #{<<"id">> => <<"second">>, <<"data">> => <<"value2">>},
        #{<<"id">> => <<"third">>, <<"data">> => <<"value1">>}
    ],

    {ok, Found} = map_utils:find_by_field(List, <<"data">>, <<"value1">>),
    ?assertEqual(<<"first">>, maps:get(<<"id">>, Found)),

    ComplexList = [
        #{key => #{nested => value1}, id => 1},
        #{key => [1, 2, 3], id => 2},
        #{key => #{nested => value1}, id => 3}
    ],

    {ok, ComplexFound} = map_utils:find_by_field(ComplexList, key, #{nested => value1}),
    ?assertEqual(1, maps:get(id, ComplexFound)),

    {ok, ListFound} = map_utils:find_by_field(ComplexList, key, [1, 2, 3]),
    ?assertEqual(2, maps:get(id, ListFound)).
