%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(list_ops_user_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

make_item_with_user_id(UserId) ->
    #{
        <<"user">> => #{<<"id">> => integer_to_binary(UserId)},
        <<"data">> => <<"member">>
    }.

replace_by_user_id_success_test() ->
    NewItem = #{
        <<"user">> => #{<<"id">> => <<"200">>},
        <<"data">> => <<"updated">>
    },
    assert_replace_by_user_id(
        [
            make_item_with_user_id(100),
            NewItem,
            make_item_with_user_id(300)
        ],
        user_items([100, 200, 300]),
        200,
        NewItem
    ).

replace_by_user_id_no_match_test() ->
    Items = user_items([100, 200]),
    NewItem = make_item_with_user_id(999),
    assert_replace_by_user_id(Items, Items, 999, NewItem).

replace_by_user_id_empty_list_test() ->
    Result = list_ops:replace_by_user_id([], 100, make_item_with_user_id(100)),
    ?assertEqual([], Result).

replace_by_user_id_nested_extraction_test() ->
    Items = [
        #{
            <<"user">> => #{<<"id">> => <<"123">>, <<"name">> => <<"alice">>},
            <<"role">> => <<"admin">>
        },
        #{
            <<"user">> => #{<<"id">> => <<"456">>, <<"name">> => <<"bob">>},
            <<"role">> => <<"user">>
        }
    ],
    NewItem = #{<<"user">> => #{<<"id">> => <<"456">>}, <<"role">> => <<"moderator">>},
    Result = list_ops:replace_by_user_id(Items, 456, NewItem),

    ?assertEqual(2, length(Result)),
    ?assertEqual(lists:nth(1, Items), lists:nth(1, Result)),
    ?assertEqual(NewItem, lists:nth(2, Result)).

replace_by_user_id_mixed_list_test() ->
    Items = [
        make_item_with_user_id(100),
        <<"string_item">>,
        make_item_with_user_id(200),
        {tuple},
        #{<<"other">> => <<"map">>}
    ],
    NewItem = make_item_with_user_id(200),
    Result = list_ops:replace_by_user_id(Items, 200, NewItem),

    ?assertEqual(5, length(Result)),
    ?assertEqual(make_item_with_user_id(100), lists:nth(1, Result)),
    ?assertEqual(<<"string_item">>, lists:nth(2, Result)),
    ?assertEqual(NewItem, lists:nth(3, Result)),
    ?assertEqual({tuple}, lists:nth(4, Result)),
    ?assertEqual(#{<<"other">> => <<"map">>}, lists:nth(5, Result)).

replace_by_user_id_invalid_structure_test() ->
    Items = [
        #{<<"user">> => <<"not_a_map">>, <<"data">> => <<"x">>},
        #{<<"no_user_key">> => <<"y">>},
        make_item_with_user_id(100)
    ],
    NewItem = make_item_with_user_id(100),
    Result = list_ops:replace_by_user_id(Items, 100, NewItem),

    ?assertEqual(3, length(Result)),
    ?assertEqual(lists:nth(1, Items), lists:nth(1, Result)),
    ?assertEqual(lists:nth(2, Items), lists:nth(2, Result)),
    ?assertEqual(NewItem, lists:nth(3, Result)).

replace_by_user_id_invalid_input_test() ->
    ?assertEqual([], list_ops:replace_by_user_id(not_a_list, 100, #{})),
    ?assertEqual([], list_ops:replace_by_user_id(undefined, 100, #{})),
    ?assertEqual(
        [], list_ops:replace_by_user_id([make_item_with_user_id(100)], invalid_user_id(), #{})
    ).

remove_by_user_id_success_test() ->
    assert_remove_by_user_id(
        [
            make_item_with_user_id(100),
            make_item_with_user_id(300)
        ],
        user_items([100, 200, 300]),
        200
    ).

remove_by_user_id_no_match_test() ->
    Items = user_items([100, 200]),
    assert_remove_by_user_id(Items, Items, 999).

remove_by_user_id_multiple_matches_test() ->
    Items = [
        make_item_with_user_id(100),
        #{<<"user">> => #{<<"id">> => <<"200">>}, <<"version">> => 1},
        #{<<"user">> => #{<<"id">> => <<"200">>}, <<"version">> => 2},
        make_item_with_user_id(300)
    ],
    Result = list_ops:remove_by_user_id(Items, 200),

    ?assertEqual(2, length(Result)),
    ?assertEqual(make_item_with_user_id(100), lists:nth(1, Result)),
    ?assertEqual(make_item_with_user_id(300), lists:nth(2, Result)).

remove_by_user_id_empty_list_test() ->
    Result = list_ops:remove_by_user_id([], 100),
    ?assertEqual([], Result).

remove_by_user_id_mixed_list_test() ->
    Items = [
        make_item_with_user_id(100),
        <<"non_map">>,
        make_item_with_user_id(200),
        [list],
        #{<<"invalid">> => <<"structure">>}
    ],
    Result = list_ops:remove_by_user_id(Items, 200),

    ?assertEqual(4, length(Result)),
    ?assertEqual(make_item_with_user_id(100), lists:nth(1, Result)),
    ?assertEqual(<<"non_map">>, lists:nth(2, Result)),
    ?assertEqual([list], lists:nth(3, Result)),
    ?assertEqual(#{<<"invalid">> => <<"structure">>}, lists:nth(4, Result)).

remove_by_user_id_invalid_nested_structure_test() ->
    Items = [
        #{<<"user">> => <<"not_a_map">>},
        #{<<"no_user">> => <<"field">>},
        #{<<"user">> => #{<<"no_id">> => <<"field">>}},
        make_item_with_user_id(100)
    ],
    Result = list_ops:remove_by_user_id(Items, 999),

    ?assertEqual(4, length(Result)),
    ?assertEqual(Items, Result).

remove_by_user_id_invalid_input_test() ->
    ?assertEqual([], list_ops:remove_by_user_id(not_a_list, 100)),
    ?assertEqual([], list_ops:remove_by_user_id(undefined, 100)),
    ?assertEqual(
        [], list_ops:remove_by_user_id([make_item_with_user_id(100)], invalid_user_id())
    ).

invalid_user_id() ->
    eqwalizer:dynamic_cast(<<"not_integer">>).

user_items(UserIds) ->
    [make_item_with_user_id(UserId) || UserId <- UserIds].

assert_replace_by_user_id(Expected, Items, UserId, NewItem) ->
    ?assertEqual(Expected, list_ops:replace_by_user_id(Items, UserId, NewItem)).

assert_remove_by_user_id(Expected, Items, UserId) ->
    ?assertEqual(Expected, list_ops:remove_by_user_id(Items, UserId)).
