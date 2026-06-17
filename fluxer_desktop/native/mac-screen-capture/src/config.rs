// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

pub const FPS_MIN: u32 = 1;
pub const FPS_MAX: u32 = 120;
pub const MAX_OUTPUT_WIDTH_DEFAULT: u32 = 3840;
pub const MAX_OUTPUT_HEIGHT_DEFAULT: u32 = 2160;
pub const OUTPUT_DIMENSION_MIN: u32 = 2;
pub const QUEUE_DEPTH_MIN: u32 = 1;
pub const QUEUE_DEPTH_MAX: u32 = 16;
pub const QUEUE_DEPTH_DEFAULT: u32 = 8;
pub const FPS_DEFAULT: u32 = 30;
pub const FRAME_INTERVAL_FACTOR_NUM: u64 = 9;
pub const FRAME_INTERVAL_FACTOR_DEN: u64 = 10;

pub const PIXEL_FORMAT_BGRA_FOURCC: u32 = u32::from_be_bytes(*b"BGRA");
pub const PIXEL_FORMAT_L10R_FOURCC: u32 = u32::from_be_bytes(*b"l10r");
pub const PIXEL_FORMAT_420V_FOURCC: u32 = u32::from_be_bytes(*b"420v");
pub const PIXEL_FORMAT_420F_FOURCC: u32 = u32::from_be_bytes(*b"420f");

pub const AUDIO_SAMPLE_RATE_DEFAULT_HZ: u32 = 48_000;
pub const AUDIO_CHANNEL_COUNT_DEFAULT: u32 = 2;
pub const AUDIO_SAMPLE_RATE_MIN_HZ: u32 = 8_000;
pub const AUDIO_SAMPLE_RATE_MAX_HZ: u32 = 192_000;
pub const AUDIO_CHANNEL_COUNT_MIN: u32 = 1;
pub const AUDIO_CHANNEL_COUNT_MAX: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SckPixelFormat {
    Bgra8,
    L10rHdr,
    Nv12VideoRange,
    Nv12FullRange,
}

impl SckPixelFormat {
    pub fn as_fourcc(self) -> u32 {
        let value = match self {
            SckPixelFormat::Bgra8 => PIXEL_FORMAT_BGRA_FOURCC,
            SckPixelFormat::L10rHdr => PIXEL_FORMAT_L10R_FOURCC,
            SckPixelFormat::Nv12VideoRange => PIXEL_FORMAT_420V_FOURCC,
            SckPixelFormat::Nv12FullRange => PIXEL_FORMAT_420F_FOURCC,
        };
        assert!(value != 0);
        value
    }

    pub fn is_hdr(self) -> bool {
        matches!(self, SckPixelFormat::L10rHdr)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SckColorSpace {
    DisplayP3,
    SrgbBt709,
}

impl SckColorSpace {
    pub fn as_cf_name(self) -> &'static str {
        let name = match self {
            SckColorSpace::DisplayP3 => "kCGColorSpaceDisplayP3",
            SckColorSpace::SrgbBt709 => "kCGColorSpaceSRGB",
        };
        assert!(!name.is_empty());
        name
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SckError {
    InvalidFps(u32),
    InvalidQueueDepth(u32),
    HdrRequiresWideColorSpace,
    InvalidAudioSampleRate(u32),
    InvalidAudioChannelCount(u32),
}

impl fmt::Display for SckError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SckError::InvalidFps(v) => write!(
                f,
                "SckCaptureConfig: target_fps={v} out of range [{FPS_MIN}..={FPS_MAX}]"
            ),
            SckError::InvalidQueueDepth(v) => write!(
                f,
                "SckCaptureConfig: queue_depth={v} out of range [{QUEUE_DEPTH_MIN}..={QUEUE_DEPTH_MAX}]"
            ),
            SckError::HdrRequiresWideColorSpace => write!(
                f,
                "SckCaptureConfig: l10r HDR pixel format requires DisplayP3 color space"
            ),
            SckError::InvalidAudioSampleRate(v) => write!(
                f,
                "SckCaptureConfig: audio_sample_rate_hz={v} out of range [{AUDIO_SAMPLE_RATE_MIN_HZ}..={AUDIO_SAMPLE_RATE_MAX_HZ}]"
            ),
            SckError::InvalidAudioChannelCount(v) => write!(
                f,
                "SckCaptureConfig: audio_channels={v} out of range [{AUDIO_CHANNEL_COUNT_MIN}..={AUDIO_CHANNEL_COUNT_MAX}]"
            ),
        }
    }
}

