%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_hotpatch_cli).
-typing([eqwalizer]).

-export([main/1, build_bundle/2, sign_bundle/3]).

-spec main([string()]) -> no_return().
main(Args) ->
    halt(run_main(Args)).

-spec run_main([string()]) -> non_neg_integer().
run_main(["bundle", BuildSha | ModuleOrder]) when ModuleOrder =/= [] ->
    write_json_result(build_bundle(type_conv:ensure_binary(BuildSha), ModuleOrder));
run_main(["sign", SignerKeyId, PrivateKeyPath, BundlePath]) ->
    write_json_result(sign_bundle_file(SignerKeyId, PrivateKeyPath, BundlePath));
run_main(["append", BuildSha, CreatedBy, EventPath]) ->
    Result = append_event(
        type_conv:ensure_binary(BuildSha), type_conv:ensure_binary(CreatedBy), EventPath
    ),
    write_append_result(Result);
run_main(_Args) ->
    write_usage(),
    64.

-spec build_bundle(binary(), [string()]) -> {ok, map()} | {error, term()}.
build_bundle(BuildSha, ModuleSpecs) when is_binary(BuildSha), is_list(ModuleSpecs) ->
    build_bundle_modules(ModuleSpecs, fun(Modules) ->
        {ok, #{<<"version">> => 1, <<"build_sha">> => BuildSha, <<"modules">> => Modules}}
    end).

-spec sign_bundle(binary(), file:filename(), binary()) -> {ok, map()} | {error, term()}.
sign_bundle(SignerKeyId, PrivateKeyPath, BundleJson) when
    is_binary(SignerKeyId), is_binary(BundleJson)
->
    case decode_json(BundleJson) of
        {ok, Bundle} -> sign_decoded_bundle_with_key(SignerKeyId, PrivateKeyPath, Bundle);
        {error, Reason} -> {error, Reason}
    end.

-spec sign_decoded_bundle_with_key(binary(), file:filename(), term()) ->
    {ok, map()} | {error, term()}.
sign_decoded_bundle_with_key(SignerKeyId, PrivateKeyPath, Bundle) ->
    case read_private_key(PrivateKeyPath) of
        {ok, PrivateKey} -> sign_decoded_bundle(SignerKeyId, Bundle, PrivateKey);
        {error, Reason} -> {error, Reason}
    end.

-spec sign_bundle_file(string(), file:filename(), file:filename()) ->
    {ok, map()} | {error, term()}.
sign_bundle_file(SignerKeyId, PrivateKeyPath, BundlePath) ->
    case file:read_file(BundlePath) of
        {ok, BundleJson} ->
            sign_bundle(type_conv:ensure_binary(SignerKeyId), PrivateKeyPath, BundleJson);
        {error, Reason} ->
            {error, {read_bundle_failed, Reason}}
    end.

-spec sign_decoded_bundle(binary(), term(), binary()) -> {ok, map()} | {error, term()}.
sign_decoded_bundle(SignerKeyId, BundleJson, PrivateKey) ->
    case json_to_bundle(BundleJson) of
        {ok, Bundle} -> sign_bundle_term(SignerKeyId, Bundle, PrivateKey);
        {error, Reason} -> {error, Reason}
    end.

-spec sign_bundle_term(binary(), map(), binary()) -> {ok, map()} | {error, term()}.
sign_bundle_term(SignerKeyId, Bundle, PrivateKey) ->
    case gateway_hotpatch_bundle:compress_term(Bundle) of
        {ok, Compressed} -> sign_compressed_bundle(SignerKeyId, Compressed, PrivateKey);
        {error, Reason} -> {error, Reason}
    end.

-spec sign_compressed_bundle(binary(), binary(), binary()) -> {ok, map()} | {error, term()}.
sign_compressed_bundle(SignerKeyId, Compressed, PrivateKey) ->
    case gateway_hotpatch_bundle:sign(Compressed, PrivateKey) of
        {ok, Signature} -> {ok, signed_event(SignerKeyId, Compressed, Signature)};
        {error, Reason} -> {error, Reason}
    end.

-spec signed_event(binary(), binary(), binary()) -> map().
signed_event(SignerKeyId, Compressed, Signature) ->
    #{
        <<"schema_version">> => 1,
        <<"kind">> => <<"beam_bundle">>,
        <<"created_by">> => gateway_hotpatch_runtime:node_name(),
        <<"signer_key_id">> => SignerKeyId,
        <<"bundle_sha256">> => encode_bytes(gateway_hotpatch_bundle:bundle_hash(Compressed)),
        <<"signature">> => encode_bytes(Signature),
        <<"bundle">> => encode_bytes(Compressed)
    }.

