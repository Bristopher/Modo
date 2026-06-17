%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_sender_retry).
-typing([eqwalizer]).

-export([
    maybe_retry_with_smaller_record_size/4,
    initial_record_size_for_endpoint/1
]).

-export_type([push_response/0]).

-define(STANDARD_PUSH_RECORD_SIZE, 4096).
-define(MOZILLA_COMPAT_PUSH_RECORD_SIZE, 2820).
-define(MOZILLA_CONSTRAINED_PUSH_RECORD_SIZE, 2048).
-define(MIN_PUSH_RECORD_SIZE, 1024).
-define(MAX_PAYLOAD_RETRY_ATTEMPTS, 2).

-type push_response() :: {ok, integer(), term(), binary()} | {error, term()}.

-spec initial_record_size_for_endpoint(binary()) -> pos_integer().
initial_record_size_for_endpoint(Endpoint) ->
    case is_mozilla_push_endpoint(Endpoint) of
        true -> ?MOZILLA_COMPAT_PUSH_RECORD_SIZE;
        false -> ?STANDARD_PUSH_RECORD_SIZE
    end.

-spec maybe_retry_with_smaller_record_size(
    binary(), push_response(), pos_integer(), non_neg_integer()
) ->
    no_retry | {retry, pos_integer()}.
maybe_retry_with_smaller_record_size(_Endpoint, _Response, _CurrentRecordSize, Attempt) when
    Attempt >= ?MAX_PAYLOAD_RETRY_ATTEMPTS
->
    no_retry;
maybe_retry_with_smaller_record_size(
    Endpoint, {ok, 413, _ResponseHeaders, ResponseBody}, CurrentRecordSize, _Attempt
) ->
    case next_record_size_for_payload_too_large(CurrentRecordSize, Endpoint, ResponseBody) of
        undefined -> no_retry;
        NextRecordSize -> {retry, NextRecordSize}
    end;
maybe_retry_with_smaller_record_size(_Endpoint, _Response, _CurrentRecordSize, _Attempt) ->
    no_retry.

-spec next_record_size_for_payload_too_large(pos_integer(), binary(), binary()) ->
    pos_integer() | undefined.
next_record_size_for_payload_too_large(CurrentRecordSize, Endpoint, ResponseBody) ->
    case parse_constrained_overage_bytes(ResponseBody) of
        OverageBytes when is_integer(OverageBytes), OverageBytes > 0 ->
            sanitize_next_record_size(CurrentRecordSize - OverageBytes, CurrentRecordSize);
        _ ->
            FallbackRecordSize = fallback_record_size_for_endpoint(CurrentRecordSize, Endpoint),
            sanitize_next_record_size(FallbackRecordSize, CurrentRecordSize)
    end.

-spec sanitize_next_record_size(integer() | undefined, pos_integer()) ->
    pos_integer() | undefined.
sanitize_next_record_size(undefined, _CurrentRecordSize) ->
    undefined;
sanitize_next_record_size(CandidateRecordSize, CurrentRecordSize) when
    is_integer(CandidateRecordSize)
->
    ClampedRecordSize = erlang:max(?MIN_PUSH_RECORD_SIZE, CandidateRecordSize),
    case ClampedRecordSize < CurrentRecordSize of
        true -> ClampedRecordSize;
        false -> undefined
    end.

-spec fallback_record_size_for_endpoint(pos_integer(), binary()) -> pos_integer() | undefined.
fallback_record_size_for_endpoint(CurrentRecordSize, Endpoint) ->
    case is_mozilla_push_endpoint(Endpoint) of
        true when CurrentRecordSize > ?MOZILLA_COMPAT_PUSH_RECORD_SIZE ->
            ?MOZILLA_COMPAT_PUSH_RECORD_SIZE;
        true when CurrentRecordSize > ?MOZILLA_CONSTRAINED_PUSH_RECORD_SIZE ->
            ?MOZILLA_CONSTRAINED_PUSH_RECORD_SIZE;
        _ ->
            undefined
    end.

-spec parse_constrained_overage_bytes(binary()) -> non_neg_integer() | undefined.
parse_constrained_overage_bytes(ResponseBody) ->
    case decode_push_error_body(ResponseBody) of
        #{<<"message">> := Message} -> parse_constrained_overage_from_message(Message);
        _ -> undefined
    end.

-spec parse_constrained_overage_from_message(binary() | list()) ->
    non_neg_integer() | undefined.
parse_constrained_overage_from_message(Message) when is_list(Message) ->
    parse_constrained_overage_from_message(list_to_binary(Message));
