// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::{AtomicU64, Ordering};

pub const SPEAKING_ATTACK_MS: u64 = 30;
pub const SPEAKING_RELEASE_MS_LOCAL: u64 = 180;
pub const SPEAKING_RELEASE_MS_REMOTE: u64 = 220;
pub const SPEAKING_HEARTBEAT_INTERVAL_MS: u64 = 1_000;
pub const SPEAKING_FRAME_TIMEOUT_MS: u64 = 250;
pub const SPEAKING_THRESHOLD_RMS_LOCAL_DEFAULT: f64 = 0.008;
pub const SPEAKING_THRESHOLD_RMS_REMOTE_DEFAULT: f64 = 0.006;
pub const SPEAKING_THRESHOLD_RMS_MIN: f64 = 0.000_1;
pub const SPEAKING_THRESHOLD_RMS_MAX: f64 = 0.5;
pub const SPEAKING_FRAME_SAMPLES_MAX: usize = 1 << 20;

const _: () = assert!(SPEAKING_ATTACK_MS < SPEAKING_RELEASE_MS_LOCAL);
const _: () = assert!(SPEAKING_ATTACK_MS < SPEAKING_RELEASE_MS_REMOTE);
const _: () = assert!(SPEAKING_RELEASE_MS_REMOTE < SPEAKING_HEARTBEAT_INTERVAL_MS);
const _: () = assert!(SPEAKING_FRAME_TIMEOUT_MS < SPEAKING_HEARTBEAT_INTERVAL_MS);

pub fn clamp_speaking_threshold_rms(threshold_rms: f64) -> f64 {
    if !threshold_rms.is_finite() {
        return SPEAKING_THRESHOLD_RMS_MIN;
    }
    threshold_rms.clamp(SPEAKING_THRESHOLD_RMS_MIN, SPEAKING_THRESHOLD_RMS_MAX)
}

pub struct SpeakingThresholds {
    local_rms_bits: AtomicU64,
    remote_rms_bits: AtomicU64,
}

impl SpeakingThresholds {
    pub fn new() -> Self {
        Self {
            local_rms_bits: AtomicU64::new(SPEAKING_THRESHOLD_RMS_LOCAL_DEFAULT.to_bits()),
            remote_rms_bits: AtomicU64::new(SPEAKING_THRESHOLD_RMS_REMOTE_DEFAULT.to_bits()),
        }
    }

    pub fn set(&self, local_rms: f64, remote_rms: f64) {
        let local = clamp_speaking_threshold_rms(local_rms);
        let remote = clamp_speaking_threshold_rms(remote_rms);
        assert!(local >= SPEAKING_THRESHOLD_RMS_MIN);
        assert!(remote >= SPEAKING_THRESHOLD_RMS_MIN);
        self.local_rms_bits
            .store(local.to_bits(), Ordering::Release);
        self.remote_rms_bits
            .store(remote.to_bits(), Ordering::Release);
    }

    pub fn local_rms(&self) -> f64 {
        let value = f64::from_bits(self.local_rms_bits.load(Ordering::Acquire));
        assert!(value.is_finite());
        value
    }

    pub fn remote_rms(&self) -> f64 {
        let value = f64::from_bits(self.remote_rms_bits.load(Ordering::Acquire));
        assert!(value.is_finite());
        value
    }
}

impl Default for SpeakingThresholds {
    fn default() -> Self {
        Self::new()
    }
}

pub fn frame_rms_i16(samples: &[i16]) -> f64 {
    assert!(!samples.is_empty());
    assert!(samples.len() <= SPEAKING_FRAME_SAMPLES_MAX);
    let mut sum_squares: f64 = 0.0;
    for sample in samples {
        let normalized = f64::from(*sample) / 32_768.0;
        sum_squares += normalized * normalized;
    }
    let rms = (sum_squares / samples.len() as f64).sqrt();
    assert!(rms.is_finite());
    assert!(rms >= 0.0);
    rms.min(1.0)
}

pub struct SpeakingGate {
    attack_ms: u64,
    release_ms: u64,
    speaking: bool,
    above_since_ms: Option<u64>,
    below_since_ms: Option<u64>,
    last_now_ms: u64,
}

impl SpeakingGate {
    pub fn new(attack_ms: u64, release_ms: u64) -> Self {
        assert!(attack_ms < release_ms);
        assert!(release_ms <= SPEAKING_HEARTBEAT_INTERVAL_MS);
        Self {
            attack_ms,
            release_ms,
            speaking: false,
            above_since_ms: None,
            below_since_ms: None,
            last_now_ms: 0,
        }
    }

    pub fn speaking(&self) -> bool {
        self.speaking
    }

