// SPDX-License-Identifier: AGPL-3.0-or-later

#[path = "parity/mod.rs"]
mod parity_support;

use parity_support::{
    PARITY_RUN_ENV, PROTECTED_ROUTES_ENV, PUBLIC_ROUTES_ENV, TEST_ACCESS_TOKEN, TEST_ADMIN_SECRET,
    TEST_ADMIN_USER_ID, api_fixtures, capture, env_flag, html_normalizer, reference_worktree,
    route_list_from_env, servers,
};
use std::{error::Error, io};

const DEFAULT_PUBLIC_ROUTES: &[&str] = &["/_health", "/robots.txt", "/static/app.css", "/login"];
const DEFAULT_PROTECTED_ROUTES: &[&str] = &[
    "/dashboard",
    "/users?q=Parity",
    "/guilds?q=Parity",
    "/guilds/1600000000000000001",
    "/reports",
    "/reports/1700000000000000001",
];

#[test]
fn html_normalizer_canonicalizes_attribute_order_and_csrf_values() {
    let left = r#"<form><input value="aaaaaaaa" name="_csrf" type="hidden"><svg><line x1="1" x2="2"></line></svg><a class="b" href="/static/app.css?v=123" id="x">Open</a></form>"#;
    let right = r#"<form><input type="hidden" name="_csrf" value="bbbbbbbb"/><svg><line x2="2" x1="1"/></svg><a id="x" href="/static/app.css?v=456" class="b">Open</a></form>"#;
    assert_eq!(
        html_normalizer::normalize_html(left),
        html_normalizer::normalize_html(right)
    );
}

#[test]
fn text_normalizer_replaces_ports_assets_query_values_and_cookie_tokens() {
    let raw = "http://127.0.0.1:31987/auth/start?state=abc&next=/static/app.css?v=dev admin_session=abcdef; csrf_token=12345; oauth_state=zz";
    let normalized = html_normalizer::normalize_text(raw);
    assert!(normalized.contains("127.0.0.1:__PORT__"));
    assert!(normalized.contains("state=__OAUTH_STATE__"));
    assert!(normalized.contains("/static/app.css"));
    assert!(normalized.contains("admin_session=__SESSION__"));
    assert!(normalized.contains("csrf_token=__CSRF_COOKIE__"));
    assert!(normalized.contains("oauth_state=__OAUTH_STATE__"));
}

