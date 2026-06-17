%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(presence_broadcast_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

is_last_published_visible_test() ->
    ?assertEqual(false, presence_broadcast:is_last_published_visible(undefined)),
    ?assertEqual(true, presence_broadcast:is_last_published_visible(#{status => <<"online">>})),
    ?assertEqual(true, presence_broadcast:is_last_published_visible(#{status => <<"idle">>})),
    ?assertEqual(true, presence_broadcast:is_last_published_visible(#{status => <<"dnd">>})),
    ?assertEqual(
        false, presence_broadcast:is_last_published_visible(#{status => <<"offline">>})
    ),
    ?assertEqual(
        false, presence_broadcast:is_last_published_visible(#{status => <<"invisible">>})
    ).

cache_if_visible_skips_offline_test() ->
    ?assertEqual(ok, presence_broadcast:cache_if_visible(1, #{<<"status">> => <<"offline">>})),
    ?assertEqual(
        ok, presence_broadcast:cache_if_visible(1, #{<<"status">> => <<"invisible">>})
    ),
    ?assertEqual(ok, presence_broadcast:cache_if_visible(<<"bad">>, #{})).

map_from_ids_test() ->
    ?assertEqual(#{1 => true, 2 => true}, presence_broadcast:map_from_ids([1, 2])),
    ?assertEqual(#{}, presence_broadcast:map_from_ids([])).
