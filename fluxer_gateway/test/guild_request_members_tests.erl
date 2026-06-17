%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_request_members_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

parse_request_valid_test() ->
    Data = #{
        <<"guild_id">> => <<"123456789">>,
        <<"query">> => <<"test">>,
        <<"limit">> => 10,
        <<"presences">> => true,
        <<"nonce">> => <<"abc123">>
    },
    {ok, Request} = guild_request_members:parse_request(Data),
    ?assertEqual([123456789], maps:get(guild_ids, Request)),
    ?assertEqual(<<"test">>, maps:get(query, Request)),
    ?assertEqual(10, maps:get(limit, Request)),
    ?assertEqual(true, maps:get(presences, Request)),
    ?assertEqual(<<"abc123">>, maps:get(nonce, Request)).

parse_request_with_user_ids_test() ->
    Data = #{
        <<"guild_id">> => <<"123">>,
        <<"user_ids">> => [<<"1">>, <<"2">>, <<"3">>]
    },
    {ok, Request} = guild_request_members:parse_request(Data),
    ?assertEqual([123], maps:get(guild_ids, Request)),
    ?assertEqual([1, 2, 3], maps:get(user_ids, Request)).

parse_request_invalid_guild_id_test() ->
    Data = #{<<"guild_id">> => <<"invalid">>},
    {error, invalid_guild_id} = guild_request_members:parse_request(Data).

parse_request_with_guild_ids_array_test() ->
    Data = #{
        <<"guild_ids">> => [<<"3">>, <<"1">>, <<"3">>, <<"2">>],
        <<"query">> => <<"test">>
    },
    {ok, Request} = guild_request_members:parse_request(Data),
    ?assertEqual([3, 1, 2], maps:get(guild_ids, Request)),
    ?assertEqual(<<"test">>, maps:get(query, Request)).

parse_request_with_guild_ids_invalid_entry_test() ->
    Data = #{
        <<"guild_ids">> => [<<"123">>, <<"not_a_guild_id">>]
    },
    ?assertEqual({error, invalid_guild_id}, guild_request_members:parse_request(Data)).

parse_request_with_multiple_guild_ids_drops_nonce_test() ->
    Data = #{
        <<"guild_ids">> => [<<"123">>, <<"456">>],
        <<"nonce">> => <<"abc123">>
    },
    {ok, Request} = guild_request_members:parse_request(Data),
    ?assertEqual([123, 456], maps:get(guild_ids, Request)),
    ?assertEqual(null, maps:get(nonce, Request)).

normalize_nonce_test() ->
    ?assertEqual(<<"abc">>, guild_request_members:normalize_nonce(<<"abc">>)),
    ?assertEqual(
        null,
        guild_request_members:normalize_nonce(<<"this_nonce_is_way_too_long_to_be_valid">>)
    ),
    ?assertEqual(null, guild_request_members:normalize_nonce(undefined)).

validate_user_ids_too_many_test() ->
    UserIds = lists:seq(1, 101),
    ?assertEqual({error, too_many_user_ids}, guild_request_members:validate_user_ids(UserIds)).

validate_user_ids_exactly_max_test() ->
    UserIds = lists:seq(1, 100),
    {ok, Parsed} = guild_request_members:validate_user_ids(UserIds),
    ?assertEqual(100, length(Parsed)).

validate_user_ids_non_list_test() ->
    {ok, []} = guild_request_members:validate_user_ids(not_a_list).

validate_user_ids_filters_invalid_test() ->
    {ok, Parsed} = guild_request_members:validate_user_ids([<<"1">>, <<"invalid">>, 3, -5, 0]),
    ?assertEqual([1, 3], Parsed).

validate_user_ids_empty_test() ->
    {ok, []} = guild_request_members:validate_user_ids([]).

parse_user_id_integer_test() ->
    ?assertEqual({ok, 42}, guild_request_members:parse_user_id(42)).

parse_user_id_binary_test() ->
    ?assertEqual({ok, 123}, guild_request_members:parse_user_id(<<"123">>)).

parse_user_id_zero_test() ->
    ?assertEqual(error, guild_request_members:parse_user_id(0)).

parse_user_id_negative_test() ->
    ?assertEqual(error, guild_request_members:parse_user_id(-1)).

