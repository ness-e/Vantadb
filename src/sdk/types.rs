//! Stable public types for the VantaDB SDK boundary.
//! All types in this module are serializable and designed for third-party bindings.

use crate::node::SparseVector;
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
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
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

/// Operadores de comparaci├│n para filtros de metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VantaFilterOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
}

/// Un filtro individual: campo + operador + valor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemoryFilterItem {
    pub field: String,
    pub op: VantaFilterOp,
    pub value: VantaValue,
}

/// Lista de filtros combinados con AND l├│gico.
pub type VantaMemoryFilter = Vec<VantaMemoryFilterItem>;

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
    /// Optional sparse term-weight vector (e.g. raw-keyword weights). Sparse
    /// vectors participate in sparse-dot search alongside the dense vector.
    #[serde(default)]
    pub sparse_vector: Option<SparseVector>,
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
            sparse_vector: None,
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
    /// Optional sparse term-weight vector persisted alongside the dense vector.
    #[serde(default)]
    pub sparse_vector: Option<SparseVector>,
    /// Absolute Unix-ms timestamp after which the record is considered
    /// expired.  ``None`` means the record never expires.
    pub expires_at_ms: Option<u64>,
}

/// Stable list options for namespace-scoped memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemoryListOptions {
    /// Metadata key-value filters to narrow results (legacy).
    #[deprecated(note = "Use filter_ops instead")]
    #[serde(default)]
    pub filters: VantaMemoryMetadata,

    /// Advanced metadata filters with operators.
    #[serde(default)]
    pub filter_ops: Option<VantaMemoryFilter>,

    /// Maximum number of records to return.
    pub limit: usize,
    /// Zero-based cursor for pagination. `None` starts from the beginning.
    pub cursor: Option<usize>,
}

