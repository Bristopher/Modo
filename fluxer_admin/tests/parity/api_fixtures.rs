// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderName, HeaderValue, Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{net::TcpListener, task::JoinHandle};

#[derive(Clone, Debug)]
struct ApiFixtureRoute {
    method: String,
    path: String,
    status: u16,
    content_type: String,
    headers: HashMap<String, String>,
    body: String,
}

#[derive(Clone)]
struct ApiFixtureSet {
    routes: Arc<Vec<ApiFixtureRoute>>,
}

#[derive(Deserialize)]
struct ApiFixtureManifest {
    routes: Vec<ApiFixtureManifestRoute>,
}

#[derive(Deserialize)]
struct ApiFixtureManifestRoute {
    method: String,
    path: String,
    status: Option<u16>,
    content_type: Option<String>,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
    body_file: Option<String>,
}

pub struct ApiFixtureServer {
    base_url: String,
    handle: JoinHandle<()>,
}

impl ApiFixtureServer {
    pub async fn start_default() -> Result<Self, String> {
        let fixture_set = ApiFixtureSet::from_default_manifest()?;
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|error| format!("failed to bind fixture API server: {error}"))?;
        let port = listener
            .local_addr()
            .map_err(|error| format!("failed to read fixture API address: {error}"))?
            .port();
        let app = Router::new()
            .fallback(any(handle_fixture_request))
            .with_state(fixture_set.routes);
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        Ok(Self {
            base_url: format!("http://127.0.0.1:{port}"),
            handle,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Drop for ApiFixtureServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl ApiFixtureSet {
    fn from_default_manifest() -> Result<Self, String> {
        let manifest: ApiFixtureManifest =
            serde_json::from_str(include_str!("fixtures/api_routes.json"))
                .map_err(|error| format!("failed to parse API fixture manifest: {error}"))?;
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/parity/fixtures/api");
        let mut routes = Vec::with_capacity(manifest.routes.len());
        for route in manifest.routes {
            routes.push(ApiFixtureRoute {
                method: route.method.to_ascii_uppercase(),
                path: route.path,
                status: route.status.unwrap_or(200),
                content_type: route
                    .content_type
                    .unwrap_or_else(|| "application/json; charset=utf-8".to_owned()),
                headers: route.headers.unwrap_or_default(),
                body: load_body(&root, route.body, route.body_file)?,
            });
        }
        Ok(Self {
            routes: Arc::new(routes),
        })
    }
}

async fn handle_fixture_request(
    State(routes): State<Arc<Vec<ApiFixtureRoute>>>,
    request: Request<Body>,
) -> Response {
    let method = request.method().as_str();
    let path = request.uri().path();
    let fixture = routes
        .iter()
        .find(|route| route.method == method && route.path == path)
        .or_else(|| {
            routes
                .iter()
                .find(|route| route.method == "ANY" && route.path == path)
        });
    match fixture {
        Some(fixture) => fixture_response(fixture),
        None => (
            StatusCode::NOT_FOUND,
            [(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json; charset=utf-8"),
            )],
            format!(r#"{{"error":"missing fixture","method":"{method}","path":"{path}"}}"#),
        )
            .into_response(),
    }
}

fn fixture_response(fixture: &ApiFixtureRoute) -> Response {
    let status = StatusCode::from_u16(fixture.status).unwrap_or(StatusCode::OK);
    let mut response = (status, fixture.body.clone()).into_response();
    if let Ok(value) = HeaderValue::from_str(&fixture.content_type) {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }
    for (name, value) in &fixture.headers {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(name.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            response.headers_mut().insert(name, value);
        }
    }
    response
}

fn load_body(
    root: &Path,
    body: Option<String>,
    body_file: Option<String>,
) -> Result<String, String> {
    if let Some(body) = body {
        return Ok(body);
    }
    let Some(body_file) = body_file else {
        return Ok(String::new());
    };
    let path = root.join(body_file);
    std::fs::read_to_string(&path)
        .map_err(|error| format!("failed to read fixture {}: {error}", path.display()))
}
