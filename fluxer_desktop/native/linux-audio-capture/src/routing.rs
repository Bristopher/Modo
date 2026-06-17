#![allow(dead_code)]

// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::{HashMap, HashSet};

pub type PropMap = HashMap<String, String>;
pub type PropPattern = PropMap;

pub const MEDIA_CLASS_PLAYBACK_STREAM: &str = "Stream/Output/Audio";
const TARGET_OBJECTS_PATTERN_KEY: &str = "fluxer.target.objects";
const DISPLAY_PATTERN_PREFIX: &str = "fluxer.display.";

#[derive(Debug, Default, Clone)]
pub struct SelfIdentity {
    pub pids: HashSet<String>,
    pub binaries: HashSet<String>,
    pub display_names: HashSet<String>,
    pub display_prefixes: Vec<String>,
}

impl SelfIdentity {
    pub fn add_pid(&mut self, pid: impl Into<String>) {
        let value = pid.into();
        if value.is_empty() {
            return;
        }
        self.pids.insert(value);
    }

    pub fn add_binary(&mut self, name: impl Into<String>) {
        let value = name.into();
        if value.is_empty() {
            return;
        }
        self.binaries.insert(value);
    }

    pub fn add_display_name(&mut self, name: impl Into<String>) {
        let value = name.into();
        if value.is_empty() {
            return;
        }
        self.display_names.insert(value);
    }

    pub fn add_display_prefix(&mut self, prefix: impl Into<String>) {
        let value = prefix.into();
        if value.is_empty() {
            return;
        }
        self.display_prefixes.push(value);
    }

    pub fn matches(&self, properties: &PropMap) -> bool {
        if let Some(raw) = properties.get("application.process.id")
            && self.pids.contains(raw)
        {
            return true;
        }
        if let Some(raw) = properties.get("pipewire.sec.pid")
            && self.pids.contains(raw)
        {
            return true;
        }
        if let Some(raw) = properties.get("application.process.binary")
            && contains_case_insensitive(&self.binaries, raw)
        {
            return true;
        }
        for key in [
            "application.name",
            "node.name",
            "node.nick",
            "node.description",
        ] {
            if let Some(raw) = properties.get(key)
                && self.matches_display_identity(raw)
            {
                return true;
            }
        }
        false
    }

    fn matches_display_identity(&self, raw: &str) -> bool {
        contains_case_insensitive(&self.binaries, raw)
            || contains_case_insensitive(&self.display_names, raw)
            || self
                .display_prefixes
                .iter()
                .any(|prefix| starts_with_case_insensitive(raw, prefix))
    }
}

fn contains_case_insensitive(values: &HashSet<String>, needle: &str) -> bool {
    values
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(needle))
}

