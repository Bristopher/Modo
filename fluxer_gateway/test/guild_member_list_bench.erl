%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_member_list_bench).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

-define(ONLINE_RATIO, 0.05).
-define(HOISTED_ROLES, [1000, 2000, 3000, 4000, 5000]).

bench_100k_test_() ->
    {timeout, 120, fun() -> run_bench(100_000) end}.

bench_1m_test_() ->
    case os:getenv("BENCH_SCALE") of
        false -> {timeout, 1, fun() -> ok end};
        _ -> {timeout, 600, fun() -> run_bench(1_000_000) end}
    end.

bench_10m_test_() ->
    case os:getenv("BENCH_SCALE") of
        "10m" ++ _ -> {timeout, 1800, fun() -> run_bench(10_000_000) end};
        "100m" ++ _ -> {timeout, 1800, fun() -> run_bench(10_000_000) end};
        _ -> {timeout, 1, fun() -> ok end}
    end.

bench_100m_test_() ->
    case os:getenv("BENCH_SCALE") of
        "100m" ++ _ -> {timeout, 7200, fun() -> run_bench(100_000_000) end};
        _ -> {timeout, 1, fun() -> ok end}
    end.

bench_presence_100k_test_() ->
    {timeout, 60, fun() -> run_presence_bench(100_000) end}.

run_bench(N) ->
    log("~n=== Benchmark: ~B members ===~n", [N]),
    {GenUs, Members} = timer:tc(fun() -> generate_members(N) end),
    log("  generate:      ~.1f ms~n", [us_to_ms(GenUs)]),
    Ref = guild_member_list_engine:new(),
    run_bulk_load_benchmark(Ref, Members, N),
    run_query_benchmarks(Ref, N),
    run_mutation_benchmarks(Ref, N),
    guild_member_list_engine:destroy(Ref),
    log("  PASSED~n", []).

run_bulk_load_benchmark(Ref, Members, N) ->
    {BulkUs, ok} = timer:tc(fun() ->
        guild_member_list_engine:bulk_load(Ref, Members, ?HOISTED_ROLES)
    end),
    log(
        "  bulk_load:     ~.1f ms  (~B inserts/s)~n",
        [us_to_ms(BulkUs), ops_per_second(N, BulkUs)]
    ),

    Info = guild_member_list_engine:info(Ref),
    TotalMB = maps:get(total_bytes, Info) / (1024.0 * 1024.0),
    PerMember = maps:get(total_bytes, Info) / float(N),
    log("  memory:        ~.1f MB  (~B bytes/member)~n", [TotalMB, trunc(PerMember)]).

run_query_benchmarks(Ref, N) ->
    {CountUs, {Total, Online}} = timer:tc(fun() ->
        guild_member_list_engine:get_counts(Ref)
    end),
    ?assertEqual(N, Total),
    log(
        "  get_counts:    ~.1f us  (total=~B, online=~B)~n",
        [float(CountUs), Total, Online]
    ),

    {GroupUs, Groups} = timer:tc(fun() ->
        guild_member_list_engine:get_groups(Ref)
    end),
    log(
        "  get_groups:    ~.1f us  (~B sections)~n",
        [float(GroupUs), length(Groups)]
    ),
    bench_get_items(Ref, 0, 99, "  get_items(0,99):       ~.1f us  (~B items)~n"),
    bench_get_items(Ref, 5000, 5099, "  get_items(5k,5099):    ~.1f us  (~B items)~n"),
    bench_get_items(Ref, 50000, 50099, "  get_items(50k,50099):  ~.1f us  (~B items)~n").

bench_get_items(Ref, First, Last, LogFormat) ->
    {ItemsUs, Items} = timer:tc(fun() ->
        guild_member_list_engine:get_items(Ref, First, Last)
    end),
    log(LogFormat, [float(ItemsUs), length(Items)]).

