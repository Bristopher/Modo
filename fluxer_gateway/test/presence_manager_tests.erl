%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(presence_manager_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

start_or_lookup_rejects_missing_user_id_test() ->
    ?assertEqual({error, invalid_user_id}, presence_manager:start_or_lookup(#{})).

track_untrack_shard_user_test() ->
    ensure_shard_user_table(),
    ?assertEqual(ok, presence_manager:track_shard_user(0, 42)),
    ?assertEqual(ok, presence_manager:track_shard_user(0, 43)),
    UserIds = presence_manager:get_shard_user_ids(0),
    ?assert(lists:member(42, UserIds)),
    ?assert(lists:member(43, UserIds)),
    ?assertEqual(ok, presence_manager:untrack_shard_user(0, 42)),
    UserIds2 = presence_manager:get_shard_user_ids(0),
    ?assertNot(lists:member(42, UserIds2)),
    ?assert(lists:member(43, UserIds2)),
    cleanup_shard_user_table().

clear_shard_user_ids_test() ->
    ensure_shard_user_table(),
    presence_manager:track_shard_user(1, 100),
    presence_manager:track_shard_user(1, 101),
    presence_manager:track_shard_user(2, 200),
    ?assertEqual(ok, presence_manager:clear_shard_user_ids(1)),
    ?assertEqual([], presence_manager:get_shard_user_ids(1)),
    ?assertEqual([200], presence_manager:get_shard_user_ids(2)),
    cleanup_shard_user_table().

get_shard_user_ids_no_table_test() ->
    cleanup_shard_user_table(),
    ?assertEqual([], presence_manager:get_shard_user_ids(0)).

ensure_shard_user_table() ->
    presence_manager:ensure_shard_user_table().

cleanup_shard_user_table() ->
    presence_manager:delete_shard_user_table().
