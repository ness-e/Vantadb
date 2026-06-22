#[cfg(feature = "prometheus")]
use prometheus::{
    exponential_buckets, Histogram, HistogramVec, IntCounter, IntCounterVec, IntGauge, Registry,
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(feature = "prometheus")]
use std::sync::LazyLock;
use web_time::Instant;

#[cfg(feature = "prometheus")]
pub static METRICS_REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

macro_rules! observe_histogram {
    ($hist:expr, $val:expr) => {
        #[cfg(feature = "prometheus")]
        if let Some(h) = $hist.as_ref() {
            h.observe($val as f64);
        }
    };
}

macro_rules! inc_counter {
    ($counter:expr) => {
        #[cfg(feature = "prometheus")]
        if let Some(c) = $counter.as_ref() {
            c.inc();
        }
    };
}

macro_rules! inc_counter_by {
    ($counter:expr, $val:expr) => {
        #[cfg(feature = "prometheus")]
        if let Some(c) = $counter.as_ref() {
            c.inc_by($val);
        }
    };
}

macro_rules! set_gauge {
    ($gauge:expr, $val:expr) => {
        #[cfg(feature = "prometheus")]
        if let Some(g) = $gauge.as_ref() {
            g.set($val as i64);
        }
    };
}

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

#[cfg(feature = "prometheus")]
pub static PROCESS_RSS_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_process_rss_bytes",
        "Process resident set size in bytes (via sysinfo)",
        PROCESS_RSS_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static PROCESS_VIRTUAL_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_process_virtual_bytes",
        "Process virtual memory in bytes (via sysinfo)",
        PROCESS_VIRTUAL_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static HNSW_NODES_COUNT: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_hnsw_nodes_count",
        "Number of nodes currently in the HNSW index",
        HNSW_NODES_COUNT
    )
});

#[cfg(feature = "prometheus")]
pub static HNSW_LOGICAL_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_hnsw_logical_bytes",
        "Estimated logical memory footprint of HNSW nodes and neighbor layers",
        HNSW_LOGICAL_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static MMAP_RESIDENT_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_mmap_resident_bytes",
        "OS-reported resident bytes for VantaDB memory-mapped files when available",
        MMAP_RESIDENT_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static VOLATILE_CACHE_ENTRIES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_volatile_cache_entries",
        "Number of entries in the volatile hot-node cache",
        VOLATILE_CACHE_ENTRIES
    )
});

#[cfg(feature = "prometheus")]
pub static VOLATILE_CACHE_CAP_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_volatile_cache_cap_bytes",
        "Maximum capacity in bytes for the volatile hot-node cache",
        VOLATILE_CACHE_CAP_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static JEMALLOC_ALLOCATED_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_allocated_bytes",
        "Number of bytes allocated by jemalloc",
        JEMALLOC_ALLOCATED_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static JEMALLOC_ACTIVE_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_active_bytes",
        "Number of bytes in active pages allocated by jemalloc",
        JEMALLOC_ACTIVE_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static JEMALLOC_METADATA_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_metadata_bytes",
        "Number of bytes dedicated to jemalloc metadata",
        JEMALLOC_METADATA_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static JEMALLOC_RESIDENT_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_resident_bytes",
        "Number of bytes in resident pages allocated by jemalloc",
        JEMALLOC_RESIDENT_BYTES
    )
});

#[cfg(feature = "prometheus")]
pub static JEMALLOC_MAPPED_BYTES: LazyLock<Option<IntGauge>> = LazyLock::new(|| {
    register_gauge!(
        "vanta_jemalloc_mapped_bytes",
        "Number of bytes mapped by jemalloc",
        JEMALLOC_MAPPED_BYTES
    )
});

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

#[cfg(not(feature = "prometheus"))]
pub fn record_http_request(_method: &str, _route: &str, _status: u16, _start: Instant) {}

