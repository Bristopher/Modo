%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_manager_shard_reload_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

select_guilds_to_reload_empty_ids_test() ->
    Guilds = #{1 => {self(), make_ref()}, 2 => {self(), make_ref()}},
    Result = guild_manager_shard_reload:select_guilds_to_reload([], Guilds),
    ?assertEqual(2, length(Result)).

select_guilds_to_reload_specific_ids_test() ->
    Pid = self(),
    Ref = make_ref(),
    Guilds = #{1 => {Pid, Ref}, 2 => {Pid, Ref}, 3 => loading},
    Result = guild_manager_shard_reload:select_guilds_to_reload([1, 3], Guilds),
    ?assertEqual(1, length(Result)).
