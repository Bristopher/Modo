%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_hotpatch_k8s_smoke).
-typing([eqwalizer]).

-export([main/0]).

-define(SIGNER_KEY_ID, <<"ops">>).
-define(CREATED_BY, <<"local-k8s-hotpatch-smoke">>).
-define(PATCH_MODULE, gateway_hotpatch_smoke_patch).
-define(PATCH_BASELINE_VERSION, 1).
-define(PATCH_TARGET_VERSION, 2).

-spec main() -> no_return().
main() ->
    try
        run(),
        halt(0)
    catch
        Class:Reason:Stacktrace ->
            write_stderr("hotpatch k8s smoke failed: ~0tp:~0tp~n~0tp~n", [
                Class, Reason, Stacktrace
            ]),
            halt(1)
    end.

-spec run() -> ok.
run() ->
    ok = ensure_runtime(),
    ok = ensure_patch_module_baseline(),
    Config = hotpatch_config(),
    PrivateKey = private_key(),
    ok = require_enabled(),
    ok = require_store(),
    ok = prepare_startup_events(Config, PrivateKey),
    StartupApplyMs = startup_reconcile(Config),
    ok = maybe_assert_patch_version(maps:get(expected_before, Config)),
    LiveApplyMs = live_reconcile(Config, PrivateKey),
    ok = maybe_assert_patch_version(
        maps:get(expected_before, Config) + maps:get(post_start_event_count, Config)
    ),
    ok = write_success(Config, StartupApplyMs, LiveApplyMs).

-spec ensure_runtime() -> ok.
ensure_runtime() ->
    ensure_started(crypto),
    ensure_started(ezstd),
    _ = fluxer_gateway_env:load(),
    ok.

-spec hotpatch_config() -> map().
hotpatch_config() ->
    BuildSha = gateway_hotpatch_runtime:build_sha(),
    EventCount = int_env("HOTPATCH_EVENT_COUNT", 8),
    PostStartEventCount = int_env("HOTPATCH_POST_START_EVENT_COUNT", 1),
    #{
        build_sha => BuildSha,
        event_count => EventCount,
        post_start_event_count => PostStartEventCount,
        expected_before => max(int_env("HOTPATCH_EXPECT_EVENT_COUNT", EventCount), EventCount),
        timeout_ms => int_env("HOTPATCH_TIMEOUT_MS", 120000),
        max_startup_apply_ms => int_env("HOTPATCH_MAX_STARTUP_APPLY_MS", 30000),
        max_live_apply_ms => int_env("HOTPATCH_MAX_LIVE_APPLY_MS", 30000)
    }.

-spec prepare_startup_events(map(), binary()) -> ok.
prepare_startup_events(#{build_sha := BuildSha, event_count := EventCount}, PrivateKey) ->
    append_events(BuildSha, PrivateKey, EventCount).

-spec startup_reconcile(map()) -> non_neg_integer().
startup_reconcile(#{
    expected_before := ExpectedBefore,
    timeout_ms := TimeoutMs,
    max_startup_apply_ms := MaxStartupApplyMs
}) ->
    StartupStartedMs = now_ms(),
    {ok, _Pid} = gateway_hotpatch_reconciler:start_link(),
    ok = wait_ready(ExpectedBefore, TimeoutMs),
    StartupApplyMs = elapsed_ms(StartupStartedMs),
    ok = assert_elapsed(startup_apply, StartupApplyMs, MaxStartupApplyMs),
    StartupApplyMs.

-spec live_reconcile(map(), binary()) -> non_neg_integer().
live_reconcile(
    #{
        build_sha := BuildSha,
        post_start_event_count := PostStartEventCount,
        expected_before := ExpectedBefore,
        timeout_ms := TimeoutMs,
        max_live_apply_ms := MaxLiveApplyMs
    },
    PrivateKey
) ->
    LiveApplyMs = apply_post_start_events(
        BuildSha, PrivateKey, PostStartEventCount, ExpectedBefore, TimeoutMs
    ),
    ok = assert_elapsed(live_apply, LiveApplyMs, MaxLiveApplyMs),
    LiveApplyMs.

