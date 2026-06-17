%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_groups).
-typing([eqwalizer]).

-export([
    get_hoisted_roles_sorted/2,
    get_effective_hoist_position/1,
    build_role_groups/2,
    count_members_with_top_role/3,
    hoisted_idx_map/1,
    member_top_role/2,
    normalize_role_ids/1,
    top_role_map/2,
    find_top_hoisted_role/2,
    count_ungrouped_online/2
]).

-spec get_hoisted_roles_sorted([map()], integer() | undefined) -> [map()].
get_hoisted_roles_sorted(Roles, GuildId) ->
    HoistedRoles = lists:filter(
        fun(Role) ->
            IsHoist = maps:get(<<"hoist">>, Role, false),
            RoleId = role_id(Role),
            IsHoist andalso RoleId =/= undefined andalso RoleId =/= GuildId
        end,
        Roles
    ),
    lists:sort(
        fun(A, B) ->
            PosA = get_effective_hoist_position(A),
            PosB = get_effective_hoist_position(B),
            PosA > PosB
        end,
        HoistedRoles
    ).

-spec get_effective_hoist_position(map()) -> integer().
get_effective_hoist_position(Role) ->
    case maps:get(<<"hoist_position">>, Role, null) of
        null -> maps:get(<<"position">>, Role, 0);
        undefined -> maps:get(<<"position">>, Role, 0);
        HoistPos when is_integer(HoistPos) -> HoistPos;
        _ -> maps:get(<<"position">>, Role, 0)
    end.

-spec build_role_groups([map()], [map()]) -> [map()].
build_role_groups(HoistedRoles, OnlineMembers) ->
    HoistedIdxMap = hoisted_idx_map(HoistedRoles),
    TopRoles = top_role_map(OnlineMembers, HoistedIdxMap),
    Counts = maps:fold(
        fun
            (_UserId, undefined, Acc) ->
                Acc;
            (_UserId, TopRoleId, Acc) ->
                maps:update_with(TopRoleId, fun increment_count/1, 1, Acc)
        end,
        #{},
        TopRoles
    ),
    lists:filtermap(
        fun(Role) -> role_group_item(Role, Counts) end,
        HoistedRoles
    ).

