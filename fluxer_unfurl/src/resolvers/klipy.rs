// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{ResolveContext, Resolver, ResolverResult};
use crate::http_fetch;
use crate::media_proxy::{MediaMetadata, MediaProxyClient, embed_media_flags};
use crate::types::{EmbedMedia, EmbedProvider, MessageEmbed};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use url::Url;

const KLIPY_FLIGHT_CHUNK_MAX_BYTES: usize = 512 * 1024;

pub struct KlipyResolver;

#[derive(Debug, Clone, Default, PartialEq)]
struct KlipyMediaFormat {
    url: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct KlipyMediaFormats {
    thumbnail: Option<KlipyMediaFormat>,
    video: Option<KlipyMediaFormat>,
}

impl Resolver for KlipyResolver {
    fn matches(&self, url: &Url) -> bool {
        url.host_str()
            .is_some_and(|h| h.eq_ignore_ascii_case("klipy.com"))
    }

    fn transform_url(&self, url: &Url) -> Option<Url> {
        if !url
            .host_str()
            .is_some_and(|h| h.eq_ignore_ascii_case("klipy.com"))
        {
            return None;
        }

        let path = url.path();
        static PATH_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r"^/(gif|gifs|clip|clips)/([^/]+)").expect("valid regex")
        });
        let caps = PATH_RE.captures(path)?;
        let kind = caps.get(1)?.as_str();
        let slug = caps.get(2)?.as_str();

        let normalized_kind = if kind.starts_with("clip") {
            "clips"
        } else {
            "gifs"
        };

        Url::parse(&format!(
            "https://klipy.com/{normalized_kind}/{slug}/player"
        ))
        .ok()
    }

    fn resolve<'a>(
        &'a self,
        ctx: &'a ResolveContext<'_>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResolverResult>> + Send + 'a>> {
        Box::pin(async move {
            let result = http_fetch::fetch_url(
                &ctx.http_client,
                ctx.url.as_str(),
                http_fetch::DEFAULT_HTML_MAX_BYTES,
                Duration::from_secs(10),
            )
            .await?;

            if result.status != 200 {
                return Ok(ResolverResult { embeds: vec![] });
            }

            let html = String::from_utf8_lossy(&result.bytes);
            let formats = match extract_klipy_media(&html) {
                Some(formats) => formats,
                None => {
                    return Ok(ResolverResult { embeds: vec![] });
                }
            };

            let mut embed = MessageEmbed::new("gifv");
            embed.url = Some(ctx.original_url.to_string());
            embed.provider = Some(EmbedProvider {
                name: Some("KLIPY".to_owned()),
                url: Some("https://klipy.com".to_owned()),
            });
            let nsfw_str = MediaProxyClient::nsfw_mode_str(ctx.nsfw_mode);

            if let Some(ref thumbnail) = formats.thumbnail {
                embed.thumbnail = resolve_klipy_media(ctx, thumbnail, nsfw_str).await;
            }

            if let Some(ref video) = formats.video {
                embed.video = resolve_klipy_media(ctx, video, nsfw_str).await;
            }

            Ok(ResolverResult {
                embeds: vec![embed],
            })
        })
    }
}

async fn resolve_klipy_media(
    ctx: &ResolveContext<'_>,
    format: &KlipyMediaFormat,
    nsfw_mode: &str,
) -> Option<EmbedMedia> {
    let url = format.url.as_deref()?;
    let resolved_url = resolve_relative_url(&ctx.original_url, url)?;
    let meta = match ctx.media_proxy.get_metadata(&resolved_url, nsfw_mode).await {
        Ok(meta) => meta,
        Err(err) => {
            tracing::warn!(error = %err, url = resolved_url, "failed to enrich KLIPY media metadata");
            return None;
        }
    };
    Some(build_embed_media_payload(
        &resolved_url,
        &meta,
        format.width,
        format.height,
    ))
}

fn resolve_relative_url(base_url: &Url, media_url: &str) -> Option<String> {
    let url = base_url.join(media_url).ok()?;
    if matches!(url.scheme(), "http" | "https") {
        Some(url.to_string())
    } else {
        None
    }
}

