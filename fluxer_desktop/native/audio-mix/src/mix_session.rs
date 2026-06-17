// SPDX-License-Identifier: AGPL-3.0-or-later

use fluxer_rt_thread::TickInfo;

use crate::AudioMixError;
use crate::source_ring::{AUDIO_OUTPUT_FRAMES, SourceRingConsumer};

pub const MAX_MIX_SOURCES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTickStat {
    pub drained_count: u32,
    pub silence_count: u32,
}

#[derive(Debug)]
pub struct MixedFrame<'a> {
    pub samples: &'a [i16; AUDIO_OUTPUT_FRAMES],
    pub tick_index: u64,
    pub scheduled_ns: u64,
    pub actual_ns: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixTickResult {
    pub tick_index: u64,
    pub scheduled_ns: u64,
    pub actual_ns: u64,
    pub lag_ns: i64,
    pub total_drained: u64,
    pub total_silence: u64,
    pub saturated_samples: u32,
}

pub struct AudioMixSession {
    consumers: Vec<SourceRingConsumer>,
    per_source_stats: Vec<SourceTickStat>,
    sample_rate_hz: u32,
    mix_buffer_len: usize,
    accumulator: Box<[i32; AUDIO_OUTPUT_FRAMES]>,
    scratch: Box<[i16; AUDIO_OUTPUT_FRAMES]>,
    output: Box<[i16; AUDIO_OUTPUT_FRAMES]>,
    last_tick_index: Option<u64>,
    last_actual_ns: u64,
    ticks_completed: u64,
}

impl AudioMixSession {
    pub fn new(
        consumers: Vec<SourceRingConsumer>,
        mix_buffer_len: usize,
    ) -> Result<Self, AudioMixError> {
        if consumers.is_empty() {
            return Err(AudioMixError::ZeroSources);
        }
        if consumers.len() > MAX_MIX_SOURCES {
            return Err(AudioMixError::TooManySources {
                requested: consumers.len(),
                limit: MAX_MIX_SOURCES,
            });
        }
        if mix_buffer_len != AUDIO_OUTPUT_FRAMES {
            return Err(AudioMixError::MixBufferLenMismatch {
                expected: AUDIO_OUTPUT_FRAMES,
                observed: mix_buffer_len,
            });
        }
        let sample_rate_hz = consumers[0].sample_rate_hz();
        for consumer in &consumers {
            let observed = consumer.sample_rate_hz();
            if observed != sample_rate_hz {
                return Err(AudioMixError::SampleRateMismatch {
                    expected_hz: sample_rate_hz,
                    observed_hz: observed,
                });
            }
        }
        assert!(!consumers.is_empty());
        assert!(consumers.len() <= MAX_MIX_SOURCES);
        let per_source_stats = vec![
            SourceTickStat {
                drained_count: 0,
                silence_count: 0,
            };
            consumers.len()
        ];
        let accumulator = Box::new([0i32; AUDIO_OUTPUT_FRAMES]);
        let scratch = Box::new([0i16; AUDIO_OUTPUT_FRAMES]);
        let output = Box::new([0i16; AUDIO_OUTPUT_FRAMES]);
        Ok(Self {
            consumers,
            per_source_stats,
            sample_rate_hz,
            mix_buffer_len,
            accumulator,
            scratch,
            output,
            last_tick_index: None,
            last_actual_ns: 0,
            ticks_completed: 0,
        })
    }

    pub fn sample_rate_hz(&self) -> u32 {
        assert!(self.sample_rate_hz > 0);
        assert!(self.sample_rate_hz <= 384_000);
        self.sample_rate_hz
    }

    pub fn mix_buffer_len(&self) -> usize {
        assert_eq!(self.mix_buffer_len, AUDIO_OUTPUT_FRAMES);
        self.mix_buffer_len
    }

    pub fn source_count(&self) -> usize {
        assert!(!self.consumers.is_empty());
        assert!(self.consumers.len() <= MAX_MIX_SOURCES);
        self.consumers.len()
    }

    pub fn ticks_completed(&self) -> u64 {
        self.ticks_completed
    }

