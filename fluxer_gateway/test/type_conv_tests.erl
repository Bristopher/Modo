%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(type_conv_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

to_integer_with_integer_test() ->
    ?assertEqual(42, type_conv:to_integer(42)),
    ?assertEqual(0, type_conv:to_integer(0)),
    ?assertEqual(-100, type_conv:to_integer(-100)).

to_integer_with_integer_edge_cases_test() ->
    ?assertEqual(1234567890123456789, type_conv:to_integer(1234567890123456789)),
    ?assertEqual(9223372036854775807, type_conv:to_integer(9223372036854775807)),
    ?assertEqual(-9223372036854775807, type_conv:to_integer(-9223372036854775807)).

to_integer_with_binary_valid_test() ->
    ?assertEqual(123, type_conv:to_integer(<<"123">>)),
    ?assertEqual(0, type_conv:to_integer(<<"0">>)),
    ?assertEqual(-456, type_conv:to_integer(<<"-456">>)).

to_integer_with_binary_edge_cases_test() ->
    ?assertEqual(1234567890123456789, type_conv:to_integer(<<"1234567890123456789">>)),
    ?assertEqual(9223372036854775807, type_conv:to_integer(<<"9223372036854775807">>)),
    ?assertEqual(-9223372036854775807, type_conv:to_integer(<<"-9223372036854775807">>)),
    ?assertEqual(1, type_conv:to_integer(<<"1">>)),
    ?assertEqual(123, type_conv:to_integer(<<"00123">>)),
    ?assertEqual(0, type_conv:to_integer(<<"0">>)).

to_integer_with_binary_invalid_test() ->
    ?assertEqual(undefined, type_conv:to_integer(<<"not_a_number">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"12.34">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"   ">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"abc123">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"123abc">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"12 34">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"--123">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"+-123">>)).

to_integer_with_binary_special_chars_test() ->
    ?assertEqual(undefined, type_conv:to_integer(<<"!@#$%">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"∞">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"①②③">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"一二三">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"null">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"NaN">>)),
    ?assertEqual(undefined, type_conv:to_integer(<<"Infinity">>)).

to_integer_with_list_valid_test() ->
    ?assertEqual(789, type_conv:to_integer("789")),
    ?assertEqual(-123, type_conv:to_integer("-123")),
    ?assertEqual(0, type_conv:to_integer("0")).

to_integer_with_list_edge_cases_test() ->
    ?assertEqual(1234567890123456789, type_conv:to_integer("1234567890123456789")),
    ?assertEqual(5, type_conv:to_integer("5")),
    ?assertEqual(42, type_conv:to_integer("00042")),
    ?assertEqual(-42, type_conv:to_integer("-00042")).

to_integer_with_list_invalid_test() ->
    ?assertEqual(undefined, type_conv:to_integer("invalid")),
    ?assertEqual(undefined, type_conv:to_integer("12.34")),
    ?assertEqual(undefined, type_conv:to_integer("")),
    ?assertEqual(undefined, type_conv:to_integer("   ")),
    ?assertEqual(undefined, type_conv:to_integer("abc")),
    ?assertEqual(undefined, type_conv:to_integer("123abc")),
    ?assertEqual(undefined, type_conv:to_integer("12 34")),
    ?assertEqual(undefined, type_conv:to_integer([1, 2, 3])).

to_integer_with_list_special_chars_test() ->
    ?assertEqual(undefined, type_conv:to_integer("!@#$%")),
    ?assertEqual(undefined, type_conv:to_integer("hello world")),
    ?assertEqual(undefined, type_conv:to_integer("--456")),
    ?assertEqual(undefined, type_conv:to_integer("null")).

to_integer_with_atom_valid_test() ->
    ?assertEqual(123, type_conv:to_integer('123')),
    ?assertEqual(-456, type_conv:to_integer('-456')),
    ?assertEqual(0, type_conv:to_integer('0')).

to_integer_with_atom_invalid_test() ->
    ?assertEqual(undefined, type_conv:to_integer(test)),
    ?assertEqual(undefined, type_conv:to_integer('not_a_number')),
    ?assertEqual(undefined, type_conv:to_integer(hello)),
    ?assertEqual(undefined, type_conv:to_integer(true)),
    ?assertEqual(undefined, type_conv:to_integer(false)),
    ?assertEqual(undefined, type_conv:to_integer(nil)),
    ?assertEqual(undefined, type_conv:to_integer('')).

to_integer_with_undefined_test() ->
    ?assertEqual(undefined, type_conv:to_integer(undefined)).

to_integer_with_invalid_types_test() ->
    ?assertEqual(undefined, type_conv:to_integer(12.34)),
    ?assertEqual(undefined, type_conv:to_integer(-45.67)),
    ?assertEqual(undefined, type_conv:to_integer(0.0)),
    ?assertEqual(undefined, type_conv:to_integer(#{key => value})),
    ?assertEqual(undefined, type_conv:to_integer(#{})),
    ?assertEqual(undefined, type_conv:to_integer({1, 2, 3})),
    ?assertEqual(undefined, type_conv:to_integer({})),
    Ref = make_ref(),
    ?assertEqual(undefined, type_conv:to_integer(Ref)),
    ?assertEqual(undefined, type_conv:to_integer(self())),
    ?assertEqual(undefined, type_conv:to_integer(erlang:list_to_port("#Port<0.0>"))).

to_binary_with_binary_test() ->
    ?assertEqual(<<"test">>, type_conv:to_binary(<<"test">>)),
    ?assertEqual(<<>>, type_conv:to_binary(<<>>)).

to_binary_with_binary_edge_cases_test() ->
    ?assertEqual(<<"hello world">>, type_conv:to_binary(<<"hello world">>)),
    ?assertEqual(<<"!@#$%^&*()">>, type_conv:to_binary(<<"!@#$%^&*()">>)),
    ?assertEqual(<<"line1\nline2">>, type_conv:to_binary(<<"line1\nline2">>)),
    ?assertEqual(<<"tab\there">>, type_conv:to_binary(<<"tab\there">>)),
    LongBinary = binary:copy(<<"x">>, 10000),
    ?assertEqual(LongBinary, type_conv:to_binary(LongBinary)).

to_binary_with_binary_unicode_test() ->
    ?assertEqual(<<"Hello 世界"/utf8>>, type_conv:to_binary(<<"Hello 世界"/utf8>>)),
    ?assertEqual(<<"Здравствуй мир"/utf8>>, type_conv:to_binary(<<"Здравствуй мир"/utf8>>)),
    ?assertEqual(<<"مرحبا بالعالم"/utf8>>, type_conv:to_binary(<<"مرحبا بالعالم"/utf8>>)),
    ?assertEqual(<<"🚀🌟💻"/utf8>>, type_conv:to_binary(<<"🚀🌟💻"/utf8>>)),
    ?assertEqual(<<"Ñoño"/utf8>>, type_conv:to_binary(<<"Ñoño"/utf8>>)),
    ?assertEqual(<<"Café"/utf8>>, type_conv:to_binary(<<"Café"/utf8>>)).

to_binary_with_integer_test() ->
    ?assertEqual(<<"42">>, type_conv:to_binary(42)),
    ?assertEqual(<<"0">>, type_conv:to_binary(0)),
    ?assertEqual(<<"-100">>, type_conv:to_binary(-100)).

to_binary_with_integer_edge_cases_test() ->
    ?assertEqual(<<"1234567890123456789">>, type_conv:to_binary(1234567890123456789)),
    ?assertEqual(<<"9223372036854775807">>, type_conv:to_binary(9223372036854775807)),
    ?assertEqual(<<"-9223372036854775807">>, type_conv:to_binary(-9223372036854775807)),
    ?assertEqual(<<"1">>, type_conv:to_binary(1)),
    ?assertEqual(<<"-1">>, type_conv:to_binary(-1)).

to_binary_with_list_valid_test() ->
    ?assertEqual(<<"hello">>, type_conv:to_binary("hello")),
    ?assertEqual(<<>>, type_conv:to_binary("")).

to_binary_with_list_edge_cases_test() ->
    ?assertEqual(<<"hello world">>, type_conv:to_binary("hello world")),
    ?assertEqual(<<"!@#$%">>, type_conv:to_binary("!@#$%")),
    ?assertEqual(<<"line1\nline2">>, type_conv:to_binary("line1\nline2")),
    LongString = lists:duplicate(10000, $x),
    LongBinary = binary:copy(<<"x">>, 10000),
    ?assertEqual(LongBinary, type_conv:to_binary(LongString)).

to_binary_with_list_unicode_test() ->
    ?assertEqual(undefined, type_conv:to_binary(unicode_world_chars())),
    ?assertEqual(undefined, type_conv:to_binary(emoji_chars())),

    ?assertEqual(<<67, 97, 102, 233>>, type_conv:to_binary(cafe_latin1_chars())),

    ?assertEqual(<<"Hello">>, type_conv:to_binary([72, 101, 108, 108, 111])),

    ?assertEqual(<<0, 1, 127, 255>>, type_conv:to_binary([0, 1, 127, 255])).

to_binary_with_list_invalid_test() ->
    ?assertEqual(<<1, 2, 3>>, type_conv:to_binary([1, 2, 3])),
    ?assertEqual(undefined, type_conv:to_binary([256])),
    ?assertEqual(undefined, type_conv:to_binary([1000])),
    ?assertEqual(undefined, type_conv:to_binary([-1])),
    ?assertEqual(undefined, type_conv:to_binary([hello, world])),
    ?assertEqual(undefined, type_conv:to_binary([1, 2, atom])).

to_binary_with_atom_test() ->
    ?assertEqual(<<"test">>, type_conv:to_binary(test)),
    ?assertEqual(<<"hello_world">>, type_conv:to_binary(hello_world)),
    ?assertEqual(<<"true">>, type_conv:to_binary(true)),
    ?assertEqual(<<"false">>, type_conv:to_binary(false)),
    ?assertEqual(<<"">>, type_conv:to_binary('')).

to_binary_with_atom_edge_cases_test() ->
    ?assertEqual(<<"Hello World">>, type_conv:to_binary('Hello World')),
    ?assertEqual(<<"123">>, type_conv:to_binary('123')),
    ?assertEqual(<<"hello-world">>, type_conv:to_binary('hello-world')),
    ?assertEqual(<<"test@example">>, type_conv:to_binary('test@example')),
    ?assertEqual(undefined, type_conv:to_binary(undefined)).

to_binary_with_invalid_types_test() ->
    ?assertEqual(undefined, type_conv:to_binary(12.34)),
    ?assertEqual(undefined, type_conv:to_binary(-45.67)),
    ?assertEqual(undefined, type_conv:to_binary(0.0)),
    ?assertEqual(undefined, type_conv:to_binary(#{key => value})),
    ?assertEqual(undefined, type_conv:to_binary(#{})),
    ?assertEqual(undefined, type_conv:to_binary({1, 2, 3})),
    ?assertEqual(undefined, type_conv:to_binary({})),
    Ref = make_ref(),
    ?assertEqual(undefined, type_conv:to_binary(Ref)),
    ?assertEqual(undefined, type_conv:to_binary(self())),
    ?assertEqual(undefined, type_conv:to_binary(erlang:list_to_port("#Port<0.0>"))).

to_list_with_list_test() ->
    ?assertEqual("test", type_conv:to_list("test")),
    ?assertEqual([], type_conv:to_list([])),
    ?assertEqual([1, 2, 3], type_conv:to_list([1, 2, 3])).

to_list_with_list_edge_cases_test() ->
    ?assertEqual("hello world", type_conv:to_list("hello world")),
    ?assertEqual("!@#$%^&*()", type_conv:to_list("!@#$%^&*()")),
    ?assertEqual([true, false, nil], type_conv:to_list([true, false, nil])),
    ?assertEqual(nested_number_lists(), type_conv:to_list(nested_number_lists())),
    LongList = lists:duplicate(10000, $x),
    ?assertEqual(LongList, type_conv:to_list(LongList)).

to_list_with_list_unicode_test() ->
    ?assertEqual(unicode_world_chars(), type_conv:to_list(unicode_world_chars())),
    ?assertEqual(cafe_latin1_chars(), type_conv:to_list(cafe_latin1_chars())),
    ?assertEqual(emoji_chars(), type_conv:to_list(emoji_chars())).

to_list_with_binary_test() ->
    ?assertEqual("hello", type_conv:to_list(<<"hello">>)),
    ?assertEqual("", type_conv:to_list(<<>>)).

to_list_with_binary_edge_cases_test() ->
    ?assertEqual("hello world", type_conv:to_list(<<"hello world">>)),
    ?assertEqual("!@#$%", type_conv:to_list(<<"!@#$%">>)),
    ?assertEqual("line1\nline2", type_conv:to_list(<<"line1\nline2">>)),
    ?assertEqual("tab\there", type_conv:to_list(<<"tab\there">>)),
    LongBinary = binary:copy(<<"x">>, 10000),
    LongList = lists:duplicate(10000, $x),
    ?assertEqual(LongList, type_conv:to_list(LongBinary)).

to_list_with_binary_unicode_test() ->
    ?assertEqual(
        [72, 101, 108, 108, 111, 32, 228, 184, 150, 231, 149, 140],
        type_conv:to_list(<<"Hello 世界"/utf8>>)
    ),
    ?assertEqual([67, 97, 102, 195, 169], type_conv:to_list(<<"Café"/utf8>>)),
    ?assertEqual(
        <<"Hello 世界"/utf8>>, list_to_binary(assert_list(type_conv:to_list(<<"Hello 世界"/utf8>>)))
    ).

to_list_with_atom_test() ->
    ?assertEqual("test", type_conv:to_list(test)),
    ?assertEqual("hello_world", type_conv:to_list(hello_world)),
    ?assertEqual("true", type_conv:to_list(true)),
    ?assertEqual("false", type_conv:to_list(false)),
    ?assertEqual("", type_conv:to_list('')).

to_list_with_atom_edge_cases_test() ->
    ?assertEqual(
        [72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100], type_conv:to_list('Hello World')
    ),
    ?assertEqual([49, 50, 51], type_conv:to_list('123')),
    ?assertEqual(
        [104, 101, 108, 108, 111, 45, 119, 111, 114, 108, 100], type_conv:to_list('hello-world')
    ),
    ?assertEqual(undefined, type_conv:to_list(undefined)).

to_list_rejects_non_list_like_values_test() ->
    lists:foreach(
        fun(Value) -> ?assertEqual(undefined, type_conv:to_list(Value)) end,
        [42, 12.34, #{}, {1, 2, 3}, make_ref(), self()]
    ).

unicode_world_chars() ->
    [72, 101, 108, 108, 111, 32, 19990, 30028].

cafe_latin1_chars() ->
    [67, 97, 102, 233].

emoji_chars() ->
    [128640, 127775, 128187].

nested_number_lists() ->
    [[1, 2], [3, 4]].

extract_id_uses_snowflake_semantics_test() ->
    ?assertEqual(123, type_conv:extract_id(#{user_id => 123}, user_id)),
    ?assertEqual(undefined, type_conv:extract_id(#{user_id => <<"0">>}, user_id)),
    ?assertEqual(456, type_conv:extract_id(#{<<"user_id">> => <<"456">>}, <<"user_id">>)),
    ?assertEqual(789, type_conv:extract_id(#{<<"user_id">> => "789"}, <<"user_id">>)),
    ?assertEqual(undefined, type_conv:extract_id(#{user_id => -1}, user_id)),
    ?assertEqual(undefined, type_conv:extract_id(#{user_id => <<"-1">>}, user_id)),
    ?assertEqual(undefined, type_conv:extract_id(#{user_id => <<"001">>}, user_id)).

extract_id_required_returns_snowflake_or_undefined_test() ->
    ?assertEqual(123, type_conv:extract_id_required(#{user_id => <<"123">>}, user_id)),
    ?assertEqual(
        undefined, type_conv:extract_id_required(#{<<"user_id">> => 0}, <<"user_id">>)
    ),
    ?assertEqual(undefined, type_conv:extract_id_required(#{user_id => "-1"}, user_id)),
    ?assertEqual(undefined, type_conv:extract_id_required(#{user_id => "001"}, user_id)).
to_list_with_invalid_types_test() ->
    ?assertEqual(undefined, type_conv:to_list(42)),
    ?assertEqual(undefined, type_conv:to_list(-123)),
    ?assertEqual(undefined, type_conv:to_list(0)),
    ?assertEqual(undefined, type_conv:to_list(12.34)),
    ?assertEqual(undefined, type_conv:to_list(-45.67)),
    ?assertEqual(undefined, type_conv:to_list(0.0)),
    ?assertEqual(undefined, type_conv:to_list(#{key => value})),
    ?assertEqual(undefined, type_conv:to_list(#{})),
    ?assertEqual(undefined, type_conv:to_list({1, 2, 3})),
    ?assertEqual(undefined, type_conv:to_list({})),
    Ref = make_ref(),
    ?assertEqual(undefined, type_conv:to_list(Ref)),
    ?assertEqual(undefined, type_conv:to_list(self())),
    ?assertEqual(undefined, type_conv:to_list(erlang:list_to_port("#Port<0.0>"))).

assert_list(Value) when is_list(Value) ->
    Value.