fn build_embed_media_payload(
    url: &str,
    metadata: &MediaMetadata,
    width: Option<u32>,
    height: Option<u32>,
) -> EmbedMedia {
    EmbedMedia {
        url: Some(url.to_owned()),
        width: width.or(metadata.width),
        height: height.or(metadata.height),
        placeholder: metadata.placeholder.clone(),
        flags: embed_media_flags(metadata),
        content_hash: Some(metadata.content_hash.clone()),
        content_type: Some(metadata.content_type.clone()),
        duration: metadata.duration.map(|duration| duration as u32),
        ..Default::default()
    }
}

fn extract_klipy_media(html: &str) -> Option<KlipyMediaFormats> {
    static FLIGHT_RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r#"(?s)self\.__next_f\.push\(\[1,"(.*?)"\]\)"#).expect("valid regex")
    });

    for cap in FLIGHT_RE.captures_iter(html) {
        let encoded = cap.get(1)?.as_str();
        if encoded.len() > KLIPY_FLIGHT_CHUNK_MAX_BYTES {
            continue;
        }
        if let Some(media) = parse_next_flight_data(encoded) {
            return Some(media);
        }
    }

    None
}

fn parse_next_flight_data(encoded: &str) -> Option<KlipyMediaFormats> {
    let unescaped = serde_json::from_str::<String>(&format!("\"{encoded}\"")).ok()?;
    let colon_idx = unescaped.find(':')?;
    let json_str = &unescaped[colon_idx + 1..];
    if json_str.len() > KLIPY_FLIGHT_CHUNK_MAX_BYTES {
        return None;
    }

    let arr: Vec<serde_json::Value> = serde_json::from_str(json_str).ok()?;

    for item in &arr {
        if let Some(media) = item.get("media")
            && media.get("file").is_some()
        {
            return Some(KlipyMediaFormats {
                thumbnail: extract_media_format(media.pointer("/file/hd/webp")),
                video: extract_media_format(media.pointer("/file/hd/mp4")),
            });
        }
    }

    None
}

