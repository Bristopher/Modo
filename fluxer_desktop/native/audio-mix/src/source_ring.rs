// SPDX-License-Identifier: AGPL-3.0-or-later

use rtrb::{Consumer, Producer, RingBuffer};

use crate::AudioMixError;

pub const AUDIO_RING_CAP_FRAMES: usize = 4096;

pub const AUDIO_RING_CAP_FRAMES_MAX: usize = 1 << 20;

pub const AUDIO_OUTPUT_FRAMES: usize = 1024;

pub const AUDIO_SAMPLE_RATE_HZ_MIN: u32 = 8_000;
pub const AUDIO_SAMPLE_RATE_HZ_MAX: u32 = 384_000;

pub struct SourceRingProducer {
    inner: Producer<i16>,
    sample_rate_hz: u32,
    capacity_frames: usize,
    pushed_total: u64,
    dropped_total: u64,
}

pub struct SourceRingConsumer {
    inner: Consumer<i16>,
    sample_rate_hz: u32,
    capacity_frames: usize,
    drained_total: u64,
}

pub struct SourceRing;

impl SourceRing {
    pub fn create(
        capacity_frames: usize,
        sample_rate_hz: u32,
    ) -> Result<(SourceRingProducer, SourceRingConsumer), AudioMixError> {
        if capacity_frames == 0 {
            return Err(AudioMixError::ZeroCapacity);
        }
        if capacity_frames > AUDIO_RING_CAP_FRAMES_MAX {
            return Err(AudioMixError::CapacityExceedsLimit {
                requested: capacity_frames,
                limit: AUDIO_RING_CAP_FRAMES_MAX,
            });
        }
        if !(AUDIO_SAMPLE_RATE_HZ_MIN..=AUDIO_SAMPLE_RATE_HZ_MAX).contains(&sample_rate_hz) {
            return Err(AudioMixError::SampleRateOutOfRange { sample_rate_hz });
        }
        assert!(capacity_frames > 0);
        assert!(capacity_frames <= AUDIO_RING_CAP_FRAMES_MAX);
        let (producer, consumer) = RingBuffer::<i16>::new(capacity_frames);
        let producer = SourceRingProducer {
            inner: producer,
            sample_rate_hz,
            capacity_frames,
            pushed_total: 0,
            dropped_total: 0,
        };
        let consumer = SourceRingConsumer {
            inner: consumer,
            sample_rate_hz,
            capacity_frames,
            drained_total: 0,
        };
        assert_eq!(producer.sample_rate_hz, consumer.sample_rate_hz);
        assert_eq!(producer.capacity_frames, consumer.capacity_frames);
        Ok((producer, consumer))
    }

    pub fn create_default(
        sample_rate_hz: u32,
    ) -> Result<(SourceRingProducer, SourceRingConsumer), AudioMixError> {
        const { assert!(AUDIO_RING_CAP_FRAMES > 0) };
        const { assert!(AUDIO_RING_CAP_FRAMES <= AUDIO_RING_CAP_FRAMES_MAX) };
        SourceRing::create(AUDIO_RING_CAP_FRAMES, sample_rate_hz)
    }
}

impl SourceRingProducer {
    pub fn sample_rate_hz(&self) -> u32 {
        assert!(self.sample_rate_hz >= AUDIO_SAMPLE_RATE_HZ_MIN);
        assert!(self.sample_rate_hz <= AUDIO_SAMPLE_RATE_HZ_MAX);
        self.sample_rate_hz
    }

    pub fn capacity_frames(&self) -> usize {
        assert!(self.capacity_frames > 0);
        assert!(self.capacity_frames <= AUDIO_RING_CAP_FRAMES_MAX);
        self.capacity_frames
    }

    pub fn slots(&self) -> usize {
        let slots = self.inner.slots();
        assert!(slots <= self.capacity_frames);
        slots
    }

    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    pub fn try_push(&mut self, sample: i16) -> bool {
        assert!(self.capacity_frames > 0);
        assert!(self.pushed_total < u64::MAX);
        match self.inner.push(sample) {
            Ok(()) => {
                self.pushed_total = self.pushed_total.saturating_add(1);
                true
            }
            Err(_) => {
                self.dropped_total = self.dropped_total.saturating_add(1);
                false
            }
        }
    }

    pub fn try_push_slice(&mut self, samples: &[i16]) -> usize {
        assert!(samples.len() <= self.capacity_frames);
        assert!(self.pushed_total < u64::MAX - samples.len() as u64);
        let (pushed_slice, remainder) = self.inner.push_partial_slice(samples);
        let pushed = pushed_slice.len();
        let dropped = remainder.len();
        assert_eq!(pushed + dropped, samples.len());
        self.pushed_total = self.pushed_total.saturating_add(pushed as u64);
        self.dropped_total = self.dropped_total.saturating_add(dropped as u64);
        pushed
    }

