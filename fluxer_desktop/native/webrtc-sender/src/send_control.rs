// SPDX-License-Identifier: AGPL-3.0-or-later

use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

pub const DEFAULT_AUDIO_BUFFER_TARGET_MS: u32 = 200;
pub const DEFAULT_AUDIO_BUFFER_MAX_MS: u32 = 750;
pub const DEFAULT_MIN_VIDEO_FPS: f64 = 15.0;

const PRESSURE_WINDOW_MS: u64 = 5_000;
const RECOVERY_WINDOW_COUNT: u32 = 12;
const AUDIO_REBUFFER_GAP_MS: u64 = 120;
const AUDIO_STABLE_GAP_MS: u64 = 60;
const AUDIO_BUFFER_STEP_MS: u32 = 100;

#[derive(Clone, Debug, PartialEq)]
pub struct SendHealthSnapshot {
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

impl SendHealthSnapshot {
    pub fn idle(audio: &AdaptiveAudioStats) -> Self {
        Self {
            outgoing_video_queue_depth: 0,
            outgoing_video_queue_capacity: 0,
            outgoing_video_max_queue_depth: 0,
            outgoing_video_frames_produced: 0,
            outgoing_video_frames_accepted: 0,
            outgoing_video_frames_dropped: 0,
            outgoing_video_frames_coalesced: 0,
            outgoing_video_frames_captured: 0,
            outgoing_video_capture_failures: 0,
            outgoing_video_effective_fps: 0.0,
            outgoing_video_target_fps: 0.0,
            outgoing_video_pacing_target_fps: 0.0,
            outgoing_video_max_queue_age_ms: 0,
            outgoing_video_max_push_latency_ms: 0,
            outgoing_video_pacing_mode: "idle".to_string(),
            outgoing_video_bus_active: false,
            outgoing_audio_buffer_target_ms: audio.target_buffer_ms(),
            outgoing_audio_buffer_max_ms: audio.max_buffer_ms(),
            outgoing_audio_underruns: audio.underruns.load(Ordering::Relaxed),
            outgoing_audio_rebuffers: audio.rebuffers.load(Ordering::Relaxed),
            outgoing_audio_max_frame_gap_ms: audio.max_frame_gap_ms.load(Ordering::Relaxed),
            adaptive_send_tier: "idle".to_string(),
            adaptive_send_reason: "notPublishing".to_string(),
        }
    }
}

pub struct AdaptiveVideoController {
    requested_fps: f64,
    min_fps: f64,
    adaptive: bool,
    state: Mutex<AdaptiveVideoState>,
}

struct AdaptiveVideoState {
    current_fps: f64,
    tier: String,
    reason: String,
    window_started_ms: u64,
    window_produced: u64,
    window_coalesced: u64,
    window_dropped: u64,
    window_max_queue_age_ms: u64,
    window_max_push_latency_ms: u64,
    window_egress_fps_sum: f64,
    window_egress_fps_samples: u32,
    stable_windows: u32,
}

impl AdaptiveVideoController {
    pub fn new(requested_fps: f64, min_fps: f64, adaptive: bool, now_ms: u64) -> Self {
        let requested_fps = sanitize_fps(requested_fps, 30.0);
        let min_fps = sanitize_fps(min_fps, DEFAULT_MIN_VIDEO_FPS).min(requested_fps);
        Self {
            requested_fps,
            min_fps,
            adaptive,
            state: Mutex::new(AdaptiveVideoState {
                current_fps: requested_fps,
                tier: "full".to_string(),
                reason: "stable".to_string(),
                window_started_ms: now_ms,
                window_produced: 0,
                window_coalesced: 0,
                window_dropped: 0,
                window_max_queue_age_ms: 0,
                window_max_push_latency_ms: 0,
                window_egress_fps_sum: 0.0,
                window_egress_fps_samples: 0,
                stable_windows: 0,
            }),
        }
    }

    pub fn current_fps(&self) -> f64 {
        self.state.lock().current_fps
    }

    pub fn tier_and_reason(&self) -> (String, String) {
        let state = self.state.lock();
        (state.tier.clone(), state.reason.clone())
    }

    pub fn record_enqueue(&self, now_ms: u64, coalesced: bool) {
        let mut state = self.state.lock();
        self.rotate_window(&mut state, now_ms);
        state.window_produced += 1;
        if coalesced {
            state.window_coalesced += 1;
        }
    }

    #[cfg(test)]
    pub fn record_drop(&self, now_ms: u64) {
        let mut state = self.state.lock();
        self.rotate_window(&mut state, now_ms);
        state.window_dropped += 1;
    }

