#![no_main]

// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_desktop_native::linux_evdev::event::{
    InputEvent, parse_input_event, parse_input_events,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = parse_input_event(data);
    let mut count = 0_usize;
    for event in parse_input_events(data) {
        let _ = event.time_sec ^ event.time_usec;
        let _ = event.event_type ^ event.code;
        let _ = event.value;
        count += 1;
    }
    assert_eq!(data.len() / InputEvent::BYTE_LEN, count);
});
