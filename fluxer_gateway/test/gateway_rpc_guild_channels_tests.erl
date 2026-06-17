%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_rpc_guild_channels_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

get_viewable_channels_rejects_zero_guild_id_test() ->
    Params = #{<<"guild_id">> => <<"0">>, <<"user_id">> => <<"1">>},
    ?assertError(
        {gateway_rpc_error, <<"guild_id_invalid">>},
        gateway_rpc_guild_channels:handle(<<"guild.get_viewable_channels">>, Params)
    ).

get_viewable_channels_rejects_leading_zero_user_id_test() ->
    Params = #{<<"guild_id">> => <<"1">>, <<"user_id">> => <<"001">>},
    ?assertError(
        {gateway_rpc_error, <<"user_id_invalid">>},
        gateway_rpc_guild_channels:handle(<<"guild.get_viewable_channels">>, Params)
    ).

resolve_channel_mentions_skips_invalid_channel_ids_test() ->
    Params = #{<<"guild_id">> => <<"1">>, <<"channel_ids">> => [<<"0">>, <<"001">>, <<"-1">>]},
    ?assertEqual(
        #{<<"channels">> => []},
        gateway_rpc_guild_channels:handle(<<"guild.resolve_channel_mentions">>, Params)
    ).
