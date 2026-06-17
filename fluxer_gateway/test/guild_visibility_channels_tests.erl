%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_visibility_channels_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

filter_connected_session_entries_excludes_pending_test() ->
    Normal = #{session_id => <<"s1">>, user_id => 1, pending_connect => false},
    Pending = #{session_id => <<"s2">>, user_id => 2, pending_connect => true},
    NoPending = #{session_id => <<"s3">>, user_id => 3},
    Sessions = #{<<"s1">> => Normal, <<"s2">> => Pending, <<"s3">> => NoPending},
    Result = guild_visibility_channels:filter_connected_session_entries(Sessions),
    ResultIds = lists:sort([Sid || {Sid, _} <- Result]),
    ?assertEqual([<<"s1">>, <<"s3">>], ResultIds).

update_viewable_map_for_channel_add_test() ->
    Map = #{100 => true},
    Result = guild_visibility_channels:update_viewable_map_for_channel(Map, 200, true),
    ?assertEqual(true, maps:is_key(200, Result)),
    ?assertEqual(true, maps:is_key(100, Result)).

update_viewable_map_for_channel_remove_test() ->
    Map = #{100 => true, 200 => true},
    Result = guild_visibility_channels:update_viewable_map_for_channel(Map, 100, false),
    ?assertEqual(false, maps:is_key(100, Result)),
    ?assertEqual(true, maps:is_key(200, Result)).

viewable_channel_map_test() ->
    Set = sets:from_list([10, 20, 30]),
    Map = guild_visibility_channels:viewable_channel_map(Set),
    ?assertEqual(3, map_size(Map)),
    ?assertEqual(true, maps:get(10, Map)),
    ?assertEqual(true, maps:get(20, Map)),
    ?assertEqual(true, maps:get(30, Map)).

viewable_channel_map_empty_test() ->
    Map = guild_visibility_channels:viewable_channel_map(sets:new()),
    ?assertEqual(#{}, Map).

viewable_channel_set_uses_cached_session_data_test() ->
    UserId = 10,
    State = #{
        sessions => #{
            <<"s1">> => #{
                user_id => UserId,
                viewable_channels => #{100 => true, 200 => true}
            }
        },
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"members">> => [],
            <<"channels">> => [],
            <<"roles">> => []
        }
    },
    ChannelSet = guild_visibility_channels:viewable_channel_set(UserId, State),
    ?assertEqual(true, sets:is_element(100, ChannelSet)),
    ?assertEqual(true, sets:is_element(200, ChannelSet)),
    ?assertEqual(false, sets:is_element(999, ChannelSet)).

administrator_sees_all_channels_test() ->
    GuildId = 50,
    UserId = 10,
    ChannelId = 100,
    Admin = constants:administrator_permission(),
    State = #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => [
                #{
                    <<"id">> => integer_to_binary(GuildId),
                    <<"permissions">> => integer_to_binary(Admin)
                }
            ],
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => integer_to_binary(UserId)}, <<"roles">> => []}
            ],
            <<"channels">> => [
                #{
                    <<"id">> => integer_to_binary(ChannelId),
                    <<"permission_overwrites">> => [
                        #{
                            <<"id">> => integer_to_binary(GuildId),
                            <<"type">> => 0,
                            <<"allow">> => <<"0">>,
                            <<"deny">> => integer_to_binary(constants:view_channel_permission())
                        }
                    ]
                }
            ]
        }
    },
    Channels = guild_visibility_channels:get_user_viewable_channels(UserId, State),
    ?assertEqual([ChannelId], Channels).

owner_sees_all_channels_test() ->
    GuildId = 60,
    OwnerId = 10,
    ChannelId = 200,
    State = #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => integer_to_binary(OwnerId)},
            <<"roles">> => [
                #{<<"id">> => integer_to_binary(GuildId), <<"permissions">> => <<"0">>}
            ],
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => integer_to_binary(OwnerId)}, <<"roles">> => []}
            ],
            <<"channels">> => [
                #{<<"id">> => integer_to_binary(ChannelId), <<"permission_overwrites">> => []}
            ]
        }
    },
    Channels = guild_visibility_channels:get_user_viewable_channels(OwnerId, State),
    ?assertEqual([ChannelId], Channels).

