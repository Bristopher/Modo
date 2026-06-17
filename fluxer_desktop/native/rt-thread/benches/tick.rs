// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;
use std::sync::Arc;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_rt_thread::{MonotonicClock, SystemMonotonicClock, TickDriver};

const AUDIO_TICK_PERIOD_NS: u64 = 21_333_333;

fn bench_audio_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_driver");
    group.sample_size(20);
    group.bench_function("audio_21_33ms", |b| {
        b.iter_custom(|iters| {
            let clock = Arc::new(SystemMonotonicClock::new());
            let mut driver = TickDriver::new(clock.clone(), AUDIO_TICK_PERIOD_NS).expect("driver");
            let _ = driver.wait_until_next_tick().expect("warmup");
            let start = std::time::Instant::now();
            for _ in 0..iters {
                let info = driver.wait_until_next_tick().expect("tick");
                black_box(info);
            }
            start.elapsed()
        })
    });
    group.finish();
}

fn bench_now_ns(c: &mut Criterion) {
    let clock = SystemMonotonicClock::new();
    c.bench_function("system_clock_now_ns", |b| {
        b.iter(|| {
            let v = MonotonicClock::now_ns(&clock);
            black_box(v)
        })
    });
}

criterion_group!(benches, bench_audio_tick, bench_now_ns);
criterion_main!(benches);
