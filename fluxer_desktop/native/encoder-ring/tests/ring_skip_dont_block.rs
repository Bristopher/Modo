// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_encoder_ring::{
    CpuMemcpyBackend, CpuSlotHandle, EncoderInputRing, RING_SIZE, TextureFormat,
};

fn fill_noop(_: &mut CpuSlotHandle) {}

#[test]
fn submit_skip_oldest_drops_first_half_when_pushed_twice_capacity() {
    let mut ring = EncoderInputRing::new(CpuMemcpyBackend::new());
    ring.initialise(64, 64, TextureFormat::Nv12)
        .expect("init ring");
    let capacity = ring.capacity();
    assert_eq!(capacity, RING_SIZE);
    assert_eq!(capacity, 8);
    let total_pushes: u64 = 2 * capacity as u64;
    for _ in 0..total_pushes {
        ring.submit_skip_oldest(fill_noop)
            .expect("skip-oldest accepts every push");
    }
    assert_eq!(ring.submitted_count(), total_pushes);
    assert_eq!(ring.dropped_count(), capacity as u64);
    assert_eq!(ring.free_count(), 0);
    let mut observed: Vec<u64> = Vec::with_capacity(capacity);
    for _ in 0..capacity {
        let ready = ring.poll_next_ready().expect("ready");
        observed.push(ready.sequence);
        ring.release_completed(ready).expect("release");
    }
    let expected: Vec<u64> = (capacity as u64 + 1..=total_pushes).collect();
    assert_eq!(
        observed, expected,
        "skip-don't-block kept newest {capacity} frames"
    );
    assert_eq!(ring.free_count(), capacity);
    assert!(ring.poll_next_ready().is_none());
}

#[test]
fn submit_skip_oldest_succeeds_when_room_available() {
    let mut ring = EncoderInputRing::new(CpuMemcpyBackend::new());
    ring.initialise(64, 64, TextureFormat::Nv12)
        .expect("init ring");
    for _ in 0..3 {
        ring.submit_skip_oldest(fill_noop).expect("submit");
    }
    assert_eq!(ring.dropped_count(), 0);
    assert_eq!(ring.submitted_count(), 3);
    assert_eq!(ring.free_count(), ring.capacity() - 3);
}

#[test]
fn submit_skip_oldest_keeps_metrics_consistent() {
    let mut ring = EncoderInputRing::new(CpuMemcpyBackend::new());
    ring.initialise(64, 64, TextureFormat::Nv12)
        .expect("init ring");
    for _ in 0..(3 * ring.capacity() as u64) {
        ring.submit_skip_oldest(fill_noop).expect("submit");
    }
    let metrics = ring.metrics();
    assert_eq!(metrics.submitted_count, 3 * ring.capacity() as u64);
    assert_eq!(metrics.dropped_count, 2 * ring.capacity() as u64);
    assert!(metrics.completed_count <= metrics.submitted_count);
}
