// SPDX-License-Identifier: AGPL-3.0-or-later

use super::TEST_ADMIN_SECRET;
use fluxer_admin::{
    build_router,
    config::{AdminConfig, ProxyConfig, RuntimeEnv},
};
use std::{
    net::TcpListener as StdTcpListener,
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use tokio::{net::TcpListener, task::JoinHandle, time::sleep};

pub struct RunningRustAdmin {
    base_url: String,
    handle: JoinHandle<()>,
}

pub struct RunningTsAdmin {
    base_url: String,
    child: Child,
}

impl RunningRustAdmin {
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl RunningTsAdmin {
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for RunningRustAdmin {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl Drop for RunningTsAdmin {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

pub fn reserve_local_port() -> Result<u16, String> {
    let listener = StdTcpListener::bind("127.0.0.1:0")
        .map_err(|error| format!("failed to reserve local port: {error}"))?;
    listener
        .local_addr()
        .map(|addr| addr.port())
        .map_err(|error| format!("failed to read reserved local port: {error}"))
}

pub async fn start_rust_admin(api_endpoint: &str) -> Result<RunningRustAdmin, String> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|error| format!("failed to bind Rust admin server: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("failed to read Rust admin address: {error}"))?
        .port();
    let base_url = format!("http://127.0.0.1:{port}");
    let config = admin_config(port, api_endpoint, &base_url);
    let router = build_router(config);
    let handle = tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    wait_for_health(&base_url).await?;
    Ok(RunningRustAdmin { base_url, handle })
}

pub async fn start_ts_admin(
    worktree: &Path,
    port: u16,
    api_endpoint: &str,
) -> Result<RunningTsAdmin, String> {
    let base_url = format!("http://127.0.0.1:{port}");
    let mut command = Command::new("pnpm");
    command
        .current_dir(worktree)
        .args(["--filter", "fluxer_admin", "start"])
        .env("BUILD_VERSION", "parity")
        .env("RELEASE_CHANNEL", "parity");
    for (key, value) in ts_admin_env(port, api_endpoint, &base_url) {
        command.env(key, value);
    }
    let child = command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to start TS admin server: {error}"))?;
    let mut server = RunningTsAdmin { base_url, child };
    if let Err(error) = wait_for_health(server.base_url()).await {
        let _ = server.child.kill();
        let _ = server.child.wait();
        return Err(error);
    }
    Ok(server)
}

fn admin_config(port: u16, api_endpoint: &str, admin_endpoint: &str) -> AdminConfig {
    AdminConfig {
        env: RuntimeEnv::Test,
        host: "127.0.0.1".to_owned(),
        port,
        secret_key_base: TEST_ADMIN_SECRET.to_owned(),
        base_path: String::new(),
        api_endpoint: api_endpoint.to_owned(),
        media_endpoint: format!("{api_endpoint}/media"),
        static_cdn_endpoint: "https://static.example.test".to_owned(),
        admin_endpoint: admin_endpoint.to_owned(),
        web_app_endpoint: "http://127.0.0.1:8088".to_owned(),
        kv_url: "redis://127.0.0.1:6379/0".to_owned(),
        oauth_client_id: "1234567890123456789".to_owned(),
        oauth_client_secret: "test-admin-oauth-secret".to_owned(),
        oauth_redirect_uri: format!("{admin_endpoint}/oauth2_callback"),
        build_version: "parity".to_owned(),
        release_channel: "parity".to_owned(),
        self_hosted: false,
        proxy: ProxyConfig {
            trust_client_ip_header: false,
            client_ip_header_name: "x-forwarded-for".to_owned(),
        },
    }
}

fn ts_admin_env(
    port: u16,
    api_endpoint: &str,
    admin_endpoint: &str,
) -> Vec<(&'static str, String)> {
    vec![
        ("FLUXER_ENV", "test".to_owned()),
        ("NODE_ENV", "production".to_owned()),
        ("FLUXER_BASE_DOMAIN", "127.0.0.1".to_owned()),
        ("FLUXER_PUBLIC_SCHEME", "http".to_owned()),
        ("FLUXER_PUBLIC_PORT", port.to_string()),
        ("FLUXER_API_ENDPOINT", api_endpoint.to_owned()),
        ("FLUXER_ADMIN_ENDPOINT", admin_endpoint.to_owned()),
        ("FLUXER_APP_ENDPOINT", "http://127.0.0.1:8088".to_owned()),
        ("FLUXER_MEDIA_ENDPOINT", format!("{api_endpoint}/media")),
        (
            "FLUXER_STATIC_CDN_ENDPOINT",
            "https://static.example.test".to_owned(),
        ),
        ("FLUXER_CASSANDRA_HOSTS", "127.0.0.1".to_owned()),
        ("FLUXER_CASSANDRA_KEYSPACE", "fluxer_test".to_owned()),
        ("FLUXER_CASSANDRA_LOCAL_DC", "datacenter1".to_owned()),
        ("FLUXER_CASSANDRA_USERNAME", "cassandra".to_owned()),
        ("FLUXER_CASSANDRA_PASSWORD", "cassandra".to_owned()),
        ("FLUXER_KV_URL", "redis://127.0.0.1:6379/0".to_owned()),
        ("FLUXER_S3_ACCESS_KEY_ID", "test".to_owned()),
        ("FLUXER_S3_SECRET_ACCESS_KEY", "test".to_owned()),
        (
            "FLUXER_MEDIA_PROXY_SECRET_KEY",
            "test-media-secret".to_owned(),
        ),
        ("FLUXER_ADMIN_PORT", port.to_string()),
        ("FLUXER_ADMIN_SECRET_KEY_BASE", TEST_ADMIN_SECRET.to_owned()),
        (
            "FLUXER_ADMIN_OAUTH_CLIENT_SECRET",
            "test-admin-oauth-secret".to_owned(),
        ),
        (
            "FLUXER_MARKETING_SECRET_KEY_BASE",
            "test-marketing-secret".to_owned(),
        ),
        ("FLUXER_APP_PROXY_PORT", "8773".to_owned()),
        (
            "FLUXER_GATEWAY_MEDIA_PROXY_ENDPOINT",
            format!("{api_endpoint}/media"),
        ),
        (
            "FLUXER_GATEWAY_RPC_AUTH_TOKEN",
            "test-gateway-rpc-token".to_owned(),
        ),
        ("FLUXER_GATEWAY_PUSH_ENABLED", "false".to_owned()),
        ("FLUXER_SUDO_MODE_SECRET", "test-sudo-secret".to_owned()),
        (
            "FLUXER_CONNECTION_INITIATION_SECRET",
            "test-connection-secret".to_owned(),
        ),
        (
            "FLUXER_VAPID_PUBLIC_KEY",
            "test-vapid-public-key".to_owned(),
        ),
        (
            "FLUXER_VAPID_PRIVATE_KEY",
            "test-vapid-private-key".to_owned(),
        ),
        ("FLUXER_VAPID_EMAIL", "test@example.com".to_owned()),
        ("FLUXER_EMAIL_ENABLED", "false".to_owned()),
        ("FLUXER_LIVEKIT_ENABLED", "false".to_owned()),
        ("FLUXER_STRIPE_ENABLED", "true".to_owned()),
        ("FLUXER_SEARCH_URL", "http://127.0.0.1:9200".to_owned()),
        ("FLUXER_SEARCH_API_KEY", "test".to_owned()),
        ("FLUXER_CAPTCHA_ENABLED", "false".to_owned()),
        ("FLUXER_CAPTCHA_PROVIDER", "none".to_owned()),
        ("FLUXER_SELF_HOSTED", "false".to_owned()),
        ("FLUXER_DISCOVERY_ENABLED", "true".to_owned()),
        ("FLUXER_DISABLE_RATE_LIMITS", "true".to_owned()),
        ("FLUXER_TEST_MODE_ENABLED", "true".to_owned()),
    ]
}

async fn wait_for_health(base_url: &str) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| format!("failed to build health client: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let url = format!("{}/_health", base_url.trim_end_matches('/'));
    let mut last_error = String::new();
    while Instant::now() < deadline {
        match client.get(&url).send().await {
            Ok(response) if response.status().is_success() => return Ok(()),
            Ok(response) => {
                last_error = format!("health returned {}", response.status());
            }
            Err(error) => {
                last_error = error.to_string();
            }
        }
        sleep(Duration::from_millis(200)).await;
    }
    Err(format!("timed out waiting for {url}: {last_error}"))
}
