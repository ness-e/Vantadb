use super::builder::VantaEmbedded;
use super::types::*;
use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use crate::executor::ExecutionResult;
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use crate::storage::StorageEngine;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::hash::Hasher;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use tracing;
use twox_hash::XxHash64;
use web_time::Instant;
use web_time::{SystemTime, UNIX_EPOCH};

const RESERVED_PREFIX: &str = "__vanta_";
pub const FIELD_NAMESPACE: &str = "__vanta_namespace";
pub const FIELD_KEY: &str = "__vanta_key";
pub const FIELD_PAYLOAD: &str = "__vanta_payload";
pub const FIELD_CREATED_AT_MS: &str = "__vanta_created_at_ms";
pub const FIELD_UPDATED_AT_MS: &str = "__vanta_updated_at_ms";
pub const FIELD_VERSION: &str = "__vanta_version";
pub const FIELD_EXPIRES_AT_MS: &str = "__vanta_expires_at_ms";
const EXPORT_SCHEMA_VERSION: u32 = 1;
const DERIVED_INDEX_SCHEMA_VERSION: u32 = 1;
pub(crate) const DERIVED_INDEX_STATE_KEY: &[u8] = b"derived_index_state";
pub(crate) const TEXT_INDEX_STATE_KEY: &[u8] = b"text_index_state";

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub(crate) fn memory_node_id(namespace: &str, key: &str) -> u64 {
    let mut hasher = XxHash64::default();
    hasher.write(namespace.as_bytes());
    hasher.write(&[0]);
    hasher.write(key.as_bytes());
    hasher.finish()
}

pub(crate) fn validate_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty() {
        return Err(VantaError::ValidationError {
            field: "namespace".into(),
            reason: "namespace must not be empty".into(),
        });
    }
    if namespace.len() > 128 {
        return Err(VantaError::ValidationError {
            field: "namespace".into(),
            reason: "namespace must be at most 128 bytes".into(),
        });
    }
    if !namespace
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'/' | b'-'))
    {
        return Err(VantaError::ValidationError {
            field: "namespace".into(),
            reason: "namespace may contain only A-Z, a-z, 0-9, '.', '_', '/', '-'".into(),
        });
    }
    Ok(())
}

pub(crate) fn validate_key(key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(VantaError::ValidationError {
            field: "key".into(),
            reason: "key must not be empty".into(),
        });
    }
    if key.len() > 512 {
        return Err(VantaError::ValidationError {
            field: "key".into(),
            reason: "key must be at most 512 bytes".into(),
        });
    }
    if key.as_bytes().contains(&0) {
        return Err(VantaError::ValidationError {
            field: "key".into(),
            reason: "key must not contain NUL bytes".into(),
        });
    }
    Ok(())
}

pub(crate) fn validate_metadata(metadata: &VantaMemoryMetadata) -> Result<()> {
    if let Some(key) = metadata.keys().find(|key| key.starts_with(RESERVED_PREFIX)) {
        return Err(VantaError::ValidationError {
            field: "metadata".into(),
            reason: format!("metadata key '{}' is reserved for VantaDB internals", key),
        });
    }
    if let Some(key) = metadata.keys().find(|key| key.as_bytes().contains(&0)) {
        return Err(VantaError::ValidationError {
            field: "metadata".into(),
            reason: format!("metadata key '{}' must not contain NUL bytes", key),
        });
    }
    Ok(())
}

pub(crate) fn namespace_index_key(namespace: &str, key: &str) -> Vec<u8> {
    let mut index_key = Vec::with_capacity(namespace.len() + 1 + key.len());
    index_key.extend_from_slice(namespace.as_bytes());
    index_key.push(0);
    index_key.extend_from_slice(key.as_bytes());
    index_key
}

pub(crate) fn namespace_index_prefix(namespace: &str) -> Vec<u8> {
    let mut prefix = Vec::with_capacity(namespace.len() + 1);
    prefix.extend_from_slice(namespace.as_bytes());
    prefix.push(0);
    prefix
}

pub(crate) fn encoded_scalar_value(value: &VantaValue) -> Result<Vec<u8>> {
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
        | VantaValue::ListDateTime(_) => Err(VantaError::ValidationError {
            field: "value".into(),
            reason: "Cannot encode list value as scalar index key".into(),
        }),
        VantaValue::Null => Ok(b"n:".to_vec()),
    }
}

