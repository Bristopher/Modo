%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_timings_payload_tests).

-include_lib("eunit/include/eunit.hrl").

finalize_sanitizes_legacy_runtime_metadata_test() ->
    RemoteNode = 'fluxer_gateway@10.0.0.91',
    with_node_pod_names(#{RemoteNode => <<"gateway-presence-0">>}, fun() ->
        assert_legacy_runtime_metadata_sanitized(RemoteNode)
    end).

finalize_omits_legacy_child_remote_metadata_test() ->
    RemoteNode = 'fluxer_gateway@10.0.0.92',
    with_node_pod_names(#{RemoteNode => <<"gateway-guilds-0">>}, fun() ->
        assert_legacy_child_remote_metadata_sanitized(RemoteNode)
    end).

sanitize_message_rewrites_ready_timings_without_reordering_trace_test() ->
    RemoteNode = 'fluxer_gateway@10.0.0.93',
    with_node_pod_names(#{RemoteNode => <<"gateway-presence-1">>}, fun() ->
        Message = ready_message_with_finalized_timings(RemoteNode),
        Sanitized = gateway_timings_payload:sanitize_message(Message),
        #{<<"d">> := Data} = Sanitized,
        Timings = require_map(maps:get(<<"_timings_gw">>, Data)),
        ?assertEqual(15, maps:get(<<"total_us">>, Timings)),
        ?assertEqual(<<"gateway-websocket-2">>, maps:get(<<"pod_name">>, Timings)),
        [First, Second] = maps:get(<<"trace">>, Timings),
        ?assertEqual(<<"first/0">>, maps:get(<<"name">>, First)),
        ?assertEqual(<<"second/0">>, maps:get(<<"name">>, Second)),
        Remote = require_map(maps:get(<<"remote">>, Second)),
        ?assertEqual(<<"presence_manager">>, maps:get(<<"operation">>, Remote)),
        ?assertEqual(<<"gateway-presence-1">>, maps:get(<<"pod_name">>, Remote)),
        ?assertNot(maps:is_key(<<"role">>, Timings)),
        ?assertNot(maps:is_key(<<"erlang_node_name">>, Timings)),
        ?assertNot(maps:is_key(<<"role">>, Remote)),
        ?assertNot(maps:is_key(<<"erlang_node_name">>, Remote))
    end).

sanitize_message_rewrites_resumed_timings_test() ->
    RemoteNode = 'fluxer_gateway@10.0.0.94',
    with_node_pod_names(#{RemoteNode => <<"gateway-presence-2">>}, fun() ->
        Message = (ready_message_with_finalized_timings(RemoteNode))#{<<"t">> => <<"RESUMED">>},
        Sanitized = gateway_timings_payload:sanitize_message(Message),
        #{<<"d">> := Data} = Sanitized,
        Timings = require_map(maps:get(<<"_timings_gw">>, Data)),
        ?assertEqual(15, maps:get(<<"total_us">>, Timings)),
        ?assertEqual(<<"gateway-websocket-2">>, maps:get(<<"pod_name">>, Timings)),
        [First, Second] = maps:get(<<"trace">>, Timings),
        ?assertEqual(<<"first/0">>, maps:get(<<"name">>, First)),
        ?assertEqual(<<"second/0">>, maps:get(<<"name">>, Second)),
        Remote = require_map(maps:get(<<"remote">>, Second)),
        ?assertEqual(<<"gateway-presence-2">>, maps:get(<<"pod_name">>, Remote)),
        ?assertNot(maps:is_key(<<"role">>, Timings)),
        ?assertNot(maps:is_key(<<"erlang_node_name">>, Timings))
    end).

assert_legacy_runtime_metadata_sanitized(RemoteNode) ->
    Timings = gateway_timings_payload:finalize(legacy_recorder(RemoteNode)),
    ?assertEqual(<<"microseconds">>, maps:get(<<"unit">>, Timings)),
    ?assertEqual(300, maps:get(<<"total_us">>, Timings)),
    ?assertEqual(<<"gateway-websocket-0">>, maps:get(<<"pod_name">>, Timings)),
    ?assertNot(maps:is_key(<<"node_name">>, Timings)),
    ?assertNot(maps:is_key(<<"erlang_node_name">>, Timings)),
    ?assertNot(maps:is_key(<<"role">>, Timings)),
    [First, Second] = maps:get(<<"trace">>, Timings),
    ?assertEqual(<<"first/0">>, maps:get(<<"name">>, First)),
    ?assertEqual(<<"second/0">>, maps:get(<<"name">>, Second)),
    Remote = maps:get(<<"remote">>, Second),
    ?assertEqual(<<"presence_manager">>, maps:get(<<"operation">>, Remote)),
    ?assertEqual(<<"gateway-presence-0">>, maps:get(<<"pod_name">>, Remote)),
    ?assertNot(maps:is_key(<<"role">>, Remote)),
    ?assertNot(maps:is_key(<<"erlang_node_name">>, Remote)).

