// SPDX-License-Identifier: AGPL-3.0-or-later

use parking_lot::Mutex;
use std::cell::UnsafeCell;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub const PCM_POOL_CAP: usize = 16;
pub const PCM_SLOT_SAMPLES_MAX: usize = 16_384;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PcmPoolError {
    ZeroCapacity,
    ZeroSamplesPerSlot,
    SamplesPerSlotTooLarge(usize),
    PayloadTooLarge { offered: usize, capacity: usize },
}

impl fmt::Display for PcmPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapacity => write!(f, "PcmFramePool capacity must be > 0"),
            Self::ZeroSamplesPerSlot => write!(f, "PcmFramePool samples_per_slot must be > 0"),
            Self::SamplesPerSlotTooLarge(n) => write!(
                f,
                "PcmFramePool samples_per_slot {n} exceeds PCM_SLOT_SAMPLES_MAX={PCM_SLOT_SAMPLES_MAX}"
            ),
            Self::PayloadTooLarge { offered, capacity } => write!(
                f,
                "PcmFramePool payload {offered} samples exceeds slot capacity {capacity}"
            ),
        }
    }
}

impl std::error::Error for PcmPoolError {}

struct PcmSlotCell {
    inner: UnsafeCell<Box<[f32]>>,
}

unsafe impl Send for PcmSlotCell {}
unsafe impl Sync for PcmSlotCell {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcmPoolStats {
    pub acquired: u64,
    pub released: u64,
    pub dropped: u64,
    pub in_flight: u32,
}

pub(crate) struct PcmFramePoolInner {
    slots: Vec<PcmSlotCell>,
    free: Mutex<Vec<usize>>,
    capacity: u32,
    samples_per_slot: u32,
    acquired_total: AtomicU64,
    released_total: AtomicU64,
    dropped_total: AtomicU64,
    in_flight: AtomicU32,
}

pub struct PcmFramePool {
    inner: Arc<PcmFramePoolInner>,
}

impl fmt::Debug for PcmFramePool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = self.stats();
        f.debug_struct("PcmFramePool")
            .field("capacity", &self.inner.capacity)
            .field("samples_per_slot", &self.inner.samples_per_slot)
            .field("stats", &stats)
            .finish()
    }
}

impl PcmFramePool {
    pub fn new(capacity: usize, samples_per_slot: usize) -> Result<Self, PcmPoolError> {
        if capacity == 0 {
            return Err(PcmPoolError::ZeroCapacity);
        }
        if samples_per_slot == 0 {
            return Err(PcmPoolError::ZeroSamplesPerSlot);
        }
        if samples_per_slot > PCM_SLOT_SAMPLES_MAX {
            return Err(PcmPoolError::SamplesPerSlotTooLarge(samples_per_slot));
        }
        assert!(capacity > 0);
        assert!(samples_per_slot > 0);
        assert!(samples_per_slot <= PCM_SLOT_SAMPLES_MAX);

        let mut slots: Vec<PcmSlotCell> = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            let buf: Box<[f32]> = vec![0.0_f32; samples_per_slot].into_boxed_slice();
            assert_eq!(buf.len(), samples_per_slot);
            slots.push(PcmSlotCell {
                inner: UnsafeCell::new(buf),
            });
        }
        assert_eq!(slots.len(), capacity);

        let mut free: Vec<usize> = Vec::with_capacity(capacity);
        for index in 0..capacity {
            free.push(index);
        }
        assert_eq!(free.len(), capacity);

        let cap_u32 = u32::try_from(capacity).map_err(|_| PcmPoolError::ZeroCapacity)?;
        let sps_u32 =
            u32::try_from(samples_per_slot).map_err(|_| PcmPoolError::ZeroSamplesPerSlot)?;
        let inner = PcmFramePoolInner {
            slots,
            free: Mutex::new(free),
            capacity: cap_u32,
            samples_per_slot: sps_u32,
            acquired_total: AtomicU64::new(0),
            released_total: AtomicU64::new(0),
            dropped_total: AtomicU64::new(0),
            in_flight: AtomicU32::new(0),
        };
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn try_acquire(&self) -> Option<PooledPcmFrame> {
        assert!(self.inner.capacity > 0);
        assert!(self.inner.samples_per_slot > 0);

        let mut free = self.inner.free.lock();
        assert!(free.len() <= self.inner.capacity as usize);
        let index = match free.pop() {
            Some(idx) => idx,
            None => {
                drop(free);
                self.inner.dropped_total.fetch_add(1, Ordering::Relaxed);
                return None;
            }
        };
        assert!(index < self.inner.capacity as usize);
        self.inner.acquired_total.fetch_add(1, Ordering::Relaxed);
        let after = self.inner.in_flight.fetch_add(1, Ordering::AcqRel) + 1;
        assert!(after <= self.inner.capacity);
        drop(free);

        Some(PooledPcmFrame {
            slot_index: index,
            filled_len: 0,
            pool: Arc::clone(&self.inner),
        })
    }

    pub fn capacity(&self) -> u32 {
        let cap = self.inner.capacity;
        assert!(cap > 0);
        assert!(cap as usize == self.inner.slots.len());
        cap
    }

