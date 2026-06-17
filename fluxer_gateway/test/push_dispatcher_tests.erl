%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_dispatcher_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

enqueue_saturation_bounds_inflight_and_queue_test_() ->
    {timeout, 30, fun enqueue_saturation_bounds_inflight_and_queue/0}.

worker_down_drains_queue_without_exceeding_inflight_limit_test_() ->
    {timeout, 30, fun worker_down_drains_queue_without_exceeding_inflight_limit/0}.

invalid_jobs_do_not_change_dispatcher_state_test() ->
    State0 = dispatcher_state(2, 4),
    {noreply, State1} = push_dispatcher:handle_cast({enqueue, #{type => invalid}}, State0),
    ?assertEqual(State0, State1).

enqueue_call_reports_drop_when_queue_full_test_() ->
    {timeout, 30, fun enqueue_call_reports_drop_when_queue_full/0}.

enqueue_saturation_bounds_inflight_and_queue() ->
    with_push_sender_blocked(fun() ->
        State0 = dispatcher_state(2, 3),
        State1 = enqueue_jobs(lists:seq(1, 10), State0),
        ?assertEqual(2, maps:get(inflight, State1)),
        ?assertEqual(3, maps:get(queued, State1)),
        ?assertEqual(3, queue:len(maps:get(queue, State1))),
        Started = collect_started(2),
        assert_no_push_started(),
        release_started(Started),
        drain_worker_downs(length(Started))
    end).

worker_down_drains_queue_without_exceeding_inflight_limit() ->
    with_push_sender_blocked(fun() ->
        State0 = dispatcher_state(2, 10),
        State1 = enqueue_jobs(lists:seq(1, 5), State0),
        Started0 = collect_started(2),
        ?assertEqual(2, maps:get(inflight, State1)),
        ?assertEqual(3, maps:get(queued, State1)),
        [First | Rest] = Started0,
        release_started([First]),
        Down = wait_worker_down(First),
        {noreply, State2} = push_dispatcher:handle_info(Down, State1),
        Started1 = collect_started(1),
        ?assertEqual(2, maps:get(inflight, State2)),
        ?assertEqual(2, maps:get(queued, State2)),
        assert_no_push_started(),
        release_started(Rest ++ Started1),
        drain_worker_downs(length(Rest ++ Started1))
    end).

enqueue_call_reports_drop_when_queue_full() ->
    with_push_sender_blocked(fun() ->
        State0 = dispatcher_state(1, 1),
        {reply, ok, State1} =
            push_dispatcher:handle_call({enqueue, send_job(1)}, from(), State0),
        Started = collect_started(1),
        {reply, ok, State2} =
            push_dispatcher:handle_call({enqueue, send_job(2)}, from(), State1),
        {reply, dropped, State3} =
            push_dispatcher:handle_call({enqueue, send_job(3)}, from(), State2),
        ?assertEqual(1, maps:get(inflight, State3)),
        ?assertEqual(1, maps:get(queued, State3)),
        ?assertEqual(1, queue:len(maps:get(queue, State3))),
        release_started(Started),
        drain_worker_downs(length(Started))
    end).

dispatcher_state(MaxInflight, MaxQueue) ->
    #{
        queue => queue:new(),
        queued => 0,
        inflight => 0,
        workers => #{},
        max_inflight => MaxInflight,
        max_queue => MaxQueue
    }.

from() ->
    {self(), make_ref()}.

enqueue_jobs(MessageIds, State0) ->
    lists:foldl(fun enqueue_job/2, State0, MessageIds).

enqueue_job(MessageId, State) ->
    {noreply, NewState} = push_dispatcher:handle_cast(
        {enqueue, send_job(MessageId)}, State
    ),
    NewState.

send_job(MessageId) ->
    #{
        type => message_create,
        user_ids => [1000 + MessageId],
        message_data => #{<<"id">> => integer_to_binary(MessageId)},
        guild_id => 10,
        channel_id => 20,
        message_id => MessageId,
        guild_name => <<"guild">>,
        channel_name => <<"channel">>,
        badge_counts_ttl_seconds => 60
    }.

with_push_sender_blocked(Fun) ->
    meck:new(push_sender, [passthrough, no_link]),
    Parent = self(),
    persistent_term:put({?MODULE, push_parent}, Parent),
    meck:expect(
        push_sender,
        send_push_notifications,
        fun push_send_notifications_mock/1
    ),
    try
        Fun()
    after
        _ = persistent_term:erase({?MODULE, push_parent}),
        meck:unload(push_sender)
    end.

push_send_notifications_mock(#{message_id := MessageId}) ->
    Parent = persistent_term:get({?MODULE, push_parent}),
    Parent ! {push_started, self(), MessageId},
    receive
        {release_push_worker, Parent} -> ok
    after 30000 ->
        ok
    end.

collect_started(Count) ->
    collect_started(Count, []).

collect_started(0, Acc) ->
    lists:reverse(Acc);
collect_started(Count, Acc) ->
    receive
        {push_started, Pid, MessageId} ->
            collect_started(Count - 1, [{Pid, MessageId} | Acc])
    after 5000 ->
        ?assert(false, {push_started_timeout, Count})
    end.

assert_no_push_started() ->
    receive
        {push_started, Pid, MessageId} ->
            ?assert(false, {unexpected_push_started, Pid, MessageId})
    after 100 ->
        ok
    end.

release_started(Started) ->
    lists:foreach(
        fun({Pid, _MessageId}) ->
            Pid ! {release_push_worker, self()}
        end,
        Started
    ).

wait_worker_down({Pid, _MessageId}) ->
    receive
        {'DOWN', Ref, process, Pid, Reason} ->
            {'DOWN', Ref, process, Pid, Reason}
    after 5000 ->
        ?assert(false, {worker_down_timeout, Pid})
    end.

drain_worker_downs(Count) ->
    lists:foreach(
        fun(_) ->
            receive
                {'DOWN', _Ref, process, _Pid, _Reason} -> ok
            after 5000 ->
                ?assert(false, worker_down_drain_timeout)
            end
        end,
        lists:seq(1, Count)
    ).
