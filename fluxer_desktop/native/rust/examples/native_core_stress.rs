// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

use fluxer_desktop_native::input::ring::Ring;
use fluxer_desktop_native::linux_evdev::event::{EV_KEY, InputEvent, parse_input_events};
use fluxer_desktop_native::mac_app_audio::process_tree::{
    Info, collect_related_pids_with_resolver,
};
use fluxer_desktop_native::mac_app_audio::source_state::{Machine, State};

fn stress_source_state() {
    for _ in 0..10_000 {
        let machine = Arc::new(Machine::new());
        machine.request_start().expect("request start");
        let stop_wins = Arc::new(AtomicU64::new(0));
        let run_machine = Arc::clone(&machine);
        let stop_machine = Arc::clone(&machine);
        let stop_counter = Arc::clone(&stop_wins);
        let run_thread = thread::spawn(move || {
            let _ = run_machine.mark_running();
        });
        let stop_thread = thread::spawn(move || {
            for _ in 0..1000 {
                if stop_machine.request_stop().is_ok() {
                    stop_counter.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                std::hint::spin_loop();
            }
        });
        run_thread.join().expect("run thread");
        stop_thread.join().expect("stop thread");
        if machine.current() == State::Running {
            machine.request_stop().expect("request stop after run");
        }
        assert_eq!(State::Stopping, machine.current());
        assert!(stop_wins.load(Ordering::Relaxed) <= 1);
    }
}

fn stress_ring() {
    let mut ring: Ring<u64, 4096> = Ring::new();
    for cycle in 0..2048_u64 {
        for index in 0..4096_u64 {
            let slot = ring.claim().expect("slot") as usize;
            ring.slots[slot] = cycle.wrapping_mul(4096).wrapping_add(index);
        }
        assert!(ring.claim().is_none());
        for index in 0..4096_u64 {
            let slot = ring.pop().expect("slot") as usize;
            assert_eq!(
                cycle.wrapping_mul(4096).wrapping_add(index),
                ring.slots[slot]
            );
            ring.release();
        }
        assert!(ring.pop().is_none());
    }
}

fn stress_evdev_parser() {
    let mut bytes = Vec::with_capacity(InputEvent::BYTE_LEN * 100_000);
    for index in 0..100_000_i32 {
        let event = InputEvent {
            time_sec: i64::from(index),
            time_usec: i64::from(index * 10),
            event_type: EV_KEY,
            code: 30,
            value: index & 1,
        };
        bytes.extend_from_slice(&event.time_sec.to_ne_bytes());
        bytes.extend_from_slice(&event.time_usec.to_ne_bytes());
        bytes.extend_from_slice(&event.event_type.to_ne_bytes());
        bytes.extend_from_slice(&event.code.to_ne_bytes());
        bytes.extend_from_slice(&event.value.to_ne_bytes());
    }
    let mut count = 0_usize;
    for event in parse_input_events(&bytes) {
        assert_eq!(EV_KEY, event.event_type);
        count += 1;
    }
    assert_eq!(100_000, count);
}

fn stress_process_tree() {
    let infos: Vec<Info> = (0..1024_i32)
        .map(|index| Info {
            pid: 20_000 + index,
            parent_pid: if index == 0 { 1 } else { 20_000 + index - 1 },
            process_group_id: 20_000,
        })
        .collect();
    let candidates: Vec<i32> = infos.iter().map(|info| info.pid).rev().collect();
    for _ in 0..1000 {
        let resolver = |pid| infos.iter().copied().find(|info| info.pid == pid);
        let related =
            collect_related_pids_with_resolver(20_000, Some(infos[0]), &candidates, 1024, resolver);
        assert_eq!(1024, related.len());
        assert_eq!(20_000, related[0]);
    }
}

fn main() {
    stress_source_state();
    stress_ring();
    stress_evdev_parser();
    stress_process_tree();
    println!("native core stress completed");
}
