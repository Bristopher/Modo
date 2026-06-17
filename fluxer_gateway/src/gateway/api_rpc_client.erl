%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(api_rpc_client).
-typing([eqwalizer]).

-export([call/1, call_with_retry/2]).
-export_type([retry_config/0]).

-type retry_config() :: {
    MaxAttempts :: pos_integer(),
    BaseMs :: pos_integer(),
    MaxMs :: pos_integer(),
    JitterMs :: non_neg_integer()
}.

-spec call(map()) -> {ok, map()} | {error, term()}.
call(Request) ->
    rpc_client:call(Request).

-spec call_with_retry(map(), retry_config()) -> {ok, map()} | {error, term()}.
call_with_retry(Request, RetryConfig) ->
    rpc_client:call_with_retry(Request, RetryConfig).
