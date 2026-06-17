%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_sharding).
-typing([eqwalizer]).

-export([
    parse_identify_shard/1,
    retain_guild_ids_for_shard/2,
    validate_session_guild_count/2,
    guild_matches_shard/2,
    shard_to_wire/1,
    maybe_put_ready_shard/2
]).

-export_type([shard/0]).

-define(MAX_SHARD_COUNT, 16384).
-define(MAX_GUILDS_PER_SHARD, 2500).

-type shard() :: {non_neg_integer(), pos_integer()}.

-spec parse_identify_shard(term()) -> {ok, shard() | undefined} | {error, invalid_shard}.
parse_identify_shard(undefined) ->
    {ok, undefined};
parse_identify_shard(null) ->
    {ok, undefined};
parse_identify_shard([ShardId, NumShards]) ->
    validate_shard_pair(ShardId, NumShards);
parse_identify_shard(_) ->
    {error, invalid_shard}.

-spec validate_shard_pair(term(), term()) -> {ok, shard()} | {error, invalid_shard}.
validate_shard_pair(ShardId, NumShards) when
    is_integer(ShardId),
    is_integer(NumShards),
    ShardId >= 0,
    NumShards > 0,
    NumShards =< ?MAX_SHARD_COUNT,
    ShardId < NumShards
->
    {ok, {ShardId, NumShards}};
validate_shard_pair(_, _) ->
    {error, invalid_shard}.

-spec retain_guild_ids_for_shard([integer()], shard() | undefined) -> [integer()].
retain_guild_ids_for_shard(GuildIds, undefined) ->
    GuildIds;
retain_guild_ids_for_shard(GuildIds, {0, 1}) ->
    GuildIds;
retain_guild_ids_for_shard(GuildIds, Shard) ->
    [GuildId || GuildId <- GuildIds, guild_matches_shard(GuildId, Shard)].

-spec guild_matches_shard(integer(), shard()) -> boolean().
guild_matches_shard(GuildId, {_ShardId, _NumShards}) when GuildId =< 0 ->
    false;
guild_matches_shard(_GuildId, {0, 1}) ->
    true;
guild_matches_shard(GuildId, {ShardId, NumShards}) ->
    ((GuildId bsr 22) rem NumShards) =:= ShardId.

-spec validate_session_guild_count([integer()], shard() | undefined) ->
    ok | {error, sharding_required}.
validate_session_guild_count(GuildIds, Shard) ->
    case length(retain_guild_ids_for_shard(GuildIds, Shard)) > ?MAX_GUILDS_PER_SHARD of
        true -> {error, sharding_required};
        false -> ok
    end.

-spec shard_to_wire(shard() | undefined) -> [non_neg_integer()] | undefined.
shard_to_wire(undefined) ->
    undefined;
shard_to_wire({ShardId, NumShards}) ->
    [ShardId, NumShards].

-spec maybe_put_ready_shard(map(), shard() | undefined) -> map().
maybe_put_ready_shard(ReadyData, undefined) ->
    ReadyData;
maybe_put_ready_shard(ReadyData, Shard) ->
    ReadyData#{<<"shard">> => shard_to_wire(Shard)}.
