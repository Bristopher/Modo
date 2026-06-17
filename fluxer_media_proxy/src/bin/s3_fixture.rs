// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_media_proxy::{config::Config, storage::Store};
use std::{env, path::PathBuf};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::load_from_env()?;
    let store = Store::new(cfg);
    let image_path = required_path("S3_FIXTURE_IMAGE")?;
    let video_path = required_path("S3_FIXTURE_VIDEO")?;
    let image = tokio::fs::read(&image_path).await?;
    let video = tokio::fs::read(&video_path).await?;

    store.ensure_bucket("cdn").await?;
    store.ensure_bucket("uploads").await?;
    store.ensure_bucket("static").await?;

    store
        .write_object(
            "cdn",
            "themes/test.css",
            b"body{color:#123456}\n",
            "text/css; charset=utf-8",
        )
        .await?;
    store
        .write_object(
            "cdn",
            "attachments/1/2/file.txt",
            b"0123456789abcdef",
            "text/plain",
        )
        .await?;
    store
        .write_object("cdn", "attachments/1/3/image.png", &image, "image/png")
        .await?;
    store
        .write_object("cdn", "attachments/1/4/video.mp4", &video, "video/mp4")
        .await?;
    store
        .write_object("cdn", "avatars/42/abc.png", &image, "image/png")
        .await?;
    store
        .write_object("cdn", "metadata/image.png", &image, "image/png")
        .await?;
    store
        .write_object("uploads", "upload-image.png", &image, "image/png")
        .await?;
    store
        .write_object("uploads", "upload-video.mp4", &video, "video/mp4")
        .await?;
    tracing::info!("seeded S3 media proxy fixtures");
    Ok(())
}

fn required_path(name: &str) -> anyhow::Result<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("{name} is required"))
}
