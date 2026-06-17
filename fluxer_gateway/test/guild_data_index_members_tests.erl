%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_data_index_members_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

member_map_edge_cases_test() ->
    ?assertEqual(#{}, guild_data_index_members:member_map(invalid_guild_data())),
    ?assertEqual(#{}, guild_data_index_members:member_map(#{<<"members">> => <<"invalid">>})),
    ?assertEqual(
        #{},
        guild_data_index_members:member_map(#{<<"members">> => [#{<<"nick">> => <<"orphan">>}]})
    ).

member_map_duplicate_user_ids_last_wins_test() ->
    Data = #{
        <<"members">> => [
            #{<<"user">> => #{<<"id">> => <<"1">>}, <<"nick">> => <<"first">>},
            #{<<"user">> => #{<<"id">> => <<"1">>}, <<"nick">> => <<"second">>}
        ]
    },
    MemberMap = guild_data_index_members:member_map(Data),
    ?assertEqual(<<"second">>, maps:get(<<"nick">>, maps:get(1, MemberMap))).

member_map_rejects_malformed_snowflake_ids_test() ->
    Data = #{
        <<"members">> => #{
            <<"001">> => #{<<"user">> => #{<<"id">> => <<"001">>}},
            <<"2">> => #{<<"user">> => #{<<"id">> => <<"02">>}},
            <<"bad">> => #{<<"user">> => #{<<"id">> => <<"3">>}}
        }
    },
    ?assertEqual(
        #{3 => #{<<"user">> => #{<<"id">> => 3}}}, guild_data_index_members:member_map(Data)
    ).

