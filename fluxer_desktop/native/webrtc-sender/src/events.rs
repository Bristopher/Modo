// SPDX-License-Identifier: AGPL-3.0-or-later

pub fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

pub enum JsonValue {
    Str(String),
    Raw(String),
}

pub fn json_object(fields: &[(&str, JsonValue)]) -> String {
    let mut out = String::from("{");
    for (i, (key, value)) in fields.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_json_string(&mut out, key);
        out.push(':');
        match value {
            JsonValue::Str(s) => push_json_string(&mut out, s),
            JsonValue::Raw(r) => out.push_str(r),
        }
    }
    out.push('}');
    out
}

pub fn json_string_array(items: &[String]) -> String {
    let mut out = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_json_string(&mut out, item);
    }
    out.push(']');
    out
}

pub fn json_u8_array(items: &[u8]) -> String {
    let mut out = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&item.to_string());
    }
    out.push(']');
    out
}

pub fn json_raw_array(items: &[String]) -> String {
    let mut out = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(item);
    }
    out.push(']');
    out
}

pub fn json_string_map(items: &std::collections::HashMap<String, String>) -> String {
    let mut entries: Vec<(&String, &String)> = items.iter().collect();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    let mut out = String::from("{");
    for (i, (key, value)) in entries.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        push_json_string(&mut out, key);
        out.push(':');
        push_json_string(&mut out, value);
    }
    out.push('}');
    out
}

#[cfg(feature = "publisher")]
mod live {
    use super::{
        JsonValue, json_object, json_raw_array, json_string_array, json_string_map, json_u8_array,
    };
    use livekit::participant::{
        ConnectionQuality, LocalParticipant, Participant, RemoteParticipant,
    };
    use livekit::publication::{
        LocalTrackPublication, RemoteTrackPublication, SubscriptionStatus, TrackPublication,
    };
    use livekit::track::{TrackKind, TrackSource};
    use livekit::{DataPacketKind, RoomEvent};

