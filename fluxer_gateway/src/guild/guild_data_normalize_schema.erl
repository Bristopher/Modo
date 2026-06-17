%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_data_normalize_schema).
-typing([eqwalizer]).

-export([
    int/1,
    normalize_int_fields/2,
    normalize_nullable_int_fields/2
]).

-define(INT32_MIN, 0).
-define(INT32_MAX, 2147483647).

-spec int(term()) -> integer() | undefined.
int(Value) when is_integer(Value) ->
    int32(Value);
int(Value) when is_binary(Value) ->
    parse_int_binary(Value);
int(Value) when is_list(Value) ->
    parse_int_list(Value);
int(_) ->
    undefined.

-spec normalize_int_fields([binary()], map()) -> map().
normalize_int_fields(Keys, Map) ->
    lists:foldl(fun normalize_int_field/2, Map, Keys).

-spec normalize_nullable_int_fields([binary()], map()) -> map().
normalize_nullable_int_fields(Keys, Map) ->
    lists:foldl(fun normalize_nullable_int_field/2, Map, Keys).

-spec normalize_int_field(binary(), map()) -> map().
normalize_int_field(Key, Map) ->
    case maps:find(Key, Map) of
        {ok, Value} -> put_int(Key, Value, Map);
        error -> Map
    end.

-spec normalize_nullable_int_field(binary(), map()) -> map().
normalize_nullable_int_field(Key, Map) ->
    case maps:find(Key, Map) of
        {ok, null} -> Map#{Key => null};
        {ok, Value} -> put_int(Key, Value, Map);
        error -> Map
    end.

-spec put_int(binary(), term(), map()) -> map().
put_int(Key, Value, Map) ->
    put_int_result(Key, int(Value), Map).

-spec put_int_result(binary(), integer() | undefined, map()) -> map().
put_int_result(Key, undefined, Map) ->
    maps:remove(Key, Map);
put_int_result(Key, Int, Map) ->
    Map#{Key => Int}.

-spec parse_int_binary(binary()) -> integer() | undefined.
parse_int_binary(<<>>) ->
    undefined;
parse_int_binary(Value) ->
    try
        int32(binary_to_integer(Value))
    catch
        error:badarg -> undefined
    end.

-spec parse_int_list([term()]) -> integer() | undefined.
parse_int_list([]) ->
    undefined;
parse_int_list(Value) ->
    parse_int_list(Value, 0).

-spec parse_int_list([term()], non_neg_integer()) -> integer() | undefined.
parse_int_list([], Acc) ->
    int32(Acc);
parse_int_list([Digit | Rest], Acc) when is_integer(Digit), Digit >= $0, Digit =< $9 ->
    parse_int_list(Rest, Acc * 10 + Digit - $0);
parse_int_list(_, _Acc) ->
    undefined.

-spec int32(integer()) -> integer() | undefined.
int32(Value) when Value >= ?INT32_MIN, Value =< ?INT32_MAX ->
    Value;
int32(_) ->
    undefined.
