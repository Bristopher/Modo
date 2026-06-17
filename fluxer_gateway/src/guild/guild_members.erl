%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_members).
-typing([eqwalizer]).

-export([
    get_users_to_mention_by_roles/2,
    get_users_to_mention_by_user_ids/2,
    get_all_users_to_mention/2,
    resolve_all_mentions/2,
    resolve_mention_sources/2,
    resolve_mention_sources_page/2,
    resolve_channel_mentions/2,
    get_members_with_role/2,
    can_manage_roles/2,
    can_manage_role/2,
    get_assignable_roles/2,
    check_target_member/2,
    get_viewable_channels/2
]).

-export_type([guild_state/0, guild_reply/1]).

-type guild_state() :: map().
-type guild_reply(T) :: {reply, T, guild_state()}.

-spec get_users_to_mention_by_roles(map(), guild_state()) -> guild_reply(map()).
get_users_to_mention_by_roles(R, S) -> guild_members_query:get_users_to_mention_by_roles(R, S).

-spec get_users_to_mention_by_user_ids(map(), guild_state()) -> guild_reply(map()).
get_users_to_mention_by_user_ids(R, S) ->
    guild_members_query:get_users_to_mention_by_user_ids(R, S).

-spec get_all_users_to_mention(map(), guild_state()) -> guild_reply(map()).
get_all_users_to_mention(R, S) -> guild_members_query:get_all_users_to_mention(R, S).

-spec resolve_all_mentions(map(), guild_state()) -> guild_reply(map()).
resolve_all_mentions(R, S) -> guild_members_mutation:resolve_all_mentions(R, S).

-spec resolve_mention_sources(map(), guild_state()) -> guild_reply(map()).
resolve_mention_sources(R, S) -> guild_members_mutation:resolve_mention_sources(R, S).

-spec resolve_mention_sources_page(map(), guild_state()) -> guild_reply(map()).
resolve_mention_sources_page(R, S) -> guild_members_mutation:resolve_mention_sources_page(R, S).

-spec resolve_channel_mentions(map(), guild_state()) -> guild_reply(map()).
resolve_channel_mentions(R, S) -> guild_members_mutation:resolve_channel_mentions(R, S).

-spec get_members_with_role(map(), guild_state()) -> guild_reply(map()).
get_members_with_role(R, S) -> guild_members_query:get_members_with_role(R, S).

-spec can_manage_roles(map(), guild_state()) -> guild_reply(map()).
can_manage_roles(R, S) -> guild_members_roles:can_manage_roles(R, S).

-spec can_manage_role(map(), guild_state()) -> guild_reply(map()).
can_manage_role(R, S) -> guild_members_roles:can_manage_role(R, S).

-spec get_assignable_roles(map(), guild_state()) -> guild_reply(map()).
get_assignable_roles(R, S) -> guild_members_roles:get_assignable_roles(R, S).

-spec check_target_member(map(), guild_state()) -> guild_reply(map()).
check_target_member(R, S) -> guild_members_roles:check_target_member(R, S).

-spec get_viewable_channels(map(), guild_state()) -> guild_reply(map()).
get_viewable_channels(R, S) -> guild_members_query:get_viewable_channels(R, S).