    pub fn track_kind_str(kind: TrackKind) -> &'static str {
        match kind {
            TrackKind::Audio => "audio",
            TrackKind::Video => "video",
        }
    }

    pub fn track_source_str(source: TrackSource) -> &'static str {
        match source {
            TrackSource::Unknown => "unknown",
            TrackSource::Camera => "camera",
            TrackSource::Microphone => "microphone",
            TrackSource::Screenshare => "screen_share",
            TrackSource::ScreenshareAudio => "screen_share_audio",
        }
    }

    pub fn connection_quality_str(quality: ConnectionQuality) -> &'static str {
        match quality {
            ConnectionQuality::Excellent => "excellent",
            ConnectionQuality::Good => "good",
            ConnectionQuality::Poor => "poor",
            ConnectionQuality::Lost => "lost",
        }
    }

    fn s(value: impl Into<String>) -> JsonValue {
        JsonValue::Str(value.into())
    }

    fn b(value: bool) -> JsonValue {
        JsonValue::Raw(value.to_string())
    }

    fn subscription_status_str(status: SubscriptionStatus) -> &'static str {
        match status {
            SubscriptionStatus::Desired => "desired",
            SubscriptionStatus::Subscribed => "subscribed",
            SubscriptionStatus::Unsubscribed => "unsubscribed",
        }
    }

    fn participant_snapshot(participant: &Participant) -> String {
        json_object(&[
            ("sid", s(participant.sid().to_string())),
            ("identity", s(participant.identity().to_string())),
            ("name", s(participant.name())),
        ])
    }

    fn push_remote_participant_fields(
        fields: &mut Vec<(&'static str, JsonValue)>,
        participant: &RemoteParticipant,
    ) {
        fields.push(("participantSid", s(participant.sid().to_string())));
        fields.push(("identity", s(participant.identity().to_string())));
        fields.push(("participantName", s(participant.name())));
    }

    fn push_local_participant_fields(
        fields: &mut Vec<(&'static str, JsonValue)>,
        participant: &LocalParticipant,
    ) {
        fields.push(("participantSid", s(participant.sid().to_string())));
        fields.push(("identity", s(participant.identity().to_string())));
        fields.push(("participantName", s(participant.name())));
    }

    fn push_participant_fields(
        fields: &mut Vec<(&'static str, JsonValue)>,
        participant: &Participant,
    ) {
        fields.push(("participantSid", s(participant.sid().to_string())));
        fields.push(("identity", s(participant.identity().to_string())));
        fields.push(("participantName", s(participant.name())));
    }

    fn push_remote_publication_fields(
        fields: &mut Vec<(&'static str, JsonValue)>,
        publication: &RemoteTrackPublication,
    ) {
        fields.push(("trackSid", s(publication.sid().to_string())));
        fields.push(("trackName", s(publication.name())));
        fields.push(("kind", s(track_kind_str(publication.kind()))));
        fields.push(("source", s(track_source_str(publication.source()))));
        fields.push(("muted", b(publication.is_muted())));
        fields.push(("subscribed", b(publication.is_subscribed())));
        fields.push((
            "subscriptionStatus",
            s(subscription_status_str(publication.subscription_status())),
        ));
    }

    fn push_local_publication_fields(
        fields: &mut Vec<(&'static str, JsonValue)>,
        publication: &LocalTrackPublication,
    ) {
        fields.push(("trackSid", s(publication.sid().to_string())));
        fields.push(("trackName", s(publication.name())));
        fields.push(("kind", s(track_kind_str(publication.kind()))));
        fields.push(("source", s(track_source_str(publication.source()))));
        fields.push(("muted", b(publication.is_muted())));
    }

    fn push_publication_fields(
        fields: &mut Vec<(&'static str, JsonValue)>,
        publication: &TrackPublication,
    ) {
        fields.push(("trackSid", s(publication.sid().to_string())));
        fields.push(("trackName", s(publication.name())));
        fields.push(("kind", s(track_kind_str(publication.kind()))));
        fields.push(("source", s(track_source_str(publication.source()))));
        fields.push(("muted", b(publication.is_muted())));
    }

    fn remote_track_payload(
        participant: &RemoteParticipant,
        publication: &RemoteTrackPublication,
    ) -> String {
        let mut fields = Vec::new();
        push_remote_participant_fields(&mut fields, participant);
        push_remote_publication_fields(&mut fields, publication);
        json_object(&fields)
    }

    const CONNECTED_ROSTER_PARTICIPANTS_MAX: usize = 1024;
    const CONNECTED_ROSTER_TRACKS_PER_PARTICIPANT_MAX: usize = 16;

    fn connected_payload(
        participants_with_tracks: &[(RemoteParticipant, Vec<RemoteTrackPublication>)],
    ) -> String {
        let participant_count = participants_with_tracks
            .len()
            .min(CONNECTED_ROSTER_PARTICIPANTS_MAX);
        let mut entries = Vec::with_capacity(participant_count);
        for (participant, publications) in participants_with_tracks
            .iter()
            .take(CONNECTED_ROSTER_PARTICIPANTS_MAX)
        {
            let track_count = publications
                .len()
                .min(CONNECTED_ROSTER_TRACKS_PER_PARTICIPANT_MAX);
            let mut tracks = Vec::with_capacity(track_count);
            for publication in publications
                .iter()
                .take(CONNECTED_ROSTER_TRACKS_PER_PARTICIPANT_MAX)
            {
                tracks.push(remote_track_payload(participant, publication));
            }
            entries.push(json_object(&[
                ("sid", s(participant.sid().to_string())),
                ("identity", s(participant.identity().to_string())),
                ("name", s(participant.name())),
                ("tracks", JsonValue::Raw(json_raw_array(&tracks))),
            ]));
        }
        assert!(entries.len() <= CONNECTED_ROSTER_PARTICIPANTS_MAX);
        json_object(&[("participants", JsonValue::Raw(json_raw_array(&entries)))])
    }

    fn local_track_payload(
        participant: &LocalParticipant,
        publication: &LocalTrackPublication,
    ) -> String {
        let mut fields = Vec::new();
        push_local_participant_fields(&mut fields, participant);
        push_local_publication_fields(&mut fields, publication);
        json_object(&fields)
    }

    pub fn map_room_event(event: &RoomEvent) -> Option<(&'static str, String)> {
        map_participant_lifecycle_room_event(event)
            .or_else(|| map_participant_profile_room_event(event))
            .or_else(|| map_track_room_event(event))
            .or_else(|| map_local_track_room_event(event))
            .or_else(|| map_connection_room_event(event))
    }

    fn map_participant_lifecycle_room_event(event: &RoomEvent) -> Option<(&'static str, String)> {
        match event {
            RoomEvent::ParticipantConnected(p) | RoomEvent::ParticipantActive(p) => Some((
                "participantJoined",
                json_object(&[
                    ("sid", s(p.sid().to_string())),
                    ("identity", s(p.identity().to_string())),
                    ("name", s(p.name())),
                ]),
            )),
            RoomEvent::ParticipantDisconnected(p) => Some((
                "participantLeft",
                json_object(&[
                    ("sid", s(p.sid().to_string())),
                    ("identity", s(p.identity().to_string())),
                    ("name", s(p.name())),
                ]),
            )),
            RoomEvent::ParticipantNameChanged {
                participant,
                old_name,
                name,
            } => Some((
                "participantNameChanged",
                json_object(&[
                    ("sid", s(participant.sid().to_string())),
                    ("identity", s(participant.identity().to_string())),
                    ("oldName", s(old_name.to_string())),
                    ("name", s(name.to_string())),
                ]),
            )),
            _ => None,
        }
    }

    fn map_participant_profile_room_event(event: &RoomEvent) -> Option<(&'static str, String)> {
        match event {
            RoomEvent::ParticipantMetadataChanged {
                participant,
                old_metadata,
                metadata,
            } => Some((
                "participantMetadataChanged",
                json_object(&[
                    ("sid", s(participant.sid().to_string())),
                    ("identity", s(participant.identity().to_string())),
                    ("name", s(participant.name())),
                    ("oldMetadata", s(old_metadata.to_string())),
                    ("metadata", s(metadata.to_string())),
                    (
                        "attributes",
                        JsonValue::Raw(json_string_map(&participant.attributes())),
                    ),
                ]),
            )),
            RoomEvent::ParticipantAttributesChanged {
                participant,
                changed_attributes,
            } => Some((
                "participantAttributesChanged",
                json_object(&[
                    ("sid", s(participant.sid().to_string())),
                    ("identity", s(participant.identity().to_string())),
                    ("name", s(participant.name())),
                    (
                        "attributes",
                        JsonValue::Raw(json_string_map(&participant.attributes())),
                    ),
                    (
                        "changedAttributes",
                        JsonValue::Raw(json_string_map(changed_attributes)),
                    ),
                ]),
            )),
            _ => None,
        }
    }

    fn map_track_room_event(event: &RoomEvent) -> Option<(&'static str, String)> {
        match event {
            RoomEvent::TrackSubscribed {
                publication,
                participant,
                ..
            } => Some((
                "trackSubscribed",
                remote_track_payload(participant, publication),
            )),
            RoomEvent::TrackUnsubscribed {
                publication,
                participant,
                ..
            } => Some((
                "trackUnsubscribed",
                remote_track_payload(participant, publication),
            )),
            RoomEvent::TrackSubscriptionFailed {
                participant,
                error,
                track_sid,
            } => {
                let mut fields = Vec::new();
                push_remote_participant_fields(&mut fields, participant);
                if let Some(publication) = participant.get_track_publication(track_sid) {
                    push_remote_publication_fields(&mut fields, &publication);
                } else {
                    fields.push(("trackSid", s(track_sid.to_string())));
                }
                fields.push(("error", s(format!("{error}"))));
                Some(("trackSubscriptionFailed", json_object(&fields)))
            }
            RoomEvent::TrackPublished {
                publication,
                participant,
            } => Some((
                "trackPublished",
                remote_track_payload(participant, publication),
            )),
            RoomEvent::TrackUnpublished {
                publication,
                participant,
            } => Some((
                "trackUnpublished",
                remote_track_payload(participant, publication),
            )),
            RoomEvent::TrackMuted {
                participant,
                publication,
            } => {
                let mut fields = Vec::new();
                push_participant_fields(&mut fields, participant);
                push_publication_fields(&mut fields, publication);
                Some(("trackMuted", json_object(&fields)))
            }
            RoomEvent::TrackUnmuted {
                participant,
                publication,
            } => {
                let mut fields = Vec::new();
                push_participant_fields(&mut fields, participant);
                push_publication_fields(&mut fields, publication);
                Some(("trackUnmuted", json_object(&fields)))
            }
            _ => None,
        }
    }

    fn map_connection_room_event(event: &RoomEvent) -> Option<(&'static str, String)> {
        match event {
            RoomEvent::ActiveSpeakersChanged { speakers } => {
                let sids: Vec<String> = speakers.iter().map(|p| p.sid().to_string()).collect();
                let participants: Vec<String> = speakers.iter().map(participant_snapshot).collect();
                Some((
                    "activeSpeakers",
                    json_object(&[
                        ("sids", JsonValue::Raw(json_string_array(&sids))),
                        (
                            "participants",
                            JsonValue::Raw(json_raw_array(&participants)),
                        ),
                    ]),
                ))
            }
            RoomEvent::ConnectionQualityChanged {
                quality,
                participant,
            } => Some((
                "connectionQuality",
                json_object(&[
                    ("sid", s(participant.sid().to_string())),
                    ("identity", s(participant.identity().to_string())),
                    ("name", s(participant.name())),
                    ("quality", s(connection_quality_str(*quality))),
                ]),
            )),
            RoomEvent::DataReceived {
                payload,
                topic,
                kind,
                participant,
            } => Some((
                "dataReceived",
                data_received_payload(
                    payload.as_ref().as_slice(),
                    topic.as_deref(),
                    kind,
                    participant.as_ref(),
                ),
            )),
            RoomEvent::E2eeStateChanged { participant, state } => Some((
                "e2eeState",
                json_object(&[
                    ("sid", s(participant.sid().to_string())),
                    ("identity", s(participant.identity().to_string())),
                    ("name", s(participant.name())),
                    ("state", s(format!("{state:?}").to_lowercase())),
                ]),
            )),
            RoomEvent::ConnectionStateChanged(state) => Some((
                "connectionState",
                json_object(&[("state", s(format!("{state:?}").to_lowercase()))]),
            )),
            RoomEvent::Disconnected { reason } => Some((
                "disconnected",
                json_object(&[("reason", s(format!("{reason:?}").to_lowercase()))]),
            )),
            RoomEvent::Connected {
                participants_with_tracks,
            } => Some(("connected", connected_payload(participants_with_tracks))),
            _ => None,
        }
    }

    fn data_received_payload(
        payload: &[u8],
        topic: Option<&str>,
        kind: &DataPacketKind,
        participant: Option<&RemoteParticipant>,
    ) -> String {
        let mut fields = vec![
            ("payloadBytes", JsonValue::Raw(json_u8_array(payload))),
            (
                "reliable",
                JsonValue::Raw(matches!(kind, DataPacketKind::Reliable).to_string()),
            ),
            (
                "kind",
                s(match kind {
                    DataPacketKind::Reliable => "reliable",
                    DataPacketKind::Lossy => "lossy",
                }),
            ),
        ];
        if let Some(topic) = topic {
            fields.push(("topic", s(topic.to_string())));
        }
        if let Ok(payload_text) = std::str::from_utf8(payload) {
            fields.push(("payloadText", s(payload_text.to_string())));
        }
        if let Some(participant) = participant {
            push_remote_participant_fields(&mut fields, participant);
        }
        json_object(&fields)
    }

    fn map_local_track_room_event(event: &RoomEvent) -> Option<(&'static str, String)> {
        match event {
            RoomEvent::LocalTrackPublished {
                publication,
                participant,
                ..
            } => Some((
                "localTrackPublished",
                local_track_payload(participant, publication),
            )),
            RoomEvent::LocalTrackUnpublished {
                publication,
                participant,
            } => Some((
                "localTrackUnpublished",
                local_track_payload(participant, publication),
            )),
            RoomEvent::LocalTrackRepublished {
                previous_sid,
                publication,
                participant,
                ..
            } => {
                let mut fields = Vec::new();
                push_local_participant_fields(&mut fields, participant);
                fields.push(("previousTrackSid", s(previous_sid.to_string())));
                push_local_publication_fields(&mut fields, publication);
                Some(("localTrackRepublished", json_object(&fields)))
            }
            _ => None,
        }
    }
}

#[cfg(feature = "publisher")]
pub use live::{map_room_event, track_kind_str, track_source_str};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_quote_backslash_and_control_chars() {
        let mut out = String::new();
        push_json_string(&mut out, "a\"b\\c\nd\te");
        assert_eq!(out, "\"a\\\"b\\\\c\\nd\\te\"");

        let mut ctrl = String::new();
        push_json_string(&mut ctrl, "\u{01}");
        assert_eq!(ctrl, "\"\\u0001\"");
    }

    #[test]
    fn leaves_plain_ascii_and_unicode_untouched() {
        let mut out = String::new();
        push_json_string(&mut out, "PA_abc123");
        assert_eq!(out, "\"PA_abc123\"");
    }

    #[test]
    fn json_object_preserves_order_and_mixes_str_and_raw() {
        let json = json_object(&[
            ("sid", JsonValue::Str("PA_1".into())),
            (
                "sids",
                JsonValue::Raw(json_string_array(&["PA_1".into(), "PA_2".into()])),
            ),
        ]);
        assert_eq!(json, "{\"sid\":\"PA_1\",\"sids\":[\"PA_1\",\"PA_2\"]}");
    }

    #[cfg(feature = "publisher")]
    #[test]
    fn track_source_strings_match_livekit_js_sources() {
        use livekit::track::TrackSource;

        assert_eq!(track_source_str(TrackSource::Camera), "camera");
        assert_eq!(track_source_str(TrackSource::Microphone), "microphone");
        assert_eq!(track_source_str(TrackSource::Screenshare), "screen_share");
        assert_eq!(
            track_source_str(TrackSource::ScreenshareAudio),
            "screen_share_audio"
        );
    }

    #[test]
    fn empty_object_and_array() {
        assert_eq!(json_object(&[]), "{}");
        assert_eq!(json_string_array(&[]), "[]");
        assert_eq!(json_u8_array(&[]), "[]");
        assert_eq!(json_raw_array(&[]), "[]");
    }

    #[test]
    fn json_u8_array_serializes_bytes_as_numbers() {
        assert_eq!(json_u8_array(&[0, 1, 127, 255]), "[0,1,127,255]");
    }

    #[test]
    fn raw_array_preserves_prebuilt_json_objects() {
        let items = vec![
            json_object(&[("sid", JsonValue::Str("PA_1".into()))]),
            json_object(&[("sid", JsonValue::Str("PA_2".into()))]),
        ];
        assert_eq!(
            json_raw_array(&items),
            "[{\"sid\":\"PA_1\"},{\"sid\":\"PA_2\"}]"
        );
    }
}
