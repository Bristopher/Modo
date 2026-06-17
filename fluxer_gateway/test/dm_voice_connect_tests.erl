%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(dm_voice_connect_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

handle_dm_connect_or_update_user_mismatch_test() ->
    State = base_state(existing_voice_states(20)),
    {reply, {error, validation_error, voice_user_mismatch}, _} = call_update(State, #{}).

handle_dm_connect_or_update_owner_match_proceeds_test() ->
    State = base_state(existing_voice_states(10)),
    case call_update(State, #{}) of
        {reply, {error, validation_error, voice_user_mismatch}, _} ->
            error(should_not_get_user_mismatch);
        {reply, _, _} ->
            ok
    end.

handle_dm_connect_or_update_accepts_dm_viewer_stream_key_test() ->
    StreamKey = <<"dm:100:conn-stream">>,
    State = base_state(stream_voice_states([])),
    {reply, #{success := true}, NewState} = call_update(State, #{
        viewer_stream_keys => [StreamKey]
    }),
    Updated = updated_voice_state(NewState),
    ?assertEqual([StreamKey], maps:get(<<"viewer_stream_keys">>, Updated)).

handle_dm_connect_or_update_preserves_viewer_keys_when_omitted_test() ->
    StreamKey = <<"dm:100:conn-stream">>,
    State = base_state(stream_voice_states([StreamKey])),
    {reply, #{success := true}, NewState} = call_update(State, #{
        self_mute => true,
        viewer_stream_keys => undefined
    }),
    Updated = updated_voice_state(NewState),
    ?assertEqual(true, maps:get(<<"self_mute">>, Updated)),
    ?assertEqual([StreamKey], maps:get(<<"viewer_stream_keys">>, Updated)).

handle_dm_connect_or_update_clears_viewer_keys_when_null_test() ->
    StreamKey = <<"dm:100:conn-stream">>,
    State = base_state(stream_voice_states([StreamKey])),
    {reply, #{success := true}, NewState} = call_update(State, #{
        viewer_stream_keys => null
    }),
    Updated = updated_voice_state(NewState),
    ?assertEqual([], maps:get(<<"viewer_stream_keys">>, Updated)).

handle_dm_connect_or_update_rejects_guild_scope_viewer_stream_key_test() ->
    State = base_state(stream_voice_states([])),
    {reply, {error, validation_error, voice_invalid_state}, _} = call_update(State, #{
        viewer_stream_keys => [<<"999:100:conn-stream">>]
    }).

handle_dm_connect_or_update_rejects_wrong_channel_viewer_stream_key_test() ->
    State = base_state(stream_voice_states([])),
    {reply, {error, validation_error, voice_invalid_state}, _} = call_update(State, #{
        viewer_stream_keys => [<<"dm:101:conn-stream">>]
    }).

handle_dm_connect_or_update_rejects_missing_watched_connection_test() ->
    State = base_state(stream_voice_states([])),
    {reply, {error, not_found, voice_connection_not_found}, _} = call_update(State, #{
        viewer_stream_keys => [<<"dm:100:missing-conn">>]
    }).

handle_dm_connect_or_update_rejects_non_list_viewer_stream_keys_test() ->
    State = base_state(stream_voice_states([])),
    {reply, {error, validation_error, voice_invalid_state}, _} = call_update(State, #{
        viewer_stream_keys => 123
    }).

