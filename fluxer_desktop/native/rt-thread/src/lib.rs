// SPDX-License-Identifier: AGPL-3.0-or-later

#![deny(clippy::too_many_lines)]

pub mod priority;
pub mod tick;

pub use priority::{PriorityProfile, RealtimePriorityGuard, RtError, RtOutcome};
pub use tick::{MonotonicClock, SystemMonotonicClock, TickDriver, TickInfo};

pub const NS_PER_MS: u64 = 1_000_000;
pub const NS_PER_SEC: u64 = 1_000_000_000;
