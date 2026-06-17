%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(cluster_multinode_remote_cases).
-typing([eqwalizer]).

-include_lib("common_test/include/ct.hrl").
-include_lib("eunit/include/eunit.hrl").

-export([
    remote_dispatch_relay_direct/1,
    remote_guild_handoff_ships_state/1,
    remote_presence_cache_rebalance/1,
    remote_presence_fanout/1,
    remote_session_state_transfer/1
]).

-spec remote_presence_fanout(list()) -> ok.
remote_presence_fanout(Config0) ->
    {ok, Peer1, Node1} = cluster_multinode_support:start_peer(node1, Config0),
    _Config = [{peers, [Peer1]} | Config0],
    true = net_kernel:connect_node(Node1),
    {ok, _LocalPgPid} = gateway_pg_scope:ensure_presence_scope(),
    {ok, _RemotePgPid} = peer:call(Peer1, gateway_pg_scope, ensure_presence_scope, []),
    Scope = gateway_pg_scope:presence_scope(),
    UserId = erlang:unique_integer([positive]),
    Group = {presence, UserId},
    Receiver = peer:call(
        Peer1,
        erlang,
        spawn,
        [cluster_multinode_support, presence_receiver_loop, [self()]]
    ),
    ok = peer:call(Peer1, pg, join, [Scope, Group, Receiver]),
    cluster_multinode_support:wait_until(
        fun() ->
            lists:member(Receiver, pg:get_members(Scope, Group))
        end,
        "remote presence pg membership"
    ),
    {ok, ShardPid} = presence_bus_shard:start_link(0),
    Payload = #{<<"status">> => <<"online">>},
    ?assertEqual(ok, gen_server:call(ShardPid, {publish, UserId, Payload}, 5000)),
    receive
        {remote_presence_received, Node1, UserId, Payload} ->
            ok
    after 5000 ->
        ct:fail({remote_presence_not_delivered, Node1, UserId})
    end,
    try
        gen_server:stop(ShardPid)
    catch
        _:_ -> ok
    end,
    peer:call(Peer1, erlang, send, [Receiver, stop]),
    ok.

-spec remote_dispatch_relay_direct(list()) -> ok.
remote_dispatch_relay_direct(Config0) ->
    {ok, Peer1, Node1} = cluster_multinode_support:start_peer(node1, Config0),
    _Config = [{peers, [Peer1]} | Config0],
    true = net_kernel:connect_node(Node1),
    Receiver = peer:call(
        Peer1,
        erlang,
        spawn,
        [cluster_multinode_support, dispatch_receiver_loop, [self()]]
    ),
    Payload = #{<<"id">> => <<"1">>},
    ok = gateway_dispatch_relay:dispatch_direct(Receiver, guild_update, Payload),
    receive
        {remote_dispatch_received, Node1, guild_update, Payload} ->
            ok
    after 5000 ->
        ct:fail({remote_dispatch_not_delivered, Node1})
    end,
    peer:call(Peer1, erlang, send, [Receiver, stop]),
    ok.

-spec remote_session_state_transfer(list()) -> ok.
remote_session_state_transfer(Config0) ->
    {ok, Peer1, Node1} = cluster_multinode_support:start_peer(node1, Config0),
    _Config = [{peers, [Peer1]} | Config0],
    true = net_kernel:connect_node(Node1),
    ok = cluster_multinode_support:ensure_remote_registered(
        Peer1, session_state_transfer, session_state_transfer, []
    ),
    SessionId = <<"ct-session-transfer">>,
    TransferState = #{token_hash => utils:hash_token(<<"resume-token">>), seq => 99},
    ?assertEqual(ok, session_state_transfer:push_state(Node1, SessionId, TransferState)),
    ?assertEqual(
        {ok, TransferState},
        peer:call(Peer1, session_state_transfer, pop_state, [SessionId])
    ),
    ok.

