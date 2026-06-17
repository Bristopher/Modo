// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::metrics::{self, RequestKind};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::Response,
};
use rand::RngExt;
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tracing::{Level, event};

const ID_ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const ID_LEN: usize = 12;

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn generate() -> Self {
        let mut raw: u64 = rand::rng().random();
        let mut id = [0u8; ID_LEN];
        for slot in id.iter_mut().rev() {
            *slot = ID_ALPHABET[(raw & 0x1f) as usize];
            raw >>= 5;
        }
        Self(String::from_utf8(id.to_vec()).expect("alphabet is ASCII"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Default)]
pub struct StageTimings {
    fetch_ms: AtomicU64,
    transform_ms: AtomicU64,
    nsfw_ms: AtomicU64,
}

#[derive(Clone, Copy, Debug)]
pub enum Stage {
    Fetch,
    Transform,
    Nsfw,
}

impl StageTimings {
    pub fn add(&self, stage: Stage, ms: u64) {
        let slot = match stage {
            Stage::Fetch => &self.fetch_ms,
            Stage::Transform => &self.transform_ms,
            Stage::Nsfw => &self.nsfw_ms,
        };
        slot.fetch_add(ms, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> (u64, u64, u64) {
        (
            self.fetch_ms.load(Ordering::Relaxed),
            self.transform_ms.load(Ordering::Relaxed),
            self.nsfw_ms.load(Ordering::Relaxed),
        )
    }
}

tokio::task_local! {
    static STAGES: Arc<StageTimings>;
}

pub fn record_stage(stage: Stage, ms: u64) {
    let _ = STAGES.try_with(|s| s.add(stage, ms));
}

pub async fn timed_stage<F, T>(stage: Stage, fut: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let start = metrics::now_ms();
    let out = fut.await;
    let elapsed = (metrics::now_ms() - start).max(0) as u64;
    record_stage(stage, elapsed);
    out
}

#[derive(Clone, Debug)]
pub struct ErrorReason {
    pub code: &'static str,
    pub source: Option<String>,
}

impl ErrorReason {
    pub fn new(code: &'static str) -> Self {
        Self { code, source: None }
    }

    pub fn with_source(code: &'static str, source: impl fmt::Debug) -> Self {
        Self {
            code,
            source: Some(format!("{source:?}")),
        }
    }

    pub fn with_message(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            source: Some(message.into()),
        }
    }
}

pub fn classify_route(path: &str) -> RequestKind {
    if let Some(kind) = match path {
        "/_health" => Some(RequestKind::Health),
        "/_metrics" => Some(RequestKind::Other),
        "/_metadata" => Some(RequestKind::Metadata),
        "/_thumbnail" => Some(RequestKind::Thumbnail),
        "/_frames" => Some(RequestKind::Frames),
        _ => None,
    } {
        return kind;
    }
    if path.starts_with("/v1/relay/") {
        return RequestKind::Upload;
    }
    if path.starts_with("/external/") {
        return RequestKind::External;
    }
    if path.starts_with("/attachments/") {
        return RequestKind::Attachment;
    }
    if path.starts_with("/themes/") {
        return RequestKind::Themes;
    }
    if path.starts_with("/guilds/") {
        return RequestKind::GuildMemberImage;
    }
    if path.starts_with("/avatars/")
        || path.starts_with("/icons/")
        || path.starts_with("/banners/")
        || path.starts_with("/splashes/")
        || path.starts_with("/embed-splashes/")
        || path.starts_with("/emojis/")
        || path.starts_with("/stickers/")
    {
        return RequestKind::AssetImage;
    }
    RequestKind::Other
}

pub async fn trace(mut req: Request, next: Next) -> Response {
    let id = RequestId::generate();
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let query = req.uri().query().map(ToOwned::to_owned);
    let kind = classify_route(&path);
    let referer = header_str(req.headers(), header::REFERER);
    let user_agent = header_str(req.headers(), header::USER_AGENT);

    req.extensions_mut().insert(id.clone());
    let stages = Arc::new(StageTimings::default());
    req.extensions_mut().insert(stages.clone());
    let started_ms = metrics::now_ms();
    let response = STAGES.scope(stages.clone(), next.run(req)).await;
    let elapsed_ms = (metrics::now_ms() - started_ms).max(0) as u64;
    let (fetch_ms, transform_ms, nsfw_ms) = stages.snapshot();
    let status = response.status();
    let reason = response.extensions().get::<ErrorReason>().cloned();

    metrics::GLOBAL.record_request_with_duration(kind, status.as_u16(), elapsed_ms);

    let path_for_log = clip(&path, 512);
    let query_for_log = query.as_deref().map(|q| clip(q, 512)).unwrap_or_default();

    if status.is_success() || status.is_redirection() {
        if !matches!(kind, RequestKind::Health | RequestKind::Other) {
            event!(
                Level::INFO,
                req = %id.as_str(),
                kind = kind.label(),
                method = %method,
                path = %path_for_log,
                query = %query_for_log,
                status = status.as_u16(),
                duration_ms = elapsed_ms,
                fetch_ms,
                transform_ms,
                nsfw_ms,
                "request"
            );
        }
        return response;
    }

    let level = if status.is_server_error() {
        Level::ERROR
    } else {
        Level::WARN
    };
    let (code, source) = match reason {
        Some(r) => (r.code, r.source.unwrap_or_default()),
        None => (default_reason(status), String::new()),
    };

    match level {
        Level::ERROR => event!(
            Level::ERROR,
            req = %id.as_str(),
            kind = kind.label(),
            method = %method,
            path = %path_for_log,
            query = %query_for_log,
            status = status.as_u16(),
            duration_ms = elapsed_ms,
            fetch_ms,
            transform_ms,
            nsfw_ms,
            reason = code,
            source = %source,
            referer = referer.as_deref().unwrap_or(""),
            user_agent = user_agent.as_deref().unwrap_or(""),
            "request failed"
        ),
        _ => event!(
            Level::WARN,
            req = %id.as_str(),
            kind = kind.label(),
            method = %method,
            path = %path_for_log,
            query = %query_for_log,
            status = status.as_u16(),
            duration_ms = elapsed_ms,
            fetch_ms,
            transform_ms,
            nsfw_ms,
            reason = code,
            source = %source,
            "request rejected"
        ),
    }
    response
}

fn header_str(headers: &HeaderMap, name: header::HeaderName) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| clip(s, 256))
}

