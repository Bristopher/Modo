// SPDX-License-Identifier: AGPL-3.0-or-later

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fluxer_webrtc_sender::bench_internals::{
    DEEP_FILTER_FRAME_SAMPLES, DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX, DeepFilterProcessor,
};
use std::hint::black_box;

fn next_noise_sample(seed: &mut u32) -> i16 {
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    ((*seed >> 16) as u16 as i16) / 4
}

fn noise_frame(seed: &mut u32) -> Vec<i16> {
    let mut frame = vec![0i16; DEEP_FILTER_FRAME_SAMPLES];
    for sample in frame.iter_mut() {
        *sample = next_noise_sample(seed);
    }
    frame
}

fn bench_deep_filter_process_frame(c: &mut Criterion) {
    let mut processor = DeepFilterProcessor::new(DEEP_FILTER_NOISE_REDUCTION_LEVEL_MAX)
        .expect("embedded model must initialize");
    let mut seed = 0x9e37_79b9u32;
    let mut group = c.benchmark_group("deep_filter");
    group.throughput(Throughput::Elements(DEEP_FILTER_FRAME_SAMPLES as u64));
    group.bench_function("process_frame_10ms", |bencher| {
        bencher.iter_batched(
            || noise_frame(&mut seed),
            |mut frame| {
                processor
                    .process_frame(black_box(&mut frame))
                    .expect("processing must succeed");
                frame
            },
            criterion::BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_deep_filter_process_frame);
criterion_main!(benches);