run_mutation_benchmarks(Ref, N) ->
    {AddUs, _} = bench_avg(100, fun(I) ->
        guild_member_list_engine:add_member(Ref, N + I, <<"bench_user">>, [1000], true)
    end),
    log("  add_member:    ~.1f us avg~n", [AddUs]),

    {RemUs, _} = bench_avg(100, fun(I) ->
        guild_member_list_engine:remove_member(Ref, N + I)
    end),
    log("  remove_member: ~.1f us avg~n", [RemUs]),

    lists:foreach(
        fun(I) ->
            guild_member_list_engine:add_member(Ref, N + I, <<"bench_user">>, [1000], false)
        end,
        lists:seq(1, 100)
    ),

    {OnlineUs, _} = bench_avg(100, fun(I) ->
        guild_member_list_engine:set_online(Ref, N + I, true)
    end),
    log("  set_online:    ~.1f us avg~n", [OnlineUs]),

    {HoistUs, _} = timer:tc(fun() ->
        guild_member_list_engine:set_hoisted_roles(Ref, [1000, 2000, 3000, 4000, 5000, 6000])
    end),
    log("  set_hoisted_roles: ~.1f ms  (full rebuild)~n", [us_to_ms(HoistUs)]),

    {Total2, _Online2} = guild_member_list_engine:get_counts(Ref),
    ?assertEqual(N + 100, Total2).

run_presence_bench(N) ->
    log("~n=== Presence Benchmark: ~B members ===~n", [N]),

    Ref = guild_member_list_engine:new(),
    Members = generate_members(N),
    ok = guild_member_list_engine:bulk_load(Ref, Members, ?HOISTED_ROLES),

    OnlineIds = [Id || {Id, _, _, true} <- Members],
    ToggleIds = lists:sublist(OnlineIds, min(10000, length(OnlineIds))),
    NToggle = length(ToggleIds),

    {OffUs, _} = bench_presence_toggle(Ref, ToggleIds, false),
    log_presence_toggle("  toggle ~B offline: ~.1f ms  (~B/s)~n", NToggle, OffUs),
    {OnUs, _} = bench_presence_toggle(Ref, ToggleIds, true),
    log_presence_toggle("  toggle ~B online:  ~.1f ms  (~B/s)~n", NToggle, OnUs),
    guild_member_list_engine:destroy(Ref),
    log("  PASSED~n", []).

bench_presence_toggle(Ref, ToggleIds, Online) ->
    timer:tc(fun() ->
        lists:foreach(
            fun(Id) ->
                guild_member_list_engine:set_online(Ref, Id, Online)
            end,
            ToggleIds
        )
    end).

log_presence_toggle(Format, Count, Us) ->
    log(Format, [Count, us_to_ms(Us), ops_per_second(Count, Us)]).

bench_concurrent_100k_test_() ->
    {timeout, 120, fun() -> run_concurrent_bench(100_000) end}.

run_concurrent_bench(N) ->
    log("~n=== Concurrent Read Benchmark: ~B members ===~n", [N]),
    {Ref, _Members} = loaded_engine(N),
    ReadCount = 5000,
    Parent = self(),
    ReaderPids = spawn_readers(Ref, Parent, ReadCount),
    CounterPids = spawn_counters(Ref, Parent),
    ReaderResults = collect_results(ReaderPids, reader_done),
    CounterResults = collect_results(CounterPids, counter_done),
    log_result_group(
        "  4 readers (~B get_items each):~n", [ReadCount], ReadCount, ReaderResults
    ),
    log_result_group("  2 counter readers (10K each):~n", [], 10000, CounterResults),
    {Total, Online} = guild_member_list_engine:get_counts(Ref),
    ?assertEqual(N, Total),
    log("  final: total=~B, online=~B~n", [Total, Online]),
    guild_member_list_engine:destroy(Ref),
    log("  PASSED~n", []).

loaded_engine(N) ->
    Ref = guild_member_list_engine:new(),
    Members = generate_members(N),
    ok = guild_member_list_engine:bulk_load(Ref, Members, ?HOISTED_ROLES),
    {Ref, Members}.

spawn_readers(Ref, Parent, ReadCount) ->
    [spawn_link(fun() -> reader_benchmark(Ref, Parent, ReadCount) end) || _ <- lists:seq(1, 4)].

reader_benchmark(Ref, Parent, ReadCount) ->
    T0 = erlang:monotonic_time(microsecond),
    lists:foreach(fun(J) -> read_item_range(Ref, J) end, lists:seq(1, ReadCount)),
    T1 = erlang:monotonic_time(microsecond),
    Parent ! {reader_done, self(), T1 - T0}.

