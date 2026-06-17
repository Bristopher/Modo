%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(map_utils_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

get_safe_basic_test() ->
    Map = #{key => value, number => 42},

    ?assertEqual(value, map_utils:get_safe(Map, key, default)),
    ?assertEqual(42, map_utils:get_safe(Map, number, 0)),

    ?assertEqual(default, map_utils:get_safe(Map, missing, default)),
    ?assertEqual(0, map_utils:get_safe(Map, missing, 0)).

get_safe_various_input_types_test() ->
    ?assertEqual(default, map_utils:get_safe(not_a_map, key, default)),
    ?assertEqual(default, map_utils:get_safe([], key, default)),
    ?assertEqual(default, map_utils:get_safe(123, key, default)),
    ?assertEqual(default, map_utils:get_safe(<<"binary">>, key, default)),
    ?assertEqual(default, map_utils:get_safe(undefined, key, default)),
    ?assertEqual(default, map_utils:get_safe(atom, key, default)),
    ?assertEqual(default, map_utils:get_safe({tuple, value}, key, default)),
    ?assertEqual(default, map_utils:get_safe(self(), key, default)).

get_safe_various_key_types_test() ->
    Map = #{
        atom_key => atom_value,
        <<"binary_key">> => binary_value,
        123 => number_key_value,
        {tuple, key} => tuple_key_value
    },

    ?assertEqual(atom_value, map_utils:get_safe(Map, atom_key, default)),
    ?assertEqual(binary_value, map_utils:get_safe(Map, <<"binary_key">>, default)),
    ?assertEqual(number_key_value, map_utils:get_safe(Map, 123, default)),
    ?assertEqual(tuple_key_value, map_utils:get_safe(Map, {tuple, key}, default)),

    ?assertEqual(default, map_utils:get_safe(Map, missing_atom, default)),
    ?assertEqual(default, map_utils:get_safe(Map, <<"missing_binary">>, default)),
    ?assertEqual(default, map_utils:get_safe(Map, 999, default)).

get_safe_default_types_test() ->
    Map = #{key => value},

    ?assertEqual(nil, map_utils:get_safe(Map, missing, nil)),
    ?assertEqual(0, map_utils:get_safe(Map, missing, 0)),
    ?assertEqual(<<"default">>, map_utils:get_safe(Map, missing, <<"default">>)),
    ?assertEqual([], map_utils:get_safe(Map, missing, [])),
    ?assertEqual(#{}, map_utils:get_safe(Map, missing, #{})),
    ?assertEqual({tuple, default}, map_utils:get_safe(Map, missing, {tuple, default})).

get_nested_basic_test() ->
    Map = #{
        level1 => #{
            level2 => #{
                level3 => deep_value
            },
            other => other_value
        },
        simple => simple_value
    },

    ?assertEqual(#{level3 => deep_value}, map_utils:get_nested(Map, [level1, level2], default)),
    ?assertEqual(
        #{other => other_value, level2 => #{level3 => deep_value}},
        map_utils:get_nested(Map, [level1], default)
    ),

    ?assertEqual(default, map_utils:get_nested(Map, [level1, level2, level3], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [level1, other], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [simple], default)),

    ?assertEqual(Map, map_utils:get_nested(Map, [], default)),

    ?assertEqual(default, map_utils:get_nested(Map, [level1, missing], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [missing, level2], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [level1, level2, missing], default)).

get_nested_deep_nesting_test() ->
    DeepMap = #{
        l1 => #{
            l2 => #{
                l3 => #{
                    l4 => #{
                        l5 => final_value,
                        other5 => value5
                    },
                    other4 => value4
                },
                other3 => value3
            }
        }
    },

    Level4Map = map_utils:get_nested(DeepMap, [l1, l2, l3, l4], default),
    ?assert(is_map(Level4Map)),
    assert_map_field(Level4Map, l5, final_value),
    assert_map_field(Level4Map, other5, value5),

    Level3Map = map_utils:get_nested(DeepMap, [l1, l2, l3], default),
    ?assert(is_map(Level3Map)),
    assert_map_field(Level3Map, other4, value4),

    Level2Map = map_utils:get_nested(DeepMap, [l1, l2], default),
    ?assert(is_map(Level2Map)),
    assert_map_field(Level2Map, other3, value3),

    ?assertEqual(default, map_utils:get_nested(DeepMap, [l1, l2, l3, l4, l5], default)),
    ?assertEqual(default, map_utils:get_nested(DeepMap, [l1, l2, l3, l4, other5], default)),
    ?assertEqual(default, map_utils:get_nested(DeepMap, [l1, l2, l3, other4], default)),
    ?assertEqual(default, map_utils:get_nested(DeepMap, [l1, l2, other3], default)),

    ?assertEqual(default, map_utils:get_nested(DeepMap, [l1, l2, l3, l4, l5, extra], default)),

    ?assertEqual(default, map_utils:get_nested(DeepMap, [l1, l2, missing, l4, l5], default)),
    ?assertEqual(default, map_utils:get_nested(DeepMap, [missing, l2, l3, l4, l5], default)).

