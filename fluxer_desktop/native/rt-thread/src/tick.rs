// SPDX-License-Identifier: AGPL-3.0-or-later

use core::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::NS_PER_MS;

pub trait MonotonicClock: Send + Sync {
    fn now_ns(&self) -> u64;
}

pub struct SystemMonotonicClock {
    origin: Instant,
    last_observed: AtomicU64,
}

impl SystemMonotonicClock {
    pub fn new() -> Self {
        let me = Self {
            origin: Instant::now(),
            last_observed: AtomicU64::new(0),
        };
        assert_eq!(me.last_observed.load(Ordering::Acquire), 0);
        assert!(me.origin.elapsed().as_nanos() < u64::MAX as u128);
        me
    }
}

impl Default for SystemMonotonicClock {
    fn default() -> Self {
        Self::new()
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn now_ns(&self) -> u64 {
        let elapsed = self.origin.elapsed();
        let ns = elapsed.as_nanos();
        assert!(ns <= u64::MAX as u128);
        let ns_u64 = ns as u64;
        let prior = self.last_observed.fetch_max(ns_u64, Ordering::AcqRel);
        let observed = ns_u64.max(prior);
        assert!(observed >= prior);
        observed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickInfo {
    pub tick_index: u64,
    pub scheduled_ns: u64,
    pub actual_ns: u64,
    pub lag_ns: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TickError {
    ZeroPeriod,
    PeriodTooLarge,
    NonMonotonicClock { prior_ns: u64, observed_ns: u64 },
}

impl fmt::Display for TickError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TickError::ZeroPeriod => write!(f, "tick period must be non-zero"),
            TickError::PeriodTooLarge => write!(f, "tick period exceeds u64 budget"),
            TickError::NonMonotonicClock {
                prior_ns,
                observed_ns,
            } => write!(
                f,
                "MonotonicClock returned non-monotonic reading: prior={prior_ns} observed={observed_ns}",
            ),
        }
    }
}

impl std::error::Error for TickError {}

pub const TICK_PERIOD_NS_MAX: u64 = 60 * crate::NS_PER_SEC;

const SLEEP_SAFETY_MARGIN_NS: u64 = NS_PER_MS;

const SPIN_THRESHOLD_NS: u64 = 150_000;

pub struct TickDriver<C: MonotonicClock> {
    clock: Arc<C>,
    period_ns: u64,
    start_ns: u64,
    next_tick_index: u64,
    last_scheduled_ns: u64,
    last_actual_ns: u64,
}

impl<C: MonotonicClock> TickDriver<C> {
    pub fn new(clock: Arc<C>, period_ns: u64) -> Result<Self, TickError> {
        if period_ns == 0 {
            return Err(TickError::ZeroPeriod);
        }
        if period_ns > TICK_PERIOD_NS_MAX {
            return Err(TickError::PeriodTooLarge);
        }
        let start_ns = clock.now_ns();
        assert!(period_ns > 0);
        assert!(period_ns <= TICK_PERIOD_NS_MAX);
        Ok(Self {
            clock,
            period_ns,
            start_ns,
            next_tick_index: 0,
            last_scheduled_ns: 0,
            last_actual_ns: start_ns,
        })
    }

    pub fn period_ns(&self) -> u64 {
        assert!(self.period_ns > 0);
        assert!(self.period_ns <= TICK_PERIOD_NS_MAX);
        self.period_ns
    }

    pub fn start_ns(&self) -> u64 {
        assert!(self.start_ns <= self.clock.now_ns().saturating_add(self.period_ns));
        self.start_ns
    }

    pub fn wait_until_next_tick(&mut self) -> Result<TickInfo, TickError> {
        assert!(self.period_ns > 0);
        let mut prior_actual = self.last_actual_ns;
        let mut tick_index = self.next_tick_index;
        let scheduled_ns = self
            .start_ns
            .saturating_add(tick_index.saturating_mul(self.period_ns));
        assert!(scheduled_ns >= self.last_scheduled_ns || tick_index == 0);
        loop {
            let now = self.clock.now_ns();
            if now < prior_actual {
                return Err(TickError::NonMonotonicClock {
                    prior_ns: prior_actual,
                    observed_ns: now,
                });
            }
            prior_actual = now;
            if now >= scheduled_ns {
                break;
            }
            let remaining = scheduled_ns - now;
            if remaining > SLEEP_SAFETY_MARGIN_NS {
                thread::park_timeout(Duration::from_nanos(remaining - SLEEP_SAFETY_MARGIN_NS));
            } else if remaining > SPIN_THRESHOLD_NS {
                thread::park_timeout(Duration::from_nanos(remaining - SPIN_THRESHOLD_NS));
            } else {
                core::hint::spin_loop();
            }
        }
        let actual_ns = self.clock.now_ns();
        if actual_ns < prior_actual {
            return Err(TickError::NonMonotonicClock {
                prior_ns: prior_actual,
                observed_ns: actual_ns,
            });
        }
        let lag_ns = (actual_ns as i128 - scheduled_ns as i128) as i64;
        let info = TickInfo {
            tick_index,
            scheduled_ns,
            actual_ns,
            lag_ns,
        };
        if actual_ns > scheduled_ns.saturating_add(self.period_ns) {
            let overshoot = (actual_ns - scheduled_ns) / self.period_ns;
            tick_index = tick_index.saturating_add(overshoot);
        }
        self.next_tick_index = tick_index.saturating_add(1);
        self.last_scheduled_ns = scheduled_ns;
        self.last_actual_ns = actual_ns;
        assert!(self.next_tick_index > tick_index);
        assert_eq!(
            info.lag_ns,
            info.actual_ns as i64 - info.scheduled_ns as i64
        );
        Ok(info)
    }