    pub fn pushed_total(&self) -> u64 {
        self.pushed_total
    }

    pub fn dropped_total(&self) -> u64 {
        self.dropped_total
    }
}

impl SourceRingConsumer {
    pub fn sample_rate_hz(&self) -> u32 {
        assert!(self.sample_rate_hz >= AUDIO_SAMPLE_RATE_HZ_MIN);
        assert!(self.sample_rate_hz <= AUDIO_SAMPLE_RATE_HZ_MAX);
        self.sample_rate_hz
    }

    pub fn capacity_frames(&self) -> usize {
        assert!(self.capacity_frames > 0);
        assert!(self.capacity_frames <= AUDIO_RING_CAP_FRAMES_MAX);
        self.capacity_frames
    }

    pub fn slots(&self) -> usize {
        let slots = self.inner.slots();
        assert!(slots <= self.capacity_frames);
        slots
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn drain_into(&mut self, dest: &mut [i16]) -> usize {
        assert!(!dest.is_empty());
        assert!(dest.len() <= self.capacity_frames);
        let requested = dest.len();
        let (popped, remainder) = self.inner.pop_partial_slice(dest);
        let drained = popped.len();
        let leftover = remainder.len();
        assert_eq!(drained + leftover, requested);
        self.drained_total = self.drained_total.saturating_add(drained as u64);
        drained
    }

    pub fn drained_total(&self) -> u64 {
        self.drained_total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_capacity() {
        let err = SourceRing::create(0, 48_000).err();
        assert_eq!(err, Some(AudioMixError::ZeroCapacity));
    }

    #[test]
    fn rejects_capacity_over_limit() {
        let err = SourceRing::create(AUDIO_RING_CAP_FRAMES_MAX + 1, 48_000).err();
        assert!(matches!(
            err,
            Some(AudioMixError::CapacityExceedsLimit { .. })
        ));
    }

    #[test]
    fn rejects_sample_rate_below_min() {
        let err = SourceRing::create(AUDIO_RING_CAP_FRAMES, 4_000).err();
        assert!(matches!(
            err,
            Some(AudioMixError::SampleRateOutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_sample_rate_above_max() {
        let err = SourceRing::create(AUDIO_RING_CAP_FRAMES, 500_000).err();
        assert!(matches!(
            err,
            Some(AudioMixError::SampleRateOutOfRange { .. })
        ));
    }

    #[test]
    fn default_capacity_pair_matches() {
        let (producer, consumer) = SourceRing::create_default(48_000).expect("pair");
        assert_eq!(producer.sample_rate_hz(), 48_000);
        assert_eq!(consumer.sample_rate_hz(), 48_000);
        assert_eq!(producer.capacity_frames(), AUDIO_RING_CAP_FRAMES);
        assert_eq!(consumer.capacity_frames(), AUDIO_RING_CAP_FRAMES);
    }

    #[test]
    fn push_then_drain_roundtrips() {
        let (mut producer, mut consumer) = SourceRing::create(64, 48_000).expect("pair");
        let samples: Vec<i16> = (0..32).map(|n| n as i16).collect();
        let pushed = producer.try_push_slice(&samples);
        assert_eq!(pushed, 32);
        let mut buf = [0i16; 32];
        let drained = consumer.drain_into(&mut buf);
        assert_eq!(drained, 32);
        assert_eq!(&buf[..], &samples[..]);
        assert_eq!(producer.pushed_total(), 32);
        assert_eq!(consumer.drained_total(), 32);
    }

    #[test]
    fn try_push_returns_false_when_full_and_increments_dropped() {
        let (mut producer, _consumer) = SourceRing::create(2, 48_000).expect("pair");
        assert!(producer.try_push(1));
        assert!(producer.try_push(2));
        assert!(!producer.try_push(3));
        assert!(producer.is_full());
        assert_eq!(producer.dropped_total(), 1);
        assert_eq!(producer.pushed_total(), 2);
    }

    #[test]
    fn try_push_slice_partial_when_capacity_runs_out() {
        let (mut producer, _consumer) = SourceRing::create(4, 48_000).expect("pair");
        let pushed = producer.try_push_slice(&[10, 20, 30, 40]);
        assert_eq!(pushed, 4);
        let pushed_again = producer.try_push_slice(&[50, 60]);
        assert_eq!(pushed_again, 0);
        assert_eq!(producer.dropped_total(), 2);
    }

    #[test]
    fn drain_into_handles_empty_ring() {
        let (_producer, mut consumer) = SourceRing::create(64, 48_000).expect("pair");
        let mut buf = [0i16; 16];
        let drained = consumer.drain_into(&mut buf);
        assert_eq!(drained, 0);
        assert!(consumer.is_empty());
    }
}
