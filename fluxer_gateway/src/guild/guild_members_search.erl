%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_members_search).
-typing([eqwalizer]).

-export([take_mention_source_page/4]).

-export_type([guild_state/0, member/0, user_id/0, channel_id/0]).

-type guild_state() :: map().
-type member() :: map().
-type user_id() :: integer().
-type channel_id() :: integer().

-spec take_mention_source_page([user_id()], non_neg_integer(), #{user_id() => member()}, map()) ->
    {[map()], user_id() | undefined}.
take_mention_source_page(Candidates, Limit, MemberMap, Context) ->
    take_page(Candidates, Limit, MemberMap, Context, []).

-spec take_page(
    [user_id()], non_neg_integer(), #{user_id() => member()}, map(), [map()]
) -> {[map()], user_id() | undefined}.
take_page([], _Limit, _MemberMap, _Context, Acc) ->
    {lists:reverse(Acc), undefined};
take_page(_Candidates, 0, _MemberMap, _Context, Acc) ->
    {lists:reverse(Acc), undefined};
take_page([UserId | Rest], Limit, MemberMap, Context, Acc) ->
    case build_entry(UserId, MemberMap, Context) of
        undefined ->
            take_page(Rest, Limit, MemberMap, Context, Acc);
        Entry ->
            collect_or_finish(Entry, UserId, Rest, Limit, MemberMap, Context, Acc)
    end.

-spec collect_or_finish(
    map(),
    user_id(),
    [user_id()],
    non_neg_integer(),
    #{user_id() => member()},
    map(),
    [map()]
) -> {[map()], user_id() | undefined}.
collect_or_finish(Entry, UserId, Rest, 1, _MemberMap, _Context, Acc) ->
    NextAcc = [Entry | Acc],
    NextCursor =
        case Rest of
            [] -> undefined;
            _ -> UserId
        end,
    {lists:reverse(NextAcc), NextCursor};
collect_or_finish(Entry, _UserId, Rest, Limit, MemberMap, Context, Acc) ->
    take_page(Rest, Limit - 1, MemberMap, Context, [Entry | Acc]).

-spec build_entry(user_id(), #{user_id() => member()}, map()) -> map() | undefined.
build_entry(UserId, _MemberMap, #{author_id := UserId}) ->
    undefined;
build_entry(UserId, MemberMap, Context) ->
    case maps:get(UserId, MemberMap, undefined) of
        undefined -> undefined;
        Member -> build_entry_for_member(UserId, Member, Context)
    end.

-spec build_entry_for_member(user_id(), member(), map()) -> map() | undefined.
build_entry_for_member(UserId, Member, Context) ->
    #{channel_id := ChannelId, state := State} = Context,
    IsBot = guild_members_common:is_member_bot(Member),
    CanView = guild_members_common:member_can_view_channel(UserId, ChannelId, Member, State),
    case IsBot orelse not CanView of
        true -> undefined;
        false -> build_mention_entry(UserId, Member, Context)
    end.

-spec build_mention_entry(user_id(), member(), map()) -> map() | undefined.
build_mention_entry(UserId, Member, Context) ->
    #{
        mention_everyone := MentionEveryone,
        mention_here := MentionHere,
        has_role_mentions := HasRoleMentions,
        has_direct_mentions := HasDirectMentions,
        role_id_set := RoleIdSet,
        direct_user_id_set := DirectUserIdSet,
        connected_user_ids := ConnectedUserIds
    } = Context,
    Direct = HasDirectMentions andalso gb_sets:is_member(UserId, DirectUserIdSet),
    Role =
        HasRoleMentions andalso guild_members_common:member_has_any_role_set(Member, RoleIdSet),
    Everyone =
        MentionEveryone orelse
            (MentionHere andalso gb_sets:is_member(UserId, ConnectedUserIds)),
    case Direct orelse Role orelse Everyone of
        true -> #{user_id => UserId, direct => Direct, role => Role, everyone => Everyone};
        false -> undefined
    end.
