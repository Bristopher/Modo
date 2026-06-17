// SPDX-License-Identifier: AGPL-3.0-or-later

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use fluxer_desktop_native::input::ring::Ring;
use fluxer_desktop_native::linux_audio::routing::{
    MEDIA_CLASS_PLAYBACK_STREAM, SelfIdentity, map, should_route_node, system_rule,
};
use fluxer_desktop_native::linux_evdev::keymap::{KEY_MAP, keycode_to_name, name_to_keycode};
use fluxer_desktop_native::mac_app_audio::audio_converter::{
    AudioBuffer, AudioBufferListN, build_input_asbd, convert_buffer_list_to_interleaved_f32,
};
use fluxer_desktop_native::mac_app_audio::process_tree::{
    Info, collect_related_pids_with_resolver,
};

fn bench_routing(c: &mut Criterion) {
    let self_identity = SelfIdentity::default();
    let rule = system_rule();
    let props = map(&[
        ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        (
            "target.object",
            "alsa_output.pci-0000_00_1f.3.analog-stereo",
        ),
    ]);
    c.bench_function("routing/default_sink_playback_stream", |b| {
        b.iter(|| {
            black_box(should_route_node(
                100,
                black_box(&props),
                black_box(&rule),
                "alsa_output.pci-0000_00_1f.3.analog-stereo",
                "",
                1,
                &self_identity,
            ))
        })
    });
}

fn bench_ring(c: &mut Criterion) {
    c.bench_function("ring/fill_drain_1024", |b| {
        b.iter(|| {
            let mut ring: Ring<u32, 1024> = Ring::new();
            for index in 0..1024 {
                let slot = ring.claim().unwrap() as usize;
                ring.slots[slot] = index;
            }
            let mut sum = 0_u32;
            while let Some(slot) = ring.pop() {
                sum = sum.wrapping_add(ring.slots[slot as usize]);
                ring.release();
            }
            black_box(sum)
        })
    });
}

fn bench_audio_convert(c: &mut Criterion) {
    let samples: Vec<f32> = (0..48_000).map(|i| (i as f32 / 48_000.0).sin()).collect();
    let list = AudioBufferListN {
        m_number_buffers: 1,
        buffers: [AudioBuffer::from_slice(1, &samples)],
    };
    let asbd = build_input_asbd(48_000.0, 1, false);
    let mut out = vec![0.0_f32; samples.len() * 2];
    c.bench_function("audio/mono_to_stereo_1s", |b| {
        b.iter(|| {
            black_box(
                convert_buffer_list_to_interleaved_f32(
                    asbd,
                    &list,
                    samples.len() as u32,
                    48_000.0,
                    2,
                    &mut out,
                )
                .unwrap(),
            )
        })
    });
}

fn bench_evdev_keymap(c: &mut Criterion) {
    c.bench_function("evdev/keymap_roundtrip_table", |b| {
        b.iter(|| {
            let mut sum = 0_u16;
            for entry in KEY_MAP {
                sum ^= black_box(name_to_keycode(black_box(entry.name)));
                black_box(keycode_to_name(black_box(entry.code)));
            }
            black_box(sum)
        })
    });
}

fn bench_process_tree_collect(c: &mut Criterion) {
    let infos: Vec<Info> = (0..512_i32)
        .map(|index| Info {
            pid: 10_000 + index,
            parent_pid: if index == 0 { 1 } else { 10_000 + index - 1 },
            process_group_id: 10_000,
        })
        .collect();
    let candidates: Vec<i32> = infos.iter().map(|info| info.pid).rev().collect();
    c.bench_function("mac_process_tree/collect_512_chain", |b| {
        b.iter(|| {
            let resolver = |pid| infos.iter().copied().find(|info| info.pid == pid);
            black_box(collect_related_pids_with_resolver(
                10_000,
                Some(infos[0]),
                black_box(&candidates),
                512,
                resolver,
            ))
        })
    });
}

criterion_group!(
    benches,
    bench_routing,
    bench_ring,
    bench_audio_convert,
    bench_evdev_keymap,
    bench_process_tree_collect
);
criterion_main!(benches);
