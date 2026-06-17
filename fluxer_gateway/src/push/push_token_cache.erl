%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_token_cache).
-typing([eqwalizer]).

-export([init/0, get/1, put/3]).

-define(TABLE, push_bearer_tokens).

-spec init() -> ok.
init() ->
    case ets:whereis(?TABLE) of
        undefined -> create_table();
        _ -> ok
    end.

-spec create_table() -> ok.
create_table() ->
    try
        _ = ets:new(?TABLE, [
            named_table, public, set, {read_concurrency, true}, {write_concurrency, true}
        ]),
        ok
    catch
        error:badarg -> ok
    end.

-spec get(term()) -> {ok, binary(), integer()} | undefined.
get(Key) ->
    try ets:lookup(?TABLE, Key) of
        [{_, Token, ExpiresAt}] when is_binary(Token), is_integer(ExpiresAt) ->
            {ok, Token, ExpiresAt};
        _ ->
            undefined
    catch
        error:badarg -> undefined
    end.

-spec put(term(), binary(), integer()) -> ok.
put(Key, Token, ExpiresAt) when is_binary(Token), is_integer(ExpiresAt) ->
    ok = init(),
    try ets:insert(?TABLE, {Key, Token, ExpiresAt}) of
        _ -> ok
    catch
        error:badarg -> ok
    end.