-spec role_group_item(map(), map()) -> {true, map()} | false.
role_group_item(Role, Counts) ->
    case role_id(Role) of
        undefined ->
            false;
        RoleId ->
            Count = maps:get(RoleId, Counts, 0),
            {true, #{<<"id">> => integer_to_binary(RoleId), <<"count">> => Count}}
    end.

-spec increment_count(term()) -> pos_integer().
increment_count(Count) when is_integer(Count), Count >= 0 ->
    Count + 1;
increment_count(_) ->
    1.

-spec count_members_with_top_role(integer(), [map()], [map()]) -> non_neg_integer().
count_members_with_top_role(RoleId, Members, HoistedRoles) ->
    HoistedIdxMap = hoisted_idx_map(HoistedRoles),
    lists:foldl(
        fun(Member, Acc) -> count_top_role_member(RoleId, HoistedIdxMap, Member, Acc) end,
        0,
        Members
    ).

-spec count_top_role_member(
    integer(), #{integer() => non_neg_integer()}, map(), non_neg_integer()
) ->
    non_neg_integer().
count_top_role_member(RoleId, HoistedIdxMap, Member, Acc) ->
    case member_top_role(Member, HoistedIdxMap) of
        RoleId -> Acc + 1;
        _ -> Acc
    end.

-spec hoisted_idx_map([map()]) -> #{integer() => non_neg_integer()}.
hoisted_idx_map(HoistedRoles) ->
    {Map, _} = lists:foldl(
        fun add_hoisted_index/2,
        {#{}, 0},
        HoistedRoles
    ),
    Map.

-spec add_hoisted_index(map(), {#{integer() => non_neg_integer()}, non_neg_integer()}) ->
    {#{integer() => non_neg_integer()}, non_neg_integer()}.
add_hoisted_index(Role, {Acc, Idx}) ->
    case role_id(Role) of
        undefined -> {Acc, Idx};
        RoleId -> {Acc#{RoleId => Idx}, Idx + 1}
    end.

-spec member_top_role(map(), #{integer() => non_neg_integer()}) -> integer() | undefined.
member_top_role(Member, HoistedIdxMap) ->
    MemberRoles = maps:get(<<"roles">>, Member, []),
    Normalized = normalize_role_ids(MemberRoles),
    {BestIdx, BestId} = lists:foldl(
        fun(RoleId, Acc) -> better_top_role(RoleId, HoistedIdxMap, Acc) end,
        {infinity, undefined},
        Normalized
    ),
    case BestIdx of
        infinity -> undefined;
        _ -> BestId
    end.

-spec better_top_role(
    integer(),
    #{integer() => non_neg_integer()},
    {non_neg_integer() | infinity, integer() | undefined}
) ->
    {non_neg_integer() | infinity, integer() | undefined}.
better_top_role(RoleId, HoistedIdxMap, {AccIdx, _AccId} = Acc) ->
    case maps:get(RoleId, HoistedIdxMap, undefined) of
        undefined -> Acc;
        Idx when Idx < AccIdx -> {Idx, RoleId};
        _ -> Acc
    end.

-spec normalize_role_ids(list() | term()) -> [integer()].
normalize_role_ids(Roles) when is_list(Roles) ->
    lists:filtermap(fun role_id_item/1, Roles);
normalize_role_ids(_) ->
    [].

-spec role_id_item(term()) -> {true, integer()} | false.
role_id_item(RoleId) ->
    case snowflake_id:parse_maybe(RoleId) of
        Id when is_integer(Id), Id > 0 -> {true, Id};
        _ -> false
    end.

-spec top_role_map([map()], #{integer() => non_neg_integer()}) ->
    #{integer() => integer() | undefined}.
top_role_map(Members, HoistedIdxMap) ->
    lists:foldl(
        fun(Member, Acc) -> add_member_top_role(Member, HoistedIdxMap, Acc) end,
        #{},
        Members
    ).

-spec add_member_top_role(
    map(),
    #{integer() => non_neg_integer()},
    #{integer() => integer() | undefined}
) ->
    #{integer() => integer() | undefined}.
add_member_top_role(Member, HoistedIdxMap, Acc) ->
    case guild_member_list_common:get_member_user_id(Member) of
        undefined -> Acc;
        UserId -> Acc#{UserId => member_top_role(Member, HoistedIdxMap)}
    end.

-spec find_top_hoisted_role([integer()], [integer()]) -> integer() | undefined.
find_top_hoisted_role(MemberRoleIds, HoistedRoleIds) ->
    Shared = [RId || RId <- HoistedRoleIds, role_is_member(RId, MemberRoleIds)],
    case Shared of
        [] -> undefined;
        [Top | _] -> Top
    end.

-spec role_is_member(integer(), [integer()]) -> boolean().
role_is_member(RoleId, MemberRoleIds) ->
    lists:member(RoleId, MemberRoleIds).

-spec count_ungrouped_online([map()], [map()]) -> non_neg_integer().
count_ungrouped_online(OnlineMembers, HoistedRoles) ->
    HoistedIdxMap = hoisted_idx_map(HoistedRoles),
    lists:foldl(
        fun(Member, Acc) -> count_ungrouped_member(Member, HoistedIdxMap, Acc) end,
        0,
        OnlineMembers
    ).

-spec count_ungrouped_member(map(), #{integer() => non_neg_integer()}, non_neg_integer()) ->
    non_neg_integer().
count_ungrouped_member(Member, HoistedIdxMap, Acc) ->
    case member_top_role(Member, HoistedIdxMap) of
        undefined -> Acc + 1;
        _ -> Acc
    end.

-spec role_id(map()) -> integer() | undefined.
role_id(Role) ->
    case snowflake_id:parse_maybe(maps:get(<<"id">>, Role, undefined)) of
        RoleId when is_integer(RoleId), RoleId > 0 -> RoleId;
        _ -> undefined
    end.
