// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::config::AdminConfig;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub config: AdminConfig,
    pub http_client: reqwest::Client,
}

impl AppState {
    pub fn new(config: AdminConfig) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent(format!("FluxerAdmin/{} (Rust)", config.build_version))
            .build()
            .expect("failed to create HTTP client");
        Self {
            inner: Arc::new(AppStateInner {
                config,
                http_client,
            }),
        }
    }

    pub fn config(&self) -> &AdminConfig {
        &self.inner.config
    }

    pub fn http_client(&self) -> &reqwest::Client {
        &self.inner.http_client
    }
}

impl axum::extract::FromRef<AppState> for AdminConfig {
    fn from_ref(state: &AppState) -> Self {
        state.inner.config.clone()
    }
}