    pub fn samples_per_slot(&self) -> u32 {
        let sps = self.inner.samples_per_slot;
        assert!(sps > 0);
        assert!(sps as usize <= PCM_SLOT_SAMPLES_MAX);
        sps
    }

    pub fn stats(&self) -> PcmPoolStats {
        assert!(self.inner.capacity > 0);
        let in_flight = self.inner.in_flight.load(Ordering::Acquire);
        assert!(in_flight <= self.inner.capacity);
        let acquired = self.inner.acquired_total.load(Ordering::Relaxed);
        let released = self.inner.released_total.load(Ordering::Relaxed);
        let dropped = self.inner.dropped_total.load(Ordering::Relaxed);
        assert!(released <= acquired);
        PcmPoolStats {
            acquired,
            released,
            dropped,
            in_flight,
        }
    }
}

impl Clone for PcmFramePool {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct PooledPcmFrame {
    slot_index: usize,
    filled_len: usize,
    pool: Arc<PcmFramePoolInner>,
}

impl PooledPcmFrame {
    pub fn write(&mut self, samples: &[f32]) -> Result<(), PcmPoolError> {
        assert!(self.slot_index < self.pool.capacity as usize);
        let cap = self.pool.samples_per_slot as usize;
        if samples.len() > cap {
            return Err(PcmPoolError::PayloadTooLarge {
                offered: samples.len(),
                capacity: cap,
            });
        }
        assert!(samples.len() <= cap);

        let cell = &self.pool.slots[self.slot_index];
        let buf: &mut [f32] = unsafe { &mut *cell.inner.get() };
        assert_eq!(buf.len(), cap);
        if !samples.is_empty() {
            buf[..samples.len()].copy_from_slice(samples);
        }
        self.filled_len = samples.len();
        assert!(self.filled_len <= cap);
        Ok(())
    }

    pub fn unfilled_mut(&mut self) -> &mut [f32] {
        assert!(self.slot_index < self.pool.capacity as usize);
        let cap = self.pool.samples_per_slot as usize;
        assert!(cap > 0);
        let cell = &self.pool.slots[self.slot_index];
        let buf: &mut [f32] = unsafe { &mut *cell.inner.get() };
        assert_eq!(buf.len(), cap);
        buf
    }

    pub fn set_filled_len(&mut self, len: usize) {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(len <= self.pool.samples_per_slot as usize);
        self.filled_len = len;
    }

    pub fn data_slice(&self) -> &[f32] {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(self.filled_len <= self.pool.samples_per_slot as usize);
        let cell = &self.pool.slots[self.slot_index];
        let buf: &[f32] = unsafe { &*cell.inner.get() };
        &buf[..self.filled_len]
    }

    pub fn filled_len(&self) -> usize {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(self.filled_len <= self.pool.samples_per_slot as usize);
        self.filled_len
    }

    pub fn capacity(&self) -> usize {
        let cap = self.pool.samples_per_slot as usize;
        assert!(cap > 0);
        assert!(cap <= PCM_SLOT_SAMPLES_MAX);
        cap
    }

    pub fn as_mut_ptr(&mut self) -> *mut f32 {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(self.filled_len <= self.pool.samples_per_slot as usize);
        let cell = &self.pool.slots[self.slot_index];
        let buf: &mut [f32] = unsafe { &mut *cell.inner.get() };
        assert_eq!(buf.len(), self.pool.samples_per_slot as usize);
        buf.as_mut_ptr()
    }

