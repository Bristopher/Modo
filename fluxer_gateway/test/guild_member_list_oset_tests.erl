%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_oset_tests).

-include_lib("eunit/include/eunit.hrl").

empty_test() ->
    O = guild_member_list_oset:new(),
    ?assertEqual(0, guild_member_list_oset:size(O)),
    ?assertEqual(none, guild_member_list_oset:at(O, 0)),
    ?assertEqual(none, guild_member_list_oset:at(O, -1)),
    ?assertEqual(not_found, guild_member_list_oset:rank(O, key(<<"a">>, 1))),
    ?assertEqual(not_found, guild_member_list_oset:delete(O, key(<<"a">>, 1))),
    ?assertEqual([], guild_member_list_oset:range(O, 0, 10)),
    ?assertEqual([], guild_member_list_oset:to_list(O)),
    guild_member_list_oset:destroy(O).

single_test() ->
    O = guild_member_list_oset:new(),
    ?assertEqual(0, guild_member_list_oset:insert(O, key(<<"b">>, 2))),
    ?assertEqual(1, guild_member_list_oset:size(O)),
    ?assertEqual(0, guild_member_list_oset:rank(O, key(<<"b">>, 2))),
    ?assertEqual(key(<<"b">>, 2), guild_member_list_oset:at(O, 0)),
    ?assertEqual(none, guild_member_list_oset:at(O, 1)),
    ?assertEqual([key(<<"b">>, 2)], guild_member_list_oset:range(O, 0, 10)),
    ?assertEqual(0, guild_member_list_oset:insert(O, key(<<"b">>, 2))),
    ?assertEqual(1, guild_member_list_oset:size(O)),
    ?assertEqual(0, guild_member_list_oset:delete(O, key(<<"b">>, 2))),
    ?assertEqual(0, guild_member_list_oset:size(O)),
    ?assertEqual(not_found, guild_member_list_oset:delete(O, key(<<"b">>, 2))),
    guild_member_list_oset:destroy(O).

ordering_and_index_test() ->
    O = ordering_oset(),
    ?assertEqual(ordering_expected(), guild_member_list_oset:to_list(O)),
    ?assertEqual(3, guild_member_list_oset:rank(O, key(<<"n">>, 6))),
    ?assertEqual(key(<<"m">>, 5), guild_member_list_oset:at(O, 2)),
    ?assertEqual([key(<<"b">>, 2), key(<<"m">>, 5)], guild_member_list_oset:range(O, 1, 2)),
    guild_member_list_oset:destroy(O).

delete_reindexes_test() ->
    O = ordering_oset(),
    ?assertEqual(2, guild_member_list_oset:delete(O, key(<<"m">>, 5))),
    ?assertEqual(
        [key(<<"a">>, 1), key(<<"b">>, 2), key(<<"n">>, 6), key(<<"z">>, 9)],
        guild_member_list_oset:to_list(O)
    ),
    ?assertEqual(2, guild_member_list_oset:rank(O, key(<<"n">>, 6))),
    guild_member_list_oset:destroy(O).

range_edges_test() ->
    O = from_list([key(<<"k">>, N) || N <- lists:seq(1, 10)]),
    ?assertEqual(
        [key(<<"k">>, 1), key(<<"k">>, 2), key(<<"k">>, 3)],
        guild_member_list_oset:range(O, 0, 3)
    ),
    ?assertEqual([key(<<"k">>, 9), key(<<"k">>, 10)], guild_member_list_oset:range(O, 8, 100)),
    ?assertEqual([], guild_member_list_oset:range(O, 10, 5)),
    ?assertEqual([], guild_member_list_oset:range(O, 100, 5)),
    ?assertEqual([], guild_member_list_oset:range(O, 0, 0)),
    ?assertEqual([key(<<"k">>, 1), key(<<"k">>, 2)], guild_member_list_oset:range(O, -3, 5)),
    guild_member_list_oset:destroy(O).

range_pages_large_requested_count_test() ->
    Keys = [key(<<"k">>, N) || N <- lists:seq(1, 70000)],
    O = from_sorted_oset(Keys),
    try
        Got = guild_member_list_oset:range(O, 0, 1000000),
        ?assertEqual(70000, length(Got)),
        ?assertEqual(Keys, Got),
        ?assertEqual(
            [key(<<"k">>, N) || N <- lists:seq(65530, 65540)],
            guild_member_list_oset:range(O, 65529, 11)
        )
    after
        guild_member_list_oset:destroy(O)
    end.

