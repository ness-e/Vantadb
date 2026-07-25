//! Prometheus metric registration and instrument definitions.
//!
//! All `pub static` prometheus metric handles, their registration
//! with the global `METRICS_REGISTRY`, and HTTP request recording.

#[cfg(feature = "prometheus")]
use prometheus::{
    exponential_buckets, Histogram, HistogramVec, IntCounter, IntCounterVec, IntGauge, Registry,
};
#[cfg(feature = "prometheus")]
use std::sync::LazyLock;
use web_time::Instant;

/// Prometheus metrics registry, available when the `prometheus` feature is enabled.
#[cfg(feature = "prometheus")]
pub static METRICS_REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

/// Query execution latency histogram.
#[cfg(feature = "prometheus")]
pub static QUERY_LATENCY: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    let hist = match Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_query_latency_ms",
        "Query execution times in ms",
    )) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create QUERY_LATENCY histogram: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register QUERY_LATENCY: {e}");
            None
        }
    }
});

/// OOM circuit breaker trip counter.
#[cfg(feature = "prometheus")]
pub static OOM_TRIPS: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new("vanta_oom_circuit_trips_total", "Governor OOM prevents") {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create OOM_TRIPS counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register OOM_TRIPS: {e}");
            None
        }
    }
});

/// Page cache hit counter.
#[cfg(feature = "prometheus")]
pub static CACHE_HITS: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new("vanta_cache_hits_total", "CP-Index fast path matches") {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create CACHE_HITS counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register CACHE_HITS: {e}");
            None
        }
    }
});

/// Engine startup latency histogram.
#[cfg(feature = "prometheus")]
pub static STARTUP_LATENCY_MS: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    let hist = match Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_startup_latency_ms",
        "Storage engine startup time in ms",
    )) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create STARTUP_LATENCY_MS histogram: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register STARTUP_LATENCY_MS: {e}");
            None
        }
    }
});

/// WAL replay latency histogram.
#[cfg(feature = "prometheus")]
pub static WAL_REPLAY_LATENCY_MS: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    let hist = match Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_wal_replay_latency_ms",
        "WAL replay time in ms during startup",
    )) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create WAL_REPLAY_LATENCY_MS histogram: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register WAL_REPLAY_LATENCY_MS: {e}");
            None
        }
    }
});

/// ANN index rebuild latency histogram.
#[cfg(feature = "prometheus")]
pub static ANN_REBUILD_LATENCY_MS: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    let hist = match Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_ann_rebuild_latency_ms",
        "Manual or startup ANN rebuild time in ms",
    )) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create ANN_REBUILD_LATENCY_MS histogram: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register ANN_REBUILD_LATENCY_MS: {e}");
            None
        }
    }
});

/// Derived index rebuild latency histogram.
#[cfg(feature = "prometheus")]
pub static DERIVED_REBUILD_LATENCY_MS: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    let hist = match Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_derived_rebuild_latency_ms",
        "Derived namespace/payload index rebuild time in ms",
    )) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create DERIVED_REBUILD_LATENCY_MS histogram: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register DERIVED_REBUILD_LATENCY_MS: {e}");
            None
        }
    }
});

/// Text index rebuild latency histogram.
#[cfg(feature = "prometheus")]
pub static TEXT_INDEX_REBUILD_LATENCY_MS: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    let hist = match Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_text_index_rebuild_latency_ms",
        "Derived text index rebuild time in ms",
    )) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create TEXT_INDEX_REBUILD_LATENCY_MS histogram: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register TEXT_INDEX_REBUILD_LATENCY_MS: {e}");
            None
        }
    }
});

/// Total records exported counter.
#[cfg(feature = "prometheus")]
pub static RECORDS_EXPORTED: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_records_exported_total",
        "Persistent memory records exported",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create RECORDS_EXPORTED counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register RECORDS_EXPORTED: {e}");
            None
        }
    }
});

/// Total records imported counter.
#[cfg(feature = "prometheus")]
pub static RECORDS_IMPORTED: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_records_imported_total",
        "Persistent memory records imported",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create RECORDS_IMPORTED counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register RECORDS_IMPORTED: {e}");
            None
        }
    }
});

/// Import error counter.
#[cfg(feature = "prometheus")]
pub static IMPORT_ERRORS: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_import_errors_total",
        "Persistent memory import errors",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create IMPORT_ERRORS counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register IMPORT_ERRORS: {e}");
            None
        }
    }
});

