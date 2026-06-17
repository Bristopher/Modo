// SPDX-License-Identifier: AGPL-3.0-or-later

use super::client::{AdminApiClient, ApiResult};
use super::types::{Application, ApplicationUpdateResponse, LookupApplicationResponse};
use serde::Serialize;

#[derive(Serialize)]
struct LookupApplicationRequest<'a> {
    application_id: &'a str,
}

#[derive(Serialize)]
struct ListUserApplicationsRequest<'a> {
    user_id: &'a str,
}

#[derive(Serialize)]
struct TransferApplicationOwnershipRequest<'a> {
    application_id: &'a str,
    new_owner_id: &'a str,
}

impl AdminApiClient {
    pub async fn lookup_application(&self, application_id: &str) -> ApiResult<Option<Application>> {
        let body = LookupApplicationRequest { application_id };
        let resp: LookupApplicationResponse =
            self.post_typed("/admin/applications/lookup", &body).await?;
        Ok(resp.application)
    }

    pub async fn list_user_applications(&self, user_id: &str) -> ApiResult<Vec<Application>> {
        let body = ListUserApplicationsRequest { user_id };
        let resp: super::types::ListUserApplicationsResponse = self
            .post_typed("/admin/applications/list-by-owner", &body)
            .await?;
        Ok(resp.applications)
    }

    pub async fn transfer_application_ownership(
        &self,
        application_id: &str,
        new_owner_id: &str,
    ) -> ApiResult<ApplicationUpdateResponse> {
        let body = TransferApplicationOwnershipRequest {
            application_id,
            new_owner_id,
        };
        self.post_typed("/admin/applications/transfer-ownership", &body)
            .await
    }
}
