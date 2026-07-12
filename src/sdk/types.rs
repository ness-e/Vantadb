//! Stable public types for the VantaDB SDK boundary.
//! All types in this module are serializable and designed for third-party bindings.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) mod u128_serde {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(val: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&val.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum U128 {
            Str(String),
            Num(u64),
        }
        match U128::deserialize(deserializer)? {
            U128::Str(s) => s.parse().map_err(serde::de::Error::custom),
            U128::Num(n) => Ok(n as u128),
        }
    }
}

/// Stable runtime profile exposed to SDKs without leaking hardware internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VantaRuntimeProfile {
    /// High-resource profile for enterprise-class hardware (AVX-512, 16+ GB RAM).
    Enterprise,
    /// Standard server profile (AVX2/NEON, 4+ GB RAM).
    Performance,
    /// Constrained profile for low-resource devices.
    LowResource,
}

/// Stable storage tier view for external SDKs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VantaStorageTier {
    /// Hot tier for frequently accessed nodes.
    Hot,
    /// Cold tier for infrequently accessed nodes.
    Cold,
}

/// Stable field value representation for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VantaValue {
    /// UTF-8 string value.
    String(String),
    /// Signed 64-bit integer.
    Int(i64),
    /// 64-bit floating point number.
    Float(f64),
    /// Boolean value.
    Bool(bool),
    /// RFC 3339 datetime with timezone.
    DateTime(chrono::DateTime<chrono::Utc>),
    /// List of UTF-8 strings.
    ListString(Vec<String>),
    /// List of signed 64-bit integers.
    ListInt(Vec<i64>),
    /// List of 64-bit floating point numbers.
    ListFloat(Vec<f64>),
    /// List of booleans.
    ListBool(Vec<bool>),
    /// List of RFC 3339 datetimes with timezone.
    ListDateTime(Vec<chrono::DateTime<chrono::Utc>>),
    /// Explicit null value.
    Null,
}

impl VantaValue {
    /// Flatten list variants into individual scalar values for index storage.
    /// Non-list variants return a single-element vector containing a clone of self.
    pub fn to_index_values(&self) -> Vec<VantaValue> {
        match self {
            VantaValue::ListString(vec) => {
                vec.iter().map(|s| VantaValue::String(s.clone())).collect()
            }
            VantaValue::ListInt(vec) => vec.iter().map(|&i| VantaValue::Int(i)).collect(),
            VantaValue::ListFloat(vec) => vec.iter().map(|&f| VantaValue::Float(f)).collect(),
            VantaValue::ListBool(vec) => vec.iter().map(|&b| VantaValue::Bool(b)).collect(),
            VantaValue::ListDateTime(vec) => {
                vec.iter().map(|&dt| VantaValue::DateTime(dt)).collect()
            }
            other => vec![other.clone()],
        }
    }
}

/// Stable relational fields map for external SDKs.
pub type VantaFields = BTreeMap<String, VantaValue>;

/// Stable metadata map for persistent memory records.
pub type VantaMemoryMetadata = VantaFields;

/// Stable persistent memory payload accepted by external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemoryInput {
    /// Namespace to scope the record under.
    pub namespace: String,
    /// Unique key within the namespace.
    pub key: String,
    /// Payload text content.
    pub payload: String,
    /// Arbitrary metadata key-value pairs.
    pub metadata: VantaMemoryMetadata,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
    /// Time-to-live in milliseconds from now.  The system computes
    /// ``expires_at_ms = now_ms() + ttl_ms`` server-side during ``put()``.
    /// ``None`` means the record never expires.
    pub ttl_ms: Option<u64>,
}

impl VantaMemoryInput {
    /// Create a new memory input with the given namespace, key, and payload.
    ///
    /// Metadata defaults to empty, vector is `None`, and TTL is `None` (no expiry).
    pub fn new(
        namespace: impl Into<String>,
        key: impl Into<String>,
        payload: impl Into<String>,
    ) -> Self {
        Self {
            namespace: namespace.into(),
            key: key.into(),
            payload: payload.into(),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            ttl_ms: None,
        }
    }
}