    pub fn record_capture(&self, now_ms: u64, queue_age_ms: u64, push_latency_ms: u64) {
        let mut state = self.state.lock();
        self.rotate_window(&mut state, now_ms);
        state.window_max_queue_age_ms = state.window_max_queue_age_ms.max(queue_age_ms);
        state.window_max_push_latency_ms = state.window_max_push_latency_ms.max(push_latency_ms);
    }

    pub fn record_egress_fps(&self, now_ms: u64, fps: f64) {
        if !fps.is_finite() || fps < 0.0 {
            return;
        }
        let mut state = self.state.lock();
        self.rotate_window(&mut state, now_ms);
        state.window_egress_fps_sum += fps;
        state.window_egress_fps_samples += 1;
    }

    fn rotate_window(&self, state: &mut AdaptiveVideoState, now_ms: u64) {
        if now_ms.saturating_sub(state.window_started_ms) < PRESSURE_WINDOW_MS {
            return;
        }
        self.apply_window(state);
        state.window_started_ms = now_ms;
        state.window_produced = 0;
        state.window_coalesced = 0;
        state.window_dropped = 0;
        state.window_max_queue_age_ms = 0;
        state.window_max_push_latency_ms = 0;
        state.window_egress_fps_sum = 0.0;
        state.window_egress_fps_samples = 0;
    }

    fn apply_window(&self, state: &mut AdaptiveVideoState) {
        if !self.adaptive {
            state.current_fps = self.requested_fps;
            state.tier = "full".to_string();
            state.reason = "adaptiveDisabled".to_string();
            return;
        }

        let frame_interval_ms = (1000.0 / state.current_fps.max(1.0)).ceil() as u64;
        let latency_pressure = state.window_max_queue_age_ms > frame_interval_ms * 2
            || state.window_max_push_latency_ms > frame_interval_ms * 2;
        let drop_ratio = state.window_dropped as f64 / state.window_produced.max(1) as f64;
        let encoder_drop_pressure = state.window_produced >= 10 && drop_ratio > 0.05;
        let average_egress_fps = if state.window_egress_fps_samples == 0 {
            None
        } else {
            Some(state.window_egress_fps_sum / state.window_egress_fps_samples as f64)
        };
        let egress_pressure = average_egress_fps.is_some_and(|fps| {
            state.window_egress_fps_samples >= 2
                && state.window_produced >= 10
                && fps < state.current_fps * 0.75
        });
        let pressure = latency_pressure || encoder_drop_pressure || egress_pressure;

        if pressure {
            let next = if state.current_fps > 30.0 {
                30.0
            } else if state.current_fps > self.min_fps {
                self.min_fps
            } else {
                state.current_fps
            };
            if next < state.current_fps {
                state.current_fps = next;
                state.tier = tier_for_fps(self.requested_fps, state.current_fps);
            }
            state.reason = if latency_pressure {
                "sendLatencyPressure".to_string()
            } else if encoder_drop_pressure {
                "encoderDropPressure".to_string()
            } else {
                "encoderEgressPressure".to_string()
            };
            state.stable_windows = 0;
            return;
        }

        state.stable_windows += 1;
        if state.current_fps >= self.requested_fps {
            state.reason = "stable".to_string();
        }
        if state.stable_windows >= RECOVERY_WINDOW_COUNT && state.current_fps < self.requested_fps {
            state.current_fps = (state.current_fps * 2.0).min(self.requested_fps);
            state.tier = tier_for_fps(self.requested_fps, state.current_fps);
            state.stable_windows = 0;
            if state.current_fps >= self.requested_fps {
                state.reason = "stable".to_string();
            }
        }
    }
}

pub struct AdaptiveVideoStats {
    produced: AtomicU64,
    accepted: AtomicU64,
    dropped: AtomicU64,
    coalesced: AtomicU64,
    captured: AtomicU64,
    capture_failures: AtomicU64,
    queue_depth: AtomicU64,
    max_queue_depth: AtomicU64,
    max_queue_age_ms: AtomicU64,
    max_push_latency_ms: AtomicU64,
    first_capture_ms: AtomicU64,
    last_capture_ms: AtomicU64,
    controller: AdaptiveVideoController,
}

#[derive(Clone, Debug)]
pub struct VideoTelemetryExtras {
    pub pacing_mode: String,
    pub pacing_target_fps: f64,
    pub queue_capacity: u64,
    pub bus_active: bool,
}

impl Default for VideoTelemetryExtras {
    fn default() -> Self {
        Self {
            pacing_mode: "unknown".to_string(),
            pacing_target_fps: 0.0,
            queue_capacity: 0,
            bus_active: false,
        }
    }
}

impl AdaptiveVideoStats {
    pub fn new(requested_fps: f64, min_fps: f64, adaptive: bool, now_ms: u64) -> Self {
        Self {
            produced: AtomicU64::new(0),
            accepted: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            coalesced: AtomicU64::new(0),
            captured: AtomicU64::new(0),
            capture_failures: AtomicU64::new(0),
            queue_depth: AtomicU64::new(0),
            max_queue_depth: AtomicU64::new(0),
            max_queue_age_ms: AtomicU64::new(0),
            max_push_latency_ms: AtomicU64::new(0),
            first_capture_ms: AtomicU64::new(0),
            last_capture_ms: AtomicU64::new(0),
            controller: AdaptiveVideoController::new(requested_fps, min_fps, adaptive, now_ms),
        }
    }

