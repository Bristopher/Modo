// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::generated::types as generated_types;

use super::client::{AdminApiClient, ApiError, ApiResult};
use super::types::{
    ListReportsResponse, ReportEntry, ResolveReportResponse, SearchReportsResponse,
};

impl AdminApiClient {
    pub async fn list_reports(
        &self,
        status: Option<i32>,
        limit: u32,
        offset: Option<u32>,
    ) -> ApiResult<ListReportsResponse> {
        let body = generated_types::ListReportsRequest {
            limit: Some(
                crate::api::generated::nonzero_u32(limit, "limit").map_err(ApiError::Parse)?,
            ),
            offset: offset.map(i64::from),
            status: status
                .map(generated_types::ReportStatus::try_from)
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
        };
        let response = self
            .generated()
            .list_reports(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_report(&self, report_id: &str) -> ApiResult<ReportEntry> {
        let response = self
            .generated()
            .get_report(report_id)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn resolve_report(
        &self,
        report_id: &str,
        public_comment: Option<&str>,
        audit_log_reason: Option<&str>,
    ) -> ApiResult<ResolveReportResponse> {
        let body = generated_types::ResolveReportRequest {
            public_comment: public_comment.map(std::borrow::ToOwned::to_owned),
            report_id: generated_types::SnowflakeType::from(report_id.to_owned()),
        };
        self.post_typed_with_reason("/admin/reports/resolve", &body, audit_log_reason)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn search_reports(
        &self,
        query: Option<&str>,
        status: Option<i32>,
        report_type: Option<i32>,
        category: Option<&str>,
        reporter_id: Option<&str>,
        reported_user_id: Option<&str>,
        reported_guild_id: Option<&str>,
        reported_channel_id: Option<&str>,
        guild_context_id: Option<&str>,
        resolved_by_admin_id: Option<&str>,
        sort_by: Option<&str>,
        sort_order: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> ApiResult<SearchReportsResponse> {
        let body = generated_types::SearchReportsRequest {
            category: nonempty_string(category),
            guild_context_id: nonempty_snowflake(guild_context_id),
            limit: Some(
                crate::api::generated::nonzero_u32(limit, "limit").map_err(ApiError::Parse)?,
            ),
            offset: Some(i64::from(offset)),
            query: nonempty_string(query),
            report_type: report_type
                .map(generated_types::ReportType::try_from)
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
            reported_channel_id: nonempty_snowflake(reported_channel_id),
            reported_guild_id: nonempty_snowflake(reported_guild_id),
            reported_user_id: nonempty_snowflake(reported_user_id),
            reporter_id: nonempty_snowflake(reporter_id),
            resolved_by_admin_id: nonempty_snowflake(resolved_by_admin_id),
            sort_by: sort_by
                .map(generated_types::SearchReportsRequestSortBy::try_from)
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
            sort_order: sort_order
                .map(generated_types::SearchReportsRequestSortOrder::try_from)
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
            status: status
                .map(generated_types::ReportStatus::try_from)
                .transpose()
                .map_err(|e| ApiError::Parse(e.to_string()))?,
        };
        let response = self
            .generated()
            .search_reports(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        let response = response.into_inner();
        Ok(SearchReportsResponse {
            reports: self.generated_value(response.reports)?,
            total: response.total as u64,
            offset: response.offset as u64,
            limit: response.limit as u64,
        })
    }

    pub async fn search_reports_by_reporter(
        &self,
        reporter_id: &str,
        limit: u32,
        offset: u32,
    ) -> ApiResult<SearchReportsResponse> {
        self.search_reports(
            None,
            None,
            None,
            None,
            Some(reporter_id),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            limit,
            offset,
        )
        .await
    }

    pub async fn search_reports_by_reported_user(
        &self,
        reported_user_id: &str,
        limit: u32,
        offset: u32,
    ) -> ApiResult<SearchReportsResponse> {
        self.search_reports(
            None,
            None,
            None,
            None,
            None,
            Some(reported_user_id),
            None,
            None,
            None,
            None,
            None,
            None,
            limit,
            offset,
        )
        .await
    }
}

fn nonempty_string(value: Option<&str>) -> Option<String> {
    value
        .filter(|value| !value.is_empty())
        .map(std::borrow::ToOwned::to_owned)
}

fn nonempty_snowflake(value: Option<&str>) -> Option<generated_types::SnowflakeType> {
    nonempty_string(value).map(generated_types::SnowflakeType::from)
}
