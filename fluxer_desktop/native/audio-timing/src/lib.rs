// SPDX-License-Identifier: AGPL-3.0-or-later

#![deny(clippy::too_many_lines)]

pub mod eviction;
pub mod ramping;
pub mod smoother;

pub use eviction::{
    EVICTION_NEVER_PUSHED_SENTINEL, MAX_TRACKED_SOURCES, StaleSourceEntry, StaleSourceTracker,
    StaleSourceTrackerError,
};
pub use ramping::{RAMP_GAIN_DENOMINATOR_Q15, RAMP_IN_TICKS_DEFAULT, RampError, SourceGainRamp};
pub use smoother::{
    AudioTimingError, AudioTimingSmoother, MAX_TS_VAR_NS, NS_PER_SECOND, SmootherAction,
    SmootherEvent, SmootherInput, SmootherResetReason, SmootherResult, TS_SMOOTHING_THRESHOLD_NS,
};
