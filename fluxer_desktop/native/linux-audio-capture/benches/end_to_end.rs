// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_audio_mix::{AUDIO_OUTPUT_FRAMES, SourceRing, SourceRingProducer};
use fluxer_linux_audio_capture::audio_mix_runtime_bench_helpers::{
    AudioMixRuntimeBuilder, CaptureSource, MIX_CHANNELS, MIX_SAMPLE_RATE_HZ, MIX_TICK_PERIOD_NS,
    NullMixOutputSink,
};

const BENCH_SOURCE_COUNT: usize = 8;

fn build_sources_and_runtime() -> (
    Vec<CaptureSource>,
    fluxer_linux_audio_capture::audio_mix_runtime_bench_helpers::AudioMixRuntime,
) {
    let mut sources = Vec::with_capacity(BENCH_SOURCE_COUNT);
    let mut builder = AudioMixRuntimeBuilder::new();
    for n in 0..BENCH_SOURCE_COUNT {
        let (source, consumer) =
            CaptureSource::create(n as u64 + 1, MIX_SAMPLE_RATE_HZ, MIX_CHANNELS).expect("source");
        sources.push(source);
        builder = builder.add_source(n as u64 + 1, consumer);
    }
    let runtime = builder.build(NullMixOutputSink).expect("build");
    (sources, runtime)
}

fn fill_sources(sources: &mut [CaptureSource], frames: usize) {
    assert!(!sources.is_empty());
    assert!(frames > 0);
    let payload: Vec<i16> = (0..frames).map(|n| ((n as i16) % 4096) - 2048).collect();
    for source in sources.iter_mut() {
        let _pushed = source.ingest_skip_apm(&payload);
    }
}

fn bench_end_to_end_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("linux_audio_end_to_end");
    group.sample_size(50);
    group.bench_function("8_sources_capture_ring_mix_policy", |b| {
        let (mut sources, mut runtime) = build_sources_and_runtime();
        let mut tick_index: u64 = 0;
        b.iter(|| {
            fill_sources(&mut sources, AUDIO_OUTPUT_FRAMES);
            let frame = runtime
                .run_one_tick_blocking(tick_index * MIX_TICK_PERIOD_NS)
                .expect("frame");
            tick_index = tick_index.wrapping_add(1);
            black_box(frame);
        });
    });
    group.finish();
}

#[allow(dead_code)]
fn _producer_helper(producer: SourceRingProducer) -> SourceRingProducer {
    producer
}

#[allow(dead_code)]
fn _ring_helper() -> usize {
    let _: usize = SourceRing::create(8192, 48_000).map(|_| 0).unwrap_or(0);
    0
}

criterion_group!(benches, bench_end_to_end_tick);
criterion_main!(benches);
