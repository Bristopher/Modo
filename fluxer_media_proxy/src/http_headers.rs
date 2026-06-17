// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::range::ByteRange;
use http::{HeaderMap, HeaderName, HeaderValue, header};

pub const ROBOTS: &str = "noindex, nofollow, nosnippet, noimageindex, notranslate, max-snippet:0, max-image-preview:none, max-video-preview:0";
pub const MEDIA_CSP: &str = "default-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'; object-src 'none'; script-src 'none'; script-src-attr 'none'; script-src-elem 'none'; style-src 'unsafe-inline'; img-src 'self' blob: data:; media-src 'self' blob:; sandbox allow-same-origin";
pub const STRICT_TRANSPORT_SECURITY: &str = "max-age=31536000; includeSubDomains; preload";
pub const REFERRER_POLICY: &str = "strict-origin-when-cross-origin";
pub const PERMISSIONS_POLICY: &str = "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()";

pub fn add_security_headers(headers: &mut HeaderMap) {
    set_static_header(
        headers,
        HeaderName::from_static("strict-transport-security"),
        STRICT_TRANSPORT_SECURITY,
    );
    set_static_header(headers, header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    set_static_header(
        headers,
        HeaderName::from_static("referrer-policy"),
        REFERRER_POLICY,
    );
    set_static_header(headers, HeaderName::from_static("x-frame-options"), "DENY");
    set_static_header(
        headers,
        HeaderName::from_static("permissions-policy"),
        PERMISSIONS_POLICY,
    );
    set_static_header(headers, header::CONTENT_SECURITY_POLICY, MEDIA_CSP);
}

fn set_static_header(headers: &mut HeaderMap, name: HeaderName, value: &'static str) {
    headers
        .entry(name)
        .or_insert(HeaderValue::from_static(value));
}

pub fn add_media_headers(
    headers: &mut HeaderMap,
    size: usize,
    content_type: &str,
    byte_range: Option<ByteRange>,
) {
    add_security_headers(headers);
    let streamable = content_type.starts_with("video/") || content_type.starts_with("audio/");
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static(if streamable {
            "public, max-age=31536000, no-transform, immutable"
        } else {
            "public, max-age=31536000, immutable"
        }),
    );
    headers.insert(
        "CDN-Cache-Control",
        HeaderValue::from_static("public, max-age=31536000"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(MEDIA_CSP),
    );
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    headers.insert(
        header::EXPIRES,
        HeaderValue::from_static("Thu, 31 Dec 2037 23:55:55 GMT"),
    );
    headers.insert(
        header::LAST_MODIFIED,
        HeaderValue::from_static("Thu, 01 Jan 1970 00:00:00 GMT"),
    );
    headers.insert(
        header::VARY,
        HeaderValue::from_static("Accept-Encoding, Range"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Robots-Tag", HeaderValue::from_static(ROBOTS));
    if let Some(r) = byte_range {
        headers.insert(
            header::CONTENT_RANGE,
            HeaderValue::from_str(&format!("bytes {}-{}/{}", r.start, r.end, size))
                .expect("content-range is ASCII"),
        );
    }
}

pub fn add_unsatisfiable_headers(headers: &mut HeaderMap, size: usize) {
    add_security_headers(headers);
    headers.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    headers.insert(
        header::CONTENT_RANGE,
        HeaderValue::from_str(&format!("bytes */{size}")).expect("content-range is ASCII"),
    );
    headers.insert(
        header::VARY,
        HeaderValue::from_static("Accept-Encoding, Range"),
    );
    headers.insert("X-Robots-Tag", HeaderValue::from_static(ROBOTS));
}
