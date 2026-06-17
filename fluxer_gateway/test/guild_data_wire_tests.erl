%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_data_wire_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

payload_strips_non_schema_channel_recipient_ids_test() ->
    Payload = payload(#{
        <<"id">> => 100,
        <<"recipient_ids">> => [10, 20],
        <<"recipients">> => [
            #{<<"id">> => 10, <<"username">> => <<"ada">>},
            #{<<"id">> => 20, <<"username">> => <<"grace">>}
        ]
    }),
    ?assertNot(maps:is_key(<<"recipient_ids">>, Payload)),
    ?assertEqual(
        [
            #{<<"id">> => <<"10">>, <<"username">> => <<"ada">>},
            #{<<"id">> => <<"20">>, <<"username">> => <<"grace">>}
        ],
        maps:get(<<"recipients">>, Payload)
    ).

payload_strips_gateway_internal_indexes_and_caches_test() ->
    Payload = payload(#{
        <<"id">> => 100,
        <<"role_index">> => #{10 => #{}},
        <<"channel_index">> => #{20 => #{}},
        <<"member_role_index">> => #{30 => #{}},
        role_perms_cache => #{10 => 1},
        overwrite_perms_cache => #{20 => [{10, 0, 1, 0}]}
    }),
    ?assertEqual(<<"100">>, maps:get(<<"id">>, Payload)),
    ?assertNot(maps:is_key(<<"role_index">>, Payload)),
    ?assertNot(maps:is_key(<<"channel_index">>, Payload)),
    ?assertNot(maps:is_key(<<"member_role_index">>, Payload)),
    ?assertNot(maps:is_key(<<"role_perms_cache">>, Payload)),
    ?assertNot(maps:is_key(<<"overwrite_perms_cache">>, Payload)).

payload_stringifies_schema_ids_and_permissions_test() ->
    Payload = payload(#{
        <<"id">> => 100,
        <<"guild_id">> => 200,
        <<"owner_id">> => null,
        <<"roles">> => [300, <<"400">>],
        <<"permissions">> => 1024,
        <<"permission_overwrites">> => [
            #{<<"id">> => 500, <<"type">> => 0, <<"allow">> => 1, <<"deny">> => 0}
        ],
        <<"nicks">> => #{10 => <<"Ada">>}
    }),
    ?assertEqual(<<"100">>, maps:get(<<"id">>, Payload)),
    ?assertEqual(<<"200">>, maps:get(<<"guild_id">>, Payload)),
    ?assertEqual(null, maps:get(<<"owner_id">>, Payload)),
    ?assertEqual([<<"300">>, <<"400">>], maps:get(<<"roles">>, Payload)),
    ?assertEqual(<<"1024">>, maps:get(<<"permissions">>, Payload)),
    ?assertEqual(
        [
            #{
                <<"id">> => <<"500">>,
                <<"type">> => 0,
                <<"allow">> => <<"1">>,
                <<"deny">> => <<"0">>
            }
        ],
        maps:get(<<"permission_overwrites">>, Payload)
    ),
    ?assertEqual(#{<<"10">> => <<"Ada">>}, maps:get(<<"nicks">>, Payload)).

payload_stringifies_rpc_atom_keys_and_schema_id_lists_test() ->
    Payload = payload(#{
        channel_id => 100,
        message_id => 200,
        user_id => 300,
        session_id => <<"session-a">>,
        connection_id => <<"conn-a">>,
        ringing => [400, <<"500">>],
        recipients => [600, <<"700">>],
        mention_roles => [800],
        nsfw_emojis => [900],
        pinned_dms => [1000],
        permissions => 8
    }),
    ?assertEqual(<<"100">>, maps:get(<<"channel_id">>, Payload)),
    ?assertEqual(<<"200">>, maps:get(<<"message_id">>, Payload)),
    ?assertEqual(<<"300">>, maps:get(<<"user_id">>, Payload)),
    ?assertEqual(<<"session-a">>, maps:get(<<"session_id">>, Payload)),
    ?assertEqual(<<"conn-a">>, maps:get(<<"connection_id">>, Payload)),
    ?assertEqual([<<"400">>, <<"500">>], maps:get(<<"ringing">>, Payload)),
    ?assertEqual([<<"600">>, <<"700">>], maps:get(<<"recipients">>, Payload)),
    ?assertEqual([<<"800">>], maps:get(<<"mention_roles">>, Payload)),
    ?assertEqual([<<"900">>], maps:get(<<"nsfw_emojis">>, Payload)),
    ?assertEqual([<<"1000">>], maps:get(<<"pinned_dms">>, Payload)),
    ?assertEqual(<<"8">>, maps:get(<<"permissions">>, Payload)).

payload_preserves_fluxer_object_lists_and_non_snowflake_ids_test() ->
    Payload = payload(#{
        <<"roles">> => [#{<<"id">> => 10, <<"name">> => <<"admin">>}],
        <<"mentions">> => [#{<<"id">> => 20, <<"username">> => <<"ada">>}],
        <<"recipients">> => [#{<<"id">> => 30, <<"username">> => <<"grace">>}],
        <<"guild_folders">> => [#{<<"id">> => -1, <<"guild_ids">> => [40]}],
        <<"rtc_regions">> => [#{<<"id">> => <<"us-east">>, <<"name">> => <<"East">>}]
    }),
    ?assertEqual(
        [#{<<"id">> => <<"10">>, <<"name">> => <<"admin">>}], maps:get(<<"roles">>, Payload)
    ),
    ?assertEqual(
        [#{<<"id">> => <<"20">>, <<"username">> => <<"ada">>}],
        maps:get(<<"mentions">>, Payload)
    ),
    ?assertEqual(
        [#{<<"id">> => <<"30">>, <<"username">> => <<"grace">>}],
        maps:get(<<"recipients">>, Payload)
    ),
    ?assertEqual(
        [#{<<"id">> => -1, <<"guild_ids">> => [<<"40">>]}],
        maps:get(<<"guild_folders">>, Payload)
    ),
    ?assertEqual(
        [#{<<"id">> => <<"us-east">>, <<"name">> => <<"East">>}],
        maps:get(<<"rtc_regions">>, Payload)
    ).

payload_stringifies_snowflake_record_list_values_test() ->
    Payload = payload(#{100 => [200, <<"300">>]}),
    ?assertEqual([<<"200">>, <<"300">>], maps:get(<<"100">>, Payload)).

payload(Data) ->
    #{} = guild_data_wire:payload(Data).
