%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(health_handler).
-typing([eqwalizer]).

-export([init/2]).

-type mode() :: liveness | readiness | drain.

-spec init(cowboy_req:req(), term()) -> {ok, cowboy_req:req(), term()}.
init(Req0, Mode0) ->
    Mode = normalize_mode(Mode0),
    {StatusCode, Body} = response_for_mode(Mode, Req0),
    Req = cowboy_req:reply(
        StatusCode,
        gateway_build_info:version_headers(#{<<"content-type">> => <<"text/plain">>}),
        Body,
        Req0
    ),
    {ok, Req, Mode}.

-spec normalize_mode(term()) -> mode().
normalize_mode(drain) -> drain;
normalize_mode(readiness) -> readiness;
normalize_mode(_) -> liveness.

-spec response_for_mode(mode(), cowboy_req:req()) -> {200 | 403 | 503, binary()}.
response_for_mode(liveness, _Req) ->
    {200, <<"OK">>};
response_for_mode(readiness, Req) ->
    readiness_response(Req);
response_for_mode(drain, Req) ->
    drain_response(Req).

-spec readiness_response(cowboy_req:req()) -> {200 | 403 | 503, binary()}.
readiness_response(Req) ->
    case is_loopback_request(Req) of
        false ->
            {403, <<"FORBIDDEN">>};
        true ->
            readiness_status(gateway_node_router:is_ready())
    end.

-spec readiness_status(boolean()) -> {200 | 503, binary()}.
readiness_status(true) ->
    {200, <<"OK">>};
readiness_status(false) ->
    {503, <<"DRAINING">>}.

-spec drain_response(cowboy_req:req()) -> {200 | 403, binary()}.
drain_response(Req) ->
    case is_loopback_request(Req) of
        false ->
            {403, <<"FORBIDDEN">>};
        true ->
            ok = activate_drain(),
            {200, <<"DRAINING">>}
    end.

-spec activate_drain() -> ok.
activate_drain() ->
    gateway_cluster_handoff:drain_async().

-spec is_loopback_request(cowboy_req:req()) -> boolean().
is_loopback_request(Req) ->
    case cowboy_req:peer(Req) of
        {{127, 0, 0, 1}, _Port} ->
            true;
        {{0, 0, 0, 0, 0, 0, 0, 1}, _Port} ->
            true;
        _ ->
            false
    end.

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

normalize_mode_test() ->
    ?assertEqual(liveness, normalize_mode(undefined)),
    ?assertEqual(liveness, normalize_mode([])),
    ?assertEqual(readiness, normalize_mode(readiness)),
    ?assertEqual(drain, normalize_mode(drain)).

readiness_status_test() ->
    ?assertEqual({200, <<"OK">>}, readiness_status(true)),
    ?assertEqual({503, <<"DRAINING">>}, readiness_status(false)).

activate_drain_sets_draining_flag_test() ->
    persistent_term:erase({fluxer_gateway, draining}),
    ?assertEqual(ok, activate_drain()),
    ?assert(gateway_node_router:is_draining()),
    persistent_term:erase({fluxer_gateway, draining}).

-endif.
