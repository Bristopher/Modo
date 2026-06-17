%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(user_flags).
-typing([eqwalizer]).

-export([
    none/0,
    staff/0,
    parse/1,
    parse_optional/1,
    normalize/1,
    require/1,
    to_integer/1,
    to_binary/1,
    has/2,
    any/2,
    add/2,
    remove/2,
    is_staff/1
]).

-export_type([t/0, flag/0]).

-type t() :: bitset:t().
-type flag() :: bitset:bit().

-define(STAFF, 16#1).

-spec none() -> t().
none() ->
    bitset:none().

-spec staff() -> flag().
staff() ->
    ?STAFF.

-spec parse(term()) -> t().
parse(Value) ->
    bitset:parse(Value).

-spec parse_optional(term()) -> t() | undefined.
parse_optional(Value) ->
    bitset:parse_optional(Value).

-spec normalize(term()) -> t().
normalize(Value) ->
    bitset:normalize(Value).

-spec require(term()) -> t().
require(Value) ->
    bitset:require(Value).

-spec to_integer(t()) -> non_neg_integer().
to_integer(Flags) ->
    bitset:to_integer(Flags).

-spec to_binary(t()) -> binary().
to_binary(Flags) ->
    bitset:to_binary(Flags).

-spec has(t(), flag()) -> boolean().
has(Flags, Flag) ->
    bitset:has(Flags, Flag).

-spec any(t(), t()) -> boolean().
any(Flags, Mask) ->
    bitset:any(Flags, Mask).

-spec add(t(), t()) -> t().
add(Flags, Mask) ->
    bitset:add(Flags, Mask).

-spec remove(t(), t()) -> t().
remove(Flags, Mask) ->
    bitset:remove(Flags, Mask).

-spec is_staff(term()) -> boolean().
is_staff(Value) ->
    has(parse(Value), staff()).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

is_staff_accepts_integer_backed_flags_test() ->
    ?assertEqual(true, is_staff(1)),
    ?assertEqual(true, is_staff(<<"1">>)),
    ?assertEqual(false, is_staff(0)).

-endif.
