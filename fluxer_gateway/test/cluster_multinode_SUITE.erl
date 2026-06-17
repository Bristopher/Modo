%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(cluster_multinode_SUITE).
-typing([eqwalizer]).

-include_lib("common_test/include/ct.hrl").
-export([
    all/0,
    init_per_suite/1,
    end_per_suite/1,
    init_per_testcase/2,
    end_per_testcase/2
]).
-export([
    two_nodes_see_each_other/1,
    owner_node_agrees_across_cluster/1,
    nodedown_shrinks_membership/1,
    remote_presence_fanout/1,
    remote_dispatch_relay_direct/1,
    remote_session_state_transfer/1,
    remote_presence_cache_rebalance/1,
    remote_guild_handoff_ships_state/1
]).

all() ->
    [
        two_nodes_see_each_other,
        owner_node_agrees_across_cluster,
        nodedown_shrinks_membership,
        remote_presence_fanout,
        remote_dispatch_relay_direct,
        remote_session_state_transfer,
        remote_presence_cache_rebalance,
        remote_guild_handoff_ships_state
    ].

init_per_suite(Config) ->
    case ensure_distribution() of
        ok ->
            true = erlang:set_cookie(node(), fluxer_cluster_ct),
            Config;
        {skip, Reason} ->
            {skip, Reason}
    end.

ensure_distribution() ->
    case net_kernel:get_state() of
        #{started := no} ->
            try_start_distribution();
        _ ->
            ok
    end.

try_start_distribution() ->
    Candidates = [
        controller,
        list_to_atom("controller@127.0.0.1")
    ],
    try_start_distribution(Candidates).

try_start_distribution([]) ->
    {skip,
        "could not start Erlang distribution (no working name); "
        "see ensure_distribution/0 in cluster_multinode_SUITE"};
try_start_distribution([Name | Rest]) ->
    case net_kernel:start([Name, longnames]) of
        {ok, _} -> ok;
        {error, {already_started, _}} -> ok;
        {error, _Reason} -> try_start_distribution(Rest)
    end.

end_per_suite(_Config) ->
    ok.

init_per_testcase(Case, Config) ->
    [{case_name, Case} | Config].

end_per_testcase(_Case, Config) ->
    case proplists:get_value(peers, Config, []) of
        Peers when is_list(Peers) ->
            lists:foreach(fun cluster_multinode_support:stop_peer/1, Peers);
        _ ->
            ok
    end,
    ok.

two_nodes_see_each_other(Config) ->
    cluster_multinode_basic_cases:two_nodes_see_each_other(Config).

owner_node_agrees_across_cluster(Config) ->
    cluster_multinode_basic_cases:owner_node_agrees_across_cluster(Config).

nodedown_shrinks_membership(Config) ->
    cluster_multinode_basic_cases:nodedown_shrinks_membership(Config).

remote_presence_fanout(Config) ->
    cluster_multinode_remote_cases:remote_presence_fanout(Config).

remote_dispatch_relay_direct(Config) ->
    cluster_multinode_remote_cases:remote_dispatch_relay_direct(Config).

remote_session_state_transfer(Config) ->
    cluster_multinode_remote_cases:remote_session_state_transfer(Config).

remote_presence_cache_rebalance(Config) ->
    cluster_multinode_remote_cases:remote_presence_cache_rebalance(Config).

remote_guild_handoff_ships_state(Config) ->
    cluster_multinode_remote_cases:remote_guild_handoff_ships_state(Config).
