// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(dead_code)]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use fluxer_audio_apm::{
    APM_MAX_FRAME_SAMPLES, ApmConfig, ApmError, AudioProcessor, StubAudioProcessor,
    expected_frame_samples,
};
use fluxer_audio_mix::{
    AUDIO_OUTPUT_FRAMES, AudioMixError, AudioMixSession, MAX_MIX_SOURCES, MixTickResult,
    SourceRing, SourceRingConsumer, SourceRingProducer,
};
use fluxer_audio_timing::{AudioTimingSmoother, SourceGainRamp, StaleSourceTracker};
use fluxer_rt_thread::{
    MonotonicClock, PriorityProfile, RealtimePriorityGuard, RtError, RtOutcome,
    SystemMonotonicClock, TickDriver,
};

use crate::ignore_audio_runtime::{
    IgnoreAudioPolicy, IgnoreAudioSourceResetEvent, IgnoreAudioSourceState, IgnoreAudioTick,
    compute_source_age_ns,
};

pub const MIX_TICK_PERIOD_NS: u64 = 21_333_333;

pub const SOURCE_RING_CAP_FRAMES: usize = 8_192;

pub const MIX_SAMPLE_RATE_HZ: u32 = 48_000;

pub const MIX_CHANNELS: u16 = 1;

pub const OUTPUT_BUFFER_POOL_DEPTH: usize = 4;

const NEVER_PUSHED_SENTINEL: u64 = u64::MAX;

pub struct DynMonotonicClock {
    inner: Arc<dyn MonotonicClock>,
}

impl DynMonotonicClock {
    pub fn new(inner: Arc<dyn MonotonicClock>) -> Self {
        Self { inner }
    }
}

impl MonotonicClock for DynMonotonicClock {
    fn now_ns(&self) -> u64 {
        self.inner.now_ns()
    }
}

#[derive(Debug)]
pub enum MixRuntimeError {
    NoSources,
    TooManySources { requested: usize, limit: usize },
    Mix(AudioMixError),
    Apm(ApmError),
    Rt(RtError),
    AlreadyRunning,
    ThreadSpawn,
}

impl core::fmt::Display for MixRuntimeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            MixRuntimeError::NoSources => write!(f, "mix runtime requires at least one source"),
            MixRuntimeError::TooManySources { requested, limit } => {
                write!(f, "source count {requested} exceeds limit {limit}")
            }
            MixRuntimeError::Mix(e) => write!(f, "{e}"),
            MixRuntimeError::Apm(e) => write!(f, "{e}"),
            MixRuntimeError::Rt(e) => write!(f, "{e}"),
            MixRuntimeError::AlreadyRunning => write!(f, "mix runtime already running"),
            MixRuntimeError::ThreadSpawn => write!(f, "failed to spawn mix thread"),
        }
    }
}

impl std::error::Error for MixRuntimeError {}

impl From<AudioMixError> for MixRuntimeError {
    fn from(value: AudioMixError) -> Self {
        MixRuntimeError::Mix(value)
    }
}

impl From<ApmError> for MixRuntimeError {
    fn from(value: ApmError) -> Self {
        MixRuntimeError::Apm(value)
    }
}

#[derive(Debug, Clone)]
pub struct MixOutputFrame {
    pub samples: Arc<[i16; AUDIO_OUTPUT_FRAMES]>,
    pub tick_index: u64,
    pub scheduled_ns: u64,
    pub actual_ns: u64,
    pub total_drained: u64,
    pub total_silence: u64,
    pub saturated_samples: u32,
}

impl PartialEq for MixOutputFrame {
    fn eq(&self, other: &Self) -> bool {
        self.tick_index == other.tick_index
            && self.scheduled_ns == other.scheduled_ns
            && self.actual_ns == other.actual_ns
            && self.total_drained == other.total_drained
            && self.total_silence == other.total_silence
            && self.saturated_samples == other.saturated_samples
            && self.samples.as_ref() == other.samples.as_ref()
    }
}

pub trait MixOutputSink: Send {
    fn on_mixed_frame(&mut self, frame: &MixOutputFrame);

    fn on_source_reset(&mut self, event: &IgnoreAudioSourceResetEvent);
}

pub struct NullMixOutputSink;

impl MixOutputSink for NullMixOutputSink {
    fn on_mixed_frame(&mut self, _frame: &MixOutputFrame) {}
    fn on_source_reset(&mut self, _event: &IgnoreAudioSourceResetEvent) {}
}

pub struct CapturedMixOutputSink {
    frames: Arc<Mutex<Vec<MixOutputFrame>>>,
    resets: Arc<Mutex<Vec<IgnoreAudioSourceResetEvent>>>,
}

impl CapturedMixOutputSink {
    pub fn new() -> Self {
        Self {
            frames: Arc::new(Mutex::new(Vec::new())),
            resets: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn frames(&self) -> Arc<Mutex<Vec<MixOutputFrame>>> {
        Arc::clone(&self.frames)
    }

    pub fn resets(&self) -> Arc<Mutex<Vec<IgnoreAudioSourceResetEvent>>> {
        Arc::clone(&self.resets)
    }
}

impl Default for CapturedMixOutputSink {
    fn default() -> Self {
        Self::new()
    }
}

impl MixOutputSink for CapturedMixOutputSink {
    fn on_mixed_frame(&mut self, frame: &MixOutputFrame) {
        if let Ok(mut guard) = self.frames.lock() {
            guard.push(frame.clone());
        }
    }

    fn on_source_reset(&mut self, event: &IgnoreAudioSourceResetEvent) {
        if let Ok(mut guard) = self.resets.lock() {
            guard.push(*event);
        }
    }
}

pub struct SourceFreshnessHandle {
    last_push_ns: Arc<AtomicU64>,
    registered_at_ns: u64,
    clock: Arc<dyn MonotonicClock>,
}

impl SourceFreshnessHandle {
    pub fn mark_push_now(&self) {
        let now = self.clock.now_ns();
        assert!(now > 0);
        self.last_push_ns.store(now, Ordering::Release);
    }

    pub fn mark_push_at(&self, push_ns: u64) {
        assert!(push_ns > 0);
        self.last_push_ns.store(push_ns, Ordering::Release);
    }

    pub fn last_push_ns(&self) -> u64 {
        self.last_push_ns.load(Ordering::Acquire)
    }

    pub fn registered_at_ns(&self) -> u64 {
        assert!(self.registered_at_ns != NEVER_PUSHED_SENTINEL);
        self.registered_at_ns
    }
}

pub struct CaptureSource {
    pub source_id: u64,
    pub producer: SourceRingProducer,
    pub apm: StubAudioProcessor,
    apm_sample_rate_hz: u32,
    apm_channels: u16,
    apm_frame_samples: usize,
    freshness: SourceFreshnessHandle,
}

impl CaptureSource {
    pub fn create(
        source_id: u64,
        sample_rate_hz: u32,
        channels: u16,
    ) -> Result<(Self, SourceRingConsumer), MixRuntimeError> {
        assert!(source_id != 0);
        assert!(sample_rate_hz >= 8_000);
        assert!(channels >= 1);
        let (producer, consumer) = SourceRing::create(SOURCE_RING_CAP_FRAMES, sample_rate_hz)?;
        let apm = StubAudioProcessor::new(ApmConfig::default(), sample_rate_hz, channels)?;
        let apm_frame_samples = expected_frame_samples(sample_rate_hz, channels);
        assert!(apm_frame_samples > 0);
        assert!(apm_frame_samples <= APM_MAX_FRAME_SAMPLES * (channels as usize));
        let clock_for_freshness: Arc<dyn MonotonicClock> = Arc::new(SystemMonotonicClock::new());
        let registered_at_ns = clock_for_freshness.now_ns().max(1);
        assert!(registered_at_ns != NEVER_PUSHED_SENTINEL);
        let freshness = SourceFreshnessHandle {
            last_push_ns: Arc::new(AtomicU64::new(NEVER_PUSHED_SENTINEL)),
            registered_at_ns,
            clock: clock_for_freshness,
        };
        let me = Self {
            source_id,
            producer,
            apm,
            apm_sample_rate_hz: sample_rate_hz,
            apm_channels: channels,
            apm_frame_samples,
            freshness,
        };
        Ok((me, consumer))
    }

    pub fn ingest(&mut self, samples: &mut [i16]) -> Result<usize, MixRuntimeError> {
        assert!(!samples.is_empty());
        assert_eq!(samples.len() % self.apm_frame_samples, 0);
        for chunk in samples.chunks_mut(self.apm_frame_samples) {
            self.apm
                .process_capture_frame(chunk, self.apm_sample_rate_hz, self.apm_channels)?;
        }
        let pushed = self.producer.try_push_slice(samples);
        self.freshness.mark_push_now();
        Ok(pushed)
    }

    pub fn ingest_skip_apm(&mut self, samples: &[i16]) -> usize {
        assert!(!samples.is_empty());
        let pushed = self.producer.try_push_slice(samples);
        self.freshness.mark_push_now();
        pushed
    }

    pub fn source_id(&self) -> u64 {
        assert!(self.source_id != 0);
        self.source_id
    }

    pub fn buffered_frames(&self) -> u64 {
        self.producer
            .capacity_frames()
            .saturating_sub(self.producer.slots()) as u64
    }

    pub fn freshness_handle(&self) -> &SourceFreshnessHandle {
        &self.freshness
    }

    pub fn last_push_ns_arc(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.freshness.last_push_ns)
    }

    pub fn apm_frames_processed(&self) -> u64 {
        self.apm.capture_frames_processed()
    }
}

struct BuilderSourceEntry {
    source_id: u64,
    consumer: SourceRingConsumer,
    last_push_ns: Arc<AtomicU64>,
    registered_at_ns: u64,
}

pub struct AudioMixRuntimeBuilder {
    consumers: Vec<BuilderSourceEntry>,
    sample_rate_hz: u32,
    period_ns: u64,
    clock: Arc<dyn MonotonicClock>,
}

impl AudioMixRuntimeBuilder {
    pub fn new() -> Self {
        Self {
            consumers: Vec::new(),
            sample_rate_hz: MIX_SAMPLE_RATE_HZ,
            period_ns: MIX_TICK_PERIOD_NS,
            clock: Arc::new(SystemMonotonicClock::new()),
        }
    }

