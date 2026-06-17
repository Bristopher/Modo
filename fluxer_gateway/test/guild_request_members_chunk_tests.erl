%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_request_members_chunk_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

build_chunk_data_basic_test() ->
    Members = [#{<<"user">> => #{<<"id">> => <<"1">>}}],
    Result = guild_request_members_chunk:build_chunk_data(Members, [], 0, 1, null),
    ?assertEqual(Members, maps:get(<<"members">>, Result)),
    ?assertEqual(0, maps:get(<<"chunk_index">>, Result)),
    ?assertEqual(1, maps:get(<<"chunk_count">>, Result)),
    ?assertNot(maps:is_key(<<"presences">>, Result)),
    ?assertNot(maps:is_key(<<"nonce">>, Result)).

build_chunk_data_with_presences_test() ->
    Members = [#{<<"user">> => #{<<"id">> => <<"1">>}}],
    Presences = [#{<<"user">> => #{<<"id">> => <<"1">>}, <<"status">> => <<"online">>}],
    Result = guild_request_members_chunk:build_chunk_data(Members, Presences, 0, 1, null),
    ?assertEqual(Presences, maps:get(<<"presences">>, Result)),
    ?assertNot(maps:is_key(<<"nonce">>, Result)).

build_chunk_data_with_nonce_test() ->
    Members = [],
    Result = guild_request_members_chunk:build_chunk_data(Members, [], 0, 1, <<"my_nonce">>),
    ?assertEqual(<<"my_nonce">>, maps:get(<<"nonce">>, Result)).

build_chunk_data_with_presences_and_nonce_test() ->
    Members = [#{<<"user">> => #{<<"id">> => <<"1">>}}],
    Presences = [#{<<"user">> => #{<<"id">> => <<"1">>}, <<"status">> => <<"online">>}],
    Result = guild_request_members_chunk:build_chunk_data(
        Members, Presences, 2, 5, <<"nonce1">>
    ),
    ?assertEqual(Presences, maps:get(<<"presences">>, Result)),
    ?assertEqual(<<"nonce1">>, maps:get(<<"nonce">>, Result)),
    ?assertEqual(2, maps:get(<<"chunk_index">>, Result)),
    ?assertEqual(5, maps:get(<<"chunk_count">>, Result)).
