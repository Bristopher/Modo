%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_sync_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

slice_items_basic_test() ->
    ?assertEqual(item_list([b, c, d]), guild_member_list_sync:slice_items(item_list5(), 1, 3)).

slice_items_empty_when_start_exceeds_end_test() ->
    ?assertEqual([], guild_member_list_sync:slice_items(item_list3(), 5, 3)).

slice_items_beyond_bounds_test() ->
    ?assertEqual([], guild_member_list_sync:slice_items(item_list3(), 10, 20)).

slice_items_empty_list_test() ->
    ?assertEqual([], guild_member_list_sync:slice_items([], 0, 5)).

slice_items_end_beyond_length_test() ->
    ?assertEqual(item_list3(), guild_member_list_sync:slice_items(item_list3(), 0, 100)).

slice_items_single_element_test() ->
    ?assertEqual(item_list([b]), guild_member_list_sync:slice_items(item_list3(), 1, 1)).

slice_items_zero_length_range_test() ->
    ?assertEqual(item_list([a]), guild_member_list_sync:slice_items(item_list3(), 0, 0)).

slice_items_full_range_test() ->
    ?assertEqual(item_list5(), guild_member_list_sync:slice_items(item_list5(), 0, 4)).

update_subscriptions_adds_session_test() ->
    Subs = #{},
    {NewSubs, _OldRanges, ShouldSync} =
        update_default_subscription(default_ranges(), Subs),
    ListSubs = maps:get(<<"500">>, NewSubs),
    ?assertEqual([{0, 99}], maps:get(<<"s1">>, ListSubs)),
    ?assertEqual(true, ShouldSync).

update_subscriptions_invalid_list_id_skips_test() ->
    Subs = #{},
    ?assertEqual(
        {Subs, [], false},
        guild_member_list_sync:update_subscriptions(<<"s1">>, <<"bad">>, [{0, 99}], Subs)
    ).