    pub fn with_period_ns(mut self, period_ns: u64) -> Self {
        assert!(period_ns > 0);
        self.period_ns = period_ns;
        self
    }

    pub fn with_sample_rate_hz(mut self, sample_rate_hz: u32) -> Self {
        assert!(sample_rate_hz >= 8_000);
        self.sample_rate_hz = sample_rate_hz;
        self
    }

    pub fn with_clock(mut self, clock: Arc<dyn MonotonicClock>) -> Self {
        self.clock = clock;
        self
    }

    pub fn add_source(mut self, source_id: u64, consumer: SourceRingConsumer) -> Self {
        assert!(self.consumers.len() < MAX_MIX_SOURCES);
        assert!(source_id != 0);
        let placeholder = Arc::new(AtomicU64::new(NEVER_PUSHED_SENTINEL));
        let registered_at_ns = self.clock.now_ns().max(1);
        assert!(registered_at_ns != NEVER_PUSHED_SENTINEL);
        self.consumers.push(BuilderSourceEntry {
            source_id,
            consumer,
            last_push_ns: placeholder,
            registered_at_ns,
        });
        self
    }

    pub fn add_source_with_freshness(
        mut self,
        source_id: u64,
        consumer: SourceRingConsumer,
        last_push_ns: Arc<AtomicU64>,
    ) -> Self {
        assert!(self.consumers.len() < MAX_MIX_SOURCES);
        assert!(source_id != 0);
        let registered_at_ns = self.clock.now_ns().max(1);
        assert!(registered_at_ns != NEVER_PUSHED_SENTINEL);
        self.consumers.push(BuilderSourceEntry {
            source_id,
            consumer,
            last_push_ns,
            registered_at_ns,
        });
        self
    }

    pub fn build<S: MixOutputSink + 'static>(
        self,
        sink: S,
    ) -> Result<AudioMixRuntime, MixRuntimeError> {
        if self.consumers.is_empty() {
            return Err(MixRuntimeError::NoSources);
        }
        if self.consumers.len() > MAX_MIX_SOURCES {
            return Err(MixRuntimeError::TooManySources {
                requested: self.consumers.len(),
                limit: MAX_MIX_SOURCES,
            });
        }
        let consumer_count = self.consumers.len();
        let mut source_ids: Vec<u64> = Vec::with_capacity(consumer_count);
        let mut consumers: Vec<SourceRingConsumer> = Vec::with_capacity(consumer_count);
        let mut last_push_ns: Vec<Arc<AtomicU64>> = Vec::with_capacity(consumer_count);
        let mut registered_at_ns: Vec<u64> = Vec::with_capacity(consumer_count);
        let mut smoothers: Vec<AudioTimingSmoother> = Vec::with_capacity(consumer_count);
        let mut ramps: Vec<SourceGainRamp> = Vec::with_capacity(consumer_count);
        let mut stale_tracker = StaleSourceTracker::new(consumer_count)
            .map_err(|_| MixRuntimeError::Mix(AudioMixError::ZeroSources))?;
        for entry in self.consumers {
            source_ids.push(entry.source_id);
            consumers.push(entry.consumer);
            last_push_ns.push(entry.last_push_ns);
            registered_at_ns.push(entry.registered_at_ns);
            let smoother = AudioTimingSmoother::new(entry.source_id, self.sample_rate_hz)
                .map_err(|_| MixRuntimeError::Mix(AudioMixError::ZeroSources))?;
            smoothers.push(smoother);
            ramps.push(SourceGainRamp::new());
            stale_tracker
                .register_source(entry.source_id, entry.registered_at_ns)
                .map_err(|_| MixRuntimeError::Mix(AudioMixError::ZeroSources))?;
        }
        let session = AudioMixSession::new(consumers, AUDIO_OUTPUT_FRAMES)?;
        let policy = IgnoreAudioPolicy::new(self.sample_rate_hz)
            .map_err(|_| MixRuntimeError::Mix(AudioMixError::ZeroSources))?;
        let pool = build_output_pool();
        let last_marked_push_ns = vec![NEVER_PUSHED_SENTINEL; consumer_count];
        Ok(AudioMixRuntime {
            session: Some(session),
            policy: Some(policy),
            source_ids,
            last_push_ns,
            last_marked_push_ns,
            registered_at_ns,
            smoothers,
            ramps,
            stale_tracker,
            mark_pushed_total: Arc::new(AtomicU64::new(0)),
            clock: self.clock,
            sample_rate_hz: self.sample_rate_hz,
            period_ns: self.period_ns,
            sink: Some(Box::new(sink)),
            running: Arc::new(AtomicBool::new(false)),
            thread: None,
            rt_outcome: Arc::new(Mutex::new(None)),
            manual_tick_index: 0,
            pre_stats: [0u64; MAX_MIX_SOURCES],
            output_pool: pool,
            output_pool_cursor: 0,
        })
    }
}

