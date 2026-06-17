%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_connected_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

list_id_channel_id_parses_binary_test() ->
    ?assertEqual(500, guild_member_list_connected:list_id_channel_id(<<"500">>)).

list_id_channel_id_invalid_value_test() ->
    ?assertEqual(undefined, guild_member_list_connected:list_id_channel_id(<<"abc">>)).

list_id_channel_id_non_binary_test() ->
    ?assertEqual(undefined, guild_member_list_connected:list_id_channel_id(invalid_list_id())),
    ?assertEqual(
        undefined, guild_member_list_connected:list_id_channel_id(undefined_list_id())
    ).

list_id_channel_id_zero_test() ->
    ?assertEqual(undefined, guild_member_list_connected:list_id_channel_id(<<"0">>)).

list_id_channel_id_empty_binary_test() ->
    ?assertEqual(undefined, guild_member_list_connected:list_id_channel_id(<<>>)).

filter_default_list_skips_invalid_user_ids_test() ->
    Valid = #{<<"user">> => #{<<"id">> => <<"1">>}},
    Members = [
        Valid,
        #{<<"user">> => #{<<"id">> => <<"0">>}},
        #{<<"user">> => #{}}
    ],
    ?assertEqual(
        [Valid], guild_member_list_connected:filter_members_for_list(<<"0">>, Members, #{})
    ).

session_can_view_channel_uses_cached_visibility_test() ->
    SessionData = #{user_id => 12, viewable_channels => #{500 => true}},
    State = #{data => #{<<"members">> => #{}}},
    ?assertEqual(
        true, guild_member_list_connected:session_can_view_channel(SessionData, 500, State)
    ).

session_can_view_channel_rejects_when_cache_misses_and_user_missing_test() ->
    SessionData = #{user_id => 99, viewable_channels => #{}},
    State = #{data => #{<<"members">> => #{}}},
    ?assertEqual(
        false, guild_member_list_connected:session_can_view_channel(SessionData, 500, State)
    ).

session_can_view_channel_non_integer_channel_test() ->
    SessionData = #{user_id => 1, viewable_channels => #{}},
    State = #{data => #{<<"members">> => #{}}},
    ?assertEqual(
        false,
        guild_member_list_connected:session_can_view_channel(
            SessionData, invalid_channel_id(), State
        )
    ).

session_can_view_channel_zero_channel_test() ->
    SessionData = #{user_id => 1, viewable_channels => #{}},
    State = #{data => #{<<"members">> => #{}}},
    ?assertEqual(
        false, guild_member_list_connected:session_can_view_channel(SessionData, 0, State)
    ).

session_can_view_channel_negative_channel_test() ->
    SessionData = #{user_id => 1, viewable_channels => #{}},
    State = #{data => #{<<"members">> => #{}}},
    ?assertEqual(
        false, guild_member_list_connected:session_can_view_channel(SessionData, -5, State)
    ).

session_can_view_channel_no_user_id_test() ->
    SessionData = #{},
    State = #{data => #{<<"members">> => #{}}},
    ?assertEqual(
        false, guild_member_list_connected:session_can_view_channel(SessionData, 500, State)
    ).

session_can_view_channel_no_viewable_channels_map_test() ->
    SessionData = #{user_id => 1},
    State = #{data => #{<<"members">> => #{}}},
    ?assertEqual(
        false, guild_member_list_connected:session_can_view_channel(SessionData, 500, State)
    ).

default_presence_returns_offline_test() ->
    P = guild_member_list_connected:default_presence(),
    ?assertEqual(<<"offline">>, maps:get(<<"status">>, P)),
    ?assertEqual(false, maps:get(<<"mobile">>, P)),
    ?assertEqual(false, maps:get(<<"afk">>, P)).

invalid_list_id() ->
    eqwalizer:dynamic_cast(123).

undefined_list_id() ->
    eqwalizer:dynamic_cast(undefined).

invalid_channel_id() ->
    eqwalizer:dynamic_cast(not_an_integer).

resolve_presence_missing_user_returns_default_test() ->
    State = #{member_presence => #{1 => #{<<"status">> => <<"online">>}}},
    P = guild_member_list_connected:resolve_presence_for_user(State, 999),
    ?assertEqual(<<"offline">>, maps:get(<<"status">>, P)).

resolve_presence_empty_presence_map_test() ->
    State = #{member_presence => #{}},
    P = guild_member_list_connected:resolve_presence_for_user(State, 1),
    ?assertEqual(<<"offline">>, maps:get(<<"status">>, P)).

resolve_presence_no_presence_key_test() ->
    State = #{},
    P = guild_member_list_connected:resolve_presence_for_user(State, 1),
    ?assertEqual(<<"offline">>, maps:get(<<"status">>, P)).

add_presence_to_member_test() ->
    Member = #{<<"user">> => #{<<"id">> => <<"42">>}},
    State = #{
        member_presence => #{42 => #{<<"status">> => <<"dnd">>}},
        connected_user_ids => sets:from_list([42])
    },
    Result = guild_member_list_connected:add_presence_to_member(Member, State),
    ?assertEqual(#{<<"status">> => <<"dnd">>}, maps:get(<<"presence">>, Result)).

add_presence_to_member_hides_unconnected_presence_test() ->
    Member = #{<<"user">> => #{<<"id">> => <<"42">>}},
    State = #{
        member_presence => #{42 => #{<<"status">> => <<"dnd">>}},
        connected_user_ids => sets:new()
    },
    Result = guild_member_list_connected:add_presence_to_member(Member, State),
    Presence = maps:get(<<"presence">>, Result),
    ?assertEqual(<<"offline">>, maps:get(<<"status">>, Presence)),
    ?assertNot(maps:is_key(<<"custom_status">>, Presence)).

