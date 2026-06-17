%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_request_members_presence).
-typing([eqwalizer]).

-export([
    maybe_fetch_presences/3,
    presence_visible/1,
    presence_user_id/1
]).

-export_type([member/0, presence/0]).

-type member() :: map().
-type presence() :: map().

-spec maybe_fetch_presences(boolean(), pid(), [member()]) -> [presence()].
maybe_fetch_presences(false, _GuildPid, _Members) ->
    [];
maybe_fetch_presences(true, _GuildPid, Members) ->
    UserIds = extract_member_user_ids(Members),
    fetch_visible_presences(UserIds).

-spec extract_member_user_ids([member()]) -> [integer()].
extract_member_user_ids(Members) ->
    lists:filtermap(
        fun member_user_id_filter/1,
        Members
    ).

-spec member_user_id_filter(member()) -> {true, integer()} | false.
member_user_id_filter(Member) ->
    case guild_request_members_search:extract_user_id(Member) of
        undefined -> false;
        UserId -> {true, UserId}
    end.

-spec fetch_visible_presences([integer()]) -> [presence()].
fetch_visible_presences([]) ->
    [];
fetch_visible_presences(UserIds) ->
    Cached = safe_presence_cache_bulk_get(UserIds),
    [P || P <- Cached, presence_visible(P)].

-spec safe_presence_cache_bulk_get([integer()]) -> [map()].
safe_presence_cache_bulk_get(UserIds) ->
    try presence_cache:bulk_get(UserIds) of
        Presences when is_list(Presences) ->
            [P || P <- Presences, is_map(P)]
    catch
        _:_ -> []
    end.

-spec presence_visible(presence()) -> boolean().
presence_visible(P) ->
    Status = maps:get(<<"status">>, P, <<"offline">>),
    Status =/= <<"offline">> andalso Status =/= <<"invisible">>.

-spec presence_user_id(presence()) -> integer() | undefined.
presence_user_id(Presence) when is_map(Presence) ->
    User = map_utils:ensure_map(maps:get(<<"user">>, Presence, #{})),
    snowflake_id:parse_maybe(maps:get(<<"id">>, User, undefined));
presence_user_id(_) ->
    undefined.
