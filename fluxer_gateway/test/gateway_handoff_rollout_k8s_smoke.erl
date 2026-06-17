%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_handoff_rollout_k8s_smoke).
-behaviour(gen_server).

-export([
    node_main/0,
    controller_main/0,
    accept_transfer/1,
    count/0,
    drain_to/1,
    increment_if_owner/1,
    seed/2,
    snapshot/0,
    version/0
]).
-export([init/1, handle_call/3, handle_cast/2, handle_info/2, terminate/2, code_change/3]).

-define(SERVER, ?MODULE).
-define(METRICS, gateway_handoff_rollout_metrics).
-define(NAME_PREFIX, "handoff-rollout").
-define(DEFAULT_CLUSTER, "sessions").
-define(CONTAINER, "node").

node_main() ->
    try
        ok = ensure_node_runtime(),
        write_stdout("handoff rollout node ready cluster=~ts node=~p version=~ts~n", [
            cluster_name(), node(), version()
        ]),
        wait_forever()
    catch
        Class:Reason:Stacktrace ->
            write_stderr("handoff rollout node failed: ~0tp:~0tp~n~0tp~n", [
                Class, Reason, Stacktrace
            ]),
            halt(1)
    end.

controller_main() ->
    try
        ok = ensure_controller_runtime(),
        run_controller(),
        halt(0)
    catch
        Class:Reason:Stacktrace ->
            write_stderr("handoff rollout controller failed: ~0tp:~0tp~n~0tp~n", [
                Class, Reason, Stacktrace
            ]),
            halt(1)
    end.

seed(Ids, Epoch) when is_list(Ids), is_binary(Epoch) ->
    gen_server:call(?SERVER, {seed, Ids, Epoch}, 60000).

count() ->
    gen_server:call(?SERVER, count, 5000).

snapshot() ->
    gen_server:call(?SERVER, snapshot, 10000).

version() ->
    type_conv:ensure_binary(os:getenv("HANDOFF_IMAGE_VERSION"), <<"unknown">>).

drain_to(TargetNodes) when is_list(TargetNodes) ->
    gen_server:call(?SERVER, {drain_to, TargetNodes}, 120000).

accept_transfer(Id) when is_binary(Id) ->
    case session_state_transfer:pop_state(Id) of
        {ok, Entity} when is_map(Entity) ->
            gen_server:call(?SERVER, {accept, Id, Entity}, 5000);
        _ ->
            {error, not_found}
    end.

increment_if_owner(Id) when is_binary(Id) ->
    gen_server:call(?SERVER, {increment, Id}, 5000).