everyone_role_grants_view_test() ->
    GuildId = 70,
    UserId = 10,
    ChannelId = 300,
    ViewPerm = constants:view_channel_permission(),
    State = #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => [
                #{
                    <<"id">> => integer_to_binary(GuildId),
                    <<"permissions">> => integer_to_binary(ViewPerm)
                }
            ],
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => integer_to_binary(UserId)}, <<"roles">> => []}
            ],
            <<"channels">> => [
                #{<<"id">> => integer_to_binary(ChannelId), <<"permission_overwrites">> => []}
            ]
        }
    },
    Channels = guild_visibility_channels:get_user_viewable_channels(UserId, State),
    ?assertEqual([ChannelId], Channels).

multiple_channels_partial_visibility_test() ->
    GuildId = 110,
    UserId = 10,
    ViewPerm = constants:view_channel_permission(),
    State = #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => [
                #{
                    <<"id">> => integer_to_binary(GuildId),
                    <<"permissions">> => integer_to_binary(ViewPerm)
                },
                #{<<"id">> => <<"300">>, <<"permissions">> => <<"0">>}
            ],
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => integer_to_binary(UserId)}, <<"roles">> => []}
            ],
            <<"channels">> => [
                #{<<"id">> => <<"100">>, <<"permission_overwrites">> => []},
                #{
                    <<"id">> => <<"101">>,
                    <<"permission_overwrites">> => [
                        #{
                            <<"id">> => integer_to_binary(GuildId),
                            <<"type">> => 0,
                            <<"allow">> => <<"0">>,
                            <<"deny">> => integer_to_binary(ViewPerm)
                        }
                    ]
                },
                #{<<"id">> => <<"102">>, <<"permission_overwrites">> => []}
            ]
        }
    },
    Channels = lists:sort(guild_visibility_channels:get_user_viewable_channels(UserId, State)),
    ?assertEqual([100, 102], Channels).

invalid_channel_ids_are_skipped_test() ->
    GuildId = 120,
    UserId = 10,
    ViewPerm = constants:view_channel_permission(),
    State = #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => [
                #{
                    <<"id">> => integer_to_binary(GuildId),
                    <<"permissions">> => integer_to_binary(ViewPerm)
                }
            ],
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => integer_to_binary(UserId)}, <<"roles">> => []}
            ],
            <<"channels">> => [
                #{<<"id">> => <<"001">>, <<"permission_overwrites">> => []},
                #{<<"id">> => <<"400">>, <<"permission_overwrites">> => []}
            ]
        }
    },
    Channels = guild_visibility_channels:get_user_viewable_channels(UserId, State),
    ?assertEqual([400], Channels).

invalid_parent_id_does_not_include_category_test() ->
    GuildId = 130,
    UserId = 10,
    ViewPerm = constants:view_channel_permission(),
    State = #{
        id => GuildId,
        data => #{
            <<"guild">> => #{<<"owner_id">> => <<"999">>},
            <<"roles">> => [
                #{
                    <<"id">> => integer_to_binary(GuildId),
                    <<"permissions">> => integer_to_binary(ViewPerm)
                }
            ],
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => integer_to_binary(UserId)}, <<"roles">> => []}
            ],
            <<"channels">> => [
                #{
                    <<"id">> => <<"2">>,
                    <<"type">> => 4,
                    <<"permission_overwrites">> => [
                        #{
                            <<"id">> => integer_to_binary(GuildId),
                            <<"type">> => 0,
                            <<"allow">> => <<"0">>,
                            <<"deny">> => integer_to_binary(ViewPerm)
                        }
                    ]
                },
                #{
                    <<"id">> => <<"401">>,
                    <<"parent_id">> => <<"002">>,
                    <<"permission_overwrites">> => []
                }
            ]
        }
    },
    Channels = guild_visibility_channels:get_user_viewable_channels(UserId, State),
    ?assertEqual([401], Channels).
