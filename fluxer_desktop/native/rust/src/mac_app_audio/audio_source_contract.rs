// SPDX-License-Identifier: AGPL-3.0-or-later

pub const ENC_DID_OUTPUT_SAMPLE: &str = "v40@0:8@16^{opaqueCMSampleBuffer=}24q32";
pub const ENC_DID_STOP_WITH_ERROR: &str = "v32@0:8@16@24";

pub const DEFAULT_TARGET_SAMPLE_RATE: f64 = 48_000.0;
pub const DEFAULT_TARGET_CHANNELS: u32 = 2;
pub const MAX_CALLBACK_INPUT_FRAMES: u32 = 48_000;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SourceOptions {
    pub target_sample_rate: f64,
    pub target_channels: u32,
}

impl Default for SourceOptions {
    fn default() -> Self {
        Self {
            target_sample_rate: DEFAULT_TARGET_SAMPLE_RATE,
            target_channels: DEFAULT_TARGET_CHANNELS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encoding_strings_have_expected_lengths() {
        assert_eq!(
            "v40@0:8@16^{opaqueCMSampleBuffer=}24q32",
            ENC_DID_OUTPUT_SAMPLE
        );
        assert_eq!("v32@0:8@16@24", ENC_DID_STOP_WITH_ERROR);
    }

    #[test]
    fn source_options_default_to_the_public_audio_contract() {
        let options = SourceOptions::default();
        assert_eq!(48_000.0, options.target_sample_rate);
        assert_eq!(2, options.target_channels);
        assert_eq!(48_000, MAX_CALLBACK_INPUT_FRAMES);
    }
}