static LAST_STARTUP_MS: AtomicU64 = AtomicU64::new(0);
static LAST_WAL_REPLAY_MS: AtomicU64 = AtomicU64::new(0);
static LAST_WAL_RECORDS_REPLAYED: AtomicU64 = AtomicU64::new(0);
static LAST_ANN_REBUILD_MS: AtomicU64 = AtomicU64::new(0);
static LAST_ANN_REBUILD_SCANNED_NODES: AtomicU64 = AtomicU64::new(0);
static LAST_DERIVED_REBUILD_MS: AtomicU64 = AtomicU64::new(0);
static LAST_TEXT_INDEX_REBUILD_MS: AtomicU64 = AtomicU64::new(0);
static LAST_TEXT_LEXICAL_QUERY_MS: AtomicU64 = AtomicU64::new(0);
static RECORDS_EXPORTED_TOTAL: AtomicU64 = AtomicU64::new(0);
static RECORDS_IMPORTED_TOTAL: AtomicU64 = AtomicU64::new(0);
static IMPORT_ERRORS_TOTAL: AtomicU64 = AtomicU64::new(0);
static DERIVED_PREFIX_SCANS_TOTAL: AtomicU64 = AtomicU64::new(0);
static DERIVED_FULL_SCAN_FALLBACKS_TOTAL: AtomicU64 = AtomicU64::new(0);
static TEXT_POSTINGS_WRITTEN_TOTAL: AtomicU64 = AtomicU64::new(0);
static TEXT_INDEX_REPAIRS_TOTAL: AtomicU64 = AtomicU64::new(0);
static LAST_PROCESS_RSS_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_PROCESS_VIRTUAL_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_HNSW_NODES_COUNT: AtomicU64 = AtomicU64::new(0);
static LAST_HNSW_LOGICAL_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_MMAP_RESIDENT_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_MMAP_RESIDENT_BYTES_PRESENT: AtomicBool = AtomicBool::new(false);
static LAST_VOLATILE_CACHE_ENTRIES: AtomicU64 = AtomicU64::new(0);
static LAST_VOLATILE_CACHE_CAP_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_JEMALLOC_ALLOCATED_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_JEMALLOC_ACTIVE_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_JEMALLOC_METADATA_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_JEMALLOC_RESIDENT_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_JEMALLOC_MAPPED_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_JEMALLOC_RETAINED_BYTES: AtomicU64 = AtomicU64::new(0);
static LAST_JEMALLOC_STATS_PRESENT: AtomicBool = AtomicBool::new(false);
static TEXT_LEXICAL_QUERIES_TOTAL: AtomicU64 = AtomicU64::new(0);
static TEXT_CANDIDATES_SCORED_TOTAL: AtomicU64 = AtomicU64::new(0);
static TEXT_CONSISTENCY_AUDITS_TOTAL: AtomicU64 = AtomicU64::new(0);
static TEXT_CONSISTENCY_AUDIT_FAILURES_TOTAL: AtomicU64 = AtomicU64::new(0);
static LAST_HYBRID_QUERY_MS: AtomicU64 = AtomicU64::new(0);
static HYBRID_CANDIDATES_FUSED_TOTAL: AtomicU64 = AtomicU64::new(0);
static PLANNER_HYBRID_QUERIES_TOTAL: AtomicU64 = AtomicU64::new(0);
static PLANNER_TEXT_ONLY_QUERIES_TOTAL: AtomicU64 = AtomicU64::new(0);
static PLANNER_VECTOR_ONLY_QUERIES_TOTAL: AtomicU64 = AtomicU64::new(0);

