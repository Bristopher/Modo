%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(list_ops_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

make_item_with_id(Id) ->
    #{<<"id">> => Id, <<"data">> => <<"test">>}.

replace_by_id_success_test() ->
    NewItem = #{<<"id">> => <<"2">>, <<"data">> => <<"updated">>},
    assert_replace_default_id_two(
        [
            make_item_with_id(<<"1">>),
            NewItem,
            make_item_with_id(<<"3">>)
        ],
        NewItem
    ).

replace_by_id_no_match_test() ->
    NewItem = #{<<"id">> => <<"99">>, <<"data">> => <<"new">>},
    assert_replace_by_id_no_change(<<"99">>, NewItem).

replace_by_id_empty_list_test() ->
    Result = list_ops:replace_by_id([], <<"1">>, #{<<"id">> => <<"1">>}),
    ?assertEqual([], Result).

replace_by_id_mixed_list_test() ->
    Items = [
        make_item_with_id(<<"1">>),
        <<"non_map_item">>,
        make_item_with_id(<<"2">>),
        {tuple, item},
        make_item_with_id(<<"3">>)
    ],
    NewItem = #{<<"id">> => <<"2">>, <<"data">> => <<"replaced">>},
    Result = list_ops:replace_by_id(Items, <<"2">>, NewItem),

    ?assertEqual(5, length(Result)),
    ?assertEqual(make_item_with_id(<<"1">>), lists:nth(1, Result)),
    ?assertEqual(<<"non_map_item">>, lists:nth(2, Result)),
    ?assertEqual(NewItem, lists:nth(3, Result)),
    ?assertEqual({tuple, item}, lists:nth(4, Result)),
    ?assertEqual(make_item_with_id(<<"3">>), lists:nth(5, Result)).

replace_by_id_integer_id_test() ->
    Items = [
        #{<<"id">> => 1, <<"data">> => <<"a">>},
        #{<<"id">> => 2, <<"data">> => <<"b">>}
    ],
    NewItem = #{<<"id">> => 2, <<"data">> => <<"updated">>},
    Result = list_ops:replace_by_id(Items, 2, NewItem),

    ?assertEqual(2, length(Result)),
    ?assertEqual(#{<<"id">> => 1, <<"data">> => <<"a">>}, lists:nth(1, Result)),
    ?assertEqual(NewItem, lists:nth(2, Result)).

replace_by_id_invalid_input_test() ->
    ?assertEqual([], list_ops:replace_by_id(not_a_list, <<"1">>, #{})),
    ?assertEqual([], list_ops:replace_by_id(#{}, <<"1">>, #{})),
    ?assertEqual([], list_ops:replace_by_id(undefined, <<"1">>, #{})).

replace_by_id_item_without_id_test() ->
    Items = [
        #{<<"id">> => <<"1">>},
        #{<<"name">> => <<"no_id">>},
        #{<<"id">> => <<"2">>}
    ],
    NewItem = #{<<"id">> => <<"2">>, <<"updated">> => true},
    Result = list_ops:replace_by_id(Items, <<"2">>, NewItem),

    ?assertEqual(3, length(Result)),
    ?assertEqual(#{<<"id">> => <<"1">>}, lists:nth(1, Result)),
    ?assertEqual(#{<<"name">> => <<"no_id">>}, lists:nth(2, Result)),
    ?assertEqual(NewItem, lists:nth(3, Result)).

remove_by_id_success_test() ->
    assert_remove_default_id_two(
        [
            make_item_with_id(<<"1">>),
            make_item_with_id(<<"3">>)
        ]
    ).

remove_by_id_no_match_test() ->
    assert_remove_by_id_no_change(<<"99">>).

remove_by_id_multiple_matches_test() ->
    Items = [
        make_item_with_id(<<"1">>),
        #{<<"id">> => <<"2">>, <<"version">> => 1},
        #{<<"id">> => <<"2">>, <<"version">> => 2},
        make_item_with_id(<<"3">>)
    ],
    Result = list_ops:remove_by_id(Items, <<"2">>),

    ?assertEqual(2, length(Result)),
    ?assertEqual(make_item_with_id(<<"1">>), lists:nth(1, Result)),
    ?assertEqual(make_item_with_id(<<"3">>), lists:nth(2, Result)).

remove_by_id_empty_list_test() ->
    Result = list_ops:remove_by_id([], <<"1">>),
    ?assertEqual([], Result).

remove_by_id_mixed_list_test() ->
    Items = [
        make_item_with_id(<<"1">>),
        <<"non_map">>,
        make_item_with_id(<<"2">>),
        [list, item],
        make_item_with_id(<<"3">>)
    ],
    Result = list_ops:remove_by_id(Items, <<"2">>),

    ?assertEqual(4, length(Result)),
    ?assertEqual(make_item_with_id(<<"1">>), lists:nth(1, Result)),
    ?assertEqual(<<"non_map">>, lists:nth(2, Result)),
    ?assertEqual([list, item], lists:nth(3, Result)),
    ?assertEqual(make_item_with_id(<<"3">>), lists:nth(4, Result)).

remove_by_id_invalid_input_test() ->
    ?assertEqual([], list_ops:remove_by_id(not_a_list, <<"1">>)),
    ?assertEqual([], list_ops:remove_by_id(undefined, <<"1">>)),
    ?assertEqual([], list_ops:remove_by_id(123, <<"1">>)).

remove_by_id_all_items_match_test() ->
    Items = [
        make_item_with_id(<<"1">>),
        make_item_with_id(<<"1">>),
        make_item_with_id(<<"1">>)
    ],
    Result = list_ops:remove_by_id(Items, <<"1">>),
    ?assertEqual([], Result).

id_items(Ids) ->
    [make_item_with_id(Id) || Id <- Ids].

default_id_items() ->
    id_items([<<"1">>, <<"2">>]).

default_three_id_items() ->
    id_items([<<"1">>, <<"2">>, <<"3">>]).

assert_replace_by_id(Expected, Items, Id, NewItem) ->
    ?assertEqual(Expected, list_ops:replace_by_id(Items, Id, NewItem)).

assert_replace_default_id_two(Expected, NewItem) ->
    assert_replace_by_id(Expected, default_three_id_items(), <<"2">>, NewItem).

assert_replace_by_id_no_change(Id, NewItem) ->
    Items = default_id_items(),
    assert_replace_by_id(Items, Items, Id, NewItem).

assert_remove_by_id(Expected, Items, Id) ->
    ?assertEqual(Expected, list_ops:remove_by_id(Items, Id)).

assert_remove_default_id_two(Expected) ->
    assert_remove_by_id(Expected, default_three_id_items(), <<"2">>).

assert_remove_by_id_no_change(Id) ->
    Items = default_id_items(),
    assert_remove_by_id(Items, Items, Id).
