%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_timings_tests).

-include_lib("eunit/include/eunit.hrl").

finalize_includes_gateway_runtime_metadata_test() ->
    with_runtime_config(#{gateway_role => sessions}, fun() ->
        Timings = gateway_timings:finalize(gateway_timings:new()),
        ?assertEqual(<<"microseconds">>, maps:get(<<"unit">>, Timings)),
        ?assert(is_integer(maps:get(<<"total_us">>, Timings))),
        ?assert(is_binary(maps:get(<<"pod_name">>, Timings))),
        ?assertEqual([], maps:get(<<"trace">>, Timings)),
        ?assertNot(maps:is_key(<<"steps">>, Timings)),
        ?assertNot(maps:is_key(<<"nodes">>, Timings)),
        ?assertNot(maps:is_key(<<"node_name">>, Timings)),
        ?assertNot(maps:is_key(<<"erlang_node_name">>, Timings)),
        ?assertNot(maps:is_key(<<"role">>, Timings))
    end).

finalize_uses_trace_total_when_recorder_crosses_nodes_test() ->
    OtherNode = 'fluxer_gateway@10.0.0.10',
    with_members_and_pods(
        #{websocket => [OtherNode]},
        #{OtherNode => <<"gateway-websocket-1">>},
        fun() ->
            Recorder = #{
                started_at_us => gateway_timings:start() - 60000000,
                started_node => OtherNode,
                steps => #{},
                nodes => [],
                trace => [
                    #{<<"name">> => <<"second/0">>, <<"duration_us">> => 200},
                    #{<<"name">> => <<"first/0">>, <<"duration_us">> => 100}
                ],
                pod_name => <<"gateway-websocket-1">>,
                node_name => <<"gateway-websocket-1">>,
                erlang_node_name => atom_to_binary(OtherNode, utf8)
            },
            Timings = gateway_timings:finalize(Recorder),
            ?assertEqual(300, maps:get(<<"total_us">>, Timings)),
            ?assertEqual(<<"gateway-websocket-1">>, maps:get(<<"pod_name">>, Timings)),
            ?assertNot(maps:is_key(<<"erlang_node_name">>, Timings)),
            ?assertNot(maps:is_key(<<"role">>, Timings))
        end
    ).

finalize_omits_legacy_node_metadata_for_older_recorder_test() ->
    OtherNode = 'fluxer_gateway@10.0.0.11',
    with_members_by_role(#{websocket => [OtherNode]}, fun() ->
        Recorder = #{
            started_at_us => gateway_timings:start() - 60000000,
            steps => #{},
            nodes => [],
            trace => [#{<<"name">> => <<"first/0">>, <<"duration_us">> => 100}],
            pod_name => <<"gateway-websocket-2">>,
            node_name => <<"gateway-websocket-2">>,
            erlang_node_name => atom_to_binary(OtherNode, utf8)
        },
        Timings = gateway_timings:finalize(Recorder),
        ?assertEqual(100, maps:get(<<"total_us">>, Timings)),
        ?assertNot(maps:is_key(<<"node_name">>, Timings)),
        ?assertNot(maps:is_key(<<"erlang_node_name">>, Timings)),
        ?assertNot(maps:is_key(<<"role">>, Timings))
    end).

record_function_emits_ordered_trace_without_steps_test() ->
    T0 = gateway_timings:new(),
    T1 = gateway_timings:record_function(
        first, <<"module:first/0">>, gateway_timings:start() - 10, T0
    ),
    T2 = gateway_timings:record_function(
        second, <<"module:second/0">>, gateway_timings:start() - 5, T1
    ),
    Timings = gateway_timings:finalize(T2),
    [First, Second] = maps:get(<<"trace">>, Timings),
    ?assertEqual(<<"module:first/0">>, maps:get(<<"name">>, First)),
    ?assertEqual(<<"module:second/0">>, maps:get(<<"name">>, Second)),
    ?assertNot(maps:is_key(<<"steps">>, Timings)).

record_function_emits_nested_remote_trace_test() ->
    NodeName = 'fluxer_gateway@10.0.0.9',
    with_members_and_pods(
        #{guilds => [NodeName]},
        #{NodeName => <<"gateway-guilds-0">>},
        fun() -> assert_nested_remote_trace(NodeName) end
    ).

