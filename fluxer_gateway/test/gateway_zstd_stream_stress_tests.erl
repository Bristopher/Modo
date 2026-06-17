%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_zstd_stream_stress_tests).

-include_lib("eunit/include/eunit.hrl").

-define(STRESS_MESSAGE_COUNT, 10000).

gateway_zstd_stream_roundtrips_10000_json_messages_test_() ->
    {timeout, 60, fun gateway_zstd_stream_roundtrips_10000_json_messages/0}.

gateway_zstd_stream_roundtrips_10000_json_messages() ->
    ?assertEqual(true, zstd_stream_available()),
    run_zstd_stream_stress().

run_zstd_stream_stress() ->
    Compression = gateway_compress:parse_compression(<<"zstd-stream">>, <<"1">>),
    ?assertEqual(zstd_stream, Compression),
    EncodeState = #{
        encoding => json,
        compress_ctx => gateway_compress:new_context(Compression)
    },
    DecodeCtx = gateway_compress:new_context(zstd_stream),
    {FinalEncodeState, FinalDecodeCtx, TotalCompressedBytes} = lists:foldl(
        fun roundtrip_gateway_message/2,
        {EncodeState, DecodeCtx, 0},
        lists:seq(1, ?STRESS_MESSAGE_COUNT)
    ),
    ?assert(TotalCompressedBytes > 0),
    ok = gateway_compress:close_context(maps:get(compress_ctx, FinalEncodeState)),
    ok = gateway_compress:close_context(FinalDecodeCtx).

roundtrip_gateway_message(Seq, {EncodeState, DecodeCtx, TotalCompressedBytes}) ->
    Message = gateway_message(Seq),
    {ok, {binary, Compressed}, NextEncodeState} =
        gateway_handler_encode:encode_and_compress(Message, EncodeState),
    ?assert(byte_size(Compressed) > 0),
    {ok, Json, NextDecodeCtx} = gateway_compress:decompress(Compressed, DecodeCtx),
    {ok, Decoded} = gateway_codec:decode(Json, json),
    ?assertEqual(Message, Decoded),
    {NextEncodeState, NextDecodeCtx, TotalCompressedBytes + byte_size(Compressed)}.

gateway_message(Seq) ->
    SeqBin = integer_to_binary(Seq),
    #{
        <<"op">> => 0,
        <<"t">> => <<"MESSAGE_CREATE">>,
        <<"s">> => Seq,
        <<"d">> => #{
            <<"id">> => SeqBin,
            <<"channel_id">> => <<"1497639278555484216">>,
            <<"guild_id">> => <<"1427764661718740994">>,
            <<"author">> => #{
                <<"id">> => <<"1042">>,
                <<"username">> => <<"canary">>,
                <<"bot">> => false
            },
            <<"content">> => iolist_to_binary([
                <<"gateway zstd stream stress payload ">>,
                SeqBin
            ]),
            <<"mentions">> => [],
            <<"attachments">> => [],
            <<"embeds">> => [],
            <<"flags">> => Seq rem 8
        }
    }.

zstd_stream_available() ->
    case code:ensure_loaded(ezstd) of
        {module, ezstd} ->
            erlang:function_exported(ezstd, create_compression_context, 1) andalso
                erlang:function_exported(ezstd, set_compression_parameter, 3) andalso
                erlang:function_exported(ezstd, compress_streaming, 2) andalso
                erlang:function_exported(ezstd, create_decompression_context, 1) andalso
                erlang:function_exported(ezstd, decompress_streaming, 2);
        _ ->
            false
    end.