prop_small_keyspace_test_() ->
    {timeout, 120, fun() -> run_prop({1, 2, 3}, 200, 8000) end}.

prop_large_keyspace_test_() ->
    {timeout, 120, fun() -> run_prop({4, 5, 6}, 20000, 8000) end}.

large_build_test_() ->
    {timeout, 120, fun() ->
        N = 100000,
        O = guild_member_list_oset:new(),
        Keys = [key(<<"u">>, K) || K <- shuffled(lists:seq(1, N), {7, 8, 9})],
        lists:foreach(fun(K) -> guild_member_list_oset:insert(O, K) end, Keys),
        ?assertEqual(N, guild_member_list_oset:size(O)),
        ?assertEqual(key(<<"u">>, 1), guild_member_list_oset:at(O, 0)),
        ?assertEqual(key(<<"u">>, N), guild_member_list_oset:at(O, N - 1)),
        ?assertEqual(N - 1, guild_member_list_oset:rank(O, key(<<"u">>, N))),
        ?assertEqual(49999, guild_member_list_oset:rank(O, key(<<"u">>, 50000))),
        ?assertEqual(
            [key(<<"u">>, 50001), key(<<"u">>, 50002)],
            guild_member_list_oset:range(O, 50000, 2)
        ),
        ?assertEqual(none, guild_member_list_oset:at(O, N)),
        guild_member_list_oset:destroy(O)
    end}.

from_sorted_equivalence_test() ->
    rand:seed(exsss, {31, 32, 33}),
    Keys = [key(<<"k">>, K) || K <- lists:seq(1, 500)],
    OBulk = from_sorted_oset(Keys),
    OIns = from_list(shuffled(Keys, {34, 35, 36})),
    ?assertEqual(Keys, guild_member_list_oset:to_list(OBulk)),
    ?assertEqual(guild_member_list_oset:to_list(OIns), guild_member_list_oset:to_list(OBulk)),
    assert_consistent(OBulk, Keys),
    guild_member_list_oset:destroy(OBulk),
    guild_member_list_oset:destroy(OIns).

from_sorted_then_mutate_test_() ->
    {timeout, 120, fun() ->
        rand:seed(exsss, {41, 42, 43}),
        InitKeys = [key(<<"k">>, K) || K <- lists:seq(1, 1000)],
        O = from_sorted_oset(InitKeys),
        assert_consistent(O, InitKeys),
        Ref = run_ops(O, InitKeys, 2000, 6000),
        assert_consistent(O, Ref),
        guild_member_list_oset:destroy(O)
    end}.

from_sorted_scale_test_() ->
    {timeout, 120, fun() ->
        N = 100000,
        Keys = [key(<<"u">>, K) || K <- lists:seq(1, N)],
        O = from_sorted_oset(Keys),
        ?assertEqual(N, guild_member_list_oset:size(O)),
        ?assertEqual(key(<<"u">>, 1), guild_member_list_oset:at(O, 0)),
        ?assertEqual(key(<<"u">>, N), guild_member_list_oset:at(O, N - 1)),
        ?assertEqual(N - 1, guild_member_list_oset:rank(O, key(<<"u">>, N))),
        ?assertEqual(49999, guild_member_list_oset:rank(O, key(<<"u">>, 50000))),
        ?assertEqual(
            [key(<<"u">>, 50001), key(<<"u">>, 50002)],
            guild_member_list_oset:range(O, 50000, 2)
        ),
        ?assertEqual(50000, guild_member_list_oset:delete(O, key(<<"u">>, 50001))),
        ?assertEqual(N - 1, guild_member_list_oset:size(O)),
        ?assertEqual(50000, guild_member_list_oset:insert(O, key(<<"u">>, 50001))),
        ?assertEqual(N, guild_member_list_oset:size(O)),
        guild_member_list_oset:destroy(O)
    end}.

