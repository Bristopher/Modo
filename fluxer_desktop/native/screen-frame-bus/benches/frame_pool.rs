// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_screen_frame_bus::frame_pool::{CpuFrameBuilder, FRAME_BYTES_MAX};

fn bench_acquire_release_cpu_1080p_rgba(c: &mut Criterion) {
    assert_eq!(FRAME_BYTES_MAX, 1920 * 1080 * 4);
    let pool = CpuFrameBuilder::build_pool(FRAME_BYTES_MAX)
        .expect("pool must allocate at construction time");
    assert!(pool.capacity() > 0);

    c.bench_function("frame_pool/acquire_release/cpu_1080p_rgba", |b| {
        b.iter(|| {
            let frame = pool
                .try_acquire()
                .expect("steady state single-threaded never starves");
            black_box(frame.slot_index());
        });
    });
    assert_eq!(pool.currently_in_flight(), 0);
}

criterion_group!(benches, bench_acquire_release_cpu_1080p_rgba);
criterion_main!(benches);
