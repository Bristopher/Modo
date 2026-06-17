// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    Json,
    http::{HeaderValue, header},
    response::{IntoResponse, Response},
};
use serde_json::json;

pub async fn apple_app_site_association() -> Response {
    let body = json!({
        "webcredentials": {
            "apps": [
                "3G5837T29K.app.fluxer",
                "3G5837T29K.app.fluxer.canary",
                "3G5837T29K.com.fluxer",
                "3G5837T29K.com.fluxer.canary"
            ]
        }
    });

    let mut response = Json(body).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=1800"),
    );
    response
}
