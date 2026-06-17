%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_common_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

get_member_user_id_extracts_id_test() ->
    Member = #{<<"user">> => #{<<"id">> => <<"42">>}},
    ?assertEqual(42, guild_member_list_common:get_member_user_id(Member)).

get_member_user_id_keeps_integer_id_test() ->
    Member = #{<<"user">> => #{<<"id">> => 42}},
    ?assertEqual(42, guild_member_list_common:get_member_user_id(Member)).

get_member_user_id_returns_undefined_for_missing_user_test() ->
    ?assertEqual(undefined, guild_member_list_common:get_member_user_id(#{})).

get_member_user_id_no_id_in_user_test() ->
    Member = #{<<"user">> => #{<<"username">> => <<"noone">>}},
    ?assertEqual(undefined, guild_member_list_common:get_member_user_id(Member)).

get_member_user_id_zero_is_invalid_test() ->
    Member = #{<<"user">> => #{<<"id">> => <<"0">>}},
    ?assertEqual(undefined, guild_member_list_common:get_member_user_id(Member)).

get_member_display_name_prefers_nick_test() ->
    Member = #{
        <<"nick">> => <<"Cool Nick">>,
        <<"user">> => #{<<"global_name">> => <<"Global">>, <<"username">> => <<"user">>}
    },
    ?assertEqual(<<"Cool Nick">>, guild_member_list_common:get_member_display_name(Member)).

get_member_display_name_falls_back_to_global_name_test() ->
    Member = #{
        <<"user">> => #{<<"global_name">> => <<"Global">>, <<"username">> => <<"user">>}
    },
    ?assertEqual(<<"Global">>, guild_member_list_common:get_member_display_name(Member)).

get_member_display_name_falls_back_to_username_test() ->
    Member = #{<<"user">> => #{<<"username">> => <<"user">>}},
    ?assertEqual(<<"user">>, guild_member_list_common:get_member_display_name(Member)).

get_member_display_name_null_nick_test() ->
    Member = #{<<"nick">> => null, <<"user">> => #{<<"username">> => <<"bob">>}},
    ?assertEqual(<<"bob">>, guild_member_list_common:get_member_display_name(Member)).

get_member_display_name_undefined_nick_test() ->
    Member = #{<<"nick">> => undefined, <<"user">> => #{<<"username">> => <<"charlie">>}},
    ?assertEqual(<<"charlie">>, guild_member_list_common:get_member_display_name(Member)).

get_member_display_name_empty_nick_test() ->
    Member = #{<<"nick">> => <<>>, <<"user">> => #{<<"username">> => <<"dave">>}},
    ?assertEqual(<<"dave">>, guild_member_list_common:get_member_display_name(Member)).

