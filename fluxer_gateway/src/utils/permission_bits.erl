%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(permission_bits).
-typing([eqwalizer]).

-export([
    none/0,
    parse/1,
    parse_optional/1,
    parse_maybe/1,
    normalize/1,
    require/1,
    to_integer/1,
    to_binary/1,
    has/2,
    any/2,
    add/2,
    remove/2,
    apply_allow_deny/3
]).

-export_type([t/0, bit/0]).

-type t() :: bitset:t().
-type bit() :: bitset:bit().

-spec none() -> t().
none() ->
    bitset:none().

-spec parse(term()) -> t().
parse(Value) ->
    bitset:parse(Value).

-spec parse_optional(term()) -> t() | undefined.
parse_optional(Value) ->
    bitset:parse_optional(Value).

-spec parse_maybe(term()) -> t() | undefined.
parse_maybe(Value) ->
    bitset:parse_maybe(Value).

-spec normalize(term()) -> t().
normalize(Value) ->
    bitset:normalize(Value).

-spec require(term()) -> t().
require(Value) ->
    bitset:require(Value).

-spec to_integer(t()) -> non_neg_integer().
to_integer(Bits) ->
    bitset:to_integer(Bits).

-spec to_binary(t()) -> binary().
to_binary(Bits) ->
    bitset:to_binary(Bits).

-spec has(t(), bit()) -> boolean().
has(Bits, Bit) ->
    bitset:has(Bits, Bit).

-spec any(t(), t()) -> boolean().
any(Bits, Mask) ->
    bitset:any(Bits, Mask).

-spec add(t(), t()) -> t().
add(Bits, Mask) ->
    bitset:add(Bits, Mask).

-spec remove(t(), t()) -> t().
remove(Bits, Mask) ->
    bitset:remove(Bits, Mask).

-spec apply_allow_deny(t(), t(), t()) -> t().
apply_allow_deny(Bits, Allow, Deny) ->
    bitset:apply_allow_deny(Bits, Allow, Deny).