fn starts_with_case_insensitive(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

#[derive(Debug, Default, Clone)]
pub struct RoutingRule {
    pub include_when: Vec<PropPattern>,
    pub never_when: Vec<PropPattern>,
    pub pin_target_for: Vec<PropPattern>,
    pub skip_hardware_devices: bool,
    pub only_audio_sinks: bool,
    pub only_default_audio_sink: bool,
}

pub fn matches_pattern(candidate: &PropMap, expected: &PropPattern) -> bool {
    for (key, value) in expected {
        if key.starts_with(DISPLAY_PATTERN_PREFIX) {
            continue;
        }
        if key == TARGET_OBJECTS_PATTERN_KEY {
            if !matches_target_object(candidate, value) {
                return false;
            }
            continue;
        }
        match candidate.get(key) {
            Some(actual) if actual == value => {}
            _ => return false,
        }
    }
    true
}

fn matches_target_object(candidate: &PropMap, expected_values: &str) -> bool {
    let Some(actual) = candidate
        .get("target.object")
        .or_else(|| candidate.get("node.target"))
    else {
        return false;
    };
    expected_values
        .split('\n')
        .filter(|value| !value.is_empty())
        .any(|expected| actual == expected)
}

pub fn matches_any(candidate: &PropMap, patterns: &[PropPattern]) -> bool {
    patterns.iter().any(|p| matches_pattern(candidate, p))
}

pub fn should_route_node(
    id: u32,
    properties: &PropMap,
    rule: &RoutingRule,
    default_sink_name: &str,
    default_sink_target_id: &str,
    sink_global_id: u32,
    self_identity: &SelfIdentity,
) -> bool {
    if id == sink_global_id {
        return false;
    }

    if self_identity.matches(properties) {
        return false;
    }
    if matches_any(properties, &rule.never_when) {
        return false;
    }
    if rule.skip_hardware_devices && properties.contains_key("device.id") {
        return false;
    }

    let Some(class) = properties.get("media.class") else {
        return false;
    };
    if class != MEDIA_CLASS_PLAYBACK_STREAM {
        return false;
    }

    if !rule.include_when.is_empty() {
        return matches_any(properties, &rule.include_when);
    }

    if rule.only_audio_sinks {
        return !rule.only_default_audio_sink
            || targets_default_sink(properties, default_sink_name, default_sink_target_id);
    }

    false
}

pub fn targets_default_sink(
    properties: &PropMap,
    default_sink_name: &str,
    default_sink_target_id: &str,
) -> bool {
    if default_sink_name.is_empty() && default_sink_target_id.is_empty() {
        return true;
    }
    let Some(target) = properties
        .get("target.object")
        .or_else(|| properties.get("node.target"))
    else {
        return true;
    };
    (!default_sink_name.is_empty() && target == default_sink_name)
        || (!default_sink_target_id.is_empty() && target == default_sink_target_id)
}

pub fn parse_default_sink_name(blob: &str) -> String {
    let trimmed = blob.trim();
    if !trimmed.starts_with('{') {
        return String::new();
    }
    let bytes = trimmed.as_bytes();
    let mut i = 1usize;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'"' {
            return String::new();
        }

        i += 1;
        let key_start = i;
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'\\' {
                i += 2;
            } else {
                i += 1;
            }
        }
        if i >= bytes.len() {
            return String::new();
        }
        let key = &trimmed[key_start..i];
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b':' {
            return String::new();
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }

        if i >= bytes.len() {
            return String::new();
        }
        if bytes[i] != b'"' {
            let mut depth = 0usize;
            while i < bytes.len() {
                match bytes[i] {
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => {
                        if depth == 0 {
                            return String::new();
                        }
                        depth -= 1;
                    }
                    b',' if depth == 0 => break,
                    _ => {}
                }
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b',' {
                i += 1;
                continue;
            }
            return String::new();
        }
        i += 1;
        let val_start = i;
        let mut buf = String::new();
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                let escaped = bytes[i + 1];
                buf.push(escaped as char);
                i += 2;
            } else {
                buf.push(bytes[i] as char);
                i += 1;
            }
        }
        if i >= bytes.len() {
            return String::new();
        }
        if key == "name" {
            if buf.len() == i - val_start {
                return trimmed[val_start..i].to_string();
            }
            return buf;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b',' {
            i += 1;
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_map(entries: &[(&str, &str)]) -> PropMap {
        entries
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    fn system_rule() -> RoutingRule {
        RoutingRule {
            skip_hardware_devices: true,
            only_audio_sinks: true,
            only_default_audio_sink: true,
            ..Default::default()
        }
    }

    #[test]
    fn empty_pattern_matches_any_candidate() {
        let candidate = make_map(&[("application.name", "Example")]);
        let empty: PropPattern = PropPattern::new();
        assert!(matches_pattern(&candidate, &empty));
    }

    #[test]
    fn missing_keys_and_mismatched_values_do_not_match() {
        let candidate = make_map(&[("application.name", "Example")]);
        let missing = make_map(&[("application.process.id", "1234")]);
        let mismatched = make_map(&[("application.name", "Other")]);
        assert!(!matches_pattern(&candidate, &missing));
        assert!(!matches_pattern(&candidate, &mismatched));
    }

    #[test]
    fn synthetic_target_object_pattern_matches_name_or_serial() {
        let candidate = make_map(&[("target.object", "42")]);
        let deprecated = make_map(&[("node.target", "alsa_output.foo")]);
        let pattern = make_map(&[(TARGET_OBJECTS_PATTERN_KEY, "alsa_output.foo\n42")]);
        let mismatch = make_map(&[(TARGET_OBJECTS_PATTERN_KEY, "alsa_output.foo\n99")]);
        assert!(matches_pattern(&candidate, &pattern));
        assert!(matches_pattern(&deprecated, &pattern));
        assert!(!matches_pattern(&candidate, &mismatch));
    }

    #[test]
    fn synthetic_display_pattern_keys_do_not_affect_routing() {
        let candidate = make_map(&[("application.name", "Example")]);
        let pattern = make_map(&[
            ("application.name", "Example"),
            ("fluxer.display.name", "Living room speakers"),
        ]);
        assert!(matches_pattern(&candidate, &pattern));
    }

    #[test]
    fn matches_any_requires_at_least_one_matching_pattern() {
        let candidate = make_map(&[("application.name", "Example")]);
        let patterns = vec![
            make_map(&[("application.name", "Other")]),
            make_map(&[("application.name", "Example")]),
        ];
        assert!(matches_any(&candidate, &patterns));
        assert!(!matches_any(&candidate, &[]));
    }

    #[test]
    fn system_mode_routes_only_default_playback_streams() {
        let identity = SelfIdentity::default();
        let analog = "alsa_output.pci-0000_00_1f.3.analog-stereo";
        let hdmi = "alsa_output.pci-0000_01_00.1.hdmi-stereo";
        let stream = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", analog),
        ]);
        let other = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", hdmi),
        ]);
        let rule = system_rule();
        assert!(should_route_node(
            100, &stream, &rule, analog, "", 1, &identity,
        ));
        assert!(!should_route_node(
            101, &other, &rule, analog, "", 1, &identity,
        ));
    }

    #[test]
    fn structural_self_identity_wins_over_include_rules() {
        let mut identity = SelfIdentity::default();
        identity.add_pid("4242");
        identity.add_binary("fluxer");
        identity.add_display_name("Fluxer Canary");
        identity.add_display_prefix("Fluxer ");

        let rule = RoutingRule {
            include_when: vec![make_map(&[("application.process.id", "4242")])],
            ..Default::default()
        };
        let by_pid = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("application.process.id", "4242"),
        ]);
        let by_binary = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("application.process.binary", "fluxer"),
        ]);
        let by_description = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("node.description", "Fluxer Direct Capture (pid 4242)"),
        ]);

        assert!(!should_route_node(
            200, &by_pid, &rule, "", "", 0, &identity
        ));
        assert!(!should_route_node(
            201, &by_binary, &rule, "", "", 0, &identity,
        ));
        assert!(!should_route_node(
            202,
            &by_description,
            &rule,
            "",
            "",
            0,
            &identity,
        ));
    }

    #[test]
    fn routing_refuses_non_playback_media_classes_even_when_included() {
        let identity = SelfIdentity::default();
        let rule = RoutingRule {
            include_when: vec![make_map(&[("application.name", "Recorder")])],
            ..Default::default()
        };
        let input_stream = make_map(&[
            ("media.class", "Stream/Input/Audio"),
            ("application.name", "Recorder"),
        ]);
        let device = make_map(&[
            ("media.class", "Audio/Source"),
            ("application.name", "Recorder"),
        ]);
        assert!(!should_route_node(
            300,
            &input_stream,
            &rule,
            "",
            "",
            0,
            &identity,
        ));
        assert!(!should_route_node(
            301, &device, &rule, "", "", 0, &identity
        ));
    }

    #[test]
    fn system_mode_accepts_untargeted_and_node_target_streams() {
        let identity = SelfIdentity::default();
        let analog = "alsa_output.pci-0000_00_1f.3.analog-stereo";
        let untargeted = make_map(&[("media.class", MEDIA_CLASS_PLAYBACK_STREAM)]);
        let deprecated = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("node.target", analog),
        ]);
        let rule = system_rule();
        assert!(should_route_node(
            100,
            &untargeted,
            &rule,
            analog,
            "",
            1,
            &identity,
        ));
        assert!(should_route_node(
            101,
            &deprecated,
            &rule,
            analog,
            "",
            1,
            &identity,
        ));
    }

    #[test]
    fn system_mode_accepts_default_sink_object_id_targets() {
        let identity = SelfIdentity::default();
        let analog = "alsa_output.pci-0000_00_1f.3.analog-stereo";
        let by_name = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", analog),
        ]);
        let by_id = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", "42"),
        ]);
        let other = make_map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", "99"),
        ]);
        let rule = system_rule();
        assert!(should_route_node(
            100, &by_name, &rule, analog, "42", 1, &identity,
        ));
        assert!(should_route_node(
            101, &by_id, &rule, analog, "42", 1, &identity,
        ));
        assert!(!should_route_node(
            102, &other, &rule, analog, "42", 1, &identity,
        ));
    }

    #[test]
    fn include_mode_honors_hardware_filtering_and_never_rules() {
        let identity = SelfIdentity::default();
        let rule = RoutingRule {
            include_when: vec![make_map(&[("application.name", "Firefox")])],
            never_when: vec![make_map(&[("application.process.id", "999")])],
            skip_hardware_devices: true,
            ..Default::default()
        };

        let app = make_map(&[
            ("application.name", "Firefox"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let hardware = make_map(&[("application.name", "Firefox"), ("device.id", "5")]);
        let blocked = make_map(&[
            ("application.name", "Firefox"),
            ("application.process.id", "999"),
        ]);
        assert!(should_route_node(10, &app, &rule, "", "", 1, &identity));
        assert!(!should_route_node(
            11, &hardware, &rule, "", "", 1, &identity
        ));
        assert!(!should_route_node(
            12, &blocked, &rule, "", "", 1, &identity
        ));
    }

    #[test]
    fn include_mode_rejects_non_playback_nodes_that_match_the_include_filter() {
        let identity = SelfIdentity::default();
        let rule = RoutingRule {
            include_when: vec![make_map(&[("application.name", "Chromium")])],
            ..Default::default()
        };
        let mic = make_map(&[
            ("application.name", "Chromium"),
            ("media.class", "Audio/Source"),
        ]);
        let sink = make_map(&[
            ("application.name", "Chromium"),
            ("media.class", "Audio/Sink"),
        ]);
        let input = make_map(&[
            ("application.name", "Chromium"),
            ("media.class", "Stream/Input/Audio"),
        ]);
        let virtual_source = make_map(&[
            ("application.name", "Chromium"),
            ("media.class", "Audio/Source/Virtual"),
        ]);
        let playback = make_map(&[
            ("application.name", "Chromium"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let unclassified = make_map(&[("application.name", "Chromium")]);
        assert!(!should_route_node(20, &mic, &rule, "", "", 1, &identity));
        assert!(!should_route_node(21, &sink, &rule, "", "", 1, &identity));
        assert!(!should_route_node(22, &input, &rule, "", "", 1, &identity));
        assert!(!should_route_node(
            23,
            &virtual_source,
            &rule,
            "",
            "",
            1,
            &identity,
        ));
        assert!(!should_route_node(
            24,
            &unclassified,
            &rule,
            "",
            "",
            1,
            &identity,
        ));
        assert!(should_route_node(
            25, &playback, &rule, "", "", 1, &identity
        ));
    }

    #[test]
    fn empty_rules_route_nothing_and_sink_id_is_excluded() {
        let identity = SelfIdentity::default();
        let app = make_map(&[
            ("application.name", "Foo"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let empty = RoutingRule::default();
        assert!(!should_route_node(1, &app, &empty, "", "", 0, &identity));
        let rule = system_rule();
        assert!(!should_route_node(7, &app, &rule, "", "", 7, &identity));
    }

    #[test]
    fn parse_default_sink_name_is_strict_and_tolerant() {
        assert_eq!(
            "alsa_output.foo",
            parse_default_sink_name(r#"{"name":"alsa_output.foo","other":"bar"}"#),
        );
        assert_eq!("", parse_default_sink_name("not-json"));
        assert_eq!("", parse_default_sink_name(r#"{"name":42}"#));
    }

    #[test]
    fn self_identity_matches_across_documented_pipewire_keys() {
        let mut identity = SelfIdentity::default();
        identity.add_pid("1234");
        identity.add_binary("fluxer");
        identity.add_display_name("Fluxer Canary");
        identity.add_display_prefix("Fluxer ");

        let by_pid = make_map(&[("application.process.id", "1234")]);
        let by_sec_pid = make_map(&[("pipewire.sec.pid", "1234")]);
        let by_binary = make_map(&[("application.process.binary", "fluxer")]);
        let by_app_name = make_map(&[("application.name", "fluxer")]);
        let by_node_name = make_map(&[("node.name", "fluxer")]);
        let by_node_nick = make_map(&[("node.nick", "Fluxer Canary")]);
        let by_node_description = make_map(&[("node.description", "Fluxer app audio capture")]);
        let stranger = make_map(&[("application.process.id", "9999")]);

        assert!(identity.matches(&by_pid));
        assert!(identity.matches(&by_sec_pid));
        assert!(identity.matches(&by_binary));
        assert!(identity.matches(&by_app_name));
        assert!(identity.matches(&by_node_name));
        assert!(identity.matches(&by_node_nick));
        assert!(identity.matches(&by_node_description));
        assert!(!identity.matches(&stranger));
    }
}