impl Default for VantaMemoryListOptions {
    fn default() -> Self {
        Self {
            #[allow(deprecated)]
            filters: VantaMemoryMetadata::new(),
            filter_ops: None,
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

/// Persisted state marker for the derived sparse-vector inverted index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct SparseIndexState {
    pub(crate) schema_version: u32,
    pub(crate) rebuilt_at_ms: u64,
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SparseIndexRebuildReport {
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
    pub(crate) duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct SparseIndexCounts {
    pub(crate) record_count: u64,
    pub(crate) posting_entries: u64,
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
    /// Optional sparse vector.
    #[serde(default)]
    pub sparse_vector: Option<SparseVector>,
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
        #[serde(with = "u128_serde")]
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

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ΓöÇΓöÇ VantaRuntimeProfile ΓöÇΓöÇ

    #[test]
    fn test_runtime_profile_variants() {
        assert_ne!(
            VantaRuntimeProfile::Enterprise,
            VantaRuntimeProfile::Performance
        );
        assert_ne!(
            VantaRuntimeProfile::LowResource,
            VantaRuntimeProfile::Enterprise
        );
    }

    #[test]
    fn test_runtime_profile_clone_copy() {
        let p = VantaRuntimeProfile::Performance;
        let copied = p;
        assert_eq!(p, copied);
    }

    #[test]
    fn test_runtime_profile_debug() {
        let d = format!("{:?}", VantaRuntimeProfile::LowResource);
        assert_eq!(d, "LowResource");
    }

    // ΓöÇΓöÇ VantaStorageTier ΓöÇΓöÇ

    #[test]
    fn test_storage_tier_variants() {
        assert_ne!(VantaStorageTier::Hot, VantaStorageTier::Cold);
    }

    #[test]
    fn test_storage_tier_debug() {
        let h = format!("{:?}", VantaStorageTier::Hot);
        assert_eq!(h, "Hot");
    }

    // ΓöÇΓöÇ VantaValue ΓöÇΓöÇ

    #[test]
    fn test_vanta_value_string() {
        let v = VantaValue::String("hello".into());
        assert_eq!(
            v.to_index_values(),
            vec![VantaValue::String("hello".into())]
        );
    }

    #[test]
    fn test_vanta_value_int() {
        let v = VantaValue::Int(42);
        assert_eq!(v.to_index_values(), vec![VantaValue::Int(42)]);
    }

    #[test]
    fn test_vanta_value_float() {
        let v = VantaValue::Float(42.5);
        assert_eq!(v.to_index_values(), vec![VantaValue::Float(42.5)]);
    }

    #[test]
    fn test_vanta_value_bool() {
        let v = VantaValue::Bool(true);
        assert_eq!(v.to_index_values(), vec![VantaValue::Bool(true)]);
    }

    #[test]
    fn test_vanta_value_null() {
        let v = VantaValue::Null;
        assert_eq!(v.to_index_values(), vec![VantaValue::Null]);
    }

    #[test]
    fn test_vanta_value_datetime() {
        let dt: chrono::DateTime<chrono::Utc> = "2025-01-01T00:00:00Z".parse().unwrap();
        let v = VantaValue::DateTime(dt);
        let values = v.to_index_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], VantaValue::DateTime(dt));
    }

    #[test]
    fn test_vanta_value_to_index_list_string() {
        let v = VantaValue::ListString(vec!["a".into(), "b".into(), "c".into()]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], VantaValue::String("a".into()));
        assert_eq!(values[2], VantaValue::String("c".into()));
    }

    #[test]
    fn test_vanta_value_to_index_list_int() {
        let v = VantaValue::ListInt(vec![1, 2, 3]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 3);
        assert_eq!(values[1], VantaValue::Int(2));
    }

    #[test]
    fn test_vanta_value_to_index_list_float() {
        let v = VantaValue::ListFloat(vec![1.0, 2.0]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_vanta_value_to_index_list_bool() {
        let v = VantaValue::ListBool(vec![true, false, true]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 3);
    }

    #[test]
    fn test_vanta_value_to_index_list_datetime() {
        let dt: chrono::DateTime<chrono::Utc> = "2025-06-15T12:00:00Z".parse().unwrap();
        let v = VantaValue::ListDateTime(vec![dt]);
        let values = v.to_index_values();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], VantaValue::DateTime(dt));
    }

    #[test]
    fn test_vanta_value_to_index_empty_list() {
        let v = VantaValue::ListString(vec![]);
        let values = v.to_index_values();
        assert!(values.is_empty());
    }

    #[test]
    fn test_vanta_value_clone() {
        let v = VantaValue::String("test".into());
        let cloned = v.clone();
        assert_eq!(v, cloned);
    }

    #[test]
    fn test_vanta_value_debug() {
        let d = format!("{:?}", VantaValue::Bool(false));
        assert!(d.contains("Bool") || d.contains("false"));
    }

    // ΓöÇΓöÇ VantaMemoryInput ΓöÇΓöÇ

    #[test]
    fn test_memory_input_new() {
        let input = VantaMemoryInput::new("ns1", "key1", "payload text");
        assert_eq!(input.namespace, "ns1");
        assert_eq!(input.key, "key1");
        assert_eq!(input.payload, "payload text");
        assert!(input.metadata.is_empty());
        assert!(input.vector.is_none());
        assert!(input.ttl_ms.is_none());
    }

    #[test]
    fn test_memory_input_clone() {
        let input = VantaMemoryInput::new("ns", "k", "p");
        let cloned = input.clone();
        assert_eq!(input, cloned);
    }

    // ΓöÇΓöÇ VantaMemoryListOptions ΓöÇΓöÇ

    #[test]
    fn test_memory_list_options_default() {
        let opts = VantaMemoryListOptions::default();
        #[allow(deprecated)]
        let _ = opts.filters.is_empty();
        assert!(opts.filter_ops.is_none());
        assert_eq!(opts.limit, 100);
        assert!(opts.cursor.is_none());
    }

    // ΓöÇΓöÇ VantaMemoryListPage ΓöÇΓöÇ

    #[test]
    fn test_memory_list_page_empty() {
        let page = VantaMemoryListPage {
            records: vec![],
            next_cursor: None,
        };
        assert!(page.records.is_empty());
        assert!(page.next_cursor.is_none());
    }

    // ΓöÇΓöÇ VantaCapabilities ΓöÇΓöÇ

    #[test]
    fn test_capabilities_default() {
        let caps = VantaCapabilities {
            runtime_profile: VantaRuntimeProfile::Performance,
            persistence: true,
            vector_search: true,
            iql_queries: false,
            read_only: false,
        };
        assert_eq!(caps.runtime_profile, VantaRuntimeProfile::Performance);
        assert!(caps.persistence);
        assert!(caps.vector_search);
        assert!(!caps.iql_queries);
        assert!(!caps.read_only);
    }

    // ΓöÇΓöÇ Reports ΓöÇΓöÇ

    #[test]
    fn test_index_rebuild_report() {
        let r = VantaIndexRebuildReport {
            scanned_nodes: 1000,
            indexed_vectors: 900,
            skipped_tombstones: 50,
            duration_ms: 500,
            derived_rebuild_ms: 100,
            index_path: "/tmp/index".into(),
            success: true,
        };
        assert_eq!(r.scanned_nodes, 1000);
        assert_eq!(r.indexed_vectors, 900);
        assert!(r.success);
    }

    #[test]
    fn test_export_report() {
        let r = VantaExportReport {
            records_exported: 500,
            namespaces: vec!["ns1".into()],
            path: "/tmp/export.jsonl".into(),
            duration_ms: 250,
        };
        assert_eq!(r.records_exported, 500);
        assert_eq!(r.namespaces, vec!["ns1"]);
    }

    #[test]
    fn test_import_report() {
        let r = VantaImportReport {
            inserted: 100,
            updated: 10,
            skipped: 2,
            errors: 1,
            duration_ms: 300,
        };
        assert_eq!(r.inserted, 100);
        assert_eq!(r.updated, 10);
        assert_eq!(r.errors, 1);
    }

    #[test]
    fn test_text_index_repair_report() {
        let r = VantaTextIndexRepairReport {
            record_count: 200,
            posting_entries: 1500,
            doc_stats_entries: 200,
            term_stats_entries: 400,
            namespace_stats_entries: 5,
            duration_ms: 600,
            success: true,
        };
        assert_eq!(r.record_count, 200);
        assert!(r.success);
    }

    // ΓöÇΓöÇ VantaQueryResult ΓöÇΓöÇ

    #[test]
    fn test_query_result_read() {
        let result = VantaQueryResult::Read(vec![]);
        match result {
            VantaQueryResult::Read(nodes) => assert!(nodes.is_empty()),
            _ => panic!("expected Read"),
        }
    }

    #[test]
    fn test_query_result_write() {
        let result = VantaQueryResult::Write {
            affected_nodes: 1,
            message: "created".into(),
            node_id: Some(42),
        };
        match result {
            VantaQueryResult::Write {
                affected_nodes,
                message,
                node_id,
            } => {
                assert_eq!(affected_nodes, 1);
                assert_eq!(message, "created");
                assert_eq!(node_id, Some(42));
            }
            _ => panic!("expected Write"),
        }
    }

    #[test]
    fn test_query_result_stale_context() {
        let result = VantaQueryResult::StaleContext { node_id: 99 };
        match result {
            VantaQueryResult::StaleContext { node_id } => {
                assert_eq!(node_id, 99);
            }
            _ => panic!("expected StaleContext"),
        }
    }

    // ΓöÇΓöÇ VantaMemoryRecord ΓöÇΓöÇ

    #[test]
    fn test_memory_record_fields() {
        let rec = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 1,
            node_id: 42,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        };
        assert_eq!(rec.namespace, "ns");
        assert_eq!(rec.node_id, 42);
        assert_eq!(rec.version, 1);
    }