pub(crate) fn payload_index_prefix(
    namespace: &str,
    field: &str,
    value: &VantaValue,
) -> Result<Vec<u8>> {
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

pub(crate) fn payload_index_key(
    namespace: &str,
    field: &str,
    value: &VantaValue,
    key: &str,
) -> Result<Vec<u8>> {
    let mut index_key = payload_index_prefix(namespace, field, value)?;
    index_key.extend_from_slice(key.as_bytes());
    Ok(index_key)
}

pub(crate) fn node_id_bytes(node_id: u64) -> Vec<u8> {
    node_id.to_le_bytes().to_vec()
}

pub(crate) fn decode_node_id(bytes: &[u8]) -> Option<u64> {
    if bytes.len() != std::mem::size_of::<u64>() {
        return None;
    }
    let mut id = [0u8; 8];
    id.copy_from_slice(bytes);
    Some(u64::from_le_bytes(id))
}

pub(crate) fn get_string_field(fields: &VantaFields, key: &str) -> Option<String> {
    match fields.get(key) {
        Some(VantaValue::String(value)) => Some(value.clone()),
        _ => None,
    }
}

pub(crate) fn get_u64_field(fields: &VantaFields, key: &str) -> Option<u64> {
    match fields.get(key) {
        Some(VantaValue::Int(value)) if *value >= 0 => Some(*value as u64),
        _ => None,
    }
}

pub(crate) fn memory_record_from_node(node: UnifiedNode) -> Option<VantaMemoryRecord> {
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

pub(crate) fn memory_record_to_node_owned(
    mut record: VantaMemoryRecord,
) -> (UnifiedNode, VantaMemoryRecord) {
    let namespace = std::mem::take(&mut record.namespace);
    let key = std::mem::take(&mut record.key);
    let payload = std::mem::take(&mut record.payload);
    let metadata = std::mem::take(&mut record.metadata);
    let vector = record.vector.take();

    let mut node = UnifiedNode::new(record.node_id);
    node.set_field(FIELD_NAMESPACE, FieldValue::String(namespace.clone()));
    node.set_field(FIELD_KEY, FieldValue::String(key.clone()));
    node.set_field(FIELD_PAYLOAD, FieldValue::String(payload.clone()));
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

    for (k, v) in metadata.clone() {
        node.set_field(k, v.into());
    }

    let vector = vector.filter(|v| !v.is_empty());
    if let Some(ref vec) = vector {
        node.vector = VectorRepresentations::Full(vec.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
    }

    record.namespace = namespace;
    record.key = key;
    record.payload = payload;
    record.metadata = metadata;
    record.vector = vector;

    (node, record)
}

pub fn export_line_from_record(record: VantaMemoryRecord) -> VantaMemoryExportLine {
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

pub(crate) fn record_from_export_line(line: VantaMemoryExportLine) -> Result<VantaMemoryRecord> {
    if line.schema_version != EXPORT_SCHEMA_VERSION {
        return Err(VantaError::ValidationError {
            field: "schema_version".into(),
            reason: format!(
                "unsupported memory export schema_version {}",
                line.schema_version
            ),
        });
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

pub(crate) fn matches_memory_filters(
    record: &VantaMemoryRecord,
    filters: &VantaMemoryMetadata,
) -> bool {
    filters
        .iter()
        .all(|(key, expected)| record.metadata.get(key) == Some(expected))
}

impl VantaEmbedded {
    pub(crate) fn ensure_indexes_current(&self) -> Result<()> {
        let engine = self.engine_handle()?;
        let nodes = engine.scan_nodes()?;

        self.ensure_derived_indexes_current_with(&engine, &nodes)?;
        self.ensure_text_index_current_with(&engine, &nodes)?;

        Ok(())
    }

    fn ensure_derived_indexes_current_with(
        &self,
        engine: &Arc<StorageEngine>,
        nodes: &[UnifiedNode],
    ) -> Result<()> {
        let state = match Self::load_derived_index_state(engine) {
            Ok(state) => state,
            Err(_) => {
                self.rebuild_derived_indexes_with_report()?;
                return Ok(());
            }
        };

        let canonical_records = Self::count_memory_records_from(nodes);
        let (namespace_entries, payload_entries) = Self::current_derived_index_counts(engine)?;

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
                engine,
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

    fn ensure_text_index_current_with(
        &self,
        engine: &Arc<StorageEngine>,
        nodes: &[UnifiedNode],
    ) -> Result<()> {
        let state = match Self::load_text_index_state(engine) {
            Ok(state) => state,
            Err(_) => {
                crate::metrics::record_text_index_repair();
                self.rebuild_text_index_with_report()?;
                return Ok(());
            }
        };

        let expected = Self::expected_text_index_counts_from(nodes);
        let current = Self::current_text_index_counts(engine)?;

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
            Self::write_text_index_state(engine, &Self::fresh_text_index_state(expected))?;
        }

        Ok(())
    }

    fn load_derived_index_state(engine: &StorageEngine) -> Result<Option<DerivedIndexState>> {
        let Some(bytes) = engine
            .get_from_partition(BackendPartition::InternalMetadata, DERIVED_INDEX_STATE_KEY)?
        else {
            return Ok(None);
        };
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
            .map(|(val, _)| Some(val))
            .map_err(|err| {
                VantaError::SerializationError(format!("derived index state decode error: {err}"))
            })
    }

    fn write_derived_index_state(engine: &StorageEngine, state: &DerivedIndexState) -> Result<()> {
        let bytes = bincode::serde::encode_to_vec(state, bincode::config::standard())
            .map_err(|err| VantaError::SerializationError(err.to_string()))?;
        engine.put_to_partition(
            BackendPartition::InternalMetadata,
            DERIVED_INDEX_STATE_KEY,
            &bytes,
        )
    }

    pub(crate) fn load_text_index_state(engine: &StorageEngine) -> Result<Option<TextIndexState>> {
        let Some(bytes) =
            engine.get_from_partition(BackendPartition::InternalMetadata, TEXT_INDEX_STATE_KEY)?
        else {
            return Ok(None);
        };
        bincode::serde::decode_from_slice(&bytes, bincode::config::standard())
            .map(|(val, _)| Some(val))
            .map_err(|err| {
                VantaError::SerializationError(format!("text index state decode error: {err}"))
            })
    }

    fn write_text_index_state(engine: &StorageEngine, state: &TextIndexState) -> Result<()> {
        let bytes = bincode::serde::encode_to_vec(state, bincode::config::standard())
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

    pub(crate) fn text_index_state_matches_spec(state: &TextIndexState) -> bool {
        let spec = crate::text_index::TextIndexSpec::default();
        state.schema_version == spec.schema_version
            && state.tokenizer == spec.tokenizer.name
            && state.tokenizer_version == spec.tokenizer.version
            && state.key_format == spec.key_format
    }

    fn count_memory_records_from(nodes: &[UnifiedNode]) -> u64 {
        let mut count = 0u64;
        for node in nodes {
            if memory_record_from_node(node.clone()).is_some() {
                count += 1;
            }
        }
        count
    }

    fn expected_text_index_counts_from(nodes: &[UnifiedNode]) -> TextIndexCounts {
        let mut counts = TextIndexCounts::default();
        let mut terms = BTreeSet::new();
        let mut namespaces = BTreeSet::new();

        for node in nodes {
            if let Some(record) = memory_record_from_node(node.clone()) {
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
        counts
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

    pub(crate) fn derived_put_ops(record: &VantaMemoryRecord) -> Result<Vec<BackendWriteOp>> {
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

    pub(crate) fn derived_delete_ops(record: &VantaMemoryRecord) -> Result<Vec<BackendWriteOp>> {
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

    pub(crate) fn load_text_term_stats(
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

    pub(crate) fn load_text_namespace_stats(
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

    pub(crate) fn load_text_doc_stats(
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

    pub(crate) fn apply_u64_delta(value: u64, delta: i64) -> u64 {
        if delta >= 0 {
            value.saturating_add(delta as u64)
        } else {
            value.saturating_sub(delta.unsigned_abs())
        }
    }

    pub(crate) fn checked_stats_value(value: i128, label: &str) -> Result<u64> {
        if value < 0 {
            return Err(VantaError::ValidationError {
                field: "stats".into(),
                reason: format!("text index {label} would become negative"),
            });
        }
        u64::try_from(value).map_err(|_| VantaError::ValidationError {
            field: "stats".into(),
            reason: format!("text index {label} exceeds supported range"),
        })
    }

    pub(crate) fn text_index_ops_for_replace(
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

    pub(crate) fn replace_derived_indexes(
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

    pub(crate) fn parse_term_stats_key(key: &[u8]) -> Option<(String, String)> {
        const INTERNAL_PREFIX: &[u8] = b"\xffvanta_text_v3\0";
        const TERM_STATS_TAG: &[u8] = b"term\0";
        let remainder = key
            .strip_prefix(INTERNAL_PREFIX)?
            .strip_prefix(TERM_STATS_TAG)?;
        let pos = remainder.iter().position(|&b| b == 0)?;
        let ns = match String::from_utf8(remainder[..pos].to_vec()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(?e, raw = ?&remainder[..pos], "Invalid UTF-8 namespace in term stats key");
                return None;
            }
        };
        let token = match String::from_utf8(remainder[pos + 1..].to_vec()) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(?e, raw = ?&remainder[pos + 1..], "Invalid UTF-8 token in term stats key");
                return None;
            }
        };
        Some((ns, token))
    }

    pub(crate) fn parse_namespace_stats_key(key: &[u8]) -> Option<String> {
        const INTERNAL_PREFIX: &[u8] = b"\xffvanta_text_v3\0";
        const NAMESPACE_STATS_TAG: &[u8] = b"ns\0";
        let remainder = key
            .strip_prefix(INTERNAL_PREFIX)?
            .strip_prefix(NAMESPACE_STATS_TAG)?;
        match String::from_utf8(remainder.to_vec()) {
            Ok(s) => Some(s),
            Err(e) => {
                tracing::warn!(?e, raw = ?remainder, "Invalid UTF-8 in namespace stats key");
                None
            }
        }
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

    pub(crate) fn rebuild_derived_indexes_with_report(&self) -> Result<DerivedIndexRebuildReport> {
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

    pub(crate) fn rebuild_derived_indexes(&self) -> Result<()> {
        self.rebuild_derived_indexes_with_report().map(|_| ())
    }

    pub(crate) fn rebuild_text_index_with_report(&self) -> Result<TextIndexRebuildReport> {
        let started = Instant::now();
        let engine = self.engine_handle()?;

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

    pub(crate) fn rebuild_text_index(&self) -> Result<()> {
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

    pub(crate) fn build_text_index_audit_report_deep(
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

    pub(crate) fn build_text_index_audit_report_shallow(
        engine: &StorageEngine,
        namespace_filter: Option<&str>,
    ) -> Result<VantaTextIndexAuditReport> {
        let started = Instant::now();
        let spec = crate::text_index::TextIndexSpec::default();
        let expected = Self::expected_text_index_entries(engine, namespace_filter)?;

        let (state_valid, state_status) =
            Self::text_index_state_audit_status(engine, expected.counts, namespace_filter);

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

    pub(crate) fn indexed_ids_by_namespace(
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

    pub(crate) fn indexed_ids_by_filter(
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

    pub(crate) fn records_for_namespace(
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

    /// Export all records from a single namespace to a JSONL file.
    #[tracing::instrument(skip(self, path), err)]
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

    /// Export all records across all namespaces to a single JSONL file.
    #[tracing::instrument(skip(self, path), err)]
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

    /// Import memory records from an in-memory vector. Rebuilds derived and text indexes after import.
    /// Reports counts of inserted, updated, skipped, and errored records.
    #[tracing::instrument(skip(self, records), err)]
    pub fn import_records(&self, records: Vec<VantaMemoryRecord>) -> Result<VantaImportReport> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "import_records is not available when VantaDB is opened read-only".into(),
            });
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

    /// Import records from a JSONL file (one record per line).
    /// Skips empty lines and reports parse errors separately from import errors.
    #[tracing::instrument(skip(self, path), err)]
    pub fn import_file(&self, path: impl AsRef<Path>) -> Result<VantaImportReport> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "import_file is not available when VantaDB is opened read-only".into(),
            });
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
}

// ---------------------------------------------------------------------------
// From impls
// ---------------------------------------------------------------------------

impl From<crate::storage::IndexRebuildReport> for VantaIndexRebuildReport {
    fn from(report: crate::storage::IndexRebuildReport) -> Self {
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
            jemalloc_allocated_bytes: metrics.memory.jemalloc_allocated_bytes,
            jemalloc_active_bytes: metrics.memory.jemalloc_active_bytes,
            jemalloc_metadata_bytes: metrics.memory.jemalloc_metadata_bytes,
            jemalloc_resident_bytes: metrics.memory.jemalloc_resident_bytes,
            jemalloc_mapped_bytes: metrics.memory.jemalloc_mapped_bytes,
            jemalloc_retained_bytes: metrics.memory.jemalloc_retained_bytes,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_term_stats_key_valid() {
        let key = b"\xffvanta_text_v3\0term\0myns\0mytoken";
        let result = VantaEmbedded::parse_term_stats_key(key);
        assert_eq!(result, Some(("myns".into(), "mytoken".into())));
    }

    #[test]
    fn test_parse_term_stats_key_invalid_utf8() {
        let key = b"\xffvanta_text_v3\0term\0myns\0\xff\xfe";
        let result = VantaEmbedded::parse_term_stats_key(key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_term_stats_key_invalid_namespace_utf8() {
        let key = b"\xffvanta_text_v3\0term\0\xff\xfe\0token";
        let result = VantaEmbedded::parse_term_stats_key(key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_term_stats_key_truncated() {
        let key = b"\xffvanta_text_v3\0term";
        let result = VantaEmbedded::parse_term_stats_key(key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_namespace_stats_key_valid() {
        let key = b"\xffvanta_text_v3\0ns\0myns";
        let result = VantaEmbedded::parse_namespace_stats_key(key);
        assert_eq!(result, Some("myns".into()));
    }

    #[test]
    fn test_parse_namespace_stats_key_invalid_utf8() {
        let key = b"\xffvanta_text_v3\0ns\0\xff\xfe";
        let result = VantaEmbedded::parse_namespace_stats_key(key);
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_namespace_stats_key_truncated() {
        let key = b"\xffvanta_text_v3\0ns";
        let result = VantaEmbedded::parse_namespace_stats_key(key);
        assert_eq!(result, None);
    }
}
