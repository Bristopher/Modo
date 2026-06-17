%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_mutual_online).
-typing([eqwalizer]).

-export([compute_count/2]).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").
-endif.

-export_type([user_id/0, guild_state/0]).

-type user_id() :: integer().
-type guild_state() :: map().

-spec compute_count(user_id() | term(), guild_state()) -> non_neg_integer().
compute_count(UserId, State) when is_integer(UserId), UserId > 0 ->
    case viewer_sees_everything(UserId, State) of
        true ->
            guild_member_list:get_online_count(State);
        false ->
            slow_count(UserId, State)
    end;
compute_count(_, _) ->
    0.

-spec viewer_sees_everything(user_id(), guild_state()) -> boolean().
viewer_sees_everything(UserId, State) ->
    Perms = guild_permissions:get_member_permissions(UserId, undefined, State),
    permission_bits:has(Perms, constants:administrator_permission()).

-spec slow_count(user_id(), guild_state()) -> non_neg_integer().
slow_count(UserId, State) ->
    ViewerSet = guild_visibility:viewable_channel_set(UserId, State),
    case sets:is_empty(ViewerSet) of
        true ->
            self_online_count(UserId, State);
        false ->
            count_mutually_visible(UserId, ViewerSet, State)
    end.

-spec self_online_count(user_id(), guild_state()) -> non_neg_integer().
self_online_count(UserId, State) ->
    case is_self_online(UserId, State) of
        true -> 1;
        false -> 0
    end.

-spec count_mutually_visible(user_id(), sets:set(), guild_state()) -> non_neg_integer().
count_mutually_visible(UserId, ViewerSet, State) ->
    Tab = maps:get(member_presence, State),
    ets:foldl(
        fun({OtherUserId, Presence}, Acc) ->
            count_online_member(UserId, ViewerSet, State, OtherUserId, Presence, Acc)
        end,
        0,
        Tab
    ).

-spec count_online_member(
    user_id(), sets:set(), guild_state(), term(), term(), non_neg_integer()
) -> non_neg_integer().
count_online_member(UserId, ViewerSet, State, OtherUserId, Presence, Acc) when
    is_integer(OtherUserId), is_map(Presence), OtherUserId > 0
->
    case is_online(Presence) of
        false -> Acc;
        true when OtherUserId =:= UserId -> Acc + 1;
        true -> count_if_mutually_visible(OtherUserId, ViewerSet, State, Acc)
    end;
count_online_member(_UserId, _ViewerSet, _State, _OtherUserId, _Presence, Acc) ->
    Acc.

-spec count_if_mutually_visible(user_id(), sets:set(), guild_state(), non_neg_integer()) ->
    non_neg_integer().
count_if_mutually_visible(OtherUserId, ViewerSet, State, Acc) ->
    OtherSet = guild_visibility:viewable_channel_set(OtherUserId, State),
    case sets:is_empty(sets:intersection(ViewerSet, OtherSet)) of
        true -> Acc;
        false -> Acc + 1
    end.

-spec is_self_online(user_id(), guild_state()) -> boolean().
is_self_online(UserId, State) ->
    Tab = maps:get(member_presence, State),
    case ets:lookup(Tab, UserId) of
        [{_, P}] -> is_online(P);
        [] -> false
    end.

-spec is_online(term()) -> boolean().
is_online(Presence) when is_map(Presence) ->
    Status = maps:get(<<"status">>, Presence, <<"offline">>),
    Status =/= <<"offline">> andalso Status =/= <<"invisible">>;
is_online(_) ->
    false.

-ifdef(TEST).

view_perm() -> constants:view_channel_permission().
admin_perm() -> constants:administrator_permission().

