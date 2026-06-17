// SPDX-License-Identifier: AGPL-3.0-or-later

#![deny(clippy::too_many_lines)]
#![deny(clippy::unwrap_used)]

pub const APM_FRAME_MS: u32 = 10;
pub const APM_MAX_SAMPLE_RATE: u32 = 48_000;
pub const APM_MAX_CHANNELS: u16 = 2;
pub const APM_MAX_FRAME_SAMPLES: usize =
    (APM_FRAME_MS as usize) * (APM_MAX_SAMPLE_RATE as usize) / 1000;

pub const APM_MIN_SAMPLE_RATE: u32 = 8_000;
pub const APM_MIN_CHANNELS: u16 = 1;

const _: () = assert!(APM_MAX_FRAME_SAMPLES == 480);
const _: () = assert!(APM_MIN_SAMPLE_RATE <= APM_MAX_SAMPLE_RATE);
const _: () = assert!(APM_MIN_CHANNELS <= APM_MAX_CHANNELS);

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ApmError {
    SampleRateOutOfRange {
        sample_rate_hz: u32,
    },
    SampleRateMismatch {
        expected_hz: u32,
        observed_hz: u32,
    },
    ChannelsOutOfRange {
        channels: u16,
    },
    ChannelsMismatch {
        expected: u16,
        observed: u16,
    },
    FrameLengthMismatch {
        expected_samples: usize,
        observed_samples: usize,
    },
    NotInitialized,
    BackendUnavailable,
}

impl core::fmt::Display for ApmError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ApmError::SampleRateOutOfRange { sample_rate_hz } => write!(
                f,
                "sample rate {sample_rate_hz} hz outside [{APM_MIN_SAMPLE_RATE}, {APM_MAX_SAMPLE_RATE}]",
            ),
            ApmError::SampleRateMismatch {
                expected_hz,
                observed_hz,
            } => write!(
                f,
                "sample rate mismatch: expected={expected_hz} observed={observed_hz}",
            ),
            ApmError::ChannelsOutOfRange { channels } => write!(
                f,
                "channel count {channels} outside [{APM_MIN_CHANNELS}, {APM_MAX_CHANNELS}]",
            ),
            ApmError::ChannelsMismatch { expected, observed } => write!(
                f,
                "channels mismatch: expected={expected} observed={observed}",
            ),
            ApmError::FrameLengthMismatch {
                expected_samples,
                observed_samples,
            } => write!(
                f,
                "frame length mismatch: expected={expected_samples} observed={observed_samples}",
            ),
            ApmError::NotInitialized => write!(f, "audio processor not initialized"),
            ApmError::BackendUnavailable => write!(f, "real APM backend unavailable"),
        }
    }
}

impl std::error::Error for ApmError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AecMetrics {
    pub echo_return_loss_db: f32,
    pub echo_return_loss_enhancement_db: f32,
    pub delay_ms: i32,
}