get_nested_partial_paths_test() ->
    Map = #{
        user => #{
            name => <<"Alice">>,
            age => 30,
            address => #{
                city => <<"New York">>,
                zip => 10001
            }
        },
        count => 42,
        tags => [tag1, tag2, tag3]
    },

    UserMap = map_utils:get_nested(Map, [user], default),
    ?assert(is_map(UserMap)),
    assert_map_field(UserMap, name, <<"Alice">>),

    AddressMap = map_utils:get_nested(Map, [user, address], default),
    ?assert(is_map(AddressMap)),
    assert_map_field(AddressMap, city, <<"New York">>),

    ?assertEqual(default, map_utils:get_nested(Map, [count], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [user, name], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [user, age], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [tags], default)),

    ?assertEqual(default, map_utils:get_nested(Map, [count, extra], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [count, deep, path], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [user, name, extra], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [tags, extra], default)),
    ?assertEqual(default, map_utils:get_nested(Map, [user, age, extra, path], default)).

assert_map_field(Map, Key, Expected) when is_map(Map) ->
    ?assertEqual(Expected, maps:get(Key, Map)).

get_nested_edge_cases_test() ->
    Map = #{key => #{nested => value}},

    ?assertEqual(default, map_utils:get_nested(not_a_map, [], default)),
    ?assertEqual(default, map_utils:get_nested([], [], default)),
    ?assertEqual(default, map_utils:get_nested(123, [], default)),

    ?assertEqual(default, map_utils:get_nested(not_a_map, [key], default)),
    ?assertEqual(default, map_utils:get_nested([], [key], default)),
    ?assertEqual(default, map_utils:get_nested(123, [key, nested], default)),
    ?assertEqual(default, map_utils:get_safe(undefined, key, default)),

    ?assertEqual(#{nested => value}, map_utils:get_nested(Map, [key], default)),

    ?assertEqual(default, map_utils:get_nested(Map, [key, nested], default)),

    BinaryMap = #{<<"key">> => #{<<"nested">> => <<"value">>}},
    ?assertEqual(
        #{<<"nested">> => <<"value">>},
        map_utils:get_nested(BinaryMap, [<<"key">>], default)
    ),
    ?assertEqual(default, map_utils:get_nested(BinaryMap, [<<"key">>, <<"nested">>], default)).

get_integer_basic_test() ->
    Map = #{user_id => <<"42">>, <<"count">> => 10},
    ?assertEqual(42, map_utils:get_integer(Map, user_id, undefined)),
    ?assertEqual(10, map_utils:get_integer(Map, <<"count">>, 0)),
    ?assertEqual(99, map_utils:get_integer(Map, missing, 99)).

get_integer_invalid_input_test() ->
    ?assertEqual(7, map_utils:get_integer(undefined, count, 7)),
    ?assertEqual(undefined, map_utils:get_integer(#{}, user_id, undefined)),
    ?assertEqual(undefined, map_utils:get_integer(#{user_id => <<"abc">>}, user_id, undefined)).

get_binary_basic_test() ->
    Map = #{<<"name">> => <<"fluxer">>, tag => atom},
    ?assertEqual(<<"fluxer">>, map_utils:get_binary(Map, <<"name">>, <<"default">>)),
    ?assertEqual(<<"atom">>, map_utils:get_binary(Map, tag, <<"default">>)).

get_binary_invalid_input_test() ->
    ?assertEqual(<<"default">>, map_utils:get_binary(not_a_map, <<"id">>, <<"default">>)),
    ?assertEqual(undefined, map_utils:get_binary(#{}, <<"missing">>, undefined)),
    ?assertEqual(<<"default">>, map_utils:get_binary(#{num => 123}, <<"num">>, <<"default">>)).

ensure_map_test() ->
    Map = #{key => value, nested => #{inner => data}},
    ?assertEqual(Map, map_utils:ensure_map(Map)),

    ?assertEqual(#{}, map_utils:ensure_map(#{})).

ensure_map_all_input_types_test() ->
    ?assertEqual(#{}, map_utils:ensure_map(not_a_map)),
    ?assertEqual(#{}, map_utils:ensure_map([])),
    ?assertEqual(#{}, map_utils:ensure_map([1, 2, 3])),
    ?assertEqual(#{}, map_utils:ensure_map(123)),
    ?assertEqual(#{}, map_utils:ensure_map(123.456)),
    ?assertEqual(#{}, map_utils:ensure_map(<<"binary">>)),
    ?assertEqual(#{}, map_utils:ensure_map("string")),
    ?assertEqual(#{}, map_utils:ensure_map(undefined)),
    ?assertEqual(#{}, map_utils:ensure_map(atom)),
    ?assertEqual(#{}, map_utils:ensure_map(true)),
    ?assertEqual(#{}, map_utils:ensure_map(false)),
    ?assertEqual(#{}, map_utils:ensure_map({tuple, value})),
    ?assertEqual(#{}, map_utils:ensure_map(self())),
    ?assertEqual(#{}, map_utils:ensure_map(make_ref())),
    ?assertEqual(#{}, map_utils:ensure_map(fun() -> ok end)).

ensure_list_test() ->
    List = [1, 2, 3],
    ?assertEqual(List, map_utils:ensure_list(List)),

    ComplexList = [#{a => 1}, {tuple}, <<"binary">>, atom],
    ?assertEqual(ComplexList, map_utils:ensure_list(ComplexList)),

    ?assertEqual([], map_utils:ensure_list([])).

ensure_list_all_input_types_test() ->
    ?assertEqual([], map_utils:ensure_list(not_a_list)),
    ?assertEqual([], map_utils:ensure_list(#{})),
    ?assertEqual([], map_utils:ensure_list(#{key => value})),
    ?assertEqual([], map_utils:ensure_list(123)),
    ?assertEqual([], map_utils:ensure_list(123.456)),
    ?assertEqual([], map_utils:ensure_list(<<"binary">>)),
    ?assertEqual("string", map_utils:ensure_list("string")),
    ?assert(is_list(map_utils:ensure_list("string"))),
    ?assertEqual([], map_utils:ensure_list(undefined)),
    ?assertEqual([], map_utils:ensure_list(atom)),
    ?assertEqual([], map_utils:ensure_list(true)),
    ?assertEqual([], map_utils:ensure_list(false)),
    ?assertEqual([], map_utils:ensure_list({tuple, value})),
    ?assertEqual([], map_utils:ensure_list(self())),
    ?assertEqual([], map_utils:ensure_list(make_ref())),
    ?assertEqual([], map_utils:ensure_list(fun() -> ok end)).