-spec append_event(binary(), binary(), file:filename()) -> {ok, binary()} | {error, term()}.
append_event(BuildSha, CreatedBy, EventPath) ->
    case file:read_file(EventPath) of
        {ok, EventJson} -> append_event_json(BuildSha, CreatedBy, EventJson);
        {error, Reason} -> {error, {read_event_failed, Reason}}
    end.

-spec append_event_json(binary(), binary(), binary()) -> {ok, binary()} | {error, term()}.
append_event_json(BuildSha, CreatedBy, EventJson) ->
    case decode_json(EventJson) of
        {ok, EventJsonTerm} -> append_event_term(BuildSha, CreatedBy, EventJsonTerm);
        {error, Reason} -> {error, Reason}
    end.

-spec append_event_term(binary(), binary(), term()) -> {ok, binary()} | {error, term()}.
append_event_term(BuildSha, CreatedBy, EventJsonTerm) ->
    case json_to_event(EventJsonTerm) of
        {ok, Event} -> append_event_row(BuildSha, CreatedBy, Event);
        {error, Reason} -> {error, Reason}
    end.

-spec build_bundle_modules([string()], fun(([map()]) -> {ok, map()})) ->
    {ok, map()} | {error, term()}.
build_bundle_modules(ModuleSpecs, Fun) ->
    case build_bundle_module_list(ModuleSpecs, []) of
        {ok, Modules} -> Fun(Modules);
        {error, Reason} -> {error, Reason}
    end.

-spec build_bundle_module_list([string()], [map()]) -> {ok, [map()]} | {error, term()}.
build_bundle_module_list([], Acc) ->
    {ok, lists:reverse(Acc)};
build_bundle_module_list([ModuleSpec | Rest], Acc) ->
    case bundle_module(ModuleSpec) of
        {ok, Module} -> build_bundle_module_list(Rest, [Module | Acc]);
        {error, Reason} -> {error, Reason}
    end.

-spec bundle_module(string()) -> {ok, map()} | {error, term()}.
bundle_module(ModuleSpec) ->
    case split_module_spec(ModuleSpec) of
        {ok, ModuleName, BeamPath0} -> bundle_named_module(ModuleName, BeamPath0);
        {error, Reason} -> {error, Reason}
    end.

-spec bundle_named_module(string(), file:filename() | undefined) ->
    {ok, map()} | {error, term()}.
bundle_named_module(ModuleName, BeamPath0) ->
    case existing_module(ModuleName) of
        {ok, Module} -> bundle_existing_module(Module, BeamPath0);
        {error, Reason} -> {error, Reason}
    end.

-spec bundle_existing_module(atom(), file:filename() | undefined) ->
    {ok, map()} | {error, term()}.
bundle_existing_module(Module, BeamPath0) ->
    case collect_bundle_parts(Module, BeamPath0) of
        {ok, CurrentMd5, TargetMd5, CompressedBeam} ->
            {ok, build_module_entry(Module, CurrentMd5, TargetMd5, CompressedBeam)};
        {error, Reason} ->
            {error, Reason}
    end.

-spec collect_bundle_parts(atom(), file:filename() | undefined) ->
    {ok, binary(), binary(), binary()} | {error, term()}.
collect_bundle_parts(Module, BeamPath0) ->
    case gateway_hotpatch_loader:current_md5(Module) of
        {ok, CurrentMd5} -> collect_bundle_beam(Module, BeamPath0, CurrentMd5);
        {error, Reason} -> {error, {current_md5_failed, Module, Reason}}
    end.

-spec collect_bundle_beam(atom(), file:filename() | undefined, binary()) ->
    {ok, binary(), binary(), binary()} | {error, term()}.
