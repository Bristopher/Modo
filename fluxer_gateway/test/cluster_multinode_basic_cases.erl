%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(cluster_multinode_basic_cases).
-typing([eqwalizer]).

-include_lib("common_test/include/ct.hrl").
-include_lib("eunit/include/eunit.hrl").

-export([
    nodedown_shrinks_membership/1,
    owner_node_agrees_across_cluster/1,
    two_nodes_see_each_other/1
]).

-spec two_nodes_see_each_other(list()) -> ok.
two_nodes_see_each_other(Config0) ->
    {ok, Peer1, Node1} = start_peer(node1, Config0),
    {ok, Peer2, Node2} = start_peer(node2, Config0),
    Config = [{peers, [Peer1, Peer2]} | Config0],
    ok = drive_membership(Peer1, [Node1, Node2]),
    ok = drive_membership(Peer2, [Node1, Node2]),
    Expected = lists:sort([Node1, Node2]),
    wait_until(
        fun() -> lists:sort(rpc_members(Peer1)) =:= Expected end,
        "node1 membership"
    ),
    wait_until(
        fun() -> lists:sort(rpc_members(Peer2)) =:= Expected end,
        "node2 membership"
    ),
    ?assertEqual(2, rpc:call(Node1, gateway_cluster_membership, alive_count, [])),
    ?assertEqual(2, rpc:call(Node2, gateway_cluster_membership, alive_count, [])),
    ct:log("two_nodes_see_each_other: ~p", [Config]),
    ok.

-spec owner_node_agrees_across_cluster(list()) -> ok.
owner_node_agrees_across_cluster(Config0) ->
    {ok, Peer1, Node1} = start_peer(node1, Config0),
    {ok, Peer2, Node2} = start_peer(node2, Config0),
    _Config = [{peers, [Peer1, Peer2]} | Config0],
    ok = drive_membership(Peer1, [Node1, Node2]),
    ok = drive_membership(Peer2, [Node1, Node2]),
    Expected = lists:sort([Node1, Node2]),
    wait_until(
        fun() -> lists:sort(rpc_members(Peer1)) =:= Expected end,
        "convergence n1"
    ),
    wait_until(
        fun() -> lists:sort(rpc_members(Peer2)) =:= Expected end,
        "convergence n2"
    ),
    Keys = [<<"key-", (integer_to_binary(I))/binary>> || I <- lists:seq(1, 50)],
    [
        ?assertEqual(
            rpc:call(Node1, gateway_node_router, owner_node, [K]),
            rpc:call(Node2, gateway_node_router, owner_node, [K]),
            {owner_disagreement, K}
        )
     || K <- Keys
    ],
    [assert_owner_in_cluster(Node1, [Node1, Node2], K) || K <- Keys],
    ok.

-spec nodedown_shrinks_membership(list()) -> ok.
nodedown_shrinks_membership(Config0) ->
    {ok, Peer1, Node1} = start_peer(node1, Config0),
    {ok, Peer2, Node2} = start_peer(node2, Config0),
    _Config = [{peers, [Peer1]} | Config0],
    ok = drive_membership(Peer1, [Node1, Node2]),
    ok = drive_membership(Peer2, [Node1, Node2]),
    Expected = lists:sort([Node1, Node2]),
    wait_until(
        fun() -> lists:sort(rpc_members(Peer1)) =:= Expected end,
        "initial n1 convergence"
    ),
    stop_peer(Peer2),
    wait_until(
        fun() -> rpc_members(Peer1) =:= [Node1] end,
        "n1 shrinks back to self after nodedown"
    ),
    ?assertEqual(1, rpc:call(Node1, gateway_cluster_membership, alive_count, [])),
    ok.

drive_membership(Peer, Nodes) ->
    cluster_multinode_support:drive_membership(Peer, Nodes).

rpc_members(Peer) ->
    cluster_multinode_support:rpc_members(Peer).

start_peer(Name, Config) ->
    cluster_multinode_support:start_peer(Name, Config).

stop_peer(Peer) ->
    cluster_multinode_support:stop_peer(Peer).

wait_until(Fun, Label) ->
    cluster_multinode_support:wait_until(Fun, Label).

assert_owner_in_cluster(Node, ClusterNodes, Key) ->
    Owner = rpc:call(Node, gateway_node_router, owner_node, [Key]),
    ?assert(lists:member(Owner, ClusterNodes), {owner_not_in_cluster, Owner, Key}).
