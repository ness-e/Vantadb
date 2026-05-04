use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use crate::executor::{ExecutionResult, Executor};
use crate::hardware::{HardwareCapabilities, HardwareProfile};
use crate::index::cosine_sim_f32;
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use crate::storage::{BackendKind, EngineConfig, IndexRebuildReport, StorageEngine};
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
const EXPORT_SCHEMA_VERSION: u32 = 1;
const DERIVED_INDEX_SCHEMA_VERSION: u32 = 1;
const DERIVED_INDEX_STATE_KEY: &[u8] = b"derived_index_state";
const TEXT_INDEX_STATE_KEY: &[u8] = b"text_index_state";

/// Stable open options for embedded SDK consumers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VantaOpenOptions {
    pub memory_limit_bytes: Option<u64>,
    pub read_only: bool,
}

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
    Null,
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
}

/// Stable vector search hit for persistent memory records.
#[derive(Debug, Clone, PartialEq)]
pub struct VantaMemorySearchHit {
    pub record: VantaMemoryRecord,
    pub score: f32,
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
    pub records_exported: u64,
    pub records_imported: u64,
    pub import_errors: u64,
    pub derived_prefix_scans: u64,
    pub derived_full_scan_fallbacks: u64,
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
    key_format: String,
    rebuilt_at_ms: u64,
    record_count: u64,
    posting_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextIndexRebuildReport {
    record_count: u64,
    posting_entries: u64,
    duration_ms: u64,
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
    options: VantaOpenOptions,
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

fn encoded_scalar_value(value: &VantaValue) -> Vec<u8> {
    match value {
        VantaValue::String(value) => {
            let mut encoded = b"s:".to_vec();
            encoded.extend_from_slice(value.as_bytes());
            encoded
        }
        VantaValue::Int(value) => format!("i:{value}").into_bytes(),
        VantaValue::Float(value) => format!("f:{:016x}", value.to_bits()).into_bytes(),
        VantaValue::Bool(value) => {
            if *value {
                b"b:1".to_vec()
            } else {
                b"b:0".to_vec()
            }
        }
        VantaValue::Null => b"n:".to_vec(),
    }
}

fn payload_index_prefix(namespace: &str, field: &str, value: &VantaValue) -> Vec<u8> {
    let encoded = encoded_scalar_value(value);
    let mut prefix = Vec::with_capacity(namespace.len() + field.len() + encoded.len() + 3);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix.extend_from_slice(field.as_bytes());
    prefix.push(0);
    prefix.extend_from_slice(&encoded);
    prefix.push(0);
    prefix
}

fn payload_index_key(namespace: &str, field: &str, value: &VantaValue, key: &str) -> Vec<u8> {
    let mut index_key = payload_index_prefix(namespace, field, value);
    index_key.extend_from_slice(key.as_bytes());
    index_key
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

    fields.remove(FIELD_NAMESPACE);
    fields.remove(FIELD_KEY);
    fields.remove(FIELD_PAYLOAD);
    fields.remove(FIELD_CREATED_AT_MS);
    fields.remove(FIELD_UPDATED_AT_MS);
    fields.remove(FIELD_VERSION);

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
    })
}

fn matches_memory_filters(record: &VantaMemoryRecord, filters: &VantaMemoryMetadata) -> bool {
    filters
        .iter()
        .all(|(key, expected)| record.metadata.get(key) == Some(expected))
}

