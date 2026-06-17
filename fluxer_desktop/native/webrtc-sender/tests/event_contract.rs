// SPDX-License-Identifier: AGPL-3.0-or-later

use serde_json::Value;

const OUTBOUND: &str = include_str!("fixtures/outbound_events.json");
const INBOUND: &str = include_str!("fixtures/inbound_events.json");
const VIDEO_FRAME_META: &str = include_str!("fixtures/video_frame_meta.json");

const TRACK_KINDS: &[&str] = &["audio", "video"];
const TRACK_SOURCES: &[&str] = &[
    "unknown",
    "camera",
    "microphone",
    "screen_share",
    "screen_share_audio",
];
const QUALITIES: &[&str] = &["excellent", "good", "poor", "lost"];
const SUBSCRIPTION_STATUSES: &[&str] = &["desired", "subscribed", "unsubscribed"];

fn records(json: &str) -> Vec<(String, Value)> {
    let parsed: Value = serde_json::from_str(json).expect("fixture is valid JSON");
    parsed
        .as_array()
        .expect("fixture is a JSON array")
        .iter()
        .map(|rec| {
            let ty = rec
                .get("eventType")
                .and_then(Value::as_str)
                .expect("record has an eventType string")
                .to_string();
            let payload = rec.get("payload").expect("record has a payload").clone();
            (ty, payload)
        })
        .collect()
}

fn assert_exact_keys(event_type: &str, payload: &Value, expected: &[&str]) {
    let obj = payload
        .as_object()
        .unwrap_or_else(|| panic!("{event_type}: payload is not a JSON object"));
    let mut got: Vec<&str> = obj.keys().map(String::as_str).collect();
    got.sort_unstable();
    let mut want: Vec<&str> = expected.to_vec();
    want.sort_unstable();
    assert_eq!(got, want, "{event_type}: payload keys mismatch");
}

fn str_field<'a>(event_type: &str, payload: &'a Value, key: &str) -> &'a str {
    payload
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("{event_type}.{key} is not a string"))
}

fn bool_field(event_type: &str, payload: &Value, key: &str) -> bool {
    payload
        .get(key)
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("{event_type}.{key} is not a boolean"))
}

fn validate_string_map(event_type: &str, payload: &Value, key: &str) {
    let obj = payload
        .get(key)
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("{event_type}.{key} is not an object"));
    for (map_key, value) in obj {
        assert!(
            !map_key.is_empty(),
            "{event_type}.{key} contains an empty key"
        );
        assert!(
            value.as_str().is_some(),
            "{event_type}.{key}.{map_key} is not a string"
        );
    }
}

fn validate_participant_snapshot(event_type: &str, payload: &Value) {
    assert_exact_keys(event_type, payload, &["sid", "identity", "name"]);
    assert!(str_field(event_type, payload, "sid").starts_with("PA_"));
    let _ = str_field(event_type, payload, "identity");
    let _ = str_field(event_type, payload, "name");
}

fn validate_track_payload(event_type: &str, payload: &Value, includes_subscription: bool) {
    let mut keys = vec![
        "participantSid",
        "identity",
        "participantName",
        "trackSid",
        "trackName",
        "kind",
        "source",
        "muted",
    ];
    if includes_subscription {
        keys.extend(["subscribed", "subscriptionStatus"]);
    }
    assert_exact_keys(event_type, payload, &keys);
    assert!(str_field(event_type, payload, "participantSid").starts_with("PA_"));
    let _ = str_field(event_type, payload, "identity");
    let _ = str_field(event_type, payload, "participantName");
    assert!(str_field(event_type, payload, "trackSid").starts_with("TR_"));
    let _ = str_field(event_type, payload, "trackName");
    assert!(TRACK_KINDS.contains(&str_field(event_type, payload, "kind")));
    assert!(TRACK_SOURCES.contains(&str_field(event_type, payload, "source")));
    let _ = bool_field(event_type, payload, "muted");
    if includes_subscription {
        let _ = bool_field(event_type, payload, "subscribed");
        assert!(SUBSCRIPTION_STATUSES.contains(&str_field(
            event_type,
            payload,
            "subscriptionStatus"
        )));
    }
}

