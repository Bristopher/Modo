// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::send_control::SendHealthSnapshot;
use fluxer_desktop_native::voice::stats as core_stats;

pub type ByteRateSample = core_stats::ByteRateSample;
pub type OutboundEntry = core_stats::OutboundStatsEntry;
pub type InboundEntry = core_stats::InboundStatsEntry;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct ConnectionStats {
    pub rtt_ms: Option<f64>,
    pub outbound: Vec<OutboundEntry>,
    pub inbound: Vec<InboundEntry>,
    pub send: Option<SendHealthSnapshot>,
}

pub fn bitrate_kbps(prev: Option<ByteRateSample>, cur: ByteRateSample) -> f64 {
    core_stats::bitrate_kbps(prev, cur)
}

#[cfg(test)]
pub fn sanitize_kbps(kbps: f64) -> f64 {
    core_stats::sanitize_kbps(kbps)
}

pub fn jitter_seconds_to_ms(jitter_s: f64) -> Option<f64> {
    core_stats::jitter_seconds_to_ms(jitter_s)
}

pub fn rtt_seconds_to_ms(rtt_s: f64) -> Option<f64> {
    core_stats::rtt_seconds_to_ms(rtt_s)
}

pub fn sanitize_audio_level(level: f64) -> Option<f64> {
    core_stats::sanitize_audio_level(level)
}

pub fn stats_to_json(stats: &ConnectionStats) -> String {
    core_stats::stats_to_json(&core_stats::ConnectionStats {
        rtt_ms: stats.rtt_ms,
        outbound: stats.outbound.clone(),
        inbound: stats.inbound.clone(),
        send: stats.send.as_ref().map(send_health_to_core),
    })
}

