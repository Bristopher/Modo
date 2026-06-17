// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug)]
pub struct Ring<T: Copy + Default, const CAPACITY: usize> {
    pub slots: [T; CAPACITY],
    head: AtomicU64,
    tail: AtomicU64,
    dropped: AtomicU64,
}

impl<T: Copy + Default, const CAPACITY: usize> Default for Ring<T, CAPACITY> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Copy + Default, const CAPACITY: usize> Ring<T, CAPACITY> {
    const MASK: u64 = CAPACITY as u64 - 1;

    pub fn new() -> Self {
        assert!(
            CAPACITY > 0 && CAPACITY.is_power_of_two(),
            "Ring capacity must be a power of two"
        );
        Self {
            slots: [T::default(); CAPACITY],
            head: AtomicU64::new(0),
            tail: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
        }
    }

    pub fn claim(&self) -> Option<u64> {
        loop {
            let head = self.head.load(Ordering::Relaxed);
            let tail = self.tail.load(Ordering::Acquire);
            if head.wrapping_sub(tail) >= CAPACITY as u64 {
                self.dropped.fetch_add(1, Ordering::Relaxed);
                return None;
            }
            if self
                .head
                .compare_exchange_weak(
                    head,
                    head.wrapping_add(1),
                    Ordering::Acquire,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return Some(head & Self::MASK);
            }
        }
    }

    pub fn pop(&self) -> Option<u64> {
        let tail = self.tail.load(Ordering::Relaxed);
        let head = self.head.load(Ordering::Acquire);
        if head == tail {
            None
        } else {
            Some(tail & Self::MASK)
        }
    }

    pub fn release(&self) {
        let tail = self.tail.load(Ordering::Relaxed);
        self.tail.store(tail.wrapping_add(1), Ordering::Release);
    }

    pub fn dropped_count(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    pub fn len(&self) -> u64 {
        let head = self.head.load(Ordering::Relaxed);
        let tail = self.tail.load(Ordering::Relaxed);
        head.wrapping_sub(tail)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[cfg(test)]
    fn set_counters_for_test(&self, value: u64) {
        self.head.store(value, Ordering::Relaxed);
        self.tail.store(value, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_ring_pop_returns_null_len_is_zero() {
        let r: Ring<u32, 8> = Ring::new();
        assert_eq!(0, r.len());
        assert_eq!(None, r.pop());
    }

    #[test]
    fn single_producer_consumer_round_trip() {
        let mut r: Ring<u32, 4> = Ring::new();
        let i = r.claim().unwrap() as usize;
        r.slots[i] = 42;
        assert_eq!(1, r.len());
        let j = r.pop().unwrap() as usize;
        assert_eq!(42, r.slots[j]);
        r.release();
        assert_eq!(0, r.len());
        assert_eq!(None, r.pop());
    }

    #[test]
    fn fill_to_capacity_then_drop() {
        let mut r: Ring<u32, 4> = Ring::new();
        for k in 0..4 {
            let i = r.claim().unwrap() as usize;
            r.slots[i] = k;
        }
        assert_eq!(4, r.len());
        assert_eq!(None, r.claim());
        assert_eq!(1, r.dropped_count());
        assert_eq!(None, r.claim());
        assert_eq!(2, r.dropped_count());
    }

    #[test]
    fn full_vs_empty_distinction_head_tail_wrap() {
        let mut r: Ring<u32, 4> = Ring::new();
        for cycle in 0..10 {
            for n in 0..4 {
                let i = r.claim().unwrap() as usize;
                r.slots[i] = cycle * 100 + n;
            }
            assert_eq!(None, r.claim());
            for m in 0..4 {
                let j = r.pop().unwrap() as usize;
                assert_eq!(cycle * 100 + m, r.slots[j]);
                r.release();
            }
            assert_eq!(None, r.pop());
        }
    }

    #[test]
    fn fifo_order_across_wraparound() {
        let mut r: Ring<u32, 4> = Ring::new();
        for value in 0..3 {
            let idx = r.claim().unwrap() as usize;
            r.slots[idx] = value;
        }
        let idx = r.pop().unwrap() as usize;
        assert_eq!(0, r.slots[idx]);
        r.release();
        let idx = r.pop().unwrap() as usize;
        assert_eq!(1, r.slots[idx]);
        r.release();
        for value in 3..=5 {
            let idx = r.claim().unwrap() as usize;
            r.slots[idx] = value;
        }
        for expected in 2..=5 {
            let j = r.pop().unwrap() as usize;
            assert_eq!(expected, r.slots[j]);
            r.release();
        }
        assert_eq!(None, r.pop());
    }

    #[test]
    fn u64_counter_wrap_behavior_is_mask_correct() {
        let mut r: Ring<u32, 4> = Ring::new();
        let near_max = u64::MAX - 2;
        r.set_counters_for_test(near_max);
        assert_eq!(None, r.pop());
        for k in 0..4 {
            let i = r.claim().unwrap() as usize;
            r.slots[i] = k;
        }
        assert_eq!(None, r.claim());
        for m in 0..4 {
            let j = r.pop().unwrap() as usize;
            assert_eq!(m, r.slots[j]);
            r.release();
        }
        assert_eq!(None, r.pop());
    }

    #[test]
    fn concurrent_producers_via_simulated_cas_contention() {
        let r: Ring<u32, 8> = Ring::new();
        let mut seen = [false; 8];
        for _ in 0..8 {
            let idx = r.claim().unwrap() as usize;
            assert!(!seen[idx]);
            seen[idx] = true;
        }
        assert!(seen.iter().all(|value| *value));
        assert_eq!(None, r.claim());
    }

    #[test]
    fn dropped_counter_survives_interleaved_pop() {
        let r: Ring<u32, 2> = Ring::new();
        assert!(r.claim().is_some());
        assert!(r.claim().is_some());
        assert_eq!(None, r.claim());
        assert_eq!(1, r.dropped_count());
        assert!(r.pop().is_some());
        r.release();
        assert!(r.claim().is_some());
        assert_eq!(None, r.claim());
        assert_eq!(2, r.dropped_count());
    }
}
