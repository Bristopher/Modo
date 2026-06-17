%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(presence_utils).
-typing([eqwalizer]).

-export([
    collect_guild_member_presences/1,
    collect_guild_member_ids/1,
    filter_self_presence/2,
    is_visible_presence/1,
    batch_presences/1,
    send_presence_bulk/4
]).

-export_type([user_id/0]).

-define(PRESENCE_BATCH_SIZE, 500).

-type user_id() :: integer().

-spec collect_guild_member_presences(map()) -> [map()].
collect_guild_member_presences(GuildState) ->
    MemberIds = collect_guild_member_ids(GuildState),
    Presences = safe_bulk_get(MemberIds),
    [P || P <- Presences, is_visible_presence(P)].

-spec collect_guild_member_ids(map()) -> [user_id()].
collect_guild_member_ids(GuildState) ->
    Members = get_members_from_guild_state(GuildState),
    MemberIds = [member_user_id(M) || M <- Members],
    [Id || Id <- MemberIds, Id =/= undefined].

-spec filter_self_presence(user_id(), [map()]) -> [map()].
filter_self_presence(UserId, Presences) ->
    [P || P <- Presences, presence_user_id(P) =/= UserId].

-spec is_visible_presence(map()) -> boolean().
is_visible_presence(Presence) ->
    Status = maps:get(<<"status">>, Presence, <<"offline">>),
    Status =/= <<"offline">> andalso Status =/= <<"invisible">>.

-spec batch_presences([map()]) -> [[map()]].
batch_presences([]) -> [];
batch_presences(Presences) -> batch_presences_acc(Presences, []).

-spec send_presence_bulk(pid(), integer(), user_id(), [map()]) -> ok.
send_presence_bulk(_Pid, _GuildId, _UserId, []) ->
    ok;
send_presence_bulk(Pid, GuildId, UserId, Presences) ->
    Filtered = filter_self_presence(UserId, Presences),
    dispatch_batches(Pid, GuildId, Filtered).

-spec batch_presences_acc([map()], [[map()]]) -> [[map()]].
batch_presences_acc([], Acc) ->
    lists:reverse(Acc);
batch_presences_acc(Presences, Acc) ->
    {Batch, Rest} = take_presence_batch(Presences, ?PRESENCE_BATCH_SIZE),
    batch_presences_acc(Rest, [Batch | Acc]).

-spec dispatch_batches(pid(), integer(), [map()]) -> ok.
dispatch_batches(_Pid, _GuildId, []) ->
    ok;
dispatch_batches(Pid, GuildId, FilteredPresences) ->
    Batches = batch_presences(FilteredPresences),
    lists:foreach(
        fun(Batch) -> dispatch_single_batch(Pid, GuildId, Batch) end,
        Batches
    ).

-spec dispatch_single_batch(pid(), integer(), [map()]) -> ok.
dispatch_single_batch(Pid, GuildId, Batch) ->
    BulkPayload = #{
        <<"guild_id">> => integer_to_binary(GuildId),
        <<"presences">> => Batch
    },
    gateway_dispatch_relay:dispatch(Pid, presence_update_bulk, BulkPayload, GuildId).

-spec get_members_from_guild_state(map()) -> [map()].
get_members_from_guild_state(GuildState) ->
    case maps:get(data, GuildState, undefined) of
        undefined -> guild_data_index:member_values(GuildState);
        Data -> guild_data_index:member_values(Data)
    end.

