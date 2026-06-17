%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_node_name).
-typing([eqwalizer]).

-export([from_string/1, from_parts/2]).

-define(MAX_NODE_NAME_LENGTH, 255).
-define(MAX_NODE_PART_LENGTH, 128).

-spec from_string(string()) -> {ok, node()} | error.
from_string(Name0) when is_list(Name0) ->
    Name = string:trim(Name0),
    case valid_node_name(Name) of
        true -> {ok, list_to_atom(Name)};
        false -> error
    end.

-spec from_parts(string(), string()) -> {ok, node()} | error.
from_parts(Base, Host) when is_list(Base), is_list(Host) ->
    from_string(Base ++ "@" ++ Host).

-spec valid_node_name(string()) -> boolean().
valid_node_name(Name) ->
    valid_node_name_length(Name) andalso valid_node_name_parts(string:split(Name, "@", all)).

-spec valid_node_name_length(string()) -> boolean().
valid_node_name_length(Name) ->
    Length = length(Name),
    Length > 0 andalso Length =< ?MAX_NODE_NAME_LENGTH.

-spec valid_node_name_parts([string()]) -> boolean().
valid_node_name_parts([NamePart, HostPart]) ->
    valid_node_part(NamePart) andalso valid_node_part(HostPart);
valid_node_name_parts(_) ->
    false.

-spec valid_node_part(string()) -> boolean().
valid_node_part(Part) ->
    Length = length(Part),
    Length > 0 andalso Length =< ?MAX_NODE_PART_LENGTH andalso
        lists:all(fun valid_node_char/1, Part).

-spec valid_node_char(char()) -> boolean().
valid_node_char(Char) when Char >= $a, Char =< $z -> true;
valid_node_char(Char) when Char >= $A, Char =< $Z -> true;
valid_node_char(Char) when Char >= $0, Char =< $9 -> true;
valid_node_char($_) -> true;
valid_node_char($-) -> true;
valid_node_char($.) -> true;
valid_node_char(_) -> false.
