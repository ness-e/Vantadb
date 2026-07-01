use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use crate::node::DistanceMetric;

/// Stable runtime profile exposed to SDKs without leaking hardware internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VantaRuntimeProfile {
    Enterprise,
    Performance,
    LowResource,
}

/// Stable storage tier view for external SDKs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VantaStorageTier {
    Hot,
    Cold,
}

/// Stable field value representation for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VantaValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    DateTime(chrono::DateTime<chrono::Utc>),
    ListString(Vec<String>),
    ListInt(Vec<i64>),
    ListFloat(Vec<f64>),
    ListBool(Vec<bool>),
    ListDateTime(Vec<chrono::DateTime<chrono::Utc>>),
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
    pub namespace: String,
    pub key: String,
    pub payload: String,
    pub metadata: VantaMemoryMetadata,
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
    pub namespace: String,
    pub key: String,
    pub payload: String,
    pub metadata: VantaMemoryMetadata,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub version: u64,
    pub node_id: u64,
    pub vector: Option<Vec<f32>>,
    /// Absolute Unix-ms timestamp after which the record is considered
    /// expired.  ``None`` means the record never expires.
    pub expires_at_ms: Option<u64>,
}

/// Stable list options for namespace-scoped memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemoryListOptions {
    pub filters: VantaMemoryMetadata,
    pub limit: usize,
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
    pub records: Vec<VantaMemoryRecord>,
    pub next_cursor: Option<usize>,
}

/// Stable vector search request for persistent memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemorySearchRequest {
    pub namespace: String,
    pub query_vector: Vec<f32>,
    pub filters: VantaMemoryMetadata,
    pub text_query: Option<String>,
    pub top_k: usize,
    /// Distance metric for vector similarity. Defaults to Cosine.
    pub distance_metric: DistanceMetric,
    /// When true, each result will carry a `VantaSearchExplanation`.
    pub explain: bool,
}

impl Default for VantaMemorySearchRequest {
    fn default() -> Self {
        Self {
            namespace: String::new(),
            query_vector: Vec::new(),
            filters: Default::default(),
            text_query: None,
            top_k: 10,
            distance_metric: DistanceMetric::Cosine,
            explain: false,
        }
    }
}

/// Stable vector search hit for persistent memory records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaMemorySearchHit {
    pub record: VantaMemoryRecord,
    pub score: f32,
    pub explanation: Option<VantaSearchExplanationHit>,
}

/// Stable report returned by manual ANN rebuild through the SDK boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaIndexRebuildReport {
    pub scanned_nodes: u64,
    pub indexed_vectors: u64,
    pub skipped_tombstones: u64,
    pub duration_ms: u64,
    pub derived_rebuild_ms: u64,
    pub index_path: String,
    pub success: bool,
}

/// Stable report returned by JSONL memory export operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaExportReport {
    pub records_exported: u64,
    pub namespaces: Vec<String>,
    pub path: String,
    pub duration_ms: u64,
}

/// Stable report returned by JSONL memory import operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaImportReport {
    pub inserted: u64,
    pub updated: u64,
    pub skipped: u64,
    pub errors: u64,
    pub duration_ms: u64,
}

/// Stable report returned by text index repair operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaTextIndexRepairReport {
    pub record_count: u64,
    pub posting_entries: u64,
    pub doc_stats_entries: u64,
    pub term_stats_entries: u64,
    pub namespace_stats_entries: u64,
    pub duration_ms: u64,
    pub success: bool,
}