-spec write_success(map(), non_neg_integer(), non_neg_integer()) -> ok.
write_success(#{build_sha := BuildSha}, StartupApplyMs, LiveApplyMs) ->
    Status = gateway_hotpatch_reconciler:status(),
    write_stdout(
        "hotpatch k8s smoke ok node=~ts build_sha=~ts startup_apply_ms=~p "
        "live_apply_ms=~p status=~0tp~n",
        [gateway_hotpatch_runtime:node_name(), BuildSha, StartupApplyMs, LiveApplyMs, Status]
    ),
    ok.

-spec apply_post_start_events(
    binary(), binary(), non_neg_integer(), non_neg_integer(), pos_integer()
) ->
    non_neg_integer().
apply_post_start_events(_BuildSha, _PrivateKey, 0, _ExpectedBefore, _TimeoutMs) ->
    0;
apply_post_start_events(BuildSha, PrivateKey, PostStartEventCount, ExpectedBefore, TimeoutMs) ->
    ok = append_events(BuildSha, PrivateKey, PostStartEventCount),
    StartedMs = now_ms(),
    gateway_hotpatch_reconciler:reconcile_async(),
    ok = wait_ready(ExpectedBefore + PostStartEventCount, TimeoutMs),
    elapsed_ms(StartedMs).

-spec ensure_started(atom()) -> ok.
ensure_started(App) ->
    case application:ensure_all_started(App) of
        {ok, _Apps} -> ok;
        {error, {App, {already_started, App}}} -> ok;
        {error, {already_started, App}} -> ok;
        {error, Reason} -> erlang:error({app_start_failed, App, Reason})
    end.

-spec require_enabled() -> ok.
require_enabled() ->
    case gateway_hotpatch_runtime:is_enabled() of
        true -> ok;
        false -> erlang:error(hotpatch_not_enabled)
    end.

-spec require_store() -> ok.
require_store() ->
    case gateway_hotpatch_store:connect() of
        ok -> ok;
        {error, Reason} -> erlang:error({hotpatch_store_connect_failed, Reason})
    end.

-spec private_key() -> binary().
private_key() ->
    Encoded = must_env("FLUXER_GATEWAY_HOTPATCH_PRIVATE_KEY_BASE64"),
    try base64:decode(Encoded) of
        PrivateKey when byte_size(PrivateKey) =:= 32 ->
            PrivateKey;
        PrivateKey ->
            erlang:error({invalid_private_key_size, byte_size(PrivateKey)})
    catch
        Class:Reason -> erlang:error({invalid_private_key_base64, Class, Reason})
    end.

-spec append_events(binary(), binary(), non_neg_integer()) -> ok.
append_events(_BuildSha, _PrivateKey, 0) ->
    ok;
append_events(BuildSha, PrivateKey, Count) when Count > 0 ->
    lists:foreach(
        fun(_Index) ->
            {ok, EventId} = append_event(BuildSha, PrivateKey),
            write_stdout("appended hotpatch event ~ts~n", [gateway_hotpatch_loader:hex(EventId)])
        end,
        lists:seq(1, Count)
    ).

-spec append_event(binary(), binary()) -> {ok, binary()}.
append_event(BuildSha, PrivateKey) ->
    {ok, Bundle} = patch_bundle(BuildSha),
    {ok, CompressedBundle} = gateway_hotpatch_bundle:compress_term(Bundle),
    {ok, Signature} = gateway_hotpatch_bundle:sign(CompressedBundle, PrivateKey),
    BundleSha256 = gateway_hotpatch_bundle:bundle_hash(CompressedBundle),
    case
        gateway_hotpatch_store:append_event(
            BuildSha,
            ?CREATED_BY,
            ?SIGNER_KEY_ID,
            Signature,
            BundleSha256,
            CompressedBundle
        )
    of
        {ok, EventId} -> {ok, EventId};
        {error, Reason} -> erlang:error({append_event_failed, Reason})
    end.

-spec patch_bundle(binary()) -> {ok, map()}.
patch_bundle(BuildSha) ->
    Module = ?PATCH_MODULE,
    ok = ensure_patch_module_loaded(),
    {ok, CurrentMd5} = gateway_hotpatch_loader:current_md5(Module),
    TargetBeam = compile_patch_module(?PATCH_TARGET_VERSION),
    {ok, TargetMd5} = gateway_hotpatch_loader:beam_md5(TargetBeam),
    {ok, BeamZstd} = compress_beam(TargetBeam),
    {ok, #{
        version => 1,
        build_sha => BuildSha,
        modules => [
            #{
                module => Module,
                expected_current_md5 => CurrentMd5,
                target_md5 => TargetMd5,
                beam_zstd => BeamZstd
            }
        ]
    }}.

-spec ensure_patch_module_baseline() -> ok.
ensure_patch_module_baseline() ->
    Module = ?PATCH_MODULE,
    Beam = compile_patch_module(?PATCH_BASELINE_VERSION),
    BeamPath = patch_beam_path(Module),
    ok = file:write_file(BeamPath, Beam),
    _ = code:soft_purge(Module),
    _ = code:purge(Module),
    _ = code:delete(Module),
    _ = code:purge(Module),
    case code:load_abs(filename:rootname(BeamPath)) of
        {module, Module} -> ok;
        {error, Reason} -> erlang:error({patch_baseline_load_failed, Reason})
    end.

-spec ensure_patch_module_loaded() -> ok.
ensure_patch_module_loaded() ->
    case code:is_loaded(?PATCH_MODULE) of
        false -> ensure_patch_module_baseline();
        {_File, _Loaded} -> ok
    end.

-spec compile_patch_module(pos_integer()) -> binary().
compile_patch_module(Version) ->
    Module = ?PATCH_MODULE,
    Source = patch_source_path(Module),
    SourceText = io_lib:format(
        "-module(~p).~n-export([version/0]).~nversion() -> ~p.~n",
        [Module, Version]
    ),
    ok = file:write_file(Source, SourceText),
    case compile:file(Source, [binary, return_errors, return_warnings]) of
        {ok, Module, Beam} when is_binary(Beam) -> Beam;
        {ok, Module, Beam, _Warnings} when is_binary(Beam) -> Beam;
        Error -> erlang:error({patch_module_compile_failed, Module, Version, Error})
    end.

