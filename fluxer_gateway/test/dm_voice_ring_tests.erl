%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(dm_voice_ring_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

convert_api_channel_missing_type_does_not_fabricate_zero_test() ->
    Channel = #{<<"id">> => <<"100">>, <<"recipients">> => []},
    GatewayChannel = dm_voice_ring:convert_api_channel_to_gateway_format(Channel, 10),
    ?assertEqual(undefined, maps:get(<<"type">>, GatewayChannel)).

convert_api_channel_normalizes_binary_type_test() ->
    Channel = #{<<"id">> => <<"100">>, <<"type">> => <<"3">>, <<"recipients">> => []},
    GatewayChannel = dm_voice_ring:convert_api_channel_to_gateway_format(Channel, 10),
    ?assertEqual(3, maps:get(<<"type">>, GatewayChannel)).
