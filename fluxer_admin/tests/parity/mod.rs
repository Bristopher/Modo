// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod api_fixtures;
pub mod capture;
pub mod html_normalizer;
pub mod reference_worktree;
pub mod servers;

use std::env;

pub const TS_REFERENCE_COMMIT: &str = "4748f2f1e6589c325fca6391d6df3e3c3f6a0345^";
pub const PARITY_RUN_ENV: &str = "FLUXER_ADMIN_PARITY_RUN";
pub const PUBLIC_ROUTES_ENV: &str = "FLUXER_ADMIN_PARITY_PUBLIC_ROUTES";
pub const PROTECTED_ROUTES_ENV: &str = "FLUXER_ADMIN_PARITY_PROTECTED_ROUTES";
pub const TS_WORKTREE_ENV: &str = "FLUXER_ADMIN_PARITY_TS_WORKTREE";
pub const TS_WORKTREE_ROOT_ENV: &str = "FLUXER_ADMIN_PARITY_WORKTREE_ROOT";
pub const SKIP_TS_PREPARE_ENV: &str = "FLUXER_ADMIN_PARITY_SKIP_TS_PREPARE";
pub const TEST_ADMIN_SECRET: &str = "test-admin-secret";
pub const TEST_ADMIN_USER_ID: &str = "1130650140672000000";
pub const TEST_ACCESS_TOKEN: &str = "parity-access-token";

pub fn env_flag(name: &str) -> bool {
    env::var(name)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

pub fn route_list_from_env(name: &str, default: &[&str]) -> Vec<String> {
    match env::var(name) {
        Ok(value) => value
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| {
                if value.starts_with('/') {
                    value.to_owned()
                } else {
                    format!("/{value}")
                }
            })
            .collect(),
        Err(_) => default.iter().map(|route| (*route).to_owned()).collect(),
    }
}
