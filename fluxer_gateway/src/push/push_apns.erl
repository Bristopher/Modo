%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(push_apns).
-typing([eqwalizer]).

-export([send/3]).

-spec send(integer(), map(), map()) -> false | {true, map()}.
send(UserId, Subscription, Payload) ->
    case fluxer_gateway_env:get(apns_enabled) of
        true ->
            send_via_api_rpc(UserId, Subscription, Payload);
        _ ->
            false
    end.

-spec send_via_api_rpc(integer(), map(), map()) -> false | {true, map()}.
send_via_api_rpc(UserId, Subscription, Payload) ->
    case extract_subscription(Subscription) of
        {ok, DeviceToken, SubscriptionId, AppId, Environment} ->
            Request = build_apns_request(
                UserId, SubscriptionId, DeviceToken, AppId, Environment, Payload
            ),
            handle_apns_rpc_result(UserId, SubscriptionId, rpc_client:call(Request));
        {error, Reason} ->
            logger:debug("Push: invalid APNs subscription", #{
                user_id => UserId, reason => Reason
            }),
            false
    end.

-spec build_apns_request(integer(), binary(), binary(), binary(), binary(), map()) -> map().
build_apns_request(UserId, SubscriptionId, DeviceToken, AppId, Environment, Payload) ->
    #{
        <<"type">> => <<"send_apns_push">>,
        <<"user_id">> => integer_to_binary(UserId),
        <<"subscription_id">> => SubscriptionId,
        <<"device_token">> => DeviceToken,
        <<"app_id">> => AppId,
        <<"provider_environment">> => Environment,
        <<"payload">> => Payload
    }.

-spec handle_apns_rpc_result(integer(), binary(), {ok, map()} | {error, term()}) ->
    false | {true, map()}.
handle_apns_rpc_result(_UserId, _SubscriptionId, {ok, #{<<"success">> := true}}) ->
    false;
handle_apns_rpc_result(UserId, SubscriptionId, {ok, #{<<"should_delete">> := true}}) ->
    {true, delete_payload(UserId, SubscriptionId)};
handle_apns_rpc_result(_UserId, _SubscriptionId, {ok, Response}) when is_map(Response) ->
    false;
handle_apns_rpc_result(UserId, _SubscriptionId, {error, Reason}) ->
    logger:debug("Push: APNs RPC failed", #{user_id => UserId, reason => Reason}),
    false.

-spec extract_subscription(map()) ->
    {ok, binary(), binary(), binary(), binary()} | {error, term()}.
extract_subscription(Subscription) ->
    DeviceToken = push_utils:normalize_binary(
        maps:get(<<"endpoint">>, Subscription, undefined), undefined
    ),
    SubscriptionId = push_utils:normalize_binary(
        maps:get(<<"subscription_id">>, Subscription, undefined), undefined
    ),
    AppId = push_utils:normalize_binary(
        maps:get(<<"app_id">>, Subscription, <<"stable">>), <<"stable">>
    ),
    DefaultEnvironment = fluxer_gateway_env:get(apns_default_environment),
    Environment = normalize_environment(
        maps:get(<<"provider_environment">>, Subscription, DefaultEnvironment)
    ),
    case {DeviceToken, SubscriptionId, AppId, Environment} of
        {Token, Id, App, Env} when
            is_binary(Token),
            byte_size(Token) > 0,
            is_binary(Id),
            is_binary(App),
            is_binary(Env)
        ->
            {ok, Token, Id, App, Env};
        _ ->
            {error, missing_fields}
    end.

-spec normalize_environment(term()) -> binary().
normalize_environment(Value) ->
    case push_utils:normalize_binary(Value, <<"production">>) of
        <<"development">> -> <<"development">>;
        <<"sandbox">> -> <<"development">>;
        _ -> <<"production">>
    end.

-spec delete_payload(integer(), binary()) -> map().
delete_payload(UserId, SubscriptionId) ->
    #{
        <<"user_id">> => integer_to_binary(UserId),
        <<"subscription_id">> => SubscriptionId
    }.