impl Default for AudioMixRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn build_output_pool() -> [Arc<[i16; AUDIO_OUTPUT_FRAMES]>; OUTPUT_BUFFER_POOL_DEPTH] {
    [
        Arc::new([0i16; AUDIO_OUTPUT_FRAMES]),
        Arc::new([0i16; AUDIO_OUTPUT_FRAMES]),
        Arc::new([0i16; AUDIO_OUTPUT_FRAMES]),
        Arc::new([0i16; AUDIO_OUTPUT_FRAMES]),
    ]
}

pub struct AudioMixRuntime {
    session: Option<AudioMixSession>,
    policy: Option<IgnoreAudioPolicy>,
    source_ids: Vec<u64>,
    last_push_ns: Vec<Arc<AtomicU64>>,
    last_marked_push_ns: Vec<u64>,
    registered_at_ns: Vec<u64>,
    smoothers: Vec<AudioTimingSmoother>,
    ramps: Vec<SourceGainRamp>,
    stale_tracker: StaleSourceTracker,
    mark_pushed_total: Arc<AtomicU64>,
    clock: Arc<dyn MonotonicClock>,
    sample_rate_hz: u32,
    period_ns: u64,
    sink: Option<Box<dyn MixOutputSink>>,
    running: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    rt_outcome: Arc<Mutex<Option<MixRtOutcome>>>,
    manual_tick_index: u64,
    pre_stats: [u64; MAX_MIX_SOURCES],
    output_pool: [Arc<[i16; AUDIO_OUTPUT_FRAMES]>; OUTPUT_BUFFER_POOL_DEPTH],
    output_pool_cursor: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixRtOutcome {
    Acquired,
    PartialFallback,
    Denied,
}

impl AudioMixRuntime {
    pub fn source_count(&self) -> usize {
        assert!(!self.source_ids.is_empty());
        assert!(self.source_ids.len() <= MAX_MIX_SOURCES);
        self.source_ids.len()
    }

    pub fn sample_rate_hz(&self) -> u32 {
        assert!(self.sample_rate_hz >= 8_000);
        self.sample_rate_hz
    }

    pub fn period_ns(&self) -> u64 {
        assert!(self.period_ns > 0);
        self.period_ns
    }

    pub fn run_one_tick_blocking(
        &mut self,
        tick_at_ns: u64,
    ) -> Result<MixOutputFrame, MixRuntimeError> {
        let tick_index = self.manual_tick_index;
        self.manual_tick_index = self.manual_tick_index.saturating_add(1);
        let tick_info = fluxer_rt_thread::TickInfo {
            tick_index,
            scheduled_ns: tick_at_ns,
            actual_ns: tick_at_ns,
            lag_ns: 0,
        };
        self.run_tick_internal(tick_info)
    }

    pub fn observe_source_pushes_without_mix(
        &mut self,
        tick_at_ns: u64,
    ) -> Result<u64, MixRuntimeError> {
        assert!(tick_at_ns > 0);
        let before = self.mark_pushed_total.load(Ordering::Acquire);
        let session = self
            .session
            .as_mut()
            .ok_or(MixRuntimeError::AlreadyRunning)?;
        let policy = self
            .policy
            .as_mut()
            .ok_or(MixRuntimeError::AlreadyRunning)?;
        let sink = self.sink.as_mut().ok_or(MixRuntimeError::AlreadyRunning)?;
        let now_ns = self.clock.now_ns();
        snapshot_pre_stats(session, &mut self.pre_stats);
        drain_pushes_into_tracker(
            &mut self.stale_tracker,
            &self.source_ids,
            &self.last_push_ns,
            &mut self.last_marked_push_ns,
            &self.mark_pushed_total,
        );
        evaluate_policy_for_all_sources(
            policy,
            &self.source_ids,
            &self.last_push_ns,
            &self.registered_at_ns,
            &self.pre_stats,
            now_ns,
            tick_at_ns,
            self.period_ns,
            sink.as_mut(),
        );
        advance_smoothers_and_ramps(
            &mut self.smoothers,
            &mut self.ramps,
            &self.last_push_ns,
            tick_at_ns,
            self.period_ns,
            self.sample_rate_hz,
        );
        let after = self.mark_pushed_total.load(Ordering::Acquire);
        Ok(after.saturating_sub(before))
    }

    fn run_tick_internal(
        &mut self,
        tick_info: fluxer_rt_thread::TickInfo,
    ) -> Result<MixOutputFrame, MixRuntimeError> {
        let period_ns = self.period_ns;
        let session = self
            .session
            .as_mut()
            .ok_or(MixRuntimeError::AlreadyRunning)?;
        let policy = self
            .policy
            .as_mut()
            .ok_or(MixRuntimeError::AlreadyRunning)?;
        let sink = self.sink.as_mut().ok_or(MixRuntimeError::AlreadyRunning)?;
        let now_ns = self.clock.now_ns();
        snapshot_pre_stats(session, &mut self.pre_stats);
        drain_pushes_into_tracker(
            &mut self.stale_tracker,
            &self.source_ids,
            &self.last_push_ns,
            &mut self.last_marked_push_ns,
            &self.mark_pushed_total,
        );
        evaluate_policy_for_all_sources(
            policy,
            &self.source_ids,
            &self.last_push_ns,
            &self.registered_at_ns,
            &self.pre_stats,
            now_ns,
            tick_info.actual_ns,
            period_ns,
            sink.as_mut(),
        );
        advance_smoothers_and_ramps(
            &mut self.smoothers,
            &mut self.ramps,
            &self.last_push_ns,
            tick_info.actual_ns,
            period_ns,
            self.sample_rate_hz,
        );
        let result: MixTickResult = session.tick(tick_info);
        let frame = build_output_frame_pooled(
            session,
            &result,
            &mut self.output_pool,
            &mut self.output_pool_cursor,
        );
        sink.on_mixed_frame(&frame);
        Ok(frame)
    }

    pub fn mark_pushed_total(&self) -> u64 {
        assert!(!self.source_ids.is_empty());
        let value = self.mark_pushed_total.load(Ordering::Acquire);
        assert!(self.source_ids.len() <= MAX_MIX_SOURCES);
        value
    }

    pub fn mark_pushed_total_arc(&self) -> Arc<AtomicU64> {
        assert!(!self.source_ids.is_empty());
        assert!(self.source_ids.len() <= MAX_MIX_SOURCES);
        Arc::clone(&self.mark_pushed_total)
    }

    pub fn current_ramp_gain(&self, index: usize) -> f32 {
        assert!(index < self.ramps.len());
        self.ramps[index].current_gain()
    }

    pub fn smoother_initialised(&self, index: usize) -> bool {
        assert!(index < self.smoothers.len());
        self.smoothers[index].initialised()
    }

    pub fn is_source_stale(&self, index: usize, now_ns: u64, stale_threshold_ns: u64) -> bool {
        assert!(index < self.source_ids.len());
        assert!(self.source_ids.len() <= MAX_MIX_SOURCES);
        let id = self.source_ids[index];
        self.stale_tracker
            .is_stale_at(index, id, now_ns, stale_threshold_ns)
            .unwrap_or(false)
    }