/// Per-subsystem memory breakdown snapshot.
///
/// These values are **observational**, not accounting-grade. RSS and virtual
/// memory come from `sysinfo::Process` and represent the OS-reported values
/// for the current process. HNSW node count and cache entries are logical
/// counters maintained by the engine.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MemoryBreakdownSnapshot {
    /// Process resident set size in bytes (OS-reported).
    pub process_rss_bytes: u64,
    /// Process virtual memory in bytes (OS-reported).
    pub process_virtual_bytes: u64,
    /// Number of nodes in the HNSW index.
    pub hnsw_nodes_count: u64,
    /// Estimated logical footprint of HNSW node/vector/edge allocations.
    pub hnsw_logical_bytes: u64,
    /// OS-reported resident bytes for mmap-backed files when available.
    pub mmap_resident_bytes: Option<u64>,
    /// Number of entries in the volatile hot-node cache.
    pub volatile_cache_entries: u64,
    /// Maximum capacity in bytes for the volatile cache.
    pub volatile_cache_cap_bytes: u64,
    /// Number of bytes allocated by jemalloc.
    pub jemalloc_allocated_bytes: Option<u64>,
    /// Number of bytes in active pages allocated by jemalloc.
    pub jemalloc_active_bytes: Option<u64>,
    /// Number of bytes dedicated to jemalloc metadata.
    pub jemalloc_metadata_bytes: Option<u64>,
    /// Number of bytes in resident pages allocated by jemalloc.
    pub jemalloc_resident_bytes: Option<u64>,
    /// Number of bytes mapped by jemalloc.
    pub jemalloc_mapped_bytes: Option<u64>,
    /// Number of bytes in retained pages by jemalloc.
    pub jemalloc_retained_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperationalMetricsSnapshot {
    pub startup_ms: u64,
    pub wal_replay_ms: u64,
    pub wal_records_replayed: u64,
    pub ann_rebuild_ms: u64,
    pub ann_rebuild_scanned_nodes: u64,
    pub derived_rebuild_ms: u64,
    pub text_index_rebuild_ms: u64,
    pub text_postings_written: u64,
    pub text_index_repairs: u64,
    pub text_lexical_queries: u64,
    pub text_lexical_query_ms: u64,
    pub text_candidates_scored: u64,
    pub text_consistency_audits: u64,
    pub text_consistency_audit_failures: u64,
    pub hybrid_query_ms: u64,
    pub hybrid_candidates_fused: u64,
    pub planner_hybrid_queries: u64,
    pub planner_text_only_queries: u64,
    pub planner_vector_only_queries: u64,
    pub records_exported: u64,
    pub records_imported: u64,
    pub import_errors: u64,
    pub derived_prefix_scans: u64,
    pub derived_full_scan_fallbacks: u64,
    /// Per-subsystem memory breakdown at snapshot time.
    pub memory: MemoryBreakdownSnapshot,
}

pub fn record_startup(startup_ms: u64, wal_replay_ms: u64, wal_records_replayed: u64) {
    LAST_STARTUP_MS.store(startup_ms, Ordering::Relaxed);
    LAST_WAL_REPLAY_MS.store(wal_replay_ms, Ordering::Relaxed);
    LAST_WAL_RECORDS_REPLAYED.store(wal_records_replayed, Ordering::Relaxed);
    observe_histogram!(STARTUP_LATENCY_MS, startup_ms);
    observe_histogram!(WAL_REPLAY_LATENCY_MS, wal_replay_ms);
}

pub fn record_ann_rebuild(duration_ms: u64, scanned_nodes: u64) {
    LAST_ANN_REBUILD_MS.store(duration_ms, Ordering::Relaxed);
    LAST_ANN_REBUILD_SCANNED_NODES.store(scanned_nodes, Ordering::Relaxed);
    observe_histogram!(ANN_REBUILD_LATENCY_MS, duration_ms);
}

pub fn record_derived_rebuild(duration_ms: u64) {
    LAST_DERIVED_REBUILD_MS.store(duration_ms, Ordering::Relaxed);
    observe_histogram!(DERIVED_REBUILD_LATENCY_MS, duration_ms);
}

pub fn record_text_index_rebuild(duration_ms: u64, postings_written: u64) {
    LAST_TEXT_INDEX_REBUILD_MS.store(duration_ms, Ordering::Relaxed);
    observe_histogram!(TEXT_INDEX_REBUILD_LATENCY_MS, duration_ms);
    record_text_postings_written(postings_written);
}

pub fn record_text_postings_written(postings_written: u64) {
    if postings_written == 0 {
        return;
    }
    TEXT_POSTINGS_WRITTEN_TOTAL.fetch_add(postings_written, Ordering::Relaxed);
    inc_counter_by!(TEXT_POSTINGS_WRITTEN, postings_written);
}