impl std::error::Error for SckError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SckCaptureConfig {
    target_fps: u32,
    queue_depth: u32,
    pixel_format: SckPixelFormat,
    color_space: SckColorSpace,
    captures_audio: bool,
    audio_sample_rate_hz: u32,
    audio_channels: u32,
}

impl SckCaptureConfig {
    pub fn new(
        target_fps: u32,
        queue_depth: u32,
        pixel_format: SckPixelFormat,
        color_space: SckColorSpace,
    ) -> Result<Self, SckError> {
        Self::new_with_audio(
            target_fps,
            queue_depth,
            pixel_format,
            color_space,
            false,
            AUDIO_SAMPLE_RATE_DEFAULT_HZ,
            AUDIO_CHANNEL_COUNT_DEFAULT,
        )
    }

    pub fn new_with_audio(
        target_fps: u32,
        queue_depth: u32,
        pixel_format: SckPixelFormat,
        color_space: SckColorSpace,
        captures_audio: bool,
        audio_sample_rate_hz: u32,
        audio_channels: u32,
    ) -> Result<Self, SckError> {
        if !(FPS_MIN..=FPS_MAX).contains(&target_fps) {
            return Err(SckError::InvalidFps(target_fps));
        }
        if !(QUEUE_DEPTH_MIN..=QUEUE_DEPTH_MAX).contains(&queue_depth) {
            return Err(SckError::InvalidQueueDepth(queue_depth));
        }
        if pixel_format.is_hdr() && color_space != SckColorSpace::DisplayP3 {
            return Err(SckError::HdrRequiresWideColorSpace);
        }
        if !(AUDIO_SAMPLE_RATE_MIN_HZ..=AUDIO_SAMPLE_RATE_MAX_HZ).contains(&audio_sample_rate_hz) {
            return Err(SckError::InvalidAudioSampleRate(audio_sample_rate_hz));
        }
        if !(AUDIO_CHANNEL_COUNT_MIN..=AUDIO_CHANNEL_COUNT_MAX).contains(&audio_channels) {
            return Err(SckError::InvalidAudioChannelCount(audio_channels));
        }
        let cfg = Self {
            target_fps,
            queue_depth,
            pixel_format,
            color_space,
            captures_audio,
            audio_sample_rate_hz,
            audio_channels,
        };
        assert!(cfg.target_fps >= FPS_MIN);
        assert!(cfg.target_fps <= FPS_MAX);
        assert!(cfg.queue_depth >= QUEUE_DEPTH_MIN);
        assert!(cfg.queue_depth <= QUEUE_DEPTH_MAX);
        assert!(cfg.audio_sample_rate_hz >= AUDIO_SAMPLE_RATE_MIN_HZ);
        assert!(cfg.audio_channels >= AUDIO_CHANNEL_COUNT_MIN);
        Ok(cfg)
    }

    pub fn builder() -> SckCaptureConfigBuilder {
        SckCaptureConfigBuilder::default()
    }

    pub fn target_fps(&self) -> u32 {
        assert!(self.target_fps >= FPS_MIN);
        assert!(self.target_fps <= FPS_MAX);
        self.target_fps
    }

    pub fn queue_depth(&self) -> u32 {
        assert!(self.queue_depth >= QUEUE_DEPTH_MIN);
        assert!(self.queue_depth <= QUEUE_DEPTH_MAX);
        self.queue_depth
    }

    pub fn pixel_format(&self) -> SckPixelFormat {
        let pf = self.pixel_format;
        assert!(pf.as_fourcc() != 0);
        pf
    }