read_item_range(Ref, J) ->
    Start = (J * 100) rem 10000,
    _ = guild_member_list_engine:get_items(Ref, Start, Start + 99).

spawn_counters(Ref, Parent) ->
    [spawn_link(fun() -> counter_benchmark(Ref, Parent) end) || _ <- lists:seq(1, 2)].

counter_benchmark(Ref, Parent) ->
    T0 = erlang:monotonic_time(microsecond),
    lists:foreach(fun(_) -> read_counts_and_groups(Ref) end, lists:seq(1, 10000)),
    T1 = erlang:monotonic_time(microsecond),
    Parent ! {counter_done, self(), T1 - T0}.

read_counts_and_groups(Ref) ->
    _ = guild_member_list_engine:get_counts(Ref),
    _ = guild_member_list_engine:get_groups(Ref).

log_result_group(HeaderFormat, HeaderArgs, OpCount, Results) ->
    log(HeaderFormat, HeaderArgs),
    lists:foreach(
        fun({_, Us}) ->
            log("    ~.1f ms  (~B ops/s)~n", [us_to_ms(Us), ops_per_second(OpCount, Us)])
        end,
        Results
    ).

collect_results(Pids, Tag) ->
    [
        receive
            {Tag, Pid, Us} -> {Pid, Us}
        after 120000 ->
            error({collect_results_timeout, Tag, Pid})
        end
     || Pid <- Pids
    ].

bench_latency_100k_test_() ->
    {timeout, 120, fun() -> run_latency_bench(100_000) end}.

run_latency_bench(N) ->
    log("~n=== Latency Percentile Benchmark: ~B members ===~n", [N]),
    {Ref, Members} = loaded_engine(N),
    OnlineIds = lists:sublist([Id || {Id, _, _, true} <- Members], 1000),
    AddLatencies = [measure_add_latency(Ref, N, I) || I <- lists:seq(1, 1000)],
    report_percentiles("add_member", AddLatencies),
    lists:foreach(
        fun(I) -> guild_member_list_engine:remove_member(Ref, N + I) end, lists:seq(1, 1000)
    ),
    ToggleLatencies = [measure_toggle_latency(Ref, Id) || Id <- OnlineIds],
    report_percentiles("set_online", ToggleLatencies),
    GetItemsLatencies = [measure_get_items_latency(Ref) || _ <- lists:seq(1, 1000)],
    report_percentiles("get_items(0,99)", GetItemsLatencies),
    CountLatencies = [measure_count_latency(Ref) || _ <- lists:seq(1, 10000)],
    report_percentiles("get_counts", CountLatencies),
    guild_member_list_engine:destroy(Ref),
    log("  PASSED~n", []).

measure_add_latency(Ref, N, I) ->
    T0 = erlang:monotonic_time(microsecond),
    guild_member_list_engine:add_member(Ref, N + I, <<"lat_user">>, [1000], true),
    erlang:monotonic_time(microsecond) - T0.

measure_toggle_latency(Ref, Id) ->
    T0 = erlang:monotonic_time(microsecond),
    guild_member_list_engine:set_online(Ref, Id, false),
    T1 = erlang:monotonic_time(microsecond),
    guild_member_list_engine:set_online(Ref, Id, true),
    T1 - T0.

measure_get_items_latency(Ref) ->
    T0 = erlang:monotonic_time(microsecond),
    _ = guild_member_list_engine:get_items(Ref, 0, 99),
    erlang:monotonic_time(microsecond) - T0.

measure_count_latency(Ref) ->
    T0 = erlang:monotonic_time(microsecond),
    _ = guild_member_list_engine:get_counts(Ref),
    erlang:monotonic_time(microsecond) - T0.

report_percentiles(Label, Latencies) ->
    Sorted = lists:sort(Latencies),
    Len = length(Sorted),
    P50 = lists:nth(max(1, Len div 2), Sorted),
    P95 = lists:nth(max(1, trunc(Len * 0.95)), Sorted),
    P99 = lists:nth(max(1, trunc(Len * 0.99)), Sorted),
    Max = lists:last(Sorted),
    log(
        "  ~s: p50=~Bus p95=~Bus p99=~Bus max=~Bus~n",
        [Label, P50, P95, P99, Max]
    ).

