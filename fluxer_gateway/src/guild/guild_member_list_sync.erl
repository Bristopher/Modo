%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_sync).
-typing([eqwalizer]).

-export([
    build_member_list_items/3,
    slice_items/3,
    update_subscriptions/4,
    remove_session_from_subscriptions/2,
    is_subset_of_ranges/2,
    compute_range_delta/2
]).

-type range() :: {non_neg_integer(), non_neg_integer()}.
-type session_id() :: binary().
-type list_id() :: binary().
-type list_item() :: map().

-export_type([range/0, session_id/0, list_id/0, list_item/0]).

-define(UNGROUPED_ITEM_CAP, 250).

-spec build_member_list_items([map()], [map()], map()) -> [list_item()].
build_member_list_items(Groups, Members, State) ->
    Data = maps:get(data, State, #{}),
    Roles = map_utils:ensure_list(maps:get(<<"roles">>, Data, [])),
    GuildId = guild_id(State),
    HoistedRoles = guild_member_list_groups:get_hoisted_roles_sorted(Roles, GuildId),
    HoistedIdxMap = guild_member_list_groups:hoisted_idx_map(HoistedRoles),
    {OnlineMembers, OfflineMembers} =
        guild_member_list_connected:partition_members_by_online(Members, State),
    OnlineTopRoles = guild_member_list_groups:top_role_map(OnlineMembers, HoistedIdxMap),
    UngroupedOnlineCount = count_ungrouped(OnlineTopRoles),
    SkipUngroupedItems = UngroupedOnlineCount > ?UNGROUPED_ITEM_CAP,
    SkipOfflineItems = length(OfflineMembers) > ?UNGROUPED_ITEM_CAP,
    Ctx = #{
        online_members => OnlineMembers,
        offline_members => OfflineMembers,
        online_top_roles => OnlineTopRoles,
        skip_ungrouped => SkipUngroupedItems,
        skip_offline => SkipOfflineItems,
        state => State
    },
    lists:flatmap(fun(Group) -> expand_group(Group, Ctx) end, Groups).