    pub fn per_source_stats(&self) -> &[SourceTickStat] {
        assert_eq!(self.per_source_stats.len(), self.consumers.len());
        &self.per_source_stats
    }

    pub fn tick(&mut self, tick_info: TickInfo) -> MixTickResult {
        assert!(!self.consumers.is_empty());
        assert!(tick_info.actual_ns >= self.last_actual_ns);
        if let Some(prior) = self.last_tick_index {
            assert!(tick_info.tick_index > prior);
        }
        for slot in self.accumulator.iter_mut() {
            *slot = 0;
        }
        let (total_drained, total_silence) = self.mix_all_sources();
        let saturated = self.finalise_output();
        self.ticks_completed = self.ticks_completed.saturating_add(1);
        self.last_tick_index = Some(tick_info.tick_index);
        self.last_actual_ns = tick_info.actual_ns;
        let result = MixTickResult {
            tick_index: tick_info.tick_index,
            scheduled_ns: tick_info.scheduled_ns,
            actual_ns: tick_info.actual_ns,
            lag_ns: tick_info.lag_ns,
            total_drained,
            total_silence,
            saturated_samples: saturated,
        };
        assert_eq!(result.tick_index, tick_info.tick_index);
        assert!(result.saturated_samples as usize <= AUDIO_OUTPUT_FRAMES);
        result
    }

    fn mix_all_sources(&mut self) -> (u64, u64) {
        let n = self.consumers.len();
        assert_eq!(self.per_source_stats.len(), n);
        let mut total_drained: u64 = 0;
        let mut total_silence: u64 = 0;
        for index in 0..n {
            let drained = self.consumers[index].drain_into(self.scratch.as_mut_slice());
            assert!(drained <= AUDIO_OUTPUT_FRAMES);
            let silence = AUDIO_OUTPUT_FRAMES - drained;
            let stat = &mut self.per_source_stats[index];
            stat.drained_count = stat.drained_count.saturating_add(drained as u32);
            stat.silence_count = stat.silence_count.saturating_add(silence as u32);
            total_drained = total_drained.saturating_add(drained as u64);
            total_silence = total_silence.saturating_add(silence as u64);
            for (acc, sample) in self.accumulator[..drained]
                .iter_mut()
                .zip(self.scratch[..drained].iter())
            {
                *acc = acc.saturating_add(*sample as i32);
            }
        }
        (total_drained, total_silence)
    }

    fn finalise_output(&mut self) -> u32 {
        let mut saturated: u32 = 0;
        for (out, acc) in self.output.iter_mut().zip(self.accumulator.iter()) {
            let value = *acc;
            let clamped = if value > i16::MAX as i32 {
                saturated = saturated.saturating_add(1);
                i16::MAX
            } else if value < i16::MIN as i32 {
                saturated = saturated.saturating_add(1);
                i16::MIN
            } else {
                value as i16
            };
            *out = clamped;
        }
        assert!(saturated as usize <= AUDIO_OUTPUT_FRAMES);
        saturated
    }

    pub fn last_output(&self) -> &[i16; AUDIO_OUTPUT_FRAMES] {
        assert_eq!(self.output.len(), AUDIO_OUTPUT_FRAMES);
        &self.output
    }