bench_hotpath_100k_test_() ->
    {timeout, 180, fun() -> run_hotpath_bench(100_000) end}.

bench_hotpath_1m_test_() ->
    case os:getenv("BENCH_SCALE") of
        false -> {timeout, 1, fun() -> ok end};
        _ -> {timeout, 600, fun() -> run_hotpath_bench(1_000_000) end}
    end.

run_hotpath_bench(N) ->
    log("~n=== Hot-path Benchmark (CPU + allocations): ~B members ===~n", [N]),
    {Ref, Members} = loaded_engine(N),
    {Total, Online} = guild_member_list_engine:get_counts(Ref),
    Offline = Total - Online,
    log("  total=~B online=~B offline=~B (offline solidly hidden)~n", [Total, Online, Offline]),
    OnlineIds = online_ids(Members),
    bench_index_hotpath(Ref, OnlineIds),
    bench_count_hotpath(Ref),
    bench_move_hotpath(Ref, OnlineIds),
    bench_read_hotpath(Ref, Total, Online),
    bench_sync_encode_once_vs_per_subscriber(Ref),
    ok = assert_flat(Ref, Online),
    guild_member_list_engine:destroy(Ref),
    log("~n  PASSED (no O(N) hotspot on the move/read hot path)~n", []),
    ok.

online_ids(Members) ->
    [Id || {Id, _, _, true} <- Members].

bench_index_hotpath(Ref, OnlineIds) ->
    Shallow = hd(OnlineIds),
    Deep = lists:last(OnlineIds),
    log("~n  -- rank: index_of (O(log N), flat across position) --~n", []),
    bench_op("index_of shallow", fun() -> guild_member_list_engine:index_of(Ref, Shallow) end),
    bench_op("index_of deep   ", fun() -> guild_member_list_engine:index_of(Ref, Deep) end).

bench_count_hotpath(Ref) ->
    log("~n  -- gate: get_counts (must be O(1), near-zero alloc) --~n", []),
    bench_op("get_counts      ", fun() -> guild_member_list_engine:get_counts(Ref) end),
    bench_op("get_groups      ", fun() -> guild_member_list_engine:get_groups(Ref) end).

bench_move_hotpath(Ref, OnlineIds) ->
    log("~n  -- engine move: set_online (O(log N)) --~n", []),
    MoveId = hd(OnlineIds),
    bench_op("set_online off  ", fun() ->
        guild_member_list_engine:set_online(Ref, MoveId, false)
    end),
    bench_op("set_online on   ", fun() ->
        guild_member_list_engine:set_online(Ref, MoveId, true)
    end).

bench_read_hotpath(Ref, Total, Online) ->
    log("~n  -- read: windowed get_items (flat across offset, no hotspot) --~n", []),
    bench_op("get_items(0,99)   ", fun() -> guild_member_list_engine:get_items(Ref, 0, 99) end),
    Mid = max(0, (Online div 2)),
    bench_op("get_items(mid,+99)", fun() ->
        guild_member_list_engine:get_items(Ref, Mid, Mid + 99)
    end),
    Near = max(0, Online - 100),
    bench_op("get_items(end,+99)", fun() ->
        guild_member_list_engine:get_items(Ref, Near, Near + 99)
    end),
    log("~n  -- full snapshot vs window read --~n", []),
    bench_op_slow("full snapshot   ", fun() ->
        guild_member_list_engine:get_items(Ref, 0, Total * 2)
    end),
    bench_op("window(0,99)    ", fun() -> guild_member_list_engine:get_items(Ref, 0, 99) end),
    log("~n  -- dispatch encode: shared SYNC payload vs per-subscriber encode --~n", []),
    ok.

