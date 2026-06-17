%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_voice_mutation).
-typing([eqwalizer]).

-export([evaluate/3, build_ack/6]).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").
-endif.

-spec evaluate(
    BaseVersion :: integer() | undefined,
    CurrentVersion :: integer(),
    ValidationResult :: valid | invalid
) -> apply | {reject, binary()}.
evaluate(_BaseVersion, _CurrentVersion, invalid) ->
    {reject, <<"invalid_payload">>};
evaluate(BaseVersion, CurrentVersion, valid) when
    is_integer(BaseVersion), BaseVersion < CurrentVersion - 1
->
    {reject, <<"stale_base_version">>};
evaluate(_BaseVersion, _CurrentVersion, valid) ->
    apply.

-spec build_ack(
    MutationId :: binary() | undefined,
    RuntimeEpoch :: binary() | undefined,
    ConnectionId :: binary() | null,
    GuildId :: integer() | null | undefined,
    ChannelId :: integer() | null | undefined,
    Outcome :: #{
        status := binary(),
        server_version := integer(),
        canonical_state := map(),
        error_code => binary(),
        error_message => binary()
    }
) -> map() | undefined.
build_ack(undefined, _RuntimeEpoch, _ConnectionId, _GuildId, _ChannelId, _Outcome) ->
    undefined;
build_ack(MutationId, RuntimeEpoch, ConnectionId, GuildId, ChannelId, Outcome) when
    is_binary(MutationId)
->
    BaseAck = #{
        <<"mutation_id">> => MutationId,
        <<"runtime_epoch">> => RuntimeEpoch,
        <<"connection_id">> => ConnectionId,
        <<"guild_id">> => maybe_int_to_binary(GuildId),
        <<"channel_id">> => maybe_int_to_binary(ChannelId),
        <<"status">> => maps:get(status, Outcome),
        <<"server_version">> => maps:get(server_version, Outcome),
        <<"canonical_state">> => maps:get(canonical_state, Outcome)
    },
    put_optional_ack_fields(BaseAck, Outcome).

-spec maybe_int_to_binary(integer() | null | undefined) -> binary() | null.
maybe_int_to_binary(null) -> null;
maybe_int_to_binary(undefined) -> null;
maybe_int_to_binary(N) when is_integer(N) -> integer_to_binary(N).

-spec maybe_put_optional_ack_field(map(), binary(), binary() | undefined) -> map().
maybe_put_optional_ack_field(Ack, _Field, undefined) ->
    Ack;
maybe_put_optional_ack_field(Ack, Field, Value) ->
    Ack#{Field => Value}.

-spec put_optional_ack_fields(map(), map()) -> map().
put_optional_ack_fields(BaseAck, Outcome) ->
    lists:foldl(
        fun({Field, Key}, Ack) ->
            maybe_put_optional_ack_field(Ack, Field, maps:get(Key, Outcome, undefined))
        end,
        BaseAck,
        [
            {<<"error_code">>, error_code},
            {<<"error_message">>, error_message}
        ]
    ).

-ifdef(TEST).

evaluate_invalid_payload_test() ->
    ?assertEqual({reject, <<"invalid_payload">>}, evaluate(0, 0, invalid)),
    ?assertEqual({reject, <<"invalid_payload">>}, evaluate(undefined, 5, invalid)).

evaluate_apply_no_version_test() ->
    ?assertEqual(apply, evaluate(undefined, 0, valid)),
    ?assertEqual(apply, evaluate(undefined, 100, valid)).

evaluate_apply_current_version_test() ->
    ?assertEqual(apply, evaluate(5, 5, valid)).

evaluate_apply_one_behind_test() ->
    ?assertEqual(apply, evaluate(4, 5, valid)).

evaluate_stale_base_test() ->
    ?assertEqual({reject, <<"stale_base_version">>}, evaluate(3, 5, valid)),
    ?assertEqual({reject, <<"stale_base_version">>}, evaluate(0, 10, valid)).

build_ack_no_mutation_id_test() ->
    Outcome = #{status => <<"ignored">>, server_version => 0, canonical_state => #{}},
    ?assertEqual(undefined, build_ack(undefined, undefined, <<"conn1">>, 123, 456, Outcome)).

test_build_ack(Outcome) ->
    build_ack(<<"mut1">>, <<"epoch1">>, <<"conn1">>, 123, 456, Outcome).

build_ack_full_test() ->
    Outcome = #{status => <<"applied">>, server_version => 7, canonical_state => #{}},
    #{
        <<"mutation_id">> := <<"mut1">>,
        <<"status">> := <<"applied">>,
        <<"server_version">> := 7
    } = test_build_ack(Outcome).

build_ack_with_error_fields_test() ->
    Outcome = #{
        status => <<"rejected">>,
        server_version => 9,
        canonical_state => #{},
        error_code => <<"stale_base_version">>,
        error_message => <<"stale_base_version">>
    },
    #{
        <<"error_code">> := <<"stale_base_version">>,
        <<"error_message">> := <<"stale_base_version">>
    } = test_build_ack(Outcome).

-endif.