api_remote_from_session_result_uses_api_runtime_metadata_test() ->
    Remote = require_map(
        gateway_timings:api_remote_from_session_result(
            {ok, #{
                <<"_timings">> => #{
                    <<"role">> => <<"api">>,
                    <<"pod_name">> => <<"api-pod-1">>,
                    <<"node_name">> => <<"worker-node-7">>,
                    <<"erlang_node_name">> => <<"unused">>
                }
            }}
        )
    ),
    ?assertEqual(<<"api">>, maps:get(<<"operation">>, Remote)),
    ?assertEqual(<<"api-pod-1">>, maps:get(<<"pod_name">>, Remote)),
    ?assertNot(maps:is_key(<<"role">>, Remote)),
    ?assertNot(maps:is_key(<<"node_name">>, Remote)),
    ?assertNot(maps:is_key(<<"erlang_node_name">>, Remote)).

remote_node_uses_pod_from_cluster_metadata_test() ->
    NodeName = 'fluxer_gateway@10.0.0.9',
    with_members_and_pods(
        #{guilds => [NodeName]},
        #{NodeName => <<"gateway-guilds-0">>},
        fun() ->
            Remote = require_map(gateway_timings:remote_node(guild_manager, NodeName)),
            ?assertEqual(<<"guild_manager">>, maps:get(<<"operation">>, Remote)),
            ?assertEqual(<<"gateway-guilds-0">>, maps:get(<<"pod_name">>, Remote)),
            ?assertNot(maps:is_key(<<"role">>, Remote)),
            ?assertNot(maps:is_key(<<"erlang_node_name">>, Remote))
        end
    ).

-spec assert_nested_remote_trace(node()) -> ok.
assert_nested_remote_trace(NodeName) ->
    Remote = gateway_timings:remote_node(guild_manager, NodeName),
    Child = gateway_timings:span(
        <<"guild_manager:start_or_lookup/2">>,
        gateway_timings:start() - 7,
        #{remote => Remote}
    ),
    Timings = gateway_timings:finalize(
        gateway_timings:record_function(
            parent,
            <<"session_connection_guild_resolve:do_remote_guild_connect/2">>,
            gateway_timings:start() - 10,
            #{children => [Child]},
            gateway_timings:new()
        )
    ),
    [Parent] = maps:get(<<"trace">>, Timings),
    [Nested] = maps:get(<<"children">>, Parent),
    RemoteNode = maps:get(<<"remote">>, Nested),
    ?assertEqual(<<"guild_manager:start_or_lookup/2">>, maps:get(<<"name">>, Nested)),
    ?assertEqual(<<"guild_manager">>, maps:get(<<"operation">>, RemoteNode)),
    ?assertEqual(<<"gateway-guilds-0">>, maps:get(<<"pod_name">>, RemoteNode)),
    ?assertNot(maps:is_key(<<"role">>, RemoteNode)),
    ?assertNot(maps:is_key(<<"erlang_node_name">>, RemoteNode)),
    ?assertNot(maps:is_key(<<"nodes">>, Timings)).

-spec require_map(term()) -> map().
require_map(Value) when is_map(Value) ->
    Value;
require_map(Value) ->
    error({expected_map, Value}).

with_runtime_config(Config, Fun) ->
    with_persistent_term({fluxer_gateway, runtime_config}, Config, Fun).

with_members_by_role(RoleMembers, Fun) ->
    with_persistent_term({gateway_cluster_membership, members_by_role}, RoleMembers, Fun).

with_node_pod_names(NodePodNames, Fun) ->
    with_persistent_term({gateway_cluster_membership, node_pod_names}, NodePodNames, Fun).

with_members_and_pods(RoleMembers, NodePodNames, Fun) ->
    with_members_by_role(
        RoleMembers,
        fun() -> with_node_pod_names(NodePodNames, Fun) end
    ).

with_persistent_term(Key, Value, Fun) ->
    Previous = persistent_term:get(Key, undefined),
    persistent_term:put(Key, Value),
    try
        Fun()
    after
        case Previous of
            undefined -> persistent_term:erase(Key);
            _ -> persistent_term:put(Key, Previous)
        end
    end.