bench_sync_encode_once_vs_per_subscriber(Ref) ->
    Groups = guild_member_list_engine:get_groups(Ref),
    Items = [
        #{<<"member">> => #{<<"user">> => #{<<"id">> => <<"123">>, <<"username">> => <<"u">>}}}
    ],
    Resp = #{
        <<"guild_id">> => <<"1">>,
        <<"id">> => <<"123">>,
        <<"groups">> => [#{<<"id">> => Id, <<"count">> => C} || {Id, C} <- Groups],
        <<"ops">> => [#{<<"op">> => <<"SYNC">>, <<"range">> => [7, 7], <<"items">> => Items}]
    },
    Subs = 1000,
    EncodeFun = fun() -> iolist_to_binary(json:encode(Resp)) end,
    OnceUs = median_us(EncodeFun),
    OnceWords = measure_words(EncodeFun),
    log("  encode once:        ~B us  ~B words~n", [OnceUs, OnceWords]),
    log(
        "  vs per-subscriber (~B subs): re-encoding ~B x costs ~B us + ~B words;"
        " shared SYNC wire sends the same binary~n",
        [Subs, Subs, OnceUs * Subs, OnceWords * Subs]
    ).

assert_flat(Ref, Online) ->
    Mid = max(0, Online div 2),
    Near = max(0, Online - 100),
    T0 = median_us(fun() -> guild_member_list_engine:get_items(Ref, 0, 99) end),
    T1 = median_us(fun() -> guild_member_list_engine:get_items(Ref, Mid, Mid + 99) end),
    T2 = median_us(fun() -> guild_member_list_engine:get_items(Ref, Near, Near + 99) end),
    Worst = lists:max([T0, T1, T2]),
    Best = max(1, lists:min([T0, T1, T2])),
    Ratio = Worst / Best,
    log("  flatness: shallow=~Bus mid=~Bus deep=~Bus  (worst/best=~.1fx)~n", [T0, T1, T2, Ratio]),
    ?assert(Ratio < 5.0),
    ok.

median_us(Fun) ->
    Samples = lists:sort([element(1, timer:tc(Fun)) || _ <- lists:seq(1, 21)]),
    lists:nth(11, Samples).

bench_op(Label, Fun) ->
    Reds = measure_reds(Fun),
    Words = measure_words(Fun),
    Us = median_us(Fun),
    log("  ~s : ~8B us  ~12B reds  ~12B words~n", [Label, Us, Reds, Words]).

bench_op_slow(Label, Fun) ->
    Reds = measure_reds(Fun),
    {Us, _} = timer:tc(Fun),
    log("  ~s : ~8B us  ~12B reds  (full-structure walk; allocs ~~ O(N))~n", [Label, Us, Reds]).

measure_reds(Fun) ->
    erlang:garbage_collect(),
    {reductions, R0} = erlang:process_info(self(), reductions),
    _ = Fun(),
    {reductions, R1} = erlang:process_info(self(), reductions),
    max(0, R1 - R0 - 1).

measure_words(Fun) ->
    Iterations = 2000,
    Parent = self(),
    _ = spawn(fun() ->
        erlang:garbage_collect(),
        {_, R0, _} = erlang:statistics(garbage_collection),
        run_n(Fun, Iterations),
        erlang:garbage_collect(),
        {_, R1, _} = erlang:statistics(garbage_collection),
        Parent ! {words, (R1 - R0) div Iterations}
    end),
    receive
        {words, W} -> W
    after 120000 -> error(measure_words_timeout)
    end.

run_n(_Fun, 0) ->
    ok;
run_n(Fun, K) ->
    _ = Fun(),
    run_n(Fun, K - 1).

bench_dispatch_scaling_test_() ->
    {timeout, 300, fun run_dispatch_scaling_bench/0}.

run_dispatch_scaling_bench() ->
    log("~n=== Dispatch scaling: shared SYNC wire vs per-subscriber SYNC encode ===~n", []),
    Sink = spawn(fun sink_loop/0),
    SyncPayload = sample_sync_payload(100),
    SyncWire = iolist_to_binary(json:encode(SyncPayload)),
    log(
        "  payload bytes: full SYNC window(100)=~B~n",
        [byte_size(SyncWire)]
    ),
    lists:foreach(
        fun(S) -> bench_dispatch_scale(Sink, SyncPayload, S) end,
        [1000, 10000, 100000]
    ),
    Sink ! stop,
    log(
        "  PASSED (shared SYNC dispatch avoids per-subscriber encode work)~n",
        []
    ),
    ok.

bench_dispatch_scale(Sink, SyncPayload, SubscriberCount) ->
    Pids = lists:duplicate(SubscriberCount, Sink),
    {SharedUs, _} = timer:tc(fun() ->
        Enc = iolist_to_binary(json:encode(SyncPayload)),
        gateway_dispatch_relay:dispatch_many(Pids, guild_member_list_update, Enc)
    end),
    {PerSubscriberUs, _} = timer:tc(fun() ->
        dispatch_per_subscriber_sync(Pids, SyncPayload)
    end),
    log_dispatch_scale(SubscriberCount, SharedUs, PerSubscriberUs).

dispatch_per_subscriber_sync(Pids, SyncPayload) ->
    lists:foreach(
        fun(Pid) ->
            Enc = iolist_to_binary(json:encode(SyncPayload)),
            gateway_dispatch_relay:dispatch(Pid, guild_member_list_update, Enc)
        end,
        Pids
    ).

log_dispatch_scale(SubscriberCount, SharedUs, PerSubscriberUs) ->
    log(
        "  S=~7B  shared=~9.1f ms (~.3f us/sub)"
        "   per-sub=~9.1f ms (~.3f us/sub)   per-sub/shared=~.1fx~n",
        [
            SubscriberCount,
            us_to_ms(SharedUs),
            SharedUs / SubscriberCount,
            us_to_ms(PerSubscriberUs),
            PerSubscriberUs / SubscriberCount,
            PerSubscriberUs / max(1, SharedUs)
        ]
    ).

sink_loop() ->
    receive
        stop -> ok;
        _ -> sink_loop()
    after 300000 ->
        ok
    end.

sample_member(U) ->
    UB = integer_to_binary(U),
    #{
        <<"user">> => #{
            <<"id">> => UB,
            <<"username">> => <<"user_", UB/binary>>,
            <<"global_name">> => <<"User ", UB/binary>>,
            <<"avatar">> => <<"abcd1234efgh5678">>,
            <<"discriminator">> => <<"0001">>,
            <<"flags">> => 0
        },
        <<"nick">> => null,
        <<"roles">> => [],
        <<"joined_at">> => <<"2026-01-01T00:00:00Z">>,
        <<"presence">> => #{<<"status">> => <<"online">>, <<"mobile">> => false}
    }.

