#[cfg(feature = "prometheus")]
use prometheus::{
    exponential_buckets, Histogram, HistogramVec, IntCounter, IntCounterVec, IntGauge, Registry,
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::LazyLock;
use web_time::Instant;

#[cfg(feature = "prometheus")]
pub static METRICS_REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

macro_rules! observe_histogram {
    ($hist:expr, $val:expr) => {
        #[cfg(feature = "prometheus")]
        $hist.observe($val as f64);
    };
}

macro_rules! inc_counter {
    ($counter:expr) => {
        #[cfg(feature = "prometheus")]
        $counter.inc();
    };
}

macro_rules! inc_counter_by {
    ($counter:expr, $val:expr) => {
        #[cfg(feature = "prometheus")]
        $counter.inc_by($val);
    };
}

macro_rules! set_gauge {
    ($gauge:expr, $val:expr) => {
        #[cfg(feature = "prometheus")]
        $gauge.set($val as i64);
    };
}

#[cfg(feature = "prometheus")]
pub static QUERY_LATENCY: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_query_latency_ms",
        "Query execution times in ms",
    ))
    .expect(
        "FATAL: Failed to create QUERY_LATENCY histogram - metric name conflict or invalid config",
    );
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register QUERY_LATENCY - registry error");
    hist
});

#[cfg(feature = "prometheus")]
pub static OOM_TRIPS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new("vanta_oom_circuit_trips_total", "Governor OOM prevents")
        .expect("FATAL: Failed to create OOM_TRIPS counter - metric name conflict");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register OOM_TRIPS - registry error");
    counter
});

#[cfg(feature = "prometheus")]
pub static CACHE_HITS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new("vanta_cache_hits_total", "CP-Index fast path matches")
        .expect("FATAL: Failed to create CACHE_HITS counter - metric name conflict");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register CACHE_HITS - registry error");
    counter
});

#[cfg(feature = "prometheus")]
pub static STARTUP_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_startup_latency_ms",
        "Storage engine startup time in ms",
    ))
    .expect("FATAL: Failed to create STARTUP_LATENCY_MS histogram");
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register STARTUP_LATENCY_MS");
    hist
});

#[cfg(feature = "prometheus")]
pub static WAL_REPLAY_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_wal_replay_latency_ms",
        "WAL replay time in ms during startup",
    ))
    .expect("FATAL: Failed to create WAL_REPLAY_LATENCY_MS histogram");
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register WAL_REPLAY_LATENCY_MS");
    hist
});

#[cfg(feature = "prometheus")]
pub static ANN_REBUILD_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_ann_rebuild_latency_ms",
        "Manual or startup ANN rebuild time in ms",
    ))
    .expect("FATAL: Failed to create ANN_REBUILD_LATENCY_MS histogram");
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register ANN_REBUILD_LATENCY_MS");
    hist
});

#[cfg(feature = "prometheus")]
pub static DERIVED_REBUILD_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_derived_rebuild_latency_ms",
        "Derived namespace/payload index rebuild time in ms",
    ))
    .expect("FATAL: Failed to create DERIVED_REBUILD_LATENCY_MS histogram");
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register DERIVED_REBUILD_LATENCY_MS");
    hist
});

#[cfg(feature = "prometheus")]
pub static TEXT_INDEX_REBUILD_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_text_index_rebuild_latency_ms",
        "Derived text index rebuild time in ms",
    ))
    .expect("FATAL: Failed to create TEXT_INDEX_REBUILD_LATENCY_MS histogram");
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register TEXT_INDEX_REBUILD_LATENCY_MS");
    hist
});

#[cfg(feature = "prometheus")]
pub static RECORDS_EXPORTED: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_records_exported_total",
        "Persistent memory records exported",
    )
    .expect("FATAL: Failed to create RECORDS_EXPORTED counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register RECORDS_EXPORTED");
    counter
});

#[cfg(feature = "prometheus")]
pub static RECORDS_IMPORTED: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_records_imported_total",
        "Persistent memory records imported",
    )
    .expect("FATAL: Failed to create RECORDS_IMPORTED counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register RECORDS_IMPORTED");
    counter
});

