%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(presence_connect).
-typing([eqwalizer]).

-export([
    handle_terminate_session_call/2,
    terminate_all_session_pids/2,
    force_terminate_all_sessions/1,
    handle_process_down/3,
    handle_join_guild/2,
    handle_leave_guild/2,
    handle_add_temporary_guild/2,
    remove_temporary_guild_id/2,
    kick_temporary_members_on_terminate/2,
    publish_offline_on_terminate/2,
    collect_session_pids/1
]).

-export_type([user_id/0, state/0]).

-type user_id() :: integer().
-type session_id() :: binary().
-type session_entry() :: map().
-type sessions() :: #{session_id() => session_entry()}.
-type state() :: map().

-spec handle_terminate_session_call([binary()], state()) ->
    {reply, term(), state()} | {stop, normal, ok, state()}.
handle_terminate_session_call(SessionIdHashes, State) ->
    case terminate_sessions_by_auth_hashes(SessionIdHashes, State) of
        {reply, Reply, NewState} -> {reply, Reply, NewState};
        {stop, ok, NewState} -> {stop, normal, ok, NewState}
    end.

-spec terminate_all_session_pids([binary()], state()) -> ok.
terminate_all_session_pids(SessionIdHashes, State) ->
    SessionPids = collect_session_pids(State),
    lists:foreach(
        fun(Pid) -> gen_server:cast(Pid, {terminate, SessionIdHashes}) end,
        SessionPids
    ).

-spec force_terminate_all_sessions(state()) -> ok.
force_terminate_all_sessions(State) ->
    SessionPids = collect_session_pids(State),
    lists:foreach(
        fun(Pid) -> gen_server:cast(Pid, {terminate_force}) end,
        SessionPids
    ).

-spec handle_process_down(reference(), term(), state()) ->
    {noreply, state()} | {stop, normal, state()}.
handle_process_down(Ref, _Reason, State) ->
    Sessions = maps:get(sessions, State),
    case presence_session:find_session_by_ref(Ref, Sessions) of
        {ok, SessionId} -> process_session_removal(SessionId, Sessions, State);
        not_found -> {noreply, State}
    end.