sample_groups() ->
    [#{<<"id">> => <<"online">>, <<"count">> => 5000}].

sample_sync_payload(W) ->
    Items = [#{<<"member">> => sample_member(U)} || U <- lists:seq(1, W)],
    #{
        <<"guild_id">> => <<"1">>,
        <<"id">> => <<"123">>,
        <<"channel_id">> => <<"123">>,
        <<"member_count">> => 5000,
        <<"online_count">> => 5000,
        <<"groups">> => sample_groups(),
        <<"ops">> => [
            #{<<"op">> => <<"SYNC">>, <<"range">> => [0, W - 1], <<"items">> => Items}
        ]
    }.

bench_batched_member_list_fanout_test_() ->
    {timeout, 180, fun run_batched_member_list_fanout_bench/0}.

run_batched_member_list_fanout_bench() ->
    log("~n=== Batched member-list fanout benchmark ===~n", []),
    with_batched_fanout_relay_mock(fun() ->
        bench_batched_fanout_case(identical_ranges, 16, 128, 5000),
        bench_batched_fanout_case(unique_ranges, 16, 128, 5000)
    end),
    log("  PASSED (backend batch coalesces list fanout and preserves range grouping)~n", []).

bench_batched_fanout_case(RangeMode, ChannelCount, SessionsPerChannel, MemberCount) ->
    {State, SubsTab} = batched_fanout_state(
        RangeMode, ChannelCount, SessionsPerChannel, MemberCount
    ),
    ExpectedDispatches = expected_batched_fanout_dispatches(
        RangeMode, ChannelCount, SessionsPerChannel
    ),
    try
        {Us, FlushedState} = timer:tc(fun() ->
            {ok, QueuedState} = guild_member_list:broadcast_all_member_list_updates(State),
            guild_member_list:flush_pending_member_list_syncs(QueuedState)
        end),
        {Dispatches, Recipients, Bytes} = collect_batched_fanout_dispatches(
            ExpectedDispatches
        ),
        assert_no_batched_fanout_dispatch(),
        ?assertEqual(ExpectedDispatches, Dispatches),
        ?assertEqual(ChannelCount * SessionsPerChannel, Recipients),
        ?assert(Bytes > 0),
        log(
            "  ~p: channels=~B sessions/channel=~B members=~B dispatches=~B"
            " recipients=~B bytes=~B elapsed=~.1f ms~n",
            [
                RangeMode,
                ChannelCount,
                SessionsPerChannel,
                MemberCount,
                Dispatches,
                Recipients,
                Bytes,
                us_to_ms(Us)
            ]
        ),
        FlushedState
    after
        guild_member_list_subs:destroy(SubsTab),
        guild_member_list_channel_engine:destroy_all(State)
    end.

expected_batched_fanout_dispatches(identical_ranges, ChannelCount, _SessionsPerChannel) ->
    ChannelCount;
expected_batched_fanout_dispatches(unique_ranges, ChannelCount, SessionsPerChannel) ->
    ChannelCount * SessionsPerChannel.

batched_fanout_state(RangeMode, ChannelCount, SessionsPerChannel, MemberCount) ->
    GuildId = 424242,
    ChannelIds = lists:seq(700000, 700000 + ChannelCount - 1),
    MemberIds = lists:seq(1, MemberCount),
    Members = [bench_member(UserId) || UserId <- MemberIds],
    SubsTab = guild_member_list_subs:new(),
    Sessions = batched_fanout_sessions(
        RangeMode, ChannelIds, SessionsPerChannel, SubsTab, MemberIds
    ),
    State = #{
        id => GuildId,
        member_count => MemberCount,
        sessions => Sessions,
        connected_user_ids => sets:from_list(MemberIds),
        member_presence => maps:from_list([
            {UserId, #{<<"status">> => <<"online">>, <<"mobile">> => false}}
         || UserId <- MemberIds
        ]),
        member_list_subscriptions => SubsTab,
        data => #{
            <<"guild">> => #{
                <<"id">> => integer_to_binary(GuildId),
                <<"owner_id">> => <<"1">>,
                <<"features">> => [],
                <<"member_count">> => MemberCount
            },
            <<"roles">> => [bench_everyone_role(GuildId)],
            <<"channels">> => [bench_channel(ChannelId) || ChannelId <- ChannelIds],
            <<"members">> => Members
        },
        channel_member_list_engines => #{}
    },
    {State, SubsTab}.