#[test]
fn text_normalizer_replaces_script_csrf_values() {
    let raw = r#"<script>(function(){var csrf="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";})()</script>"#;
    let normalized = html_normalizer::normalize_text(raw);
    assert!(normalized.contains(r#"var csrf="__CSRF_TOKEN__""#));
}

#[test]
fn html_normalizer_allows_intentional_rust_markup_fixes() {
    let ts = r#"<html><head></head><body><div class="flex flex-col gap-8 items-stretch"></div><div class="flex flex-col gap-4 items-stretch"></div><script defer>window.__fluxerDrawerInit = true;</script><aside data-drawer-panel="user-peek" aria-hidden="true"></aside></body></html>"#;
    let rust = r#"<!DOCTYPE html><html><head><script src="/static/htmx.min.js?t=parity" defer></script></head><body hx-boost="true"><div class="flex flex-col gap-8 items-center"></div><div id="users-results" class="flex flex-col gap-4 items-stretch"></div><script defer>document.body.addEventListener('showFlash', function () {});</script><script defer>window.__adminCopyToClipboard = function () {};</script><aside id="user-peek" data-drawer-panel="user-peek" popover="auto"></aside></body></html>"#;
    assert_eq!(
        html_normalizer::normalize_html(ts),
        html_normalizer::normalize_html(rust)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn rust_admin_fixture_routes_cover_default_protected_routes() -> Result<(), Box<dyn Error>> {
    let api_server = api_fixtures::ApiFixtureServer::start_default()
        .await
        .map_err(test_error)?;
    let rust_admin = servers::start_rust_admin(api_server.base_url())
        .await
        .map_err(test_error)?;
    let client = capture::capture_client().map_err(test_error)?;
    let session = fluxer_admin::session::create_session(
        TEST_ADMIN_USER_ID,
        TEST_ACCESS_TOKEN,
        TEST_ADMIN_SECRET,
    );
    let session_cookie = format!("{}={session}", fluxer_admin::session::SESSION_COOKIE_NAME);
    let cases = [
        ("/dashboard", 302, Some("/users"), None),
        ("/users?q=Parity", 200, None, Some("Parity User")),
        ("/guilds?q=Parity", 200, None, Some("Parity Guild")),
        (
            "/guilds/1600000000000000001",
            200,
            None,
            Some("Parity Guild"),
        ),
        ("/reports", 200, None, Some("1700000000000000001")),
        (
            "/reports/1700000000000000001",
            200,
            None,
            Some("Report Details"),
        ),
    ];
    for (route, expected_status, expected_location, expected_body) in cases {
        let response =
            capture::fetch_route(&client, rust_admin.base_url(), route, Some(&session_cookie))
                .await
                .map_err(test_error)?;
        assert_eq!(response.status, expected_status, "{route}: {response:#?}");
        if let Some(expected_location) = expected_location {
            assert_eq!(
                response.location.as_deref(),
                Some(expected_location),
                "{route}: {response:#?}"
            );
        }
        if let Some(expected_body) = expected_body {
            assert!(
                response.body.contains(expected_body),
                "{route}: expected body to contain {expected_body:?}\n{response:#?}"
            );
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "set FLUXER_ADMIN_PARITY_RUN=1 to create/use the TS worktree and run dual-server parity"]
async fn dual_server_static_public_and_protected_routes() -> Result<(), Box<dyn Error>> {
    if !env_flag(PARITY_RUN_ENV) {
        eprintln!("skipping dual-server parity; set {PARITY_RUN_ENV}=1 to run it");
        return Ok(());
    }
    let repo_root = repo_root()?;
    let api_server = api_fixtures::ApiFixtureServer::start_default()
        .await
        .map_err(test_error)?;
    let worktree = reference_worktree::ensure_reference_worktree(&repo_root).map_err(test_error)?;
    reference_worktree::prepare_reference_package(&worktree).map_err(test_error)?;
    let ts_port = servers::reserve_local_port().map_err(test_error)?;
    let ts_admin = servers::start_ts_admin(&worktree, ts_port, api_server.base_url())
        .await
        .map_err(test_error)?;
    let rust_admin = servers::start_rust_admin(api_server.base_url())
        .await
        .map_err(test_error)?;
    let client = capture::capture_client().map_err(test_error)?;
    let public_routes = route_list_from_env(PUBLIC_ROUTES_ENV, DEFAULT_PUBLIC_ROUTES);
    for route in public_routes {
        capture::compare_route(
            &client,
            &route,
            ts_admin.base_url(),
            rust_admin.base_url(),
            None,
        )
        .await
        .map_err(test_error)?;
    }
    let session = fluxer_admin::session::create_session(
        TEST_ADMIN_USER_ID,
        TEST_ACCESS_TOKEN,
        TEST_ADMIN_SECRET,
    );
    let session_cookie = format!("{}={session}", fluxer_admin::session::SESSION_COOKIE_NAME);
    let protected_routes = route_list_from_env(PROTECTED_ROUTES_ENV, DEFAULT_PROTECTED_ROUTES);
    for route in protected_routes {
        capture::compare_route(
            &client,
            &route,
            ts_admin.base_url(),
            rust_admin.base_url(),
            Some(&session_cookie),
        )
        .await
        .map_err(test_error)?;
    }
    Ok(())
}

fn repo_root() -> Result<std::path::PathBuf, Box<dyn Error>> {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .map(std::path::Path::to_path_buf)
        .ok_or_else(|| test_error("fluxer_admin must have a repository parent".to_owned()))
}

fn test_error(message: String) -> Box<dyn Error> {
    Box::new(io::Error::other(message))
}
