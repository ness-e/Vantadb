//! Injectable clock + [`ManagedTimer`] (port of TDAM
//! `MC/utils/managed-timer.ts`, reimplemented pull-based — MEM-16).
//!
//! Divergence from TDAM: there is no `setTimeout`. A timer records a deadline
//! plus its callback and fires when the owner polls it (directly via
//! [`ManagedTimer::poll`] or through the [`crate::utils::timer_scanner`]).
//! This keeps the whole orchestration synchronous, thread-free and — with the
//! [`FakeClock`] — fully deterministic in tests (no sleeps).

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Source of wall time. Injected everywhere a deadline is compared so tests
/// can drive time by hand.
pub trait Clock: Send + Sync {
    /// Current epoch ms.
    fn now_ms(&self) -> u64;
}

/// Real system clock (production).
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Deterministic fake clock for tests: advance it manually, nothing sleeps.
#[derive(Debug, Default)]
pub struct FakeClock(AtomicU64);

impl FakeClock {
    pub fn new(start_ms: u64) -> Self {
        Self(AtomicU64::new(start_ms))
    }

    /// Jump to an absolute epoch ms.
    pub fn set_time(&self, ms: u64) {
        self.0.store(ms, Ordering::SeqCst);
    }

    /// Advance by a relative delta.
    pub fn advance(&self, delta_ms: u64) {
        let _ = self.0.fetch_add(delta_ms, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// A named, single-slot lifecycle-managed timer.
///
/// The callback fires at most once per schedule; `poll`/`flush` consume it.
/// The optional `destroyed` guard mirrors TDAM: natural expiry skips the
/// callback when the owner is torn down, while `flush` intentionally ignores
/// the guard (shutdown must drain pending work).
pub struct ManagedTimer<'a, C: Clock> {
    name: String,
    clock: &'a C,
    destroyed: Option<&'a dyn Fn() -> bool>,
    scheduled_at: Option<u64>,
    callback: Option<Box<dyn FnOnce()>>,
}

impl<'a, C: Clock> ManagedTimer<'a, C> {
    pub fn new(name: impl Into<String>, clock: &'a C) -> Self {
        Self {
            name: name.into(),
            clock,
            destroyed: None,
            scheduled_at: None,
            callback: None,
        }
    }

    /// Attach the destroyed-guard checked before natural firing.
    pub fn with_destroyed_guard(mut self, guard: &'a dyn Fn() -> bool) -> Self {
        self.destroyed = Some(guard);
        self
    }

    /// Cancel any pending timer and schedule a new one after `delay_ms`.
    pub fn schedule(&mut self, delay_ms: u64, callback: Box<dyn FnOnce()>) {
        self.schedule_at(self.clock.now_ms().saturating_add(delay_ms), callback);
    }

    /// Cancel any pending timer and schedule to fire at an absolute epoch ms.
    /// A past deadline fires on the next poll.
    pub fn schedule_at(&mut self, epoch_ms: u64, callback: Box<dyn FnOnce()>) {
        self.cancel();
        self.scheduled_at = Some(epoch_ms);
        self.callback = Some(callback);
    }

    /// Downward-only reschedule (TDAM L2 pattern): only moves the deadline if
    /// `epoch_ms` is earlier than the current one. Returns whether the timer
    /// was (re)set.
    pub fn try_advance_to(&mut self, epoch_ms: u64, callback: Box<dyn FnOnce()>) -> bool {
        match self.scheduled_at {
            None => {
                self.schedule_at(epoch_ms, callback);
                true
            }
            Some(current) if epoch_ms < current => {
                self.schedule_at(epoch_ms, callback);
                true
            }
            Some(_) => false,
        }
    }

    /// Cancel without triggering.
    pub fn cancel(&mut self) {
        self.scheduled_at = None;
        self.callback = None;
    }

    /// Fire immediately if pending, bypassing the destroyed guard (shutdown
    /// drain). Returns whether a callback ran.
    pub fn flush(&mut self) -> bool {
        match self.callback.take() {
            Some(cb) => {
                self.scheduled_at = None;
                cb();
                true
            }
            None => false,
        }
    }

    /// Fire if the deadline passed on the injected clock. Respects the
    /// destroyed guard. Returns whether a callback ran.
    pub fn poll(&mut self) -> bool {
        let due = matches!(self.scheduled_at, Some(at) if at <= self.clock.now_ms());
        if !due {
            return false;
        }
        let cb = self.callback.take();
        self.scheduled_at = None;
        match (cb, self.destroyed) {
            (Some(cb), Some(is_destroyed)) if is_destroyed() => {
                let _ = cb; // dropped without running — owner is gone
                false
            }
            (Some(cb), _) => {
                cb();
                true
            }
            (None, _) => false,
        }
    }

    /// Whether a timer is pending.
    pub fn pending(&self) -> bool {
        self.callback.is_some()
    }

    /// Deadline of the pending timer (0 when none).
    pub fn scheduled_time(&self) -> u64 {
        self.scheduled_at.unwrap_or(0)
    }

    /// Human-readable name (logging).
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use std::rc::Rc;

    fn flag() -> (Rc<Cell<bool>>, impl Fn() -> bool) {
        let cell = Rc::new(Cell::new(false));
        let cell2 = Rc::clone(&cell);
        (cell, move || cell2.get())
    }

    #[test]
    fn fires_only_after_deadline_on_fake_clock() {
        let clock = FakeClock::new(1_000);
        let mut timer = ManagedTimer::new("t", &clock);
        timer.schedule_at(
            1_500,
            Box::new(|| unreachable!("must not fire before the deadline")),
        );
        assert!(timer.pending());
        assert_eq!(timer.scheduled_time(), 1_500);

        clock.set_time(1_499);
        assert!(!timer.poll());

        // Replace with an observable callback and cross the deadline.
        let (fired, is_fired) = flag();
        timer.schedule_at(1_500, Box::new(move || fired.set(true)));
        clock.set_time(1_500);
        assert!(timer.poll());
        assert!(is_fired());
    }

    #[test]
    fn poll_runs_callback_once_when_due() {
        let clock = FakeClock::new(0);
        let (fired, is_fired) = flag();
        let mut timer = ManagedTimer::new("t", &clock);
        timer.schedule_at(100, Box::new(move || fired.set(true)));
        clock.set_time(100);
        assert!(timer.poll());
        assert!(!timer.pending());
        assert!(!timer.poll()); // consumed exactly once
        assert!(is_fired());
    }

    #[test]
    fn try_advance_to_is_downward_only() {
        let clock = FakeClock::new(0);
        let mut timer = ManagedTimer::new("t", &clock);
        timer.schedule_at(1_000, Box::new(|| {}));
        assert!(!timer.try_advance_to(2_000, Box::new(|| {})));
        assert_eq!(timer.scheduled_time(), 1_000);
        assert!(timer.try_advance_to(500, Box::new(|| {})));
        assert_eq!(timer.scheduled_time(), 500);
    }

    #[test]
    fn flush_runs_even_when_destroyed_and_ignores_guard() {
        let clock = FakeClock::new(0);
        let destroyed = || true;
        let mut timer = ManagedTimer::new("t", &clock).with_destroyed_guard(&destroyed);
        timer.schedule_at(10_000, Box::new(|| {}));
        assert!(timer.flush());
        assert!(!timer.pending());
    }

    #[test]
    fn poll_respects_destroyed_guard() {
        let clock = FakeClock::new(0);
        let destroyed = || true;
        let mut timer = ManagedTimer::new("t", &clock).with_destroyed_guard(&destroyed);
        timer.schedule_at(10, Box::new(|| {}));
        clock.set_time(20);
        assert!(!timer.poll());
        assert!(!timer.pending());
    }

    #[test]
    fn cancel_drops_without_firing() {
        let clock = FakeClock::new(0);
        let mut timer = ManagedTimer::new("t", &clock);
        timer.schedule_at(10, Box::new(|| {}));
        timer.cancel();
        clock.set_time(100);
        assert!(!timer.poll());
        assert!(!timer.pending());
    }
}
