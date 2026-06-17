// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::{BTreeMap, HashSet};

pub type PropMap = BTreeMap<String, String>;
pub type PropPattern = PropMap;

pub const MEDIA_CLASS_PLAYBACK_STREAM: &str = "Stream/Output/Audio";

#[derive(Debug, Default, Clone)]
pub struct SelfIdentity {
    pub pids: HashSet<String>,
    pub binaries: HashSet<String>,
    pub display_names: HashSet<String>,
    pub display_prefixes: Vec<String>,
}

impl SelfIdentity {
    pub fn add_pid(&mut self, pid: impl Into<String>) {
        let pid = pid.into();
        if !pid.is_empty() {
            self.pids.insert(pid);
        }
    }

    pub fn add_binary(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !name.is_empty() {
            self.binaries.insert(name);
        }
    }

    pub fn add_display_name(&mut self, name: impl Into<String>) {
        let name = name.into();
        if !name.is_empty() {
            self.display_names.insert(name);
        }
    }

    pub fn add_display_prefix(&mut self, prefix: impl Into<String>) {
        let prefix = prefix.into();
        if !prefix.is_empty() {
            self.display_prefixes.push(prefix);
        }
    }

    pub fn matches(&self, properties: &PropMap) -> bool {
        properties
            .get("application.process.id")
            .is_some_and(|raw| self.pids.contains(raw))
            || properties
                .get("pipewire.sec.pid")
                .is_some_and(|raw| self.pids.contains(raw))
            || properties
                .get("application.process.binary")
                .is_some_and(|raw| contains_case_insensitive(&self.binaries, raw))
            || [
                "application.name",
                "node.name",
                "node.nick",
                "node.description",
            ]
            .iter()
            .any(|key| {
                properties
                    .get(*key)
                    .is_some_and(|raw| self.matches_display_identity(raw))
            })
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

pub fn matches_pattern(candidate: &PropMap, expected_pattern: &PropPattern) -> bool {
    expected_pattern
        .iter()
        .all(|(key, expected)| candidate.get(key).is_some_and(|actual| actual == expected))
}

pub fn matches_any(candidate: &PropMap, patterns: &[PropPattern]) -> bool {
    patterns.iter().any(|item| matches_pattern(candidate, item))
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
    if properties.get("media.class").map(String::as_str) != Some(MEDIA_CLASS_PLAYBACK_STREAM) {
        return false;
    }
    if !rule.include_when.is_empty() {
        return matches_any(properties, &rule.include_when);
    }
    if rule.only_audio_sinks {
        if rule.only_default_audio_sink
            && !targets_default_sink(properties, default_sink_name, default_sink_target_id)
        {
            return false;
        }
        return true;
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
    let mut idx = 0;
    while let Some(name_pos) = blob[idx..].find("\"name\"") {
        idx += name_pos + "\"name\"".len();
        let rest = blob[idx..].trim_start();
        let Some(after_colon) = rest.strip_prefix(':') else {
            continue;
        };
        let value = after_colon.trim_start();
        let Some(mut value) = value.strip_prefix('"') else {
            return String::new();
        };
        let mut out = String::new();
        while let Some(ch) = value.chars().next() {
            value = &value[ch.len_utf8()..];
            match ch {
                '"' => return out,
                '\\' => {
                    if let Some(escaped) = value.chars().next() {
                        value = &value[escaped.len_utf8()..];
                        out.push(escaped);
                    }
                }
                _ => out.push(ch),
            }
        }
        return String::new();
    }
    String::new()
}

pub fn map(entries: &[(&str, &str)]) -> PropMap {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

pub fn system_rule() -> RoutingRule {
    RoutingRule {
        skip_hardware_devices: true,
        only_audio_sinks: true,
        only_default_audio_sink: true,
        ..RoutingRule::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pattern_matches_any_candidate() {
        let candidate = map(&[("application.name", "Example")]);
        let empty = PropPattern::new();
        assert!(matches_pattern(&candidate, &empty));
    }

    #[test]
    fn missing_keys_and_mismatched_values_do_not_match() {
        let candidate = map(&[("application.name", "Example")]);
        let missing = map(&[("application.process.id", "1234")]);
        let mismatched = map(&[("application.name", "Other")]);
        assert!(!matches_pattern(&candidate, &missing));
        assert!(!matches_pattern(&candidate, &mismatched));
    }

    #[test]
    fn matches_any_requires_at_least_one_matching_pattern() {
        let candidate = map(&[("application.name", "Example")]);
        let patterns = vec![
            map(&[("application.name", "Other")]),
            map(&[("application.name", "Example")]),
        ];
        assert!(matches_any(&candidate, &patterns));
        assert!(!matches_any(&candidate, &[]));
    }

    #[test]
    fn system_mode_routes_only_default_playback_streams() {
        let self_identity = SelfIdentity::default();
        let analog = "alsa_output.pci-0000_00_1f.3.analog-stereo";
        let hdmi = "alsa_output.pci-0000_01_00.1.hdmi-stereo";
        let stream = map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", analog),
        ]);
        let other = map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", hdmi),
        ]);
        let rule = system_rule();
        assert!(should_route_node(
            100,
            &stream,
            &rule,
            analog,
            "",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            101,
            &other,
            &rule,
            analog,
            "",
            1,
            &self_identity
        ));
    }

    #[test]
    fn structural_self_identity_wins_over_include_rules() {
        let mut self_identity = SelfIdentity::default();
        self_identity.add_pid("4242");
        self_identity.add_binary("fluxer");
        let rule = RoutingRule {
            include_when: vec![map(&[("application.process.id", "4242")])],
            ..RoutingRule::default()
        };
        let by_pid = map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("application.process.id", "4242"),
        ]);
        let by_binary = map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("application.process.binary", "fluxer"),
        ]);
        assert!(!should_route_node(
            200,
            &by_pid,
            &rule,
            "",
            "",
            0,
            &self_identity
        ));
        assert!(!should_route_node(
            201,
            &by_binary,
            &rule,
            "",
            "",
            0,
            &self_identity
        ));
    }

    #[test]
    fn routing_refuses_non_playback_media_classes_even_when_included() {
        let self_identity = SelfIdentity::default();
        let rule = RoutingRule {
            include_when: vec![map(&[("application.name", "Recorder")])],
            ..RoutingRule::default()
        };
        let input_stream = map(&[
            ("media.class", "Stream/Input/Audio"),
            ("application.name", "Recorder"),
        ]);
        let device = map(&[
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
            &self_identity
        ));
        assert!(!should_route_node(
            301,
            &device,
            &rule,
            "",
            "",
            0,
            &self_identity
        ));
    }

    #[test]
    fn system_mode_accepts_untargeted_and_node_target_streams() {
        let self_identity = SelfIdentity::default();
        let analog = "alsa_output.pci-0000_00_1f.3.analog-stereo";
        let untargeted = map(&[("media.class", MEDIA_CLASS_PLAYBACK_STREAM)]);
        let deprecated = map(&[
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
            &self_identity
        ));
        assert!(should_route_node(
            101,
            &deprecated,
            &rule,
            analog,
            "",
            1,
            &self_identity
        ));
    }

    #[test]
    fn system_mode_accepts_default_sink_object_id_targets() {
        let self_identity = SelfIdentity::default();
        let analog = "alsa_output.pci-0000_00_1f.3.analog-stereo";
        let by_name = map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", analog),
        ]);
        let by_id = map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", "42"),
        ]);
        let other = map(&[
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
            ("target.object", "99"),
        ]);
        let rule = system_rule();
        assert!(should_route_node(
            100,
            &by_name,
            &rule,
            analog,
            "42",
            1,
            &self_identity
        ));
        assert!(should_route_node(
            101,
            &by_id,
            &rule,
            analog,
            "42",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            102,
            &other,
            &rule,
            analog,
            "42",
            1,
            &self_identity
        ));
    }

    #[test]
    fn include_mode_honors_hardware_filtering_and_never_rules() {
        let self_identity = SelfIdentity::default();
        let rule = RoutingRule {
            include_when: vec![map(&[("application.name", "Firefox")])],
            never_when: vec![map(&[("application.process.id", "999")])],
            skip_hardware_devices: true,
            ..RoutingRule::default()
        };
        let app = map(&[
            ("application.name", "Firefox"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let hardware = map(&[("application.name", "Firefox"), ("device.id", "5")]);
        let blocked = map(&[
            ("application.name", "Firefox"),
            ("application.process.id", "999"),
        ]);
        assert!(should_route_node(
            10,
            &app,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            11,
            &hardware,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            12,
            &blocked,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
    }

    #[test]
    fn include_mode_rejects_non_playback_nodes_that_match_include_filter() {
        let self_identity = SelfIdentity::default();
        let rule = RoutingRule {
            include_when: vec![map(&[("application.name", "Chromium")])],
            ..RoutingRule::default()
        };
        let rejected = [
            map(&[
                ("application.name", "Chromium"),
                ("media.class", "Audio/Source"),
            ]),
            map(&[
                ("application.name", "Chromium"),
                ("media.class", "Audio/Sink"),
            ]),
            map(&[
                ("application.name", "Chromium"),
                ("media.class", "Stream/Input/Audio"),
            ]),
            map(&[
                ("application.name", "Chromium"),
                ("media.class", "Audio/Source/Virtual"),
            ]),
            map(&[("application.name", "Chromium")]),
        ];
        for (idx, props) in rejected.iter().enumerate() {
            assert!(!should_route_node(
                20 + idx as u32,
                props,
                &rule,
                "",
                "",
                1,
                &self_identity
            ));
        }
        let playback = map(&[
            ("application.name", "Chromium"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        assert!(should_route_node(
            25,
            &playback,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
    }

    #[test]
    fn empty_rules_route_nothing_and_sink_id_is_excluded() {
        let self_identity = SelfIdentity::default();
        let app = map(&[
            ("application.name", "Foo"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        assert!(!should_route_node(
            1,
            &app,
            &RoutingRule::default(),
            "",
            "",
            0,
            &self_identity
        ));
        assert!(!should_route_node(
            7,
            &app,
            &system_rule(),
            "",
            "",
            7,
            &self_identity
        ));
    }

    #[test]
    fn parse_default_sink_name_is_strict_and_tolerant() {
        assert_eq!(
            "alsa_output.foo",
            parse_default_sink_name("{\"name\":\"alsa_output.foo\",\"other\":\"bar\"}")
        );
        assert_eq!("", parse_default_sink_name("not-json"));
        assert_eq!("", parse_default_sink_name("{\"name\":42}"));
    }

    #[test]
    fn structural_self_exclude_beats_include_rules_across_pid_binary_name_keys() {
        let mut self_identity = SelfIdentity::default();
        let pid = std::process::id().to_string();
        self_identity.add_pid(&pid);
        self_identity.add_binary("fluxer");
        self_identity.add_binary("fluxer.exe");
        self_identity.add_display_name("Fluxer Canary");
        self_identity.add_display_prefix("Fluxer ");

        let rule = RoutingRule {
            include_when: vec![map(&[("application.process.binary", "firefox")])],
            ..RoutingRule::default()
        };
        let by_pid = map(&[
            ("application.process.id", &pid),
            ("application.name", "fluxer"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let by_binary = map(&[
            ("application.process.id", "999999"),
            ("application.process.binary", "fluxer"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let by_app_name = map(&[
            ("application.name", "fluxer"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let by_node_name = map(&[
            ("node.name", "fluxer"),
            ("application.name", "Other"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let by_node_description = map(&[
            ("node.description", "Fluxer Direct Capture (pid 4242)"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        let unrelated = map(&[
            ("application.name", "Firefox"),
            ("application.process.id", "424242"),
            ("application.process.binary", "firefox"),
            ("media.class", MEDIA_CLASS_PLAYBACK_STREAM),
        ]);
        assert!(!should_route_node(
            101,
            &by_pid,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            102,
            &by_binary,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            103,
            &by_app_name,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            104,
            &by_node_name,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            107,
            &by_node_description,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
        assert!(should_route_node(
            105,
            &unrelated,
            &rule,
            "",
            "",
            1,
            &self_identity
        ));
        assert!(!should_route_node(
            106,
            &by_pid,
            &system_rule(),
            "",
            "",
            1,
            &self_identity
        ));
    }

    #[test]
    fn self_identity_matches_across_the_four_documented_pipewire_keys() {
        let mut self_identity = SelfIdentity::default();
        self_identity.add_pid("1234");
        self_identity.add_binary("fluxer");
        self_identity.add_display_name("Fluxer Canary");
        self_identity.add_display_prefix("Fluxer ");
        assert!(self_identity.matches(&map(&[("application.process.id", "1234")])));
        assert!(self_identity.matches(&map(&[("pipewire.sec.pid", "1234")])));
        assert!(self_identity.matches(&map(&[("application.process.binary", "fluxer")])));
        assert!(self_identity.matches(&map(&[("application.name", "fluxer")])));
        assert!(self_identity.matches(&map(&[("node.name", "fluxer")])));
        assert!(self_identity.matches(&map(&[("node.nick", "Fluxer Canary")])));
        assert!(self_identity.matches(&map(&[("node.description", "Fluxer app audio capture",)])));
        assert!(!self_identity.matches(&map(&[("application.process.id", "9999")])));
    }
}
