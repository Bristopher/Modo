#![no_main]

// SPDX-License-Identifier: AGPL-3.0-or-later

use arbitrary::Arbitrary;
use fluxer_desktop_native::mac_app_audio::process_tree::{
    Info, collect_related_pids_with_resolver, is_same_launch_tree_with_resolver,
};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct RawInfo {
    pid: i32,
    parent_pid: i32,
    process_group_id: i32,
}

#[derive(Arbitrary, Debug)]
struct Input {
    target_pid: i32,
    max_count: u8,
    infos: Vec<RawInfo>,
    candidates: Vec<i32>,
}

fuzz_target!(|input: Input| {
    let infos: Vec<Info> = input
        .infos
        .into_iter()
        .take(128)
        .map(|raw| Info {
            pid: raw.pid,
            parent_pid: raw.parent_pid,
            process_group_id: raw.process_group_id,
        })
        .collect();
    let candidates: Vec<i32> = input.candidates.into_iter().take(128).collect();
    let target_info = infos
        .iter()
        .copied()
        .find(|info| info.pid == input.target_pid);
    let resolver = |pid| infos.iter().copied().find(|info| info.pid == pid);
    let related = collect_related_pids_with_resolver(
        input.target_pid,
        target_info,
        &candidates,
        usize::from(input.max_count),
        resolver,
    );
    assert!(related.len() <= usize::from(input.max_count));
    if input.target_pid <= 0 || input.max_count == 0 {
        assert!(related.is_empty());
    } else {
        assert_eq!(Some(&input.target_pid), related.first());
    }

    for candidate in candidates.into_iter().take(16) {
        let resolver = |pid| infos.iter().copied().find(|info| info.pid == pid);
        let _ =
            is_same_launch_tree_with_resolver(candidate, input.target_pid, target_info, resolver);
    }
});
