// SPDX-License-Identifier: AGPL-3.0-or-later

use super::html_normalizer;
use reqwest::{Client, redirect::Policy};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizedResponse {
    pub status: u16,
    pub content_type: Option<String>,
    pub location: Option<String>,
    pub body: String,
}

pub fn capture_client() -> Result<Client, String> {
    Client::builder()
        .redirect(Policy::none())
        .build()
        .map_err(|error| format!("failed to build capture client: {error}"))
}

pub async fn compare_route(
    client: &Client,
    route: &str,
    ts_base_url: &str,
    rust_base_url: &str,
    cookie: Option<&str>,
) -> Result<(), String> {
    let ts = fetch_route(client, ts_base_url, route, cookie).await?;
    let rust = fetch_route(client, rust_base_url, route, cookie).await?;
    if ts == rust {
        return Ok(());
    }
    Err(format!(
        "parity mismatch for {route}\nTS: {ts:#?}\nRust: {rust:#?}\nfirst body diff: {}",
        first_body_diff(&ts.body, &rust.body)
    ))
}

pub async fn fetch_route(
    client: &Client,
    base_url: &str,
    route: &str,
    cookie: Option<&str>,
) -> Result<NormalizedResponse, String> {
    let url = format!("{}{}", base_url.trim_end_matches('/'), route);
    let mut request = client.get(&url);
    if let Some(cookie) = cookie {
        request = request.header(reqwest::header::COOKIE, cookie);
    }
    let response = request
        .send()
        .await
        .map_err(|error| format!("failed to fetch {url}: {error}"))?;
    let status = response.status().as_u16();
    let headers = response.headers().clone();
    let content_type = headers
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| html_normalizer::normalize_header_value("content-type", value));
    let location = headers
        .get(reqwest::header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .map(|value| html_normalizer::normalize_header_value("location", value));
    let raw_body = response
        .text()
        .await
        .map_err(|error| format!("failed to read body for {url}: {error}"))?;
    let body = html_normalizer::normalize_body(content_type.as_deref(), &raw_body);
    Ok(NormalizedResponse {
        status,
        content_type,
        location,
        body,
    })
}

fn first_body_diff(left: &str, right: &str) -> String {
    let left_chars = left.chars().collect::<Vec<_>>();
    let right_chars = right.chars().collect::<Vec<_>>();
    let max_len = left_chars.len().max(right_chars.len());
    for index in 0..max_len {
        if left_chars.get(index) != right_chars.get(index) {
            let left_preview = preview_from(&left_chars, index);
            let right_preview = preview_from(&right_chars, index);
            return format!("at char {index}: left `{left_preview}`, right `{right_preview}`");
        }
    }
    "bodies differ but no character diff was found".to_owned()
}

fn preview_from(chars: &[char], start: usize) -> String {
    chars
        .iter()
        .skip(start.saturating_sub(20))
        .take(80)
        .collect::<String>()
}
