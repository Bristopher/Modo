// SPDX-License-Identifier: AGPL-3.0-or-later

use parking_lot::Mutex;
use std::cell::UnsafeCell;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub const MAC_AUDIO_POOL_CAP: usize = 16;
pub const MAX_FRAME_BYTES_PER_SLOT: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacAudioError {
    ZeroCapacity,
    ZeroBytesPerSlot,
    BytesPerSlotTooLarge(usize),
    PayloadTooLarge { offered: usize, capacity: usize },
}

impl fmt::Display for MacAudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapacity => write!(f, "MacAudioFramePool capacity must be > 0"),
            Self::ZeroBytesPerSlot => write!(f, "MacAudioFramePool bytes_per_slot must be > 0"),
            Self::BytesPerSlotTooLarge(n) => write!(
                f,
                "MacAudioFramePool bytes_per_slot {n} exceeds MAX_FRAME_BYTES_PER_SLOT={MAX_FRAME_BYTES_PER_SLOT}"
            ),
            Self::PayloadTooLarge { offered, capacity } => write!(
                f,
                "MacAudioFramePool payload {offered} bytes exceeds slot capacity {capacity}"
            ),
        }
    }
}

impl std::error::Error for MacAudioError {}

struct PoolSlotCell {
    inner: UnsafeCell<Box<[u8]>>,
}

unsafe impl Send for PoolSlotCell {}
unsafe impl Sync for PoolSlotCell {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MacAudioPoolStats {
    pub acquired: u64,
    pub released: u64,
    pub dropped: u64,
    pub in_flight: u32,
}

pub(crate) struct MacAudioFramePoolInner {
    slots: Vec<PoolSlotCell>,
    free: Mutex<Vec<usize>>,
    capacity: u32,
    bytes_per_slot: u32,
    acquired_total: AtomicU64,
    released_total: AtomicU64,
    dropped_total: AtomicU64,
    in_flight: AtomicU32,
}

pub struct MacAudioFramePool {
    inner: Arc<MacAudioFramePoolInner>,
}

impl fmt::Debug for MacAudioFramePool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stats = self.stats();
        f.debug_struct("MacAudioFramePool")
            .field("capacity", &self.inner.capacity)
            .field("bytes_per_slot", &self.inner.bytes_per_slot)
            .field("stats", &stats)
            .finish()
    }
}

impl MacAudioFramePool {
    pub fn new(capacity: usize, bytes_per_slot: usize) -> Result<Self, MacAudioError> {
        if capacity == 0 {
            return Err(MacAudioError::ZeroCapacity);
        }
        if bytes_per_slot == 0 {
            return Err(MacAudioError::ZeroBytesPerSlot);
        }
        if bytes_per_slot > MAX_FRAME_BYTES_PER_SLOT {
            return Err(MacAudioError::BytesPerSlotTooLarge(bytes_per_slot));
        }
        assert!(capacity > 0);
        assert!(bytes_per_slot > 0);
        assert!(bytes_per_slot <= MAX_FRAME_BYTES_PER_SLOT);

        let mut slots: Vec<PoolSlotCell> = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            let buf: Box<[u8]> = vec![0u8; bytes_per_slot].into_boxed_slice();
            assert_eq!(buf.len(), bytes_per_slot);
            slots.push(PoolSlotCell {
                inner: UnsafeCell::new(buf),
            });
        }
        assert_eq!(slots.len(), capacity);

        let mut free: Vec<usize> = Vec::with_capacity(capacity);
        for index in 0..capacity {
            free.push(index);
        }
        assert_eq!(free.len(), capacity);

