%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_tests).
-typing([eqwalizer]).
-include_lib("eunit/include/eunit.hrl").

clear_channel_notifications_disabled_by_default_test() ->
    erase_persistent_term(push_noop),
    erase_persistent_term(push_clear_notifications_enabled),
    ?assertEqual(ok, push:clear_channel_notifications(1, 2, 3)).

push_owner_key_prefers_first_recipient_test() ->
    ?assertEqual(
        42,
        push:push_owner_key(#{
            user_ids => [42, 99],
            author_id => 10,
            guild_id => 20
        })
    ).

push_owner_key_falls_back_to_author_then_guild_test() ->
    ?assertEqual(10, push:push_owner_key(#{user_ids => [], author_id => 10, guild_id => 20})),
    ?assertEqual(20, push:push_owner_key(#{guild_id => 20})).

push_owner_key_rejects_malformed_ids_test() ->
    ?assertEqual(42, push:push_owner_key(#{user_ids => [<<"42">>, 99]})),
    ?assertEqual(
        undefined,
        push:push_owner_key(#{
            user_ids => [<<"bad">>],
            author_id => <<"not-an-id">>,
            guild_id => <<"001">>
        })
    ).

message_params_context_normalizes_string_ids_test() ->
    Params = #{
        message_data => #{<<"channel_id">> => <<"123">>, <<"id">> => <<"456">>},
        user_ids => [<<"42">>],
        guild_id => <<"789">>,
        author_id => <<"7">>,
        guild_default_notifications => <<"1">>,
        role_names => #{10 => <<"Admins">>}
    },
    {ok, Context} = push_message_params:context(Params),
    ?assertEqual([42], maps:get(user_ids, Context)),
    ?assertEqual(789, maps:get(guild_id, Context)),
    ?assertEqual(7, maps:get(author_id, Context)),
    ?assertEqual(123, maps:get(channel_id, Context)),
    ?assertEqual(456, maps:get(message_id, Context)),
    ?assertEqual(1, maps:get(guild_default_notifications, Context)),
    ?assertEqual(#{10 => <<"Admins">>}, maps:get(role_names, Context)).

message_params_context_defaults_invalid_role_names_test() ->
    Params = #{
        message_data => #{<<"channel_id">> => <<"123">>, <<"id">> => <<"456">>},
        user_ids => [42],
        guild_id => <<"789">>,
        author_id => <<"7">>,
        guild_default_notifications => <<"1">>,
        role_names => invalid
    },
    {ok, Context} = push_message_params:context(Params),
    ?assertEqual(#{}, maps:get(role_names, Context)).

message_params_context_builds_group_dm_markdown_context_with_nicks_test() ->
    Params = #{
        message_data => #{
            <<"channel_id">> => <<"123">>,
            <<"id">> => <<"456">>,
            <<"content">> => <<"Hi <@42>">>,
            <<"channel_type">> => 3,
            <<"nicks">> => #{<<"42">> => <<"Group Nick">>},
            <<"mentions">> => [
                #{
                    <<"id">> => <<"42">>,
                    <<"global_name">> => <<"Global Name">>,
                    <<"username">> => <<"user42">>
                }
            ]
        },
        user_ids => [42],
        guild_id => 0,
        author_id => <<"7">>
    },
    {ok, Context} = push_message_params:context(Params),
    MarkdownContext = maps:get(markdown_context, Context),
    ?assertEqual(
        <<"Group Nick">>, maps:get(<<"42">>, maps:get(<<"users">>, MarkdownContext))
    ),
    ?assertEqual(
        <<"Group Nick">>,
        maps:get(<<"42">>, maps:get(<<"user_nicknames">>, MarkdownContext))
    ).

message_params_context_rejects_malformed_ids_test() ->
    Params = #{
        message_data => #{<<"channel_id">> => <<"bad">>, <<"id">> => <<"456">>},
        user_ids => [42],
        guild_id => <<"789">>,
        author_id => <<"7">>
    },
    ?assertEqual({error, invalid_channel_id}, push_message_params:context(Params)).

message_params_context_requires_explicit_guild_id_test() ->
    Params = #{
        message_data => #{<<"channel_id">> => <<"123">>, <<"id">> => <<"456">>},
        user_ids => [42],
        author_id => 7
    },
    DmParams = Params#{guild_id => 0},
    ?assertEqual({error, invalid_guild_id}, push_message_params:context(Params)),
    {ok, Context} = push_message_params:context(DmParams),
    ?assertEqual(0, maps:get(guild_id, Context)).

erase_persistent_term(Key) ->
    try persistent_term:erase(Key) of
        _ -> ok
    catch
        error:badarg -> ok
    end.
