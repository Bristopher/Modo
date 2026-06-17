%% SPDX-License-Identifier: AGPL-3.0-or-later

-module(list_ops).
-typing([eqwalizer]).

-export([
    replace_by_id/3,
    remove_by_id/2,
    replace_by_user_id/3,
    remove_by_user_id/2,
    bulk_update/2,
    extract_user_id/1
]).

-export_type([item/0, id/0, item_list/0]).

-type item() :: map() | term().
-type id() :: binary() | integer().
-type item_list() :: [item()].

-spec replace_by_id(term(), id(), item()) -> item_list().
replace_by_id(Items, Id, NewItem) when is_list(Items) ->
    lists:map(
        fun(Item) -> replace_item_by_id(Item, Id, NewItem) end,
        Items
    );
replace_by_id(_, _, _) ->
    [].

-spec remove_by_id(term(), id()) -> item_list().
remove_by_id(Items, Id) when is_list(Items) ->
    lists:filter(fun(Item) -> keep_item_with_id(Item, Id) end, Items);
remove_by_id(_, _) ->
    [].

-spec replace_by_user_id(term(), integer(), item()) -> item_list().
replace_by_user_id(Items, UserId, NewItem) when is_list(Items), is_integer(UserId) ->
    lists:map(
        fun(Item) -> replace_item_by_user_id(Item, UserId, NewItem) end,
        Items
    );
replace_by_user_id(_, _, _) ->
    [].

-spec remove_by_user_id(term(), integer()) -> item_list().
remove_by_user_id(Items, UserId) when is_list(Items), is_integer(UserId) ->
    lists:filter(fun(Item) -> keep_item_with_user_id(Item, UserId) end, Items);
remove_by_user_id(_, _) ->
    [].

-spec bulk_update(term(), term()) -> item_list().
bulk_update(Items, Updates) when is_list(Items), is_list(Updates) ->
    UpdateMap = lists:foldl(fun add_update_item/2, #{}, Updates),
    lists:map(fun(Item) -> apply_update_item(Item, UpdateMap) end, Items);
bulk_update(Items, _) when is_list(Items) ->
    Items;
bulk_update(_, _) ->
    [].

-spec extract_user_id(map() | term()) -> pos_integer() | undefined.
extract_user_id(Item) ->
    UserMap = map_utils:ensure_map(map_utils:get_safe(Item, <<"user">>, #{})),
    type_conv:extract_id(UserMap, <<"id">>).

-spec replace_item_by_id(item(), id(), item()) -> item().
replace_item_by_id(Item, Id, NewItem) when is_map(Item) ->
    case maps:get(<<"id">>, Item, undefined) of
        Id -> NewItem;
        _ -> Item
    end;
replace_item_by_id(Item, _Id, _NewItem) ->
    Item.

-spec keep_item_with_id(item(), id()) -> boolean().
keep_item_with_id(Item, Id) when is_map(Item) ->
    maps:get(<<"id">>, Item, undefined) =/= Id;
keep_item_with_id(_Item, _Id) ->
    true.

-spec replace_item_by_user_id(item(), integer(), item()) -> item().
replace_item_by_user_id(Item, UserId, NewItem) when is_map(Item) ->
    case extract_user_id(Item) =:= UserId of
        true -> NewItem;
        false -> Item
    end;
replace_item_by_user_id(Item, _UserId, _NewItem) ->
    Item.

-spec keep_item_with_user_id(item(), integer()) -> boolean().
keep_item_with_user_id(Item, UserId) when is_map(Item) ->
    extract_user_id(Item) =/= UserId;
keep_item_with_user_id(_Item, _UserId) ->
    true.

-spec add_update_item(item(), map()) -> map().
add_update_item(Item, Acc) when is_map(Item) ->
    case maps:get(<<"id">>, Item, undefined) of
        undefined -> Acc;
        ItemId -> Acc#{ItemId => Item}
    end;
add_update_item(_Item, Acc) ->
    Acc.

-spec apply_update_item(item(), map()) -> item().
apply_update_item(Item, UpdateMap) when is_map(Item) ->
    ItemId = maps:get(<<"id">>, Item, undefined),
    case maps:get(ItemId, UpdateMap, undefined) of
        undefined -> Item;
        UpdatedItem -> UpdatedItem
    end;
apply_update_item(Item, _UpdateMap) ->
    Item.
