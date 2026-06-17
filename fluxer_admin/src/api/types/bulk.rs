// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BulkJobResponse {
    pub job_id: Option<String>,
    pub status: Option<String>,
    pub total: Option<u64>,
    pub processed: Option<u64>,
    pub failed: Option<u64>,
}