        let cap_u32 = u32::try_from(capacity).map_err(|_| MacAudioError::ZeroCapacity)?;
        let bps_u32 = u32::try_from(bytes_per_slot).map_err(|_| MacAudioError::ZeroBytesPerSlot)?;
        let inner = MacAudioFramePoolInner {
            slots,
            free: Mutex::new(free),
            capacity: cap_u32,
            bytes_per_slot: bps_u32,
            acquired_total: AtomicU64::new(0),
            released_total: AtomicU64::new(0),
            dropped_total: AtomicU64::new(0),
            in_flight: AtomicU32::new(0),
        };
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn try_acquire(&self) -> Option<PooledMacAudioFrame> {
        assert!(self.inner.capacity > 0);
        assert!(self.inner.bytes_per_slot > 0);

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

        Some(PooledMacAudioFrame {
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

    pub fn bytes_per_slot(&self) -> u32 {
        let bps = self.inner.bytes_per_slot;
        assert!(bps > 0);
        assert!(bps as usize <= MAX_FRAME_BYTES_PER_SLOT);
        bps
    }

    pub fn stats(&self) -> MacAudioPoolStats {
        assert!(self.inner.capacity > 0);
        let in_flight = self.inner.in_flight.load(Ordering::Acquire);
        assert!(in_flight <= self.inner.capacity);
        let acquired = self.inner.acquired_total.load(Ordering::Relaxed);
        let released = self.inner.released_total.load(Ordering::Relaxed);
        let dropped = self.inner.dropped_total.load(Ordering::Relaxed);
        assert!(released <= acquired);
        MacAudioPoolStats {
            acquired,
            released,
            dropped,
            in_flight,
        }
    }
}

impl Clone for MacAudioFramePool {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub struct PooledMacAudioFrame {
    slot_index: usize,
    filled_len: usize,
    pool: Arc<MacAudioFramePoolInner>,
}

impl PooledMacAudioFrame {
    pub fn write(&mut self, samples: &[u8]) -> Result<(), MacAudioError> {
        assert!(self.slot_index < self.pool.capacity as usize);
        let cap = self.pool.bytes_per_slot as usize;
        if samples.len() > cap {
            return Err(MacAudioError::PayloadTooLarge {
                offered: samples.len(),
                capacity: cap,
            });
        }
        assert!(samples.len() <= cap);

        let cell = &self.pool.slots[self.slot_index];
        let buf: &mut [u8] = unsafe { &mut *cell.inner.get() };
        assert_eq!(buf.len(), cap);
        if !samples.is_empty() {
            buf[..samples.len()].copy_from_slice(samples);
        }
        self.filled_len = samples.len();
        assert!(self.filled_len <= cap);
        Ok(())
    }

    pub fn append(&mut self, samples: &[u8]) -> Result<(), MacAudioError> {
        assert!(self.slot_index < self.pool.capacity as usize);
        let cap = self.pool.bytes_per_slot as usize;
        let new_len =
            self.filled_len
                .checked_add(samples.len())
                .ok_or(MacAudioError::PayloadTooLarge {
                    offered: usize::MAX,
                    capacity: cap,
                })?;
        if new_len > cap {
            return Err(MacAudioError::PayloadTooLarge {
                offered: new_len,
                capacity: cap,
            });
        }
        assert!(new_len <= cap);

        let cell = &self.pool.slots[self.slot_index];
        let buf: &mut [u8] = unsafe { &mut *cell.inner.get() };
        assert_eq!(buf.len(), cap);
        if !samples.is_empty() {
            buf[self.filled_len..new_len].copy_from_slice(samples);
        }
        self.filled_len = new_len;
        Ok(())
    }

    pub fn data_slice(&self) -> &[u8] {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(self.filled_len <= self.pool.bytes_per_slot as usize);
        let cell = &self.pool.slots[self.slot_index];
        let buf: &[u8] = unsafe { &*cell.inner.get() };
        &buf[..self.filled_len]
    }

    pub fn filled_len(&self) -> usize {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(self.filled_len <= self.pool.bytes_per_slot as usize);
        self.filled_len
    }

    pub fn capacity(&self) -> usize {
        let cap = self.pool.bytes_per_slot as usize;
        assert!(cap > 0);
        assert!(cap <= MAX_FRAME_BYTES_PER_SLOT);
        cap
    }

    pub fn slot_index(&self) -> usize {
        assert!(self.slot_index < self.pool.capacity as usize);
        self.slot_index
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(self.filled_len <= self.pool.bytes_per_slot as usize);
        let cell = &self.pool.slots[self.slot_index];
        let buf: &mut [u8] = unsafe { &mut *cell.inner.get() };
        assert_eq!(buf.len(), self.pool.bytes_per_slot as usize);
        buf.as_mut_ptr()
    }

    pub fn into_external_parts(mut self) -> (*mut u8, usize, Self) {
        assert!(self.slot_index < self.pool.capacity as usize);
        assert!(self.filled_len <= self.pool.bytes_per_slot as usize);
        let len = self.filled_len;
        let ptr = self.as_mut_ptr();
        assert!(!ptr.is_null());
        (ptr, len, self)
    }
}

impl Drop for PooledMacAudioFrame {
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
    use std::sync::Barrier;
    use std::sync::atomic::AtomicUsize;
    use std::thread;
    use std::time::{Duration, Instant};

    const SLOT_BYTES: usize = 3840;

    fn default_pool() -> MacAudioFramePool {
        MacAudioFramePool::new(MAC_AUDIO_POOL_CAP, SLOT_BYTES).expect("default pool builds")
    }

    #[test]
    fn new_rejects_zero_capacity() {
        let err = MacAudioFramePool::new(0, SLOT_BYTES).unwrap_err();
        assert_eq!(err, MacAudioError::ZeroCapacity);
    }

    #[test]
    fn new_rejects_zero_bytes_per_slot() {
        let err = MacAudioFramePool::new(4, 0).unwrap_err();
        assert_eq!(err, MacAudioError::ZeroBytesPerSlot);
    }

    #[test]
    fn new_rejects_bytes_per_slot_above_max() {
        let err = MacAudioFramePool::new(4, MAX_FRAME_BYTES_PER_SLOT + 1).unwrap_err();
        assert_eq!(
            err,
            MacAudioError::BytesPerSlotTooLarge(MAX_FRAME_BYTES_PER_SLOT + 1)
        );
    }

    #[test]
    fn acquire_release_cycle_increments_counters() {
        let pool = MacAudioFramePool::new(4, SLOT_BYTES).expect("pool");
        let stats_before = pool.stats();
        assert_eq!(stats_before.acquired, 0);
        assert_eq!(stats_before.released, 0);
        assert_eq!(stats_before.in_flight, 0);
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
    fn pool_exhausts_at_cap_seventeenth_acquire_returns_none() {
        let pool = default_pool();
        let mut held = Vec::with_capacity(MAC_AUDIO_POOL_CAP);
        for _ in 0..MAC_AUDIO_POOL_CAP {
            held.push(pool.try_acquire().expect("slot in capacity"));
        }
        assert!(pool.try_acquire().is_none());
        let stats = pool.stats();
        assert_eq!(stats.dropped, 1);
        assert_eq!(stats.acquired as usize, MAC_AUDIO_POOL_CAP);
        assert_eq!(stats.in_flight as usize, MAC_AUDIO_POOL_CAP);
    }

    #[test]
    fn drop_returns_slot_to_free_list() {
        let pool = MacAudioFramePool::new(2, SLOT_BYTES).expect("pool");
        let first = pool.try_acquire().expect("first");
        let second = pool.try_acquire().expect("second");
        assert!(pool.try_acquire().is_none());
        drop(first);
        let revived = pool.try_acquire().expect("revived");
        assert_eq!(pool.stats().in_flight, 2);
        drop(second);
        drop(revived);
        assert_eq!(pool.stats().in_flight, 0);
    }

    #[test]
    fn write_then_data_slice_matches_payload() {
        let pool = MacAudioFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        let payload = [0xAB_u8; 32];
        slot.write(&payload).expect("payload fits");
        assert_eq!(slot.data_slice(), &payload[..]);
        assert_eq!(slot.filled_len(), 32);
    }

    #[test]
    fn write_rejects_payload_larger_than_slot() {
        let pool = MacAudioFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        let big = [0u8; 128];
        let err = slot.write(&big).unwrap_err();
        assert!(matches!(err, MacAudioError::PayloadTooLarge { .. }));
    }

    #[test]
    fn append_concatenates_planar_to_interleaved_like_payload() {
        let pool = MacAudioFramePool::new(1, 32).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        let left = [1_u8, 2, 3, 4];
        let right = [5_u8, 6, 7, 8];
        slot.append(&left).expect("left fits");
        slot.append(&right).expect("right fits");
        assert_eq!(slot.data_slice(), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(slot.filled_len(), 8);
    }

    #[test]
    fn append_rejects_overflow() {
        let pool = MacAudioFramePool::new(1, 4).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        slot.append(&[1, 2, 3]).expect("3 of 4 fits");
        let err = slot.append(&[4, 5]).unwrap_err();
        assert!(matches!(err, MacAudioError::PayloadTooLarge { .. }));
    }

    #[test]
    fn recycled_slot_persists_buffer_storage_until_overwritten() {
        let pool = MacAudioFramePool::new(1, 16).expect("pool");
        let mut slot = pool.try_acquire().expect("first");
        let payload = [0x42_u8; 8];
        slot.write(&payload).expect("write");
        let observed_first = slot.data_slice().to_vec();
        drop(slot);
        let slot2 = pool.try_acquire().expect("recycled");
        assert_eq!(slot2.filled_len(), 0);
        assert_eq!(observed_first, payload.to_vec());
    }

    #[test]
    fn multi_thread_acquire_release_stress_8_threads_1000_ops_no_deadlock() {
        const THREADS: usize = 8;
        const OPS_PER_THREAD: usize = 1000;
        const POOL_CAP: usize = 4;

        let pool = Arc::new(MacAudioFramePool::new(POOL_CAP, 64).expect("pool"));
        let barrier = Arc::new(Barrier::new(THREADS));
        let acquired_obs = Arc::new(AtomicUsize::new(0));
        let dropped_obs = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::with_capacity(THREADS);
        for _ in 0..THREADS {
            let pool = Arc::clone(&pool);
            let barrier = Arc::clone(&barrier);
            let acquired_obs = Arc::clone(&acquired_obs);
            let dropped_obs = Arc::clone(&dropped_obs);
            handles.push(thread::spawn(move || {
                barrier.wait();
                let mut local_acq: usize = 0;
                let mut local_drop: usize = 0;
                for _ in 0..OPS_PER_THREAD {
                    match pool.try_acquire() {
                        Some(mut slot) => {
                            local_acq += 1;
                            let _ = slot.write(&[0xAA; 16]);
                            drop(slot);
                        }
                        None => {
                            local_drop += 1;
                        }
                    }
                }
                acquired_obs.fetch_add(local_acq, Ordering::Relaxed);
                dropped_obs.fetch_add(local_drop, Ordering::Relaxed);
            }));
        }

        let deadline = Instant::now() + Duration::from_secs(30);
        for h in handles {
            assert!(Instant::now() < deadline, "stress test exceeded 30s budget");
            h.join().expect("worker panicked");
        }

        let stats = pool.stats();
        let total = stats.acquired + stats.dropped;
        assert_eq!(total as usize, THREADS * OPS_PER_THREAD);
        assert_eq!(
            stats.acquired as usize,
            acquired_obs.load(Ordering::Relaxed)
        );
        assert_eq!(stats.dropped as usize, dropped_obs.load(Ordering::Relaxed));
        assert_eq!(stats.in_flight, 0);
        assert_eq!(stats.acquired, stats.released);
    }

    #[test]
    fn default_pool_dimensions_match_constants() {
        let pool = default_pool();
        assert_eq!(pool.capacity() as usize, MAC_AUDIO_POOL_CAP);
        assert_eq!(pool.bytes_per_slot() as usize, SLOT_BYTES);
    }

    #[test]
    fn stats_in_flight_matches_simultaneous_holders() {
        let pool = MacAudioFramePool::new(8, 64).expect("pool");
        let a = pool.try_acquire().expect("a");
        let b = pool.try_acquire().expect("b");
        let c = pool.try_acquire().expect("c");
        assert_eq!(pool.stats().in_flight, 3);
        drop(b);
        assert_eq!(pool.stats().in_flight, 2);
        drop(a);
        drop(c);
        assert_eq!(pool.stats().in_flight, 0);
    }

    #[test]
    fn into_external_parts_exposes_filled_pointer_and_length() {
        let pool = MacAudioFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        let payload = [0x5A_u8; 16];
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
        let pool = MacAudioFramePool::new(1, 32).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        slot.write(&[0xCC_u8; 8]).expect("write");
        let (_ptr, _len, owned) = slot.into_external_parts();
        assert!(pool.try_acquire().is_none());
        drop(owned);
        let revived = pool.try_acquire().expect("revived");
        assert_eq!(revived.filled_len(), 0);
        drop(revived);
        assert_eq!(pool.stats().in_flight, 0);
    }

    #[test]
    fn into_external_parts_holds_slot_across_send() {
        let pool = MacAudioFramePool::new(2, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        slot.write(&[0x77_u8; 32]).expect("write");
        let (ptr, len, owned) = slot.into_external_parts();
        let ptr_addr = ptr as usize;
        let handle = thread::spawn(move || {
            let owned = owned;
            assert_eq!(owned.filled_len(), 32);
            let observed = unsafe { core::slice::from_raw_parts(ptr_addr as *const u8, len) };
            assert_eq!(observed[0], 0x77);
            drop(owned);
        });
        handle.join().expect("worker");
        assert_eq!(pool.stats().in_flight, 0);
    }

    #[test]
    fn as_mut_ptr_returns_slot_base_address() {
        let pool = MacAudioFramePool::new(1, 64).expect("pool");
        let mut slot = pool.try_acquire().expect("slot");
        let payload = [0x11_u8; 8];
        slot.write(&payload).expect("write");
        let ptr = slot.as_mut_ptr();
        assert!(!ptr.is_null());
        unsafe { ptr.add(0).write(0x22) };
        unsafe { ptr.add(1).write(0x33) };
        assert_eq!(&slot.data_slice()[..2], &[0x22, 0x33]);
    }

    #[test]
    fn capacity_one_pool_round_trips() {
        let pool = MacAudioFramePool::new(1, 32).expect("pool");
        for _ in 0..5 {
            let mut slot = pool.try_acquire().expect("slot");
            slot.write(&[1, 2, 3]).expect("write");
            assert_eq!(slot.data_slice(), &[1, 2, 3]);
            drop(slot);
        }
        let stats = pool.stats();
        assert_eq!(stats.acquired, 5);
        assert_eq!(stats.released, 5);
        assert_eq!(stats.in_flight, 0);
    }
}
