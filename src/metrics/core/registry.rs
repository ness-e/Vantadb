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