collect_bundle_beam(Module, BeamPath0, CurrentMd5) ->
    case resolve_beam_path(Module, BeamPath0) of
        {ok, BeamPath} -> read_bundle_beam(Module, BeamPath, CurrentMd5);
        {error, Reason} -> {error, Reason}
    end.

-spec read_bundle_beam(atom(), file:filename(), binary()) ->
    {ok, binary(), binary(), binary()} | {error, term()}.
read_bundle_beam(Module, BeamPath, CurrentMd5) ->
    case file:read_file(BeamPath) of
        {ok, Beam} -> hash_bundle_beam(Module, CurrentMd5, Beam);
        {error, Reason} -> {error, {read_beam_failed, Module, BeamPath, Reason}}
    end.

-spec hash_bundle_beam(atom(), binary(), binary()) ->
    {ok, binary(), binary(), binary()} | {error, term()}.
hash_bundle_beam(Module, CurrentMd5, Beam) ->
    case gateway_hotpatch_loader:beam_md5(Beam) of
        {ok, TargetMd5} -> compress_bundle_beam(Module, CurrentMd5, TargetMd5, Beam);
        {error, Reason} -> {error, {target_md5_failed, Module, Reason}}
    end.

-spec compress_bundle_beam(atom(), binary(), binary(), binary()) ->
    {ok, binary(), binary(), binary()} | {error, term()}.
compress_bundle_beam(Module, CurrentMd5, TargetMd5, Beam) ->
    case compress_beam(Beam) of
        {ok, CompressedBeam} -> {ok, CurrentMd5, TargetMd5, CompressedBeam};
        {error, Reason} -> {error, {beam_compress_failed, Module, Reason}}
    end.

-spec build_module_entry(atom(), binary(), binary(), binary()) -> map().
build_module_entry(Module, CurrentMd5, TargetMd5, CompressedBeam) ->
    #{
        <<"module">> => atom_to_binary(Module, utf8),
        <<"expected_current_md5">> => encode_bytes(CurrentMd5),
        <<"target_md5">> => encode_bytes(TargetMd5),
        <<"beam_zstd">> => encode_bytes(CompressedBeam)
    }.

-spec append_event_row(binary(), binary(), map()) -> {ok, binary()} | {error, term()}.
append_event_row(BuildSha, CreatedBy, Event) ->
    _ = fluxer_gateway_env:load(),
    case gateway_hotpatch_store:connect() of
        ok ->
            gateway_hotpatch_store:append_event(
                BuildSha,
                CreatedBy,
                maps:get(signer_key_id, Event),
                maps:get(signature, Event),
                maps:get(bundle_sha256, Event),
                maps:get(bundle, Event)
            );
        {error, Reason} ->
            {error, Reason}
    end.

-spec split_module_spec(string()) ->
    {ok, string(), file:filename() | undefined} | {error, term()}.
split_module_spec(ModuleSpec) ->
    case string:split(ModuleSpec, "=", leading) of
        [ModuleName, BeamPath] when ModuleName =/= "", BeamPath =/= "" ->
            {ok, ModuleName, BeamPath};
        [ModuleName] when ModuleName =/= "" ->
            {ok, ModuleName, undefined};
        _ ->
            {error, {invalid_module_spec, ModuleSpec}}
    end.

-spec existing_module(string()) -> {ok, atom()} | {error, term()}.
existing_module(ModuleName) ->
    try
        {ok, list_to_existing_atom(ModuleName)}
    catch
        error:badarg -> {error, {unknown_module, ModuleName}}
    end.

-spec resolve_beam_path(atom(), file:filename() | undefined) ->
    {ok, file:filename()} | {error, term()}.
resolve_beam_path(Module, undefined) ->
    case code:which(Module) of
        File when is_list(File) -> {ok, File};
        Other -> {error, {module_beam_not_found, Module, Other}}
    end;
resolve_beam_path(_Module, BeamPath) ->
    {ok, BeamPath}.