-spec handle_join_guild(integer(), state()) -> {reply, ok, state()}.
handle_join_guild(GuildId, State) ->
    Guilds = maps:get(guild_ids, State, #{}),
    case maps:is_key(GuildId, Guilds) of
        true ->
            {reply, ok, State};
        false ->
            NewState = State#{guild_ids := Guilds#{GuildId => true}},
            presence_session:notify_sessions_guild_join(GuildId, NewState),
            {reply, ok, NewState}
    end.

-spec handle_leave_guild(integer(), state()) -> {reply, ok, state()}.
handle_leave_guild(GuildId, State) ->
    Guilds = maps:get(guild_ids, State, #{}),
    case maps:is_key(GuildId, Guilds) of
        false ->
            {reply, ok, State};
        true ->
            TemporaryGuildIds = maps:get(temporary_guild_ids, State, #{}),
            NewState = State#{
                guild_ids := maps:remove(GuildId, Guilds),
                temporary_guild_ids := maps:remove(GuildId, TemporaryGuildIds)
            },
            presence_session:notify_sessions_guild_leave(GuildId, NewState),
            {reply, ok, NewState}
    end.

-spec handle_add_temporary_guild(integer(), state()) -> {reply, ok, state()}.
handle_add_temporary_guild(GuildId, State) ->
    {reply, JoinReply, JoinedState} = handle_join_guild(GuildId, State),
    TemporaryGuildIds = maps:get(temporary_guild_ids, JoinedState, #{}),
    NewState = JoinedState#{temporary_guild_ids := TemporaryGuildIds#{GuildId => true}},
    {reply, JoinReply, NewState}.

-spec remove_temporary_guild_id(integer(), state()) -> state().
remove_temporary_guild_id(GuildId, State) ->
    TemporaryGuildIds = maps:get(temporary_guild_ids, State, #{}),
    State#{temporary_guild_ids := maps:remove(GuildId, TemporaryGuildIds)}.

-spec kick_temporary_members_on_terminate(user_id(), state()) -> ok.
kick_temporary_members_on_terminate(UserId, State) ->
    TemporaryGuildIds = maps:get(temporary_guild_ids, State, #{}),
    case map_size(TemporaryGuildIds) of
        0 -> ok;
        _ -> spawn_kick_request(UserId, maps:keys(TemporaryGuildIds))
    end.

-spec publish_offline_on_terminate(user_id(), state()) -> ok.
publish_offline_on_terminate(UserId, State) ->
    LastPublished = maps:get(last_published_presence, State, undefined),
    case presence_broadcast:is_last_published_visible(LastPublished) of
        true ->
            UserData = user_utils:normalize_user(maps:get(user_data, State, #{})),
            Payload = #{
                <<"user">> => UserData,
                <<"status">> => <<"offline">>,
                <<"mobile">> => false,
                <<"afk">> => false,
                <<"custom_status">> => null
            },
            presence_bus:publish(UserId, Payload);
        false ->
            ok
    end.

-spec collect_session_pids(state()) -> [pid()].
collect_session_pids(State) ->
    Sessions = maps:get(sessions, State, #{}),
    [Pid || #{pid := Pid} <- maps:values(Sessions), is_pid(Pid)].

-spec terminate_sessions_by_auth_hashes([binary()], state()) ->
    {reply, ok | {error, term()}, state()} | {stop, ok, state()}.
terminate_sessions_by_auth_hashes(SessionIdHashes, State) ->
    Sessions = maps:get(sessions, State),
    {NewSessions, Removed, Errors} = fold_terminate_sessions(SessionIdHashes, Sessions),
    NewState0 = State#{sessions := NewSessions},
    NewState = maybe_publish_after_removal(Removed, NewSessions, NewState0),
    finalize_session_termination(Errors, Removed, NewSessions, NewState).

-spec fold_terminate_sessions([binary()], sessions()) -> {sessions(), boolean(), [term()]}.
fold_terminate_sessions(SessionIdHashes, Sessions) ->
    maps:fold(
        fun(SessionId, Session, {AccSessions, AccRemoved, AccErrors}) ->
            classify_termination(
                SessionId,
                Session,
                SessionIdHashes,
                AccSessions,
                AccRemoved,
                AccErrors
            )
        end,
        {#{}, false, []},
        Sessions
    ).

-spec classify_termination(
    session_id(),
    session_entry(),
    [binary()],
    sessions(),
    boolean(),
    [term()]
) -> {sessions(), boolean(), [term()]}.
classify_termination(
    SessionId,
    Session,
    SessionIdHashes,
    AccSessions,
    AccRemoved,
    AccErrors
) ->
    Pid = maps:get(pid, Session),
    case terminate_session_process(Pid, SessionIdHashes) of
        terminated ->
            demonitor(maps:get(mref, Session), [flush]),
            {AccSessions, true, AccErrors};
        gone ->
            demonitor(maps:get(mref, Session), [flush]),
            {AccSessions, true, AccErrors};
        ignored ->
            {AccSessions#{SessionId => Session}, AccRemoved, AccErrors};
        {error, Reason} ->
            {AccSessions#{SessionId => Session}, AccRemoved, [Reason | AccErrors]}
    end.

-spec maybe_publish_after_removal(boolean(), sessions(), state()) -> state().
maybe_publish_after_removal(true, NewSessions, State) ->
    presence_broadcast:publish_global_presence(NewSessions, State);
maybe_publish_after_removal(false, _NewSessions, State) ->
    State.

-spec finalize_session_termination([term()], boolean(), sessions(), state()) ->
    {reply, ok | {error, term()}, state()} | {stop, ok, state()}.
finalize_session_termination([], true, NewSessions, NewState) when
    map_size(NewSessions) =:= 0
->
    {stop, ok, NewState};
finalize_session_termination([], true, _NewSessions, NewState) ->
    presence_session:dispatch_sessions_replace(NewState),
    {reply, ok, NewState};
finalize_session_termination([], false, _NewSessions, NewState) ->
    {reply, ok, NewState};
finalize_session_termination([Error | _], _, _NewSessions, NewState) ->
    {reply, {error, Error}, NewState}.

-spec terminate_session_process(pid(), [binary()]) ->
    terminated | ignored | gone | {error, term()}.
terminate_session_process(Pid, SessionIdHashes) when is_pid(Pid) ->
    try gen_server:call(Pid, {terminate, SessionIdHashes}, 5000) of
        terminated -> terminated;
        ignored -> ignored;
        Other -> {error, {unexpected_terminate_reply, Other}}
    catch
        exit:{noproc, _} -> gone;
        exit:{nodedown, _} -> gone;
        exit:{normal, _} -> gone;
        exit:{timeout, _} -> {error, timeout};
        exit:Reason -> {error, Reason}
    end.

-spec process_session_removal(session_id(), sessions(), state()) ->
    {noreply, state()} | {stop, normal, state()}.
process_session_removal(SessionId, Sessions, State) ->
    NewSessions = maps:remove(SessionId, Sessions),
    NewState0 = State#{sessions := NewSessions},
    NewState = presence_broadcast:publish_global_presence(NewSessions, NewState0),
    case map_size(NewSessions) of
        0 ->
            {stop, normal, NewState};
        _ ->
            presence_session:dispatch_sessions_replace(NewState),
            {noreply, NewState}
    end.

-spec spawn_kick_request(user_id(), [integer()]) -> ok.
spawn_kick_request(UserId, GuildIdsList) ->
    spawn(fun() ->
        Request = #{
            <<"type">> => <<"kick_temporary_member">>,
            <<"user_id">> => type_conv:to_binary(UserId),
            <<"guild_ids">> => [type_conv:to_binary(Gid) || Gid <- GuildIdsList]
        },
        rpc_client:call(Request)
    end),
    ok.
