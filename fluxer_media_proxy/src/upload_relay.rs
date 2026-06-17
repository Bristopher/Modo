// SPDX-License-Identifier: AGPL-3.0-or-later

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, KeyInit, Mac};
use http::Method;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

pub const RELAY_PATH_PREFIX: &str = "/v1/relay/";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenMethod {
    Put,
}

impl TokenMethod {
    pub fn http(self) -> Method {
        match self {
            Self::Put => Method::PUT,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TokenPayload {
    pub b: String,
    pub k: String,
    pub m: TokenMethod,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub u: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ct: Option<String>,
    pub mb: u64,
    pub e: u64,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum TokenError {
    #[error("malformed token")]
    Malformed,
    #[error("bad token encoding")]
    BadEncoding,
    #[error("bad token JSON")]
    BadJson,
    #[error("bad token signature")]
    BadSignature,
    #[error("expired token")]
    Expired,
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum RelayError {
    #[error("missing relay token")]
    MissingToken,
    #[error("invalid relay token")]
    InvalidToken,
    #[error("relay token expired")]
    RelayTokenExpired,
    #[error("wrong bucket")]
    WrongBucket,
    #[error("key mismatch")]
    KeyMismatch,
    #[error("method mismatch")]
    MethodMismatch,
    #[error("part number mismatch")]
    PartNumberMismatch,
    #[error("upload id mismatch")]
    UploadIdMismatch,
    #[error("payload too large")]
    PayloadTooLarge,
    #[error("bad query")]
    BadQuery,
    #[error("client upload failed")]
    ClientUploadFailed,
    #[error("upstream S3 error")]
    UpstreamS3Error,
    #[error("upstream retryable error")]
    UpstreamRetryable,
    #[error("internal relay error")]
    InternalError,
}

static BUFFERED_RETRY_IN_FLIGHT: AtomicU64 = AtomicU64::new(0);
static SPOOL_IN_FLIGHT_BYTES: AtomicU64 = AtomicU64::new(0);

pub fn try_reserve_buffer_budget(needed: u64, ceiling: u64) -> bool {
    if ceiling == 0 || needed > ceiling {
        return false;
    }
    let mut current = BUFFERED_RETRY_IN_FLIGHT.load(Ordering::Acquire);
    loop {
        if current.saturating_add(needed) > ceiling {
            return false;
        }
        match BUFFERED_RETRY_IN_FLIGHT.compare_exchange_weak(
            current,
            current + needed,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return true,
            Err(next) => current = next,
        }
    }
}

pub fn release_buffer_budget(amount: u64) {
    BUFFERED_RETRY_IN_FLIGHT.fetch_sub(amount, Ordering::AcqRel);
}

pub fn try_reserve_spool_budget(needed: u64, ceiling: u64) -> bool {
    if ceiling == 0 || needed > ceiling {
        return false;
    }
    let mut current = SPOOL_IN_FLIGHT_BYTES.load(Ordering::Acquire);
    loop {
        if current.saturating_add(needed) > ceiling {
            return false;
        }
        match SPOOL_IN_FLIGHT_BYTES.compare_exchange_weak(
            current,
            current + needed,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => return true,
            Err(next) => current = next,
        }
    }
}

pub fn release_spool_budget(amount: u64) {
    SPOOL_IN_FLIGHT_BYTES.fetch_sub(amount, Ordering::AcqRel);
}

pub fn spool_in_flight_bytes() -> u64 {
    SPOOL_IN_FLIGHT_BYTES.load(Ordering::Relaxed)
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn encode_token(token: &TokenPayload, secret: &[u8]) -> anyhow::Result<String> {
    let payload = serde_json::to_vec(token)?;
    let encoded_payload = URL_SAFE_NO_PAD.encode(payload);
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(encoded_payload.as_bytes());
    let sig = mac.finalize().into_bytes();
    Ok(format!(
        "{}.{}",
        encoded_payload,
        URL_SAFE_NO_PAD.encode(sig)
    ))
}

pub fn decode_token(raw: &str, secret: &[u8], now_unix: u64) -> Result<TokenPayload, TokenError> {
    let (payload_b64, sig_b64) = raw.split_once('.').ok_or(TokenError::Malformed)?;
    let sig = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| TokenError::BadEncoding)?;
    if sig.len() != 32 {
        return Err(TokenError::BadSignature);
    }
    let mut mac = HmacSha256::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(payload_b64.as_bytes());
    let expected = mac.finalize().into_bytes();
    let mut diff = 0u8;
    for (a, b) in sig.iter().zip(expected.iter()) {
        diff |= a ^ b;
    }
    if diff != 0 {
        return Err(TokenError::BadSignature);
    }
    let payload = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| TokenError::BadEncoding)?;
    let parsed: TokenPayload = serde_json::from_slice(&payload).map_err(|_| TokenError::BadJson)?;
    if now_unix >= parsed.e {
        return Err(TokenError::Expired);
    }
    Ok(parsed)
}

pub fn map_token_error(err: TokenError) -> RelayError {
    match err {
        TokenError::Expired => RelayError::RelayTokenExpired,
        _ => RelayError::InvalidToken,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RelayRequest<'a> {
    pub uploads_bucket: &'a str,
    pub request_key: &'a str,
    pub request_method: &'a Method,
    pub query_upload_id: Option<&'a str>,
    pub query_part_number: Option<u32>,
    pub content_length: Option<u64>,
    pub max_body_bytes: u64,
}

pub fn validate_relay_request(
    token: &TokenPayload,
    request: RelayRequest<'_>,
) -> Result<(), RelayError> {
    if token.b != request.uploads_bucket {
        return Err(RelayError::WrongBucket);
    }
    if token.k != request.request_key {
        return Err(RelayError::KeyMismatch);
    }
    if token.m != TokenMethod::Put || request.request_method != Method::PUT {
        return Err(RelayError::MethodMismatch);
    }
    switch_upload_id(token.u.as_deref(), request.query_upload_id)?;
    switch_part_number(token.p, request.query_part_number)?;
    if let Some(declared) = request.content_length
        && (declared > token.mb || declared > request.max_body_bytes)
    {
        return Err(RelayError::PayloadTooLarge);
    }
    Ok(())
}

fn switch_upload_id(
    token_upload_id: Option<&str>,
    request_upload_id: Option<&str>,
) -> Result<(), RelayError> {
    match (token_upload_id, request_upload_id) {
        (Some(expected), Some(actual)) if expected == actual => Ok(()),
        (Some(_), _) => Err(RelayError::UploadIdMismatch),
        (None, Some(actual)) if !actual.is_empty() => Err(RelayError::UploadIdMismatch),
        _ => Ok(()),
    }
}

fn switch_part_number(
    token_part_number: Option<u32>,
    request_part_number: Option<u32>,
) -> Result<(), RelayError> {
    match (token_part_number, request_part_number) {
        (None, None) => Ok(()),
        (Some(a), Some(b)) if a == b => Ok(()),
        _ => Err(RelayError::PartNumberMismatch),
    }
}

pub fn query_part_number(raw: Option<&str>) -> Result<Option<u32>, RelayError> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    if raw.is_empty() {
        return Err(RelayError::BadQuery);
    }
    raw.parse().map(Some).map_err(|_| RelayError::BadQuery)
}

pub fn is_relay_path(path: &str) -> bool {
    path.starts_with(RELAY_PATH_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token() -> TokenPayload {
        TokenPayload {
            b: "uploads".to_owned(),
            k: "guild/file.bin".to_owned(),
            m: TokenMethod::Put,
            u: Some("upload-id".to_owned()),
            p: Some(7),
            ct: Some("application/octet-stream".to_owned()),
            mb: 100,
            e: 2_000,
        }
    }

    #[test]
    fn token_roundtrip_and_tamper_detection() {
        let secret = [3u8; 32];
        let encoded = encode_token(&token(), &secret).unwrap();
        assert_eq!(token(), decode_token(&encoded, &secret, 1_000).unwrap());
        let mut tampered = encoded.clone();
        tampered.push('x');
        assert_eq!(
            Err(TokenError::BadSignature),
            decode_token(&tampered, &secret, 1_000)
        );
    }

    #[test]
    fn expired_token_rejected() {
        let secret = [3u8; 32];
        let encoded = encode_token(&token(), &secret).unwrap();
        assert_eq!(
            Err(TokenError::Expired),
            decode_token(&encoded, &secret, 2_000)
        );
    }

    #[test]
    fn validates_matching_relay_request() {
        validate_relay_request(
            &token(),
            RelayRequest {
                uploads_bucket: "uploads",
                request_key: "guild/file.bin",
                request_method: &Method::PUT,
                query_upload_id: Some("upload-id"),
                query_part_number: Some(7),
                content_length: Some(99),
                max_body_bytes: 100,
            },
        )
        .unwrap();
    }

    #[test]
    fn validates_relay_mismatches() {
        assert_eq!(
            Err(RelayError::WrongBucket),
            validate_relay_request(
                &token(),
                RelayRequest {
                    uploads_bucket: "cdn",
                    request_key: "guild/file.bin",
                    request_method: &Method::PUT,
                    query_upload_id: Some("upload-id"),
                    query_part_number: Some(7),
                    content_length: Some(99),
                    max_body_bytes: 100,
                }
            )
        );
        assert_eq!(
            Err(RelayError::PayloadTooLarge),
            validate_relay_request(
                &token(),
                RelayRequest {
                    uploads_bucket: "uploads",
                    request_key: "guild/file.bin",
                    request_method: &Method::PUT,
                    query_upload_id: Some("upload-id"),
                    query_part_number: Some(7),
                    content_length: Some(101),
                    max_body_bytes: 100,
                }
            )
        );
        validate_relay_request(
            &token(),
            RelayRequest {
                uploads_bucket: "uploads",
                request_key: "guild/file.bin",
                request_method: &Method::PUT,
                query_upload_id: Some("upload-id"),
                query_part_number: Some(7),
                content_length: None,
                max_body_bytes: 100,
            },
        )
        .unwrap();
    }

    #[test]
    fn buffer_budget_is_bounded() {
        assert!(try_reserve_buffer_budget(4, 8));
        assert!(!try_reserve_buffer_budget(5, 8));
        release_buffer_budget(4);
        assert!(try_reserve_buffer_budget(8, 8));
        release_buffer_budget(8);
    }
}
