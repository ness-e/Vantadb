//! Prometheus and internal metrics collection.
//!
//! Registers counters, histograms, and gauges under the `prometheus`
//! feature flag; provides atomic gauges for use without the feature.

mod registry;
mod snapshot;

pub use registry::*;
pub use snapshot::*;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

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

// ── PERF-10: Eviction counters ───────────────────────────────

/// Total number of nodes evicted from hot cache to cold storage.
static EVICTIONS_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Total nodes scanned across all eviction cycles.
static EVICTION_SCANNED_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Total eviction cycles run.
static EVICTION_CYCLES_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Total bytes freed by eviction.
static EVICTION_BYTES_TOTAL: AtomicU64 = AtomicU64::new(0);

// ── PERF-09: Quantization counters ───────────────────────────

/// Total nodes quantized from f32 → SQ8.
pub(crate) static QUANTIZED_NODES_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Total nodes promoted from SQ8 → f32.
pub(crate) static PROMOTED_NODES_TOTAL: AtomicU64 = AtomicU64::new(0);
/// Current number of SQ8-quantized nodes.
pub(crate) static CURRENT_QUANTIZED_NODES: AtomicU64 = AtomicU64::new(0);

/// Record engine startup and WAL replay duration.
pub fn record_startup(startup_ms: u64, wal_replay_ms: u64, wal_records_replayed: u64) {
    LAST_STARTUP_MS.fetch_max(startup_ms, Ordering::Relaxed);
    LAST_WAL_REPLAY_MS.fetch_max(wal_replay_ms, Ordering::Relaxed);
    LAST_WAL_RECORDS_REPLAYED.fetch_max(wal_records_replayed, Ordering::Relaxed);
    observe_histogram!(STARTUP_LATENCY_MS, startup_ms);
    observe_histogram!(WAL_REPLAY_LATENCY_MS, wal_replay_ms);
}

/// Record an ANN index rebuild event.
pub fn record_ann_rebuild(duration_ms: u64, scanned_nodes: u64) {
    LAST_ANN_REBUILD_MS.fetch_max(duration_ms, Ordering::Relaxed);
    LAST_ANN_REBUILD_SCANNED_NODES.fetch_max(scanned_nodes, Ordering::Relaxed);
    observe_histogram!(ANN_REBUILD_LATENCY_MS, duration_ms);
}

/// Record a derived (namespace/payload) index rebuild.
pub fn record_derived_rebuild(duration_ms: u64) {
    LAST_DERIVED_REBUILD_MS.fetch_max(duration_ms, Ordering::Relaxed);
    observe_histogram!(DERIVED_REBUILD_LATENCY_MS, duration_ms);
}

/// Record a text index rebuild event.
pub fn record_text_index_rebuild(duration_ms: u64, postings_written: u64) {
    LAST_TEXT_INDEX_REBUILD_MS.fetch_max(duration_ms, Ordering::Relaxed);
    observe_histogram!(TEXT_INDEX_REBUILD_LATENCY_MS, duration_ms);
    record_text_postings_written(postings_written);
}

/// Record text index postings written to storage.
pub fn record_text_postings_written(postings_written: u64) {
    if postings_written == 0 {
        return;
    }
    TEXT_POSTINGS_WRITTEN_TOTAL.fetch_add(postings_written, Ordering::Relaxed);
    inc_counter_by!(TEXT_POSTINGS_WRITTEN, postings_written);
}

/// Record a text index repair event.
pub fn record_text_index_repair() {
    TEXT_INDEX_REPAIRS_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(TEXT_INDEX_REPAIRS);
}

