%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_hotpatch_reconciler).
-typing([eqwalizer]).

-behaviour(gen_server).

-export([start_link/0, is_ready/0, status/0, reconcile_async/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2, terminate/2, code_change/3]).

-record(state, {
    enabled = false :: boolean(),
    build_sha = <<"dev">> :: binary(),
    public_keys = #{} :: #{binary() => binary()},
    applied_event_ids = [] :: [term()],
    applied_count = 0 :: non_neg_integer(),
    poll_interval_ms = 5000 :: pos_integer(),
    last_error = undefined :: term()
}).

-type state() :: #state{}.

-define(SERVER, ?MODULE).
-define(POLL, poll).
-define(STARTUP_RECONCILE_RETRY_MS, 1000).

-spec start_link() -> gen_server:start_ret().
start_link() ->
    gen_server:start_link({local, ?SERVER}, ?MODULE, [], []).

-spec is_ready() -> boolean().
is_ready() ->
    gateway_hotpatch_runtime:is_ready().

-spec status() -> map().
status() ->
    gateway_hotpatch_runtime:status().

-spec reconcile_async() -> ok.
reconcile_async() ->
    gen_server:cast(?SERVER, reconcile).

-spec init([]) -> {ok, state()} | {stop, term()}.
init([]) ->
    case gateway_hotpatch_runtime:is_enabled() of
        false -> init_disabled();
        true -> init_enabled()
    end.

-spec handle_call(term(), {pid(), term()}, state()) -> {reply, term(), state()}.
handle_call(status, _From, State) ->
    {reply, state_status(State, gateway_hotpatch_runtime:is_ready()), State};
handle_call(_Request, _From, State) ->
    {reply, {error, unsupported_call}, State}.

-spec handle_cast(term(), state()) -> {noreply, state()}.
handle_cast(reconcile, State) ->
    {noreply, reconcile_and_publish(State)};
handle_cast(_Request, State) ->
    {noreply, State}.

-spec handle_info(term(), state()) -> {noreply, state()}.
handle_info(?POLL, #state{enabled = true} = State) ->
    NewState = reconcile_and_publish(State),
    schedule_poll(NewState),
    {noreply, NewState};
handle_info(_Info, State) ->
    {noreply, State}.

-spec terminate(term(), state()) -> ok.
terminate(_Reason, _State) ->
    ok.

-spec code_change(term(), state(), term()) -> {ok, state()}.
code_change(_OldVsn, State, _Extra) ->
    {ok, State}.

-spec init_disabled() -> {ok, state()}.
init_disabled() ->
    BuildSha = gateway_hotpatch_runtime:build_sha(),
    State = #state{enabled = false, build_sha = BuildSha},
    gateway_hotpatch_runtime:put_ready(true),
    publish_status(State, true),
    {ok, State}.

-spec init_enabled() -> {ok, state()} | {stop, term()}.
init_enabled() ->
    gateway_hotpatch_runtime:put_ready(false),
    BuildSha = gateway_hotpatch_runtime:build_sha(),
    PollIntervalMs = runtime_pos_integer(hotpatch_poll_interval_ms, 5000),
    PublicKeyConfig = gateway_hotpatch_runtime:get(hotpatch_public_keys, undefined),
    case gateway_hotpatch_bundle:parse_public_keys(PublicKeyConfig) of
        {ok, PublicKeys} when map_size(PublicKeys) > 0 ->
            State0 = #state{
                enabled = true,
                build_sha = BuildSha,
                public_keys = PublicKeys,
                poll_interval_ms = max(1000, PollIntervalMs)
            },
            publish_status(State0, false),
            init_enabled_connected(State0);
        {ok, _Empty} ->
            init_enabled_config_error(BuildSha, PollIntervalMs, missing_hotpatch_public_keys);
        {error, Reason} ->
            init_enabled_config_error(
                BuildSha, PollIntervalMs, {invalid_hotpatch_public_keys, Reason}
            )
    end.

-spec init_enabled_config_error(binary(), pos_integer(), term()) -> {ok, state()}.
init_enabled_config_error(BuildSha, PollIntervalMs, Reason) ->
    State = #state{
        enabled = true,
        build_sha = BuildSha,
        poll_interval_ms = max(1000, PollIntervalMs),
        last_error = Reason
    },
    gateway_hotpatch_runtime:put_ready(false),
    publish_status(State, false),
    logger:error("Gateway hotpatch configuration invalid: ~0tp", [Reason]),
    schedule_poll(State),
    {ok, State}.

