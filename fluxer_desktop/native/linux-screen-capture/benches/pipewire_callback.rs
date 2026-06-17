// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_linux_screen_capture::frame_buffer_pool::{
    LINUX_SCREEN_FRAME_POOL_CAP, LinuxFrameBufferPool,
};
use fluxer_linux_screen_capture::nv12_packing::{Nv12Layout, bgra_to_nv12};

const BENCH_NV12_WIDTH: usize = 1920;
const BENCH_NV12_HEIGHT: usize = 1080;
const BENCH_NV12_BYTES: usize = BENCH_NV12_WIDTH * BENCH_NV12_HEIGHT * 3 / 2;

fn bench_pool_acquire_release_steady_state(c: &mut Criterion) {
    assert_eq!(LINUX_SCREEN_FRAME_POOL_CAP, 8);
    let pool = LinuxFrameBufferPool::new(BENCH_NV12_BYTES)
        .expect("pool must allocate at construction time");
    assert_eq!(pool.capacity(), LINUX_SCREEN_FRAME_POOL_CAP);
    assert_eq!(pool.bytes_per_buffer(), BENCH_NV12_BYTES);

    c.bench_function(
        "linux_screen_capture/pool_acquire_release/1080p_nv12",
        |b| {
            b.iter(|| {
                let mut pooled = pool
                    .try_acquire()
                    .expect("steady state single-thread must not starve");
                let buf = pooled.buffer_mut();
                buf[0] = black_box(buf.len() as u8);
                pooled.set_len(BENCH_NV12_BYTES);
                black_box(pooled.slot_index());
            });
        },
    );
    assert_eq!(pool.currently_in_flight(), 0);
}

fn bench_simulated_callback_fill_path(c: &mut Criterion) {
    assert_eq!(LINUX_SCREEN_FRAME_POOL_CAP, 8);
    let pool = LinuxFrameBufferPool::new(BENCH_NV12_BYTES).expect("pool init");

    let source = vec![0xA5u8; BENCH_NV12_BYTES];
    assert_eq!(source.len(), BENCH_NV12_BYTES);

    c.bench_function(
        "linux_screen_capture/simulated_callback/1080p_nv12_copy",
        |b| {
            b.iter(|| {
                let mut pooled = pool.try_acquire().expect("pool capacity");
                let buf = pooled.buffer_mut();
                buf[..BENCH_NV12_BYTES].copy_from_slice(&source);
                pooled.set_len(BENCH_NV12_BYTES);
                black_box(pooled.as_slice().len());
            });
        },
    );
    assert_eq!(pool.currently_in_flight(), 0);
}

fn bench_legacy_vec_clone_baseline(c: &mut Criterion) {
    let source = vec![0xA5u8; BENCH_NV12_BYTES];
    assert_eq!(source.len(), BENCH_NV12_BYTES);
    let mut scratch = vec![0u8; BENCH_NV12_BYTES];

    c.bench_function(
        "linux_screen_capture/legacy_baseline/1080p_nv12_vec_clone",
        |b| {
            b.iter(|| {
                scratch.copy_from_slice(&source);
                let cloned = scratch.clone();
                black_box(cloned.len());
            });
        },
    );
}

const BENCH_4K_WIDTH: usize = 3840;
const BENCH_4K_HEIGHT: usize = 2160;

fn bench_bgra_to_nv12_4k_conversion(c: &mut Criterion) {
    let layout = Nv12Layout {
        width: BENCH_4K_WIDTH as u32,
        height: BENCH_4K_HEIGHT as u32,
        stride_y: BENCH_4K_WIDTH as u32,
        stride_uv: BENCH_4K_WIDTH as u32,
    };
    let bgra_stride = (BENCH_4K_WIDTH * 4) as u32;
    let mut bgra = vec![0u8; BENCH_4K_WIDTH * BENCH_4K_HEIGHT * 4];
    for (index, byte) in bgra.iter_mut().enumerate() {
        *byte = (index % 253) as u8;
    }
    let total = layout.packed_size().expect("4K layout is valid");
    let mut dst = vec![0u8; total];

    c.bench_function("linux_screen_capture/bgra_to_nv12/4k_unflipped", |b| {
        b.iter(|| {
            let ok = bgra_to_nv12(layout, &bgra, bgra_stride, &mut dst, false);
            assert!(ok);
            black_box(dst[0]);
        });
    });

    c.bench_function("linux_screen_capture/bgra_to_nv12/4k_flipped", |b| {
        b.iter(|| {
            let ok = bgra_to_nv12(layout, &bgra, bgra_stride, &mut dst, true);
            assert!(ok);
            black_box(dst[total - 1]);
        });
    });
}

criterion_group!(
    benches,
    bench_pool_acquire_release_steady_state,
    bench_simulated_callback_fill_path,
    bench_legacy_vec_clone_baseline,
    bench_bgra_to_nv12_4k_conversion,
);
criterion_main!(benches);
