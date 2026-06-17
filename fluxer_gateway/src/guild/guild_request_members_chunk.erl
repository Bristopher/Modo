%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_request_members_chunk).
-typing([eqwalizer]).

-export([
    send_member_chunks/5,
    build_chunk_data/5
]).

-export_type([member/0, presence/0]).

-define(CHUNK_SIZE, 1000).

-type member() :: map().
-type presence() :: map().

-spec send_member_chunks(
    integer(), pid() | undefined, [member()], [presence()], term()
) -> ok.
send_member_chunks(_GuildId, SessionPid, _Members, _Presences, _Nonce) when
    not is_pid(SessionPid)
->
    ok;
send_member_chunks(GuildId, SessionPid, Members, Presences, Nonce) ->
    Chunks = member_presence_chunks(Members, Presences, ?CHUNK_SIZE),
    TotalChunks = length(Chunks),
    dispatch_chunks(GuildId, SessionPid, Chunks, TotalChunks, Nonce, 0),
    ok.

-spec dispatch_chunks(
    integer(),
    pid(),
    [{[member()], [presence()]}],
    non_neg_integer(),
    term(),
    non_neg_integer()
) -> ok.
dispatch_chunks(_GuildId, _SessionPid, [], _TotalChunks, _Nonce, _Idx) ->
    ok;
dispatch_chunks(GuildId, SessionPid, [{MC, PC} | Rest], Total, Nonce, Idx) ->
    ChunkData0 = build_chunk_data(MC, PC, Idx, Total, Nonce),
    ChunkData = ChunkData0#{<<"guild_id">> => integer_to_binary(GuildId)},
    gateway_dispatch_relay:dispatch(
        SessionPid, guild_members_chunk, ChunkData, GuildId
    ),
    dispatch_chunks(GuildId, SessionPid, Rest, Total, Nonce, Idx + 1).

-spec member_presence_chunks(
    [member()], [presence()], pos_integer()
) -> [{[member()], [presence()]}].
member_presence_chunks([], _Presences, _Size) ->
    [{[], []}];
member_presence_chunks(Members, Presences, Size) ->
    PresMap = presence_map(Presences),
    do_member_presence_chunks(Members, PresMap, Size, []).

-spec do_member_presence_chunks(
    [member()], map(), pos_integer(), [{[member()], [presence()]}]
) -> [{[member()], [presence()]}].
do_member_presence_chunks([], _PresMap, _Size, Acc) ->
    lists:reverse(Acc);
do_member_presence_chunks(Members, PresMap, Size, Acc) ->
    {MC, PC, Rest} = take_chunk(Members, PresMap, Size, [], [], 0),
    do_member_presence_chunks(Rest, PresMap, Size, [{MC, PC} | Acc]).

-spec take_chunk(
    [member()],
    map(),
    pos_integer(),
    [member()],
    [presence()],
    non_neg_integer()
) -> {[member()], [presence()], [member()]}.
take_chunk(Rest, _PresMap, Size, MAcc, PAcc, Size) ->
    {lists:reverse(MAcc), lists:reverse(PAcc), Rest};
take_chunk([], _PresMap, _Size, MAcc, PAcc, _Count) ->
    {lists:reverse(MAcc), lists:reverse(PAcc), []};
take_chunk([Member | Rest], PresMap, Size, MAcc, PAcc, Count) ->
    PAcc1 = maybe_add_presence(Member, PresMap, PAcc),
    take_chunk(Rest, PresMap, Size, [Member | MAcc], PAcc1, Count + 1).

-spec maybe_add_presence(member(), map(), [presence()]) -> [presence()].
maybe_add_presence(Member, PresMap, PAcc) ->
    case guild_request_members_search:extract_user_id(Member) of
        UserId when is_integer(UserId) ->
            add_presence_for_user(UserId, PresMap, PAcc);
        _ ->
            PAcc
    end.

-spec add_presence_for_user(integer(), map(), [presence()]) -> [presence()].
add_presence_for_user(UserId, PresMap, PAcc) ->
    case maps:get(UserId, PresMap, undefined) of
        undefined -> PAcc;
        Presence -> [Presence | PAcc]
    end.

-spec presence_map([presence()]) -> #{integer() => presence()}.
presence_map(Presences) ->
    lists:foldl(
        fun add_presence_to_map/2,
        #{},
        Presences
    ).

-spec add_presence_to_map(presence(), map()) -> map().
add_presence_to_map(Presence, Acc) ->
    case guild_request_members_presence:presence_user_id(Presence) of
        UserId when is_integer(UserId) ->
            Acc#{UserId => Presence};
        _ ->
            Acc
    end.

-spec build_chunk_data(
    [member()], [presence()], non_neg_integer(), non_neg_integer(), term()
) -> map().
build_chunk_data(Members, Presences, ChunkIndex, TotalChunks, Nonce) ->
    Base = #{
        <<"members">> => Members,
        <<"chunk_index">> => ChunkIndex,
        <<"chunk_count">> => TotalChunks
    },
    add_optional_fields(Base, Presences, Nonce).

-spec add_optional_fields(map(), [presence()], term()) -> map().
add_optional_fields(Base, [], null) ->
    Base;
add_optional_fields(Base, [], Nonce) ->
    Base#{<<"nonce">> => Nonce};
add_optional_fields(Base, Presences, null) ->
    Base#{<<"presences">> => Presences};
add_optional_fields(Base, Presences, Nonce) ->
    Base#{<<"presences">> => Presences, <<"nonce">> => Nonce}.
