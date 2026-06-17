// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::{
    extract::Request,
    http::{HeaderValue, Uri, header},
    middleware::Next,
    response::Response,
};

#[derive(Clone, Copy, Debug)]
pub struct SelfHostedOverride;

pub async fn self_hosted_override(mut request: Request, next: Next) -> Response {
    let has_sh = request
        .uri()
        .query()
        .map(|q| q.split('&').any(|p| p == "sh=1"))
        .unwrap_or(false);

    if has_sh {
        request.extensions_mut().insert(SelfHostedOverride);
    }

    let mut response = next.run(request).await;

    if has_sh {
        rewrite_location_header(&mut response);
    }

    response
}

fn rewrite_location_header(response: &mut Response) {
    let location = match response.headers().get(header::LOCATION) {
        Some(v) => match v.to_str() {
            Ok(s) => s.to_owned(),
            Err(_) => return,
        },
        None => return,
    };

    if !location.starts_with('/') || location.contains("sh=1") {
        return;
    }

    let separator = if location
        .parse::<Uri>()
        .ok()
        .and_then(|u| u.query().map(|_| ()))
        .is_some()
    {
        '&'
    } else {
        '?'
    };

    let new_location = format!("{location}{separator}sh=1");
    if let Ok(v) = HeaderValue::from_str(&new_location) {
        response.headers_mut().insert(header::LOCATION, v);
    }
}
