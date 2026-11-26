#![allow(dead_code)]
use crate::config::Config;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// FIND-642: process-global reservation (bytes) for resident ONNX embedding
/// sessions. The `LocalOnnxProvider` lives outside any single `StorageEngine`,
/// so its footprint cannot be charged to one governor instance — it is tracked
/// globally and folded into every governor's `used_bytes()` instead. With
/// `embed-local` the model resident would otherwise sit outside the
/// `memory_limit` budget entirely.
static ONNX_SESSION_RESERVED_BYTES: AtomicU64 = AtomicU64::new(0);

/// Fixed overhead per ONNX Runtime session on top of the model weights:
/// CPU EP arena allocator, intra/inter-op thread pools and graph partition
/// buffers. Heuristic — there is no published official figure; 64 MiB is a
/// conservative floor observed for a minimal session.
pub(crate) const ONNX_RUNTIME_OVERHEAD_BYTES: u64 = 64 * 1024 * 1024;

/// FIND-642: estimated resident bytes of an ORT session whose weights occupy
/// `weights_bytes` on disk (`model.onnx` + `model.onnx_data` sidecars).
///
/// Factor 1.5× weights + [`ONNX_RUNTIME_OVERHEAD_BYTES`]: weights are loaded
/// into the session (or mapped), plus a copy window during commit and the
/// runtime overhead above. Heuristic, deliberately biased high — the
/// documented failure mode is underestimation, never overestimation.
pub(crate) fn estimate_onnx_session_bytes(weights_bytes: u64) -> u64 {
    weights_bytes
        .saturating_mul(3)
        .saturating_div(2)
        .saturating_add(ONNX_RUNTIME_OVERHEAD_BYTES)
}

/// Reserve `bytes` of estimated ONNX session memory against the global budget.
pub(crate) fn reserve_onnx_session(bytes: u64) {
    ONNX_SESSION_RESERVED_BYTES.fetch_add(bytes, Ordering::Relaxed);
}

/// Release a previous [`reserve_onnx_session`] reservation (saturating: a
/// double-release leaves the counter at 0, never wraps).
pub(crate) fn release_onnx_session(bytes: u64) {
    let _ = ONNX_SESSION_RESERVED_BYTES.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |cur| {
        Some(cur.saturating_sub(bytes))
    });
}

/// Bytes currently reserved for resident ONNX sessions.
pub(crate) fn onnx_session_reserved() -> u64 {
    ONNX_SESSION_RESERVED_BYTES.load(Ordering::Relaxed)
}

/// Memory usage governor with watermarks and eviction control.
pub(crate) struct MemoryGovernor {
    /// Maximum allowed memory in bytes.
    memory_limit: u64,
    /// Target usage ratio (0.0–1.0).
    target_ratio: f64,
    /// Low watermark: usage below this is safe.
    low_water_mark: u64,
    /// High watermark: usage above this triggers eviction.
    high_water_mark: u64,
    /// Current estimated usage in bytes.
    used_bytes: AtomicU64,
    /// Whether eviction is currently running.
    eviction_running: AtomicBool,
    /// Timestamp of last eviction in milliseconds.
    last_eviction_ms: AtomicU64,
    /// Count of out-of-memory events.
    oom_count: AtomicU64,
}

impl MemoryGovernor {
    /// Create a new governor from config.
    pub fn new(config: &Config) -> Self {
        let caps = crate::hardware::HardwareCapabilities::global();
        let memory_limit = config.memory_limit.unwrap_or(caps.total_memory);
        let target_ratio = 0.75;
        // A zero/unknown limit means "not configured" (e.g. hardware detection
        // unavailable, or under Miri where machine memory reports 0). Treat it as
        // unlimited: pin watermarks to u64::MAX so no usage level triggers
        // eviction — otherwise `limit == 0` makes every insert look like 100%
        // pressure and rejects all writes (same policy as stats.rs AUDIT-03).
        let (low_water_mark, high_water_mark) = if memory_limit == 0 {
            (u64::MAX, u64::MAX)
        } else {
            (
                (memory_limit as f64 * target_ratio * 0.9) as u64,
                (memory_limit as f64 * target_ratio) as u64,
            )
        };
        Self {
            memory_limit,
            target_ratio,
            low_water_mark,
            high_water_mark,
            used_bytes: AtomicU64::new(0),
            eviction_running: AtomicBool::new(false),
            last_eviction_ms: AtomicU64::new(0),
            oom_count: AtomicU64::new(0),
        }
    }

