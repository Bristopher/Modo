%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_errors_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

error_test() ->
    ?assertEqual(
        {error, not_found, voice_connection_not_found},
        gateway_errors:error(voice_connection_not_found)
    ),
    ?assertEqual(
        {error, validation_error, voice_channel_not_voice},
        gateway_errors:error(voice_channel_not_voice)
    ),
    ?assertEqual(
        {error, permission_denied, voice_permission_denied},
        gateway_errors:error(voice_permission_denied)
    ).

error_code_test() ->
    ?assertEqual(
        <<"VOICE_CONNECTION_NOT_FOUND">>,
        gateway_errors:error_code(voice_connection_not_found)
    ),
    ?assertEqual(
        <<"VOICE_CHANNEL_NOT_FOUND">>, gateway_errors:error_code(voice_channel_not_found)
    ),
    ?assertEqual(
        <<"VOICE_PERMISSION_DENIED">>,
        gateway_errors:error_code(voice_permission_denied)
    ),
    ?assertEqual(
        <<"VOICE_PERMISSION_DENIED">>,
        gateway_errors:error_code(voice_moderator_missing_connect)
    ),
    ?assertEqual(<<"UNKNOWN_ERROR">>, gateway_errors:error_code(some_random_error)),
    ?assertEqual(<<"TIMEOUT">>, gateway_errors:error_code(timeout)),
    ?assertEqual(<<"INTERNAL_ERROR">>, gateway_errors:error_code(internal_error)).

error_message_test() ->
    ?assertEqual(
        <<"Voice connection not found">>,
        gateway_errors:error_message(voice_connection_not_found)
    ),
    ?assertEqual(
        <<"Voice channel not found">>,
        gateway_errors:error_message(voice_channel_not_found)
    ),
    ?assertEqual(
        <<"Missing voice permissions">>,
        gateway_errors:error_message(voice_permission_denied)
    ),
    ?assertEqual(
        <<"Voice channel is full">>,
        gateway_errors:error_message(voice_channel_full)
    ),
    ?assertEqual(
        <<"An unknown error occurred">>,
        gateway_errors:error_message(some_random_error)
    ),
    ?assertEqual(<<"Request timed out">>, gateway_errors:error_message(timeout)).

error_category_test() ->
    ?assertEqual(not_found, gateway_errors:error_category(voice_connection_not_found)),
    ?assertEqual(not_found, gateway_errors:error_category(voice_channel_not_found)),
    ?assertEqual(not_found, gateway_errors:error_category(dm_channel_not_found)),
    ?assertEqual(validation_error, gateway_errors:error_category(voice_channel_not_voice)),
    ?assertEqual(validation_error, gateway_errors:error_category(validation_invalid_snowflake)),
    ?assertEqual(permission_denied, gateway_errors:error_category(voice_permission_denied)),
    ?assertEqual(permission_denied, gateway_errors:error_category(voice_channel_full)),
    ?assertEqual(voice_error, gateway_errors:error_category(voice_token_failed)),
    ?assertEqual(rate_limited, gateway_errors:error_category(voice_update_rate_limited)),
    ?assertEqual(timeout, gateway_errors:error_category(timeout)),
    ?assertEqual(unknown, gateway_errors:error_category(unknown_error)),
    ?assertEqual(unknown, gateway_errors:error_category(some_random_error)).

is_recoverable_test() ->
    ?assert(gateway_errors:is_recoverable(not_found)),
    ?assert(gateway_errors:is_recoverable(permission_denied)),
    ?assert(gateway_errors:is_recoverable(voice_error)),
    ?assert(gateway_errors:is_recoverable(validation_error)),
    ?assert(gateway_errors:is_recoverable(timeout)),
    ?assert(gateway_errors:is_recoverable(unknown)),
    ?assertNot(gateway_errors:is_recoverable(rate_limited)),
    ?assertNot(gateway_errors:is_recoverable(auth_failed)).

all_voice_errors_have_codes_test() ->
    VoiceErrors = [
        voice_connection_not_found,
        voice_channel_not_found,
        voice_channel_not_voice,
        voice_member_not_found,
        voice_user_not_in_voice,
        voice_guild_not_found,
        voice_permission_denied,
        voice_member_timed_out,
        voice_channel_full,
        voice_connection_limit_reached,
        voice_missing_connection_id,
        voice_invalid_user_id,
        voice_invalid_channel_id,
        voice_invalid_state,
        voice_user_mismatch,
        voice_token_failed,
        voice_guild_id_missing,
        voice_invalid_guild_id,
        voice_moderator_missing_connect,
        voice_unclaimed_account,
        voice_update_rate_limited,
        voice_nonce_mismatch,
        voice_pending_expired,
        voice_camera_user_limit,
        voice_e2ee_required
    ],
    assert_known_error_codes(VoiceErrors).

all_dm_errors_have_codes_test() ->
    DmErrors = [dm_channel_not_found, dm_not_recipient, dm_invalid_channel_type],
    assert_known_error_codes(DmErrors).

all_validation_errors_have_codes_test() ->
    ValidationErrors = [
        validation_invalid_snowflake,
        validation_null_snowflake,
        validation_invalid_snowflake_list,
        validation_expected_list,
        validation_expected_map,
        validation_missing_field,
        validation_invalid_params
    ],
    assert_known_error_codes(ValidationErrors).

assert_known_error_codes(Errors) ->
    lists:foreach(fun assert_known_error_code/1, Errors).

assert_known_error_code(Error) ->
    Code = gateway_errors:error_code(Error),
    ?assert(is_binary(Code)),
    ?assertNotEqual(<<"UNKNOWN_ERROR">>, Code).
