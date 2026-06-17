// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureFrameDesc {
    pub handle: u64,
    pub width: u32,
    pub height: u32,
    pub dxgi_format: u32,
    pub timestamp_us: i64,
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DmabufFrameDesc {
    pub plane_count: u8,
    pub width: u32,
    pub height: u32,
    pub drm_format: u32,
    pub modifier: u64,
    pub strides: [u32; 4],
    pub offsets: [u32; 4],
    pub device_uuid: [u8; 16],
    pub timestamp_us: i64,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureEncodeError {
    NoTexture,
    InvalidDimensions,
    UnsupportedFormat,
    InvalidPlanes,
    UnsupportedCodec,
    NoHardwareEncoder,
    SdkNativeTextureUnsupported,
}

impl TextureEncodeError {
    #[allow(dead_code)]
    pub fn as_str(self) -> &'static str {
        match self {
            TextureEncodeError::NoTexture => "noTexture",
            TextureEncodeError::InvalidDimensions => "invalidDimensions",
            TextureEncodeError::UnsupportedFormat => "unsupportedFormat",
            TextureEncodeError::InvalidPlanes => "invalidPlanes",
            TextureEncodeError::UnsupportedCodec => "unsupportedCodec",
            TextureEncodeError::NoHardwareEncoder => "noHardwareEncoder",
            TextureEncodeError::SdkNativeTextureUnsupported => "sdkNativeTextureUnsupported",
        }
    }
}

#[cfg(any(target_os = "windows", test))]
const DXGI_FORMAT_R8G8B8A8_UNORM: u32 = 28;
#[cfg(any(target_os = "windows", test))]
const DXGI_FORMAT_R8G8B8A8_UNORM_SRGB: u32 = 29;
#[cfg(any(target_os = "windows", test))]
const DXGI_FORMAT_B8G8R8A8_UNORM: u32 = 87;
#[cfg(any(target_os = "windows", test))]
const DXGI_FORMAT_B8G8R8A8_UNORM_SRGB: u32 = 91;
#[cfg(any(target_os = "windows", test))]
const DXGI_FORMAT_NV12: u32 = 103;

#[cfg(any(target_os = "windows", test))]
pub fn dxgi_format_supported(dxgi_format: u32) -> bool {
    matches!(
        dxgi_format,
        DXGI_FORMAT_R8G8B8A8_UNORM
            | DXGI_FORMAT_R8G8B8A8_UNORM_SRGB
            | DXGI_FORMAT_B8G8R8A8_UNORM
            | DXGI_FORMAT_B8G8R8A8_UNORM_SRGB
            | DXGI_FORMAT_NV12
    )
}

#[cfg(any(target_os = "linux", test))]
const fn fourcc(bytes: [u8; 4]) -> u32 {
    u32::from_le_bytes(bytes)
}

#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_XRGB8888: u32 = fourcc(*b"XR24");
#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_ARGB8888: u32 = fourcc(*b"AR24");
#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_XBGR8888: u32 = fourcc(*b"XB24");
#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_ABGR8888: u32 = fourcc(*b"AB24");
#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_XRGB2101010: u32 = fourcc(*b"XR30");
#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_ARGB2101010: u32 = fourcc(*b"AR30");
#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_XBGR2101010: u32 = fourcc(*b"XB30");
#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_ABGR2101010: u32 = fourcc(*b"AB30");
#[cfg(any(target_os = "linux", test))]
const DRM_FORMAT_NV12: u32 = fourcc(*b"NV12");

#[cfg(any(target_os = "linux", test))]
pub fn drm_format_supported(drm_format: u32) -> bool {
    matches!(
        drm_format,
        DRM_FORMAT_XRGB8888
            | DRM_FORMAT_ARGB8888
            | DRM_FORMAT_XBGR8888
            | DRM_FORMAT_ABGR8888
            | DRM_FORMAT_XRGB2101010
            | DRM_FORMAT_ARGB2101010
            | DRM_FORMAT_XBGR2101010
            | DRM_FORMAT_ABGR2101010
            | DRM_FORMAT_NV12
    )
}