/// Record a BM25 lexical query execution.
pub fn record_text_lexical_query(duration_ms: u64, candidates_scored: u64) {
    LAST_TEXT_LEXICAL_QUERY_MS.fetch_max(duration_ms, Ordering::Relaxed);
    TEXT_LEXICAL_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    TEXT_CANDIDATES_SCORED_TOTAL.fetch_add(candidates_scored, Ordering::Relaxed);
    observe_histogram!(TEXT_LEXICAL_QUERY_LATENCY_MS, duration_ms);
    inc_counter!(TEXT_LEXICAL_QUERIES);
    inc_counter_by!(TEXT_CANDIDATES_SCORED, candidates_scored);
}

/// Record a text index consistency audit result.
pub fn record_text_consistency_audit(failed: bool) {
    TEXT_CONSISTENCY_AUDITS_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(TEXT_CONSISTENCY_AUDITS);
    if failed {
        TEXT_CONSISTENCY_AUDIT_FAILURES_TOTAL.fetch_add(1, Ordering::Relaxed);
        inc_counter!(TEXT_CONSISTENCY_AUDIT_FAILURES);
    }
}

/// Record a hybrid (text+vector) query execution.
pub fn record_hybrid_query(duration_ms: u64, candidates_fused: u64) {
    LAST_HYBRID_QUERY_MS.fetch_max(duration_ms, Ordering::Relaxed);
    HYBRID_CANDIDATES_FUSED_TOTAL.fetch_add(candidates_fused, Ordering::Relaxed);
    observe_histogram!(HYBRID_QUERY_LATENCY_MS, duration_ms);
    inc_counter_by!(HYBRID_CANDIDATES_FUSED, candidates_fused);
}

/// Record a query planned as hybrid (text+vector).
pub fn record_planner_hybrid_query() {
    PLANNER_HYBRID_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(PLANNER_HYBRID_QUERIES);
}

/// Record a query planned as text-only.
pub fn record_planner_text_only_query() {
    PLANNER_TEXT_ONLY_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(PLANNER_TEXT_ONLY_QUERIES);
}

/// Record a query planned as vector-only.
pub fn record_planner_vector_only_query() {
    PLANNER_VECTOR_ONLY_QUERIES_TOTAL.fetch_add(1, Ordering::Relaxed);
    inc_counter!(PLANNER_VECTOR_ONLY_QUERIES);
}

/// Record memory record export.
pub fn record_export(records: u64) {
    RECORDS_EXPORTED_TOTAL.fetch_add(records, Ordering::Relaxed);
    inc_counter_by!(RECORDS_EXPORTED, records);
}

/// Record memory record import with error count.
pub fn record_import(records: u64, errors: u64) {
    RECORDS_IMPORTED_TOTAL.fetch_add(records, Ordering::Relaxed);
    IMPORT_ERRORS_TOTAL.fetch_add(errors, Ordering::Relaxed);
    inc_counter_by!(RECORDS_IMPORTED, records);
    inc_counter_by!(IMPORT_ERRORS, errors);
}