pub fn record_text_index_repair() {
    TEXT_INDEX_REPAIRS_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(TEXT_INDEX_REPAIRS);
}

pub fn record_text_lexical_query(duration_ms: u64, candidates_scored: u64) {
    LAST_TEXT_LEXICAL_QUERY_MS.store(duration_ms, Ordering::Relaxed);
    TEXT_LEXICAL_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    TEXT_CANDIDATES_SCORED_TOTAL.fetch_add(candidates_scored, Ordering::Relaxed);
    observe_histogram!(TEXT_LEXICAL_QUERY_LATENCY_MS, duration_ms);
    inc_counter!(TEXT_LEXICAL_QUERIES);
    inc_counter_by!(TEXT_CANDIDATES_SCORED, candidates_scored);
}

pub fn record_text_consistency_audit(failed: bool) {
    TEXT_CONSISTENCY_AUDITS_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(TEXT_CONSISTENCY_AUDITS);
    if failed {
        TEXT_CONSISTENCY_AUDIT_FAILURES_TOTAL.fetch_add(1, Ordering::Relaxed);
        inc_counter!(TEXT_CONSISTENCY_AUDIT_FAILURES);
    }
}

pub fn record_hybrid_query(duration_ms: u64, candidates_fused: u64) {
    LAST_HYBRID_QUERY_MS.store(duration_ms, Ordering::Relaxed);
    HYBRID_CANDIDATES_FUSED_TOTAL.fetch_add(candidates_fused, Ordering::Relaxed);
    observe_histogram!(HYBRID_QUERY_LATENCY_MS, duration_ms);
    inc_counter_by!(HYBRID_CANDIDATES_FUSED, candidates_fused);
}

pub fn record_planner_hybrid_query() {
    PLANNER_HYBRID_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(PLANNER_HYBRID_QUERIES);
}

pub fn record_planner_text_only_query() {
    PLANNER_TEXT_ONLY_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(PLANNER_TEXT_ONLY_QUERIES);
}

pub fn record_planner_vector_only_query() {
    PLANNER_VECTOR_ONLY_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(PLANNER_VECTOR_ONLY_QUERIES);
}

pub fn record_export(records: u64) {
    RECORDS_EXPORTED_TOTAL.fetch_add(records, Ordering::Relaxed);
    inc_counter_by!(RECORDS_EXPORTED, records);
}

pub fn record_import(records: u64, errors: u64) {
    RECORDS_IMPORTED_TOTAL.fetch_add(records, Ordering::Relaxed);
    IMPORT_ERRORS_TOTAL.fetch_add(errors, Ordering::Relaxed);
    inc_counter_by!(RECORDS_IMPORTED, records);
    inc_counter_by!(IMPORT_ERRORS, errors);
}

