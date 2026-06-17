// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_media_proxy::constants::AssetExtension;
use fluxer_media_proxy::media_process::{ImageOptions, transform_image};

fn main() {
    let input = std::fs::read("/tmp/source.bin").expect("read source");
    let out = transform_image(
        &input,
        &ImageOptions {
            width: Some(240),
            height: Some(240),
            format: AssetExtension::Gif,
            quality: "high".to_owned(),
            animated: true,
            cover_crop: false,
            ..Default::default()
        },
    )
    .expect("transform");
    std::fs::write("/tmp/out_current.gif", &out.bytes).expect("write");
    eprintln!(
        "current (ffmpeg path) wrote /tmp/out_current.gif: {} bytes",
        out.bytes.len()
    );
}
