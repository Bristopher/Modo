%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_write_context).
-typing([eqwalizer]).

-export([with_guild_id/2, list_channel_id/1, list_dispatch_channel_id/1]).

-type guild_state() :: map().
-type list_id() :: binary().
-type channel_id() :: integer().

-export_type([guild_state/0, list_id/0, channel_id/0]).

-spec with_guild_id(guild_state(), fun((pos_integer()) -> {ok, guild_state()})) ->
    {ok, guild_state()}.
with_guild_id(State, Fun) ->
    case guild_id(State) of
        GuildId when is_integer(GuildId), GuildId > 0 -> Fun(GuildId);
        _ -> {ok, State}
    end.

-spec list_channel_id(list_id()) -> channel_id() | undefined.
list_channel_id(<<"0">>) ->
    undefined;
list_channel_id(ListId) when is_binary(ListId) ->
    case snowflake_id:parse_optional(ListId) of
        Id when is_integer(Id), Id > 0 -> Id;
        _ -> undefined
    end;
list_channel_id(_) ->
    undefined.

-spec list_dispatch_channel_id(list_id()) -> {ok, channel_id() | undefined} | error.
list_dispatch_channel_id(<<"0">>) ->
    {ok, undefined};
list_dispatch_channel_id(ListId) ->
    case list_channel_id(ListId) of
        undefined -> error;
        ChannelId -> {ok, ChannelId}
    end.

-spec guild_id(guild_state()) -> integer() | undefined.
guild_id(State) ->
    case snowflake_id:parse_optional(maps:get(id, State, undefined)) of
        GuildId when is_integer(GuildId), GuildId > 0 -> GuildId;
        _ -> undefined
    end.