batched_fanout_sessions(RangeMode, ChannelIds, SessionsPerChannel, SubsTab, MemberIds) ->
    ViewableChannels = maps:from_list([{ChannelId, true} || ChannelId <- ChannelIds]),
    maps:from_list(
        lists:append([
            batched_fanout_channel_sessions(
                RangeMode, ChannelId, SessionsPerChannel, SubsTab, MemberIds, ViewableChannels
            )
         || ChannelId <- ChannelIds
        ])
    ).

batched_fanout_channel_sessions(
    RangeMode, ChannelId, SessionsPerChannel, SubsTab, MemberIds, ViewableChannels
) ->
    [
        batched_fanout_session_entry(
            RangeMode, ChannelId, SessionIdx, SubsTab, MemberIds, ViewableChannels
        )
     || SessionIdx <- lists:seq(1, SessionsPerChannel)
    ].

batched_fanout_session_entry(
    RangeMode, ChannelId, SessionIdx, SubsTab, MemberIds, ViewableChannels
) ->
    SessionId = batched_fanout_session_id(ChannelId, SessionIdx),
    Ranges = batched_fanout_ranges(RangeMode, SessionIdx),
    {_OldRanges, _ShouldSync} = guild_member_list_subs:subscribe(
        SessionId, integer_to_binary(ChannelId), Ranges, SubsTab
    ),
    {SessionId, #{
        pid => self(),
        user_id => lists:nth(((SessionIdx - 1) rem length(MemberIds)) + 1, MemberIds),
        viewable_channels => ViewableChannels
    }}.

batched_fanout_session_id(ChannelId, SessionIdx) ->
    <<"bench_", (integer_to_binary(ChannelId))/binary, "_",
        (integer_to_binary(SessionIdx))/binary>>.

