use crate::backend::{BackendPartition, BackendWriteOp};
use crate::config::VantaConfig;
use crate::error::{Result, VantaError};
use crate::executor::{ExecutionResult, Executor};
// use crate::hardware::{HardwareCapabilities, HardwareProfile}; // Temporarily commented out to fix unused_imports warning in CI
use crate::index::cosine_sim_f32;
use crate::node::{DistanceMetric, FieldValue, UnifiedNode, VectorRepresentations};
use crate::storage::{IndexRebuildReport, StorageEngine};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use twox_hash::XxHash64;

const RESERVED_PREFIX: &str = "__vanta_";
const FIELD_NAMESPACE: &str = "__vanta_namespace";
const FIELD_KEY: &str = "__vanta_key";
const FIELD_PAYLOAD: &str = "__vanta_payload";
const FIELD_CREATED_AT_MS: &str = "__vanta_created_at_ms";
const FIELD_UPDATED_AT_MS: &str = "__vanta_updated_at_ms";
const FIELD_VERSION: &str = "__vanta_version";
const FIELD_EXPIRES_AT_MS: &str = "__vanta_expires_at_ms";
const EXPORT_SCHEMA_VERSION: u32 = 1;
const DERIVED_INDEX_SCHEMA_VERSION: u32 = 1;
const DERIVED_INDEX_STATE_KEY: &[u8] = b"derived_index_state";
const TEXT_INDEX_STATE_KEY: &[u8] = b"text_index_state";
// RRF and budget constants live in crate::planner — imported below as needed.

// VantaOpenOptions was removed in favor of VantaConfig.

/// Stable runtime profile exposed to SDKs without leaking hardware internals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VantaRuntimeProfile {
    Enterprise,
    Performance,
    LowResource,
}

/// Stable storage tier view for external SDKs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct VantaMemoryListPage {
    pub records: Vec<VantaMemoryRecord>,
    pub next_cursor: Option<usize>,
}

