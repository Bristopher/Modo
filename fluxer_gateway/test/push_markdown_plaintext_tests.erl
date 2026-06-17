%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_markdown_plaintext_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

nif_available_test() ->
    ?assertEqual(true, push_markdown_plaintext:available()).

renders_notification_plaintext_with_mentions_and_preserved_markdown_test() ->
    Context = #{
        <<"preserve_markdown">> => true,
        <<"include_emoji_names">> => true,
        <<"users">> => #{<<"1">> => <<"Alice">>, <<"2">> => <<"Bob">>},
        <<"roles">> => #{<<"3">> => <<"On Call">>},
        <<"channels">> => #{<<"4">> => <<"alerts">>}
    },
    Content =
        <<
            "**Deploy** <@1> <@!2> <@&3> <#4> [docs](https://fluxer.app/docs) "
            "`code` ~~done~~ ||spoiler|| <:party:99>"
        >>,
    ?assertEqual(
        <<
            "**Deploy** @Alice @Bob @On Call #alerts [docs](https://fluxer.app/docs) "
            "`code` ~~done~~ ||spoiler|| :party:"
        >>,
        push_markdown_plaintext:render_push_preview(Content, Context)
    ).

renders_blocks_lists_tables_alerts_and_code_like_client_plaintext_test() ->
    Context = #{<<"preserve_markdown">> => true},
    Content =
        <<
            "# Title\nBody\n- one\n- two\n> quote\n\n"
            "| A | B |\n| - | - |\n| 1 | 2 |\n\n"
            "```erl\nok.\n```"
        >>,
    ?assertEqual(
        <<
            "# Title\nBody\n- one\n- two\n> quote\n"
            "| A | B |\n| --- | --- |\n| 1 | 2 |\n"
            "```erl\nok.\n```"
        >>,
        push_markdown_plaintext:render_push_preview(Content, Context)
    ).

renders_unknown_mentions_without_raw_parser_tokens_test() ->
    Context = #{<<"preserve_markdown">> => true},
    Content =
        <<
            "<@9> <@&10> <#11> @everyone @here </kick soft now:12> "
            "<id:linked-roles:77>"
        >>,
    ?assertEqual(
        <<
            "@9 @unknown-role #unknown-channel @everyone @here /kick soft now "
            "#linked-roles:77"
        >>,
        push_markdown_plaintext:render_push_preview(Content, Context)
    ).

context_builder_resolves_user_role_and_channel_names_test() ->
    MessageData = #{
        <<"content">> => <<"Hi <@1> <@&2> <#3> <#4>">>,
        <<"mentions">> => [
            #{<<"id">> => <<"1">>, <<"global_name">> => <<"Ada">>, <<"username">> => <<"ada1">>}
        ],
        <<"mention_channels">> => [
            #{<<"id">> => <<"3">>, <<"name">> => <<"release-notes">>}
        ]
    },
    GuildData = #{
        <<"channel_index">> => #{
            4 => #{<<"id">> => <<"4">>, <<"name">> => <<"ops">>}
        }
    },
    Context = push_notification_format:build_markdown_context(
        MessageData, 123, #{2 => <<"Engineers">>}, GuildData
    ),
    ?assertEqual(<<"Ada">>, maps:get(<<"1">>, maps:get(<<"users">>, Context))),
    ?assertEqual(<<"Engineers">>, maps:get(<<"2">>, maps:get(<<"roles">>, Context))),
    ?assertEqual(<<"release-notes">>, maps:get(<<"3">>, maps:get(<<"channels">>, Context))),
    ?assertEqual(<<"ops">>, maps:get(<<"4">>, maps:get(<<"channels">>, Context))).