-spec patch_source_path(atom()) -> file:filename().
patch_source_path(Module) ->
    filename:join(patch_compile_dir(), atom_to_list(Module) ++ ".erl").

-spec patch_beam_path(atom()) -> file:filename().
patch_beam_path(Module) ->
    filename:join(patch_compile_dir(), atom_to_list(Module) ++ ".beam").

-spec patch_compile_dir() -> file:filename().
patch_compile_dir() ->
    Root =
        case os:getenv("TMPDIR") of
            false -> "/tmp";
            Value -> Value
        end,
    Dir = filename:join(Root, "fluxer_hotpatch_k8s_smoke"),
    ok = filelib:ensure_dir(filename:join(Dir, "placeholder")),
    Dir.

-spec maybe_assert_patch_version(non_neg_integer()) -> ok.
maybe_assert_patch_version(0) ->
    ok;
maybe_assert_patch_version(_ExpectedAppliedCount) ->
    assert_patch_version(?PATCH_TARGET_VERSION).

-spec assert_patch_version(pos_integer()) -> ok.
assert_patch_version(ExpectedVersion) ->
    case erlang:apply(?PATCH_MODULE, version, []) of
        ExpectedVersion -> ok;
        OtherVersion -> erlang:error({patch_version_mismatch, OtherVersion, ExpectedVersion})
    end.

-spec compress_beam(binary()) -> {ok, binary()}.
compress_beam(Beam) ->
    try erlang:apply(ezstd, compress, [Beam, 3]) of
        Compressed when is_binary(Compressed) -> {ok, Compressed};
        {error, Reason} -> erlang:error({beam_compress_failed, Reason});
        Other -> erlang:error({beam_compress_failed, Other})
    catch
        Class:Reason -> erlang:error({beam_compress_failed, Class, Reason})
    end.

-spec wait_ready(non_neg_integer(), pos_integer()) -> ok.
wait_ready(ExpectedAppliedCount, TimeoutMs) ->
    Deadline = erlang:monotonic_time(millisecond) + TimeoutMs,
    wait_ready_until(ExpectedAppliedCount, Deadline).

-spec wait_ready_until(non_neg_integer(), integer()) -> ok.
wait_ready_until(ExpectedAppliedCount, Deadline) ->
    Status = gateway_hotpatch_reconciler:status(),
    Ready = maps:get(ready, Status, false),
    AppliedCount = maps:get(applied_count, Status, 0),
    case Ready andalso AppliedCount >= ExpectedAppliedCount of
        true ->
            ok;
        false ->
            case erlang:monotonic_time(millisecond) >= Deadline of
                true ->
                    erlang:error({hotpatch_not_ready, ExpectedAppliedCount, Status});
                false ->
                    timer:sleep(500),
                    gateway_hotpatch_reconciler:reconcile_async(),
                    wait_ready_until(ExpectedAppliedCount, Deadline)
            end
    end.

-spec assert_elapsed(atom(), non_neg_integer(), integer()) -> ok.
assert_elapsed(_Label, _ElapsedMs, MaxMs) when MaxMs =< 0 ->
    ok;
assert_elapsed(_Label, ElapsedMs, MaxMs) when ElapsedMs =< MaxMs ->
    ok;
assert_elapsed(Label, ElapsedMs, MaxMs) ->
    erlang:error({hotpatch_elapsed_ms_exceeded, Label, ElapsedMs, MaxMs}).

-spec now_ms() -> integer().
now_ms() ->
    erlang:monotonic_time(millisecond).

-spec elapsed_ms(integer()) -> non_neg_integer().
elapsed_ms(StartedMs) ->
    max(0, now_ms() - StartedMs).

-spec int_env(string(), integer()) -> integer().
int_env(Name, Default) ->
    case os:getenv(Name) of
        false ->
            Default;
        "" ->
            Default;
        Value ->
            try list_to_integer(string:trim(Value)) of
                Parsed -> Parsed
            catch
                error:badarg -> erlang:error({invalid_integer_env, Name, Value})
            end
    end.

-spec must_env(string()) -> string().
must_env(Name) ->
    case os:getenv(Name) of
        false -> erlang:error({missing_env, Name});
        "" -> erlang:error({missing_env, Name});
        Value -> Value
    end.

-spec write_stdout(io:format(), [term()]) -> ok.
write_stdout(Format, Args) ->
    write_stream(standard_io, Format, Args).

-spec write_stderr(io:format(), [term()]) -> ok.
write_stderr(Format, Args) ->
    write_stream(standard_error, Format, Args).

-spec write_stream(file:io_device() | standard_io | standard_error, io:format(), [term()]) ->
    ok.
write_stream(Device, Format, Args) ->
    Output = iolist_to_binary(io_lib:format(Format, Args)),
    _ = file:write(Device, Output),
    ok.