    #[cfg(test)]
    pub fn record_enqueue(&self, now_ms: u64, replaced_pending: bool) {
        self.record_enqueue_with_depth(now_ms, replaced_pending, 1);
    }

    pub fn record_enqueue_with_depth(&self, now_ms: u64, replaced_pending: bool, queue_depth: u64) {
        self.produced.fetch_add(1, Ordering::Relaxed);
        self.accepted.fetch_add(1, Ordering::Relaxed);
        self.queue_depth.store(queue_depth, Ordering::Relaxed);
        update_max(&self.max_queue_depth, queue_depth);
        if replaced_pending {
            self.coalesced.fetch_add(1, Ordering::Relaxed);
        }
        self.controller.record_enqueue(now_ms, replaced_pending);
    }

    #[cfg(test)]
    pub fn record_drop(&self, now_ms: u64) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
        self.controller.record_drop(now_ms);
    }

    pub fn record_reject(&self) {
        self.dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_capture(&self, now_ms: u64, queue_age_ms: u64, push_latency_ms: u64) {
        update_min_nonzero(&self.first_capture_ms, now_ms);
        update_max(&self.last_capture_ms, now_ms);
        self.captured.fetch_add(1, Ordering::Relaxed);
        self.queue_depth.store(0, Ordering::Relaxed);
        update_max(&self.max_queue_age_ms, queue_age_ms);
        update_max(&self.max_push_latency_ms, push_latency_ms);
        self.controller
            .record_capture(now_ms, queue_age_ms, push_latency_ms);
    }

    pub fn record_egress_fps(&self, now_ms: u64, fps: f64) {
        self.controller.record_egress_fps(now_ms, fps);
    }

    pub fn record_capture_failure(&self) {
        self.capture_failures.fetch_add(1, Ordering::Relaxed);
        self.queue_depth.store(0, Ordering::Relaxed);
    }

    pub fn record_queue_cleared(&self) {
        self.queue_depth.store(0, Ordering::Relaxed);
    }

    pub fn current_fps(&self) -> f64 {
        self.controller.current_fps()
    }

    fn effective_fps(&self) -> f64 {
        let first = self.first_capture_ms.load(Ordering::Relaxed);
        let last = self.last_capture_ms.load(Ordering::Relaxed);
        let captured = self.captured.load(Ordering::Relaxed);
        if first == 0 || last <= first || captured <= 1 {
            return 0.0;
        }
        let elapsed_s = (last - first) as f64 / 1000.0;
        ((captured - 1) as f64 / elapsed_s * 100.0).round() / 100.0
    }

    pub fn snapshot(
        &self,
        audio: &AdaptiveAudioStats,
        extras: VideoTelemetryExtras,
    ) -> SendHealthSnapshot {
        let (tier, reason) = self.controller.tier_and_reason();
        SendHealthSnapshot {
            outgoing_video_queue_depth: self.queue_depth.load(Ordering::Relaxed),
            outgoing_video_queue_capacity: extras.queue_capacity,
            outgoing_video_max_queue_depth: self.max_queue_depth.load(Ordering::Relaxed),
            outgoing_video_frames_produced: self.produced.load(Ordering::Relaxed),
            outgoing_video_frames_accepted: self.accepted.load(Ordering::Relaxed),
            outgoing_video_frames_dropped: self.dropped.load(Ordering::Relaxed),
            outgoing_video_frames_coalesced: self.coalesced.load(Ordering::Relaxed),
            outgoing_video_frames_captured: self.captured.load(Ordering::Relaxed),
            outgoing_video_capture_failures: self.capture_failures.load(Ordering::Relaxed),
            outgoing_video_effective_fps: self.effective_fps(),
            outgoing_video_target_fps: (self.current_fps() * 100.0).round() / 100.0,
            outgoing_video_pacing_target_fps: (extras.pacing_target_fps * 100.0).round() / 100.0,
            outgoing_video_max_queue_age_ms: self.max_queue_age_ms.load(Ordering::Relaxed),
            outgoing_video_max_push_latency_ms: self.max_push_latency_ms.load(Ordering::Relaxed),
            outgoing_video_pacing_mode: extras.pacing_mode,
            outgoing_video_bus_active: extras.bus_active,
            outgoing_audio_buffer_target_ms: audio.target_buffer_ms(),
            outgoing_audio_buffer_max_ms: audio.max_buffer_ms(),
            outgoing_audio_underruns: audio.underruns.load(Ordering::Relaxed),
            outgoing_audio_rebuffers: audio.rebuffers.load(Ordering::Relaxed),
            outgoing_audio_max_frame_gap_ms: audio.max_frame_gap_ms.load(Ordering::Relaxed),
            adaptive_send_tier: tier,
            adaptive_send_reason: reason,
        }
    }
}

pub struct AdaptiveAudioStats {
    max_buffer_ms: AtomicU64,
    target_buffer_ms: AtomicU64,
    underruns: AtomicU64,
    rebuffers: AtomicU64,
    max_frame_gap_ms: AtomicU64,
    last_push_ms: AtomicU64,
    stable_started_ms: AtomicU64,
}

impl AdaptiveAudioStats {
    pub fn new(max_buffer_ms: u32, now_ms: u64) -> Self {
        let max_buffer_ms = clamp_audio_buffer_ms(max_buffer_ms);
        Self {
            max_buffer_ms: AtomicU64::new(max_buffer_ms as u64),
            target_buffer_ms: AtomicU64::new(
                DEFAULT_AUDIO_BUFFER_TARGET_MS.min(max_buffer_ms) as u64
            ),
            underruns: AtomicU64::new(0),
            rebuffers: AtomicU64::new(0),
            max_frame_gap_ms: AtomicU64::new(0),
            last_push_ms: AtomicU64::new(0),
            stable_started_ms: AtomicU64::new(now_ms),
        }
    }

