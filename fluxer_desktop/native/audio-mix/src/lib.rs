// SPDX-License-Identifier: AGPL-3.0-or-later

#![deny(clippy::too_many_lines)]

pub mod mix_session;
pub mod source_ring;

pub use mix_session::{
    AudioMixSession, MAX_MIX_SOURCES, MixTickResult, MixedFrame, SourceTickStat,
};
pub use source_ring::{
    AUDIO_OUTPUT_FRAMES, AUDIO_RING_CAP_FRAMES, AUDIO_SAMPLE_RATE_HZ_MAX, AUDIO_SAMPLE_RATE_HZ_MIN,
    SourceRing, SourceRingConsumer, SourceRingProducer,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AudioMixError {
    ZeroCapacity,
    CapacityExceedsLimit { requested: usize, limit: usize },
    SampleRateOutOfRange { sample_rate_hz: u32 },
    SampleRateMismatch { expected_hz: u32, observed_hz: u32 },
    ZeroSources,
    TooManySources { requested: usize, limit: usize },
    MixBufferLenMismatch { expected: usize, observed: usize },
}

impl core::fmt::Display for AudioMixError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AudioMixError::ZeroCapacity => write!(f, "ring capacity must be non-zero"),
            AudioMixError::CapacityExceedsLimit { requested, limit } => {
                write!(f, "ring capacity {requested} exceeds hard limit {limit}",)
            }
            AudioMixError::SampleRateOutOfRange { sample_rate_hz } => {
                write!(f, "sample rate {sample_rate_hz} hz outside accepted range")
            }
            AudioMixError::SampleRateMismatch {
                expected_hz,
                observed_hz,
            } => write!(
                f,
                "source sample rate mismatch: expected={expected_hz} observed={observed_hz}",
            ),
            AudioMixError::ZeroSources => write!(f, "AudioMixSession requires at least one source"),
            AudioMixError::TooManySources { requested, limit } => write!(
                f,
                "AudioMixSession source count {requested} exceeds limit {limit}",
            ),
            AudioMixError::MixBufferLenMismatch { expected, observed } => write!(
                f,
                "mix_buffer_len mismatch: expected={expected} observed={observed}",
            ),
        }
    }
}

impl std::error::Error for AudioMixError {}
