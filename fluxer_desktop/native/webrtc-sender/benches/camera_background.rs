// SPDX-License-Identifier: AGPL-3.0-or-later

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use fluxer_webrtc_sender::bench_internals::{
    BlurScratch, MaskRefiner, blur_plane_masked, composite_masked_plane,
};
use std::hint::black_box;

const BENCH_WIDTH: usize = 1280;
const BENCH_HEIGHT: usize = 720;
const BENCH_BLUR_RADIUS_PASS: usize = 6;
const BENCH_MASK_FEATHER_PX: usize = 64;

fn synth_plane(seed: usize) -> Vec<u8> {
    let mut plane = vec![0u8; BENCH_WIDTH * BENCH_HEIGHT];
    for (index, value) in plane.iter_mut().enumerate() {
        *value = ((index * 31 + seed) % 251) as u8;
    }
    plane
}

fn synth_person_mask() -> Vec<u8> {
    let mut mask = vec![0u8; BENCH_WIDTH * BENCH_HEIGHT];
    let person_edge = BENCH_WIDTH / 2;
    for row in mask.chunks_exact_mut(BENCH_WIDTH) {
        for (x, value) in row.iter_mut().enumerate() {
            *value = if x < person_edge {
                255
            } else if x < person_edge + BENCH_MASK_FEATHER_PX {
                (255 - (x - person_edge) * 255 / BENCH_MASK_FEATHER_PX) as u8
            } else {
                0
            };
        }
    }
    mask
}

fn bench_blur_plane_masked(c: &mut Criterion) {
    let source = synth_plane(7);
    let mask = synth_person_mask();
    let mut plane = source.clone();
    let mut scratch = BlurScratch::new(BENCH_WIDTH, BENCH_HEIGHT);

    let mut group = c.benchmark_group("camera_background::blur_plane_masked");
    group.throughput(Throughput::Bytes((BENCH_WIDTH * BENCH_HEIGHT) as u64));
    group.bench_function("1280x720_radius_pass6", |b| {
        b.iter(|| {
            plane.copy_from_slice(&source);
            blur_plane_masked(
                black_box(&mut plane),
                BENCH_WIDTH,
                BENCH_HEIGHT,
                black_box(&mask),
                BENCH_BLUR_RADIUS_PASS,
                &mut scratch,
            );
            black_box(&plane);
        })
    });
    group.finish();
}

fn bench_mask_refine(c: &mut Criterion) {
    let luma = synth_plane(13);
    let raw_mask = synth_person_mask();
    let mut mask = raw_mask.clone();
    let mut refiner = MaskRefiner::new(BENCH_WIDTH, BENCH_HEIGHT);

    let mut group = c.benchmark_group("camera_background::mask_refine");
    group.throughput(Throughput::Bytes((BENCH_WIDTH * BENCH_HEIGHT) as u64));
    group.bench_function("1280x720_guided_refine", |b| {
        b.iter(|| {
            mask.copy_from_slice(&raw_mask);
            refiner.refine(black_box(&luma), black_box(&mut mask));
            black_box(&mask);
        })
    });
    group.finish();
}

fn bench_composite_masked_plane(c: &mut Criterion) {
    let source = synth_plane(11);
    let background = synth_plane(151);
    let mask = synth_person_mask();
    let mut plane = source.clone();

    let mut group = c.benchmark_group("camera_background::composite_masked_plane");
    group.throughput(Throughput::Bytes((BENCH_WIDTH * BENCH_HEIGHT) as u64));
    group.bench_function("1280x720_feathered_mask", |b| {
        b.iter(|| {
            plane.copy_from_slice(&source);
            composite_masked_plane(
                black_box(&mut plane),
                black_box(&background),
                BENCH_WIDTH,
                BENCH_HEIGHT,
                &mask,
            );
            black_box(&plane);
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_blur_plane_masked,
    bench_mask_refine,
    bench_composite_masked_plane
);
criterion_main!(benches);
