%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_sender).
-typing([eqwalizer]).

-export([
    send_to_user_subscriptions/3,
    send_clear_to_user_subscriptions/5,
    send_push_notifications/1,
    send_push_notifications/8,
    send_clear_channel_notifications/4
]).

-export_type([send_context/0]).

-type send_context() :: #{
    message_data := map(),
    guild_id := integer(),
    channel_id := integer(),
    message_id := integer(),
    guild_name := binary() | undefined,
    channel_name := binary() | undefined,
    badge_count := non_neg_integer(),
    content_preview := binary(),
    markdown_context := map()
}.

-spec send_to_user_subscriptions(integer(), list(), send_context()) -> ok.
send_to_user_subscriptions(UserId, Subscriptions, SendContext) ->
    BadgeCount = maps:get(badge_count, SendContext),
    NotificationPayload = notification_payload(UserId, SendContext),
    logger:debug("Push: sending to user subscriptions", #{
        user_id => UserId,
        subscription_count => length(Subscriptions),
        badge_count => BadgeCount
    }),
    FailedSubscriptions = send_subscriptions(UserId, NotificationPayload, Subscriptions, []),
    handle_failed_subscriptions(UserId, FailedSubscriptions).

-spec notification_payload(integer(), send_context()) -> map().
notification_payload(UserId, SendContext) ->
    #{
        message_data := MessageData,
        guild_id := GuildId,
        channel_id := ChannelId,
        message_id := MessageId,
        guild_name := GuildName,
        channel_name := ChannelName,
        badge_count := BadgeCount,
        content_preview := ContentPreview,
        markdown_context := MarkdownContext
    } = SendContext,
    AuthorData = maps:get(<<"author">>, MessageData, #{}),
    AuthorUsername = maps:get(<<"username">>, AuthorData, <<"Unknown">>),
    AuthorAvatar = maps:get(<<"avatar">>, AuthorData, null),
    AuthorAvatarUrl = resolve_avatar_url(AuthorData, AuthorAvatar),
    push_notification:build_notification_payload(#{
        message_data => MessageData,
        guild_id => GuildId,
        channel_id => ChannelId,
        message_id => MessageId,
        guild_name => GuildName,
        channel_name => ChannelName,
        author_username => AuthorUsername,
        author_avatar_url => AuthorAvatarUrl,
        target_user_id => UserId,
        badge_count => BadgeCount,
        content_preview => ContentPreview,
        markdown_context => MarkdownContext
    }).

-spec send_clear_to_user_subscriptions(
    integer(), list(), integer(), integer(), non_neg_integer()
) -> ok.
send_clear_to_user_subscriptions(UserId, Subscriptions, ChannelId, MessageId, BadgeCount) ->
    Payload = push_notification:build_clear_notification_payload(
        UserId, ChannelId, MessageId, BadgeCount
    ),
    FailedSubscriptions = send_subscriptions(UserId, Payload, Subscriptions, []),
    handle_failed_subscriptions(UserId, FailedSubscriptions).

-spec send_push_notifications(
    [integer()],
    map(),
    integer(),
    integer(),
    integer(),
    binary() | undefined,
    binary() | undefined,
    non_neg_integer()
) -> ok.
send_push_notifications(
    UserIds,
    MessageData,
    GuildId,
    ChannelId,
    MessageId,
    GuildName,
    ChannelName,
    BadgeCountsTtlSeconds
) ->
    send_push_notifications(#{
        user_ids => UserIds,
        message_data => MessageData,
        markdown_context => #{},
        guild_id => GuildId,
        channel_id => ChannelId,
        message_id => MessageId,
        guild_name => GuildName,
        channel_name => ChannelName,
        badge_counts_ttl_seconds => BadgeCountsTtlSeconds
    }).

-spec send_push_notifications(map()) -> ok.
send_push_notifications(#{
    user_ids := UserIds,
    message_data := MessageData,
    markdown_context := MarkdownContext,
    guild_id := GuildId,
    channel_id := ChannelId,
    message_id := MessageId,
    guild_name := GuildName,
    channel_name := ChannelName,
    badge_counts_ttl_seconds := BadgeCountsTtlSeconds
}) ->
    logger:debug("Push: send_push_notifications starting", #{
        message_id => MessageId,
        channel_id => ChannelId,
        guild_id => GuildId,
        user_count => length(UserIds)
    }),
    BadgeCounts = ensure_badge_counts(UserIds, BadgeCountsTtlSeconds),
    logger:debug(
        "Push: badge counts fetched",
        #{message_id => MessageId, badge_count_users => map_size(BadgeCounts)}
    ),
    push_subscriptions:fetch_and_send_subscriptions(
        UserIds,
        MessageData,
        GuildId,
        ChannelId,
        MessageId,
        GuildName,
        ChannelName,
        MarkdownContext,
        BadgeCounts
    ),
    ok.

-spec send_clear_channel_notifications(integer(), integer(), integer(), non_neg_integer()) ->
    ok.
send_clear_channel_notifications(UserId, ChannelId, MessageId, BadgeCountsTtlSeconds) ->
    BadgeCounts = ensure_badge_counts([UserId], BadgeCountsTtlSeconds),
    BadgeCount = maps:get(UserId, BadgeCounts, 0),
    push_subscriptions:fetch_and_send_clear_notification(
        UserId, ChannelId, MessageId, BadgeCount
    ),
    ok.

-spec resolve_avatar_url(map(), binary() | null) -> binary().
resolve_avatar_url(AuthorData, null) ->
    default_avatar_url(author_id_binary(AuthorData));