    pub fn start(&mut self) -> Result<(), MixRuntimeError> {
        if self.running.swap(true, Ordering::AcqRel) {
            return Err(MixRuntimeError::AlreadyRunning);
        }
        let session = self.session.take().ok_or(MixRuntimeError::AlreadyRunning)?;
        let policy = self.policy.take().ok_or(MixRuntimeError::AlreadyRunning)?;
        let sink = self.sink.take().ok_or(MixRuntimeError::AlreadyRunning)?;
        let source_ids = self.source_ids.clone();
        let last_push_ns = self.last_push_ns.clone();
        let registered_at_ns = self.registered_at_ns.clone();
        let period_ns = self.period_ns;
        let sample_rate_hz = self.sample_rate_hz;
        let running = Arc::clone(&self.running);
        let rt_outcome = Arc::clone(&self.rt_outcome);
        let clock = Arc::clone(&self.clock);
        let pool = self.output_pool.clone();
        let smoothers = std::mem::take(&mut self.smoothers);
        let ramps = std::mem::take(&mut self.ramps);
        let stale_tracker = std::mem::replace(
            &mut self.stale_tracker,
            StaleSourceTracker::new(self.source_ids.len())
                .map_err(|_| MixRuntimeError::Mix(AudioMixError::ZeroSources))?,
        );
        let last_marked = std::mem::take(&mut self.last_marked_push_ns);
        let mark_pushed_total = Arc::clone(&self.mark_pushed_total);
        let handle = thread::Builder::new()
            .name("fluxer-audio-mix".into())
            .spawn(move || {
                spawn_mix_thread(MixThreadArgs {
                    session,
                    policy,
                    source_ids,
                    last_push_ns,
                    registered_at_ns,
                    smoothers,
                    ramps,
                    period_ns,
                    sample_rate_hz,
                    sink,
                    running,
                    rt_outcome,
                    clock,
                    pool,
                    stale_tracker,
                    last_marked,
                    mark_pushed_total,
                });
            })
            .map_err(|_| MixRuntimeError::ThreadSpawn)?;
        self.thread = Some(handle);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Release);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }

    pub fn rt_outcome(&self) -> Option<MixRtOutcome> {
        match self.rt_outcome.lock() {
            Ok(guard) => *guard,
            Err(_) => None,
        }
    }

    pub fn last_push_ns_for(&self, index: usize) -> u64 {
        assert!(index < self.last_push_ns.len());
        self.last_push_ns[index].load(Ordering::Acquire)
    }

    pub fn output_pool_depth(&self) -> usize {
        assert_eq!(self.output_pool.len(), OUTPUT_BUFFER_POOL_DEPTH);
        OUTPUT_BUFFER_POOL_DEPTH
    }
}

impl Drop for AudioMixRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}

struct MixThreadArgs {
    session: AudioMixSession,
    policy: IgnoreAudioPolicy,
    source_ids: Vec<u64>,
    last_push_ns: Vec<Arc<AtomicU64>>,
    registered_at_ns: Vec<u64>,
    smoothers: Vec<AudioTimingSmoother>,
    ramps: Vec<SourceGainRamp>,
    period_ns: u64,
    sample_rate_hz: u32,
    sink: Box<dyn MixOutputSink>,
    running: Arc<AtomicBool>,
    rt_outcome: Arc<Mutex<Option<MixRtOutcome>>>,
    clock: Arc<dyn MonotonicClock>,
    pool: [Arc<[i16; AUDIO_OUTPUT_FRAMES]>; OUTPUT_BUFFER_POOL_DEPTH],
    stale_tracker: StaleSourceTracker,
    last_marked: Vec<u64>,
    mark_pushed_total: Arc<AtomicU64>,
}

fn spawn_mix_thread(args: MixThreadArgs) {
    let MixThreadArgs {
        mut session,
        mut policy,
        source_ids,
        last_push_ns,
        registered_at_ns,
        mut smoothers,
        mut ramps,
        period_ns,
        sample_rate_hz,
        mut sink,
        running,
        rt_outcome,
        clock,
        pool,
        mut stale_tracker,
        mut last_marked,
        mark_pushed_total,
    } = args;
    assert!(!source_ids.is_empty());
    assert_eq!(source_ids.len(), last_push_ns.len());
    assert_eq!(source_ids.len(), registered_at_ns.len());
    assert_eq!(source_ids.len(), last_marked.len());
    let guard_result = RealtimePriorityGuard::acquire(PriorityProfile::Audio);
    let (rt_state, _rt_guard) = match guard_result {
        Ok(guard) => {
            let state = match guard.outcome() {
                RtOutcome::Acquired => MixRtOutcome::Acquired,
                RtOutcome::PartialFallback => MixRtOutcome::PartialFallback,
            };
            (state, Some(guard))
        }
        Err(_) => (MixRtOutcome::Denied, None),
    };
    if let Ok(mut guard_slot) = rt_outcome.lock() {
        *guard_slot = Some(rt_state);
    }
    let mut pre_stats = [0u64; MAX_MIX_SOURCES];
    let mut pool = pool;
    let mut cursor: usize = 0;
    let _ = run_mix_thread_loop(MixLoopArgs {
        session: &mut session,
        policy: &mut policy,
        source_ids: &source_ids,
        last_push_ns: &last_push_ns,
        registered_at_ns: &registered_at_ns,
        smoothers: &mut smoothers,
        ramps: &mut ramps,
        period_ns,
        sample_rate_hz,
        sink: sink.as_mut(),
        running: &running,
        clock: &clock,
        pre_stats: &mut pre_stats,
        pool: &mut pool,
        cursor: &mut cursor,
        stale_tracker: &mut stale_tracker,
        last_marked: &mut last_marked,
        mark_pushed_total: &mark_pushed_total,
    });
}

struct MixLoopArgs<'a> {
    session: &'a mut AudioMixSession,
    policy: &'a mut IgnoreAudioPolicy,
    source_ids: &'a [u64],
    last_push_ns: &'a [Arc<AtomicU64>],
    registered_at_ns: &'a [u64],
    smoothers: &'a mut [AudioTimingSmoother],
    ramps: &'a mut [SourceGainRamp],
    period_ns: u64,
    sample_rate_hz: u32,
    sink: &'a mut dyn MixOutputSink,
    running: &'a AtomicBool,
    clock: &'a Arc<dyn MonotonicClock>,
    pre_stats: &'a mut [u64; MAX_MIX_SOURCES],
    pool: &'a mut [Arc<[i16; AUDIO_OUTPUT_FRAMES]>; OUTPUT_BUFFER_POOL_DEPTH],
    cursor: &'a mut usize,
    stale_tracker: &'a mut StaleSourceTracker,
    last_marked: &'a mut [u64],
    mark_pushed_total: &'a Arc<AtomicU64>,
}

fn run_mix_thread_loop(args: MixLoopArgs<'_>) -> Result<(), MixRuntimeError> {
    let MixLoopArgs {
        session,
        policy,
        source_ids,
        last_push_ns,
        registered_at_ns,
        smoothers,
        ramps,
        period_ns,
        sample_rate_hz,
        sink,
        running,
        clock,
        pre_stats,
        pool,
        cursor,
        stale_tracker,
        last_marked,
        mark_pushed_total,
    } = args;
    assert!(!source_ids.is_empty());
    assert_eq!(source_ids.len(), last_push_ns.len());
    let driver_clock = Arc::new(DynMonotonicClock::new(Arc::clone(clock)));
    let mut driver = TickDriver::new(driver_clock, period_ns)
        .map_err(|_| MixRuntimeError::Mix(AudioMixError::ZeroSources))?;
    while running.load(Ordering::Acquire) {
        let tick_info = match driver.wait_until_next_tick() {
            Ok(info) => info,
            Err(_) => break,
        };
        if !running.load(Ordering::Acquire) {
            break;
        }
        let _ = run_single_tick(SingleTickArgs {
            session,
            policy,
            source_ids,
            last_push_ns,
            registered_at_ns,
            smoothers,
            ramps,
            period_ns,
            sample_rate_hz,
            tick_info,
            sink,
            clock,
            pre_stats,
            pool,
            cursor,
            stale_tracker,
            last_marked,
            mark_pushed_total,
        });
    }
    Ok(())
}

