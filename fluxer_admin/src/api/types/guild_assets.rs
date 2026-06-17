// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildAssetItem {
    pub id: String,
    pub name: String,
    pub animated: bool,
    pub creator_id: String,
    pub media_url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListGuildEmojisResponse {
    pub guild_id: String,
    pub emojis: Vec<GuildAssetItem>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListGuildStickersResponse {
    pub guild_id: String,
    pub stickers: Vec<GuildAssetItem>,
}