    // ΓöÇΓöÇ VantaMemoryExportLine ΓöÇΓöÇ

    #[test]
    fn test_export_line() {
        let line = VantaMemoryExportLine {
            schema_version: 1,
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            created_at_ms: 1000,
            updated_at_ms: 1000,
            version: 1,
            expires_at_ms: None,
        };
        assert_eq!(line.schema_version, 1);
        assert_eq!(line.namespace, "ns");
    }

    // ΓöÇΓöÇ VantaHybridFusionReport ΓöÇΓöÇ

    #[test]
    fn test_hybrid_fusion_report() {
        let r = VantaHybridFusionReport {
            text_candidates: 50,
            vector_candidates: 30,
            fused_candidates: 70,
            rrf_k: 60,
        };
        assert_eq!(r.rrf_k, 60);
        assert_eq!(r.fused_candidates, 70);
    }

    // ΓöÇΓöÇ VantaBm25TermContribution ΓöÇΓöÇ

    #[test]
    fn test_bm25_term_contribution() {
        let c = VantaBm25TermContribution {
            token: "rust".into(),
            tf: 3,
            df: 10,
            doc_len: 100,
            contribution: 2.5,
        };
        assert_eq!(c.token, "rust");
        assert_eq!(c.tf, 3);
    }

    // ΓöÇΓöÇ VantaSearchExplanation ΓöÇΓöÇ

