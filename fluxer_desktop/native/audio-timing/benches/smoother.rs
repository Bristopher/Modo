// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_audio_timing::{AudioTimingSmoother, SmootherInput, SourceGainRamp, StaleSourceTracker};

const SR: u32 = 48_000;

const FRAMES_PER_TICK: u32 = 960;

const PERIOD_NS: u64 = 20_000_000;

fn bench_smoother_steady_snap(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_timing_smoother");
    group.sample_size(100);
    group.bench_function("snap_steady_state", |b| {
        let mut s = AudioTimingSmoother::new(1, SR).expect("ok");
        let _ = s
            .adjust(SmootherInput {
                media_ts_ns: 0,
                frames: FRAMES_PER_TICK,
                wall_ns: 0,
            })
            .expect("ok");
        let mut n: u64 = 1;
        b.iter(|| {
            let result = s
                .adjust(SmootherInput {
                    media_ts_ns: n * PERIOD_NS,
                    frames: FRAMES_PER_TICK,
                    wall_ns: n * PERIOD_NS,
                })
                .expect("ok");
            n = n.wrapping_add(1);
            black_box(result);
        });
    });
    group.finish();
}

fn bench_ramp_advance(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_timing_ramp");
    group.sample_size(100);
    group.bench_function("advance_and_query_gain", |b| {
        let mut r = SourceGainRamp::new();
        b.iter(|| {
            r.advance_tick();
            let g = r.current_gain_q15();
            if r.is_complete() {
                r.reset();
            }
            black_box(g);
        });
    });
    group.finish();
}

fn bench_stale_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_timing_eviction");
    group.sample_size(100);
    group.bench_function("is_stale_lookup", |b| {
        let mut t = StaleSourceTracker::new(16).expect("ok");
        for n in 1..=16u64 {
            t.register_source(n, n * 1_000).expect("ok");
        }
        let threshold = 5_000_000_000u64;
        let mut now: u64 = 1_000_000_000;
        b.iter(|| {
            let stale = t.is_stale(8, now, threshold).expect("ok");
            now = now.wrapping_add(PERIOD_NS);
            black_box(stale);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_smoother_steady_snap,
    bench_ramp_advance,
    bench_stale_check
);
criterion_main!(benches);
