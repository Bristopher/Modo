// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

pub const SERVICE_NAME: &str = "snowflakes";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum SnowflakeRequest {
    GenerateBatch {
        count: u32,
        #[serde(default)]
        routing_key: Option<String>,
    },
}

impl SnowflakeRequest {
    pub fn routing_key(&self) -> Option<&str> {
        match self {
            Self::GenerateBatch { routing_key, .. } => routing_key
                .as_deref()
                .map(str::trim)
                .filter(|key| !key.is_empty()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnowflakeResponse {
    pub ids: Vec<String>,
}