/// Record a derived index prefix scan.
pub fn record_derived_prefix_scan() {
    DERIVED_PREFIX_SCANS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

/// Record a fallback full scan when derived index is absent.
pub fn record_derived_full_scan_fallback() {
    DERIVED_FULL_SCAN_FALLBACKS_TOTAL.fetch_add(1, Ordering::Relaxed);
}

// ── PERF-10: Eviction recording ──────────────────────────────

/// Record an eviction cycle result: nodes evicted, scanned, bytes freed.
pub fn record_eviction(evicted: u64, scanned: u64, bytes_freed: u64) {
    EVICTIONS_TOTAL.fetch_add(evicted, Ordering::Relaxed);
    EVICTION_SCANNED_TOTAL.fetch_add(scanned, Ordering::Relaxed);
    EVICTION_CYCLES_TOTAL.fetch_add(1, Ordering::Relaxed);
    EVICTION_BYTES_TOTAL.fetch_add(bytes_freed, Ordering::Relaxed);
}

// ── PERF-09: Quantization recording ──────────────────────────

/// Record a quantization event (f32 → SQ8).
pub fn record_quantization() {
    QUANTIZED_NODES_TOTAL.fetch_add(1, Ordering::Relaxed);
    CURRENT_QUANTIZED_NODES.fetch_add(1, Ordering::Relaxed);
}

/// Record a promotion event (SQ8 → f32).
pub fn record_promotion() {
    PROMOTED_NODES_TOTAL.fetch_add(1, Ordering::Relaxed);
    CURRENT_QUANTIZED_NODES.fetch_sub(1, Ordering::Relaxed);
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
                        let page_size = 4096;
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
        // SAFETY: mach_task_self() always returns a valid task port for the current process.
        // task_info() writes to a zeroed POD struct we control; the pointer cast is required
        // by the Mach FFI. Failure is handled via return code check (kr == 0).
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
        // SAFETY: GetCurrentProcess() always returns the pseudo-handle -1 (no failure path).
        // GetProcessMemoryInfo() writes to a zeroed POD struct we control; the buffer size
        // matches the struct exactly. Failure is handled via non-zero return check.
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
    let allocated = tikv_jemalloc_ctl::stats::allocated::read()
        .ok()
        .unwrap_or(0) as u64;
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

/// Record per-subsystem memory breakdown snapshot.
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

/// Return a point-in-time memory breakdown snapshot.
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
        jemalloc_allocated_bytes: jemalloc_present
            .then(|| LAST_JEMALLOC_ALLOCATED_BYTES.load(Ordering::Relaxed)),
        jemalloc_active_bytes: jemalloc_present
            .then(|| LAST_JEMALLOC_ACTIVE_BYTES.load(Ordering::Relaxed)),
        jemalloc_metadata_bytes: jemalloc_present
            .then(|| LAST_JEMALLOC_METADATA_BYTES.load(Ordering::Relaxed)),
        jemalloc_resident_bytes: jemalloc_present
            .then(|| LAST_JEMALLOC_RESIDENT_BYTES.load(Ordering::Relaxed)),
        jemalloc_mapped_bytes: jemalloc_present
            .then(|| LAST_JEMALLOC_MAPPED_BYTES.load(Ordering::Relaxed)),
        jemalloc_retained_bytes: jemalloc_present
            .then(|| LAST_JEMALLOC_RETAINED_BYTES.load(Ordering::Relaxed)),
    }
}

