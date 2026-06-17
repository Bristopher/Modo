%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_request_members_presence_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

presence_visible_online_test() ->
    ?assertEqual(
        true, guild_request_members_presence:presence_visible(#{<<"status">> => <<"online">>})
    ).

presence_visible_idle_test() ->
    ?assertEqual(
        true, guild_request_members_presence:presence_visible(#{<<"status">> => <<"idle">>})
    ).

presence_visible_dnd_test() ->
    ?assertEqual(
        true, guild_request_members_presence:presence_visible(#{<<"status">> => <<"dnd">>})
    ).

presence_visible_offline_test() ->
    ?assertEqual(
        false, guild_request_members_presence:presence_visible(#{<<"status">> => <<"offline">>})
    ).

presence_visible_invisible_test() ->
    ?assertEqual(
        false,
        guild_request_members_presence:presence_visible(#{<<"status">> => <<"invisible">>})
    ).

presence_visible_missing_status_test() ->
    ?assertEqual(false, guild_request_members_presence:presence_visible(#{})).

presence_user_id_valid_test() ->
    ?assertEqual(
        42,
        guild_request_members_presence:presence_user_id(
            #{<<"user">> => #{<<"id">> => <<"42">>}}
        )
    ).

presence_user_id_rejects_malformed_snowflake_test() ->
    ?assertEqual(
        undefined,
        guild_request_members_presence:presence_user_id(
            #{<<"user">> => #{<<"id">> => <<"042">>}}
        )
    ).
