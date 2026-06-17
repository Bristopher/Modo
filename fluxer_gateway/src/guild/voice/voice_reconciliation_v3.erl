%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(voice_reconciliation_v3).
-typing([eqwalizer]).

-export([
    schedule_tick/1,
    interval_ms/0,
    enabled_for/2,
    find_absent_guild_connections/1,
    find_absent_call_entries/1,
    find_absent_entries/2
]).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").
-endif.

-export_type([
    owner_kind/0,
    participant_entry/0,
    room_key/0,
    snapshot_fun/0
]).

-type owner_kind() :: guild | call.
-type room_key() :: {integer() | null, integer(), binary(), binary()}.
-type participant_entry() :: #{
    connection_id := binary(),
    user_id := integer(),
    channel_id := integer(),
    guild_id := integer() | null,
    region_id := binary(),
    server_id := binary(),
    pending := boolean()
}.
-type snapshot_fun() ::
    fun((integer() | null, integer(), binary(), binary()) -> {ok, term()} | {error, term()}).

-define(DEFAULT_INTERVAL_MS, 2000).

-spec schedule_tick(term()) -> reference().
schedule_tick(Message) ->
    erlang:send_after(jittered_interval_ms(), self(), Message).

-spec interval_ms() -> pos_integer().
interval_ms() ->
    normalize_interval(gateway_rollout_config:voice_reconciliation_v3_interval_ms()).

-spec enabled_for(owner_kind(), integer()) -> boolean().
enabled_for(_Kind, OwnerId) when is_integer(OwnerId), OwnerId > 0 ->
    Percentage = gateway_rollout_config:voice_reconciliation_v3_percentage(),
    Percentage > 0 andalso
        (Percentage >= 100 orelse erlang:phash2(integer_to_binary(OwnerId), 100) < Percentage);
enabled_for(_Kind, _OwnerId) ->
    false.

