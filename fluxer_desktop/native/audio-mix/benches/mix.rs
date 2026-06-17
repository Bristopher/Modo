// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_audio_mix::{
    AUDIO_OUTPUT_FRAMES, AudioMixSession, SourceRing, SourceRingConsumer, SourceRingProducer,
};
use fluxer_rt_thread::TickInfo;

const BENCH_SOURCE_COUNT: usize = 8;

fn build_session(source_count: usize) -> (Vec<SourceRingProducer>, AudioMixSession) {
    assert!(source_count > 0);
    let mut producers = Vec::with_capacity(source_count);
    let mut consumers: Vec<SourceRingConsumer> = Vec::with_capacity(source_count);
    for _ in 0..source_count {
        let (producer, consumer) = SourceRing::create(8192, 48_000).expect("pair");
        producers.push(producer);
        consumers.push(consumer);
    }
    let session = AudioMixSession::new(consumers, AUDIO_OUTPUT_FRAMES).expect("session");
    (producers, session)
}

fn synthetic_tick(index: u64) -> TickInfo {
    let scheduled_ns = index * 21_333_333;
    TickInfo {
        tick_index: index,
        scheduled_ns,
        actual_ns: scheduled_ns,
        lag_ns: 0,
    }
}

fn fill_producers(producers: &mut [SourceRingProducer], frames: usize) {
    assert!(!producers.is_empty());
    assert!(frames > 0);
    let payload: Vec<i16> = (0..frames).map(|n| ((n as i16) % 4096) - 2048).collect();
    for producer in producers.iter_mut() {
        let pushed = producer.try_push_slice(&payload);
        assert!(pushed > 0);
    }
}

fn bench_mix_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_mix_tick");
    group.sample_size(50);
    group.bench_function("mix_8_sources_1024_samples", |b| {
        let (mut producers, mut session) = build_session(BENCH_SOURCE_COUNT);
        let mut tick_index: u64 = 0;
        b.iter(|| {
            fill_producers(&mut producers, AUDIO_OUTPUT_FRAMES);
            let result = session.tick(synthetic_tick(tick_index));
            tick_index = tick_index.wrapping_add(1);
            black_box(result);
        });
    });
    group.finish();
}

criterion_group!(benches, bench_mix_tick);
criterion_main!(benches);