returns_zero_for_invalid_user_test() ->
    ?assertEqual(0, compute_count(0, #{})),
    ?assertEqual(0, compute_count(undefined, #{})),
    ?assertEqual(0, compute_count(-5, #{})).

admin_viewer_returns_count_without_member_list_store_test() ->
    GuildId = 1,
    AdminRoleId = 9001,
    State = admin_viewer_state(GuildId, AdminRoleId),
    Result = compute_count(100, State),
    ?assert(is_integer(Result) andalso Result >= 0).

admin_viewer_state(GuildId, AdminRoleId) ->
    Roles = [
        #{<<"id">> => integer_to_binary(GuildId), <<"permissions">> => <<"0">>},
        #{
            <<"id">> => integer_to_binary(AdminRoleId),
            <<"permissions">> => integer_to_binary(admin_perm())
        }
    ],
    Members = #{
        100 => #{
            <<"user">> => #{<<"id">> => <<"100">>},
            <<"roles">> => [integer_to_binary(AdminRoleId)]
        }
    },
    #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => Roles,
            <<"members">> => Members,
            <<"channels">> => [
                #{<<"id">> => <<"5">>, <<"type">> => 0, <<"permission_overwrites">> => []}
            ]
        },
        member_presence => make_presence_tab(#{100 => #{<<"status">> => <<"online">>}}),
        sessions => #{}
    }.

slow_path_counts_only_mutually_visible_members_test() ->
    GuildId = 1,
    BotRoleId = 5000,
    State = mutual_visibility_state(GuildId, BotRoleId),
    Result = compute_count(10, State),
    ?assertEqual(2, Result).

mutual_visibility_state(GuildId, BotRoleId) ->
    #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => mutual_visibility_roles(GuildId, BotRoleId),
            <<"members">> => mutual_visibility_members(BotRoleId),
            <<"channels">> => mutual_visibility_channels(BotRoleId)
        },
        member_presence => mutual_visibility_presence(),
        sessions => #{}
    }.

mutual_visibility_roles(GuildId, BotRoleId) ->
    [
        #{<<"id">> => integer_to_binary(GuildId), <<"permissions">> => <<"0">>},
        #{<<"id">> => integer_to_binary(BotRoleId), <<"permissions">> => <<"0">>}
    ].

mutual_visibility_members(BotRoleId) ->
    #{
        10 => #{<<"user">> => #{<<"id">> => <<"10">>}, <<"roles">> => []},
        20 => #{
            <<"user">> => #{<<"id">> => <<"20">>},
            <<"roles">> => [integer_to_binary(BotRoleId)]
        },
        30 => #{<<"user">> => #{<<"id">> => <<"30">>}, <<"roles">> => []}
    }.

mutual_visibility_channels(BotRoleId) ->
    [channel_with_user_view_overwrites(), channel_with_role_view(BotRoleId)].

channel_with_role_view(BotRoleId) ->
    #{
        <<"id">> => <<"101">>,
        <<"type">> => 0,
        <<"permission_overwrites">> => [
            #{
                <<"id">> => integer_to_binary(BotRoleId),
                <<"type">> => 0,
                <<"allow">> => integer_to_binary(view_perm()),
                <<"deny">> => <<"0">>
            }
        ]
    }.

channel_with_user_view_overwrites() ->
    #{
        <<"id">> => <<"100">>,
        <<"type">> => 0,
        <<"permission_overwrites">> => [
            user_view_overwrite(<<"10">>),
            user_view_overwrite(<<"30">>)
        ]
    }.

user_view_overwrite(UserId) ->
    #{
        <<"id">> => UserId,
        <<"type">> => 1,
        <<"allow">> => integer_to_binary(view_perm()),
        <<"deny">> => <<"0">>
    }.

mutual_visibility_presence() ->
    make_presence_tab(#{
        10 => #{<<"status">> => <<"online">>},
        20 => #{<<"status">> => <<"online">>},
        30 => #{<<"status">> => <<"online">>},
        40 => #{<<"status">> => <<"online">>}
    }).

slow_path_returns_self_when_viewer_sees_no_channels_test() ->
    GuildId = 1,
    Roles = [#{<<"id">> => integer_to_binary(GuildId), <<"permissions">> => <<"0">>}],
    Members = #{
        10 => #{<<"user">> => #{<<"id">> => <<"10">>}, <<"roles">> => []}
    },
    State = #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => Roles,
            <<"members">> => Members,
            <<"channels">> => []
        },
        member_presence => make_presence_tab(#{10 => #{<<"status">> => <<"online">>}}),
        sessions => #{}
    },
    ?assertEqual(1, compute_count(10, State)).

slow_path_returns_zero_when_viewer_offline_and_no_channels_test() ->
    GuildId = 1,
    State = #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => [
                #{<<"id">> => integer_to_binary(GuildId), <<"permissions">> => <<"0">>}
            ],
            <<"members">> => #{
                10 => #{<<"user">> => #{<<"id">> => <<"10">>}, <<"roles">> => []}
            },
            <<"channels">> => []
        },
        member_presence => make_presence_tab(#{10 => #{<<"status">> => <<"offline">>}}),
        sessions => #{}
    },
    ?assertEqual(0, compute_count(10, State)).

make_presence_tab(Map) ->
    Tab = ets:new(test_member_presence, [set, public]),
    maps:foreach(fun(K, V) -> ets:insert(Tab, {K, V}) end, Map),
    Tab.

-endif.
