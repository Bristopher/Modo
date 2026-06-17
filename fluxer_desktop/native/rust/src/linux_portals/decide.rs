// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalEntry {
    pub id: String,
    pub description: String,
    pub preferred_trigger: String,
}

impl PortalEntry {
    pub fn new(id: &str, description: &str) -> Self {
        Self {
            id: id.to_owned(),
            description: description.to_owned(),
            preferred_trigger: String::new(),
        }
    }

    pub fn with_trigger(id: &str, description: &str, preferred_trigger: &str) -> Self {
        Self {
            id: id.to_owned(),
            description: description.to_owned(),
            preferred_trigger: preferred_trigger.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortalBoundShortcut {
    pub id: String,
    pub description: String,
    pub trigger_description: String,
}

impl PortalBoundShortcut {
    pub fn new(id: &str, description: &str, trigger_description: &str) -> Self {
        Self {
            id: id.to_owned(),
            description: description.to_owned(),
            trigger_description: trigger_description.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindReason {
    NoPersistedShortcuts,
    NewIdsAdded,
}

impl BindReason {
    pub fn name(self) -> &'static str {
        match self {
            Self::NoPersistedShortcuts => "no-persisted-shortcuts",
            Self::NewIdsAdded => "new-ids-added",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigureAction {
    Reuse(Vec<PortalBoundShortcut>),
    Bind(BindReason),
}

pub fn decide(requested: &[PortalEntry], persisted: &[PortalBoundShortcut]) -> ConfigureAction {
    if persisted.is_empty() {
        return ConfigureAction::Bind(BindReason::NoPersistedShortcuts);
    }
    if requested
        .iter()
        .any(|entry| !has_persisted_shortcut(&entry.id, persisted))
    {
        return ConfigureAction::Bind(BindReason::NewIdsAdded);
    }
    ConfigureAction::Reuse(persisted.to_vec())
}

pub fn has_persisted_shortcut(id: &str, persisted: &[PortalBoundShortcut]) -> bool {
    persisted.iter().any(|shortcut| shortcut.id == id)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathError {
    NoSpaceLeft,
}

pub fn request_path(
    unique_bus_name: &str,
    handle_token: &str,
    capacity: usize,
) -> Result<String, PathError> {
    let trimmed = unique_bus_name.strip_prefix(':').unwrap_or(unique_bus_name);
    let normalized = trimmed.replace('.', "_");
    let path = format!("/org/freedesktop/portal/desktop/request/{normalized}/{handle_token}");
    if path.len() > capacity {
        Err(PathError::NoSpaceLeft)
    } else {
        Ok(path)
    }
}

pub fn stable_key(entries: &[PortalEntry]) -> String {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = String::from("[");
    for (index, entry) in sorted.iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push_str("{id=");
        out.push_str(&entry.id);
        out.push_str(",desc=");
        out.push_str(&entry.description);
        out.push_str(",trig=");
        out.push_str(&entry.preferred_trigger);
        out.push('}');
    }
    out.push(']');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decide_empty_persisted_set_requests_first_time_bind() {
        let action = decide(&[PortalEntry::new("mute", "Toggle mute")], &[]);
        assert_eq!(
            ConfigureAction::Bind(BindReason::NoPersistedShortcuts),
            action
        );
    }

    #[test]
    fn decide_matching_ids_on_restart_reuses_persisted_bindings() {
        let persisted = [
            PortalBoundShortcut::new("mute", "Toggle mute", "Ctrl+Shift+M"),
            PortalBoundShortcut::new("deafen", "Toggle deafen", "Ctrl+Shift+D"),
        ];
        let requested = [
            PortalEntry::new("mute", "Toggle mute"),
            PortalEntry::new("deafen", "Toggle deafen"),
        ];
        let action = decide(&requested, &persisted);
        assert!(matches!(action, ConfigureAction::Reuse(shortcuts) if shortcuts.len() == 2));
    }

    #[test]
    fn decide_new_id_added_since_last_launch_triggers_fresh_bind() {
        let persisted = [PortalBoundShortcut::new(
            "mute",
            "Toggle mute",
            "Ctrl+Shift+M",
        )];
        let requested = [
            PortalEntry::new("mute", "Toggle mute"),
            PortalEntry::new("push_to_talk", "Push to talk"),
        ];
        assert_eq!(
            ConfigureAction::Bind(BindReason::NewIdsAdded),
            decide(&requested, &persisted)
        );
    }

    #[test]
    fn decide_removing_previously_persisted_id_does_not_rebind() {
        let persisted = [
            PortalBoundShortcut::new("mute", "Toggle mute", "Ctrl+Shift+M"),
            PortalBoundShortcut::new("deafen", "Toggle deafen", "Ctrl+Shift+D"),
        ];
        let requested = [PortalEntry::new("mute", "Toggle mute")];
        assert!(matches!(
            decide(&requested, &persisted),
            ConfigureAction::Reuse(_)
        ));
    }

    #[test]
    fn decide_empty_requested_with_empty_persisted_reports_no_persisted() {
        assert_eq!(
            ConfigureAction::Bind(BindReason::NoPersistedShortcuts),
            decide(&[], &[])
        );
    }

    #[test]
    fn request_path_well_known_shape_per_portal_spec() {
        assert_eq!(
            "/org/freedesktop/portal/desktop/request/1_42/fluxer_gs_create_xyz",
            request_path(":1.42", "fluxer_gs_create_xyz", 256).unwrap()
        );
    }

    #[test]
    fn request_path_handles_unique_names_without_leading_colon() {
        assert_eq!(
            "/org/freedesktop/portal/desktop/request/1_0_7/tok",
            request_path("1.0.7", "tok", 256).unwrap()
        );
    }

    #[test]
    fn request_path_rejects_too_small_buffer() {
        assert_eq!(
            Err(PathError::NoSpaceLeft),
            request_path(":1.42", "tok", 16)
        );
    }

    #[test]
    fn stable_key_sorts_by_id_and_includes_description_and_trigger() {
        let key = stable_key(&[
            PortalEntry::new("mute", "Toggle mute"),
            PortalEntry::with_trigger("deafen", "Toggle deafen", "Ctrl+D"),
        ]);
        assert_eq!(
            "[{id=deafen,desc=Toggle deafen,trig=Ctrl+D},{id=mute,desc=Toggle mute,trig=}]",
            key
        );
    }

    #[test]
    fn bind_reason_name_matches_js_side_strings() {
        assert_eq!(
            "no-persisted-shortcuts",
            BindReason::NoPersistedShortcuts.name()
        );
        assert_eq!("new-ids-added", BindReason::NewIdsAdded.name());
    }
}