/// Stable persistent memory view returned to external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemoryRecord {
    /// Namespace the record belongs to.
    pub namespace: String,
    /// Unique key within the namespace.
    pub key: String,
    /// Payload text content.
    pub payload: String,
    /// Arbitrary metadata key-value pairs.
    pub metadata: VantaMemoryMetadata,
    /// Unix-ms creation timestamp.
    pub created_at_ms: u64,
    /// Unix-ms last-update timestamp.
    pub updated_at_ms: u64,
    /// Monotonic version counter.
    pub version: u64,
    /// Deterministic node id derived from namespace and key.
    #[serde(with = "u128_serde")]
    pub node_id: u128,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
    /// Absolute Unix-ms timestamp after which the record is considered
    /// expired.  ``None`` means the record never expires.
    pub expires_at_ms: Option<u64>,
}

/// Stable list options for namespace-scoped memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemoryListOptions {
    /// Metadata key-value filters to narrow results.
    pub filters: VantaMemoryMetadata,
    /// Maximum number of records to return.
    pub limit: usize,
    /// Zero-based cursor for pagination. `None` starts from the beginning.
    pub cursor: Option<usize>,
}

impl Default for VantaMemoryListOptions {
    fn default() -> Self {
        Self {
            filters: VantaMemoryMetadata::new(),
            limit: 100,
            cursor: None,
        }
    }
}

/// Stable list page returned by namespace-scoped scans.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemoryListPage {
    /// Records in the current page.
    pub records: Vec<VantaMemoryRecord>,
    /// Cursor for the next page, or `None` if this was the last page.
    pub next_cursor: Option<usize>,
}

pub use super::serialization::vector_types::{
    VantaMemorySearchHit, VantaMemorySearchRequest, VantaSearchHit,
};

/// Stable report returned by manual ANN rebuild through the SDK boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaIndexRebuildReport {
    /// Number of nodes scanned during the rebuild.
    pub scanned_nodes: u64,
    /// Number of vectors indexed into HNSW.
    pub indexed_vectors: u64,
    /// Number of tombstoned (deleted) nodes skipped.
    pub skipped_tombstones: u64,
    /// Duration of the rebuild in milliseconds.
    pub duration_ms: u64,
    /// Duration of the derived index rebuild in milliseconds.
    pub derived_rebuild_ms: u64,
    /// Filesystem path to the rebuilt index file.
    pub index_path: String,
    /// Whether the rebuild completed successfully.
    pub success: bool,
}

/// Stable report returned by JSONL memory export operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaExportReport {
    /// Number of records written to the export file.
    pub records_exported: u64,
    /// Namespaces that were included in the export.
    pub namespaces: Vec<String>,
    /// Filesystem path to the export file.
    pub path: String,
    /// Duration of the export in milliseconds.
    pub duration_ms: u64,
}

/// Stable report returned by JSONL memory import operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaImportReport {
    /// Number of new records inserted.
    pub inserted: u64,
    /// Number of existing records updated.
    pub updated: u64,
    /// Number of lines skipped (empty lines during file import).
    pub skipped: u64,
    /// Number of records that failed to import.
    pub errors: u64,
    /// Duration of the import in milliseconds.
    pub duration_ms: u64,
}

/// Stable report returned by text index repair operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaTextIndexRepairReport {
    /// Number of memory records indexed.
    pub record_count: u64,
    /// Number of posting list entries written.
    pub posting_entries: u64,
    /// Number of document stats entries written.
    pub doc_stats_entries: u64,
    /// Number of term stats entries written.
    pub term_stats_entries: u64,
    /// Number of namespace stats entries written.
    pub namespace_stats_entries: u64,
    /// Duration of the repair in milliseconds.
    pub duration_ms: u64,
    /// Whether the repair completed successfully.
    pub success: bool,
}

/// Stable snapshot of operational metrics used for validation and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaOperationalMetrics {
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
    /// Process resident set size in bytes (OS-reported).
    pub process_rss_bytes: u64,
    /// Process virtual memory in bytes (OS-reported).
    pub process_virtual_bytes: u64,
    /// Number of nodes in the HNSW index.
    pub hnsw_nodes_count: u64,
    /// Estimated logical footprint of HNSW allocations.
    pub hnsw_logical_bytes: u64,
    /// OS-reported resident bytes for mmap-backed files when available.
    pub mmap_resident_bytes: Option<u64>,
    /// Number of entries in the volatile hot-node cache.
    pub volatile_cache_entries: u64,
    /// Maximum capacity in bytes for the volatile cache.
    pub volatile_cache_cap_bytes: u64,
    /// Bytes allocated by jemalloc, if available.
    pub jemalloc_allocated_bytes: Option<u64>,
    /// Bytes in active pages allocated by jemalloc, if available.
    pub jemalloc_active_bytes: Option<u64>,
    /// Bytes dedicated to jemalloc metadata, if available.
    pub jemalloc_metadata_bytes: Option<u64>,
    /// Bytes in resident pages allocated by jemalloc, if available.
    pub jemalloc_resident_bytes: Option<u64>,
    /// Bytes mapped by jemalloc, if available.
    pub jemalloc_mapped_bytes: Option<u64>,
    /// Bytes in retained pages by jemalloc, if available.
    pub jemalloc_retained_bytes: Option<u64>,
}

