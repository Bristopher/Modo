%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(gateway_rpc_voice_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

voice_channel_read_malformed_payloads_return_empty_results_test() ->
    ?assertEqual(
        #{<<"voice_states">> => []},
        gateway_rpc_voice:execute_method(<<"voice.get_voice_states_for_channel">>, #{})
    ),
    ?assertEqual(
        #{<<"voice_states">> => []},
        gateway_rpc_voice:execute_method(<<"voice.get_voice_states_for_channel">>, #{
            <<"guild_id">> => <<"1">>,
            <<"channel_id">> => null
        })
    ),
    ?assertEqual(
        #{<<"pending_joins">> => []},
        gateway_rpc_voice:execute_method(<<"voice.get_pending_joins_for_channel">>, #{
            <<"guild_id">> => <<"bad">>,
            <<"channel_id">> => <<"2">>
        })
    ).

validate_voice_channel_read_params_normalizes_ids_test() ->
    ?assertEqual(
        {ok, <<"2">>, 1},
        gateway_rpc_voice:validate_voice_channel_read_params(#{
            <<"guild_id">> => <<"1">>, <<"channel_id">> => 2
        })
    ),
    ?assertEqual(
        {ok, <<"2">>, undefined},
        gateway_rpc_voice:validate_voice_channel_read_params(#{
            <<"guild_id">> => null, <<"channel_id">> => <<"2">>
        })
    ),
    ?assertEqual(
        {ok, <<"2">>, undefined},
        gateway_rpc_voice:validate_voice_channel_read_params(#{
            <<"guild_id">> => <<"0">>, <<"channel_id">> => <<"2">>
        })
    ).

normalize_pending_joins_ignores_malformed_entries_test() ->
    ?assertEqual(
        [
            #{
                <<"connection_id">> => <<"conn">>,
                <<"user_id">> => <<"42">>,
                <<"token_nonce">> => <<>>,
                <<"expires_at">> => 0
            }
        ],
        gateway_rpc_voice:normalize_pending_joins([
            not_a_map,
            #{<<"connection_id">> => <<"conn">>, <<"user_id">> => 42},
            #{<<"connection_id">> => undefined, <<"user_id">> => 84},
            #{<<"connection_id">> => <<"missing-user">>}
        ])
    ).

normalize_voice_states_ignores_malformed_entries_test() ->
    ?assertEqual(
        [
            #{
                <<"connection_id">> => <<"conn">>,
                <<"user_id">> => <<"42">>,
                <<"channel_id">> => <<"9">>,
                <<"region_id">> => <<"local">>,
                <<"server_id">> => <<"dev-1">>
            }
        ],
        gateway_rpc_voice:normalize_voice_states([
            not_a_map,
            #{
                <<"connection_id">> => <<"conn">>,
                <<"user_id">> => 42,
                <<"channel_id">> => 9,
                region_id => <<"local">>,
                server_id => <<"dev-1">>
            },
            #{<<"connection_id">> => undefined, <<"user_id">> => 84, <<"channel_id">> => 9}
        ])
    ).
