%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_rpc_misc_push).
-typing([eqwalizer]).

-export([
    get_local_active_voice_rooms/0,
    collect_and_aggregate_active_voice_rooms/1
]).

-define(NODE_RPC_TIMEOUT, 10000).

-spec get_local_active_voice_rooms() -> map().
get_local_active_voice_rooms() ->
    #{
        <<"node_id">> => gateway_rpc_misc_session:node_id(node()),
        <<"rooms">> => get_local_guild_active_voice_rooms() ++ get_local_dm_active_voice_rooms()
    }.

-spec collect_and_aggregate_active_voice_rooms([node()]) -> map().
collect_and_aggregate_active_voice_rooms(Nodes) ->
    NodeRooms = [
        fetch_node_active_voice_rooms(N)
     || N <- gateway_rpc_misc_session:normalize_nodes(Nodes)
    ],
    aggregate_active_voice_rooms(NodeRooms).

-spec get_local_guild_active_voice_rooms() -> [map()].
get_local_guild_active_voice_rooms() ->
    try
        ets:foldl(
            fun
                ({GuildId, Pid}, Acc) when is_integer(GuildId), is_pid(Pid) ->
                    get_guild_active_voice_rooms(GuildId, Pid) ++ Acc;
                (_Row, Acc) ->
                    Acc
            end,
            [],
            guild_voice_registry
        )
    catch
        error:badarg ->
            []
    end.

-spec get_guild_active_voice_rooms(integer(), pid()) -> [map()].
get_guild_active_voice_rooms(GuildId, Pid) ->
    case process_liveness:is_alive(Pid) of
        true -> fetch_guild_voice_states(GuildId, Pid);
        false -> []
    end.

-spec fetch_guild_voice_states(integer(), pid()) -> [map()].
fetch_guild_voice_states(GuildId, Pid) ->
    try gen_server:call(Pid, {get_voice_states_map}, 500) of
        VoiceStates when is_map(VoiceStates) ->
            build_guild_voice_room_entries(GuildId, VoiceStates);
        _ ->
            []
    catch
        _:_ -> []
    end.

-spec build_guild_voice_room_entries(integer(), map()) -> [map()].
build_guild_voice_room_entries(GuildId, VoiceStates) ->
    Counts = maps:fold(
        fun(_ConnId, VoiceState, Acc) ->
            fold_voice_state(VoiceState, Acc)
        end,
        #{},
        VoiceStates
    ),
    [
        #{
            <<"guild_id">> => integer_to_binary(GuildId),
            <<"channel_id">> => ChannelId,
            <<"voice_state_count">> => Count
        }
     || {ChannelId, Count} <- lists:sort(maps:to_list(Counts)),
        Count > 0
    ].

-spec fold_voice_state(term(), map()) -> map().
fold_voice_state(State, Acc) when is_map(State) ->
    ChannelId = normalize_id(maps:get(<<"channel_id">>, State, undefined)),
    increment_channel_count(ChannelId, Acc);
fold_voice_state(_, Acc) ->
    Acc.

-spec increment_channel_count(binary() | undefined, map()) -> map().
increment_channel_count(undefined, Acc) ->
    Acc;
increment_channel_count(ChannelId, Acc) ->
    maps:update_with(ChannelId, fun(C) -> C + 1 end, 1, Acc).

-spec get_local_dm_active_voice_rooms() -> [map()].
get_local_dm_active_voice_rooms() ->
    RawIds = shard_utils:safe_apply(fun call_manager:local_call_ids/0, []),
    CallIds = integer_entries(RawIds),
    lists:flatmap(fun get_dm_active_voice_room/1, CallIds).

-spec integer_entries(term()) -> [integer()].
integer_entries(Values) when is_list(Values) ->
    [Value || Value <- Values, is_integer(Value)];
integer_entries(_) ->
    [].

-spec get_dm_active_voice_room(integer()) -> [map()].
get_dm_active_voice_room(ChannelId) ->
    case lookup_call(ChannelId) of
        {ok, Pid} -> get_dm_room_from_pid(ChannelId, Pid);
        _ -> []
    end.