fn extract_media_format(value: Option<&serde_json::Value>) -> Option<KlipyMediaFormat> {
    let value = value?;
    Some(KlipyMediaFormat {
        url: value
            .get("url")
            .and_then(|v| v.as_str())
            .map(|url| url.to_owned()),
        width: value
            .get("width")
            .and_then(|v| v.as_u64())
            .and_then(|width| u32::try_from(width).ok()),
        height: value
            .get("height")
            .and_then(|v| v.as_u64())
            .and_then(|height| u32::try_from(height).ok()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_next_flight_data_extracts_media_formats() {
        let encoded = r#"0:[{\"media\":{\"file\":{\"hd\":{\"webp\":{\"url\":\"https://img.example/a.webp\",\"width\":320,\"height\":180},\"mp4\":{\"url\":\"https://img.example/a.mp4\",\"width\":640,\"height\":360}}}}}]"#;
        assert_eq!(
            parse_next_flight_data(encoded),
            Some(KlipyMediaFormats {
                thumbnail: Some(KlipyMediaFormat {
                    url: Some("https://img.example/a.webp".to_owned()),
                    width: Some(320),
                    height: Some(180),
                }),
                video: Some(KlipyMediaFormat {
                    url: Some("https://img.example/a.mp4".to_owned()),
                    width: Some(640),
                    height: Some(360),
                }),
            })
        );
    }

    #[test]
    fn extract_klipy_media_skips_oversized_flight_chunks() {
        let html = format!(
            r#"self.__next_f.push([1,"{}"])"#,
            "x".repeat(KLIPY_FLIGHT_CHUNK_MAX_BYTES + 1)
        );
        assert_eq!(extract_klipy_media(&html), None);
    }

    #[test]
    fn resolve_relative_url_uses_original_url_like_ts() {
        let base = Url::parse("https://klipy.com/gifs/funny").unwrap();
        assert_eq!(
            resolve_relative_url(&base, "/media/a.webp").as_deref(),
            Some("https://klipy.com/media/a.webp")
        );
    }

    #[test]
    fn build_embed_media_payload_prefers_format_dimensions() {
        let meta = MediaMetadata {
            format: "webp".to_owned(),
            content_type: "image/webp".to_owned(),
            content_hash: "hash".to_owned(),
            size: 123,
            width: Some(640),
            height: Some(360),
            duration: Some(2.9),
            placeholder: Some("placeholder".to_owned()),
            animated: Some(true),
            nsfw: false,
            nsfw_probability: None,
        };
        let media = build_embed_media_payload("https://img.example/a.webp", &meta, Some(320), None);
        assert_eq!(media.width, Some(320));
        assert_eq!(media.height, Some(360));
        assert_eq!(media.duration, Some(2));
        assert_eq!(media.content_hash.as_deref(), Some("hash"));
        assert_eq!(media.content_type.as_deref(), Some("image/webp"));
        assert_eq!(media.placeholder.as_deref(), Some("placeholder"));
        assert_eq!(media.flags, 1 << 5);
    }

    #[test]
    fn matches_klipy_com_only() {
        let r = KlipyResolver;
        assert!(r.matches(&Url::parse("https://klipy.com/gifs/funny").unwrap()));
        assert!(!r.matches(&Url::parse("https://notklipy.com/gifs/funny").unwrap()));
    }

    #[test]
    fn transform_url_normalises_gif_to_gifs_player() {
        let r = KlipyResolver;
        let transformed = r.transform_url(&Url::parse("https://klipy.com/gif/abc-123").unwrap());
        assert_eq!(
            transformed.as_ref().map(|u| u.as_str()),
            Some("https://klipy.com/gifs/abc-123/player")
        );
    }

    #[test]
    fn transform_url_normalises_clip_to_clips_player() {
        let r = KlipyResolver;
        let transformed = r.transform_url(&Url::parse("https://klipy.com/clip/xyz").unwrap());
        assert_eq!(
            transformed.as_ref().map(|u| u.as_str()),
            Some("https://klipy.com/clips/xyz/player")
        );
    }

    #[test]
    fn transform_url_already_pluralised() {
        let r = KlipyResolver;
        let transformed = r.transform_url(&Url::parse("https://klipy.com/gifs/abc").unwrap());
        assert_eq!(
            transformed.as_ref().map(|u| u.as_str()),
            Some("https://klipy.com/gifs/abc/player")
        );
    }

    #[test]
    fn transform_url_non_matching_path_returns_none() {
        let r = KlipyResolver;
        assert!(
            r.transform_url(&Url::parse("https://klipy.com/about").unwrap())
                .is_none()
        );
    }

    #[test]
    fn extract_klipy_media_from_realistic_html() {
        let html = r#"
        <script>self.__next_f.push([1,"0:[{\"media\":{\"file\":{\"hd\":{\"webp\":{\"url\":\"https://img.klipy.com/hd.webp\",\"width\":640,\"height\":360},\"mp4\":{\"url\":\"https://img.klipy.com/hd.mp4\",\"width\":1280,\"height\":720}}}}}]"])</script>
        "#;
        let result = extract_klipy_media(html);
        assert!(result.is_some());
        let formats = result.unwrap();
        assert_eq!(
            formats.thumbnail.as_ref().unwrap().url.as_deref(),
            Some("https://img.klipy.com/hd.webp")
        );
        assert_eq!(
            formats.video.as_ref().unwrap().url.as_deref(),
            Some("https://img.klipy.com/hd.mp4")
        );
    }

    #[test]
    fn extract_klipy_media_returns_none_for_non_media_chunks() {
        let html = r#"<script>self.__next_f.push([1,"0:[{\"status\":\"ok\"}]"])</script>"#;
        assert!(extract_klipy_media(html).is_none());
    }

    #[test]
    fn resolve_relative_url_rejects_non_http() {
        let base = Url::parse("https://klipy.com/gifs/test").unwrap();
        assert!(resolve_relative_url(&base, "ftp://evil.com/file").is_none());
    }
}