#[cfg(any(target_os = "linux", target_os = "windows", test))]
const MAX_TEXTURE_EDGE: u32 = 8192;

#[cfg(any(target_os = "linux", target_os = "windows", test))]
fn validate_dimensions(width: u32, height: u32) -> Result<(), TextureEncodeError> {
    if width < 2
        || height < 2
        || !width.is_multiple_of(2)
        || !height.is_multiple_of(2)
        || width > MAX_TEXTURE_EDGE
        || height > MAX_TEXTURE_EDGE
    {
        return Err(TextureEncodeError::InvalidDimensions);
    }
    Ok(())
}

#[cfg(any(target_os = "windows", test))]
pub fn validate_texture_desc(desc: &TextureFrameDesc) -> Result<(), TextureEncodeError> {
    if desc.handle == 0 {
        return Err(TextureEncodeError::NoTexture);
    }
    validate_dimensions(desc.width, desc.height)?;
    if !dxgi_format_supported(desc.dxgi_format) {
        return Err(TextureEncodeError::UnsupportedFormat);
    }
    Ok(())
}

#[cfg(any(target_os = "linux", test))]
pub fn validate_dmabuf_desc(desc: &DmabufFrameDesc) -> Result<(), TextureEncodeError> {
    validate_dimensions(desc.width, desc.height)?;
    if !drm_format_supported(desc.drm_format) {
        return Err(TextureEncodeError::UnsupportedFormat);
    }
    let plane_count = desc.plane_count as usize;
    if !(1..=4).contains(&plane_count) {
        return Err(TextureEncodeError::InvalidPlanes);
    }
    for plane in 0..plane_count {
        if desc.strides[plane] == 0 {
            return Err(TextureEncodeError::InvalidPlanes);
        }
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub fn dmabuf_desc_from_parts(
    fds: &[i32],
    plane_count: u32,
    width: u32,
    height: u32,
    drm_format: u32,
    modifier: u64,
    strides: &[u32],
    offsets: &[u32],
    device_uuid: &[u8],
    timestamp_us: f64,
) -> Option<(DmabufFrameDesc, [i32; 4])> {
    let plane_count_u8 = u8::try_from(plane_count).ok()?;
    let planes = plane_count as usize;
    if !(1..=4).contains(&planes)
        || fds.len() < planes
        || strides.len() < planes
        || offsets.len() < planes
        || device_uuid.len() != 16
    {
        return None;
    }
    if fds.iter().take(planes).any(|fd| *fd < 0) {
        return None;
    }

    let mut fd_array = [-1; 4];
    let mut stride_array = [0; 4];
    let mut offset_array = [0; 4];
    fd_array[..planes].copy_from_slice(&fds[..planes]);
    stride_array[..planes].copy_from_slice(&strides[..planes]);
    offset_array[..planes].copy_from_slice(&offsets[..planes]);
    let mut uuid = [0u8; 16];
    uuid.copy_from_slice(device_uuid);

    Some((
        DmabufFrameDesc {
            plane_count: plane_count_u8,
            width,
            height,
            drm_format,
            modifier,
            strides: stride_array,
            offsets: offset_array,
            device_uuid: uuid,
            timestamp_us: timestamp_us as i64,
        },
        fd_array,
    ))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureCapability {
    pub available: bool,
    pub reason: TextureEncodeError,
}

impl TextureCapability {
    pub fn unavailable(reason: TextureEncodeError) -> Self {
        Self {
            available: false,
            reason,
        }
    }

    pub fn for_screen_codec(codec: &str, has_hardware_encoder: bool) -> Self {
        if !codec_allows_native_gpu(codec) {
            return Self::unavailable(TextureEncodeError::UnsupportedCodec);
        }
        if !has_hardware_encoder {
            return Self {
                available: false,
                reason: TextureEncodeError::NoHardwareEncoder,
            };
        }
        Self {
            available: true,
            reason: TextureEncodeError::NoTexture,
        }
    }
}

pub(crate) fn codec_allows_native_gpu(codec: &str) -> bool {
    matches!(
        codec.trim().to_ascii_lowercase().as_str(),
        "h264" | "h265" | "hevc"
    )
}

#[cfg(any(target_os = "windows", test))]
fn sdk_accepts_d3d11_texture_buffers() -> bool {
    true
}

#[cfg(target_os = "linux")]
fn sdk_accepts_dmabuf_texture_buffers() -> bool {
    cfg!(target_os = "linux")
}

#[cfg(any(target_os = "windows", test))]
pub fn should_attempt_texture_encode(
    capability: &TextureCapability,
    desc: &TextureFrameDesc,
) -> Result<(), TextureEncodeError> {
    if !capability.available {
        return Err(capability.reason);
    }
    if !sdk_accepts_d3d11_texture_buffers() {
        return Err(TextureEncodeError::SdkNativeTextureUnsupported);
    }
    validate_texture_desc(desc)
}

#[cfg(target_os = "linux")]
pub fn should_attempt_dmabuf_encode(
    capability: &TextureCapability,
    desc: &DmabufFrameDesc,
) -> Result<(), TextureEncodeError> {
    if !capability.available {
        return Err(capability.reason);
    }
    if !sdk_accepts_dmabuf_texture_buffers() {
        return Err(TextureEncodeError::SdkNativeTextureUnsupported);
    }
    validate_dmabuf_desc(desc)
}

#[cfg(test)]
pub fn should_attempt_texture_encode_for_tests(
    capability: &TextureCapability,
    desc: &TextureFrameDesc,
    sdk_accepts_d3d11: bool,
) -> Result<(), TextureEncodeError> {
    if !capability.available {
        return Err(capability.reason);
    }
    if !sdk_accepts_d3d11 {
        return Err(TextureEncodeError::SdkNativeTextureUnsupported);
    }
    validate_texture_desc(desc)
}

#[cfg(test)]
pub fn should_attempt_dmabuf_encode_for_tests(
    capability: &TextureCapability,
    desc: &DmabufFrameDesc,
    sdk_accepts_dmabuf: bool,
) -> Result<(), TextureEncodeError> {
    if !capability.available {
        return Err(capability.reason);
    }
    if !sdk_accepts_dmabuf {
        return Err(TextureEncodeError::SdkNativeTextureUnsupported);
    }
    validate_dmabuf_desc(desc)
}

#[cfg(all(feature = "publisher", any(target_os = "linux", target_os = "windows")))]
pub mod bridge {
    #[cfg(target_os = "linux")]
    use super::{DmabufFrameDesc, should_attempt_dmabuf_encode};
    use super::{TextureCapability, TextureEncodeError};
    #[cfg(target_os = "windows")]
    use super::{TextureFrameDesc, should_attempt_texture_encode};
    use livekit::webrtc::video_frame::{VideoFrame, VideoRotation, native::NativeBuffer};
    use livekit::webrtc::video_source::native::NativeVideoSource;

    #[cfg(target_os = "windows")]
    pub fn try_publish_texture(
        source: &NativeVideoSource,
        capability: &TextureCapability,
        desc: &TextureFrameDesc,
    ) -> Result<(), TextureEncodeError> {
        should_attempt_texture_encode(capability, desc)?;

        let Some(buffer) = NativeBuffer::from_fluxer_d3d11_texture(
            desc.handle,
            desc.width,
            desc.height,
            desc.dxgi_format,
        ) else {
            return Err(TextureEncodeError::SdkNativeTextureUnsupported);
        };
        publish_native_buffer(source, buffer, desc.timestamp_us);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    pub fn try_publish_dmabuf(
        source: &NativeVideoSource,
        capability: &TextureCapability,
        desc: &DmabufFrameDesc,
        fds: [i32; 4],
    ) -> Result<(), TextureEncodeError> {
        should_attempt_dmabuf_encode(capability, desc)?;
        let uuid_hi = u64::from_be_bytes(desc.device_uuid[0..8].try_into().unwrap_or([0; 8]));
        let uuid_lo = u64::from_be_bytes(desc.device_uuid[8..16].try_into().unwrap_or([0; 8]));
        let Some(buffer) = NativeBuffer::from_fluxer_dmabuf_texture(
            fds,
            desc.plane_count as u32,
            desc.width,
            desc.height,
            desc.drm_format,
            desc.modifier,
            desc.strides,
            desc.offsets,
            uuid_hi,
            uuid_lo,
        ) else {
            return Err(TextureEncodeError::SdkNativeTextureUnsupported);
        };
        publish_native_buffer(source, buffer, desc.timestamp_us);
        Ok(())
    }

    fn publish_native_buffer(source: &NativeVideoSource, buffer: NativeBuffer, timestamp_us: i64) {
        source.capture_frame(&VideoFrame {
            rotation: VideoRotation::VideoRotation0,
            timestamp_us,
            frame_metadata: None,
            buffer,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_desc() -> TextureFrameDesc {
        TextureFrameDesc {
            handle: 0xDEAD_BEEF,
            width: 1920,
            height: 1080,
            dxgi_format: DXGI_FORMAT_B8G8R8A8_UNORM,
            timestamp_us: 123_456,
        }
    }

    #[test]
    fn dxgi_format_gate_accepts_8bit_rgba_bgra_only() {
        assert!(dxgi_format_supported(DXGI_FORMAT_B8G8R8A8_UNORM));
        assert!(dxgi_format_supported(DXGI_FORMAT_B8G8R8A8_UNORM_SRGB));
        assert!(dxgi_format_supported(DXGI_FORMAT_R8G8B8A8_UNORM));
        assert!(dxgi_format_supported(DXGI_FORMAT_R8G8B8A8_UNORM_SRGB));
        assert!(dxgi_format_supported(DXGI_FORMAT_NV12));
        assert!(!dxgi_format_supported(24));
        assert!(!dxgi_format_supported(10));
        assert!(!dxgi_format_supported(0));
    }

    #[test]
    fn drm_format_gate_accepts_obs_vkcapture_texture_formats() {
        assert!(drm_format_supported(DRM_FORMAT_ARGB8888));
        assert!(drm_format_supported(DRM_FORMAT_ABGR8888));
        assert!(drm_format_supported(DRM_FORMAT_ARGB2101010));
        assert!(drm_format_supported(DRM_FORMAT_ABGR2101010));
        assert!(drm_format_supported(DRM_FORMAT_NV12));
        assert!(
            !drm_format_supported(fourcc(*b"AB4H")),
            "16-bit float DMA-BUF import is not encodable by the native NVENC bridge yet"
        );
        assert!(!drm_format_supported(0));
    }

    #[test]
    fn validate_rejects_zero_handle_as_no_texture() {
        let mut d = good_desc();
        d.handle = 0;
        assert_eq!(
            validate_texture_desc(&d),
            Err(TextureEncodeError::NoTexture)
        );
    }

    #[test]
    fn validate_rejects_odd_zero_and_oversized_dims() {
        for (w, h) in [
            (1921, 1080),
            (1920, 1081),
            (0, 1080),
            (1920, 0),
            (8194, 1080),
            (1920, 8194),
        ] {
            let mut d = good_desc();
            d.width = w;
            d.height = h;
            assert_eq!(
                validate_texture_desc(&d),
                Err(TextureEncodeError::InvalidDimensions),
                "dims {w}x{h} should be rejected"
            );
        }
    }

    #[test]
    fn validate_rejects_unsupported_format() {
        let mut d = good_desc();
        d.dxgi_format = 24;
        assert_eq!(
            validate_texture_desc(&d),
            Err(TextureEncodeError::UnsupportedFormat)
        );
    }

    #[test]
    fn validate_accepts_a_clean_bgra_texture() {
        assert_eq!(validate_texture_desc(&good_desc()), Ok(()));
    }

    fn good_dmabuf_desc() -> DmabufFrameDesc {
        DmabufFrameDesc {
            plane_count: 1,
            width: 1920,
            height: 1080,
            drm_format: DRM_FORMAT_ARGB8888,
            modifier: 0,
            strides: [1920 * 4, 0, 0, 0],
            offsets: [0, 0, 0, 0],
            device_uuid: [1; 16],
            timestamp_us: 123_456,
        }
    }

    #[test]
    fn validate_dmabuf_accepts_supported_formats_with_optional_uuid() {
        assert_eq!(validate_dmabuf_desc(&good_dmabuf_desc()), Ok(()));
        let mut desc = good_dmabuf_desc();
        desc.drm_format = DRM_FORMAT_NV12;
        desc.strides[0] = 1920;
        assert_eq!(validate_dmabuf_desc(&desc), Ok(()));
        desc = good_dmabuf_desc();
        desc.device_uuid = [0; 16];
        assert_eq!(validate_dmabuf_desc(&desc), Ok(()));
    }

    #[test]
    fn validate_dmabuf_rejects_invalid_planes() {
        let mut desc = good_dmabuf_desc();
        desc.plane_count = 0;
        assert_eq!(
            validate_dmabuf_desc(&desc),
            Err(TextureEncodeError::InvalidPlanes)
        );
        desc = good_dmabuf_desc();
        desc.strides[0] = 0;
        assert_eq!(
            validate_dmabuf_desc(&desc),
            Err(TextureEncodeError::InvalidPlanes)
        );
    }

    #[test]
    fn dmabuf_desc_from_parts_rejects_negative_fds() {
        assert!(
            dmabuf_desc_from_parts(
                &[-1],
                1,
                1920,
                1080,
                DRM_FORMAT_ARGB8888,
                0,
                &[1920 * 4],
                &[0],
                &[1; 16],
                123.0,
            )
            .is_none()
        );
    }

    #[test]
    fn dmabuf_desc_from_parts_populates_all_plane_arrays() {
        let uuid = [7u8; 16];
        let (desc, fds) = dmabuf_desc_from_parts(
            &[10, 11, 12, 99],
            3,
            1920,
            1080,
            DRM_FORMAT_NV12,
            0xABCD,
            &[1920, 960, 960, 777],
            &[0, 2_073_600, 3_110_400, 999],
            &uuid,
            123_456.75,
        )
        .expect("valid multi-plane descriptor");

        assert_eq!(desc.plane_count, 3);
        assert_eq!(desc.width, 1920);
        assert_eq!(desc.height, 1080);
        assert_eq!(desc.drm_format, DRM_FORMAT_NV12);
        assert_eq!(desc.modifier, 0xABCD);
        assert_eq!(desc.strides, [1920, 960, 960, 0]);
        assert_eq!(desc.offsets, [0, 2_073_600, 3_110_400, 0]);
        assert_eq!(desc.device_uuid, uuid);
        assert_eq!(desc.timestamp_us, 123_456);
        assert_eq!(fds, [10, 11, 12, -1]);
    }

    #[test]
    fn dmabuf_desc_from_parts_rejects_incomplete_native_inputs() {
        let uuid = [1u8; 16];
        for (fds, strides, offsets, uuid_bytes, label) in [
            (&[4][..], &[128][..], &[0][..], &uuid[..], "too few fds"),
            (
                &[4, 5][..],
                &[128][..],
                &[0, 64][..],
                &uuid[..],
                "too few strides",
            ),
            (
                &[4, 5][..],
                &[128, 128][..],
                &[0][..],
                &uuid[..],
                "too few offsets",
            ),
            (
                &[4, 5][..],
                &[128, 128][..],
                &[0, 64][..],
                &[1u8; 15][..],
                "bad uuid",
            ),
        ] {
            assert!(
                dmabuf_desc_from_parts(
                    fds,
                    2,
                    128,
                    128,
                    DRM_FORMAT_ARGB8888,
                    0,
                    strides,
                    offsets,
                    uuid_bytes,
                    0.0,
                )
                .is_none(),
                "{label} should be rejected"
            );
        }
        assert!(
            dmabuf_desc_from_parts(
                &[4, 5, 6, 7, 8],
                5,
                128,
                128,
                DRM_FORMAT_ARGB8888,
                0,
                &[128; 5],
                &[0; 5],
                &uuid,
                0.0,
            )
            .is_none()
        );
    }

    #[test]
    fn validate_precedence_handle_before_dims_before_format() {
        let d = TextureFrameDesc {
            handle: 0,
            width: 1921,
            height: 0,
            dxgi_format: 999,
            timestamp_us: 0,
        };
        assert_eq!(
            validate_texture_desc(&d),
            Err(TextureEncodeError::NoTexture)
        );
    }

    #[test]
    fn probe_is_available_only_for_explicit_hardware_codecs() {
        for codec in ["h264", "H264", "h265", "hevc", "HEVC"] {
            let cap = TextureCapability::for_screen_codec(codec, true);
            assert!(cap.available, "{codec} should allow native GPU buffers");
            assert_eq!(cap.reason, TextureEncodeError::NoTexture);
        }
        for codec in ["", "vp8", "vp9", "av1", "rubbish"] {
            let cap = TextureCapability::for_screen_codec(codec, true);
            assert!(
                !cap.available,
                "{codec} should not allow native GPU buffers"
            );
            assert_eq!(cap.reason, TextureEncodeError::UnsupportedCodec);
        }
        let cap = TextureCapability::for_screen_codec("h264", false);
        assert!(!cap.available);
        assert_eq!(cap.reason, TextureEncodeError::NoHardwareEncoder);
    }

    #[test]
    fn should_attempt_falls_back_when_capability_unavailable() {
        let cap = TextureCapability {
            available: false,
            reason: TextureEncodeError::SdkNativeTextureUnsupported,
        };
        assert_eq!(
            should_attempt_texture_encode(&cap, &good_desc()),
            Err(TextureEncodeError::SdkNativeTextureUnsupported)
        );
    }

    #[test]
    fn should_attempt_validates_frame_when_capability_available() {
        let open = TextureCapability {
            available: true,
            reason: TextureEncodeError::NoTexture,
        };
        assert_eq!(
            should_attempt_texture_encode_for_tests(&open, &good_desc(), true),
            Ok(())
        );
        assert_eq!(should_attempt_texture_encode(&open, &good_desc()), Ok(()));
        let mut bad = good_desc();
        bad.handle = 0;
        assert_eq!(
            should_attempt_texture_encode_for_tests(&open, &bad, true),
            Err(TextureEncodeError::NoTexture)
        );
        bad = good_desc();
        bad.dxgi_format = 24;
        assert_eq!(
            should_attempt_texture_encode_for_tests(&open, &bad, true),
            Err(TextureEncodeError::UnsupportedFormat)
        );
    }

    #[test]
    fn should_attempt_dmabuf_is_sdk_gated_after_validation_capability() {
        let open = TextureCapability {
            available: true,
            reason: TextureEncodeError::NoTexture,
        };
        assert_eq!(
            should_attempt_dmabuf_encode_for_tests(&open, &good_dmabuf_desc(), true),
            Ok(())
        );
        assert_eq!(
            should_attempt_dmabuf_encode_for_tests(&open, &good_dmabuf_desc(), false),
            Err(TextureEncodeError::SdkNativeTextureUnsupported)
        );
        let mut invalid = good_dmabuf_desc();
        invalid.plane_count = 5;
        assert_eq!(
            should_attempt_dmabuf_encode_for_tests(&open, &invalid, true),
            Err(TextureEncodeError::InvalidPlanes)
        );
    }

    #[test]
    fn error_strings_are_stable() {
        assert_eq!(TextureEncodeError::NoTexture.as_str(), "noTexture");
        assert_eq!(
            TextureEncodeError::InvalidDimensions.as_str(),
            "invalidDimensions"
        );
        assert_eq!(
            TextureEncodeError::UnsupportedFormat.as_str(),
            "unsupportedFormat"
        );
        assert_eq!(TextureEncodeError::InvalidPlanes.as_str(), "invalidPlanes");
        assert_eq!(
            TextureEncodeError::UnsupportedCodec.as_str(),
            "unsupportedCodec"
        );
        assert_eq!(
            TextureEncodeError::NoHardwareEncoder.as_str(),
            "noHardwareEncoder"
        );
        assert_eq!(
            TextureEncodeError::SdkNativeTextureUnsupported.as_str(),
            "sdkNativeTextureUnsupported"
        );
    }
}