/// Text index postings written counter.
#[cfg(feature = "prometheus")]
pub static TEXT_POSTINGS_WRITTEN: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_text_postings_written_total",
        "Derived text index postings written",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create TEXT_POSTINGS_WRITTEN counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register TEXT_POSTINGS_WRITTEN: {e}");
            None
        }
    }
});

/// Text index repair counter.
#[cfg(feature = "prometheus")]
pub static TEXT_INDEX_REPAIRS: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_text_index_repairs_total",
        "Derived text index repairs from canonical records",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create TEXT_INDEX_REPAIRS counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register TEXT_INDEX_REPAIRS: {e}");
            None
        }
    }
});

/// BM25 lexical query latency histogram.
#[cfg(feature = "prometheus")]
pub static TEXT_LEXICAL_QUERY_LATENCY_MS: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    let hist = match Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_text_lexical_query_latency_ms",
        "BM25 lexical memory query time in ms",
    )) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create TEXT_LEXICAL_QUERY_LATENCY_MS histogram: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register TEXT_LEXICAL_QUERY_LATENCY_MS: {e}");
            None
        }
    }
});

/// Total lexical queries executed counter.
#[cfg(feature = "prometheus")]
pub static TEXT_LEXICAL_QUERIES: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_text_lexical_queries_total",
        "BM25 lexical memory queries executed",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create TEXT_LEXICAL_QUERIES counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register TEXT_LEXICAL_QUERIES: {e}");
            None
        }
    }
});

/// Total lexical candidates scored counter.
#[cfg(feature = "prometheus")]
pub static TEXT_CANDIDATES_SCORED: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_text_candidates_scored_total",
        "BM25 lexical candidates scored",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create TEXT_CANDIDATES_SCORED counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register TEXT_CANDIDATES_SCORED: {e}");
            None
        }
    }
});

/// Text index consistency audit counter.
#[cfg(feature = "prometheus")]
pub static TEXT_CONSISTENCY_AUDITS: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_text_consistency_audits_total",
        "Structural text index consistency audits executed",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create TEXT_CONSISTENCY_AUDITS counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register TEXT_CONSISTENCY_AUDITS: {e}");
            None
        }
    }
});

/// Text consistency audit failure counter.
#[cfg(feature = "prometheus")]
pub static TEXT_CONSISTENCY_AUDIT_FAILURES: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_text_consistency_audit_failures_total",
        "Structural text index consistency audits that detected mismatch",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create TEXT_CONSISTENCY_AUDIT_FAILURES counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register TEXT_CONSISTENCY_AUDIT_FAILURES: {e}");
            None
        }
    }
});

/// Hybrid (text+vector) query latency histogram.
#[cfg(feature = "prometheus")]
pub static HYBRID_QUERY_LATENCY_MS: LazyLock<Option<Histogram>> = LazyLock::new(|| {
    let hist = match Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_hybrid_query_latency_ms",
        "Hybrid memory query fusion time in ms",
    )) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create HYBRID_QUERY_LATENCY_MS histogram: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register HYBRID_QUERY_LATENCY_MS: {e}");
            None
        }
    }
});

/// Hybrid query candidates fused counter.
#[cfg(feature = "prometheus")]
pub static HYBRID_CANDIDATES_FUSED: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_hybrid_candidates_fused_total",
        "Unique memory candidates fused by hybrid retrieval",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create HYBRID_CANDIDATES_FUSED counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register HYBRID_CANDIDATES_FUSED: {e}");
            None
        }
    }
});

/// Queries planned as hybrid route counter.
#[cfg(feature = "prometheus")]
pub static PLANNER_HYBRID_QUERIES: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_planner_hybrid_queries_total",
        "Memory searches planned as hybrid text+vector retrieval",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create PLANNER_HYBRID_QUERIES counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register PLANNER_HYBRID_QUERIES: {e}");
            None
        }
    }
});

/// Queries planned as text-only route counter.
#[cfg(feature = "prometheus")]
pub static PLANNER_TEXT_ONLY_QUERIES: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_planner_text_only_queries_total",
        "Memory searches planned as text-only retrieval",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create PLANNER_TEXT_ONLY_QUERIES counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register PLANNER_TEXT_ONLY_QUERIES: {e}");
            None
        }
    }
});