-spec init_enabled_connected(state()) -> {ok, state()}.
init_enabled_connected(State0) ->
    finish_enabled_startup(State0).

-spec finish_enabled_startup(state()) -> {ok, state()}.
finish_enabled_startup(State0) ->
    TimeoutMs = runtime_pos_integer(hotpatch_startup_sync_timeout_ms, 30000),
    DeadlineMs = erlang:monotonic_time(millisecond) + TimeoutMs,
    finish_startup_reconcile(State0, startup_reconcile(State0, DeadlineMs)).

-spec finish_startup_reconcile(state(), {ok, state()} | {error, term()}) -> {ok, state()}.
finish_startup_reconcile(_State0, {ok, State}) ->
    gateway_hotpatch_runtime:put_ready(true),
    publish_status(State, true),
    schedule_poll(State),
    {ok, State};
finish_startup_reconcile(State0, {error, Reason}) ->
    Error = {hotpatch_startup_sync_failed, Reason},
    State = State0#state{last_error = Error},
    gateway_hotpatch_runtime:put_ready(false),
    publish_status(State, false),
    logger:error("Gateway hotpatch startup sync failed; keeping node unready: ~0tp", [Error]),
    schedule_poll(State),
    {ok, State}.

-spec startup_reconcile(state(), integer()) -> {ok, state()} | {error, term()}.
startup_reconcile(State, DeadlineMs) ->
    case reconcile_once(State) of
        {ok, NewState} ->
            {ok, NewState};
        {error, Reason} ->
            maybe_retry_startup_reconcile(State, DeadlineMs, Reason)
    end.

-spec maybe_retry_startup_reconcile(state(), integer(), term()) ->
    {ok, state()} | {error, term()}.