/// Stable vector search request for persistent memory records.
#[derive(Debug, Clone, PartialEq)]
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
struct DerivedIndexState {
    schema_version: u32,
    rebuilt_at_ms: u64,
    record_count: u64,
    namespace_entries: u64,
    payload_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DerivedIndexRebuildReport {
    record_count: u64,
    namespace_entries: u64,
    payload_entries: u64,
    duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct TextIndexState {
    schema_version: u32,
    tokenizer: String,
    tokenizer_version: u32,
    key_format: String,
    rebuilt_at_ms: u64,
    record_count: u64,
    posting_entries: u64,
    doc_stats_entries: u64,
    term_stats_entries: u64,
    namespace_stats_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextIndexRebuildReport {
    record_count: u64,
    posting_entries: u64,
    doc_stats_entries: u64,
    term_stats_entries: u64,
    namespace_stats_entries: u64,
    duration_ms: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextIndexCounts {
    record_count: u64,
    posting_entries: u64,
    doc_stats_entries: u64,
    term_stats_entries: u64,
    namespace_stats_entries: u64,
    unknown_entries: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct TextIndexMutationReport {
    postings_written: u64,
    doc_stats_delta: i64,
    term_stats_delta: i64,
    namespace_stats_delta: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct ExpectedTextIndexEntries {
    entries: BTreeMap<Vec<u8>, Vec<u8>>,
    counts: TextIndexCounts,
    records_scanned: u64,
    namespaces: BTreeSet<String>,
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
struct VantaMemoryExportLine {
    schema_version: u32,
    namespace: String,
    key: String,
    payload: String,
    metadata: VantaMemoryMetadata,
    vector: Option<Vec<f32>>,
    created_at_ms: u64,
    updated_at_ms: u64,
    version: u64,
    expires_at_ms: Option<u64>,
}

/// Stable graph edge representation for external SDKs.
#[derive(Debug, Clone, PartialEq)]
pub struct VantaEdgeRecord {
    pub target: u64,
    pub label: String,
    pub weight: f32,
}

/// Stable node payload accepted by external SDKs.
#[derive(Debug, Clone, PartialEq)]
pub struct VantaNodeInput {
    pub id: u64,
    pub content: Option<String>,
    pub vector: Option<Vec<f32>>,
    pub fields: VantaFields,
}

impl VantaNodeInput {
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
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
pub struct VantaSearchHit {
    pub node_id: u64,
    pub distance: f32,
}

/// Stable query result enum for external SDKs.
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VantaCapabilities {
    pub runtime_profile: VantaRuntimeProfile,
    pub persistence: bool,
    pub vector_search: bool,
    pub iql_queries: bool,
    pub read_only: bool,
}

/// Stable embedded database handle used by SDKs and bindings.
#[derive(Clone)]
pub struct VantaEmbedded {
    engine: Arc<RwLock<Option<Arc<StorageEngine>>>>,
    config: VantaConfig,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn memory_node_id(namespace: &str, key: &str) -> u64 {
    let mut hasher = XxHash64::default();
    hasher.write(namespace.as_bytes());
    hasher.write(&[0]);
    hasher.write(key.as_bytes());
    hasher.finish()
}

fn validate_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty() {
        return Err(VantaError::Execution(
            "namespace must not be empty".to_string(),
        ));
    }
    if namespace.len() > 128 {
        return Err(VantaError::Execution(
            "namespace must be at most 128 bytes".to_string(),
        ));
    }
    if !namespace
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'))
    {
        return Err(VantaError::Execution(
            "namespace may contain only A-Z, a-z, 0-9, '.', '_', '/', '-'".to_string(),
        ));
    }
    Ok(())
}

fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(VantaError::Execution("key must not be empty".to_string()));
    }
    if key.len() > 512 {
        return Err(VantaError::Execution(
            "key must be at most 512 bytes".to_string(),
        ));
    }
    if key.as_bytes().contains(&0) {
        return Err(VantaError::Execution(
            "key must not contain NUL bytes".to_string(),
        ));
    }
    Ok(())
}

fn validate_metadata(metadata: &VantaMemoryMetadata) -> Result<()> {
    if let Some(key) = metadata.keys().find(|key| key.starts_with(RESERVED_PREFIX)) {
        return Err(VantaError::Execution(format!(
            "metadata key '{}' is reserved for VantaDB internals",
            key
        )));
    }
    if let Some(key) = metadata.keys().find(|key| key.as_bytes().contains(&0)) {
        return Err(VantaError::Execution(format!(
            "metadata key '{}' must not contain NUL bytes",
            key
        )));
    }
    Ok(())
}

fn namespace_index_key(namespace: &str, key: &str) -> Vec<u8> {
    let mut index_key = Vec::with_capacity(namespace.len() + 1 + key.len());
    index_key.extend_from_slice(namespace.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(key.as_bytes());
    index_key
}

fn namespace_index_prefix(namespace: &str) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(namespace.len() + 1);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix
}

fn encoded_scalar_value(value: &VantaValue) -> Result<Vec<u8>> {
    match value {
        VantaValue::String(value) => {
            let mut encoded = b"s:".to_vec();
            encoded.extend_from_slice(value.as_bytes());
            Ok(encoded)
        }
        VantaValue::Int(value) => Ok(format!("i:{value}").into_bytes()),
        VantaValue::Float(value) => Ok(format!("f:{:016x}", value.to_bits()).into_bytes()),
        VantaValue::Bool(value) => {
            if *value {
                Ok(b"b:1".to_vec())
            } else {
                Ok(b"b:0".to_vec())
            }
        }
        VantaValue::DateTime(dt) => {
            let mut encoded = b"d:".to_vec();
            encoded.extend_from_slice(
                dt.to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
                    .as_bytes(),
            );
            Ok(encoded)
        }
        VantaValue::ListString(_)
        | VantaValue::ListInt(_)
        | VantaValue::ListFloat(_)
        | VantaValue::ListBool(_)
        | VantaValue::ListDateTime(_) => Err(VantaError::Execution(
            "Cannot encode list value as scalar index key".to_string(),
        )),
        VantaValue::Null => Ok(b"n:".to_vec()),
    }
}

fn payload_index_prefix(namespace: &str, field: &str, value: &VantaValue) -> Result<Vec<u8>> {
    let encoded = encoded_scalar_value(value)?;
    let mut prefix = Vec::with_capacity(namespace.len() + field.len() + encoded.len() + 3);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix.extend_from_slice(field.as_bytes());
    prefix.push(0);
    prefix.extend_from_slice(&encoded);
    prefix.push(0);
    Ok(prefix)
}

fn payload_index_key(namespace: &str, field: &str, value: &VantaValue, key: &str) -> Result<Vec<u8>> {
    let mut index_key = payload_index_prefix(namespace, field, value)?;
    index_key.extend_from_slice(key.as_bytes());
    Ok(index_key)
}

fn node_id_bytes(node_id: u64) -> Vec<u8> {
    node_id.to_le_bytes().to_vec()
}

fn decode_node_id(bytes: &[u8]) -> Option<u64> {
    if bytes.len() != std::mem::size_of::<u64>() {
        return None;
    }
    let mut id = [0u8; 8];
    id.copy_from_slice(bytes);
    Some(u64::from_le_bytes(id))
}

fn get_string_field(fields: &VantaFields, key: &str) -> Option<String> {
    match fields.get(key) {
        Some(VantaValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn get_u64_field(fields: &VantaFields, key: &str) -> Option<u64> {
    match fields.get(key) {
        Some(VantaValue::Int(value)) if *value >= 0 => Some(*value as u64),
        _ => None,
    }
}

fn memory_record_from_node(node: UnifiedNode) -> Option<VantaMemoryRecord> {
    if !node.is_alive() {
        return None;
    }

    let mut fields: VantaFields = node
        .relational
        .into_iter()
        .map(|(key, value)| (key, value.into()))
        .collect();

    let namespace = get_string_field(&fields, FIELD_NAMESPACE)?;
    let key = get_string_field(&fields, FIELD_KEY)?;
    let payload = get_string_field(&fields, FIELD_PAYLOAD)?;
    let created_at_ms = get_u64_field(&fields, FIELD_CREATED_AT_MS)?;
    let updated_at_ms = get_u64_field(&fields, FIELD_UPDATED_AT_MS)?;
    let version = get_u64_field(&fields, FIELD_VERSION)?;
    let expires_at_ms = get_u64_field(&fields, FIELD_EXPIRES_AT_MS);

    fields.remove(FIELD_NAMESPACE);
    fields.remove(FIELD_KEY);
    fields.remove(FIELD_PAYLOAD);
    fields.remove(FIELD_CREATED_AT_MS);
    fields.remove(FIELD_UPDATED_AT_MS);
    fields.remove(FIELD_VERSION);
    fields.remove(FIELD_EXPIRES_AT_MS);

    // Lazy TTL eviction: if expires_at_ms is set and the deadline
    // has passed, the record is treated as if it no longer exists.
    if let Some(deadline) = expires_at_ms {
        if deadline > 0 {
            let now = now_ms();
            if now > deadline {
                return None;
            }
        }
    }

    let vector = match node.vector {
        VectorRepresentations::Full(vector) => Some(vector),
        _ => None,
    };

    Some(VantaMemoryRecord {
        namespace,
        key,
        payload,
        metadata: fields,
        created_at_ms,
        updated_at_ms,
        version,
        node_id: node.id,
        vector,
        expires_at_ms,
    })
}

fn memory_record_to_node(record: &VantaMemoryRecord) -> UnifiedNode {
    let mut node = UnifiedNode::new(record.node_id);
    node.set_field(
        FIELD_NAMESPACE,
        FieldValue::String(record.namespace.clone()),
    );
    node.set_field(FIELD_KEY, FieldValue::String(record.key.clone()));
    node.set_field(FIELD_PAYLOAD, FieldValue::String(record.payload.clone()));
    node.set_field(
        FIELD_CREATED_AT_MS,
        FieldValue::Int(record.created_at_ms as i64),
    );
    node.set_field(
        FIELD_UPDATED_AT_MS,
        FieldValue::Int(record.updated_at_ms as i64),
    );
    node.set_field(FIELD_VERSION, FieldValue::Int(record.version as i64));

    if let Some(expires_at) = record.expires_at_ms {
        node.set_field(FIELD_EXPIRES_AT_MS, FieldValue::Int(expires_at as i64));
    }

    for (key, value) in record.metadata.clone() {
        node.set_field(key, value.into());
    }

    if let Some(vector) = record.vector.clone().filter(|vector| !vector.is_empty()) {
        node.vector = VectorRepresentations::Full(vector);
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
    }

    node
}

fn export_line_from_record(record: VantaMemoryRecord) -> VantaMemoryExportLine {
    VantaMemoryExportLine {
        schema_version: EXPORT_SCHEMA_VERSION,
        namespace: record.namespace,
        key: record.key,
        payload: record.payload,
        metadata: record.metadata,
        vector: record.vector,
        created_at_ms: record.created_at_ms,
        updated_at_ms: record.updated_at_ms,
        version: record.version,
        expires_at_ms: record.expires_at_ms,
    }
}

fn record_from_export_line(line: VantaMemoryExportLine) -> Result<VantaMemoryRecord> {
    if line.schema_version != EXPORT_SCHEMA_VERSION {
        return Err(VantaError::Execution(format!(
            "unsupported memory export schema_version {}",
            line.schema_version
        )));
    }

    let node_id = memory_node_id(&line.namespace, &line.key);
    Ok(VantaMemoryRecord {
        namespace: line.namespace,
        key: line.key,
        payload: line.payload,
        metadata: line.metadata,
        created_at_ms: line.created_at_ms,
        updated_at_ms: line.updated_at_ms,
        version: line.version,
        node_id,
        vector: line.vector,
        expires_at_ms: line.expires_at_ms,
    })
}

fn matches_memory_filters(record: &VantaMemoryRecord, filters: &VantaMemoryMetadata) -> bool {
    filters
        .iter()
        .all(|(key, expected)| record.metadata.get(key) == Some(expected))
}

impl VantaEmbedded {
    pub fn from_engine(engine: Arc<StorageEngine>) -> Self {
        let config = engine.config.clone();
        Self {
            engine: Arc::new(RwLock::new(Some(engine))),
            config,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let config = VantaConfig {
            storage_path: path.as_ref().to_string_lossy().into_owned(),
            ..Default::default()
        };
        Self::open_with_config(config)
    }

    pub fn open_with_config(config: VantaConfig) -> Result<Self> {
        let final_config = config.clone();

        let engine = StorageEngine::open_with_config(
            &final_config.storage_path,
            Some(final_config.clone()),
        )?;
        let embedded = Self {
            engine: Arc::new(RwLock::new(Some(Arc::new(engine)))),
            config: final_config,
        };
        if !embedded.config.read_only {
            embedded.ensure_derived_indexes_current()?;
            embedded.ensure_text_index_current()?;
        }
        Ok(embedded)
    }

    fn engine_handle(&self) -> Result<Arc<StorageEngine>> {
        self.engine.read().clone().ok_or(VantaError::NotInitialized)
    }

    fn load_derived_index_state(engine: &StorageEngine) -> Result<Option<DerivedIndexState>> {
        let Some(bytes) = engine
            .get_from_partition(BackendPartition::InternalMetadata, DERIVED_INDEX_STATE_KEY)?
        else {
            return Ok(None);
        };
        bincode::deserialize(&bytes).map(Some).map_err(|err| {
            VantaError::SerializationError(format!("derived index state decode error: {err}"))
        })
    }

    fn write_derived_index_state(engine: &StorageEngine, state: &DerivedIndexState) -> Result<()> {
        let bytes = bincode::serialize(state)
            .map_err(|err| VantaError::SerializationError(err.to_string()))?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            DERIVED_INDEX_STATE_KEY,
            &bytes,
        )
    }

    fn load_text_index_state(engine: &StorageEngine) -> Result<Option<TextIndexState>> {
        let Some(bytes) =
            engine.get_from_partition(BackendPartition::InternalMetadata, TEXT_INDEX_STATE_KEY)?
        else {
            return Ok(None);
        };
        bincode::deserialize(&bytes).map(Some).map_err(|err| {
            VantaError::SerializationError(format!("text index state decode error: {err}"))
        })
    }

    fn write_text_index_state(engine: &StorageEngine, state: &TextIndexState) -> Result<()> {
        let bytes = bincode::serialize(state)
            .map_err(|err| VantaError::SerializationError(err.to_string()))?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            TEXT_INDEX_STATE_KEY,
            &bytes,
        )
    }

    fn fresh_text_index_state(counts: TextIndexCounts) -> TextIndexState {
        let spec = crate::text_index::TextIndexSpec::default();
        TextIndexState {
            schema_version: spec.schema_version,
            tokenizer: spec.tokenizer.name.to_string(),
            tokenizer_version: spec.tokenizer.version,
            key_format: spec.key_format.to_string(),
            rebuilt_at_ms: now_ms(),
            record_count: counts.record_count,
            posting_entries: counts.posting_entries,
            doc_stats_entries: counts.doc_stats_entries,
            term_stats_entries: counts.term_stats_entries,
            namespace_stats_entries: counts.namespace_stats_entries,
        }
    }

    fn text_index_state_matches_spec(state: &TextIndexState) -> bool {
        let spec = crate::text_index::TextIndexSpec::default();
        state.schema_version == spec.schema_version
            && state.tokenizer == spec.tokenizer.name
            && state.tokenizer_version == spec.tokenizer.version
            && state.key_format == spec.key_format
    }

    fn count_memory_records(engine: &StorageEngine) -> Result<u64> {
        let mut count = 0u64;
        for node in engine.scan_nodes()? {
            if memory_record_from_node(node).is_some() {
                count += 1;
            }
        }
        Ok(count)
    }

    fn expected_text_index_counts(engine: &StorageEngine) -> Result<TextIndexCounts> {
        let mut counts = TextIndexCounts::default();
        let mut terms = BTreeSet::new();
        let mut namespaces = BTreeSet::new();

        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node(node) {
                counts.record_count += 1;
                counts.posting_entries += crate::text_index::posting_count(&record.payload);
                counts.doc_stats_entries += 1;
                namespaces.insert(record.namespace.clone());
                for token in crate::text_index::unique_tokens(&record.payload) {
                    terms.insert((record.namespace.clone(), token));
                }
            }
        }

        counts.term_stats_entries = terms.len() as u64;
        counts.namespace_stats_entries = namespaces.len() as u64;
        Ok(counts)
    }

    fn current_text_index_counts(engine: &StorageEngine) -> Result<TextIndexCounts> {
        let mut counts = TextIndexCounts::default();
        for (key, _value) in engine.scan_partition(BackendPartition::TextIndex)? {
            if !crate::text_index::is_internal_key(&key) {
                counts.posting_entries += 1;
                continue;
            }

            if crate::text_index::is_doc_stats_key(&key) {
                counts.doc_stats_entries += 1;
            } else if crate::text_index::is_term_stats_key(&key) {
                counts.term_stats_entries += 1;
            } else if crate::text_index::is_namespace_stats_key(&key) {
                counts.namespace_stats_entries += 1;
            } else {
                counts.unknown_entries += 1;
            }
        }
        Ok(counts)
    }

    fn current_derived_index_counts(engine: &StorageEngine) -> Result<(u64, u64)> {
        let namespace_entries = engine
            .scan_partition(BackendPartition::NamespaceIndex)?
            .len() as u64;
        let payload_entries = engine.scan_partition(BackendPartition::PayloadIndex)?.len() as u64;
        Ok((namespace_entries, payload_entries))
    }

    fn derived_put_ops(record: &VantaMemoryRecord) -> Result<Vec<BackendWriteOp>> {
        let mut ops = Vec::new();
        ops.push(BackendWriteOp::Put {
            partition: BackendPartition::NamespaceIndex,
            key: namespace_index_key(&record.namespace, &record.key),
            value: node_id_bytes(record.node_id),
        });

        for (field, value) in &record.metadata {
            for val in value.to_index_values() {
                ops.push(BackendWriteOp::Put {
                    partition: BackendPartition::PayloadIndex,
                    key: payload_index_key(&record.namespace, field, &val, &record.key)?,
                    value: node_id_bytes(record.node_id),
                });
            }
        }

        Ok(ops)
    }

    fn derived_delete_ops(record: &VantaMemoryRecord) -> Result<Vec<BackendWriteOp>> {
        let mut ops = Vec::new();
        ops.push(BackendWriteOp::Delete {
            partition: BackendPartition::NamespaceIndex,
            key: namespace_index_key(&record.namespace, &record.key),
        });

        for (field, value) in &record.metadata {
            for val in value.to_index_values() {
                ops.push(BackendWriteOp::Delete {
                    partition: BackendPartition::PayloadIndex,
                    key: payload_index_key(&record.namespace, field, &val, &record.key)?,
                });
            }
        }

        Ok(ops)
    }

    fn load_text_term_stats(
        engine: &StorageEngine,
        namespace: &str,
        token: &str,
    ) -> Result<Option<crate::text_index::TextTermStats>> {
        let cache_key = (namespace.to_string(), token.to_string());
        {
            let cache = engine.text_stats_cache.read();
            if let Some(stats) = cache.get(&cache_key) {
                return Ok(Some(stats.clone()));
            }
        }

        let skey = crate::text_index::term_stats_key(namespace, token);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &skey)? else {
            return Ok(None);
        };
        let stats = crate::text_index::decode_term_stats(&bytes)?;

        {
            let mut cache = engine.text_stats_cache.write();
            cache.insert(cache_key, stats.clone());
        }
        Ok(Some(stats))
    }

    fn load_text_namespace_stats(
        engine: &StorageEngine,
        namespace: &str,
    ) -> Result<Option<crate::text_index::TextNamespaceStats>> {
        {
            let cache = engine.text_ns_cache.read();
            if let Some(stats) = cache.get(namespace) {
                return Ok(Some(stats.clone()));
            }
        }

        let skey = crate::text_index::namespace_stats_key(namespace);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &skey)? else {
            return Ok(None);
        };
        let stats = crate::text_index::decode_namespace_stats(&bytes)?;

        {
            let mut cache = engine.text_ns_cache.write();
            cache.insert(namespace.to_string(), stats.clone());
        }
        Ok(Some(stats))
    }

    fn load_text_doc_stats(
        engine: &StorageEngine,
        namespace: &str,
        key: &str,
    ) -> Result<Option<crate::text_index::TextDocStats>> {
        let Some(bytes) = engine.get_from_partition(
            BackendPartition::TextIndex,
            &crate::text_index::doc_stats_key(namespace, key),
        )?
        else {
            return Ok(None);
        };
        crate::text_index::decode_doc_stats(&bytes).map(Some)
    }

    fn apply_u64_delta(value: u64, delta: i64) -> u64 {
        if delta >= 0 {
            value.saturating_add(delta as u64)
        } else {
            value.saturating_sub(delta.unsigned_abs())
        }
    }

    fn checked_stats_value(value: i128, label: &str) -> Result<u64> {
        if value < 0 {
            return Err(VantaError::Execution(format!(
                "text index {label} would become negative"
            )));
        }
        u64::try_from(value).map_err(|_| {
            VantaError::Execution(format!("text index {label} exceeds supported range"))
        })
    }

    fn text_index_ops_for_replace(
        engine: &StorageEngine,
        previous: Option<&VantaMemoryRecord>,
        current: Option<&VantaMemoryRecord>,
    ) -> Result<(Vec<BackendWriteOp>, TextIndexMutationReport)> {
        let mut ops = Vec::new();
        let mut report = TextIndexMutationReport::default();
        let mut term_deltas: BTreeMap<(String, String), i64> = BTreeMap::new();
        let mut namespace_deltas: BTreeMap<String, (i64, i64)> = BTreeMap::new();

        if let Some(previous) = previous {
            let terms = crate::text_index::record_terms(&previous.payload);
            ops.extend(crate::text_index::posting_delete_ops(
                &previous.namespace,
                &previous.key,
                &previous.payload,
            ));
            ops.push(crate::text_index::doc_stats_delete_op(
                &previous.namespace,
                &previous.key,
            ));
            report.doc_stats_delta -= 1;

            for token in terms.token_counts.keys() {
                *term_deltas
                    .entry((previous.namespace.clone(), token.clone()))
                    .or_default() -= 1;
            }
            let namespace_delta = namespace_deltas
                .entry(previous.namespace.clone())
                .or_insert((0, 0));
            namespace_delta.0 -= 1;
            namespace_delta.1 -= i64::from(terms.doc_len);
        }

        if let Some(current) = current {
            let terms = crate::text_index::record_terms(&current.payload);
            let posting_ops = crate::text_index::posting_put_ops(
                &current.namespace,
                &current.key,
                &current.payload,
                current.node_id,
            )?;
            report.postings_written = posting_ops.len() as u64;
            ops.extend(posting_ops);
            ops.push(crate::text_index::doc_stats_put_op(
                &current.namespace,
                &current.key,
                &current.payload,
                current.node_id,
            )?);
            report.doc_stats_delta += 1;

            for token in terms.token_counts.keys() {
                *term_deltas
                    .entry((current.namespace.clone(), token.clone()))
                    .or_default() += 1;
            }
            let namespace_delta = namespace_deltas
                .entry(current.namespace.clone())
                .or_insert((0, 0));
            namespace_delta.0 += 1;
            namespace_delta.1 += i64::from(terms.doc_len);
        }

        for ((namespace, token), delta) in term_deltas {
            if delta == 0 {
                continue;
            }

            let existing = Self::load_text_term_stats(engine, &namespace, &token)?
                .map(|stats| stats.df)
                .unwrap_or(0);
            let next = Self::checked_stats_value(existing as i128 + delta as i128, "df")?;
            match (existing == 0, next == 0) {
                (true, false) => report.term_stats_delta += 1,
                (false, true) => report.term_stats_delta -= 1,
                _ => {}
            }
            if next == 0 {
                ops.push(crate::text_index::term_stats_delete_op(&namespace, &token));
            } else {
                ops.push(crate::text_index::term_stats_put_op(
                    &namespace, &token, next,
                )?);
            }
        }

        for (namespace, (doc_delta, len_delta)) in namespace_deltas {
            if doc_delta == 0 && len_delta == 0 {
                continue;
            }

            let existing = Self::load_text_namespace_stats(engine, &namespace)?.unwrap_or(
                crate::text_index::TextNamespaceStats {
                    doc_count: 0,
                    total_doc_len: 0,
                },
            );
            let next_doc_count = Self::checked_stats_value(
                existing.doc_count as i128 + doc_delta as i128,
                "doc_count",
            )?;
            let next_total_doc_len = Self::checked_stats_value(
                existing.total_doc_len as i128 + len_delta as i128,
                "total_doc_len",
            )?;

            match (existing.doc_count == 0, next_doc_count == 0) {
                (true, false) => report.namespace_stats_delta += 1,
                (false, true) => report.namespace_stats_delta -= 1,
                _ => {}
            }

            if next_doc_count == 0 {
                ops.push(crate::text_index::namespace_stats_delete_op(&namespace));
            } else {
                ops.push(crate::text_index::namespace_stats_put_op(
                    &namespace,
                    &crate::text_index::TextNamespaceStats {
                        doc_count: next_doc_count,
                        total_doc_len: next_total_doc_len,
                    },
                )?);
            }
        }

        Ok((ops, report))
    }

    fn replace_derived_indexes(
        &self,
        engine: &StorageEngine,
        previous: Option<&VantaMemoryRecord>,
        current: Option<&VantaMemoryRecord>,
    ) -> Result<()> {
        let mut ops = Vec::new();
        if let Some(previous) = previous {
            ops.extend(Self::derived_delete_ops(previous)?);
        }
        if let Some(current) = current {
            ops.extend(Self::derived_put_ops(current)?);
        }
        let (text_ops, text_report) = Self::text_index_ops_for_replace(engine, previous, current)?;
        ops.extend(text_ops);
        if ops.is_empty() {
            return Ok(());
        }
        engine.write_backend_batch(ops.clone())?;

        // Actualizar/Invalidar la caché en memoria en base a las operaciones escritas
        for op in &ops {
            match op {
                BackendWriteOp::Put {
                    partition: BackendPartition::TextIndex,
                    key,
                    value,
                } => {
                    if crate::text_index::is_term_stats_key(key) {
                        if let Some((ns, token)) = Self::parse_term_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_term_stats(value) {
                                let mut cache = engine.text_stats_cache.write();
                                cache.insert((ns, token), stats);
                            }
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_namespace_stats(value) {
                                let mut cache = engine.text_ns_cache.write();
                                cache.insert(ns, stats);
                            }
                        }
                    }
                }
                BackendWriteOp::Delete {
                    partition: BackendPartition::TextIndex,
                    key,
                } => {
                    if crate::text_index::is_term_stats_key(key) {
                        if let Some((ns, token)) = Self::parse_term_stats_key(key) {
                            let mut cache = engine.text_stats_cache.write();
                            cache.remove(&(ns, token));
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            let mut cache = engine.text_ns_cache.write();
                            cache.remove(&ns);
                        }
                    }
                }
                _ => {}
            }
        }

        Self::adjust_derived_index_state_after_replace(engine, previous, current)?;
        Self::adjust_text_index_state_after_replace(engine, previous, current, text_report)?;
        crate::metrics::record_text_postings_written(text_report.postings_written);
        Ok(())
    }

    fn parse_term_stats_key(key: &[u8]) -> Option<(String, String)> {
        const INTERNAL_PREFIX: &[u8] = b"\xffvanta_text_v3\0";
        const TERM_STATS_TAG: &[u8] = b"term\0";
        let remainder = key
            .strip_prefix(INTERNAL_PREFIX)?
            .strip_prefix(TERM_STATS_TAG)?;
        let pos = remainder.iter().position(|&b| b == 0)?;
        let ns = String::from_utf8(remainder[..pos].to_vec()).ok()?;
        let token = String::from_utf8(remainder[pos + 1..].to_vec()).ok()?;
        Some((ns, token))
    }

    fn parse_namespace_stats_key(key: &[u8]) -> Option<String> {
        const INTERNAL_PREFIX: &[u8] = b"\xffvanta_text_v3\0";
        const NAMESPACE_STATS_TAG: &[u8] = b"ns\0";
        let remainder = key
            .strip_prefix(INTERNAL_PREFIX)?
            .strip_prefix(NAMESPACE_STATS_TAG)?;
        String::from_utf8(remainder.to_vec()).ok()
    }

    fn adjust_derived_index_state_after_replace(
        engine: &StorageEngine,
        previous: Option<&VantaMemoryRecord>,
        current: Option<&VantaMemoryRecord>,
    ) -> Result<()> {
        let Some(mut state) = Self::load_derived_index_state(engine)? else {
            return Ok(());
        };
        if state.schema_version != DERIVED_INDEX_SCHEMA_VERSION {
            return Ok(());
        }

        match (previous, current) {
            (None, Some(current)) => {
                state.record_count = state.record_count.saturating_add(1);
                state.namespace_entries = state.namespace_entries.saturating_add(1);
                state.payload_entries = state
                    .payload_entries
                    .saturating_add(current.metadata.len() as u64);
            }
            (Some(previous), None) => {
                state.record_count = state.record_count.saturating_sub(1);
                state.namespace_entries = state.namespace_entries.saturating_sub(1);
                state.payload_entries = state
                    .payload_entries
                    .saturating_sub(previous.metadata.len() as u64);
            }
            (Some(previous), Some(current)) => {
                state.payload_entries = state
                    .payload_entries
                    .saturating_sub(previous.metadata.len() as u64)
                    .saturating_add(current.metadata.len() as u64);
            }
            (None, None) => {}
        }

        Self::write_derived_index_state(engine, &state)
    }

    fn adjust_text_index_state_after_replace(
        engine: &StorageEngine,
        previous: Option<&VantaMemoryRecord>,
        current: Option<&VantaMemoryRecord>,
        report: TextIndexMutationReport,
    ) -> Result<()> {
        let Some(mut state) = Self::load_text_index_state(engine)? else {
            return Ok(());
        };
        if !Self::text_index_state_matches_spec(&state) {
            return Ok(());
        }

        match (previous, current) {
            (None, Some(current)) => {
                state.record_count = state.record_count.saturating_add(1);
                state.posting_entries = state
                    .posting_entries
                    .saturating_add(crate::text_index::posting_count(&current.payload));
                state.doc_stats_entries =
                    Self::apply_u64_delta(state.doc_stats_entries, report.doc_stats_delta);
            }
            (Some(previous), None) => {
                state.record_count = state.record_count.saturating_sub(1);
                state.posting_entries = state
                    .posting_entries
                    .saturating_sub(crate::text_index::posting_count(&previous.payload));
                state.doc_stats_entries =
                    Self::apply_u64_delta(state.doc_stats_entries, report.doc_stats_delta);
            }
            (Some(previous), Some(current)) => {
                state.posting_entries = state
                    .posting_entries
                    .saturating_sub(crate::text_index::posting_count(&previous.payload))
                    .saturating_add(crate::text_index::posting_count(&current.payload));
                state.doc_stats_entries =
                    Self::apply_u64_delta(state.doc_stats_entries, report.doc_stats_delta);
            }
            (None, None) => {}
        }
        state.term_stats_entries =
            Self::apply_u64_delta(state.term_stats_entries, report.term_stats_delta);
        state.namespace_stats_entries =
            Self::apply_u64_delta(state.namespace_stats_entries, report.namespace_stats_delta);

        Self::write_text_index_state(engine, &state)
    }

    fn ensure_derived_indexes_current(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        let state = match Self::load_derived_index_state(&engine) {
            Ok(state) => state,
            Err(_) => {
                self.rebuild_derived_indexes_with_report()?;
                return Ok(());
            }
        };

        let canonical_records = Self::count_memory_records(&engine)?;
        let (namespace_entries, payload_entries) = Self::current_derived_index_counts(&engine)?;

        let has_state = state.is_some();
        let needs_rebuild = match &state {
            Some(state) => {
                state.schema_version != DERIVED_INDEX_SCHEMA_VERSION
                    || state.record_count != canonical_records
                    || state.namespace_entries != namespace_entries
                    || state.payload_entries != payload_entries
                    || namespace_entries < canonical_records
            }
            None => canonical_records > 0 || namespace_entries > 0 || payload_entries > 0,
        };

        if needs_rebuild {
            self.rebuild_derived_indexes_with_report()?;
        } else if !has_state {
            Self::write_derived_index_state(
                &engine,
                &DerivedIndexState {
                    schema_version: DERIVED_INDEX_SCHEMA_VERSION,
                    rebuilt_at_ms: now_ms(),
                    record_count: canonical_records,
                    namespace_entries,
                    payload_entries,
                },
            )?;
        }

        Ok(())
    }

    fn ensure_text_index_current(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        let state = match Self::load_text_index_state(&engine) {
            Ok(state) => state,
            Err(_) => {
                crate::metrics::record_text_index_repair();
                self.rebuild_text_index_with_report()?;
                return Ok(());
            }
        };

        let expected = Self::expected_text_index_counts(&engine)?;
        let current = Self::current_text_index_counts(&engine)?;

        let has_state = state.is_some();
        let needs_rebuild = match &state {
            Some(state) => {
                !Self::text_index_state_matches_spec(state)
                    || state.record_count != expected.record_count
                    || state.posting_entries != current.posting_entries
                    || state.posting_entries != expected.posting_entries
                    || state.doc_stats_entries != current.doc_stats_entries
                    || state.doc_stats_entries != expected.doc_stats_entries
                    || state.term_stats_entries != current.term_stats_entries
                    || state.term_stats_entries != expected.term_stats_entries
                    || state.namespace_stats_entries != current.namespace_stats_entries
                    || state.namespace_stats_entries != expected.namespace_stats_entries
                    || current.posting_entries != expected.posting_entries
                    || current.doc_stats_entries != expected.doc_stats_entries
                    || current.term_stats_entries != expected.term_stats_entries
                    || current.namespace_stats_entries != expected.namespace_stats_entries
                    || current.unknown_entries != 0
            }
            None => {
                expected.record_count > 0
                    || current.posting_entries > 0
                    || current.doc_stats_entries > 0
                    || current.term_stats_entries > 0
                    || current.namespace_stats_entries > 0
                    || current.unknown_entries > 0
            }
        };

        if needs_rebuild {
            crate::metrics::record_text_index_repair();
            self.rebuild_text_index_with_report()?;
        } else if !has_state {
            Self::write_text_index_state(&engine, &Self::fresh_text_index_state(expected))?;
        }

        Ok(())
    }

    fn rebuild_derived_indexes_with_report(&self) -> Result<DerivedIndexRebuildReport> {
        let started = Instant::now();
        let engine = self.engine_handle()?;
        let mut ops = Vec::new();
        let mut record_count = 0u64;
        let mut namespace_entries = 0u64;
        let mut payload_entries = 0u64;

        for (key, _value) in engine.scan_partition(BackendPartition::NamespaceIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::NamespaceIndex,
                key,
            });
        }
        for (key, _value) in engine.scan_partition(BackendPartition::PayloadIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::PayloadIndex,
                key,
            });
        }
        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node(node) {
                record_count += 1;
                namespace_entries += 1;
                payload_entries += record.metadata.len() as u64;
                ops.extend(Self::derived_put_ops(&record)?);
            }
        }

