%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_sessions).
-typing([eqwalizer]).

-export([
    handle_session_connect/3,
    handle_session_down/2,
    remove_session/2,
    filter_sessions_for_channel/4,
    filter_sessions_for_message/5,
    filter_sessions_for_manage_channels/4,
    filter_sessions_exclude_session/2,
    handle_user_offline/2,
    set_session_active_guild/3,
    set_session_passive_guild/3,
    build_initial_last_message_ids/1,
    is_session_active/2,
    subscribe_connected_user_presence/2,
    subscribe_to_user_presence/2,
    unsubscribe_from_user_presence/2,
    set_session_viewable_channels/3,
    refresh_user_session_cache/2,
    refresh_all_viewable_channels/1,
    handle_set_typing_override/3,
    handle_send_guild_sync/2,
    handle_send_members_chunk/3,
    build_viewable_channel_map/1
]).

-type guild_state() :: map().
-type session_id() :: binary().
-type user_id() :: integer().
-type guild_id() :: integer().
-type channel_id() :: integer().
-type session_data() :: map().
-type sessions_map() :: #{session_id() => session_data()}.
-type session_pair() :: {session_id(), session_data()}.
-export_type([
    guild_state/0,
    session_id/0,
    user_id/0,
    guild_id/0,
    channel_id/0,
    sessions_map/0,
    session_pair/0
]).

-spec handle_session_connect(map(), pid(), guild_state()) ->
    {reply,
        {ok, map()}
        | {ok, unavailable, map()}
        | {error, too_many_sessions}
        | {error, not_member},
        guild_state()}.
handle_session_connect(Request, Pid, State) ->
    guild_sessions_connect:handle_session_connect(Request, Pid, State).

-spec handle_session_down(reference(), guild_state()) ->
    {noreply, guild_state()} | {stop, normal, guild_state()}.
handle_session_down(Ref, State) ->
    guild_sessions_connect:handle_session_down(Ref, State).

-spec remove_session(session_id(), guild_state()) -> guild_state().
remove_session(SessionId, State) ->
    guild_sessions_connect:remove_session(SessionId, State).

-spec build_initial_last_message_ids(map()) -> #{binary() => binary()}.
build_initial_last_message_ids(GuildState) ->
    guild_sessions_connect:build_initial_last_message_ids(GuildState).

-spec subscribe_connected_user_presence(user_id(), guild_state()) -> guild_state().
subscribe_connected_user_presence(UserId, State) ->
    guild_sessions_presence:subscribe_connected_user_presence(UserId, State).

-spec subscribe_to_user_presence(user_id(), guild_state()) -> guild_state().
subscribe_to_user_presence(UserId, State) ->
    guild_sessions_presence:subscribe_to_user_presence(UserId, State).

-spec unsubscribe_from_user_presence(user_id(), guild_state()) -> guild_state().
unsubscribe_from_user_presence(UserId, State) ->
    guild_sessions_presence:unsubscribe_from_user_presence(UserId, State).

-spec handle_user_offline(user_id(), guild_state()) -> guild_state().
handle_user_offline(UserId, State) ->
    guild_sessions_presence:handle_user_offline(UserId, State).

-spec set_session_active_guild(session_id(), guild_id(), guild_state()) -> guild_state().
set_session_active_guild(SessionId, GuildId, State) ->
    guild_sessions_passive:set_session_active_guild(SessionId, GuildId, State).

-spec set_session_passive_guild(session_id(), guild_id(), guild_state()) -> guild_state().
set_session_passive_guild(SessionId, GuildId, State) ->
    guild_sessions_passive:set_session_passive_guild(SessionId, GuildId, State).

-spec is_session_active(session_id(), guild_state()) -> boolean().
is_session_active(SessionId, State) ->
    guild_sessions_passive:is_session_active(SessionId, State).

-spec handle_set_typing_override(session_id(), boolean(), guild_state()) -> guild_state().
handle_set_typing_override(SessionId, TypingFlag, State) ->
    guild_sessions_passive:handle_set_typing_override(SessionId, TypingFlag, State).

-spec handle_send_guild_sync(session_id(), guild_state()) -> guild_state().
handle_send_guild_sync(SessionId, State) ->
    guild_sessions_passive:handle_send_guild_sync(SessionId, State).

-spec handle_send_members_chunk(session_id(), map(), guild_state()) -> ok.
handle_send_members_chunk(SessionId, ChunkData, State) ->
    guild_sessions_passive:handle_send_members_chunk(SessionId, ChunkData, State).

-spec filter_sessions_for_channel(
    sessions_map(), channel_id(), session_id() | undefined, guild_state()
) -> [session_pair()].
filter_sessions_for_channel(Sessions, ChannelId, SessionIdOpt, State) ->
    filter_active_sessions(Sessions, SessionIdOpt, fun(S, _Sid) ->
        session_can_view_channel(S, ChannelId, State)
    end).

