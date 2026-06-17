%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_node_metadata_tests).

-include_lib("eunit/include/eunit.hrl").

pod_name_for_node_uses_cached_remote_pod_name_test() ->
    Node = 'fluxer_gateway@10.0.0.30',
    with_node_pod_names(#{Node => <<"gateway-presence-0">>}, fun() ->
        ?assertEqual(<<"gateway-presence-0">>, gateway_node_metadata:pod_name_for_node(Node))
    end).

cache_node_pod_name_updates_cached_remote_pod_name_test() ->
    Node = 'fluxer_gateway@10.0.0.31',
    with_node_pod_names(#{}, fun() ->
        ?assertEqual(
            ok, gateway_node_metadata:cache_node_pod_name(Node, <<"gateway-guilds-0">>)
        ),
        ?assertEqual(<<"gateway-guilds-0">>, gateway_node_metadata:pod_name_for_node(Node))
    end).

refresh_node_pod_names_caches_local_pod_name_test() ->
    with_env("POD_NAME", "gateway-local-0", fun() ->
        Result = gateway_node_metadata:refresh_node_pod_names([node()]),
        ?assertEqual(<<"gateway-local-0">>, maps:get(node(), Result)),
        ?assertEqual(<<"gateway-local-0">>, gateway_node_metadata:pod_name_for_node(node()))
    end).

with_node_pod_names(NodePodNames, Fun) ->
    with_persistent_term({gateway_cluster_membership, node_pod_names}, NodePodNames, Fun).

with_persistent_term(Key, Value, Fun) ->
    Previous = persistent_term:get(Key, undefined),
    persistent_term:put(Key, Value),
    try
        Fun()
    after
        restore_persistent_term(Key, Previous)
    end.

restore_persistent_term(Key, undefined) ->
    persistent_term:erase(Key);
restore_persistent_term(Key, Previous) ->
    persistent_term:put(Key, Previous).

with_env(Name, Value, Fun) ->
    Previous = os:getenv(Name),
    os:putenv(Name, Value),
    try
        Fun()
    after
        restore_env(Name, Previous)
    end.

restore_env(Name, false) ->
    os:unsetenv(Name);
restore_env(Name, Value) ->
    os:putenv(Name, Value).