fn validate_record(event_type: &str, payload: &Value) {
    match event_type {
        "connected" | "Reconnecting" | "Reconnected" => assert_exact_keys(event_type, payload, &[]),
        "connectionState" => {
            assert_exact_keys(event_type, payload, &["state"]);
            assert!(!str_field(event_type, payload, "state").is_empty());
        }
        "disconnected" => {
            assert_exact_keys(event_type, payload, &["reason"]);
            assert!(!str_field(event_type, payload, "reason").is_empty());
        }
        "participantJoined" => {
            assert_exact_keys(event_type, payload, &["sid", "identity", "name"]);
            assert!(str_field(event_type, payload, "sid").starts_with("PA_"));
            let _ = str_field(event_type, payload, "identity");
            let _ = str_field(event_type, payload, "name");
        }
        "participantLeft" => {
            assert_exact_keys(event_type, payload, &["sid", "identity", "name"]);
            assert!(str_field(event_type, payload, "sid").starts_with("PA_"));
            let _ = str_field(event_type, payload, "name");
        }
        "trackPublished" | "trackUnpublished" | "trackSubscribed" | "trackUnsubscribed" => {
            validate_track_payload(event_type, payload, true);
        }
        "trackMuted" | "trackUnmuted" => {
            validate_track_payload(event_type, payload, false);
            assert_eq!(
                bool_field(event_type, payload, "muted"),
                event_type == "trackMuted"
            );
        }
        "trackSubscriptionFailed" => {
            let has_publication = payload.get("kind").is_some();
            if has_publication {
                assert_exact_keys(
                    event_type,
                    payload,
                    &[
                        "participantSid",
                        "identity",
                        "participantName",
                        "trackSid",
                        "trackName",
                        "kind",
                        "source",
                        "muted",
                        "subscribed",
                        "subscriptionStatus",
                        "error",
                    ],
                );
            } else {
                assert_exact_keys(
                    event_type,
                    payload,
                    &[
                        "participantSid",
                        "identity",
                        "participantName",
                        "trackSid",
                        "error",
                    ],
                );
            }
            assert!(str_field(event_type, payload, "participantSid").starts_with("PA_"));
            assert!(str_field(event_type, payload, "trackSid").starts_with("TR_"));
            if has_publication {
                assert!(TRACK_KINDS.contains(&str_field(event_type, payload, "kind")));
                assert!(TRACK_SOURCES.contains(&str_field(event_type, payload, "source")));
                assert!(SUBSCRIPTION_STATUSES.contains(&str_field(
                    event_type,
                    payload,
                    "subscriptionStatus"
                )));
                let _ = bool_field(event_type, payload, "muted");
                let _ = bool_field(event_type, payload, "subscribed");
            }
            assert!(!str_field(event_type, payload, "error").is_empty());
        }
        "localTrackPublished" => {
            validate_track_payload(event_type, payload, false);
        }
        "localTrackUnpublished" => {
            validate_track_payload(event_type, payload, false);
        }
        "localTrackRepublished" => {
            let keys = [
                "participantSid",
                "identity",
                "participantName",
                "previousTrackSid",
                "trackSid",
                "trackName",
                "kind",
                "source",
                "muted",
            ];
            assert_exact_keys(event_type, payload, &keys);
            assert!(str_field(event_type, payload, "participantSid").starts_with("PA_"));
            assert!(str_field(event_type, payload, "previousTrackSid").starts_with("TR_"));
            assert!(str_field(event_type, payload, "trackSid").starts_with("TR_"));
            assert!(TRACK_KINDS.contains(&str_field(event_type, payload, "kind")));
            assert!(TRACK_SOURCES.contains(&str_field(event_type, payload, "source")));
            let _ = bool_field(event_type, payload, "muted");
        }
        "activeSpeakers" => {
            assert_exact_keys(event_type, payload, &["sids", "participants"]);
            let sids = payload
                .get("sids")
                .and_then(Value::as_array)
                .expect("activeSpeakers.sids is an array");
            for sid in sids {
                let sid = sid.as_str().expect("activeSpeakers.sids entry is a string");
                assert!(
                    sid.starts_with("PA_"),
                    "activeSpeakers sid {sid} lacks PA_ prefix"
                );
            }
            let participants = payload
                .get("participants")
                .and_then(Value::as_array)
                .expect("activeSpeakers.participants is an array");
            for participant in participants {
                validate_participant_snapshot("activeSpeakers.participants[]", participant);
            }
        }
        "connectionQuality" => {
            assert_exact_keys(event_type, payload, &["sid", "identity", "name", "quality"]);
            assert!(str_field(event_type, payload, "sid").starts_with("PA_"));
            let _ = str_field(event_type, payload, "identity");
            let _ = str_field(event_type, payload, "name");
            assert!(QUALITIES.contains(&str_field(event_type, payload, "quality")));
        }
        "e2eeState" => {
            assert_exact_keys(event_type, payload, &["sid", "identity", "name", "state"]);
            assert!(str_field(event_type, payload, "sid").starts_with("PA_"));
            let _ = str_field(event_type, payload, "identity");
            let _ = str_field(event_type, payload, "name");
            assert!(!str_field(event_type, payload, "state").is_empty());
        }
        "stats" => {
            assert_exact_keys(event_type, payload, &["rttMs", "outbound", "inbound"]);
            let rtt = payload.get("rttMs").expect("stats.rttMs exists");
            assert!(
                rtt.is_null() || rtt.as_f64().is_some(),
                "stats.rttMs is null or number"
            );
            let outbound = payload
                .get("outbound")
                .and_then(Value::as_array)
                .expect("stats.outbound is an array");
            for entry in outbound {
                let obj = entry
                    .as_object()
                    .expect("stats.outbound[] is a JSON object");
                assert!(obj.contains_key("trackSid"));
                assert!(TRACK_SOURCES.contains(&str_field("stats.outbound[]", entry, "source")));
                assert!(TRACK_KINDS.contains(&str_field("stats.outbound[]", entry, "kind")));
                assert!(entry.get("bitrateKbps").and_then(Value::as_f64).is_some());
                assert!(entry.get("packetsLost").and_then(Value::as_u64).is_some());
            }
            let inbound = payload
                .get("inbound")
                .and_then(Value::as_array)
                .expect("stats.inbound is an array");
            for entry in inbound {
                let obj = entry.as_object().expect("stats.inbound[] is a JSON object");
                assert!(obj.contains_key("participantSid"));
                assert!(obj.contains_key("trackSid"));
                assert!(TRACK_KINDS.contains(&str_field("stats.inbound[]", entry, "kind")));
                assert!(entry.get("bitrateKbps").and_then(Value::as_f64).is_some());
                assert!(entry.get("packetsLost").and_then(Value::as_u64).is_some());
            }
        }
        "audioPlaybackUnavailable" => {
            assert_exact_keys(event_type, payload, &["message"]);
            assert!(!str_field(event_type, payload, "message").is_empty());
        }
        "participantNameChanged" => {
            assert_exact_keys(event_type, payload, &["sid", "identity", "oldName", "name"]);
            assert!(str_field(event_type, payload, "sid").starts_with("PA_"));
            let _ = str_field(event_type, payload, "identity");
            let _ = str_field(event_type, payload, "oldName");
            let _ = str_field(event_type, payload, "name");
        }
        "participantMetadataChanged" => {
            assert_exact_keys(
                event_type,
                payload,
                &[
                    "sid",
                    "identity",
                    "name",
                    "oldMetadata",
                    "metadata",
                    "attributes",
                ],
            );
            assert!(str_field(event_type, payload, "sid").starts_with("PA_"));
            let _ = str_field(event_type, payload, "identity");
            let _ = str_field(event_type, payload, "name");
            let _ = str_field(event_type, payload, "oldMetadata");
            let _ = str_field(event_type, payload, "metadata");
            validate_string_map(event_type, payload, "attributes");
        }
        "participantAttributesChanged" => {
            assert_exact_keys(
                event_type,
                payload,
                &["sid", "identity", "name", "attributes", "changedAttributes"],
            );
            assert!(str_field(event_type, payload, "sid").starts_with("PA_"));
            let _ = str_field(event_type, payload, "identity");
            let _ = str_field(event_type, payload, "name");
            validate_string_map(event_type, payload, "attributes");
            validate_string_map(event_type, payload, "changedAttributes");
        }
        other => panic!("unknown eventType in fixture: {other}"),
    }
}

