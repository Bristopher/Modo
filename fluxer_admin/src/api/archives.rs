// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::generated::types as generated_types;

use super::client::{AdminApiClient, ApiError, ApiResult};
use super::types::{Archive, ArchiveDownloadUrlResponse, ListArchivesResponse};

impl AdminApiClient {
    pub async fn trigger_user_archive(
        &self,
        user_id: &str,
        include_attachments: bool,
    ) -> ApiResult<Archive> {
        let body = generated_types::TriggerUserArchiveRequest {
            include_attachments: include_attachments.then_some(true),
            user_id: snowflake(user_id),
        };
        let response = self
            .generated()
            .trigger_user_archive(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn trigger_guild_archive(
        &self,
        guild_id: &str,
        include_attachments: bool,
    ) -> ApiResult<Archive> {
        let body = generated_types::TriggerGuildArchiveRequest {
            guild_id: snowflake(guild_id),
            include_attachments: include_attachments.then_some(true),
        };
        let response = self
            .generated()
            .trigger_guild_archive(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn list_archives(
        &self,
        subject_type: &str,
        subject_id: Option<&str>,
        include_expired: bool,
        requested_by: Option<&str>,
    ) -> ApiResult<ListArchivesResponse> {
        let body = generated_types::ListArchivesRequest {
            include_expired: Some(include_expired),
            limit: None,
            requested_by: requested_by.map(snowflake),
            subject_id: subject_id.map(snowflake),
            subject_type: Some(
                generated_types::ListArchivesRequestSubjectType::try_from(subject_type)
                    .map_err(|e| ApiError::Parse(e.to_string()))?,
            ),
        };
        let response = self
            .generated()
            .list_archives(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_archive_download_url(
        &self,
        subject_type: &str,
        subject_id: &str,
        archive_id: &str,
    ) -> ApiResult<ArchiveDownloadUrlResponse> {
        let response = self
            .generated()
            .get_archive_download_url(subject_type, subject_id, archive_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }
}

fn snowflake(value: &str) -> generated_types::SnowflakeType {
    generated_types::SnowflakeType::from(value.to_owned())
}
