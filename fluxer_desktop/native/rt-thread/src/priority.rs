// SPDX-License-Identifier: AGPL-3.0-or-later

use core::cell::Cell;
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityProfile {
    Audio,
    Video,
    Network,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RtError {
    AlreadyAcquired,
    PlatformDenied(i32),
    Unsupported,
}

impl fmt::Display for RtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RtError::AlreadyAcquired => {
                write!(f, "RealtimePriorityGuard already acquired on this thread")
            }
            RtError::PlatformDenied(errno) => {
                write!(f, "platform denied RT priority (errno={errno})")
            }
            RtError::Unsupported => write!(f, "RT priority not supported on this platform/profile"),
        }
    }
}

impl std::error::Error for RtError {}

#[derive(Debug, PartialEq, Eq)]
pub enum RtOutcome {
    Acquired,
    PartialFallback,
}

thread_local! {
    static GUARD_ACTIVE: Cell<bool> = const { Cell::new(false) };
}

pub struct RealtimePriorityGuard {
    inner: PlatformGuard,
    outcome: RtOutcome,
}

impl RealtimePriorityGuard {
    pub fn acquire(profile: PriorityProfile) -> Result<Self, RtError> {
        let already = GUARD_ACTIVE.with(|c| c.replace(true));
        if already {
            GUARD_ACTIVE.with(|c| c.set(true));
            return Err(RtError::AlreadyAcquired);
        }
        let acquired = PlatformGuard::acquire(profile);
        match acquired {
            Ok((inner, outcome)) => {
                assert!(matches!(
                    outcome,
                    RtOutcome::Acquired | RtOutcome::PartialFallback
                ));
                Ok(Self { inner, outcome })
            }
            Err(e) => {
                GUARD_ACTIVE.with(|c| c.set(false));
                Err(e)
            }
        }
    }

    pub fn outcome(&self) -> &RtOutcome {
        assert!(GUARD_ACTIVE.with(Cell::get));
        &self.outcome
    }
}

