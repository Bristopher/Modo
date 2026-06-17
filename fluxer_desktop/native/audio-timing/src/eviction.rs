// SPDX-License-Identifier: AGPL-3.0-or-later

pub const EVICTION_NEVER_PUSHED_SENTINEL: u64 = u64::MAX;

pub const MAX_TRACKED_SOURCES: usize = 256;

const STALE_THRESHOLD_NS_MIN: u64 = 1_000_000;

const STALE_THRESHOLD_NS_MAX: u64 = 60_000_000_000;

#[derive(Debug, PartialEq, Eq)]
pub enum StaleSourceTrackerError {
    CapacityExceeded { capacity: usize },
    SourceIdZero,
    UnknownSource { source_id: u64 },
    StaleThresholdOutOfRange { stale_threshold_ns: u64 },
}

impl core::fmt::Display for StaleSourceTrackerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            StaleSourceTrackerError::CapacityExceeded { capacity } => {
                write!(f, "tracker capacity {capacity} exceeded")
            }
            StaleSourceTrackerError::SourceIdZero => write!(f, "source_id must be non-zero"),
            StaleSourceTrackerError::UnknownSource { source_id } => {
                write!(f, "source_id {source_id} not registered")
            }
            StaleSourceTrackerError::StaleThresholdOutOfRange { stale_threshold_ns } => {
                write!(
                    f,
                    "stale threshold {stale_threshold_ns} ns outside accepted range"
                )
            }
        }
    }
}

impl std::error::Error for StaleSourceTrackerError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaleSourceEntry {
    pub source_id: u64,
    pub registered_at_ns: u64,
    pub last_active_ns: u64,
}

impl StaleSourceEntry {
    pub fn ever_pushed(&self) -> bool {
        self.last_active_ns != EVICTION_NEVER_PUSHED_SENTINEL
    }
}

pub struct StaleSourceTracker {
    entries: Vec<StaleSourceEntry>,
    capacity: usize,
}

impl StaleSourceTracker {
    pub fn new(capacity: usize) -> Result<Self, StaleSourceTrackerError> {
        if capacity == 0 || capacity > MAX_TRACKED_SOURCES {
            return Err(StaleSourceTrackerError::CapacityExceeded { capacity });
        }
        assert!(capacity > 0);
        assert!(capacity <= MAX_TRACKED_SOURCES);
        Ok(Self {
            entries: Vec::with_capacity(capacity),
            capacity,
        })
    }