    pub fn next_tick_index(&self) -> u64 {
        assert!(self.next_tick_index >= 1 || self.last_actual_ns == self.start_ns);
        self.next_tick_index
    }
}

pub struct ScriptedClock {
    readings: Mutex<Vec<u64>>,
    cursor: AtomicU64,
}

impl ScriptedClock {
    pub fn new(readings: Vec<u64>) -> Self {
        assert!(!readings.is_empty());
        Self {
            readings: Mutex::new(readings),
            cursor: AtomicU64::new(0),
        }
    }
}

impl MonotonicClock for ScriptedClock {
    fn now_ns(&self) -> u64 {
        let readings = self.readings.lock().expect("readings lock poisoned");
        assert!(!readings.is_empty());
        let idx = self.cursor.fetch_add(1, Ordering::AcqRel) as usize;
        let n = readings.len();
        let chosen = if idx < n {
            readings[idx]
        } else {
            *readings.last().unwrap()
        };
        assert!(n > 0);
        chosen
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    fn ci_mode() -> bool {
        std::env::var_os("CI").is_some()
    }

    #[test]
    fn zero_period_rejected() {
        let clock = Arc::new(SystemMonotonicClock::new());
        let err = TickDriver::new(clock, 0).err();
        assert_eq!(err, Some(TickError::ZeroPeriod));
    }

    #[test]
    fn period_too_large_rejected() {
        let clock = Arc::new(SystemMonotonicClock::new());
        let err = TickDriver::new(clock, TICK_PERIOD_NS_MAX + 1).err();
        assert_eq!(err, Some(TickError::PeriodTooLarge));
    }

    #[test]
    fn system_clock_is_monotonic_over_many_samples() {
        let clock = SystemMonotonicClock::new();
        let mut prev = clock.now_ns();
        let samples = 10_000;
        for _ in 0..samples {
            let now = clock.now_ns();
            assert!(now >= prev, "clock went backwards: prev={prev} now={now}");
            prev = now;
        }
    }

    #[test]
    fn tick_driver_meets_jitter_budget_at_1ms_period() {
        let clock = Arc::new(SystemMonotonicClock::new());
        let period_ns = NS_PER_MS;
        let mut driver = TickDriver::new(clock, period_ns).expect("driver");
        let n: u64 = 100;
        let mut total_error_ns: i128 = 0;
        let mut max_error_ns: i64 = 0;
        for _ in 0..n {
            let info = driver.wait_until_next_tick().expect("tick ok");
            assert!(info.lag_ns >= -(period_ns as i64));
            let err = info.lag_ns.abs();
            total_error_ns += err as i128;
            if err > max_error_ns {
                max_error_ns = err;
            }
        }
        let mean_error_ns = (total_error_ns / n as i128) as i64;
        let max_budget_ns: i64 = if ci_mode() {
            25 * NS_PER_MS as i64
        } else {
            NS_PER_MS as i64
        };
        let mean_budget_ns: i64 = if ci_mode() {
            5 * NS_PER_MS as i64
        } else {
            200_000
        };
        assert!(
            mean_error_ns < mean_budget_ns,
            "mean error {mean_error_ns} ns exceeded budget {mean_budget_ns} ns (ci={})",
            ci_mode(),
        );
        assert!(
            max_error_ns < max_budget_ns,
            "max error {max_error_ns} ns exceeded budget {max_budget_ns} ns (ci={})",
            ci_mode(),
        );
    }

    #[test]
    fn lag_reported_when_caller_sleeps_over_a_tick_boundary() {
        let clock = Arc::new(SystemMonotonicClock::new());
        let period_ns = 5 * NS_PER_MS;
        let mut driver = TickDriver::new(clock, period_ns).expect("driver");
        let _first = driver.wait_until_next_tick().expect("first tick");
        thread::sleep(Duration::from_millis(20));
        let info = driver.wait_until_next_tick().expect("after sleep");
        assert!(
            info.lag_ns > 0,
            "expected positive lag, got {}",
            info.lag_ns
        );
    }

    #[test]
    fn tick_index_is_monotonic() {
        let clock = Arc::new(SystemMonotonicClock::new());
        let mut driver = TickDriver::new(clock, NS_PER_MS).expect("driver");
        let mut prior_index = None::<u64>;
        let mut prior_scheduled = None::<u64>;
        for _ in 0..16 {
            let info = driver.wait_until_next_tick().expect("tick");
            if let Some(prev) = prior_index {
                assert!(
                    info.tick_index > prev,
                    "tick_index not monotonic: prev={prev} cur={}",
                    info.tick_index
                );
            }
            if let Some(prev) = prior_scheduled {
                assert!(info.scheduled_ns > prev, "scheduled_ns not monotonic");
            }
            prior_index = Some(info.tick_index);
            prior_scheduled = Some(info.scheduled_ns);
        }
    }

    #[test]
    fn non_monotonic_clock_is_detected() {
        let clock = Arc::new(ScriptedClock::new(vec![
            0, 10_000_000, 5_000_000, 5_000_000,
        ]));
        let mut driver = TickDriver::new(clock, NS_PER_MS).expect("driver");
        let err = driver.wait_until_next_tick().err();
        assert!(
            matches!(err, Some(TickError::NonMonotonicClock { .. })),
            "got {err:?}"
        );
    }

    #[test]
    fn tick_info_lag_invariant_holds() {
        let clock = Arc::new(SystemMonotonicClock::new());
        let mut driver = TickDriver::new(clock, NS_PER_MS).expect("driver");
        for _ in 0..5 {
            let info = driver.wait_until_next_tick().expect("tick");
            let derived = info.actual_ns as i64 - info.scheduled_ns as i64;
            assert_eq!(info.lag_ns, derived);
        }
    }
}
