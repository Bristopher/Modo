%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_hotpatch_bundle).
-typing([eqwalizer]).

-export([
    compress_term/1,
    decompress_term/1,
    bundle_hash/1,
    signing_payload/1,
    sign/2,
    verify_signature/4,
    parse_public_keys/1,
    decode_signed_event/2
]).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").
-endif.

-define(DOMAIN, <<"fluxer-gateway-hotpatch-v1">>).
-define(ZSTD_LEVEL, 3).

-spec compress_term(term()) -> {ok, binary()} | {error, term()}.
compress_term(Term) ->
    compress_binary(term_to_binary(Term, [deterministic])).

-spec decompress_term(binary()) -> {ok, term()} | {error, term()}.
decompress_term(Compressed) when is_binary(Compressed) ->
    case decompress_binary(Compressed) of
        {ok, Binary} -> decode_term(Binary);
        {error, Reason} -> {error, Reason}
    end.

-spec decode_term(binary()) -> {ok, term()} | {error, term()}.
decode_term(Binary) ->
    try
        {ok, binary_to_term(Binary, [safe])}
    catch
        Class:Reason -> {error, {invalid_term, Class, Reason}}
    end.

-spec bundle_hash(binary()) -> binary().
bundle_hash(CompressedBundle) when is_binary(CompressedBundle) ->
    crypto:hash(sha256, CompressedBundle).

-spec signing_payload(binary()) -> binary().
signing_payload(CompressedBundle) when is_binary(CompressedBundle) ->
    <<?DOMAIN/binary, 0, CompressedBundle/binary>>.

-spec sign(binary(), binary()) -> {ok, binary()} | {error, term()}.
sign(CompressedBundle, PrivateKey) when is_binary(CompressedBundle), is_binary(PrivateKey) ->
    try
        {ok, crypto:sign(eddsa, none, signing_payload(CompressedBundle), [PrivateKey, ed25519])}
    catch
        Class:Reason -> {error, {sign_failed, Class, Reason}}
    end.

