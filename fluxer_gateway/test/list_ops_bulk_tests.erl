%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(list_ops_bulk_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

make_item_with_id(Id) ->
    #{<<"id">> => Id, <<"data">> => <<"test">>}.

default_items() ->
    [
        make_item_with_id(<<"1">>),
        make_item_with_id(<<"2">>)
    ].

bulk_update_multiple_updates_test() ->
    Items = [
        make_item_with_id(<<"1">>),
        make_item_with_id(<<"2">>),
        make_item_with_id(<<"3">>),
        make_item_with_id(<<"4">>)
    ],
    Updates = [
        #{<<"id">> => <<"2">>, <<"data">> => <<"updated_2">>},
        #{<<"id">> => <<"4">>, <<"data">> => <<"updated_4">>}
    ],
    Result = list_ops:bulk_update(Items, Updates),

    ?assertEqual(4, length(Result)),
    ?assertEqual(make_item_with_id(<<"1">>), lists:nth(1, Result)),
    ?assertEqual(#{<<"id">> => <<"2">>, <<"data">> => <<"updated_2">>}, lists:nth(2, Result)),
    ?assertEqual(make_item_with_id(<<"3">>), lists:nth(3, Result)),
    ?assertEqual(#{<<"id">> => <<"4">>, <<"data">> => <<"updated_4">>}, lists:nth(4, Result)).

bulk_update_partial_updates_test() ->
    Items = [
        make_item_with_id(<<"1">>),
        make_item_with_id(<<"2">>),
        make_item_with_id(<<"3">>)
    ],
    Updates = [
        #{<<"id">> => <<"2">>, <<"data">> => <<"updated">>}
    ],
    Result = list_ops:bulk_update(Items, Updates),

    ?assertEqual(3, length(Result)),
    ?assertEqual(make_item_with_id(<<"1">>), lists:nth(1, Result)),
    ?assertEqual(#{<<"id">> => <<"2">>, <<"data">> => <<"updated">>}, lists:nth(2, Result)),
    ?assertEqual(make_item_with_id(<<"3">>), lists:nth(3, Result)).

bulk_update_no_matches_test() ->
    Items = default_items(),
    Updates = [
        #{<<"id">> => <<"99">>, <<"data">> => <<"new">>},
        #{<<"id">> => <<"98">>, <<"data">> => <<"new2">>}
    ],
    Result = list_ops:bulk_update(Items, Updates),

    ?assertEqual(Items, Result).

bulk_update_empty_lists_test() ->
    ?assertEqual([], list_ops:bulk_update([], [])),
    ?assertEqual([], list_ops:bulk_update([], [make_item_with_id(<<"1">>)])),

    Items = [make_item_with_id(<<"1">>)],
    ?assertEqual(Items, list_ops:bulk_update(Items, [])).

bulk_update_updates_without_id_test() ->
    Items = default_items(),
    Updates = [
        #{<<"name">> => <<"no_id">>},
        #{<<"id">> => <<"2">>, <<"data">> => <<"updated">>}
    ],
    Result = list_ops:bulk_update(Items, Updates),

    ?assertEqual(2, length(Result)),
    ?assertEqual(make_item_with_id(<<"1">>), lists:nth(1, Result)),
    ?assertEqual(#{<<"id">> => <<"2">>, <<"data">> => <<"updated">>}, lists:nth(2, Result)).

bulk_update_mixed_items_list_test() ->
    Items = [
        make_item_with_id(<<"1">>),
        <<"non_map_item">>,
        make_item_with_id(<<"2">>),
        {tuple, item}
    ],
    Updates = [
        #{<<"id">> => <<"2">>, <<"data">> => <<"updated">>}
    ],
    Result = list_ops:bulk_update(Items, Updates),

    ?assertEqual(4, length(Result)),
    ?assertEqual(make_item_with_id(<<"1">>), lists:nth(1, Result)),
    ?assertEqual(<<"non_map_item">>, lists:nth(2, Result)),
    ?assertEqual(#{<<"id">> => <<"2">>, <<"data">> => <<"updated">>}, lists:nth(3, Result)),
    ?assertEqual({tuple, item}, lists:nth(4, Result)).

bulk_update_mixed_updates_list_test() ->
    Items = default_items(),
    Updates = [
        <<"non_map">>,
        #{<<"id">> => <<"1">>, <<"data">> => <<"updated">>},
        {tuple},
        #{<<"no_id">> => <<"field">>}
    ],
    Result = list_ops:bulk_update(Items, Updates),

    ?assertEqual(2, length(Result)),
    ?assertEqual(#{<<"id">> => <<"1">>, <<"data">> => <<"updated">>}, lists:nth(1, Result)),
    ?assertEqual(make_item_with_id(<<"2">>), lists:nth(2, Result)).

bulk_update_duplicate_ids_in_updates_test() ->
    Items = default_items(),
    Updates = [
        #{<<"id">> => <<"1">>, <<"data">> => <<"first_update">>},
        #{<<"id">> => <<"1">>, <<"data">> => <<"second_update">>}
    ],
    Result = list_ops:bulk_update(Items, Updates),

    ?assertEqual(2, length(Result)),
    ?assertEqual(
        #{<<"id">> => <<"1">>, <<"data">> => <<"second_update">>}, lists:nth(1, Result)
    ),
    ?assertEqual(make_item_with_id(<<"2">>), lists:nth(2, Result)).

bulk_update_invalid_input_test() ->
    Items = [make_item_with_id(<<"1">>)],

    ?assertEqual(Items, list_ops:bulk_update(Items, not_a_list)),
    ?assertEqual(Items, list_ops:bulk_update(Items, undefined)),
    ?assertEqual(Items, list_ops:bulk_update(Items, #{})),

    ?assertEqual([], list_ops:bulk_update(not_a_list, [make_item_with_id(<<"1">>)])),
    ?assertEqual([], list_ops:bulk_update(undefined, [])).

bulk_update_item_without_id_preserved_test() ->
    Items = [
        make_item_with_id(<<"1">>),
        #{<<"name">> => <<"no_id_item">>},
        make_item_with_id(<<"2">>)
    ],
    Updates = [
        #{<<"id">> => <<"1">>, <<"data">> => <<"updated">>}
    ],
    Result = list_ops:bulk_update(Items, Updates),

    ?assertEqual(3, length(Result)),
    ?assertEqual(#{<<"id">> => <<"1">>, <<"data">> => <<"updated">>}, lists:nth(1, Result)),
    ?assertEqual(#{<<"name">> => <<"no_id_item">>}, lists:nth(2, Result)),
    ?assertEqual(make_item_with_id(<<"2">>), lists:nth(3, Result)).