    #[test]
    fn test_search_explanation_empty() {
        let expl = VantaSearchExplanation {
            route: "empty".into(),
            hits: vec![],
            fusion_report: None,
        };
        assert!(expl.hits.is_empty());
        assert!(expl.fusion_report.is_none());
    }

    // ΓöÇΓöÇ VantaSearchExplanationHit ΓöÇΓöÇ

    #[test]
    fn test_search_explanation_hit() {
        let hit = VantaSearchExplanationHit {
            identity: "ns\0k".into(),
            score: 0.95,
            snippet: Some("...hello world...".into()),
            matched_tokens: vec!["hello".into()],
            matched_phrases: vec![],
            bm25_terms: vec![],
            rrf_text_rank: Some(1),
            rrf_vector_rank: Some(3),
        };
        assert_eq!(hit.identity, "ns\0k");
        assert_eq!(hit.score, 0.95);
        assert!(hit.rrf_text_rank.is_some());
    }

    // ΓöÇΓöÇ VantaTextIndexAuditReport ΓöÇΓöÇ

    #[test]
    fn test_text_index_audit_report_ok() {
        let r = VantaTextIndexAuditReport {
            schema_version: 1,
            tokenizer: "default".into(),
            tokenizer_version: 1,
            key_format: "v1".into(),
            namespace_filter: None,
            namespaces_audited: vec!["ns".into()],
            records_scanned: 100,
            expected_entries: 500,
            actual_entries: 500,
            missing_entries: 0,
            unexpected_entries: 0,
            value_mismatches: 0,
            unreadable_entries: 0,
            mismatches: 0,
            deep_audit: true,
            position_errors: 0,
            tf_errors: 0,
            df_errors: 0,
            doc_len_errors: 0,
            logical_corruptions: 0,
            state_valid: true,
            state_status: "healthy".into(),
            duration_ms: 100,
            passed: true,
            status: "ok".into(),
        };
        assert!(r.passed);
        assert_eq!(r.status, "ok");
    }

    // ΓöÇΓöÇ VantaOperationalMetrics ΓöÇΓöÇ

    #[test]
    fn test_operational_metrics_defaults() {
        let m = VantaOperationalMetrics {
            startup_ms: 100,
            wal_replay_ms: 50,
            wal_records_replayed: 200,
            ann_rebuild_ms: 300,
            ann_rebuild_scanned_nodes: 1000,
            derived_rebuild_ms: 80,
            text_index_rebuild_ms: 150,
            text_postings_written: 5000,
            text_index_repairs: 1,
            text_lexical_queries: 42,
            text_lexical_query_ms: 120,
            text_candidates_scored: 10000,
            text_consistency_audits: 3,
            text_consistency_audit_failures: 0,
            hybrid_query_ms: 200,
            hybrid_candidates_fused: 500,
            planner_hybrid_queries: 10,
            planner_text_only_queries: 5,
            planner_vector_only_queries: 8,
            records_exported: 100,
            records_imported: 50,
            import_errors: 2,
            derived_prefix_scans: 30,
            derived_full_scan_fallbacks: 1,
            process_rss_bytes: 1_000_000,
            process_virtual_bytes: 2_000_000,
            hnsw_nodes_count: 500,
            hnsw_logical_bytes: 10_000_000,
            mmap_resident_bytes: Some(500_000),
            volatile_cache_entries: 100,
            volatile_cache_cap_bytes: 1_000_000,
            jemalloc_allocated_bytes: Some(2_000_000),
            jemalloc_active_bytes: Some(1_500_000),
            jemalloc_metadata_bytes: Some(100_000),
            jemalloc_resident_bytes: Some(1_800_000),
            jemalloc_mapped_bytes: Some(3_000_000),
            jemalloc_retained_bytes: Some(500_000),
        };
        assert_eq!(m.startup_ms, 100);
        assert_eq!(m.hnsw_nodes_count, 500);
        assert_eq!(m.jemalloc_allocated_bytes, Some(2_000_000));
    }

