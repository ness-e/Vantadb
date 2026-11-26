//! Request-scoped metrics for MCP tool calls.

use std::sync::atomic::{AtomicU64, Ordering};

// ── Request-scoped metrics ─────────────────────────────────────────────────

#[derive(Default)]
pub(crate) struct McpMetrics {
    pub(crate) requests_total: AtomicU64,
    pub(crate) errors_total: AtomicU64,
    pub(crate) active_requests: AtomicU64,
}

/// RAII guard that increments `active_requests` on construction and guarantees
/// decrement on *every* exit path of the handler body — including `?`
/// early-returns and panic unwinding (via `Drop`).
///
/// Invariant protected: `active_requests` is the count of in-flight requests,
/// surfaced by the periodic 30s metrics log. Nothing awaits/blocks on it
/// (concurrency is enforced by the semaphore), so a leaked increment is not a
/// correctness bug — but it makes the gauge drift monotonically upward with
/// phantom "stuck" requests, corrupting capacity accounting. The guard keeps
/// the gauge an accurate measure of in-flight work.
pub(crate) struct ActiveRequestGuard<'a>(&'a AtomicU64);

impl<'a> ActiveRequestGuard<'a> {
    pub(crate) fn new(counter: &'a AtomicU64) -> Self {
        counter.fetch_add(1, Ordering::Relaxed);
        Self(counter)
    }
}

impl Drop for ActiveRequestGuard<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    /// `active_requests` must return to its base value after a handler panic:
    /// the guard's Drop decrements on unwind just like on normal exit, so the
    /// gauge never leaks a phantom in-flight request.
    #[test]
    fn active_request_guard_decrements_on_panic() {
        let counter = AtomicU64::new(7); // base = prior in-flight requests

        let result = catch_unwind(AssertUnwindSafe(|| {
            let _guard = ActiveRequestGuard::new(&counter);
            // Simulate mid-handler work, then a panic before the handler body ends.
            assert_eq!(counter.load(Ordering::SeqCst), 8);
            panic!("simulated handler panic");
        }));

        assert!(result.is_err(), "closure should have panicked");
        assert_eq!(
            counter.load(Ordering::SeqCst),
            7,
            "counter must be restored to its base value after panic"
        );
    }
}