member_accessors_test() ->
    Data = #{
        <<"members">> => #{
            5 => #{<<"user">> => #{<<"id">> => <<"5">>}},
            3 => #{<<"user">> => #{<<"id">> => <<"3">>}}
        }
    },
    ?assertEqual(2, length(guild_data_index_members:member_values(Data))),
    ?assertEqual([], guild_data_index_members:member_list(#{<<"members">> => #{}})),
    ?assertEqual([3, 5], lists:sort(guild_data_index_members:member_ids(Data))),
    ?assertEqual(undefined, guild_data_index_members:get_member(invalid_user_id(), Data)),
    ?assertEqual(undefined, guild_data_index_members:get_member(999, #{<<"members">> => #{}})).

member_list_sorted_by_user_id_test() ->
    Data = #{
        <<"members">> => #{
            9 => #{<<"user">> => #{<<"id">> => <<"9">>}},
            3 => #{<<"user">> => #{<<"id">> => <<"3">>}},
            7 => #{<<"user">> => #{<<"id">> => <<"7">>}}
        }
    },
    Ids = [member_id(M) || M <- guild_data_index_members:member_list(Data)],
    ?assertEqual([<<"3">>, <<"7">>, <<"9">>], Ids).

member_list_uses_cache_consistently_test() ->
    Base = guild_data_index_members:put_member_map(
        #{
            5 => #{<<"user">> => #{<<"id">> => <<"5">>}},
            1 => #{<<"user">> => #{<<"id">> => <<"1">>}}
        },
        #{<<"members">> => #{}}
    ),
    ?assert(maps:is_key(members_sorted_ids, Base)),
    ?assertEqual([1, 5], [member_id(M) || M <- guild_data_index_members:member_list(Base)]),
    Added = guild_data_index_members:put_member(
        #{<<"user">> => #{<<"id">> => <<"3">>}}, Base
    ),
    ?assertNot(maps:is_key(members_sorted_ids, Added)),
    ?assertEqual([1, 3, 5], [member_id(M) || M <- guild_data_index_members:member_list(Added)]),
    Removed = guild_data_index_members:remove_member(1, Added),
    ?assertEqual([3, 5], [member_id(M) || M <- guild_data_index_members:member_list(Removed)]).

member_id(Member) ->
    maps:get(<<"id">>, maps:get(<<"user">>, Member)).

put_member_test() ->
    Data = #{<<"members">> => #{}},
    ?assertEqual(
        Data, guild_data_index_members:put_member(#{<<"nick">> => <<"orphan">>}, Data)
    ),
    ?assertEqual(Data, guild_data_index_members:put_member(invalid_member(), Data)),
    ?assertEqual(
        not_a_map,
        guild_data_index_members:put_member(
            #{<<"user">> => #{<<"id">> => <<"1">>}}, invalid_guild_data()
        )
    ),
    Updated = guild_data_index_members:put_member(
        #{<<"user">> => #{<<"id">> => <<"42">>}, <<"nick">> => <<"new">>},
        Data
    ),
    #{<<"nick">> := Nick} = guild_data_index_members:get_member(42, Updated),
    ?assertEqual(<<"new">>, Nick).

put_member_map_test() ->
    Data = #{<<"members">> => #{1 => #{<<"user">> => #{<<"id">> => <<"1">>}}}},
    NewMap = #{2 => #{<<"user">> => #{<<"id">> => <<"2">>}}},
    Updated = guild_data_index_members:put_member_map(NewMap, Data),
    ?assertEqual(undefined, guild_data_index_members:get_member(1, Updated)),
    #{} = Members = maps:get(<<"members">>, Updated),
    ?assertMatch(#{2 := _}, Members),
    ?assertEqual(Data, guild_data_index_members:put_member_map(invalid_member_map(), Data)).

put_member_list_test() ->
    Data = #{<<"members">> => #{}},
    Members = [
        #{<<"user">> => #{<<"id">> => <<"1">>}}, #{<<"user">> => #{<<"id">> => <<"2">>}}
    ],
    Updated = guild_data_index_members:put_member_list(Members, Data),
    #{} = UpdatedMembers = maps:get(<<"members">>, Updated),
    ?assertEqual(2, map_size(UpdatedMembers)),
    ?assertEqual(Data, guild_data_index_members:put_member_list(invalid_member_list(), Data)).

remove_member_test() ->
    Data = #{<<"members">> => #{1 => #{<<"user">> => #{<<"id">> => <<"1">>}}}},
    ?assertEqual(Data, guild_data_index_members:remove_member(invalid_user_id(), Data)),
    #{} =
        UnchangedMembers = maps:get(
            <<"members">>, guild_data_index_members:remove_member(999, Data)
        ),
    ?assertEqual(
        1, map_size(UnchangedMembers)
    ),
    ?assertEqual(
        undefined,
        guild_data_index_members:get_member(1, guild_data_index_members:remove_member(1, Data))
    ).

member_role_index_test() ->
    ?assertEqual(#{}, guild_data_index_members:member_role_index(invalid_guild_data())),
    Data = #{
        <<"members">> => #{
            1 => #{<<"user">> => #{<<"id">> => <<"1">>}, <<"roles">> => [<<"10">>, <<"11">>]},
            2 => #{<<"user">> => #{<<"id">> => <<"2">>}, <<"roles">> => [<<"11">>]}
        }
    },
    Index = guild_data_index_members:member_role_index(Data),
    ?assertEqual(#{1 => true}, maps:get(10, Index)),
    ?assertEqual(#{1 => true, 2 => true}, maps:get(11, Index)),
    ?assertEqual(
        #{},
        guild_data_index_members:member_role_index(#{
            <<"members">> => #{
                1 => #{<<"user">> => #{<<"id">> => <<"1">>}}
            }
        })
    ).

role_index_sync_test() ->
    Data0 = #{
        <<"members">> => #{
            3 => #{<<"user">> => #{<<"id">> => <<"3">>}, <<"roles">> => [<<"20">>]}
        }
    },
    Data1 = guild_data_index_members:put_member(
        #{<<"user">> => #{<<"id">> => <<"3">>}, <<"roles">> => [<<"30">>]}, Data0
    ),
    ?assertEqual(
        undefined, maps:get(20, guild_data_index_members:member_role_index(Data1), undefined)
    ),
    ?assertEqual(#{3 => true}, maps:get(30, guild_data_index_members:member_role_index(Data1))),
    Data2 = guild_data_index_members:remove_member(3, Data1),
    ?assertEqual(
        undefined, maps:get(30, guild_data_index_members:member_role_index(Data2), undefined)
    ).

normalize_member_map_with_binary_keys_test() ->
    Normalized = guild_data_index_members:normalize_member_map(#{
        <<"42">> => #{<<"user">> => #{<<"id">> => <<"42">>}, <<"nick">> => <<"test">>}
    }),
    ?assertMatch(#{42 := _}, Normalized).

get_member_ets_test() ->
    Tab = ets:new(test_members_ets, [set, public, {read_concurrency, true}]),
    Member = #{<<"user">> => #{<<"id">> => 100}, <<"nick">> => <<"ets_user">>},
    ets:insert(Tab, {100, Member}),
    Data = #{members_ets => Tab, <<"members">> => #{}},
    ?assertEqual(Member, guild_data_index_members:get_member_ets(100, Data)),
    ?assertEqual(undefined, guild_data_index_members:get_member_ets(999, Data)),
    ets:delete(Tab).

get_member_ets_falls_back_to_map_when_no_ets_test() ->
    Data = #{<<"members">> => #{42 => #{<<"user">> => #{<<"id">> => 42}}}},
    ?assertEqual(
        #{<<"user">> => #{<<"id">> => 42}}, guild_data_index_members:get_member_ets(42, Data)
    ).

invalid_guild_data() ->
    eqwalizer:dynamic_cast(not_a_map).

invalid_member() ->
    eqwalizer:dynamic_cast(not_a_map).

invalid_member_map() ->
    eqwalizer:dynamic_cast(not_a_map).

invalid_member_list() ->
    eqwalizer:dynamic_cast(not_a_list).

invalid_user_id() ->
    eqwalizer:dynamic_cast(not_an_integer).
