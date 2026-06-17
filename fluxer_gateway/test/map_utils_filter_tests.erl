%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(map_utils_filter_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

filter_by_field_basic_test() ->
    List = [
        #{id => 1, type => a, name => <<"first">>},
        #{id => 2, type => b, name => <<"second">>},
        #{id => 3, type => a, name => <<"third">>},
        #{id => 4, type => c},
        #{id => 5, type => a}
    ],

    Filtered = map_utils:filter_by_field(List, type, a),
    ?assertEqual(3, length(Filtered)),
    assert_all_type(a, Filtered),

    ?assertEqual(
        [#{id => 2, type => b, name => <<"second">>}],
        map_utils:filter_by_field(List, id, 2)
    ),

    ?assertEqual([], map_utils:filter_by_field(List, type, nonexistent)),

    ?assertEqual([], map_utils:filter_by_field(List, missing_field, value)).

filter_by_field_mixed_lists_test() ->
    MixedList = [
        #{id => 1, type => a},
        not_a_map,
        #{id => 2, type => b},
        123,
        #{id => 3, type => a},
        <<"binary">>,
        undefined,
        #{id => 4, type => a},
        [],
        {tuple, value},
        #{id => 5, type => c}
    ],

    Result = map_utils:filter_by_field(MixedList, type, a),
    ?assertEqual(3, length(Result)),
    ?assert(lists:all(fun is_map/1, Result)),
    assert_all_type(a, Result),

    Ids = [maps:get(id, M) || M <- Result],
    ?assertEqual([1, 3, 4], Ids),

    ResultB = map_utils:filter_by_field(MixedList, type, b),
    ?assertEqual(1, length(ResultB)),
    ?assertEqual([#{id => 2, type => b}], ResultB).

filter_by_field_edge_cases_test() ->
    ?assertEqual([], map_utils:filter_by_field([], field, value)),

    NonMaps = [123, atom, <<"binary">>, {tuple}, []],
    ?assertEqual([], map_utils:filter_by_field(NonMaps, field, value)),

    NoFieldList = [#{a => 1}, #{b => 2}, #{c => 3}],
    ?assertEqual([], map_utils:filter_by_field(NoFieldList, missing, value)),

    ?assertEqual([], map_utils:filter_by_field(not_a_list, field, value)),
    ?assertEqual([], map_utils:filter_by_field(#{}, field, value)),
    ?assertEqual([], map_utils:filter_by_field(123, field, value)),

    BinaryList = [
        #{<<"key">> => <<"value1">>},
        #{<<"key">> => <<"value2">>},
        #{<<"other">> => <<"value1">>}
    ],
    ?assertEqual(
        [#{<<"key">> => <<"value1">>}],
        map_utils:filter_by_field(BinaryList, <<"key">>, <<"value1">>)
    ),

    ComplexList = [
        #{data => #{nested => value}},
        #{data => [1, 2, 3]},
        #{data => #{nested => value}},
        #{other => data}
    ],
    ComplexFiltered = map_utils:filter_by_field(ComplexList, data, #{nested => value}),
    ?assertEqual(2, length(ComplexFiltered)).

assert_all_type(ExpectedType, Result) ->
    ?assert(lists:all(fun(M) -> maps:get(type, M) =:= ExpectedType end, Result)).
