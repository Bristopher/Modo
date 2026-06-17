%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_permissions_check_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

find_member_by_user_id_found_test() ->
    State = #{
        data => #{
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => <<"123">>}, <<"nick">> => <<"Test">>}
            ]
        }
    },
    #{<<"nick">> := Nick} = guild_permissions_check:find_member_by_user_id(123, State),
    ?assertEqual(<<"Test">>, Nick).

find_member_by_user_id_not_found_test() ->
    State = #{data => #{<<"members">> => []}},
    ?assertEqual(undefined, guild_permissions_check:find_member_by_user_id(123, State)).

find_member_by_user_id_map_storage_test() ->
    State = #{
        data => #{
            <<"members">> => #{
                321 => #{<<"user">> => #{<<"id">> => <<"321">>}, <<"nick">> => <<"Mapped">>}
            }
        }
    },
    #{<<"nick">> := Nick} = guild_permissions_check:find_member_by_user_id(321, State),
    ?assertEqual(<<"Mapped">>, Nick).

find_member_by_user_id_binary_map_storage_test() ->
    State = #{
        data => #{
            <<"members">> => #{
                <<"654">> => #{
                    <<"user">> => #{<<"id">> => <<"654">>}, <<"nick">> => <<"Binary">>
                }
            }
        }
    },
    #{<<"nick">> := Nick} = guild_permissions_check:find_member_by_user_id(654, State),
    ?assertEqual(<<"Binary">>, Nick).

find_member_by_user_id_rejects_invalid_snowflake_test() ->
    State = #{
        data => #{
            <<"members">> => [
                #{<<"user">> => #{<<"id">> => <<"001">>}, <<"nick">> => <<"Invalid">>}
            ]
        }
    },
    ?assertEqual(undefined, guild_permissions_check:find_member_by_user_id(1, State)).

find_role_by_id_found_test() ->
    Roles = [#{<<"id">> => <<"100">>, <<"name">> => <<"Admin">>}],
    #{<<"name">> := Name} = guild_permissions_check:find_role_by_id(100, Roles),
    ?assertEqual(<<"Admin">>, Name).

find_role_by_id_not_found_test() ->
    Roles = [#{<<"id">> => <<"100">>}],
    ?assertEqual(undefined, guild_permissions_check:find_role_by_id(999, Roles)).

find_role_by_id_map_index_test() ->
    Roles = #{100 => #{<<"id">> => <<"100">>, <<"name">> => <<"Admin">>}},
    #{<<"name">> := Name} = guild_permissions_check:find_role_by_id(100, Roles),
    ?assertEqual(<<"Admin">>, Name).

find_role_by_id_rejects_invalid_snowflake_test() ->
    Roles = [#{<<"id">> => <<"001">>, <<"name">> => <<"Invalid">>}],
    ?assertEqual(undefined, guild_permissions_check:find_role_by_id(1, Roles)).

find_channel_by_id_with_index_test() ->
    State = #{
        data => #{
            <<"channels">> => [#{<<"id">> => <<"900">>, <<"name">> => <<"general">>}],
            <<"channel_index">> => #{
                900 => #{<<"id">> => <<"900">>, <<"name">> => <<"general">>}
            }
        }
    },
    #{<<"name">> := Name} = guild_permissions_check:find_channel_by_id(900, State),
    ?assertEqual(<<"general">>, Name).