struct SingleTickArgs<'a> {
    session: &'a mut AudioMixSession,
    policy: &'a mut IgnoreAudioPolicy,
    source_ids: &'a [u64],
    last_push_ns: &'a [Arc<AtomicU64>],
    registered_at_ns: &'a [u64],
    smoothers: &'a mut [AudioTimingSmoother],
    ramps: &'a mut [SourceGainRamp],
    period_ns: u64,
    sample_rate_hz: u32,
    tick_info: fluxer_rt_thread::TickInfo,
    sink: &'a mut dyn MixOutputSink,
    clock: &'a Arc<dyn MonotonicClock>,
    pre_stats: &'a mut [u64; MAX_MIX_SOURCES],
    pool: &'a mut [Arc<[i16; AUDIO_OUTPUT_FRAMES]>; OUTPUT_BUFFER_POOL_DEPTH],
    cursor: &'a mut usize,
    stale_tracker: &'a mut StaleSourceTracker,
    last_marked: &'a mut [u64],
    mark_pushed_total: &'a Arc<AtomicU64>,
}

fn run_single_tick(args: SingleTickArgs<'_>) -> Result<MixOutputFrame, MixRuntimeError> {
    let SingleTickArgs {
        session,
        policy,
        source_ids,
        last_push_ns,
        registered_at_ns,
        smoothers,
        ramps,
        period_ns,
        sample_rate_hz,
        tick_info,
        sink,
        clock,
        pre_stats,
        pool,
        cursor,
        stale_tracker,
        last_marked,
        mark_pushed_total,
    } = args;
    assert!(!source_ids.is_empty());
    assert_eq!(source_ids.len(), session.source_count());
    assert_eq!(source_ids.len(), last_push_ns.len());
    let now_ns = clock.now_ns();
    snapshot_pre_stats(session, pre_stats);
    drain_pushes_into_tracker(
        stale_tracker,
        source_ids,
        last_push_ns,
        last_marked,
        mark_pushed_total,
    );
    evaluate_policy_for_all_sources(
        policy,
        source_ids,
        last_push_ns,
        registered_at_ns,
        pre_stats,
        now_ns,
        tick_info.actual_ns,
        period_ns,
        sink,
    );
    advance_smoothers_and_ramps(
        smoothers,
        ramps,
        last_push_ns,
        tick_info.actual_ns,
        period_ns,
        sample_rate_hz,
    );
    let result: MixTickResult = session.tick(tick_info);
    let frame = build_output_frame_pooled(session, &result, pool, cursor);
    sink.on_mixed_frame(&frame);
    Ok(frame)
}

fn drain_pushes_into_tracker(
    stale_tracker: &mut StaleSourceTracker,
    source_ids: &[u64],
    last_push_ns: &[Arc<AtomicU64>],
    last_marked: &mut [u64],
    mark_pushed_total: &Arc<AtomicU64>,
) {
    assert_eq!(source_ids.len(), last_push_ns.len());
    assert_eq!(source_ids.len(), last_marked.len());
    assert_eq!(source_ids.len(), stale_tracker.len());
    for idx in 0..source_ids.len() {
        let observed = last_push_ns[idx].load(Ordering::Acquire);
        if observed == NEVER_PUSHED_SENTINEL {
            continue;
        }
        if observed == last_marked[idx] {
            continue;
        }
        let source_id = source_ids[idx];
        if stale_tracker
            .mark_pushed_at(idx, source_id, observed)
            .is_ok()
        {
            last_marked[idx] = observed;
            mark_pushed_total.fetch_add(1, Ordering::AcqRel);
        }
    }
}

fn advance_smoothers_and_ramps(
    smoothers: &mut [AudioTimingSmoother],
    ramps: &mut [SourceGainRamp],
    last_push_ns: &[Arc<AtomicU64>],
    tick_at_ns: u64,
    period_ns: u64,
    sample_rate_hz: u32,
) {
    assert_eq!(smoothers.len(), ramps.len());
    assert_eq!(smoothers.len(), last_push_ns.len());
    let frames_per_tick = compute_frames_per_tick(period_ns, sample_rate_hz);
    for index in 0..smoothers.len() {
        let last_push = last_push_ns[index].load(Ordering::Acquire);
        ramps[index].advance_tick();
        if last_push == NEVER_PUSHED_SENTINEL {
            continue;
        }
        let smoother = &mut smoothers[index];
        let input = fluxer_audio_timing::SmootherInput {
            media_ts_ns: last_push,
            frames: frames_per_tick,
            wall_ns: tick_at_ns,
        };
        let _ = smoother.adjust(input);
    }
}

fn compute_frames_per_tick(period_ns: u64, sample_rate_hz: u32) -> u32 {
    assert!(period_ns > 0);
    assert!(sample_rate_hz >= 8_000);
    let product = (period_ns as u128) * (sample_rate_hz as u128);
    let frames = product / 1_000_000_000u128;
    let frames_u32 = frames.min(u32::MAX as u128) as u32;
    frames_u32.max(1)
}

