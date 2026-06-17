%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(term_detach).
-typing([eqwalizer]).

-export([detach/1]).

-spec detach(term()) -> term().
detach(Bin) when is_binary(Bin) ->
    binary:copy(Bin);
detach(List) when is_list(List) ->
    [detach(Item) || Item <- List];
detach(Map) when is_map(Map) ->
    maps:fold(
        fun(Key, Value, Acc) ->
            Acc#{detach(Key) => detach(Value)}
        end,
        #{},
        Map
    );
detach(Tuple) when is_tuple(Tuple) ->
    list_to_tuple([detach(Item) || Item <- tuple_to_list(Tuple)]);
detach(Term) ->
    Term.
