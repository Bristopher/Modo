// SPDX-License-Identifier: AGPL-3.0-or-later

pub const TS_SMOOTHING_THRESHOLD_NS: u64 = 70_000_000;

pub const MAX_TS_VAR_NS: u64 = 2_000_000_000;

pub const NS_PER_SECOND: u64 = 1_000_000_000;

const SAMPLE_RATE_HZ_MIN: u32 = 8_000;

const SAMPLE_RATE_HZ_MAX: u32 = 384_000;

const FRAMES_PER_INPUT_MAX: u32 = 1 << 24;

#[derive(Debug, PartialEq, Eq)]
pub enum AudioTimingError {
    SampleRateOutOfRange { sample_rate_hz: u32 },
    SourceIdZero,
    FramesOutOfRange { frames: u32 },
}

impl core::fmt::Display for AudioTimingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AudioTimingError::SampleRateOutOfRange { sample_rate_hz } => {
                write!(f, "sample rate {sample_rate_hz} hz outside accepted range")
            }
            AudioTimingError::SourceIdZero => write!(f, "source_id must be non-zero"),
            AudioTimingError::FramesOutOfRange { frames } => {
                write!(f, "frames {frames} outside accepted range")
            }
        }
    }
}

impl std::error::Error for AudioTimingError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmootherInput {
    pub media_ts_ns: u64,
    pub frames: u32,
    pub wall_ns: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmootherAction {
    Snap,
    Jump,
    Reset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmootherResetReason {
    FirstSample,
    GapBeyondMaxVar,
    Regression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmootherEvent {
    Snapped {
        source_id: u64,
        incoming_media_ts_ns: u64,
        snapped_ts_ns: u64,
        delta_ns: i64,
        frames_accumulated: u64,
    },
    Jumped {
        source_id: u64,
        incoming_media_ts_ns: u64,
        previous_predicted_ts_ns: u64,
        delta_ns: i64,
        frames_accumulated: u64,
    },
    Reset {
        source_id: u64,
        incoming_media_ts_ns: u64,
        wall_ns: u64,
        timing_adjust_ns: i64,
        frames_accumulated: u64,
        reason: SmootherResetReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmootherResult {
    pub action: SmootherAction,
    pub output_ts_ns: u64,
    pub predicted_next_ts_ns: u64,
    pub frames_accumulated: u64,
    pub event: SmootherEvent,
}

pub struct AudioTimingSmoother {
    source_id: u64,
    sample_rate_hz: u32,
    initialised: bool,
    predicted_next_ts_ns: u64,
    timing_adjust_ns: i64,
    frames_accumulated: u64,
    consecutive_jumps: u32,
}

impl AudioTimingSmoother {
    pub fn new(source_id: u64, sample_rate_hz: u32) -> Result<Self, AudioTimingError> {
        if source_id == 0 {
            return Err(AudioTimingError::SourceIdZero);
        }
        if !(SAMPLE_RATE_HZ_MIN..=SAMPLE_RATE_HZ_MAX).contains(&sample_rate_hz) {
            return Err(AudioTimingError::SampleRateOutOfRange { sample_rate_hz });
        }
        assert!(source_id != 0);
        assert!(sample_rate_hz >= SAMPLE_RATE_HZ_MIN);
        assert!(sample_rate_hz <= SAMPLE_RATE_HZ_MAX);
        Ok(Self {
            source_id,
            sample_rate_hz,
            initialised: false,
            predicted_next_ts_ns: 0,
            timing_adjust_ns: 0,
            frames_accumulated: 0,
            consecutive_jumps: 0,
        })
    }

    pub fn source_id(&self) -> u64 {
        assert!(self.source_id != 0);
        self.source_id
    }

    pub fn sample_rate_hz(&self) -> u32 {
        assert!(self.sample_rate_hz >= SAMPLE_RATE_HZ_MIN);
        assert!(self.sample_rate_hz <= SAMPLE_RATE_HZ_MAX);
        self.sample_rate_hz
    }

    pub fn initialised(&self) -> bool {
        self.initialised
    }

    pub fn predicted_next_ts_ns(&self) -> u64 {
        self.predicted_next_ts_ns
    }

    pub fn timing_adjust_ns(&self) -> i64 {
        self.timing_adjust_ns
    }

    pub fn frames_accumulated(&self) -> u64 {
        self.frames_accumulated
    }

    pub fn consecutive_jumps(&self) -> u32 {
        self.consecutive_jumps
    }

    pub fn adjust(&mut self, input: SmootherInput) -> Result<SmootherResult, AudioTimingError> {
        if input.frames == 0 || input.frames > FRAMES_PER_INPUT_MAX {
            return Err(AudioTimingError::FramesOutOfRange {
                frames: input.frames,
            });
        }
        assert!(input.frames >= 1);
        assert!(input.frames <= FRAMES_PER_INPUT_MAX);
        if !self.initialised {
            return Ok(self.initialise_from_first_sample(input));
        }
        Ok(self.adjust_initialised(input))
    }

    fn initialise_from_first_sample(&mut self, input: SmootherInput) -> SmootherResult {
        assert!(!self.initialised);
        assert_eq!(self.frames_accumulated, 0);
        let increment_ns = frames_to_ns(input.frames as u64, self.sample_rate_hz);
        let predicted_next = input.media_ts_ns.saturating_add(increment_ns);
        let timing_adjust = wall_minus_media_signed(input.wall_ns, input.media_ts_ns);
        self.frames_accumulated = input.frames as u64;
        self.predicted_next_ts_ns = predicted_next;
        self.timing_adjust_ns = timing_adjust;
        self.initialised = true;
        self.consecutive_jumps = 0;
        assert!(predicted_next >= input.media_ts_ns);
        let event = SmootherEvent::Reset {
            source_id: self.source_id,
            incoming_media_ts_ns: input.media_ts_ns,
            wall_ns: input.wall_ns,
            timing_adjust_ns: timing_adjust,
            frames_accumulated: self.frames_accumulated,
            reason: SmootherResetReason::FirstSample,
        };
        SmootherResult {
            action: SmootherAction::Reset,
            output_ts_ns: input.media_ts_ns,
            predicted_next_ts_ns: predicted_next,
            frames_accumulated: self.frames_accumulated,
            event,
        }
    }

    fn adjust_initialised(&mut self, input: SmootherInput) -> SmootherResult {
        assert!(self.initialised);
        let expected = self.predicted_next_ts_ns;
        let delta = signed_delta(input.media_ts_ns, expected);
        let absolute_delta = delta.unsigned_abs();
        if absolute_delta <= TS_SMOOTHING_THRESHOLD_NS {
            return self.apply_snap(input, expected, delta);
        }
        if absolute_delta <= MAX_TS_VAR_NS {
            return self.apply_jump(input, expected, delta);
        }
        self.apply_full_reset(input, SmootherResetReason::GapBeyondMaxVar)
    }

    fn apply_snap(&mut self, input: SmootherInput, expected: u64, delta: i64) -> SmootherResult {
        assert!(self.initialised);
        let increment_ns = frames_to_ns(input.frames as u64, self.sample_rate_hz);
        let new_predicted = expected.saturating_add(increment_ns);
        self.frames_accumulated = self.frames_accumulated.saturating_add(input.frames as u64);
        self.predicted_next_ts_ns = new_predicted;
        self.consecutive_jumps = 0;
        assert!(new_predicted >= expected);
        let event = SmootherEvent::Snapped {
            source_id: self.source_id,
            incoming_media_ts_ns: input.media_ts_ns,
            snapped_ts_ns: expected,
            delta_ns: delta,
            frames_accumulated: self.frames_accumulated,
        };
        SmootherResult {
            action: SmootherAction::Snap,
            output_ts_ns: expected,
            predicted_next_ts_ns: new_predicted,
            frames_accumulated: self.frames_accumulated,
            event,
        }
    }

    fn apply_jump(&mut self, input: SmootherInput, expected: u64, delta: i64) -> SmootherResult {
        assert!(self.initialised);
        if delta < 0 {
            return self.apply_full_reset(input, SmootherResetReason::Regression);
        }
        let increment_ns = frames_to_ns(input.frames as u64, self.sample_rate_hz);
        let new_predicted = input.media_ts_ns.saturating_add(increment_ns);
        self.frames_accumulated = self.frames_accumulated.saturating_add(input.frames as u64);
        self.predicted_next_ts_ns = new_predicted;
        self.consecutive_jumps = self.consecutive_jumps.saturating_add(1);
        assert!(new_predicted >= input.media_ts_ns);
        let event = SmootherEvent::Jumped {
            source_id: self.source_id,
            incoming_media_ts_ns: input.media_ts_ns,
            previous_predicted_ts_ns: expected,
            delta_ns: delta,
            frames_accumulated: self.frames_accumulated,
        };
        SmootherResult {
            action: SmootherAction::Jump,
            output_ts_ns: input.media_ts_ns,
            predicted_next_ts_ns: new_predicted,
            frames_accumulated: self.frames_accumulated,
            event,
        }
    }

    fn apply_full_reset(
        &mut self,
        input: SmootherInput,
        reason: SmootherResetReason,
    ) -> SmootherResult {
        assert!(self.initialised);
        let increment_ns = frames_to_ns(input.frames as u64, self.sample_rate_hz);
        let new_predicted = input.media_ts_ns.saturating_add(increment_ns);
        let timing_adjust = wall_minus_media_signed(input.wall_ns, input.media_ts_ns);
        self.frames_accumulated = self.frames_accumulated.saturating_add(input.frames as u64);
        self.predicted_next_ts_ns = new_predicted;
        self.timing_adjust_ns = timing_adjust;
        self.consecutive_jumps = 0;
        assert!(new_predicted >= input.media_ts_ns);
        let event = SmootherEvent::Reset {
            source_id: self.source_id,
            incoming_media_ts_ns: input.media_ts_ns,
            wall_ns: input.wall_ns,
            timing_adjust_ns: timing_adjust,
            frames_accumulated: self.frames_accumulated,
            reason,
        };
        SmootherResult {
            action: SmootherAction::Reset,
            output_ts_ns: input.media_ts_ns,
            predicted_next_ts_ns: new_predicted,
            frames_accumulated: self.frames_accumulated,
            event,
        }
    }
}

pub fn frames_to_ns(frames: u64, sample_rate_hz: u32) -> u64 {
    assert!(sample_rate_hz >= SAMPLE_RATE_HZ_MIN);
    assert!(sample_rate_hz <= SAMPLE_RATE_HZ_MAX);
    let product = (frames as u128) * (NS_PER_SECOND as u128);
    let div = product / (sample_rate_hz as u128);
    assert!(div <= u64::MAX as u128);
    div as u64
}

fn signed_delta(a: u64, b: u64) -> i64 {
    if a >= b {
        let diff = a - b;
        assert!(diff <= i64::MAX as u64);
        diff as i64
    } else {
        let diff = b - a;
        assert!(diff <= i64::MAX as u64);
        -(diff as i64)
    }
}

fn wall_minus_media_signed(wall_ns: u64, media_ts_ns: u64) -> i64 {
    signed_delta(wall_ns, media_ts_ns)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 48_000;

    fn frames_per_20ms() -> u32 {
        SR / 50
    }

    fn period_ns_20ms() -> u64 {
        20_000_000
    }

    #[test]
    fn first_sample_initialises_state() {
        let mut smoother = AudioTimingSmoother::new(1, SR).expect("ok");
        assert!(!smoother.initialised());
        let result = smoother
            .adjust(SmootherInput {
                media_ts_ns: 1_000_000,
                frames: frames_per_20ms(),
                wall_ns: 1_500_000,
            })
            .expect("ok");
        assert_eq!(result.action, SmootherAction::Reset);
        assert!(smoother.initialised());
        assert!(matches!(
            result.event,
            SmootherEvent::Reset {
                reason: SmootherResetReason::FirstSample,
                ..
            }
        ));
        assert_eq!(result.output_ts_ns, 1_000_000);
        assert_eq!(result.frames_accumulated, frames_per_20ms() as u64);
        assert!(result.predicted_next_ts_ns >= result.output_ts_ns);
    }

    #[test]
    fn second_sample_within_threshold_snaps_to_expected() {
        let mut smoother = AudioTimingSmoother::new(2, SR).expect("ok");
        let frames = frames_per_20ms();
        let _ = smoother
            .adjust(SmootherInput {
                media_ts_ns: 0,
                frames,
                wall_ns: 0,
            })
            .expect("ok");
        let expected = smoother.predicted_next_ts_ns();
        let drift_ns = 10_000_000;
        let r = smoother
            .adjust(SmootherInput {
                media_ts_ns: expected + drift_ns,
                frames,
                wall_ns: expected + drift_ns,
            })
            .expect("ok");
        assert_eq!(r.action, SmootherAction::Snap);
        assert_eq!(r.output_ts_ns, expected);
        assert!(matches!(r.event, SmootherEvent::Snapped { .. }));
    }

    #[test]
    fn jump_outside_threshold_but_within_max_var() {
        let mut smoother = AudioTimingSmoother::new(3, SR).expect("ok");
        let frames = frames_per_20ms();
        let _ = smoother
            .adjust(SmootherInput {
                media_ts_ns: 0,
                frames,
                wall_ns: 0,
            })
            .expect("ok");
        let expected = smoother.predicted_next_ts_ns();
        let jump_ns = 100_000_000;
        let r = smoother
            .adjust(SmootherInput {
                media_ts_ns: expected + jump_ns,
                frames,
                wall_ns: expected + jump_ns,
            })
            .expect("ok");
        assert_eq!(r.action, SmootherAction::Jump);
        assert_eq!(r.output_ts_ns, expected + jump_ns);
        assert!(matches!(r.event, SmootherEvent::Jumped { .. }));
        assert_eq!(smoother.consecutive_jumps(), 1);
    }

    #[test]
    fn gap_beyond_max_var_triggers_full_reset() {
        let mut smoother = AudioTimingSmoother::new(4, SR).expect("ok");
        let frames = frames_per_20ms();
        let _ = smoother
            .adjust(SmootherInput {
                media_ts_ns: 0,
                frames,
                wall_ns: 0,
            })
            .expect("ok");
        let expected = smoother.predicted_next_ts_ns();
        let huge_gap = 3 * NS_PER_SECOND;
        let r = smoother
            .adjust(SmootherInput {
                media_ts_ns: expected + huge_gap,
                frames,
                wall_ns: expected + huge_gap,
            })
            .expect("ok");
        assert_eq!(r.action, SmootherAction::Reset);
        assert!(matches!(
            r.event,
            SmootherEvent::Reset {
                reason: SmootherResetReason::GapBeyondMaxVar,
                ..
            }
        ));
    }

    #[test]
    fn regression_inside_jump_window_triggers_reset() {
        let mut smoother = AudioTimingSmoother::new(5, SR).expect("ok");
        let frames = frames_per_20ms();
        let _ = smoother
            .adjust(SmootherInput {
                media_ts_ns: NS_PER_SECOND,
                frames,
                wall_ns: NS_PER_SECOND,
            })
            .expect("ok");
        let expected = smoother.predicted_next_ts_ns();
        assert!(expected > 200_000_000);
        let r = smoother
            .adjust(SmootherInput {
                media_ts_ns: expected - 200_000_000,
                frames,
                wall_ns: NS_PER_SECOND,
            })
            .expect("ok");
        assert_eq!(r.action, SmootherAction::Reset);
        assert!(matches!(
            r.event,
            SmootherEvent::Reset {
                reason: SmootherResetReason::Regression,
                ..
            }
        ));
    }

    #[test]
    fn monotonicity_preserved_across_1000_samples_with_jitter() {
        let mut smoother = AudioTimingSmoother::new(6, SR).expect("ok");
        let frames = frames_per_20ms();
        let mut wall_ns: u64 = 0;
        let mut media_ts: u64 = 0;
        let mut last_predicted = 0u64;
        for n in 0..1000u64 {
            let jitter: i64 = ((n as i64 * 1_234_567) % 100_000_000) - 50_000_000;
            let jittered_ts = if jitter < 0 {
                media_ts.saturating_sub(jitter.unsigned_abs())
            } else {
                media_ts.saturating_add(jitter as u64)
            };
            let r = smoother
                .adjust(SmootherInput {
                    media_ts_ns: jittered_ts,
                    frames,
                    wall_ns,
                })
                .expect("ok");
            assert!(r.predicted_next_ts_ns >= last_predicted || r.action == SmootherAction::Reset);
            last_predicted = r.predicted_next_ts_ns;
            wall_ns = wall_ns.saturating_add(period_ns_20ms());
            media_ts = media_ts.saturating_add(period_ns_20ms());
        }
        assert!(smoother.frames_accumulated() >= 1000 * frames as u64);
    }

    #[test]
    fn frame_count_accumulates_monotonically() {
        let mut smoother = AudioTimingSmoother::new(7, SR).expect("ok");
        let frames = frames_per_20ms();
        let mut prev = 0u64;
        for n in 0..50u64 {
            let r = smoother
                .adjust(SmootherInput {
                    media_ts_ns: n * period_ns_20ms(),
                    frames,
                    wall_ns: n * period_ns_20ms(),
                })
                .expect("ok");
            assert!(r.frames_accumulated > prev);
            prev = r.frames_accumulated;
        }
    }

    #[test]
    fn rejects_zero_source_id() {
        let err = AudioTimingSmoother::new(0, SR).err();
        assert!(matches!(err, Some(AudioTimingError::SourceIdZero)));
    }

    #[test]
    fn rejects_sample_rate_out_of_range() {
        let err = AudioTimingSmoother::new(1, 1_000).err();
        assert!(matches!(
            err,
            Some(AudioTimingError::SampleRateOutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_zero_frame_count() {
        let mut smoother = AudioTimingSmoother::new(1, SR).expect("ok");
        let err = smoother
            .adjust(SmootherInput {
                media_ts_ns: 0,
                frames: 0,
                wall_ns: 0,
            })
            .err();
        assert!(matches!(
            err,
            Some(AudioTimingError::FramesOutOfRange { .. })
        ));
    }

    #[test]
    fn frames_to_ns_is_round_trip_consistent() {
        let ns = frames_to_ns(48_000, 48_000);
        assert_eq!(ns, NS_PER_SECOND);
        let ns_half = frames_to_ns(24_000, 48_000);
        assert_eq!(ns_half, NS_PER_SECOND / 2);
    }

    #[test]
    fn determinism_two_runs_produce_identical_outputs() {
        let run = || {
            let mut s = AudioTimingSmoother::new(9, SR).expect("ok");
            let mut events: Vec<SmootherResult> = Vec::with_capacity(100);
            for n in 0..100u64 {
                let r = s
                    .adjust(SmootherInput {
                        media_ts_ns: n * period_ns_20ms(),
                        frames: frames_per_20ms(),
                        wall_ns: n * period_ns_20ms() + 1_000,
                    })
                    .expect("ok");
                events.push(r);
            }
            events
        };
        let a = run();
        let b = run();
        assert_eq!(a, b);
    }
}