-spec count_ungrouped(#{integer() => integer() | undefined}) -> non_neg_integer().
count_ungrouped(OnlineTopRoles) ->
    maps:fold(
        fun
            (_U, undefined, Acc) -> Acc + 1;
            (_U, _TopId, Acc) -> Acc
        end,
        0,
        OnlineTopRoles
    ).

-spec expand_group(map(), map()) -> [list_item()].
expand_group(Group, Ctx) ->
    GroupId = maps:get(<<"id">>, Group),
    GroupHeader = #{<<"group">> => Group},
    State = maps:get(state, Ctx),
    case GroupId of
        <<"online">> ->
            expand_online_group(GroupHeader, Ctx, State);
        <<"offline">> ->
            expand_offline_group(GroupHeader, Ctx, State);
        RoleIdBin ->
            expand_role_group(GroupHeader, RoleIdBin, Ctx, State)
    end.

-spec expand_online_group(map(), map(), map()) -> [list_item()].
expand_online_group(GroupHeader, #{skip_ungrouped := true}, _State) ->
    [GroupHeader];
expand_online_group(GroupHeader, Ctx, State) ->
    OnlineMembers = maps:get(online_members, Ctx),
    OnlineTopRoles = maps:get(online_top_roles, Ctx),
    UngroupedOnline = [
        M
     || M <- OnlineMembers,
        maps:get(
            guild_member_list_common:get_member_user_id(M), OnlineTopRoles, undefined
        ) =:= undefined
    ],
    [GroupHeader | member_items(UngroupedOnline, State)].

-spec expand_offline_group(map(), map(), map()) -> [list_item()].
expand_offline_group(GroupHeader, #{skip_offline := true}, _State) ->
    [GroupHeader];
expand_offline_group(GroupHeader, Ctx, State) ->
    OfflineMembers = maps:get(offline_members, Ctx),
    [GroupHeader | member_items(OfflineMembers, State)].

-spec expand_role_group(map(), binary(), map(), map()) -> [list_item()].
expand_role_group(GroupHeader, RoleIdBin, Ctx, State) ->
    case snowflake_id:parse_maybe(RoleIdBin) of
        RoleId when is_integer(RoleId), RoleId > 0 ->
            OnlineMembers = maps:get(online_members, Ctx),
            OnlineTopRoles = maps:get(online_top_roles, Ctx),
            RoleMembers = [
                M
             || M <- OnlineMembers,
                maps:get(
                    guild_member_list_common:get_member_user_id(M), OnlineTopRoles, undefined
                ) =:= RoleId
            ],
            [GroupHeader | member_items(RoleMembers, State)];
        _ ->
            [GroupHeader]
    end.

-spec member_items([map()], map()) -> [list_item()].
member_items(Members, State) ->
    [
        #{<<"member">> => guild_member_list_connected:add_presence_to_member(M, State)}
     || M <- Members, is_integer(guild_member_list_common:get_member_user_id(M))
    ].

-spec slice_items([list_item()], non_neg_integer(), non_neg_integer()) -> [list_item()].
slice_items(Items, Start, End) ->
    SafeEnd = min(End, length(Items) - 1),
    case Start > SafeEnd of
        true -> [];
        false -> lists:sublist(Items, Start + 1, SafeEnd - Start + 1)
    end.

-spec update_subscriptions(session_id(), list_id(), [range()], map()) ->
    {map(), [range()], boolean()}.
update_subscriptions(SessionId, ListId, NormalizedRanges, Subscriptions) ->
    case valid_list_id(ListId) of
        true ->
            ListSubs0 = maps:get(ListId, Subscriptions, #{}),
            OldRanges = maps:get(SessionId, ListSubs0, []),
            NewSubscriptions = apply_subscription_change(
                SessionId, ListId, NormalizedRanges, ListSubs0, Subscriptions
            ),
            ShouldSync = NormalizedRanges =/= [] andalso NormalizedRanges =/= OldRanges,
            {NewSubscriptions, OldRanges, ShouldSync};
        false ->
            {Subscriptions, [], false}
    end.

-spec apply_subscription_change(session_id(), list_id(), [range()], map(), map()) -> map().
apply_subscription_change(SessionId, ListId, [], ListSubs0, Subscriptions) ->
    Trimmed = maps:remove(SessionId, ListSubs0),
    case map_size(Trimmed) of
        0 -> maps:remove(ListId, Subscriptions);
        _ -> Subscriptions#{ListId => Trimmed}
    end;
apply_subscription_change(SessionId, ListId, NormalizedRanges, ListSubs0, Subscriptions) ->
    Updated = ListSubs0#{SessionId => NormalizedRanges},
    Subscriptions#{ListId => Updated}.

-spec is_subset_of_ranges([range()], [range()]) -> boolean().
is_subset_of_ranges([], _Outer) ->
    true;
is_subset_of_ranges(_Inner, []) ->
    false;
is_subset_of_ranges(Inner, Outer) ->
    lists:all(fun(Range) -> range_is_subset(Range, Outer) end, Inner).

-spec range_is_subset(range(), [range()]) -> boolean().
range_is_subset({InStart, InEnd}, Outer) ->
    lists:any(
        fun({OutStart, OutEnd}) ->
            OutStart =< InStart andalso OutEnd >= InEnd
        end,
        Outer
    ).

-spec compute_range_delta([range()], [range()]) -> [range()].
compute_range_delta(NewRanges, OldRanges) ->
    Subtracted = lists:foldl(
        fun subtract_range_from_list/2,
        NewRanges,
        OldRanges
    ),
    guild_member_list:normalize_ranges(Subtracted).

-spec subtract_range_from_list(range(), [range()]) -> [range()].
subtract_range_from_list(_SubRange, []) ->
    [];
subtract_range_from_list({SubStart, SubEnd}, Ranges) ->
    lists:flatmap(
        fun({RStart, REnd}) ->
            subtract_one_range(RStart, REnd, SubStart, SubEnd)
        end,
        Ranges
    ).

-spec subtract_one_range(
    non_neg_integer(),
    non_neg_integer(),
    non_neg_integer(),
    non_neg_integer()
) -> [range()].
subtract_one_range(RStart, REnd, SubStart, SubEnd) when
    REnd < SubStart; RStart > SubEnd
->
    [{RStart, REnd}];
subtract_one_range(RStart, REnd, SubStart, SubEnd) ->
    Left =
        case RStart < SubStart of
            true -> [{RStart, SubStart - 1}];
            false -> []
        end,
    Right =
        case REnd > SubEnd of
            true -> [{SubEnd + 1, REnd}];
            false -> []
        end,
    Left ++ Right.

-spec remove_session_from_subscriptions(session_id(), map()) -> map().
remove_session_from_subscriptions(SessionId, Subscriptions) ->
    maps:fold(
        fun(ListId, ListSubs, Acc) ->
            remove_session_from_list(SessionId, ListId, ListSubs, Acc)
        end,
        #{},
        Subscriptions
    ).

-spec remove_session_from_list(session_id(), list_id(), map(), map()) -> map().
remove_session_from_list(SessionId, ListId, ListSubs, Acc) ->
    Trimmed = maps:remove(SessionId, ListSubs),
    case map_size(Trimmed) of
        0 -> Acc;
        _ -> Acc#{ListId => Trimmed}
    end.

-spec guild_id(map()) -> integer() | undefined.
guild_id(State) ->
    case snowflake_id:parse_maybe(maps:get(id, State, undefined)) of
        GuildId when is_integer(GuildId), GuildId > 0 -> GuildId;
        _ -> undefined
    end.

-spec valid_list_id(list_id()) -> boolean().
valid_list_id(<<"0">>) ->
    true;
valid_list_id(ListId) when is_binary(ListId) ->
    case snowflake_id:parse_maybe(ListId) of
        Id when is_integer(Id), Id > 0 -> true;
        _ -> false
    end;
valid_list_id(_) ->
    false.
