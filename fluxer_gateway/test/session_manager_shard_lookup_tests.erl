%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(session_manager_shard_lookup_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

normalize_status_test() ->
    ?assertEqual(undefined, session_manager_shard_lookup:normalize_status(undefined)),
    ?assertEqual(undefined, session_manager_shard_lookup:normalize_status(null)),
    ?assertEqual(<<"online">>, session_manager_shard_lookup:normalize_status(<<"online">>)),
    ?assertEqual(<<"idle">>, session_manager_shard_lookup:normalize_status(<<"idle">>)),
    ?assertEqual(
        <<"invisible">>, session_manager_shard_lookup:normalize_status(<<"invisible">>)
    ),
    ?assertEqual(undefined, session_manager_shard_lookup:normalize_status(123)),
    ok.

select_initial_status_test() ->
    ?assertEqual(
        <<"idle">>, session_manager_shard_lookup:select_initial_status(undefined, <<"idle">>)
    ),
    ?assertEqual(
        <<"dnd">>, session_manager_shard_lookup:select_initial_status(<<"unknown">>, <<"dnd">>)
    ),
    ?assertEqual(
        <<"idle">>, session_manager_shard_lookup:select_initial_status(<<"online">>, <<"idle">>)
    ),
    ?assertEqual(
        <<"online">>,
        session_manager_shard_lookup:select_initial_status(<<"online">>, <<"online">>)
    ),
    ?assertEqual(
        <<"dnd">>, session_manager_shard_lookup:select_initial_status(<<"dnd">>, <<"online">>)
    ),
    ok.

parse_presence_preserves_invisible_stored_status_test() ->
    Data = #{<<"user_settings">> => #{<<"status">> => <<"invisible">>}},
    IdentifyData = #{presence => #{<<"status">> => <<"online">>}},
    ?assertEqual(invisible, session_manager_shard_lookup:parse_presence(Data, IdentifyData)).

parse_presence_preserves_invisible_identify_status_test() ->
    Data = #{<<"user_settings">> => #{<<"status">> => <<"online">>}},
    IdentifyData = #{presence => #{<<"status">> => <<"invisible">>}},
    ?assertEqual(invisible, session_manager_shard_lookup:parse_presence(Data, IdentifyData)).

normalize_coordinate_test() ->
    ?assertEqual(undefined, session_manager_shard_lookup:normalize_coordinate(undefined)),
    ?assertEqual(undefined, session_manager_shard_lookup:normalize_coordinate(null)),
    ?assertEqual(1.5, session_manager_shard_lookup:normalize_coordinate(1.5)),
    ?assertEqual(<<"test">>, session_manager_shard_lookup:normalize_coordinate(<<"test">>)),
    ok.

add_coordinates_test() ->
    Base = #{<<"type">> => <<"session">>},
    ?assertEqual(
        Base, session_manager_shard_lookup:add_coordinates(Base, undefined, undefined)
    ),
    ?assertEqual(
        #{<<"type">> => <<"session">>, <<"latitude">> => 1.0},
        session_manager_shard_lookup:add_coordinates(Base, 1.0, undefined)
    ),
    ?assertEqual(
        #{<<"type">> => <<"session">>, <<"longitude">> => 2.0},
        session_manager_shard_lookup:add_coordinates(Base, undefined, 2.0)
    ),
    ?assertEqual(
        #{<<"type">> => <<"session">>, <<"latitude">> => 1.0, <<"longitude">> => 2.0},
        session_manager_shard_lookup:add_coordinates(Base, 1.0, 2.0)
    ),
    ok.

validate_transfer_token_test() ->
    GoodToken = <<"good_token">>,
    GoodHash = utils:hash_token(GoodToken),
    ?assertEqual(
        ok,
        session_manager_shard_lookup:validate_transfer_token(GoodToken, #{
            token_hash => GoodHash
        })
    ),
    ?assertEqual(
        {error, invalid_token},
        session_manager_shard_lookup:validate_transfer_token(<<"bad_token">>, #{
            token_hash => GoodHash
        })
    ),
    ?assertEqual(
        {error, invalid_token},
        session_manager_shard_lookup:validate_transfer_token(GoodToken, #{})
    ).

lookup_or_rehydrate_restores_state_on_invalid_token_test() ->
    with_session_state_transfer(fun() ->
        SessionId = <<"session-rehydrate-invalid-token">>,
        TransferState = #{token_hash => utils:hash_token(<<"expected">>)},
        ok = session_state_transfer:push_state(node(), SessionId, TransferState),
        {Reply, _StateAfter} = session_manager_shard_lookup:lookup_or_rehydrate(
            SessionId, <<"unexpected">>, self(), new_test_state()
        ),
        ?assertEqual({error, invalid_token}, Reply),
        ?assertEqual({ok, TransferState}, session_state_transfer:pop_state(SessionId))
    end).

lookup_remote_on_nodes_empty_test() ->
    ?assertEqual(
        {error, not_found},
        session_manager_shard_lookup:lookup_remote_on_nodes([], {session, <<"missing">>})
    ).

select_remote_session_pid_ignores_not_found_entries_test() ->
    Pid = spawn(fun() ->
        receive
            stop -> ok
        after infinity ->
            ok
        end
    end),
    try
        ?assertEqual(
            {ok, Pid},
            session_manager_shard_lookup:select_remote_session_pid(
                [undefined, {badrpc, nodedown}, Pid]
            )
        )
    after
        Pid ! stop
    end.

select_remote_session_pid_returns_not_found_without_pid_test() ->
    ?assertEqual(
        {error, not_found},
        session_manager_shard_lookup:select_remote_session_pid(
            [undefined, {badrpc, nodedown}, {error, not_found}]
        )
    ).

remote_lookup_nodes_excludes_local_node_test() ->
    ?assertEqual([], session_manager_shard_lookup:remote_lookup_nodes(<<"session">>, [node()])).

remote_lookup_nodes_prioritizes_hash_owner_test() ->
    RemoteA = 'gateway_a@example',
    RemoteB = 'gateway_b@example',
    Nodes = lists:usort([node(), RemoteA, RemoteB]),
    SessionId = session_id_with_remote_owner(Nodes, 0),
    OwnerNode = gateway_node_router:select_owner_node(SessionId, Nodes),
    LookupNodes = session_manager_shard_lookup:remote_lookup_nodes(SessionId, Nodes),
    ?assert(OwnerNode =/= node()),
    ?assertEqual(OwnerNode, hd(LookupNodes)),
    ?assertNot(lists:member(node(), LookupNodes)),
    ?assertEqual(lists:sort([RemoteA, RemoteB]), lists:sort(LookupNodes)).

get_presence_custom_status_test() ->
    ?assertEqual(null, session_manager_shard_lookup:get_presence_custom_status(null)),
    ?assertEqual(null, session_manager_shard_lookup:get_presence_custom_status(#{})),
    ?assertEqual(
        #{<<"text">> => <<"hello">>},
        session_manager_shard_lookup:get_presence_custom_status(
            #{<<"custom_status">> => #{<<"text">> => <<"hello">>}}
        )
    ),
    ?assertEqual(null, session_manager_shard_lookup:get_presence_custom_status(not_a_map)),
    ok.

parse_guild_ids_extracts_from_guilds_array_test() ->
    Data = #{
        <<"guilds">> => [
            #{<<"id">> => <<"100">>},
            #{<<"id">> => <<"200">>},
            #{<<"id">> => <<"300">>}
        ]
    },
    ?assertEqual([100, 200, 300], session_manager_shard_lookup:parse_guild_ids(Data)).

parse_guild_ids_returns_empty_when_no_guilds_test() ->
    ?assertEqual([], session_manager_shard_lookup:parse_guild_ids(#{})),
    ?assertEqual([], session_manager_shard_lookup:parse_guild_ids(#{<<"guilds">> => []})).

parse_guild_ids_skips_non_map_entries_test() ->
    Data = #{
        <<"guilds">> => [
            #{<<"id">> => <<"100">>},
            <<"not_a_map">>,
            #{<<"id">> => <<"200">>}
        ]
    },
    ?assertEqual([100, 200], session_manager_shard_lookup:parse_guild_ids(Data)).

parse_guild_ids_skips_entries_without_id_test() ->
    Data = #{
        <<"guilds">> => [
            #{<<"id">> => <<"100">>},
            #{<<"name">> => <<"no_id">>},
            #{<<"id">> => <<"200">>}
        ]
    },
    ?assertEqual([100, 200], session_manager_shard_lookup:parse_guild_ids(Data)).

parse_guild_ids_handles_unavailable_guilds_test() ->
    Data = #{
        <<"guilds">> => [
            #{<<"id">> => <<"100">>, <<"unavailable">> => true},
            #{<<"id">> => <<"200">>}
        ]
    },
    ?assertEqual([100, 200], session_manager_shard_lookup:parse_guild_ids(Data)).

new_test_state() ->
    #{
        sessions => #{},
        identify_attempts => [],
        pending_identifies => #{},
        identify_workers => #{},
        shard_index => 0
    }.

session_id_with_remote_owner(Nodes, Index) ->
    SessionId = <<"session-remote-owner-", (integer_to_binary(Index))/binary>>,
    case gateway_node_router:select_owner_node(SessionId, Nodes) =/= node() of
        true -> SessionId;
        false when Index < 1000 -> session_id_with_remote_owner(Nodes, Index + 1);
        false -> error(no_remote_owner_session_id)
    end.

with_session_state_transfer(Fun) ->
    process_registry:init(),
    case whereis(session_state_transfer) of
        undefined ->
            {ok, Pid} = session_state_transfer:start_link(),
            try
                Fun()
            after
                try
                    gen_server:stop(Pid)
                catch
                    _:_ -> ok
                end
            end;
        _Pid ->
            Fun()
    end.
