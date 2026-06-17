// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::decide::{PortalBoundShortcut, PortalEntry, has_persisted_shortcut, stable_key};

static TOKEN_SEQ: AtomicU64 = AtomicU64::new(1);

pub fn mint_token(prefix: &str) -> String {
    let seq = TOKEN_SEQ.fetch_add(1, Ordering::Relaxed);
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    format!("{prefix}_{ms:x}_{seq:x}")
}

pub fn compute_stable_key(entries: &[PortalEntry]) -> String {
    stable_key(entries)
}

pub fn merge_shortcut_snapshots(
    persisted: &[PortalBoundShortcut],
    bound: &[PortalBoundShortcut],
) -> Vec<PortalBoundShortcut> {
    let mut out = persisted.to_vec();
    for shortcut in bound {
        if !has_persisted_shortcut(&shortcut.id, &out) {
            out.push(shortcut.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linux_portals::decide::{PortalBoundShortcut, PortalEntry};

    #[test]
    fn compute_stable_key_is_deterministic_across_orderings() {
        let a = compute_stable_key(&[
            PortalEntry::new("mute", "Toggle mute"),
            PortalEntry::new("deafen", "Toggle deafen"),
        ]);
        let b = compute_stable_key(&[
            PortalEntry::new("deafen", "Toggle deafen"),
            PortalEntry::new("mute", "Toggle mute"),
        ]);
        assert_eq!(a, b);
    }

    #[test]
    fn mint_token_is_well_formed_and_monotonically_distinct_within_process() {
        let a = mint_token("fluxer_gs_create");
        let b = mint_token("fluxer_gs_create");
        assert_ne!(a, b);
        assert!(a.starts_with("fluxer_gs_create_"));
    }

    #[test]
    fn merge_shortcut_snapshots_preserves_existing_and_adds_newly_bound_ids() {
        let persisted = [PortalBoundShortcut::new(
            "mute",
            "Toggle mute",
            "Ctrl+Shift+M",
        )];
        let bound = [PortalBoundShortcut::new(
            "push_to_talk",
            "Push to talk",
            "Ctrl+Shift+Space",
        )];
        let merged = merge_shortcut_snapshots(&persisted, &bound);
        assert_eq!(2, merged.len());
        assert_eq!("mute", merged[0].id);
        assert_eq!("push_to_talk", merged[1].id);
    }
}