#[cfg(debug_assertions)]
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc(hidden)]
pub struct VantaMemorySearchDebugReport {
    pub route: String,
    pub budget: usize,
    pub text_candidates: usize,
    pub vector_candidates: usize,
    pub fused_candidates: usize,
    pub top_identities: Vec<String>,
}

/// Counts and configuration for a hybrid (text+vector) fusion pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaHybridFusionReport {
    /// Number of candidates from the BM25 text search.
    pub text_candidates: usize,
    /// Number of candidates from the HNSW vector search.
    pub vector_candidates: usize,
    /// Number of unique candidates after RRF fusion.
    pub fused_candidates: usize,
    /// The k parameter used for reciprocal rank fusion.
    pub rrf_k: usize,
}

/// Explanation of a memory search result, including route, hits, and fusion report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaSearchExplanation {
    /// Route used for the search (hybrid, text-only, vector-only, empty).
    pub route: String,
    /// Explained search hits.
    pub hits: Vec<VantaSearchExplanationHit>,
    /// Fusion report present when the route was hybrid.
    pub fusion_report: Option<VantaHybridFusionReport>,
}

/// Per-hit explanation with score, snippet, matched tokens, and BM25 breakdown.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaSearchExplanationHit {
    /// Unique identity string (`namespace\0key`) of the matched record.
    pub identity: String,
    /// Combined relevance score for this hit.
    pub score: f32,
    /// Text snippet surrounding the matched query terms, if available.
    pub snippet: Option<String>,
    /// Query tokens that matched in this record.
    pub matched_tokens: Vec<String>,
    /// Query phrases that matched in this record.
    pub matched_phrases: Vec<String>,
    /// Per-term BM25 scoring breakdown.
    pub bm25_terms: Vec<VantaBm25TermContribution>,
    /// Rank of this hit in the text-only result set, if applicable.
    pub rrf_text_rank: Option<usize>,
    /// Rank of this hit in the vector-only result set, if applicable.
    pub rrf_vector_rank: Option<usize>,
}