maybe_retry_startup_reconcile(State, DeadlineMs, Reason) ->
    case erlang:monotonic_time(millisecond) >= DeadlineMs of
        true ->
            {error, Reason};
        false ->
            retry_startup_reconcile(State#state{last_error = Reason}, DeadlineMs, Reason)
    end.

-spec retry_startup_reconcile(state(), integer(), term()) -> {ok, state()} | {error, term()}.
retry_startup_reconcile(State, DeadlineMs, Reason) ->
    case gateway_retry_timer:wait_until(?STARTUP_RECONCILE_RETRY_MS, DeadlineMs) of
        ok -> startup_reconcile(State, DeadlineMs);
        expired -> {error, Reason};
        {error, _InvalidDelay} -> {error, Reason}
    end.

-spec reconcile_and_publish(state()) -> state().
reconcile_and_publish(#state{enabled = false} = State) ->
    gateway_hotpatch_runtime:put_ready(true),
    publish_status(State, true),
    State;
reconcile_and_publish(State) ->
    case reconcile_once(State) of
        {ok, NewState} ->
            gateway_hotpatch_runtime:put_ready(true),
            publish_status(NewState, true),
            NewState;
        {error, Reason} ->
            publish_reconcile_error(State, Reason)
    end.

-spec reconcile_once(state()) -> {ok, state()} | {error, term()}.
reconcile_once(#state{build_sha = BuildSha} = State) ->
    case gateway_hotpatch_store:connect() of
        ok -> fetch_and_apply_events(BuildSha, State);
        {error, Reason} -> {error, {store_connect_failed, Reason}}
    end.

-spec fetch_and_apply_events(binary(), state()) -> {ok, state()} | {error, term()}.
fetch_and_apply_events(BuildSha, State) ->
    case gateway_hotpatch_store:fetch_events(BuildSha) of
        {ok, Events} -> apply_events(Events, State);
        {error, Reason} -> {error, {fetch_events_failed, Reason}}
    end.

-spec publish_reconcile_error(state(), term()) -> state().
publish_reconcile_error(State, {fetch_events_failed, _Reason} = Error) ->
    ErrorState = State#state{last_error = Error},
    Ready = gateway_hotpatch_runtime:is_ready(),
    gateway_hotpatch_runtime:put_ready(Ready),
    publish_status(ErrorState, Ready),
    logger:warning("Gateway hotpatch fetch failed; keeping current readiness: ~0tp", [Error]),
    ErrorState;
publish_reconcile_error(State, Reason) ->
    ErrorState = State#state{last_error = Reason},
    gateway_hotpatch_runtime:put_ready(false),
    publish_status(ErrorState, false),
    logger:error("Gateway hotpatch reconciliation failed: ~0tp", [Reason]),
    ErrorState.

-spec apply_events([map()], state()) -> {ok, state()} | {error, term()}.
apply_events([], State) ->
    {ok, State#state{last_error = undefined}};
apply_events([Event | Rest], State) ->
    EventId = maps:get(event_id, Event, undefined),
    case lists:member(EventId, State#state.applied_event_ids) of
        true ->
            apply_events(Rest, State);
        false ->
            apply_new_event(EventId, Event, Rest, State)
    end.

-spec apply_new_event(term(), map(), [map()], state()) -> {ok, state()} | {error, term()}.
apply_new_event(EventId, Event, Rest, State) ->
    case apply_event(Event, State) of
        {ok, NewState} -> apply_events(Rest, NewState);
        {error, Reason} -> {error, {event_apply_failed, EventId, Reason}}
    end.

-spec apply_event(map(), state()) -> {ok, state()} | {error, term()}.
apply_event(Event, #state{public_keys = PublicKeys, build_sha = BuildSha} = State) ->
    EventId = maps:get(event_id, Event, undefined),
    BundleHash = maps:get(bundle_sha256, Event, <<>>),
    _ = code:ensure_loaded(gateway_hotpatch_loader),
    case gateway_hotpatch_bundle:decode_signed_event(Event, PublicKeys) of
        {ok, Bundle} ->
            apply_decoded_event(BuildSha, EventId, BundleHash, Bundle, State);
        {error, Reason} ->
            audit_event(BuildSha, EventId, #{bundle_sha256 => BundleHash}, {error, Reason}),
            {error, Reason}
    end.

-spec apply_decoded_event(binary(), term(), binary(), map(), state()) ->
    {ok, state()} | {error, term()}.
apply_decoded_event(BuildSha, EventId, BundleHash, Bundle, State) ->
    Summary0 = #{bundle_sha256 => BundleHash},
    case gateway_hotpatch_loader:apply_bundle(Bundle) of
        {ok, Summary} ->
            handle_applied_event(BuildSha, EventId, Summary0, Summary, State);
        {error, Reason} ->
            audit_event(BuildSha, EventId, Summary0, {error, Reason}),
            {error, Reason}
    end.

-spec handle_applied_event(binary(), term(), map(), map(), state()) -> {ok, state()}.
handle_applied_event(BuildSha, EventId, Summary0, Summary, State) ->
    AuditSummary = maps:merge(Summary0, Summary),
    audit_event(BuildSha, EventId, AuditSummary, ok),
    logger:notice("Applied gateway hotpatch event ~0tp summary=~0tp", [EventId, AuditSummary]),
    {ok, State#state{
        applied_event_ids = [EventId | State#state.applied_event_ids],
        applied_count = State#state.applied_count + 1,
        last_error = undefined
    }}.

-spec audit_event(binary(), term(), map(), ok | {error, term()}) -> ok.
audit_event(BuildSha, EventId, Summary, Result) when is_binary(EventId) ->
    case
        gateway_hotpatch_store:audit_applied(
            BuildSha, gateway_hotpatch_runtime:node_name(), EventId, Summary, Result
        )
    of
        ok ->
            ok;
        {error, Reason} ->
            logger:warning("Gateway hotpatch audit write failed: ~0tp", [Reason]),
            ok
    end;
audit_event(_BuildSha, _EventId, _Summary, _Result) ->
    ok.

-spec publish_status(state(), boolean()) -> ok.
publish_status(State, Ready) ->
    gateway_hotpatch_runtime:put_status(state_status(State, Ready)).

-spec state_status(state(), boolean()) -> map().
state_status(State, Ready) ->
    #{
        enabled => State#state.enabled,
        ready => Ready,
        build_sha => State#state.build_sha,
        applied_count => State#state.applied_count,
        applied_event_count => length(State#state.applied_event_ids),
        last_error => State#state.last_error
    }.

-spec schedule_poll(state()) -> ok.
schedule_poll(#state{poll_interval_ms = PollIntervalMs}) ->
    erlang:send_after(PollIntervalMs, self(), ?POLL),
    ok.

-spec runtime_pos_integer(atom(), pos_integer()) -> pos_integer().
runtime_pos_integer(Key, Default) ->
    normalize_pos_integer(gateway_hotpatch_runtime:get(Key, Default), Default).

-spec normalize_pos_integer(term(), pos_integer()) -> pos_integer().
normalize_pos_integer(Value, _Default) when is_integer(Value), Value > 0 ->
    Value;
normalize_pos_integer(_Value, Default) ->
    Default.