-spec member_user_id(map()) -> user_id() | undefined.
member_user_id(Member) ->
    User = maps:get(<<"user">>, Member, #{}),
    user_id(User).

-spec presence_user_id(map() | term()) -> user_id() | undefined.
presence_user_id(P) when is_map(P) ->
    User = maps:get(<<"user">>, P, #{}),
    user_id(User);
presence_user_id(_) ->
    undefined.

-spec user_id(term()) -> user_id() | undefined.
user_id(User) when is_map(User) ->
    snowflake_id:parse_maybe(maps:get(<<"id">>, User, undefined));
user_id(_) ->
    undefined.

-spec safe_bulk_get([user_id()]) -> [map()].
safe_bulk_get(UserIds) ->
    try presence_cache:bulk_get(UserIds) of
        Presences -> [Presence || Presence <- Presences, is_map(Presence)]
    catch
        _:_ -> []
    end.

-spec take_presence_batch([map()], pos_integer()) -> {[map()], [map()]}.
take_presence_batch(List, N) ->
    safe_split(List, N).

-spec safe_split([T], pos_integer()) -> {[T], [T]}.
safe_split(List, N) ->
    try lists:split(N, List) of
        {Batch, Rest} -> {Batch, Rest}
    catch
        error:badarg -> {List, []}
    end.

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

-spec take_batch([term()], pos_integer()) -> {[term()], [term()]}.
take_batch(List, N) ->
    safe_split(List, N).

batch_presences_empty_test() ->
    ?assertEqual([], batch_presences([])).

batch_presences_small_list_test() ->
    Presences = [#{<<"user">> => #{<<"id">> => I}} || I <- lists:seq(1, 10)],
    Batches = batch_presences(Presences),
    ?assertEqual(1, length(Batches)),
    ?assertEqual(10, length(hd(Batches))).

batch_presences_exact_batch_size_test() ->
    Presences = [#{<<"user">> => #{<<"id">> => I}} || I <- lists:seq(1, 500)],
    Batches = batch_presences(Presences),
    ?assertEqual(1, length(Batches)),
    ?assertEqual(500, length(hd(Batches))).

batch_presences_multiple_batches_test() ->
    Presences = [#{<<"user">> => #{<<"id">> => I}} || I <- lists:seq(1, 1250)],
    Batches = batch_presences(Presences),
    ?assertEqual(3, length(Batches)),
    ?assertEqual(500, length(lists:nth(1, Batches))),
    ?assertEqual(500, length(lists:nth(2, Batches))),
    ?assertEqual(250, length(lists:nth(3, Batches))).

filter_self_presence_test() ->
    Presences = [
        #{<<"user">> => #{<<"id">> => <<"1">>}},
        #{<<"user">> => #{<<"id">> => <<"2">>}},
        #{<<"user">> => #{<<"id">> => <<"3">>}}
    ],
    Filtered = filter_self_presence(2, Presences),
    ?assertEqual(2, length(Filtered)),
    ?assert(
        not lists:any(
            fun(P) -> presence_user_id(P) =:= 2 end,
            Filtered
        )
    ).

is_visible_presence_online_test() ->
    ?assert(is_visible_presence(#{<<"status">> => <<"online">>})),
    ?assert(is_visible_presence(#{<<"status">> => <<"idle">>})),
    ?assert(is_visible_presence(#{<<"status">> => <<"dnd">>})).

is_visible_presence_offline_test() ->
    ?assertNot(is_visible_presence(#{<<"status">> => <<"offline">>})),
    ?assertNot(is_visible_presence(#{<<"status">> => <<"invisible">>})),
    ?assertNot(is_visible_presence(#{})).

collect_guild_member_ids_internal_format_test() ->
    GuildState = #{
        data => #{
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => <<"100">>}},
                #{<<"user">> => #{<<"id">> => <<"200">>}}
            ]
        }
    },
    Ids = collect_guild_member_ids(GuildState),
    ?assertEqual([100, 200], lists:sort(Ids)).

collect_guild_member_ids_rejects_malformed_id_test() ->
    GuildState = #{
        data => #{
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => <<"100">>}},
                #{<<"user">> => #{<<"id">> => <<"001">>}}
            ]
        }
    },
    ?assertEqual([100], collect_guild_member_ids(GuildState)).

collect_guild_member_ids_external_format_test() ->
    GuildState = #{
        <<"members">> => [
            #{<<"user">> => #{<<"id">> => <<"100">>}},
            #{<<"user">> => #{<<"id">> => <<"200">>}}
        ]
    },
    Ids = collect_guild_member_ids(GuildState),
    ?assertEqual([100, 200], lists:sort(Ids)).

take_batch_small_list_test() ->
    {Batch, Rest} = take_batch([1, 2, 3], 10),
    ?assertEqual([1, 2, 3], Batch),
    ?assertEqual([], Rest).

take_batch_exact_test() ->
    {Batch, Rest} = take_batch([1, 2, 3], 3),
    ?assertEqual([1, 2, 3], Batch),
    ?assertEqual([], Rest).

take_batch_split_test() ->
    {Batch, Rest} = take_batch([1, 2, 3, 4, 5], 2),
    ?assertEqual([1, 2], Batch),
    ?assertEqual([3, 4, 5], Rest).

presence_user_id_test() ->
    ?assertEqual(123, presence_user_id(#{<<"user">> => #{<<"id">> => <<"123">>}})),
    ?assertEqual(undefined, presence_user_id(#{<<"user">> => #{<<"id">> => <<"001">>}})),
    ?assertEqual(undefined, presence_user_id(#{<<"user">> => #{}})),
    ?assertEqual(undefined, presence_user_id(#{})),
    ?assertEqual(undefined, presence_user_id(invalid)).
-endif.
