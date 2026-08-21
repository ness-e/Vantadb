//! `TimerScanner` (port of TDAM `MC/services/timer-scanner.ts`, pull-based —
//! MEM-16).
//!
//! TDAM runs a background interval that polls Redis for expired timers. Here
//! the owner calls [`TimerScanner::run_once`] whenever it wants due timers
//! dispatched (worker loop tick, test step). With a [`FakeClock`] this makes
//! timer firing fully deterministic.

use crate::core::state::TimerEntry;
use crate::utils::local_backend::LocalStateBackend;
use crate::utils::managed_timer::Clock;

/// Dispatches expired timers from the backend to one handler.
pub struct TimerScanner<'a, C: Clock> {
    backend: &'a LocalStateBackend<C>,
}

impl<'a, C: Clock> TimerScanner<'a, C> {
    pub fn new(backend: &'a LocalStateBackend<C>) -> Self {
        Self { backend }
    }

    /// Remove every expired timer and hand it to `handler`. Returns how many
    /// timers fired.
    pub fn run_once(&self, mut handler: impl FnMut(&TimerEntry)) -> usize {
        let now = self.backend_clock();
        let expired = self.backend.take_expired_timers(now);
        let count = expired.len();
        for entry in &expired {
            handler(entry);
        }
        count
    }

    fn backend_clock(&self) -> u64 {
        self.backend.now_ms()
    }
}
