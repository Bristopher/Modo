%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_ets_cache).
-typing([eqwalizer]).

-export([
    init/0,
    get_user_guild_settings/2,
    put_user_guild_settings/3,
    delete_user_guild_settings/2,
    get_subscriptions/1,
    get_subscriptions_many/1,
    put_subscriptions/2,
    delete_subscriptions/1,
    get_blocked_ids/1,
    put_blocked_ids/2,
    get_badge_count/1,
    put_badge_count/3,
    delete_badge_count/1,
    rebalance/0,
    rebalance_async/0,
    cache_stats/0,
    evict_tables/1,
    table_size/1
]).

-define(USER_GUILD_SETTINGS, push_user_guild_settings).
-define(SUBSCRIPTIONS, push_subscriptions).
-define(BLOCKED_IDS, push_blocked_ids).
-define(BADGE_COUNTS, push_badge_counts).

-define(MAX_TABLE_ENTRIES, 500000).
-define(EVICT_BATCH, 4096).

-define(ETS_OPTS, [
    named_table, public, set, {read_concurrency, true}, {write_concurrency, true}
]).

-spec init() -> ok.
init() ->
    ensure_table(?USER_GUILD_SETTINGS),
    ensure_table(?SUBSCRIPTIONS),
    ensure_table(?BLOCKED_IDS),
    ensure_table(?BADGE_COUNTS),
    ok.

-spec get_user_guild_settings(integer(), integer()) -> map() | undefined.
get_user_guild_settings(UserId, GuildId) ->
    get_user_guild_settings_ets(UserId, GuildId).

-spec get_user_guild_settings_ets(integer(), integer()) -> map() | undefined.
get_user_guild_settings_ets(UserId, GuildId) ->
    try ets:lookup(?USER_GUILD_SETTINGS, {UserId, GuildId}) of
        [{{UserId, GuildId}, Settings}] -> Settings;
        _ -> undefined
    catch
        error:badarg -> undefined
    end.

-spec put_user_guild_settings(integer(), integer(), map()) -> ok.
put_user_guild_settings(UserId, GuildId, Settings) ->
    guard_table_size(?USER_GUILD_SETTINGS),
    ets:insert(?USER_GUILD_SETTINGS, {{UserId, GuildId}, Settings}),
    ok.

-spec delete_user_guild_settings(integer(), integer()) -> ok.
delete_user_guild_settings(UserId, GuildId) ->
    try ets:delete(?USER_GUILD_SETTINGS, {UserId, GuildId}) of
        _ -> ok
    catch
        throw:_ -> ok;
        error:_ -> ok;
        exit:_ -> ok
    end.

-spec get_subscriptions(integer()) -> list() | undefined.
get_subscriptions(UserId) ->
    get_subscriptions_ets(UserId).

-spec get_subscriptions_ets(integer()) -> list() | undefined.
get_subscriptions_ets(UserId) ->
    try ets:lookup(?SUBSCRIPTIONS, UserId) of
        [{UserId, Subs}] -> Subs;
        _ -> undefined
    catch
        error:badarg -> undefined
    end.

