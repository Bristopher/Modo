// SPDX-License-Identifier: AGPL-3.0-or-later

pub const RAMP_IN_TICKS_DEFAULT: u32 = 5;

pub const RAMP_GAIN_DENOMINATOR_Q15: i32 = 32_768;

const RAMP_IN_TICKS_MIN: u32 = 1;
const RAMP_IN_TICKS_MAX: u32 = 1_024;

#[derive(Debug, PartialEq, Eq)]
pub enum RampError {
    RampInTicksOutOfRange { ramp_in_ticks: u32 },
}

impl core::fmt::Display for RampError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RampError::RampInTicksOutOfRange { ramp_in_ticks } => {
                write!(f, "ramp_in_ticks {ramp_in_ticks} outside accepted range")
            }
        }
    }
}

impl std::error::Error for RampError {}

pub struct SourceGainRamp {
    tick_count: u32,
    ramp_in_ticks: u32,
}

impl SourceGainRamp {
    pub fn new() -> Self {
        Self {
            tick_count: 0,
            ramp_in_ticks: RAMP_IN_TICKS_DEFAULT,
        }
    }

    pub fn with_ramp_in_ticks(ramp_in_ticks: u32) -> Result<Self, RampError> {
        if !(RAMP_IN_TICKS_MIN..=RAMP_IN_TICKS_MAX).contains(&ramp_in_ticks) {
            return Err(RampError::RampInTicksOutOfRange { ramp_in_ticks });
        }
        assert!(ramp_in_ticks >= RAMP_IN_TICKS_MIN);
        assert!(ramp_in_ticks <= RAMP_IN_TICKS_MAX);
        Ok(Self {
            tick_count: 0,
            ramp_in_ticks,
        })
    }

    pub fn tick_count(&self) -> u32 {
        self.tick_count
    }

    pub fn ramp_in_ticks(&self) -> u32 {
        assert!(self.ramp_in_ticks >= RAMP_IN_TICKS_MIN);
        assert!(self.ramp_in_ticks <= RAMP_IN_TICKS_MAX);
        self.ramp_in_ticks
    }

    pub fn is_complete(&self) -> bool {
        self.tick_count >= self.ramp_in_ticks
    }

    pub fn current_gain(&self) -> f32 {
        assert!(self.ramp_in_ticks >= RAMP_IN_TICKS_MIN);
        let clamped = self.tick_count.min(self.ramp_in_ticks);
        let g = (clamped as f32) / (self.ramp_in_ticks as f32);
        assert!(g >= 0.0);
        assert!(g <= 1.0);
        g
    }

    pub fn current_gain_q15(&self) -> i32 {
        assert!(self.ramp_in_ticks >= RAMP_IN_TICKS_MIN);
        let clamped = self.tick_count.min(self.ramp_in_ticks);
        let num = (clamped as i64) * (RAMP_GAIN_DENOMINATOR_Q15 as i64);
        let den = self.ramp_in_ticks as i64;
        let q = num / den;
        assert!(q >= 0);
        assert!(q <= RAMP_GAIN_DENOMINATOR_Q15 as i64);
        q as i32
    }

    pub fn advance_tick(&mut self) {
        let before = self.tick_count;
        self.tick_count = self.tick_count.saturating_add(1);
        assert!(self.tick_count >= before);
        assert!(self.tick_count >= 1);
    }

    pub fn reset(&mut self) {
        self.tick_count = 0;
        assert_eq!(self.tick_count, 0);
        assert!(self.ramp_in_ticks >= RAMP_IN_TICKS_MIN);
    }
}

impl Default for SourceGainRamp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_ramp_starts_at_zero_gain() {
        let r = SourceGainRamp::new();
        assert_eq!(r.tick_count(), 0);
        assert_eq!(r.current_gain(), 0.0);
        assert!(!r.is_complete());
    }

    #[test]
    fn five_ticks_increase_monotonically_to_one() {
        let mut r = SourceGainRamp::new();
        let mut prior = -1.0f32;
        for expected_n in 1..=5u32 {
            r.advance_tick();
            let g = r.current_gain();
            assert!(g > prior);
            assert!(g >= 0.0);
            assert!(g <= 1.0);
            assert_eq!(r.tick_count(), expected_n);
            prior = g;
        }
        assert_eq!(r.current_gain(), 1.0);
        assert!(r.is_complete());
    }

    #[test]
    fn gain_clamps_to_one_after_completion() {
        let mut r = SourceGainRamp::new();
        for _ in 0..20 {
            r.advance_tick();
        }
        assert_eq!(r.current_gain(), 1.0);
        assert!(r.is_complete());
    }

    #[test]
    fn first_five_gains_are_quintiles() {
        let mut r = SourceGainRamp::new();
        let mut observed: Vec<f32> = Vec::with_capacity(5);
        for _ in 0..5 {
            r.advance_tick();
            observed.push(r.current_gain());
        }
        assert_eq!(observed.len(), 5);
        assert!((observed[0] - 0.2).abs() < f32::EPSILON);
        assert!((observed[1] - 0.4).abs() < f32::EPSILON * 4.0);
        assert!((observed[2] - 0.6).abs() < f32::EPSILON * 4.0);
        assert!((observed[3] - 0.8).abs() < f32::EPSILON * 4.0);
        assert_eq!(observed[4], 1.0);
    }

    #[test]
    fn q15_gain_at_completion_equals_denominator() {
        let mut r = SourceGainRamp::new();
        for _ in 0..5 {
            r.advance_tick();
        }
        assert_eq!(r.current_gain_q15(), RAMP_GAIN_DENOMINATOR_Q15);
    }

    #[test]
    fn q15_gain_starts_at_zero() {
        let r = SourceGainRamp::new();
        assert_eq!(r.current_gain_q15(), 0);
    }

    #[test]
    fn reset_returns_ramp_to_zero_gain() {
        let mut r = SourceGainRamp::new();
        for _ in 0..10 {
            r.advance_tick();
        }
        assert_eq!(r.current_gain(), 1.0);
        r.reset();
        assert_eq!(r.tick_count(), 0);
        assert_eq!(r.current_gain(), 0.0);
    }

    #[test]
    fn custom_ramp_in_ticks_validates() {
        let r = SourceGainRamp::with_ramp_in_ticks(10).expect("ok");
        assert_eq!(r.ramp_in_ticks(), 10);
        let err = SourceGainRamp::with_ramp_in_ticks(0).err();
        assert!(matches!(err, Some(RampError::RampInTicksOutOfRange { .. })));
        let err2 = SourceGainRamp::with_ramp_in_ticks(10_000).err();
        assert!(matches!(
            err2,
            Some(RampError::RampInTicksOutOfRange { .. })
        ));
    }

    #[test]
    fn determinism_two_ramps_produce_same_sequence() {
        let run = || {
            let mut r = SourceGainRamp::new();
            let mut g: Vec<i32> = Vec::with_capacity(7);
            for _ in 0..7 {
                g.push(r.current_gain_q15());
                r.advance_tick();
            }
            g
        };
        let a = run();
        let b = run();
        assert_eq!(a, b);
    }
}
