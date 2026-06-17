// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_mac_screen_capture::audio_pool::{
    MAC_AUDIO_POOL_CAP, MAX_FRAME_BYTES_PER_SLOT, MacAudioFramePool,
};

fn bench_acquire_write_release_960_floats(c: &mut Criterion) {
    let pool = MacAudioFramePool::new(MAC_AUDIO_POOL_CAP, MAX_FRAME_BYTES_PER_SLOT)
        .expect("pool must build");
    let payload = vec![0xCC_u8; 960 * 4];
    c.bench_function("audio_pool/acquire_write_release/960_floats", |b| {
        b.iter(|| {
            let mut slot = pool
                .try_acquire()
                .expect("steady state single-thread never starves");
            slot.write(&payload).expect("write fits");
            black_box(slot.data_slice().len());
            drop(slot);
        });
    });
    assert_eq!(pool.stats().in_flight, 0);
}

fn bench_acquire_release_only(c: &mut Criterion) {
    let pool = MacAudioFramePool::new(MAC_AUDIO_POOL_CAP, MAX_FRAME_BYTES_PER_SLOT)
        .expect("pool must build");
    c.bench_function("audio_pool/acquire_release", |b| {
        b.iter(|| {
            let slot = pool.try_acquire().expect("slot");
            black_box(slot.slot_index());
        });
    });
    assert_eq!(pool.stats().in_flight, 0);
}

fn bench_acquire_write_into_external_parts(c: &mut Criterion) {
    let pool = MacAudioFramePool::new(MAC_AUDIO_POOL_CAP, MAX_FRAME_BYTES_PER_SLOT)
        .expect("pool must build");
    let payload = vec![0xCC_u8; 960 * 4];
    c.bench_function(
        "audio_pool/acquire_write_into_external_parts/960_floats",
        |b| {
            b.iter(|| {
                let mut slot = pool
                    .try_acquire()
                    .expect("steady state single-thread never starves");
                slot.write(&payload).expect("write fits");
                let (ptr, len, owned) = slot.into_external_parts();
                black_box((ptr, len));
                drop(owned);
            });
        },
    );
    assert_eq!(pool.stats().in_flight, 0);
}

criterion_group!(
    benches,
    bench_acquire_write_release_960_floats,
    bench_acquire_release_only,
    bench_acquire_write_into_external_parts,
);
criterion_main!(benches);