    pub fn color_space(&self) -> SckColorSpace {
        let cs = self.color_space;
        assert!(!cs.as_cf_name().is_empty());
        cs
    }

    pub fn minimum_frame_interval_ns(&self) -> u64 {
        assert!(self.target_fps >= FPS_MIN);
        assert!(self.target_fps <= FPS_MAX);
        let base_ns: u64 = 1_000_000_000 / (self.target_fps as u64);
        let scaled = base_ns * FRAME_INTERVAL_FACTOR_NUM / FRAME_INTERVAL_FACTOR_DEN;
        assert!(scaled > 0);
        assert!(scaled <= 1_000_000_000);
        scaled
    }

    pub fn captures_audio(&self) -> bool {
        self.captures_audio
    }

    pub fn audio_sample_rate_hz(&self) -> u32 {
        assert!(self.audio_sample_rate_hz >= AUDIO_SAMPLE_RATE_MIN_HZ);
        assert!(self.audio_sample_rate_hz <= AUDIO_SAMPLE_RATE_MAX_HZ);
        self.audio_sample_rate_hz
    }

    pub fn audio_channels(&self) -> u32 {
        assert!(self.audio_channels >= AUDIO_CHANNEL_COUNT_MIN);
        assert!(self.audio_channels <= AUDIO_CHANNEL_COUNT_MAX);
        self.audio_channels
    }
}

