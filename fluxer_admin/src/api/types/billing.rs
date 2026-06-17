// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BillingOverview {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentListResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PaymentMethodListResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InvoiceListResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RefundCancelResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}