    #[test]
    fn test_operational_metrics_clone_debug() {
        let m = VantaOperationalMetrics {
            startup_ms: 1,
            wal_replay_ms: 2,
            wal_records_replayed: 3,
            ann_rebuild_ms: 4,
            ann_rebuild_scanned_nodes: 5,
            derived_rebuild_ms: 6,
            text_index_rebuild_ms: 7,
            text_postings_written: 8,
            text_index_repairs: 9,
            text_lexical_queries: 10,
            text_lexical_query_ms: 11,
            text_candidates_scored: 12,
            text_consistency_audits: 13,
            text_consistency_audit_failures: 14,
            hybrid_query_ms: 15,
            hybrid_candidates_fused: 16,
            planner_hybrid_queries: 17,
            planner_text_only_queries: 18,
            planner_vector_only_queries: 19,
            records_exported: 20,
            records_imported: 21,
            import_errors: 22,
            derived_prefix_scans: 23,
            derived_full_scan_fallbacks: 24,
            process_rss_bytes: 25,
            process_virtual_bytes: 26,
            hnsw_nodes_count: 27,
            hnsw_logical_bytes: 28,
            mmap_resident_bytes: None,
            volatile_cache_entries: 29,
            volatile_cache_cap_bytes: 30,
            jemalloc_allocated_bytes: None,
            jemalloc_active_bytes: None,
            jemalloc_metadata_bytes: None,
            jemalloc_resident_bytes: None,
            jemalloc_mapped_bytes: None,
            jemalloc_retained_bytes: None,
        };
        let cloned = m.clone();
        assert_eq!(m, cloned);
        let dbg = format!("{:?}", m);
        assert!(dbg.contains("startup_ms"));
    }

    // ΓöÇΓöÇ VantaMemoryInput with vector and ttl ΓöÇΓöÇ

    #[test]
    fn test_memory_input_with_vector_ttl() {
        let input = VantaMemoryInput {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: [("lang".into(), VantaValue::String("en".into()))].into(),
            vector: Some(vec![0.1, 0.2, 0.3]),
            sparse_vector: None,
            ttl_ms: Some(60000),
        };
        assert_eq!(input.namespace, "ns");
        assert!(input.vector.is_some());
        assert_eq!(input.vector.as_ref().unwrap().len(), 3);
        assert_eq!(input.ttl_ms, Some(60000));
        assert_eq!(
            input.metadata.get("lang").unwrap(),
            &VantaValue::String("en".into())
        );
    }

    // ΓöÇΓöÇ VantaMemoryRecord with expiry ΓöÇΓöÇ

    #[test]
    fn test_memory_record_with_expiry() {
        let rec = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 5,
            node_id: 42,
            vector: Some(vec![0.5, 0.6]),
            sparse_vector: None,
            expires_at_ms: Some(99999),
        };
        assert_eq!(rec.version, 5);
        assert_eq!(rec.expires_at_ms, Some(99999));
        assert_eq!(rec.vector.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_memory_record_clone() {
        let rec = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 1,
            node_id: 42,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        };
        let cloned = rec.clone();
        assert_eq!(rec, cloned);
    }

    // ΓöÇΓöÇ VantaCapabilities clone/debug ΓöÇΓöÇ

    #[test]
    fn test_capabilities_clone() {
        let caps = VantaCapabilities {
            runtime_profile: VantaRuntimeProfile::Enterprise,
            persistence: true,
            vector_search: false,
            iql_queries: true,
            read_only: false,
        };
        let cloned = caps.clone();
        assert_eq!(caps, cloned);
    }

