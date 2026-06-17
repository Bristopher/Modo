%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(list_ops_extract_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

extract_user_id_valid_structure_test() ->
    Item = #{<<"user">> => #{<<"id">> => <<"12345">>}},
    ?assertEqual(12345, list_ops:extract_user_id(Item)).

extract_user_id_missing_user_test() ->
    Item = #{<<"other">> => <<"field">>},
    ?assertEqual(undefined, list_ops:extract_user_id(Item)).

extract_user_id_missing_id_test() ->
    Item = #{<<"user">> => #{<<"name">> => <<"alice">>}},
    ?assertEqual(undefined, list_ops:extract_user_id(Item)).

extract_user_id_non_map_test() ->
    ?assertEqual(undefined, list_ops:extract_user_id(<<"string">>)),
    ?assertEqual(undefined, list_ops:extract_user_id([list])),
    ?assertEqual(undefined, list_ops:extract_user_id({tuple})),
    ?assertEqual(undefined, list_ops:extract_user_id(undefined)),
    ?assertEqual(undefined, list_ops:extract_user_id(123)).

extract_user_id_user_not_map_test() ->
    Item = #{<<"user">> => <<"not_a_map">>},
    ?assertEqual(undefined, list_ops:extract_user_id(Item)).

extract_user_id_nested_structure_test() ->
    Item = #{
        <<"user">> => #{
            <<"id">> => <<"999">>,
            <<"name">> => <<"bob">>,
            <<"extra">> => #{<<"nested">> => <<"data">>}
        },
        <<"role">> => <<"admin">>
    },
    ?assertEqual(999, list_ops:extract_user_id(Item)).

extract_user_id_empty_id_test() ->
    Item = #{<<"user">> => #{<<"id">> => <<>>}},
    ?assertEqual(undefined, list_ops:extract_user_id(Item)).

extract_user_id_empty_user_map_test() ->
    Item = #{<<"user">> => #{}},
    ?assertEqual(undefined, list_ops:extract_user_id(Item)).

extract_user_id_invalid_id_format_test() ->
    Item = #{<<"user">> => #{<<"id">> => <<"not_a_number">>}},
    ?assertEqual(undefined, list_ops:extract_user_id(Item)).

extract_user_id_rejects_zero_and_leading_zero_test() ->
    ?assertEqual(undefined, list_ops:extract_user_id(#{<<"user">> => #{<<"id">> => <<"0">>}})),
    ?assertEqual(
        undefined, list_ops:extract_user_id(#{<<"user">> => #{<<"id">> => <<"001">>}})
    ).