    pub fn len(&self) -> usize {
        assert!(self.entries.len() <= self.capacity);
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn capacity(&self) -> usize {
        assert!(self.capacity > 0);
        assert!(self.capacity <= MAX_TRACKED_SOURCES);
        self.capacity
    }

    pub fn register_source(
        &mut self,
        source_id: u64,
        registered_at_ns: u64,
    ) -> Result<(), StaleSourceTrackerError> {
        if source_id == 0 {
            return Err(StaleSourceTrackerError::SourceIdZero);
        }
        if self.entries.len() >= self.capacity {
            return Err(StaleSourceTrackerError::CapacityExceeded {
                capacity: self.capacity,
            });
        }
        assert!(source_id != 0);
        if let Some(existing) = self.find_mut(source_id) {
            existing.registered_at_ns = registered_at_ns;
            existing.last_active_ns = EVICTION_NEVER_PUSHED_SENTINEL;
            return Ok(());
        }
        self.entries.push(StaleSourceEntry {
            source_id,
            registered_at_ns,
            last_active_ns: EVICTION_NEVER_PUSHED_SENTINEL,
        });
        assert!(self.entries.len() <= self.capacity);
        Ok(())
    }

    pub fn unregister_source(&mut self, source_id: u64) -> Result<(), StaleSourceTrackerError> {
        if source_id == 0 {
            return Err(StaleSourceTrackerError::SourceIdZero);
        }
        assert!(source_id != 0);
        let before = self.entries.len();
        self.entries.retain(|e| e.source_id != source_id);
        if self.entries.len() == before {
            return Err(StaleSourceTrackerError::UnknownSource { source_id });
        }
        assert!(self.entries.len() < before);
        Ok(())
    }

    pub fn mark_pushed(
        &mut self,
        source_id: u64,
        ts_ns: u64,
    ) -> Result<(), StaleSourceTrackerError> {
        if source_id == 0 {
            return Err(StaleSourceTrackerError::SourceIdZero);
        }
        assert!(source_id != 0);
        let Some(entry) = self.find_mut(source_id) else {
            return Err(StaleSourceTrackerError::UnknownSource { source_id });
        };
        entry.last_active_ns = ts_ns;
        assert!(entry.last_active_ns == ts_ns);
        Ok(())
    }

    pub fn mark_pushed_at(
        &mut self,
        index: usize,
        source_id: u64,
        ts_ns: u64,
    ) -> Result<(), StaleSourceTrackerError> {
        if source_id == 0 {
            return Err(StaleSourceTrackerError::SourceIdZero);
        }
        assert!(source_id != 0);
        assert!(self.entries.len() <= self.capacity);
        if index >= self.entries.len() {
            return Err(StaleSourceTrackerError::UnknownSource { source_id });
        }
        if self.entries[index].source_id != source_id {
            return Err(StaleSourceTrackerError::UnknownSource { source_id });
        }
        self.entries[index].last_active_ns = ts_ns;
        assert!(self.entries[index].last_active_ns == ts_ns);
        Ok(())
    }

    pub fn is_stale(
        &self,
        source_id: u64,
        now_ns: u64,
        stale_threshold_ns: u64,
    ) -> Result<bool, StaleSourceTrackerError> {
        if source_id == 0 {
            return Err(StaleSourceTrackerError::SourceIdZero);
        }
        if !(STALE_THRESHOLD_NS_MIN..=STALE_THRESHOLD_NS_MAX).contains(&stale_threshold_ns) {
            return Err(StaleSourceTrackerError::StaleThresholdOutOfRange { stale_threshold_ns });
        }
        assert!(source_id != 0);
        let Some(entry) = self.find(source_id) else {
            return Err(StaleSourceTrackerError::UnknownSource { source_id });
        };
        Ok(Self::entry_is_stale(entry, now_ns, stale_threshold_ns))
    }

    pub fn is_stale_at(
        &self,
        index: usize,
        source_id: u64,
        now_ns: u64,
        stale_threshold_ns: u64,
    ) -> Result<bool, StaleSourceTrackerError> {
        if source_id == 0 {
            return Err(StaleSourceTrackerError::SourceIdZero);
        }
        if !(STALE_THRESHOLD_NS_MIN..=STALE_THRESHOLD_NS_MAX).contains(&stale_threshold_ns) {
            return Err(StaleSourceTrackerError::StaleThresholdOutOfRange { stale_threshold_ns });
        }
        assert!(source_id != 0);
        assert!(self.entries.len() <= self.capacity);
        if index >= self.entries.len() {
            return Err(StaleSourceTrackerError::UnknownSource { source_id });
        }
        let entry = &self.entries[index];
        if entry.source_id != source_id {
            return Err(StaleSourceTrackerError::UnknownSource { source_id });
        }
        Ok(Self::entry_is_stale(entry, now_ns, stale_threshold_ns))
    }

    fn entry_is_stale(entry: &StaleSourceEntry, now_ns: u64, stale_threshold_ns: u64) -> bool {
        assert!(entry.source_id != 0);
        assert!(stale_threshold_ns >= STALE_THRESHOLD_NS_MIN);
        assert!(stale_threshold_ns <= STALE_THRESHOLD_NS_MAX);
        let baseline_ns = if entry.last_active_ns == EVICTION_NEVER_PUSHED_SENTINEL {
            entry.registered_at_ns
        } else {
            entry.last_active_ns
        };
        if now_ns <= baseline_ns {
            return false;
        }
        let age_ns = now_ns - baseline_ns;
        age_ns > stale_threshold_ns
    }

    pub fn age_ns(&self, source_id: u64, now_ns: u64) -> Result<u64, StaleSourceTrackerError> {
        if source_id == 0 {
            return Err(StaleSourceTrackerError::SourceIdZero);
        }
        assert!(source_id != 0);
        let Some(entry) = self.find(source_id) else {
            return Err(StaleSourceTrackerError::UnknownSource { source_id });
        };
        let baseline_ns = if entry.last_active_ns == EVICTION_NEVER_PUSHED_SENTINEL {
            entry.registered_at_ns
        } else {
            entry.last_active_ns
        };
        if now_ns <= baseline_ns {
            return Ok(0);
        }
        Ok(now_ns - baseline_ns)
    }

    pub fn entry(&self, source_id: u64) -> Option<StaleSourceEntry> {
        self.find(source_id).copied()
    }

    fn find(&self, source_id: u64) -> Option<&StaleSourceEntry> {
        self.entries.iter().find(|e| e.source_id == source_id)
    }

    fn find_mut(&mut self, source_id: u64) -> Option<&mut StaleSourceEntry> {
        self.entries.iter_mut().find(|e| e.source_id == source_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unregistered_source_is_error() {
        let tracker = StaleSourceTracker::new(4).expect("ok");
        let err = tracker.is_stale(7, 1_000_000_000, 5_000_000_000).err();
        assert!(matches!(
            err,
            Some(StaleSourceTrackerError::UnknownSource { source_id: 7 })
        ));
    }

    #[test]
    fn registered_source_starts_not_stale_at_registration_time() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(1, 1_000_000).expect("ok");
        let stale = t.is_stale(1, 1_000_000, 5_000_000_000).expect("ok");
        assert!(!stale);
    }

    #[test]
    fn registered_but_never_pushed_becomes_stale_past_threshold() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(1, 1_000_000).expect("ok");
        let threshold = 5_000_000_000;
        let now = 1_000_000 + threshold + 1;
        let stale = t.is_stale(1, now, threshold).expect("ok");
        assert!(stale, "never-pushed source must go stale past threshold");
    }

    #[test]
    fn registered_but_never_pushed_within_window_is_fresh() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(1, 1_000_000).expect("ok");
        let threshold = 5_000_000_000;
        let now = 1_000_000 + threshold - 1;
        let stale = t.is_stale(1, now, threshold).expect("ok");
        assert!(!stale);
    }