fn snapshot_pre_stats(session: &AudioMixSession, pre_stats: &mut [u64; MAX_MIX_SOURCES]) {
    let stats = session.per_source_stats();
    assert!(stats.len() <= MAX_MIX_SOURCES);
    for (slot, stat) in pre_stats.iter_mut().zip(stats.iter()) {
        *slot = stat.drained_count as u64;
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_policy_for_all_sources(
    policy: &mut IgnoreAudioPolicy,
    source_ids: &[u64],
    last_push_ns: &[Arc<AtomicU64>],
    registered_at_ns: &[u64],
    pre_stats: &[u64; MAX_MIX_SOURCES],
    now_ns: u64,
    tick_at_ns: u64,
    period_ns: u64,
    sink: &mut dyn MixOutputSink,
) {
    assert_eq!(source_ids.len(), last_push_ns.len());
    assert_eq!(source_ids.len(), registered_at_ns.len());
    let tick = IgnoreAudioTick {
        at_ns: tick_at_ns,
        period_ns,
    };
    for (idx, source_id) in source_ids.iter().enumerate() {
        let drained_before = pre_stats[idx];
        let last_push = last_push_ns[idx].load(Ordering::Acquire);
        let age_ns = compute_age_ns(last_push, registered_at_ns[idx], now_ns);
        let state = IgnoreAudioSourceState {
            id: *source_id,
            buffered_frames: drained_before,
            last_frame_age_ns: age_ns,
            is_muted: false,
        };
        if let Ok(eval) = policy.evaluate(&state, tick)
            && let Some(event) = eval.event
        {
            sink.on_source_reset(&event);
        }
    }
}

fn compute_age_ns(last_push_ns: u64, registered_at_ns: u64, now_ns: u64) -> u64 {
    compute_source_age_ns(last_push_ns, registered_at_ns, now_ns)
}

fn build_output_frame_pooled(
    session: &AudioMixSession,
    result: &MixTickResult,
    pool: &mut [Arc<[i16; AUDIO_OUTPUT_FRAMES]>; OUTPUT_BUFFER_POOL_DEPTH],
    cursor: &mut usize,
) -> MixOutputFrame {
    let slot_index = *cursor;
    assert!(slot_index < OUTPUT_BUFFER_POOL_DEPTH);
    let src = session.last_output();
    let target_arc = &mut pool[slot_index];
    let target = Arc::make_mut(target_arc);
    target.copy_from_slice(src);
    assert_eq!(target.len(), AUDIO_OUTPUT_FRAMES);
    *cursor = (slot_index + 1) % OUTPUT_BUFFER_POOL_DEPTH;
    MixOutputFrame {
        samples: Arc::clone(&pool[slot_index]),
        tick_index: result.tick_index,
        scheduled_ns: result.scheduled_ns,
        actual_ns: result.actual_ns,
        total_drained: result.total_drained,
        total_silence: result.total_silence,
        saturated_samples: result.saturated_samples,
    }
}

pub fn audio_apm_frame_samples(sample_rate_hz: u32, channels: u16) -> usize {
    assert!(sample_rate_hz >= 8_000);
    assert!(channels >= 1);
    expected_frame_samples(sample_rate_hz, channels)
}

#[cfg(test)]
pub(crate) static ALLOC_PROBE: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
thread_local! {
    pub(crate) static THREAD_ALLOC_TRACK: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    pub(crate) static THREAD_ALLOC_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
struct CountingAllocator;

#[cfg(test)]
unsafe impl std::alloc::GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        ALLOC_PROBE.fetch_add(1, Ordering::Relaxed);
        let _ = THREAD_ALLOC_TRACK.try_with(|tracked| {
            if tracked.get() {
                let _ = THREAD_ALLOC_COUNT.try_with(|c| c.set(c.get().saturating_add(1)));
            }
        });
        unsafe { std::alloc::System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe { std::alloc::System.dealloc(ptr, layout) }
    }
}

#[cfg(test)]
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

#[cfg(test)]
pub(crate) fn begin_thread_alloc_probe() {
    THREAD_ALLOC_COUNT.with(|c| c.set(0));
    THREAD_ALLOC_TRACK.with(|t| t.set(true));
}

#[cfg(test)]
pub(crate) fn end_thread_alloc_probe() -> usize {
    THREAD_ALLOC_TRACK.with(|t| t.set(false));
    THREAD_ALLOC_COUNT.with(|c| c.get())
}

#[cfg(test)]
mod tests {
    use super::*;

    use fluxer_rt_thread::TickInfo;

    fn snapshot_allocs() -> usize {
        ALLOC_PROBE.load(Ordering::Relaxed)
    }

    fn begin_thread_alloc_count() {
        begin_thread_alloc_probe();
    }

    fn end_thread_alloc_count() -> usize {
        end_thread_alloc_probe()
    }

    fn build_test_runtime(
        source_count: usize,
    ) -> (Vec<CaptureSource>, AudioMixRuntime, CapturedMixOutputSink) {
        assert!(source_count > 0);
        let mut sources: Vec<CaptureSource> = Vec::with_capacity(source_count);
        let mut builder = AudioMixRuntimeBuilder::new();
        for n in 0..source_count {
            let (source, consumer) =
                CaptureSource::create(n as u64 + 1, MIX_SAMPLE_RATE_HZ, MIX_CHANNELS)
                    .expect("source");
            let push_ns = source.last_push_ns_arc();
            builder = builder.add_source_with_freshness(n as u64 + 1, consumer, push_ns);
            sources.push(source);
        }
        let sink = CapturedMixOutputSink::new();
        let cloned_frames = sink.frames();
        let cloned_resets = sink.resets();
        let runtime = builder
            .build(CapturedMixOutputSink {
                frames: cloned_frames,
                resets: cloned_resets,
            })
            .expect("build");
        (sources, runtime, sink)
    }

    fn make_pure_tone(amplitude: i16, len: usize) -> Vec<i16> {
        assert!(len > 0);
        (0..len).map(|n| (n as i16 % 8) * amplitude / 8).collect()
    }

    #[derive(Debug)]
    struct FakeClock {
        value_ns: AtomicU64,
    }

    impl FakeClock {
        fn new(initial_ns: u64) -> Self {
            Self {
                value_ns: AtomicU64::new(initial_ns),
            }
        }
        fn set(&self, value_ns: u64) {
            self.value_ns.store(value_ns, Ordering::Release);
        }
    }

    impl MonotonicClock for FakeClock {
        fn now_ns(&self) -> u64 {
            self.value_ns.load(Ordering::Acquire)
        }
    }

    fn build_runtime_with_clock(
        source_count: usize,
        clock: Arc<dyn MonotonicClock>,
    ) -> (Vec<CaptureSource>, AudioMixRuntime, CapturedMixOutputSink) {
        assert!(source_count > 0);
        let mut sources: Vec<CaptureSource> = Vec::with_capacity(source_count);
        let mut builder = AudioMixRuntimeBuilder::new().with_clock(clock);
        for n in 0..source_count {
            let (source, consumer) =
                CaptureSource::create(n as u64 + 1, MIX_SAMPLE_RATE_HZ, MIX_CHANNELS)
                    .expect("source");
            let push_ns = source.last_push_ns_arc();
            builder = builder.add_source_with_freshness(n as u64 + 1, consumer, push_ns);
            sources.push(source);
        }
        let sink = CapturedMixOutputSink::new();
        let cloned_frames = sink.frames();
        let cloned_resets = sink.resets();
        let runtime = builder
            .build(CapturedMixOutputSink {
                frames: cloned_frames,
                resets: cloned_resets,
            })
            .expect("build");
        (sources, runtime, sink)
    }

    #[test]
    fn realtime_priority_guard_acquires_and_releases() {
        let guard = RealtimePriorityGuard::acquire(PriorityProfile::Audio);
        match guard {
            Ok(g) => {
                let is_valid = matches!(
                    g.outcome(),
                    RtOutcome::Acquired | RtOutcome::PartialFallback
                );
                drop(g);
                assert!(is_valid);
            }
            Err(RtError::PlatformDenied(_)) => {}
            Err(other) => panic!("unexpected guard error: {other:?}"),
        }
    }

    #[test]
    fn source_ring_produce_consume_1000_frames_deterministic() {
        let (producer, mut consumer) = SourceRing::create(2048, 48_000).expect("pair");
        let mut producer = producer;
        let payload: Vec<i16> = (0..1000).map(|n| (n as i16 % 4096) - 2048).collect();
        let pushed = producer.try_push_slice(&payload);
        assert_eq!(pushed, 1000);
        let mut buf = vec![0i16; 1000];
        let drained = consumer.drain_into(&mut buf);
        assert_eq!(drained, 1000);
        assert_eq!(buf, payload);
    }

    #[test]
    fn audio_mix_session_with_eight_pure_tone_sources_sums_to_saturated() {
        let (mut sources, mut runtime, sink) = build_test_runtime(8);
        let payload = vec![i16::MAX / 4; AUDIO_OUTPUT_FRAMES];
        for source in sources.iter_mut() {
            let pushed = source.ingest_skip_apm(&payload);
            assert_eq!(pushed, AUDIO_OUTPUT_FRAMES);
        }
        let frame = runtime.run_one_tick_blocking(0).expect("frame");
        assert_eq!(frame.samples.len(), AUDIO_OUTPUT_FRAMES);
        for sample in frame.samples.iter() {
            assert!(*sample > 0);
        }
        let frames_arc = sink.frames();
        let frames_guard = frames_arc.lock().expect("lock");
        assert_eq!(frames_guard.len(), 1);
    }

    #[test]
    fn tick_driver_maintains_cumulative_drift_free_schedule_over_100_ticks() {
        let clock = Arc::new(SystemMonotonicClock::new());
        let period_ns = 1_000_000;
        let mut driver = TickDriver::new(clock, period_ns).expect("driver");
        for _ in 0..100 {
            let info = driver.wait_until_next_tick().expect("tick");
            assert!(info.actual_ns >= info.scheduled_ns.saturating_sub(period_ns));
        }
        assert!(driver.next_tick_index() >= 100);
    }

    #[test]
    fn apm_stub_is_noop_on_capture_frame_but_counts() {
        let mut source = CaptureSource::create(1, MIX_SAMPLE_RATE_HZ, MIX_CHANNELS)
            .expect("src")
            .0;
        let original = make_pure_tone(1000, source.apm_frame_samples * 2);
        let mut samples = original.clone();
        let before = source.apm_frames_processed();
        source.ingest(&mut samples).expect("ingest");
        let after = source.apm_frames_processed();
        assert_eq!(samples, original);
        assert_eq!(after - before, 2);
    }

    #[test]
    fn ignore_audio_policy_reset_triggered_after_threshold() {
        use crate::ignore_audio_runtime::{
            IgnoreAudioDecision, IgnoreAudioSourceState, IgnoreAudioTick,
            SOURCE_RESET_AFTER_BUFFERED_TICKS,
        };
        let mut policy = IgnoreAudioPolicy::new(48_000).expect("ok");
        let state = IgnoreAudioSourceState {
            id: 1,
            buffered_frames: (SOURCE_RESET_AFTER_BUFFERED_TICKS as u64) * 1024 + 1,
            last_frame_age_ns: 0,
            is_muted: false,
        };
        let eval = policy
            .evaluate(
                &state,
                IgnoreAudioTick {
                    at_ns: 1,
                    period_ns: 21_333_333,
                },
            )
            .expect("ok");
        assert_eq!(eval.decision, IgnoreAudioDecision::ResetSource);
    }

    #[test]
    fn empty_source_yields_silence_frame() {
        let (_sources, mut runtime, _sink) = build_test_runtime(1);
        let frame = runtime.run_one_tick_blocking(0).expect("frame");
        for sample in frame.samples.iter() {
            assert_eq!(*sample, 0);
        }
        assert_eq!(frame.total_drained, 0);
        assert_eq!(frame.total_silence, AUDIO_OUTPUT_FRAMES as u64);
    }

    #[test]
    fn determinism_same_input_same_output() {
        let payload: Vec<i16> = (0..AUDIO_OUTPUT_FRAMES)
            .map(|n| (n as i16 % 1000) - 500)
            .collect();
        let mut outputs: Vec<Vec<i16>> = Vec::new();
        for _ in 0..2 {
            let (mut sources, mut runtime, _sink) = build_test_runtime(2);
            for source in sources.iter_mut() {
                let _ = source.ingest_skip_apm(&payload);
            }
            let frame = runtime.run_one_tick_blocking(0).expect("frame");
            outputs.push(frame.samples.to_vec());
        }
        assert_eq!(outputs[0], outputs[1]);
    }

    #[test]
    fn skipped_tick_increments_silence_count_on_empty_ring() {
        let (_sources, mut runtime, _sink) = build_test_runtime(1);
        let frame = runtime.run_one_tick_blocking(0).expect("frame");
        assert_eq!(frame.total_silence, AUDIO_OUTPUT_FRAMES as u64);
    }

    #[test]
    fn end_to_end_one_source_drained_to_mix_session() {
        let (mut sources, mut runtime, sink) = build_test_runtime(1);
        let payload: Vec<i16> = (0..AUDIO_OUTPUT_FRAMES)
            .map(|n| (n as i16 % 2000) - 1000)
            .collect();
        let pushed = sources[0].ingest_skip_apm(&payload);
        assert_eq!(pushed, AUDIO_OUTPUT_FRAMES);
        let _ = runtime.run_one_tick_blocking(0).expect("frame");
        let frames = sink.frames();
        let frames_guard = frames.lock().expect("lock");
        assert_eq!(frames_guard.len(), 1);
        assert_eq!(frames_guard[0].samples.as_ref(), &payload[..]);
    }

    #[test]
    fn build_rejects_zero_sources() {
        let sink = NullMixOutputSink;
        let err = AudioMixRuntimeBuilder::new().build(sink).err();
        assert!(matches!(err, Some(MixRuntimeError::NoSources)));
    }

    #[test]
    fn capture_source_buffered_frames_grows_on_ingest() {
        let mut source = CaptureSource::create(1, MIX_SAMPLE_RATE_HZ, MIX_CHANNELS)
            .expect("src")
            .0;
        let initial = source.buffered_frames();
        assert_eq!(initial, 0);
        let payload = vec![5i16; 256];
        let pushed = source.ingest_skip_apm(&payload);
        assert_eq!(pushed, 256);
        assert!(source.buffered_frames() > 0);
    }

    #[test]
    fn run_single_tick_lap_through_eight_sources_executes() {
        let (mut sources, mut runtime, _sink) = build_test_runtime(8);
        for (idx, source) in sources.iter_mut().enumerate() {
            let payload = vec![(idx as i16) * 100; AUDIO_OUTPUT_FRAMES];
            let _ = source.ingest_skip_apm(&payload);
        }
        let frame = runtime.run_one_tick_blocking(0).expect("frame");
        assert!(frame.total_drained > 0);
    }

    #[allow(clippy::no_effect_underscore_binding)]
    #[test]
    fn tick_info_synthetic_can_be_consumed() {
        let _ti = TickInfo {
            tick_index: 0,
            scheduled_ns: 0,
            actual_ns: 0,
            lag_ns: 0,
        };
    }

    #[test]
    fn ingest_increments_apm_frame_count_per_chunk() {
        let mut source = CaptureSource::create(1, MIX_SAMPLE_RATE_HZ, MIX_CHANNELS)
            .expect("src")
            .0;
        assert_eq!(source.apm_frames_processed(), 0);
        let chunks = 7;
        let mut samples = vec![0i16; source.apm_frame_samples * chunks];
        source.ingest(&mut samples).expect("ingest");
        assert_eq!(source.apm_frames_processed(), chunks as u64);
    }

    #[test]
    fn stale_source_triggered_via_clock_emits_reset() {
        let clock = Arc::new(FakeClock::new(1_000_000));
        let (sources, mut runtime, sink) =
            build_runtime_with_clock(1, Arc::clone(&clock) as Arc<dyn MonotonicClock>);
        sources[0].freshness.mark_push_at(1_000_000);
        clock.set(1_000_000 + 10_000_000_000);
        let _ = runtime.run_one_tick_blocking(0).expect("frame");
        let resets = sink.resets();
        let guard = resets.lock().expect("lock");
        assert!(!guard.is_empty());
        assert!(
            guard
                .iter()
                .any(|e| e.reason
                    == crate::ignore_audio_runtime::IgnoreAudioResetReason::StaleSource)
        );
    }

    #[test]
    fn last_frame_age_monotonic_within_quiescence() {
        let clock = Arc::new(FakeClock::new(1_000_000));
        let (sources, mut runtime, _sink) =
            build_runtime_with_clock(1, Arc::clone(&clock) as Arc<dyn MonotonicClock>);
        sources[0].freshness.mark_push_at(1_000_000);
        let mut prior = 0u64;
        for n in 0..5 {
            clock.set(1_000_000 + (n + 1) * 1_000_000);
            let _ = runtime.run_one_tick_blocking(0).expect("frame");
            let push = runtime.last_push_ns_for(0);
            let now = clock.now_ns();
            let age = compute_age_ns(push, 1_000_000, now);
            assert!(age >= prior);
            prior = age;
        }
    }

    #[test]
    fn output_pool_buffers_rotate_modulo_depth_when_consumer_releases() {
        let (mut sources, mut runtime) = build_null_runtime(1);
        let payload = vec![123i16; AUDIO_OUTPUT_FRAMES];
        let _ = sources[0].ingest_skip_apm(&payload);
        let f0 = runtime.run_one_tick_blocking(0).expect("frame");
        let p0 = Arc::as_ptr(&f0.samples);
        drop(f0);
        let _ = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS)
            .expect("frame");
        let _ = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS * 2)
            .expect("frame");
        let _ = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS * 3)
            .expect("frame");
        let f4 = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS * 4)
            .expect("frame");
        let p4 = Arc::as_ptr(&f4.samples);
        assert_eq!(p0, p4);
    }

    #[test]
    fn output_pool_returns_distinct_buffers_when_consumer_retains() {
        let (mut sources, mut runtime, _sink) = build_test_runtime(1);
        let payload = vec![7i16; AUDIO_OUTPUT_FRAMES];
        let _ = sources[0].ingest_skip_apm(&payload);
        let f0 = runtime.run_one_tick_blocking(0).expect("frame");
        let f1 = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS)
            .expect("frame");
        let f2 = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS * 2)
            .expect("frame");
        let f3 = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS * 3)
            .expect("frame");
        let p0 = Arc::as_ptr(&f0.samples);
        let p1 = Arc::as_ptr(&f1.samples);
        let p2 = Arc::as_ptr(&f2.samples);
        let p3 = Arc::as_ptr(&f3.samples);
        assert!(p0 != p1);
        assert!(p1 != p2);
        assert!(p2 != p3);
        assert_eq!(f0.samples.len(), AUDIO_OUTPUT_FRAMES);
    }

    #[test]
    fn output_arc_samples_length_invariant_holds() {
        let (mut sources, mut runtime, _sink) = build_test_runtime(1);
        let payload = vec![1i16; AUDIO_OUTPUT_FRAMES];
        let _ = sources[0].ingest_skip_apm(&payload);
        let frame = runtime.run_one_tick_blocking(0).expect("frame");
        assert_eq!(frame.samples.len(), AUDIO_OUTPUT_FRAMES);
        assert_eq!(frame.samples.as_ref().len(), 1024);
    }

    fn build_null_runtime(source_count: usize) -> (Vec<CaptureSource>, AudioMixRuntime) {
        assert!(source_count > 0);
        let mut sources: Vec<CaptureSource> = Vec::with_capacity(source_count);
        let mut builder = AudioMixRuntimeBuilder::new();
        for n in 0..source_count {
            let (source, consumer) =
                CaptureSource::create(n as u64 + 1, MIX_SAMPLE_RATE_HZ, MIX_CHANNELS)
                    .expect("source");
            let push_ns = source.last_push_ns_arc();
            builder = builder.add_source_with_freshness(n as u64 + 1, consumer, push_ns);
            sources.push(source);
        }
        let runtime = builder.build(NullMixOutputSink).expect("build");
        (sources, runtime)
    }

    #[test]
    fn tick_steady_state_does_not_allocate() {
        let (mut sources, mut runtime) = build_null_runtime(4);
        let payload = vec![10i16; AUDIO_OUTPUT_FRAMES];
        for _warm in 0..OUTPUT_BUFFER_POOL_DEPTH + 2 {
            for source in sources.iter_mut() {
                let _ = source.ingest_skip_apm(&payload);
            }
            let _ = runtime.run_one_tick_blocking(0).expect("warmup");
        }
        for source in sources.iter_mut() {
            let _ = source.ingest_skip_apm(&payload);
        }
        begin_thread_alloc_count();
        let _ = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS * 10)
            .expect("frame");
        let allocations = end_thread_alloc_count();
        assert_eq!(
            allocations, 0,
            "steady-state tick allocated {allocations} times",
        );
        let _ = snapshot_allocs();
    }

    #[test]
    fn freshness_handle_records_push_time() {
        let source = CaptureSource::create(7, MIX_SAMPLE_RATE_HZ, MIX_CHANNELS)
            .expect("src")
            .0;
        let initial = source.freshness_handle().last_push_ns();
        assert_eq!(initial, NEVER_PUSHED_SENTINEL);
        source.freshness_handle().mark_push_at(42);
        assert_eq!(source.freshness_handle().last_push_ns(), 42);
    }

    #[test]
    fn compute_age_ns_uses_registration_for_never_pushed_source() {
        let age = compute_age_ns(NEVER_PUSHED_SENTINEL, 1_000, 5_000);
        assert_eq!(age, 4_000);
    }

    #[test]
    fn compute_age_ns_returns_zero_when_now_before_push() {
        let age = compute_age_ns(1_000, 500, 500);
        assert_eq!(age, 0);
    }

    #[test]
    fn compute_age_ns_returns_difference_after_push() {
        let age = compute_age_ns(1_000, 0, 5_000);
        assert_eq!(age, 4_000);
    }

    #[test]
    fn ramp_advances_per_tick_through_runtime() {
        let (mut sources, mut runtime, _sink) = build_test_runtime(1);
        let payload = vec![100i16; AUDIO_OUTPUT_FRAMES];
        let _ = sources[0].ingest_skip_apm(&payload);
        assert_eq!(runtime.current_ramp_gain(0), 0.0);
        let _ = runtime.run_one_tick_blocking(0).expect("frame");
        let g1 = runtime.current_ramp_gain(0);
        assert!(g1 > 0.0);
        for n in 1..6u64 {
            let _ = runtime
                .run_one_tick_blocking(n * MIX_TICK_PERIOD_NS)
                .expect("frame");
        }
        let g_after = runtime.current_ramp_gain(0);
        assert_eq!(g_after, 1.0);
    }

    #[test]
    fn smoother_initialises_after_first_push() {
        let (mut sources, mut runtime, _sink) = build_test_runtime(1);
        assert!(!runtime.smoother_initialised(0));
        let payload = vec![1i16; AUDIO_OUTPUT_FRAMES];
        let _ = sources[0].ingest_skip_apm(&payload);
        let _ = runtime.run_one_tick_blocking(0).expect("frame");
        assert!(runtime.smoother_initialised(0));
    }

    #[test]
    fn registered_never_pushed_source_eligible_for_stale_after_threshold() {
        let clock = Arc::new(FakeClock::new(1_000_000));
        let (_sources, runtime, _sink) =
            build_runtime_with_clock(1, Arc::clone(&clock) as Arc<dyn MonotonicClock>);
        clock.set(1_000_000 + 6_000_000_000);
        let stale = runtime.is_source_stale(0, 1_000_000 + 6_000_000_000, 5_000_000_000);
        assert!(stale);
    }

    #[test]
    fn tick_marks_pushed_when_freshness_atomic_advances() {
        let (mut sources, mut runtime, _sink) = build_test_runtime(1);
        assert_eq!(runtime.mark_pushed_total(), 0);
        let payload = vec![100i16; AUDIO_OUTPUT_FRAMES];
        let pushed = sources[0].ingest_skip_apm(&payload);
        assert_eq!(pushed, AUDIO_OUTPUT_FRAMES);
        let _ = runtime.run_one_tick_blocking(0).expect("frame");
        assert!(runtime.mark_pushed_total() >= 1);
        let observed_push = sources[0].freshness_handle().last_push_ns();
        assert_ne!(observed_push, u64::MAX);
        let stale = runtime.is_source_stale(0, observed_push + 1_000_000, 5_000_000_000);
        assert!(!stale);
    }

    #[test]
    fn tick_does_not_double_count_unchanged_push_atomic() {
        let (mut sources, mut runtime, _sink) = build_test_runtime(1);
        let payload = vec![50i16; AUDIO_OUTPUT_FRAMES];
        let _ = sources[0].ingest_skip_apm(&payload);
        let _ = runtime.run_one_tick_blocking(0).expect("frame");
        let first_total = runtime.mark_pushed_total();
        assert!(first_total >= 1);
        let _ = runtime
            .run_one_tick_blocking(MIX_TICK_PERIOD_NS)
            .expect("frame");
        let second_total = runtime.mark_pushed_total();
        assert_eq!(first_total, second_total);
    }
}