    pub fn into_external_parts(mut self) -> (*mut f32, usize, Self) {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(self.filled_len <= self.pool.samples_per_slot as usize);
        let len = self.filled_len;
        let ptr = self.as_mut_ptr();
        assert!(!ptr.is_null());
        (ptr, len, self)
    }
}

impl Drop for PooledPcmFrame {
    fn drop(&mut self) {
        assert!(self.slot_index < self.pool.capacity as usize);
        let mut free = self.pool.free.lock();
        assert!(free.len() < self.pool.capacity as usize);
        free.push(self.slot_index);
        let before = self.pool.in_flight.fetch_sub(1, Ordering::AcqRel);
        assert!(before >= 1);
        self.pool.released_total.fetch_add(1, Ordering::Relaxed);
        drop(free);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    const SLOT_SAMPLES: usize = 2_048;

    fn default_pool() -> PcmFramePool {
        PcmFramePool::new(PCM_POOL_CAP, SLOT_SAMPLES).expect("default pool builds")
    }

    #[test]
    fn new_rejects_zero_capacity() {
        let err = PcmFramePool::new(0, SLOT_SAMPLES).unwrap_err();
        assert_eq!(err, PcmPoolError::ZeroCapacity);
    }

    #[test]
    fn new_rejects_zero_samples_per_slot() {
        let err = PcmFramePool::new(4, 0).unwrap_err();
        assert_eq!(err, PcmPoolError::ZeroSamplesPerSlot);
    }

    #[test]
    fn new_rejects_samples_per_slot_above_max() {
        let err = PcmFramePool::new(4, PCM_SLOT_SAMPLES_MAX + 1).unwrap_err();
        assert_eq!(
            err,
            PcmPoolError::SamplesPerSlotTooLarge(PCM_SLOT_SAMPLES_MAX + 1)
        );
    }

    #[test]
    fn acquire_release_cycle_increments_counters() {
        let pool = PcmFramePool::new(4, SLOT_SAMPLES).expect("pool");
        {
            let _slot = pool.try_acquire().expect("slot");
            let stats_held = pool.stats();
            assert_eq!(stats_held.acquired, 1);
            assert_eq!(stats_held.in_flight, 1);
        }
        let stats_after = pool.stats();
        assert_eq!(stats_after.acquired, 1);
        assert_eq!(stats_after.released, 1);
        assert_eq!(stats_after.in_flight, 0);
    }

    #[test]
    fn pool_exhausts_at_cap_and_counts_drop() {
        let pool = default_pool();
        let mut held = Vec::with_capacity(PCM_POOL_CAP);
        for _ in 0..PCM_POOL_CAP {
            held.push(pool.try_acquire().expect("slot in capacity"));
        }
        assert!(pool.try_acquire().is_none());
        let stats = pool.stats();
        assert_eq!(stats.dropped, 1);
        assert_eq!(stats.acquired as usize, PCM_POOL_CAP);
        assert_eq!(stats.in_flight as usize, PCM_POOL_CAP);
    }

    #[test]
    fn write_then_data_slice_matches_payload() {
        let pool = PcmFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        let payload = [0.25_f32; 32];
        slot.write(&payload).expect("payload fits");
        assert_eq!(slot.data_slice(), &payload[..]);
        assert_eq!(slot.filled_len(), 32);
    }

    #[test]
    fn write_rejects_payload_larger_than_slot() {
        let pool = PcmFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        let big = [0.0_f32; 128];
        let err = slot.write(&big).unwrap_err();
        assert!(matches!(err, PcmPoolError::PayloadTooLarge { .. }));
    }

    #[test]
    fn unfilled_mut_then_set_filled_len_matches_data_slice() {
        let pool = PcmFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        {
            let buf = slot.unfilled_mut();
            assert_eq!(buf.len(), 64);
            buf[0] = 0.5;
            buf[1] = -0.5;
            buf[2] = 1.0;
        }
        slot.set_filled_len(3);
        assert_eq!(slot.filled_len(), 3);
        assert_eq!(slot.data_slice(), &[0.5, -0.5, 1.0]);
    }

    #[test]
    #[should_panic]
    fn set_filled_len_rejects_overflow() {
        let pool = PcmFramePool::new(1, 32).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        slot.set_filled_len(33);
    }

    #[test]
    fn into_external_parts_exposes_filled_pointer_and_length() {
        let pool = PcmFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        let payload = [1.5_f32; 16];
        slot.write(&payload).expect("write fits");
        let (ptr, len, owned) = slot.into_external_parts();
        assert!(!ptr.is_null());
        assert_eq!(len, 16);
        let observed = unsafe { core::slice::from_raw_parts(ptr, len) };
        assert_eq!(observed, &payload[..]);
        assert_eq!(pool.stats().in_flight, 1);
        drop(owned);
        assert_eq!(pool.stats().in_flight, 0);
    }

    #[test]
    fn into_external_parts_drop_returns_slot_to_pool() {
        let pool = PcmFramePool::new(1, 32).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        slot.write(&[0.75_f32; 8]).expect("write");
        let (_ptr, _len, owned) = slot.into_external_parts();
        assert!(pool.try_acquire().is_none());
        drop(owned);
        let revived = pool.try_acquire().expect("revived");
        assert_eq!(revived.filled_len(), 0);
        drop(revived);
        assert_eq!(pool.stats().in_flight, 0);
    }

    #[test]
    fn pooled_frame_survives_send_across_threads() {
        let pool = PcmFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        slot.write(&[0.5_f32; 32]).expect("write");
        let handle = thread::spawn(move || {
            assert_eq!(slot.filled_len(), 32);
            assert_eq!(slot.data_slice()[0], 0.5);
            drop(slot);
        });
        handle.join().expect("worker");
        assert_eq!(pool.stats().in_flight, 0);
        assert_eq!(pool.stats().released, 1);
    }

    #[test]
    fn capacity_one_pool_round_trips() {
        let pool = PcmFramePool::new(1, 32).expect("pool");
        for _ in 0..5 {
            let mut slot = pool.try_acquire().expect("slot");
            slot.write(&[1.0, 2.0, 3.0]).expect("write");
            assert_eq!(slot.data_slice(), &[1.0, 2.0, 3.0]);
            drop(slot);
        }
        let stats = pool.stats();
        assert_eq!(stats.acquired, 5);
        assert_eq!(stats.released, 5);
        assert_eq!(stats.in_flight, 0);
    }

    #[test]
    fn default_pool_dimensions_match_constants() {
        let pool = default_pool();
        assert_eq!(pool.capacity() as usize, PCM_POOL_CAP);
        assert_eq!(pool.samples_per_slot() as usize, SLOT_SAMPLES);
    }
}
