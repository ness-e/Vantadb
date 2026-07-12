//! Metric snapshot types for runtime introspection.

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

/// Point-in-time snapshot of all operational metrics counters and latencies.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OperationalMetricsSnapshot {
    /// Engine startup duration in milliseconds.
    pub startup_ms: u64,
    /// WAL replay duration in milliseconds.
    pub wal_replay_ms: u64,
    /// Number of records replayed from the WAL during startup.
    pub wal_records_replayed: u64,
    /// ANN index rebuild duration in milliseconds.
    pub ann_rebuild_ms: u64,
    /// Number of nodes scanned during the last ANN rebuild.
    pub ann_rebuild_scanned_nodes: u64,
    /// Derived (namespace/payload) index rebuild duration in milliseconds.
    pub derived_rebuild_ms: u64,
    /// Text index rebuild duration in milliseconds.
    pub text_index_rebuild_ms: u64,
    /// Total text index postings written.
    pub text_postings_written: u64,
    /// Total text index repairs triggered.
    pub text_index_repairs: u64,
    /// Total BM25 lexical queries executed.
    pub text_lexical_queries: u64,
    /// Cumulative time spent on BM25 lexical queries in milliseconds.
    pub text_lexical_query_ms: u64,
    /// Total BM25 candidates scored across all queries.
    pub text_candidates_scored: u64,
    /// Total text index consistency audits performed.
    pub text_consistency_audits: u64,
    /// Total text index consistency audits that detected drift.
    pub text_consistency_audit_failures: u64,
    /// Cumulative time spent on hybrid queries in milliseconds.
    pub hybrid_query_ms: u64,
    /// Total unique candidates fused across all hybrid queries.
    pub hybrid_candidates_fused: u64,
    /// Total queries planned as hybrid (text+vector).
    pub planner_hybrid_queries: u64,
    /// Total queries planned as text-only.
    pub planner_text_only_queries: u64,
    /// Total queries planned as vector-only.
    pub planner_vector_only_queries: u64,
    /// Total records exported.
    pub records_exported: u64,
    /// Total records imported.
    pub records_imported: u64,
    /// Total import errors encountered.
    pub import_errors: u64,
    /// Total derived index prefix scans performed.
    pub derived_prefix_scans: u64,
    /// Total fallbacks to full scan when derived index was absent.
    pub derived_full_scan_fallbacks: u64,
    /// Total nodes evicted from hot cache to cold tier.
    pub evictions_total: u64,
    /// Total nodes scanned across all eviction cycles.
    pub eviction_scanned_total: u64,
    /// Total eviction cycles executed.
    pub eviction_cycles_total: u64,
    /// Total bytes freed by eviction.
    pub eviction_bytes_total: u64,
    /// Total nodes quantized from f32 → SQ8.
    pub quantized_nodes_total: u64,
    /// Total nodes promoted from SQ8 → f32.
    pub promoted_nodes_total: u64,
    /// Current number of SQ8-quantized nodes.
    pub current_quantized_nodes: u64,
    /// Per-subsystem memory breakdown at snapshot time.
    pub memory: MemoryBreakdownSnapshot,
}