update_subscriptions_removes_session_on_empty_ranges_test() ->
    Subs = #{<<"500">> => #{<<"s1">> => [{0, 99}]}},
    {NewSubs, _OldRanges, ShouldSync} = update_default_subscription([], Subs),
    ?assertEqual(#{}, NewSubs),
    ?assertEqual(false, ShouldSync).

update_subscriptions_same_range_twice_no_sync_test() ->
    {Subs1, ShouldSync1} = new_default_subscription(),
    ?assertEqual(true, ShouldSync1),
    {_Subs2, _, ShouldSync2} = update_default_subscription(default_ranges(), Subs1),
    ?assertEqual(false, ShouldSync2).

update_subscriptions_changed_ranges_triggers_sync_test() ->
    {Subs1, _ShouldSync1} = new_default_subscription(),
    {_Subs2, OldRanges, ShouldSync2} = update_default_subscription([{0, 199}], Subs1),
    ?assertEqual([{0, 99}], OldRanges),
    ?assertEqual(true, ShouldSync2).

update_subscriptions_multiple_sessions_overlapping_test() ->
    Subs0 = #{},
    {Subs1, _, _} =
        guild_member_list_sync:update_subscriptions(<<"s1">>, <<"500">>, [{0, 50}], Subs0),
    {Subs2, _, _} =
        guild_member_list_sync:update_subscriptions(<<"s2">>, <<"500">>, [{25, 75}], Subs1),
    ListSubs = maps:get(<<"500">>, Subs2),
    ?assertEqual([{0, 50}], maps:get(<<"s1">>, ListSubs)),
    ?assertEqual([{25, 75}], maps:get(<<"s2">>, ListSubs)).

remove_session_from_subscriptions_removes_all_lists_test() ->
    Subs = #{
        <<"500">> => #{<<"s1">> => [{0, 99}], <<"s2">> => [{0, 50}]},
        <<"600">> => #{<<"s1">> => [{0, 50}]}
    },
    NewSubs = guild_member_list_sync:remove_session_from_subscriptions(<<"s1">>, Subs),
    ?assertEqual(#{<<"500">> => #{<<"s2">> => [{0, 50}]}}, NewSubs).

remove_session_from_subscriptions_removes_empty_lists_test() ->
    Subs = #{<<"500">> => #{<<"s1">> => [{0, 99}]}},
    NewSubs = guild_member_list_sync:remove_session_from_subscriptions(<<"s1">>, Subs),
    ?assertEqual(#{}, NewSubs).

remove_session_nonexistent_session_test() ->
    Subs = #{<<"500">> => #{<<"s1">> => [{0, 99}]}},
    NewSubs = guild_member_list_sync:remove_session_from_subscriptions(<<"nonexistent">>, Subs),
    ?assertEqual(Subs, NewSubs).

remove_session_empty_subscriptions_test() ->
    NewSubs = guild_member_list_sync:remove_session_from_subscriptions(<<"s1">>, #{}),
    ?assertEqual(#{}, NewSubs).

is_subset_of_ranges_empty_inner_test() ->
    ?assertEqual(true, guild_member_list_sync:is_subset_of_ranges([], [{0, 99}])).

is_subset_of_ranges_empty_outer_test() ->
    ?assertEqual(false, guild_member_list_sync:is_subset_of_ranges([{0, 99}], [])).

is_subset_of_ranges_exact_match_test() ->
    ?assertEqual(true, guild_member_list_sync:is_subset_of_ranges([{0, 99}], [{0, 99}])).

is_subset_of_ranges_true_subset_test() ->
    ?assertEqual(true, guild_member_list_sync:is_subset_of_ranges([{10, 50}], [{0, 99}])).

is_subset_of_ranges_not_subset_test() ->
    ?assertEqual(false, guild_member_list_sync:is_subset_of_ranges([{0, 150}], [{0, 99}])).

is_subset_of_ranges_partial_overlap_not_subset_test() ->
    ?assertEqual(false, guild_member_list_sync:is_subset_of_ranges([{50, 150}], [{0, 99}])).

is_subset_of_ranges_multiple_inner_covered_test() ->
    ?assertEqual(
        true, guild_member_list_sync:is_subset_of_ranges([{10, 20}, {30, 40}], [{0, 99}])
    ).

is_subset_of_ranges_multiple_outer_test() ->
    ?assertEqual(
        true,
        guild_member_list_sync:is_subset_of_ranges([{10, 20}, {110, 120}], [{0, 50}, {100, 150}])
    ).

is_subset_of_ranges_multiple_outer_not_covered_test() ->
    ?assertEqual(
        false,
        guild_member_list_sync:is_subset_of_ranges([{10, 20}, {60, 80}], [{0, 50}, {100, 150}])
    ).

compute_range_delta_no_overlap_test() ->
    ?assertEqual(
        [{100, 199}], guild_member_list_sync:compute_range_delta([{100, 199}], [{0, 50}])
    ).

compute_range_delta_full_overlap_test() ->
    ?assertEqual([], guild_member_list_sync:compute_range_delta([{10, 50}], [{0, 99}])).

compute_range_delta_partial_overlap_right_test() ->
    ?assertEqual(
        [{100, 199}], guild_member_list_sync:compute_range_delta([{0, 199}], [{0, 99}])
    ).

compute_range_delta_partial_overlap_left_test() ->
    ?assertEqual(
        [{0, 49}], guild_member_list_sync:compute_range_delta([{0, 150}], [{50, 200}])
    ).

compute_range_delta_middle_gap_test() ->
    ?assertEqual(
        [{51, 99}],
        guild_member_list_sync:compute_range_delta([{0, 150}], [{0, 50}, {100, 200}])
    ).

compute_range_delta_empty_old_test() ->
    ?assertEqual([{0, 99}], guild_member_list_sync:compute_range_delta([{0, 99}], [])).

compute_range_delta_empty_new_test() ->
    ?assertEqual([], guild_member_list_sync:compute_range_delta([], [{0, 99}])).

item_list3() ->
    item_list([a, b, c]).

item_list5() ->
    item_list([a, b, c, d, e]).

item_list(Names) ->
    [#{<<"name">> => atom_to_binary(Name, utf8)} || Name <- Names].

default_ranges() ->
    [{0, 99}].

new_default_subscription() ->
    {Subs, _OldRanges, ShouldSync} = update_default_subscription(default_ranges(), #{}),
    {Subs, ShouldSync}.

update_default_subscription(Ranges, Subs) ->
    guild_member_list_sync:update_subscriptions(<<"s1">>, <<"500">>, Ranges, Subs).
