// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::api::client::AdminApiClient;
use crate::api::types::{FlashLevel, FlashMessage};
use crate::middleware::auth::AuthContext;
use crate::templates;
use axum::{
    http::HeaderMap,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct BanFormData {
    #[serde(default)]
    pub ip: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub phrase: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub sha256_hex: Option<String>,
    #[serde(default)]
    pub hash_short: Option<String>,
    #[serde(default)]
    pub hashes: Option<String>,
    #[serde(default)]
    pub sha256_list: Option<String>,
    #[serde(default)]
    pub match_subdomains: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub substring: Option<String>,
    #[serde(default)]
    pub audit_log_reason: Option<String>,
    #[serde(default)]
    pub _csrf: Option<String>,
}

pub fn extract_value(form: &BanFormData, field: &str) -> String {
    let v = match field {
        "ip" => form.ip.as_deref(),
        "email" => form.email.as_deref(),
        "domain" => form.domain.as_deref(),
        "phrase" => form.phrase.as_deref(),
        "url" => form.url.as_deref(),
        "sha256_hex" => form.sha256_hex.as_deref(),
        "hash_short" => form.hash_short.as_deref(),
        _ => None,
    };
    v.unwrap_or("").trim().to_owned()
}

pub async fn execute_ban(
    client: &AdminApiClient,
    ban_type: &str,
    action: &str,
    value: &str,
    bulk_hashes: Option<&str>,
    bulk_sha256_list: Option<&str>,
    audit_log_reason: Option<&str>,
) -> (String, String) {
    if (action == "bulk-ban" || action == "bulk-ban-files") && ban_type == "file-sha-bans" {
        let raw_hashes = if action == "bulk-ban-files" {
            bulk_sha256_list
        } else {
            bulk_hashes
        };
        return execute_bulk_ban(client, raw_hashes, audit_log_reason).await;
    }
    if value.is_empty() {
        return ("error".into(), "Value is required".into());
    }
    match action {
        "ban" => execute_single_ban(client, ban_type, value, audit_log_reason).await,
        "unban" => execute_single_unban(client, ban_type, value, audit_log_reason).await,
        "check" => execute_check(client, ban_type, value).await,
        _ => ("error".into(), "Unknown action".into()),
    }
}

async fn execute_bulk_ban(
    client: &AdminApiClient,
    bulk_hashes: Option<&str>,
    audit_log_reason: Option<&str>,
) -> (String, String) {
    let hashes: Vec<String> = bulk_hashes
        .unwrap_or("")
        .split(|c: char| c.is_whitespace() || c == ',' || c == ';')
        .map(|l| l.trim().to_lowercase())
        .filter(|h| h.len() == 64 && h.chars().all(|c| c.is_ascii_hexdigit()))
        .collect();
    if hashes.is_empty() {
        return (
            "error".into(),
            "No valid 64-character hex hashes found in input".into(),
        );
    }
    match client.bulk_ban_file_shas(&hashes, audit_log_reason).await {
        Ok(r) => (
            "success".into(),
            format!("Bulk ban job created: {}", r.job_id),
        ),
        Err(error) => {
            tracing::warn!(%error, "admin API request failed: bulk ban file SHAs");
            ("error".into(), "Failed to enqueue bulk ban job".into())
        }
    }
}

async fn execute_single_ban(
    client: &AdminApiClient,
    ban_type: &str,
    value: &str,
    audit_log_reason: Option<&str>,
) -> (String, String) {
    let success = format!("{value} banned successfully");
    let failure = format!("Failed to ban {value}");
    match ban_type {
        "ip-bans" => ban_action_result(client.ban_ip(value).await, success, failure),
        "email-bans" => ban_action_result(client.ban_email(value).await, success, failure),
        "suspicious-email-domains" => ban_action_result(
            client.add_suspicious_email_domain(value).await,
            success,
            failure,
        ),
        "phrase-bans" => ban_action_result(client.ban_phrase(value).await, success, failure),
        "url-bans" => ban_action_result(client.ban_url(value).await, success, failure),
        "file-sha-bans" => ban_action_result(
            client.ban_file_sha(value, audit_log_reason).await,
            success,
            failure,
        ),
        "avatar-hash-bans" => {
            ban_action_result(client.ban_avatar_hash(value).await, success, failure)
        }
        _ => ("error".into(), "Unknown ban type".into()),
    }
}

async fn execute_single_unban(
    client: &AdminApiClient,
    ban_type: &str,
    value: &str,
    audit_log_reason: Option<&str>,
) -> (String, String) {
    let success = format!("{value} unbanned successfully");
    let failure = format!("Failed to unban {value}");
    match ban_type {
        "ip-bans" => ban_action_result(client.unban_ip(value).await, success, failure),
        "email-bans" => ban_action_result(client.unban_email(value).await, success, failure),
        "suspicious-email-domains" => ban_action_result(
            client.remove_suspicious_email_domain(value).await,
            success,
            failure,
        ),
        "phrase-bans" => ban_action_result(client.unban_phrase(value).await, success, failure),
        "url-bans" => ban_action_result(client.unban_url(value).await, success, failure),
        "file-sha-bans" => ban_action_result(
            client.unban_file_sha(value, audit_log_reason).await,
            success,
            failure,
        ),
        "avatar-hash-bans" => {
            ban_action_result(client.unban_avatar_hash(value).await, success, failure)
        }
        _ => ("error".into(), "Unknown ban type".into()),
    }
}

async fn execute_check(client: &AdminApiClient, ban_type: &str, value: &str) -> (String, String) {
    let result = match ban_type {
        "ip-bans" => client.check_ip_ban(value).await,
        "email-bans" => client.check_email_ban(value).await,
        "suspicious-email-domains" => client.check_suspicious_email_domain(value).await,
        "phrase-bans" => client.check_phrase_ban(value).await,
        "url-bans" => client.check_url_ban(value).await,
        "file-sha-bans" => client.check_file_sha_ban(value).await,
        "avatar-hash-bans" => client.check_avatar_hash_ban(value).await,
        _ => return ("error".into(), "Unknown ban type".into()),
    };
    match result {
        Ok(r) if r.banned => ("info".into(), format!("{value} is banned")),
        Ok(_) => ("info".into(), format!("{value} is NOT banned")),
        Err(error) => {
            tracing::warn!(%error, ban_type, value, "admin API request failed: check ban status");
            ("error".into(), "Error checking ban status".into())
        }
    }
}

fn ban_action_result<T, E: std::fmt::Display>(
    result: Result<T, E>,
    success_message: String,
    error_message: String,
) -> (String, String) {
    match result {
        Ok(_) => ("success".into(), success_message),
        Err(error) => {
            tracing::warn!(%error, "admin API request failed: ban action");
            ("error".into(), error_message)
        }
    }
}

pub fn flash_response(
    config: &crate::config::AdminConfig,
    auth: &AuthContext,
    is_htmx: bool,
    level: &str,
    message: &str,
    ban_cfg: &templates::pages::bans::BanConfig,
    csrf_token: &str,
) -> Response {
    if is_htmx {
        render_inline_flash(level, message)
    } else {
        let flash = to_flash(level, message);
        let markup =
            templates::pages::bans::bans_page(config, auth, ban_cfg, Some(&flash), csrf_token);
        Html(markup.into_string()).into_response()
    }
}

pub fn htmx_flash(level: &str, message: &str, headers: &HeaderMap) -> Response {
    let _ = headers;
    render_inline_flash(level, message)
}

pub fn render_inline_flash(level: &str, message: &str) -> Response {
    let (border, bg, text) = match level {
        "success" => ("border-green-300", "bg-green-50", "text-green-800"),
        "error" => ("border-red-300", "bg-red-50", "text-red-800"),
        _ => ("border-blue-300", "bg-blue-50", "text-blue-800"),
    };
    let markup = maud::html! {
        div class="mb-4 sm:mb-6" {
            div class={"rounded-lg border px-4 py-3 text-sm " (border) " " (bg) " " (text)}
                role="alert" {
                (message)
            }
        }
    };
    Html(markup.into_string()).into_response()
}

pub fn to_flash(level: &str, message: &str) -> FlashMessage {
    FlashMessage {
        level: match level {
            "success" => FlashLevel::Success,
            "error" => FlashLevel::Error,
            _ => FlashLevel::Info,
        },
        message: message.to_owned(),
    }
}

pub fn custom_flash(
    config: &crate::config::AdminConfig,
    auth: &AuthContext,
    is_htmx: bool,
    level: &str,
    message: &str,
    csrf_token: &str,
    page_type: &str,
) -> Response {
    if is_htmx {
        return render_inline_flash(level, message);
    }
    let flash = to_flash(level, message);
    let markup = match page_type {
        "url-domain" => templates::pages::url_domain_bans::url_domain_bans_page(
            config,
            auth,
            Some(&flash),
            csrf_token,
        ),
        _ => templates::pages::profile_substring_bans::profile_substring_bans_page(
            config,
            auth,
            Some(&flash),
            csrf_token,
        ),
    };
    Html(markup.into_string()).into_response()
}