    pub fn reset(&self, max_buffer_ms: u32, now_ms: u64) {
        let max_buffer_ms = clamp_audio_buffer_ms(max_buffer_ms);
        self.max_buffer_ms
            .store(max_buffer_ms as u64, Ordering::Relaxed);
        self.target_buffer_ms.store(
            DEFAULT_AUDIO_BUFFER_TARGET_MS.min(max_buffer_ms) as u64,
            Ordering::Relaxed,
        );
        self.underruns.store(0, Ordering::Relaxed);
        self.rebuffers.store(0, Ordering::Relaxed);
        self.max_frame_gap_ms.store(0, Ordering::Relaxed);
        self.last_push_ms.store(0, Ordering::Relaxed);
        self.stable_started_ms.store(now_ms, Ordering::Relaxed);
    }

    pub fn record_push(&self, now_ms: u64) {
        let Some(previous) = advance_monotonic(&self.last_push_ms, now_ms) else {
            return;
        };
        if previous == 0 {
            self.stable_started_ms.store(now_ms, Ordering::Relaxed);
            return;
        }
        let gap = now_ms - previous;
        update_max(&self.max_frame_gap_ms, gap);
        if gap > AUDIO_REBUFFER_GAP_MS {
            self.rebuffers.fetch_add(1, Ordering::Relaxed);
            if gap > self.target_buffer_ms() as u64 {
                self.underruns.fetch_add(1, Ordering::Relaxed);
            }
            let next = (self.target_buffer_ms() + AUDIO_BUFFER_STEP_MS).min(self.max_buffer_ms());
            self.target_buffer_ms.store(next as u64, Ordering::Relaxed);
            self.stable_started_ms.store(now_ms, Ordering::Relaxed);
            return;
        }
        if gap <= AUDIO_STABLE_GAP_MS {
            let stable_started = self.stable_started_ms.load(Ordering::Relaxed);
            if now_ms.saturating_sub(stable_started) >= 30_000 {
                let current = self.target_buffer_ms();
                let next = current
                    .saturating_sub(AUDIO_BUFFER_STEP_MS)
                    .max(DEFAULT_AUDIO_BUFFER_TARGET_MS.min(self.max_buffer_ms()));
                self.target_buffer_ms.store(next as u64, Ordering::Relaxed);
                self.stable_started_ms.store(now_ms, Ordering::Relaxed);
            }
        } else {
            self.stable_started_ms.store(now_ms, Ordering::Relaxed);
        }
    }

    pub fn max_buffer_ms(&self) -> u32 {
        self.max_buffer_ms.load(Ordering::Relaxed) as u32
    }

