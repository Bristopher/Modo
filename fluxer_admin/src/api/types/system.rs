// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildMemoryStatsResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ReloadAllGuildsResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeStatsResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GatewayVoiceStateCountsResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}