-spec remote_presence_cache_rebalance(list()) -> ok.
remote_presence_cache_rebalance(Config0) ->
    {ok, Peer1, Node1} = cluster_multinode_support:start_peer(node1, Config0),
    _Config = [{peers, [Peer1]} | Config0],
    true = net_kernel:connect_node(Node1),
    Members = lists:usort([node(), Node1]),
    RoleMembers = #{presence => Members},
    TermKeys = presence_cache_cluster_term_keys(),
    LocalTerms = save_local_terms(TermKeys),
    RemoteTerms = save_remote_terms(Peer1, TermKeys),
    put_presence_cache_cluster_terms(Members, RoleMembers),
    put_remote_presence_cache_cluster_terms(Peer1, Members, RoleMembers),
    {ok, LocalPresenceStarted} = cluster_multinode_support:ensure_local_registered(
        presence_cache, presence_cache, []
    ),
    ok = cluster_multinode_support:ensure_remote_registered(
        Peer1, presence_cache, presence_cache, []
    ),
    Presence = #{<<"status">> => <<"online">>, <<"user">> => #{<<"id">> => <<"1">>}},
    RemoteUserId = cluster_multinode_support:find_role_cache_owner_id(Members, presence, Node1),
    try
        ok = gen_server:call(presence_cache, {put_local, RemoteUserId, Presence}, 5000),
        ?assertMatch({ok, _}, gen_server:call(presence_cache, {get_local, RemoteUserId}, 5000)),
        ?assertEqual(ok, presence_cache:rebalance()),
        cluster_multinode_support:wait_until(
            fun() ->
                peer:call(
                    Peer1,
                    gen_server,
                    call,
                    [presence_cache, {get_local, RemoteUserId}, 5000]
                ) =:= {ok, Presence}
            end,
            "presence cache remote rebalance"
        ),
        ?assertEqual(
            not_found, gen_server:call(presence_cache, {get_local, RemoteUserId}, 5000)
        )
    after
        restore_local_terms(LocalTerms),
        restore_remote_terms(Peer1, RemoteTerms),
        cluster_multinode_support:maybe_stop_local(presence_cache, LocalPresenceStarted)
    end,
    ok.

presence_cache_cluster_term_keys() ->
    [
        {fluxer_gateway, runtime_config},
        {gateway_cluster_membership, members},
        {gateway_cluster_membership, members_by_role}
    ].

save_local_terms(Keys) ->
    [{Key, persistent_term:get(Key, undefined)} || Key <- Keys].

save_remote_terms(Peer, Keys) ->
    [
        {Key, peer:call(Peer, persistent_term, get, [Key, undefined])}
     || Key <- Keys
    ].

put_presence_cache_cluster_terms(Members, RoleMembers) ->
    persistent_term:put(
        {fluxer_gateway, runtime_config},
        runtime_config_with_role(
            persistent_term:get({fluxer_gateway, runtime_config}, undefined), presence
        )
    ),
    persistent_term:put({gateway_cluster_membership, members}, Members),
    persistent_term:put({gateway_cluster_membership, members_by_role}, RoleMembers).

put_remote_presence_cache_cluster_terms(Peer, Members, RoleMembers) ->
    RemoteRuntime = peer:call(
        Peer, persistent_term, get, [{fluxer_gateway, runtime_config}, undefined]
    ),
    ok = peer:call(Peer, persistent_term, put, [
        {fluxer_gateway, runtime_config}, runtime_config_with_role(RemoteRuntime, presence)
    ]),
    ok = peer:call(Peer, persistent_term, put, [
        {gateway_cluster_membership, members}, Members
    ]),
    ok = peer:call(Peer, persistent_term, put, [
        {gateway_cluster_membership, members_by_role}, RoleMembers
    ]).

runtime_config_with_role(Config, Role) when is_map(Config) ->
    Config#{gateway_role => Role};
runtime_config_with_role(_Config, Role) ->
    #{gateway_role => Role}.

restore_local_terms(Terms) ->
    lists:foreach(fun restore_local_term/1, Terms).

restore_local_term({Key, undefined}) ->
    persistent_term:erase(Key);
restore_local_term({Key, Value}) ->
    persistent_term:put(Key, Value).

