%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(presence_broadcast_subscriptions).
-typing([eqwalizer]).

-export([
    ensure_initial_global_subscriptions/1,
    sync_friend_subscriptions/3,
    sync_group_dm_subscriptions/2,
    maybe_send_cached_presences/2,
    send_cached_presences_to_session/2,
    maybe_force_offline/2,
    map_from_ids/1,
    normalize_group_dm_recipients/3
]).

-export_type([user_id/0, state/0, recipients_by_channel/0]).

-type user_id() :: integer().
-type state() :: map().
-type recipients_by_channel() :: map().

-spec ensure_initial_global_subscriptions(state()) -> state().
ensure_initial_global_subscriptions(State) ->
    case maps:get(is_bot, State, false) of
        true -> State;
        false -> subscribe_friends_and_gdm(State)
    end.

-spec sync_friend_subscriptions([user_id()], [user_id()], state()) -> state().
sync_friend_subscriptions(_FriendIds, _FlushedIds, #{is_bot := true} = State) ->
    State;
sync_friend_subscriptions(FriendIds, FlushedIds, State) ->
    ExistingIds = maps:keys(maps:get(friends, State, #{})),
    Additions = lists:subtract(FriendIds, ExistingIds),
    Removals = lists:subtract(ExistingIds, FriendIds),
    State1 = add_friend_subscriptions(Additions, State),
    State2 = remove_friend_subscriptions(Removals, State1),
    State3 = State2#{friends := map_from_ids(FriendIds)},
    CacheTargets = lists:subtract(Additions, FlushedIds),
    State4 = maybe_send_cached_presences(CacheTargets, State3),
    maybe_force_offline(Removals, State4).

-spec sync_group_dm_subscriptions(recipients_by_channel(), state()) -> state().
sync_group_dm_subscriptions(_RecipientsByChannel, #{is_bot := true} = State) ->
    State;
sync_group_dm_subscriptions(RecipientsByChannel, State) ->
    Current = maps:get(group_dm_recipients, State, #{}),
    Normalized = normalize_group_dm_recipients(
        RecipientsByChannel, maps:get(user_id, State), false
    ),
    {ToAdd, ToRemove} = diff_group_dm_recipients(Current, Normalized),
    State1 = apply_gdm_additions(ToAdd, State),
    State2 = apply_gdm_removals(ToRemove, State1),
    AddedUsers = lists:usort([UserId || {UserId, _} <- ToAdd]),
    State3 = maybe_send_cached_presences(AddedUsers, State2),
    RemovedUsers = lists:usort([UserId || {UserId, _} <- ToRemove]),
    State4 = maybe_force_offline(RemovedUsers, State3),
    State4#{group_dm_recipients := Normalized}.

-spec maybe_send_cached_presences([user_id()], state()) -> state().
maybe_send_cached_presences([], State) ->
    State;
maybe_send_cached_presences(UserIds, State) ->
    lists:foreach(fun(Uid) -> maybe_notify_cached_presence(Uid, State) end, UserIds),
    State.

-spec send_cached_presences_to_session(pid(), state()) -> ok.
send_cached_presences_to_session(_SessionPid, #{is_bot := true}) ->
    ok;
send_cached_presences_to_session(SessionPid, State) when is_pid(SessionPid) ->
    Targets = collect_initial_presence_targets(State),
    dispatch_visible_presences_to_session(SessionPid, Targets);
send_cached_presences_to_session(_SessionPid, _State) ->
    ok.

-spec maybe_force_offline([user_id()], state()) -> state().
maybe_force_offline(UserIds, State) ->
    Subscriptions = maps:get(subscriptions, State, #{}),
    lists:foldl(
        fun(Uid, Acc) -> send_offline_if_unsubscribed(Uid, Subscriptions, Acc) end,
        State,
        UserIds
    ).

-spec map_from_ids([term()]) -> #{user_id() => true}.
map_from_ids(Ids) when is_list(Ids) ->
    presence_targets:map_from_ids(user_ids(Ids)).

-spec normalize_group_dm_recipients(recipients_by_channel(), user_id(), boolean()) ->
    #{integer() => #{user_id() => true}}.
normalize_group_dm_recipients(_RecipientsByChannel, _UserId, true) ->
    #{};
normalize_group_dm_recipients(RecipientsByChannel, UserId, false) ->
    maps:fold(
        fun
            (ChannelId, RecipientIds, Acc) when is_integer(ChannelId) ->
                Acc#{
                    ChannelId =>
                        map_from_ids([
                            RId
                         || RId <- recipient_list(RecipientIds), RId =/= UserId
                        ])
                };
            (_ChannelId, _RecipientIds, Acc) ->
                Acc
        end,
        #{},
        RecipientsByChannel
    ).

-spec subscribe_friends_and_gdm(state()) -> state().
subscribe_friends_and_gdm(State) ->
    FriendIds = maps:keys(maps:get(friends, State, #{})),
    State1 = add_friend_subscriptions(FriendIds, State),
    GroupDm = maps:get(group_dm_recipients, State, #{}),
    maps:fold(
        fun(ChannelId, Recipients, AccState) ->
            subscribe_gdm_recipients(ChannelId, maps:keys(Recipients), AccState)
        end,
        State1,
        GroupDm
    ).

-spec subscribe_gdm_recipients(integer(), [user_id()], state()) -> state().
subscribe_gdm_recipients(ChannelId, RecipientIds, State) ->
    lists:foldl(
        fun(RId, A) -> ensure_subscription(RId, gdm, ChannelId, A) end,
        State,
        RecipientIds
    ).

-spec add_friend_subscriptions([user_id()], state()) -> state().
add_friend_subscriptions(Additions, State) ->
    lists:foldl(
        fun(FId, Acc) -> ensure_subscription(FId, friend, undefined, Acc) end,
        State,
        Additions
    ).

-spec remove_friend_subscriptions([user_id()], state()) -> state().
remove_friend_subscriptions(Removals, State) ->
    lists:foldl(
        fun(FId, Acc) -> remove_subscription_reason(FId, friend, undefined, Acc) end,
        State,
        Removals
    ).

-spec apply_gdm_additions([{user_id(), integer()}], state()) -> state().
apply_gdm_additions(ToAdd, State) ->
    lists:foldl(
        fun({UserId, ChannelId}, Acc) -> ensure_subscription(UserId, gdm, ChannelId, Acc) end,
        State,
        ToAdd
    ).

-spec apply_gdm_removals([{user_id(), integer()}], state()) -> state().
apply_gdm_removals(ToRemove, State) ->
    lists:foldl(
        fun({UserId, ChannelId}, Acc) ->
            remove_subscription_reason(UserId, gdm, ChannelId, Acc)
        end,
        State,
        ToRemove
    ).

-spec diff_group_dm_recipients(
    #{integer() => #{user_id() => true}},
    #{integer() => #{user_id() => true}}
) ->
    {[{user_id(), integer()}], [{user_id(), integer()}]}.
diff_group_dm_recipients(Old, New) ->
    OldPairs = extract_channel_user_pairs(Old),
    NewPairs = extract_channel_user_pairs(New),
    {lists:subtract(NewPairs, OldPairs), lists:subtract(OldPairs, NewPairs)}.

-spec extract_channel_user_pairs(#{integer() => #{user_id() => true}}) ->
    [{user_id(), integer()}].
extract_channel_user_pairs(Map) ->
    maps:fold(
        fun(ChannelId, Recipients, Acc) ->
            channel_user_pairs(ChannelId, maps:keys(Recipients), Acc)
        end,
        [],
        Map
    ).

-spec channel_user_pairs(integer(), [user_id()], [{user_id(), integer()}]) ->
    [{user_id(), integer()}].
channel_user_pairs(ChannelId, UserIds, Acc) ->
    [{UserId, ChannelId} || UserId <- UserIds] ++ Acc.

-spec ensure_subscription(user_id(), friend | gdm, integer() | undefined, state()) -> state().
ensure_subscription(UserId, _Reason, _ChannelId, #{user_id := UserId} = State) ->
    State;
ensure_subscription(UserId, Reason, ChannelId, State) ->
    Subscriptions = maps:get(subscriptions, State, #{}),
    Entry0 = maps:get(UserId, Subscriptions, #{friend => false, gdm_channels => #{}}),
    Entry1 = update_subscription_entry(Reason, ChannelId, Entry0),
    WasEmpty = not has_subscription(Entry0),
    maybe_subscribe(UserId, WasEmpty, Entry1),
    State#{subscriptions := Subscriptions#{UserId => Entry1}}.

-spec remove_subscription_reason(user_id(), friend | gdm, integer() | undefined, state()) ->
    state().
remove_subscription_reason(UserId, Reason, ChannelId, State) ->
    Subscriptions = maps:get(subscriptions, State, #{}),
    Entry0 = maps:get(UserId, Subscriptions, #{friend => false, gdm_channels => #{}}),
    Entry1 = clear_subscription_entry(Reason, ChannelId, Entry0),
    ShouldRemove = not has_subscription(Entry1),
    NewSubscriptions = subscription_map_after_removal(
        UserId, Entry1, ShouldRemove, Subscriptions
    ),
    maybe_unsubscribe(UserId, ShouldRemove),
    State#{subscriptions := NewSubscriptions}.

-spec update_subscription_entry(friend | gdm, integer() | undefined, map()) -> map().
update_subscription_entry(friend, _ChannelId, Entry) ->
    Entry#{friend => true};
update_subscription_entry(gdm, ChannelId, Entry) ->
    Channels = maps:get(gdm_channels, Entry, #{}),
    Entry#{gdm_channels => Channels#{ChannelId => true}}.

-spec clear_subscription_entry(friend | gdm, integer() | undefined, map()) -> map().
clear_subscription_entry(friend, _ChannelId, Entry) ->
    Entry#{friend => false};
clear_subscription_entry(gdm, ChannelId, Entry) ->
    Channels = maps:get(gdm_channels, Entry, #{}),
    Entry#{gdm_channels => maps:remove(ChannelId, Channels)}.

-spec maybe_subscribe(user_id(), boolean(), map()) -> ok.
maybe_subscribe(UserId, true, Entry) ->
    case has_subscription(Entry) of
        true -> presence_bus:subscribe(UserId);
        false -> ok
    end;
maybe_subscribe(_UserId, false, _Entry) ->
    ok.

-spec maybe_unsubscribe(user_id(), boolean()) -> ok.
maybe_unsubscribe(UserId, true) ->
    presence_bus:unsubscribe(UserId);
maybe_unsubscribe(_UserId, false) ->
    ok.

-spec subscription_map_after_removal(user_id(), map(), boolean(), map()) -> map().
subscription_map_after_removal(UserId, _Entry, true, Subscriptions) ->
    maps:remove(UserId, Subscriptions);
subscription_map_after_removal(UserId, Entry, false, Subscriptions) ->
    Subscriptions#{UserId => Entry}.

-spec has_subscription(map()) -> boolean().
has_subscription(Entry) ->
    maps:get(friend, Entry, false) orelse
        (map_size(maps:get(gdm_channels, Entry, #{})) > 0).

-spec maybe_notify_cached_presence(user_id(), state()) -> ok.
maybe_notify_cached_presence(UserId, State) ->
    case presence_cache_safe:get_visible(UserId) of
        {ok, Presence} ->
            _ = notify_sessions_presence(Presence, State),
            ok;
        not_found ->
            ok
    end.

-spec send_offline_if_unsubscribed(user_id(), map(), state()) -> state().
send_offline_if_unsubscribed(Uid, Subscriptions, State) ->
    case maps:is_key(Uid, Subscriptions) of
        true ->
            State;
        false ->
            Offline = #{
                <<"user">> => #{<<"id">> => integer_to_binary(Uid)},
                <<"status">> => <<"offline">>,
                <<"mobile">> => false,
                <<"afk">> => false,
                <<"custom_status">> => null
            },
            notify_sessions_presence(Offline, State)
    end.

-spec notify_sessions_presence(map(), state()) -> state().
notify_sessions_presence(Payload, State) ->
    SessionPids = presence_connect:collect_session_pids(State),
    gateway_dispatch_relay:dispatch_many(SessionPids, presence_update, Payload),
    State.

-spec collect_initial_presence_targets(state()) -> [user_id()].
collect_initial_presence_targets(State) ->
    FriendIds = maps:keys(maps:get(friends, State, #{})),
    GdmIds = maps:fold(
        fun(_, Recipients, Acc) ->
            maps:keys(Recipients) ++ Acc
        end,
        [],
        maps:get(group_dm_recipients, State, #{})
    ),
    lists:usort(FriendIds ++ GdmIds).

-spec dispatch_visible_presences_to_session(pid(), [user_id()]) -> ok.
dispatch_visible_presences_to_session(_SessionPid, []) ->
    ok;
dispatch_visible_presences_to_session(SessionPid, Targets) ->
    lists:foreach(
        fun(Presence) ->
            gateway_dispatch_relay:dispatch(SessionPid, presence_update, Presence)
        end,
        presence_cache_safe:visible_bulk_get(Targets)
    ),
    ok.

-spec recipient_list([user_id()] | #{user_id() => true} | term()) -> [user_id()].
recipient_list(Value) when is_list(Value) ->
    user_ids(Value);
recipient_list(Value) when is_map(Value) ->
    user_ids(maps:keys(Value));
recipient_list(_) ->
    [].

-spec user_ids([term()]) -> [user_id()].
user_ids(Ids) ->
    [Id || Id <- Ids, is_integer(Id)].