#[cfg(feature = "prometheus")]
pub static IMPORT_ERRORS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_import_errors_total",
        "Persistent memory import errors",
    )
    .expect("FATAL: Failed to create IMPORT_ERRORS counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register IMPORT_ERRORS");
    counter
});

#[cfg(feature = "prometheus")]
pub static TEXT_POSTINGS_WRITTEN: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_postings_written_total",
        "Derived text index postings written",
    )
    .expect("FATAL: Failed to create TEXT_POSTINGS_WRITTEN counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register TEXT_POSTINGS_WRITTEN");
    counter
});

#[cfg(feature = "prometheus")]
pub static TEXT_INDEX_REPAIRS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_index_repairs_total",
        "Derived text index repairs from canonical records",
    )
    .expect("FATAL: Failed to create TEXT_INDEX_REPAIRS counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register TEXT_INDEX_REPAIRS");
    counter
});

#[cfg(feature = "prometheus")]
pub static TEXT_LEXICAL_QUERY_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_text_lexical_query_latency_ms",
        "BM25 lexical memory query time in ms",
    ))
    .expect("FATAL: Failed to create TEXT_LEXICAL_QUERY_LATENCY_MS histogram");
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register TEXT_LEXICAL_QUERY_LATENCY_MS");
    hist
});

#[cfg(feature = "prometheus")]
pub static TEXT_LEXICAL_QUERIES: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_lexical_queries_total",
        "BM25 lexical memory queries executed",
    )
    .expect("FATAL: Failed to create TEXT_LEXICAL_QUERIES counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register TEXT_LEXICAL_QUERIES");
    counter
});

#[cfg(feature = "prometheus")]
pub static TEXT_CANDIDATES_SCORED: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_candidates_scored_total",
        "BM25 lexical candidates scored",
    )
    .expect("FATAL: Failed to create TEXT_CANDIDATES_SCORED counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register TEXT_CANDIDATES_SCORED");
    counter
});

#[cfg(feature = "prometheus")]
pub static TEXT_CONSISTENCY_AUDITS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_consistency_audits_total",
        "Structural text index consistency audits executed",
    )
    .expect("FATAL: Failed to create TEXT_CONSISTENCY_AUDITS counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register TEXT_CONSISTENCY_AUDITS");
    counter
});

#[cfg(feature = "prometheus")]
pub static TEXT_CONSISTENCY_AUDIT_FAILURES: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_consistency_audit_failures_total",
        "Structural text index consistency audits that detected mismatch",
    )
    .expect("FATAL: Failed to create TEXT_CONSISTENCY_AUDIT_FAILURES counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register TEXT_CONSISTENCY_AUDIT_FAILURES");
    counter
});

#[cfg(feature = "prometheus")]
pub static HYBRID_QUERY_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_hybrid_query_latency_ms",
        "Hybrid memory query fusion time in ms",
    ))
    .expect("FATAL: Failed to create HYBRID_QUERY_LATENCY_MS histogram");
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register HYBRID_QUERY_LATENCY_MS");
    hist
});

#[cfg(feature = "prometheus")]
pub static HYBRID_CANDIDATES_FUSED: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_hybrid_candidates_fused_total",
        "Unique memory candidates fused by hybrid retrieval",
    )
    .expect("FATAL: Failed to create HYBRID_CANDIDATES_FUSED counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register HYBRID_CANDIDATES_FUSED");
    counter
});

#[cfg(feature = "prometheus")]
pub static PLANNER_HYBRID_QUERIES: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_planner_hybrid_queries_total",
        "Memory searches planned as hybrid text+vector retrieval",
    )
    .expect("FATAL: Failed to create PLANNER_HYBRID_QUERIES counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register PLANNER_HYBRID_QUERIES");
    counter
});

