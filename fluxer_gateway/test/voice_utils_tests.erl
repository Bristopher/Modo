%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(voice_utils_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

build_voice_token_rpc_request_guild_test() ->
    Req = voice_utils:build_voice_token_rpc_request(123, 456, 789, null, null, null),
    ?assertEqual(<<"voice_get_token">>, maps:get(<<"type">>, Req)),
    ?assertEqual(<<"123">>, maps:get(<<"guild_id">>, Req)),
    ?assertEqual(<<"456">>, maps:get(<<"channel_id">>, Req)),
    ?assertEqual(<<"789">>, maps:get(<<"user_id">>, Req)),
    ?assertNot(maps:is_key(<<"connection_id">>, Req)).

build_voice_token_rpc_request_dm_test() ->
    Req = voice_utils:build_voice_token_rpc_request(null, 456, 789, null, null, null),
    ?assertEqual(<<"voice_get_token">>, maps:get(<<"type">>, Req)),
    ?assertNot(maps:is_key(<<"guild_id">>, Req)),
    ?assertEqual(<<"456">>, maps:get(<<"channel_id">>, Req)).

build_voice_token_rpc_request_with_connection_test() ->
    Req = voice_utils:build_voice_token_rpc_request(123, 456, 789, <<"conn-id">>, null, null),
    ?assertEqual(<<"conn-id">>, maps:get(<<"connection_id">>, Req)).

build_voice_token_rpc_request_dm_with_connection_test() ->
    Req = voice_utils:build_voice_token_rpc_request(null, 456, 789, <<"conn-id">>, null, null),
    ?assertEqual(<<"conn-id">>, maps:get(<<"connection_id">>, Req)).

add_geolocation_to_request_test() ->
    BaseReq = #{<<"type">> => <<"test">>},
    WithGeo = voice_utils:add_geolocation_to_request(BaseReq, <<"1.0">>, <<"2.0">>),
    ?assertEqual(<<"1.0">>, maps:get(<<"latitude">>, WithGeo)),
    ?assertEqual(<<"2.0">>, maps:get(<<"longitude">>, WithGeo)),
    WithoutGeo = voice_utils:add_geolocation_to_request(BaseReq, null, null),
    ?assertNot(maps:is_key(<<"latitude">>, WithoutGeo)).

add_geolocation_to_request_number_test() ->
    BaseReq = #{<<"type">> => <<"test">>},
    WithGeo = voice_utils:add_geolocation_to_request(BaseReq, 1.5, 2.25),
    ?assertEqual(<<"1.5">>, maps:get(<<"latitude">>, WithGeo)),
    ?assertEqual(<<"2.25">>, maps:get(<<"longitude">>, WithGeo)).

add_rtc_region_to_request_test() ->
    BaseReq = #{<<"type">> => <<"test">>},
    WithRegion = voice_utils:add_rtc_region_to_request(BaseReq, <<"us-east">>),
    ?assertEqual(<<"us-east">>, maps:get(<<"rtc_region">>, WithRegion)),
    WithoutRegion = voice_utils:add_rtc_region_to_request(BaseReq, null),
    ?assertNot(maps:is_key(<<"rtc_region">>, WithoutRegion)).

build_force_disconnect_rpc_request_test() ->
    Req = voice_utils:build_force_disconnect_rpc_request(123, 456, 789, <<"conn">>),
    ?assertEqual(<<"voice_force_disconnect_participant">>, maps:get(<<"type">>, Req)),
    ?assertEqual(<<"123">>, maps:get(<<"guild_id">>, Req)),
    ?assertEqual(<<"conn">>, maps:get(<<"connection_id">>, Req)).

build_list_participants_rpc_request_test() ->
    Req = voice_utils:build_list_participants_rpc_request(
        123, 456, <<"local">>, <<"server-1">>
    ),
    ?assertEqual(<<"voice_list_participants">>, maps:get(<<"type">>, Req)),
    ?assertEqual(<<"123">>, maps:get(<<"guild_id">>, Req)),
    ?assertEqual(<<"456">>, maps:get(<<"channel_id">>, Req)),
    ?assertEqual(<<"local">>, maps:get(<<"region_id">>, Req)),
    ?assertEqual(<<"server-1">>, maps:get(<<"server_id">>, Req)).

build_list_participants_rpc_request_dm_test() ->
    Req = voice_utils:build_list_participants_rpc_request(
        null, 456, <<"local">>, <<"server-1">>
    ),
    ?assertEqual(<<"voice_list_participants">>, maps:get(<<"type">>, Req)),
    ?assertNot(maps:is_key(<<"guild_id">>, Req)).

build_update_participant_rpc_request_test() ->
    Req = voice_utils:build_update_participant_rpc_request(123, 456, 789, true, false),
    ?assertEqual(<<"voice_update_participant">>, maps:get(<<"type">>, Req)),
    ?assertEqual(true, maps:get(<<"mute">>, Req)),
    ?assertEqual(false, maps:get(<<"deaf">>, Req)),
    ?assertEqual(true, maps:get(<<"can_speak">>, Req)),
    ?assertEqual(true, maps:get(<<"can_stream">>, Req)),
    ?assertEqual(true, maps:get(<<"can_video">>, Req)).

build_update_participant_rpc_request_with_permissions_test() ->
    VoicePerms = #{
        can_speak => true,
        can_stream => false,
        can_video => false
    },
    Req = voice_utils:build_update_participant_rpc_request(
        123, 456, 789, true, false, VoicePerms
    ),
    ?assertEqual(true, maps:get(<<"can_speak">>, Req)),
    ?assertEqual(false, maps:get(<<"can_stream">>, Req)),
    ?assertEqual(false, maps:get(<<"can_video">>, Req)).

