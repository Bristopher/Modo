%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_normalize).
-typing([eqwalizer]).

-export([
    optional_guild_id/1,
    notification_level/1
]).

-spec optional_guild_id(term()) -> pos_integer() | undefined.
optional_guild_id(Value) -> snowflake_id:parse_maybe(Value).

-spec notification_level(term()) -> integer() | undefined.
notification_level(undefined) ->
    0;
notification_level(null) ->
    0;
notification_level(Value) ->
    case guild_data_normalize_schema:int(Value) of
        Level when Level >= -1, Level =< 3 -> Level;
        _ -> undefined
    end.
