%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_event_pause).
-typing([eqwalizer]).
-behaviour(gen_server).

-export([
    start_link/0,
    enable/0,
    freeze/0,
    disable/0,
    is_paused/0,
    is_frozen/0
]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2, terminate/2, code_change/3]).

-define(PAUSED_KEY, {gateway_event_pause, paused}).
-define(FROZEN_KEY, {gateway_event_pause, frozen}).

-type state() :: #{}.

-spec start_link() -> {ok, pid()} | {error, term()}.
start_link() ->
    case gen_server:start_link({local, ?MODULE}, ?MODULE, [], []) of
        {ok, Pid} -> {ok, Pid};
        {error, Reason} -> {error, Reason};
        ignore -> {error, ignore}
    end.

-spec enable() -> ok.
enable() ->
    gen_server:call(?MODULE, enable, 5000).

-spec freeze() -> ok.
freeze() ->
    gen_server:call(?MODULE, freeze, 5000).

-spec disable() -> ok.
disable() ->
    gen_server:call(?MODULE, disable, 5000).

-spec is_paused() -> boolean().
is_paused() ->
    persistent_term:get(?PAUSED_KEY, false).

-spec is_frozen() -> boolean().
is_frozen() ->
    persistent_term:get(?FROZEN_KEY, false).

-spec init([]) -> {ok, state()}.
init([]) ->
    erlang:process_flag(fullsweep_after, 50),
    persistent_term:put(?PAUSED_KEY, false),
    persistent_term:put(?FROZEN_KEY, false),
    {ok, #{}}.

-spec handle_call(term(), gen_server:from(), state()) -> {reply, term(), state()}.
handle_call(enable, _From, State) ->
    persistent_term:put(?PAUSED_KEY, true),
    logger:info("Gateway event pause enabled: mutations paused"),
    {reply, ok, State};
handle_call(freeze, _From, State) ->
    persistent_term:put(?FROZEN_KEY, true),
    logger:info("Gateway event pause frozen: event dispatch stopped"),
    {reply, ok, State};
handle_call(disable, _From, State) ->
    persistent_term:put(?PAUSED_KEY, false),
    persistent_term:put(?FROZEN_KEY, false),
    logger:info("Gateway event pause disabled: normal operation resumed"),
    {reply, ok, State};
handle_call(_Request, _From, State) ->
    {reply, ok, State}.

-spec handle_cast(term(), state()) -> {noreply, state()}.
handle_cast(_Msg, State) ->
    {noreply, State}.

-spec handle_info(term(), state()) -> {noreply, state()}.
handle_info(_Info, State) ->
    {noreply, State}.

-spec terminate(term(), state()) -> ok.
terminate(_Reason, _State) ->
    ok.

-spec code_change(term(), state(), term()) -> {ok, state()}.
code_change(_OldVsn, State, _Extra) ->
    erlang:garbage_collect(),
    {ok, State}.

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

pause_lifecycle_test() ->
    ?assertEqual(false, persistent_term:get(?PAUSED_KEY, false)),
    ?assertEqual(false, persistent_term:get(?FROZEN_KEY, false)),
    persistent_term:put(?PAUSED_KEY, true),
    ?assertEqual(true, is_paused()),
    ?assertEqual(false, is_frozen()),
    persistent_term:put(?FROZEN_KEY, true),
    ?assertEqual(true, is_frozen()),
    persistent_term:put(?PAUSED_KEY, false),
    persistent_term:put(?FROZEN_KEY, false),
    ?assertEqual(false, is_paused()),
    ?assertEqual(false, is_frozen()).

-endif.
