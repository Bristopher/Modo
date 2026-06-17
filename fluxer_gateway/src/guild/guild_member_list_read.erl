%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_read).
-typing([eqwalizer]).

-export([
    get_member_groups/2,
    get_counts/2,
    get_items_in_range/3,
    get_online_count/1,
    build_sync_response/4,
    build_sync_response_builder/3,
    build_normalized_sync_response_builder/3,
    member_list_snapshot/2,
    snapshot/2,
    hydrate_engine_items/2,
    get_members_cursor/2
]).

-type guild_state() :: map().
-type list_id() :: binary().
-type range() :: {non_neg_integer(), non_neg_integer()}.
-type group_item() :: map().
-type list_item() :: map().
-type store_item() :: {group, binary(), non_neg_integer()} | {member, integer()}.
-type read_context() :: #{
    list_id := list_id(),
    hide_offline := boolean(),
    member_map := map(),
    presence_context := map(),
    state := guild_state()
}.

-export_type([guild_state/0, list_id/0, range/0, group_item/0, list_item/0, store_item/0]).

-spec get_member_groups(list_id(), guild_state()) -> [group_item()].
get_member_groups(ListId, State) ->
    case store_ref_for(ListId, State) of
        undefined ->
            [];
        Ref ->
            StoreGroups = guild_member_list_store:get_groups(Ref),
            visible_groups(StoreGroups)
    end.

-spec get_counts(list_id(), guild_state()) -> {non_neg_integer(), non_neg_integer()}.
get_counts(ListId, State) ->
    case store_ref_for(ListId, State) of
        undefined -> {0, 0};
        Ref -> guild_member_list_store:get_counts(Ref)
    end.

-spec get_items_in_range(list_id(), range(), guild_state()) -> [list_item()].
get_items_in_range(ListId, {Start, End}, State) ->
    case store_ref_for(ListId, State) of
        undefined ->
            [];
        Ref ->
            StoreGroups = guild_member_list_store:get_groups(Ref),
            StoreItems = store_range_items(Ref, StoreGroups, Start, End),
            hydrate_engine_items(ListId, StoreItems, StoreGroups, State)
    end.

-spec get_online_count(guild_state()) -> non_neg_integer().
get_online_count(State) ->
    case store_ref_for(<<"0">>, State) of
        undefined ->
            0;
        Ref ->
            {_Total, Online} = guild_member_list_store:get_counts(Ref),
            Online
    end.

-spec build_sync_response(integer(), list_id(), [range()], guild_state()) -> map().
build_sync_response(GuildId, ListId, Ranges, State) ->
    Builder = build_sync_response_builder(GuildId, ListId, State),
    Builder(Ranges).

-spec build_sync_response_builder(integer(), list_id(), guild_state()) ->
    fun(([range()]) -> map()).
build_sync_response_builder(GuildId, ListId, State) ->
    NormalizedBuilder = build_normalized_sync_response_builder(GuildId, ListId, State),
    fun(Ranges) ->
        NormalizedBuilder(guild_member_list:normalize_ranges(Ranges))
    end.

-spec build_normalized_sync_response_builder(integer(), list_id(), guild_state()) ->
    fun(([range()]) -> map()).
build_normalized_sync_response_builder(GuildId, ListId, State) ->
    build_normalized_sync_response_builder_for_store(
        integer_to_binary(GuildId), ListId, store_ref_for(ListId, State), State
    ).

-spec build_normalized_sync_response_builder_for_store(
    binary(), list_id(), guild_member_list_store:store_ref() | undefined, guild_state()
) -> fun(([range()]) -> map()).
build_normalized_sync_response_builder_for_store(GuildIdBin, ListId, undefined, _State) ->
    fun(NormalizedRanges) ->
        #{
            <<"guild_id">> => GuildIdBin,
            <<"id">> => ListId,
            <<"member_count">> => 0,
            <<"online_count">> => 0,
            <<"groups">> => [],
            <<"ops">> => [sync_op(Range, []) || Range <- NormalizedRanges]
        }
    end;
