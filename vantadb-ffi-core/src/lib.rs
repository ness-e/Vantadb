//! # vantadb-ffi-core
//!
//! Neutral **stdlib-only leaf** shared by VantaDB's FFI transports:
//! `vantadb-node` (napi-rs), `vantadb-python` (PyO3) and `vantadb-wasm`
//! (wasm-bindgen). It exists because those three crates carried literal copies
//! of the same durability gate and clamp policy, with a real divergence risk:
//! one transport fixing a race would silently leave the other two behind.
//!
//! ## Scope — what lives here, and what deliberately does not
//!
//! - [`OpGate`] / [`OpGuard`]: the cross-transport durability barrier. Rejects
//!   new operations once `drain()` starts and keeps `drain()` waiting until
//!   every in-flight operation finishes. Single semantics, three consumers.
//! - [`clamp_top_k`]: the ERR-022 clamp policy (compare → cap → report), so
//!   "silent truncation stays observable" is defined once. The **limit itself**
//!   (`MAX_K`, `MAX_VEC_DIM`, …) stays in the core (`vantadb::config`,
//!   re-exported at the crate root; WSM-09 single source of truth) and is
//!   passed in by the caller — this leaf owns no constants.
//! - **Error mapping stays per transport on purpose.** Each transport has a
//!   different error channel: napi needs a `Status` + message, PyO3 has a real
//!   exception hierarchy, wasm-bindgen attaches a `.code` property to a
//!   `js_sys::Error`. Sharing that mapping would force `napi`, `pyo3` and
//!   `js-sys` dependencies into this leaf — dragging all three toolchains into
//!   every binding and the wasm32 build, which is exactly the contamination
//!   this crate exists to prevent. The 3-line `enter()` wrapper that maps a
//!   rejected gate to the transport's own error type also stays in each
//!   binding.
//!
//! ## Zero dependencies
//!
//! This crate must stay dependency-free (`std` only). Precedent for the leaf
//! profile: `src/index_port.rs` in the core (neutral cycle-breaking leaf).
//! `cargo tree -p vantadb-ffi-core -e normal` must stay empty.
//!
//! ## Contract (consumers may rely on)
//!
//! [`OpGate`] is `Clone` (a clone shares the same gate state).
//! [`OpGate::try_enter`] is `None` **only** after [`OpGate::drain`] has been
//! called; [`OpGate::drain`] returns only after every previously admitted
//! [`OpGuard`] has been dropped. Neither method panics on poisoned state
//! (poison is recovered with `PoisonError::into_inner`), and neither blocks a
//! caller while holding an external lock.
//!
//! ## wasm32 note
//!
//! On `wasm32-unknown-unknown`, [`OpGate::drain`] cannot block:
//! `std::sync::Condvar::wait` panics on the single-threaded shim and the only
//! thread that could decrement the count is the caller itself (blocking would
//! deadlock the JS event loop). `drain()` still flips the barrier (new ops are
//! rejected); in-flight async ops finish on the event loop afterwards.
//!
//! ## Python (PyO3) note — MOD-17
//!
//! PyO3 callers MUST call [`OpGate::drain`] with the GIL released whenever
//! Python threads may hold an [`OpGuard`]: an op returning from its own
//! `py.detach` needs to re-acquire the GIL before it can drop its guard, so
//! waiting with the GIL held deadlocks the interpreter. Keep that call in
//! `py.detach(…)` (see `vantadb-python::Client::close`).
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
#![warn(missing_docs)]

use std::sync::{Arc, Condvar, Mutex, PoisonError};

/// Durability gate shared by the three FFI transports.
///
/// Rejects new operations once [`drain`](OpGate::drain) has begun and keeps
/// `drain()` waiting until every in-flight operation finishes. This closes the
/// write-after-close race where an async op whose engine call had not yet run
/// (node: `spawn_blocking` queued; python: thread paused before the engine
/// call; wasm: microtask queued) would write after `close()` returned —
/// silently lost on process exit.
#[derive(Clone)]
pub struct OpGate {
    state: Arc<(Mutex<OpState>, Condvar)>,
}

/// Gate state: `closing` flips once and stays set; `count` tracks in-flight ops.
struct OpState {
    closing: bool,
    count: usize,
}

impl OpGate {
    /// Creates an open gate (`closing = false`, zero in-flight operations).
    pub fn new() -> Self {
        Self {
            state: Arc::new((
                Mutex::new(OpState {
                    closing: false,
                    count: 0,
                }),
                Condvar::new(),
            )),
        }
    }

