%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(cluster_multinode_support).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

-export([
    dispatch_receiver_loop/1,
    drive_membership/2,
    ensure_local_registered/3,
    ensure_remote_registered/4,
    find_cache_owner_id/2,
    find_role_cache_owner_id/3,
    handoff_session_loop/1,
    maybe_stop_local/2,
    minimal_guild_data/1,
    presence_receiver_loop/1,
    remote_guild_live_state/2,
    rpc_members/1,
    start_peer/2,
    stop_peer/1,
    wait_until/2
]).

-type peer_ref() :: pid().

-export_type([peer_ref/0]).

-spec start_peer(atom(), list()) -> {ok, peer_ref(), node()}.
start_peer(NameSeed, _Config) ->
    Name = list_to_atom(
        lists:flatten(
            io_lib:format(
                "~p_~p_~p",
                [NameSeed, os:system_time(microsecond), rand:uniform(99999)]
            )
        )
    ),
    CodePaths = code:get_path(),
    {ok, Peer, Node} = peer:start_link(#{
        name => Name,
        host => "127.0.0.1",
        connection => standard_io,
        args => ["-setcookie", atom_to_list(fluxer_cluster_ct)]
    }),
    ok = peer:call(Peer, code, add_paths, [CodePaths]),
    {ok, _} = peer:call(
        Peer,
        gen_server,
        start,
        [
            {local, gateway_cluster_membership},
            gateway_cluster_membership,
            #{auto_subscribe => false},
            []
        ]
    ),
    {ok, Peer, Node}.

-spec ensure_local_registered(atom(), module(), [term()]) -> {ok, boolean()}.
ensure_local_registered(Name, Module, Args) ->
    case whereis(Name) of
        undefined ->
            {ok, _Pid} = gen_server:start({local, Name}, Module, Args, []),
            {ok, true};
        Pid when is_pid(Pid) ->
            {ok, false}
    end.

-spec ensure_remote_registered(peer_ref(), atom(), module(), [term()]) -> ok.
ensure_remote_registered(Peer, Name, Module, Args) ->
    case peer:call(Peer, erlang, whereis, [Name]) of
        undefined ->
            {ok, _Pid} = peer:call(Peer, gen_server, start, [{local, Name}, Module, Args, []]),
            ok;
        Pid when is_pid(Pid) ->
            ok
    end.

-spec maybe_stop_local(atom(), boolean()) -> ok.
maybe_stop_local(Name, true) ->
    case whereis(Name) of
        Pid when is_pid(Pid) ->
            try
                gen_server:stop(Pid)
            catch
                _:_ -> ok
            end,
            ok;
        undefined ->
            ok
    end;
maybe_stop_local(_Name, false) ->
    ok.

-spec stop_peer(peer_ref()) -> ok.
stop_peer(Peer) ->
    try
        peer:stop(Peer)
    catch
        _:_ -> ok
    end,
    ok.

-spec drive_membership(peer_ref(), [node()]) -> ok.
drive_membership(Peer, Peers) ->
    peer:call(Peer, erlang, send, [gateway_cluster_membership, {cluster_peers_changed, Peers}]),
    ok.

-spec rpc_members(peer_ref()) -> [node()].
rpc_members(Peer) ->
    peer:call(Peer, gateway_cluster_membership, members, []).

-spec presence_receiver_loop(pid()) -> ok.
presence_receiver_loop(Controller) ->
    receive
        {presence, UserId, Payload} ->
            Controller ! {remote_presence_received, node(), UserId, Payload},
            presence_receiver_loop(Controller);
        stop ->
            ok
    after infinity ->
        ok
    end.

-spec dispatch_receiver_loop(pid()) -> ok.
dispatch_receiver_loop(Controller) ->
    receive
        {'$gen_cast', {dispatch, Event, Payload}} ->
            Controller ! {remote_dispatch_received, node(), Event, Payload},
            dispatch_receiver_loop(Controller);
        stop ->
            ok
    after infinity ->
        ok
    end.