-spec get_subscriptions_many([integer()]) -> {#{integer() => list()}, [integer()]}.
get_subscriptions_many(UserIds) ->
    lists:foldl(
        fun add_cached_subscriptions/2,
        {#{}, []},
        UserIds
    ).

-spec add_cached_subscriptions(integer(), {#{integer() => list()}, [integer()]}) ->
    {#{integer() => list()}, [integer()]}.
add_cached_subscriptions(UserId, {CachedAcc, MissingAcc}) ->
    case get_subscriptions(UserId) of
        Subscriptions when is_list(Subscriptions) ->
            {CachedAcc#{UserId => Subscriptions}, MissingAcc};
        undefined ->
            {CachedAcc, [UserId | MissingAcc]}
    end.

-spec put_subscriptions(integer(), list()) -> ok.
put_subscriptions(UserId, Subscriptions) ->
    guard_table_size(?SUBSCRIPTIONS),
    ets:insert(?SUBSCRIPTIONS, {UserId, Subscriptions}),
    ok.

-spec delete_subscriptions(integer()) -> ok.
delete_subscriptions(UserId) ->
    try ets:delete(?SUBSCRIPTIONS, UserId) of
        _ -> ok
    catch
        error:badarg -> ok
    end.

-spec get_blocked_ids(integer()) -> [integer()] | undefined.
get_blocked_ids(UserId) ->
    get_blocked_ids_ets(UserId).

-spec get_blocked_ids_ets(integer()) -> [integer()] | undefined.
get_blocked_ids_ets(UserId) ->
    try ets:lookup(?BLOCKED_IDS, UserId) of
        [{UserId, BlockedIds}] -> BlockedIds;
        _ -> undefined
    catch
        error:badarg -> undefined
    end.

-spec put_blocked_ids(integer(), [integer()]) -> ok.
put_blocked_ids(UserId, BlockedIds) ->
    guard_table_size(?BLOCKED_IDS),
    ets:insert(?BLOCKED_IDS, {UserId, BlockedIds}),
    ok.

-spec get_badge_count(integer()) -> {non_neg_integer(), integer()} | undefined.
get_badge_count(UserId) ->
    get_badge_count_ets(UserId).

-spec get_badge_count_ets(integer()) -> {non_neg_integer(), integer()} | undefined.
get_badge_count_ets(UserId) ->
    try ets:lookup(?BADGE_COUNTS, UserId) of
        [{UserId, Count, CachedAt}] -> {Count, CachedAt};
        _ -> undefined
    catch
        error:badarg -> undefined
    end.

-spec put_badge_count(integer(), non_neg_integer(), integer()) -> ok.
put_badge_count(UserId, Count, CachedAt) ->
    guard_table_size(?BADGE_COUNTS),
    try
        case ets:insert_new(?BADGE_COUNTS, {UserId, Count, CachedAt}) of
            true ->
                ok;
            false ->
                conditional_replace_badge_count(UserId, Count, CachedAt)
        end
    catch
        error:badarg -> ok
    end,
    ok.

-spec conditional_replace_badge_count(integer(), non_neg_integer(), integer()) -> ok.
conditional_replace_badge_count(UserId, Count, CachedAt) ->
    MatchSpec = [
        {
            {UserId, '$1', '$2'},
            [{'=<', '$2', CachedAt}],
            [{{{const, UserId}, {const, Count}, {const, CachedAt}}}]
        }
    ],
    try ets:select_replace(?BADGE_COUNTS, MatchSpec) of
        Replaced when is_integer(Replaced), Replaced > 0 ->
            ok;
        _ ->
            retry_conditional_replace_badge_count(UserId, Count, CachedAt)
    catch
        error:badarg -> ok
    end.

-spec retry_conditional_replace_badge_count(integer(), non_neg_integer(), integer()) -> ok.
retry_conditional_replace_badge_count(UserId, Count, CachedAt) ->
    case get_badge_count_ets(UserId) of
        {_ExistingCount, ExistingCachedAt} when ExistingCachedAt > CachedAt ->
            ok;
        _ ->
            insert_badge_count_safely(UserId, Count, CachedAt)
    end.

-spec insert_badge_count_safely(integer(), non_neg_integer(), integer()) -> ok.
insert_badge_count_safely(UserId, Count, CachedAt) ->
    try ets:insert(?BADGE_COUNTS, {UserId, Count, CachedAt}) of
        _ -> ok
    catch
        error:badarg -> ok
    end.

-spec delete_badge_count(integer()) -> ok.
delete_badge_count(UserId) ->
    try ets:delete(?BADGE_COUNTS, UserId) of
        _ -> ok
    catch
        error:badarg -> ok
    end.

-spec rebalance_async() -> ok.
rebalance_async() ->
    _ = spawn(fun rebalance/0),
    ok.

-spec rebalance() -> ok.
rebalance() ->
    init(),
    _ = rebalance_table(?USER_GUILD_SETTINGS, fun user_id_from_user_guild_key/1),
    _ = rebalance_table(?SUBSCRIPTIONS, fun user_id_from_key/1),
    _ = rebalance_table(?BLOCKED_IDS, fun user_id_from_key/1),
    _ = rebalance_table(?BADGE_COUNTS, fun user_id_from_key/1),
    ok.

-spec cache_stats() -> map().
cache_stats() ->
    #{
        user_guild_settings_size => table_size(?USER_GUILD_SETTINGS),
        push_subscriptions_size => table_size(?SUBSCRIPTIONS),
        blocked_ids_size => table_size(?BLOCKED_IDS),
        badge_counts_size => table_size(?BADGE_COUNTS)
    }.

-spec evict_tables(map()) -> ok.
evict_tables(MaxEntries) ->
    evict_table(?USER_GUILD_SETTINGS, maps:get(user_guild_settings, MaxEntries, undefined)),
    evict_table(?SUBSCRIPTIONS, maps:get(subscriptions, MaxEntries, undefined)),
    evict_table(?BLOCKED_IDS, maps:get(blocked_ids, MaxEntries, undefined)),
    evict_table(?BADGE_COUNTS, maps:get(badge_counts, MaxEntries, undefined)),
    ok.

-spec guard_table_size(atom()) -> ok.
guard_table_size(Table) ->
    case table_size(Table) >= ?MAX_TABLE_ENTRIES of
        true -> evict_table(Table, ?MAX_TABLE_ENTRIES - ?EVICT_BATCH);
        false -> ok
    end.

-spec ensure_table(atom()) -> ok.
ensure_table(Name) ->
    case ets:whereis(Name) of
        undefined -> create_table(Name);
        _ -> ok
    end.

-spec create_table(atom()) -> ok.
create_table(Name) ->
    try
        _ = ets:new(Name, ?ETS_OPTS),
        ok
    catch
        error:badarg -> ok
    end.

-spec table_size(atom()) -> non_neg_integer().
table_size(Table) ->
    try ets:info(Table, size) of
        Size when is_integer(Size) -> Size;
        _ -> 0
    catch
        error:badarg -> 0
    end.

-spec evict_table(atom(), non_neg_integer() | undefined) -> ok.
evict_table(_Table, undefined) ->
    ok;
evict_table(Table, MaxEntries) ->
    Size = table_size(Table),
    case Size > MaxEntries of
        true ->
            ToDelete = Size - MaxEntries,
            evict_n(Table, ets:first(Table), ToDelete);
        false ->
            ok
    end.

-spec evict_n(atom(), term(), non_neg_integer()) -> ok.
evict_n(_Table, '$end_of_table', _N) ->
    ok;
evict_n(_Table, _Key, 0) ->
    ok;
evict_n(Table, Key, N) ->
    Next = ets:next(Table, Key),
    ets:delete(Table, Key),
    evict_n(Table, Next, N - 1).

-spec rebalance_table(atom(), fun((term()) -> integer() | undefined)) -> non_neg_integer().
rebalance_table(Table, UserIdFun) ->
    KeysToDelete = collect_non_local_keys(Table, UserIdFun),
    lists:foreach(fun(Key) -> safe_delete(Table, Key) end, KeysToDelete),
    length(KeysToDelete).

-spec collect_non_local_keys(atom(), fun((term()) -> integer() | undefined)) -> [term()].
collect_non_local_keys(Table, UserIdFun) ->
    try
        ets:foldl(
            fun(Record, Acc) ->
                Key = element(1, Record),
                accumulate_if_remote(Key, UserIdFun, Acc)
            end,
            [],
            Table
        )
    catch
        error:badarg -> []
    end.

-spec accumulate_if_remote(term(), fun((term()) -> integer() | undefined), [term()]) ->
    [term()].
accumulate_if_remote(Key, UserIdFun, Acc) ->
    case UserIdFun(Key) of
        UserId when is_integer(UserId) -> accumulate_owner_key(Key, UserId, Acc);
        undefined -> Acc
    end.

-spec accumulate_owner_key(term(), integer(), [term()]) -> [term()].
accumulate_owner_key(Key, UserId, Acc) ->
    case gateway_node_router:owner_node_result(UserId, push) of
        {ok, OwnerNode} when OwnerNode =:= node() -> Acc;
        {ok, OwnerNode} when is_atom(OwnerNode) -> [Key | Acc];
        {error, _Reason} -> Acc
    end.

-spec safe_delete(atom(), term()) -> ok.
safe_delete(Table, Key) ->
    try ets:delete(Table, Key) of
        _ -> ok
    catch
        error:badarg -> ok
    end.

-spec user_id_from_user_guild_key(term()) -> integer() | undefined.
user_id_from_user_guild_key({UserId, _GuildId}) when is_integer(UserId) ->
    UserId;
user_id_from_user_guild_key(_) ->
    undefined.

-spec user_id_from_key(term()) -> integer() | undefined.
user_id_from_key(UserId) when is_integer(UserId) ->
    UserId;
user_id_from_key(_) ->
    undefined.
