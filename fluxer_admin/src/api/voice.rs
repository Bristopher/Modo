// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::generated::types as generated_types;

use super::client::{AdminApiClient, ApiError, ApiResult};
use super::types::{
    CreateVoiceRegionResponse, CreateVoiceServerResponse, DeleteVoiceResponse,
    GetVoiceRegionResponse, GetVoiceServerResponse, ListVoiceRegionsResponse,
    ListVoiceServersResponse, UpdateVoiceRegionResponse, UpdateVoiceServerResponse,
};

impl AdminApiClient {
    pub async fn list_voice_regions(
        &self,
        include_servers: bool,
    ) -> ApiResult<ListVoiceRegionsResponse> {
        let body = generated_types::ListVoiceRegionsRequest {
            include_servers: Some(include_servers),
        };
        let response = self
            .generated()
            .list_voice_regions(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_voice_region(
        &self,
        id: &str,
        include_servers: bool,
    ) -> ApiResult<GetVoiceRegionResponse> {
        let body = generated_types::GetVoiceRegionRequest {
            id: id.to_owned(),
            include_servers: Some(include_servers),
        };
        let response = self
            .generated()
            .get_voice_region(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn create_voice_region(
        &self,
        params: &serde_json::Value,
    ) -> ApiResult<CreateVoiceRegionResponse> {
        let body =
            serde_json::from_value::<generated_types::CreateVoiceRegionRequest>(params.clone())
                .map_err(|e| ApiError::Parse(e.to_string()))?;
        let response = self
            .generated()
            .create_voice_region(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn update_voice_region(
        &self,
        params: &serde_json::Value,
    ) -> ApiResult<UpdateVoiceRegionResponse> {
        let body =
            serde_json::from_value::<generated_types::UpdateVoiceRegionRequest>(params.clone())
                .map_err(|e| ApiError::Parse(e.to_string()))?;
        let response = self
            .generated()
            .update_voice_region(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn delete_voice_region(&self, id: &str) -> ApiResult<DeleteVoiceResponse> {
        let body = generated_types::DeleteVoiceRegionRequest { id: id.to_owned() };
        let response = self
            .generated()
            .delete_voice_region(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn list_voice_servers(&self, region_id: &str) -> ApiResult<ListVoiceServersResponse> {
        let body = generated_types::ListVoiceServersRequest {
            region_id: region_id.to_owned(),
        };
        let response = self
            .generated()
            .list_voice_servers(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn get_voice_server(
        &self,
        region_id: &str,
        server_id: &str,
    ) -> ApiResult<GetVoiceServerResponse> {
        let body = generated_types::GetVoiceServerRequest {
            region_id: region_id.to_owned(),
            server_id: server_id.to_owned(),
        };
        let response = self
            .generated()
            .get_voice_server(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn create_voice_server(
        &self,
        params: &serde_json::Value,
    ) -> ApiResult<CreateVoiceServerResponse> {
        let body =
            serde_json::from_value::<generated_types::CreateVoiceServerRequest>(params.clone())
                .map_err(|e| ApiError::Parse(e.to_string()))?;
        let response = self
            .generated()
            .create_voice_server(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn update_voice_server(
        &self,
        params: &serde_json::Value,
    ) -> ApiResult<UpdateVoiceServerResponse> {
        let body =
            serde_json::from_value::<generated_types::UpdateVoiceServerRequest>(params.clone())
                .map_err(|e| ApiError::Parse(e.to_string()))?;
        let response = self
            .generated()
            .update_voice_server(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }

    pub async fn delete_voice_server(
        &self,
        region_id: &str,
        server_id: &str,
    ) -> ApiResult<DeleteVoiceResponse> {
        let body = generated_types::DeleteVoiceServerRequest {
            region_id: region_id.to_owned(),
            server_id: server_id.to_owned(),
        };
        let response = self
            .generated()
            .delete_voice_server(&body)
            .await
            .map_err(|e| self.generated_error(e))?;
        self.generated_value(response.into_inner())
    }
}
