#![no_main]

// SPDX-License-Identifier: AGPL-3.0-or-later

use arbitrary::Arbitrary;
use fluxer_desktop_native::mac_app_audio::audio_converter::{
    AudioBuffer, AudioBufferListN, build_input_asbd, convert_buffer_list_to_interleaved_f32,
};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct Input {
    samples: Vec<f32>,
    sample_rate: f64,
    output_rate: f64,
    channels: u8,
}

fuzz_target!(|input: Input| {
    let channels = u32::from(input.channels.clamp(1, 2));
    let sample_rate = if input.sample_rate.is_finite() && input.sample_rate > 0.0 {
        input.sample_rate.min(384_000.0)
    } else {
        48_000.0
    };
    let output_rate = if input.output_rate.is_finite() && input.output_rate > 0.0 {
        input.output_rate.min(384_000.0)
    } else {
        48_000.0
    };
    let frame_count = (input.samples.len() as u32 / channels).min(4096);
    let list = AudioBufferListN {
        m_number_buffers: 1,
        buffers: [AudioBuffer::from_slice(channels, &input.samples)],
    };
    let asbd = build_input_asbd(sample_rate, channels, false);
    let mut out = vec![0.0_f32; frame_count as usize * 4 + 16];
    let _ =
        convert_buffer_list_to_interleaved_f32(asbd, &list, frame_count, output_rate, 2, &mut out);
});