impl VantaEmbedded {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_options(path, VantaOpenOptions::default())
    }

    pub fn open_with_options(path: impl AsRef<Path>, options: VantaOpenOptions) -> Result<Self> {
        let config = EngineConfig {
            memory_limit: options.memory_limit_bytes,
            force_mmap: false,
            read_only: options.read_only,
            backend_kind: BackendKind::Fjall,
        };
        let path = path.as_ref().to_string_lossy().into_owned();
        let engine = StorageEngine::open_with_config(&path, Some(config))?;
        let embedded = Self {
            engine: Arc::new(RwLock::new(Some(Arc::new(engine)))),
            options,
        };
        if !embedded.options.read_only {
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

    fn fresh_text_index_state(record_count: u64, posting_entries: u64) -> TextIndexState {
        let spec = crate::text_index::TextIndexSpec::default();
        TextIndexState {
            schema_version: spec.schema_version,
            tokenizer: spec.tokenizer.to_string(),
            key_format: spec.key_format.to_string(),
            rebuilt_at_ms: now_ms(),
            record_count,
            posting_entries,
        }
    }

    fn text_index_state_matches_spec(state: &TextIndexState) -> bool {
        let spec = crate::text_index::TextIndexSpec::default();
        state.schema_version == spec.schema_version
            && state.tokenizer == spec.tokenizer
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

    fn expected_text_index_counts(engine: &StorageEngine) -> Result<(u64, u64)> {
        let mut record_count = 0u64;
        let mut posting_entries = 0u64;

        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node(node) {
                record_count += 1;
                posting_entries += crate::text_index::posting_count(&record.payload);
            }
        }

        Ok((record_count, posting_entries))
    }

    fn current_text_index_count(engine: &StorageEngine) -> Result<u64> {
        Ok(engine.scan_partition(BackendPartition::TextIndex)?.len() as u64)
    }

    fn current_derived_index_counts(engine: &StorageEngine) -> Result<(u64, u64)> {
        let namespace_entries = engine
            .scan_partition(BackendPartition::NamespaceIndex)?
            .len() as u64;
        let payload_entries = engine.scan_partition(BackendPartition::PayloadIndex)?.len() as u64;
        Ok((namespace_entries, payload_entries))
    }

    fn derived_put_ops(record: &VantaMemoryRecord) -> Vec<BackendWriteOp> {
        let mut ops = Vec::new();
        ops.push(BackendWriteOp::Put {
            partition: BackendPartition::NamespaceIndex,
            key: namespace_index_key(&record.namespace, &record.key),
            value: node_id_bytes(record.node_id),
        });

        for (field, value) in &record.metadata {
            ops.push(BackendWriteOp::Put {
                partition: BackendPartition::PayloadIndex,
                key: payload_index_key(&record.namespace, field, value, &record.key),
                value: node_id_bytes(record.node_id),
            });
        }

        ops
    }

    fn derived_delete_ops(record: &VantaMemoryRecord) -> Vec<BackendWriteOp> {
        let mut ops = Vec::new();
        ops.push(BackendWriteOp::Delete {
            partition: BackendPartition::NamespaceIndex,
            key: namespace_index_key(&record.namespace, &record.key),
        });

        for (field, value) in &record.metadata {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::PayloadIndex,
                key: payload_index_key(&record.namespace, field, value, &record.key),
            });
        }

        ops
    }

    fn replace_derived_indexes(
        &self,
        engine: &StorageEngine,
        previous: Option<&VantaMemoryRecord>,
        current: Option<&VantaMemoryRecord>,
    ) -> Result<()> {
        let mut ops = Vec::new();
        if let Some(previous) = previous {
            ops.extend(Self::derived_delete_ops(previous));
            ops.extend(crate::text_index::posting_delete_ops(
                &previous.namespace,
                &previous.key,
                &previous.payload,
            ));
        }
        let text_postings_written = current
            .map(|record| crate::text_index::posting_count(&record.payload))
            .unwrap_or(0);
        if let Some(current) = current {
            ops.extend(Self::derived_put_ops(current));
            ops.extend(crate::text_index::posting_put_ops(
                &current.namespace,
                &current.key,
                &current.payload,
                current.node_id,
            ));
        }
        if ops.is_empty() {
            return Ok(());
        }
        engine.write_backend_batch(ops)?;
        Self::adjust_derived_index_state_after_replace(engine, previous, current)?;
        Self::adjust_text_index_state_after_replace(engine, previous, current)?;
        crate::metrics::record_text_postings_written(text_postings_written);
        Ok(())
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
            }
            (Some(previous), None) => {
                state.record_count = state.record_count.saturating_sub(1);
                state.posting_entries = state
                    .posting_entries
                    .saturating_sub(crate::text_index::posting_count(&previous.payload));
            }
            (Some(previous), Some(current)) => {
                state.posting_entries = state
                    .posting_entries
                    .saturating_sub(crate::text_index::posting_count(&previous.payload))
                    .saturating_add(crate::text_index::posting_count(&current.payload));
            }
            (None, None) => {}
        }

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

        let (canonical_records, expected_postings) = Self::expected_text_index_counts(&engine)?;
        let current_postings = Self::current_text_index_count(&engine)?;

        let has_state = state.is_some();
        let needs_rebuild = match &state {
            Some(state) => {
                !Self::text_index_state_matches_spec(state)
                    || state.record_count != canonical_records
                    || state.posting_entries != current_postings
                    || state.posting_entries != expected_postings
                    || current_postings != expected_postings
            }
            None => canonical_records > 0 || current_postings > 0,
        };

        if needs_rebuild {
            crate::metrics::record_text_index_repair();
            self.rebuild_text_index_with_report()?;
        } else if !has_state {
            Self::write_text_index_state(
                &engine,
                &Self::fresh_text_index_state(canonical_records, expected_postings),
            )?;
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
                ops.extend(Self::derived_put_ops(&record));
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
        let mut ops = Vec::new();
        let mut record_count = 0u64;
        let mut posting_entries = 0u64;

        for (key, _value) in engine.scan_partition(BackendPartition::TextIndex)? {
            ops.push(BackendWriteOp::Delete {
                partition: BackendPartition::TextIndex,
                key,
            });
        }

        for node in engine.scan_nodes()? {
            if let Some(record) = memory_record_from_node(node) {
                record_count += 1;
                let posting_ops = crate::text_index::posting_put_ops(
                    &record.namespace,
                    &record.key,
                    &record.payload,
                    record.node_id,
                );
                posting_entries += posting_ops.len() as u64;
                ops.extend(posting_ops);
            }
        }

        if !ops.is_empty() {
            engine.write_backend_batch(ops)?;
        }

        Self::write_text_index_state(
            &engine,
            &Self::fresh_text_index_state(record_count, posting_entries),
        )?;

        let report = TextIndexRebuildReport {
            record_count,
            posting_entries,
            duration_ms: started.elapsed().as_millis() as u64,
        };
        crate::metrics::record_text_index_rebuild(report.duration_ms, report.posting_entries);
        Ok(report)
    }

    fn rebuild_text_index(&self) -> Result<()> {
        self.rebuild_text_index_with_report().map(|_| ())
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
        let prefix = payload_index_prefix(namespace, field, value);
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

        let record = VantaMemoryRecord {
            namespace: input.namespace,
            key: input.key,
            payload: input.payload,
            metadata: input.metadata,
            created_at_ms,
            updated_at_ms: timestamp,
            version,
            node_id,
            vector: input.vector.filter(|vector| !vector.is_empty()),
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
            _ => Err(VantaError::Execution(format!(
                "node id collision for namespace='{}' key='{}'",
                namespace, key
            ))),
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

        if request
            .text_query
            .as_deref()
            .map(str::trim)
            .is_some_and(|text| !text.is_empty())
        {
            return Err(VantaError::Execution(
                "text_query requires BM25/RRF, which is deferred for this MVP block".to_string(),
            ));
        }

        if request.query_vector.is_empty() || request.top_k == 0 {
            return Ok(Vec::new());
        }

        let mut hits = Vec::new();

        for record in self.records_for_namespace(&request.namespace, &request.filters)? {
            let Some(vector) = record.vector.as_ref() else {
                continue;
            };
            if vector.len() != request.query_vector.len() {
                continue;
            }

            hits.push(VantaMemorySearchHit {
                score: cosine_sim_f32(&request.query_vector, vector),
                record,
            });
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(request.top_k);
        Ok(hits)
    }

    pub fn rebuild_index(&self) -> Result<VantaIndexRebuildReport> {
        let report = self.engine_handle()?.rebuild_vector_index()?;
        let derived = self.rebuild_derived_indexes_with_report()?;
        self.rebuild_text_index_with_report()?;
        let mut report: VantaIndexRebuildReport = report.into();
        report.derived_rebuild_ms = derived.duration_ms;
        Ok(report)
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

    pub fn operational_metrics(&self) -> VantaOperationalMetrics {
        crate::metrics::operational_metrics_snapshot().into()
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
    pub fn debug_text_index_posting_keys_for_tests(&self) -> Result<Vec<Vec<u8>>> {
        let engine = self.engine_handle()?;
        let mut keys: Vec<Vec<u8>> = engine
            .scan_partition(BackendPartition::TextIndex)?
            .into_iter()
            .map(|(key, _value)| key)
            .collect();
        keys.sort();
        Ok(keys)
    }

    pub fn search_vector(&self, vector: &[f32], top_k: usize) -> Result<Vec<VantaSearchHit>> {
        let engine = self.engine_handle()?;
        let index = engine.hnsw.read();
        let hits = index.search_nearest(vector, None, None, 0, top_k, None);
        Ok(hits
            .into_iter()
            .map(|(node_id, distance)| VantaSearchHit { node_id, distance })
            .collect())
    }

    pub fn query(&self, query: &str) -> Result<VantaQueryResult> {
        let engine = self.engine_handle()?;
        let executor = Executor::new(engine.as_ref());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| VantaError::Execution(err.to_string()))?;
        let result = runtime.block_on(async { executor.execute_hybrid(query).await })?;
        Ok(result.into())
    }

    pub fn flush(&self) -> Result<()> {
        self.engine_handle()?.flush()
    }

    pub fn close(&self) -> Result<()> {
        if self.options.read_only {
            return Ok(());
        }

        if let Some(engine) = self.engine.write().take() {
            engine.flush()?;
        }

        Ok(())
    }

    pub fn add_edge(
        &self,
        source_id: u64,
        target_id: u64,
        label: impl Into<String>,
        weight: Option<f32>,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut node = self
            .engine_handle()?
            .get(source_id)?
            .ok_or(VantaError::NodeNotFound(source_id))?;

        match weight {
            Some(weight) => node.add_weighted_edge(target_id, label, weight),
            None => node.add_edge(target_id, label),
        }

        engine.insert(&node)
    }

    pub fn capabilities(&self) -> VantaCapabilities {
        let profile = match HardwareCapabilities::global().profile {
            HardwareProfile::Enterprise => VantaRuntimeProfile::Enterprise,
            HardwareProfile::Performance => VantaRuntimeProfile::Performance,
            HardwareProfile::LowResource => VantaRuntimeProfile::LowResource,
        };

        VantaCapabilities {
            runtime_profile: profile,
            persistence: true,
            vector_search: true,
            iql_queries: true,
            read_only: self.options.read_only,
        }
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
            records_exported: metrics.records_exported,
            records_imported: metrics.records_imported,
            import_errors: metrics.import_errors,
            derived_prefix_scans: metrics.derived_prefix_scans,
            derived_full_scan_fallbacks: metrics.derived_full_scan_fallbacks,
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
