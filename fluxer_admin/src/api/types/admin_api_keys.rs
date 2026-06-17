// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateAdminApiKeyResponse {
    pub key_id: String,
    pub key: String,
    pub name: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    #[serde(default)]
    pub acls: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListAdminApiKeyEntry {
    pub key_id: String,
    pub name: String,
    #[serde(default)]
    pub acls: Vec<String>,
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub expires_at: Option<String>,
    pub created_by_user_id: String,
}