    pub fn last_mixed_frame(&self) -> Option<MixedFrame<'_>> {
        let tick_index = self.last_tick_index?;
        assert!(self.ticks_completed > 0);
        Some(MixedFrame {
            samples: &self.output,
            tick_index,
            scheduled_ns: 0,
            actual_ns: self.last_actual_ns,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_ring::SourceRing;
    use fluxer_rt_thread::TickInfo;
    use std::sync::{Arc, Barrier};
    use std::thread;

    fn synthetic_tick(index: u64) -> TickInfo {
        let scheduled_ns = index * 21_333_333;
        TickInfo {
            tick_index: index,
            scheduled_ns,
            actual_ns: scheduled_ns,
            lag_ns: 0,
        }
    }

    #[test]
    fn rejects_empty_consumer_list() {
        let err = AudioMixSession::new(Vec::new(), AUDIO_OUTPUT_FRAMES).err();
        assert_eq!(err, Some(AudioMixError::ZeroSources));
    }

    #[test]
    fn rejects_mismatched_sample_rates() {
        let (_p1, c1) = SourceRing::create(2048, 48_000).expect("pair 1");
        let (_p2, c2) = SourceRing::create(2048, 44_100).expect("pair 2");
        let err = AudioMixSession::new(vec![c1, c2], AUDIO_OUTPUT_FRAMES).err();
        assert!(matches!(
            err,
            Some(AudioMixError::SampleRateMismatch { .. })
        ));
    }

    #[test]
    fn rejects_wrong_mix_buffer_len() {
        let (_p, c) = SourceRing::create(2048, 48_000).expect("pair");
        let err = AudioMixSession::new(vec![c], 512).err();
        assert!(matches!(
            err,
            Some(AudioMixError::MixBufferLenMismatch { .. })
        ));
    }

    #[test]
    fn single_source_passes_through_samples() {
        let (mut producer, consumer) = SourceRing::create(2048, 48_000).expect("pair");
        let payload: Vec<i16> = (0..AUDIO_OUTPUT_FRAMES)
            .map(|n| (n as i16) % 1000)
            .collect();
        let pushed = producer.try_push_slice(&payload);
        assert_eq!(pushed, AUDIO_OUTPUT_FRAMES);
        let mut session =
            AudioMixSession::new(vec![consumer], AUDIO_OUTPUT_FRAMES).expect("session");
        let result = session.tick(synthetic_tick(0));
        assert_eq!(result.total_drained, AUDIO_OUTPUT_FRAMES as u64);
        assert_eq!(result.total_silence, 0);
        assert_eq!(&session.last_output()[..], &payload[..]);
    }

    #[test]
    fn empty_ring_produces_silence_without_blocking() {
        let (_producer, consumer) = SourceRing::create(2048, 48_000).expect("pair");
        let mut session =
            AudioMixSession::new(vec![consumer], AUDIO_OUTPUT_FRAMES).expect("session");
        let result = session.tick(synthetic_tick(0));
        assert_eq!(result.total_drained, 0);
        assert_eq!(result.total_silence, AUDIO_OUTPUT_FRAMES as u64);
        for sample in session.last_output().iter() {
            assert_eq!(*sample, 0);
        }
    }

    #[test]
    fn drained_count_accuracy_across_partial_fills() {
        let (mut producer, consumer) = SourceRing::create(2048, 48_000).expect("pair");
        let half: Vec<i16> = vec![100; AUDIO_OUTPUT_FRAMES / 2];
        producer.try_push_slice(&half);
        let mut session =
            AudioMixSession::new(vec![consumer], AUDIO_OUTPUT_FRAMES).expect("session");
        let result = session.tick(synthetic_tick(0));
        assert_eq!(result.total_drained, (AUDIO_OUTPUT_FRAMES / 2) as u64);
        assert_eq!(result.total_silence, (AUDIO_OUTPUT_FRAMES / 2) as u64);
        let stats = session.per_source_stats();
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].drained_count as usize, AUDIO_OUTPUT_FRAMES / 2);
        assert_eq!(stats[0].silence_count as usize, AUDIO_OUTPUT_FRAMES / 2);
    }

    #[test]
    fn opposite_phase_sources_sum_to_silence() {
        let (mut p1, c1) = SourceRing::create(2048, 48_000).expect("pair 1");
        let (mut p2, c2) = SourceRing::create(2048, 48_000).expect("pair 2");
        let wave: Vec<i16> = (0..AUDIO_OUTPUT_FRAMES)
            .map(|n| ((n as i16) % 1000) - 500)
            .collect();
        let inverse: Vec<i16> = wave.iter().map(|s| -*s).collect();
        p1.try_push_slice(&wave);
        p2.try_push_slice(&inverse);
        let mut session = AudioMixSession::new(vec![c1, c2], AUDIO_OUTPUT_FRAMES).expect("session");
        let _ = session.tick(synthetic_tick(0));
        for sample in session.last_output().iter() {
            assert_eq!(*sample, 0);
        }
    }