context_builder_prefers_guild_nicknames_test() ->
    MessageData = #{
        <<"content">> => <<"Hi <@1> <@2>">>,
        <<"mentions">> => [
            #{
                <<"id">> => <<"1">>,
                <<"global_name">> => <<"Ada">>,
                <<"username">> => <<"ada1">>
            },
            #{
                <<"id">> => <<"2">>,
                <<"global_name">> => <<"Grace">>,
                <<"username">> => <<"grace1">>
            }
        ]
    },
    GuildData = #{
        <<"members">> => #{
            1 => #{<<"nick">> => <<"Guild Ada">>},
            2 => #{<<"nick">> => null}
        }
    },
    Context = push_notification_format:build_markdown_context(
        MessageData, 123, #{}, GuildData
    ),
    Users = maps:get(<<"users">>, Context),
    UserNicknames = maps:get(<<"user_nicknames">>, Context),
    ?assertEqual(<<"Guild Ada">>, maps:get(<<"1">>, Users)),
    ?assertEqual(<<"Grace">>, maps:get(<<"2">>, Users)),
    ?assertEqual(<<"Guild Ada">>, maps:get(<<"1">>, UserNicknames)).

context_builder_prefers_group_dm_nicknames_test() ->
    MessageData = #{
        <<"content">> => <<"Hi <@1>">>,
        <<"nicks">> => #{<<"1">> => <<"Group Ada">>},
        <<"mentions">> => [
            #{<<"id">> => <<"1">>, <<"global_name">> => <<"Ada">>, <<"username">> => <<"ada1">>}
        ]
    },
    Context = push_notification_format:build_markdown_context(MessageData, 0, #{}, #{}),
    ?assertEqual(<<"Group Ada">>, maps:get(<<"1">>, maps:get(<<"users">>, Context))),
    ?assertEqual(
        <<"Group Ada">>, maps:get(<<"1">>, maps:get(<<"user_nicknames">>, Context))
    ).

context_builder_filters_non_copyable_channel_types_test() ->
    MessageData = #{
        <<"content">> => <<"See <#3> <#4> <#5>">>,
        <<"mention_channels">> => [
            #{<<"id">> => <<"3">>, <<"name">> => <<"dm">>, <<"type">> => 1},
            #{<<"id">> => <<"4">>, <<"name">> => <<"category">>, <<"type">> => 4}
        ]
    },
    GuildData = #{
        <<"channel_index">> => #{
            5 => #{<<"id">> => <<"5">>, <<"name">> => <<"group-dm">>, <<"type">> => 3}
        }
    },
    Channels = maps:get(
        <<"channels">>,
        push_notification_format:build_markdown_context(MessageData, 123, #{}, GuildData)
    ),
    ?assertNot(maps:is_key(<<"3">>, Channels)),
    ?assertEqual(<<"category">>, maps:get(<<"4">>, Channels)),
    ?assertNot(maps:is_key(<<"5">>, Channels)).

content_preview_uses_markdown_context_and_truncates_after_rendering_test() ->
    Context = #{
        <<"preserve_markdown">> => true,
        <<"users">> => #{<<"1">> => <<"Ada">>},
        <<"roles">> => #{<<"2">> => <<"Ops">>},
        <<"channels">> => #{<<"3">> => <<"alerts">>}
    },
    MessageData = #{
        <<"content">> => <<"**Hello** <@1> <@&2> <#3>">>
    },
    ?assertEqual(
        <<"**Hello** @Ada @Ops #alerts">>,
        push_notification_format:build_content_preview(MessageData, Context)
    ).

oversized_content_skips_markdown_nif_test() ->
    Content = binary:copy(<<"<@1> ">>, 4000),
    Context = #{<<"users">> => #{<<"1">> => <<"Ada">>}},
    ?assertEqual(Content, push_markdown_plaintext:render_push_preview(Content, Context)).

oversized_context_name_map_is_dropped_before_nif_test() ->
    Users = maps:from_list([{integer_to_binary(I), <<"User">>} || I <- lists:seq(1, 600)]),
    Context = #{<<"users">> => Users},
    ?assertEqual(<<"@1">>, push_markdown_plaintext:render_push_preview(<<"<@1>">>, Context)).
