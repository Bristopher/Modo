// SPDX-License-Identifier: AGPL-3.0-or-later

pub const DIRECT_CAPTURE_SAMPLE_RATE: u32 = 48_000;
pub const DIRECT_CAPTURE_CHANNELS: u32 = 2;
pub const DIRECT_CAPTURE_MAX_SAMPLES: usize =
    DIRECT_CAPTURE_SAMPLE_RATE as usize * DIRECT_CAPTURE_CHANNELS as usize * 2;
pub const DIRECT_CAPTURE_MAX_READ_SAMPLES: usize =
    DIRECT_CAPTURE_SAMPLE_RATE as usize * DIRECT_CAPTURE_CHANNELS as usize / 10;
pub const MAX_ROUTING_RULE_PATTERNS: u32 = 64;
pub const MAX_ROUTING_RULE_KEYS_PER_PATTERN: u32 = 32;
pub const MAX_ROUTING_RULE_KEY_LENGTH: usize = 128;
pub const MAX_ROUTING_RULE_VALUE_LENGTH: usize = 512;
pub const MAX_INVENTORY_FIELDS: u32 = 32;
pub const MAX_INVENTORY_FIELD_LENGTH: usize = 128;

const _: () = {
    assert!(MAX_ROUTING_RULE_PATTERNS > 0);
    assert!(MAX_ROUTING_RULE_KEYS_PER_PATTERN > 0);
    assert!(MAX_ROUTING_RULE_KEY_LENGTH > 0);
    assert!(MAX_ROUTING_RULE_VALUE_LENGTH >= MAX_ROUTING_RULE_KEY_LENGTH);
    assert!(MAX_INVENTORY_FIELDS <= MAX_ROUTING_RULE_KEYS_PER_PATTERN);
    assert!(MAX_INVENTORY_FIELD_LENGTH <= MAX_ROUTING_RULE_KEY_LENGTH);
};

pub fn whole_frame_sample_count(sample_count: usize, channels: u32) -> usize {
    if channels == 0 {
        return 0;
    }
    let channel_count = channels as usize;
    sample_count - (sample_count % channel_count)
}

pub fn direct_whole_frame_sample_count(sample_count: usize) -> usize {
    whole_frame_sample_count(sample_count, DIRECT_CAPTURE_CHANNELS)
}

pub fn bounded_direct_read_sample_count(available: usize) -> usize {
    direct_whole_frame_sample_count(available.min(DIRECT_CAPTURE_MAX_READ_SAMPLES))
}

pub fn bounded_direct_append_slice(input: &[f32]) -> &[f32] {
    let whole = direct_whole_frame_sample_count(input.len());
    let framed = &input[..whole];
    if framed.len() > DIRECT_CAPTURE_MAX_SAMPLES {
        &framed[framed.len() - DIRECT_CAPTURE_MAX_SAMPLES..]
    } else {
        framed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_frame_sample_count_trims_incomplete_channel_frames() {
        assert_eq!(0, whole_frame_sample_count(1, 2));
        assert_eq!(2, whole_frame_sample_count(2, 2));
        assert_eq!(4, whole_frame_sample_count(5, 2));
        assert_eq!(6, whole_frame_sample_count(7, 3));
        assert_eq!(0, whole_frame_sample_count(7, 0));
    }

    #[test]
    fn direct_read_count_is_bounded_and_stereo_aligned() {
        assert_eq!(0, bounded_direct_read_sample_count(1));
        assert_eq!(2, bounded_direct_read_sample_count(3));
        assert_eq!(
            DIRECT_CAPTURE_MAX_READ_SAMPLES,
            bounded_direct_read_sample_count(DIRECT_CAPTURE_MAX_READ_SAMPLES + 1)
        );
    }

    #[test]
    fn direct_append_slice_keeps_only_complete_stereo_samples_within_queue_cap() {
        let samples = [1.0, 2.0, 3.0, 4.0, 5.0];
        let trimmed = bounded_direct_append_slice(&samples);
        assert_eq!(4, trimmed.len());
        assert_eq!(&samples[..4], trimmed);
    }
}