    /// Registers a new in-flight operation. Returns `None` if
    /// [`drain`](OpGate::drain) has started (new operations are rejected past
    /// the durability barrier).
    pub fn try_enter(&self) -> Option<OpGuard> {
        let (lock, _) = &*self.state;
        let mut state = lock.lock().unwrap_or_else(PoisonError::into_inner);
        if state.closing {
            return None;
        }
        state.count += 1;
        Some(OpGuard {
            state: self.state.clone(),
        })
    }

    /// Starts closing and blocks until every in-flight operation drains.
    ///
    /// Sets `closing = true` (so new ops are rejected) then waits until
    /// `count == 0`. Blocks the calling thread; acceptable: this is the
    /// durability barrier and engine operations are bounded. Returns with the
    /// internal lock released, so it never leaks a `MutexGuard` across the
    /// caller's `.await` (napi futures must stay `Send`).
    ///
    /// MOD-17 (PyO3 consumers): MUST be called with the GIL released whenever
    /// Python threads may hold an [`OpGuard`] — see the crate-level docs.
    pub fn drain(&self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap_or_else(PoisonError::into_inner);
        state.closing = true;
        // wasm32-unknown-unknown: std Condvar::wait panics (single-threaded
        // no_threads shim) and could never make progress anyway — the only
        // thread that could drain `count` is this one, so a blocking wait
        // would deadlock the JS event loop. The barrier still rejects new
        // ops (closing=true); in-flight async ops finish on the event loop.
        #[cfg(not(target_arch = "wasm32"))]
        while state.count > 0 {
            state = cvar.wait(state).unwrap_or_else(PoisonError::into_inner);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = cvar;
    }
}

/// RAII guard that decrements the in-flight count and wakes
/// [`drain`](OpGate::drain) when dropped (at the end of the owning operation,
/// after the engine call completes).
pub struct OpGuard {
    state: Arc<(Mutex<OpState>, Condvar)>,
}

impl Drop for OpGuard {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.state;
        let mut state = lock.lock().unwrap_or_else(PoisonError::into_inner);
        state.count -= 1;
        cvar.notify_one();
    }
}

/// ERR-022 clamp policy for `top_k`/`k` across search entry points.
///
/// Returns `(effective, was_clamped)`: `effective = min(requested, max)` and
/// `was_clamped = requested > max`. Transports MUST surface `was_clamped`
/// through their own logging channel (node: `eprintln!`, python:
/// `tracing::warn!`) so silent truncation stays observable — the shared policy
/// is *compare → cap → report*, not the logging backend.
///
/// `max` is passed in by the caller (core single source of truth: `MAX_K` in
/// `vantadb::config`, WSM-09); this leaf owns no constants.
#[must_use]
pub fn clamp_top_k(requested: usize, max: usize) -> (usize, bool) {
    if requested > max {
        (max, true)
    } else {
        (requested, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn clamp_top_k_caps_only_above_max_and_reports() {
        assert_eq!(clamp_top_k(0, 100), (0, false));
        assert_eq!(clamp_top_k(10, 100), (10, false));
        assert_eq!(clamp_top_k(100, 100), (100, false));
        assert_eq!(clamp_top_k(101, 100), (100, true));
        assert_eq!(clamp_top_k(usize::MAX, 100), (100, true));
    }

    #[test]
    fn gate_admits_ops_until_drain_and_then_rejects() {
        let gate = OpGate::new();
        let guard = gate.try_enter().expect("open gate admits operations");
        drop(guard);
        gate.drain();
        assert!(
            gate.try_enter().is_none(),
            "drain must reject new operations"
        );
    }

    #[test]
    fn drain_blocks_on_in_flight_guard_and_wakes_when_dropped() {
        let gate = OpGate::new();
        let guard = gate.try_enter().expect("open gate admits operations");

        // A clone shares the same gate state (consumer contract).
        let draining_gate = gate.clone();
        let (started_tx, started_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let handle = std::thread::spawn(move || {
            started_tx.send(()).expect("started signal");
            draining_gate.drain();
            done_tx.send(()).expect("done signal");
        });

        started_rx.recv().expect("drain thread started");
        // Cannot return while the guard is alive (count >= 1), regardless of
        // scheduling: the drain loop only exits on count == 0.
        assert!(
            done_rx.try_recv().is_err(),
            "drain must block while an op is in flight"
        );

        drop(guard);
        done_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("drain wakes after the last in-flight guard drops");
        handle.join().expect("drain thread joins");
    }
}