/// Stable snapshot of operational metrics used for validation and diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaOperationalMetrics {
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
    // Per-subsystem memory breakdown
    pub process_rss_bytes: u64,
    pub process_virtual_bytes: u64,
    pub hnsw_nodes_count: u64,
    pub hnsw_logical_bytes: u64,
    pub mmap_resident_bytes: Option<u64>,
    pub volatile_cache_entries: u64,
    pub volatile_cache_cap_bytes: u64,
    pub jemalloc_allocated_bytes: Option<u64>,
    pub jemalloc_active_bytes: Option<u64>,
    pub jemalloc_metadata_bytes: Option<u64>,
    pub jemalloc_resident_bytes: Option<u64>,
    pub jemalloc_mapped_bytes: Option<u64>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaHybridFusionReport {
    pub text_candidates: usize,
    pub vector_candidates: usize,
    pub fused_candidates: usize,
    pub rrf_k: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaSearchExplanation {
    pub route: String,
    pub hits: Vec<VantaSearchExplanationHit>,
    pub fusion_report: Option<VantaHybridFusionReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaSearchExplanationHit {
    pub identity: String,
    pub score: f32,
    pub snippet: Option<String>,
    pub matched_tokens: Vec<String>,
    pub matched_phrases: Vec<String>,
    pub bm25_terms: Vec<VantaBm25TermContribution>,
    pub rrf_text_rank: Option<usize>,
    pub rrf_vector_rank: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaBm25TermContribution {
    pub token: String,
    pub tf: u32,
    pub df: u64,
    pub doc_len: u32,
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
    pub schema_version: u32,
    pub tokenizer: String,
    pub tokenizer_version: u32,
    pub key_format: String,
    pub namespace_filter: Option<String>,
    pub namespaces_audited: Vec<String>,
    pub records_scanned: u64,
    pub expected_entries: u64,
    pub actual_entries: u64,
    pub missing_entries: u64,
    pub unexpected_entries: u64,
    pub value_mismatches: u64,
    pub unreadable_entries: u64,
    pub mismatches: u64,
    pub deep_audit: bool,
    pub position_errors: u64,
    pub tf_errors: u64,
    pub df_errors: u64,
    pub doc_len_errors: u64,
    pub logical_corruptions: u64,
    pub state_valid: bool,
    pub state_status: String,
    pub duration_ms: u64,
    pub passed: bool,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VantaMemoryExportLine {
    pub schema_version: u32,
    pub namespace: String,
    pub key: String,
    pub payload: String,
    pub metadata: VantaMemoryMetadata,
    pub vector: Option<Vec<f32>>,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub version: u64,
    pub expires_at_ms: Option<u64>,
}

/// Stable graph edge representation for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaEdgeRecord {
    pub target: u64,
    pub label: String,
    pub weight: f32,
}

/// Stable node payload accepted by external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaNodeInput {
    pub id: u64,
    pub content: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub fields: VantaFields,
}

impl VantaNodeInput {
    /// Create a new node input with the given id.
    /// Content, vector, and fields default to empty/None.
    pub fn new(id: u64) -> Self {
        Self {
            id,
            content: None,
            vector: None,
            fields: VantaFields::new(),
        }
    }
}

/// Stable node view returned to external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaNodeRecord {
    pub id: u64,
    pub fields: VantaFields,
    pub vector: Option<Vec<f32>>,
    pub vector_dimensions: usize,
    pub edges: Vec<VantaEdgeRecord>,
    pub confidence_score: f32,
    pub importance: f32,
    pub hits: u32,
    pub last_accessed: u64,
    pub epoch: u32,
    pub tier: VantaStorageTier,
    pub is_alive: bool,
}

/// Stable vector search hit for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VantaSearchHit {
    pub node_id: u64,
    pub distance: f32,
}

/// Stable query result enum for external SDKs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VantaQueryResult {
    Read(Vec<VantaNodeRecord>),
    Write {
        affected_nodes: usize,
        message: String,
        node_id: Option<u64>,
    },
    StaleContext {
        node_id: u64,
    },
}

/// Stable capabilities summary exposed to external SDKs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VantaCapabilities {
    pub runtime_profile: VantaRuntimeProfile,
    pub persistence: bool,
    pub vector_search: bool,
    pub iql_queries: bool,
    pub read_only: bool,
}