handle_dm_connect_or_update_enforces_per_user_channel_connection_limit_test() ->
    State = base_state(limit_voice_states()),
    {reply, {error, permission_denied, voice_connection_limit_reached}, _} =
        call_update(State, #{connection_id => null}).

handle_dm_voice_with_channel_accepts_group_dm_existing_update_test() ->
    State = base_state(existing_voice_states(10)),
    Channel = #{<<"type">> => 3, <<"recipient_ids">> => [20, 30]},
    {reply, #{success := true}, _} = dm_voice_connect:handle_dm_voice_with_channel(
        Channel, 100, 10, gateway_request(#{connection_id => <<"conn-1">>}), State
    ).

handle_dm_voice_with_channel_rejects_non_dm_channel_type_test() ->
    State = base_state(existing_voice_states(10)),
    Channel = #{<<"type">> => 2, <<"recipient_ids">> => [20]},
    {reply, {error, validation_error, dm_invalid_channel_type}, _} =
        dm_voice_connect:handle_dm_voice_with_channel(
            Channel, 100, 10, gateway_request(#{connection_id => <<"conn-1">>}), State
        ).

handle_dm_voice_with_channel_rejects_non_recipient_test() ->
    State = base_state(existing_voice_states(20)),
    Channel = #{<<"type">> => 1, <<"recipient_ids">> => [10]},
    {reply, {error, permission_denied, dm_not_recipient}, _} =
        dm_voice_connect:handle_dm_voice_with_channel(
            Channel, 100, 20, gateway_request(#{connection_id => <<"conn-1">>}), State
        ).

handle_dm_connect_or_update_stress_many_watch_unwatch_updates_test() ->
    StreamKey = <<"dm:100:conn-stream">>,
    InitialState = base_state(stream_voice_states([])),
    FinalState = lists:foldl(
        fun(Index, State) ->
            Keys =
                case Index rem 3 of
                    0 -> [];
                    _ -> [StreamKey]
                end,
            SelfMute = Index rem 2 =:= 0,
            {reply, #{success := true}, NewState} = call_update(State, #{
                self_mute => SelfMute,
                viewer_stream_keys => Keys
            }),
            Updated = updated_voice_state(NewState),
            ?assertEqual(SelfMute, maps:get(<<"self_mute">>, Updated)),
            ?assertEqual(Keys, maps:get(<<"viewer_stream_keys">>, Updated)),
            NewState
        end,
        InitialState,
        lists:seq(1, 60)
    ),
    ?assertMatch(#{dm_voice_states := #{}}, FinalState).

existing_voice_states(UserId) ->
    #{<<"conn-1">> => voice_state(<<"conn-1">>, UserId, 100, [])}.

stream_voice_states(InitialViewerKeys) ->
    #{
        <<"conn-1">> => voice_state(<<"conn-1">>, 10, 100, InitialViewerKeys),
        <<"conn-stream">> => voice_state(<<"conn-stream">>, 20, 100, [])
    }.

limit_voice_states() ->
    maps:from_list([
        {
            <<"conn-", (integer_to_binary(Index))/binary>>,
            voice_state(conn_id(Index), 10, 100, [])
        }
     || Index <- lists:seq(1, 5)
    ]).

conn_id(Index) ->
    <<"conn-", (integer_to_binary(Index))/binary>>.

voice_state(ConnectionId, UserId, ChannelId, ViewerKeys) ->
    #{
        <<"connection_id">> => ConnectionId,
        <<"channel_id">> => integer_to_binary(ChannelId),
        <<"user_id">> => integer_to_binary(UserId),
        <<"session_id">> => <<"sess">>,
        <<"self_mute">> => false,
        <<"self_deaf">> => false,
        <<"self_video">> => false,
        <<"self_stream">> => false,
        <<"is_mobile">> => false,
        <<"viewer_stream_keys">> => ViewerKeys
    }.

base_state(VoiceStates) ->
    #{
        dm_voice_states => VoiceStates,
        channels => #{100 => #{<<"type">> => 1, <<"recipient_ids">> => [20, 30]}},
        user_id => 10,
        id => <<"sess">>,
        session_pid => self()
    }.

gateway_request(Overrides) ->
    maps:merge(
        #{
            session_id => <<"sess">>,
            connection_id => null,
            self_mute => false,
            self_deaf => false,
            self_video => false,
            self_stream => false,
            viewer_stream_keys => undefined,
            is_mobile => false,
            latitude => null,
            longitude => null,
            e2ee_capable => false,
            bot => false
        },
        Overrides
    ).

connect_request(State, Overrides) ->
    VoiceStates = maps:get(dm_voice_states, State, #{}),
    maps:merge(
        gateway_request(#{
            user_id => 10,
            channel_id => 100,
            connection_id => <<"conn-1">>,
            voice_states => VoiceStates,
            state => State
        }),
        Overrides
    ).

call_update(State, Overrides) ->
    dm_voice_connect:handle_dm_connect_or_update(connect_request(State, Overrides)).

updated_voice_state(State) ->
    maps:get(<<"conn-1">>, maps:get(dm_voice_states, State)).
