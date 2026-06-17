#![no_main]

// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_media_proxy::thumbhash;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    let w = (data[0] as u32 % thumbhash::MAX_DIM).max(1);
    let h = (data[1] as u32 % thumbhash::MAX_DIM).max(1);
    let needed = w as usize * h as usize * 4;
    if data.len() < needed + 2 {
        return;
    }
    let _ = thumbhash::encode_rgba(&data[2..2 + needed], w, h);
});
