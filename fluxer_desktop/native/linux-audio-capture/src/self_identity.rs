// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::routing::SelfIdentity;

const PRODUCT_DISPLAY_NAMES: &[&str] = &["Fluxer", "Fluxer Canary"];
const PRODUCT_DISPLAY_PREFIXES: &[&str] = &[
    "Fluxer ", "fluxer ", "Fluxer-", "fluxer-", "Fluxer_", "fluxer_", "Fluxer.", "fluxer.",
];

pub fn populate_self_identity(out: &mut SelfIdentity) {
    let own_pid = std::process::id();
    out.add_pid(own_pid.to_string());
    for name in PRODUCT_DISPLAY_NAMES {
        out.add_display_name((*name).to_string());
    }
    for prefix in PRODUCT_DISPLAY_PREFIXES {
        out.add_display_prefix((*prefix).to_string());
    }

    if let Ok(comm) = fs::read_to_string("/proc/self/comm") {
        let trimmed = comm.trim();
        if !trimmed.is_empty() {
            out.add_binary(trimmed.to_string());
        }
    }

    if let Ok(exe) = fs::read_link("/proc/self/exe")
        && let Some(name) = exe.file_name().and_then(|s| s.to_str())
    {
        out.add_binary(name.to_string());
        if let Some(stripped) = strip_exe_suffix(name) {
            out.add_binary(stripped.to_string());
        }
    }

    out.add_binary("Electron".to_string());
    out.add_binary("electron".to_string());

    let _ = add_descendant_pids(out, own_pid);
}

fn strip_exe_suffix(name: &str) -> Option<&str> {
    for suffix in [".AppImage", ".bin"] {
        if name.len() >= suffix.len()
            && name[name.len() - suffix.len()..].eq_ignore_ascii_case(suffix)
        {
            return Some(&name[..name.len() - suffix.len()]);
        }
    }
    None
}

pub fn parse_ppid_from_stat(blob: &str) -> Option<u32> {
    let close = blob.rfind(')')?;
    let tail = &blob[close + 1..];
    let mut fields = tail.split_ascii_whitespace();
    let _state = fields.next()?;
    let ppid = fields.next()?;
    ppid.parse::<u32>().ok()
}

fn add_descendant_pids(out: &mut SelfIdentity, own_pid: u32) -> std::io::Result<()> {
    let mut entries: Vec<(u32, u32)> = Vec::new();
    for entry in fs::read_dir(Path::new("/proc"))? {
        let Ok(entry) = entry else { continue };
        let Some(name) = entry.file_name().to_str().map(|s| s.to_string()) else {
            continue;
        };
        let Ok(pid) = name.parse::<u32>() else {
            continue;
        };
        let stat_path = format!("/proc/{pid}/stat");
        let Ok(blob) = fs::read_to_string(&stat_path) else {
            continue;
        };
        let Some(ppid) = parse_ppid_from_stat(&blob) else {
            continue;
        };
        entries.push((pid, ppid));
    }

    let mut by_parent: HashMap<u32, Vec<u32>> = HashMap::new();
    for &(pid, ppid) in &entries {
        by_parent.entry(ppid).or_default().push(pid);
    }

    let mut ours: HashSet<u32> = HashSet::new();
    ours.insert(own_pid);

    let mut frontier = vec![own_pid];
    while let Some(parent) = frontier.pop() {
        if let Some(children) = by_parent.get(&parent) {
            for &child in children {
                if ours.insert(child) {
                    frontier.push(child);
                }
            }
        }
    }

    for pid in ours {
        out.add_pid(pid.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ppid_handles_comm_with_spaces_and_parens() {
        let stat = "1234 (weird (comm) name) S 4321 1234 1234 0 -1 4194304 0 0 0 0";
        assert_eq!(Some(4321), parse_ppid_from_stat(stat));
    }

    #[test]
    fn parse_ppid_handles_plain_stat() {
        let stat = "42 (cat) R 7 42 7 34816 42 4194304 91 0 0 0";
        assert_eq!(Some(7), parse_ppid_from_stat(stat));
    }

    #[test]
    fn parse_ppid_returns_none_on_malformed_input() {
        assert_eq!(None, parse_ppid_from_stat(""));
        assert_eq!(None, parse_ppid_from_stat("no closing paren"));
        assert_eq!(None, parse_ppid_from_stat("1 (cat)"));
    }

    #[test]
    fn strip_exe_suffix_handles_known_extensions() {
        assert_eq!(Some("fluxer"), strip_exe_suffix("fluxer.AppImage"));
        assert_eq!(Some("fluxer"), strip_exe_suffix("fluxer.bin"));
        assert_eq!(None, strip_exe_suffix("fluxer"));
        assert_eq!(Some("fluxer"), strip_exe_suffix("fluxer.APPIMAGE"));
    }

    #[test]
    fn populate_self_identity_records_own_pid() {
        let mut id = SelfIdentity::default();
        populate_self_identity(&mut id);
        assert!(id.pids.contains(&std::process::id().to_string()));

        assert!(id.binaries.contains("Electron"));
        assert!(id.binaries.contains("electron"));
    }
}