    pub fn update(&mut self, rms: f64, threshold_rms: f64, now_ms: u64) -> Option<bool> {
        assert!(rms.is_finite());
        assert!(rms >= 0.0);
        assert!(threshold_rms >= SPEAKING_THRESHOLD_RMS_MIN);
        assert!(threshold_rms <= SPEAKING_THRESHOLD_RMS_MAX);
        assert!(now_ms >= self.last_now_ms);
        self.last_now_ms = now_ms;
        if rms >= threshold_rms {
            self.below_since_ms = None;
            let above_since_ms = *self.above_since_ms.get_or_insert(now_ms);
            if self.speaking {
                return None;
            }
            if now_ms - above_since_ms < self.attack_ms {
                return None;
            }
            self.speaking = true;
            return Some(true);
        }
        self.above_since_ms = None;
        let below_since_ms = *self.below_since_ms.get_or_insert(now_ms);
        if !self.speaking {
            return None;
        }
        if now_ms - below_since_ms < self.release_ms {
            return None;
        }
        self.speaking = false;
        Some(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESHOLD: f64 = 0.01;

    fn gate() -> SpeakingGate {
        SpeakingGate::new(SPEAKING_ATTACK_MS, SPEAKING_RELEASE_MS_LOCAL)
    }

    #[test]
    fn stays_quiet_below_threshold() {
        let mut gate = gate();
        for tick in 0..100u64 {
            assert_eq!(gate.update(0.001, THRESHOLD, tick * 10), None);
        }
        assert!(!gate.speaking());
    }

    #[test]
    fn attack_requires_sustained_signal() {
        let mut gate = gate();
        assert_eq!(gate.update(0.5, THRESHOLD, 0), None);
        assert_eq!(gate.update(0.5, THRESHOLD, 10), None);
        assert_eq!(gate.update(0.5, THRESHOLD, 20), None);
        assert_eq!(gate.update(0.5, THRESHOLD, 30), Some(true));
        assert!(gate.speaking());
    }

    #[test]
    fn single_frame_blip_does_not_trigger() {
        let mut gate = gate();
        assert_eq!(gate.update(0.5, THRESHOLD, 0), None);
        assert_eq!(gate.update(0.001, THRESHOLD, 10), None);
        assert_eq!(gate.update(0.5, THRESHOLD, 20), None);
        assert_eq!(gate.update(0.001, THRESHOLD, 30), None);
        assert!(!gate.speaking());
    }

    #[test]
    fn release_bridges_inter_word_gaps() {
        let mut gate = gate();
        for tick in 0..=3u64 {
            gate.update(0.5, THRESHOLD, tick * 10);
        }
        assert!(gate.speaking());
        for tick in 4..=20u64 {
            assert_eq!(gate.update(0.001, THRESHOLD, tick * 10), None);
        }
        assert!(gate.speaking());
        assert_eq!(gate.update(0.5, THRESHOLD, 210), None);
        assert!(gate.speaking());
    }

    #[test]
    fn release_fires_after_sustained_silence() {
        let mut gate = gate();
        for tick in 0..=3u64 {
            gate.update(0.5, THRESHOLD, tick * 10);
        }
        assert!(gate.speaking());
        assert_eq!(gate.update(0.001, THRESHOLD, 40), None);
        assert_eq!(gate.update(0.001, THRESHOLD, 219), None);
        assert_eq!(gate.update(0.001, THRESHOLD, 220), Some(false));
        assert!(!gate.speaking());
    }

    #[test]
    fn retrigger_after_release_needs_full_attack() {
        let mut gate = gate();
        for tick in 0..=3u64 {
            gate.update(0.5, THRESHOLD, tick * 10);
        }
        gate.update(0.001, THRESHOLD, 40);
        assert_eq!(gate.update(0.001, THRESHOLD, 220), Some(false));
        assert_eq!(gate.update(0.5, THRESHOLD, 230), None);
        assert_eq!(gate.update(0.5, THRESHOLD, 260), Some(true));
    }

    #[test]
    fn frame_rms_of_silence_is_zero() {
        let samples = [0i16; 480];
        assert_eq!(frame_rms_i16(&samples), 0.0);
    }

    #[test]
    fn frame_rms_of_full_scale_square_wave_is_one() {
        let mut samples = [i16::MIN; 480];
        for (index, sample) in samples.iter_mut().enumerate() {
            if index % 2 == 0 {
                *sample = i16::MAX;
            }
        }
        let rms = frame_rms_i16(&samples);
        assert!(rms > 0.999);
        assert!(rms <= 1.0);
    }

    #[test]
    fn frame_rms_scales_with_amplitude() {
        let loud = [8_192i16; 480];
        let quiet = [1_024i16; 480];
        assert!(frame_rms_i16(&loud) > frame_rms_i16(&quiet));
        assert!((frame_rms_i16(&loud) - 0.25).abs() < 0.001);
    }

    #[test]
    fn thresholds_default_and_clamp() {
        let thresholds = SpeakingThresholds::new();
        assert_eq!(thresholds.local_rms(), SPEAKING_THRESHOLD_RMS_LOCAL_DEFAULT);
        assert_eq!(
            thresholds.remote_rms(),
            SPEAKING_THRESHOLD_RMS_REMOTE_DEFAULT
        );
        thresholds.set(-1.0, f64::NAN);
        assert_eq!(thresholds.local_rms(), SPEAKING_THRESHOLD_RMS_MIN);
        assert_eq!(thresholds.remote_rms(), SPEAKING_THRESHOLD_RMS_MIN);
        thresholds.set(9.0, 0.02);
        assert_eq!(thresholds.local_rms(), SPEAKING_THRESHOLD_RMS_MAX);
        assert_eq!(thresholds.remote_rms(), 0.02);
    }

    #[test]
    fn gate_update_is_monotonic_in_time() {
        let mut gate = gate();
        gate.update(0.5, THRESHOLD, 100);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            gate.update(0.5, THRESHOLD, 50);
        }));
        assert!(result.is_err());
    }
}