-spec find_absent_guild_connections(map()) -> [binary()].
find_absent_guild_connections(State) ->
    GuildId = maps:get(guild_id, State, maps:get(id, State, undefined)),
    VoiceStates = voice_state_utils:voice_states(State),
    Pending = ensure_map(maps:get(pending_voice_connections, State, #{})),
    Entries = guild_entries(GuildId, VoiceStates, Pending),
    Absent = find_absent_entries(Entries, snapshot_fun(State)),
    [maps:get(connection_id, Entry) || Entry <- Absent].

-spec find_absent_call_entries(map()) -> [participant_entry()].
find_absent_call_entries(#{channel_id := ChannelId} = State) when
    is_integer(ChannelId), ChannelId > 0
->
    VoiceStates = ensure_map(maps:get(voice_states, State, #{})),
    Pending = ensure_map(maps:get(pending_connections, State, #{})),
    Entries = call_entries(ChannelId, VoiceStates, Pending),
    find_absent_entries(Entries, snapshot_fun(State));
find_absent_call_entries(_State) ->
    [].

-spec find_absent_entries([participant_entry()], snapshot_fun()) -> [participant_entry()].
find_absent_entries(Entries, SnapshotFun) ->
    RoomGroups = group_entries_by_room([E || E <- Entries, not maps:get(pending, E)]),
    maps:fold(
        fun(RoomKey, RoomEntries, Acc) ->
            find_absent_entries_for_room(RoomKey, RoomEntries, SnapshotFun, Acc)
        end,
        [],
        RoomGroups
    ).

-spec find_absent_entries_for_room(room_key(), [participant_entry()], snapshot_fun(), [
    participant_entry()
]) ->
    [participant_entry()].
find_absent_entries_for_room(
    {GuildId, ChannelId, RegionId, ServerId} = RoomKey, RoomEntries, SnapshotFun, Acc
) ->
    case fetch_present_connections(SnapshotFun, GuildId, ChannelId, RegionId, ServerId) of
        {ok, Present} ->
            absent_from_present(RoomEntries, Present, Acc);
        {error, Reason} ->
            log_snapshot_error(RoomKey, Reason),
            Acc
    end.

-spec absent_from_present([participant_entry()], map(), [participant_entry()]) ->
    [participant_entry()].
absent_from_present(RoomEntries, Present, Acc) ->
    lists:foldl(
        fun(Entry, EntryAcc) -> prepend_if_absent(Entry, Present, EntryAcc) end,
        Acc,
        RoomEntries
    ).

-spec prepend_if_absent(participant_entry(), map(), [participant_entry()]) ->
    [participant_entry()].
prepend_if_absent(Entry, Present, Acc) ->
    case maps:is_key(maps:get(connection_id, Entry), Present) of
        true -> Acc;
        false -> [Entry | Acc]
    end.

-spec guild_entries(term(), map(), map()) -> [participant_entry()].
guild_entries(GuildId, VoiceStates, Pending) ->
    case normalize_guild_id(GuildId) of
        undefined ->
            [];
        NormalizedGuildId ->
            guild_entries_for_id(NormalizedGuildId, VoiceStates, Pending)
    end.

-spec guild_entries_for_id(integer(), map(), map()) -> [participant_entry()].
guild_entries_for_id(GuildId, VoiceStates, Pending) ->
    maps:fold(
        fun(ConnId, VoiceState, Acc) ->
            prepend_entry(
                entry_from_voice_state(GuildId, ConnId, undefined, VoiceState, Pending),
                Acc
            )
        end,
        [],
        VoiceStates
    ).

-spec call_entries(integer(), map(), map()) -> [participant_entry()].
call_entries(ChannelId, VoiceStates, Pending) ->
    maps:fold(
        fun(UserId, VoiceState, Acc) ->
            ConnId = normalize_connection_id(
                maps:get(
                    <<"connection_id">>,
                    VoiceState,
                    maps:get(connection_id, VoiceState, undefined)
                )
            ),
            prepend_entry(
                entry_from_voice_state(null, ConnId, UserId, VoiceState, Pending, ChannelId),
                Acc
            )
        end,
        [],
        VoiceStates
    ).

-spec entry_from_voice_state(
    integer() | null, term(), term(), term(), map()
) -> {ok, participant_entry()} | skip.
entry_from_voice_state(GuildId, ConnId, FallbackUserId, VoiceState, Pending) ->
    entry_from_voice_state(GuildId, ConnId, FallbackUserId, VoiceState, Pending, undefined).

-spec entry_from_voice_state(
    integer() | null, term(), term(), term(), map(), integer() | undefined
) -> {ok, participant_entry()} | skip.
entry_from_voice_state(
    GuildId, ConnId0, FallbackUserId, VoiceState, Pending, FallbackChannelId
) when
    is_map(VoiceState)
->
    ConnId = normalize_connection_id(ConnId0),
    UserId = normalize_user_id(VoiceState, FallbackUserId),
    ChannelId = normalize_channel_id(VoiceState, FallbackChannelId),
    RegionId = normalize_optional_binary(
        maps:get(<<"region_id">>, VoiceState, maps:get(region_id, VoiceState, undefined))
    ),
    ServerId = normalize_optional_binary(
        maps:get(<<"server_id">>, VoiceState, maps:get(server_id, VoiceState, undefined))
    ),
    case {ConnId, UserId, ChannelId, RegionId, ServerId} of
        {C, U, Ch, R, S} when
            is_binary(C),
            is_integer(U),
            U > 0,
            is_integer(Ch),
            Ch > 0,
            is_binary(R),
            is_binary(S)
        ->
            {ok, #{
                connection_id => C,
                user_id => U,
                channel_id => Ch,
                guild_id => GuildId,
                region_id => R,
                server_id => S,
                pending => maps:is_key(C, Pending)
            }};
        _ ->
            skip
    end;
entry_from_voice_state(_GuildId, _ConnId, _FallbackUserId, _VoiceState, _Pending, _ChannelId) ->
    skip.

-spec prepend_entry({ok, participant_entry()} | skip, [participant_entry()]) ->
    [participant_entry()].
prepend_entry({ok, Entry}, Acc) -> [Entry | Acc];
prepend_entry(skip, Acc) -> Acc.

-spec group_entries_by_room([participant_entry()]) -> #{room_key() => [participant_entry()]}.
group_entries_by_room(Entries) ->
    lists:foldl(
        fun(Entry, Acc) ->
            Key = room_key(Entry),
            Acc#{Key => [Entry | maps:get(Key, Acc, [])]}
        end,
        #{},
        Entries
    ).

-spec room_key(participant_entry()) -> room_key().
room_key(Entry) ->
    {
        maps:get(guild_id, Entry),
        maps:get(channel_id, Entry),
        maps:get(region_id, Entry),
        maps:get(server_id, Entry)
    }.

-spec fetch_present_connections(
    snapshot_fun(), integer() | null, integer(), binary(), binary()
) -> {ok, map()} | {error, term()}.
fetch_present_connections(SnapshotFun, GuildId, ChannelId, RegionId, ServerId) ->
    try SnapshotFun(GuildId, ChannelId, RegionId, ServerId) of
        {ok, Snapshot} -> {ok, connection_set(Snapshot)};
        {error, Reason} -> {error, Reason};
        Other -> {error, {unexpected_snapshot_result, Other}}
    catch
        Class:Reason -> {error, {Class, Reason}}
    end.

-spec snapshot_fun(map()) -> snapshot_fun().
snapshot_fun(State) ->
    case maps:get(test_voice_reconciliation_v3_fun, State, undefined) of
        Fun when is_function(Fun, 4) -> Fun;
        _ -> fun fetch_snapshot_from_rpc/4
    end.

-spec fetch_snapshot_from_rpc(integer() | null, integer(), binary(), binary()) ->
    {ok, term()} | {error, term()}.
fetch_snapshot_from_rpc(GuildId, ChannelId, RegionId, ServerId) ->
    Req = voice_utils:build_list_participants_rpc_request(
        GuildId, ChannelId, RegionId, ServerId
    ),
    case rpc_client:call(Req) of
        {ok, #{<<"status">> := <<"ok">>, <<"participants">> := Participants}} ->
            {ok, Participants};
        {ok, #{<<"status">> := <<"error">>} = ErrorData} ->
            {error, ErrorData};
        {ok, Other} ->
            {error, {unexpected_rpc_response, Other}};
        {error, Reason} ->
            {error, Reason}
    end.

-spec connection_set(term()) -> map().
connection_set(#{<<"participants">> := Participants}) ->
    connection_set(Participants);
connection_set(#{participants := Participants}) ->
    connection_set(Participants);
connection_set(Participants) when is_list(Participants) ->
    lists:foldl(fun add_snapshot_connection/2, #{}, Participants);
connection_set(_) ->
    #{}.

-spec add_snapshot_connection(term(), map()) -> map().
add_snapshot_connection(ConnectionId, Acc) when
    is_binary(ConnectionId), byte_size(ConnectionId) > 0
->
    Acc#{ConnectionId => true};
add_snapshot_connection(Participant, Acc) when is_map(Participant) ->
    case participant_connection_id(Participant) of
        undefined -> Acc;
        ConnectionId -> Acc#{ConnectionId => true}
    end;
add_snapshot_connection(_Participant, Acc) ->
    Acc.

-spec participant_connection_id(map()) -> binary() | undefined.
participant_connection_id(Participant) ->
    case
        normalize_connection_id(
            maps:get(
                <<"connection_id">>,
                Participant,
                maps:get(connection_id, Participant, undefined)
            )
        )
    of
        undefined ->
            parse_identity_connection_id(
                maps:get(
                    <<"identity">>, Participant, maps:get(identity, Participant, undefined)
                )
            );
        ConnectionId ->
            ConnectionId
    end.

-spec parse_identity_connection_id(term()) -> binary() | undefined.
parse_identity_connection_id(Identity) when is_binary(Identity) ->
    parse_identity_parts(binary:split(Identity, <<"_">>, [global]));
parse_identity_connection_id(_) ->
    undefined.

-spec parse_identity_parts([binary()]) -> binary() | undefined.
parse_identity_parts([<<"user">>, UserId, ConnId]) when
    byte_size(UserId) > 0, byte_size(ConnId) > 0
->
    ConnId;
parse_identity_parts([<<"user">>, UserId | Rest]) when byte_size(UserId) > 0, Rest =/= [] ->
    nonempty_joined_connection_id(Rest);
parse_identity_parts(_) ->
    undefined.

-spec nonempty_joined_connection_id([binary()]) -> binary() | undefined.
nonempty_joined_connection_id(Parts) ->
    case join_binary(Parts, <<"_">>) of
        <<>> -> undefined;
        ConnId -> ConnId
    end.

-spec join_binary([binary()], binary()) -> binary().
join_binary([Part], _Separator) ->
    Part;
join_binary([Part | Rest], Separator) ->
    <<Part/binary, Separator/binary, (join_binary(Rest, Separator))/binary>>.

-spec normalize_user_id(map(), term()) -> integer() | undefined.
normalize_user_id(VoiceState, FallbackUserId) ->
    case voice_state_utils:voice_state_user_id(VoiceState) of
        undefined -> normalize_positive_snowflake(FallbackUserId);
        UserId -> UserId
    end.

-spec normalize_channel_id(map(), integer() | undefined) -> integer() | undefined.
normalize_channel_id(VoiceState, FallbackChannelId) ->
    case voice_state_utils:voice_state_channel_id(VoiceState) of
        undefined -> FallbackChannelId;
        ChannelId -> ChannelId
    end.

-spec normalize_guild_id(term()) -> integer() | undefined.
normalize_guild_id(GuildId) ->
    normalize_positive_snowflake(GuildId).

-spec normalize_positive_snowflake(term()) -> integer() | undefined.
normalize_positive_snowflake(Value) ->
    guild_voice_connection_normalize:normalize_positive_snowflake(Value).

-spec normalize_connection_id(term()) -> binary() | undefined.
normalize_connection_id(Value) ->
    normalize_optional_binary(Value).

-spec normalize_optional_binary(term()) -> binary() | undefined.
normalize_optional_binary(Value) ->
    guild_voice_connection_normalize:normalize_optional_binary(Value).

-spec normalize_interval(term()) -> pos_integer().
normalize_interval(Value) when is_integer(Value), Value >= 500, Value =< 60000 ->
    Value;
normalize_interval(_) ->
    ?DEFAULT_INTERVAL_MS.

-spec jittered_interval_ms() -> pos_integer().
jittered_interval_ms() ->
    Interval = interval_ms(),
    Jitter = min(250, max(0, Interval div 10)),
    Interval + erlang:phash2(self(), Jitter + 1).

-spec ensure_map(term()) -> map().
ensure_map(Map) when is_map(Map) -> Map;
ensure_map(_) -> #{}.

-spec log_snapshot_error(room_key(), term()) -> ok.
log_snapshot_error({GuildId, ChannelId, RegionId, ServerId}, Reason) ->
    logger:debug(
        "voice_reconciliation_v3_snapshot_error:"
        " guild_id=~p channel_id=~p region_id=~p server_id=~p reason=~p",
        [GuildId, ChannelId, RegionId, ServerId, Reason]
    ),
    ok.

-ifdef(TEST).

voice_state(UserId, ChannelId, ConnId, RegionId, ServerId) ->
    #{
        <<"user_id">> => integer_to_binary(UserId),
        <<"channel_id">> => integer_to_binary(ChannelId),
        <<"connection_id">> => ConnId,
        <<"region_id">> => RegionId,
        <<"server_id">> => ServerId
    }.

find_absent_guild_connections_skips_pending_test() ->
    State = #{
        guild_id => 10,
        voice_states => #{
            <<"conn-present">> => voice_state(1, 20, <<"conn-present">>, <<"local">>, <<"s1">>),
            <<"conn-absent">> => voice_state(2, 20, <<"conn-absent">>, <<"local">>, <<"s1">>),
            <<"conn-pending">> => voice_state(3, 20, <<"conn-pending">>, <<"local">>, <<"s1">>)
        },
        pending_voice_connections => #{<<"conn-pending">> => #{}},
        test_voice_reconciliation_v3_fun =>
            fun(10, 20, <<"local">>, <<"s1">>) -> {ok, [<<"conn-present">>]} end
    },
    ?assertEqual([<<"conn-absent">>], find_absent_guild_connections(State)).

find_absent_entries_keeps_room_on_snapshot_error_test() ->
    Entries = [
        #{
            connection_id => <<"conn">>,
            user_id => 1,
            channel_id => 20,
            guild_id => 10,
            region_id => <<"local">>,
            server_id => <<"s1">>,
            pending => false
        }
    ],
    Fun = fun(_GuildId, _ChannelId, _RegionId, _ServerId) -> {error, unavailable} end,
    ?assertEqual([], find_absent_entries(Entries, Fun)).

find_absent_call_entries_uses_connection_id_from_voice_state_test() ->
    State = #{
        channel_id => 30,
        voice_states => #{5 => voice_state(5, 30, <<"conn-a">>, <<"local">>, <<"s1">>)},
        pending_connections => #{},
        test_voice_reconciliation_v3_fun =>
            fun(null, 30, <<"local">>, <<"s1">>) -> {ok, []} end
    },
    ?assertMatch(
        [#{connection_id := <<"conn-a">>, user_id := 5}], find_absent_call_entries(State)
    ).

connection_set_parses_identity_fallback_test() ->
    Snapshot = [
        #{<<"identity">> => <<"user_42_conn_with_underscores">>},
        #{<<"connection_id">> => <<"conn-direct">>}
    ],
    Set = connection_set(Snapshot),
    ?assert(maps:is_key(<<"conn_with_underscores">>, Set)),
    ?assert(maps:is_key(<<"conn-direct">>, Set)).

-endif.