-spec handoff_session_loop(pid()) -> ok.
handoff_session_loop(Controller) ->
    receive
        {'$gen_cast', Msg} ->
            Controller ! {handoff_session_cast, Msg},
            handoff_session_loop(Controller);
        stop ->
            ok;
        _Other ->
            handoff_session_loop(Controller)
    after infinity ->
        ok
    end.

-spec find_cache_owner_id([node()], node()) -> pos_integer().
find_cache_owner_id(Members, OwnerNode) ->
    PreviousMembers = persistent_term:get({gateway_cluster_membership, members}, undefined),
    persistent_term:put({gateway_cluster_membership, members}, Members),
    try
        hd([
            Id
         || Id <- lists:seq(1, 5000),
            clustered_ets_cache:resolve_owner_nodes(Id, 1) =:= [OwnerNode]
        ])
    after
        case PreviousMembers of
            undefined -> persistent_term:erase({gateway_cluster_membership, members});
            _ -> persistent_term:put({gateway_cluster_membership, members}, PreviousMembers)
        end
    end.

-spec find_role_cache_owner_id([node()], atom(), node()) -> pos_integer().
find_role_cache_owner_id(Members, Role, OwnerNode) ->
    MembersKey = {gateway_cluster_membership, members},
    RoleMembersKey = {gateway_cluster_membership, members_by_role},
    PreviousMembers = persistent_term:get(MembersKey, undefined),
    PreviousRoleMembers = persistent_term:get(RoleMembersKey, undefined),
    persistent_term:put(MembersKey, Members),
    persistent_term:put(RoleMembersKey, #{Role => Members}),
    try
        hd([
            Id
         || Id <- lists:seq(1, 5000),
            clustered_ets_cache:resolve_owner_nodes(Id, 1, Role) =:= [OwnerNode]
        ])
    after
        restore_persistent_term(MembersKey, PreviousMembers),
        restore_persistent_term(RoleMembersKey, PreviousRoleMembers)
    end.

-spec restore_persistent_term(term(), term()) -> ok.
restore_persistent_term(Key, undefined) ->
    _ = persistent_term:erase(Key),
    ok;
restore_persistent_term(Key, Value) ->
    persistent_term:put(Key, Value).

-spec minimal_guild_data(integer()) -> map().
minimal_guild_data(GuildId) ->
    GuildIdBin = integer_to_binary(GuildId),
    #{
        <<"guild">> => #{
            <<"id">> => GuildIdBin,
            <<"owner_id">> => <<"42">>,
            <<"features">> => []
        },
        <<"members">> => [
            #{
                <<"user">> => #{<<"id">> => <<"42">>},
                <<"roles">> => []
            }
        ],
        <<"roles">> => [],
        <<"channels">> => []
    }.

-spec remote_guild_live_state(peer_ref(), integer()) -> term().
remote_guild_live_state(Peer, GuildId) ->
    GuildKey = process_registry:build_process_key(guild, GuildId),
    GuildPid = peer:call(Peer, process_registry, registry_whereis, [GuildKey]),
    ?assert(is_pid(GuildPid), {remote_guild_not_registered, GuildId}),
    peer:call(Peer, gen_server, call, [GuildPid, {get_sessions}, 5000]).

-spec wait_until(fun(() -> boolean()), term()) -> ok.
wait_until(Pred, Label) ->
    wait_until(Pred, Label, 50, 100).

-spec wait_until(fun(() -> boolean()), term(), non_neg_integer(), pos_integer()) -> ok.
wait_until(_Pred, Label, 0, _IntervalMs) ->
    ct:fail({wait_timeout, Label});
wait_until(Pred, Label, Retries, IntervalMs) ->
    case eval_predicate(Pred) of
        true ->
            ok;
        _ ->
            timer:sleep(IntervalMs),
            wait_until(Pred, Label, Retries - 1, IntervalMs)
    end.

eval_predicate(Pred) ->
    try
        Pred()
    catch
        _:_ -> false
    end.
