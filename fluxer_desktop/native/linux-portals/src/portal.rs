// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

pub const REQUEST_INTERFACE: &str = "org.freedesktop.portal.Request";

pub fn request_path(unique_bus_name: &str, handle_token: &str) -> String {
    let trimmed = unique_bus_name.strip_prefix(':').unwrap_or(unique_bus_name);
    let mut out = String::with_capacity(40 + trimmed.len() + handle_token.len());
    out.push_str("/org/freedesktop/portal/desktop/request/");
    for ch in trimmed.chars() {
        out.push(if ch == '.' { '_' } else { ch });
    }
    out.push('/');
    out.push_str(handle_token);
    out
}

static TOKEN_SEQ: AtomicU64 = AtomicU64::new(1);

pub fn mint_token(prefix: &str) -> String {
    let seq = TOKEN_SEQ.fetch_add(1, Ordering::Relaxed);
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    format!("{prefix}_{ms:x}_{seq:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_path_sanitizes_unique_bus_name() {
        assert_eq!(
            request_path(":1.42", "fluxer_fc_open_1"),
            "/org/freedesktop/portal/desktop/request/1_42/fluxer_fc_open_1"
        );
    }

    #[test]
    fn mint_token_is_distinct_and_prefixed() {
        let a = mint_token("fluxer_fc_open");
        let b = mint_token("fluxer_fc_open");
        assert_ne!(a, b);
        assert!(a.starts_with("fluxer_fc_open_"));
    }
}
