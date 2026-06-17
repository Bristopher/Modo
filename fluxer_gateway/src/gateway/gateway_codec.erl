%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_codec).
-typing([eqwalizer]).

-export([
    encode/2,
    decode/2,
    parse_encoding/1
]).

-type encoding() :: json.
-type frame_type() :: text | binary.

-export_type([encoding/0, frame_type/0]).

-spec parse_encoding(binary() | undefined) -> encoding().
parse_encoding(_) ->
    json.

-spec encode(map(), encoding()) -> {ok, iodata(), frame_type()} | {error, term()}.
encode(Message, json) ->
    try
        Encoded = iolist_to_binary(json:encode(Message)),
        {ok, Encoded, text}
    catch
        _:Reason ->
            {error, {encode_failed, Reason}}
    end.

-spec decode(binary(), encoding()) -> {ok, map()} | {error, term()}.
decode(Data, json) ->
    try
        Decoded = json:decode(Data),
        case Decoded of
            M when is_map(M) -> {ok, M};
            _ -> {error, {decode_failed, not_a_map}}
        end
    catch
        _:Reason ->
            {error, {decode_failed, Reason}}
    end.

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

parse_encoding_test_() ->
    [
        ?_assertEqual(json, parse_encoding(<<"json">>)),
        ?_assertEqual(json, parse_encoding(<<"etf">>)),
        ?_assertEqual(json, parse_encoding(undefined)),
        ?_assertEqual(json, parse_encoding(<<"invalid">>)),
        ?_assertEqual(json, parse_encoding(<<>>))
    ].

encode_json_test_() ->
    Message = #{<<"op">> => 0, <<"d">> => #{<<"test">> => true}},
    [
        ?_assertMatch({ok, _, text}, encode(Message, json)),
        ?_test(assert_encode_json_is_binary(Message))
    ].

assert_encode_json_is_binary(Message) ->
    {ok, Encoded, text} = encode(Message, json),
    ?assert(is_binary(Encoded)).

encode_empty_map_test() ->
    {ok, Encoded, text} = encode(#{}, json),
    ?assertEqual(<<"{}">>, Encoded).

encode_nested_test() ->
    Message = #{<<"a">> => #{<<"b">> => #{<<"c">> => 1}}},
    {ok, Encoded, text} = encode(Message, json),
    ?assert(is_binary(Encoded)),
    {ok, Decoded} = decode(iolist_to_binary(Encoded), json),
    ?assertEqual(Message, Decoded).

decode_json_test_() ->
    Data = <<"{\"op\":0,\"d\":{\"test\":true}}">>,
    [
        ?_assertMatch({ok, _}, decode(Data, json)),
        ?_test(assert_decode_json_op(Data))
    ].

assert_decode_json_op(Data) ->
    {ok, Decoded} = decode(Data, json),
    ?assertEqual(0, maps:get(<<"op">>, Decoded)).

decode_invalid_json_test() ->
    ?assertMatch({error, {decode_failed, _}}, decode(<<"not json">>, json)).

decode_empty_object_test() ->
    {ok, Decoded} = decode(<<"{}">>, json),
    ?assertEqual(#{}, Decoded).

roundtrip_json_test_() ->
    Messages = [
        #{<<"op">> => 10, <<"d">> => #{<<"heartbeat_interval">> => 41250}},
        #{<<"op">> => 0, <<"s">> => 1, <<"t">> => <<"READY">>, <<"d">> => #{}},
        #{<<"list">> => [1, 2, 3], <<"bool">> => true, <<"null">> => null}
    ],
    [
        ?_test(assert_roundtrip_json(Msg))
     || Msg <- Messages
    ].

assert_roundtrip_json(Msg) ->
    {ok, Encoded, _} = encode(Msg, json),
    {ok, Decoded} = decode(iolist_to_binary(Encoded), json),
    ?assertEqual(Msg, Decoded).

-endif.
