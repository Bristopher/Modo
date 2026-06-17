// SPDX-License-Identifier: AGPL-3.0-or-later

use super::client::{AdminApiClient, ApiResult};
use super::types::{LimitConfigResponse, LimitConfigUpdateRequest};

impl AdminApiClient {
    pub async fn get_limit_config(&self) -> ApiResult<LimitConfigResponse> {
        self.post("/admin/limit-config/get", Some(&serde_json::json!({})))
            .await
    }

    pub async fn update_limit_config(
        &self,
        request: &LimitConfigUpdateRequest,
    ) -> ApiResult<LimitConfigResponse> {
        self.post_typed("/admin/limit-config/update", request).await
    }
}