parse_constrained_overage_from_message(Message) when is_binary(Message) ->
    case
        re:run(Message, <<"too long by ([0-9]+) bytes">>, [caseless, {capture, [1], binary}])
    of
        {match, [OverageBytesBin]} -> parse_non_neg_integer(OverageBytesBin);
        _ -> undefined
    end.

-spec decode_push_error_body(binary()) -> map() | undefined.
decode_push_error_body(ResponseBody) when
    is_binary(ResponseBody), byte_size(ResponseBody) > 0
->
    try json:decode(ResponseBody) of
        ParsedBody when is_map(ParsedBody) -> ParsedBody;
        _ -> undefined
    catch
        error:_ -> undefined;
        throw:_ -> undefined;
        exit:_ -> undefined
    end;
decode_push_error_body(_ResponseBody) ->
    undefined.

-spec parse_non_neg_integer(binary()) -> non_neg_integer() | undefined.
parse_non_neg_integer(Value) ->
    case guild_data_normalize_schema:int(Value) of
        ParsedValue when ParsedValue >= 0 -> ParsedValue;
        _ -> undefined
    end.

-spec is_mozilla_push_endpoint(binary()) -> boolean().
is_mozilla_push_endpoint(Endpoint) ->
    LowerEndpoint = lowercase_binary(Endpoint),
    case binary:match(LowerEndpoint, <<"push.services.mozilla.com">>) of
        nomatch -> false;
        _ -> true
    end.

-spec lowercase_binary(binary()) -> binary().
lowercase_binary(Value) ->
    iolist_to_binary(string:lowercase(Value)).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

is_mozilla_push_endpoint_test() ->
    Endpoint = <<"https://updates.push.services.mozilla.com/wpush/v2/token">>,
    ?assertEqual(true, is_mozilla_push_endpoint(Endpoint)),
    ?assertEqual(
        true,
        is_mozilla_push_endpoint(<<"https://push.services.mozilla.com/wpush/x">>)
    ),
    ?assertEqual(
        false,
        is_mozilla_push_endpoint(<<"https://fcm.googleapis.com/fcm/send">>)
    ).

initial_record_size_for_endpoint_test() ->
    MozUrl = <<"https://updates.push.services.mozilla.com/wpush/v2/x">>,
    ?assertEqual(?MOZILLA_COMPAT_PUSH_RECORD_SIZE, initial_record_size_for_endpoint(MozUrl)),
    FcmUrl = <<"https://fcm.googleapis.com/fcm/send">>,
    ?assertEqual(?STANDARD_PUSH_RECORD_SIZE, initial_record_size_for_endpoint(FcmUrl)).

next_record_size_for_payload_too_large_overage_test() ->
    ResponseBody = <<
        "{\"code\":413,\"errno\":104,\"error\":\"Payload Too Large\","
        "\"message\":\"This message is intended for a constrained device and is limited in size. "
        "Converted buffer is too long by 1441 bytes\"}"
    >>,
    ?assertEqual(
        2655,
        next_record_size_for_payload_too_large(
            ?STANDARD_PUSH_RECORD_SIZE,
            <<"https://updates.push.services.mozilla.com/wpush/v2/x">>,
            ResponseBody
        )
    ).

next_record_size_for_payload_too_large_fallback_test() ->
    ResponseBody = <<"{\"code\":413,\"errno\":104,\"error\":\"Payload Too Large\"}">>,
    MozillaEndpoint = <<"https://updates.push.services.mozilla.com/wpush/v2/x">>,
    ?assertEqual(
        ?MOZILLA_COMPAT_PUSH_RECORD_SIZE,
        next_record_size_for_payload_too_large(
            ?STANDARD_PUSH_RECORD_SIZE, MozillaEndpoint, ResponseBody
        )
    ),
    ?assertEqual(
        ?MOZILLA_CONSTRAINED_PUSH_RECORD_SIZE,
        next_record_size_for_payload_too_large(
            ?MOZILLA_COMPAT_PUSH_RECORD_SIZE, MozillaEndpoint, ResponseBody
        )
    ),
    ?assertEqual(
        undefined,
        next_record_size_for_payload_too_large(
            ?MOZILLA_CONSTRAINED_PUSH_RECORD_SIZE, MozillaEndpoint, ResponseBody
        )
    ),
    ?assertEqual(
        undefined,
        next_record_size_for_payload_too_large(
            ?STANDARD_PUSH_RECORD_SIZE, <<"https://fcm.googleapis.com/fcm/send">>, ResponseBody
        )
    ).

-endif.
