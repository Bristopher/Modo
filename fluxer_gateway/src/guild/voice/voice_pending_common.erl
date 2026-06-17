%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(voice_pending_common).
-typing([eqwalizer]).

-export([
    add_pending_connection/3,
    remove_pending_connection/2,
    get_pending_connection/2,
    confirm_pending_connection/2
]).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").
-endif.

-export_type([
    connection_id/0,
    pending_metadata/0,
    pending_map/0
]).

-type connection_id() :: binary().
-type pending_metadata() :: map().
-type pending_map() :: #{connection_id() => pending_metadata()}.

-spec add_pending_connection(connection_id(), pending_metadata(), pending_map()) ->
    pending_map().
add_pending_connection(ConnectionId, Metadata, PendingMap) ->
    PendingMap#{ConnectionId => Metadata#{joined_at => erlang:system_time(millisecond)}}.

-spec remove_pending_connection(connection_id() | undefined, pending_map()) -> pending_map().
remove_pending_connection(undefined, PendingMap) ->
    PendingMap;
remove_pending_connection(ConnectionId, PendingMap) ->
    maps:remove(ConnectionId, PendingMap).

-spec get_pending_connection(connection_id() | undefined, pending_map()) ->
    pending_metadata() | undefined.
get_pending_connection(undefined, _PendingMap) ->
    undefined;
get_pending_connection(ConnectionId, PendingMap) ->
    maps:get(ConnectionId, PendingMap, undefined).

-spec confirm_pending_connection(connection_id() | undefined, pending_map()) ->
    {confirmed, pending_map()} | {not_found, pending_map()}.
confirm_pending_connection(undefined, PendingMap) ->
    {not_found, PendingMap};
confirm_pending_connection(ConnectionId, PendingMap) ->
    case maps:get(ConnectionId, PendingMap, undefined) of
        undefined ->
            {not_found, PendingMap};
        _Metadata ->
            {confirmed, maps:remove(ConnectionId, PendingMap)}
    end.

-ifdef(TEST).

add_pending_connection_test() ->
    PendingMap = #{},
    Metadata = #{user_id => 1, channel_id => 2},
    Result = add_pending_connection(<<"conn">>, Metadata, PendingMap),
    ?assert(maps:is_key(<<"conn">>, Result)),
    StoredMetadata = maps:get(<<"conn">>, Result),
    ?assertEqual(1, maps:get(user_id, StoredMetadata)),
    ?assertEqual(2, maps:get(channel_id, StoredMetadata)),
    ?assert(maps:is_key(joined_at, StoredMetadata)).

remove_pending_connection_test() ->
    PendingMap = #{<<"conn">> => #{user_id => 1}},
    ?assertEqual(#{}, remove_pending_connection(<<"conn">>, PendingMap)),
    ?assertEqual(PendingMap, remove_pending_connection(undefined, PendingMap)),
    ?assertEqual(PendingMap, remove_pending_connection(<<"other">>, PendingMap)).

get_pending_connection_test() ->
    PendingMap = #{<<"conn">> => #{user_id => 1}},
    ?assertEqual(#{user_id => 1}, get_pending_connection(<<"conn">>, PendingMap)),
    ?assertEqual(undefined, get_pending_connection(<<"other">>, PendingMap)),
    ?assertEqual(undefined, get_pending_connection(undefined, PendingMap)).

confirm_pending_connection_test() ->
    PendingMap = #{<<"conn">> => #{user_id => 1}},
    ?assertMatch({confirmed, #{}}, confirm_pending_connection(<<"conn">>, PendingMap)),
    ?assertMatch({not_found, _}, confirm_pending_connection(<<"other">>, PendingMap)),
    ?assertMatch({not_found, _}, confirm_pending_connection(undefined, PendingMap)).

-endif.