    /// Record an allocation of `bytes`.
    pub fn allocated(&self, bytes: u64) {
        self.used_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    /// Record a deallocation of `bytes`.
    pub fn freed(&self, bytes: u64) {
        self.used_bytes.fetch_sub(bytes, Ordering::Relaxed);
    }

    /// Current estimated memory usage in bytes, including the process-global
    /// ONNX session reservation (FIND-642).
    pub fn used_bytes(&self) -> u64 {
        self.used_bytes
            .load(Ordering::Relaxed)
            .saturating_add(onnx_session_reserved())
    }

    /// Bytes tracked by this governor instance only (no ONNX reservation).
    /// Used by tests that need the per-engine counter in isolation.
    pub fn engine_used_bytes(&self) -> u64 {
        self.used_bytes.load(Ordering::Relaxed)
    }

    /// Maximum allowed memory in bytes.
    pub fn limit(&self) -> u64 {
        self.memory_limit
    }

    /// Returns `true` if usage is above the high watermark.
    pub fn should_evict(&self) -> bool {
        self.used_bytes() > self.high_water_mark
    }

    /// Returns `true` if usage is above the memory limit (urgent).
    pub fn needs_urgent_eviction(&self) -> bool {
        self.memory_limit != 0 && self.used_bytes() > self.memory_limit
    }

    /// Returns `true` if usage is above the low watermark.
    pub fn above_low_water(&self) -> bool {
        self.used_bytes() > self.low_water_mark
    }

    /// Try to start an eviction cycle. Returns `false` if already running.
    pub fn try_start_eviction(&self) -> bool {
        self.eviction_running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
    }

    /// Mark eviction as finished and record the timestamp.
    pub fn finish_eviction(&self) {
        self.eviction_running.store(false, Ordering::Release);
        self.last_eviction_ms.store(
            web_time::SystemTime::now()
                .duration_since(web_time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            Ordering::Relaxed,
        );
    }

    /// Count of out-of-memory events.
    pub fn oom_count(&self) -> u64 {
        self.oom_count.load(Ordering::Relaxed)
    }

    /// Record an out-of-memory event.
    pub fn record_oom(&self) {
        self.oom_count.fetch_add(1, Ordering::Relaxed);
        #[cfg(feature = "prometheus")]
        if let Some(c) = crate::metrics::OOM_TRIPS.as_ref() {
            c.inc();
        }
    }

    /// Returns `true` if eviction is currently running.
    pub fn eviction_running(&self) -> bool {
        self.eviction_running.load(Ordering::Acquire)
    }

    /// Directly set the used bytes counter (for testing).
    pub fn set_used_bytes(&self, bytes: u64) {
        self.used_bytes.store(bytes, Ordering::Relaxed);
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn make_gov(memory_limit: u64) -> MemoryGovernor {
        let config = Config {
            memory_limit: Some(memory_limit),
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        MemoryGovernor::new(&config)
    }

    #[test]
    fn test_memory_governor_allocated_freed() {
        let gov = make_gov(1_000_000);
        assert_eq!(gov.used_bytes(), 0);

        gov.allocated(100);
        assert_eq!(gov.used_bytes(), 100);

        gov.freed(40);
        assert_eq!(gov.used_bytes(), 60);
    }

    #[test]
    fn test_memory_governor_memory_limit() {
        let gov = make_gov(512_000);
        assert_eq!(gov.limit(), 512_000);
    }

    #[test]
    fn test_memory_governor_should_evict() {
        let gov = make_gov(1000);
        // high_water_mark = 1000 * 0.75 = 750
        gov.set_used_bytes(800);
        assert!(gov.should_evict());

        gov.set_used_bytes(700);
        assert!(!gov.should_evict());
    }

    #[test]
    fn test_memory_governor_needs_urgent_eviction() {
        let gov = make_gov(1000);
        gov.set_used_bytes(500);
        assert!(!gov.needs_urgent_eviction());

        gov.set_used_bytes(1500);
        assert!(gov.needs_urgent_eviction());
    }

    #[test]
    fn test_memory_governor_above_low_water() {
        let gov = make_gov(1000);
        // low_water_mark = 1000 * 0.75 * 0.9 = 675
        gov.set_used_bytes(700);
        assert!(gov.above_low_water());

        gov.set_used_bytes(600);
        assert!(!gov.above_low_water());
    }

    #[test]
    fn test_memory_governor_eviction_running() {
        let gov = make_gov(1_000_000);
        assert!(!gov.eviction_running());
        assert!(gov.try_start_eviction());
        assert!(gov.eviction_running());
        // Second attempt should fail
        assert!(!gov.try_start_eviction());
    }

    #[test]
    fn test_memory_governor_finish_eviction() {
        let gov = make_gov(1_000_000);
        gov.try_start_eviction();
        gov.finish_eviction();
        assert!(!gov.eviction_running());
        assert!(gov.try_start_eviction()); // can start again
    }

    #[test]
    fn test_memory_governor_oom_count() {
        let gov = make_gov(1_000_000);
        assert_eq!(gov.oom_count(), 0);
        gov.record_oom();
        assert_eq!(gov.oom_count(), 1);
        gov.record_oom();
        assert_eq!(gov.oom_count(), 2);
    }

    #[test]
    fn test_memory_governor_set_used_bytes() {
        let gov = make_gov(1_000_000);
        gov.set_used_bytes(5000);
        assert_eq!(gov.used_bytes(), 5000);
    }

    // ── FIND-642: ONNX session budget accounting (no dylib required) ──

    #[test]
    #[serial_test::serial]
    fn onnx_budget_estimate_factor() {
        // estimate = 1.5x weights + fixed ORT overhead; documented heuristic.
        assert_eq!(estimate_onnx_session_bytes(0), ONNX_RUNTIME_OVERHEAD_BYTES);
        assert_eq!(
            estimate_onnx_session_bytes(100 * 1024 * 1024),
            150 * 1024 * 1024 + ONNX_RUNTIME_OVERHEAD_BYTES
        );
        // saturating: u64::MAX weights must not wrap around to a small value.
        assert_eq!(
            estimate_onnx_session_bytes(u64::MAX),
            u64::MAX / 2 + ONNX_RUNTIME_OVERHEAD_BYTES
        );
    }

    #[test]
    #[serial_test::serial]
    fn onnx_budget_reservation_counts_against_limit() {
        let gov = make_gov(1_000_000); // high water = 750_000
        reserve_onnx_session(900_000);
        assert!(gov.used_bytes() >= 900_000);
        assert!(gov.should_evict(), "900k over 750k high watermark");
        assert!(!gov.needs_urgent_eviction(), "900k under 1M limit");
        release_onnx_session(900_000);
        assert_eq!(onnx_session_reserved(), 0);
        assert_eq!(gov.used_bytes(), 0);
    }

    #[test]
    #[serial_test::serial]
    fn onnx_budget_reservation_can_trip_urgent_eviction() {
        let gov = make_gov(1_000_000);
        reserve_onnx_session(1_500_000);
        assert!(gov.needs_urgent_eviction(), "reservation above the limit");
        release_onnx_session(1_500_000);
        assert!(!gov.needs_urgent_eviction());
    }

    #[test]
    #[serial_test::serial]
    fn onnx_budget_release_saturates_not_wraps() {
        // Double release must leave the counter at 0, never wrap (FIND-572 policy).
        reserve_onnx_session(100);
        release_onnx_session(100);
        release_onnx_session(100);
        assert_eq!(onnx_session_reserved(), 0);
    }
}
