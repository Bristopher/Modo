// SPDX-License-Identifier: AGPL-3.0-or-later

use base64::prelude::*;
use thiserror::Error;

const V2_PREFIX: &str = "v2/";

#[derive(Debug, Error, Eq, PartialEq)]
pub enum ExternalPathError {
    #[error("invalid external path")]
    InvalidExternalPath,
    #[error("invalid base64")]
    InvalidBase64,
    #[error("invalid utf-8")]
    InvalidUtf8,
}

pub fn build_external_media_proxy_path(input_url: &str) -> String {
    format!(
        "{V2_PREFIX}{}",
        BASE64_URL_SAFE_NO_PAD.encode(input_url.as_bytes())
    )
}

fn decode_v2(proxy_path: &str) -> Result<String, ExternalPathError> {
    let encoded = proxy_path
        .strip_prefix(V2_PREFIX)
        .ok_or(ExternalPathError::InvalidExternalPath)?;
    if encoded.is_empty() {
        return Err(ExternalPathError::InvalidExternalPath);
    }
    let bytes = BASE64_URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| ExternalPathError::InvalidBase64)?;
    String::from_utf8(bytes).map_err(|_| ExternalPathError::InvalidUtf8)
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

pub fn percent_decode(input: &str, plus_as_space: bool) -> Vec<u8> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let ch = bytes[i];
        if ch == b'%'
            && i + 2 < bytes.len()
            && let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2]))
        {
            out.push((hi << 4) | lo);
            i += 3;
            continue;
        }
        out.push(if plus_as_space && ch == b'+' {
            b' '
        } else {
            ch
        });
        i += 1;
    }
    out
}

pub fn percent_decode_string(input: &str, plus_as_space: bool) -> String {
    String::from_utf8_lossy(&percent_decode(input, plus_as_space)).into_owned()
}

fn legacy_protocol_index(parts: &[&str]) -> Option<usize> {
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if index == 0 && part.contains("%3D") {
            continue;
        }
        let mut chars = part.bytes();
        let first = chars.next()?;
        if !first.is_ascii_alphabetic() {
            continue;
        }
        if chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, b'+' | b'.' | b'-')) {
            return Some(index);
        }
    }
    None
}

fn reconstruct_legacy(proxy_path: &str) -> Result<String, ExternalPathError> {
    let parts: Vec<&str> = proxy_path.split('/').collect();
    let protocol_index =
        legacy_protocol_index(&parts).ok_or(ExternalPathError::InvalidExternalPath)?;
    if protocol_index + 1 >= parts.len() {
        return Err(ExternalPathError::InvalidExternalPath);
    }
    let protocol = parts[protocol_index];
    let host_port = percent_decode_string(parts[protocol_index + 1], false);
    let query_raw = parts[..protocol_index].join("/");
    let path_raw = parts[protocol_index + 2..].join("/");
    let query = percent_decode_string(&query_raw, false);
    let path = percent_decode_string(&path_raw, false);
    Ok(format!(
        "{protocol}://{host_port}/{path}{}{}",
        if query.is_empty() { "" } else { "?" },
        query
    ))
}

pub fn reconstruct_original_url(proxy_path: &str) -> Result<String, ExternalPathError> {
    if proxy_path.starts_with(V2_PREFIX) {
        decode_v2(proxy_path)
    } else {
        reconstruct_legacy(proxy_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v2_path_roundtrip() {
        let path = build_external_media_proxy_path("https://example.com/a b.png?x=1");
        let decoded = reconstruct_original_url(&path).unwrap();
        assert_eq!("https://example.com/a b.png?x=1", decoded);
    }
}