pub fn record_derived_prefix_scan() {
    DERIVED_PREFIX_SCANS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_derived_full_scan_fallback() {
    DERIVED_FULL_SCAN_FALLBACKS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

fn get_native_memory() -> Option<(u64, u64)> {
    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::io::Read;
        if let Ok(mut file) = File::open("/proc/self/statm") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                let mut parts = content.split_whitespace();
                if let (Some(size_str), Some(resident_str)) = (parts.next(), parts.next()) {
                    if let (Ok(size_pages), Ok(resident_pages)) =
                        (size_str.parse::<u64>(), resident_str.parse::<u64>())
                    {
                        let page_size = 4096; // Standard page size on Linux
                        return Some((resident_pages * page_size, size_pages * page_size));
                    }
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use libc::mach_task_basic_info;
        use mach2::task::task_info;
        use mach2::task_info::MACH_TASK_BASIC_INFO;
        use mach2::traps::mach_task_self;
        use std::mem;
        unsafe {
            let mut info: mach_task_basic_info = mem::zeroed();
            let mut count = (mem::size_of::<mach_task_basic_info>() / mem::size_of::<u32>()) as u32;
            let kr = task_info(
                mach_task_self(),
                MACH_TASK_BASIC_INFO,
                &mut info as *mut mach_task_basic_info as *mut _,
                &mut count,
            );
            if kr == 0 {
                return Some((info.resident_size as u64, info.virtual_size as u64));
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::mem;
        use windows_sys::Win32::System::ProcessStatus::{
            GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
        };
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        unsafe {
            let mut counters: PROCESS_MEMORY_COUNTERS = mem::zeroed();
            let process_handle = GetCurrentProcess();
            if GetProcessMemoryInfo(
                process_handle,
                &mut counters,
                mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ) != 0
            {
                return Some((
                    counters.WorkingSetSize as u64,
                    counters.PagefileUsage as u64,
                ));
            }
        }
    }

    None
}

#[cfg(all(feature = "jemalloc", not(target_os = "windows")))]
fn get_jemalloc_stats() -> Option<(u64, u64, u64, u64, u64, u64)> {
    let _ = tikv_jemalloc_ctl::epoch::advance();
    let allocated = tikv_jemalloc_ctl::stats::allocated::read().ok().unwrap_or(0) as u64;
    let active = tikv_jemalloc_ctl::stats::active::read().ok().unwrap_or(0) as u64;
    let metadata = tikv_jemalloc_ctl::stats::metadata::read().ok().unwrap_or(0) as u64;
    let resident = tikv_jemalloc_ctl::stats::resident::read().ok().unwrap_or(0) as u64;
    let mapped = tikv_jemalloc_ctl::stats::mapped::read().ok().unwrap_or(0) as u64;
    let retained = tikv_jemalloc_ctl::stats::retained::read().ok().unwrap_or(0) as u64;
    Some((allocated, active, metadata, resident, mapped, retained))
}

#[cfg(not(all(feature = "jemalloc", not(target_os = "windows"))))]
fn get_jemalloc_stats() -> Option<(u64, u64, u64, u64, u64, u64)> {
    None
}

pub fn record_memory_breakdown(
    hnsw_nodes: u64,
    hnsw_logical_bytes: u64,
    mmap_resident_bytes: Option<u64>,
    cache_entries: u64,
    cache_cap_bytes: u64,
) {
    #[cfg(any(
        feature = "sysinfo",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    ))]
    let (rss, virt) = _get_rss_virt();
    #[cfg(not(any(
        feature = "sysinfo",
        target_os = "linux",
        target_os = "macos",
        target_os = "windows"
    )))]
    let (rss, virt) = (0, 0);

    LAST_PROCESS_RSS_BYTES.store(rss, Ordering::Relaxed);
    LAST_PROCESS_VIRTUAL_BYTES.store(virt, Ordering::Relaxed);
    LAST_HNSW_NODES_COUNT.store(hnsw_nodes, Ordering::Relaxed);
    LAST_HNSW_LOGICAL_BYTES.store(hnsw_logical_bytes, Ordering::Relaxed);
    match mmap_resident_bytes {
        Some(bytes) => {
            LAST_MMAP_RESIDENT_BYTES.store(bytes, Ordering::Relaxed);
            LAST_MMAP_RESIDENT_BYTES_PRESENT.store(true, Ordering::Relaxed);
            set_gauge!(MMAP_RESIDENT_BYTES, bytes);
        }
        None => {
            LAST_MMAP_RESIDENT_BYTES.store(0, Ordering::Relaxed);
            LAST_MMAP_RESIDENT_BYTES_PRESENT.store(false, Ordering::Relaxed);
            set_gauge!(MMAP_RESIDENT_BYTES, 0);
        }
    }
    LAST_VOLATILE_CACHE_ENTRIES.store(cache_entries, Ordering::Relaxed);
    LAST_VOLATILE_CACHE_CAP_BYTES.store(cache_cap_bytes, Ordering::Relaxed);

    let jemalloc_stats = get_jemalloc_stats();
    if let Some((allocated, active, metadata, resident, mapped, retained)) = jemalloc_stats {
        LAST_JEMALLOC_ALLOCATED_BYTES.store(allocated, Ordering::Relaxed);
        LAST_JEMALLOC_ACTIVE_BYTES.store(active, Ordering::Relaxed);
        LAST_JEMALLOC_METADATA_BYTES.store(metadata, Ordering::Relaxed);
        LAST_JEMALLOC_RESIDENT_BYTES.store(resident, Ordering::Relaxed);
        LAST_JEMALLOC_MAPPED_BYTES.store(mapped, Ordering::Relaxed);
        LAST_JEMALLOC_RETAINED_BYTES.store(retained, Ordering::Relaxed);
        LAST_JEMALLOC_STATS_PRESENT.store(true, Ordering::Relaxed);

        set_gauge!(JEMALLOC_ALLOCATED_BYTES, allocated);
        set_gauge!(JEMALLOC_ACTIVE_BYTES, active);
        set_gauge!(JEMALLOC_METADATA_BYTES, metadata);
        set_gauge!(JEMALLOC_RESIDENT_BYTES, resident);
        set_gauge!(JEMALLOC_MAPPED_BYTES, mapped);
        set_gauge!(JEMALLOC_RETAINED_BYTES, retained);
    } else {
        LAST_JEMALLOC_STATS_PRESENT.store(false, Ordering::Relaxed);
    }

    set_gauge!(PROCESS_RSS_BYTES, rss);
    set_gauge!(PROCESS_VIRTUAL_BYTES, virt);
    set_gauge!(HNSW_NODES_COUNT, hnsw_nodes);
    set_gauge!(HNSW_LOGICAL_BYTES, hnsw_logical_bytes);
    set_gauge!(VOLATILE_CACHE_ENTRIES, cache_entries);
    set_gauge!(VOLATILE_CACHE_CAP_BYTES, cache_cap_bytes);
}