    pub fn target_buffer_ms(&self) -> u32 {
        self.target_buffer_ms.load(Ordering::Relaxed) as u32
    }
}

pub fn clamp_audio_buffer_ms(value: u32) -> u32 {
    let clamped = value.clamp(DEFAULT_AUDIO_BUFFER_TARGET_MS, DEFAULT_AUDIO_BUFFER_MAX_MS);
    clamped - (clamped % 10)
}

fn sanitize_fps(value: f64, fallback: f64) -> f64 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn tier_for_fps(requested_fps: f64, current_fps: f64) -> String {
    if current_fps >= requested_fps {
        "full".to_string()
    } else if current_fps >= 30.0 {
        "fps30".to_string()
    } else {
        "fps15".to_string()
    }
}

fn update_max(slot: &AtomicU64, value: u64) {
    let mut current = slot.load(Ordering::Relaxed);
    while value > current {
        match slot.compare_exchange(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(next) => current = next,
        }
    }
}

fn update_min_nonzero(slot: &AtomicU64, value: u64) {
    if value == 0 {
        return;
    }
    let mut current = slot.load(Ordering::Relaxed);
    loop {
        if current != 0 && current <= value {
            return;
        }
        match slot.compare_exchange(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return,
            Err(next) => current = next,
        }
    }
}

fn advance_monotonic(slot: &AtomicU64, value: u64) -> Option<u64> {
    let mut current = slot.load(Ordering::Relaxed);
    loop {
        if current != 0 && value < current {
            return None;
        }
        if value == current {
            return Some(current);
        }
        match slot.compare_exchange(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return Some(current),
            Err(next) => current = next,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn video_controller_ignores_pure_coalescing_jitter() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, true, 0);
        for _ in 0..200 {
            controller.record_enqueue(1_000, true);
        }
        controller.record_enqueue(5_001, false);

        assert_eq!(controller.current_fps(), 60.0);
        assert_eq!(controller.tier_and_reason().0, "full");
        assert_eq!(controller.tier_and_reason().1, "stable");
    }

    #[test]
    fn video_controller_ignores_sustained_coalescing_jitter_across_windows() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, true, 0);
        for _ in 0..200 {
            controller.record_enqueue(1_000, true);
        }
        controller.record_enqueue(5_001, false);
        assert_eq!(controller.current_fps(), 60.0);