fn send_health_to_core(send: &SendHealthSnapshot) -> core_stats::SendHealthStats {
    core_stats::SendHealthStats {
        outgoing_video_queue_depth: send.outgoing_video_queue_depth,
        outgoing_video_queue_capacity: send.outgoing_video_queue_capacity,
        outgoing_video_max_queue_depth: send.outgoing_video_max_queue_depth,
        outgoing_video_frames_produced: send.outgoing_video_frames_produced,
        outgoing_video_frames_accepted: send.outgoing_video_frames_accepted,
        outgoing_video_frames_dropped: send.outgoing_video_frames_dropped,
        outgoing_video_frames_coalesced: send.outgoing_video_frames_coalesced,
        outgoing_video_frames_captured: send.outgoing_video_frames_captured,
        outgoing_video_capture_failures: send.outgoing_video_capture_failures,
        outgoing_video_effective_fps: send.outgoing_video_effective_fps,
        outgoing_video_target_fps: send.outgoing_video_target_fps,
        outgoing_video_pacing_target_fps: send.outgoing_video_pacing_target_fps,
        outgoing_video_max_queue_age_ms: send.outgoing_video_max_queue_age_ms,
        outgoing_video_max_push_latency_ms: send.outgoing_video_max_push_latency_ms,
        outgoing_video_pacing_mode: send.outgoing_video_pacing_mode.clone(),
        outgoing_video_bus_active: send.outgoing_video_bus_active,
        outgoing_audio_buffer_target_ms: send.outgoing_audio_buffer_target_ms,
        outgoing_audio_buffer_max_ms: send.outgoing_audio_buffer_max_ms,
        outgoing_audio_underruns: send.outgoing_audio_underruns,
        outgoing_audio_rebuffers: send.outgoing_audio_rebuffers,
        outgoing_audio_max_frame_gap_ms: send.outgoing_audio_max_frame_gap_ms,
        adaptive_send_tier: send.adaptive_send_tier.clone(),
        adaptive_send_reason: send.adaptive_send_reason.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitrate_first_sample_is_zero() {
        let cur = ByteRateSample {
            bytes: 1000,
            timestamp_us: 1_000_000,
        };
        assert_eq!(bitrate_kbps(None, cur), 0.0);
    }

    #[test]
    fn bitrate_computes_kbps_from_byte_delta() {
        let prev = ByteRateSample {
            bytes: 0,
            timestamp_us: 0,
        };
        let cur = ByteRateSample {
            bytes: 12_500,
            timestamp_us: 1_000_000,
        };
        assert_eq!(bitrate_kbps(Some(prev), cur), 100.0);
    }

    #[test]
    fn bitrate_half_second_doubles_rate() {
        let prev = ByteRateSample {
            bytes: 1000,
            timestamp_us: 1_000_000,
        };
        let cur = ByteRateSample {
            bytes: 13_500,
            timestamp_us: 1_500_000,
        };
        assert_eq!(bitrate_kbps(Some(prev), cur), 200.0);
    }

    #[test]
    fn bitrate_rejects_backwards_time_and_bytes() {
        let prev = ByteRateSample {
            bytes: 5000,
            timestamp_us: 2_000_000,
        };
        assert_eq!(
            bitrate_kbps(
                Some(prev),
                ByteRateSample {
                    bytes: 6000,
                    timestamp_us: 1_000_000
                }
            ),
            0.0
        );
        assert_eq!(
            bitrate_kbps(
                Some(prev),
                ByteRateSample {
                    bytes: 6000,
                    timestamp_us: 2_000_000
                }
            ),
            0.0
        );
        assert_eq!(
            bitrate_kbps(
                Some(prev),
                ByteRateSample {
                    bytes: 100,
                    timestamp_us: 3_000_000
                }
            ),
            0.0
        );
    }

    #[test]
    fn sanitize_kbps_drops_nan_inf_negative_and_rounds() {
        assert_eq!(sanitize_kbps(f64::NAN), 0.0);
        assert_eq!(sanitize_kbps(f64::INFINITY), 0.0);
        assert_eq!(sanitize_kbps(-5.0), 0.0);
        assert_eq!(sanitize_kbps(123.456), 123.5);
        assert_eq!(sanitize_kbps(100.0), 100.0);
    }

    #[test]
    fn unit_conversions_seconds_to_ms() {
        assert_eq!(jitter_seconds_to_ms(0.012), Some(12.0));
        assert_eq!(jitter_seconds_to_ms(-1.0), None);
        assert_eq!(jitter_seconds_to_ms(f64::NAN), None);
        assert_eq!(rtt_seconds_to_ms(0.045), Some(45.0));
        assert_eq!(rtt_seconds_to_ms(0.0), None);
        assert_eq!(rtt_seconds_to_ms(f64::INFINITY), None);
    }

    #[test]
    fn sanitize_audio_level_clamps() {
        assert_eq!(sanitize_audio_level(0.5), Some(0.5));
        assert_eq!(sanitize_audio_level(2.0), Some(1.0));
        assert_eq!(sanitize_audio_level(-0.1), Some(0.0));
        assert_eq!(sanitize_audio_level(f64::NAN), None);
    }

    #[test]
    fn empty_stats_serialise_to_null_rtt_and_empty_arrays() {
        let stats = ConnectionStats::default();
        assert_eq!(
            stats_to_json(&stats),
            "{\"rttMs\":null,\"outbound\":[],\"inbound\":[],\"send\":null}"
        );
    }

    #[test]
    fn full_stats_serialise_to_exact_contract_shape() {
        let stats = ConnectionStats {
            rtt_ms: Some(42.0),
            outbound: vec![
                OutboundEntry {
                    track_sid: "TR_mic1".into(),
                    source: "microphone".into(),
                    kind: "audio".into(),
                    codec: Some("audio/opus".into()),
                    bitrate_kbps: 32.0,
                    packets_lost: 0,
                    fps: None,
                    audio_level: Some(0.62),
                    ..Default::default()
                },
                OutboundEntry {
                    track_sid: "TR_screen1".into(),
                    source: "screen_share".into(),
                    kind: "video".into(),
                    codec: Some("video/H265".into()),
                    bitrate_kbps: 2500.5,
                    packets_lost: 3,
                    packets_sent: 9000,
                    fps: Some(30.0),
                    audio_level: None,
                    width: Some(2176),
                    height: Some(1200),
                    source_width: Some(2176),
                    source_height: Some(1200),
                    target_bitrate_kbps: Some(50_000.0),
                    configured_fps: Some(60.0),
                    target_fps: Some(30.0),
                    effective_fps: Some(29.8),
                    frames_produced: Some(120),
                    frames_accepted: Some(118),
                    frames_dropped: Some(1),
                    frames_coalesced: Some(2),
                    frames_captured: Some(117),
                    capture_failures: Some(0),
                    max_queue_age_ms: Some(18),
                    max_push_latency_ms: Some(12),
                    adaptive_send_tier: Some("fps30".into()),
                    adaptive_send_reason: Some("encoderEgressPressure".into()),
                },
            ],
            inbound: vec![InboundEntry {
                participant_sid: "PA_remote1".into(),
                participant_identity: Some("user_2_connection_2".into()),
                track_sid: "TR_remoteAudio".into(),
                source: Some("microphone".into()),
                kind: "audio".into(),
                codec: Some("audio/opus".into()),
                bitrate_kbps: 28.0,
                packets_lost: 1,
                packets_received: 990,
                jitter_ms: Some(5.0),
                audio_level: Some(0.75),
                fps: None,
                width: None,
                height: None,
                source_width: None,
                source_height: None,
            }],
            send: None,
        };
        let json = stats_to_json(&stats);
        assert_eq!(
            json,
            "{\"rttMs\":42,\"outbound\":[\
{\"trackSid\":\"TR_mic1\",\"source\":\"microphone\",\"kind\":\"audio\",\"bitrateKbps\":32,\"packetsLost\":0,\"packetsSent\":0,\"audioLevel\":0.62,\"codec\":\"audio/opus\"},\
{\"trackSid\":\"TR_screen1\",\"source\":\"screen_share\",\"kind\":\"video\",\"bitrateKbps\":2500.5,\"packetsLost\":3,\"packetsSent\":9000,\"fps\":30,\"width\":2176,\"height\":1200,\"sourceWidth\":2176,\"sourceHeight\":1200,\"targetBitrateKbps\":50000,\"configuredFps\":60,\"targetFps\":30,\"effectiveFps\":29.8,\"framesProduced\":120,\"framesAccepted\":118,\"framesDropped\":1,\"framesCoalesced\":2,\"framesCaptured\":117,\"captureFailures\":0,\"maxQueueAgeMs\":18,\"maxPushLatencyMs\":12,\"adaptiveSendTier\":\"fps30\",\"adaptiveSendReason\":\"encoderEgressPressure\",\"codec\":\"video/H265\"}\
],\"inbound\":[\
{\"participantSid\":\"PA_remote1\",\"trackSid\":\"TR_remoteAudio\",\"kind\":\"audio\",\"bitrateKbps\":28,\"packetsLost\":1,\"packetsReceived\":990,\"participantIdentity\":\"user_2_connection_2\",\"source\":\"microphone\",\"jitterMs\":5,\"audioLevel\":0.75,\"codec\":\"audio/opus\"}\
],\"send\":null}"
        );
        let _ = json;
    }

    #[test]
    fn send_health_serialises_to_exact_contract_shape() {
        let stats = ConnectionStats {
            rtt_ms: None,
            outbound: vec![],
            inbound: vec![],
            send: Some(SendHealthSnapshot {
                outgoing_video_queue_depth: 1,
                outgoing_video_queue_capacity: 8,
                outgoing_video_max_queue_depth: 4,
                outgoing_video_frames_produced: 2,
                outgoing_video_frames_accepted: 3,
                outgoing_video_frames_dropped: 4,
                outgoing_video_frames_coalesced: 5,
                outgoing_video_frames_captured: 6,
                outgoing_video_capture_failures: 7,
                outgoing_video_effective_fps: 59.94,
                outgoing_video_target_fps: 30.0,
                outgoing_video_pacing_target_fps: 60.0,
                outgoing_video_max_queue_age_ms: 8,
                outgoing_video_max_push_latency_ms: 9,
                outgoing_video_pacing_mode: "source".to_string(),
                outgoing_video_bus_active: true,
                outgoing_audio_buffer_target_ms: 300,
                outgoing_audio_buffer_max_ms: 750,
                outgoing_audio_underruns: 10,
                outgoing_audio_rebuffers: 11,
                outgoing_audio_max_frame_gap_ms: 120,
                adaptive_send_tier: "fps30".to_string(),
                adaptive_send_reason: "sendLatencyPressure".to_string(),
            }),
        };

        assert_eq!(
            stats_to_json(&stats),
            "{\"rttMs\":null,\"outbound\":[],\"inbound\":[],\"send\":{\
\"outgoingVideoQueueDepth\":1,\
\"outgoingVideoQueueCapacity\":8,\
\"outgoingVideoMaxQueueDepth\":4,\
\"outgoingVideoFramesProduced\":2,\
\"outgoingVideoFramesAccepted\":3,\
\"outgoingVideoFramesDropped\":4,\
\"outgoingVideoFramesCoalesced\":5,\
\"outgoingVideoFramesCaptured\":6,\
\"outgoingVideoCaptureFailures\":7,\
\"outgoingVideoEffectiveFps\":59.94,\
\"outgoingVideoTargetFps\":30,\
\"outgoingVideoPacingTargetFps\":60,\
\"outgoingVideoMaxQueueAgeMs\":8,\
\"outgoingVideoMaxPushLatencyMs\":9,\
\"outgoingVideoPacingMode\":\"source\",\
\"outgoingVideoBusActive\":true,\
\"outgoingAudioBufferTargetMs\":300,\
\"outgoingAudioBufferMaxMs\":750,\
\"outgoingAudioUnderruns\":10,\
\"outgoingAudioRebuffers\":11,\
\"outgoingAudioMaxFrameGapMs\":120,\
\"adaptiveSendTier\":\"fps30\",\
\"adaptiveSendReason\":\"sendLatencyPressure\"\
}}"
        );
    }

    #[test]
    fn video_inbound_omits_audio_level_audio_omits_fps() {
        let stats = ConnectionStats {
            rtt_ms: None,
            outbound: vec![],
            inbound: vec![InboundEntry {
                participant_sid: "PA_x".into(),
                participant_identity: None,
                track_sid: "TR_v".into(),
                source: Some("screen_share".into()),
                kind: "video".into(),
                codec: None,
                bitrate_kbps: 1000.0,
                packets_lost: 0,
                packets_received: 5000,
                jitter_ms: None,
                audio_level: None,
                fps: Some(29.94),
                width: Some(3840),
                height: Some(2160),
                source_width: Some(3840),
                source_height: Some(2160),
            }],
            send: None,
        };
        let json = stats_to_json(&stats);
        assert!(!json.contains("audioLevel"));
        assert!(!json.contains("jitterMs"));
        assert!(json.contains("\"source\":\"screen_share\""));
        assert!(json.contains("\"fps\":29.9"));
        assert!(json.contains("\"width\":3840"));
        assert!(json.contains("\"height\":2160"));
        assert!(json.contains("\"rttMs\":null"));
    }
}