apply_voice_permissions_to_flags_suppresses_without_speak_test() ->
    Flags = #{self_mute => false, suppress => false},
    VoicePerms = #{
        can_speak => false,
        can_stream => true,
        can_video => true
    },
    Result = voice_utils:apply_voice_permissions_to_flags(Flags, VoicePerms),
    ?assertEqual(false, maps:get(self_mute, Result)),
    ?assertEqual(true, maps:get(suppress, Result)).

generate_token_nonce_format_test() ->
    Nonce = voice_utils:generate_token_nonce(),
    ?assert(is_binary(Nonce)),
    ?assertEqual(32, byte_size(Nonce)),
    ?assert(
        lists:all(
            fun(C) ->
                (C >= $0 andalso C =< $9) orelse (C >= $a andalso C =< $f)
            end,
            binary_to_list(Nonce)
        )
    ).

generate_token_nonce_unique_test() ->
    Nonce1 = voice_utils:generate_token_nonce(),
    Nonce2 = voice_utils:generate_token_nonce(),
    Nonce3 = voice_utils:generate_token_nonce(),
    ?assertNot(Nonce1 =:= Nonce2),
    ?assertNot(Nonce2 =:= Nonce3),
    ?assertNot(Nonce1 =:= Nonce3).

build_voice_token_rpc_request_with_nonce_test() ->
    VoicePerms = #{
        can_speak => true,
        can_stream => false,
        can_video => false
    },
    Req = voice_utils:build_voice_token_rpc_request(
        123, 456, 789, null, null, null, VoicePerms, <<"test-nonce-123">>
    ),
    ?assertEqual(<<"test-nonce-123">>, maps:get(<<"token_nonce">>, Req)),
    ?assertEqual(true, maps:get(<<"can_speak">>, Req)),
    ?assertEqual(false, maps:get(<<"can_stream">>, Req)).

build_voice_token_rpc_request_without_nonce_test() ->
    VoicePerms = #{
        can_speak => true,
        can_stream => true,
        can_video => true
    },
    Req = voice_utils:build_voice_token_rpc_request(
        123, 456, 789, null, null, null, VoicePerms, null
    ),
    ?assertNot(maps:is_key(<<"token_nonce">>, Req)),
    ?assertEqual(true, maps:get(<<"can_speak">>, Req)).

build_voice_token_rpc_request_undefined_nonce_test() ->
    VoicePerms = #{
        can_speak => false,
        can_stream => true,
        can_video => true
    },
    Req = voice_utils:build_voice_token_rpc_request(
        123, 456, 789, null, null, null, VoicePerms, undefined_token_nonce()
    ),
    ?assertNot(maps:is_key(<<"token_nonce">>, Req)),
    ?assertEqual(false, maps:get(<<"can_speak">>, Req)).

undefined_token_nonce() ->
    eqwalizer:dynamic_cast(undefined).
