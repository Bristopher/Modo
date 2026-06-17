%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(jose_json_otp).
-typing([eqwalizer]).
-behaviour(jose_json).

-export([decode/1, encode/1]).

-spec decode(iodata()) -> term().
decode(Bin) ->
    json:decode(iolist_to_binary(Bin)).

-spec encode(json:encode_value()) -> binary().
encode(Term) ->
    iolist_to_binary(json:encode(Term)).