/// Queries planned as vector-only route counter.
#[cfg(feature = "prometheus")]
pub static PLANNER_VECTOR_ONLY_QUERIES: LazyLock<Option<IntCounter>> = LazyLock::new(|| {
    let counter = match IntCounter::new(
        "vanta_planner_vector_only_queries_total",
        "Memory searches planned as vector-only retrieval",
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create PLANNER_VECTOR_ONLY_QUERIES counter: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register PLANNER_VECTOR_ONLY_QUERIES: {e}");
            None
        }
    }
});

// ── Memory breakdown gauges ──────────────────────────────────────────────

#[cfg(feature = "prometheus")]
macro_rules! register_gauge {
    ($name:expr, $help:expr, $static_name:tt) => {{
        let gauge = match IntGauge::new($name, $help) {
            Ok(g) => g,
            Err(e) => {
                tracing::warn!("Failed to create {} gauge: {e}", stringify!($static_name));
                return None;
            }
        };
        match METRICS_REGISTRY.register(Box::new(gauge.clone())) {
            Ok(_) => Some(gauge),
            Err(e) => {
                tracing::warn!("Failed to register {}: {e}", stringify!($static_name));
                None
            }
        }
    }};
}

/// Process resident set size (RSS) in bytes.
#[cfg(feature = "prometheus")]
pub static PROCESS_RSS_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_process_rss_bytes",
        "Process resident set size in bytes (via sysinfo)",
        PROCESS_RSS_BYTES
    )
});

/// Process virtual memory size in bytes.
#[cfg(feature = "prometheus")]
pub static PROCESS_VIRTUAL_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_process_virtual_bytes",
        "Process virtual memory in bytes (via sysinfo)",
        PROCESS_VIRTUAL_BYTES
    )
});

/// Current HNSW graph node count.
#[cfg(feature = "prometheus")]
pub static HNSW_NODES_COUNT: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_hnsw_nodes_count",
        "Number of nodes currently in the HNSW index",
        HNSW_NODES_COUNT
    )
});

/// HNSW graph logical memory usage in bytes.
#[cfg(feature = "prometheus")]
pub static HNSW_LOGICAL_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_hnsw_logical_bytes",
        "Estimated logical memory footprint of HNSW nodes and neighbor layers",
        HNSW_LOGICAL_BYTES
    )
});

/// Memory-mapped file resident bytes.
#[cfg(feature = "prometheus")]
pub static MMAP_RESIDENT_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_mmap_resident_bytes",
        "OS-reported resident bytes for VantaDB memory-mapped files when available",
        MMAP_RESIDENT_BYTES
    )
});

/// Volatile page cache entry count.
#[cfg(feature = "prometheus")]
pub static VOLATILE_CACHE_ENTRIES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_volatile_cache_entries",
        "Number of entries in the volatile hot-node cache",
        VOLATILE_CACHE_ENTRIES
    )
});

/// Volatile cache capacity in bytes.
#[cfg(feature = "prometheus")]
pub static VOLATILE_CACHE_CAP_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_volatile_cache_cap_bytes",
        "Maximum capacity in bytes for the volatile hot-node cache",
        VOLATILE_CACHE_CAP_BYTES
    )
});

/// Jemalloc allocated bytes.
#[cfg(feature = "prometheus")]
pub static JEMALLOC_ALLOCATED_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_allocated_bytes",
        "Number of bytes allocated by jemalloc",
        JEMALLOC_ALLOCATED_BYTES
    )
});

/// Jemalloc active bytes.
#[cfg(feature = "prometheus")]
pub static JEMALLOC_ACTIVE_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_active_bytes",
        "Number of bytes in active pages allocated by jemalloc",
        JEMALLOC_ACTIVE_BYTES
    )
});

/// Jemalloc metadata bytes.
#[cfg(feature = "prometheus")]
pub static JEMALLOC_METADATA_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_metadata_bytes",
        "Number of bytes dedicated to jemalloc metadata",
        JEMALLOC_METADATA_BYTES
    )
});

/// Jemalloc resident bytes.
#[cfg(feature = "prometheus")]
pub static JEMALLOC_RESIDENT_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_resident_bytes",
        "Number of bytes in resident pages allocated by jemalloc",
        JEMALLOC_RESIDENT_BYTES
    )
});