fn _get_rss_virt() -> (u64, u64) {
    if let Some((rss, virt)) = get_native_memory() {
        return (rss, virt);
    }
    #[cfg(feature = "sysinfo")]
    {
        tracing::warn!("Native memory telemetry failed. Falling back to sysinfo.");
        use sysinfo::{Pid, System};
        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new();
        sys.refresh_process(pid);
        if let Some(proc) = sys.process(pid) {
            return (proc.memory(), proc.virtual_memory());
        }
    }
    (0, 0)
}

pub fn memory_breakdown_snapshot() -> MemoryBreakdownSnapshot {
    let mmap_resident_bytes = LAST_MMAP_RESIDENT_BYTES_PRESENT
        .load(Ordering::Relaxed)
        .then(|| LAST_MMAP_RESIDENT_BYTES.load(Ordering::Relaxed));

    let jemalloc_present = LAST_JEMALLOC_STATS_PRESENT.load(Ordering::Relaxed);

    MemoryBreakdownSnapshot {
        process_rss_bytes: LAST_PROCESS_RSS_BYTES.load(Ordering::Relaxed),
        process_virtual_bytes: LAST_PROCESS_VIRTUAL_BYTES.load(Ordering::Relaxed),
        hnsw_nodes_count: LAST_HNSW_NODES_COUNT.load(Ordering::Relaxed),
        hnsw_logical_bytes: LAST_HNSW_LOGICAL_BYTES.load(Ordering::Relaxed),
        mmap_resident_bytes,
        volatile_cache_entries: LAST_VOLATILE_CACHE_ENTRIES.load(Ordering::Relaxed),
        volatile_cache_cap_bytes: LAST_VOLATILE_CACHE_CAP_BYTES.load(Ordering::Relaxed),
        jemalloc_allocated_bytes: jemalloc_present.then(|| LAST_JEMALLOC_ALLOCATED_BYTES.load(Ordering::Relaxed)),
        jemalloc_active_bytes: jemalloc_present.then(|| LAST_JEMALLOC_ACTIVE_BYTES.load(Ordering::Relaxed)),
        jemalloc_metadata_bytes: jemalloc_present.then(|| LAST_JEMALLOC_METADATA_BYTES.load(Ordering::Relaxed)),
        jemalloc_resident_bytes: jemalloc_present.then(|| LAST_JEMALLOC_RESIDENT_BYTES.load(Ordering::Relaxed)),
        jemalloc_mapped_bytes: jemalloc_present.then(|| LAST_JEMALLOC_MAPPED_BYTES.load(Ordering::Relaxed)),
        jemalloc_retained_bytes: jemalloc_present.then(|| LAST_JEMALLOC_RETAINED_BYTES.load(Ordering::Relaxed)),
    }
}

