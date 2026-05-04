use prometheus::{Histogram, IntCounter, Registry};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::LazyLock;

// Ensure singleton metrics registry across the binary
pub static METRICS_REGISTRY: LazyLock<Registry> = LazyLock::new(Registry::new);

pub static QUERY_LATENCY: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_query_latency_ms",
        "Query execution times in ms",
    ))
    .unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static OOM_TRIPS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter =
        IntCounter::new("vanta_oom_circuit_trips_total", "Governor OOM prevents").unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static CACHE_HITS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new("vanta_cache_hits_total", "CP-Index fast path matches").unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static STARTUP_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_startup_latency_ms",
        "Storage engine startup time in ms",
    ))
    .unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static WAL_REPLAY_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_wal_replay_latency_ms",
        "WAL replay time in ms during startup",
    ))
    .unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static ANN_REBUILD_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_ann_rebuild_latency_ms",
        "Manual or startup ANN rebuild time in ms",
    ))
    .unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static DERIVED_REBUILD_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_derived_rebuild_latency_ms",
        "Derived namespace/payload index rebuild time in ms",
    ))
    .unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static TEXT_INDEX_REBUILD_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_text_index_rebuild_latency_ms",
        "Derived text index rebuild time in ms",
    ))
    .unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static RECORDS_EXPORTED: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_records_exported_total",
        "Persistent memory records exported",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static RECORDS_IMPORTED: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_records_imported_total",
        "Persistent memory records imported",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static IMPORT_ERRORS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_import_errors_total",
        "Persistent memory import errors",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static TEXT_POSTINGS_WRITTEN: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_postings_written_total",
        "Derived text index postings written",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static TEXT_INDEX_REPAIRS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_index_repairs_total",
        "Derived text index repairs from canonical records",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static TEXT_LEXICAL_QUERY_LATENCY_MS: LazyLock<Histogram> = LazyLock::new(|| {
    let hist = Histogram::with_opts(prometheus::HistogramOpts::new(
        "vanta_text_lexical_query_latency_ms",
        "BM25 lexical memory query time in ms",
    ))
    .unwrap();
    METRICS_REGISTRY.register(Box::new(hist.clone())).unwrap();
    hist
});

pub static TEXT_LEXICAL_QUERIES: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_lexical_queries_total",
        "BM25 lexical memory queries executed",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static TEXT_CANDIDATES_SCORED: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_candidates_scored_total",
        "BM25 lexical candidates scored",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static TEXT_CONSISTENCY_AUDITS: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_consistency_audits_total",
        "Structural text index consistency audits executed",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

pub static TEXT_CONSISTENCY_AUDIT_FAILURES: LazyLock<IntCounter> = LazyLock::new(|| {
    let counter = IntCounter::new(
        "vanta_text_consistency_audit_failures_total",
        "Structural text index consistency audits that detected mismatch",
    )
    .unwrap();
    METRICS_REGISTRY
        .register(Box::new(counter.clone()))
        .unwrap();
    counter
});

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
static TEXT_LEXICAL_QUERIES_TOTAL: AtomicU64 = AtomicU64::new(0);
static TEXT_CANDIDATES_SCORED_TOTAL: AtomicU64 = AtomicU64::new(0);
static TEXT_CONSISTENCY_AUDITS_TOTAL: AtomicU64 = AtomicU64::new(0);
static TEXT_CONSISTENCY_AUDIT_FAILURES_TOTAL: AtomicU64 = AtomicU64::new(0);

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
    pub records_exported: u64,
    pub records_imported: u64,
    pub import_errors: u64,
    pub derived_prefix_scans: u64,
    pub derived_full_scan_fallbacks: u64,
}

pub fn record_startup(startup_ms: u64, wal_replay_ms: u64, wal_records_replayed: u64) {
    LAST_STARTUP_MS.store(startup_ms, Ordering::Relaxed);
    LAST_WAL_REPLAY_MS.store(wal_replay_ms, Ordering::Relaxed);
    LAST_WAL_RECORDS_REPLAYED.store(wal_records_replayed, Ordering::Relaxed);
    STARTUP_LATENCY_MS.observe(startup_ms as f64);
    WAL_REPLAY_LATENCY_MS.observe(wal_replay_ms as f64);
}