presence_status_changed_online_to_offline_test() ->
    OldState = #{member_presence => #{1 => #{<<"status">> => <<"online">>}}},
    NewState = #{member_presence => #{1 => #{<<"status">> => <<"offline">>}}},
    ?assertEqual(
        true, guild_member_list_connected:presence_status_changed(1, OldState, NewState)
    ).

presence_status_changed_same_status_test() ->
    OldState = #{member_presence => #{1 => #{<<"status">> => <<"online">>}}},
    NewState = #{member_presence => #{1 => #{<<"status">> => <<"online">>}}},
    ?assertEqual(
        false, guild_member_list_connected:presence_status_changed(1, OldState, NewState)
    ).

presence_status_changed_user_not_in_either_test() ->
    OldState = #{member_presence => #{}},
    NewState = #{member_presence => #{}},
    ?assertEqual(
        false, guild_member_list_connected:presence_status_changed(999, OldState, NewState)
    ).

presence_status_changed_user_added_test() ->
    OldState = #{member_presence => #{}},
    NewState = #{member_presence => #{1 => #{<<"status">> => <<"online">>}}},
    ?assertEqual(
        true, guild_member_list_connected:presence_status_changed(1, OldState, NewState)
    ).

partition_members_no_sessions_test() ->
    Members = [
        #{<<"user">> => #{<<"id">> => <<"1">>}},
        #{<<"user">> => #{<<"id">> => <<"2">>}}
    ],
    State = #{
        sessions => #{},
        member_presence => #{
            1 => #{<<"status">> => <<"online">>},
            2 => #{<<"status">> => <<"online">>}
        }
    },
    {Online, Offline} = guild_member_list_connected:partition_members_by_online(Members, State),
    ?assertEqual(0, length(Online)),
    ?assertEqual(2, length(Offline)).

partition_members_invisible_is_offline_test() ->
    Members = [#{<<"user">> => #{<<"id">> => <<"1">>}}],
    State = #{
        sessions => #{<<"s1">> => #{user_id => 1}},
        member_presence => #{1 => #{<<"status">> => <<"invisible">>}}
    },
    {Online, Offline} = guild_member_list_connected:partition_members_by_online(Members, State),
    ?assertEqual(0, length(Online)),
    ?assertEqual(1, length(Offline)).

partition_members_empty_list_test() ->
    State = #{sessions => #{}, member_presence => #{}},
    {Online, Offline} = guild_member_list_connected:partition_members_by_online([], State),
    ?assertEqual([], Online),
    ?assertEqual([], Offline).

connected_session_user_ids_ignores_invalid_test() ->
    State = #{
        sessions => #{
            <<"s1">> => #{user_id => 10},
            <<"s2">> => #{user_id => 0},
            <<"s3">> => #{user_id => -1},
            <<"s4">> => #{},
            <<"s5">> => #{user_id => undefined}
        }
    },
    Ids = guild_member_list_connected:connected_session_user_ids(State),
    ?assertEqual(true, sets:is_element(10, Ids)),
    ?assertEqual(false, sets:is_element(0, Ids)),
    ?assertEqual(false, sets:is_element(-1, Ids)),
    ?assertEqual(1, sets:size(Ids)).

member_in_list_found_test() ->
    Members = [
        #{<<"user">> => #{<<"id">> => <<"1">>}},
        #{<<"user">> => #{<<"id">> => <<"2">>}}
    ],
    ?assertEqual(true, guild_member_list_connected:member_in_list(1, Members)).

member_in_list_not_found_test() ->
    Members = [#{<<"user">> => #{<<"id">> => <<"1">>}}],
    ?assertEqual(false, guild_member_list_connected:member_in_list(999, Members)).

member_in_list_empty_test() ->
    ?assertEqual(false, guild_member_list_connected:member_in_list(1, [])).
