// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(dead_code)]

#[cfg(feature = "publisher")]
use livekit::webrtc::video_frame::{VideoBuffer, VideoFrame, VideoRotation, native::NativeBuffer};
#[cfg(feature = "publisher")]
use livekit::webrtc::video_source::native::NativeVideoSource;
use napi_derive::napi;

pub const NATIVE_CAMERA_FRAME_QUEUE_CAPACITY: usize = 3;
const MIN_NATIVE_CAMERA_EDGE: u32 = 2;
const MAX_NATIVE_CAMERA_EDGE: u32 = 8192;
const TRANSPORT_CV_PIXEL_BUFFER: &str = "cvPixelBuffer";
const TRANSPORT_D3D11_TEXTURE: &str = "d3d11Texture";
const TRANSPORT_DMABUF: &str = "dmabuf";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeCameraTransport {
    CvPixelBuffer,
    D3d11Texture,
    Dmabuf,
}

impl NativeCameraTransport {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CvPixelBuffer => TRANSPORT_CV_PIXEL_BUFFER,
            Self::D3d11Texture => TRANSPORT_D3D11_TEXTURE,
            Self::Dmabuf => TRANSPORT_DMABUF,
        }
    }
}

pub fn required_transports() -> &'static [NativeCameraTransport] {
    const TRANSPORTS: &[NativeCameraTransport] = &[
        NativeCameraTransport::CvPixelBuffer,
        NativeCameraTransport::D3d11Texture,
        NativeCameraTransport::Dmabuf,
    ];
    assert_eq!(TRANSPORTS.len(), 3);
    TRANSPORTS
}

pub fn required_transport_names() -> [&'static str; 3] {
    let transports = required_transports();
    [
        transports[0].as_str(),
        transports[1].as_str(),
        transports[2].as_str(),
    ]
}

pub fn platform_native_backgrounds_available() -> bool {
    platform_unavailable_reason().is_none()
}

pub fn camera_backgrounds_available() -> bool {
    cfg!(feature = "camera-native")
}

#[napi]
pub fn has_native_camera_backgrounds() -> bool {
    camera_backgrounds_available()
}

pub fn platform_unavailable_reason() -> Option<&'static str> {
    Some(match std::env::consts::OS {
        "macos" => "macOS AVFoundation CVPixelBuffer camera backend is not compiled",
        "windows" => "Windows Media Foundation D3D11 camera backend is not compiled",
        "linux" => "Linux PipeWire/V4L2 dmabuf camera backend is not compiled",
        _ => "native platform-buffer camera backend is unsupported on this platform",
    })
}

pub fn validate_native_frame_dimensions(width: u32, height: u32) -> bool {
    if width < MIN_NATIVE_CAMERA_EDGE {
        return false;
    }
    if height < MIN_NATIVE_CAMERA_EDGE {
        return false;
    }
    if !width.is_multiple_of(2) {
        return false;
    }
    if !height.is_multiple_of(2) {
        return false;
    }
    width <= MAX_NATIVE_CAMERA_EDGE && height <= MAX_NATIVE_CAMERA_EDGE
}

pub fn unavailable_error() -> String {
    let reason = platform_unavailable_reason().unwrap_or("native camera backend unavailable");
    format!(
        "native camera backgrounds require platform camera buffers ({}, {}, {}): {reason}",
        TRANSPORT_CV_PIXEL_BUFFER, TRANSPORT_D3D11_TEXTURE, TRANSPORT_DMABUF
    )
}

#[cfg(feature = "publisher")]
pub fn publish_native_buffer(
    source: &NativeVideoSource,
    buffer: NativeBuffer,
    timestamp_us: i64,
) -> bool {
    assert!(timestamp_us >= 0);
    let width = buffer.width();
    let height = buffer.height();
    if !validate_native_frame_dimensions(width, height) {
        return false;
    }
    source.capture_frame(&VideoFrame {
        rotation: VideoRotation::VideoRotation0,
        timestamp_us,
        frame_metadata: None,
        buffer,
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_transports_are_stable() {
        assert_eq!(
            required_transport_names(),
            ["cvPixelBuffer", "d3d11Texture", "dmabuf"]
        );
    }

    #[test]
    fn queue_capacity_stays_bounded_for_realtime_capture() {
        assert_eq!(NATIVE_CAMERA_FRAME_QUEUE_CAPACITY, 3);
        assert!(NATIVE_CAMERA_FRAME_QUEUE_CAPACITY < 8);
    }

    #[test]
    fn camera_background_capability_tracks_native_camera_feature() {
        assert_eq!(
            camera_backgrounds_available(),
            cfg!(feature = "camera-native")
        );
    }

    #[test]
    fn native_frame_dimensions_require_even_reasonable_sizes() {
        assert!(validate_native_frame_dimensions(1280, 720));
        assert!(!validate_native_frame_dimensions(0, 720));
        assert!(!validate_native_frame_dimensions(1280, 1));
        assert!(!validate_native_frame_dimensions(1279, 720));
        assert!(!validate_native_frame_dimensions(1280, 721));
        assert!(!validate_native_frame_dimensions(16_384, 720));
    }

    #[test]
    fn unavailable_error_names_all_required_native_transports() {
        let error = unavailable_error();
        assert!(error.contains("cvPixelBuffer"));
        assert!(error.contains("d3d11Texture"));
        assert!(error.contains("dmabuf"));
    }
}