-spec compress_beam(binary()) -> {ok, binary()} | {error, term()}.
compress_beam(Beam) ->
    try erlang:apply(ezstd, compress, [Beam, 3]) of
        Compressed when is_binary(Compressed) -> {ok, Compressed};
        {error, Reason} -> {error, Reason};
        Other -> {error, Other}
    catch
        Class:Reason -> {error, {Class, Reason}}
    end.

-spec decode_json(binary()) -> {ok, term()} | {error, term()}.
decode_json(Json) ->
    try
        {ok, json:decode(Json)}
    catch
        Class:Reason -> {error, {decode_json_failed, Class, Reason}}
    end.

-spec read_private_key(file:filename()) -> {ok, binary()} | {error, term()}.
read_private_key(Path) ->
    case file:read_file(Path) of
        {ok, PrivateKey} when byte_size(PrivateKey) =:= 32 ->
            {ok, PrivateKey};
        {ok, PrivateKey} ->
            {error, {invalid_private_key_size, byte_size(PrivateKey)}};
        {error, Reason} ->
            {error, {read_private_key_failed, Reason}}
    end.

-spec json_to_bundle(term()) -> {ok, map()} | {error, term()}.
json_to_bundle(#{
    <<"version">> := Version, <<"build_sha">> := BuildSha, <<"modules">> := Modules
}) when is_list(Modules) ->
    case json_to_module_entries(Modules, []) of
        {ok, Entries} ->
            {ok, #{
                <<"version">> => Version,
                <<"build_sha">> => BuildSha,
                <<"modules">> => Entries
            }};
        {error, Reason} ->
            {error, Reason}
    end;
json_to_bundle(Other) ->
    {error, {invalid_bundle_json, Other}}.

-spec json_to_module_entries([term()], [map()]) -> {ok, [map()]} | {error, term()}.
json_to_module_entries([], Acc) ->
    {ok, lists:reverse(Acc)};
json_to_module_entries([Module | Rest], Acc) ->
    case json_to_module_entry(Module) of
        {ok, Entry} -> json_to_module_entries(Rest, [Entry | Acc]);
        {error, Reason} -> {error, Reason}
    end.

-spec json_to_module_entry(term()) -> {ok, map()} | {error, term()}.
json_to_module_entry(#{
    <<"module">> := Module,
    <<"expected_current_md5">> := ExpectedMd5,
    <<"target_md5">> := TargetMd5,
    <<"beam_zstd">> := BeamZstd
}) ->
    decode_module_entry(Module, ExpectedMd5, TargetMd5, BeamZstd);
json_to_module_entry(Other) ->
    {error, {invalid_module_entry_json, Other}}.

-spec decode_module_entry(term(), term(), term(), term()) ->
    {ok, map()} | {error, term()}.
decode_module_entry(Module, ExpectedMd5, TargetMd5, BeamZstd) when
    is_binary(Module), is_binary(ExpectedMd5), is_binary(TargetMd5), is_binary(BeamZstd)
->
    case decode_bytes(ExpectedMd5) of
        {ok, ExpectedMd5Bytes} ->
            decode_module_entry_target(Module, ExpectedMd5Bytes, TargetMd5, BeamZstd);
        {error, Reason} ->
            {error, Reason}
    end;
decode_module_entry(Module, ExpectedMd5, TargetMd5, BeamZstd) ->
    {error, {invalid_module_entry_json, {Module, ExpectedMd5, TargetMd5, BeamZstd}}}.

-spec decode_module_entry_target(binary(), binary(), binary(), binary()) ->
    {ok, map()} | {error, term()}.
decode_module_entry_target(Module, ExpectedMd5, TargetMd5, BeamZstd) ->
    case decode_bytes(TargetMd5) of
        {ok, TargetMd5Bytes} ->
            decode_module_entry_beam(Module, ExpectedMd5, TargetMd5Bytes, BeamZstd);
        {error, Reason} ->
            {error, Reason}
    end.

-spec decode_module_entry_beam(binary(), binary(), binary(), binary()) ->
    {ok, map()} | {error, term()}.