impl AecMetrics {
    pub const NEUTRAL: AecMetrics = AecMetrics {
        echo_return_loss_db: 0.0,
        echo_return_loss_enhancement_db: 0.0,
        delay_ms: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ApmReport {
    pub aec_metrics: AecMetrics,
    pub voice_detected: bool,
    pub level_dbfs: f32,
}

impl ApmReport {
    pub const NEUTRAL: ApmReport = ApmReport {
        aec_metrics: AecMetrics::NEUTRAL,
        voice_detected: false,
        level_dbfs: -120.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ApmConfig {
    pub aec_enabled: bool,
    pub ns_enabled: bool,
    pub agc_enabled: bool,
    pub aec_mobile_mode: bool,
    pub target_level_dbfs: i32,
}

impl Default for ApmConfig {
    fn default() -> Self {
        ApmConfig {
            aec_enabled: true,
            ns_enabled: true,
            agc_enabled: true,
            aec_mobile_mode: false,
            target_level_dbfs: -3,
        }
    }
}

pub struct ApmConfigBuilder {
    config: ApmConfig,
}

impl ApmConfigBuilder {
    pub fn new() -> Self {
        ApmConfigBuilder {
            config: ApmConfig::default(),
        }
    }

    pub fn aec(mut self, enabled: bool) -> Self {
        self.config.aec_enabled = enabled;
        self
    }

    pub fn ns(mut self, enabled: bool) -> Self {
        self.config.ns_enabled = enabled;
        self
    }

    pub fn agc(mut self, enabled: bool) -> Self {
        self.config.agc_enabled = enabled;
        self
    }

    pub fn aec_mobile_mode(mut self, enabled: bool) -> Self {
        self.config.aec_mobile_mode = enabled;
        self
    }

    pub fn target_level_dbfs(mut self, target: i32) -> Self {
        self.config.target_level_dbfs = target;
        self
    }

    pub fn build(self) -> ApmConfig {
        assert!(self.config.target_level_dbfs <= 0);
        assert!(self.config.target_level_dbfs >= -60);
        self.config
    }
}

impl Default for ApmConfigBuilder {
    fn default() -> Self {
        ApmConfigBuilder::new()
    }
}

pub trait AudioProcessor: Send {
    fn process_capture_frame(
        &mut self,
        samples: &mut [i16],
        sample_rate_hz: u32,
        channels: u16,
    ) -> Result<ApmReport, ApmError>;

    fn process_render_frame(
        &mut self,
        samples: &[i16],
        sample_rate_hz: u32,
        channels: u16,
    ) -> Result<(), ApmError>;

    fn reset(&mut self) -> Result<(), ApmError>;
}

pub fn expected_frame_samples(sample_rate_hz: u32, channels: u16) -> usize {
    assert!(sample_rate_hz >= APM_MIN_SAMPLE_RATE);
    assert!(sample_rate_hz <= APM_MAX_SAMPLE_RATE);
    assert!(channels >= APM_MIN_CHANNELS);
    assert!(channels <= APM_MAX_CHANNELS);
    let per_channel = (APM_FRAME_MS as usize) * (sample_rate_hz as usize) / 1000;
    per_channel * (channels as usize)
}

pub(crate) fn validate_frame_shape(
    samples_len: usize,
    sample_rate_hz: u32,
    channels: u16,
    expected_sample_rate_hz: u32,
    expected_channels: u16,
) -> Result<(), ApmError> {
    assert!(expected_sample_rate_hz >= APM_MIN_SAMPLE_RATE);
    assert!(expected_sample_rate_hz <= APM_MAX_SAMPLE_RATE);
    if !(APM_MIN_SAMPLE_RATE..=APM_MAX_SAMPLE_RATE).contains(&sample_rate_hz) {
        return Err(ApmError::SampleRateOutOfRange { sample_rate_hz });
    }
    if !(APM_MIN_CHANNELS..=APM_MAX_CHANNELS).contains(&channels) {
        return Err(ApmError::ChannelsOutOfRange { channels });
    }
    if sample_rate_hz != expected_sample_rate_hz {
        return Err(ApmError::SampleRateMismatch {
            expected_hz: expected_sample_rate_hz,
            observed_hz: sample_rate_hz,
        });
    }
    if channels != expected_channels {
        return Err(ApmError::ChannelsMismatch {
            expected: expected_channels,
            observed: channels,
        });
    }
    let expected_samples = expected_frame_samples(sample_rate_hz, channels);
    if samples_len != expected_samples {
        return Err(ApmError::FrameLengthMismatch {
            expected_samples,
            observed_samples: samples_len,
        });
    }
    Ok(())
}

#[derive(Debug)]
pub struct StubAudioProcessor {
    config: ApmConfig,
    expected_sample_rate_hz: u32,
    expected_channels: u16,
    capture_frames_processed: u64,
    render_frames_processed: u64,
}

impl StubAudioProcessor {
    pub fn new(config: ApmConfig, sample_rate_hz: u32, channels: u16) -> Result<Self, ApmError> {
        if !(APM_MIN_SAMPLE_RATE..=APM_MAX_SAMPLE_RATE).contains(&sample_rate_hz) {
            return Err(ApmError::SampleRateOutOfRange { sample_rate_hz });
        }
        if !(APM_MIN_CHANNELS..=APM_MAX_CHANNELS).contains(&channels) {
            return Err(ApmError::ChannelsOutOfRange { channels });
        }
        assert!(sample_rate_hz >= APM_MIN_SAMPLE_RATE);
        assert!(channels >= APM_MIN_CHANNELS);
        Ok(StubAudioProcessor {
            config,
            expected_sample_rate_hz: sample_rate_hz,
            expected_channels: channels,
            capture_frames_processed: 0,
            render_frames_processed: 0,
        })
    }

    pub fn config(&self) -> ApmConfig {
        self.config
    }

    pub fn capture_frames_processed(&self) -> u64 {
        self.capture_frames_processed
    }

    pub fn render_frames_processed(&self) -> u64 {
        self.render_frames_processed
    }
}

impl AudioProcessor for StubAudioProcessor {
    fn process_capture_frame(
        &mut self,
        samples: &mut [i16],
        sample_rate_hz: u32,
        channels: u16,
    ) -> Result<ApmReport, ApmError> {
        assert!(self.expected_sample_rate_hz >= APM_MIN_SAMPLE_RATE);
        assert!(self.expected_channels >= APM_MIN_CHANNELS);
        validate_frame_shape(
            samples.len(),
            sample_rate_hz,
            channels,
            self.expected_sample_rate_hz,
            self.expected_channels,
        )?;
        assert!(self.capture_frames_processed < u64::MAX);
        self.capture_frames_processed = self.capture_frames_processed.saturating_add(1);
        Ok(ApmReport::NEUTRAL)
    }

    fn process_render_frame(
        &mut self,
        samples: &[i16],
        sample_rate_hz: u32,
        channels: u16,
    ) -> Result<(), ApmError> {
        assert!(self.expected_sample_rate_hz >= APM_MIN_SAMPLE_RATE);
        assert!(self.expected_channels >= APM_MIN_CHANNELS);
        validate_frame_shape(
            samples.len(),
            sample_rate_hz,
            channels,
            self.expected_sample_rate_hz,
            self.expected_channels,
        )?;
        assert!(self.render_frames_processed < u64::MAX);
        self.render_frames_processed = self.render_frames_processed.saturating_add(1);
        Ok(())
    }

    fn reset(&mut self) -> Result<(), ApmError> {
        assert!(self.expected_sample_rate_hz >= APM_MIN_SAMPLE_RATE);
        assert!(self.expected_channels >= APM_MIN_CHANNELS);
        self.capture_frames_processed = 0;
        self.render_frames_processed = 0;
        Ok(())
    }
}

#[cfg(feature = "real-apm")]
mod real {
    use super::{
        APM_MAX_CHANNELS, APM_MAX_FRAME_SAMPLES, APM_MAX_SAMPLE_RATE, APM_MIN_CHANNELS,
        APM_MIN_SAMPLE_RATE, AecMetrics, ApmConfig, ApmError, ApmReport, AudioProcessor,
        validate_frame_shape,
    };
    use webrtc_audio_processing::{Processor, Stats};
    use webrtc_audio_processing_config::{
        Config, EchoCanceller, GainController, GainController1, GainControllerMode, HighPassFilter,
        NoiseSuppression, NoiseSuppressionLevel,
    };

    const I16_TO_F32_SCALE: f32 = 1.0 / 32768.0;
    const F32_TO_I16_SCALE: f32 = 32767.0;

    #[derive(Debug)]
    pub struct WebRtcAudioProcessor {
        inner: Processor,
        config: ApmConfig,
        expected_sample_rate_hz: u32,
        expected_channels: u16,
        samples_per_channel: usize,
        capture_channels: [[f32; APM_MAX_FRAME_SAMPLES]; APM_MAX_CHANNELS as usize],
        render_channels: [[f32; APM_MAX_FRAME_SAMPLES]; APM_MAX_CHANNELS as usize],
        capture_frames_processed: u64,
        render_frames_processed: u64,
    }

    impl WebRtcAudioProcessor {
        pub fn new(
            config: ApmConfig,
            sample_rate_hz: u32,
            channels: u16,
        ) -> Result<Self, ApmError> {
            if !(APM_MIN_SAMPLE_RATE..=APM_MAX_SAMPLE_RATE).contains(&sample_rate_hz) {
                return Err(ApmError::SampleRateOutOfRange { sample_rate_hz });
            }
            if !(APM_MIN_CHANNELS..=APM_MAX_CHANNELS).contains(&channels) {
                return Err(ApmError::ChannelsOutOfRange { channels });
            }
            assert!(sample_rate_hz >= APM_MIN_SAMPLE_RATE);
            assert!(channels >= APM_MIN_CHANNELS);
            let inner = Processor::new(sample_rate_hz).map_err(|_| ApmError::BackendUnavailable)?;
            inner.set_config(build_webrtc_config(config));
            let samples_per_channel = inner.num_samples_per_frame();
            assert!(samples_per_channel <= APM_MAX_FRAME_SAMPLES);
            assert!(samples_per_channel > 0);
            Ok(WebRtcAudioProcessor {
                inner,
                config,
                expected_sample_rate_hz: sample_rate_hz,
                expected_channels: channels,
                samples_per_channel,
                capture_channels: [[0.0; APM_MAX_FRAME_SAMPLES]; APM_MAX_CHANNELS as usize],
                render_channels: [[0.0; APM_MAX_FRAME_SAMPLES]; APM_MAX_CHANNELS as usize],
                capture_frames_processed: 0,
                render_frames_processed: 0,
            })
        }

        pub fn config(&self) -> ApmConfig {
            self.config
        }

        pub fn capture_frames_processed(&self) -> u64 {
            self.capture_frames_processed
        }

        pub fn render_frames_processed(&self) -> u64 {
            self.render_frames_processed
        }

        pub fn samples_per_channel(&self) -> usize {
            self.samples_per_channel
        }
    }

    fn build_webrtc_config(config: ApmConfig) -> Config {
        let echo_canceller = if !config.aec_enabled {
            None
        } else if config.aec_mobile_mode {
            Some(EchoCanceller::Mobile { stream_delay_ms: 0 })
        } else {
            Some(EchoCanceller::Full {
                stream_delay_ms: None,
            })
        };
        let noise_suppression = if config.ns_enabled {
            Some(NoiseSuppression {
                level: NoiseSuppressionLevel::Moderate,
                analyze_linear_aec_output: false,
            })
        } else {
            None
        };
        let gain_controller = if config.agc_enabled {
            Some(gain_controller_from(config.target_level_dbfs))
        } else {
            None
        };
        Config {
            pipeline: Default::default(),
            capture_amplifier: None,
            high_pass_filter: Some(HighPassFilter {
                apply_in_full_band: true,
            }),
            echo_canceller,
            noise_suppression,
            gain_controller,
        }
    }

    fn gain_controller_from(target_level_dbfs: i32) -> GainController {
        assert!(target_level_dbfs <= 0);
        assert!(target_level_dbfs >= -60);
        let target_clamped = (-target_level_dbfs).clamp(0, 31) as u8;
        GainController::GainController1(GainController1 {
            mode: GainControllerMode::AdaptiveDigital,
            target_level_dbfs: target_clamped,
            compression_gain_db: 9,
            enable_limiter: true,
            analog_gain_controller: None,
        })
    }

    fn deinterleave_i16(
        samples: &[i16],
        channels: usize,
        samples_per_channel: usize,
        out: &mut [[f32; APM_MAX_FRAME_SAMPLES]; APM_MAX_CHANNELS as usize],
    ) {
        assert!(channels >= 1);
        assert!(channels <= APM_MAX_CHANNELS as usize);
        assert!(samples_per_channel <= APM_MAX_FRAME_SAMPLES);
        assert!(samples.len() == channels * samples_per_channel);
        for (sample_index, frame) in samples.chunks_exact(channels).enumerate() {
            for (channel_index, &value) in frame.iter().enumerate() {
                out[channel_index][sample_index] = (value as f32) * I16_TO_F32_SCALE;
            }
        }
    }

    fn interleave_to_i16(
        channels_data: &[[f32; APM_MAX_FRAME_SAMPLES]; APM_MAX_CHANNELS as usize],
        channels: usize,
        samples_per_channel: usize,
        out: &mut [i16],
    ) {
        assert!(channels >= 1);
        assert!(channels <= APM_MAX_CHANNELS as usize);
        assert!(samples_per_channel <= APM_MAX_FRAME_SAMPLES);
        assert!(out.len() == channels * samples_per_channel);
        for (sample_index, frame) in out.chunks_exact_mut(channels).enumerate() {
            for (channel_index, slot) in frame.iter_mut().enumerate() {
                let value = channels_data[channel_index][sample_index];
                let scaled = (value * F32_TO_I16_SCALE).clamp(i16::MIN as f32, i16::MAX as f32);
                *slot = scaled as i16;
            }
        }
    }

    fn peak_level_dbfs(samples: &[i16]) -> f32 {
        assert!(!samples.is_empty());
        let mut peak_abs: i32 = 0;
        for &sample in samples {
            let abs_value = (sample as i32).unsigned_abs() as i32;
            if abs_value > peak_abs {
                peak_abs = abs_value;
            }
        }
        if peak_abs == 0 {
            return -120.0;
        }
        let normalised = (peak_abs as f32) * I16_TO_F32_SCALE;
        20.0 * normalised.log10()
    }

    fn report_from(stats: Stats, level_dbfs: f32) -> ApmReport {
        let erl = stats.echo_return_loss.unwrap_or(0.0) as f32;
        let erle = stats.echo_return_loss_enhancement.unwrap_or(0.0) as f32;
        let delay_ms = stats.delay_ms.unwrap_or(0) as i32;
        ApmReport {
            aec_metrics: AecMetrics {
                echo_return_loss_db: erl,
                echo_return_loss_enhancement_db: erle,
                delay_ms,
            },
            voice_detected: level_dbfs > -50.0,
            level_dbfs,
        }
    }

    impl AudioProcessor for WebRtcAudioProcessor {
        fn process_capture_frame(
            &mut self,
            samples: &mut [i16],
            sample_rate_hz: u32,
            channels: u16,
        ) -> Result<ApmReport, ApmError> {
            assert!(self.expected_sample_rate_hz >= APM_MIN_SAMPLE_RATE);
            assert!(self.expected_channels >= APM_MIN_CHANNELS);
            validate_frame_shape(
                samples.len(),
                sample_rate_hz,
                channels,
                self.expected_sample_rate_hz,
                self.expected_channels,
            )?;
            let channels_usize = channels as usize;
            let per_channel = self.samples_per_channel;
            deinterleave_i16(
                samples,
                channels_usize,
                per_channel,
                &mut self.capture_channels,
            );
            let mut view: [&mut [f32]; APM_MAX_CHANNELS as usize] = {
                let (first, second) = self.capture_channels.split_at_mut(1);
                [&mut first[0][..per_channel], &mut second[0][..per_channel]]
            };
            let frame = &mut view[..channels_usize];
            self.inner
                .process_capture_frame(frame)
                .map_err(|_| ApmError::BackendUnavailable)?;
            interleave_to_i16(&self.capture_channels, channels_usize, per_channel, samples);
            let level_dbfs = peak_level_dbfs(samples);
            let stats = self.inner.get_stats();
            self.capture_frames_processed = self.capture_frames_processed.saturating_add(1);
            Ok(report_from(stats, level_dbfs))
        }

        fn process_render_frame(
            &mut self,
            samples: &[i16],
            sample_rate_hz: u32,
            channels: u16,
        ) -> Result<(), ApmError> {
            assert!(self.expected_sample_rate_hz >= APM_MIN_SAMPLE_RATE);
            assert!(self.expected_channels >= APM_MIN_CHANNELS);
            validate_frame_shape(
                samples.len(),
                sample_rate_hz,
                channels,
                self.expected_sample_rate_hz,
                self.expected_channels,
            )?;
            let channels_usize = channels as usize;
            let per_channel = self.samples_per_channel;
            deinterleave_i16(
                samples,
                channels_usize,
                per_channel,
                &mut self.render_channels,
            );
            let mut view: [&mut [f32]; APM_MAX_CHANNELS as usize] = {
                let (first, second) = self.render_channels.split_at_mut(1);
                [&mut first[0][..per_channel], &mut second[0][..per_channel]]
            };
            let frame = &mut view[..channels_usize];
            self.inner
                .process_render_frame(frame)
                .map_err(|_| ApmError::BackendUnavailable)?;
            self.render_frames_processed = self.render_frames_processed.saturating_add(1);
            Ok(())
        }

        fn reset(&mut self) -> Result<(), ApmError> {
            assert!(self.expected_sample_rate_hz >= APM_MIN_SAMPLE_RATE);
            assert!(self.expected_channels >= APM_MIN_CHANNELS);
            self.inner.reinitialize();
            self.capture_frames_processed = 0;
            self.render_frames_processed = 0;
            Ok(())
        }
    }
}

#[cfg(feature = "real-apm")]
pub use real::WebRtcAudioProcessor;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_frame(sample_rate_hz: u32, channels: u16) -> Vec<i16> {
        let n = expected_frame_samples(sample_rate_hz, channels);
        (0..n).map(|i| (i as i16).wrapping_mul(7)).collect()
    }

    #[test]
    fn frame_samples_constant_matches_formula() {
        assert_eq!(APM_MAX_FRAME_SAMPLES, 480);
        assert_eq!(expected_frame_samples(48_000, 1), 480);
        assert_eq!(expected_frame_samples(48_000, 2), 960);
        assert_eq!(expected_frame_samples(16_000, 1), 160);
    }

    #[test]
    fn stub_capture_does_not_modify_samples() {
        let config = ApmConfigBuilder::new().build();
        let mut stub = StubAudioProcessor::new(config, 48_000, 1).expect("ctor");
        let original = make_frame(48_000, 1);
        let mut samples = original.clone();
        let report = stub
            .process_capture_frame(&mut samples, 48_000, 1)
            .expect("ok");
        assert_eq!(samples, original);
        assert_eq!(report, ApmReport::NEUTRAL);
    }

    #[test]
    fn stub_report_has_neutral_metrics() {
        let config = ApmConfig::default();
        let mut stub = StubAudioProcessor::new(config, 48_000, 1).expect("ctor");
        let mut samples = make_frame(48_000, 1);
        let report = stub
            .process_capture_frame(&mut samples, 48_000, 1)
            .expect("ok");
        assert_eq!(report.aec_metrics.echo_return_loss_db, 0.0);
        assert_eq!(report.aec_metrics.echo_return_loss_enhancement_db, 0.0);
        assert_eq!(report.aec_metrics.delay_ms, 0);
        assert!(!report.voice_detected);
        assert_eq!(report.level_dbfs, -120.0);
    }

    #[test]
    fn rejects_mismatched_sample_rate() {
        let mut stub = StubAudioProcessor::new(ApmConfig::default(), 48_000, 1).expect("ctor");
        let mut samples = make_frame(16_000, 1);
        let err = stub
            .process_capture_frame(&mut samples, 16_000, 1)
            .expect_err("err");
        assert!(matches!(err, ApmError::SampleRateMismatch { .. }));
    }

    #[test]
    fn rejects_mismatched_channels() {
        let mut stub = StubAudioProcessor::new(ApmConfig::default(), 48_000, 1).expect("ctor");
        let mut samples = make_frame(48_000, 2);
        let err = stub
            .process_capture_frame(&mut samples, 48_000, 2)
            .expect_err("err");
        assert!(matches!(err, ApmError::ChannelsMismatch { .. }));
    }

    #[test]
    fn rejects_wrong_frame_length() {
        let mut stub = StubAudioProcessor::new(ApmConfig::default(), 48_000, 1).expect("ctor");
        let mut samples = vec![0i16; 100];
        let err = stub
            .process_capture_frame(&mut samples, 48_000, 1)
            .expect_err("err");
        assert!(matches!(err, ApmError::FrameLengthMismatch { .. }));
    }

    #[test]
    fn rejects_render_frame_length() {
        let mut stub = StubAudioProcessor::new(ApmConfig::default(), 48_000, 1).expect("ctor");
        let samples = vec![0i16; 99];
        let err = stub
            .process_render_frame(&samples, 48_000, 1)
            .expect_err("err");
        assert!(matches!(err, ApmError::FrameLengthMismatch { .. }));
    }

    #[test]
    fn rejects_sample_rate_out_of_range_on_construct() {
        let err = StubAudioProcessor::new(ApmConfig::default(), 4_000, 1).expect_err("err");
        assert!(matches!(err, ApmError::SampleRateOutOfRange { .. }));
    }

    #[test]
    fn rejects_channels_out_of_range_on_construct() {
        let err = StubAudioProcessor::new(ApmConfig::default(), 48_000, 4).expect_err("err");
        assert!(matches!(err, ApmError::ChannelsOutOfRange { .. }));
    }

    #[test]
    fn reset_returns_ok() {
        let mut stub = StubAudioProcessor::new(ApmConfig::default(), 48_000, 1).expect("ctor");
        let mut samples = make_frame(48_000, 1);
        stub.process_capture_frame(&mut samples, 48_000, 1)
            .expect("ok");
        assert_eq!(stub.capture_frames_processed(), 1);
        stub.reset().expect("reset");
        assert_eq!(stub.capture_frames_processed(), 0);
        assert_eq!(stub.render_frames_processed(), 0);
    }

    #[test]
    fn builder_pattern_produces_valid_config() {
        let config = ApmConfigBuilder::new()
            .aec(false)
            .ns(true)
            .agc(false)
            .aec_mobile_mode(true)
            .target_level_dbfs(-6)
            .build();
        assert!(!config.aec_enabled);
        assert!(config.ns_enabled);
        assert!(!config.agc_enabled);
        assert!(config.aec_mobile_mode);
        assert_eq!(config.target_level_dbfs, -6);
    }

    #[test]
    fn builder_default_matches_struct_default() {
        let builder = ApmConfigBuilder::new().build();
        let default = ApmConfig::default();
        assert_eq!(builder, default);
    }

    #[test]
    fn state_preserved_across_many_frames() {
        let mut stub = StubAudioProcessor::new(ApmConfig::default(), 48_000, 1).expect("ctor");
        let mut samples = make_frame(48_000, 1);
        const N: u64 = 100;
        for _ in 0..N {
            stub.process_capture_frame(&mut samples, 48_000, 1)
                .expect("ok");
        }
        assert_eq!(stub.capture_frames_processed(), N);
        let render = vec![0i16; expected_frame_samples(48_000, 1)];
        for _ in 0..N {
            stub.process_render_frame(&render, 48_000, 1).expect("ok");
        }
        assert_eq!(stub.render_frames_processed(), N);
    }

    #[test]
    fn determinism_stub_produces_identical_output() {
        let mut stub_a = StubAudioProcessor::new(ApmConfig::default(), 48_000, 1).expect("ctor");
        let mut stub_b = StubAudioProcessor::new(ApmConfig::default(), 48_000, 1).expect("ctor");
        let original = make_frame(48_000, 1);
        let mut samples_a = original.clone();
        let mut samples_b = original.clone();
        let report_a = stub_a
            .process_capture_frame(&mut samples_a, 48_000, 1)
            .expect("ok");
        let report_b = stub_b
            .process_capture_frame(&mut samples_b, 48_000, 1)
            .expect("ok");
        assert_eq!(samples_a, samples_b);
        assert_eq!(samples_a, original);
        assert_eq!(report_a, report_b);
    }

    #[test]
    fn stereo_round_trip_capture_succeeds() {
        let mut stub = StubAudioProcessor::new(ApmConfig::default(), 48_000, 2).expect("ctor");
        let mut samples = make_frame(48_000, 2);
        let original = samples.clone();
        let report = stub
            .process_capture_frame(&mut samples, 48_000, 2)
            .expect("ok");
        assert_eq!(samples, original);
        assert_eq!(report, ApmReport::NEUTRAL);
    }

    #[test]
    fn validate_frame_shape_accepts_canonical_48k_mono() {
        validate_frame_shape(480, 48_000, 1, 48_000, 1).expect("ok");
    }

    #[test]
    #[cfg(not(feature = "real-apm"))]
    fn real_apm_type_absent_when_feature_off() {
        let _ = ApmConfig::default();
    }

    #[cfg(feature = "real-apm")]
    mod real_apm_tests {
        use super::*;

        fn sine_frame_i16(sample_rate_hz: u32, channels: u16, frequency_hz: f32) -> Vec<i16> {
            let per_channel = (APM_FRAME_MS as usize) * (sample_rate_hz as usize) / 1000;
            let total = per_channel * (channels as usize);
            let mut samples = Vec::with_capacity(total);
            for sample_index in 0..per_channel {
                let phase = (sample_index as f32) * frequency_hz / (sample_rate_hz as f32);
                let value = (phase * 2.0 * core::f32::consts::PI).sin() * 0.5;
                let scaled = (value * 32767.0) as i16;
                for _ in 0..channels {
                    samples.push(scaled);
                }
            }
            samples
        }

        #[test]
        fn real_apm_constructs_at_48k_mono() {
            let config = ApmConfig::default();
            let processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            assert_eq!(processor.samples_per_channel(), 480);
            assert_eq!(processor.capture_frames_processed(), 0);
            assert_eq!(processor.render_frames_processed(), 0);
            assert_eq!(processor.config(), config);
        }

        #[test]
        fn real_apm_rejects_invalid_sample_rate_on_construct() {
            let config = ApmConfig::default();
            let err = WebRtcAudioProcessor::new(config, 4_000, 1).expect_err("err");
            assert!(matches!(err, ApmError::SampleRateOutOfRange { .. }));
        }

        #[test]
        fn real_apm_rejects_invalid_channels_on_construct() {
            let config = ApmConfig::default();
            let err = WebRtcAudioProcessor::new(config, 48_000, 8).expect_err("err");
            assert!(matches!(err, ApmError::ChannelsOutOfRange { .. }));
        }

        #[test]
        fn real_apm_capture_increments_counter_and_returns_report() {
            let config = ApmConfig::default();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            let mut samples = sine_frame_i16(48_000, 1, 440.0);
            let report = processor
                .process_capture_frame(&mut samples, 48_000, 1)
                .expect("capture ok");
            assert_eq!(processor.capture_frames_processed(), 1);
            assert!(report.level_dbfs <= 0.0);
            assert!(report.level_dbfs >= -120.0);
        }

        #[test]
        fn real_apm_render_increments_counter() {
            let config = ApmConfig::default();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            let samples = sine_frame_i16(48_000, 1, 880.0);
            processor
                .process_render_frame(&samples, 48_000, 1)
                .expect("render ok");
            assert_eq!(processor.render_frames_processed(), 1);
        }

        #[test]
        fn real_apm_rejects_mismatched_frame_length() {
            let config = ApmConfig::default();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            let mut samples = vec![0i16; 100];
            let err = processor
                .process_capture_frame(&mut samples, 48_000, 1)
                .expect_err("err");
            assert!(matches!(err, ApmError::FrameLengthMismatch { .. }));
        }

        #[test]
        fn real_apm_rejects_mismatched_sample_rate_at_process() {
            let config = ApmConfig::default();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            let mut samples = sine_frame_i16(16_000, 1, 440.0);
            let err = processor
                .process_capture_frame(&mut samples, 16_000, 1)
                .expect_err("err");
            assert!(matches!(err, ApmError::SampleRateMismatch { .. }));
        }

        #[test]
        fn real_apm_processes_many_capture_frames_without_panic() {
            let config = ApmConfigBuilder::new().aec(true).ns(true).agc(true).build();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            let mut samples = sine_frame_i16(48_000, 1, 440.0);
            const N: u64 = 50;
            for _ in 0..N {
                let mut frame = samples.clone();
                processor
                    .process_capture_frame(&mut frame, 48_000, 1)
                    .expect("capture ok");
                processor
                    .process_render_frame(&samples, 48_000, 1)
                    .expect("render ok");
            }
            assert_eq!(processor.capture_frames_processed(), N);
            assert_eq!(processor.render_frames_processed(), N);
            samples.clear();
        }

        #[test]
        fn real_apm_reset_clears_counters() {
            let config = ApmConfig::default();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            let mut samples = sine_frame_i16(48_000, 1, 440.0);
            processor
                .process_capture_frame(&mut samples, 48_000, 1)
                .expect("capture ok");
            processor
                .process_render_frame(&samples, 48_000, 1)
                .expect("render ok");
            assert_eq!(processor.capture_frames_processed(), 1);
            assert_eq!(processor.render_frames_processed(), 1);
            processor.reset().expect("reset ok");
            assert_eq!(processor.capture_frames_processed(), 0);
            assert_eq!(processor.render_frames_processed(), 0);
        }

        #[test]
        fn real_apm_silent_frame_reports_quiet_level() {
            let config = ApmConfig::default();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            let mut samples = vec![0i16; expected_frame_samples(48_000, 1)];
            let report = processor
                .process_capture_frame(&mut samples, 48_000, 1)
                .expect("capture ok");
            assert!(report.level_dbfs <= -50.0);
            assert!(!report.voice_detected);
        }

        #[test]
        fn real_apm_stereo_round_trip_succeeds() {
            let config = ApmConfig::default();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 2).expect("ctor");
            let mut samples = sine_frame_i16(48_000, 2, 440.0);
            processor
                .process_capture_frame(&mut samples, 48_000, 2)
                .expect("capture ok");
            assert_eq!(processor.capture_frames_processed(), 1);
        }

        #[test]
        fn real_apm_disabling_all_submodules_constructs_ok() {
            let config = ApmConfigBuilder::new()
                .aec(false)
                .ns(false)
                .agc(false)
                .target_level_dbfs(-6)
                .build();
            let mut processor = WebRtcAudioProcessor::new(config, 48_000, 1).expect("ctor");
            let mut samples = sine_frame_i16(48_000, 1, 440.0);
            processor
                .process_capture_frame(&mut samples, 48_000, 1)
                .expect("capture ok");
            assert_eq!(processor.capture_frames_processed(), 1);
        }
    }
}