pub fn operational_metrics_snapshot() -> OperationalMetricsSnapshot {
    OperationalMetricsSnapshot {
        startup_ms: LAST_STARTUP_MS.load(Ordering::Relaxed),
        wal_replay_ms: LAST_WAL_REPLAY_MS.load(Ordering::Relaxed),
        wal_records_replayed: LAST_WAL_RECORDS_REPLAYED.load(Ordering::Relaxed),
        ann_rebuild_ms: LAST_ANN_REBUILD_MS.load(Ordering::Relaxed),
        ann_rebuild_scanned_nodes: LAST_ANN_REBUILD_SCANNED_NODES.load(Ordering::Relaxed),
        derived_rebuild_ms: LAST_DERIVED_REBUILD_MS.load(Ordering::Relaxed),
        text_index_rebuild_ms: LAST_TEXT_INDEX_REBUILD_MS.load(Ordering::Relaxed),
        text_postings_written: TEXT_POSTINGS_WRITTEN_TOTAL.load(Ordering::Relaxed),
        text_index_repairs: TEXT_INDEX_REPAIRS_TOTAL.load(Ordering::Relaxed),
        text_lexical_queries: TEXT_LEXICAL_QUERIES_TOTAL.load(Ordering::Relaxed),
        text_lexical_query_ms: LAST_TEXT_LEXICAL_QUERY_MS.load(Ordering::Relaxed),
        text_candidates_scored: TEXT_CANDIDATES_SCORED_TOTAL.load(Ordering::Relaxed),
        text_consistency_audits: TEXT_CONSISTENCY_AUDITS_TOTAL.load(Ordering::Relaxed),
        text_consistency_audit_failures: TEXT_CONSISTENCY_AUDIT_FAILURES_TOTAL
            .load(Ordering::Relaxed),
        hybrid_query_ms: LAST_HYBRID_QUERY_MS.load(Ordering::Relaxed),
        hybrid_candidates_fused: HYBRID_CANDIDATES_FUSED_TOTAL.load(Ordering::Relaxed),
        planner_hybrid_queries: PLANNER_HYBRID_QUERIES_TOTAL.load(Ordering::Relaxed),
        planner_text_only_queries: PLANNER_TEXT_ONLY_QUERIES_TOTAL.load(Ordering::Relaxed),
        planner_vector_only_queries: PLANNER_VECTOR_ONLY_QUERIES_TOTAL.load(Ordering::Relaxed),
        records_exported: RECORDS_EXPORTED_TOTAL.load(Ordering::Relaxed),
        records_imported: RECORDS_IMPORTED_TOTAL.load(Ordering::Relaxed),
        import_errors: IMPORT_ERRORS_TOTAL.load(Ordering::Relaxed),
        derived_prefix_scans: DERIVED_PREFIX_SCANS_TOTAL.load(Ordering::Relaxed),
        derived_full_scan_fallbacks: DERIVED_FULL_SCAN_FALLBACKS_TOTAL.load(Ordering::Relaxed),
        memory: memory_breakdown_snapshot(),
    }
}

/// Export utility suitable for the `/metrics` Axum endpoint
#[cfg(feature = "prometheus")]
pub fn export_metrics_text() -> String {
    use prometheus::TextEncoder;
    let encoder = TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = String::new();
    if encoder.encode_utf8(&metric_families, &mut buffer).is_err() {
        return String::new();
    }
    buffer
}

#[cfg(not(feature = "prometheus"))]
pub fn export_metrics_text() -> String {
    String::new()
}