impl Drop for RealtimePriorityGuard {
    fn drop(&mut self) {
        self.inner.restore();
        GUARD_ACTIVE.with(|c| c.set(false));
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::{PriorityProfile, RtError, RtOutcome};
    use libc::qos_class_t::{
        QOS_CLASS_DEFAULT, QOS_CLASS_USER_INITIATED, QOS_CLASS_USER_INTERACTIVE, QOS_CLASS_UTILITY,
    };

    pub struct PlatformGuard {
        prior_qos: libc::qos_class_t,
        prior_relative: i32,
    }

    impl PlatformGuard {
        pub fn acquire(profile: PriorityProfile) -> Result<(Self, RtOutcome), RtError> {
            assert!(matches!(
                profile,
                PriorityProfile::Audio | PriorityProfile::Video | PriorityProfile::Network
            ));
            let mut prior_qos: libc::qos_class_t = QOS_CLASS_DEFAULT;
            let mut prior_relative: i32 = 0;
            let read_rc = unsafe {
                libc::pthread_get_qos_class_np(
                    libc::pthread_self(),
                    &mut prior_qos,
                    &mut prior_relative,
                )
            };
            if read_rc != 0 {
                return Err(RtError::PlatformDenied(read_rc));
            }
            let target = map_profile(profile);
            let set_rc = unsafe { libc::pthread_set_qos_class_self_np(target, 0) };
            if set_rc != 0 {
                return Err(RtError::PlatformDenied(set_rc));
            }
            Ok((
                Self {
                    prior_qos,
                    prior_relative,
                },
                RtOutcome::Acquired,
            ))
        }

        pub fn restore(&mut self) {
            unsafe {
                let _ = libc::pthread_set_qos_class_self_np(self.prior_qos, self.prior_relative);
            }
        }
    }

    fn map_profile(profile: PriorityProfile) -> libc::qos_class_t {
        match profile {
            PriorityProfile::Audio => QOS_CLASS_USER_INTERACTIVE,
            PriorityProfile::Video => QOS_CLASS_USER_INITIATED,
            PriorityProfile::Network => QOS_CLASS_UTILITY,
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use super::{PriorityProfile, RtError, RtOutcome};

    pub struct PlatformGuard {
        prior_policy: i32,
        prior_param: libc::sched_param,
        used_fallback: bool,
        prior_nice: Option<i32>,
    }

    impl PlatformGuard {
        pub fn acquire(profile: PriorityProfile) -> Result<(Self, RtOutcome), RtError> {
            assert!(matches!(
                profile,
                PriorityProfile::Audio | PriorityProfile::Video | PriorityProfile::Network
            ));
            let mut prior_policy: i32 = 0;
            let mut prior_param: libc::sched_param = unsafe { core::mem::zeroed() };
            let read_rc = unsafe {
                libc::pthread_getschedparam(
                    libc::pthread_self(),
                    &mut prior_policy,
                    &mut prior_param,
                )
            };
            if read_rc != 0 {
                return Err(RtError::PlatformDenied(read_rc));
            }
            let (policy, sched_priority) = map_profile(profile);
            assert!(sched_priority >= 0);
            let mut param: libc::sched_param = unsafe { core::mem::zeroed() };
            param.sched_priority = sched_priority;
            let set_rc =
                unsafe { libc::pthread_setschedparam(libc::pthread_self(), policy, &param) };
            if set_rc == 0 {
                return Ok((
                    Self {
                        prior_policy,
                        prior_param,
                        used_fallback: false,
                        prior_nice: None,
                    },
                    RtOutcome::Acquired,
                ));
            }
            if set_rc != libc::EPERM {
                return Err(RtError::PlatformDenied(set_rc));
            }
            let fallback_nice = match profile {
                PriorityProfile::Audio => -19,
                PriorityProfile::Video => -10,
                PriorityProfile::Network => -5,
            };
            let prior_nice = unsafe {
                *libc::__errno_location() = 0;
                let cur = libc::getpriority(libc::PRIO_PROCESS, 0);
                let saved_errno = *libc::__errno_location();
                if cur == -1 && saved_errno != 0 {
                    return Err(RtError::PlatformDenied(saved_errno));
                }
                cur
            };
            let _ = unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, fallback_nice) };
            Ok((
                Self {
                    prior_policy,
                    prior_param,
                    used_fallback: true,
                    prior_nice: Some(prior_nice),
                },
                RtOutcome::PartialFallback,
            ))
        }

        pub fn restore(&mut self) {
            if self.used_fallback {
                if let Some(prior_nice) = self.prior_nice {
                    unsafe { libc::setpriority(libc::PRIO_PROCESS, 0, prior_nice) };
                }
            } else {
                unsafe {
                    let _ = libc::pthread_setschedparam(
                        libc::pthread_self(),
                        self.prior_policy,
                        &self.prior_param,
                    );
                }
            }
        }
    }

    fn map_profile(profile: PriorityProfile) -> (i32, i32) {
        let policy = libc::SCHED_FIFO;
        let max = unsafe { libc::sched_get_priority_max(policy) };
        let min = unsafe { libc::sched_get_priority_min(policy) };
        assert!(max >= min);
        let span = (max - min).max(1);
        let prio = match profile {
            PriorityProfile::Audio => max,
            PriorityProfile::Video => min + (span * 2) / 3,
            PriorityProfile::Network => min + span / 3,
        };
        (policy, prio)
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::{PriorityProfile, RtError, RtOutcome};
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::Threading::{
        AvRevertMmThreadCharacteristics, AvSetMmThreadCharacteristicsW,
    };
    use windows::core::PCWSTR;

    pub struct PlatformGuard {
        handle: HANDLE,
    }

    impl PlatformGuard {
        pub fn acquire(profile: PriorityProfile) -> Result<(Self, RtOutcome), RtError> {
            assert!(matches!(
                profile,
                PriorityProfile::Audio | PriorityProfile::Video | PriorityProfile::Network
            ));
            let task_name: &[u16] = match profile {
                PriorityProfile::Audio => &AUDIO_W,
                PriorityProfile::Video => &CAPTURE_W,
                PriorityProfile::Network => &PLAYBACK_W,
            };
            let mut task_index: u32 = 0;
            let handle = unsafe {
                AvSetMmThreadCharacteristicsW(PCWSTR(task_name.as_ptr()), &mut task_index)
            };
            match handle {
                Ok(h) if !h.is_invalid() => Ok((Self { handle: h }, RtOutcome::Acquired)),
                Ok(_) => Err(RtError::PlatformDenied(0)),
                Err(e) => Err(RtError::PlatformDenied(e.code().0)),
            }
        }

        pub fn restore(&mut self) {
            if !self.handle.is_invalid() {
                unsafe {
                    let _ = AvRevertMmThreadCharacteristics(self.handle);
                }
            }
        }
    }

    const AUDIO_W: [u16; 6] = [
        b'A' as u16,
        b'u' as u16,
        b'd' as u16,
        b'i' as u16,
        b'o' as u16,
        0,
    ];
    const CAPTURE_W: [u16; 8] = [
        b'C' as u16,
        b'a' as u16,
        b'p' as u16,
        b't' as u16,
        b'u' as u16,
        b'r' as u16,
        b'e' as u16,
        0,
    ];
    const PLAYBACK_W: [u16; 9] = [
        b'P' as u16,
        b'l' as u16,
        b'a' as u16,
        b'y' as u16,
        b'b' as u16,
        b'a' as u16,
        b'c' as u16,
        b'k' as u16,
        0,
    ];
}

#[cfg(not(any(unix, target_os = "windows")))]
mod platform {
    use super::{PriorityProfile, RtError, RtOutcome};

    pub struct PlatformGuard {}

    impl PlatformGuard {
        pub fn acquire(_profile: PriorityProfile) -> Result<(Self, RtOutcome), RtError> {
            Err(RtError::Unsupported)
        }

        pub fn restore(&mut self) {}
    }
}

use platform::PlatformGuard;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_and_release_audio_does_not_panic() {
        let guard = RealtimePriorityGuard::acquire(PriorityProfile::Audio);
        match guard {
            Ok(g) => {
                assert!(matches!(
                    *g.outcome(),
                    RtOutcome::Acquired | RtOutcome::PartialFallback
                ));
                drop(g);
            }
            Err(RtError::PlatformDenied(_)) => {}
            Err(other) => panic!("unexpected error: {other:?}"),
        }
        let again = RealtimePriorityGuard::acquire(PriorityProfile::Audio);
        if let Ok(g) = again {
            drop(g);
        }
    }

    #[test]
    fn double_acquire_on_same_thread_errors() {
        let first = RealtimePriorityGuard::acquire(PriorityProfile::Audio);
        if let Ok(first_guard) = first {
            let second = RealtimePriorityGuard::acquire(PriorityProfile::Audio);
            assert_eq!(second.err(), Some(RtError::AlreadyAcquired));
            drop(first_guard);
            let third = RealtimePriorityGuard::acquire(PriorityProfile::Network);
            if let Ok(t) = third {
                drop(t);
            }
        }
    }

    #[test]
    fn all_profiles_round_trip() {
        for profile in [
            PriorityProfile::Audio,
            PriorityProfile::Video,
            PriorityProfile::Network,
        ] {
            let g = RealtimePriorityGuard::acquire(profile);
            match g {
                Ok(g) => drop(g),
                Err(RtError::PlatformDenied(_)) => {}
                Err(other) => panic!("unexpected error for {profile:?}: {other:?}"),
            }
        }
    }
}
