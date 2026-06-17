%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_rpc_push).
-typing([eqwalizer]).

-export([execute_method/2]).

-spec execute_method(binary(), map()) -> true.
execute_method(<<"push.sync_user_guild_settings">>, P) ->
    do_sync_user_guild_settings(P);
execute_method(<<"push.sync_user_blocked_ids">>, P) ->
    do_sync_user_blocked_ids(P);
execute_method(<<"push.invalidate_badge_count">>, P) ->
    do_invalidate_badge_count(P);
execute_method(<<"push.clear_channel_notifications">>, P) ->
    do_clear_channel_notifications(P);
execute_method(<<"push.invalidate_subscriptions">>, P) ->
    do_invalidate_subscriptions(P).

-spec do_sync_user_guild_settings(map()) -> true.
do_sync_user_guild_settings(#{
    <<"user_id">> := UBin,
    <<"guild_id">> := GBin,
    <<"user_guild_settings">> := Settings
}) ->
    UserId = validation:snowflake_or_throw(<<"user_id">>, UBin),
    GuildId = validation:snowflake_or_throw(<<"guild_id">>, GBin),
    fanout_local_cache_mutation(push, sync_user_guild_settings_local, [
        UserId, GuildId, Settings
    ]),
    true.

-spec do_sync_user_blocked_ids(map()) -> true.
do_sync_user_blocked_ids(#{<<"user_id">> := UBin, <<"blocked_user_ids">> := BlockedRaw}) ->
    UserId = validation:snowflake_or_throw(<<"user_id">>, UBin),
    BlockedIds = validation:snowflake_list_or_throw(<<"blocked_user_ids">>, BlockedRaw),
    fanout_local_cache_mutation(push, sync_user_blocked_ids_local, [UserId, BlockedIds]),
    true.

-spec do_invalidate_badge_count(map()) -> true.
do_invalidate_badge_count(#{<<"user_id">> := UBin}) ->
    UserId = validation:snowflake_or_throw(<<"user_id">>, UBin),
    fanout_local_cache_mutation(push, invalidate_user_badge_count_local, [UserId]),
    true.

-spec do_clear_channel_notifications(map()) -> true.
do_clear_channel_notifications(#{
    <<"user_id">> := UBin,
    <<"channel_id">> := CBin,
    <<"message_id">> := MBin
}) ->
    UserId = validation:snowflake_or_throw(<<"user_id">>, UBin),
    ChannelId = validation:snowflake_or_throw(<<"channel_id">>, CBin),
    MessageId = validation:snowflake_or_throw(<<"message_id">>, MBin),
    ClearFun = fun() ->
        push:clear_channel_notifications(UserId, ChannelId, MessageId)
    end,
    shard_utils:safe_apply(ClearFun, ok),
    true.

-spec do_invalidate_subscriptions(map()) -> true.
do_invalidate_subscriptions(#{<<"user_id">> := UBin}) ->
    UserId = validation:snowflake_or_throw(<<"user_id">>, UBin),
    fanout_local_cache_mutation(push, invalidate_user_subscriptions_local, [UserId]),
    true.

-spec fanout_local_cache_mutation(module(), atom(), [term()]) -> ok.
fanout_local_cache_mutation(Module, Function, Args) ->
    Nodes = [node() | nodes()],
    {_Replies, FailedNodes} = rpc:multicall(Nodes, Module, Function, Args, 5000),
    case FailedNodes of
        [] ->
            ok;
        _ ->
            logger:warning(
                "push cache mutation fanout failed"
                " module=~p function=~p failed_nodes=~p",
                [Module, Function, FailedNodes]
            ),
            ok
    end.