        for _ in 0..200 {
            controller.record_enqueue(6_000, true);
        }
        controller.record_enqueue(10_002, false);
        assert_eq!(controller.current_fps(), 60.0);
        assert_eq!(controller.tier_and_reason().1, "stable");
    }

    #[test]
    fn video_controller_degrades_on_encoder_drop_pressure() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, true, 0);
        for _ in 0..100 {
            controller.record_enqueue(1_000, false);
        }
        for _ in 0..20 {
            controller.record_drop(1_000);
        }
        controller.record_enqueue(5_001, false);

        assert_eq!(controller.current_fps(), 30.0);
        assert_eq!(controller.tier_and_reason().1, "encoderDropPressure");
    }

    #[test]
    fn video_controller_continues_degrading_under_sustained_latency_pressure() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, true, 0);
        controller.record_capture(1_000, 80, 1);
        controller.record_capture(5_001, 1, 1);
        assert_eq!(controller.current_fps(), 30.0);

        controller.record_capture(6_000, 80, 1);
        controller.record_capture(10_002, 1, 1);

        assert_eq!(controller.current_fps(), 15.0);
        assert_eq!(controller.tier_and_reason().0, "fps15");
        assert_eq!(controller.tier_and_reason().1, "sendLatencyPressure");
    }

    #[test]
    fn video_controller_degrades_on_latency_pressure_without_coalescing() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, true, 0);
        controller.record_capture(1_000, 80, 5);
        controller.record_capture(5_001, 1, 1);

        assert_eq!(controller.current_fps(), 30.0);
        assert_eq!(controller.tier_and_reason().1, "sendLatencyPressure");
    }

    #[test]
    fn video_controller_degrades_on_encoder_egress_pressure() {
        let controller = AdaptiveVideoController::new(30.0, 15.0, true, 0);
        for _ in 0..60 {
            controller.record_enqueue(1_000, false);
        }
        controller.record_egress_fps(1_000, 7.0);
        controller.record_egress_fps(2_000, 8.0);
        controller.record_egress_fps(4_000, 7.0);

        controller.record_egress_fps(5_001, 7.0);

        assert_eq!(controller.current_fps(), 15.0);
        assert_eq!(controller.tier_and_reason().0, "fps15");
        assert_eq!(controller.tier_and_reason().1, "encoderEgressPressure");
    }

    #[test]
    fn video_controller_preserves_pressure_reason_while_degraded_but_stable() {
        let controller = AdaptiveVideoController::new(30.0, 15.0, true, 0);
        for _ in 0..60 {
            controller.record_enqueue(1_000, false);
        }
        controller.record_egress_fps(1_000, 7.0);
        controller.record_egress_fps(2_000, 8.0);
        controller.record_egress_fps(4_000, 7.0);
        controller.record_egress_fps(5_001, 7.0);

        assert_eq!(controller.current_fps(), 15.0);
        assert_eq!(controller.tier_and_reason().1, "encoderEgressPressure");

        controller.record_capture(10_002, 1, 1);

        assert_eq!(controller.current_fps(), 15.0);
        assert_eq!(controller.tier_and_reason().0, "fps15");
        assert_eq!(controller.tier_and_reason().1, "encoderEgressPressure");

        for index in 2..=12 {
            controller.record_capture(10_002 + index * 5_001, 1, 1);
        }

        assert_eq!(controller.current_fps(), 30.0);
        assert_eq!(
            controller.tier_and_reason(),
            ("full".to_string(), "stable".to_string())
        );
    }

    #[test]
    fn video_controller_ignores_single_encoder_egress_sample() {
        let controller = AdaptiveVideoController::new(30.0, 15.0, true, 0);
        for _ in 0..60 {
            controller.record_enqueue(1_000, false);
        }
        controller.record_egress_fps(1_000, 7.0);

        controller.record_egress_fps(5_001, 7.0);

        assert_eq!(controller.current_fps(), 30.0);
        assert_eq!(controller.tier_and_reason().1, "stable");
    }

    #[test]
    fn video_controller_keeps_requested_fps_when_adaptive_send_is_disabled() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, false, 0);
        for _ in 0..100 {
            controller.record_enqueue(1_000, false);
        }
        controller.record_capture(5_001, 200, 200);

        assert_eq!(controller.current_fps(), 60.0);
        assert_eq!(
            controller.tier_and_reason(),
            ("full".to_string(), "adaptiveDisabled".to_string())
        );
    }

    #[test]
    fn video_controller_recovers_after_stable_windows() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, true, 0);
        controller.record_capture(1_000, 80, 1);
        controller.record_capture(5_001, 1, 1);
        assert_eq!(controller.current_fps(), 30.0);

        for index in 1..=12 {
            controller.record_capture(5_001 + index * 5_001, 1, 1);
        }
        assert_eq!(controller.current_fps(), 60.0);
    }

    #[test]
    fn video_controller_recovers_from_minimum_in_two_stable_steps() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, true, 0);
        controller.record_capture(1_000, 80, 1);
        controller.record_capture(5_001, 1, 1);
        assert_eq!(controller.current_fps(), 30.0);

        controller.record_capture(6_000, 70, 1);
        controller.record_capture(10_002, 1, 1);
        assert_eq!(controller.current_fps(), 15.0);

        for index in 1..=12 {
            controller.record_capture(10_002 + index * 5_001, 1, 1);
        }
        assert_eq!(controller.current_fps(), 30.0);

        for index in 13..=24 {
            controller.record_capture(10_002 + index * 5_001, 1, 1);
        }
        assert_eq!(controller.current_fps(), 60.0);
    }

    #[test]
    fn video_controller_requires_a_full_stable_recovery_window() {
        let controller = AdaptiveVideoController::new(60.0, 15.0, true, 0);
        controller.record_capture(1_000, 80, 1);
        controller.record_capture(5_001, 1, 1);
        assert_eq!(controller.current_fps(), 30.0);

        for index in 1..12 {
            controller.record_capture(5_001 + index * 5_001, 1, 1);
        }
        assert_eq!(controller.current_fps(), 30.0);
        controller.record_capture(5_001 + 12 * 5_001, 1, 1);
        assert_eq!(controller.current_fps(), 60.0);
    }

    #[test]
    fn video_stats_snapshot_counts_coalescing_drops_failures_and_effective_fps() {
        let audio = AdaptiveAudioStats::new(750, 0);
        let stats = AdaptiveVideoStats::new(60.0, 15.0, true, 0);

        stats.record_enqueue(1_000, false);
        stats.record_enqueue(1_010, true);
        stats.record_drop(1_011);
        stats.record_capture(1_020, 20, 24);
        stats.record_enqueue(2_000, false);
        stats.record_capture(2_020, 20, 26);
        stats.record_capture_failure();

        let snapshot = stats.snapshot(&audio, VideoTelemetryExtras::default());
        assert_eq!(snapshot.outgoing_video_frames_produced, 3);
        assert_eq!(snapshot.outgoing_video_frames_accepted, 3);
        assert_eq!(snapshot.outgoing_video_frames_dropped, 1);
        assert_eq!(snapshot.outgoing_video_frames_coalesced, 1);
        assert_eq!(snapshot.outgoing_video_frames_captured, 2);
        assert_eq!(snapshot.outgoing_video_capture_failures, 1);
        assert_eq!(snapshot.outgoing_video_effective_fps, 1.0);
        assert_eq!(snapshot.outgoing_video_max_queue_age_ms, 20);
        assert_eq!(snapshot.outgoing_video_max_push_latency_ms, 26);
        assert_eq!(snapshot.outgoing_video_queue_depth, 0);
    }

    #[test]
    fn video_stats_effective_fps_uses_capture_time_bounds_for_out_of_order_records() {
        let audio = AdaptiveAudioStats::new(750, 0);
        let stats = AdaptiveVideoStats::new(60.0, 15.0, true, 0);

        stats.record_capture(2_000, 5, 6);
        stats.record_capture(1_000, 7, 8);
        stats.record_capture(3_000, 9, 10);
        stats.record_capture(2_500, 11, 12);

        let snapshot = stats.snapshot(&audio, VideoTelemetryExtras::default());
        assert_eq!(snapshot.outgoing_video_frames_captured, 4);
        assert_eq!(snapshot.outgoing_video_effective_fps, 1.5);
        assert_eq!(snapshot.outgoing_video_max_queue_age_ms, 11);
        assert_eq!(snapshot.outgoing_video_max_push_latency_ms, 12);
    }

    #[test]
    fn video_stats_handles_concurrent_recording_without_lost_counts() {
        let audio = AdaptiveAudioStats::new(750, 0);
        let stats = Arc::new(AdaptiveVideoStats::new(60.0, 15.0, true, 0));
        let mut workers = Vec::new();
        for worker in 0..8 {
            let stats = stats.clone();
            workers.push(std::thread::spawn(move || {
                for index in 0..250 {
                    let now_ms = 1_000 + worker * 1_000 + index;
                    stats.record_enqueue(now_ms, index % 3 == 0);
                    if index % 5 == 0 {
                        stats.record_drop(now_ms);
                    }
                    if index % 7 == 0 {
                        stats.record_capture(now_ms + 1, 1, 2);
                    }
                }
            }));
        }
        for worker in workers {
            worker.join().expect("worker should not panic");
        }

        let snapshot = stats.snapshot(&audio, VideoTelemetryExtras::default());
        assert_eq!(snapshot.outgoing_video_frames_produced, 2_000);
        assert_eq!(snapshot.outgoing_video_frames_accepted, 2_000);
        assert_eq!(snapshot.outgoing_video_frames_coalesced, 672);
        assert_eq!(snapshot.outgoing_video_frames_dropped, 400);
        assert_eq!(snapshot.outgoing_video_frames_captured, 288);
        assert_eq!(snapshot.outgoing_video_max_queue_age_ms, 1);
        assert_eq!(snapshot.outgoing_video_max_push_latency_ms, 2);
    }

    #[test]
    fn idle_snapshot_reflects_audio_pressure_without_video_state() {
        let audio = AdaptiveAudioStats::new(750, 0);
        audio.record_push(1_000);
        audio.record_push(1_300);

        let snapshot = SendHealthSnapshot::idle(&audio);
        assert_eq!(snapshot.outgoing_video_frames_produced, 0);
        assert_eq!(snapshot.outgoing_audio_buffer_target_ms, 300);
        assert_eq!(snapshot.outgoing_audio_buffer_max_ms, 750);
        assert_eq!(snapshot.outgoing_audio_rebuffers, 1);
        assert_eq!(snapshot.outgoing_audio_underruns, 1);
        assert_eq!(snapshot.outgoing_audio_max_frame_gap_ms, 300);
        assert_eq!(snapshot.adaptive_send_tier, "idle");
        assert_eq!(snapshot.adaptive_send_reason, "notPublishing");
    }

    #[test]
    fn video_controller_stress_keeps_target_inside_configured_bounds() {
        let controller = AdaptiveVideoController::new(144.0, 24.0, true, 0);
        for window in 0..240 {
            let base = window * 5_001;
            match window % 4 {
                0 => controller.record_capture(base + 1_000, 200, 1),
                1 => {
                    for _ in 0..30 {
                        controller.record_enqueue(base + 1_000, true);
                    }
                    controller.record_capture(base + 1_500, 1, 1);
                }
                2 => {
                    for _ in 0..60 {
                        controller.record_enqueue(base + 1_000, false);
                    }
                    controller.record_egress_fps(base + 1_500, 30.0);
                    controller.record_egress_fps(base + 2_500, 30.0);
                }
                _ => controller.record_capture(base + 1_000, 1, 1),
            }
            controller.record_capture(base + 5_001, 1, 1);
            let fps = controller.current_fps();
            assert!(
                (24.0..=144.0).contains(&fps),
                "fps target escaped configured bounds: {fps}"
            );
        }
    }

    #[test]
    fn audio_buffer_expands_on_gaps_and_shrinks_after_stability() {
        let audio = AdaptiveAudioStats::new(750, 0);
        audio.record_push(10);
        audio.record_push(200);
        assert_eq!(audio.target_buffer_ms(), 300);
        assert_eq!(audio.rebuffers.load(Ordering::Relaxed), 1);

        for now_ms in (220..=30_240).step_by(20) {
            audio.record_push(now_ms);
        }
        assert_eq!(audio.target_buffer_ms(), 200);
    }

    #[test]
    fn audio_buffer_ignores_clock_regression_without_false_rebuffer() {
        let audio = AdaptiveAudioStats::new(750, 0);

        audio.record_push(1_000);
        audio.record_push(1_020);
        audio.record_push(900);
        audio.record_push(1_040);

        assert_eq!(audio.target_buffer_ms(), 200);
        assert_eq!(audio.rebuffers.load(Ordering::Relaxed), 0);
        assert_eq!(audio.underruns.load(Ordering::Relaxed), 0);
        assert_eq!(audio.max_frame_gap_ms.load(Ordering::Relaxed), 20);
    }

    #[test]
    fn audio_buffer_ignores_concurrent_stale_pushes_without_moving_last_push_backwards() {
        let audio = Arc::new(AdaptiveAudioStats::new(750, 0));
        audio.record_push(10_000);

        let mut workers = Vec::new();
        for worker in 0..8 {
            let audio = audio.clone();
            workers.push(std::thread::spawn(move || {
                for index in 0..100 {
                    audio.record_push(1_000 + worker * 100 + index);
                }
            }));
        }
        for worker in workers {
            worker.join().expect("worker should not panic");
        }

        audio.record_push(10_020);

        assert_eq!(audio.target_buffer_ms(), 200);
        assert_eq!(audio.rebuffers.load(Ordering::Relaxed), 0);
        assert_eq!(audio.underruns.load(Ordering::Relaxed), 0);
        assert_eq!(audio.max_frame_gap_ms.load(Ordering::Relaxed), 20);
    }

    #[test]
    fn audio_buffer_growth_caps_at_configured_max_and_reset_clears_pressure() {
        let audio = AdaptiveAudioStats::new(350, 0);
        audio.record_push(10);
        for index in 1..=10 {
            audio.record_push(10 + index * 500);
        }
        assert_eq!(audio.target_buffer_ms(), 350);
        assert_eq!(audio.rebuffers.load(Ordering::Relaxed), 10);
        assert_eq!(audio.underruns.load(Ordering::Relaxed), 10);

        audio.reset(250, 10_000);
        assert_eq!(audio.target_buffer_ms(), 200);
        assert_eq!(audio.max_buffer_ms(), 250);
        assert_eq!(audio.rebuffers.load(Ordering::Relaxed), 0);
        assert_eq!(audio.underruns.load(Ordering::Relaxed), 0);
        assert_eq!(audio.max_frame_gap_ms.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn audio_buffer_stress_stays_within_realtime_bounds_under_jitter() {
        let audio = AdaptiveAudioStats::new(620, 0);
        let mut now_ms = 10;
        audio.record_push(now_ms);

        for index in 0..5_000 {
            now_ms += match index % 11 {
                0 => 180,
                1 | 2 => 80,
                _ => 20,
            };
            audio.record_push(now_ms);
            assert!(
                (DEFAULT_AUDIO_BUFFER_TARGET_MS..=620).contains(&audio.target_buffer_ms()),
                "audio target escaped configured bounds"
            );
        }

        assert_eq!(audio.max_buffer_ms(), 620);
        assert!(audio.rebuffers.load(Ordering::Relaxed) > 0);
        assert!(audio.max_frame_gap_ms.load(Ordering::Relaxed) >= 180);
    }

    #[test]
    fn audio_buffer_max_is_clamped_to_real_time_bounds() {
        assert_eq!(clamp_audio_buffer_ms(50), 200);
        assert_eq!(clamp_audio_buffer_ms(777), 750);
        assert_eq!(clamp_audio_buffer_ms(333), 330);
    }
}