build_normalized_sync_response_builder_for_store(GuildIdBin, ListId, Ref, State) ->
    {MemberCount, OnlineCount} = guild_member_list_store:get_counts(Ref),
    StoreGroups = guild_member_list_store:get_groups(Ref),
    VisibleGroups = visible_groups(StoreGroups),
    ReadCtx = read_context(ListId, StoreGroups, State),
    fun(NormalizedRanges) ->
        #{
            <<"guild_id">> => GuildIdBin,
            <<"id">> => ListId,
            <<"member_count">> => MemberCount,
            <<"online_count">> => OnlineCount,
            <<"groups">> => VisibleGroups,
            <<"ops">> => [
                range_sync_op_from_context(Ref, Range, StoreGroups, ReadCtx)
             || Range <- NormalizedRanges
            ]
        }
    end.

-spec read_context(list_id(), [{binary(), non_neg_integer()}], guild_state()) -> read_context().
read_context(ListId, StoreGroups, State) ->
    Data = maps:get(data, State, #{}),
    #{
        list_id => ListId,
        hide_offline => offline_hidden(StoreGroups),
        member_map => guild_data_index:member_map(Data),
        presence_context => guild_member_list_connected:presence_context(State),
        state => State
    }.

-spec range_sync_op_from_context(
    guild_member_list_store:store_ref(),
    range(),
    [{binary(), non_neg_integer()}],
    read_context()
) -> map().
range_sync_op_from_context(Ref, {Start, End} = Range, StoreGroups, ReadCtx) ->
    StoreItems = store_range_items_from_context(Ref, StoreGroups, Start, End, ReadCtx),
    sync_op(Range, hydrate_engine_items_from_context(StoreItems, ReadCtx)).