get_member_display_name_no_user_field_test() ->
    ?assertEqual(<<>>, guild_member_list_common:get_member_display_name(#{})).

sort_key_same_display_name_deterministic_by_user_id_test() ->
    MemberA = #{<<"nick">> => <<"Alice">>, <<"user">> => #{<<"id">> => <<"100">>}},
    MemberB = #{<<"nick">> => <<"Alice">>, <<"user">> => #{<<"id">> => <<"200">>}},
    ?assert(
        guild_member_list_common:get_member_sort_key(MemberA) <
            guild_member_list_common:get_member_sort_key(MemberB)
    ).

sort_key_same_name_lower_id_first_test() ->
    MemberA = #{<<"nick">> => <<"zebra">>, <<"user">> => #{<<"id">> => <<"1">>}},
    MemberB = #{<<"nick">> => <<"zebra">>, <<"user">> => #{<<"id">> => <<"999">>}},
    ?assert(
        guild_member_list_common:get_member_sort_key(MemberA) <
            guild_member_list_common:get_member_sort_key(MemberB)
    ).

sort_key_mixed_case_names_test() ->
    MemberLower = #{<<"nick">> => <<"alice">>, <<"user">> => #{<<"id">> => <<"1">>}},
    MemberUpper = #{<<"nick">> => <<"Alice">>, <<"user">> => #{<<"id">> => <<"2">>}},
    {FoldedLower, _} = guild_member_list_common:get_member_sort_key(MemberLower),
    {FoldedUpper, _} = guild_member_list_common:get_member_sort_key(MemberUpper),
    ?assertEqual(FoldedLower, FoldedUpper).

sort_key_unicode_names_test() ->
    MemberUnicode = #{<<"nick">> => <<"Ωmega"/utf8>>, <<"user">> => #{<<"id">> => <<"1">>}},
    MemberAscii = #{<<"nick">> => <<"alpha">>, <<"user">> => #{<<"id">> => <<"2">>}},
    ?assert(is_tuple(guild_member_list_common:get_member_sort_key(MemberUnicode))),
    ?assert(is_tuple(guild_member_list_common:get_member_sort_key(MemberAscii))).

sort_key_empty_display_name_test() ->
    MemberEmpty = #{<<"user">> => #{<<"id">> => <<"1">>}},
    {Folded, Id} = guild_member_list_common:get_member_sort_key(MemberEmpty),
    ?assertEqual(<<>>, Folded),
    ?assertEqual(1, Id).

sort_key_empty_name_sorts_before_nonempty_test() ->
    MemberEmpty = #{<<"user">> => #{<<"id">> => <<"1">>}},
    MemberNamed = #{<<"nick">> => <<"alice">>, <<"user">> => #{<<"id">> => <<"2">>}},
    ?assert(
        guild_member_list_common:get_member_sort_key(MemberEmpty) <
            guild_member_list_common:get_member_sort_key(MemberNamed)
    ).

casefold_binary_mixed_case_test() ->
    ?assertEqual(<<"hello">>, guild_member_list_common:casefold_binary(<<"HeLLo">>)).

casefold_binary_already_lower_test() ->
    ?assertEqual(<<"world">>, guild_member_list_common:casefold_binary(<<"world">>)).

casefold_binary_empty_test() ->
    ?assertEqual(<<>>, guild_member_list_common:casefold_binary(<<>>)).

casefold_binary_integer_test() ->
    ?assertEqual(<<"42">>, guild_member_list_common:casefold_binary(42)).

casefold_binary_undefined_test() ->
    ?assertEqual(<<>>, guild_member_list_common:casefold_binary(undefined)).

normalize_name_binary_test() ->
    ?assertEqual(<<"hello">>, guild_member_list_common:normalize_name(<<"hello">>)).

normalize_name_integer_test() ->
    ?assertEqual(<<"42">>, guild_member_list_common:normalize_name(42)).

normalize_name_null_test() ->
    ?assertEqual(<<>>, guild_member_list_common:normalize_name(null)).

normalize_name_undefined_test() ->
    ?assertEqual(<<>>, guild_member_list_common:normalize_name(undefined)).

normalize_name_list_test() ->
    ?assertEqual(<<"hello">>, guild_member_list_common:normalize_name("hello")).

normalize_name_other_test() ->
    ?assertEqual(<<>>, guild_member_list_common:normalize_name(#{})).

deep_merge_member_merges_user_test() ->
    Current = #{
        <<"user">> => #{<<"id">> => <<"1">>, <<"username">> => <<"alice">>},
        <<"nick">> => <<"a">>
    },
    Update = #{
        <<"user">> => #{<<"id">> => <<"1">>, <<"avatar">> => <<"new">>},
        <<"nick">> => <<"b">>
    },
    Result = guild_member_list_common:deep_merge_member(Current, Update),
    User = maps:get(<<"user">>, Result),
    ?assertEqual(1, maps:get(<<"id">>, User)),
    ?assertEqual(<<"alice">>, maps:get(<<"username">>, User)),
    ?assertEqual(<<"new">>, maps:get(<<"avatar">>, User)),
    ?assertEqual(<<"b">>, maps:get(<<"nick">>, Result)).

deep_merge_member_update_without_user_field_test() ->
    Current = #{
        <<"user">> => #{<<"id">> => <<"1">>, <<"username">> => <<"alice">>},
        <<"nick">> => <<"a">>
    },
    Update = #{<<"nick">> => <<"b">>},
    Result = guild_member_list_common:deep_merge_member(Current, Update),
    User = maps:get(<<"user">>, Result),
    ?assertEqual(<<"alice">>, maps:get(<<"username">>, User)),
    ?assertEqual(<<"b">>, maps:get(<<"nick">>, Result)).

deep_merge_member_null_user_in_update_test() ->
    Current = #{<<"user">> => #{<<"id">> => <<"1">>, <<"username">> => <<"alice">>}},
    Update = #{<<"user">> => null, <<"nick">> => <<"new">>},
    Result = guild_member_list_common:deep_merge_member(Current, Update),
    ?assertEqual(null, maps:get(<<"user">>, Result)),
    ?assertEqual(<<"new">>, maps:get(<<"nick">>, Result)).

deep_merge_member_new_roles_test() ->
    Current = #{<<"user">> => #{<<"id">> => <<"1">>}, <<"roles">> => [<<"100">>]},
    Update = #{<<"roles">> => [<<"200">>, <<"300">>]},
    Result = guild_member_list_common:deep_merge_member(Current, Update),
    assert_roles([200, 300], Result).

deep_merge_member_removing_all_roles_test() ->
    Current = #{<<"user">> => #{<<"id">> => <<"1">>}, <<"roles">> => [<<"100">>, <<"200">>]},
    Update = #{<<"roles">> => []},
    Result = guild_member_list_common:deep_merge_member(Current, Update),
    ?assertEqual([], maps:get(<<"roles">>, Result)).

deep_merge_member_non_map_user_in_update_test() ->
    Current = #{<<"user">> => #{<<"id">> => <<"1">>}},
    Update = #{<<"user">> => <<"invalid">>},
    Result = guild_member_list_common:deep_merge_member(Current, Update),
    ?assertEqual(<<"invalid">>, maps:get(<<"user">>, Result)).

upsert_new_member_into_empty_state_test() ->
    Data = guild_data_index:normalize_data(#{
        <<"members">> => [],
        <<"roles">> => [],
        <<"channels">> => []
    }),
    State = #{data => Data},
    MemberUpdate = #{<<"user">> => #{<<"id">> => <<"42">>}, <<"nick">> => <<"new">>},
    {CurrentMember, UpdatedMember, NewState} =
        guild_member_list_common:upsert_member_in_state(42, MemberUpdate, State),
    ?assertEqual(undefined, CurrentMember),
    ?assertEqual(<<"new">>, maps:get(<<"nick">>, UpdatedMember)),
    NewData = maps:get(data, NewState),
    NewMembers = guild_data_index:member_map(NewData),
    ?assertEqual(true, maps:is_key(42, NewMembers)).

upsert_existing_member_with_changed_roles_test() ->
    ExistingMember = #{<<"user">> => #{<<"id">> => <<"42">>}, <<"roles">> => [<<"100">>]},
    Data = guild_data_index:normalize_data(#{
        <<"members">> => [ExistingMember],
        <<"roles">> => [],
        <<"channels">> => []
    }),
    State = #{data => Data},
    MemberUpdate = #{<<"roles">> => [<<"200">>, <<"300">>]},
    {CurrentMember, UpdatedMember, _NewState} =
        guild_member_list_common:upsert_member_in_state(42, MemberUpdate, State),
    #{<<"roles">> := CurrentRoles} = CurrentMember,
    ?assertEqual([100], CurrentRoles),
    assert_roles([200, 300], UpdatedMember).

assert_roles(ExpectedRoles, Member) ->
    ?assertEqual(ExpectedRoles, maps:get(<<"roles">>, Member)).