#[cfg(feature = "prometheus")]
pub static PLANNER_TEXT_ONLY_QUERIES: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_planner_text_only_queries_total",
        "Memory searches planned as text-only retrieval",
    )
    .expect("FATAL: Failed to create PLANNER_TEXT_ONLY_QUERIES counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register PLANNER_TEXT_ONLY_QUERIES");
    counter
});

#[cfg(feature = "prometheus")]
pub static PLANNER_VECTOR_ONLY_QUERIES: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_planner_vector_only_queries_total",
        "Memory searches planned as vector-only retrieval",
    )
    .expect("FATAL: Failed to create PLANNER_VECTOR_ONLY_QUERIES counter");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register PLANNER_VECTOR_ONLY_QUERIES");
    counter
});

// ── Memory breakdown gauges ──────────────────────────────────────────────

#[cfg(feature = "prometheus")]
pub static PROCESS_RSS_BYTES: LazyLock<IntGauge> = LazyLock::new(|| {
    let gauge = IntGauge::new(
        "vanta_process_rss_bytes",
        "Process resident set size in bytes (via sysinfo)",
    )
    .expect("FATAL: Failed to create PROCESS_RSS_BYTES gauge");
    METRICS_REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("FATAL: Failed to register PROCESS_RSS_BYTES");
    gauge
});

#[cfg(feature = "prometheus")]
pub static PROCESS_VIRTUAL_BYTES: LazyLock<IntGauge> = LazyLock::new(|| {
    let gauge = IntGauge::new(
        "vanta_process_virtual_bytes",
        "Process virtual memory in bytes (via sysinfo)",
    )
    .expect("FATAL: Failed to create PROCESS_VIRTUAL_BYTES gauge");
    METRICS_REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("FATAL: Failed to register PROCESS_VIRTUAL_BYTES");
    gauge
});

#[cfg(feature = "prometheus")]
pub static HNSW_NODES_COUNT: LazyLock<IntGauge> = LazyLock::new(|| {
    let gauge = IntGauge::new(
        "vanta_hnsw_nodes_count",
        "Number of nodes currently in the HNSW index",
    )
    .expect("FATAL: Failed to create HNSW_NODES_COUNT gauge");
    METRICS_REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("FATAL: Failed to register HNSW_NODES_COUNT");
    gauge
});

#[cfg(feature = "prometheus")]
pub static HNSW_LOGICAL_BYTES: LazyLock<IntGauge> = LazyLock::new(|| {
    let gauge = IntGauge::new(
        "vanta_hnsw_logical_bytes",
        "Estimated logical memory footprint of HNSW nodes and neighbor layers",
    )
    .expect("FATAL: Failed to create HNSW_LOGICAL_BYTES gauge");
    METRICS_REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("FATAL: Failed to register HNSW_LOGICAL_BYTES");
    gauge
});

#[cfg(feature = "prometheus")]
pub static MMAP_RESIDENT_BYTES: LazyLock<IntGauge> = LazyLock::new(|| {
    let gauge = IntGauge::new(
        "vanta_mmap_resident_bytes",
        "OS-reported resident bytes for VantaDB memory-mapped files when available",
    )
    .expect("FATAL: Failed to create MMAP_RESIDENT_BYTES gauge");
    METRICS_REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("FATAL: Failed to register MMAP_RESIDENT_BYTES");
    gauge
});

#[cfg(feature = "prometheus")]
pub static VOLATILE_CACHE_ENTRIES: LazyLock<IntGauge> = LazyLock::new(|| {
    let gauge = IntGauge::new(
        "vanta_volatile_cache_entries",
        "Number of entries in the volatile hot-node cache",
    )
    .expect("FATAL: Failed to create VOLATILE_CACHE_ENTRIES gauge");
    METRICS_REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("FATAL: Failed to register VOLATILE_CACHE_ENTRIES");
    gauge
});

