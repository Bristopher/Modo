%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_members_query).
-typing([eqwalizer]).

-export([
    get_users_to_mention_by_roles/2,
    get_users_to_mention_by_user_ids/2,
    get_all_users_to_mention/2,
    get_members_with_role/2,
    get_viewable_channels/2
]).

-export_type([guild_state/0, guild_reply/1]).

-type guild_state() :: map().
-type guild_reply(T) :: {reply, T, guild_state()}.

-spec get_users_to_mention_by_roles(map(), guild_state()) -> guild_reply(map()).
get_users_to_mention_by_roles(
    #{channel_id := ChannelId, role_ids := RoleIds, author_id := AuthorId}, State
) ->
    RoleIdList = guild_members_common:normalize_int_list(RoleIds),
    CandidateUserIds = guild_members_roles:user_ids_for_any_role(RoleIdList, State),
    UserIds = collect_user_mentions(CandidateUserIds, AuthorId, ChannelId, State),
    {reply, #{user_ids => UserIds}, State}.

-spec get_users_to_mention_by_user_ids(map(), guild_state()) -> guild_reply(map()).
get_users_to_mention_by_user_ids(
    #{channel_id := ChannelId, user_ids := UserIdsReq, author_id := AuthorId}, State
) ->
    TargetIds = guild_members_common:normalize_int_list(UserIdsReq),
    UserIds = collect_user_mentions(TargetIds, AuthorId, ChannelId, State),
    {reply, #{user_ids => UserIds}, State}.

-spec get_all_users_to_mention(map(), guild_state()) -> guild_reply(map()).
get_all_users_to_mention(#{channel_id := ChannelId, author_id := AuthorId}, State) ->
    Members = guild_members_common:guild_members(State),
    UserIds = guild_members_common:collect_mentions(
        Members, AuthorId, ChannelId, State, fun(_) -> true end
    ),
    {reply, #{user_ids => UserIds}, State}.

-spec collect_user_mentions([integer()], integer(), integer(), guild_state()) -> [integer()].
collect_user_mentions(UserIds, AuthorId, ChannelId, State) ->
    guild_members_common:collect_mentions_for_user_ids(
        UserIds, AuthorId, ChannelId, State, fun(_UserId, _Member) -> true end
    ).

-spec get_members_with_role(map(), guild_state()) -> guild_reply(map()).
get_members_with_role(#{role_id := RoleId}, State) ->
    Data = guild_members_common:guild_data(State),
    MemberRoleIndex = guild_data_index:member_role_index(Data),
    UserMap = user_map_for_role(RoleId, MemberRoleIndex),
    {reply, #{user_ids => lists:sort(maps:keys(UserMap))}, State}.

-spec user_map_for_role(term(), map()) -> map().
user_map_for_role(RoleId, MemberRoleIndex) ->
    case snowflake_id:parse_maybe(RoleId) of
        undefined -> #{};
        TargetRoleId -> maps:get(TargetRoleId, MemberRoleIndex, #{})
    end.

-spec get_viewable_channels(map(), guild_state()) -> guild_reply(map()).
get_viewable_channels(#{user_id := UserId}, State) ->
    Channels = guild_members_common:guild_channels(State),
    case guild_permissions:find_member_by_user_id(UserId, State) of
        undefined ->
            {reply, #{channel_ids => []}, State};
        Member ->
            ChannelIds = viewable_channel_ids(Channels, UserId, Member, State),
            {reply, #{channel_ids => ChannelIds}, State}
    end.

-spec viewable_channel_ids([map()], integer(), map(), guild_state()) -> [integer()].
viewable_channel_ids(Channels, UserId, Member, State) ->
    lists:filtermap(
        fun(Channel) ->
            channel_viewable_filter(Channel, UserId, Member, State)
        end,
        Channels
    ).

-spec channel_viewable_filter(map(), integer(), map(), guild_state()) ->
    false | {true, integer()}.
channel_viewable_filter(Channel, UserId, Member, State) ->
    case channel_id(Channel) of
        undefined -> false;
        ChannelId -> viewable_channel_id(UserId, ChannelId, Member, State)
    end.

-spec channel_id(map()) -> integer() | undefined.
channel_id(Channel) ->
    SafeChannel = map_utils:ensure_map(Channel),
    snowflake_id:parse_maybe(maps:get(<<"id">>, SafeChannel, undefined)).

-spec viewable_channel_id(integer(), integer(), map(), guild_state()) ->
    false | {true, integer()}.
viewable_channel_id(UserId, ChannelId, Member, State) ->
    case guild_permissions:can_view_channel(UserId, ChannelId, Member, State) of
        true -> {true, ChannelId};
        false -> false
    end.
