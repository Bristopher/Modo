// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum State {
    Idle = 0,
    Starting = 1,
    Running = 2,
    Stopping = 3,
    Stopped = 4,
}

impl State {
    fn from_raw(raw: u32) -> Self {
        match raw {
            0 => Self::Idle,
            1 => Self::Starting,
            2 => Self::Running,
            3 => Self::Stopping,
            4 => Self::Stopped,
            _ => Self::Stopped,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    IllegalTransition,
    DoubleStart,
    StopBeforeStart,
    StartWhileStopping,
}

pub fn is_allowed(from: State, to: State) -> bool {
    match from {
        State::Idle => matches!(to, State::Starting | State::Stopped),
        State::Starting => matches!(to, State::Running | State::Stopped),
        State::Running => matches!(to, State::Stopping | State::Stopped),
        State::Stopping => to == State::Stopped,
        State::Stopped => false,
    }
}

#[derive(Debug)]
pub struct Machine {
    state: AtomicU32,
}

impl Default for Machine {
    fn default() -> Self {
        Self::new()
    }
}

impl Machine {
    pub fn new() -> Self {
        Self {
            state: AtomicU32::new(State::Idle as u32),
        }
    }

    pub fn current(&self) -> State {
        State::from_raw(self.state.load(Ordering::Acquire))
    }

    fn cas(&self, from: State, to: State) -> Result<(), TransitionError> {
        if !is_allowed(from, to) {
            return Err(TransitionError::IllegalTransition);
        }
        self.state
            .compare_exchange(from as u32, to as u32, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| TransitionError::IllegalTransition)
    }

    pub fn request_start(&self) -> Result<(), TransitionError> {
        match self.current() {
            State::Idle => self.cas(State::Idle, State::Starting),
            State::Starting | State::Running => Err(TransitionError::DoubleStart),
            State::Stopping => Err(TransitionError::StartWhileStopping),
            State::Stopped => Err(TransitionError::IllegalTransition),
        }
    }

    pub fn mark_running(&self) -> Result<(), TransitionError> {
        self.cas(State::Starting, State::Running)
    }

    pub fn request_stop(&self) -> Result<(), TransitionError> {
        match self.current() {
            State::Running => self.cas(State::Running, State::Stopping),
            State::Idle => Err(TransitionError::StopBeforeStart),
            State::Starting | State::Stopping | State::Stopped => {
                Err(TransitionError::IllegalTransition)
            }
        }
    }

    pub fn mark_stopped(&self) -> Result<(), TransitionError> {
        self.cas(State::Stopping, State::Stopped)
    }

    pub fn cancel_idle(&self) -> Result<(), TransitionError> {
        self.cas(State::Idle, State::Stopped)
    }

    pub fn mark_fatal(&self) -> State {
        loop {
            let raw = self.state.load(Ordering::Acquire);
            let prev = State::from_raw(raw);
            if prev == State::Stopped {
                return prev;
            }
            if self
                .state
                .compare_exchange(
                    raw,
                    State::Stopped as u32,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return prev;
            }
        }
    }

    #[cfg(test)]
    fn force_state(&self, state: State) {
        self.state.store(state as u32, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::thread;

    use super::*;

    #[test]
    fn is_allowed_exhaustive_transition_table() {
        let all = [
            State::Idle,
            State::Starting,
            State::Running,
            State::Stopping,
            State::Stopped,
        ];
        let allowed = [
            (State::Idle, State::Starting),
            (State::Idle, State::Stopped),
            (State::Starting, State::Running),
            (State::Starting, State::Stopped),
            (State::Running, State::Stopping),
            (State::Running, State::Stopped),
            (State::Stopping, State::Stopped),
        ];
        for from in all {
            for to in all {
                assert_eq!(allowed.contains(&(from, to)), is_allowed(from, to));
            }
        }
    }

    #[test]
    fn happy_path_idle_starting_running_stopping_stopped() {
        let m = Machine::new();
        assert_eq!(State::Idle, m.current());
        m.request_start().unwrap();
        assert_eq!(State::Starting, m.current());
        m.mark_running().unwrap();
        assert_eq!(State::Running, m.current());
        m.request_stop().unwrap();
        assert_eq!(State::Stopping, m.current());
        m.mark_stopped().unwrap();
        assert_eq!(State::Stopped, m.current());
    }

    #[test]
    fn double_start_rejected() {
        let m = Machine::new();
        m.request_start().unwrap();
        assert_eq!(Err(TransitionError::DoubleStart), m.request_start());
        m.mark_running().unwrap();
        assert_eq!(Err(TransitionError::DoubleStart), m.request_start());
    }

    #[test]
    fn stop_before_start_rejected() {
        let m = Machine::new();
        assert_eq!(Err(TransitionError::StopBeforeStart), m.request_stop());
    }

    #[test]
    fn start_while_stopping_rejected() {
        let m = Machine::new();
        m.request_start().unwrap();
        m.mark_running().unwrap();
        m.request_stop().unwrap();
        assert_eq!(Err(TransitionError::StartWhileStopping), m.request_start());
    }

    #[test]
    fn cancel_idle_short_circuits_idle_to_stopped() {
        let m = Machine::new();
        m.cancel_idle().unwrap();
        assert_eq!(State::Stopped, m.current());
        assert_eq!(Err(TransitionError::IllegalTransition), m.request_start());
    }

    #[test]
    fn mark_fatal_forces_stopped_from_any_state() {
        for start in [
            State::Idle,
            State::Starting,
            State::Running,
            State::Stopping,
        ] {
            let m = Machine::new();
            m.force_state(start);
            assert_eq!(start, m.mark_fatal());
            assert_eq!(State::Stopped, m.current());
        }
    }

    #[test]
    fn mark_fatal_idempotent() {
        let m = Machine::new();
        m.force_state(State::Stopped);
        assert_eq!(State::Stopped, m.mark_fatal());
        assert_eq!(State::Stopped, m.current());
    }

    #[test]
    fn concurrent_mark_running_vs_request_stop_reaches_consistent_terminal() {
        for _ in 0..200 {
            let machine = Arc::new(Machine::new());
            machine.request_start().unwrap();
            let run_wins = Arc::new(AtomicU32::new(0));
            let stop_wins = Arc::new(AtomicU32::new(0));

            let runner_machine = Arc::clone(&machine);
            let runner_wins = Arc::clone(&run_wins);
            let runner = thread::spawn(move || {
                if runner_machine.mark_running().is_ok() {
                    runner_wins.fetch_add(1, Ordering::Relaxed);
                }
            });

            let stopper_machine = Arc::clone(&machine);
            let stopper_wins = Arc::clone(&stop_wins);
            let stopper = thread::spawn(move || {
                for _ in 0..1000 {
                    if stopper_machine.request_stop().is_ok() {
                        stopper_wins.fetch_add(1, Ordering::Relaxed);
                        return;
                    }
                    std::hint::spin_loop();
                }
            });

            runner.join().unwrap();
            stopper.join().unwrap();
            assert_eq!(1, run_wins.load(Ordering::Relaxed));
            let stop_wins = stop_wins.load(Ordering::Relaxed);
            assert!(stop_wins <= 1);
            assert_eq!(
                if stop_wins == 1 {
                    State::Stopping
                } else {
                    State::Running
                },
                machine.current()
            );
            if stop_wins == 0 {
                machine.request_stop().unwrap();
            }
            assert_eq!(State::Stopping, machine.current());
        }
    }
}
