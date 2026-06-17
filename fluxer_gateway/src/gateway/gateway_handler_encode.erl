%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_handler_encode).
-typing([eqwalizer]).

-export([
    encode_and_compress/2,
    close_with_reason/3,
    make_frame/3,
    compression_error_reason/1,
    dispatch_event_name/1,
    ensure_compress_ctx/1
]).

-type state() :: gateway_handler:state().
-type ws_frame() :: gateway_handler:ws_frame().
-type ws_result() :: gateway_handler:ws_result().

-export_type([state/0, ws_frame/0, ws_result/0]).

-spec encode_and_compress(map(), state()) -> {ok, ws_frame(), state()} | {error, term()}.
encode_and_compress(Message, #{encoding := Encoding, compress_ctx := CompressCtx} = State) ->
    SanitizedMessage = gateway_timings_payload:sanitize_message(Message),
    case gateway_codec:encode(SanitizedMessage, Encoding) of
        {ok, Encoded, FrameType} ->
            EncodedBin = iolist_to_binary(Encoded),
            Ctx = ensure_compress_ctx(CompressCtx),
            compress_encoded(EncodedBin, FrameType, Ctx, State);
        {error, Reason} ->
            {error, {encode_failed, Reason}}
    end.

-spec ensure_compress_ctx(
    gateway_compress:compress_ctx() | undefined
) -> gateway_compress:compress_ctx().
ensure_compress_ctx(undefined) -> gateway_compress:new_context(none);
ensure_compress_ctx(Ctx) -> Ctx.

-spec compress_encoded(binary(), text | binary, gateway_compress:compress_ctx(), state()) ->
    {ok, ws_frame(), state()} | {error, term()}.
compress_encoded(Encoded, FrameType, CompressCtx, State) ->
    case gateway_compress:compress(Encoded, CompressCtx) of
        {ok, Compressed, NewCompressCtx} ->
            Frame = make_frame(Compressed, FrameType, NewCompressCtx),
            {ok, Frame, State#{compress_ctx => NewCompressCtx}};
        {error, Reason} ->
            {error, {compress_failed, gateway_compress:get_type(CompressCtx), Reason}}
    end.

-spec compression_error_reason(atom()) -> binary().
compression_error_reason(zstd_frame) -> <<"Compression failed: zstd-stream">>;
compression_error_reason(zstd_stream) -> <<"Compression failed: zstd-stream">>;
compression_error_reason(_) -> <<"Encode failed">>.

-spec close_with_reason(atom(), binary(), state()) -> ws_result().
close_with_reason(Reason, Message, State) ->
    CloseCode = constants:close_code_to_num(Reason),
    {[{close, CloseCode, Message}], State}.

-spec make_frame(binary(), text | binary, gateway_compress:compress_ctx()) -> ws_frame().
make_frame(Data, FrameType, CompressCtx) ->
    case gateway_compress:get_type(CompressCtx) of
        none -> {FrameType, Data};
        _ -> {binary, Data}
    end.

-spec dispatch_event_name(atom() | binary()) -> binary().
dispatch_event_name(Event) when is_binary(Event) -> Event;
dispatch_event_name(Event) when is_atom(Event) ->
    case constants:dispatch_event_atom(Event) of
        Result when is_binary(Result) -> Result;
        _ -> <<"UNKNOWN">>
    end;
dispatch_event_name(_) ->
    <<"UNKNOWN">>.
