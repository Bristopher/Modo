%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_hotpatch_runtime).
-typing([eqwalizer]).

-export([
    build_sha/0,
    node_name/0,
    is_enabled/0,
    get/2,
    put_ready/1,
    is_ready/0,
    put_status/1,
    status/0
]).

-define(READY_KEY, {gateway_hotpatch, ready}).
-define(STATUS_KEY, {gateway_hotpatch, status}).

-spec build_sha() -> binary().
build_sha() ->
    case os:getenv("BUILD_SHA") of
        false -> build_version();
        "" -> build_version();
        Value -> type_conv:ensure_binary(Value, <<"dev">>)
    end.

-spec build_version() -> binary().
build_version() ->
    case os:getenv("BUILD_VERSION") of
        false -> <<"dev">>;
        "" -> <<"dev">>;
        Value -> type_conv:ensure_binary(Value, <<"dev">>)
    end.

-spec node_name() -> binary().
node_name() ->
    atom_to_binary(node(), utf8).

-spec is_enabled() -> boolean().
is_enabled() ->
    case fluxer_gateway_env:get(hotpatch_enabled) of
        true -> true;
        _ -> false
    end.

-spec get(atom(), term()) -> term().
get(Key, Default) ->
    case fluxer_gateway_env:get(Key) of
        undefined -> Default;
        Value -> Value
    end.

-spec put_ready(boolean()) -> ok.
put_ready(Ready) when is_boolean(Ready) ->
    persistent_term:put(?READY_KEY, Ready),
    ok.

-spec is_ready() -> boolean().
is_ready() ->
    case persistent_term:get(?READY_KEY, true) of
        false -> false;
        _ -> true
    end.

-spec put_status(map()) -> ok.
put_status(Status) ->
    persistent_term:put(?STATUS_KEY, Status),
    ok.

-spec status() -> map().
status() ->
    persistent_term:get(?STATUS_KEY, #{
        enabled => false,
        ready => true,
        build_sha => build_sha(),
        applied_count => 0
    }).