#[test]
fn outbound_fixtures_match_contract() {
    let recs = records(OUTBOUND);
    assert!(!recs.is_empty(), "outbound fixture is non-empty");
    for (ty, payload) in &recs {
        validate_record(ty, payload);
    }
}

#[test]
fn inbound_fixtures_match_contract() {
    let recs = records(INBOUND);
    assert!(!recs.is_empty(), "inbound fixture is non-empty");
    for (ty, payload) in &recs {
        validate_record(ty, payload);
    }
}

#[test]
fn outbound_covers_the_single_identity_publish_path() {
    let types: Vec<String> = records(OUTBOUND).into_iter().map(|(ty, _)| ty).collect();
    for expected in [
        "connected",
        "connectionState",
        "Reconnecting",
        "Reconnected",
        "localTrackPublished",
        "localTrackUnpublished",
        "localTrackRepublished",
        "e2eeState",
        "activeSpeakers",
        "stats",
        "audioPlaybackUnavailable",
    ] {
        assert!(
            types.iter().any(|t| t == expected),
            "outbound missing {expected}"
        );
    }
    let published: Vec<(String, String)> = records(OUTBOUND)
        .into_iter()
        .filter(|(ty, _)| ty == "localTrackPublished")
        .map(|(_, p)| {
            (
                p.get("kind").and_then(Value::as_str).unwrap().to_string(),
                p.get("source").and_then(Value::as_str).unwrap().to_string(),
            )
        })
        .collect();
    assert!(published.contains(&("video".into(), "screen_share".into())));
    assert!(published.contains(&("audio".into(), "microphone".into())));
}