-spec lookup_call(integer()) -> {ok, pid()} | error.
lookup_call(ChannelId) ->
    try call_manager:lookup(ChannelId) of
        {ok, Pid} when is_pid(Pid) -> {ok, Pid};
        _ -> error
    catch
        _:_ -> error
    end.

-spec get_dm_room_from_pid(integer(), pid()) -> [map()].
get_dm_room_from_pid(ChannelId, Pid) ->
    try gen_server:call(Pid, {get_state}, 500) of
        {ok, CallData} when is_map(CallData) ->
            build_dm_room_entry(ChannelId, CallData);
        _ ->
            []
    catch
        _:_ -> []
    end.

-spec build_dm_room_entry(integer(), map()) -> [map()].
build_dm_room_entry(ChannelId, CallData) ->
    VoiceStates = maps:get(
        voice_states,
        CallData,
        maps:get(<<"voice_states">>, CallData, [])
    ),
    Count =
        case VoiceStates of
            Values when is_list(Values) -> length(Values);
            _ -> 0
        end,
    case Count > 0 of
        true ->
            [
                #{
                    <<"guild_id">> => null,
                    <<"channel_id">> => integer_to_binary(ChannelId),
                    <<"voice_state_count">> => Count
                }
            ];
        false ->
            []
    end.

-spec fetch_node_active_voice_rooms(node()) -> map().
fetch_node_active_voice_rooms(TargetNode) ->
    case
        gateway_rpc_misc_session:safe_node_call(
            TargetNode, get_local_active_voice_rooms, [], ?NODE_RPC_TIMEOUT
        )
    of
        #{<<"rooms">> := Rooms} = Result when is_list(Rooms) ->
            Result#{<<"node_id">> => gateway_rpc_misc_session:node_id(TargetNode)};
        _ ->
            #{
                <<"node_id">> => gateway_rpc_misc_session:node_id(TargetNode),
                <<"rooms">> => []
            }
    end.

-spec aggregate_active_voice_rooms([term()]) -> map().
aggregate_active_voice_rooms(NodeRooms) ->
    {RoomCounts, NodeCount} = lists:foldl(
        fun merge_node_rooms/2,
        {#{}, 0},
        NodeRooms
    ),
    Rooms = [
        build_room_entry(RoomKey, Count)
     || {RoomKey, Count} <- lists:sort(maps:to_list(RoomCounts)),
        Count > 0
    ],
    #{<<"node_count">> => NodeCount, <<"rooms">> => Rooms}.

-spec merge_node_rooms(term(), {map(), non_neg_integer()}) -> {map(), non_neg_integer()}.
merge_node_rooms(NodeRooms, {RoomAcc, NodeCount}) when is_map(NodeRooms) ->
    Rooms = maps:get(<<"rooms">>, NodeRooms, []),
    MergedRooms =
        case Rooms of
            RoomList when is_list(RoomList) ->
                lists:foldl(fun merge_single_room/2, RoomAcc, RoomList);
            _ ->
                RoomAcc
        end,
    {MergedRooms, NodeCount + 1};
merge_node_rooms(_, Acc) ->
    Acc.

-spec merge_single_room(term(), map()) -> map().
merge_single_room(Room, Acc) when is_map(Room) ->
    ChannelId = normalize_id(maps:get(<<"channel_id">>, Room, undefined)),
    GuildIdResult = normalize_room_guild_id(maps:get(<<"guild_id">>, Room, null)),
    Count = gateway_rpc_misc_session:decode_integer(
        maps:get(<<"voice_state_count">>, Room, 0)
    ),
    apply_room_merge(ChannelId, GuildIdResult, Count, Acc);
merge_single_room(_, Acc) ->
    Acc.

-spec apply_room_merge(
    binary() | undefined, {ok, binary() | null} | error, non_neg_integer(), map()
) -> map().
apply_room_merge(undefined, _, _, Acc) ->
    Acc;
apply_room_merge(_, error, _, Acc) ->
    Acc;
apply_room_merge(_, _, Count, Acc) when Count =< 0 -> Acc;
apply_room_merge(ChannelId, {ok, GuildId}, Count, Acc) ->
    Key =
        case GuildId of
            null -> {dm, ChannelId};
            _ -> {guild, GuildId, ChannelId}
        end,
    maps:update_with(Key, fun(Current) -> Current + Count end, Count, Acc).

-spec build_room_entry(tuple(), non_neg_integer()) -> map().
build_room_entry({dm, ChannelId}, Count) ->
    #{
        <<"guild_id">> => null,
        <<"channel_id">> => ChannelId,
        <<"voice_state_count">> => Count
    };