/// Return a point-in-time snapshot of all operational metrics.
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
        evictions_total: EVICTIONS_TOTAL.load(Ordering::Relaxed),
        eviction_scanned_total: EVICTION_SCANNED_TOTAL.load(Ordering::Relaxed),
        eviction_cycles_total: EVICTION_CYCLES_TOTAL.load(Ordering::Relaxed),
        eviction_bytes_total: EVICTION_BYTES_TOTAL.load(Ordering::Relaxed),
        quantized_nodes_total: QUANTIZED_NODES_TOTAL.load(Ordering::Relaxed),
        promoted_nodes_total: PROMOTED_NODES_TOTAL.load(Ordering::Relaxed),
        current_quantized_nodes: CURRENT_QUANTIZED_NODES.load(Ordering::Relaxed),
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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use serial_test::serial;

    // ── Snapshot defaults ──────────────────────────────────────

    #[test]
    fn test_memory_breakdown_snapshot_default() {
        let snap = MemoryBreakdownSnapshot::default();
        assert_eq!(snap.process_rss_bytes, 0);
        assert_eq!(snap.hnsw_nodes_count, 0);
        assert_eq!(snap.hnsw_logical_bytes, 0);
        assert_eq!(snap.mmap_resident_bytes, None);
        assert_eq!(snap.volatile_cache_entries, 0);
        assert_eq!(snap.jemalloc_allocated_bytes, None);
    }

    #[test]
    fn test_operational_metrics_snapshot_default() {
        let snap = OperationalMetricsSnapshot::default();
        assert_eq!(snap.startup_ms, 0);
        assert_eq!(snap.wal_records_replayed, 0);
        assert_eq!(snap.text_postings_written, 0);
        assert_eq!(snap.records_exported, 0);
        assert_eq!(snap.memory, MemoryBreakdownSnapshot::default());
    }

    // ── Record + snapshot round-trips (delta-based for parallel safety) ──

    #[test]
    fn test_record_startup() {
        record_startup(100, 25, 500);
        let snap = operational_metrics_snapshot();
        assert!(snap.startup_ms >= 100);
        assert!(snap.wal_replay_ms >= 25);
        assert!(snap.wal_records_replayed >= 500);
    }

    #[test]
    fn test_record_startup_idempotent() {
        // startup_ms, wal_replay_ms, wal_records_replayed use store semantics
        record_startup(1, 2, 3);
        record_startup(10, 20, 30);
        let snap = operational_metrics_snapshot();
        assert!(snap.startup_ms >= 10);
        assert!(snap.wal_replay_ms >= 20);
        assert!(snap.wal_records_replayed >= 30);
    }

    #[test]
    fn test_record_ann_rebuild() {
        record_ann_rebuild(250, 10_000);
        let snap = operational_metrics_snapshot();
        // ann_rebuild_ms uses store semantics
        assert!(snap.ann_rebuild_ms >= 250);
        // ann_rebuild_scanned_nodes uses store semantics
        assert!(snap.ann_rebuild_scanned_nodes >= 10_000);
    }

    #[test]
    fn test_record_derived_rebuild() {
        record_derived_rebuild(75);
        let snap = operational_metrics_snapshot();
        // derived_rebuild_ms uses store semantics
        assert!(snap.derived_rebuild_ms >= 75);
    }

    #[test]
    fn test_record_text_index_rebuild() {
        record_text_index_rebuild(300, 1500);
        let snap = operational_metrics_snapshot();
        // text_index_rebuild_ms uses store semantics
        assert!(snap.text_index_rebuild_ms >= 300);
        // text_postings_written uses fetch_add
        assert!(snap.text_postings_written >= 1500);
    }

    #[test]
    fn test_record_text_postings_written_zero_guard() {
        let before = TEXT_POSTINGS_WRITTEN_TOTAL.load(Ordering::Relaxed);
        record_text_postings_written(0);
        let after = TEXT_POSTINGS_WRITTEN_TOTAL.load(Ordering::Relaxed);
        // Under parallel execution other tests can increment the counter,
        // so we can only assert it never regressed.
        assert!(
            after >= before,
            "counter regressed from {before} to {after}"
        );
    }

    #[test]
    fn test_record_text_postings_written_accumulates() {
        let before = operational_metrics_snapshot().text_postings_written;
        record_text_postings_written(100);
        record_text_postings_written(200);
        let after = operational_metrics_snapshot().text_postings_written;
        let delta = after.saturating_sub(before);
        assert!(delta >= 300, "expected delta >= 300, got {delta}");
    }

    #[test]
    fn test_record_text_index_repair() {
        let before = operational_metrics_snapshot().text_index_repairs;
        record_text_index_repair();
        let after = operational_metrics_snapshot().text_index_repairs;
        assert!(after > before);
    }

    #[test]
    fn test_record_text_lexical_query() {
        let before = operational_metrics_snapshot();
        record_text_lexical_query(42, 500);
        let snap = operational_metrics_snapshot();
        // text_lexical_query_ms uses store semantics
        assert!(snap.text_lexical_query_ms >= 42);
        // text_candidates_scored and text_lexical_queries use fetch_add
        assert!(snap.text_candidates_scored > before.text_candidates_scored + 499);
        assert!(snap.text_lexical_queries > before.text_lexical_queries);
    }

    #[test]
    fn test_record_text_lexical_query_multi() {
        let before = operational_metrics_snapshot();
        record_text_lexical_query(10, 50);
        record_text_lexical_query(20, 100);
        let snap = operational_metrics_snapshot();
        // text_lexical_queries and text_candidates_scored use fetch_add
        assert!(snap.text_lexical_queries >= before.text_lexical_queries + 2);
        assert!(snap.text_candidates_scored >= before.text_candidates_scored + 150);
        // text_lexical_query_ms uses store (last write wins)
        assert!(snap.text_lexical_query_ms >= 20);
    }

    #[test]
    fn test_record_text_consistency_audit_no_failure() {
        let before = operational_metrics_snapshot();
        record_text_consistency_audit(false);
        let snap = operational_metrics_snapshot();
        assert!(snap.text_consistency_audits > before.text_consistency_audits);
        // text_consistency_audit_failures uses fetch_add — never regresses
        assert!(
            snap.text_consistency_audit_failures >= before.text_consistency_audit_failures,
            "failures counter regressed"
        );
    }

    #[test]
    fn test_record_text_consistency_audit_with_failure() {
        let before = operational_metrics_snapshot();
        record_text_consistency_audit(true);
        let snap = operational_metrics_snapshot();
        assert!(snap.text_consistency_audits > before.text_consistency_audits);
        assert!(snap.text_consistency_audit_failures > before.text_consistency_audit_failures);
    }

    #[test]
    fn test_record_hybrid_query() {
        record_hybrid_query(150, 25);
        let snap = operational_metrics_snapshot();
        // hybrid_query_ms uses store semantics
        assert!(snap.hybrid_query_ms >= 150);
        assert!(
            snap.hybrid_candidates_fused >= 25,
            "expected hybrid_candidates_fused >= 25, got {}",
            snap.hybrid_candidates_fused
        );
    }

    #[test]
    fn test_record_hybrid_query_accumulates() {
        let before = operational_metrics_snapshot().hybrid_candidates_fused;
        record_hybrid_query(10, 5);
        record_hybrid_query(20, 3);
        let delta = operational_metrics_snapshot()
            .hybrid_candidates_fused
            .saturating_sub(before);
        assert!(delta >= 8, "expected delta >= 8, got {delta}");
    }

    #[test]
    fn test_record_planner_queries() {
        let before = operational_metrics_snapshot();
        record_planner_hybrid_query();
        record_planner_text_only_query();
        record_planner_vector_only_query();
        let snap = operational_metrics_snapshot();
        assert!(snap.planner_hybrid_queries > before.planner_hybrid_queries);
        assert!(snap.planner_text_only_queries > before.planner_text_only_queries);
        assert!(snap.planner_vector_only_queries > before.planner_vector_only_queries);
    }

    #[test]
    fn test_record_planner_accumulates() {
        let before = operational_metrics_snapshot();
        for _ in 0..5 {
            record_planner_hybrid_query();
        }
        for _ in 0..3 {
            record_planner_text_only_query();
        }
        let snap = operational_metrics_snapshot();
        assert!(snap.planner_hybrid_queries >= before.planner_hybrid_queries + 5);
        assert!(snap.planner_text_only_queries >= before.planner_text_only_queries + 3);
        // planner_vector_only_queries uses fetch_add — never regresses
        assert!(
            snap.planner_vector_only_queries >= before.planner_vector_only_queries,
            "vector-only counter regressed"
        );
    }

    // ── Export / Import ────────────────────────────────────────

    #[test]
    fn test_record_export() {
        let before = operational_metrics_snapshot().records_exported;
        record_export(100);
        let after = operational_metrics_snapshot().records_exported;
        assert!(after >= before + 100);
    }

    #[test]
    fn test_record_export_accumulates() {
        let before = operational_metrics_snapshot().records_exported;
        record_export(50);
        record_export(25);
        let after = operational_metrics_snapshot().records_exported;
        let delta = after.saturating_sub(before);
        assert!(delta >= 75, "expected delta >= 75, got {delta}");
    }

    #[test]
    fn test_record_import() {
        let before = operational_metrics_snapshot();
        record_import(200, 3);
        let after = operational_metrics_snapshot();
        assert!(after.records_imported >= before.records_imported + 200);
        assert!(after.import_errors >= before.import_errors + 3);
    }

    #[test]
    fn test_record_import_no_errors() {
        let before = operational_metrics_snapshot();
        record_import(50, 0);
        let after = operational_metrics_snapshot();
        assert!(after.records_imported >= before.records_imported + 50);
        // import_errors uses fetch_add — never regresses
        assert!(
            after.import_errors >= before.import_errors,
            "import_errors regressed"
        );
    }

    // ── Derived scans ──────────────────────────────────────────

    #[test]
    fn test_record_derived_scan() {
        let before = operational_metrics_snapshot();
        record_derived_prefix_scan();
        record_derived_full_scan_fallback();
        let snap = operational_metrics_snapshot();
        assert!(snap.derived_prefix_scans > before.derived_prefix_scans);
        assert!(snap.derived_full_scan_fallbacks > before.derived_full_scan_fallbacks);
    }

    #[test]
    fn test_record_derived_scan_accumulates() {
        let before = operational_metrics_snapshot().derived_prefix_scans;
        for _ in 0..7 {
            record_derived_prefix_scan();
        }
        let after = operational_metrics_snapshot().derived_prefix_scans;
        let delta = after.saturating_sub(before);
        assert!(delta >= 7, "expected delta >= 7, got {delta}");
    }

    // ── Memory breakdown (serialized — uses assert_eq on shared atomics) ──

    #[test]
    #[serial(memory)]
    fn test_record_memory_breakdown() {
        record_memory_breakdown(1000, 50_000_000, Some(8_000_000), 500, 100_000_000);
        let snap = memory_breakdown_snapshot();
        assert_eq!(snap.hnsw_nodes_count, 1000);
        assert_eq!(snap.hnsw_logical_bytes, 50_000_000);
        assert_eq!(snap.mmap_resident_bytes, Some(8_000_000));
        assert_eq!(snap.volatile_cache_entries, 500);
        assert_eq!(snap.volatile_cache_cap_bytes, 100_000_000);
    }

    #[test]
    #[serial(memory)]
    fn test_record_memory_breakdown_no_mmap() {
        record_memory_breakdown(0, 0, None, 0, 0);
        let snap = memory_breakdown_snapshot();
        assert_eq!(snap.mmap_resident_bytes, None);
    }

    #[test]
    #[serial(memory)]
    fn test_record_memory_breakdown_idempotent() {
        record_memory_breakdown(1, 100, Some(10), 2, 200);
        record_memory_breakdown(2, 200, None, 3, 300);
        let snap = memory_breakdown_snapshot();
        assert_eq!(snap.hnsw_nodes_count, 2);
        assert_eq!(snap.mmap_resident_bytes, None);
        assert_eq!(snap.volatile_cache_entries, 3);
    }

    // ── Snapshot isolation ─────────────────────────────────────

    #[test]
    #[serial(memory)]
    fn test_operational_snapshot_includes_memory() {
        record_memory_breakdown(42, 99_000, Some(77), 5, 10_000);
        let snap = operational_metrics_snapshot();
        assert_eq!(snap.memory.hnsw_nodes_count, 42);
        assert_eq!(snap.memory.hnsw_logical_bytes, 99_000);
        assert_eq!(snap.memory.mmap_resident_bytes, Some(77));
    }

    // ── Export ─────────────────────────────────────────────────

    #[test]
    #[cfg(feature = "prometheus")]
    #[serial(memory)]
    fn test_export_metrics_text_non_empty() {
        record_memory_breakdown(1, 100, Some(50), 10, 200);
        let text = export_metrics_text();
        assert!(
            !text.is_empty(),
            "export_metrics_text() should return non-empty prometheus output"
        );
    }
}
