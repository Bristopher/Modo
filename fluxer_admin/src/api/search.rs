// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::generated::types as generated_types;

use super::client::{AdminApiClient, ApiError, ApiResult};
use super::types::{IndexRefreshStatusResponse, RefreshSearchIndexResponse};

impl AdminApiClient {
    pub async fn refresh_search_index(
        &self,
        index_type: &str,
        guild_id: Option<&str>,
    ) -> ApiResult<RefreshSearchIndexResponse> {
        let body = generated_types::RefreshSearchIndexRequest {
            guild_id: guild_id.map(|id| generated_types::SnowflakeType::from(id.to_owned())),
            index_type: generated_types::RefreshSearchIndexRequestIndexType::try_from(index_type)
                .map_err(|e| ApiError::Parse(e.to_string()))?,
            user_id: None,
        };
        let response = self
            .generated()
            .refresh_search_index(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_index_refresh_status(
        &self,
        job_id: &str,
    ) -> ApiResult<IndexRefreshStatusResponse> {
        let body = generated_types::GetIndexRefreshStatusRequest {
            job_id: job_id.to_owned(),
        };
        let response = self
            .generated()
            .get_search_index_refresh_status(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }
}
