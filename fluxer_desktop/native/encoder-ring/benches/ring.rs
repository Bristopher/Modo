// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_encoder_ring::{CpuMemcpyBackend, EncoderInputRing, TextureFormat};

fn bench_submit_poll_release_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoder_input_ring");
    group.sample_size(60);
    group.bench_function("submit_poll_release_1080p_nv12", |b| {
        let mut ring = EncoderInputRing::new(CpuMemcpyBackend::new());
        ring.initialise(1920, 1080, TextureFormat::Nv12)
            .expect("init");
        b.iter(|| {
            ring.submit(|_h| {}).expect("submit");
            let ready = ring.poll_next_ready().expect("ready");
            black_box(&ready);
            ring.release_completed(ready).expect("release");
        });
    });
    group.finish();
}

fn bench_submit_only_drained_after(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoder_input_ring");
    group.sample_size(60);
    group.bench_function("submit_then_drain_eight_1080p_nv12", |b| {
        b.iter(|| {
            let mut ring = EncoderInputRing::new(CpuMemcpyBackend::new());
            ring.initialise(1920, 1080, TextureFormat::Nv12)
                .expect("init");
            for _ in 0..8 {
                ring.submit(|_h| {}).expect("submit");
            }
            for _ in 0..8 {
                let ready = ring.poll_next_ready().expect("ready");
                ring.release_completed(ready).expect("release");
            }
            black_box(ring.metrics());
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_submit_poll_release_cycle,
    bench_submit_only_drained_after
);
criterion_main!(benches);