from_sorted_empty_test() ->
    O = guild_member_list_oset:new(),
    ok = guild_member_list_oset:from_sorted(O, []),
    ?assertEqual(0, guild_member_list_oset:size(O)),
    ?assertEqual(0, guild_member_list_oset:insert(O, key(<<"a">>, 1))),
    ?assertEqual([key(<<"a">>, 1)], guild_member_list_oset:to_list(O)),
    guild_member_list_oset:destroy(O).

run_prop(Seed, KeySpace, Ops) ->
    rand:seed(exsss, Seed),
    O = guild_member_list_oset:new(),
    Ref = run_ops(O, [], KeySpace, Ops),
    assert_consistent(O, Ref),
    guild_member_list_oset:destroy(O).

run_ops(_O, Ref, _KeySpace, 0) ->
    Ref;
run_ops(O, Ref, KeySpace, N) ->
    Key = key(<<"k">>, rand:uniform(KeySpace)),
    Ref1 =
        case rand:uniform(2) of
            1 ->
                Idx = guild_member_list_oset:insert(O, Key),
                {ExpIdx, R} = ref_insert(Ref, Key),
                ?assertEqual({insert, Key, ExpIdx}, {insert, Key, Idx}),
                R;
            2 ->
                Res = guild_member_list_oset:delete(O, Key),
                {ExpRes, R} = ref_delete(Ref, Key),
                ?assertEqual({delete, Key, ExpRes}, {delete, Key, Res}),
                R
        end,
    case N rem 250 of
        0 -> assert_consistent(O, Ref1);
        _ -> ok
    end,
    run_ops(O, Ref1, KeySpace, N - 1).

assert_consistent(O, Ref) ->
    ?assertEqual(length(Ref), guild_member_list_oset:size(O)),
    ?assertEqual(Ref, guild_member_list_oset:to_list(O)),
    lists:foldl(
        fun(Key, I) ->
            ?assertEqual({rank, Key, I}, {rank, Key, guild_member_list_oset:rank(O, Key)}),
            ?assertEqual({at, I, Key}, {at, I, guild_member_list_oset:at(O, I)}),
            I + 1
        end,
        0,
        Ref
    ),
    case Ref of
        [] ->
            ok;
        _ ->
            L = length(Ref),
            Start = rand:uniform(L) - 1,
            Count = rand:uniform(L),
            ?assertEqual(
                lists:sublist(Ref, Start + 1, Count),
                guild_member_list_oset:range(O, Start, Count)
            )
    end.

ref_insert(Ref, Key) ->
    case index_of(Key, Ref, 0) of
        {found, I} -> {I, Ref};
        notfound -> ref_insert_sorted(Ref, Key, 0, [])
    end.

ref_insert_sorted([H | T], Key, I, Acc) when H < Key ->
    ref_insert_sorted(T, Key, I + 1, [H | Acc]);
ref_insert_sorted(Rest, Key, I, Acc) ->
    {I, lists:reverse(Acc) ++ [Key | Rest]}.

ref_delete(Ref, Key) ->
    case index_of(Key, Ref, 0) of
        {found, I} -> {I, lists:delete(Key, Ref)};
        notfound -> {not_found, Ref}
    end.

index_of(_Key, [], _I) -> notfound;
index_of(Key, [Key | _], I) -> {found, I};
index_of(Key, [_ | T], I) -> index_of(Key, T, I + 1).

from_list(Keys) ->
    O = guild_member_list_oset:new(),
    lists:foreach(fun(K) -> guild_member_list_oset:insert(O, K) end, Keys),
    O.

from_sorted_oset(SortedKeys) ->
    O = guild_member_list_oset:new(),
    guild_member_list_oset:from_sorted(O, SortedKeys),
    O.

ordering_oset() ->
    from_list([
        key(<<"m">>, 5),
        key(<<"a">>, 1),
        key(<<"z">>, 9),
        key(<<"n">>, 6),
        key(<<"b">>, 2)
    ]).

ordering_expected() ->
    [
        key(<<"a">>, 1),
        key(<<"b">>, 2),
        key(<<"m">>, 5),
        key(<<"n">>, 6),
        key(<<"z">>, 9)
    ].

shuffled(L, Seed) ->
    rand:seed(exsss, Seed),
    [X || {_, X} <- lists:sort([{rand:uniform(), E} || E <- L])].

key(SortKey, UserId) ->
    {0, SortKey, UserId}.