/// Jemalloc mapped bytes.
#[cfg(feature = "prometheus")]
pub static JEMALLOC_MAPPED_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_mapped_bytes",
        "Number of bytes mapped by jemalloc",
        JEMALLOC_MAPPED_BYTES
    )
});

/// Jemalloc retained bytes.
#[cfg(feature = "prometheus")]
pub static JEMALLOC_RETAINED_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_retained_bytes",
        "Number of bytes in retained pages by jemalloc",
        JEMALLOC_RETAINED_BYTES
    )
});

// ── HTTP request metrics (middleware in cli_server) ─────────────────────

#[cfg(feature = "prometheus")]
fn http_buckets() -> Option<Vec<f64>> {
    match exponential_buckets(0.5, 2.0, 12) {
        Ok(b) => Some(b),
        Err(e) => {
            tracing::warn!("Failed to create http_buckets: {e}");
            None
        }
    }
}

/// HTTP request duration histogram (labelled by method, path).
#[cfg(feature = "prometheus")]
pub static HTTP_REQUEST_DURATION_MS: LazyLock<Option<HistogramVec>> = LazyLock::new(|| {
    let buckets = match http_buckets() {
        Some(b) => b,
        None => return None,
    };
    let hist = match HistogramVec::new(
        prometheus::HistogramOpts::new(
            "vanta_http_request_duration_ms",
            "HTTP request latency in ms by method and route",
        )
        .buckets(buckets),
        &["method", "route"],
    ) {
        Ok(h) => h,
        Err(e) => {
            tracing::warn!("Failed to create HTTP_REQUEST_DURATION_MS: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(hist.clone())) {
        Ok(_) => Some(hist),
        Err(e) => {
            tracing::warn!("Failed to register HTTP_REQUEST_DURATION_MS: {e}");
            None
        }
    }
});

/// HTTP request total counter (labelled by method, path, status).
#[cfg(feature = "prometheus")]
pub static HTTP_REQUESTS_TOTAL: LazyLock<Option<IntCounterVec>> = LazyLock::new(|| {
    let counter = match IntCounterVec::new(
        prometheus::Opts::new(
            "vanta_http_requests_total",
            "Total HTTP requests by method, route, and status",
        ),
        &["method", "route", "status"],
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to create HTTP_REQUESTS_TOTAL: {e}");
            return None;
        }
    };
    match METRICS_REGISTRY.register(Box::new(counter.clone())) {
        Ok(_) => Some(counter),
        Err(e) => {
            tracing::warn!("Failed to register HTTP_REQUESTS_TOTAL: {e}");
            None
        }
    }
});

/// Record an HTTP request duration, method, route, and status for Prometheus metrics.
#[cfg(feature = "prometheus")]
pub fn record_http_request(method: &str, route: &str, status: u16, start: Instant) {
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    if let Some(hist) = HTTP_REQUEST_DURATION_MS.as_ref() {
        hist.with_label_values(&[method, route]).observe(elapsed_ms);
    }
    if let Some(counter) = HTTP_REQUESTS_TOTAL.as_ref() {
        counter
            .with_label_values(&[method, route, &status.to_string()])
            .inc();
    }
}

/// Record an HTTP request (no-op when the `prometheus` feature is disabled).
#[cfg(not(feature = "prometheus"))]
pub fn record_http_request(_method: &str, _route: &str, _status: u16, _start: Instant) {}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
#[cfg(feature = "prometheus")]
mod tests {
    use super::*;
    use web_time::Instant;

    /// Helper: deref a LazyLock<Option<M>> and return &M.
    fn init_metric<M>(lazy: &std::sync::LazyLock<Option<M>>) -> &M {
        lazy.as_ref()
            .as_ref()
            .expect("metric handle should be Some — registration must succeed")
    }

    #[test]
    fn test_metrics_registry_created() {
        // The global registry must be constructable and collectable.
        let _registry = &*METRICS_REGISTRY;
        let _families = METRICS_REGISTRY.gather();
        // If we got here without panicking, the registry works.
        // Each metric is a separate LazyLock and only registers when first
        // dereferenced (e.g. test_query_latency_histogram_init below).
    }

    #[test]
    fn test_query_latency_histogram_init() {
        let h = init_metric(&QUERY_LATENCY);
        h.observe(42.0);
        h.observe(100.0);
        // The histogram has default buckets; just verify it collected samples.
        assert!(h.get_sample_count() >= 2, "expected >= 2 observations");
    }

    #[test]
    fn test_oom_trips_counter_ops() {
        let c = init_metric(&OOM_TRIPS);
        let before = c.get();
        c.inc();
        assert!(c.get() > before, "counter should have incremented");
    }

    #[test]
    fn test_cache_hits_counter_ops() {
        let c = init_metric(&CACHE_HITS);
        let before = c.get();
        c.inc_by(5);
        assert!(
            c.get() >= before + 5,
            "counter should have incremented by 5"
        );
    }

    #[test]
    fn test_startup_latency_histogram_init() {
        let h = init_metric(&STARTUP_LATENCY_MS);
        h.observe(200.0);
        assert!(h.get_sample_count() >= 1);
    }

    #[test]
    fn test_wal_replay_latency_histogram_init() {
        let h = init_metric(&WAL_REPLAY_LATENCY_MS);
        h.observe(50.0);
        assert!(h.get_sample_count() >= 1);
    }

    #[test]
    fn test_ann_rebuild_latency_histogram_init() {
        let h = init_metric(&ANN_REBUILD_LATENCY_MS);
        h.observe(500.0);
        assert!(h.get_sample_count() >= 1);
    }

    #[test]
    fn test_derived_rebuild_latency_histogram_init() {
        let h = init_metric(&DERIVED_REBUILD_LATENCY_MS);
        h.observe(300.0);
        assert!(h.get_sample_count() >= 1);
    }

    #[test]
    fn test_text_index_rebuild_latency_histogram_init() {
        let h = init_metric(&TEXT_INDEX_REBUILD_LATENCY_MS);
        h.observe(150.0);
        assert!(h.get_sample_count() >= 1);
    }

    #[test]
    fn test_records_exported_imported_and_errors_counters() {
        let exported = init_metric(&RECORDS_EXPORTED);
        let imported = init_metric(&RECORDS_IMPORTED);
        let errors = init_metric(&IMPORT_ERRORS);

        exported.inc_by(10);
        imported.inc_by(5);
        errors.inc_by(2);

        assert!(exported.get() >= 10);
        assert!(imported.get() >= 5);
        assert!(errors.get() >= 2);
    }

    #[test]
    fn test_text_postings_written_counter() {
        let c = init_metric(&TEXT_POSTINGS_WRITTEN);
        c.inc_by(100);
        assert!(c.get() >= 100);
    }

    #[test]
    fn test_text_index_repairs_counter() {
        let c = init_metric(&TEXT_INDEX_REPAIRS);
        c.inc();
        assert!(c.get() >= 1);
    }

    #[test]
    fn test_text_lexical_metrics_init() {
        let hist = init_metric(&TEXT_LEXICAL_QUERY_LATENCY_MS);
        let queries = init_metric(&TEXT_LEXICAL_QUERIES);
        let scored = init_metric(&TEXT_CANDIDATES_SCORED);

        hist.observe(10.0);
        queries.inc_by(3);
        scored.inc_by(50);

        assert!(hist.get_sample_count() >= 1);
        assert!(queries.get() >= 3);
        assert!(scored.get() >= 50);
    }

    #[test]
    fn test_text_consistency_audits_init() {
        let audits = init_metric(&TEXT_CONSISTENCY_AUDITS);
        let failures = init_metric(&TEXT_CONSISTENCY_AUDIT_FAILURES);

        audits.inc_by(5);
        failures.inc();

        assert!(audits.get() >= 5);
        assert!(failures.get() >= 1);
    }

    #[test]
    fn test_hybrid_metrics_init() {
        let hist = init_metric(&HYBRID_QUERY_LATENCY_MS);
        let fused = init_metric(&HYBRID_CANDIDATES_FUSED);
        hist.observe(80.0);
        fused.inc_by(15);
        assert!(hist.get_sample_count() >= 1);
        assert!(fused.get() >= 15);
    }

    #[test]
    fn test_planner_route_counters_init() {
        let hybrid = init_metric(&PLANNER_HYBRID_QUERIES);
        let text = init_metric(&PLANNER_TEXT_ONLY_QUERIES);
        let vector = init_metric(&PLANNER_VECTOR_ONLY_QUERIES);

        hybrid.inc_by(7);
        text.inc();
        vector.inc_by(3);

        assert!(hybrid.get() >= 7);
        assert!(text.get() >= 1);
        assert!(vector.get() >= 3);
    }

    // ── Gauge registration tests ──────────────────────────────

    #[test]
    fn test_process_rss_gauge_init() {
        let g = init_metric(&PROCESS_RSS_BYTES);
        g.set(4_000_000);
        assert_eq!(g.get(), 4_000_000);
    }

    #[test]
    fn test_process_virtual_gauge_init() {
        let g = init_metric(&PROCESS_VIRTUAL_BYTES);
        g.set(8_000_000);
        assert!(g.get() >= 8_000_000);
    }

    #[test]
    fn test_hnsw_gauges_init() {
        let nodes = init_metric(&HNSW_NODES_COUNT);
        let logical = init_metric(&HNSW_LOGICAL_BYTES);
        nodes.set(100);
        logical.set(1_000_000);
        assert_eq!(nodes.get(), 100);
        assert_eq!(logical.get(), 1_000_000);
    }

    #[test]
    fn test_mmap_resident_gauge_init() {
        let g = init_metric(&MMAP_RESIDENT_BYTES);
        g.set(2_000_000);
        assert_eq!(g.get(), 2_000_000);
    }

    #[test]
    fn test_volatile_cache_gauges_init() {
        let entries = init_metric(&VOLATILE_CACHE_ENTRIES);
        let cap = init_metric(&VOLATILE_CACHE_CAP_BYTES);
        entries.set(50);
        cap.set(10_000_000);
        assert_eq!(entries.get(), 50);
        assert_eq!(cap.get(), 10_000_000);
    }

    #[test]
    fn test_jemalloc_gauges_init() {
        let allocated = init_metric(&JEMALLOC_ALLOCATED_BYTES);
        let active = init_metric(&JEMALLOC_ACTIVE_BYTES);
        let metadata = init_metric(&JEMALLOC_METADATA_BYTES);
        let resident = init_metric(&JEMALLOC_RESIDENT_BYTES);
        let mapped = init_metric(&JEMALLOC_MAPPED_BYTES);
        let retained = init_metric(&JEMALLOC_RETAINED_BYTES);

        allocated.set(1_000_000);
        active.set(800_000);
        metadata.set(100_000);
        resident.set(900_000);
        mapped.set(1_100_000);
        retained.set(50_000);

        assert_eq!(allocated.get(), 1_000_000);
        assert_eq!(active.get(), 800_000);
        assert_eq!(metadata.get(), 100_000);
        assert_eq!(resident.get(), 900_000);
        assert_eq!(mapped.get(), 1_100_000);
        assert_eq!(retained.get(), 50_000);
    }

    // ── HTTP metrics ──────────────────────────────────────────

    #[test]
    fn test_http_buckets_returns_valid_buckets() {
        let buckets = http_buckets().expect("http_buckets should succeed");
        assert_eq!(buckets.len(), 12, "expected 12 exponential buckets");
        // First bucket should be 0.5, last should be 0.5 * 2^11
        assert!((buckets[0] - 0.5).abs() < 1e-6);
        assert!((buckets[11] - 0.5 * 2.0f64.powi(11)).abs() < 1e-4);
    }

    #[test]
    fn test_http_request_duration_histogram_init() {
        let hist_vec = init_metric(&HTTP_REQUEST_DURATION_MS);
        hist_vec
            .with_label_values(&["GET", "/api/search"])
            .observe(15.0);
        hist_vec
            .with_label_values(&["POST", "/api/insert"])
            .observe(42.0);
        // Should not panic; we rely on the histogram having collected samples.
    }

    #[test]
    fn test_http_requests_total_counter_init() {
        let counter_vec = init_metric(&HTTP_REQUESTS_TOTAL);
        counter_vec
            .with_label_values(&["GET", "/api/search", "200"])
            .inc_by(3);
        counter_vec
            .with_label_values(&["POST", "/api/insert", "201"])
            .inc();
        let get_total = counter_vec
            .with_label_values(&["GET", "/api/search", "200"])
            .get();
        assert!(get_total >= 3, "GET requests should be >= 3");
    }

    #[test]
    fn test_record_http_request_no_panic() {
        let start = Instant::now();
        // Normal request
        record_http_request("GET", "/api/query", 200, start);
        // POST request
        let start2 = Instant::now();
        record_http_request("POST", "/api/insert", 201, start2);
        // Edge cases: empty method/route
        let start3 = Instant::now();
        record_http_request("", "", 0, start3);
        let start4 = Instant::now();
        record_http_request("GET", "/api/query", 200, start4);
    }

    #[test]
    fn test_record_http_request_various_statuses() {
        let start = Instant::now();
        record_http_request("GET", "/api/status", 200, start);
        let start2 = Instant::now();
        record_http_request("GET", "/api/status", 404, start2);
        let start3 = Instant::now();
        record_http_request("POST", "/api/data", 500, start3);
        // No assertion — just verifying no panic with varied inputs
    }

    // ── All metric handles should be Some ─────────────────────

    #[test]
    fn test_all_counter_handles_some() {
        assert!(OOM_TRIPS.as_ref().is_some());
        assert!(CACHE_HITS.as_ref().is_some());
        assert!(RECORDS_EXPORTED.as_ref().is_some());
        assert!(RECORDS_IMPORTED.as_ref().is_some());
        assert!(IMPORT_ERRORS.as_ref().is_some());
        assert!(TEXT_POSTINGS_WRITTEN.as_ref().is_some());
        assert!(TEXT_INDEX_REPAIRS.as_ref().is_some());
        assert!(TEXT_LEXICAL_QUERIES.as_ref().is_some());
        assert!(TEXT_CANDIDATES_SCORED.as_ref().is_some());
        assert!(TEXT_CONSISTENCY_AUDITS.as_ref().is_some());
        assert!(TEXT_CONSISTENCY_AUDIT_FAILURES.as_ref().is_some());
        assert!(HYBRID_CANDIDATES_FUSED.as_ref().is_some());
        assert!(PLANNER_HYBRID_QUERIES.as_ref().is_some());
        assert!(PLANNER_TEXT_ONLY_QUERIES.as_ref().is_some());
        assert!(PLANNER_VECTOR_ONLY_QUERIES.as_ref().is_some());
    }

    #[test]
    fn test_all_histogram_handles_some() {
        assert!(QUERY_LATENCY.as_ref().is_some());
        assert!(STARTUP_LATENCY_MS.as_ref().is_some());
        assert!(WAL_REPLAY_LATENCY_MS.as_ref().is_some());
        assert!(ANN_REBUILD_LATENCY_MS.as_ref().is_some());
        assert!(DERIVED_REBUILD_LATENCY_MS.as_ref().is_some());
        assert!(TEXT_INDEX_REBUILD_LATENCY_MS.as_ref().is_some());
        assert!(TEXT_LEXICAL_QUERY_LATENCY_MS.as_ref().is_some());
        assert!(HYBRID_QUERY_LATENCY_MS.as_ref().is_some());
    }

    #[test]
    fn test_all_gauge_handles_some() {
        assert!(PROCESS_RSS_BYTES.as_ref().is_some());
        assert!(PROCESS_VIRTUAL_BYTES.as_ref().is_some());
        assert!(HNSW_NODES_COUNT.as_ref().is_some());
        assert!(HNSW_LOGICAL_BYTES.as_ref().is_some());
        assert!(MMAP_RESIDENT_BYTES.as_ref().is_some());
        assert!(VOLATILE_CACHE_ENTRIES.as_ref().is_some());
        assert!(VOLATILE_CACHE_CAP_BYTES.as_ref().is_some());
        assert!(JEMALLOC_ALLOCATED_BYTES.as_ref().is_some());
        assert!(JEMALLOC_ACTIVE_BYTES.as_ref().is_some());
        assert!(JEMALLOC_METADATA_BYTES.as_ref().is_some());
        assert!(JEMALLOC_RESIDENT_BYTES.as_ref().is_some());
        assert!(JEMALLOC_MAPPED_BYTES.as_ref().is_some());
        assert!(JEMALLOC_RETAINED_BYTES.as_ref().is_some());
    }

    #[test]
    fn test_http_vec_handles_some() {
        assert!(HTTP_REQUEST_DURATION_MS.as_ref().is_some());
        assert!(HTTP_REQUESTS_TOTAL.as_ref().is_some());
    }
}
