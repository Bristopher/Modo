// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_desktop_native::audio::contract::{
    DIRECT_CAPTURE_CHANNELS, DIRECT_CAPTURE_MAX_READ_SAMPLES, bounded_direct_read_sample_count,
    direct_whole_frame_sample_count, whole_frame_sample_count,
};
use fluxer_desktop_native::input::ring::Ring;
use fluxer_desktop_native::linux_audio::routing::{
    MEDIA_CLASS_PLAYBACK_STREAM, RoutingRule, SelfIdentity, map, matches_pattern, should_route_node,
};
use fluxer_desktop_native::linux_evdev::event::{InputEvent, parse_input_event};
use fluxer_desktop_native::linux_portals::pid_payload::parse_shell_eval_pid_payload;
use fluxer_desktop_native::mac_app_audio::process_tree::{
    Info, collect_related_pids_with_resolver, is_same_launch_tree_with_resolver,
};
use proptest::prelude::*;

fn input_event_bytes(event: InputEvent) -> [u8; InputEvent::BYTE_LEN] {
    let mut out = [0_u8; InputEvent::BYTE_LEN];
    out[0..8].copy_from_slice(&event.time_sec.to_ne_bytes());
    out[8..16].copy_from_slice(&event.time_usec.to_ne_bytes());
    out[16..18].copy_from_slice(&event.event_type.to_ne_bytes());
    out[18..20].copy_from_slice(&event.code.to_ne_bytes());
    out[20..24].copy_from_slice(&event.value.to_ne_bytes());
    out
}

proptest! {
    #[test]
    fn whole_frame_count_never_exceeds_input_and_is_channel_aligned(sample_count in 0usize..1_000_000, channels in 0u32..16) {
        let count = whole_frame_sample_count(sample_count, channels);
        prop_assert!(count <= sample_count);
        if channels == 0 {
            prop_assert_eq!(0, count);
        } else {
            prop_assert_eq!(0, count % channels as usize);
        }
    }

    #[test]
    fn direct_read_bound_is_stereo_aligned_and_capped(available in 0usize..1_000_000) {
        let count = bounded_direct_read_sample_count(available);
        prop_assert!(count <= DIRECT_CAPTURE_MAX_READ_SAMPLES);
        prop_assert_eq!(0, count % DIRECT_CAPTURE_CHANNELS as usize);
    }

    #[test]
    fn pid_payload_returns_positive_u32_digit_runs(prefix in "[A-Za-z_\\[\\], ]*", pid in 1u32..u32::MAX, suffix in "[A-Za-z_\\[\\], ]*") {
        let payload = format!("{prefix}{pid}{suffix}");
        prop_assert_eq!(Some(pid), parse_shell_eval_pid_payload(&payload));
    }

    #[test]
    fn routing_pattern_matching_is_subset_exact(key in "[a-z.]{1,32}", value in "[a-z0-9_-]{1,32}", other in "[a-z0-9_-]{1,32}") {
        let candidate = map(&[(&key, &value)]);
        let matching = map(&[(&key, &value)]);
        prop_assert!(matches_pattern(&candidate, &matching));
        if other != value {
            let mismatched = map(&[(&key, &other)]);
            prop_assert!(!matches_pattern(&candidate, &mismatched));
        }
    }

    #[test]
    fn evdev_input_event_parser_round_trips_native_endian_fields(
        time_sec in any::<i64>(),
        time_usec in any::<i64>(),
        event_type in any::<u16>(),
        code in any::<u16>(),
        value in any::<i32>(),
    ) {
        let event = InputEvent {
            time_sec,
            time_usec,
            event_type,
            code,
            value,
        };
        prop_assert_eq!(Some(event), parse_input_event(&input_event_bytes(event)));
    }

    #[test]
    fn mac_process_tree_direct_children_are_collected(target in 2i32..100_000, child_delta in 1i32..1000) {
        let child = target + child_delta;
        let infos = [
            Info { pid: target, parent_pid: 1, process_group_id: target },
            Info { pid: child, parent_pid: target, process_group_id: target },
        ];
        let resolver = |pid| infos.iter().copied().find(|info| info.pid == pid);
        prop_assert!(is_same_launch_tree_with_resolver(child, target, Some(infos[0]), resolver));
        let resolver = |pid| infos.iter().copied().find(|info| info.pid == pid);
        prop_assert_eq!(
            vec![target, child],
            collect_related_pids_with_resolver(target, Some(infos[0]), &[child], 4, resolver)
        );
    }
}

#[test]
fn ring_stress_preserves_fifo_under_repeated_fill_drain_cycles() {
    let mut ring: Ring<u64, 1024> = Ring::new();
    for cycle in 0..512_u64 {
        for index in 0..1024_u64 {
            let slot = ring.claim().expect("slot") as usize;
            ring.slots[slot] = cycle * 10_000 + index;
        }
        assert!(ring.claim().is_none());
        for index in 0..1024_u64 {
            let slot = ring.pop().expect("slot") as usize;
            assert_eq!(cycle * 10_000 + index, ring.slots[slot]);
            ring.release();
        }
        assert!(ring.pop().is_none());
    }
}

#[test]
fn self_identity_always_beats_user_include_rules() {
    let mut self_identity = SelfIdentity::default();
    self_identity.add_pid("42");
    self_identity.add_binary("fluxer");
    self_identity.add_display_name("Fluxer Canary");
    self_identity.add_display_prefix("Fluxer ");
    let rule = RoutingRule {
        include_when: vec![
            map(&[("application.process.id", "42")]),
            map(&[("application.name", "fluxer")]),
        ],
        ..RoutingRule::default()
    };
    for props in [
        map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("application.process.id", "42"),
        ]),
        map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("application.process.binary", "fluxer"),
        ]),
        map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("application.name", "fluxer"),
        ]),
        map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("node.name", "fluxer"),
        ]),
        map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            (
                "node.description",
                "Fluxer desktop audio tap excluding pid 42",
            ),
        ]),
    ] {
        assert!(!should_route_node(
            1,
            &props,
            &rule,
            "",
            "",
            0,
            &self_identity
        ));
    }
}

#[test]
fn direct_whole_frame_count_is_same_as_generic_stereo_helper() {
    for value in 0..10_000 {
        assert_eq!(
            whole_frame_sample_count(value, 2),
            direct_whole_frame_sample_count(value)
        );
    }
}
