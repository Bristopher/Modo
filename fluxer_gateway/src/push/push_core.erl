%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_core).
-typing([eqwalizer]).

-export([handle_message_create/1]).
-export([sync_user_guild_settings/3]).
-export([sync_user_blocked_ids/2]).
-export([invalidate_user_subscriptions/1]).

-spec handle_message_create(map()) -> ok.
handle_message_create(Params) ->
    case fluxer_gateway_env:get(push_enabled) of
        true ->
            push:handle_message_create(Params);
        false ->
            ok
    end.

-spec sync_user_guild_settings(integer(), integer(), map()) -> ok.
sync_user_guild_settings(UserId, GuildId, UserGuildSettings) ->
    push:sync_user_guild_settings(UserId, GuildId, UserGuildSettings).

-spec sync_user_blocked_ids(integer(), [integer()]) -> ok.
sync_user_blocked_ids(UserId, BlockedIds) ->
    push:sync_user_blocked_ids(UserId, BlockedIds).

-spec invalidate_user_subscriptions(integer()) -> ok.
invalidate_user_subscriptions(UserId) ->
    push:invalidate_user_subscriptions(UserId).
