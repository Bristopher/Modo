%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_http_client_response_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

-define(CIRCUIT_TABLE, gateway_http_circuit_breaker).

allow_circuit_request_uses_recovery_timeout_test() ->
    cleanup_circuit_table(),
    ensure_circuit_table(),
    Key = {rpc, <<"example.test">>},
    Now = erlang:system_time(millisecond),
    OpenedAt = Now - 4000,
    ets:insert(
        ?CIRCUIT_TABLE,
        {Key, #{
            state => open,
            results => [],
            opened_at => OpenedAt,
            updated_at => OpenedAt
        }}
    ),
    ?assertEqual(
        {error, circuit_open}, gateway_http_client_response:allow_circuit_request(Key, 5000)
    ),
    ?assertEqual(ok, gateway_http_client_response:allow_circuit_request(Key, 3000)),
    [{Key, State}] = ets:lookup(?CIRCUIT_TABLE, Key),
    ?assertEqual(half_open, maps:get(state, State)),
    cleanup_circuit_table().

update_circuit_state_uses_failure_threshold_test() ->
    cleanup_circuit_table(),
    ensure_circuit_table(),
    Key = {push, <<"push.example.test">>},
    Failure = {ok, 503, [], <<>>},
    ok = gateway_http_client_response:update_circuit_state_direct(Key, Failure, 3),
    ok = gateway_http_client_response:update_circuit_state_direct(Key, Failure, 3),
    [{Key, ClosedState}] = ets:lookup(?CIRCUIT_TABLE, Key),
    ?assertEqual(closed, maps:get(state, ClosedState)),
    ok = gateway_http_client_response:update_circuit_state_direct(Key, Failure, 3),
    [{Key, OpenState}] = ets:lookup(?CIRCUIT_TABLE, Key),
    ?assertEqual(open, maps:get(state, OpenState)),
    ?assertEqual(
        {error, circuit_open}, gateway_http_client_response:allow_circuit_request(Key, 60000)
    ),
    cleanup_circuit_table().

is_stale_circuit_matches_closed_entries_without_failures_key_test() ->
    Now = 10000,
    OldClosed = #{state => closed, results => [], updated_at => 8000},
    FreshClosed = #{state => closed, results => [], updated_at => 9500},
    ?assertEqual(true, gateway_http_client_response:is_stale_circuit(OldClosed, Now, 1000)),
    ?assertEqual(false, gateway_http_client_response:is_stale_circuit(FreshClosed, Now, 1000)).

ensure_circuit_table() ->
    case ets:whereis(?CIRCUIT_TABLE) of
        undefined ->
            ets:new(?CIRCUIT_TABLE, [named_table, public, set]),
            ok;
        _ ->
            ok
    end.

cleanup_circuit_table() ->
    try ets:delete(?CIRCUIT_TABLE) of
        _ -> ok
    catch
        error:badarg -> ok
    end.
