// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::audio::{
    DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX, DEEP_FILTER_NOISE_REDUCTION_LEVEL_MIN,
    clamp_deep_filter_noise_reduction_level,
};
use df::tract::{DfParams, DfTract, RuntimeParams};
use ndarray::Array2;

pub const DEEP_FILTER_SAMPLE_RATE_HZ: u32 = 48_000;
pub const DEEP_FILTER_NUM_CHANNELS: u32 = 1;
pub const DEEP_FILTER_FRAME_SAMPLES: usize = 480;

const SAMPLE_SCALE_I16_TO_F32: f32 = 1.0 / 32_768.0;
const SAMPLE_SCALE_F32_TO_I16: f32 = 32_767.0;

const _: () = assert!(DEEP_FILTER_FRAME_SAMPLES == DEEP_FILTER_SAMPLE_RATE_HZ as usize / 100);
const _: () = assert!(DEEP_FILTER_NUM_CHANNELS == 1);

pub struct DeepFilterProcessor {
    model: DfTract,
    input: Array2<f32>,
    output: Array2<f32>,
}

impl DeepFilterProcessor {
    pub fn new(noise_reduction_level: f64) -> Result<DeepFilterProcessor, String> {
        let level_db = clamp_deep_filter_noise_reduction_level(noise_reduction_level);
        assert!(level_db >= DEEP_FILTER_NOISE_REDUCTION_LEVEL_MIN);
        assert!(level_db <= DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX);
        let params = RuntimeParams::default_with_ch(DEEP_FILTER_NUM_CHANNELS as usize)
            .with_atten_lim(level_db as f32);
        let model = DfTract::new(DfParams::default(), &params)
            .map_err(|error| format!("deep filter model init: {error:#}"))?;
        if model.sr != DEEP_FILTER_SAMPLE_RATE_HZ as usize {
            return Err(format!(
                "deep filter model sample rate {} != {DEEP_FILTER_SAMPLE_RATE_HZ}",
                model.sr
            ));
        }
        if model.ch != DEEP_FILTER_NUM_CHANNELS as usize {
            return Err(format!(
                "deep filter model channels {} != {DEEP_FILTER_NUM_CHANNELS}",
                model.ch
            ));
        }
        if model.hop_size != DEEP_FILTER_FRAME_SAMPLES {
            return Err(format!(
                "deep filter model hop {} != {DEEP_FILTER_FRAME_SAMPLES}",
                model.hop_size
            ));
        }
        Ok(DeepFilterProcessor {
            model,
            input: Array2::zeros((1, DEEP_FILTER_FRAME_SAMPLES)),
            output: Array2::zeros((1, DEEP_FILTER_FRAME_SAMPLES)),
        })
    }

    pub fn process_frame(&mut self, samples: &mut [i16]) -> Result<(), String> {
        assert_eq!(samples.len(), DEEP_FILTER_FRAME_SAMPLES);
        assert_eq!(self.input.len(), DEEP_FILTER_FRAME_SAMPLES);
        assert_eq!(self.output.len(), DEEP_FILTER_FRAME_SAMPLES);
        for (target, sample) in self.input.iter_mut().zip(samples.iter()) {
            *target = f32::from(*sample) * SAMPLE_SCALE_I16_TO_F32;
        }
        self.model
            .process(self.input.view(), self.output.view_mut())
            .map_err(|error| format!("deep filter process: {error:#}"))?;
        for (sample, enhanced) in samples.iter_mut().zip(self.output.iter()) {
            *sample = sample_f32_to_i16(*enhanced);
        }
        Ok(())
    }
}

fn sample_f32_to_i16(sample: f32) -> i16 {
    if !sample.is_finite() {
        return 0;
    }
    let clamped = sample.clamp(-1.0, 1.0);
    assert!(clamped >= -1.0);
    assert!(clamped <= 1.0);
    (clamped * SAMPLE_SCALE_F32_TO_I16) as i16
}

#[cfg(test)]
mod tests {
    use super::*;

    fn next_noise_sample(seed: &mut u32) -> i16 {
        *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((*seed >> 16) as u16 as i16) / 4
    }

    fn noise_frame(seed: &mut u32) -> [i16; DEEP_FILTER_FRAME_SAMPLES] {
        let mut frame = [0i16; DEEP_FILTER_FRAME_SAMPLES];
        for sample in frame.iter_mut() {
            *sample = next_noise_sample(seed);
        }
        frame
    }

    fn frame_rms(samples: &[i16]) -> f64 {
        assert!(!samples.is_empty());
        let sum_squares: f64 = samples
            .iter()
            .map(|sample| {
                let normalized = f64::from(*sample) / 32_768.0;
                normalized * normalized
            })
            .sum();
        (sum_squares / samples.len() as f64).sqrt()
    }

    #[test]
    fn sample_conversion_holds_the_contract_range() {
        assert_eq!(sample_f32_to_i16(0.0), 0);
        assert_eq!(sample_f32_to_i16(1.0), 32_767);
        assert_eq!(sample_f32_to_i16(-1.0), -32_767);
        assert_eq!(sample_f32_to_i16(2.0), 32_767);
        assert_eq!(sample_f32_to_i16(-2.0), -32_767);
        assert_eq!(sample_f32_to_i16(0.5), 16_383);
    }

    #[test]
    fn sample_conversion_maps_non_finite_to_silence() {
        assert_eq!(sample_f32_to_i16(f32::NAN), 0);
        assert_eq!(sample_f32_to_i16(f32::INFINITY), 0);
        assert_eq!(sample_f32_to_i16(f32::NEG_INFINITY), 0);
    }

    #[test]
    fn zero_level_passes_audio_through() {
        let mut processor = DeepFilterProcessor::new(0.0).expect("embedded model must initialize");
        let mut seed = 0x2545_f491u32;
        for _ in 0..5 {
            let original = noise_frame(&mut seed);
            let mut processed = original;
            processor
                .process_frame(&mut processed)
                .expect("processing must succeed");
            for (output, input) in processed.iter().zip(original.iter()) {
                assert!((i32::from(*output) - i32::from(*input)).abs() <= 1);
            }
        }
    }

    #[test]
    fn full_level_attenuates_steady_noise() {
        let mut processor = DeepFilterProcessor::new(DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX)
            .expect("embedded model must initialize");
        let mut seed = 0x9e37_79b9u32;
        let mut input_rms = 0.0;
        let mut output_rms = 0.0;
        for frame_index in 0..30 {
            let mut frame = noise_frame(&mut seed);
            let frame_input_rms = frame_rms(&frame);
            processor
                .process_frame(&mut frame)
                .expect("processing must succeed");
            if frame_index >= 20 {
                input_rms += frame_input_rms;
                output_rms += frame_rms(&frame);
            }
        }
        assert!(input_rms > 0.0);
        assert!(output_rms < input_rms * 0.5);
    }

    #[test]
    fn frame_length_contract_is_enforced() {
        let mut processor = DeepFilterProcessor::new(DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX)
            .expect("embedded model must initialize");
        let mut short_frame = [0i16; DEEP_FILTER_FRAME_SAMPLES - 1];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = processor.process_frame(&mut short_frame);
        }));
        assert!(result.is_err());
    }
}
