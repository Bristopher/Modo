%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(map_utils).
-typing([eqwalizer]).

-export([
    get_safe/3,
    get_nested/3,
    ensure_map/1,
    ensure_list/1,
    filter_by_field/3,
    find_by_field/3,
    get_integer/3,
    get_binary/3
]).

-export_type([key/0, path/0, default/0]).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").
-endif.

-type key() :: atom() | binary() | term().
-type path() :: [key()].
-type default() :: term().

-spec get_safe(Map :: map() | term(), Key :: key(), Default :: default()) -> term().
get_safe(Map, Key, Default) when is_map(Map) ->
    maps:get(Key, Map, Default);
get_safe(_NotMap, _Key, Default) ->
    Default.

-spec get_nested(Map :: map() | term(), Path :: path(), Default :: default()) -> term().
get_nested(Map, [], _Default) when is_map(Map) ->
    Map;
get_nested(_NotMap, [], Default) ->
    Default;
get_nested(Map, [Key | Rest], Default) when is_map(Map) ->
    case maps:find(Key, Map) of
        {ok, Value} ->
            get_nested(Value, Rest, Default);
        error ->
            Default
    end;
get_nested(_NotMap, _Path, Default) ->
    Default.

-spec ensure_map(term()) -> map().
ensure_map(Map) when is_map(Map) ->
    Map;
ensure_map(_NotMap) ->
    #{}.

-spec ensure_list(term()) -> list().
ensure_list(List) when is_list(List) ->
    List;
ensure_list(_NotList) ->
    [].

-spec get_integer(term(), key(), term()) -> integer() | term().
get_integer(Map, Key, Default) when is_map(Map) ->
    Value = maps:get(Key, Map, undefined),
    case type_conv:to_integer(Value) of
        undefined -> Default;
        Converted -> Converted
    end;
get_integer(_NotMap, _Key, Default) ->
    Default.

-spec get_binary(term(), key(), term()) -> binary() | term().
get_binary(Map, Key, Default) when is_map(Map) ->
    Value = maps:get(Key, Map, undefined),
    case type_conv:to_binary(Value) of
        undefined -> Default;
        Converted -> Converted
    end;
get_binary(_NotMap, _Key, Default) ->
    Default.

-spec filter_by_field(List :: term(), Field :: key(), Value :: term()) -> list(map()).
filter_by_field(List, Field, Value) when is_list(List) ->
    filter_by_field_loop(List, Field, Value, []);
filter_by_field(_NotList, _Field, _Value) ->
    [].

-spec filter_by_field_loop(list(), key(), term(), [map()]) -> [map()].
filter_by_field_loop([], _Field, _Value, Acc) ->
    lists:reverse(Acc);
filter_by_field_loop([Item | Rest], Field, Value, Acc) when is_map(Item) ->
    filter_by_field_loop(Rest, Field, Value, append_matching_field(Item, Field, Value, Acc));
filter_by_field_loop([_NotMap | Rest], Field, Value, Acc) ->
    filter_by_field_loop(Rest, Field, Value, Acc).

-spec append_matching_field(map(), key(), term(), [map()]) -> [map()].
append_matching_field(Item, Field, Value, Acc) ->
    case field_matches(Item, Field, Value) of
        true -> [Item | Acc];
        false -> Acc
    end.

-spec find_by_field(List :: term(), Field :: key(), Value :: term()) -> {ok, map()} | error.
find_by_field(List, Field, Value) when is_list(List) ->
    find_by_field_loop(List, Field, Value);
find_by_field(_NotList, _Field, _Value) ->
    error.

-spec find_by_field_loop(list(), key(), term()) -> {ok, map()} | error.
find_by_field_loop([], _Field, _Value) ->
    error;
find_by_field_loop([Item | Rest], Field, Value) when is_map(Item) ->
    case maps:find(Field, Item) of
        {ok, Value} ->
            {ok, Item};
        _ ->
            find_by_field_loop(Rest, Field, Value)
    end;
find_by_field_loop([_NotMap | Rest], Field, Value) ->
    find_by_field_loop(Rest, Field, Value).

-spec field_matches(map(), key(), term()) -> boolean().
field_matches(Item, Field, Value) ->
    case maps:find(Field, Item) of
        {ok, Value} -> true;
        _ -> false
    end.
