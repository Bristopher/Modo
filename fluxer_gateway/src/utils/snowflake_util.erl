%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(snowflake_util).
-typing([eqwalizer]).

-export([extract_timestamp/1]).

-define(FLUXER_EPOCH, 1420070400000).
-define(TIMESTAMP_SHIFT, 22).

-spec extract_timestamp(term()) -> integer() | undefined.
extract_timestamp(SnowflakeValue) ->
    try snowflake_id:parse_optional(SnowflakeValue) of
        Snowflake when is_integer(Snowflake), Snowflake > 0 ->
            (Snowflake bsr ?TIMESTAMP_SHIFT) + ?FLUXER_EPOCH;
        undefined ->
            undefined
    catch
        error:{invalid_snowflake, _} -> undefined
    end.

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

extract_timestamp_from_integer_test() ->
    Timestamp = 1704067200000,
    RelativeTs = Timestamp - ?FLUXER_EPOCH,
    Snowflake = RelativeTs bsl ?TIMESTAMP_SHIFT,
    ?assertEqual(Timestamp, extract_timestamp(Snowflake)).

extract_timestamp_from_binary_test() ->
    Timestamp = 1704067200000,
    RelativeTs = Timestamp - ?FLUXER_EPOCH,
    Snowflake = RelativeTs bsl ?TIMESTAMP_SHIFT,
    SnowflakeBin = integer_to_binary(Snowflake),
    ?assertEqual(Timestamp, extract_timestamp(SnowflakeBin)).

extract_timestamp_with_worker_and_sequence_test() ->
    Timestamp = 1704067200000,
    RelativeTs = Timestamp - ?FLUXER_EPOCH,
    WorkerId = 5,
    Sequence = 100,
    Snowflake = (RelativeTs bsl ?TIMESTAMP_SHIFT) bor (WorkerId bsl 12) bor Sequence,
    ?assertEqual(Timestamp, extract_timestamp(Snowflake)).

extract_timestamp_rejects_malformed_snowflake_test() ->
    ?assertEqual(undefined, extract_timestamp(0)),
    ?assertEqual(undefined, extract_timestamp(<<"0">>)),
    ?assertEqual(undefined, extract_timestamp(<<"001">>)),
    ?assertEqual(undefined, extract_timestamp(<<"-1">>)),
    ?assertEqual(undefined, extract_timestamp(<<"not_a_snowflake">>)),
    ?assertEqual(undefined, extract_timestamp(-1)).

-endif.