impl Default for SckCaptureConfig {
    fn default() -> Self {
        let cfg = Self::new_with_audio(
            FPS_DEFAULT,
            QUEUE_DEPTH_DEFAULT,
            SckPixelFormat::Nv12VideoRange,
            SckColorSpace::SrgbBt709,
            false,
            AUDIO_SAMPLE_RATE_DEFAULT_HZ,
            AUDIO_CHANNEL_COUNT_DEFAULT,
        );
        match cfg {
            Ok(c) => c,
            Err(_) => unreachable!("default SckCaptureConfig must validate"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SckCaptureConfigBuilder {
    target_fps: u32,
    queue_depth: u32,
    pixel_format: SckPixelFormat,
    color_space: SckColorSpace,
    captures_audio: bool,
    audio_sample_rate_hz: u32,
    audio_channels: u32,
}

impl Default for SckCaptureConfigBuilder {
    fn default() -> Self {
        Self {
            target_fps: FPS_DEFAULT,
            queue_depth: QUEUE_DEPTH_DEFAULT,
            pixel_format: SckPixelFormat::Nv12VideoRange,
            color_space: SckColorSpace::SrgbBt709,
            captures_audio: false,
            audio_sample_rate_hz: AUDIO_SAMPLE_RATE_DEFAULT_HZ,
            audio_channels: AUDIO_CHANNEL_COUNT_DEFAULT,
        }
    }
}

impl SckCaptureConfigBuilder {
    pub fn target_fps(mut self, target_fps: u32) -> Self {
        self.target_fps = target_fps;
        self
    }

    pub fn queue_depth(mut self, queue_depth: u32) -> Self {
        self.queue_depth = queue_depth;
        self
    }

    pub fn pixel_format(mut self, pixel_format: SckPixelFormat) -> Self {
        self.pixel_format = pixel_format;
        self
    }

    pub fn color_space(mut self, color_space: SckColorSpace) -> Self {
        self.color_space = color_space;
        self
    }

    pub fn captures_audio(mut self, captures_audio: bool) -> Self {
        self.captures_audio = captures_audio;
        self
    }

    pub fn audio_sample_rate_hz(mut self, audio_sample_rate_hz: u32) -> Self {
        self.audio_sample_rate_hz = audio_sample_rate_hz;
        self
    }

    pub fn audio_channels(mut self, audio_channels: u32) -> Self {
        self.audio_channels = audio_channels;
        self
    }

    pub fn build(self) -> Result<SckCaptureConfig, SckError> {
        SckCaptureConfig::new_with_audio(
            self.target_fps,
            self.queue_depth,
            self.pixel_format,
            self.color_space,
            self.captures_audio,
            self.audio_sample_rate_hz,
            self.audio_channels,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SckCaptureFailure {
    StreamStoppedWithError(String),
    StreamStartFailed(String),
    SystemDeniedAccess,
    DisplayDisconnected,
    Unknown(String),
}

impl SckCaptureFailure {
    pub fn reason(&self) -> &str {
        match self {
            SckCaptureFailure::StreamStoppedWithError(m) => m.as_str(),
            SckCaptureFailure::StreamStartFailed(m) => m.as_str(),
            SckCaptureFailure::SystemDeniedAccess => "screen recording permission denied",
            SckCaptureFailure::DisplayDisconnected => "captured display was disconnected",
            SckCaptureFailure::Unknown(m) => m.as_str(),
        }
    }
}

pub trait CaptureFailureSurface: Send + Sync {
    fn on_failure(&self, reason: SckCaptureFailure);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum AudioSampleFormat {
    F32Planar = 0,
    F32Interleaved = 1,
    I16Interleaved = 2,
    Unknown = 3,
}

pub const AUDIO_SAMPLE_FORMAT_CODE_MAX: u32 = 3;

impl AudioSampleFormat {
    pub fn code(self) -> u32 {
        let code = match self {
            AudioSampleFormat::F32Planar => 0,
            AudioSampleFormat::F32Interleaved => 1,
            AudioSampleFormat::I16Interleaved => 2,
            AudioSampleFormat::Unknown => 3,
        };
        assert!(code <= AUDIO_SAMPLE_FORMAT_CODE_MAX);
        assert_eq!(code, self as u32);
        code
    }

    pub fn as_str(self) -> &'static str {
        let s = match self {
            AudioSampleFormat::F32Planar => "f32_planar",
            AudioSampleFormat::F32Interleaved => "f32_interleaved",
            AudioSampleFormat::I16Interleaved => "i16_interleaved",
            AudioSampleFormat::Unknown => "unknown",
        };
        assert!(!s.is_empty());
        s
    }

    pub fn bytes_per_sample(self) -> u32 {
        let bytes = match self {
            AudioSampleFormat::F32Planar => 4,
            AudioSampleFormat::F32Interleaved => 4,
            AudioSampleFormat::I16Interleaved => 2,
            AudioSampleFormat::Unknown => 0,
        };
        assert!(bytes <= 4);
        bytes
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacScreenShareAudioFrame {
    pub sample_rate_hz: u32,
    pub channels: u32,
    pub num_samples_per_channel: u32,
    pub pts_us: i64,
}

impl MacScreenShareAudioFrame {
    pub fn new(
        sample_rate_hz: u32,
        channels: u32,
        num_samples_per_channel: u32,
        pts_us: i64,
    ) -> Result<Self, SckError> {
        if !(AUDIO_SAMPLE_RATE_MIN_HZ..=AUDIO_SAMPLE_RATE_MAX_HZ).contains(&sample_rate_hz) {
            return Err(SckError::InvalidAudioSampleRate(sample_rate_hz));
        }
        if !(AUDIO_CHANNEL_COUNT_MIN..=AUDIO_CHANNEL_COUNT_MAX).contains(&channels) {
            return Err(SckError::InvalidAudioChannelCount(channels));
        }
        assert!(sample_rate_hz >= AUDIO_SAMPLE_RATE_MIN_HZ);
        assert!(channels >= AUDIO_CHANNEL_COUNT_MIN);
        Ok(Self {
            sample_rate_hz,
            channels,
            num_samples_per_channel,
            pts_us,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MacScreenShareAudioFrameWithBytes {
    pub sample_rate_hz: u32,
    pub channels: u32,
    pub num_samples_per_channel: u32,
    pub pts_us: i64,
    pub format: AudioSampleFormat,
    pub samples: Vec<u8>,
}

impl MacScreenShareAudioFrameWithBytes {
    pub fn new(
        sample_rate_hz: u32,
        channels: u32,
        num_samples_per_channel: u32,
        pts_us: i64,
        format: AudioSampleFormat,
        samples: Vec<u8>,
    ) -> Result<Self, SckError> {
        if !(AUDIO_SAMPLE_RATE_MIN_HZ..=AUDIO_SAMPLE_RATE_MAX_HZ).contains(&sample_rate_hz) {
            return Err(SckError::InvalidAudioSampleRate(sample_rate_hz));
        }
        if !(AUDIO_CHANNEL_COUNT_MIN..=AUDIO_CHANNEL_COUNT_MAX).contains(&channels) {
            return Err(SckError::InvalidAudioChannelCount(channels));
        }
        assert!(sample_rate_hz >= AUDIO_SAMPLE_RATE_MIN_HZ);
        assert!(channels >= AUDIO_CHANNEL_COUNT_MIN);
        Ok(Self {
            sample_rate_hz,
            channels,
            num_samples_per_channel,
            pts_us,
            format,
            samples,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_defaults_produce_valid_config() {
        let cfg = SckCaptureConfig::builder()
            .build()
            .expect("default builder");
        assert_eq!(cfg.target_fps(), FPS_DEFAULT);
        assert_eq!(cfg.queue_depth(), QUEUE_DEPTH_DEFAULT);
        assert_eq!(cfg.pixel_format(), SckPixelFormat::Nv12VideoRange);
        assert_eq!(cfg.color_space(), SckColorSpace::SrgbBt709);
    }

    #[test]
    fn builder_rejects_fps_zero() {
        let err = SckCaptureConfig::builder()
            .target_fps(0)
            .build()
            .unwrap_err();
        assert_eq!(err, SckError::InvalidFps(0));
    }

    #[test]
    fn builder_rejects_fps_above_max() {
        let err = SckCaptureConfig::builder()
            .target_fps(FPS_MAX + 1)
            .build()
            .unwrap_err();
        assert_eq!(err, SckError::InvalidFps(FPS_MAX + 1));
    }

    #[test]
    fn builder_accepts_fps_120() {
        let cfg = SckCaptureConfig::builder()
            .target_fps(FPS_MAX)
            .build()
            .expect("120 fps ok");
        assert_eq!(cfg.target_fps(), FPS_MAX);
    }

    #[test]
    fn builder_rejects_queue_depth_zero() {
        let err = SckCaptureConfig::builder()
            .queue_depth(0)
            .build()
            .unwrap_err();
        assert_eq!(err, SckError::InvalidQueueDepth(0));
    }

    #[test]
    fn builder_rejects_queue_depth_above_max() {
        let err = SckCaptureConfig::builder()
            .queue_depth(QUEUE_DEPTH_MAX + 1)
            .build()
            .unwrap_err();
        assert_eq!(err, SckError::InvalidQueueDepth(QUEUE_DEPTH_MAX + 1));
    }

    #[test]
    fn builder_rejects_hdr_without_displayp3() {
        let err = SckCaptureConfig::builder()
            .pixel_format(SckPixelFormat::L10rHdr)
            .color_space(SckColorSpace::SrgbBt709)
            .build()
            .unwrap_err();
        assert_eq!(err, SckError::HdrRequiresWideColorSpace);
    }

    #[test]
    fn builder_accepts_hdr_with_displayp3() {
        let cfg = SckCaptureConfig::builder()
            .pixel_format(SckPixelFormat::L10rHdr)
            .color_space(SckColorSpace::DisplayP3)
            .build()
            .expect("hdr with p3 ok");
        assert!(cfg.pixel_format().is_hdr());
        assert_eq!(cfg.color_space(), SckColorSpace::DisplayP3);
    }

    #[test]
    fn minimum_frame_interval_60_fps_is_15_ms() {
        let cfg = SckCaptureConfig::builder()
            .target_fps(60)
            .build()
            .expect("60 fps");
        let ns = cfg.minimum_frame_interval_ns();
        let expected = (1_000_000_000_u64 / 60) * 9 / 10;
        assert_eq!(ns, expected);
        assert!((14_000_000..=16_000_000).contains(&ns));
    }

    #[test]
    fn minimum_frame_interval_30_fps_factor_applied() {
        let cfg = SckCaptureConfig::builder()
            .target_fps(30)
            .build()
            .expect("30 fps");
        let ns = cfg.minimum_frame_interval_ns();
        let base = 1_000_000_000_u64 / 30;
        assert_eq!(ns, base * 9 / 10);
        assert!(ns < base);
    }

    #[test]
    fn minimum_frame_interval_120_fps_under_8_3ms() {
        let cfg = SckCaptureConfig::builder()
            .target_fps(120)
            .build()
            .expect("120 fps");
        let ns = cfg.minimum_frame_interval_ns();
        assert!(ns < 8_400_000);
        assert!(ns > 6_000_000);
    }

    #[test]
    fn pixel_format_fourcc_matches_obs_constants() {
        assert_eq!(SckPixelFormat::Bgra8.as_fourcc(), PIXEL_FORMAT_BGRA_FOURCC);
        assert_eq!(
            SckPixelFormat::L10rHdr.as_fourcc(),
            PIXEL_FORMAT_L10R_FOURCC
        );
        assert_eq!(
            SckPixelFormat::Nv12VideoRange.as_fourcc(),
            PIXEL_FORMAT_420V_FOURCC
        );
        assert_eq!(
            SckPixelFormat::Nv12FullRange.as_fourcc(),
            PIXEL_FORMAT_420F_FOURCC
        );
    }

    #[test]
    fn color_space_names_present() {
        assert_eq!(
            SckColorSpace::DisplayP3.as_cf_name(),
            "kCGColorSpaceDisplayP3"
        );
        assert_eq!(SckColorSpace::SrgbBt709.as_cf_name(), "kCGColorSpaceSRGB");
    }

    #[test]
    fn failure_reason_strings_non_empty() {
        let f = SckCaptureFailure::StreamStoppedWithError("oops".into());
        assert_eq!(f.reason(), "oops");
        let f = SckCaptureFailure::SystemDeniedAccess;
        assert!(!f.reason().is_empty());
        let f = SckCaptureFailure::DisplayDisconnected;
        assert!(!f.reason().is_empty());
    }

    #[test]
    fn default_config_validates() {
        let cfg = SckCaptureConfig::default();
        assert_eq!(cfg.target_fps(), FPS_DEFAULT);
        assert_eq!(cfg.queue_depth(), QUEUE_DEPTH_DEFAULT);
    }

    #[test]
    fn captures_audio_defaults_to_false() {
        let cfg = SckCaptureConfig::default();
        assert!(!cfg.captures_audio());
        assert_eq!(cfg.audio_sample_rate_hz(), AUDIO_SAMPLE_RATE_DEFAULT_HZ);
        assert_eq!(cfg.audio_channels(), AUDIO_CHANNEL_COUNT_DEFAULT);
    }

    #[test]
    fn builder_captures_audio_toggle_preserves_frame_interval() {
        let cfg_off = SckCaptureConfig::builder()
            .target_fps(60)
            .build()
            .expect("60 fps off");
        let cfg_on = SckCaptureConfig::builder()
            .target_fps(60)
            .captures_audio(true)
            .build()
            .expect("60 fps on");
        assert!(!cfg_off.captures_audio());
        assert!(cfg_on.captures_audio());
        assert_eq!(
            cfg_off.minimum_frame_interval_ns(),
            cfg_on.minimum_frame_interval_ns(),
            "captures_audio toggle must not alter minimum_frame_interval_ns"
        );
    }

    #[test]
    fn builder_rejects_audio_sample_rate_zero() {
        let err = SckCaptureConfig::builder()
            .audio_sample_rate_hz(0)
            .build()
            .unwrap_err();
        assert_eq!(err, SckError::InvalidAudioSampleRate(0));
    }

    #[test]
    fn builder_rejects_audio_channel_count_zero() {
        let err = SckCaptureConfig::builder()
            .audio_channels(0)
            .build()
            .unwrap_err();
        assert_eq!(err, SckError::InvalidAudioChannelCount(0));
    }

    #[test]
    fn builder_accepts_48khz_stereo_audio() {
        let cfg = SckCaptureConfig::builder()
            .captures_audio(true)
            .audio_sample_rate_hz(48_000)
            .audio_channels(2)
            .build()
            .expect("48k stereo ok");
        assert!(cfg.captures_audio());
        assert_eq!(cfg.audio_sample_rate_hz(), 48_000);
        assert_eq!(cfg.audio_channels(), 2);
    }

    #[test]
    fn mac_screen_share_audio_frame_constructs_valid() {
        let frame =
            MacScreenShareAudioFrame::new(48_000, 2, 1024, 12_345).expect("valid audio frame");
        assert_eq!(frame.sample_rate_hz, 48_000);
        assert_eq!(frame.channels, 2);
        assert_eq!(frame.num_samples_per_channel, 1024);
        assert_eq!(frame.pts_us, 12_345);
    }

    #[test]
    fn mac_screen_share_audio_frame_rejects_invalid_sample_rate() {
        let err = MacScreenShareAudioFrame::new(0, 2, 1024, 0).expect_err("0 hz invalid");
        assert_eq!(err, SckError::InvalidAudioSampleRate(0));
    }

    #[test]
    fn mac_screen_share_audio_frame_rejects_too_many_channels() {
        let err = MacScreenShareAudioFrame::new(48_000, 16, 1024, 0).expect_err("16 ch invalid");
        assert_eq!(err, SckError::InvalidAudioChannelCount(16));
    }

    #[test]
    fn audio_sample_format_strings_and_widths() {
        assert_eq!(AudioSampleFormat::F32Planar.as_str(), "f32_planar");
        assert_eq!(
            AudioSampleFormat::F32Interleaved.as_str(),
            "f32_interleaved"
        );
        assert_eq!(
            AudioSampleFormat::I16Interleaved.as_str(),
            "i16_interleaved"
        );
        assert_eq!(AudioSampleFormat::Unknown.as_str(), "unknown");
        assert_eq!(AudioSampleFormat::F32Planar.bytes_per_sample(), 4);
        assert_eq!(AudioSampleFormat::F32Interleaved.bytes_per_sample(), 4);
        assert_eq!(AudioSampleFormat::I16Interleaved.bytes_per_sample(), 2);
        assert_eq!(AudioSampleFormat::Unknown.bytes_per_sample(), 0);
    }

    #[test]
    fn audio_sample_format_codes_are_stable_and_bounded() {
        assert_eq!(AudioSampleFormat::F32Planar.code(), 0);
        assert_eq!(AudioSampleFormat::F32Interleaved.code(), 1);
        assert_eq!(AudioSampleFormat::I16Interleaved.code(), 2);
        assert_eq!(AudioSampleFormat::Unknown.code(), 3);
        assert!(AudioSampleFormat::Unknown.code() <= AUDIO_SAMPLE_FORMAT_CODE_MAX);
    }

    #[test]
    fn mac_screen_share_audio_frame_with_bytes_round_trip() {
        let payload = vec![0xAA_u8; 16];
        let f = MacScreenShareAudioFrameWithBytes::new(
            48_000,
            2,
            4,
            999,
            AudioSampleFormat::F32Planar,
            payload.clone(),
        )
        .expect("valid frame with bytes");
        assert_eq!(f.sample_rate_hz, 48_000);
        assert_eq!(f.channels, 2);
        assert_eq!(f.num_samples_per_channel, 4);
        assert_eq!(f.pts_us, 999);
        assert_eq!(f.format, AudioSampleFormat::F32Planar);
        assert_eq!(f.samples, payload);
    }

    #[test]
    fn mac_screen_share_audio_frame_with_bytes_rejects_bad_sample_rate() {
        let err = MacScreenShareAudioFrameWithBytes::new(
            0,
            2,
            4,
            0,
            AudioSampleFormat::F32Planar,
            Vec::new(),
        )
        .expect_err("zero hz invalid");
        assert_eq!(err, SckError::InvalidAudioSampleRate(0));
    }
}
