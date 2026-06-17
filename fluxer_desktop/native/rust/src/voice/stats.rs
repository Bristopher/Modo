// SPDX-License-Identifier: AGPL-3.0-or-later

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ByteRateSample {
    pub bytes: u64,
    pub timestamp_us: i64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct OutboundStatsEntry {
    pub track_sid: String,
    pub source: String,
    pub kind: String,
    pub codec: Option<String>,
    pub bitrate_kbps: f64,
    pub packets_lost: i64,
    pub packets_sent: u64,
    pub fps: Option<f64>,
    pub audio_level: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
    pub target_bitrate_kbps: Option<f64>,
    pub configured_fps: Option<f64>,
    pub target_fps: Option<f64>,
    pub effective_fps: Option<f64>,
    pub frames_produced: Option<u64>,
    pub frames_accepted: Option<u64>,
    pub frames_dropped: Option<u64>,
    pub frames_coalesced: Option<u64>,
    pub frames_captured: Option<u64>,
    pub capture_failures: Option<u64>,
    pub max_queue_age_ms: Option<u64>,
    pub max_push_latency_ms: Option<u64>,
    pub adaptive_send_tier: Option<String>,
    pub adaptive_send_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct InboundStatsEntry {
    pub participant_sid: String,
    pub participant_identity: Option<String>,
    pub track_sid: String,
    pub source: Option<String>,
    pub kind: String,
    pub codec: Option<String>,
    pub bitrate_kbps: f64,
    pub packets_lost: i64,
    pub packets_received: u64,
    pub jitter_ms: Option<f64>,
    pub audio_level: Option<f64>,
    pub fps: Option<f64>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub source_width: Option<u32>,
    pub source_height: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SendHealthStats {
    pub outgoing_video_queue_depth: u64,
    pub outgoing_video_queue_capacity: u64,
    pub outgoing_video_max_queue_depth: u64,
    pub outgoing_video_frames_produced: u64,
    pub outgoing_video_frames_accepted: u64,
    pub outgoing_video_frames_dropped: u64,
    pub outgoing_video_frames_coalesced: u64,
    pub outgoing_video_frames_captured: u64,
    pub outgoing_video_capture_failures: u64,
    pub outgoing_video_effective_fps: f64,
    pub outgoing_video_target_fps: f64,
    pub outgoing_video_pacing_target_fps: f64,
    pub outgoing_video_max_queue_age_ms: u64,
    pub outgoing_video_max_push_latency_ms: u64,
    pub outgoing_video_pacing_mode: String,
    pub outgoing_video_bus_active: bool,
    pub outgoing_audio_buffer_target_ms: u32,
    pub outgoing_audio_buffer_max_ms: u32,
    pub outgoing_audio_underruns: u64,
    pub outgoing_audio_rebuffers: u64,
    pub outgoing_audio_max_frame_gap_ms: u64,
    pub adaptive_send_tier: String,
    pub adaptive_send_reason: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ConnectionStats {
    pub rtt_ms: Option<f64>,
    pub outbound: Vec<OutboundStatsEntry>,
    pub inbound: Vec<InboundStatsEntry>,
    pub send: Option<SendHealthStats>,
}

pub fn bitrate_kbps(prev: Option<ByteRateSample>, cur: ByteRateSample) -> f64 {
    let Some(prev) = prev else {
        return 0.0;
    };
    let dt_us = cur.timestamp_us - prev.timestamp_us;
    if dt_us <= 0 {
        return 0.0;
    }
    if cur.bytes < prev.bytes {
        return 0.0;
    }
    let delta_bytes = (cur.bytes - prev.bytes) as f64;
    let dt_seconds = dt_us as f64 / 1_000_000.0;
    (delta_bytes * 8.0) / dt_seconds / 1000.0
}

pub fn sanitize_kbps(kbps: f64) -> f64 {
    if !kbps.is_finite() || kbps < 0.0 {
        return 0.0;
    }
    (kbps * 10.0).round() / 10.0
}

pub fn jitter_seconds_to_ms(jitter_s: f64) -> Option<f64> {
    if !jitter_s.is_finite() || jitter_s < 0.0 {
        return None;
    }
    Some((jitter_s * 1000.0 * 100.0).round() / 100.0)
}

pub fn rtt_seconds_to_ms(rtt_s: f64) -> Option<f64> {
    if !rtt_s.is_finite() || rtt_s <= 0.0 {
        return None;
    }
    Some((rtt_s * 1000.0 * 100.0).round() / 100.0)
}

pub fn sanitize_audio_level(level: f64) -> Option<f64> {
    if !level.is_finite() {
        return None;
    }
    Some(level.clamp(0.0, 1.0))
}

pub fn stats_to_json(stats: &ConnectionStats) -> String {
    let rtt = match stats.rtt_ms {
        Some(ms) if ms.is_finite() && ms >= 0.0 => {
            JsonValue::Raw(fmt_num((ms * 100.0).round() / 100.0))
        }
        _ => JsonValue::Raw("null".to_string()),
    };
    let outbound_items: Vec<String> = stats.outbound.iter().map(outbound_json).collect();
    let inbound_items: Vec<String> = stats.inbound.iter().map(inbound_json).collect();
    let send = match &stats.send {
        Some(send) => JsonValue::Raw(send_health_json(send)),
        None => JsonValue::Raw("null".to_string()),
    };
    json_object(&[
        ("rttMs", rtt),
        ("outbound", JsonValue::Raw(raw_array(&outbound_items))),
        ("inbound", JsonValue::Raw(raw_array(&inbound_items))),
        ("send", send),
    ])
}

enum JsonValue {
    Str(String),
    Raw(String),
}

fn outbound_json(entry: &OutboundStatsEntry) -> String {
    let mut fields = vec![
        ("trackSid", JsonValue::Str(entry.track_sid.clone())),
        ("source", JsonValue::Str(entry.source.clone())),
        ("kind", JsonValue::Str(entry.kind.clone())),
        (
            "bitrateKbps",
            JsonValue::Raw(fmt_num(sanitize_kbps(entry.bitrate_kbps))),
        ),
        (
            "packetsLost",
            JsonValue::Raw(entry.packets_lost.to_string()),
        ),
        (
            "packetsSent",
            JsonValue::Raw(entry.packets_sent.to_string()),
        ),
    ];
    if let Some(fps) = entry.fps
        && fps.is_finite()
        && fps >= 0.0
    {
        fields.push(("fps", JsonValue::Raw(fmt_num((fps * 10.0).round() / 10.0))));
    }
    if let Some(level) = entry.audio_level {
        fields.push((
            "audioLevel",
            JsonValue::Raw(fmt_num((level * 1000.0).round() / 1000.0)),
        ));
    }
    if let Some(width) = entry.width {
        fields.push(("width", JsonValue::Raw(width.to_string())));
    }
    if let Some(height) = entry.height {
        fields.push(("height", JsonValue::Raw(height.to_string())));
    }
    if let Some(width) = entry.source_width {
        fields.push(("sourceWidth", JsonValue::Raw(width.to_string())));
    }
    if let Some(height) = entry.source_height {
        fields.push(("sourceHeight", JsonValue::Raw(height.to_string())));
    }
    if let Some(kbps) = entry.target_bitrate_kbps
        && kbps.is_finite()
        && kbps >= 0.0
    {
        fields.push((
            "targetBitrateKbps",
            JsonValue::Raw(fmt_num((kbps * 10.0).round() / 10.0)),
        ));
    }
    if let Some(fps) = entry.configured_fps
        && fps.is_finite()
        && fps >= 0.0
    {
        fields.push((
            "configuredFps",
            JsonValue::Raw(fmt_num((fps * 10.0).round() / 10.0)),
        ));
    }
    if let Some(fps) = entry.target_fps
        && fps.is_finite()
        && fps >= 0.0
    {
        fields.push((
            "targetFps",
            JsonValue::Raw(fmt_num((fps * 10.0).round() / 10.0)),
        ));
    }
    if let Some(fps) = entry.effective_fps
        && fps.is_finite()
        && fps >= 0.0
    {
        fields.push((
            "effectiveFps",
            JsonValue::Raw(fmt_num((fps * 10.0).round() / 10.0)),
        ));
    }
    if let Some(value) = entry.frames_produced {
        fields.push(("framesProduced", JsonValue::Raw(value.to_string())));
    }
    if let Some(value) = entry.frames_accepted {
        fields.push(("framesAccepted", JsonValue::Raw(value.to_string())));
    }
    if let Some(value) = entry.frames_dropped {
        fields.push(("framesDropped", JsonValue::Raw(value.to_string())));
    }
    if let Some(value) = entry.frames_coalesced {
        fields.push(("framesCoalesced", JsonValue::Raw(value.to_string())));
    }
    if let Some(value) = entry.frames_captured {
        fields.push(("framesCaptured", JsonValue::Raw(value.to_string())));
    }
    if let Some(value) = entry.capture_failures {
        fields.push(("captureFailures", JsonValue::Raw(value.to_string())));
    }
    if let Some(value) = entry.max_queue_age_ms {
        fields.push(("maxQueueAgeMs", JsonValue::Raw(value.to_string())));
    }
    if let Some(value) = entry.max_push_latency_ms {
        fields.push(("maxPushLatencyMs", JsonValue::Raw(value.to_string())));
    }
    if let Some(value) = &entry.adaptive_send_tier {
        fields.push(("adaptiveSendTier", JsonValue::Str(value.clone())));
    }
    if let Some(value) = &entry.adaptive_send_reason {
        fields.push(("adaptiveSendReason", JsonValue::Str(value.clone())));
    }
    if let Some(codec) = &entry.codec {
        fields.push(("codec", JsonValue::Str(codec.clone())));
    }
    json_object(&fields)
}

fn inbound_json(entry: &InboundStatsEntry) -> String {
    let mut fields = vec![
        (
            "participantSid",
            JsonValue::Str(entry.participant_sid.clone()),
        ),
        ("trackSid", JsonValue::Str(entry.track_sid.clone())),
        ("kind", JsonValue::Str(entry.kind.clone())),
        (
            "bitrateKbps",
            JsonValue::Raw(fmt_num(sanitize_kbps(entry.bitrate_kbps))),
        ),
        (
            "packetsLost",
            JsonValue::Raw(entry.packets_lost.to_string()),
        ),
        (
            "packetsReceived",
            JsonValue::Raw(entry.packets_received.to_string()),
        ),
    ];
    if let Some(identity) = &entry.participant_identity {
        fields.push(("participantIdentity", JsonValue::Str(identity.clone())));
    }
    if let Some(source) = &entry.source {
        fields.push(("source", JsonValue::Str(source.clone())));
    }
    if let Some(jitter_ms) = entry.jitter_ms {
        fields.push(("jitterMs", JsonValue::Raw(fmt_num(jitter_ms))));
    }
    if let Some(level) = entry.audio_level {
        fields.push((
            "audioLevel",
            JsonValue::Raw(fmt_num((level * 1000.0).round() / 1000.0)),
        ));
    }
    if let Some(fps) = entry.fps
        && fps.is_finite()
        && fps >= 0.0
    {
        fields.push(("fps", JsonValue::Raw(fmt_num((fps * 10.0).round() / 10.0))));
    }
    if let Some(width) = entry.width {
        fields.push(("width", JsonValue::Raw(width.to_string())));
    }
    if let Some(height) = entry.height {
        fields.push(("height", JsonValue::Raw(height.to_string())));
    }
    if let Some(width) = entry.source_width {
        fields.push(("sourceWidth", JsonValue::Raw(width.to_string())));
    }
    if let Some(height) = entry.source_height {
        fields.push(("sourceHeight", JsonValue::Raw(height.to_string())));
    }
    if let Some(codec) = &entry.codec {
        fields.push(("codec", JsonValue::Str(codec.clone())));
    }
    json_object(&fields)
}

fn send_health_json(send: &SendHealthStats) -> String {
    json_object(&[
        (
            "outgoingVideoQueueDepth",
            JsonValue::Raw(send.outgoing_video_queue_depth.to_string()),
        ),
        (
            "outgoingVideoQueueCapacity",
            JsonValue::Raw(send.outgoing_video_queue_capacity.to_string()),
        ),
        (
            "outgoingVideoMaxQueueDepth",
            JsonValue::Raw(send.outgoing_video_max_queue_depth.to_string()),
        ),
        (
            "outgoingVideoFramesProduced",
            JsonValue::Raw(send.outgoing_video_frames_produced.to_string()),
        ),
        (
            "outgoingVideoFramesAccepted",
            JsonValue::Raw(send.outgoing_video_frames_accepted.to_string()),
        ),
        (
            "outgoingVideoFramesDropped",
            JsonValue::Raw(send.outgoing_video_frames_dropped.to_string()),
        ),
        (
            "outgoingVideoFramesCoalesced",
            JsonValue::Raw(send.outgoing_video_frames_coalesced.to_string()),
        ),
        (
            "outgoingVideoFramesCaptured",
            JsonValue::Raw(send.outgoing_video_frames_captured.to_string()),
        ),
        (
            "outgoingVideoCaptureFailures",
            JsonValue::Raw(send.outgoing_video_capture_failures.to_string()),
        ),
        (
            "outgoingVideoEffectiveFps",
            JsonValue::Raw(fmt_num(send.outgoing_video_effective_fps)),
        ),
        (
            "outgoingVideoTargetFps",
            JsonValue::Raw(fmt_num(send.outgoing_video_target_fps)),
        ),
        (
            "outgoingVideoPacingTargetFps",
            JsonValue::Raw(fmt_num(send.outgoing_video_pacing_target_fps)),
        ),
        (
            "outgoingVideoMaxQueueAgeMs",
            JsonValue::Raw(send.outgoing_video_max_queue_age_ms.to_string()),
        ),
        (
            "outgoingVideoMaxPushLatencyMs",
            JsonValue::Raw(send.outgoing_video_max_push_latency_ms.to_string()),
        ),
        (
            "outgoingVideoPacingMode",
            JsonValue::Str(send.outgoing_video_pacing_mode.clone()),
        ),
        (
            "outgoingVideoBusActive",
            JsonValue::Raw(send.outgoing_video_bus_active.to_string()),
        ),
        (
            "outgoingAudioBufferTargetMs",
            JsonValue::Raw(send.outgoing_audio_buffer_target_ms.to_string()),
        ),
        (
            "outgoingAudioBufferMaxMs",
            JsonValue::Raw(send.outgoing_audio_buffer_max_ms.to_string()),
        ),
        (
            "outgoingAudioUnderruns",
            JsonValue::Raw(send.outgoing_audio_underruns.to_string()),
        ),
        (
            "outgoingAudioRebuffers",
            JsonValue::Raw(send.outgoing_audio_rebuffers.to_string()),
        ),
        (
            "outgoingAudioMaxFrameGapMs",
            JsonValue::Raw(send.outgoing_audio_max_frame_gap_ms.to_string()),
        ),
        (
            "adaptiveSendTier",
            JsonValue::Str(send.adaptive_send_tier.clone()),
        ),
        (
            "adaptiveSendReason",
            JsonValue::Str(send.adaptive_send_reason.clone()),
        ),
    ])
}

fn json_object(fields: &[(&str, JsonValue)]) -> String {
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

fn raw_array(items: &[String]) -> String {
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

fn push_json_string(out: &mut String, value: &str) {
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

fn fmt_num(value: f64) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }
    if value.fract() == 0.0 {
        return (value as i64).to_string();
    }
    format!("{value}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrate_uses_byte_delta_over_time() {
        let prev = ByteRateSample {
            bytes: 2_000,
            timestamp_us: 1_000_000,
        };
        let cur = ByteRateSample {
            bytes: 14_500,
            timestamp_us: 1_500_000,
        };

        assert_eq!(bitrate_kbps(Some(prev), cur), 200.0);
    }

    #[test]
    fn stats_json_uses_canonical_voice_contract_shape() {
        let stats = ConnectionStats {
            rtt_ms: Some(18.25),
            outbound: vec![OutboundStatsEntry {
                track_sid: "TR_audio".to_string(),
                source: "microphone".to_string(),
                kind: "audio".to_string(),
                codec: Some("audio/opus".to_string()),
                bitrate_kbps: 48.04,
                packets_lost: 0,
                fps: None,
                audio_level: Some(0.1234),
                ..Default::default()
            }],
            inbound: vec![InboundStatsEntry {
                participant_sid: "PA_remote".to_string(),
                participant_identity: Some("user_2_connection_2".to_string()),
                track_sid: "TR_remote".to_string(),
                source: Some("microphone".to_string()),
                kind: "audio".to_string(),
                codec: None,
                bitrate_kbps: 31.96,
                packets_lost: 2,
                packets_received: 100,
                jitter_ms: Some(4.5),
                audio_level: Some(0.1234),
                fps: None,
                width: None,
                height: None,
                source_width: None,
                source_height: None,
            }],
            send: None,
        };

        assert_eq!(
            stats_to_json(&stats),
            "{\"rttMs\":18.25,\"outbound\":[{\"trackSid\":\"TR_audio\",\"source\":\"microphone\",\"kind\":\"audio\",\"bitrateKbps\":48,\"packetsLost\":0,\"packetsSent\":0,\"audioLevel\":0.123,\"codec\":\"audio/opus\"}],\"inbound\":[{\"participantSid\":\"PA_remote\",\"trackSid\":\"TR_remote\",\"kind\":\"audio\",\"bitrateKbps\":32,\"packetsLost\":2,\"packetsReceived\":100,\"participantIdentity\":\"user_2_connection_2\",\"source\":\"microphone\",\"jitterMs\":4.5,\"audioLevel\":0.123}],\"send\":null}"
        );
    }
}