#[cfg(feature = "prometheus")]
pub static VOLATILE_CACHE_CAP_BYTES: LazyLock<IntGauge> = LazyLock::new(|| {
    let gauge = IntGauge::new(
        "vanta_volatile_cache_cap_bytes",
        "Maximum capacity in bytes for the volatile hot-node cache",
    )
    .expect("FATAL: Failed to create VOLATILE_CACHE_CAP_BYTES gauge");
    METRICS_REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("FATAL: Failed to register VOLATILE_CACHE_CAP_BYTES");
    gauge
});

// ── HTTP request metrics (middleware in cli_server) ─────────────────────

#[cfg(feature = "prometheus")]
fn http_buckets() -> Vec<f64> {
    exponential_buckets(0.5, 2.0, 12).expect("FATAL: http_buckets")
}

#[cfg(feature = "prometheus")]
pub static HTTP_REQUEST_DURATION_MS: LazyLock<HistogramVec> = LazyLock::new(|| {
    let hist = HistogramVec::new(
        prometheus::HistogramOpts::new(
            "vanta_http_request_duration_ms",
            "HTTP request latency in ms by method and route",
        )
        .buckets(http_buckets()),
        &["method", "route"],
    )
    .expect("FATAL: Failed to create HTTP_REQUEST_DURATION_MS");
    METRICS_REGISTRY
        .register(Box::new(hist.clone()))
        .expect("FATAL: Failed to register HTTP_REQUEST_DURATION_MS");
    hist
});

#[cfg(feature = "prometheus")]
pub static HTTP_REQUESTS_TOTAL: LazyLock<IntCounterVec> = LazyLock::new(|| {
    let counter = IntCounterVec::new(
        prometheus::Opts::new(
            "vanta_http_requests_total",
            "Total HTTP requests by method, route, and status",
        ),
        &["method", "route", "status"],
    )
    .expect("FATAL: Failed to create HTTP_REQUESTS_TOTAL");
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .expect("FATAL: Failed to register HTTP_REQUESTS_TOTAL");
    counter
});

#[cfg(feature = "prometheus")]
pub fn record_http_request(method: &str, route: &str, status: u16, start: Instant) {
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
    HTTP_REQUEST_DURATION_MS
        .with_label_values(&[method, route])
        .observe(elapsed_ms);
    HTTP_REQUESTS_TOTAL
        .with_label_values(&[method, route, &status.to_string()])
        .inc();
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

pub fn record_memory_breakdown(
    hnsw_nodes: u64,
    hnsw_logical_bytes: u64,
    mmap_resident_bytes: Option<u64>,
    cache_entries: u64,
    cache_cap_bytes: u64,
) {
    #[cfg(any(feature = "sysinfo", target_os = "linux", target_os = "macos", target_os = "windows"))]
    let (rss, virt) = _get_rss_virt();
    #[cfg(not(any(feature = "sysinfo", target_os = "linux", target_os = "macos", target_os = "windows")))]
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
        match sys.process(pid) {
            Some(proc) => return (proc.memory(), proc.virtual_memory()),
            None => {}
        }
    }
    (0, 0)
}

pub fn memory_breakdown_snapshot() -> MemoryBreakdownSnapshot {
    let mmap_resident_bytes = LAST_MMAP_RESIDENT_BYTES_PRESENT
        .load(Ordering::Relaxed)
        .then(|| LAST_MMAP_RESIDENT_BYTES.load(Ordering::Relaxed));

    MemoryBreakdownSnapshot {
        process_rss_bytes: LAST_PROCESS_RSS_BYTES.load(Ordering::Relaxed),
        process_virtual_bytes: LAST_PROCESS_VIRTUAL_BYTES.load(Ordering::Relaxed),
        hnsw_nodes_count: LAST_HNSW_NODES_COUNT.load(Ordering::Relaxed),
        hnsw_logical_bytes: LAST_HNSW_LOGICAL_BYTES.load(Ordering::Relaxed),
        mmap_resident_bytes,
        volatile_cache_entries: LAST_VOLATILE_CACHE_ENTRIES.load(Ordering::Relaxed),
        volatile_cache_cap_bytes: LAST_VOLATILE_CACHE_CAP_BYTES.load(Ordering::Relaxed),
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
