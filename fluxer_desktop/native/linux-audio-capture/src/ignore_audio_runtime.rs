// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(dead_code)]
use std::collections::BTreeMap;

pub const AUDIO_BUFFERING_MAX_TICKS: u32 = 64;

pub const SOURCE_RESET_AFTER_BUFFERED_TICKS: u32 = 128;

pub const SOURCE_STALE_AFTER_NS: u64 = 5_000_000_000;

pub const NEVER_PUSHED_SENTINEL: u64 = u64::MAX;

pub fn compute_source_age_ns(last_push_ns: u64, registered_at_ns: u64, now_ns: u64) -> u64 {
    let baseline_ns = if last_push_ns == NEVER_PUSHED_SENTINEL {
        registered_at_ns
    } else {
        last_push_ns
    };
    assert!(baseline_ns != NEVER_PUSHED_SENTINEL);
    if now_ns <= baseline_ns {
        return 0;
    }
    now_ns - baseline_ns
}

pub const SAMPLE_RATE_HZ_MIN: u32 = 8_000;
pub const SAMPLE_RATE_HZ_MAX: u32 = 384_000;

pub const TICK_PERIOD_NS_MIN: u64 = 1_000_000;
pub const TICK_PERIOD_NS_MAX: u64 = 100_000_000;

