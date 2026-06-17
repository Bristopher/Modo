// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Archive {
    pub archive_id: String,
    pub subject_type: String,
    pub subject_id: String,
    pub requested_by: String,
    pub requested_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub failed_at: Option<String>,
    pub file_size: Option<String>,
    pub progress_percent: f64,
    pub progress_step: Option<String>,
    pub error_message: Option<String>,
    pub download_url_expires_at: Option<String>,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ListArchivesResponse {
    pub archives: Vec<Archive>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArchiveDownloadUrlResponse {
    #[serde(rename = "downloadUrl")]
    pub download_url: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}
