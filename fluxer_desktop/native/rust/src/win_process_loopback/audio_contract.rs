// SPDX-License-Identifier: AGPL-3.0-or-later

pub const TARGET_SAMPLE_RATE: u32 = 48_000;
pub const TARGET_CHANNELS: u16 = 2;
pub const BITS_PER_SAMPLE: u16 = 32;
pub const BYTES_PER_SAMPLE: u16 = BITS_PER_SAMPLE / 8;
pub const FRAME_BLOCK_ALIGN: u16 = TARGET_CHANNELS * BYTES_PER_SAMPLE;
pub const AVG_BYTES_PER_SECOND: u32 =
    TARGET_SAMPLE_RATE * TARGET_CHANNELS as u32 * BYTES_PER_SAMPLE as u32;

pub fn validate_sample_rate(value: u32) -> bool {
    value == TARGET_SAMPLE_RATE
}

pub fn validate_channels(value: u32) -> bool {
    value == TARGET_CHANNELS as u32
}

pub fn sample_count_for_frames(frames: u32) -> Option<usize> {
    (frames as usize).checked_mul(TARGET_CHANNELS as usize)
}

pub fn qpc_100ns_to_timestamp_us(qpc_100ns: u64) -> i64 {
    (qpc_100ns / 10).min(i64::MAX as u64) as i64
}

pub fn pcm16_to_float32(sample: i16) -> f32 {
    sample as f32 / 32768.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_constants_describe_48khz_stereo_float32_frames() {
        assert_eq!(48_000, TARGET_SAMPLE_RATE);
        assert_eq!(2, TARGET_CHANNELS);
        assert_eq!(32, BITS_PER_SAMPLE);
        assert_eq!(8, FRAME_BLOCK_ALIGN);
        assert_eq!(384_000, AVG_BYTES_PER_SECOND);
    }

    #[test]
    fn option_validation_rejects_unsupported_public_shapes() {
        assert!(validate_sample_rate(48_000));
        assert!(!validate_sample_rate(44_100));
        assert!(validate_channels(2));
        assert!(!validate_channels(1));
    }

    #[test]
    fn sample_counts_and_timestamps_are_bounded() {
        assert_eq!(Some(0), sample_count_for_frames(0));
        assert_eq!(Some(2), sample_count_for_frames(1));
        assert_eq!(Some(9_600), sample_count_for_frames(4_800));
        assert_eq!(123_456, qpc_100ns_to_timestamp_us(1_234_560));
        assert_eq!((u64::MAX / 10) as i64, qpc_100ns_to_timestamp_us(u64::MAX));
    }

    #[test]
    fn pcm16_samples_convert_to_normalized_float32_frames() {
        assert_eq!(-1.0, pcm16_to_float32(i16::MIN));
        assert_eq!(0.0, pcm16_to_float32(0));
        assert!((pcm16_to_float32(i16::MAX) - 0.9999695).abs() < 0.0000001);
    }
}