pub const BUFFERED_FRAMES_MAX: u64 = 1 << 28;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreAudioDecision {
    Mix,
    IgnoreThisTick,
    ResetSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IgnoreAudioResetReason {
    BufferOverflow,
    StaleSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IgnoreAudioSourceState {
    pub id: u64,
    pub buffered_frames: u64,
    pub last_frame_age_ns: u64,
    pub is_muted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IgnoreAudioTick {
    pub at_ns: u64,
    pub period_ns: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IgnoreAudioMetrics {
    pub ignored_tick_count: u64,
    pub reset_count: u64,
}

impl IgnoreAudioMetrics {
    pub const ZERO: IgnoreAudioMetrics = IgnoreAudioMetrics {
        ignored_tick_count: 0,
        reset_count: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IgnoreAudioSourceResetEvent {
    pub source_id: u64,
    pub at_ns: u64,
    pub buffered_frames_at_reset: u64,
    pub last_frame_age_ns: u64,
    pub reason: IgnoreAudioResetReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IgnoreAudioEvaluation {
    pub decision: IgnoreAudioDecision,
    pub event: Option<IgnoreAudioSourceResetEvent>,
}

#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub enum IgnoreAudioError {
    SampleRateOutOfRange { sample_rate_hz: u32 },
    TickPeriodOutOfRange { period_ns: u64 },
    BufferedFramesOutOfRange { buffered_frames: u64 },
}

pub struct IgnoreAudioPolicy {
    sample_rate_hz: u32,
    metrics_by_source: BTreeMap<u64, IgnoreAudioMetrics>,
}

impl IgnoreAudioPolicy {
    pub fn new(sample_rate_hz: u32) -> Result<Self, IgnoreAudioError> {
        if !(SAMPLE_RATE_HZ_MIN..=SAMPLE_RATE_HZ_MAX).contains(&sample_rate_hz) {
            return Err(IgnoreAudioError::SampleRateOutOfRange { sample_rate_hz });
        }
        assert!(sample_rate_hz >= SAMPLE_RATE_HZ_MIN);
        assert!(sample_rate_hz <= SAMPLE_RATE_HZ_MAX);
        Ok(Self {
            sample_rate_hz,
            metrics_by_source: BTreeMap::new(),
        })
    }

    pub fn sample_rate_hz(&self) -> u32 {
        assert!(self.sample_rate_hz >= SAMPLE_RATE_HZ_MIN);
        assert!(self.sample_rate_hz <= SAMPLE_RATE_HZ_MAX);
        self.sample_rate_hz
    }

    pub fn metrics_for(&self, source_id: u64) -> IgnoreAudioMetrics {
        assert!(self.sample_rate_hz >= SAMPLE_RATE_HZ_MIN);
        self.metrics_by_source
            .get(&source_id)
            .copied()
            .unwrap_or(IgnoreAudioMetrics::ZERO)
    }

    pub fn evaluate(
        &mut self,
        source_state: &IgnoreAudioSourceState,
        tick: IgnoreAudioTick,
    ) -> Result<IgnoreAudioEvaluation, IgnoreAudioError> {
        validate_source_state(source_state)?;
        validate_tick(tick)?;
        let tick_frames = compute_tick_frames(tick.period_ns, self.sample_rate_hz)?;
        assert!(tick_frames >= 1);
        let decision = decide(source_state, tick_frames);
        let event = self.apply_decision(source_state, tick, decision);
        let evaluation = IgnoreAudioEvaluation { decision, event };
        assert_evaluation_invariant(&evaluation);
        Ok(evaluation)
    }

    fn apply_decision(
        &mut self,
        source_state: &IgnoreAudioSourceState,
        tick: IgnoreAudioTick,
        decision: IgnoreAudioDecision,
    ) -> Option<IgnoreAudioSourceResetEvent> {
        assert!(self.sample_rate_hz >= SAMPLE_RATE_HZ_MIN);
        match decision {
            IgnoreAudioDecision::Mix => None,
            IgnoreAudioDecision::IgnoreThisTick => {
                let entry = self
                    .metrics_by_source
                    .entry(source_state.id)
                    .or_insert(IgnoreAudioMetrics::ZERO);
                entry.ignored_tick_count = entry.ignored_tick_count.saturating_add(1);
                None
            }
            IgnoreAudioDecision::ResetSource => {
                let entry = self
                    .metrics_by_source
                    .entry(source_state.id)
                    .or_insert(IgnoreAudioMetrics::ZERO);
                entry.reset_count = entry.reset_count.saturating_add(1);
                let reason = if source_state.last_frame_age_ns > SOURCE_STALE_AFTER_NS {
                    IgnoreAudioResetReason::StaleSource
                } else {
                    IgnoreAudioResetReason::BufferOverflow
                };
                Some(IgnoreAudioSourceResetEvent {
                    source_id: source_state.id,
                    at_ns: tick.at_ns,
                    buffered_frames_at_reset: source_state.buffered_frames,
                    last_frame_age_ns: source_state.last_frame_age_ns,
                    reason,
                })
            }
        }
    }
}

pub fn compute_tick_frames(period_ns: u64, sample_rate_hz: u32) -> Result<u64, IgnoreAudioError> {
    if !(TICK_PERIOD_NS_MIN..=TICK_PERIOD_NS_MAX).contains(&period_ns) {
        return Err(IgnoreAudioError::TickPeriodOutOfRange { period_ns });
    }
    if !(SAMPLE_RATE_HZ_MIN..=SAMPLE_RATE_HZ_MAX).contains(&sample_rate_hz) {
        return Err(IgnoreAudioError::SampleRateOutOfRange { sample_rate_hz });
    }
    let product: u128 = (period_ns as u128) * (sample_rate_hz as u128);
    let frames = (product / 1_000_000_000u128) as u64;
    let frames = frames.max(1);
    assert!(frames >= 1);
    assert!(frames <= BUFFERED_FRAMES_MAX);
    Ok(frames)
}

fn decide(source_state: &IgnoreAudioSourceState, tick_frames: u64) -> IgnoreAudioDecision {
    assert!(tick_frames >= 1);
    if source_state.is_muted {
        return IgnoreAudioDecision::Mix;
    }
    let reset_by_stale = source_state.last_frame_age_ns > SOURCE_STALE_AFTER_NS;
    let reset_by_buffer =
        source_state.buffered_frames > (SOURCE_RESET_AFTER_BUFFERED_TICKS as u64) * tick_frames;
    if reset_by_stale {
        return IgnoreAudioDecision::ResetSource;
    }
    if reset_by_buffer {
        return IgnoreAudioDecision::ResetSource;
    }
    if source_state.buffered_frames <= tick_frames {
        return IgnoreAudioDecision::Mix;
    }
    let over_buffering =
        source_state.buffered_frames > (AUDIO_BUFFERING_MAX_TICKS as u64) * tick_frames;
    if over_buffering {
        return IgnoreAudioDecision::IgnoreThisTick;
    }
    IgnoreAudioDecision::Mix
}

fn validate_source_state(state: &IgnoreAudioSourceState) -> Result<(), IgnoreAudioError> {
    if state.buffered_frames > BUFFERED_FRAMES_MAX {
        return Err(IgnoreAudioError::BufferedFramesOutOfRange {
            buffered_frames: state.buffered_frames,
        });
    }
    Ok(())
}

fn validate_tick(tick: IgnoreAudioTick) -> Result<(), IgnoreAudioError> {
    if !(TICK_PERIOD_NS_MIN..=TICK_PERIOD_NS_MAX).contains(&tick.period_ns) {
        return Err(IgnoreAudioError::TickPeriodOutOfRange {
            period_ns: tick.period_ns,
        });
    }
    Ok(())
}

fn assert_evaluation_invariant(evaluation: &IgnoreAudioEvaluation) {
    match evaluation.decision {
        IgnoreAudioDecision::ResetSource => {
            assert!(evaluation.event.is_some());
        }
        _ => {
            assert!(evaluation.event.is_none());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_tick(at_ns: u64) -> IgnoreAudioTick {
        IgnoreAudioTick {
            at_ns,
            period_ns: 21_333_333,
        }
    }

    #[test]
    fn rejects_sample_rate_below_min() {
        let err = IgnoreAudioPolicy::new(4_000).err();
        assert!(matches!(
            err,
            Some(IgnoreAudioError::SampleRateOutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_sample_rate_above_max() {
        let err = IgnoreAudioPolicy::new(500_000).err();
        assert!(matches!(
            err,
            Some(IgnoreAudioError::SampleRateOutOfRange { .. })
        ));
    }

    #[test]
    fn compute_tick_frames_at_48k_21_3ms_yields_1023_ish() {
        let frames = compute_tick_frames(21_333_333, 48_000).expect("ok");
        assert!(frames >= 1023);
        assert!(frames <= 1024);
    }

    #[test]
    fn empty_buffer_yields_mix_decision() {
        let mut policy = IgnoreAudioPolicy::new(48_000).expect("ok");
        let state = IgnoreAudioSourceState {
            id: 1,
            buffered_frames: 0,
            last_frame_age_ns: 0,
            is_muted: false,
        };
        let evaluation = policy.evaluate(&state, canonical_tick(0)).expect("ok");
        assert_eq!(evaluation.decision, IgnoreAudioDecision::Mix);
        assert!(evaluation.event.is_none());
    }

    #[test]
    fn muted_source_always_yields_mix_decision() {
        let mut policy = IgnoreAudioPolicy::new(48_000).expect("ok");
        let state = IgnoreAudioSourceState {
            id: 5,
            buffered_frames: 10_000_000,
            last_frame_age_ns: 0,
            is_muted: true,
        };
        let evaluation = policy.evaluate(&state, canonical_tick(0)).expect("ok");
        assert_eq!(evaluation.decision, IgnoreAudioDecision::Mix);
    }

    #[test]
    fn over_buffered_yields_ignore_this_tick() {
        let mut policy = IgnoreAudioPolicy::new(48_000).expect("ok");
        let state = IgnoreAudioSourceState {
            id: 9,
            buffered_frames: (AUDIO_BUFFERING_MAX_TICKS as u64) * 1024 + 1,
            last_frame_age_ns: 0,
            is_muted: false,
        };
        let evaluation = policy.evaluate(&state, canonical_tick(0)).expect("ok");
        assert_eq!(evaluation.decision, IgnoreAudioDecision::IgnoreThisTick);
        assert_eq!(policy.metrics_for(9).ignored_tick_count, 1);
    }

    #[test]
    fn over_threshold_buffer_triggers_reset() {
        let mut policy = IgnoreAudioPolicy::new(48_000).expect("ok");
        let state = IgnoreAudioSourceState {
            id: 12,
            buffered_frames: (SOURCE_RESET_AFTER_BUFFERED_TICKS as u64) * 1024 + 1,
            last_frame_age_ns: 0,
            is_muted: false,
        };
        let evaluation = policy.evaluate(&state, canonical_tick(7)).expect("ok");
        assert_eq!(evaluation.decision, IgnoreAudioDecision::ResetSource);
        let event = evaluation.event.expect("reset emits event");
        assert_eq!(event.source_id, 12);
        assert_eq!(event.at_ns, 7);
        assert_eq!(event.reason, IgnoreAudioResetReason::BufferOverflow);
        assert_eq!(policy.metrics_for(12).reset_count, 1);
    }

    #[test]
    fn stale_source_triggers_reset_with_stale_reason() {
        let mut policy = IgnoreAudioPolicy::new(48_000).expect("ok");
        let state = IgnoreAudioSourceState {
            id: 1,
            buffered_frames: 0,
            last_frame_age_ns: SOURCE_STALE_AFTER_NS + 1,
            is_muted: false,
        };
        let evaluation = policy.evaluate(&state, canonical_tick(0)).expect("ok");
        assert_eq!(evaluation.decision, IgnoreAudioDecision::ResetSource);
        let event = evaluation.event.expect("reset emits event");
        assert_eq!(event.reason, IgnoreAudioResetReason::StaleSource);
    }

    #[test]
    fn period_too_small_rejected() {
        let mut policy = IgnoreAudioPolicy::new(48_000).expect("ok");
        let state = IgnoreAudioSourceState {
            id: 1,
            buffered_frames: 0,
            last_frame_age_ns: 0,
            is_muted: false,
        };
        let err = policy
            .evaluate(
                &state,
                IgnoreAudioTick {
                    at_ns: 0,
                    period_ns: 0,
                },
            )
            .err();
        assert!(matches!(
            err,
            Some(IgnoreAudioError::TickPeriodOutOfRange { .. })
        ));
    }

    #[test]
    fn compute_source_age_uses_registration_for_never_pushed() {
        let registered_at_ns = 1_000_000;
        let now_ns = 1_000_000 + 6_000_000_000;
        let age = compute_source_age_ns(NEVER_PUSHED_SENTINEL, registered_at_ns, now_ns);
        assert_eq!(age, 6_000_000_000);
    }

    #[test]
    fn compute_source_age_uses_last_push_after_first_push() {
        let registered_at_ns = 1_000;
        let last_push_ns = 5_000;
        let age = compute_source_age_ns(last_push_ns, registered_at_ns, 9_000);
        assert_eq!(age, 4_000);
    }

    #[test]
    fn compute_source_age_zero_when_now_before_baseline() {
        let age = compute_source_age_ns(NEVER_PUSHED_SENTINEL, 5_000, 1_000);
        assert_eq!(age, 0);
    }

    #[test]
    fn metrics_accumulate_across_calls() {
        let mut policy = IgnoreAudioPolicy::new(48_000).expect("ok");
        let state = IgnoreAudioSourceState {
            id: 2,
            buffered_frames: (AUDIO_BUFFERING_MAX_TICKS as u64) * 1024 + 1,
            last_frame_age_ns: 0,
            is_muted: false,
        };
        for _ in 0..5 {
            let _ = policy.evaluate(&state, canonical_tick(0));
        }
        assert_eq!(policy.metrics_for(2).ignored_tick_count, 5);
    }
}