decode_module_entry_beam(Module, ExpectedMd5, TargetMd5, BeamZstd) ->
    case decode_bytes(BeamZstd) of
        {ok, BeamZstdBytes} ->
            {ok, #{
                <<"module">> => Module,
                <<"expected_current_md5">> => ExpectedMd5,
                <<"target_md5">> => TargetMd5,
                <<"beam_zstd">> => BeamZstdBytes
            }};
        {error, Reason} ->
            {error, Reason}
    end.

-spec json_to_event(term()) -> {ok, map()} | {error, term()}.
json_to_event(#{
    <<"signer_key_id">> := SignerKeyId,
    <<"bundle_sha256">> := BundleSha256,
    <<"signature">> := Signature,
    <<"bundle">> := Bundle
}) ->
    decode_event_bytes(SignerKeyId, BundleSha256, Signature, Bundle);
json_to_event(Other) ->
    {error, {invalid_event_json, Other}}.

-spec decode_event_bytes(term(), term(), term(), term()) ->
    {ok, map()} | {error, term()}.
decode_event_bytes(SignerKeyId, BundleSha256, Signature, Bundle) when
    is_binary(SignerKeyId), is_binary(BundleSha256), is_binary(Signature), is_binary(Bundle)
->
    case decode_bytes(BundleSha256) of
        {ok, BundleSha256Bytes} ->
            decode_event_signature(SignerKeyId, BundleSha256Bytes, Signature, Bundle);
        {error, Reason} ->
            {error, Reason}
    end;
decode_event_bytes(SignerKeyId, BundleSha256, Signature, Bundle) ->
    {error, {invalid_event_json, {SignerKeyId, BundleSha256, Signature, Bundle}}}.

-spec decode_event_signature(binary(), binary(), binary(), binary()) ->
    {ok, map()} | {error, term()}.
decode_event_signature(SignerKeyId, BundleSha256, Signature, Bundle) ->
    case decode_bytes(Signature) of
        {ok, SignatureBytes} ->
            decode_event_bundle(SignerKeyId, BundleSha256, SignatureBytes, Bundle);
        {error, Reason} ->
            {error, Reason}
    end.

-spec decode_event_bundle(binary(), binary(), binary(), binary()) ->
    {ok, map()} | {error, term()}.
decode_event_bundle(SignerKeyId, BundleSha256, Signature, Bundle) ->
    case decode_bytes(Bundle) of
        {ok, BundleBytes} ->
            {ok, #{
                signer_key_id => SignerKeyId,
                bundle_sha256 => BundleSha256,
                signature => Signature,
                bundle => BundleBytes
            }};
        {error, Reason} ->
            {error, Reason}
    end.

-spec encode_bytes(binary()) -> binary().
encode_bytes(Binary) ->
    base64:encode(Binary).

-spec decode_bytes(binary()) -> {ok, binary()} | {error, term()}.
decode_bytes(Encoded) when is_binary(Encoded) ->
    try
        {ok, base64:decode(Encoded)}
    catch
        Class:Reason -> {error, {invalid_base64_bytes, Encoded, Class, Reason}}
    end.

-spec write_json_result({ok, map()} | {error, term()}) -> non_neg_integer().
write_json_result({ok, Term}) ->
    write_stdout("~ts~n", [json:encode(Term)]),
    0;
write_json_result({error, Reason}) ->
    write_error(Reason).

-spec write_append_result({ok, binary()} | {error, term()}) -> non_neg_integer().
write_append_result({ok, EventId}) ->
    write_stdout("appended hotpatch event ~ts~n", [gateway_hotpatch_loader:hex(EventId)]),
    0;
write_append_result({error, Reason}) ->
    write_error(Reason).

-spec write_error(term()) -> non_neg_integer().
write_error(Reason) ->
    write_stderr("gateway hotpatch failed: ~0tp~n", [Reason]),
    1.

-spec write_usage() -> ok.
write_usage() ->
    write_stderr(
        "usage: gateway_hotpatch_cli bundle BUILD_SHA module_a=/path/to/module_a.beam ...~n"
        "       gateway_hotpatch_cli sign SIGNER_KEY_ID PRIVATE_KEY_RAW_FILE bundle.json~n"
        "       gateway_hotpatch_cli append BUILD_SHA CREATED_BY signed-event.json~n",
        []
    ).

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
