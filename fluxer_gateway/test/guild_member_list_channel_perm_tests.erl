%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_channel_perm_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

session_can_view_channel_members_grants_when_bit_set_test() ->
    Bit = constants:view_channel_members_permission(),
    View = constants:view_channel_permission(),
    GuildId = 700,
    UserId = 800,
    OwnerId = 999,
    Combined = Bit bor View,
    State = perm_state(GuildId, UserId, OwnerId, Combined),
    ?assert(
        guild_member_list_connected:session_can_view_channel_members(
            #{user_id => UserId}, 123, State
        )
    ).

session_can_view_channel_members_denies_when_bit_unset_test() ->
    View = constants:view_channel_permission(),
    GuildId = 701,
    UserId = 801,
    OwnerId = 998,
    State = perm_state(GuildId, UserId, OwnerId, View),
    ?assertNot(
        guild_member_list_connected:session_can_view_channel_members(
            #{user_id => UserId}, 123, State
        )
    ).

session_can_view_channel_members_denies_when_cant_view_channel_test() ->
    Bit = constants:view_channel_members_permission(),
    GuildId = 702,
    UserId = 802,
    OwnerId = 997,
    State = perm_state(GuildId, UserId, OwnerId, Bit),
    ?assertNot(
        guild_member_list_connected:session_can_view_channel_members(
            #{user_id => UserId}, 123, State
        )
    ).

session_can_view_channel_members_invalid_channel_id_test() ->
    ?assertNot(
        guild_member_list_connected:session_can_view_channel_members(
            #{user_id => 1}, invalid_channel_id(), #{}
        )
    ),
    ?assertNot(
        guild_member_list_connected:session_can_view_channel_members(
            #{user_id => 1}, 0, #{}
        )
    ),
    ?assertNot(
        guild_member_list_connected:session_can_view_channel_members(
            #{user_id => 1}, -1, #{}
        )
    ).

session_can_view_channel_members_no_user_id_test() ->
    ?assertNot(guild_member_list_connected:session_can_view_channel_members(#{}, 5, #{})).

perm_state(GuildId, UserId, OwnerId, Perms) ->
    #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => integer_to_binary(OwnerId)},
            <<"roles">> => [
                #{
                    <<"id">> => integer_to_binary(GuildId),
                    <<"permissions">> => integer_to_binary(Perms)
                }
            ],
            <<"members">> => [
                #{
                    <<"user">> => #{<<"id">> => integer_to_binary(UserId)},
                    <<"roles">> => []
                }
            ],
            <<"channels">> => [
                #{<<"id">> => <<"123">>, <<"permission_overwrites">> => []}
            ]
        }
    }.

invalid_channel_id() ->
    eqwalizer:dynamic_cast(not_an_integer).