#[test]
fn inbound_covers_subscribe_and_lifecycle() {
    let types: Vec<String> = records(INBOUND).into_iter().map(|(ty, _)| ty).collect();
    for expected in [
        "participantJoined",
        "participantNameChanged",
        "participantMetadataChanged",
        "participantAttributesChanged",
        "trackPublished",
        "trackSubscribed",
        "trackMuted",
        "trackUnmuted",
        "trackSubscriptionFailed",
        "trackUnsubscribed",
        "trackUnpublished",
        "e2eeState",
        "activeSpeakers",
        "connectionQuality",
        "participantLeft",
    ] {
        assert!(
            types.iter().any(|t| t == expected),
            "inbound missing {expected}"
        );
    }
}

#[test]
fn video_frame_meta_matches_contract() {
    let parsed: Value = serde_json::from_str(VIDEO_FRAME_META).expect("fixture is valid JSON");
    let recs = parsed.as_array().expect("video_frame_meta is a JSON array");
    assert!(!recs.is_empty(), "video_frame_meta fixture is non-empty");
    for meta in recs {
        let obj = meta.as_object().expect("meta is a JSON object");
        let mut got: Vec<&str> = obj.keys().map(String::as_str).collect();
        got.sort_unstable();
        assert_eq!(
            got,
            vec![
                "bridgeVersion",
                "height",
                "participantSid",
                "source",
                "timestampUs",
                "trackName",
                "trackSid",
                "width"
            ],
            "video_frame_meta key set mismatch"
        );
        let bridge_version = obj
            .get("bridgeVersion")
            .and_then(Value::as_u64)
            .expect("bridgeVersion is an unsigned integer");
        assert!(
            bridge_version >= 1,
            "bridgeVersion {bridge_version} must be positive"
        );
        let participant_sid = obj
            .get("participantSid")
            .and_then(Value::as_str)
            .expect("participantSid is a string");
        let track_sid = obj
            .get("trackSid")
            .and_then(Value::as_str)
            .expect("trackSid is a string");
        let track_name = obj
            .get("trackName")
            .and_then(Value::as_str)
            .expect("trackName is a string");
        let source = obj
            .get("source")
            .and_then(Value::as_str)
            .expect("source is a string");
        assert!(
            participant_sid.starts_with("PA_"),
            "participantSid {participant_sid} lacks PA_"
        );
        assert!(
            track_sid.starts_with("TR_"),
            "trackSid {track_sid} lacks TR_"
        );
        assert!(!track_name.is_empty(), "trackName is not empty");
        assert!(
            TRACK_SOURCES.contains(&source),
            "source {source} is a known LiveKit source"
        );
        let width = obj
            .get("width")
            .and_then(Value::as_u64)
            .expect("width is an integer");
        let height = obj
            .get("height")
            .and_then(Value::as_u64)
            .expect("height is an integer");
        assert!(
            width >= 2 && width % 2 == 0,
            "width {width} must be even and >= 2"
        );
        assert!(
            height >= 2 && height % 2 == 0,
            "height {height} must be even and >= 2"
        );
        assert!(
            obj.get("timestampUs").and_then(Value::as_i64).is_some(),
            "timestampUs is an i64-range integer"
        );
    }
    assert!(
        recs.iter()
            .filter_map(|m| m.get("timestampUs").and_then(Value::as_i64))
            .any(|ts| ts > u32::MAX as i64),
        "expect a >2^32 timestampUs in the fixture to lock the i64 contract"
    );
}

#[test]
fn e2ee_state_is_ok_on_the_wire() {
    for json in [OUTBOUND, INBOUND] {
        for (ty, payload) in records(json) {
            if ty == "e2eeState" {
                assert_eq!(payload.get("state").and_then(Value::as_str), Some("ok"));
            }
        }
    }
}