        if !ops.is_empty() {
            engine.write_backend_batch(ops)?;
        }

        Self::write_derived_index_state(
            &engine,
            &DerivedIndexState {
                schema_version: DERIVED_INDEX_SCHEMA_VERSION,
                rebuilt_at_ms: now_ms(),
                record_count,
                namespace_entries,
                payload_entries,
            },
        )?;

        let report = DerivedIndexRebuildReport {
            record_count,
            namespace_entries,
            payload_entries,
            duration_ms: started.elapsed().as_millis() as u64,
        };
        crate::metrics::record_derived_rebuild(report.duration_ms);
        Ok(report)
    }

    fn rebuild_derived_indexes(&self) -> Result<()> {
        self.rebuild_derived_indexes_with_report().map(|_| ())
    }

    fn rebuild_text_index_with_report(&self) -> Result<TextIndexRebuildReport> {
        let started = Instant::now();
        let engine = self.engine_handle()?;

        // Limpiar la caché en memoria antes de la reconstrucción masiva
        {
            let mut cache = engine.text_stats_cache.write();
            cache.clear();
        }
        {
            let mut cache = engine.text_ns_cache.write();
            cache.clear();
        }

        let mut ops = Vec::new();
        let mut counts = TextIndexCounts::default();
        let mut term_stats: BTreeMap<(String, String), u64> = BTreeMap::new();
        let mut namespace_stats: BTreeMap<String, crate::text_index::TextNamespaceStats> =
            BTreeMap::new();

        for (key, _value) in engine.scan_partition(BackendPartition::TextIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::TextIndex,
                key,
            });
        }

        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node(node) {
                counts.record_count += 1;
                let posting_ops = crate::text_index::posting_put_ops(
                    &record.namespace,
                    &record.key,
                    &record.payload,
                    record.node_id,
                )?;
                counts.posting_entries += posting_ops.len() as u64;
                ops.extend(posting_ops);
                ops.push(crate::text_index::doc_stats_put_op(
                    &record.namespace,
                    &record.key,
                    &record.payload,
                    record.node_id,
                )?);
                counts.doc_stats_entries += 1;

                let terms = crate::text_index::record_terms(&record.payload);
                for token in terms.token_counts.keys() {
                    *term_stats
                        .entry((record.namespace.clone(), token.clone()))
                        .or_default() += 1;
                }
                let namespace = namespace_stats.entry(record.namespace.clone()).or_insert(
                    crate::text_index::TextNamespaceStats {
                        doc_count: 0,
                        total_doc_len: 0,
                    },
                );
                namespace.doc_count += 1;
                namespace.total_doc_len += u64::from(terms.doc_len);
            }
        }

        for ((namespace, token), df) in &term_stats {
            ops.push(crate::text_index::term_stats_put_op(namespace, token, *df)?);
        }
        for (namespace, stats) in &namespace_stats {
            ops.push(crate::text_index::namespace_stats_put_op(namespace, stats)?);
        }
        counts.term_stats_entries = term_stats.len() as u64;
        counts.namespace_stats_entries = namespace_stats.len() as u64;

        if !ops.is_empty() {
            engine.write_backend_batch(ops)?;
        }

        Self::write_text_index_state(&engine, &Self::fresh_text_index_state(counts))?;

        let report = TextIndexRebuildReport {
            record_count: counts.record_count,
            posting_entries: counts.posting_entries,
            doc_stats_entries: counts.doc_stats_entries,
            term_stats_entries: counts.term_stats_entries,
            namespace_stats_entries: counts.namespace_stats_entries,
            duration_ms: started.elapsed().as_millis() as u64,
        };
        crate::metrics::record_text_index_rebuild(report.duration_ms, report.posting_entries);
        Ok(report)
    }

    fn rebuild_text_index(&self) -> Result<()> {
        self.rebuild_text_index_with_report().map(|_| ())
    }

    fn expected_text_index_entries(
        engine: &StorageEngine,
        namespace_filter: Option<&str>,
    ) -> Result<ExpectedTextIndexEntries> {
        let mut audit = ExpectedTextIndexEntries::default();
        let mut term_stats: BTreeMap<(String, String), u64> = BTreeMap::new();
        let mut namespace_stats: BTreeMap<String, crate::text_index::TextNamespaceStats> =
            BTreeMap::new();

        for node in engine.scan_nodes()? {
            audit.records_scanned += 1;
            if let Some(record) = memory_record_from_node(node) {
                if matches!(namespace_filter, Some(namespace) if record.namespace != namespace) {
                    continue;
                }
                audit.counts.record_count += 1;
                audit.namespaces.insert(record.namespace.clone());
                let terms = crate::text_index::record_terms(&record.payload);
                for (token, tf) in &terms.token_counts {
                    audit.entries.insert(
                        crate::text_index::posting_key(&record.namespace, token, &record.key),
                        crate::text_index::posting_value(
                            record.node_id,
                            *tf,
                            terms
                                .token_positions
                                .get(token)
                                .map(Vec::as_slice)
                                .unwrap_or(&[]),
                        )?,
                    );
                    audit.counts.posting_entries += 1;
                    *term_stats
                        .entry((record.namespace.clone(), token.clone()))
                        .or_default() += 1;
                }
                audit.entries.insert(
                    crate::text_index::doc_stats_key(&record.namespace, &record.key),
                    crate::text_index::doc_stats_value(record.node_id, terms.doc_len)?,
                );
                audit.counts.doc_stats_entries += 1;
                let namespace = namespace_stats.entry(record.namespace.clone()).or_insert(
                    crate::text_index::TextNamespaceStats {
                        doc_count: 0,
                        total_doc_len: 0,
                    },
                );
                namespace.doc_count += 1;
                namespace.total_doc_len += u64::from(terms.doc_len);
            }
        }

        for ((namespace, token), df) in term_stats {
            audit.entries.insert(
                crate::text_index::term_stats_key(&namespace, &token),
                crate::text_index::term_stats_value(df)?,
            );
        }
        for (namespace, stats) in namespace_stats {
            audit.entries.insert(
                crate::text_index::namespace_stats_key(&namespace),
                crate::text_index::namespace_stats_value(stats.doc_count, stats.total_doc_len)?,
            );
        }

        audit.counts.term_stats_entries = audit
            .entries
            .keys()
            .filter(|key| crate::text_index::is_term_stats_key(key))
            .count() as u64;
        audit.counts.namespace_stats_entries = audit
            .entries
            .keys()
            .filter(|key| crate::text_index::is_namespace_stats_key(key))
            .count() as u64;

        Ok(audit)
    }

    fn text_index_value_readable(key: &[u8], value: &[u8]) -> bool {
        if !crate::text_index::is_internal_key(key) {
            return crate::text_index::decode_posting(value).is_ok();
        }

        if crate::text_index::is_doc_stats_key(key) {
            crate::text_index::decode_doc_stats(value).is_ok()
        } else if crate::text_index::is_term_stats_key(key) {
            crate::text_index::decode_term_stats(value).is_ok()
        } else if crate::text_index::is_namespace_stats_key(key) {
            crate::text_index::decode_namespace_stats(value).is_ok()
        } else {
            false
        }
    }

    fn text_index_state_audit_status(
        engine: &StorageEngine,
        expected_counts: TextIndexCounts,
        namespace_filter: Option<&str>,
    ) -> (bool, String) {
        let state = match Self::load_text_index_state(engine) {
            Ok(Some(state)) => state,
            Ok(None) => return (false, "missing".to_string()),
            Err(err) => return (false, format!("decode_error: {err}")),
        };

        if !Self::text_index_state_matches_spec(&state) {
            return (false, "incompatible".to_string());
        }

        if namespace_filter.is_none()
            && (state.record_count != expected_counts.record_count
                || state.posting_entries != expected_counts.posting_entries
                || state.doc_stats_entries != expected_counts.doc_stats_entries
                || state.term_stats_entries != expected_counts.term_stats_entries
                || state.namespace_stats_entries != expected_counts.namespace_stats_entries)
        {
            return (false, "count_mismatch".to_string());
        }

        (true, "current".to_string())
    }

    fn build_text_index_audit_report_deep(
        engine: &StorageEngine,
        namespace_filter: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        let started = Instant::now();
        let spec = crate::text_index::TextIndexSpec::default();
        let expected = Self::expected_text_index_entries(engine, namespace_filter)?;
        let actual: BTreeMap<Vec<u8>, Vec<u8>> = engine
            .scan_partition(BackendPartition::TextIndex)?
            .into_iter()
            .filter(|(key, _value)| {
                namespace_filter
                    .map(|namespace| {
                        crate::text_index::text_index_key_belongs_to_namespace(key, namespace)
                    })
                    .unwrap_or(true)
            })
            .collect();

        let mut missing_entries = 0u64;
        let mut unexpected_entries = 0u64;
        let mut value_mismatches = 0u64;
        let mut unreadable_entries = 0u64;
        let mut position_errors = 0u64;
        let mut tf_errors = 0u64;
        let mut df_errors = 0u64;
        let mut doc_len_errors = 0u64;
        let mut logical_corruptions = 0u64;

        for (key, value) in &expected.entries {
            match actual.get(key) {
                Some(actual_value) if actual_value == value => {}
                Some(actual_value) => {
                    value_mismatches += 1;
                    if !Self::text_index_value_readable(key, actual_value) {
                        unreadable_entries += 1;
                    } else if crate::text_index::is_doc_stats_key(key) {
                        if let (Ok(expected_stats), Ok(actual_stats)) = (
                            crate::text_index::decode_doc_stats(value),
                            crate::text_index::decode_doc_stats(actual_value),
                        ) {
                            if expected_stats.doc_len != actual_stats.doc_len {
                                doc_len_errors += 1;
                            } else {
                                logical_corruptions += 1;
                            }
                        }
                    } else if crate::text_index::is_term_stats_key(key) {
                        if let (Ok(expected_stats), Ok(actual_stats)) = (
                            crate::text_index::decode_term_stats(value),
                            crate::text_index::decode_term_stats(actual_value),
                        ) {
                            if expected_stats.df != actual_stats.df {
                                df_errors += 1;
                            } else {
                                logical_corruptions += 1;
                            }
                        }
                    } else if !crate::text_index::is_internal_key(key) {
                        if let (Ok(expected_posting), Ok(actual_posting)) = (
                            crate::text_index::decode_posting(value),
                            crate::text_index::decode_posting(actual_value),
                        ) {
                            if expected_posting.tf != actual_posting.tf {
                                tf_errors += 1;
                            }
                            if expected_posting.positions != actual_posting.positions {
                                position_errors += 1;
                            }
                            if expected_posting.tf == actual_posting.tf
                                && expected_posting.positions == actual_posting.positions
                            {
                                logical_corruptions += 1;
                            }
                        }
                    } else {
                        logical_corruptions += 1;
                    }
                }
                None => missing_entries += 1,
            }
        }
        for key in actual.keys() {
            if !expected.entries.contains_key(key) {
                unexpected_entries += 1;
                if let Some(value) = actual.get(key) {
                    if !Self::text_index_value_readable(key, value) {
                        unreadable_entries += 1;
                    }
                }
            }
        }

        let (state_valid, state_status) =
            Self::text_index_state_audit_status(engine, expected.counts, namespace_filter);
        let state_mismatches = u64::from(!state_valid);
        let mismatches = missing_entries + unexpected_entries + value_mismatches + state_mismatches;
        let passed = mismatches == 0;
        let mut namespaces_audited: Vec<String> = expected.namespaces.into_iter().collect();
        if namespaces_audited.is_empty() {
            if let Some(namespace) = namespace_filter {
                namespaces_audited.push(namespace.to_string());
            }
        }

        let report = VantaTextIndexAuditReport {
            schema_version: spec.schema_version,
            tokenizer: spec.tokenizer.name.to_string(),
            tokenizer_version: spec.tokenizer.version,
            key_format: spec.key_format.to_string(),
            namespace_filter: namespace_filter.map(ToOwned::to_owned),
            namespaces_audited,
            records_scanned: expected.records_scanned,
            expected_entries: expected.entries.len() as u64,
            actual_entries: actual.len() as u64,
            missing_entries,
            unexpected_entries,
            value_mismatches,
            unreadable_entries,
            mismatches,
            deep_audit: true,
            position_errors,
            tf_errors,
            df_errors,
            doc_len_errors,
            logical_corruptions,
            state_valid,
            state_status,
            duration_ms: started.elapsed().as_millis() as u64,
            passed,
            status: if passed {
                "ok".to_string()
            } else {
                "repair_recommended".to_string()
            },
        };
        crate::metrics::record_text_consistency_audit(!report.passed);
        Ok(report)
    }

    fn build_text_index_audit_report_shallow(
        engine: &StorageEngine,
        namespace_filter: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        let started = Instant::now();
        let spec = crate::text_index::TextIndexSpec::default();
        let expected = Self::expected_text_index_entries(engine, namespace_filter)?;

        let (state_valid, state_status) =
            Self::text_index_state_audit_status(engine, expected.counts, namespace_filter);

        // Shallow scan: count actual entries and check key presence (no value decoding).
        let actual: BTreeSet<Vec<u8>> = engine
            .scan_partition(BackendPartition::TextIndex)?
            .into_iter()
            .filter(|(key, _value)| {
                namespace_filter
                    .map(|namespace| {
                        crate::text_index::text_index_key_belongs_to_namespace(key, namespace)
                    })
                    .unwrap_or(true)
            })
            .map(|(key, _value)| key)
            .collect();

        let actual_entries = actual.len() as u64;
        let expected_keys: BTreeSet<&Vec<u8>> = expected.entries.keys().collect();
        let missing_entries = expected_keys
            .iter()
            .filter(|key| !actual.contains(**key))
            .count() as u64;
        let unexpected_entries = actual
            .iter()
            .filter(|key| !expected.entries.contains_key(*key))
            .count() as u64;
        let mismatches = missing_entries + unexpected_entries;

        let passed = state_valid && mismatches == 0;

        let mut namespaces_audited: Vec<String> = expected.namespaces.into_iter().collect();
        if namespaces_audited.is_empty() {
            if let Some(namespace) = namespace_filter {
                namespaces_audited.push(namespace.to_string());
            }
        }

        let report = VantaTextIndexAuditReport {
            schema_version: spec.schema_version,
            tokenizer: spec.tokenizer.name.to_string(),
            tokenizer_version: spec.tokenizer.version,
            key_format: spec.key_format.to_string(),
            namespace_filter: namespace_filter.map(ToOwned::to_owned),
            namespaces_audited,
            records_scanned: expected.records_scanned,
            expected_entries: expected.entries.len() as u64,
            actual_entries,
            missing_entries,
            unexpected_entries,
            value_mismatches: 0,
            unreadable_entries: 0,
            mismatches,
            deep_audit: false,
            position_errors: 0,
            tf_errors: 0,
            df_errors: 0,
            doc_len_errors: 0,
            logical_corruptions: 0,
            state_valid,
            state_status,
            duration_ms: started.elapsed().as_millis() as u64,
            passed,
            status: if passed {
                "ok".to_string()
            } else {
                "repair_recommended".to_string()
            },
        };
        crate::metrics::record_text_consistency_audit(!report.passed);
        Ok(report)
    }

    fn indexed_ids_by_namespace(
        &self,
        engine: &StorageEngine,
        namespace: &str,
    ) -> Result<(Vec<u64>, bool)> {
        let prefix = namespace_index_prefix(namespace);
        let entries = engine.scan_partition_prefix(BackendPartition::NamespaceIndex, &prefix)?;
        let mut ids = Vec::new();
        let has_index_entries = Self::load_derived_index_state(engine)?.is_some();
        crate::metrics::record_derived_prefix_scan();

        for (_key, value) in entries {
            if let Some(node_id) = decode_node_id(&value) {
                ids.push(node_id);
            }
        }

        Ok((ids, has_index_entries))
    }

    fn indexed_ids_by_filter(
        &self,
        engine: &StorageEngine,
        namespace: &str,
        field: &str,
        value: &VantaValue,
    ) -> Result<(Vec<u64>, bool)> {
        let prefix = payload_index_prefix(namespace, field, value)?;
        let entries = engine.scan_partition_prefix(BackendPartition::PayloadIndex, &prefix)?;
        let mut ids = Vec::new();
        let has_index_entries = Self::load_derived_index_state(engine)?.is_some();
        crate::metrics::record_derived_prefix_scan();

        for (_key, value) in entries {
            if let Some(node_id) = decode_node_id(&value) {
                ids.push(node_id);
            }
        }

        Ok((ids, has_index_entries))
    }

    fn ensure_text_index_query_ready(engine: &StorageEngine) -> Result<TextIndexState> {
        let state = Self::load_text_index_state(engine).map_err(|_| {
            VantaError::Execution(
                "text_query requires a current BM25 text index; reopen writable or run rebuild_index"
                    .to_string(),
            )
        })?;
        let Some(state) = state else {
            return Err(VantaError::Execution(
                "text_query requires a current BM25 text index; reopen writable or run rebuild_index"
                    .to_string(),
            ));
        };
        if !Self::text_index_state_matches_spec(&state) {
            return Err(VantaError::Execution(
                "text_query requires text_index schema v3; reopen writable or run rebuild_index"
                    .to_string(),
            ));
        }
        Ok(state)
    }

    fn text_positions_match_phrases(
        term_positions: &BTreeMap<String, Vec<u32>>,
        phrases: &[Vec<String>],
    ) -> bool {
        phrases
            .iter()
            .all(|phrase| Self::text_positions_match_phrase(term_positions, phrase))
    }

    fn text_positions_match_phrase(
        term_positions: &BTreeMap<String, Vec<u32>>,
        phrase: &[String],
    ) -> bool {
        let Some(first_token) = phrase.first() else {
            return true;
        };
        let Some(first_positions) = term_positions.get(first_token) else {
            return false;
        };
        if phrase.len() == 1 {
            return !first_positions.is_empty();
        }

        first_positions.iter().any(|start| {
            phrase.iter().enumerate().skip(1).all(|(offset, token)| {
                let Some(positions) = term_positions.get(token) else {
                    return false;
                };
                positions.contains(&start.saturating_add(offset as u32))
            })
        })
    }

    fn lexical_search(
        &self,
        namespace: &str,
        query_text: &str,
        filters: &VantaMemoryMetadata,
        top_k: usize,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        let started = Instant::now();
        let engine = self.engine_handle()?;
        Self::ensure_text_index_query_ready(&engine)?;

        if top_k == 0 {
            crate::metrics::record_text_lexical_query(0, 0);
            return Ok(Vec::new());
        }

        let query_plan = crate::text_index::query_plan(query_text);
        if query_plan.terms.is_empty() {
            crate::metrics::record_text_lexical_query(0, 0);
            return Ok(Vec::new());
        }

        let Some(namespace_stats) = Self::load_text_namespace_stats(&engine, namespace)? else {
            crate::metrics::record_text_lexical_query(started.elapsed().as_millis() as u64, 0);
            return Ok(Vec::new());
        };
        if namespace_stats.doc_count == 0 {
            crate::metrics::record_text_lexical_query(started.elapsed().as_millis() as u64, 0);
            return Ok(Vec::new());
        }

        let doc_count = namespace_stats.doc_count as f32;
        let avg_doc_len = if namespace_stats.total_doc_len == 0 {
            1.0
        } else {
            namespace_stats.total_doc_len as f32 / doc_count
        };
        let mut scores: BTreeMap<u64, f32> = BTreeMap::new();
        let mut candidate_positions: BTreeMap<u64, BTreeMap<String, Vec<u32>>> = BTreeMap::new();
        let mut doc_stats_cache: BTreeMap<String, crate::text_index::TextDocStats> =
            BTreeMap::new();
        let mut candidates_scored = 0u64;

        for token in query_plan.terms {
            let Some(term_stats) = Self::load_text_term_stats(&engine, namespace, &token)? else {
                continue;
            };
            if term_stats.df == 0 {
                continue;
            }

            let df = term_stats.df as f32;
            let idf = (1.0 + ((doc_count - df + 0.5) / (df + 0.5))).ln();
            let prefix = crate::text_index::posting_prefix(namespace, &token);
            for (posting_key, posting_value) in
                engine.scan_partition_prefix(BackendPartition::TextIndex, &prefix)?
            {
                if crate::text_index::is_internal_key(&posting_key) {
                    continue;
                }
                let posting = crate::text_index::decode_posting(&posting_value).map_err(|err| {
                    VantaError::Execution(format!(
                        "text_query found an unreadable posting; run rebuild_index: {err}"
                    ))
                })?;
                let Some(record_key) =
                    crate::text_index::posting_record_key(namespace, &token, &posting_key)
                else {
                    continue;
                };
                let doc_stats = if let Some(stats) = doc_stats_cache.get(&record_key) {
                    stats.clone()
                } else {
                    let Some(stats) = Self::load_text_doc_stats(&engine, namespace, &record_key)?
                    else {
                        return Err(VantaError::Execution(
                            "text_query found posting without document stats; run rebuild_index"
                                .to_string(),
                        ));
                    };
                    doc_stats_cache.insert(record_key.clone(), stats.clone());
                    stats
                };
                if doc_stats.node_id != posting.node_id {
                    return Err(VantaError::Execution(
                        "text_query found posting/doc stats mismatch; run rebuild_index"
                            .to_string(),
                    ));
                }

                let tf = posting.tf as f32;
                let doc_len = doc_stats.doc_len as f32;
                let denominator = tf
                    + crate::text_index::BM25_K1
                        * (1.0 - crate::text_index::BM25_B
                            + crate::text_index::BM25_B * (doc_len / avg_doc_len));
                let contribution = idf * ((tf * (crate::text_index::BM25_K1 + 1.0)) / denominator);
                scores
                    .entry(posting.node_id)
                    .and_modify(|score| *score += contribution)
                    .or_insert(contribution);
                candidate_positions
                    .entry(posting.node_id)
                    .or_default()
                    .insert(token.clone(), posting.positions);
                candidates_scored += 1;
            }
        }

        let mut hits = Vec::new();
        for (node_id, score) in scores {
            let positions_match = candidate_positions
                .get(&node_id)
                .map(|positions| Self::text_positions_match_phrases(positions, &query_plan.phrases))
                .unwrap_or(query_plan.phrases.is_empty());
            if !positions_match {
                continue;
            }
            if let Some(node) = engine.get(node_id)? {
                if let Some(record) = memory_record_from_node(node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        hits.push(VantaMemorySearchHit {
                            record,
                            score,
                            explanation: None,
                        });
                    }
                }
            }
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.record.key.cmp(&b.record.key))
                .then(a.record.node_id.cmp(&b.record.node_id))
        });
        hits.truncate(top_k);
        crate::metrics::record_text_lexical_query(
            started.elapsed().as_millis() as u64,
            candidates_scored,
        );
        Ok(hits)
    }

    fn vector_memory_search(
        &self,
        namespace: &str,
        query_vector: &[f32],
        filters: &VantaMemoryMetadata,
        top_k: usize,
        distance_metric: DistanceMetric,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        if query_vector.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }

        let engine = self.engine_handle()?;

        // Paso 1: buscar candidatos via HNSW con un budget ampliado para compensar el
        // post-filtrado por namespace. El índice HNSW no conoce namespaces — son una
        // abstracción del SDK — así que buscamos más candidatos de los estrictamente
        // necesarios y luego filtramos. Budget: min(top_k * 10, 500) garantiza cobertura
        // en datasets típicos sin disparar el costo de traversal.
        let budget = (top_k.saturating_mul(10)).min(500).max(top_k);
        let candidates = {
            let hnsw = engine.hnsw.load();
            let vs = engine.vector_store.read();
            hnsw.search_nearest(query_vector, None, None, u128::MAX, budget, Some(&*vs))
        };

        // Paso 2: post-filtrado por namespace y metadata, cargando cada nodo candidato.
        let mut hits = Vec::with_capacity(top_k);
        for (node_id, raw_score) in candidates {
            if hits.len() >= top_k {
                break;
            }
            if let Some(node) = engine.get(node_id)? {
                if let Some(record) = memory_record_from_node(node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        // El score de HNSW ya viene ajustado (negativo para euclidean),
                        // coherente con el contrato de `distance_metric` del caller.
                        let score = if distance_metric == DistanceMetric::Euclidean {
                            // HNSW retorna -sqrt(dist²) para Euclidean; lo propagamos tal cual.
                            raw_score
                        } else {
                            raw_score
                        };
                        hits.push(VantaMemorySearchHit {
                            score,
                            record,
                            explanation: None,
                        });
                    }
                }
            }
        }

        // Si el HNSW no encontró suficientes resultados del namespace (e.g. namespace muy
        // pequeño o HNSW vacío), hacemos fallback al scan lineal para garantizar correctitud.
        if hits.is_empty() && !query_vector.is_empty() {
            for record in self.records_for_namespace(namespace, filters)? {
                let Some(vector) = record.vector.as_ref() else {
                    continue;
                };
                if vector.len() != query_vector.len() {
                    continue;
                }
                let score = match distance_metric {
                    DistanceMetric::Cosine => cosine_sim_f32(query_vector, vector),
                    DistanceMetric::Euclidean => {
                        -crate::index::euclidean_distance_squared_f32(query_vector, vector)
                    }
                };
                hits.push(VantaMemorySearchHit {
                    score,
                    record,
                    explanation: None,
                });
            }
            Self::sort_memory_hits(&mut hits);
            hits.truncate(top_k);
            if distance_metric == DistanceMetric::Euclidean {
                for hit in hits.iter_mut() {
                    hit.score = -(-hit.score).max(0.0).sqrt();
                }
            }
        }

        Ok(hits)
    }

    fn sort_memory_hits(hits: &mut [VantaMemorySearchHit]) {
        crate::planner::sort_hits(hits);
    }

    fn hybrid_candidate_budget(top_k: usize) -> usize {
        crate::planner::hybrid_candidate_budget(top_k)
    }

    fn hybrid_search(
        &self,
        namespace: &str,
        query_vector: &[f32],
        text_query: &str,
        filters: &VantaMemoryMetadata,
        top_k: usize,
        distance_metric: DistanceMetric,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        let started = Instant::now();
        if top_k == 0 {
            crate::metrics::record_hybrid_query(0, 0);
            return Ok(Vec::new());
        }

        let budget = Self::hybrid_candidate_budget(top_k);
        let lexical_hits = self.lexical_search(namespace, text_query, filters, budget)?;
        let vector_hits =
            self.vector_memory_search(namespace, query_vector, filters, budget, distance_metric)?;
        let mut hits = Self::fuse_rrf(lexical_hits, vector_hits);
        let candidates_fused = hits.len() as u64;
        hits.truncate(top_k);
        crate::metrics::record_hybrid_query(started.elapsed().as_millis() as u64, candidates_fused);
        Ok(hits)
    }

    fn fuse_rrf(
        lexical_hits: Vec<VantaMemorySearchHit>,
        vector_hits: Vec<VantaMemorySearchHit>,
    ) -> Vec<VantaMemorySearchHit> {
        crate::planner::fuse_rrf(lexical_hits, vector_hits)
    }

    fn records_for_namespace(
        &self,
        namespace: &str,
        filters: &VantaMemoryMetadata,
    ) -> Result<Vec<VantaMemoryRecord>> {
        let engine = self.engine_handle()?;

        let (candidate_ids, has_index_entries) = if let Some((field, value)) = filters.iter().next()
        {
            self.indexed_ids_by_filter(&engine, namespace, field, value)?
        } else {
            self.indexed_ids_by_namespace(&engine, namespace)?
        };

        let mut records = Vec::new();
        let mut seen = BTreeSet::new();

        for node_id in candidate_ids {
            if !seen.insert(node_id) {
                continue;
            }
            if let Some(node) = engine.get(node_id)? {
                if let Some(record) = memory_record_from_node(node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        records.push(record);
                    }
                }
            }
        }

        if records.is_empty() && !has_index_entries {
            crate::metrics::record_derived_full_scan_fallback();
            for node in engine.scan_nodes()? {
                if let Some(record) = memory_record_from_node(node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        records.push(record);
                    }
                }
            }
        }

        records.sort_by(|a, b| a.key.cmp(&b.key).then(a.node_id.cmp(&b.node_id)));
        Ok(records)
    }

    fn put_record_exact(&self, record: VantaMemoryRecord) -> Result<VantaMemoryRecord> {
        validate_namespace(&record.namespace)?;
        validate_key(&record.key)?;
        validate_metadata(&record.metadata)?;

        let expected_node_id = memory_node_id(&record.namespace, &record.key);
        if record.node_id != expected_node_id {
            return Err(VantaError::Execution(format!(
                "node_id does not match deterministic namespace/key hash for namespace='{}' key='{}'",
                record.namespace, record.key
            )));
        }

        let engine = self.engine_handle()?;
        let previous = match engine.get(record.node_id)? {
            Some(node) => match memory_record_from_node(node) {
                Some(previous)
                    if previous.namespace == record.namespace && previous.key == record.key =>
                {
                    Some(previous)
                }
                _ => {
                    return Err(VantaError::Execution(format!(
                        "node id collision for namespace='{}' key='{}'",
                        record.namespace, record.key
                    )));
                }
            },
            None => None,
        };

        let node = memory_record_to_node(&record);
        engine.insert(&node)?;
        self.replace_derived_indexes(&engine, previous.as_ref(), Some(&record))?;

        Ok(record)
    }

    pub fn insert_node(&self, input: VantaNodeInput) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut node = UnifiedNode::new(input.id);

        if let Some(content) = input.content {
            node.set_field("content", FieldValue::String(content));
        }

        for (key, value) in input.fields {
            node.set_field(key, value.into());
        }

        if let Some(vector) = input.vector.filter(|vector| !vector.is_empty()) {
            node.vector = VectorRepresentations::Full(vector);
            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        }

        engine.insert(&node)
    }

    pub fn get_node(&self, id: u64) -> Result<Option<VantaNodeRecord>> {
        self.engine_handle()?
            .get(id)
            .map(|node| node.map(Into::into))
    }

    pub fn delete_node(&self, id: u64, reason: &str) -> Result<()> {
        self.engine_handle()?.delete(id, reason)
    }

    /// Insert or update multiple namespace-scoped persistent memory records in parallel.
    ///
    /// Validates all inputs upfront (fail-fast on invalid namespaces/keys/metadata),
    /// then processes the batch in parallel using Rayon for up to 5x throughput
    /// improvement over sequential `put()` calls.
    pub fn put_batch(&self, inputs: Vec<VantaMemoryInput>) -> Result<Vec<VantaMemoryRecord>> {
        use rayon::prelude::*;

        for input in &inputs {
            validate_namespace(&input.namespace)?;
            validate_key(&input.key)?;
            validate_metadata(&input.metadata)?;
        }

        let results: Vec<Result<VantaMemoryRecord>> = inputs
            .into_par_iter()
            .map(|input| {
                let engine = self.engine_handle()?;
                let node_id = memory_node_id(&input.namespace, &input.key);
                let existing = match engine.get(node_id)? {
                    Some(node) => match memory_record_from_node(node) {
                        Some(record)
                            if record.namespace == input.namespace && record.key == input.key =>
                        {
                            Some(record)
                        }
                        _ => {
                            return Err(VantaError::Execution(format!(
                                "node id collision for namespace='{}' key='{}'",
                                input.namespace, input.key
                            )));
                        }
                    },
                    None => None,
                };

                let timestamp = now_ms();
                let created_at_ms = existing
                    .as_ref()
                    .map(|record| record.created_at_ms)
                    .unwrap_or(timestamp);
                let version = existing
                    .as_ref()
                    .map(|record| record.version.saturating_add(1))
                    .unwrap_or(1);
                let expires_at_ms = input.ttl_ms.map(|ttl| timestamp.saturating_add(ttl));

                let record = VantaMemoryRecord {
                    namespace: input.namespace,
                    key: input.key,
                    payload: input.payload,
                    metadata: input.metadata,
                    created_at_ms,
                    updated_at_ms: timestamp,
                    version,
                    node_id,
                    vector: input.vector.filter(|v| !v.is_empty()),
                    expires_at_ms,
                };
                let node = memory_record_to_node(&record);
                engine.insert(&node)?;
                self.replace_derived_indexes(&engine, existing.as_ref(), Some(&record))?;
                Ok(record)
            })
            .collect();

        results.into_iter().collect()
    }

    pub fn put(&self, input: VantaMemoryInput) -> Result<VantaMemoryRecord> {
        validate_namespace(&input.namespace)?;
        validate_key(&input.key)?;
        validate_metadata(&input.metadata)?;

        let engine = self.engine_handle()?;
        let node_id = memory_node_id(&input.namespace, &input.key);
        let existing = match engine.get(node_id)? {
            Some(node) => match memory_record_from_node(node) {
                Some(record) if record.namespace == input.namespace && record.key == input.key => {
                    Some(record)
                }
                // TTL-expired or stale node — treat as non-existing.
                _ => None,
            },
            None => None,
        };

        let timestamp = now_ms();
        let created_at_ms = existing
            .as_ref()
            .map(|r| r.created_at_ms)
            .unwrap_or(timestamp);
        let version = existing
            .as_ref()
            .map(|r| r.version.saturating_add(1))
            .unwrap_or(1);
        let expires_at_ms = input.ttl_ms.map(|ttl| timestamp.saturating_add(ttl));

        let record = VantaMemoryRecord {
            namespace: input.namespace,
            key: input.key,
            payload: input.payload,
            metadata: input.metadata,
            created_at_ms,
            updated_at_ms: timestamp,
            version,
            node_id,
            vector: input.vector.filter(|v| !v.is_empty()),
            expires_at_ms,
        };
        let node = memory_record_to_node(&record);
        engine.insert(&node)?;
        self.replace_derived_indexes(&engine, existing.as_ref(), Some(&record))?;

        Ok(record)
    }

    pub fn get(&self, namespace: &str, key: &str) -> Result<Option<VantaMemoryRecord>> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let node_id = memory_node_id(namespace, key);
        let Some(node) = self.engine_handle()?.get(node_id)? else {
            return Ok(None);
        };

        match memory_record_from_node(node) {
            Some(record) if record.namespace == namespace && record.key == key => Ok(Some(record)),
            Some(_record) => Err(VantaError::Execution(format!(
                "node id collision for namespace='{}' key='{}'",
                namespace, key
            ))),
            None => Ok(None),
        }
    }

    pub fn delete(&self, namespace: &str, key: &str) -> Result<bool> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let Some(existing) = self.get(namespace, key)? else {
            return Ok(false);
        };

        let node_id = memory_node_id(namespace, key);
        let engine = self.engine_handle()?;
        engine.delete(node_id, "memory delete")?;
        self.replace_derived_indexes(&engine, Some(&existing), None)?;
        Ok(true)
    }

    pub fn list_namespaces(&self) -> Result<Vec<String>> {
        let engine = self.engine_handle()?;
        let mut namespaces = BTreeSet::new();
        let entries = engine.scan_partition(BackendPartition::NamespaceIndex)?;

        if entries.is_empty() {
            for node in engine.scan_nodes()? {
                if let Some(record) = memory_record_from_node(node) {
                    namespaces.insert(record.namespace);
                }
            }
        } else {
            for (key, _value) in entries {
                if let Some(separator) = key.iter().position(|byte| *byte == 0) {
                    if let Ok(namespace) = String::from_utf8(key[..separator].to_vec()) {
                        namespaces.insert(namespace);
                    }
                }
            }
        }

        Ok(namespaces.into_iter().collect())
    }

    pub fn list(
        &self,
        namespace: &str,
        options: VantaMemoryListOptions,
    ) -> Result<VantaMemoryListPage> {
        validate_namespace(namespace)?;
        validate_metadata(&options.filters)?;

        let records = self.records_for_namespace(namespace, &options.filters)?;

        let start = options.cursor.unwrap_or(0).min(records.len());
        let limit = options.limit.max(1);
        let end = start.saturating_add(limit).min(records.len());
        let next_cursor = (end < records.len()).then_some(end);

        Ok(VantaMemoryListPage {
            records: records[start..end].to_vec(),
            next_cursor,
        })
    }

    pub fn search(&self, request: VantaMemorySearchRequest) -> Result<Vec<VantaMemorySearchHit>> {
        validate_namespace(&request.namespace)?;
        validate_metadata(&request.filters)?;

        let text_query = crate::planner::trimmed_text_query(&request);
        let has_vector = !request.query_vector.is_empty();

        if request.top_k == 0 {
            return Ok(Vec::new());
        }

        if request.explain {
            let engine = self.engine_handle()?;
            let (hits, text_ranks, vector_ranks) = match (text_query, has_vector) {
                (Some(text_query), true) => {
                    let budget = Self::hybrid_candidate_budget(request.top_k);
                    let lexical_hits = self.lexical_search(
                        &request.namespace,
                        text_query,
                        &request.filters,
                        budget,
                    )?;
                    let vector_hits = self.vector_memory_search(
                        &request.namespace,
                        &request.query_vector,
                        &request.filters,
                        budget,
                        request.distance_metric,
                    )?;
                    let text_ranks = Self::debug_rank_map(&lexical_hits);
                    let vector_ranks = Self::debug_rank_map(&vector_hits);
                    let (mut hits, _report) =
                        crate::planner::fuse_rrf_with_report(lexical_hits, vector_hits);
                    hits.truncate(request.top_k);
                    (hits, text_ranks, vector_ranks)
                }
                (Some(text_query), false) => {
                    let hits = self.lexical_search(
                        &request.namespace,
                        text_query,
                        &request.filters,
                        request.top_k,
                    )?;
                    let text_ranks = Self::debug_rank_map(&hits);
                    (hits, text_ranks, BTreeMap::new())
                }
                (None, true) => {
                    let hits = self.vector_memory_search(
                        &request.namespace,
                        &request.query_vector,
                        &request.filters,
                        request.top_k,
                        request.distance_metric,
                    )?;
                    let vector_ranks = Self::debug_rank_map(&hits);
                    (hits, BTreeMap::new(), vector_ranks)
                }
                (None, false) => (Vec::new(), BTreeMap::new(), BTreeMap::new()),
            };

            let explained_hits = hits
                .into_iter()
                .map(|mut hit| {
                    let explanation = Self::debug_explain_hit(
                        &engine,
                        hit.clone(),
                        text_query,
                        &text_ranks,
                        &vector_ranks,
                    )?;
                    hit.explanation = Some(explanation);
                    Ok(hit)
                })
                .collect::<Result<Vec<_>>>()?;

            return Ok(explained_hits);
        }

        match (text_query, has_vector) {
            (Some(text_query), true) => {
                crate::metrics::record_planner_hybrid_query();
                self.hybrid_search(
                    &request.namespace,
                    &request.query_vector,
                    text_query,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                )
            }
            (Some(text_query), false) => {
                crate::metrics::record_planner_text_only_query();
                self.lexical_search(
                    &request.namespace,
                    text_query,
                    &request.filters,
                    request.top_k,
                )
            }
            (None, true) => {
                crate::metrics::record_planner_vector_only_query();
                self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                )
            }
            (None, false) => Ok(Vec::new()),
        }
    }

    pub fn rebuild_index(&self) -> Result<VantaIndexRebuildReport> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "rebuild_index is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        let report = self.engine_handle()?.rebuild_vector_index()?;
        let derived = self.rebuild_derived_indexes_with_report()?;
        self.rebuild_text_index_with_report()?;
        let mut report: VantaIndexRebuildReport = report.into();
        report.derived_rebuild_ms = derived.duration_ms;
        Ok(report)
    }

    /// Compacta físicamente el archivo de vectores (`vector_store.vanta`) reescribiendo
    /// los nodos en orden BFS desde el entry point del grafo HNSW.
    ///
    /// Esta operación reduce drásticamente los page-faults en accesos MMap durante
    /// búsquedas semánticas, ya que agrupa los nodos más conectados (hubs y capas
    /// superiores del HNSW) en las primeras páginas virtuales del archivo.
    ///
    /// Retorna el número de nodos compactados.
    pub fn compact_layout(&self) -> Result<u64> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "compact_layout is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        self.engine_handle()?.compact_layout_bfs()
    }

    pub fn export_namespace(
        &self,
        path: impl AsRef<Path>,
        namespace: &str,
    ) -> Result<VantaExportReport> {
        validate_namespace(namespace)?;
        let started = Instant::now();
        let records = self.records_for_namespace(namespace, &VantaMemoryMetadata::new())?;
        self.write_export_file(path.as_ref(), records, vec![namespace.to_string()], started)
    }

    pub fn export_all(&self, path: impl AsRef<Path>) -> Result<VantaExportReport> {
        let started = Instant::now();
        let namespaces = self.list_namespaces()?;
        let mut records = Vec::new();
        for namespace in &namespaces {
            records.extend(self.records_for_namespace(namespace, &VantaMemoryMetadata::new())?);
        }
        self.write_export_file(path.as_ref(), records, namespaces, started)
    }

    fn write_export_file(
        &self,
        path: &Path,
        records: Vec<VantaMemoryRecord>,
        namespaces: Vec<String>,
        started: Instant,
    ) -> Result<VantaExportReport> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(VantaError::IoError)?;
        }

        let file = File::create(path).map_err(VantaError::IoError)?;
        let mut writer = BufWriter::new(file);
        let records_exported = records.len() as u64;

        for record in records {
            let line = export_line_from_record(record);
            serde_json::to_writer(&mut writer, &line)
                .map_err(|err| VantaError::SerializationError(err.to_string()))?;
            writer.write_all(b"\n").map_err(VantaError::IoError)?;
        }
        writer.flush().map_err(VantaError::IoError)?;
        crate::metrics::record_export(records_exported);

        Ok(VantaExportReport {
            records_exported,
            namespaces,
            path: path.to_string_lossy().into_owned(),
            duration_ms: started.elapsed().as_millis() as u64,
        })
    }

    pub fn import_records(&self, records: Vec<VantaMemoryRecord>) -> Result<VantaImportReport> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "import_records is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        let started = Instant::now();
        let mut report = VantaImportReport {
            inserted: 0,
            updated: 0,
            skipped: 0,
            errors: 0,
            duration_ms: 0,
        };

        for record in records {
            let existed = matches!(self.get(&record.namespace, &record.key), Ok(Some(_)));
            match self.put_record_exact(record) {
                Ok(_) if existed => report.updated += 1,
                Ok(_) => report.inserted += 1,
                Err(_) => report.errors += 1,
            }
        }

        self.rebuild_derived_indexes()?;
        self.rebuild_text_index()?;
        report.duration_ms = started.elapsed().as_millis() as u64;
        crate::metrics::record_import(report.inserted + report.updated, report.errors);
        Ok(report)
    }

    pub fn import_file(&self, path: impl AsRef<Path>) -> Result<VantaImportReport> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "import_file is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        let started = Instant::now();
        let file = File::open(path.as_ref()).map_err(VantaError::IoError)?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        let mut skipped = 0u64;
        let mut errors = 0u64;

        for line in reader.lines() {
            let line = line.map_err(VantaError::IoError)?;
            if line.trim().is_empty() {
                skipped += 1;
                continue;
            }

            match serde_json::from_str::<VantaMemoryExportLine>(&line)
                .map_err(|err| VantaError::SerializationError(err.to_string()))
                .and_then(record_from_export_line)
            {
                Ok(record) => records.push(record),
                Err(_) => errors += 1,
            }
        }

        let mut report = self.import_records(records)?;
        report.skipped += skipped;
        report.errors += errors;
        if errors > 0 {
            crate::metrics::record_import(0, errors);
        }
        report.duration_ms = started.elapsed().as_millis() as u64;
        Ok(report)
    }

    /// Run a read-only structural audit of the derived persistent text index.
    ///
    /// The audit compares postings, BM25 stats, phrase positions, and the
    /// state marker against canonical memory records. It never repairs state;
    /// callers should use `rebuild_index` when the report returns `passed =
    /// false`.
    pub fn audit_text_index(&self, namespace: Option<&str>) -> Result<VantaTextIndexAuditReport> {
        if let Some(namespace) = namespace {
            validate_namespace(namespace)?;
        }
        let engine = self.engine_handle()?;
        Self::build_text_index_audit_report_shallow(&engine, namespace)
    }

    /// Run a deep structural audit of the derived persistent text index.
    ///
    /// The audit decodes and compares individual fields (TF, positions, DF, doc lengths)
    /// across all postings against the canonical memory records.
    pub fn audit_text_index_deep(
        &self,
        namespace: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        if let Some(namespace) = namespace {
            validate_namespace(namespace)?;
        }
        let engine = self.engine_handle()?;
        Self::build_text_index_audit_report_deep(&engine, namespace)
    }

    /// Public repair primitive for the text index. Rebuilds all postings,
    /// doc stats, term stats, and namespace stats from canonical memory records.
    pub fn repair_text_index(&self) -> Result<VantaTextIndexRepairReport> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "repair_text_index is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        crate::metrics::record_text_index_repair();
        let report = self.rebuild_text_index_with_report()?;
        Ok(VantaTextIndexRepairReport {
            record_count: report.record_count,
            posting_entries: report.posting_entries,
            doc_stats_entries: report.doc_stats_entries,
            term_stats_entries: report.term_stats_entries,
            namespace_stats_entries: report.namespace_stats_entries,
            duration_ms: report.duration_ms,
            success: true,
        })
    }

    pub fn operational_metrics(&self) -> VantaOperationalMetrics {
        if let Ok(engine) = self.engine_handle() {
            let stats = engine.get_memory_stats();
            crate::metrics::record_memory_breakdown(
                stats.node_count,
                stats.logical_bytes,
                stats.physical_rss,
                stats.cache_entries as u64,
                0,
            );
        }
        crate::metrics::operational_metrics_snapshot().into()
    }

    /// K-NN vector search across all nodes via HNSW index.
    ///
    /// Complejidad: O(log N) en promedio. Anteriormente era O(N) brute-force.
    pub fn search_vector(&self, vector: &[f32], top_k: usize) -> Result<Vec<VantaSearchHit>> {
        if vector.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }
        let engine = self.engine_handle()?;
        let results = {
            let hnsw = engine.hnsw.load();
            let vs = engine.vector_store.read();
            hnsw.search_nearest(
                vector,
                None,      // q_1bit: no aplica para búsqueda de alta precisión
                None,      // q_3bit: no aplica
                u128::MAX, // query_mask: sin filtro de bitset — retornar todos
                top_k,
                Some(&*vs), // vector_store para MMap path
            )
        };
        Ok(results
            .into_iter()
            .map(|(node_id, distance)| VantaSearchHit { node_id, distance })
            .collect())
    }

    /// Flush WAL y archivos memory-mapped a disco para garantizar durabilidad.
    ///
    /// Delega al `StorageEngine::flush()` que sincroniza el backend KV y el
    /// archivo de vectores MMap. Antes era un no-op silencioso.
    pub fn flush(&self) -> Result<()> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "flush is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        self.engine_handle()?.flush()
    }

    /// Compact the WAL: flush, archive the current WAL file as
    /// ``vanta.wal.<timestamp>``, and start a fresh WAL.
    ///
    /// Safe to call at any time.  Archived WALs can be removed
    /// once no longer needed for crash recovery.
    pub fn compact_wal(&self) -> Result<()> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "compact_wal is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        self.engine_handle()?.compact_wal()
    }

    /// Scan all memory records and physically delete those whose
    /// ``expires_at_ms`` deadline has passed.  Returns the number
    /// of records purged.
    pub fn purge_expired(&self) -> Result<u64> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "purge_expired is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        let engine = self.engine_handle()?;
        let now = now_ms();
        let mut to_delete: Vec<(String, String, u64)> = Vec::new();

        // Scan all nodes without TTL filtering (scan directly from engine).
        for node in engine.scan_nodes()? {
            if !node.is_alive() {
                continue;
            }
            let namespace = match node.get_field(FIELD_NAMESPACE) {
                Some(crate::node::FieldValue::String(ns)) => ns.clone(),
                _ => continue,
            };
            let key = match node.get_field(FIELD_KEY) {
                Some(crate::node::FieldValue::String(k)) => k.clone(),
                _ => continue,
            };
            let expires = match node.get_field(FIELD_EXPIRES_AT_MS) {
                Some(crate::node::FieldValue::Int(ms)) if *ms > 0 => *ms as u64,
                _ => continue,
            };
            if now > expires {
                to_delete.push((namespace, key, node.id));
            }
        }

        let count = to_delete.len() as u64;
        for (_, _, node_id) in &to_delete {
            engine.delete(*node_id, "purge_expired")?;
            self.replace_derived_indexes(&engine, None, None)?;
        }

        Ok(count)
    }

    /// Return stable runtime capabilities.
    pub fn capabilities(&self) -> VantaCapabilities {
        VantaCapabilities {
            runtime_profile: VantaRuntimeProfile::Performance,
            persistence: true,
            vector_search: true,
            iql_queries: true,
            read_only: self.config.read_only,
        }
    }

    /// Add a directed edge between two nodes.
    pub fn add_edge(
        &self,
        source_id: u64,
        target_id: u64,
        label: &str,
        weight: Option<f32>,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut node = engine
            .get(source_id)?
            .ok_or(VantaError::NodeNotFound(source_id))?;
        node.edges.push(crate::node::Edge {
            target: target_id,
            label: label.to_string(),
            weight: weight.unwrap_or(1.0),
        });
        engine.insert(&node)
    }

    /// Flush and close the embedded engine handle.
    pub fn close(&self) -> Result<()> {
        let _ = self.flush();
        let mut guard = self.engine.write();
        *guard = None;
        Ok(())
    }

    /// Execute an IQL query.
    pub fn query(&self, query: &str) -> Result<VantaQueryResult> {
        let engine = self.engine_handle()?;
        let executor = Executor::new(&engine);
        let result = executor.execute_hybrid(query)?;
        Ok(result.into())
    }

    /// Generate a text snippet with optional highlighting of matched terms.
    ///
    /// # Arguments
    /// * `payload` - The original text content
    /// * `text_query` - The search query used to find matches
    /// * `with_highlighting` - Whether to add HTML highlighting to matched terms
    ///
    /// # Returns
    /// * `Option<String>` - The snippet with optional highlighting, or None if no match found
    pub fn generate_snippet(
        &self,
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
        Self::generate_snippet_with_highlighting(payload, text_query, with_highlighting)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_memory_breakdown(&self) -> serde_json::Value {
        let metrics = self.operational_metrics();
        serde_json::json!({
            "process_rss_bytes": metrics.process_rss_bytes,
            "process_virtual_bytes": metrics.process_virtual_bytes,
            "hnsw_nodes_count": metrics.hnsw_nodes_count,
            "hnsw_logical_bytes": metrics.hnsw_logical_bytes,
            "mmap_resident_bytes": metrics.mmap_resident_bytes,
            "volatile_cache_entries": metrics.volatile_cache_entries,
            "volatile_cache_cap_bytes": metrics.volatile_cache_cap_bytes,
        })
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_derived_index_state_for_tests(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            DERIVED_INDEX_STATE_KEY,
            b"corrupt-derived-index-state",
        )
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_clear_derived_indexes_for_tests(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut ops = Vec::new();
        for (key, _value) in engine.scan_partition(BackendPartition::NamespaceIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::NamespaceIndex,
                key,
            });
        }
        for (key, _value) in engine.scan_partition(BackendPartition::PayloadIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::PayloadIndex,
                key,
            });
        }
        engine.write_backend_batch(ops)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_state_for_tests(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            TEXT_INDEX_STATE_KEY,
            b"corrupt-text-index-state",
        )
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_clear_text_index_for_tests(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut ops = Vec::new();
        for (key, _value) in engine.scan_partition(BackendPartition::TextIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::TextIndex,
                key,
            });
        }
        engine.write_backend_batch(ops)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_posting_tf_for_tests(
        &self,
        namespace: &str,
        token: &str,
        key: &str,
        new_tf: u32,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let pkey = crate::text_index::posting_key(namespace, token, key);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &pkey)? else {
            return Err(VantaError::Execution("posting not found".to_string()));
        };
        let posting = crate::text_index::decode_posting(&bytes)?;
        let val = crate::text_index::posting_value(posting.node_id, new_tf, &posting.positions)?;
        engine.put_to_partition(BackendPartition::TextIndex, &pkey, &val)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_posting_positions_for_tests(
        &self,
        namespace: &str,
        token: &str,
        key: &str,
        new_positions: Vec<u32>,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let pkey = crate::text_index::posting_key(namespace, token, key);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &pkey)? else {
            return Err(VantaError::Execution("posting not found".to_string()));
        };
        let posting = crate::text_index::decode_posting(&bytes)?;
        let val = crate::text_index::posting_value(posting.node_id, posting.tf, &new_positions)?;
        engine.put_to_partition(BackendPartition::TextIndex, &pkey, &val)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_term_stats_for_tests(
        &self,
        namespace: &str,
        token: &str,
        new_df: u64,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let skey = crate::text_index::term_stats_key(namespace, token);
        let val = crate::text_index::term_stats_value(new_df)?;
        engine.put_to_partition(BackendPartition::TextIndex, &skey, &val)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_corrupt_text_index_doc_stats_for_tests(
        &self,
        namespace: &str,
        key: &str,
        new_doc_len: u32,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let dkey = crate::text_index::doc_stats_key(namespace, key);
        let Some(bytes) = engine.get_from_partition(BackendPartition::TextIndex, &dkey)? else {
            return Err(VantaError::Execution("doc stats not found".to_string()));
        };
        let stats = crate::text_index::decode_doc_stats(&bytes)?;
        let val = crate::text_index::doc_stats_value(stats.node_id, new_doc_len)?;
        engine.put_to_partition(BackendPartition::TextIndex, &dkey, &val)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_text_index_posting_keys_for_tests(&self) -> Result<Vec<Vec<u8>>> {
        let engine = self.engine_handle()?;
        let mut keys: Vec<Vec<u8>> = engine
            .scan_partition(BackendPartition::TextIndex)?
            .into_iter()
            .map(|(key, _value)| key)
            .filter(|key| !crate::text_index::is_internal_key(key))
            .collect();
        keys.sort();
        Ok(keys)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_text_index_posting_for_tests(
        &self,
        namespace: &str,
        token: &str,
        key: &str,
    ) -> Result<Option<(u64, u32)>> {
        let engine = self.engine_handle()?;
        let Some(bytes) = engine.get_from_partition(
            BackendPartition::TextIndex,
            &crate::text_index::posting_key(namespace, token, key),
        )?
        else {
            return Ok(None);
        };
        let posting = crate::text_index::decode_posting(&bytes)?;
        Ok(Some((posting.node_id, posting.tf)))
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_text_index_audit_for_tests(&self) -> Result<VantaTextIndexAuditReport> {
        self.audit_text_index_deep(None)
    }

    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn debug_memory_search_plan_for_tests(
        &self,
        request: VantaMemorySearchRequest,
    ) -> Result<VantaMemorySearchDebugReport> {
        validate_namespace(&request.namespace)?;
        validate_metadata(&request.filters)?;

        let text_query = crate::planner::trimmed_text_query(&request);
        let has_vector = !request.query_vector.is_empty();
        if request.top_k == 0 {
            return Ok(VantaMemorySearchDebugReport {
                route: "empty".to_string(),
                budget: 0,
                text_candidates: 0,
                vector_candidates: 0,
                fused_candidates: 0,
                top_identities: Vec::new(),
            });
        }

        match (text_query, has_vector) {
            (Some(text_query), true) => {
                let budget = Self::hybrid_candidate_budget(request.top_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let vector_hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    budget,
                    request.distance_metric,
                )?;
                let text_candidates = lexical_hits.len();
                let vector_candidates = vector_hits.len();
                let mut fused_hits = Self::fuse_rrf(lexical_hits, vector_hits);
                let fused_candidates = fused_hits.len();
                fused_hits.truncate(request.top_k);
                Ok(VantaMemorySearchDebugReport {
                    route: "hybrid".to_string(),
                    budget,
                    text_candidates,
                    vector_candidates,
                    fused_candidates,
                    top_identities: Self::debug_hit_identities(&fused_hits),
                })
            }
            (Some(text_query), false) => {
                let hits = self.lexical_search(
                    &request.namespace,
                    text_query,
                    &request.filters,
                    request.top_k,
                )?;
                Ok(VantaMemorySearchDebugReport {
                    route: "text-only".to_string(),
                    budget: request.top_k,
                    text_candidates: hits.len(),
                    vector_candidates: 0,
                    fused_candidates: hits.len(),
                    top_identities: Self::debug_hit_identities(&hits),
                })
            }
            (None, true) => {
                let hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                )?;
                Ok(VantaMemorySearchDebugReport {
                    route: "vector-only".to_string(),
                    budget: request.top_k,
                    text_candidates: 0,
                    vector_candidates: hits.len(),
                    fused_candidates: hits.len(),
                    top_identities: Self::debug_hit_identities(&hits),
                })
            }
            (None, false) => Ok(VantaMemorySearchDebugReport {
                route: "empty".to_string(),
                budget: 0,
                text_candidates: 0,
                vector_candidates: 0,
                fused_candidates: 0,
                top_identities: Vec::new(),
            }),
        }
    }

    pub fn explain_memory_search(
        &self,
        request: VantaMemorySearchRequest,
    ) -> Result<VantaSearchExplanation> {
        validate_namespace(&request.namespace)?;
        validate_metadata(&request.filters)?;

        let text_query = request
            .text_query
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty());
        let has_vector = !request.query_vector.is_empty();
        if request.top_k == 0 {
            return Ok(VantaSearchExplanation {
                route: "empty".to_string(),
                hits: Vec::new(),
                fusion_report: None,
            });
        }

        let engine = self.engine_handle()?;
        #[allow(clippy::type_complexity)]
        let (route, hits, text_ranks, vector_ranks, fusion_report): (
            String,
            Vec<VantaMemorySearchHit>,
            std::collections::BTreeMap<(String, String), usize>,
            std::collections::BTreeMap<(String, String), usize>,
            Option<VantaHybridFusionReport>,
        ) = match (text_query, has_vector) {
            (Some(text_query), true) => {
                let budget = Self::hybrid_candidate_budget(request.top_k);
                let lexical_hits =
                    self.lexical_search(&request.namespace, text_query, &request.filters, budget)?;
                let vector_hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    budget,
                    request.distance_metric,
                )?;
                let text_ranks = Self::debug_rank_map(&lexical_hits);
                let vector_ranks = Self::debug_rank_map(&vector_hits);
                let (mut hits, report) =
                    crate::planner::fuse_rrf_with_report(lexical_hits, vector_hits);
                hits.truncate(request.top_k);
                (
                    "hybrid".to_string(),
                    hits,
                    text_ranks,
                    vector_ranks,
                    Some(report),
                )
            }
            (Some(text_query), false) => {
                let hits = self.lexical_search(
                    &request.namespace,
                    text_query,
                    &request.filters,
                    request.top_k,
                )?;
                let text_ranks = Self::debug_rank_map(&hits);
                (
                    "text-only".to_string(),
                    hits,
                    text_ranks,
                    BTreeMap::new(),
                    None,
                )
            }
            (None, true) => {
                let hits = self.vector_memory_search(
                    &request.namespace,
                    &request.query_vector,
                    &request.filters,
                    request.top_k,
                    request.distance_metric,
                )?;
                let vector_ranks = Self::debug_rank_map(&hits);
                (
                    "vector-only".to_string(),
                    hits,
                    BTreeMap::new(),
                    vector_ranks,
                    None,
                )
            }
            (None, false) => {
                return Ok(VantaSearchExplanation {
                    route: "empty".to_string(),
                    hits: Vec::new(),
                    fusion_report: None,
                });
            }
        };

        let explained_hits = hits
            .into_iter()
            .map(|hit| {
                Self::debug_explain_hit(&engine, hit, text_query, &text_ranks, &vector_ranks)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(VantaSearchExplanation {
            route,
            hits: explained_hits,
            fusion_report,
        })
    }

    #[cfg(debug_assertions)]
    fn debug_hit_identities(hits: &[VantaMemorySearchHit]) -> Vec<String> {
        hits.iter()
            .map(|hit| format!("{}\0{}", hit.record.namespace, hit.record.key))
            .collect()
    }

    fn debug_rank_map(hits: &[VantaMemorySearchHit]) -> BTreeMap<(String, String), usize> {
        hits.iter()
            .enumerate()
            .map(|(index, hit)| {
                (
                    (hit.record.namespace.clone(), hit.record.key.clone()),
                    index + 1,
                )
            })
            .collect()
    }

    fn debug_explain_hit(
        engine: &StorageEngine,
        hit: VantaMemorySearchHit,
        text_query: Option<&str>,
        text_ranks: &BTreeMap<(String, String), usize>,
        vector_ranks: &BTreeMap<(String, String), usize>,
    ) -> Result<VantaSearchExplanationHit> {
        let identity_tuple = (hit.record.namespace.clone(), hit.record.key.clone());
        let identity = format!("{}\0{}", hit.record.namespace, hit.record.key);
        let bm25_terms = if let Some(text_query) = text_query {
            Self::debug_bm25_terms_for_record(engine, &hit.record, text_query)?
        } else {
            Vec::new()
        };
        let matched_tokens = bm25_terms
            .iter()
            .map(|term| term.token.clone())
            .collect::<Vec<_>>();
        let matched_phrases = if let Some(text_query) = text_query {
            Self::debug_matched_phrases_for_record(engine, &hit.record, text_query)?
        } else {
            Vec::new()
        };
        let snippet = text_query.and_then(|query| Self::debug_snippet(&hit.record.payload, query));

        Ok(VantaSearchExplanationHit {
            identity,
            score: hit.score,
            snippet,
            matched_tokens,
            matched_phrases,
            bm25_terms,
            rrf_text_rank: text_ranks.get(&identity_tuple).copied(),
            rrf_vector_rank: vector_ranks.get(&identity_tuple).copied(),
        })
    }

    fn debug_bm25_terms_for_record(
        engine: &StorageEngine,
        record: &VantaMemoryRecord,
        text_query: &str,
    ) -> Result<Vec<VantaBm25TermContribution>> {
        let query_plan = crate::text_index::query_plan(text_query);
        if query_plan.terms.is_empty() {
            return Ok(Vec::new());
        }
        let Some(namespace_stats) = Self::load_text_namespace_stats(engine, &record.namespace)?
        else {
            return Ok(Vec::new());
        };
        let Some(doc_stats) = Self::load_text_doc_stats(engine, &record.namespace, &record.key)?
        else {
            return Ok(Vec::new());
        };
        if namespace_stats.doc_count == 0 {
            return Ok(Vec::new());
        }

        let doc_count = namespace_stats.doc_count as f32;
        let avg_doc_len = if namespace_stats.total_doc_len == 0 {
            1.0
        } else {
            namespace_stats.total_doc_len as f32 / doc_count
        };
        let doc_len = doc_stats.doc_len as f32;
        let mut terms = Vec::new();

        for token in query_plan.terms {
            let Some(term_stats) = Self::load_text_term_stats(engine, &record.namespace, &token)?
            else {
                continue;
            };
            let Some(posting_value) = engine.get_from_partition(
                BackendPartition::TextIndex,
                &crate::text_index::posting_key(&record.namespace, &token, &record.key),
            )?
            else {
                continue;
            };
            let posting = crate::text_index::decode_posting(&posting_value)?;
            let df = term_stats.df as f32;
            let idf = (1.0 + ((doc_count - df + 0.5) / (df + 0.5))).ln();
            let tf = posting.tf as f32;
            let denominator = tf
                + crate::text_index::BM25_K1
                    * (1.0 - crate::text_index::BM25_B
                        + crate::text_index::BM25_B * (doc_len / avg_doc_len));
            let contribution = idf * ((tf * (crate::text_index::BM25_K1 + 1.0)) / denominator);
            terms.push(VantaBm25TermContribution {
                token,
                tf: posting.tf,
                df: term_stats.df,
                doc_len: doc_stats.doc_len,
                contribution,
            });
        }

        Ok(terms)
    }

    fn debug_matched_phrases_for_record(
        engine: &StorageEngine,
        record: &VantaMemoryRecord,
        text_query: &str,
    ) -> Result<Vec<String>> {
        let query_plan = crate::text_index::query_plan(text_query);
        if query_plan.phrases.is_empty() {
            return Ok(Vec::new());
        }

        let mut term_positions = BTreeMap::new();
        for token in query_plan.terms {
            if let Some(value) = engine.get_from_partition(
                BackendPartition::TextIndex,
                &crate::text_index::posting_key(&record.namespace, &token, &record.key),
            )? {
                let posting = crate::text_index::decode_posting(&value)?;
                term_positions.insert(token, posting.positions);
            }
        }

        Ok(query_plan
            .phrases
            .into_iter()
            .filter(|phrase| Self::text_positions_match_phrase(&term_positions, phrase))
            .map(|phrase| phrase.join(" "))
            .collect())
    }

    fn debug_snippet(payload: &str, text_query: &str) -> Option<String> {
        Self::generate_snippet_with_highlighting(payload, text_query, false)
    }

    /// Generate a text snippet with optional highlighting of matched terms.
    ///
    /// # Arguments
    /// * `payload` - The original text content
    /// * `text_query` - The search query used to find matches
    /// * `with_highlighting` - Whether to add HTML highlighting to matched terms
    ///
    /// # Returns
    /// * `Option<String>` - The snippet with optional highlighting, or None if no match found
    fn generate_snippet_with_highlighting(
        payload: &str,
        text_query: &str,
        with_highlighting: bool,
    ) -> Option<String> {
        let query_plan = crate::text_index::query_plan(text_query);
        let first_token = query_plan.terms.iter().next()?;

        if payload.len() <= 120 {
            if with_highlighting {
                return Some(Self::highlight_terms(payload, &query_plan.terms));
            }
            return Some(payload.to_string());
        }

        let lower_payload = payload.to_ascii_lowercase();
        let match_at = lower_payload.find(first_token).unwrap_or(0);
        let mut start = match_at.saturating_sub(48);
        let mut end = match_at
            .saturating_add(first_token.len())
            .saturating_add(72)
            .min(payload.len());
        while start > 0 && !payload.is_char_boundary(start) {
            start -= 1;
        }
        while end < payload.len() && !payload.is_char_boundary(end) {
            end += 1;
        }

        let snippet_text = payload[start..end].trim();

        if with_highlighting {
            let highlighted = Self::highlight_terms(snippet_text, &query_plan.terms);
            let mut snippet = String::new();
            if start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(&highlighted);
            if end < payload.len() {
                snippet.push_str("...");
            }
            Some(snippet)
        } else {
            let mut snippet = String::new();
            if start > 0 {
                snippet.push_str("...");
            }
            snippet.push_str(snippet_text);
            if end < payload.len() {
                snippet.push_str("...");
            }
            Some(snippet)
        }
    }

    /// Add HTML highlighting to matched terms in text.
    ///
    /// # Arguments
    /// * `text` - The text to highlight
    /// * `terms` - The terms to highlight
    ///
    /// # Returns
    /// * `String` - The text with HTML highlighting markers
    fn highlight_terms(text: &str, terms: &std::collections::BTreeSet<String>) -> String {
        let mut result = String::new();
        let mut i = 0;
        let chars: Vec<char> = text.chars().collect();

        while i < chars.len() {
            let mut matched = false;

            for term in terms {
                let term_chars: Vec<char> = term.chars().collect();
                if i + term_chars.len() <= chars.len() {
                    let slice: String = chars[i..i + term_chars.len()].iter().collect();
                    if slice.eq_ignore_ascii_case(term) {
                        result.push_str("<strong>");
                        result.push_str(&slice);
                        result.push_str("</strong>");
                        i += term_chars.len();
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                result.push(chars[i]);
                i += 1;
            }
        }

        result
    }

    pub fn graph_bfs(&self, roots: &[u64], max_depth: usize) -> Result<Vec<u64>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.bfs_traverse(roots, max_depth)
    }

    pub fn graph_dfs(&self, roots: &[u64], max_depth: usize) -> Result<Vec<u64>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.dfs_traverse(roots, max_depth)
    }

    pub fn graph_topological_sort(&self, roots: &[u64]) -> Result<Vec<u64>> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.topological_sort(roots)
    }

    pub fn graph_is_dag(&self, roots: &[u64]) -> Result<bool> {
        let engine = self.engine_handle()?;
        let traverser = crate::graph::GraphTraverser::new(&engine);
        traverser.is_dag(roots)
    }
}

impl From<IndexRebuildReport> for VantaIndexRebuildReport {
    fn from(report: IndexRebuildReport) -> Self {
        Self {
            scanned_nodes: report.scanned_nodes,
            indexed_vectors: report.indexed_vectors,
            skipped_tombstones: report.skipped_tombstones,
            duration_ms: report.duration_ms,
            derived_rebuild_ms: 0,
            index_path: report.index_path.to_string_lossy().into_owned(),
            success: report.success,
        }
    }
}

impl From<crate::metrics::OperationalMetricsSnapshot> for VantaOperationalMetrics {
    fn from(metrics: crate::metrics::OperationalMetricsSnapshot) -> Self {
        Self {
            startup_ms: metrics.startup_ms,
            wal_replay_ms: metrics.wal_replay_ms,
            wal_records_replayed: metrics.wal_records_replayed,
            ann_rebuild_ms: metrics.ann_rebuild_ms,
            ann_rebuild_scanned_nodes: metrics.ann_rebuild_scanned_nodes,
            derived_rebuild_ms: metrics.derived_rebuild_ms,
            text_index_rebuild_ms: metrics.text_index_rebuild_ms,
            text_postings_written: metrics.text_postings_written,
            text_index_repairs: metrics.text_index_repairs,
            text_lexical_queries: metrics.text_lexical_queries,
            text_lexical_query_ms: metrics.text_lexical_query_ms,
            text_candidates_scored: metrics.text_candidates_scored,
            text_consistency_audits: metrics.text_consistency_audits,
            text_consistency_audit_failures: metrics.text_consistency_audit_failures,
            hybrid_query_ms: metrics.hybrid_query_ms,
            hybrid_candidates_fused: metrics.hybrid_candidates_fused,
            planner_hybrid_queries: metrics.planner_hybrid_queries,
            planner_text_only_queries: metrics.planner_text_only_queries,
            planner_vector_only_queries: metrics.planner_vector_only_queries,
            records_exported: metrics.records_exported,
            records_imported: metrics.records_imported,
            import_errors: metrics.import_errors,
            derived_prefix_scans: metrics.derived_prefix_scans,
            derived_full_scan_fallbacks: metrics.derived_full_scan_fallbacks,
            process_rss_bytes: metrics.memory.process_rss_bytes,
            process_virtual_bytes: metrics.memory.process_virtual_bytes,
            hnsw_nodes_count: metrics.memory.hnsw_nodes_count,
            hnsw_logical_bytes: metrics.memory.hnsw_logical_bytes,
            mmap_resident_bytes: metrics.memory.mmap_resident_bytes,
            volatile_cache_entries: metrics.memory.volatile_cache_entries,
            volatile_cache_cap_bytes: metrics.memory.volatile_cache_cap_bytes,
        }
    }
}

impl From<VantaValue> for FieldValue {
    fn from(value: VantaValue) -> Self {
        match value {
            VantaValue::String(value) => FieldValue::String(value),
            VantaValue::Int(value) => FieldValue::Int(value),
            VantaValue::Float(value) => FieldValue::Float(value),
            VantaValue::Bool(value) => FieldValue::Bool(value),
            VantaValue::DateTime(value) => FieldValue::DateTime(value),
            VantaValue::ListString(value) => FieldValue::ListString(value),
            VantaValue::ListInt(value) => FieldValue::ListInt(value),
            VantaValue::ListFloat(value) => FieldValue::ListFloat(value),
            VantaValue::ListBool(value) => FieldValue::ListBool(value),
            VantaValue::ListDateTime(value) => FieldValue::ListDateTime(value),
            VantaValue::Null => FieldValue::Null,
        }
    }
}

impl From<FieldValue> for VantaValue {
    fn from(value: FieldValue) -> Self {
        match value {
            FieldValue::String(value) => VantaValue::String(value),
            FieldValue::Int(value) => VantaValue::Int(value),
            FieldValue::Float(value) => VantaValue::Float(value),
            FieldValue::Bool(value) => VantaValue::Bool(value),
            FieldValue::DateTime(value) => VantaValue::DateTime(value),
            FieldValue::ListString(value) => VantaValue::ListString(value),
            FieldValue::ListInt(value) => VantaValue::ListInt(value),
            FieldValue::ListFloat(value) => VantaValue::ListFloat(value),
            FieldValue::ListBool(value) => VantaValue::ListBool(value),
            FieldValue::ListDateTime(value) => VantaValue::ListDateTime(value),
            FieldValue::Null => VantaValue::Null,
        }
    }
}

impl From<ExecutionResult> for VantaQueryResult {
    fn from(result: ExecutionResult) -> Self {
        match result {
            ExecutionResult::Read(nodes) => {
                VantaQueryResult::Read(nodes.into_iter().map(Into::into).collect())
            }
            ExecutionResult::Write {
                affected_nodes,
                message,
                node_id,
            } => VantaQueryResult::Write {
                affected_nodes,
                message,
                node_id,
            },
            ExecutionResult::StaleContext(node_id) => VantaQueryResult::StaleContext { node_id },
        }
    }
}

impl From<UnifiedNode> for VantaNodeRecord {
    fn from(node: UnifiedNode) -> Self {
        let is_alive = node.is_alive();
        let (vector, vector_dimensions) = match node.vector {
            VectorRepresentations::Full(vector) => {
                let dims = vector.len();
                (Some(vector), dims)
            }
            VectorRepresentations::None => (None, 0),
            other => (None, other.dimensions()),
        };

        let tier = match node.tier {
            crate::node::NodeTier::Hot => VantaStorageTier::Hot,
            crate::node::NodeTier::Cold => VantaStorageTier::Cold,
        };

        let fields = node
            .relational
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect();

        let edges = node
            .edges
            .into_iter()
            .map(|edge| VantaEdgeRecord {
                target: edge.target,
                label: edge.label,
                weight: edge.weight,
            })
            .collect();

        Self {
            id: node.id,
            fields,
            vector,
            vector_dimensions,
            edges,
            confidence_score: node.confidence_score,
            importance: node.importance,
            hits: node.hits,
            last_accessed: node.last_accessed,
            epoch: node.epoch,
            tier,
            is_alive,
        }
    }
}
