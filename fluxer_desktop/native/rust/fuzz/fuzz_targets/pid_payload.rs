#![no_main]

// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_desktop_native::linux_portals::pid_payload::parse_shell_eval_pid_payload;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(payload) = std::str::from_utf8(data) {
        let _ = parse_shell_eval_pid_payload(payload);
    }
});