-spec filter_sessions_for_message(
    sessions_map(), channel_id(), binary(), session_id() | undefined, guild_state()
) -> [session_pair()].
filter_sessions_for_message(Sessions, ChannelId, MessageId, SessionIdOpt, State) ->
    filter_active_sessions(Sessions, SessionIdOpt, fun(S, _Sid) ->
        session_can_view_channel(S, ChannelId, State) andalso
            session_can_access_message(S, ChannelId, MessageId, State)
    end).

-spec session_can_access_message(map(), channel_id(), binary(), guild_state()) -> boolean().
session_can_access_message(SessionData, ChannelId, MessageId, State) ->
    case maps:get(user_id, SessionData, undefined) of
        UserId when is_integer(UserId) ->
            Perms = guild_permissions:get_member_permissions(UserId, ChannelId, State),
            guild_permissions:can_access_message_by_permissions(Perms, MessageId, State);
        _ ->
            false
    end.

-spec filter_sessions_for_manage_channels(
    sessions_map(), channel_id(), session_id() | undefined, guild_state()
) -> [session_pair()].
filter_sessions_for_manage_channels(Sessions, ChannelId, SessionIdOpt, State) ->
    filter_active_sessions(Sessions, SessionIdOpt, fun(S, _Sid) ->
        UserId = maps:get(user_id, S),
        guild_permissions:can_manage_channel(UserId, ChannelId, State)
    end).

-spec filter_active_sessions(
    sessions_map(),
    session_id() | undefined,
    fun((session_data(), session_id()) -> boolean())
) -> [session_pair()].
filter_active_sessions(Sessions, SessionIdOpt, Pred) ->
    maps:fold(
        fun(Sid, S, Acc) ->
            collect_active_session(Sid, S, SessionIdOpt, Pred, Acc)
        end,
        [],
        Sessions
    ).

-spec collect_active_session(
    session_id(),
    session_data(),
    session_id() | undefined,
    fun((session_data(), session_id()) -> boolean()),
    [session_pair()]
) -> [session_pair()].
collect_active_session(Sid, S, SessionIdOpt, Pred, Acc) ->
    case not is_pending_or_excluded(Sid, S, SessionIdOpt) andalso Pred(S, Sid) of
        true -> [{Sid, S} | Acc];
        false -> Acc
    end.

-spec is_pending_or_excluded(session_id(), session_data(), session_id() | undefined) ->
    boolean().
is_pending_or_excluded(Sid, S, SessionIdOpt) ->
    maps:get(pending_connect, S, false) orelse
        should_exclude_session(Sid, SessionIdOpt).

-spec filter_sessions_exclude_session(sessions_map(), session_id() | undefined) ->
    [session_pair()].
filter_sessions_exclude_session(Sessions, SessionIdOpt) ->
    maps:fold(
        fun(Sid, S, Acc) ->
            collect_non_excluded(Sid, S, SessionIdOpt, Acc)
        end,
        [],
        Sessions
    ).

-spec collect_non_excluded(
    session_id(), session_data(), session_id() | undefined, [session_pair()]
) -> [session_pair()].
collect_non_excluded(Sid, S, SessionIdOpt, Acc) ->
    Excluded = is_pending_or_excluded(Sid, S, SessionIdOpt),
    case not Excluded of
        true -> [{Sid, S} | Acc];
        false -> Acc
    end.

-spec should_exclude_session(session_id(), session_id() | undefined) -> boolean().
should_exclude_session(_, undefined) -> false;
should_exclude_session(Sid, SessionId) -> Sid =:= SessionId.

