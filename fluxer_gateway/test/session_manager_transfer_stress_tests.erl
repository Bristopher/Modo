%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(session_manager_transfer_stress_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

aggregate_counts_many_shards_mixed_replies_test_() ->
    {timeout, 15, fun aggregate_counts_many_shards_mixed_replies/0}.

aggregate_handoff_many_shards_mixed_replies_test_() ->
    {timeout, 15, fun aggregate_handoff_many_shards_mixed_replies/0}.

aggregate_counts_many_shards_mixed_replies() ->
    {Shards, LivePids, ExpectedCount} = build_count_shards(256),
    State = #{shards => Shards, shard_count => 256},
    try
        {ExpectedCount, State} = session_manager_transfer:aggregate_counts(
            reconnect_drain, State
        )
    after
        stop_live_shards(LivePids)
    end.

aggregate_handoff_many_shards_mixed_replies() ->
    {Shards, LivePids, ExpectedResult} = build_handoff_shards(128),
    State = #{shards => Shards, shard_count => 128},
    try
        {ExpectedResult, State} =
            session_manager_transfer:aggregate_handoff_to_topology(['peer@host'], State)
    after
        stop_live_shards(LivePids)
    end.

build_count_shards(Count) ->
    lists:foldl(
        fun(Index, {ShardAcc, PidAcc, ExpectedAcc}) ->
            {Pid, MaybeLive, Expected} = count_shard(Index),
            {
                ShardAcc#{Index => #{pid => Pid, ref => make_ref()}},
                add_live_pid(MaybeLive, Pid, PidAcc),
                ExpectedAcc + Expected
            }
        end,
        {#{}, [], 0},
        lists:seq(1, Count)
    ).

count_shard(Index) ->
    case Index rem 10 of
        0 ->
            dead_shard(0);
        1 ->
            {spawn_shard(invalid_reply), live, 0};
        _ ->
            {spawn_shard({ok, 1}), live, 1}
    end.

build_handoff_shards(Count) ->
    lists:foldl(
        fun(Index, {ShardAcc, PidAcc, ExpectedAcc}) ->
            {Pid, MaybeLive, Expected} = handoff_shard(Index),
            {
                ShardAcc#{Index => #{pid => Pid, ref => make_ref()}},
                add_live_pid(MaybeLive, Pid, PidAcc),
                sum_handoff(ExpectedAcc, Expected)
            }
        end,
        {#{}, [], #{attempted => 0, handed_off => 0}},
        lists:seq(1, Count)
    ).

handoff_shard(Index) ->
    case Index rem 8 of
        0 ->
            dead_shard(#{attempted => 1, handed_off => 0});
        1 ->
            {spawn_shard(invalid_reply), live, #{attempted => 1, handed_off => 0}};
        2 ->
            {spawn_shard(2), live, #{attempted => 2, handed_off => 2}};
        _ ->
            {spawn_shard({ok, #{attempted => 3, handed_off => 2}}), live, #{
                attempted => 3,
                handed_off => 2
            }}
    end.

dead_shard(Expected) ->
    Pid = spawn(fun() -> ok end),
    wait_for_dead(Pid, 100),
    {Pid, dead, Expected}.

spawn_shard(Reply) ->
    spawn(fun() -> shard_loop(Reply) end).

shard_loop(Reply) ->
    receive
        {'$gen_call', From, _Request} ->
            gen_server:reply(From, Reply),
            shard_loop(Reply);
        stop ->
            ok
    after 10000 ->
        ok
    end.

add_live_pid(live, Pid, Pids) ->
    [Pid | Pids];
add_live_pid(dead, _Pid, Pids) ->
    Pids.

stop_live_shards(Pids) ->
    lists:foreach(fun(Pid) -> Pid ! stop end, Pids).

sum_handoff(Acc, Result) ->
    #{
        attempted => maps:get(attempted, Acc) + maps:get(attempted, Result),
        handed_off => maps:get(handed_off, Acc) + maps:get(handed_off, Result)
    }.

wait_for_dead(Pid, Attempts) ->
    case is_process_alive(Pid) of
        false ->
            ok;
        true when Attempts > 0 ->
            timer:sleep(1),
            wait_for_dead(Pid, Attempts - 1);
        true ->
            exit(Pid, kill),
            ok
    end.
