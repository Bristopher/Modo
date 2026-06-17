%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(session_manager_shard_session_index_tests).

-include_lib("eunit/include/eunit.hrl").

put_builds_ref_index_test() ->
    Ref = make_ref(),
    State = session_manager_shard_session_index:put(<<"s1">>, {self(), Ref}, #{
        sessions => #{}
    }),
    ?assertEqual(#{<<"s1">> => {self(), Ref}}, maps:get(sessions, State)),
    ?assertEqual(#{Ref => <<"s1">>}, maps:get(session_refs, State)).

remove_clears_ref_index_test() ->
    Ref = make_ref(),
    State0 = session_manager_shard_session_index:put(<<"s1">>, {self(), Ref}, #{
        sessions => #{}
    }),
    State1 = session_manager_shard_session_index:remove(<<"s1">>, State0),
    ?assertEqual(#{}, maps:get(sessions, State1)),
    ?assertEqual(#{}, maps:get(session_refs, State1)).

cleanup_down_uses_ref_index_test() ->
    Ref1 = make_ref(),
    Ref2 = make_ref(),
    Pid1 = spawn(fun wait_forever/0),
    Pid2 = spawn(fun wait_forever/0),
    State0 = #{
        sessions => #{
            <<"s1">> => {Pid1, Ref1},
            <<"s2">> => {Pid2, Ref2}
        },
        session_refs => #{Ref1 => <<"s1">>, Ref2 => <<"s2">>}
    },
    State1 = session_manager_shard_session_index:cleanup_down(Ref1, Pid1, State0),
    ?assertEqual(#{<<"s2">> => {Pid2, Ref2}}, maps:get(sessions, State1)),
    ?assertEqual(#{Ref2 => <<"s2">>}, maps:get(session_refs, State1)),
    exit(Pid1, kill),
    exit(Pid2, kill).

cleanup_down_rebuilds_missing_ref_index_test() ->
    Ref1 = make_ref(),
    Ref2 = make_ref(),
    DeadPid = spawn(fun() -> ok end),
    LivePid = self(),
    State0 = #{
        sessions => #{
            <<"s1">> => {DeadPid, Ref1},
            <<"s2">> => {LivePid, Ref2}
        }
    },
    State1 = session_manager_shard_session_index:cleanup_down(Ref1, DeadPid, State0),
    ?assertEqual(#{<<"s2">> => {LivePid, Ref2}}, maps:get(sessions, State1)),
    ?assertEqual(#{Ref2 => <<"s2">>}, maps:get(session_refs, State1)).

wait_forever() ->
    receive
    after infinity -> ok
    end.
