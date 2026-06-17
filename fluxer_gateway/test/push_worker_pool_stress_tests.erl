%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_worker_pool_stress_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

-define(PUSH_WORKER_COUNTER, push_worker_counter).
-define(MAX_PUSH_WORKERS, 256).
-define(STAMPEDE_CALLERS, 1024).

stampede_respects_worker_cap_and_recovers_test_() ->
    {timeout, 15, fun stampede_respects_worker_cap_and_recovers/0}.

stampede_respects_worker_cap_and_recovers() ->
    cleanup_counter(),
    ok = push_worker_pool:init_counter(),
    Ref = make_ref(),
    Parent = self(),
    WorkerFun = worker_fun(Ref, Parent),
    _Callers = spawn_callers(Ref, WorkerFun, Parent, ?STAMPEDE_CALLERS),
    Results = collect_spawn_results(Ref, ?STAMPEDE_CALLERS, []),
    OkCount = count_result(ok, Results),
    DroppedCount = count_result(dropped, Results),
    WorkerPids = collect_worker_starts(Ref, OkCount, []),
    try
        ?assertEqual(?MAX_PUSH_WORKERS, OkCount),
        ?assertEqual(?STAMPEDE_CALLERS - ?MAX_PUSH_WORKERS, DroppedCount),
        ?assertEqual(?STAMPEDE_CALLERS, OkCount + DroppedCount),
        ?assertEqual(OkCount, length(lists:usort(WorkerPids))),
        ?assertEqual(?MAX_PUSH_WORKERS, push_worker_pool:active_count())
    after
        release_workers(WorkerPids),
        wait_until(fun() -> push_worker_pool:active_count() =:= 0 end, 200),
        cleanup_counter()
    end.

worker_fun(Ref, Parent) ->
    fun() ->
        Parent ! {Ref, worker_started, self()},
        receive
            release -> ok
        after 10000 ->
            timeout
        end
    end.

spawn_callers(Ref, WorkerFun, Parent, Count) ->
    [
        spawn(fun() ->
            Parent ! {Ref, spawn_result, push_worker_pool:maybe_spawn(WorkerFun)}
        end)
     || _ <- lists:seq(1, Count)
    ].

collect_spawn_results(_Ref, 0, Acc) ->
    Acc;
collect_spawn_results(Ref, Remaining, Acc) ->
    receive
        {Ref, spawn_result, Result} ->
            collect_spawn_results(Ref, Remaining - 1, [Result | Acc])
    after 5000 ->
        ?assert(false)
    end.

collect_worker_starts(_Ref, 0, Acc) ->
    Acc;
collect_worker_starts(Ref, Remaining, Acc) ->
    receive
        {Ref, worker_started, Pid} ->
            collect_worker_starts(Ref, Remaining - 1, [Pid | Acc])
    after 5000 ->
        ?assert(false)
    end.

count_result(Expected, Results) ->
    length([Result || Result <- Results, Result =:= Expected]).

release_workers(WorkerPids) ->
    lists:foreach(fun(Pid) -> Pid ! release end, WorkerPids).

wait_until(Predicate, Attempts) ->
    case Predicate() of
        true ->
            ok;
        false when Attempts > 0 ->
            timer:sleep(10),
            wait_until(Predicate, Attempts - 1);
        false ->
            ?assert(false)
    end.

cleanup_counter() ->
    try ets:delete(?PUSH_WORKER_COUNTER) of
        _ -> ok
    catch
        throw:_ -> ok;
        error:_ -> ok;
        exit:_ -> ok
    end.
