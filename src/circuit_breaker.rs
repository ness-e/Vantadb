//! Circuit breaker state machine for fast-failing HTTP requests.
//!
//! Closed → Open when `failure_threshold` consecutive failures are recorded.
//! While Open, `allow_request()` returns `false` (fast-fail). After
//! `open_timeout` elapses the breaker moves to HalfOpen and allows a single
//! probe request: success → Closed, failure → Open.
//!
//! Lock-free: state is an `AtomicU8`, failures an `AtomicU32`, and the open
//! timestamp a `AtomicU64` (epoch seconds) — no external dependencies.

use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::time::Duration;

const CLOSED: u8 = 0;
const OPEN: u8 = 1;
const HALF_OPEN: u8 = 2;

/// Circuit breaker state, exposed for observability and tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation; requests pass through.
    Closed,
    /// Fast-failing all requests until the open timeout elapses.
    Open,
    /// Allowing a single probe request to test recovery.
    HalfOpen,
}

/// Lock-free circuit breaker with Closed → Open → HalfOpen transitions.
pub struct CircuitBreaker {
    state: AtomicU8,
    failure_count: AtomicU32,
    failure_threshold: u32,
    open_timeout_secs: u64,
    last_open_secs: AtomicU64,
}

impl CircuitBreaker {
    /// Create a breaker that opens after `failure_threshold` consecutive
    /// failures and stays open for `open_timeout`.
    pub fn new(failure_threshold: u32, open_timeout: Duration) -> Self {
        Self {
            state: AtomicU8::new(CLOSED),
            failure_count: AtomicU32::new(0),
            failure_threshold: failure_threshold.max(1),
            open_timeout_secs: open_timeout.as_secs(),
            last_open_secs: AtomicU64::new(0),
        }
    }

    /// Returns `true` if the request may proceed.
    ///
    /// A HalfOpen breaker allows exactly one probe request to pass.
    pub fn allow_request(&self) -> bool {
        loop {
            match self.state.load(Ordering::Acquire) {
                CLOSED => return true,
                OPEN => {
                    let now = now_secs();
                    let opened = self.last_open_secs.load(Ordering::Relaxed);
                    if now.saturating_sub(opened) >= self.open_timeout_secs {
                        // Try to claim the single half-open probe slot.
                        if self
                            .state
                            .compare_exchange(OPEN, HALF_OPEN, Ordering::AcqRel, Ordering::Acquire)
                            .is_ok()
                        {
                            return true;
                        }
                    } else {
                        return false;
                    }
                }
                HALF_OPEN => return false,
                _ => return false,
            }
        }
    }

    /// Record a successful request: reset failures and close the breaker.
    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.state.store(CLOSED, Ordering::Release);
    }

    /// Record a failed request. Trips the breaker when the failure threshold
    /// is reached (from Closed) or when a half-open probe fails.
    pub fn record_failure(&self) {
        match self.state.load(Ordering::Acquire) {
            CLOSED => {
                let failures = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
                if failures >= self.failure_threshold
                    && self
                        .state
                        .compare_exchange(CLOSED, OPEN, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                {
                    self.last_open_secs.store(now_secs(), Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                }
            }
            HALF_OPEN => {
                // A failed probe re-opens immediately.
                self.state.store(OPEN, Ordering::Release);
                self.last_open_secs.store(now_secs(), Ordering::Relaxed);
                self.failure_count.store(0, Ordering::Relaxed);
            }
            OPEN => {}
            _ => {}
        }
    }

    /// Current breaker state.
    pub fn state(&self) -> CircuitState {
        match self.state.load(Ordering::Acquire) {
            CLOSED => CircuitState::Closed,
            OPEN => CircuitState::Open,
            _ => CircuitState::HalfOpen,
        }
    }

    /// Remaining seconds until an Open breaker may probe, for `Retry-After`.
    pub fn retry_after_secs(&self) -> u64 {
        if self.state.load(Ordering::Acquire) != OPEN {
            return 0;
        }
        let opened = self.last_open_secs.load(Ordering::Relaxed);
        let elapsed = now_secs().saturating_sub(opened);
        self.open_timeout_secs.saturating_sub(elapsed).max(1)
    }
}

fn now_secs() -> u64 {
    web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_closed_allows_and_success_resets() {
        let b = CircuitBreaker::new(5, Duration::from_secs(30));
        assert_eq!(b.state(), CircuitState::Closed);
        assert!(b.allow_request());
        b.record_failure();
        b.record_failure();
        assert!(b.allow_request());
        b.record_success();
        assert_eq!(b.state(), CircuitState::Closed);
    }

    #[test]
    fn test_opens_after_threshold_failures() {
        let b = CircuitBreaker::new(3, Duration::from_secs(30));
        b.record_failure();
        b.record_failure();
        assert!(b.allow_request());
        b.record_failure();
        assert_eq!(b.state(), CircuitState::Open);
        assert!(!b.allow_request());
    }

    #[test]
    fn test_open_fast_fails_until_timeout() {
        let b = CircuitBreaker::new(1, Duration::from_secs(30));
        b.record_failure();
        assert_eq!(b.state(), CircuitState::Open);
        assert!(!b.allow_request());
        assert!(b.retry_after_secs() > 0);
    }

    #[test]
    fn test_half_open_allows_single_probe_then_reopens() {
        let b = CircuitBreaker::new(1, Duration::from_secs(0));
        b.record_failure();
        assert_eq!(b.state(), CircuitState::Open);
        // Timeout already elapsed (0s): first call claims the probe slot.
        assert!(b.allow_request());
        assert_eq!(b.state(), CircuitState::HalfOpen);
        // Second request while probe in flight is rejected.
        assert!(!b.allow_request());
        // Failed probe re-opens.
        b.record_failure();
        assert_eq!(b.state(), CircuitState::Open);
    }

    #[test]
    fn test_half_open_probe_success_closes() {
        let b = CircuitBreaker::new(1, Duration::from_secs(0));
        b.record_failure();
        assert!(b.allow_request()); // probe
        b.record_success();
        assert_eq!(b.state(), CircuitState::Closed);
        assert!(b.allow_request());
    }
}