parse_user_id_invalid_binary_test() ->
    ?assertEqual(error, guild_request_members:parse_user_id(<<"abc">>)).

parse_user_id_other_type_test() ->
    ?assertEqual(error, guild_request_members:parse_user_id(1.5)).

ensure_binary_binary_test() ->
    ?assertEqual(<<"hello">>, guild_request_members:ensure_binary(<<"hello">>)).

ensure_binary_integer_test() ->
    ?assertEqual(<<>>, guild_request_members:ensure_binary(42)).

ensure_binary_undefined_test() ->
    ?assertEqual(<<>>, guild_request_members:ensure_binary(undefined)).

ensure_limit_valid_test() ->
    ?assertEqual(10, guild_request_members:ensure_limit(10)).

ensure_limit_zero_test() ->
    ?assertEqual(0, guild_request_members:ensure_limit(0)).

ensure_limit_negative_test() ->
    ?assertEqual(0, guild_request_members:ensure_limit(-1)).

ensure_limit_non_integer_test() ->
    ?assertEqual(0, guild_request_members:ensure_limit(<<"10">>)).

ensure_limit_clamped_test() ->
    ?assertEqual(100, guild_request_members:ensure_limit(101)).

validate_guild_id_integer_test() ->
    ?assertEqual({ok, 123}, guild_request_members:validate_guild_id(123)).

validate_guild_id_zero_test() ->
    ?assertEqual({error, invalid_guild_id}, guild_request_members:validate_guild_id(0)).

validate_guild_id_negative_test() ->
    ?assertEqual({error, invalid_guild_id}, guild_request_members:validate_guild_id(-1)).

validate_guild_id_atom_test() ->
    ?assertEqual({error, invalid_guild_id}, guild_request_members:validate_guild_id(undefined)).

normalize_nonce_exactly_max_length_test() ->
    Nonce = list_to_binary(lists:duplicate(32, $a)),
    ?assertEqual(Nonce, guild_request_members:normalize_nonce(Nonce)).

normalize_nonce_one_over_max_test() ->
    Nonce = list_to_binary(lists:duplicate(33, $a)),
    ?assertEqual(null, guild_request_members:normalize_nonce(Nonce)).

normalize_nonce_empty_binary_test() ->
    ?assertEqual(<<>>, guild_request_members:normalize_nonce(<<>>)).

normalize_nonce_integer_test() ->
    ?assertEqual(null, guild_request_members:normalize_nonce(42)).

normalize_nonce_null_atom_test() ->
    ?assertEqual(null, guild_request_members:normalize_nonce(null)).

parse_request_defaults_test() ->
    Data = #{<<"guild_id">> => 12345},
    {ok, Request} = guild_request_members:parse_request(Data),
    ?assertEqual([12345], maps:get(guild_ids, Request)),
    ?assertEqual(<<>>, maps:get(query, Request)),
    ?assertEqual(0, maps:get(limit, Request)),
    ?assertEqual([], maps:get(user_ids, Request)),
    ?assertEqual(false, maps:get(presences, Request)),
    ?assertEqual(null, maps:get(nonce, Request)).

parse_request_non_binary_query_test() ->
    Data = #{<<"guild_id">> => 123, <<"query">> => 42},
    {ok, Request} = guild_request_members:parse_request(Data),
    ?assertEqual(<<>>, maps:get(query, Request)).

parse_request_negative_limit_test() ->
    Data = #{<<"guild_id">> => 123, <<"limit">> => -5},
    {ok, Request} = guild_request_members:parse_request(Data),
    ?assertEqual(0, maps:get(limit, Request)).

parse_request_presences_not_true_test() ->
    Data = #{<<"guild_id">> => 123, <<"presences">> => <<"yes">>},
    {ok, Request} = guild_request_members:parse_request(Data),
    ?assertEqual(false, maps:get(presences, Request)).

parse_request_missing_guild_id_test() ->
    Data = #{<<"query">> => <<"test">>},
    ?assertEqual({error, invalid_guild_id}, guild_request_members:parse_request(Data)).

handle_request_invalid_data_test() ->
    ?assertEqual(
        {error, invalid_request},
        guild_request_members:handle_request(invalid_request(), self(), #{})
    ).

invalid_request() ->
    eqwalizer:dynamic_cast(not_a_map).