fn clip(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_owned();
    }
    let mut out = value[..max].to_owned();
    out.push('~');
    out
}

fn default_reason(status: StatusCode) -> &'static str {
    status.canonical_reason().unwrap_or("error")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, middleware, response::IntoResponse, routing::get};
    use tower::ServiceExt;

    fn req_id_is_alphabet_only(id: &str) -> bool {
        id.len() == ID_LEN && id.bytes().all(|c| ID_ALPHABET.contains(&c))
    }

    #[test]
    fn request_id_is_stable_length_and_alphabet() {
        let id = RequestId::generate();
        assert!(req_id_is_alphabet_only(id.as_str()));
    }

    #[test]
    fn two_back_to_back_ids_differ() {
        assert_ne!(RequestId::generate().0, RequestId::generate().0);
    }

    #[test]
    fn classify_route_buckets_known_prefixes() {
        assert_eq!(RequestKind::Health, classify_route("/_health"));
        assert_eq!(RequestKind::Metadata, classify_route("/_metadata"));
        assert_eq!(RequestKind::Upload, classify_route("/v1/relay/abc"));
        assert_eq!(RequestKind::External, classify_route("/external/x/y"));
        assert_eq!(RequestKind::Attachment, classify_route("/attachments/a/b"));
        assert_eq!(RequestKind::Themes, classify_route("/themes/x.css"));
        assert_eq!(
            RequestKind::GuildMemberImage,
            classify_route("/guilds/1/users/2/avatars/h.png")
        );
        assert_eq!(RequestKind::AssetImage, classify_route("/emojis/1.png"));
        assert_eq!(RequestKind::AssetImage, classify_route("/icons/1/h.png"));
        assert_eq!(RequestKind::Other, classify_route("/unknown"));
    }

    #[test]
    fn clip_truncates_with_marker() {
        assert_eq!("abc", clip("abc", 8));
        assert_eq!("abcdefgh~", clip("abcdefghIJ", 8));
    }

    #[tokio::test]
    async fn middleware_inserts_request_id_into_extensions() {
        async fn handler(req: Request) -> impl IntoResponse {
            let id = req
                .extensions()
                .get::<RequestId>()
                .expect("middleware must inject RequestId")
                .clone();
            id.0
        }
        let app = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn(trace));
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(StatusCode::OK, resp.status());
        let bytes = axum::body::to_bytes(resp.into_body(), 64).await.unwrap();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert!(req_id_is_alphabet_only(body));
    }

    #[tokio::test]
    async fn middleware_records_metrics_and_reads_error_reason() {
        async fn handler() -> Response {
            let mut resp = Response::new(Body::from("boom"));
            *resp.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            resp.extensions_mut()
                .insert(ErrorReason::with_message("transcode_failed", "vips OOM"));
            resp
        }
        let app = Router::new()
            .route("/", get(handler))
            .layer(middleware::from_fn(trace));
        let resp = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.status());
        let reason = resp.extensions().get::<ErrorReason>().unwrap();
        assert_eq!("transcode_failed", reason.code);
        assert_eq!(Some("vips OOM".to_owned()), reason.source);
    }
}