    #[test]
    fn saturation_clamps_at_i16_limits() {
        let (mut p1, c1) = SourceRing::create(2048, 48_000).expect("pair 1");
        let (mut p2, c2) = SourceRing::create(2048, 48_000).expect("pair 2");
        let high: Vec<i16> = vec![i16::MAX; AUDIO_OUTPUT_FRAMES];
        p1.try_push_slice(&high);
        p2.try_push_slice(&high);
        let mut session = AudioMixSession::new(vec![c1, c2], AUDIO_OUTPUT_FRAMES).expect("session");
        let result = session.tick(synthetic_tick(0));
        assert_eq!(result.saturated_samples as usize, AUDIO_OUTPUT_FRAMES);
        for sample in session.last_output().iter() {
            assert_eq!(*sample, i16::MAX);
        }
    }

    #[test]
    fn saturation_clamps_at_i16_min_limits() {
        let (mut p1, c1) = SourceRing::create(2048, 48_000).expect("pair 1");
        let (mut p2, c2) = SourceRing::create(2048, 48_000).expect("pair 2");
        let low: Vec<i16> = vec![i16::MIN; AUDIO_OUTPUT_FRAMES];
        p1.try_push_slice(&low);
        p2.try_push_slice(&low);
        let mut session = AudioMixSession::new(vec![c1, c2], AUDIO_OUTPUT_FRAMES).expect("session");
        let result = session.tick(synthetic_tick(0));
        assert_eq!(result.saturated_samples as usize, AUDIO_OUTPUT_FRAMES);
        for sample in session.last_output().iter() {
            assert_eq!(*sample, i16::MIN);
        }
    }

    #[test]
    fn full_ring_producer_drops_overflow() {
        let (mut producer, consumer) =
            SourceRing::create(AUDIO_OUTPUT_FRAMES, 48_000).expect("pair");
        let payload: Vec<i16> = vec![7; AUDIO_OUTPUT_FRAMES];
        let pushed_first = producer.try_push_slice(&payload);
        assert_eq!(pushed_first, AUDIO_OUTPUT_FRAMES);
        let pushed_second = producer.try_push_slice(&payload);
        assert_eq!(pushed_second, 0);
        assert_eq!(producer.dropped_total(), AUDIO_OUTPUT_FRAMES as u64);
        let mut session =
            AudioMixSession::new(vec![consumer], AUDIO_OUTPUT_FRAMES).expect("session");
        let result = session.tick(synthetic_tick(0));
        assert_eq!(result.total_drained, AUDIO_OUTPUT_FRAMES as u64);
    }

    #[test]
    fn ten_producer_threads_one_mixer_no_panic() {
        const SOURCE_COUNT: usize = 10;
        const TICKS: u64 = 50;
        let mut producers = Vec::with_capacity(SOURCE_COUNT);
        let mut consumers = Vec::with_capacity(SOURCE_COUNT);
        for _ in 0..SOURCE_COUNT {
            let (producer, consumer) = SourceRing::create(8192, 48_000).expect("pair");
            producers.push(producer);
            consumers.push(consumer);
        }
        let barrier = Arc::new(Barrier::new(SOURCE_COUNT + 1));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut handles = Vec::with_capacity(SOURCE_COUNT);
        for (index, mut producer) in producers.into_iter().enumerate() {
            let barrier = Arc::clone(&barrier);
            let stop = Arc::clone(&stop);
            handles.push(thread::spawn(move || {
                barrier.wait();
                let mut value: i16 = index as i16;
                while !stop.load(std::sync::atomic::Ordering::Acquire) {
                    let _ = producer.try_push(value);
                    value = value.wrapping_add(1);
                }
            }));
        }
        let mut session = AudioMixSession::new(consumers, AUDIO_OUTPUT_FRAMES).expect("session");
        barrier.wait();
        let mut last_drained: u64 = 0;
        for tick_index in 0..TICKS {
            let result = session.tick(synthetic_tick(tick_index));
            last_drained = last_drained.saturating_add(result.total_drained);
        }
        stop.store(true, std::sync::atomic::Ordering::Release);
        for handle in handles {
            handle.join().expect("producer join");
        }
        assert_eq!(session.ticks_completed(), TICKS);
        assert!(last_drained > 0);
    }
}
