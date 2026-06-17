// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveryPendingApplication {
    pub guild_id: String,
    pub guild_name: Option<String>,
    pub guild_icon: Option<String>,
    pub guild_owner_id: Option<String>,
    pub guild_owner_username: Option<String>,
    pub guild_owner_global_name: Option<String>,
    pub guild_owner_discriminator: Option<String>,
    pub guild_member_count: Option<u64>,
    pub guild_nsfw_level: Option<i32>,
    #[serde(default)]
    pub guild_features: Vec<String>,
    pub description: Option<String>,
    pub category_type: Option<i32>,
    pub primary_language: Option<String>,
    #[serde(default)]
    pub custom_tags: Vec<String>,
    pub member_count: Option<u64>,
    pub applied_at: Option<String>,
    pub category: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveryListedGuild {
    pub guild_id: String,
    pub guild_name: Option<String>,
    pub guild_icon: Option<String>,
    pub guild_owner_id: Option<String>,
    pub guild_owner_username: Option<String>,
    pub guild_owner_global_name: Option<String>,
    pub guild_owner_discriminator: Option<String>,
    pub guild_member_count: Option<u64>,
    pub guild_nsfw_level: Option<i32>,
    #[serde(default)]
    pub guild_features: Vec<String>,
    pub description: Option<String>,
    pub category_type: Option<i32>,
    pub primary_language: Option<String>,
    #[serde(default)]
    pub custom_tags: Vec<String>,
    pub member_count: Option<u64>,
    pub applied_at: Option<String>,
    pub approved_at: Option<String>,
    pub listed_at: Option<String>,
    pub category: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveryApplicationResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}