-spec set_session_viewable_channels(session_id(), map(), guild_state()) -> guild_state().
set_session_viewable_channels(SessionId, ViewableChannels, State) ->
    Sessions = maps:get(sessions, State, #{}),
    case maps:get(SessionId, Sessions, undefined) of
        undefined ->
            State;
        SessionData ->
            NewSessionData = SessionData#{viewable_channels => ViewableChannels},
            NewSessions = Sessions#{SessionId => NewSessionData},
            State#{sessions => NewSessions}
    end.

-spec refresh_user_session_cache(user_id(), guild_state()) -> guild_state().
refresh_user_session_cache(UserId, State) when is_integer(UserId), UserId > 0 ->
    Sessions = maps:get(sessions, State, #{}),
    UserRoles = session_passive:get_user_roles_for_guild(UserId, State),
    ViewableChannels = build_viewable_channel_map(
        guild_visibility:get_user_viewable_channels(UserId, State)
    ),
    NewSessions = maps:map(
        fun(_SessionId, SessionData) ->
            maybe_refresh_user_session_cache(UserId, UserRoles, ViewableChannels, SessionData)
        end,
        Sessions
    ),
    State#{sessions => NewSessions};
refresh_user_session_cache(_UserId, State) ->
    State.

-spec maybe_refresh_user_session_cache(user_id(), [integer()], map(), session_data()) ->
    session_data().
maybe_refresh_user_session_cache(UserId, UserRoles, ViewableChannels, SessionData) ->
    case maps:get(user_id, SessionData, undefined) of
        UserId ->
            SessionData#{
                user_roles => UserRoles,
                viewable_channels => ViewableChannels
            };
        _ ->
            SessionData
    end.

-spec refresh_all_viewable_channels(guild_state()) -> guild_state().
refresh_all_viewable_channels(State) ->
    guild_sessions_connect:invalidate_viewable_channels_cache(State),
    Sessions = maps:get(sessions, State, #{}),
    maps:fold(
        fun refresh_session_viewable/3,
        State,
        Sessions
    ).

-spec refresh_session_viewable(session_id(), session_data(), guild_state()) -> guild_state().
refresh_session_viewable(SessionId, SessionData, AccState) ->
    UserId = maps:get(user_id, SessionData, undefined),
    case is_integer(UserId) of
        true ->
            ViewableChannels = build_viewable_channel_map(
                guild_visibility:get_user_viewable_channels(UserId, AccState)
            ),
            set_session_viewable_channels(SessionId, ViewableChannels, AccState);
        false ->
            AccState
    end.

-spec session_can_view_channel(session_data(), channel_id(), guild_state()) -> boolean().
session_can_view_channel(SessionData, ChannelId, State) ->
    UserId = maps:get(user_id, SessionData, undefined),
    case {UserId, maps:get(viewable_channels, SessionData, undefined)} of
        {Uid, ViewableChannels} when is_integer(Uid), is_map(ViewableChannels) ->
            maps:is_key(ChannelId, ViewableChannels) orelse
                check_member_channel_access(Uid, ChannelId, State);
        {Uid, _} when is_integer(Uid) ->
            check_member_channel_access(Uid, ChannelId, State);
        _ ->
            false
    end.

-spec check_member_channel_access(user_id(), channel_id(), guild_state()) -> boolean().
check_member_channel_access(UserId, ChannelId, State) ->
    Member = guild_permissions:find_member_by_user_id(UserId, State),
    case Member of
        undefined -> false;
        _ -> guild_permissions:can_view_channel(UserId, ChannelId, Member, State)
    end.

-spec build_viewable_channel_map([channel_id()]) -> #{channel_id() => true}.
build_viewable_channel_map(ChannelIds) ->
    lists:foldl(
        fun(ChannelId, Acc) -> Acc#{ChannelId => true} end,
        #{},
        ChannelIds
    ).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

should_exclude_session_test() ->
    ?assertEqual(false, should_exclude_session(<<"s1">>, undefined)),
    ?assertEqual(true, should_exclude_session(<<"s1">>, <<"s1">>)),
    ?assertEqual(false, should_exclude_session(<<"s1">>, <<"s2">>)).

is_pending_or_excluded_pending_test() ->
    S = #{pending_connect => true},
    ?assertEqual(true, is_pending_or_excluded(<<"s1">>, S, undefined)).

is_pending_or_excluded_excluded_test() ->
    S = #{},
    ?assertEqual(true, is_pending_or_excluded(<<"s1">>, S, <<"s1">>)).

is_pending_or_excluded_neither_test() ->
    S = #{},
    ?assertEqual(false, is_pending_or_excluded(<<"s1">>, S, <<"s2">>)),
    ?assertEqual(false, is_pending_or_excluded(<<"s1">>, S, undefined)).

is_pending_or_excluded_pending_false_test() ->
    S = #{pending_connect => false},
    ?assertEqual(false, is_pending_or_excluded(<<"s1">>, S, undefined)).

filter_active_sessions_test() ->
    S1 = #{user_id => 1, pending_connect => false},
    S2 = #{user_id => 2, pending_connect => true},
    S3 = #{user_id => 3},
    Sessions = #{<<"a">> => S1, <<"b">> => S2, <<"c">> => S3},
    Result = filter_active_sessions(Sessions, <<"a">>, fun(_S, _Sid) -> true end),
    ResultSids = lists:sort([Sid || {Sid, _} <- Result]),
    ?assertEqual([<<"c">>], ResultSids).

filter_active_sessions_with_predicate_test() ->
    S1 = #{user_id => 1},
    S2 = #{user_id => 2},
    Sessions = #{<<"a">> => S1, <<"b">> => S2},
    Result = filter_active_sessions(Sessions, undefined, fun(S, _Sid) ->
        maps:get(user_id, S) =:= 2
    end),
    ?assertEqual(1, length(Result)),
    [{<<"b">>, _}] = Result.

-endif.
