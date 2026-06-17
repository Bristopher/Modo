%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(session_manager_routing_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

select_shard_deterministic_test() ->
    SessionId = <<"session-abc">>,
    ShardCount = 8,
    Shard1 = session_manager_routing:select_shard(SessionId, ShardCount),
    Shard2 = session_manager_routing:select_shard(SessionId, ShardCount),
    ?assertEqual(Shard1, Shard2).

select_shard_in_range_test() ->
    ShardCount = 8,
    lists:foreach(
        fun(N) ->
            SessionId = list_to_binary(integer_to_list(N)),
            Shard = session_manager_routing:select_shard(SessionId, ShardCount),
            ?assert(Shard >= 0 andalso Shard < ShardCount)
        end,
        lists:seq(1, 100)
    ).

start_call_with_drain_guard_rejects_during_drain_test() ->
    State = #{shards => #{}, shard_count => 1},
    ?assertEqual(
        {{error, draining}, State},
        session_manager_routing:start_call_with_drain_guard(true, #{}, self(), State)
    ).
