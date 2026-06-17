// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{invariant_text::SWISH_PAYMENT_MESSAGE, request_context::AppState};
use axum::{
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use moka::{future::Cache, policy::EvictionPolicy};
use serde::Deserialize;

const SWISH_QR_ENDPOINT: &str = "https://mpc.getswish.net/qrg-swish/api/v1/prefilled";
const PAYEE: &str = "1232376820";
const QR_SIZE: u32 = 300;
const MIN_AMOUNT_SEK: u32 = 1;
const MAX_AMOUNT_SEK: u32 = 999_999;
const DEFAULT_AMOUNT_SEK: u32 = 50;
const CACHE_MAX_ENTRIES: u64 = 256;

#[derive(Clone, Debug)]
pub struct SwishQrCache {
    entries: Cache<String, SwishQrEntry>,
}

#[derive(Clone, Debug)]
struct SwishQrEntry {
    body: Vec<u8>,
    content_type: String,
}

#[derive(Deserialize)]
pub struct SwishQrQuery {
    amount: Option<String>,
}

impl SwishQrCache {
    pub fn new() -> Self {
        Self {
            entries: Cache::builder()
                .max_capacity(CACHE_MAX_ENTRIES)
                .eviction_policy(EvictionPolicy::lru())
                .build(),
        }
    }

    async fn get(&self, key: &str) -> Option<SwishQrEntry> {
        self.entries.get(key).await
    }

    async fn insert(&self, key: String, entry: SwishQrEntry) {
        self.entries.insert(key, entry).await;
    }
}

impl Default for SwishQrCache {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn swish_qr(
    State(state): State<AppState>,
    Query(query): Query<SwishQrQuery>,
) -> Response {
    let Some(amount) = parse_amount(query.amount.as_deref()) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let cache_key = format!("v1:amount={amount}");
    if let Some(entry) = state.swish_qr_cache.get(&cache_key).await {
        return swish_response(entry);
    }
    let entry = match fetch_swish_qr(&state.http_client, amount).await {
        Ok(entry) => entry,
        Err(error) => {
            tracing::warn!(?error, amount, "failed to fetch Swish QR code");
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };
    state.swish_qr_cache.insert(cache_key, entry.clone()).await;
    swish_response(entry)
}

fn parse_amount(raw: Option<&str>) -> Option<u32> {
    let Some(raw) = raw else {
        return Some(DEFAULT_AMOUNT_SEK);
    };
    if raw.is_empty() || raw.len() > 7 || !raw.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let amount = raw.parse::<u32>().ok()?;
    if (MIN_AMOUNT_SEK..=MAX_AMOUNT_SEK).contains(&amount) {
        Some(amount)
    } else {
        None
    }
}

async fn fetch_swish_qr(client: &reqwest::Client, amount: u32) -> anyhow::Result<SwishQrEntry> {
    let response = client
        .post(SWISH_QR_ENDPOINT)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "image/svg+xml,application/json")
        .json(&serde_json::json!({
            "format": "svg",
            "size": QR_SIZE,
            "payee": {"value": PAYEE, "editable": false},
            "amount": {"value": amount, "editable": true},
            "message": {"value": SWISH_PAYMENT_MESSAGE, "editable": false},
        }))
        .send()
        .await?;
    if !response.status().is_success() {
        anyhow::bail!("Swish QR upstream returned {}", response.status());
    }
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("image/svg+xml")
        .to_owned();
    let body = response.bytes().await?.to_vec();
    Ok(SwishQrEntry { body, content_type })
}

fn swish_response(entry: SwishQrEntry) -> Response {
    let mut response = entry.body.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(&entry.content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("image/svg+xml")),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=86400, immutable"),
    );
    response
}