build_room_entry({guild, GuildId, ChannelId}, Count) ->
    #{
        <<"guild_id">> => GuildId,
        <<"channel_id">> => ChannelId,
        <<"voice_state_count">> => Count
    }.

-spec normalize_id(term()) -> binary() | undefined.
normalize_id(Value) when is_binary(Value), byte_size(Value) > 0 ->
    case validation:validate_snowflake(Value) of
        {ok, Id} when Id > 0 -> integer_to_binary(Id);
        _ -> undefined
    end;
normalize_id(Value) when is_integer(Value), Value > 0 ->
    integer_to_binary(Value);
normalize_id(Value) when is_list(Value), Value =/= [] ->
    normalize_integer_id(type_conv:to_integer(Value));
normalize_id(_) ->
    undefined.

-spec normalize_integer_id(integer() | undefined) -> binary() | undefined.
normalize_integer_id(Id) when is_integer(Id), Id > 0 ->
    integer_to_binary(Id);
normalize_integer_id(_) ->
    undefined.

-spec normalize_room_guild_id(term()) -> {ok, binary() | null} | error.
normalize_room_guild_id(null) ->
    {ok, null};
normalize_room_guild_id(undefined) ->
    {ok, null};
normalize_room_guild_id(Value) ->
    case normalize_id(Value) of
        undefined -> error;
        Normalized -> {ok, Normalized}
    end.

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

malformed_test_node_rooms() ->
    [
        #{
            <<"rooms">> => [
                #{
                    <<"guild_id">> => <<"10">>,
                    <<"channel_id">> => <<"20">>,
                    <<"voice_state_count">> => <<"2">>
                },
                #{
                    <<"guild_id">> => 10,
                    <<"channel_id">> => 20,
                    <<"voice_state_count">> => 3
                },
                #{
                    <<"guild_id">> => null,
                    <<"channel_id">> => "30",
                    <<"voice_state_count">> => "4"
                },
                #{
                    <<"guild_id">> => <<"bad">>,
                    <<"channel_id">> => <<"40">>,
                    <<"voice_state_count">> => 5
                },
                #{
                    <<"guild_id">> => <<"10">>,
                    <<"channel_id">> => <<"bad">>,
                    <<"voice_state_count">> => 5
                },
                #{
                    <<"guild_id">> => <<"10">>,
                    <<"channel_id">> => <<"50">>,
                    <<"voice_state_count">> => 0
                },
                not_a_room
            ]
        },
        #{<<"rooms">> => not_a_list},
        not_a_node
    ].

aggregate_active_voice_rooms_ignores_malformed_nodes_and_rooms_test() ->
    Aggregate = aggregate_active_voice_rooms(malformed_test_node_rooms()),
    ?assertEqual(2, maps:get(<<"node_count">>, Aggregate)),
    ?assertEqual(
        [
            #{
                <<"guild_id">> => null,
                <<"channel_id">> => <<"30">>,
                <<"voice_state_count">> => 4
            },
            #{
                <<"guild_id">> => <<"10">>,
                <<"channel_id">> => <<"20">>,
                <<"voice_state_count">> => 5
            }
        ],
        maps:get(<<"rooms">>, Aggregate)
    ).

build_guild_voice_room_entries_ignores_malformed_voice_states_test() ->
    VoiceStates = #{
        <<"good">> => #{<<"channel_id">> => <<"20">>},
        <<"same">> => #{<<"channel_id">> => 20},
        <<"bad_channel">> => #{<<"channel_id">> => <<"bad">>},
        <<"not_a_map">> => not_a_map
    },
    ?assertEqual(
        [
            #{
                <<"guild_id">> => <<"10">>,
                <<"channel_id">> => <<"20">>,
                <<"voice_state_count">> => 2
            }
        ],
        build_guild_voice_room_entries(10, VoiceStates)
    ).

-endif.