/// Per-term BM25 scoring decomposition for a single search hit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaBm25TermContribution {
    /// The query term token.
    pub token: String,
    /// Term frequency in the matched document.
    pub tf: u32,
    /// Document frequency across the namespace.
    pub df: u64,
    /// Total length (in tokens) of the matched document.
    pub doc_len: u32,
    /// BM25 score contribution for this term.
    pub contribution: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DerivedIndexState {
    pub(crate) schema_version: u32,
    pub(crate) rebuilt_at_ms: u64,
    pub(crate) record_count: u64,
    pub(crate) namespace_entries: u64,
    pub(crate) payload_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DerivedIndexRebuildReport {
    pub(crate) record_count: u64,
    pub(crate) namespace_entries: u64,
    pub(crate) payload_entries: u64,
    pub(crate) duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TextIndexState {
    pub(crate) schema_version: u32,
    pub(crate) tokenizer: String,
    pub(crate) tokenizer_version: u32,
    pub(crate) key_format: String,
    pub(crate) rebuilt_at_ms: u64,
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
    pub(crate) doc_stats_entries: u64,
    pub(crate) term_stats_entries: u64,
    pub(crate) namespace_stats_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TextIndexRebuildReport {
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
    pub(crate) doc_stats_entries: u64,
    pub(crate) term_stats_entries: u64,
    pub(crate) namespace_stats_entries: u64,
    pub(crate) duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TextIndexCounts {
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
    pub(crate) doc_stats_entries: u64,
    pub(crate) term_stats_entries: u64,
    pub(crate) namespace_stats_entries: u64,
    pub(crate) unknown_entries: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct TextIndexMutationReport {
    pub(crate) postings_written: u64,
    pub(crate) doc_stats_delta: i64,
    pub(crate) term_stats_delta: i64,
    pub(crate) namespace_stats_delta: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct ExpectedTextIndexEntries {
    pub(crate) entries: BTreeMap<Vec<u8>, Vec<u8>>,
    pub(crate) counts: TextIndexCounts,
    pub(crate) records_scanned: u64,
    pub(crate) namespaces: BTreeSet<String>,
}

/// Stable structural audit report for the derived persistent text index.
///
/// The audit is read-only. It compares text-index postings and BM25/phrase
/// stats against canonical memory records and reports drift without repairing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaTextIndexAuditReport {
    /// Schema version of the text index spec.
    pub schema_version: u32,
    /// Tokenizer name used by the index.
    pub tokenizer: String,
    /// Tokenizer version used by the index.
    pub tokenizer_version: u32,
    /// Key format identifier used by the index.
    pub key_format: String,
    /// Optional namespace filter applied during the audit.
    pub namespace_filter: Option<String>,
    /// Namespaces that were audited.
    pub namespaces_audited: Vec<String>,
    /// Number of memory records scanned.
    pub records_scanned: u64,
    /// Number of entries expected from canonical records.
    pub expected_entries: u64,
    /// Number of entries actually present in the text index.
    pub actual_entries: u64,
    /// Entries that exist in canonical records but are missing from the index.
    pub missing_entries: u64,
    /// Entries present in the index but not expected from canonical records.
    pub unexpected_entries: u64,
    /// Entries whose value differs (deep audit only).
    pub value_mismatches: u64,
    /// Entries that could not be decoded.
    pub unreadable_entries: u64,
    /// Total mismatch count (sum of missing, unexpected, value, state).
    pub mismatches: u64,
    /// Whether a deep (value-level) audit was performed.
    pub deep_audit: bool,
    /// Posting position errors detected (deep audit only).
    pub position_errors: u64,
    /// Posting term-frequency errors detected (deep audit only).
    pub tf_errors: u64,
    /// Term-statistics document-frequency errors (deep audit only).
    pub df_errors: u64,
    /// Document-stats length errors (deep audit only).
    pub doc_len_errors: u64,
    /// Logical corruptions where values matched but key category mismatched.
    pub logical_corruptions: u64,
    /// Whether the persisted index state is valid and current.
    pub state_valid: bool,
    /// Human-readable status of the index state check.
    pub state_status: String,
    /// Duration of the audit in milliseconds.
    pub duration_ms: u64,
    /// Whether the audit passed (no mismatches found).
    pub passed: bool,
    /// Machine-readable status string ("ok" or "repair_recommended").
    pub status: String,
}

/// A single JSONL export line representing one memory record at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VantaMemoryExportLine {
    /// Export format schema version for forward compatibility.
    pub schema_version: u32,
    /// Namespace the record belongs to.
    pub namespace: String,
    /// Unique key within the namespace.
    pub key: String,
    /// Payload text content.
    pub payload: String,
    /// Arbitrary metadata key-value pairs.
    pub metadata: VantaMemoryMetadata,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
    /// Unix-ms creation timestamp.
    pub created_at_ms: u64,
    /// Unix-ms last-update timestamp.
    pub updated_at_ms: u64,
    /// Monotonic version counter.
    pub version: u64,
    /// Optional Unix-ms expiry deadline.
    pub expires_at_ms: Option<u64>,
}

pub use super::serialization::graph_types::{VantaEdgeRecord, VantaNodeInput, VantaNodeRecord};

/// Stable query result enum for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VantaQueryResult {
    /// Query returned a set of matching nodes.
    Read(Vec<VantaNodeRecord>),
    /// Query performed a write operation.
    Write {
        /// Number of nodes affected by the write.
        affected_nodes: usize,
        /// Human-readable result message.
        message: String,
        /// Node id returned by the write, if applicable.
        node_id: Option<u128>,
    },
    /// Query detected stale context for the given node.
    StaleContext {
        /// Node id with stale context.
        node_id: u128,
    },
}

/// Stable capabilities summary exposed to external SDKs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaCapabilities {
    /// Current runtime performance profile.
    pub runtime_profile: VantaRuntimeProfile,
    /// Whether the database persists data to disk.
    pub persistence: bool,
    /// Whether vector search via HNSW is available.
    pub vector_search: bool,
    /// Whether IQL query parsing and execution is available.
    pub iql_queries: bool,
    /// Whether the database is in read-only mode.
    pub read_only: bool,
}
