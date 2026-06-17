%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_retry_timer_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

wait_zero_returns_ok_test() ->
    ?assertEqual(ok, gateway_retry_timer:wait(0)).

wait_rejects_invalid_delay_test() ->
    ?assertEqual({error, invalid_delay}, gateway_retry_timer:wait(-1)),
    ?assertEqual({error, invalid_delay}, gateway_retry_timer:wait(not_an_integer)).

wait_until_expired_deadline_test() ->
    DeadlineMs = erlang:monotonic_time(millisecond) - 1,
    ?assertEqual(expired, gateway_retry_timer:wait_until(10, DeadlineMs)).

wait_keeps_unrelated_mailbox_messages_test() ->
    self() ! unrelated_message,
    ?assertEqual(ok, gateway_retry_timer:wait(0)),
    receive
        unrelated_message -> ok
    after 0 ->
        ?assert(false, unrelated_message_was_consumed)
    end.