init([]) ->
    {ok, #{entities => #{}}}.

handle_call({seed, Ids, Epoch}, _From, State) ->
    Entities0 = maps:get(entities, State),
    Entities = lists:foldl(fun(Id, Acc) -> seed_entity(Id, Epoch, Acc) end, Entities0, Ids),
    {reply, {ok, maps:size(Entities)}, State#{entities := Entities}};
handle_call(count, _From, State) ->
    {reply, maps:size(maps:get(entities, State)), State};
handle_call(snapshot, _From, State) ->
    {reply, maps:values(maps:get(entities, State)), State};
handle_call({accept, Id, Entity}, _From, State) ->
    Entities = maps:get(entities, State),
    {reply, ok, State#{entities := Entities#{Id => Entity}}};
handle_call({increment, Id}, _From, State) ->
    handle_increment(Id, State);
handle_call({drain_to, TargetNodes}, _From, State) ->
    handle_drain(TargetNodes, State);
handle_call(_Request, _From, State) ->
    {reply, ok, State}.

handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info(_Info, State) ->
    {noreply, State}.

terminate(_Reason, _State) ->
    ok.

code_change(_OldVsn, State, _Extra) ->
    {ok, State}.

ensure_node_runtime() ->
    ok = ensure_started(crypto),
    ok = ensure_session_transfer(),
    {ok, _Pid} = gen_server:start_link({local, ?SERVER}, ?MODULE, [], []),
    ok.

ensure_controller_runtime() ->
    ok = ensure_started(crypto),
    ensure_metrics(),
    ok.

ensure_started(App) ->
    case application:ensure_all_started(App) of
        {ok, _Apps} -> ok;
        {error, {App, {already_started, App}}} -> ok;
        {error, {already_started, App}} -> ok;
        {error, Reason} -> erlang:error({app_start_failed, App, Reason})
    end.

ensure_session_transfer() ->
    case whereis(session_state_transfer) of
        undefined ->
            {ok, _Pid} = session_state_transfer:start_link(),
            ok;
        Pid when is_pid(Pid) ->
            ok
    end.

wait_forever() ->
    receive
        stop -> halt(0)
    after 86400000 ->
        wait_forever()
    end.

seed_entity(Id, Epoch, Entities) when is_binary(Id) ->
    Entity = #{
        id => Id,
        seq => 0,
        owner_version => version(),
        epoch => Epoch,
        checksum => entity_checksum(Id, Epoch, 0)
    },
    Entities#{Id => Entity};
seed_entity(_Id, _Epoch, Entities) ->
    Entities.

handle_increment(Id, State) ->
    Entities = maps:get(entities, State),
    case maps:get(Id, Entities, undefined) of
        undefined -> {reply, not_found, State};
        Entity -> reply_incremented(Id, Entity, Entities, State)
    end.

reply_incremented(Id, Entity, Entities, State) ->
    Seq = maps:get(seq, Entity, 0) + 1,
    Epoch = maps:get(epoch, Entity, <<"seed">>),
    Updated = Entity#{
        seq := Seq,
        owner_version := version(),
        checksum := entity_checksum(Id, Epoch, Seq)
    },
    {reply, {ok, Seq}, State#{entities := Entities#{Id := Updated}}}.

handle_drain(TargetNodes, State) ->
    Entities = maps:get(entities, State),
    {Moved, Failed, Remaining} = drain_entities(maps:to_list(Entities), TargetNodes, 0, 0, #{}),
    Reply = #{moved => Moved, failed => Failed, remaining => maps:size(Remaining)},
    {reply, {ok, Reply}, State#{entities := Remaining}}.

drain_entities([], _TargetNodes, Moved, Failed, Remaining) ->
    {Moved, Failed, Remaining};
drain_entities([{Id, Entity} | Rest], TargetNodes, Moved, Failed, Remaining) ->
    TargetNode = select_target(Id, TargetNodes),
    case transfer_entity(TargetNode, Id, Entity) of
        ok ->
            drain_entities(Rest, TargetNodes, Moved + 1, Failed, Remaining);
        {error, _Reason} ->
            drain_entities(Rest, TargetNodes, Moved, Failed + 1, Remaining#{Id => Entity})
    end.

transfer_entity(undefined, _Id, _Entity) ->
    {error, no_target};
transfer_entity(TargetNode, Id, Entity) ->
    case session_state_transfer:push_state(TargetNode, Id, Entity) of
        ok -> rpc_accept_entity(TargetNode, Id);
        {error, Reason} -> {error, Reason}
    end.

rpc_accept_entity(TargetNode, Id) ->
    case rpc:call(TargetNode, ?MODULE, accept_transfer, [Id], 10000) of
        ok -> ok;
        {error, Reason} -> {error, Reason};
        Other -> {error, {accept_failed, TargetNode, Other}}
    end.

select_target(_Id, []) ->
    undefined;
select_target(Id, TargetNodes) ->
    Index = erlang:phash2(Id, length(TargetNodes)),
    lists:nth(Index + 1, TargetNodes).

entity_checksum(Id, Epoch, Seq) ->
    crypto:hash(sha256, term_to_binary({Id, Epoch, Seq})).

run_controller() ->
    Config = controller_config(),
    Nodes = prepare_cluster(Config),
    Workers = start_workload(
        maps:get(workload_concurrency, Config),
        maps:get(entity_count, Config),
        Nodes
    ),
    FinalNodes = rollout_cluster(Config, Nodes),
    stop_workload(Workers),
    ok = verify_rollout(Config, FinalNodes).

controller_config() ->
    Cluster = cluster_name(),
    EntityCount = int_env("HANDOFF_ENTITY_COUNT", 600),
    #{
        cluster => Cluster,
        namespace => namespace(),
        replicas => int_env("HANDOFF_REPLICAS", 3),
        entity_count => EntityCount,
        workload_concurrency => int_env("HANDOFF_WORKLOAD_CONCURRENCY", 8),
        max_latency_ms => int_env("HANDOFF_MAX_INCREMENT_LATENCY_MS", 5000),
        min_workload_ops => int_env("HANDOFF_MIN_WORKLOAD_OPS", EntityCount),
        image_v2 => must_env("HANDOFF_IMAGE_V2")
    }.

prepare_cluster(#{namespace := Namespace, replicas := Replicas, entity_count := EntityCount}) ->
    ok = kubectl_wait_initial_rollout(Namespace),
    Nodes = current_nodes(Namespace, Replicas),
    ok = wait_nodes(Nodes),
    ok = seed_cluster(Nodes, EntityCount),
    ok = assert_cluster(Nodes, EntityCount),
    Nodes.

rollout_cluster(
    #{
        namespace := Namespace,
        image_v2 := ImageV2,
        replicas := Replicas,
        entity_count := EntityCount
    },
    Nodes
) ->
    ok = set_next_revision(Namespace, ImageV2),
    rollout_ordinals(Namespace, Replicas, EntityCount, Nodes).

verify_rollout(
    #{
        cluster := Cluster,
        entity_count := EntityCount,
        max_latency_ms := MaxLatencyMs,
        min_workload_ops := MinWorkloadOps
    },
    FinalNodes
) ->
    Metrics = metrics(),
    ok = assert_workload(Metrics, MaxLatencyMs, MinWorkloadOps),
    ok = assert_versions(FinalNodes, <<"v2">>),
    FinalSnapshot = cluster_snapshot(FinalNodes),
    ok = assert_cluster_snapshot(FinalSnapshot, EntityCount),
    ok = assert_workload_effects(FinalSnapshot, Metrics),
    write_stdout("handoff rollout local Kubernetes smoke passed cluster=~ts metrics=~0tp~n", [
        Cluster, Metrics
    ]),
    ok.

kubectl_wait_initial_rollout(Namespace) ->
    kubectl(Namespace, "rollout status statefulset/" ++ statefulset() ++ " --timeout=180s").

seed_cluster(Nodes, EntityCount) ->
    Partitions = partition_ids(Nodes, EntityCount),
    Epoch = iolist_to_binary([cluster_name(), "-seed-v1"]),
    lists:foreach(fun({Node, Ids}) -> rpc_expect(Node, seed, [Ids, Epoch]) end, Partitions),
    ok.

partition_ids(Nodes, EntityCount) ->
    Empty = [{Node, []} || Node <- Nodes],
    lists:foldl(fun add_partition_id/2, Empty, entity_ids(EntityCount)).

add_partition_id(Id, Partitions) ->
    Index = erlang:phash2(Id, length(Partitions)) + 1,
    {Node, Ids} = lists:nth(Index, Partitions),
    lists:sublist(Partitions, Index - 1) ++ [{Node, [Id | Ids]}] ++
        lists:nthtail(Index, Partitions).

rollout_ordinals(Namespace, Replicas, EntityCount, Nodes0) ->
    lists:foldl(
        fun(Ordinal, Nodes) -> rollout_ordinal(Namespace, Ordinal, EntityCount, Nodes) end,
        Nodes0,
        lists:seq(Replicas - 1, 0, -1)
    ).

rollout_ordinal(Namespace, Ordinal, EntityCount, Nodes0) ->
    Source = lists:nth(Ordinal + 1, Nodes0),
    Survivors = lists:delete(Source, Nodes0),
    ok = put_workload_nodes(Survivors),
    ok = drain_and_verify(Source, Survivors),
    ok = patch_partition(Namespace, Ordinal),
    ok = wait_pod_ready(Namespace, Ordinal),
    Nodes1 = replace_ordinal_node(Namespace, Ordinal, Nodes0),
    ok = wait_nodes(Nodes1),
    ok = wait_node_version(lists:nth(Ordinal + 1, Nodes1), <<"v2">>),
    ok = put_workload_nodes(Nodes1),
    ok = assert_cluster(Nodes1, EntityCount),
    Nodes1.

drain_and_verify(Source, Survivors) ->
    case rpc:call(Source, ?MODULE, drain_to, [Survivors], 120000) of
        {ok, #{failed := 0}} -> wait_empty(Source);
        Other -> erlang:error({drain_failed, Source, Other})
    end.

patch_partition(Namespace, Ordinal) ->
    Patch =
        "{\"spec\":{\"updateStrategy\":{\"rollingUpdate\":{\"partition\":" ++
            integer_to_list(Ordinal) ++ "}}}}",
    kubectl(
        Namespace,
        "patch statefulset " ++ statefulset() ++ " --type merge -p " ++
            shell_quote(Patch)
    ).

wait_pod_ready(Namespace, Ordinal) ->
    Pod = pod_name(Ordinal),
    ok = kubectl(Namespace, "wait --for=condition=Ready pod/" ++ Pod ++ " --timeout=180s"),
    wait_pod_observed_v2(Namespace, Pod).

wait_pod_observed_v2(Namespace, Pod) ->
    wait_until(
        fun() ->
            Image = string:trim(
                kubectl_out(
                    Namespace,
                    "get pod " ++ Pod ++
                        " -o jsonpath='{.spec.containers[0].image}'"
                )
            ),
            has_suffix(Image, ":v2")
        end,
        {pod_image_v2, Pod},
        60,
        1000
    ).

replace_ordinal_node(Namespace, Ordinal, Nodes0) ->
    Node = pod_node(Namespace, Ordinal),
    lists:sublist(Nodes0, Ordinal) ++ [Node] ++ lists:nthtail(Ordinal + 1, Nodes0).

set_next_revision(Namespace, ImageV2) ->
    ok = kubectl(
        Namespace,
        "set image statefulset/" ++ statefulset() ++ " " ?CONTAINER "=" ++
            ImageV2
    ),
    kubectl(Namespace, "set env statefulset/" ++ statefulset() ++ " HANDOFF_IMAGE_VERSION=v2").

wait_nodes(Nodes) ->
    lists:foreach(fun wait_node/1, Nodes),
    ok = put_workload_nodes(Nodes),
    ok.

wait_node(Node) ->
    wait_until(
        fun() -> node_connectable(Node) andalso rpc_version(Node) =/= undefined end,
        {node_ready, Node},
        120,
        1000
    ).

node_connectable(Node) ->
    case net_kernel:connect_node(Node) of
        true -> true;
        false -> false;
        ignored -> false
    end.

rpc_version(Node) ->
    case rpc:call(Node, ?MODULE, version, [], 5000) of
        Version when is_binary(Version) -> Version;
        _ -> undefined
    end.

wait_node_version(Node, Version) ->
    wait_until(
        fun() -> rpc_version(Node) =:= Version end, {node_version, Node, Version}, 120, 1000
    ).

wait_empty(Node) ->
    wait_until(fun() -> rpc_count(Node) =:= 0 end, {node_empty, Node}, 120, 500).

assert_versions(Nodes, Expected) ->
    Versions = [{Node, rpc_version(Node)} || Node <- Nodes],
    case lists:all(fun({_Node, Version}) -> Version =:= Expected end, Versions) of
        true -> ok;
        false -> erlang:error({unexpected_versions, Versions})
    end.

assert_cluster(Nodes, EntityCount) ->
    Snapshot = cluster_snapshot(Nodes),
    assert_cluster_snapshot(Snapshot, EntityCount).

assert_cluster_snapshot(Snapshot, EntityCount) ->
    ok = assert_entity_count(Snapshot, EntityCount),
    ok = assert_unique_entities(Snapshot, EntityCount),
    ok = assert_checksums(Snapshot).

assert_entity_count(Snapshot, EntityCount) ->
    case length(Snapshot) of
        EntityCount -> ok;
        Actual -> erlang:error({entity_count_mismatch, Actual, EntityCount})
    end.

assert_unique_entities(Snapshot, EntityCount) ->
    Unique = lists:usort([maps:get(id, Entity) || Entity <- Snapshot]),
    case length(Unique) of
        EntityCount -> ok;
        Actual -> erlang:error({duplicate_or_missing_entities, Actual, EntityCount})
    end.

assert_checksums(Snapshot) ->
    Bad = [Entity || Entity <- Snapshot, not valid_checksum(Entity)],
    case Bad of
        [] -> ok;
        _ -> erlang:error({bad_entity_checksums, length(Bad)})
    end.

valid_checksum(#{id := Id, epoch := Epoch, seq := Seq, checksum := Checksum}) ->
    entity_checksum(Id, Epoch, Seq) =:= Checksum;
valid_checksum(_Entity) ->
    false.

cluster_snapshot(Nodes) ->
    case retry_cluster_snapshot(Nodes, 20) of
        {ok, Snapshot} -> Snapshot;
        {error, Reason} -> erlang:error(Reason)
    end.

retry_cluster_snapshot(Nodes, Attempts) ->
    connect_nodes(Nodes),
    case collect_cluster_snapshot(Nodes, []) of
        {ok, Snapshot} ->
            {ok, Snapshot};
        {error, _Reason} when Attempts > 0 ->
            timer:sleep(500),
            retry_cluster_snapshot(Nodes, Attempts - 1);
        {error, Reason} ->
            {error, Reason}
    end.

connect_nodes(Nodes) ->
    lists:foreach(fun(Node) -> _ = net_kernel:connect_node(Node) end, Nodes).

collect_cluster_snapshot([], Acc) ->
    {ok, lists:append(lists:reverse(Acc))};
collect_cluster_snapshot([Node | Rest], Acc) ->
    case rpc_snapshot(Node) of
        {ok, Snapshot} -> collect_cluster_snapshot(Rest, [Snapshot | Acc]);
        {error, Reason} -> {error, Reason}
    end.

rpc_snapshot(Node) ->
    case rpc:call(Node, ?MODULE, snapshot, [], 10000) of
        Snapshot when is_list(Snapshot) -> {ok, Snapshot};
        Other -> {error, {snapshot_failed, Node, Other}}
    end.

rpc_count(Node) ->
    case rpc:call(Node, ?MODULE, count, [], 5000) of
        Count when is_integer(Count) -> Count;
        _ -> -1
    end.

rpc_expect(Node, Function, Args) ->
    case rpc:call(Node, ?MODULE, Function, Args, 60000) of
        {ok, _Value} -> ok;
        ok -> ok;
        Other -> erlang:error({rpc_failed, Node, Function, Other})
    end.

start_workload(Concurrency, EntityCount, Nodes) ->
    ok = put_workload_nodes(Nodes),
    [spawn_link(fun() -> workload_loop(EntityCount) end) || _ <- lists:seq(1, Concurrency)].

stop_workload(Workers) ->
    ets:insert(?METRICS, {stop, true}),
    lists:foreach(fun wait_worker_down/1, Workers).

wait_worker_down(Pid) ->
    Ref = monitor(process, Pid),
    receive
        {'DOWN', Ref, process, Pid, _Reason} -> ok
    after 5000 ->
        exit(Pid, kill),
        ok
    end.

workload_loop(EntityCount) ->
    case metric(stop, false) of
        true ->
            ok;
        false ->
            Id = entity_id(rand:uniform(EntityCount)),
            run_increment(Id),
            workload_loop(EntityCount)
    end.

run_increment(Id) ->
    Started = erlang:monotonic_time(microsecond),
    case increment_until_found(Id, workload_nodes(), 200) of
        ok -> record_increment_latency(Started);
        {error, Reason} -> record_increment_error(Id, Reason)
    end.

increment_until_found(_Id, _Nodes, 0) ->
    {error, not_found};
increment_until_found(Id, Nodes, Attempts) ->
    case try_increment_nodes(Id, Nodes) of
        ok ->
            ok;
        not_found ->
            timer:sleep(25),
            increment_until_found(Id, workload_nodes(), Attempts - 1)
    end.

try_increment_nodes(_Id, []) ->
    not_found;
try_increment_nodes(Id, [Node | Rest]) ->
    case rpc:call(Node, ?MODULE, increment_if_owner, [Id], 250) of
        {ok, _Seq} -> ok;
        _Other -> try_increment_nodes(Id, Rest)
    end.

record_increment_latency(Started) ->
    LatencyUs = erlang:monotonic_time(microsecond) - Started,
    ets:update_counter(?METRICS, ops, {2, 1}, {ops, 0}),
    update_max_latency(LatencyUs).

update_max_latency(LatencyUs) ->
    Current = metric(max_latency_us, 0),
    case LatencyUs > Current of
        true -> ets:insert(?METRICS, {max_latency_us, LatencyUs});
        false -> ok
    end.

record_increment_error(Id, Reason) ->
    ets:update_counter(?METRICS, errors, {2, 1}, {errors, 0}),
    ets:insert(?METRICS, {last_error, {Id, Reason}}).

assert_workload(Metrics, MaxLatencyMs, MinOps) ->
    #{ops := Ops, errors := Errors, max_latency_us := MaxLatencyUs} = Metrics,
    MaxLatencyLimitUs = MaxLatencyMs * 1000,
    case {Ops >= MinOps, Errors, MaxLatencyUs =< MaxLatencyLimitUs} of
        {true, 0, true} -> ok;
        _ -> erlang:error({workload_failed, Metrics, MaxLatencyMs, MinOps})
    end.

assert_workload_effects(Snapshot, #{ops := Ops} = Metrics) ->
    SeqTotal = lists:sum([maps:get(seq, Entity, 0) || Entity <- Snapshot]),
    case SeqTotal of
        Ops -> ok;
        _ -> erlang:error({workload_effect_mismatch, SeqTotal, Metrics})
    end.

metrics() ->
    #{
        ops => metric(ops, 0),
        errors => metric(errors, 0),
        max_latency_us => metric(max_latency_us, 0),
        last_error => metric(last_error, undefined)
    }.

ensure_metrics() ->
    case ets:info(?METRICS) of
        undefined -> ets:new(?METRICS, [named_table, public, set]);
        _ -> ok
    end,
    ets:insert(?METRICS, [{stop, false}, {ops, 0}, {errors, 0}, {max_latency_us, 0}]),
    ok.

put_workload_nodes(Nodes) ->
    ets:insert(?METRICS, {nodes, Nodes}),
    ok.

workload_nodes() ->
    metric(nodes, []).

metric(Key, Default) ->
    case ets:lookup(?METRICS, Key) of
        [{Key, Value}] -> Value;
        [] -> Default
    end.

current_nodes(Namespace, Replicas) ->
    [pod_node(Namespace, Ordinal) || Ordinal <- lists:seq(0, Replicas - 1)].

pod_node(Namespace, Ordinal) ->
    Pod = pod_name(Ordinal),
    Ip = wait_pod_ip(Namespace, Pod),
    list_to_atom("handoff@" ++ Ip).

wait_pod_ip(Namespace, Pod) ->
    wait_until(
        fun() ->
            valid_ip(
                kubectl_out(Namespace, "get pod " ++ Pod ++ " -o jsonpath='{.status.podIP}'")
            )
        end,
        {pod_ip, Pod},
        120,
        1000
    ),
    string:trim(kubectl_out(Namespace, "get pod " ++ Pod ++ " -o jsonpath='{.status.podIP}'")).

valid_ip(Value) ->
    string:trim(Value) =/= "".

pod_name(Ordinal) ->
    statefulset() ++ "-" ++ integer_to_list(Ordinal).

entity_ids(EntityCount) ->
    [entity_id(I) || I <- lists:seq(1, EntityCount)].

entity_id(I) ->
    iolist_to_binary([cluster_name(), "-entity-", integer_to_binary(I)]).

namespace() ->
    type_conv:ensure_binary(os:getenv("POD_NAMESPACE"), <<"fluxer-it">>).

cluster_name() ->
    case os:getenv("HANDOFF_CLUSTER_NAME") of
        false -> ?DEFAULT_CLUSTER;
        "" -> ?DEFAULT_CLUSTER;
        Value -> validate_cluster_name(string:trim(Value))
    end.

validate_cluster_name(Name) ->
    case Name =/= [] andalso lists:all(fun valid_cluster_char/1, Name) of
        true -> Name;
        false -> erlang:error({invalid_cluster_name, Name})
    end.

valid_cluster_char(Char) when Char >= $a, Char =< $z ->
    true;
valid_cluster_char(Char) when Char >= $0, Char =< $9 ->
    true;
valid_cluster_char($-) ->
    true;
valid_cluster_char(_Char) ->
    false.

statefulset() ->
    ?NAME_PREFIX ++ "-" ++ cluster_name().

int_env(Name, Default) ->
    case os:getenv(Name) of
        false -> Default;
        "" -> Default;
        Value -> list_to_integer(string:trim(Value))
    end.

must_env(Name) ->
    case os:getenv(Name) of
        false -> erlang:error({missing_env, Name});
        "" -> erlang:error({missing_env, Name});
        Value -> Value
    end.

kubectl(Namespace, Args) ->
    case run_cmd(kubectl_command(Namespace, Args)) of
        {ok, _Output} -> ok;
        {error, Status, Output} -> erlang:error({kubectl_failed, Status, Args, Output})
    end.

kubectl_out(Namespace, Args) ->
    case run_cmd(kubectl_command(Namespace, Args)) of
        {ok, Output} -> Output;
        {error, Status, Output} -> erlang:error({kubectl_failed, Status, Args, Output})
    end.

kubectl_command(Namespace, Args) ->
    "kubectl -n " ++ shell_quote(binary_to_list(Namespace)) ++ " " ++ Args.

run_cmd(Command) ->
    Port = open_port({spawn, Command}, [exit_status, stderr_to_stdout]),
    collect_cmd(Port, []).

collect_cmd(Port, Acc) ->
    receive
        {Port, {data, Data}} ->
            collect_cmd(Port, [Data | Acc]);
        {Port, {exit_status, 0}} ->
            {ok, lists:flatten(lists:reverse(Acc))};
        {Port, {exit_status, Status}} ->
            {error, Status, lists:flatten(lists:reverse(Acc))}
    after 300000 ->
        port_close(Port),
        {error, timeout, lists:flatten(lists:reverse(Acc))}
    end.

shell_quote(Value) ->
    "'" ++ escape_single_quotes(Value) ++ "'".

escape_single_quotes([]) ->
    [];
escape_single_quotes([$' | Rest]) ->
    "'\\''" ++ escape_single_quotes(Rest);
escape_single_quotes([Char | Rest]) ->
    [Char | escape_single_quotes(Rest)].

has_suffix(Value, Suffix) ->
    ValueLength = length(Value),
    SuffixLength = length(Suffix),
    ValueLength >= SuffixLength andalso
        lists:nthtail(ValueLength - SuffixLength, Value) =:= Suffix.

wait_until(Pred, Label, Retries, IntervalMs) ->
    case safe_pred(Pred) of
        true -> ok;
        false -> wait_again(Pred, Label, Retries, IntervalMs)
    end.

wait_again(_Pred, Label, 0, _IntervalMs) ->
    erlang:error({wait_timeout, Label});
wait_again(Pred, Label, Retries, IntervalMs) ->
    timer:sleep(IntervalMs),
    wait_until(Pred, Label, Retries - 1, IntervalMs).

safe_pred(Pred) ->
    try Pred() of
        true -> true;
        _ -> false
    catch
        _:_ -> false
    end.

write_stdout(Format, Args) ->
    write_stream(standard_io, Format, Args).

write_stderr(Format, Args) ->
    write_stream(standard_error, Format, Args).

write_stream(Device, Format, Args) ->
    Output = iolist_to_binary(io_lib:format(Format, Args)),
    _ = file:write(Device, Output),
    ok.