restore_remote_terms(Peer, Terms) ->
    lists:foreach(fun(Term) -> restore_remote_term(Peer, Term) end, Terms).

restore_remote_term(Peer, {Key, undefined}) ->
    peer:call(Peer, persistent_term, erase, [Key]);
restore_remote_term(Peer, {Key, Value}) ->
    peer:call(Peer, persistent_term, put, [Key, Value]).

-spec remote_guild_handoff_ships_state(list()) -> ok.
remote_guild_handoff_ships_state(Config0) ->
    {Peer1, Node1, LocalGuildStarted, GuildId, SessionId, SessionPid, VoiceState, TransferState} =
        remote_guild_handoff_setup(Config0),
    try
        ?assertMatch(
            {ok, _},
            gen_server:call(
                guild_manager,
                {start_transferred, GuildId, TransferState},
                10000
            )
        ),
        ?assert(lists:member(GuildId, guild_manager:local_guild_ids())),
        ?assertEqual(
            #{attempted => 1, handed_off => 1},
            guild_manager:handoff_to_target(Node1)
        ),
        cluster_multinode_support:wait_until(
            fun() ->
                lists:member(GuildId, peer:call(Peer1, guild_manager, local_guild_ids, []))
            end,
            "remote guild registered after handoff"
        ),
        ?assertNot(lists:member(GuildId, guild_manager:local_guild_ids())),
        assert_remote_guild_handoff_state(Peer1, GuildId, SessionId, SessionPid, VoiceState)
    after
        SessionPid ! stop,
        cluster_multinode_support:maybe_stop_local(guild_manager, LocalGuildStarted)
    end,
    ok.

remote_guild_handoff_setup(Config0) ->
    {ok, Peer1, Node1} = cluster_multinode_support:start_peer(node1, Config0),
    _Config = [{peers, [Peer1]} | Config0],
    true = net_kernel:connect_node(Node1),
    process_registry:init(),
    passive_sync_registry:init(),
    {ok, LocalGuildStarted} = cluster_multinode_support:ensure_local_registered(
        guild_manager, guild_manager, []
    ),
    ok = peer:call(Peer1, process_registry, init, []),
    ok = peer:call(Peer1, passive_sync_registry, init, []),
    ok = cluster_multinode_support:ensure_remote_registered(
        Peer1, guild_manager, guild_manager, []
    ),
    GuildId = erlang:unique_integer([positive]) + 1000000,
    SessionId = <<"ct-guild-handoff-session">>,
    SessionPid = spawn(cluster_multinode_support, handoff_session_loop, [self()]),
    GuildData = cluster_multinode_support:minimal_guild_data(GuildId),
    VoiceState = #{<<"channel_id">> => <<"55">>, <<"user_id">> => <<"42">>},
    TransferState = #{
        id => GuildId,
        data => GuildData,
        sessions => #{
            SessionId => #{
                session_id => SessionId,
                user_id => 42,
                pid => SessionPid,
                mref => make_ref(),
                active_guilds => sets:new(),
                bot => false,
                is_staff => false,
                viewable_channels => #{}
            }
        },
        voice_states => #{<<"voice-ct">> => VoiceState},
        virtual_channel_access => #{55 => sets:from_list([42])}
    },
    {Peer1, Node1, LocalGuildStarted, GuildId, SessionId, SessionPid, VoiceState,
        TransferState}.

assert_remote_guild_handoff_state(Peer1, GuildId, SessionId, SessionPid, VoiceState) ->
    #{} = RemoteState = cluster_multinode_support:remote_guild_live_state(Peer1, GuildId),
    #{} = Sessions = maps:get(sessions, RemoteState),
    #{} = VoiceStates = maps:get(voice_states, RemoteState),
    #{} = VirtualChannelAccess = maps:get(virtual_channel_access, RemoteState),
    #{pid := SessionPid, mref := MonitorRef} = maps:get(SessionId, Sessions),
    ?assert(is_reference(MonitorRef)),
    ?assertEqual(#{<<"voice-ct">> => VoiceState}, VoiceStates),
    ?assertEqual(#{55 => sets:from_list([42])}, VirtualChannelAccess).