    #[test]
    fn marking_pushed_extends_freshness() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(2, 0).expect("ok");
        t.mark_pushed(2, 1_000_000_000).expect("ok");
        let threshold = 500_000_000;
        let now = 1_000_000_000 + threshold + 1;
        let stale = t.is_stale(2, now, threshold).expect("ok");
        assert!(stale);
        let now_within = 1_000_000_000 + threshold - 1;
        let fresh = t.is_stale(2, now_within, threshold).expect("ok");
        assert!(!fresh);
    }

    #[test]
    fn unregister_removes_entry() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(3, 0).expect("ok");
        assert_eq!(t.len(), 1);
        t.unregister_source(3).expect("ok");
        assert_eq!(t.len(), 0);
    }

    #[test]
    fn capacity_exceeded_rejected() {
        let mut t = StaleSourceTracker::new(2).expect("ok");
        t.register_source(1, 0).expect("ok");
        t.register_source(2, 0).expect("ok");
        let err = t.register_source(3, 0).err();
        assert!(matches!(
            err,
            Some(StaleSourceTrackerError::CapacityExceeded { .. })
        ));
    }

    #[test]
    fn age_ns_uses_registration_for_never_pushed() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(1, 1_000).expect("ok");
        let age = t.age_ns(1, 5_000).expect("ok");
        assert_eq!(age, 4_000);
    }

    #[test]
    fn age_ns_uses_last_active_after_push() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(1, 1_000).expect("ok");
        t.mark_pushed(1, 2_000).expect("ok");
        let age = t.age_ns(1, 5_000).expect("ok");
        assert_eq!(age, 3_000);
    }

    #[test]
    fn determinism_two_runs_produce_same_decisions() {
        let run = || {
            let mut t = StaleSourceTracker::new(8).expect("ok");
            t.register_source(1, 0).expect("ok");
            t.register_source(2, 0).expect("ok");
            t.mark_pushed(2, 1_000_000_000).expect("ok");
            let threshold = 2_000_000_000;
            let out: Vec<bool> = vec![
                t.is_stale(1, 3_000_000_000, threshold).expect("ok"),
                t.is_stale(2, 3_000_000_000, threshold).expect("ok"),
            ];
            out
        };
        let a = run();
        let b = run();
        assert_eq!(a, b);
    }

    #[test]
    fn rejects_invalid_threshold() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(1, 0).expect("ok");
        let err = t.is_stale(1, 1_000_000, 0).err();
        assert!(matches!(
            err,
            Some(StaleSourceTrackerError::StaleThresholdOutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_zero_source_id() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        let err = t.register_source(0, 0).err();
        assert!(matches!(err, Some(StaleSourceTrackerError::SourceIdZero)));
    }

    #[test]
    fn mark_pushed_at_succeeds_when_index_and_id_agree() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(10, 0).expect("ok");
        t.register_source(20, 0).expect("ok");
        t.mark_pushed_at(1, 20, 2_000).expect("ok");
        let e = t.entry(20).expect("entry");
        assert_eq!(e.last_active_ns, 2_000);
        let untouched = t.entry(10).expect("entry");
        assert!(!untouched.ever_pushed());
    }

    #[test]
    fn mark_pushed_at_rejects_index_id_mismatch() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(10, 0).expect("ok");
        t.register_source(20, 0).expect("ok");
        let err = t.mark_pushed_at(0, 20, 2_000).err();
        assert!(matches!(
            err,
            Some(StaleSourceTrackerError::UnknownSource { source_id: 20 })
        ));
        assert!(!t.entry(10).expect("entry").ever_pushed());
        assert!(!t.entry(20).expect("entry").ever_pushed());
    }

    #[test]
    fn mark_pushed_at_rejects_out_of_range_index() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(10, 0).expect("ok");
        let err = t.mark_pushed_at(1, 10, 2_000).err();
        assert!(matches!(
            err,
            Some(StaleSourceTrackerError::UnknownSource { source_id: 10 })
        ));
    }

    #[test]
    fn mark_pushed_at_rejects_zero_source_id() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(10, 0).expect("ok");
        let err = t.mark_pushed_at(0, 0, 2_000).err();
        assert!(matches!(err, Some(StaleSourceTrackerError::SourceIdZero)));
    }

    #[test]
    fn is_stale_at_matches_is_stale() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(10, 0).expect("ok");
        t.register_source(20, 0).expect("ok");
        t.mark_pushed_at(1, 20, 1_000_000_000).expect("ok");
        let threshold = 2_000_000_000;
        for now in [500_000_000, 2_500_000_000, 4_000_000_000] {
            let by_id = t.is_stale(20, now, threshold).expect("ok");
            let by_index = t.is_stale_at(1, 20, now, threshold).expect("ok");
            assert_eq!(by_id, by_index);
        }
    }

    #[test]
    fn is_stale_at_rejects_index_id_mismatch() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(10, 0).expect("ok");
        t.register_source(20, 0).expect("ok");
        let err = t.is_stale_at(0, 20, 3_000_000_000, 2_000_000_000).err();
        assert!(matches!(
            err,
            Some(StaleSourceTrackerError::UnknownSource { source_id: 20 })
        ));
        let err = t.is_stale_at(5, 10, 3_000_000_000, 2_000_000_000).err();
        assert!(matches!(
            err,
            Some(StaleSourceTrackerError::UnknownSource { source_id: 10 })
        ));
    }

    #[test]
    fn is_stale_at_rejects_invalid_threshold() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(10, 0).expect("ok");
        let err = t.is_stale_at(0, 10, 1_000_000, 0).err();
        assert!(matches!(
            err,
            Some(StaleSourceTrackerError::StaleThresholdOutOfRange { .. })
        ));
    }

    #[test]
    fn ever_pushed_flag_reflects_state() {
        let mut t = StaleSourceTracker::new(4).expect("ok");
        t.register_source(1, 10).expect("ok");
        let e = t.entry(1).expect("entry");
        assert!(!e.ever_pushed());
        t.mark_pushed(1, 100).expect("ok");
        let e2 = t.entry(1).expect("entry");
        assert!(e2.ever_pushed());
    }
}