legacy_recorder(RemoteNode) ->
    #{
        started_at_us => gateway_timings:start() - 60000000,
        started_node => 'fluxer_gateway@10.0.0.90',
        steps => #{},
        nodes => [],
        trace => [
            legacy_remote_span(RemoteNode),
            #{<<"name">> => <<"first/0">>, <<"duration_us">> => 100}
        ],
        pod_name => <<"gateway-websocket-0">>,
        node_name => <<"gateway-websocket-0">>,
        erlang_node_name => <<"fluxer_gateway@10.0.0.90">>,
        role => <<"websocket">>
    }.

legacy_remote_span(RemoteNode) ->
    #{
        <<"name">> => <<"second/0">>,
        <<"duration_us">> => 200,
        <<"remote">> => #{
            <<"operation">> => <<"presence_manager">>,
            <<"role">> => <<"presence">>,
            <<"erlang_node_name">> => atom_to_binary(RemoteNode, utf8)
        }
    }.

assert_legacy_child_remote_metadata_sanitized(RemoteNode) ->
    Timings = gateway_timings_payload:finalize(legacy_child_recorder(RemoteNode)),
    [Parent] = maps:get(<<"trace">>, Timings),
    [Child] = maps:get(<<"children">>, Parent),
    Remote = maps:get(<<"remote">>, Child),
    ?assertEqual(<<"guild_manager">>, maps:get(<<"operation">>, Remote)),
    ?assertEqual(<<"gateway-guilds-0">>, maps:get(<<"pod_name">>, Remote)),
    ?assertEqual(50, maps:get(<<"total_us">>, Timings)),
    ?assertNot(maps:is_key(<<"role">>, Remote)),
    ?assertNot(maps:is_key(<<"erlang_node_name">>, Remote)).

legacy_child_recorder(RemoteNode) ->
    #{
        started_at_us => gateway_timings:start() - 1000,
        steps => #{},
        nodes => [],
        trace => [legacy_child_parent_span(RemoteNode)],
        pod_name => <<"gateway-websocket-1">>
    }.

legacy_child_parent_span(RemoteNode) ->
    #{
        <<"name">> => <<"parent/0">>,
        <<"duration_us">> => 50,
        <<"children">> => [legacy_child_span(RemoteNode)]
    }.

legacy_child_span(RemoteNode) ->
    #{
        <<"name">> => <<"child/0">>,
        <<"duration_us">> => 40,
        <<"remote">> => #{
            <<"operation">> => <<"guild_manager">>,
            <<"erlang_node_name">> => atom_to_binary(RemoteNode, utf8),
            <<"role">> => <<"guilds">>
        }
    }.

ready_message_with_finalized_timings(RemoteNode) ->
    #{
        <<"op">> => 0,
        <<"t">> => <<"READY">>,
        <<"s">> => 1,
        <<"d">> => #{
            <<"_timings_gw">> => #{
                <<"unit">> => <<"microseconds">>,
                <<"total_us">> => 60000000,
                <<"pod_name">> => <<"gateway-websocket-2">>,
                <<"role">> => <<"websocket">>,
                <<"erlang_node_name">> => <<"fluxer_gateway@10.0.0.90">>,
                <<"trace">> => finalized_trace(RemoteNode)
            }
        }
    }.

finalized_trace(RemoteNode) ->
    [
        #{<<"name">> => <<"first/0">>, <<"duration_us">> => 5},
        #{
            <<"name">> => <<"second/0">>,
            <<"duration_us">> => 10,
            <<"remote">> => #{
                <<"operation">> => <<"presence_manager">>,
                <<"role">> => <<"presence">>,
                <<"erlang_node_name">> => atom_to_binary(RemoteNode, utf8)
            }
        }
    ].

require_map(Value) when is_map(Value) ->
    Value;
require_map(Value) ->
    error({expected_map, Value}).

with_node_pod_names(NodePodNames, Fun) ->
    with_persistent_term({gateway_cluster_membership, node_pod_names}, NodePodNames, Fun).

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