pub fn record_ann_rebuild(duration_ms: u64, scanned_nodes: u64) {
    LAST_ANN_REBUILD_MS.store(duration_ms, Ordering::Relaxed);
    LAST_ANN_REBUILD_SCANNED_NODES.store(scanned_nodes, Ordering::Relaxed);
    ANN_REBUILD_LATENCY_MS.observe(duration_ms as f64);
}

pub fn record_derived_rebuild(duration_ms: u64) {
    LAST_DERIVED_REBUILD_MS.store(duration_ms, Ordering::Relaxed);
    DERIVED_REBUILD_LATENCY_MS.observe(duration_ms as f64);
}

pub fn record_text_index_rebuild(duration_ms: u64, postings_written: u64) {
    LAST_TEXT_INDEX_REBUILD_MS.store(duration_ms, Ordering::Relaxed);
    TEXT_INDEX_REBUILD_LATENCY_MS.observe(duration_ms as f64);
    record_text_postings_written(postings_written);
}

pub fn record_text_postings_written(postings_written: u64) {
    if postings_written == 0 {
        return;
    }
    TEXT_POSTINGS_WRITTEN_TOTAL.fetch_add(postings_written, Ordering::Relaxed);
    TEXT_POSTINGS_WRITTEN.inc_by(postings_written);
}

pub fn record_text_index_repair() {
    TEXT_INDEX_REPAIRS_TOTAL.fetch_add(1, Ordering::Relaxed);
    TEXT_INDEX_REPAIRS.inc();
}

pub fn record_text_lexical_query(duration_ms: u64, candidates_scored: u64) {
    LAST_TEXT_LEXICAL_QUERY_MS.store(duration_ms, Ordering::Relaxed);
    TEXT_LEXICAL_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    TEXT_CANDIDATES_SCORED_TOTAL.fetch_add(candidates_scored, Ordering::Relaxed);
    TEXT_LEXICAL_QUERY_LATENCY_MS.observe(duration_ms as f64);
    TEXT_LEXICAL_QUERIES.inc();
    TEXT_CANDIDATES_SCORED.inc_by(candidates_scored);
}

pub fn record_text_consistency_audit(failed: bool) {
    TEXT_CONSISTENCY_AUDITS_TOTAL.fetch_add(1, Ordering::Relaxed);
    TEXT_CONSISTENCY_AUDITS.inc();
    if failed {
        TEXT_CONSISTENCY_AUDIT_FAILURES_TOTAL.fetch_add(1, Ordering::Relaxed);
        TEXT_CONSISTENCY_AUDIT_FAILURES.inc();
    }
}

pub fn record_export(records: u64) {
    RECORDS_EXPORTED_TOTAL.fetch_add(records, Ordering::Relaxed);
    RECORDS_EXPORTED.inc_by(records);
}

pub fn record_import(records: u64, errors: u64) {
    RECORDS_IMPORTED_TOTAL.fetch_add(records, Ordering::Relaxed);
    IMPORT_ERRORS_TOTAL.fetch_add(errors, Ordering::Relaxed);
    RECORDS_IMPORTED.inc_by(records);
    IMPORT_ERRORS.inc_by(errors);
}

pub fn record_derived_prefix_scan() {
    DERIVED_PREFIX_SCANS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

pub fn record_derived_full_scan_fallback() {
    DERIVED_FULL_SCAN_FALLBACKS_TOTAL.fetch_add(1, Ordering::Relaxed);
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
        records_exported: RECORDS_EXPORTED_TOTAL.load(Ordering::Relaxed),
        records_imported: RECORDS_IMPORTED_TOTAL.load(Ordering::Relaxed),
        import_errors: IMPORT_ERRORS_TOTAL.load(Ordering::Relaxed),
        derived_prefix_scans: DERIVED_PREFIX_SCANS_TOTAL.load(Ordering::Relaxed),
        derived_full_scan_fallbacks: DERIVED_FULL_SCAN_FALLBACKS_TOTAL.load(Ordering::Relaxed),
    }
}

/// Export utility suitable for the `/metrics` Axum endpoint
pub fn export_metrics_text() -> String {
    use prometheus::TextEncoder;
    let encoder = TextEncoder::new();
    let metric_families = METRICS_REGISTRY.gather();
    let mut buffer = String::new();
    encoder.encode_utf8(&metric_families, &mut buffer).unwrap();
    buffer
}
