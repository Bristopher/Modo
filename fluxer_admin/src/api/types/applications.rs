// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub owner_user_id: String,
    pub owner_username: Option<String>,
    pub owner_global_name: Option<String>,
    pub owner_discriminator: Option<String>,
    pub bot_user_id: Option<String>,
    pub bot_username: Option<String>,
    pub bot_global_name: Option<String>,
    pub bot_discriminator: Option<String>,
    #[serde(default)]
    pub bot_is_public: bool,
    #[serde(default)]
    pub bot_require_code_grant: bool,
    #[serde(default)]
    pub oauth2_redirect_uris: Vec<String>,
    #[serde(default)]
    pub has_client_secret: bool,
    #[serde(default)]
    pub has_bot_token: bool,
    pub bot_token_preview: Option<String>,
    pub bot_token_created_at: Option<String>,
    pub client_secret_created_at: Option<String>,
    #[serde(default)]
    pub version: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LookupApplicationResponse {
    pub application: Option<Application>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListUserApplicationsResponse {
    pub applications: Vec<Application>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicationUpdateResponse {
    pub application: Application,
}