-spec verify_signature(binary(), binary(), binary(), #{binary() => binary()}) ->
    ok | {error, term()}.
verify_signature(CompressedBundle, SignerKeyId, Signature, PublicKeys) when
    is_binary(CompressedBundle),
    is_binary(SignerKeyId),
    is_binary(Signature),
    is_map(PublicKeys)
->
    case maps:get(SignerKeyId, PublicKeys, undefined) of
        undefined ->
            {error, {unknown_signer, SignerKeyId}};
        PublicKey ->
            verify_with_key(CompressedBundle, Signature, PublicKey)
    end.

-spec parse_public_keys(term()) ->
    {ok, #{binary() => binary()}} | {error, term()}.
parse_public_keys(undefined) ->
    {ok, #{}};
parse_public_keys(Value) when is_list(Value) ->
    parse_public_keys(type_conv:ensure_binary(Value));
parse_public_keys(Value) when is_binary(Value) ->
    Tokens = [
        string:trim(Token)
     || Token <- binary:split(Value, [<<",">>, <<"\n">>, <<";">>], [global]),
        string:trim(Token) =/= <<>>
    ],
    parse_public_key_tokens(Tokens, #{});
parse_public_keys(Value) ->
    {error, {invalid_public_keys_config, Value}}.

-spec decode_signed_event(map(), #{binary() => binary()}) -> {ok, map()} | {error, term()}.
decode_signed_event(Event, PublicKeys) when is_map(Event), is_map(PublicKeys) ->
    Bundle = event_value(bundle, Event),
    SignerKeyId = event_value(signer_key_id, Event),
    Signature = event_value(signature, Event),
    BundleHash = event_value(bundle_sha256, Event),
    decode_event_payload(Bundle, SignerKeyId, Signature, BundleHash, PublicKeys).

-spec decode_event_payload(term(), term(), term(), term(), #{binary() => binary()}) ->
    {ok, map()} | {error, term()}.
decode_event_payload(Bundle, SignerKeyId, Signature, BundleHash, PublicKeys) when
    is_binary(Bundle), is_binary(SignerKeyId), is_binary(Signature), is_binary(BundleHash)
->
    case validate_event_hash(Bundle, BundleHash) of
        ok -> decode_verified_event(Bundle, SignerKeyId, Signature, PublicKeys);
        {error, Reason} -> {error, Reason}
    end;
decode_event_payload(_Bundle, _SignerKeyId, _Signature, _BundleHash, _PublicKeys) ->
    {error, invalid_event_payload}.

-spec decode_verified_event(binary(), binary(), binary(), #{binary() => binary()}) ->
    {ok, map()} | {error, term()}.
decode_verified_event(Bundle, SignerKeyId, Signature, PublicKeys) ->
    case verify_signature(Bundle, SignerKeyId, Signature, PublicKeys) of
        ok -> decompress_bundle_map(Bundle);
        {error, Reason} -> {error, Reason}
    end.

-spec decompress_bundle_map(binary()) -> {ok, map()} | {error, term()}.
decompress_bundle_map(Bundle) ->
    case decompress_term(Bundle) of
        {ok, Map} when is_map(Map) -> {ok, Map};
        {ok, Other} -> {error, {invalid_bundle_term, Other}};
        {error, Reason} -> {error, Reason}
    end.

-spec compress_binary(binary()) -> {ok, binary()} | {error, term()}.
compress_binary(Binary) ->
    try erlang:apply(ezstd, compress, [Binary, ?ZSTD_LEVEL]) of
        Compressed when is_binary(Compressed) -> {ok, Compressed};
        {error, Reason} -> {error, {compress_failed, Reason}};
        Other -> {error, {compress_failed, Other}}
    catch
        Class:Reason -> {error, {compress_failed, Class, Reason}}
    end.

-spec decompress_binary(binary()) -> {ok, binary()} | {error, term()}.
decompress_binary(Binary) ->
    try erlang:apply(ezstd, decompress, [Binary]) of
        Decompressed when is_binary(Decompressed) -> {ok, Decompressed};
        Decompressed when is_list(Decompressed) -> {ok, iolist_to_binary(Decompressed)};
        {error, Reason} -> {error, {decompress_failed, Reason}};
        Other -> {error, {decompress_failed, Other}}
    catch
        Class:Reason -> {error, {decompress_failed, Class, Reason}}
    end.

-spec verify_with_key(binary(), binary(), binary()) -> ok | {error, term()}.
verify_with_key(CompressedBundle, Signature, PublicKey) when byte_size(PublicKey) =:= 32 ->
    verify_with_valid_key(CompressedBundle, Signature, PublicKey);
verify_with_key(_CompressedBundle, _Signature, PublicKey) ->
    {error, {invalid_public_key_size, byte_size(PublicKey)}}.

-spec verify_with_valid_key(binary(), binary(), binary()) -> ok | {error, term()}.
verify_with_valid_key(CompressedBundle, Signature, PublicKey) ->
    try
        crypto:verify(eddsa, none, signing_payload(CompressedBundle), Signature, [
            PublicKey, ed25519
        ])
    of
        true -> ok;
        false -> {error, invalid_signature}
    catch
        Class:Reason -> {error, {verify_failed, Class, Reason}}
    end.

-spec parse_public_key_tokens([binary()], #{binary() => binary()}) ->
    {ok, #{binary() => binary()}} | {error, term()}.
parse_public_key_tokens([], Acc) ->
    {ok, Acc};
parse_public_key_tokens([Token | Rest], Acc) ->
    case parse_public_key_token(Token) of
        {ok, KeyId, PublicKey} -> parse_public_key_tokens(Rest, Acc#{KeyId => PublicKey});
        {error, Reason} -> {error, Reason}
    end.

-spec parse_public_key_token(binary()) -> {ok, binary(), binary()} | {error, term()}.
parse_public_key_token(Token) ->
    case split_key_token(Token) of
        {ok, KeyId, Encoded} -> parse_public_key_material(KeyId, Encoded);
        error -> {error, {invalid_public_key_token, Token}}
    end.

-spec parse_public_key_material(binary(), binary()) ->
    {ok, binary(), binary()} | {error, term()}.
parse_public_key_material(KeyId, Encoded) ->
    case decode_key_material(Encoded) of
        {ok, PublicKey} when byte_size(PublicKey) =:= 32 -> {ok, KeyId, PublicKey};
        {ok, PublicKey} -> {error, {invalid_public_key_size, KeyId, byte_size(PublicKey)}};
        {error, Reason} -> {error, {invalid_public_key, KeyId, Reason}}
    end.

-spec split_key_token(binary()) -> {ok, binary(), binary()} | error.
split_key_token(Token) ->
    case binary:split(Token, <<":">>) of
        [KeyId, Encoded] -> {ok, string:trim(KeyId), string:trim(Encoded)};
        _ -> split_key_token_equals(Token)
    end.

-spec split_key_token_equals(binary()) -> {ok, binary(), binary()} | error.
split_key_token_equals(Token) ->
    case binary:split(Token, <<"=">>) of
        [KeyId, Encoded] -> {ok, string:trim(KeyId), string:trim(Encoded)};
        _ -> error
    end.

-spec decode_key_material(binary()) -> {ok, binary()} | {error, term()}.
decode_key_material(Encoded) ->
    case try_base64(Encoded) of
        {ok, Decoded} -> {ok, Decoded};
        {error, _} -> try_base64url(Encoded)
    end.

-spec try_base64(binary()) -> {ok, binary()} | {error, term()}.
try_base64(Encoded) ->
    try
        {ok, base64:decode(Encoded)}
    catch
        Class:Reason -> {error, {Class, Reason}}
    end.

-spec try_base64url(binary()) -> {ok, binary()} | {error, term()}.
try_base64url(Encoded) ->
    try
        {ok, base64url:decode(Encoded)}
    catch
        Class:Reason -> {error, {Class, Reason}}
    end.

-spec validate_event_hash(binary(), binary()) -> ok | {error, term()}.
validate_event_hash(Bundle, BundleHash) ->
    case bundle_hash(Bundle) of
        BundleHash -> ok;
        Other -> {error, {bundle_hash_mismatch, Other, BundleHash}}
    end.

-spec event_value(atom(), map()) -> term().
event_value(Key, Event) ->
    maps:get(Key, Event, maps:get(atom_to_binary(Key, utf8), Event, undefined)).

-ifdef(TEST).

compress_decompress_roundtrip_test() ->
    Term = #{
        version => 1,
        build_sha => <<"abc123">>,
        modules => [#{module => <<"session_lifecycle">>, expected_current_md5 => <<0:128>>}]
    },
    {ok, Compressed} = compress_term(Term),
    ?assert(is_binary(Compressed)),
    ?assertEqual({ok, Term}, decompress_term(Compressed)).

sign_and_verify_roundtrip_test() ->
    {PublicKey, PrivateKey} = ed25519_keypair(),
    {ok, Compressed} = compress_term(#{version => 1}),
    {ok, Signature} = sign(Compressed, PrivateKey),
    Keys = #{<<"ops">> => PublicKey},
    ?assertEqual(ok, verify_signature(Compressed, <<"ops">>, Signature, Keys)),
    ?assertEqual(
        {error, invalid_signature},
        verify_signature(<<Compressed/binary, 0>>, <<"ops">>, Signature, Keys)
    ).

decode_signed_event_rejects_hash_mismatch_test() ->
    {PublicKey, PrivateKey} = ed25519_keypair(),
    {ok, Compressed} = compress_term(#{version => 1}),
    {ok, Signature} = sign(Compressed, PrivateKey),
    Event = #{
        signer_key_id => <<"ops">>,
        signature => Signature,
        bundle_sha256 => <<0:256>>,
        bundle => Compressed
    },
    ?assertMatch(
        {error, {bundle_hash_mismatch, _, _}},
        decode_signed_event(Event, #{<<"ops">> => PublicKey})
    ).

parse_public_keys_test() ->
    PublicKey = <<1:256>>,
    Encoded = base64:encode(PublicKey),
    ?assertEqual(
        {ok, #{<<"ops">> => PublicKey}}, parse_public_keys(<<"ops:", Encoded/binary>>)
    ).

ed25519_keypair() ->
    {PublicKey, PrivateKey} = crypto:generate_key(eddsa, ed25519),
    {require_binary(PublicKey), require_binary(PrivateKey)}.

require_binary(Value) when is_binary(Value) ->
    Value.

-endif.