resolve_avatar_url(AuthorData, Hash) ->
    case author_id_binary(AuthorData) of
        undefined -> default_avatar_url(undefined);
        UserId -> push_utils:construct_avatar_url(UserId, Hash)
    end.

-spec author_id_binary(map()) -> binary() | undefined.
author_id_binary(AuthorData) ->
    case snowflake_id:parse_optional(maps:get(<<"id">>, AuthorData, undefined)) of
        undefined -> undefined;
        UserId -> integer_to_binary(UserId)
    end.

-spec default_avatar_url(binary() | undefined) -> binary().
default_avatar_url(undefined) ->
    push_utils:get_default_avatar_url(<<>>);
default_avatar_url(UserId) ->
    push_utils:get_default_avatar_url(UserId).

-spec handle_failed_subscriptions(integer(), list()) -> ok.
handle_failed_subscriptions(_UserId, []) ->
    ok;
handle_failed_subscriptions(UserId, FailedSubscriptions) ->
    logger:debug(
        "Push: removing failed subscriptions",
        #{user_id => UserId, failed_count => length(FailedSubscriptions)}
    ),
    _ = push_subscriptions:delete_failed_subscriptions(FailedSubscriptions),
    ok.

-spec send_subscriptions(integer(), map(), list(), [map()]) -> [map()].
send_subscriptions(_UserId, _Payload, [], FailedAcc) ->
    lists:reverse(FailedAcc);
send_subscriptions(UserId, Payload, [Subscription | Rest], FailedAcc) ->
    case send_notification_to_subscription(UserId, Subscription, Payload) of
        {true, FailedSubscription} ->
            send_subscriptions(UserId, Payload, Rest, [FailedSubscription | FailedAcc]);
        false ->
            send_subscriptions(UserId, Payload, Rest, FailedAcc)
    end.

-spec send_notification_to_subscription(integer(), map(), map()) -> false | {true, map()}.
send_notification_to_subscription(UserId, Subscription, Payload) ->
    logger:debug("Push: sending to subscription", #{
        user_id => UserId,
        endpoint => maps:get(<<"endpoint">>, Subscription, undefined),
        platform => subscription_platform(Subscription)
    }),
    case subscription_platform(Subscription) of
        <<"web_push">> ->
            push_sender_delivery:send_webpush_notification(UserId, Subscription, Payload);
        <<"android_unified_push">> ->
            push_sender_delivery:send_webpush_notification(UserId, Subscription, Payload);
        <<"android_fcm">> ->
            push_fcm:send(UserId, Subscription, Payload);
        <<"ios_apns">> ->
            push_apns:send(UserId, Subscription, Payload);
        Platform ->
            logger:warning(
                "Push: unsupported subscription platform",
                #{user_id => UserId, platform => Platform}
            ),
            false
    end.

-spec subscription_platform(map()) -> binary().
subscription_platform(Subscription) ->
    Platform = maps:get(<<"platform">>, Subscription, <<"web_push">>),
    push_utils:normalize_binary(Platform, <<"web_push">>).

-spec ensure_badge_counts([integer()], non_neg_integer()) -> map().
ensure_badge_counts(UserIds, TTL) ->
    Now = erlang:system_time(second),
    {CachedCounts, Missing} = lists:foldl(
        fun(UserId, {Acc, MissingAcc}) ->
            check_badge_cache(UserId, TTL, Now, Acc, MissingAcc)
        end,
        {#{}, []},
        UserIds
    ),
    case lists:usort(Missing) of
        [] -> CachedCounts;
        UniqueMissing -> fetch_badge_counts(UniqueMissing, CachedCounts, Now)
    end.

-spec check_badge_cache(integer(), non_neg_integer(), integer(), map(), [integer()]) ->
    {map(), [integer()]}.
check_badge_cache(UserId, TTL, Now, Acc, MissingAcc) ->
    case push_ets_cache:get_badge_count(UserId) of
        {Count, Timestamp} when TTL > 0, Now - Timestamp < TTL ->
            {Acc#{UserId => Count}, MissingAcc};
        _ ->
            {Acc, [UserId | MissingAcc]}
    end.

-spec fetch_badge_counts([integer()], map(), integer()) -> map().
fetch_badge_counts(UserIds, Counts, CachedAt) ->
    Request = #{
        <<"type">> => <<"get_badge_counts">>,
        <<"user_ids">> => [integer_to_binary(UserId) || UserId <- UserIds]
    },
    case rpc_client:call(Request) of
        {ok, Data} ->
            BadgeData = maps:get(<<"badge_counts">>, Data, #{}),
            merge_badge_data(UserIds, BadgeData, Counts, CachedAt);
        {error, _Reason} ->
            Counts
    end.

-spec merge_badge_data([integer()], map(), map(), integer()) -> map().
merge_badge_data(UserIds, BadgeData, Counts, CachedAt) ->
    lists:foldl(
        fun(UserId, Acc) ->
            UserIdBin = integer_to_binary(UserId),
            Count = normalize_badge_count(maps:get(UserIdBin, BadgeData, 0)),
            push_ets_cache:put_badge_count(UserId, Count, CachedAt),
            Acc#{UserId => Count}
        end,
        Counts,
        UserIds
    ).

-spec normalize_badge_count(integer() | term()) -> non_neg_integer().
normalize_badge_count(Value) when is_integer(Value), Value >= 0 -> Value;
normalize_badge_count(_) -> 0.
