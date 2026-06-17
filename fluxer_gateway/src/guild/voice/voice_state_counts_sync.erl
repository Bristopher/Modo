%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(voice_state_counts_sync).
-typing([eqwalizer]).
-behaviour(gen_server).

-export([start_link/0]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2, terminate/2, code_change/3]).

-export_type([
    state/0
]).

-define(DEFAULT_SYNC_INTERVAL_MS, 30000).

-type state() :: #{sync_interval_ms := pos_integer()}.

-spec start_link() -> gen_server:start_ret().
start_link() ->
    gen_server:start_link({local, ?MODULE}, ?MODULE, [], []).

-spec init([]) -> {ok, state()}.
init([]) ->
    erlang:process_flag(fullsweep_after, 10),
    SyncIntervalMs = sync_interval_ms(),
    schedule_sync(SyncIntervalMs),
    {ok, #{sync_interval_ms => SyncIntervalMs}}.

-spec handle_call(term(), gen_server:from(), state()) -> {reply, ok, state()}.
handle_call(_Request, _From, State) ->
    {reply, ok, State}.

-spec handle_cast(term(), state()) -> {noreply, state()}.
handle_cast(_Msg, State) ->
    {noreply, State}.

-spec handle_info(term(), state()) -> {noreply, state()}.
handle_info(run_sync, State) ->
    safe_rebuild_counts(),
    schedule_sync(maps:get(sync_interval_ms, State, ?DEFAULT_SYNC_INTERVAL_MS)),
    {noreply, State};
handle_info(_Info, State) ->
    {noreply, State}.

-spec terminate(term(), state()) -> ok.
terminate(_Reason, _State) ->
    ok.

-spec code_change(term(), state(), term()) -> {ok, state()}.
code_change(_OldVsn, State, _Extra) ->
    _ = erlang:garbage_collect(),
    {ok, State}.

-spec schedule_sync(pos_integer()) -> reference().
schedule_sync(SyncIntervalMs) ->
    erlang:send_after(SyncIntervalMs, self(), run_sync).

-spec sync_interval_ms() -> pos_integer().
sync_interval_ms() ->
    normalize_sync_interval_ms(safe_config_value(voice_state_counts_sync_interval_ms)).

-spec safe_config_value(atom()) -> term().
safe_config_value(Key) ->
    try fluxer_gateway_env:get(Key) of
        Value -> Value
    catch
        throw:_Reason -> undefined;
        error:_Reason -> undefined;
        exit:_Reason -> undefined
    end.

-spec normalize_sync_interval_ms(term()) -> pos_integer().
normalize_sync_interval_ms(Value) when is_integer(Value), Value >= 1000 ->
    Value;
normalize_sync_interval_ms(_) ->
    ?DEFAULT_SYNC_INTERVAL_MS.

-spec safe_rebuild_counts() -> ok.
safe_rebuild_counts() ->
    try voice_state_counts_cache:rebuild_from_live() of
        _ -> ok
    catch
        throw:_Reason -> ok;
        error:_Reason -> ok;
        exit:_Reason -> ok
    end.