    #[test]
    fn test_capabilities_debug() {
        let caps = VantaCapabilities {
            runtime_profile: VantaRuntimeProfile::Performance,
            persistence: false,
            vector_search: true,
            iql_queries: false,
            read_only: true,
        };
        let dbg = format!("{:?}", caps);
        assert!(dbg.contains("Performance"));
        assert!(dbg.contains("read_only"));
    }

    // ΓöÇΓöÇ VantaMemoryListOptions custom ΓöÇΓöÇ

    #[test]
    fn test_memory_list_options_custom() {
        let opts = VantaMemoryListOptions {
            #[allow(deprecated)]
            filters: [("type".into(), VantaValue::String("doc".into()))].into(),
            filter_ops: None,
            limit: 50,
            cursor: Some(10),
        };
        assert_eq!(opts.limit, 50);
        assert_eq!(opts.cursor, Some(10));
        #[allow(deprecated)]
        let _ = opts.filters.get("type").unwrap() == &VantaValue::String("doc".into());
    }

    // ΓöÇΓöÇ VantaQueryResult clone/debug ΓöÇΓöÇ

    #[test]
    fn test_query_result_clone_read() {
        let r = VantaQueryResult::Read(vec![]);
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    #[test]
    fn test_query_result_clone_write() {
        let r = VantaQueryResult::Write {
            affected_nodes: 3,
            message: "done".into(),
            node_id: None,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    #[test]
    fn test_query_result_debug() {
        let r = VantaQueryResult::Write {
            affected_nodes: 1,
            message: "ok".into(),
            node_id: Some(7),
        };
        let dbg = format!("{:?}", r);
        assert!(dbg.contains("Write") || dbg.contains("affected_nodes"));
    }

    // ΓöÇΓöÇ VantaHybridFusionReport clone/debug ΓöÇΓöÇ

    #[test]
    fn test_hybrid_fusion_report_clone_debug() {
        let r = VantaHybridFusionReport {
            text_candidates: 10,
            vector_candidates: 20,
            fused_candidates: 25,
            rrf_k: 60,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
        let dbg = format!("{:?}", r);
        assert!(dbg.contains("rrf_k"));
    }

    // ΓöÇΓöÇ VantaBm25TermContribution clone ΓöÇΓöÇ

    #[test]
    fn test_bm25_term_contribution_clone() {
        let c = VantaBm25TermContribution {
            token: "test".into(),
            tf: 2,
            df: 5,
            doc_len: 50,
            contribution: 1.5,
        };
        let cloned = c.clone();
        assert_eq!(c, cloned);
    }

    // ΓöÇΓöÇ VantaSearchExplanationHit clone ΓöÇΓöÇ

    #[test]
    fn test_search_explanation_hit_clone() {
        let hit = VantaSearchExplanationHit {
            identity: "ns\0k".into(),
            score: 0.9,
            snippet: None,
            matched_tokens: vec!["hi".into()],
            matched_phrases: vec![],
            bm25_terms: vec![],
            rrf_text_rank: None,
            rrf_vector_rank: None,
        };
        let cloned = hit.clone();
        assert_eq!(hit, cloned);
    }

    // ΓöÇΓöÇ VantaSearchExplanation with fusion ΓöÇΓöÇ

    #[test]
    fn test_search_explanation_with_fusion() {
        let expl = VantaSearchExplanation {
            route: "hybrid".into(),
            hits: vec![],
            fusion_report: Some(VantaHybridFusionReport {
                text_candidates: 10,
                vector_candidates: 5,
                fused_candidates: 12,
                rrf_k: 60,
            }),
        };
        assert_eq!(expl.route, "hybrid");
        assert!(expl.fusion_report.is_some());
        assert_eq!(expl.fusion_report.unwrap().fused_candidates, 12);
    }

    // ΓöÇΓöÇ VantaExportReport clone ΓöÇΓöÇ

    #[test]
    fn test_export_report_clone() {
        let r = VantaExportReport {
            records_exported: 100,
            namespaces: vec!["ns1".into()],
            path: "/tmp/x.jsonl".into(),
            duration_ms: 50,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    // ΓöÇΓöÇ VantaImportReport clone ΓöÇΓöÇ

    #[test]
    fn test_import_report_clone() {
        let r = VantaImportReport {
            inserted: 10,
            updated: 5,
            skipped: 1,
            errors: 0,
            duration_ms: 100,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    // ΓöÇΓöÇ VantaIndexRebuildReport clone ΓöÇΓöÇ

    #[test]
    fn test_index_rebuild_report_clone() {
        let r = VantaIndexRebuildReport {
            scanned_nodes: 100,
            indexed_vectors: 90,
            skipped_tombstones: 5,
            duration_ms: 200,
            derived_rebuild_ms: 50,
            index_path: "/tmp/idx".into(),
            success: true,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    // ΓöÇΓöÇ VantaTextIndexRepairReport clone ΓöÇΓöÇ

    #[test]
    fn test_text_index_repair_report_clone() {
        let r = VantaTextIndexRepairReport {
            record_count: 50,
            posting_entries: 200,
            doc_stats_entries: 50,
            term_stats_entries: 100,
            namespace_stats_entries: 3,
            duration_ms: 150,
            success: true,
        };
        let cloned = r.clone();
        assert_eq!(r, cloned);
    }

    // ΓöÇΓöÇ VantaMemoryExportLine all fields ΓöÇΓöÇ

    #[test]
    fn test_export_line_full() {
        let line = VantaMemoryExportLine {
            schema_version: 2,
            namespace: "ns".into(),
            key: "k".into(),
            payload: "text".into(),
            metadata: [("score".into(), VantaValue::Float(9.5))].into(),
            vector: Some(vec![0.1, 0.2]),
            sparse_vector: None,
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 3,
            expires_at_ms: Some(99999),
        };
        assert_eq!(line.schema_version, 2);
        assert_eq!(line.version, 3);
        assert!(line.vector.is_some());
        assert!(line.expires_at_ms.is_some());
    }

    // ΓöÇΓöÇ VantaValue Debug variant coverage ΓöÇΓöÇ

    #[test]
    fn test_vanta_value_debug_variants() {
        assert!(format!("{:?}", VantaValue::String("a".into())).contains("String"));
        assert!(format!("{:?}", VantaValue::Int(1)).contains("Int"));
        assert!(format!("{:?}", VantaValue::Float(1.0)).contains("Float"));
        assert!(format!("{:?}", VantaValue::Null).contains("Null"));
        assert!(format!("{:?}", VantaValue::ListString(vec!["a".into()])).contains("List"));
    }

    // ΓöÇΓöÇ VantaTextIndexAuditReport failure ΓöÇΓöÇ

    #[test]
    fn test_text_index_audit_report_failure() {
        let r = VantaTextIndexAuditReport {
            schema_version: 1,
            tokenizer: "default".into(),
            tokenizer_version: 1,
            key_format: "v1".into(),
            namespace_filter: Some("ns".into()),
            namespaces_audited: vec!["ns".into()],
            records_scanned: 50,
            expected_entries: 300,
            actual_entries: 280,
            missing_entries: 20,
            unexpected_entries: 5,
            value_mismatches: 3,
            unreadable_entries: 1,
            mismatches: 29,
            deep_audit: true,
            position_errors: 2,
            tf_errors: 1,
            df_errors: 0,
            doc_len_errors: 0,
            logical_corruptions: 0,
            state_valid: true,
            state_status: "healthy".into(),
            duration_ms: 80,
            passed: false,
            status: "repair_recommended".into(),
        };
        assert!(!r.passed);
        assert_eq!(r.missing_entries, 20);
        assert_eq!(r.position_errors, 2);
        assert_eq!(r.status, "repair_recommended");
    }

    // ΓöÇΓöÇ VantaMemoryListPage with data ΓöÇΓöÇ

    #[test]
    fn test_memory_list_page_with_data() {
        let rec = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "p".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1,
            updated_at_ms: 2,
            version: 1,
            node_id: 1,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
        };
        let page = VantaMemoryListPage {
            records: vec![rec],
            next_cursor: Some(1),
        };
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.next_cursor, Some(1));
    }
}
