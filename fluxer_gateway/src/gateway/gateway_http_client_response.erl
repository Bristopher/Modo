%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_http_client_response).
-typing([eqwalizer]).

-export([
    allow_circuit_request/2,
    update_circuit_state_direct/3,
    acquire_inflight_slot/2,
    release_inflight_slot/1,
    prune_circuit_table/0,
    is_stale_circuit/3
]).
-export_type([response/0]).

-define(CIRCUIT_TABLE, gateway_http_circuit_breaker).
-define(INFLIGHT_TABLE, gateway_http_inflight).

-define(CB_WINDOW_MS, 10000).
-define(CB_FAILURE_RATE_PCT, 80).

-type response() :: {ok, non_neg_integer(), [{binary(), binary()}], binary()} | {error, term()}.

-spec allow_circuit_request({atom(), binary()}, pos_integer()) -> ok | {error, circuit_open}.
allow_circuit_request(CircuitKey, RecoveryTimeoutMs) ->
    Now = erlang:system_time(millisecond),
    case safe_lookup_circuit(CircuitKey) of
        [] ->
            ok;
        [{_, #{state := open, opened_at := OpenedAt}}] ->
            maybe_transition_half_open(CircuitKey, OpenedAt, Now, RecoveryTimeoutMs);
        _ ->
            ok
    end.

-spec update_circuit_state_direct({atom(), binary()}, response(), pos_integer()) -> ok.
update_circuit_state_direct(CircuitKey, Result, FailureThreshold) ->
    Now = erlang:system_time(millisecond),
    IsFailure = is_countable_failure(Result),
    record_result(CircuitKey, IsFailure, Now, FailureThreshold).

-spec acquire_inflight_slot(atom(), pos_integer()) -> ok | {error, overloaded}.
acquire_inflight_slot(Workload, MaxConcurrency) ->
    case safe_update_counter(?INFLIGHT_TABLE, Workload, {2, 1}) of
        {ok, Count} when Count =< MaxConcurrency ->
            ok;
        {ok, _Count} ->
            _ = safe_update_counter(?INFLIGHT_TABLE, Workload, {2, -1}),
            {error, overloaded};
        {error, _Reason} ->
            {error, overloaded}
    end.

-spec release_inflight_slot(atom()) -> ok.
release_inflight_slot(Workload) ->
    case safe_update_counter(?INFLIGHT_TABLE, Workload, {2, -1}) of
        {ok, V} when V < 0 ->
            reset_inflight_slot(Workload);
        _ ->
            ok
    end.

-spec reset_inflight_slot(atom()) -> ok.
reset_inflight_slot(Workload) ->
    try ets:insert(?INFLIGHT_TABLE, {Workload, 0}) of
        _ -> ok
    catch
        error:badarg -> ok
    end.

-spec prune_circuit_table() -> ok.
prune_circuit_table() ->
    Now = erlang:system_time(millisecond),
    MaxAgeMs = gateway_http_client:cleanup_max_age_ms(),
    ok = ensure_named_table(?CIRCUIT_TABLE),
    try
        _ = ets:foldl(
            fun({Key, CircuitState}, Acc) ->
                delete_stale_circuit(Key, CircuitState, Now, MaxAgeMs),
                Acc
            end,
            ok,
            ?CIRCUIT_TABLE
        ),
        ok
    catch
        error:badarg -> ok
    end.

-spec delete_stale_circuit({atom(), binary()}, map(), integer(), integer()) -> ok.
delete_stale_circuit(Key, CircuitState, Now, MaxAgeMs) ->
    case is_stale_circuit(CircuitState, Now, MaxAgeMs) of
        true ->
            safe_delete(?CIRCUIT_TABLE, Key),
            ok;
        false ->
            ok
    end.

-spec is_stale_circuit(map(), integer(), integer()) -> boolean().
is_stale_circuit(#{state := open, opened_at := OpenedAt}, Now, MaxAgeMs) ->
    Now - OpenedAt > MaxAgeMs;
is_stale_circuit(#{state := closed, updated_at := UpdatedAt}, Now, MaxAgeMs) ->
    Now - UpdatedAt > MaxAgeMs;
is_stale_circuit(_, _, _) ->
    false.

-spec maybe_transition_half_open({atom(), binary()}, integer(), integer(), pos_integer()) ->
    ok | {error, circuit_open}.
maybe_transition_half_open(CircuitKey, OpenedAt, Now, RecoveryTimeoutMs) ->
    case Now - OpenedAt >= RecoveryTimeoutMs of
        true ->
            insert_circuit(
                CircuitKey,
                #{
                    state => half_open,
                    results => [],
                    opened_at => OpenedAt,
                    updated_at => Now
                }
            ),
            ok;
        false ->
            {error, circuit_open}
    end.

-spec safe_lookup_circuit({atom(), binary()}) -> list().
safe_lookup_circuit(Key) ->
    try ets:lookup(?CIRCUIT_TABLE, Key) of
        Result -> Result
    catch
        error:badarg -> []
    end.

-spec safe_update_counter(atom(), term(), {pos_integer(), integer()}) ->
    {ok, integer()} | {error, term()}.
safe_update_counter(Table, Key, Op) ->
    try
        {ok, ets:update_counter(Table, Key, Op, {Key, 0})}
    catch
        error:badarg ->
            ok = gateway_http_client:ensure_started(),
            ok = ensure_named_table(Table),
            retry_update_counter(Table, Key, Op)
    end.

-spec retry_update_counter(atom(), term(), {pos_integer(), integer()}) ->
    {ok, integer()} | {error, badarg}.
retry_update_counter(Table, Key, Op) ->
    try
        {ok, ets:update_counter(Table, Key, Op, {Key, 0})}
    catch
        error:badarg -> {error, badarg}
    end.

-spec is_countable_failure(response()) -> boolean().
is_countable_failure({error, nxdomain}) -> false;
is_countable_failure({error, {failed_connect, _}}) -> false;
is_countable_failure({error, timeout}) -> false;
is_countable_failure({error, {timeout, _}}) -> false;
is_countable_failure({error, _}) -> true;
is_countable_failure({ok, StatusCode, _, _}) when StatusCode >= 500 -> true;
is_countable_failure(_) -> false.

-spec record_result({atom(), binary()}, boolean(), integer(), pos_integer()) -> ok.
record_result(CircuitKey, IsFailure, Now, FailureThreshold) ->
    Entry = {IsFailure, Now},
    case safe_lookup_circuit(CircuitKey) of
        [] ->
            record_new(CircuitKey, Entry, Now);
        [{_, #{state := half_open} = Existing}] ->
            record_half_open(CircuitKey, Existing, IsFailure, Entry, Now);
        [{_, #{state := open} = Existing}] ->
            insert_circuit(CircuitKey, Existing#{updated_at => Now}),
            ok;
        [{_, #{results := Results} = Existing}] ->
            record_closed(CircuitKey, Existing, Results, Entry, Now, FailureThreshold)
    end.

-spec record_new({atom(), binary()}, {boolean(), integer()}, integer()) -> ok.
record_new(CircuitKey, Entry, Now) ->
    insert_circuit(
        CircuitKey,
        #{
            state => closed,
            results => [Entry],
            updated_at => Now
        }
    ),
    ok.

-spec record_half_open({atom(), binary()}, map(), boolean(), {boolean(), integer()}, integer()) ->
    ok.
record_half_open(CircuitKey, _Existing, false, Entry, Now) ->
    insert_circuit(
        CircuitKey,
        #{
            state => closed,
            results => [Entry],
            updated_at => Now
        }
    ),
    ok;
record_half_open(CircuitKey, Existing, true, _Entry, Now) ->
    insert_circuit(
        CircuitKey,
        Existing#{
            state => open,
            opened_at => Now,
            updated_at => Now
        }
    ),
    ok.

-spec record_closed(
    {atom(), binary()}, map(), list(), {boolean(), integer()}, integer(), pos_integer()
) ->
    ok.
record_closed(CircuitKey, Existing, Results, Entry, Now, FailureThreshold) ->
    Cutoff = Now - ?CB_WINDOW_MS,
    Pruned = [R || {_, T} = R <- Results, T > Cutoff],
    NewResults = lists:sublist([Entry | Pruned], erlang:max(100, FailureThreshold)),
    case has_open_failure_rate(NewResults, FailureThreshold) of
        true ->
            open_circuit(CircuitKey, NewResults, Now);
        false ->
            insert_circuit(
                CircuitKey,
                Existing#{
                    results => NewResults,
                    updated_at => Now
                }
            ),
            ok
    end.

-spec has_open_failure_rate(list(), pos_integer()) -> boolean().
has_open_failure_rate(NewResults, FailureThreshold) ->
    Total = length(NewResults),
    Failures = length([1 || {true, _} <- NewResults]),
    Rate = (Failures * 100) div Total,
    Failures >= FailureThreshold andalso Rate >= ?CB_FAILURE_RATE_PCT.

-spec open_circuit({atom(), binary()}, list(), integer()) -> ok.
open_circuit(CircuitKey, NewResults, Now) ->
    Total = length(NewResults),
    Failures = length([1 || {true, _} <- NewResults]),
    Rate = (Failures * 100) div Total,
    logger:warning("Circuit breaker opening", #{
        failure_rate => Rate, failures => Failures, total => Total
    }),
    insert_circuit(
        CircuitKey,
        #{
            state => open,
            results => NewResults,
            opened_at => Now,
            updated_at => Now
        }
    ),
    ok.

-spec insert_circuit({atom(), binary()}, map()) -> ok.
insert_circuit(CircuitKey, CircuitState) ->
    try
        ets:insert(?CIRCUIT_TABLE, {CircuitKey, CircuitState}),
        ok
    catch
        error:badarg ->
            ok = gateway_http_client:ensure_started(),
            ok = ensure_named_table(?CIRCUIT_TABLE),
            retry_insert_circuit(CircuitKey, CircuitState)
    end.

-spec retry_insert_circuit({atom(), binary()}, map()) -> ok.
retry_insert_circuit(CircuitKey, CircuitState) ->
    try
        ets:insert(?CIRCUIT_TABLE, {CircuitKey, CircuitState}),
        ok
    catch
        error:badarg -> ok
    end.

-spec ensure_named_table(atom()) -> ok.
ensure_named_table(Name) ->
    case ets:whereis(Name) of
        undefined -> create_named_table(Name);
        _ -> ok
    end.

-spec create_named_table(atom()) -> ok.
create_named_table(Name) ->
    try
        _ = ets:new(Name, [
            named_table,
            public,
            set,
            {read_concurrency, true},
            {write_concurrency, true}
        ]),
        ok
    catch
        error:badarg -> ok
    end.

-spec safe_delete(atom(), term()) -> ok.
safe_delete(Table, Key) ->
    try
        ets:delete(Table, Key),
        ok
    catch
        error:badarg -> ok
    end.
