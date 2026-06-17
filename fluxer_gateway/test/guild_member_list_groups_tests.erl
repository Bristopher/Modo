%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_groups_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

hoisted_roles_same_position_sort_by_id_test() ->
    RoleA = #{<<"id">> => <<"200">>, <<"hoist">> => true, <<"position">> => 5},
    RoleB = #{<<"id">> => <<"300">>, <<"hoist">> => true, <<"position">> => 5},
    Sorted = guild_member_list_groups:get_hoisted_roles_sorted([RoleA, RoleB], 100),
    ?assertEqual(2, length(Sorted)).

hoisted_roles_excludes_everyone_role_test() ->
    GuildId = 100,
    EveryoneRole = #{<<"id">> => <<"100">>, <<"hoist">> => true, <<"position">> => 0},
    OtherRole = #{<<"id">> => <<"200">>, <<"hoist">> => true, <<"position">> => 1},
    Sorted = guild_member_list_groups:get_hoisted_roles_sorted(
        [EveryoneRole, OtherRole], GuildId
    ),
    ?assertEqual(1, length(Sorted)),
    ?assertEqual(<<"200">>, maps:get(<<"id">>, hd(Sorted))).

hoisted_roles_missing_hoist_defaults_false_test() ->
    GuildId = 100,
    RoleNoHoist = #{<<"id">> => <<"200">>, <<"position">> => 5},
    Sorted = guild_member_list_groups:get_hoisted_roles_sorted([RoleNoHoist], GuildId),
    ?assertEqual([], Sorted).

hoisted_roles_zero_id_is_skipped_test() ->
    Role = #{<<"id">> => <<"0">>, <<"hoist">> => true, <<"position">> => 5},
    ?assertEqual([], guild_member_list_groups:get_hoisted_roles_sorted([Role], 100)).

hoisted_roles_sorted_by_descending_position_test() ->
    GuildId = 100,
    RoleLow = #{<<"id">> => <<"200">>, <<"hoist">> => true, <<"position">> => 1},
    RoleHigh = #{<<"id">> => <<"300">>, <<"hoist">> => true, <<"position">> => 10},
    Sorted = guild_member_list_groups:get_hoisted_roles_sorted([RoleLow, RoleHigh], GuildId),
    [First, Second] = Sorted,
    ?assertEqual(<<"300">>, maps:get(<<"id">>, First)),
    ?assertEqual(<<"200">>, maps:get(<<"id">>, Second)).

hoisted_roles_uses_hoist_position_over_position_test() ->
    Role = #{
        <<"id">> => <<"200">>,
        <<"hoist">> => true,
        <<"position">> => 3,
        <<"hoist_position">> => 10
    },
    ?assertEqual(10, guild_member_list_groups:get_effective_hoist_position(Role)).

hoisted_roles_hoist_position_null_falls_back_test() ->
    Role = #{<<"id">> => <<"200">>, <<"position">> => 7, <<"hoist_position">> => null},
    ?assertEqual(7, guild_member_list_groups:get_effective_hoist_position(Role)).

hoisted_roles_hoist_position_undefined_falls_back_test() ->
    Role = #{<<"id">> => <<"200">>, <<"position">> => 7, <<"hoist_position">> => undefined},
    ?assertEqual(7, guild_member_list_groups:get_effective_hoist_position(Role)).

hoisted_roles_empty_list_test() ->
    ?assertEqual([], guild_member_list_groups:get_hoisted_roles_sorted([], 100)).

build_role_groups_empty_guild_test() ->
    ?assertEqual([], guild_member_list_groups:build_role_groups([], [])).

build_role_groups_all_members_same_hoisted_role_test() ->
    HoistedRole = #{<<"id">> => <<"200">>, <<"hoist">> => true, <<"position">> => 5},
    Member1 = #{<<"user">> => #{<<"id">> => <<"1">>}, <<"roles">> => [<<"200">>]},
    Member2 = #{<<"user">> => #{<<"id">> => <<"2">>}, <<"roles">> => [<<"200">>]},
    Groups = guild_member_list_groups:build_role_groups([HoistedRole], [Member1, Member2]),
    ?assertEqual(1, length(Groups)),
    ?assertEqual(2, maps:get(<<"count">>, hd(Groups))).

count_ungrouped_online_no_hoisted_roles_test() ->
    Members = [
        #{<<"user">> => #{<<"id">> => <<"1">>}, <<"roles">> => []},
        #{<<"user">> => #{<<"id">> => <<"2">>}, <<"roles">> => []}
    ],
    ?assertEqual(2, guild_member_list_groups:count_ungrouped_online(Members, [])).

count_ungrouped_online_all_hoisted_test() ->
    HoistedRole = #{<<"id">> => <<"200">>, <<"hoist">> => true, <<"position">> => 5},
    Members = [
        #{<<"user">> => #{<<"id">> => <<"1">>}, <<"roles">> => [<<"200">>]},
        #{<<"user">> => #{<<"id">> => <<"2">>}, <<"roles">> => [<<"200">>]}
    ],
    ?assertEqual(0, guild_member_list_groups:count_ungrouped_online(Members, [HoistedRole])).

count_members_with_top_role_no_matching_members_test() ->
    HoistedRole = #{<<"id">> => <<"200">>, <<"hoist">> => true, <<"position">> => 5},
    Members = [
        #{<<"user">> => #{<<"id">> => <<"1">>}, <<"roles">> => [<<"300">>]}
    ],
    ?assertEqual(
        0, guild_member_list_groups:count_members_with_top_role(200, Members, [HoistedRole])
    ).

find_top_hoisted_role_respects_position_test() ->
    MemberRoleIds = [100, 200, 300],
    HoistedRoleIdsSortedByPosition = [300, 200],
    ?assertEqual(
        300,
        guild_member_list_groups:find_top_hoisted_role(
            MemberRoleIds, HoistedRoleIdsSortedByPosition
        )
    ).

find_top_hoisted_role_returns_undefined_when_no_match_test() ->
    MemberRoleIds = [100, 400],
    HoistedRoleIdsSortedByPosition = [300, 200],
    ?assertEqual(
        undefined,
        guild_member_list_groups:find_top_hoisted_role(
            MemberRoleIds, HoistedRoleIdsSortedByPosition
        )
    ).
