%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_manager_cache_tests).
-typing([eqwalizer]).

-include_lib("eunit/include/eunit.hrl").

cleanup_guild_from_cache_deletes_all_matches_test() ->
    guild_manager_cache:ensure_guild_pid_cache(),
    Pid = self(),
    ets:insert(guild_pid_cache, {101, Pid}),
    ets:insert(guild_pid_cache, {102, Pid}),
    ets:insert(guild_pid_cache, {103, spawn(fun() -> ok end)}),
    try
        ok = guild_manager_cache:cleanup_guild_from_cache(Pid),
        ?assertEqual([], ets:lookup(guild_pid_cache, 101)),
        ?assertEqual([], ets:lookup(guild_pid_cache, 102))
    after
        ets:delete(guild_pid_cache, 101),
        ets:delete(guild_pid_cache, 102),
        ets:delete(guild_pid_cache, 103)
    end.

lookup_cached_guild_pid_requires_registration_test() ->
    process_registry:init(),
    guild_manager_cache:ensure_guild_pid_cache(),
    GuildId = 20201,
    GuildPid = spawn(fun() ->
        receive
            stop -> ok
        after infinity ->
            ok
        end
    end),
    timer:sleep(10),
    ets:insert(guild_pid_cache, {GuildId, GuildPid}),
    try
        ?assertEqual(not_found, guild_manager_cache:lookup_cached_guild_pid(GuildId)),
        ?assertEqual([], ets:lookup(guild_pid_cache, GuildId)),
        GuildKey = process_registry:build_process_key(guild, GuildId),
        ets:insert(guild_pid_cache, {GuildId, GuildPid}),
        ets:insert(process_registry_table, {GuildKey, GuildPid}),
        ?assertEqual({ok, GuildPid}, guild_manager_cache:lookup_cached_guild_pid(GuildId)),
        process_registry:registry_unregister(GuildKey)
    after
        ets:delete(guild_pid_cache, GuildId),
        GuildPid ! stop
    end.
