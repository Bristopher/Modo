%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(presence_broadcast_subscriptions_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

gdm_subscription_add_remove_test() ->
    maybe_start_presence_bus(),
    maybe_start_presence_cache(),
    BaseState = #{
        user_id => 1,
        is_bot => false,
        sessions => #{},
        user_data => #{},
        subscriptions => #{},
        friends => #{},
        group_dm_recipients => #{}
    },
    State1 = presence_broadcast_subscriptions:sync_group_dm_subscriptions(
        #{1 => [10]}, BaseState
    ),
    Subscriptions1 = maps:get(subscriptions, State1),
    Entry1 = maps:get(10, Subscriptions1),
    ?assertEqual(true, maps:get(1, maps:get(gdm_channels, Entry1, #{}))),
    State2 = presence_broadcast_subscriptions:sync_group_dm_subscriptions(#{}, State1),
    ?assertEqual(false, maps:is_key(10, maps:get(subscriptions, State2, #{}))).

one_to_one_dm_recipients_get_subscriptions_test() ->
    maybe_start_presence_bus(),
    maybe_start_presence_cache(),
    SessionState = #{
        user_id => 1,
        channels => #{
            100 => #{
                <<"id">> => <<"100">>,
                <<"type">> => 1,
                <<"recipients">> => [#{<<"id">> => <<"2">>, <<"username">> => <<"dm-user">>}]
            },
            200 => #{
                <<"id">> => <<"200">>,
                <<"type">> => 3,
                <<"recipients">> => [
                    #{<<"id">> => <<"3">>, <<"username">> => <<"gdm-a">>},
                    #{<<"id">> => <<"4">>, <<"username">> => <<"gdm-b">>}
                ]
            }
        }
    },
    DmRecipients = presence_targets:dm_recipients_from_state(SessionState),
    BaseState = #{
        user_id => 1,
        is_bot => false,
        sessions => #{},
        user_data => #{},
        subscriptions => #{},
        friends => #{},
        group_dm_recipients => #{}
    },
    State1 = presence_broadcast_subscriptions:sync_group_dm_subscriptions(
        DmRecipients, BaseState
    ),
    Subscriptions = maps:get(subscriptions, State1),
    ?assertEqual(true, maps:get(100, maps:get(gdm_channels, maps:get(2, Subscriptions)))),
    ?assertEqual(true, maps:get(200, maps:get(gdm_channels, maps:get(3, Subscriptions)))),
    ?assertEqual(true, maps:get(200, maps:get(gdm_channels, maps:get(4, Subscriptions)))),
    StateAfterDmClose = presence_broadcast_subscriptions:sync_group_dm_subscriptions(
        maps:remove(100, DmRecipients), State1
    ),
    SubsAfterClose = maps:get(subscriptions, StateAfterDmClose),
    ?assertEqual(false, maps:is_key(2, SubsAfterClose)),
    ?assertEqual(true, maps:is_key(3, SubsAfterClose)).

initial_subscriptions_include_one_to_one_dm_recipients_test() ->
    maybe_start_presence_bus(),
    SessionState = #{
        user_id => 1,
        channels => #{
            100 => #{
                <<"id">> => <<"100">>,
                <<"type">> => 1,
                <<"recipients">> => [#{<<"id">> => <<"2">>, <<"username">> => <<"dm-user">>}]
            }
        }
    },
    State = #{
        user_id => 1,
        is_bot => false,
        sessions => #{},
        user_data => #{},
        subscriptions => #{},
        friends => #{},
        group_dm_recipients => presence_targets:dm_recipients_from_state(SessionState)
    },
    State1 = presence_broadcast_subscriptions:ensure_initial_global_subscriptions(State),
    Subscriptions = maps:get(subscriptions, State1),
    ?assertEqual(true, maps:get(100, maps:get(gdm_channels, maps:get(2, Subscriptions)))).

map_from_ids_test() ->
    ?assertEqual(#{}, presence_broadcast_subscriptions:map_from_ids([])),
    ?assertEqual(
        #{1 => true, 2 => true}, presence_broadcast_subscriptions:map_from_ids([1, 2])
    ).

maybe_start_presence_bus() ->
    case whereis(presence_bus) of
        undefined -> start_presence_bus();
        _ -> ok
    end.

start_presence_bus() ->
    case presence_bus:start_link() of
        {ok, _Pid} -> ok;
        {error, {already_started, _Pid}} -> ok;
        Other -> Other
    end.

maybe_start_presence_cache() ->
    case whereis(presence_cache) of
        undefined -> start_presence_cache();
        _ -> ok
    end.

start_presence_cache() ->
    case presence_cache:start_link() of
        {ok, _Pid} -> ok;
        {error, {already_started, _Pid}} -> ok;
        Other -> Other
    end.
