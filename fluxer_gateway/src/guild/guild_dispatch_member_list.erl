%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(guild_dispatch_member_list).
-typing([eqwalizer]).

-export([
    maybe_broadcast_member_list_update/4
]).

-type event() :: atom().
-type event_data() :: map().
-type guild_state() :: map().
-type user_id() :: integer().
-export_type([event/0, event_data/0, guild_state/0]).

-spec maybe_broadcast_member_list_update(event(), event_data(), guild_state(), guild_state()) ->
    guild_state().
maybe_broadcast_member_list_update(Event, EventData, OldState, UpdatedState) ->
    case guild_dispatch_config:is_member_list_updates_enabled(UpdatedState) of
        true ->
            dispatch_member_list_event(Event, EventData, OldState, UpdatedState);
        false ->
            UpdatedState
    end.

-spec dispatch_member_list_event(event(), event_data(), guild_state(), guild_state()) ->
    guild_state().
dispatch_member_list_event(guild_member_add, EventData, OldState, UpdatedState) ->
    broadcast_member_update(EventData, OldState, UpdatedState);
dispatch_member_list_event(guild_member_remove, EventData, OldState, UpdatedState) ->
    broadcast_member_update(EventData, OldState, UpdatedState);
dispatch_member_list_event(guild_member_update, EventData, OldState, UpdatedState) ->
    broadcast_member_update(EventData, OldState, UpdatedState);
dispatch_member_list_event(guild_role_create, _EventData, _OldState, UpdatedState) ->
    broadcast_all_updates(UpdatedState);
dispatch_member_list_event(guild_role_update, _EventData, _OldState, UpdatedState) ->
    broadcast_all_updates(UpdatedState);
dispatch_member_list_event(guild_role_update_bulk, _EventData, _OldState, UpdatedState) ->
    broadcast_all_updates(UpdatedState);
dispatch_member_list_event(guild_role_delete, _EventData, _OldState, UpdatedState) ->
    broadcast_all_updates(UpdatedState);
dispatch_member_list_event(channel_create, _EventData, _OldState, UpdatedState) ->
    broadcast_all_updates(UpdatedState);
dispatch_member_list_event(channel_delete, _EventData, _OldState, UpdatedState) ->
    broadcast_all_updates(UpdatedState);
dispatch_member_list_event(channel_update, EventData, _OldState, UpdatedState) ->
    broadcast_channel_update(EventData, UpdatedState);
dispatch_member_list_event(channel_update_bulk, EventData, _OldState, UpdatedState) ->
    Channels = maps:get(<<"channels">>, EventData, []),
    lists:foldl(
        fun broadcast_channel_update/2,
        UpdatedState,
        Channels
    );
dispatch_member_list_event(_Event, _FinalData, _OldState, UpdatedState) ->
    UpdatedState.

-spec broadcast_member_update(event_data(), guild_state(), guild_state()) -> guild_state().
broadcast_member_update(EventData, OldState, UpdatedState) ->
    UserId = extract_user_id_from_event(EventData),
    case UserId of
        undefined ->
            UpdatedState;
        _ ->
            {ok, NewState} = guild_member_list:broadcast_member_list_updates(
                UserId, OldState, UpdatedState
            ),
            NewState
    end.

-spec broadcast_all_updates(guild_state()) -> guild_state().
broadcast_all_updates(UpdatedState) ->
    {ok, NewState} = guild_member_list:broadcast_all_member_list_updates(UpdatedState),
    NewState.

-spec broadcast_channel_update(event_data(), guild_state()) -> guild_state().
broadcast_channel_update(EventData, UpdatedState) ->
    ChannelIdBin = maps:get(<<"id">>, EventData, undefined),
    case guild_dispatch_decorate:parse_snowflake(<<"id">>, ChannelIdBin) of
        undefined ->
            UpdatedState;
        ChannelId ->
            {ok, NewState} = guild_member_list:broadcast_member_list_updates_for_channel(
                ChannelId, UpdatedState
            ),
            NewState
    end.

-spec extract_user_id_from_event(event_data()) -> user_id() | undefined.
extract_user_id_from_event(EventData) ->
    MUser = maps:get(<<"user">>, EventData, #{}),
    guild_dispatch_decorate:parse_snowflake(
        <<"user.id">>, maps:get(<<"id">>, MUser, undefined)
    ).

-ifdef(TEST).
-include_lib("eunit/include/eunit.hrl").

extract_user_id_from_event_test() ->
    EventData = #{<<"user">> => #{<<"id">> => <<"42">>}},
    ?assertEqual(42, extract_user_id_from_event(EventData)).

extract_user_id_from_event_missing_test() ->
    ?assertEqual(undefined, extract_user_id_from_event(#{})).

extract_user_id_from_event_invalid_test() ->
    EventData = #{<<"user">> => #{<<"id">> => <<"invalid">>}},
    ?assertEqual(undefined, extract_user_id_from_event(EventData)).

-endif.