batched_fanout_ranges(identical_ranges, _SessionIdx) ->
    [{0, 99}];
batched_fanout_ranges(unique_ranges, SessionIdx) ->
    Start = SessionIdx * 2,
    [{Start, Start}].

bench_everyone_role(GuildId) ->
    #{
        <<"id">> => GuildId,
        <<"name">> => <<"everyone">>,
        <<"permissions">> =>
            constants:view_channel_permission() bor constants:view_channel_members_permission(),
        <<"hoist">> => false,
        <<"position">> => 0
    }.

bench_channel(ChannelId) ->
    #{
        <<"id">> => ChannelId,
        <<"name">> => <<"bench">>,
        <<"type">> => 0,
        <<"permission_overwrites">> => []
    }.

bench_member(UserId) ->
    UserIdBin = integer_to_binary(UserId),
    #{
        <<"user">> => #{
            <<"id">> => UserIdBin,
            <<"username">> => <<"bench_user_", UserIdBin/binary>>
        },
        <<"roles">> => []
    }.

with_batched_fanout_relay_mock(Fun) ->
    meck:new(gateway_dispatch_relay, [passthrough, no_link]),
    Parent = self(),
    Ref = make_ref(),
    put(batched_fanout_ref, Ref),
    meck:expect(
        gateway_dispatch_relay,
        dispatch_many,
        fun(Pids, guild_member_list_update, Payload, _GuildId) ->
            Parent ! {batched_fanout_dispatch, Ref, length(Pids), payload_bytes(Payload)},
            ok
        end
    ),
    try
        Fun()
    after
        erase(batched_fanout_ref),
        meck:unload(gateway_dispatch_relay)
    end.

payload_bytes({pre_encoded, Bin}) when is_binary(Bin) ->
    byte_size(Bin);
payload_bytes(Bin) when is_binary(Bin) ->
    byte_size(Bin);
payload_bytes(_) ->
    0.

collect_batched_fanout_dispatches(ExpectedCount) ->
    Ref = get(batched_fanout_ref),
    collect_batched_fanout_dispatches(ExpectedCount, Ref, 0, 0, 0).

collect_batched_fanout_dispatches(0, _Ref, Dispatches, Recipients, Bytes) ->
    {Dispatches, Recipients, Bytes};
collect_batched_fanout_dispatches(ExpectedCount, Ref, Dispatches, Recipients, Bytes) ->
    receive
        {batched_fanout_dispatch, Ref, PidCount, PayloadBytes} ->
            collect_batched_fanout_dispatches(
                ExpectedCount - 1,
                Ref,
                Dispatches + 1,
                Recipients + PidCount,
                Bytes + PayloadBytes
            )
    after 30000 ->
        error({batched_fanout_dispatch_timeout, ExpectedCount})
    end.

assert_no_batched_fanout_dispatch() ->
    Ref = get(batched_fanout_ref),
    receive
        {batched_fanout_dispatch, Ref, PidCount, PayloadBytes} ->
            ?assert(false, {unexpected_batched_fanout_dispatch, PidCount, PayloadBytes})
    after 100 ->
        ok
    end.

generate_members(N) ->
    NumHoisted = length(?HOISTED_ROLES),
    [
        {I, iolist_to_binary(io_lib:format("user_~10..0B", [I])),
            case I rem (NumHoisted + 3) of
                R when R < NumHoisted -> [lists:nth(R + 1, ?HOISTED_ROLES)];
                _ -> []
            end,
            rand:uniform() < ?ONLINE_RATIO}
     || I <- lists:seq(1, N)
    ].

bench_avg(Iterations, Fun) ->
    {TotalUs, Results} = timer:tc(fun() ->
        [Fun(I) || I <- lists:seq(1, Iterations)]
    end),
    AvgUs = TotalUs / float(Iterations),
    {AvgUs, Results}.

us_to_ms(Us) -> Us / 1000.0.
us_to_s(Us) -> Us / 1_000_000.0.

ops_per_second(Count, Us) -> trunc(Count / us_to_s(Us)).

log(Fmt, Args) ->
    Msg = lists:flatten(io_lib:format(Fmt, Args)),
    ?debugMsg(Msg).
