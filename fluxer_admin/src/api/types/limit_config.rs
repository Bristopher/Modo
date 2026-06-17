// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LimitConfigResponse {
    pub limit_config: LimitConfigSnapshot,
    pub limit_config_json: String,
    pub self_hosted: Option<bool>,
    pub defaults: std::collections::BTreeMap<String, std::collections::BTreeMap<String, u64>>,
    pub metadata: std::collections::BTreeMap<String, LimitKeyMetadata>,
    pub categories: Option<std::collections::BTreeMap<String, String>>,
    pub limit_keys: Vec<String>,
    pub bounds: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LimitConfigSnapshot {
    #[serde(default, rename = "traitDefinitions")]
    pub trait_definitions: Vec<String>,
    pub rules: Vec<LimitRule>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LimitRule {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filters: Option<LimitRuleFilters>,
    pub limits: std::collections::BTreeMap<String, u64>,
    #[serde(default, rename = "modifiedFields", skip_serializing)]
    pub modified_fields: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LimitRuleFilters {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<String>,
    #[serde(
        default,
        rename = "guildFeatures",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub guild_features: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LimitKeyMetadata {
    pub key: String,
    pub label: String,
    pub description: String,
    pub category: String,
    pub scope: String,
    #[serde(rename = "isToggle")]
    pub is_toggle: bool,
    pub unit: Option<String>,
    pub min: Option<u64>,
    pub max: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct LimitConfigUpdateRequest {
    pub limit_config: LimitConfigSnapshot,
}