-spec store_range_items_from_context(
    guild_member_list_store:store_ref(),
    [{binary(), non_neg_integer()}],
    non_neg_integer(),
    non_neg_integer(),
    read_context()
) -> [store_item()].
store_range_items_from_context(Ref, _StoreGroups, Start, End, #{hide_offline := false}) ->
    guild_member_list_store:get_items(Ref, Start, End);
store_range_items_from_context(Ref, StoreGroups, Start, End, #{hide_offline := true}) ->
    visible_range_items(Ref, StoreGroups, Start, End).

-spec sync_op(range(), [list_item()]) -> map().
sync_op({Start, End}, Items) ->
    #{<<"op">> => <<"SYNC">>, <<"range">> => [Start, End], <<"items">> => Items}.

-spec member_list_snapshot(list_id(), guild_state()) ->
    {non_neg_integer(), non_neg_integer(), [group_item()], [list_item()]}.
member_list_snapshot(ListId, State) ->
    snapshot(ListId, State).

-spec snapshot(list_id(), guild_state()) ->
    {non_neg_integer(), non_neg_integer(), [group_item()], [list_item()]}.
snapshot(ListId, State) ->
    case store_ref_for(ListId, State) of
        undefined ->
            {0, 0, [], []};
        Ref ->
            {Total, Online} = guild_member_list_store:get_counts(Ref),
            StoreGroups = guild_member_list_store:get_groups(Ref),
            StoreItems = guild_member_list_store:get_items(Ref, 0, Total * 2),
            Items = hydrate_engine_items(ListId, StoreItems, StoreGroups, State),
            {Total, Online, visible_groups(StoreGroups), Items}
    end.

-spec hydrate_engine_items([store_item()], guild_state()) -> [list_item()].
hydrate_engine_items(StoreItems, State) ->
    hydrate_engine_items(<<"0">>, StoreItems, group_tuples(StoreItems), State).

-spec hydrate_engine_items(
    list_id(), [store_item()], [{binary(), non_neg_integer()}], guild_state()
) ->
    [list_item()].
hydrate_engine_items(ListId, StoreItems, StoreGroups, State) ->
    Data = maps:get(data, State, #{}),
    MemberMap = guild_data_index:member_map(Data),
    HideOffline = offline_hidden(StoreGroups),
    hydrate_engine_items_with_context(StoreItems, ListId, HideOffline, MemberMap, State).

-spec hydrate_engine_items_from_context([store_item()], read_context()) -> [list_item()].
hydrate_engine_items_from_context(StoreItems, #{
    list_id := ListId,
    hide_offline := HideOffline,
    member_map := MemberMap,
    presence_context := PresenceCtx,
    state := State
}) ->
    hydrate_engine_items_with_context(
        StoreItems, ListId, HideOffline, MemberMap, PresenceCtx, State
    ).

-spec hydrate_engine_items_with_context(
    [store_item()], list_id(), boolean(), map(), guild_state()
) -> [list_item()].
hydrate_engine_items_with_context(StoreItems, ListId, HideOffline, MemberMap, State) ->
    PresenceCtx = guild_member_list_connected:presence_context(State),
    hydrate_engine_items_with_context(
        StoreItems, ListId, HideOffline, MemberMap, PresenceCtx, State
    ).

-spec hydrate_engine_items_with_context(
    [store_item()], list_id(), boolean(), map(), map(), guild_state()
) -> [list_item()].
hydrate_engine_items_with_context(
    StoreItems, ListId, HideOffline, MemberMap, PresenceCtx, State
) ->
    {Items, _Section} = lists:foldl(
        fun(Item, Acc) ->
            hydrate_store_item(Item, ListId, HideOffline, MemberMap, PresenceCtx, State, Acc)
        end,
        {[], undefined},
        StoreItems
    ),
    lists:reverse(Items).

-spec hydrate_store_item(
    store_item(), list_id(), boolean(), map(), map(), guild_state(), {[list_item()], term()}
) -> {[list_item()], term()}.
hydrate_store_item(
    {group, <<"offline">>, _Count}, _ListId, true, _MemberMap, _PresenceCtx, _State, {Acc, _}
) ->
    {Acc, offline};
hydrate_store_item(
    {group, Id, Count}, _ListId, _HideOffline, _MemberMap, _PresenceCtx, _State, {Acc, _}
) ->
    {
        [
            #{<<"group">> => #{<<"id">> => Id, <<"count">> => Count}}
            | Acc
        ],
        Id
    };
hydrate_store_item(
    {member, _UserId}, _ListId, true, _MemberMap, _PresenceCtx, _State, {Acc, offline}
) ->
    {Acc, offline};
hydrate_store_item(
    {member, UserId}, _ListId, _HideOffline, MemberMap, PresenceCtx, _State, {Acc, Current}
) ->
    case hydrate_member_ref(UserId, MemberMap, PresenceCtx) of
        {true, Item} -> {[Item | Acc], Current};
        false -> {Acc, Current}
    end.

-spec hydrate_member_ref(integer(), map(), map()) -> {true, list_item()} | false.
hydrate_member_ref(UserId, MemberMap, PresenceCtx) ->
    case maps:find(UserId, MemberMap) of
        error ->
            false;
        {ok, Member} ->
            {true, #{
                <<"member">> =>
                    guild_member_list_connected:add_presence_to_member(
                        Member, UserId, PresenceCtx
                    )
            }}
    end.

-spec get_members_cursor(map(), guild_state()) -> {reply, map(), guild_state()}.
get_members_cursor(Request, State) ->
    guild_member_list_read_cursor:get_members_cursor(Request, State).

-spec store_ref_for(list_id(), guild_state()) ->
    guild_member_list_store:store_ref() | undefined.
store_ref_for(<<"0">>, State) ->
    case maps:get(member_list_engine, State, undefined) of
        Ref when is_reference(Ref); is_atom(Ref) -> Ref;
        _ -> undefined
    end;
store_ref_for(ListId, State) ->
    guild_member_list_channel_engine:ref(ListId, State).

-spec visible_groups([{binary(), non_neg_integer()}]) -> [group_item()].
visible_groups(StoreGroups) ->
    Threshold = guild_member_list_offline:threshold(),
    [
        #{<<"id">> => Id, <<"count">> => Count}
     || {Id, Count} <- StoreGroups,
        group_visible(Id, Count, Threshold)
    ].

-spec group_visible(binary(), non_neg_integer(), pos_integer()) -> boolean().
group_visible(<<"offline">>, Count, Threshold) ->
    Count > 0 andalso Count =< Threshold;
group_visible(_Id, Count, _Threshold) ->
    Count > 0.

-spec store_range_items(
    guild_member_list_store:store_ref(),
    [{binary(), non_neg_integer()}],
    non_neg_integer(),
    non_neg_integer()
) -> [store_item()].
store_range_items(Ref, StoreGroups, Start, End) ->
    case offline_hidden(StoreGroups) of
        false ->
            guild_member_list_store:get_items(Ref, Start, End);
        true ->
            visible_range_items(Ref, StoreGroups, Start, End)
    end.

-spec visible_range_items(
    guild_member_list_store:store_ref(),
    [{binary(), non_neg_integer()}],
    non_neg_integer(),
    non_neg_integer()
) -> [store_item()].
visible_range_items(Ref, StoreGroups, Start, End) ->
    VisibleEnd = visible_online_last_index(StoreGroups),
    case Start > VisibleEnd of
        true -> [];
        false -> guild_member_list_store:get_items(Ref, Start, min(End, VisibleEnd))
    end.

-spec visible_online_last_index([{binary(), non_neg_integer()}]) -> integer().
visible_online_last_index(StoreGroups) ->
    visible_online_rows(StoreGroups) - 1.

-spec visible_online_rows([{binary(), non_neg_integer()}]) -> non_neg_integer().
visible_online_rows([]) ->
    0;
visible_online_rows([{<<"offline">>, _Count} | Rest]) ->
    visible_online_rows(Rest);
visible_online_rows([{_Id, Count} | Rest]) when Count > 0 ->
    1 + Count + visible_online_rows(Rest);
visible_online_rows([_ | Rest]) ->
    visible_online_rows(Rest).

-spec offline_hidden([{binary(), non_neg_integer()}]) -> boolean().
offline_hidden(StoreGroups) ->
    OfflineCount = maps:get(<<"offline">>, maps:from_list(StoreGroups), 0),
    OfflineCount > guild_member_list_offline:threshold().

-spec group_tuples([store_item()]) -> [{binary(), non_neg_integer()}].
group_tuples(Items) ->
    [{Id, Count} || {group, Id, Count} <- Items].

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

group_visible_is_threshold_based_test() ->
    Threshold = guild_member_list_offline:threshold(),
    ?assert(group_visible(<<"offline">>, Threshold, Threshold)),
    ?assert(group_visible(<<"offline">>, 1, Threshold)),
    ?assertNot(group_visible(<<"offline">>, 0, Threshold)),
    ?assertNot(group_visible(<<"offline">>, Threshold + 1, Threshold)),
    ?assert(group_visible(<<"online">>, 1, Threshold)),
    ?assertNot(group_visible(<<"online">>, 0, Threshold)).

hidden_offline_range_is_clamped_test() ->
    Ref = guild_member_list_engine:new(),
    HiddenOfflineCount = guild_member_list_offline:threshold() + 1,
    [
        guild_member_list_engine:add_member(
            Ref, U, <<"online_", (integer_to_binary(U))/binary>>, [], true
        )
     || U <- lists:seq(1, 5)
    ],
    [
        guild_member_list_engine:add_member(
            Ref, U, <<"offline_", (integer_to_binary(U))/binary>>, [], false
        )
     || U <- lists:seq(101, 100 + HiddenOfflineCount)
    ],
    StoreGroups = guild_member_list_engine:get_groups(Ref),
    ?assertEqual(5, visible_online_last_index(StoreGroups)),
    ?assertEqual([], [G || #{<<"id">> := <<"offline">>} = G <- visible_groups(StoreGroups)]),
    ?assertEqual([], [
        I
     || {member, U} = I <- store_range_items(Ref, StoreGroups, 0, 100), U >= 101
    ]),
    guild_member_list_engine:destroy(Ref).

build_sync_response_without_store_still_emits_sync_windows_test() ->
    Resp = build_sync_response(123, <<"0">>, [{0, 2}], #{}),
    ?assertEqual(0, maps:get(<<"member_count">>, Resp)),
    ?assertMatch(
        [#{<<"op">> := <<"SYNC">>, <<"range">> := [0, 2], <<"items">> := []}],
        maps:get(<<"ops">>, Resp)
    ).

-endif.
