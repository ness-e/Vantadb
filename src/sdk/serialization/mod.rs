//! Wire-format helpers for reading and writing VantaDB memory records
//! to/from internal node representations and JSONL export lines.

#[cfg(test)]
use super::builder::VantaEmbedded;
use super::types::*;
use crate::error::{Result, VantaError};
use crate::node::{FieldValue, SparseVector, UnifiedNode, VectorRepresentations};
use twox_hash::XxHash3_128;
use web_time::{SystemTime, UNIX_EPOCH};

const RESERVED_PREFIX: &str = "__vanta_";
/// Internal field name used to store the namespace on a memory record node.
pub const FIELD_NAMESPACE: &str = "__vanta_namespace";
/// Internal field name used to store the record key on a memory record node.
pub const FIELD_KEY: &str = "__vanta_key";
/// Internal field name used to store the payload text on a memory record node.
pub const FIELD_PAYLOAD: &str = "__vanta_payload";
/// Internal field name storing the Unix-ms creation timestamp.
pub const FIELD_CREATED_AT_MS: &str = "__vanta_created_at_ms";
/// Internal field name storing the Unix-ms last-update timestamp.
pub const FIELD_UPDATED_AT_MS: &str = "__vanta_updated_at_ms";
/// Internal field name storing the monotonic version counter.
pub const FIELD_VERSION: &str = "__vanta_version";
/// Internal field name storing the optional Unix-ms expiry deadline.
pub const FIELD_EXPIRES_AT_MS: &str = "__vanta_expires_at_ms";
/// Internal field name storing the key of the record that supersedes this one (ADR-028).
pub const FIELD_SUPERSEDED_BY: &str = "__vanta_superseded_by";
/// Internal field name storing the Unix-ms timestamp when the supersession was recorded (ADR-028).
pub const FIELD_SUPERSEDED_AT_MS: &str = "__vanta_superseded_at_ms";
/// Internal `ext_metadata` key storing the sparse vector on a memory record
/// node as interleaved `ListFloat` pairs (ADR-019). Kept out of the
/// bincode-graph so old databases read missing keys as `None`.
pub const SPARSE_VECTOR_EXT_KEY: &str = "__vanta_sparse_vector";
const EXPORT_SCHEMA_VERSION: u32 = 1;
pub(crate) const DERIVED_INDEX_SCHEMA_VERSION: u32 = 1;
pub(crate) const DERIVED_INDEX_STATE_KEY: &[u8] = b"derived_index_state";
pub(crate) const TEXT_INDEX_STATE_KEY: &[u8] = b"text_index_state";
pub(crate) const SPARSE_INDEX_STATE_KEY: &[u8] = b"sparse_index_state";
pub(crate) const SPARSE_INDEX_SCHEMA_VERSION: u32 = 1;

pub(crate) mod conversions;
pub mod graph_types;
pub(crate) mod impl_export;
pub(crate) mod impl_index;
pub(crate) mod impl_rebuild;
pub(crate) mod impl_sparse_index;
pub(crate) mod impl_text_index;
pub mod vector_types;

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

pub(crate) fn memory_node_id(namespace: &str, key: &str) -> u128 {
    let mut hasher = XxHash3_128::default();
    hasher.write(namespace.as_bytes());
    hasher.write(&[0]);
    hasher.write(key.as_bytes());
    hasher.finish_128()
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

/// Whether a `VantaValue` can be encoded as a scalar payload-index key
/// (`encoded_scalar_value`). List variants cannot — the derived payload index
/// stores flattened scalar entries, so a whole-list prefix scan is impossible.
/// `list()`/`records_for_namespace` fall back to a namespace scan and apply
/// the filter by equality (`matches_memory_filters`) for non-scalar filter
/// values instead of failing (ERR-026: a list/null filter must narrow, not
/// error out).
pub(crate) fn is_scalar_indexable(value: &VantaValue) -> bool {
    !matches!(
        value,
        VantaValue::ListString(_)
            | VantaValue::ListInt(_)
            | VantaValue::ListFloat(_)
            | VantaValue::ListBool(_)
            | VantaValue::ListDateTime(_)
    )
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

pub(crate) fn node_id_bytes(node_id: u128) -> Vec<u8> {
    node_id.to_le_bytes().to_vec()
}

pub(crate) fn decode_node_id(bytes: &[u8]) -> Option<u128> {
    if bytes.len() != std::mem::size_of::<u128>() {
        return None;
    }
    let mut id = [0u8; 16];
    id.copy_from_slice(bytes);
    Some(u128::from_le_bytes(id))
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

/// Encode a `SparseVector` into the persisted `ListFloat` field format:
/// interleaved `[dim_0, val_0, dim_1, val_1, ...]` (ADR-019). Both `u32` dims
/// and `f32` weights are exactly representable in `f64`, so the round-trip is
/// lossless, and `BTreeMap` iteration keeps the encoding deterministic.
fn sparse_vector_to_field(sparse: &SparseVector) -> FieldValue {
    let mut flat = Vec::with_capacity(sparse.0.len() * 2);
    for (dim, weight) in &sparse.0 {
        flat.push(*dim as f64);
        flat.push(*weight as f64);
    }
    FieldValue::ListFloat(flat)
}

/// Decode a persisted `ListFloat` sparse field back into a `SparseVector`.
/// Returns `None` for malformed payloads (odd length or invalid dims),
/// matching the corrupt handling of the legacy JSON path.
fn sparse_vector_from_field(flat: &[f64]) -> Option<SparseVector> {
    if flat.len() % 2 != 0 {
        return None;
    }
    let mut map = std::collections::BTreeMap::new();
    for pair in flat.chunks_exact(2) {
        let dim = pair[0];
        // AUD-023 (P2-7): `dim as u32` satura silenciosamente NaN/negativos/
        // out-of-range y trunca dims no-enteras, corrompiendo el vector. Rechazar
        // el payload completo en vez de persistir un dim inválido.
        if !dim.is_finite() || dim < 0.0 || dim > u32::MAX as f64 || dim.fract() != 0.0 {
            return None;
        }
        map.insert(dim as u32, pair[1] as f32);
    }
    Some(SparseVector(map))
}

pub fn memory_record_from_node(node: &UnifiedNode) -> Option<VantaMemoryRecord> {
    memory_record_from_node_inner(node, true)
}

/// Like [`memory_record_from_node`] but **without** lazy TTL eviction: records
/// whose deadline has passed are still returned so callers can observe them
/// (e.g. per-namespace TTL statistics).
pub(crate) fn memory_record_from_node_include_expired(
    node: &UnifiedNode,
) -> Option<VantaMemoryRecord> {
    memory_record_from_node_inner(node, false)
}

fn memory_record_from_node_inner(
    node: &UnifiedNode,
    apply_lazy_ttl: bool,
) -> Option<VantaMemoryRecord> {
    if !node.is_alive() {
        return None;
    }

    let mut fields: VantaFields = node
        .relational
        .iter()
        .map(|(key, value)| (key.clone(), value.clone().into()))
        .collect();

    let namespace = get_string_field(&fields, FIELD_NAMESPACE)?;
    let key = get_string_field(&fields, FIELD_KEY)?;
    let payload = get_string_field(&fields, FIELD_PAYLOAD)?;
    let created_at_ms = get_u64_field(&fields, FIELD_CREATED_AT_MS)?;
    let updated_at_ms = get_u64_field(&fields, FIELD_UPDATED_AT_MS)?;
    let version = get_u64_field(&fields, FIELD_VERSION)?;
    let expires_at_ms = get_u64_field(&fields, FIELD_EXPIRES_AT_MS);
    let superseded_by = get_string_field(&fields, FIELD_SUPERSEDED_BY);
    let superseded_at_ms = get_u64_field(&fields, FIELD_SUPERSEDED_AT_MS);

    fields.remove(FIELD_NAMESPACE);
    fields.remove(FIELD_KEY);
    fields.remove(FIELD_PAYLOAD);
    fields.remove(FIELD_CREATED_AT_MS);
    fields.remove(FIELD_UPDATED_AT_MS);
    fields.remove(FIELD_VERSION);
    fields.remove(FIELD_EXPIRES_AT_MS);
    fields.remove(FIELD_SUPERSEDED_BY);
    fields.remove(FIELD_SUPERSEDED_AT_MS);
    fields.remove(SPARSE_VECTOR_EXT_KEY);

    // Lazy TTL eviction: if expires_at_ms is set and the deadline
    // has passed, the record is treated as if it no longer exists.
    if apply_lazy_ttl {
        if let Some(deadline) = expires_at_ms {
            if deadline > 0 {
                let now = now_ms();
                if now > deadline {
                    return None;
                }
            }
        }
    }

    let vector = match &node.vector {
        VectorRepresentations::Full(vector) => Some(vector.clone()),
        _ => None,
    };

    // Sparse vector persisted as a reserved relational field (survives the
    // KV round-trip via NodeMetadata; `ext_metadata` is memory-only).
    // Missing key => None (old records). The parse is conditional: nodes
    // without SPARSE_VECTOR_EXT_KEY skip serde_json entirely (PERF-07).
    // ADR-019: new writes are ListFloat pairs; legacy String/JSON remains
    // readable for backward compat.
    let sparse_vector = match node.get_field(SPARSE_VECTOR_EXT_KEY) {
        Some(crate::node::FieldValue::ListFloat(flat)) => match sparse_vector_from_field(flat) {
            Some(parsed) => Some(parsed),
            None => {
                tracing::warn!(
                    node_id = %node.id,
                    "corrupt sparse vector payload (malformed ListFloat pairs) ignored during read"
                );
                None
            }
        },
        Some(crate::node::FieldValue::String(json)) => match serde_json::from_str(json) {
            Ok(parsed) => parsed,
            Err(err) => {
                // PERF-07: a present-but-corrupt sparse payload was silently
                // dropped by `.ok()`. Log it once so the corruption is
                // visible, but keep returning None (old behavior) rather than
                // failing the whole read.
                tracing::warn!(
                    node_id = %node.id,
                    error = %err,
                    "corrupt sparse vector payload ignored during read"
                );
                None
            }
        },
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
        sparse_vector,
        expires_at_ms,
        superseded_by,
        superseded_at_ms,
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
    let sparse_vector = std::mem::take(&mut record.sparse_vector);

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

    if let Some(superseded_by) = record.superseded_by.clone() {
        node.set_field(FIELD_SUPERSEDED_BY, FieldValue::String(superseded_by));
    }
    if let Some(superseded_at) = record.superseded_at_ms {
        node.set_field(
            FIELD_SUPERSEDED_AT_MS,
            FieldValue::Int(superseded_at as i64),
        );
    }

    // ponytail: iterar por referencia, no clonar todo el HashMap solo para leerlo
    for (k, v) in &metadata {
        node.set_field(k.clone(), v.clone().into());
    }

    let vector = vector.filter(|v| !v.is_empty());
    if let Some(ref vec) = vector {
        node.vector = VectorRepresentations::Full(vec.clone());
        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
    }

    if let Some(sparse) = &sparse_vector {
        // ADR-019: persist as interleaved ListFloat pairs — no serde_json on
        // the write hot path. Empty vectors write an empty ListFloat (matches
        // legacy `"{}"` round-trip semantics).
        node.set_field(SPARSE_VECTOR_EXT_KEY, sparse_vector_to_field(sparse));
    }

    record.namespace = namespace;
    record.key = key;
    record.payload = payload;
    record.metadata = metadata;
    record.vector = vector;
    record.sparse_vector = sparse_vector;

    (node, record)
}

/// Convert a `VantaMemoryRecord` into a JSONL export line with schema version.
pub fn export_line_from_record(record: VantaMemoryRecord) -> VantaMemoryExportLine {
    VantaMemoryExportLine {
        schema_version: EXPORT_SCHEMA_VERSION,
        namespace: record.namespace,
        key: record.key,
        payload: record.payload,
        metadata: record.metadata,
        vector: record.vector,
        sparse_vector: record.sparse_vector,
        created_at_ms: record.created_at_ms,
        updated_at_ms: record.updated_at_ms,
        version: record.version,
        expires_at_ms: record.expires_at_ms,
        superseded_by: record.superseded_by,
        superseded_at_ms: record.superseded_at_ms,
    }
}

/// Rebuild a `VantaMemoryRecord` from a JSONL export line, recomputing the
/// deterministic node id (`memory_node_id(namespace, key)`). The inverse of
/// [`export_line_from_record`] — used by import paths that receive JSONL
/// content as a string (e.g. the MCP `import` tool) instead of a file.
///
/// Fails if `schema_version` is not the current export schema.
pub fn record_from_export_line(line: VantaMemoryExportLine) -> Result<VantaMemoryRecord> {
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
        sparse_vector: line.sparse_vector,
        expires_at_ms: line.expires_at_ms,
        superseded_by: line.superseded_by,
        superseded_at_ms: line.superseded_at_ms,
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

pub(crate) fn matches_advanced_filters(
    record: &VantaMemoryRecord,
    filter_ops: &crate::sdk::types::VantaMemoryFilter,
) -> bool {
    filter_ops.iter().all(|op_item| {
        if let Some(actual) = record.metadata.get(&op_item.field) {
            match op_item.op {
                crate::sdk::types::VantaFilterOp::Eq => actual == &op_item.value,
                crate::sdk::types::VantaFilterOp::Neq => actual != &op_item.value,
                crate::sdk::types::VantaFilterOp::Gt => actual > &op_item.value,
                crate::sdk::types::VantaFilterOp::Gte => actual >= &op_item.value,
                crate::sdk::types::VantaFilterOp::Lt => actual < &op_item.value,
                crate::sdk::types::VantaFilterOp::Lte => actual <= &op_item.value,
            }
        } else {
            false
        }
    })
}

#[cfg(test)]
#[allow(missing_docs)]
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

    // ΓöÇΓöÇΓöÇ now_ms ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_now_ms_non_zero() {
        let t = now_ms();
        assert!(
            t > 1_700_000_000_000,
            "expected reasonable Unix ms, got {t}"
        );
    }

    // ΓöÇΓöÇΓöÇ memory_node_id ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_memory_node_id_deterministic() {
        let a = memory_node_id("ns", "key1");
        let b = memory_node_id("ns", "key1");
        assert_eq!(a, b);

        let c = memory_node_id("ns", "key2");
        assert_ne!(a, c);

        let d = memory_node_id("other", "key1");
        assert_ne!(a, d);
    }

    // ΓöÇΓöÇΓöÇ validate_namespace ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_validate_namespace_empty() {
        let err = validate_namespace("").unwrap_err();
        assert!(err.to_string().contains("namespace must not be empty"));
    }

    #[test]
    fn test_validate_namespace_too_long() {
        let long = "a".repeat(129);
        let err = validate_namespace(&long).unwrap_err();
        assert!(err.to_string().contains("at most 128"));
    }

    #[test]
    fn test_validate_namespace_invalid_chars() {
        let err = validate_namespace("hello world").unwrap_err();
        assert!(err.to_string().contains("may contain only"));
        let err2 = validate_namespace("ns@name").unwrap_err();
        assert!(err2.to_string().contains("may contain only"));
    }

    #[test]
    fn test_validate_namespace_valid() {
        assert!(validate_namespace("my.namespace/foo_bar-baz").is_ok());
        assert!(validate_namespace("a").is_ok());
        assert!(validate_namespace("a.b/c_d-e").is_ok());
        assert!(validate_namespace("default").is_ok());
    }

    // ΓöÇΓöÇΓöÇ validate_key ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_validate_key_empty() {
        let err = validate_key("").unwrap_err();
        assert!(err.to_string().contains("key must not be empty"));
    }

    #[test]
    fn test_validate_key_too_long() {
        let long = "a".repeat(513);
        let err = validate_key(&long).unwrap_err();
        assert!(err.to_string().contains("at most 512"));
    }

    #[test]
    fn test_validate_key_nul_byte() {
        let err = validate_key("bad\0key").unwrap_err();
        assert!(err.to_string().contains("must not contain NUL"));
    }

    #[test]
    fn test_validate_key_valid() {
        assert!(validate_key("hello_world").is_ok());
        assert!(validate_key("a").is_ok());
        assert!(validate_key("Σ╜áσÑ╜").is_ok());
    }

    // ΓöÇΓöÇΓöÇ validate_metadata ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_validate_metadata_reserved_prefix() {
        let mut meta = VantaMemoryMetadata::new();
        meta.insert("__vanta_foo".into(), VantaValue::String("x".into()));
        let err = validate_metadata(&meta).unwrap_err();
        assert!(err.to_string().contains("reserved"));
    }

    #[test]
    fn test_validate_metadata_nul_in_key() {
        let mut meta = VantaMemoryMetadata::new();
        meta.insert("bad\0key".into(), VantaValue::Int(1));
        let err = validate_metadata(&meta).unwrap_err();
        assert!(err.to_string().contains("NUL"));
    }

    #[test]
    fn test_validate_metadata_valid() {
        let mut meta = VantaMemoryMetadata::new();
        meta.insert("color".into(), VantaValue::String("blue".into()));
        assert!(validate_metadata(&meta).is_ok());
        assert!(validate_metadata(&VantaMemoryMetadata::new()).is_ok());
    }

    // ΓöÇΓöÇΓöÇ namespace_index_key / namespace_index_prefix ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_namespace_index_key_format() {
        let key = namespace_index_key("myns", "mykey");
        assert_eq!(key, b"myns\0mykey");
    }

    #[test]
    fn test_namespace_index_prefix_format() {
        let prefix = namespace_index_prefix("myns");
        assert_eq!(prefix, b"myns\0");
    }

    // ΓöÇΓöÇΓöÇ encoded_scalar_value ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_encoded_scalar_value_string() {
        let encoded = encoded_scalar_value(&VantaValue::String("hello".into())).unwrap();
        assert_eq!(encoded, b"s:hello");
    }

    #[test]
    fn test_encoded_scalar_value_int() {
        let encoded = encoded_scalar_value(&VantaValue::Int(42)).unwrap();
        assert_eq!(encoded, b"i:42");
    }

    #[test]
    fn test_encoded_scalar_value_float() {
        let encoded = encoded_scalar_value(&VantaValue::Float(1.5)).unwrap();
        assert!(encoded.starts_with(b"f:"));
        assert_eq!(encoded.len(), 18); // "f:" + 16 hex chars
    }

    #[test]
    fn test_encoded_scalar_value_bool() {
        assert_eq!(
            encoded_scalar_value(&VantaValue::Bool(true)).unwrap(),
            b"b:1"
        );
        assert_eq!(
            encoded_scalar_value(&VantaValue::Bool(false)).unwrap(),
            b"b:0"
        );
    }

    #[test]
    fn test_encoded_scalar_value_datetime() {
        let dt = chrono::DateTime::parse_from_rfc3339("2025-01-15T10:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let encoded = encoded_scalar_value(&VantaValue::DateTime(dt)).unwrap();
        assert!(encoded.starts_with(b"d:"));
        assert!(String::from_utf8_lossy(&encoded).contains("2025-01-15"));
    }

    #[test]
    fn test_encoded_scalar_value_null() {
        assert_eq!(encoded_scalar_value(&VantaValue::Null).unwrap(), b"n:");
    }

    #[test]
    fn test_encoded_scalar_value_list_returns_error() {
        assert!(encoded_scalar_value(&VantaValue::ListString(vec!["a".into()])).is_err());
        assert!(encoded_scalar_value(&VantaValue::ListInt(vec![1])).is_err());
        assert!(encoded_scalar_value(&VantaValue::ListFloat(vec![1.0])).is_err());
        assert!(encoded_scalar_value(&VantaValue::ListBool(vec![true])).is_err());
        assert!(encoded_scalar_value(&VantaValue::ListDateTime(vec![])).is_err());
    }

    // ΓöÇΓöÇΓöÇ payload_index_prefix / payload_index_key ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_payload_index_prefix_string() {
        let prefix =
            payload_index_prefix("ns", "color", &VantaValue::String("red".into())).unwrap();
        assert_eq!(prefix, b"ns\0color\0s:red\0");
    }

    #[test]
    fn test_payload_index_prefix_int() {
        let prefix = payload_index_prefix("ns", "age", &VantaValue::Int(30)).unwrap();
        assert_eq!(prefix, b"ns\0age\0i:30\0");
    }

    #[test]
    fn test_payload_index_prefix_list_error() {
        let result = payload_index_prefix("ns", "f", &VantaValue::ListInt(vec![1]));
        assert!(result.is_err());
    }

    #[test]
    fn test_payload_index_key() {
        let key =
            payload_index_key("ns", "status", &VantaValue::String("ok".into()), "mykey").unwrap();
        assert_eq!(key, b"ns\0status\0s:ok\0mykey");
    }

    // ΓöÇΓöÇΓöÇ node_id_bytes / decode_node_id ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_node_id_roundtrip() {
        let id: u128 = 0xdeadbeefcafe;
        let bytes = node_id_bytes(id);
        assert_eq!(bytes.len(), 16);
        let decoded = decode_node_id(&bytes).unwrap();
        assert_eq!(decoded, id);
    }

    #[test]
    fn test_decode_node_id_wrong_length() {
        assert!(decode_node_id(&[0u8; 8]).is_none());
        assert!(decode_node_id(&[]).is_none());
        assert!(decode_node_id(&[0u8; 17]).is_none());
    }

    #[test]
    fn test_decode_node_id_zero() {
        let bytes = node_id_bytes(0);
        assert_eq!(decode_node_id(&bytes), Some(0));
    }

    // ΓöÇΓöÇΓöÇ get_string_field / get_u64_field ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_get_string_field_present() {
        let mut fields = VantaFields::new();
        fields.insert("name".into(), VantaValue::String("Alice".into()));
        assert_eq!(get_string_field(&fields, "name"), Some("Alice".into()));
    }

    #[test]
    fn test_get_string_field_missing() {
        let fields = VantaFields::new();
        assert_eq!(get_string_field(&fields, "name"), None);
    }

    #[test]
    fn test_get_string_field_wrong_type() {
        let mut fields = VantaFields::new();
        fields.insert("age".into(), VantaValue::Int(30));
        assert_eq!(get_string_field(&fields, "age"), None);
    }

    #[test]
    fn test_get_u64_field_present() {
        let mut fields = VantaFields::new();
        fields.insert("count".into(), VantaValue::Int(42));
        assert_eq!(get_u64_field(&fields, "count"), Some(42));
    }

    #[test]
    fn test_get_u64_field_negative_rejected() {
        let mut fields = VantaFields::new();
        fields.insert("neg".into(), VantaValue::Int(-5));
        assert_eq!(get_u64_field(&fields, "neg"), None);
    }

    #[test]
    fn test_get_u64_field_missing() {
        let fields = VantaFields::new();
        assert_eq!(get_u64_field(&fields, "x"), None);
    }

    #[test]
    fn test_get_u64_field_wrong_type() {
        let mut fields = VantaFields::new();
        fields.insert("name".into(), VantaValue::String("x".into()));
        assert_eq!(get_u64_field(&fields, "name"), None);
    }

    // ΓöÇΓöÇΓöÇ memory_record_from_node ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    fn make_memory_node(id: u128, namespace: &str, key: &str) -> UnifiedNode {
        use crate::node::FieldValue;
        let mut node = UnifiedNode::new(id);
        node.set_field(FIELD_NAMESPACE, FieldValue::String(namespace.to_string()));
        node.set_field(FIELD_KEY, FieldValue::String(key.to_string()));
        node.set_field(FIELD_PAYLOAD, FieldValue::String("test payload".into()));
        node.set_field(FIELD_CREATED_AT_MS, FieldValue::Int(1000));
        node.set_field(FIELD_UPDATED_AT_MS, FieldValue::Int(1000));
        node.set_field(FIELD_VERSION, FieldValue::Int(1));
        node
    }

    #[test]
    fn test_memory_record_from_node_valid() {
        let node = make_memory_node(42, "myns", "mykey");
        let record = memory_record_from_node(&node).unwrap();
        assert_eq!(record.namespace, "myns");
        assert_eq!(record.key, "mykey");
        assert_eq!(record.payload, "test payload");
        assert_eq!(record.created_at_ms, 1000);
        assert_eq!(record.updated_at_ms, 1000);
        assert_eq!(record.version, 1);
        assert_eq!(record.node_id, 42);
        assert!(record.metadata.is_empty());
        assert!(record.vector.is_none());
        assert!(record.expires_at_ms.is_none());
    }

    #[test]
    fn test_memory_record_from_node_deleted() {
        let mut node = make_memory_node(1, "ns", "k");
        node.mark_deleted();
        assert!(memory_record_from_node(&node).is_none());
    }

    #[test]
    fn test_memory_record_from_node_missing_required_fields() {
        let node = UnifiedNode::new(1);
        assert!(memory_record_from_node(&node).is_none());
    }

    #[test]
    fn test_memory_record_from_node_expired() {
        let mut node = UnifiedNode::new(1);
        node.set_field(
            FIELD_NAMESPACE,
            crate::node::FieldValue::String("ns".into()),
        );
        node.set_field(FIELD_KEY, crate::node::FieldValue::String("k".into()));
        node.set_field(FIELD_PAYLOAD, crate::node::FieldValue::String("p".into()));
        node.set_field(FIELD_CREATED_AT_MS, crate::node::FieldValue::Int(1000));
        node.set_field(FIELD_UPDATED_AT_MS, crate::node::FieldValue::Int(1000));
        node.set_field(FIELD_VERSION, crate::node::FieldValue::Int(1));
        // Expire in the past
        node.set_field(FIELD_EXPIRES_AT_MS, crate::node::FieldValue::Int(1));
        assert!(memory_record_from_node(&node).is_none());
    }

    #[test]
    fn test_memory_record_from_node_strips_internal_fields() {
        let mut node = make_memory_node(1, "ns", "k");
        node.set_field(
            "custom_field",
            crate::node::FieldValue::String("keep".into()),
        );
        let record = memory_record_from_node(&node).unwrap();
        assert!(!record.metadata.contains_key(FIELD_NAMESPACE));
        assert!(!record.metadata.contains_key(FIELD_KEY));
        assert!(!record.metadata.contains_key(FIELD_PAYLOAD));
        assert!(!record.metadata.contains_key(FIELD_CREATED_AT_MS));
        assert!(!record.metadata.contains_key(FIELD_UPDATED_AT_MS));
        assert!(!record.metadata.contains_key(FIELD_VERSION));
        assert!(!record.metadata.contains_key(FIELD_EXPIRES_AT_MS));
        assert_eq!(
            record.metadata.get("custom_field"),
            Some(&VantaValue::String("keep".into()))
        );
    }

    #[test]
    fn test_memory_record_from_node_with_vector() {
        let mut node = make_memory_node(1, "ns", "k");
        node.vector = VectorRepresentations::Full(vec![0.1, 0.2]);
        let record = memory_record_from_node(&node).unwrap();
        assert_eq!(record.vector, Some(vec![0.1, 0.2]));
    }

    #[test]
    fn test_memory_record_from_node_without_expiry() {
        let mut node = make_memory_node(1, "ns", "k");
        node.set_field(FIELD_EXPIRES_AT_MS, crate::node::FieldValue::Int(0));
        let record = memory_record_from_node(&node).unwrap();
        assert_eq!(record.expires_at_ms, Some(0));
    }

    // ΓöÇΓöÇΓöÇ memory_record_to_node_owned ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_memory_record_to_node_owned_roundtrip() {
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "payload".into(),
            metadata: {
                let mut m = VantaMemoryMetadata::new();
                m.insert("color".into(), VantaValue::String("red".into()));
                m
            },
            created_at_ms: 100,
            updated_at_ms: 200,
            version: 3,
            node_id: memory_node_id("ns", "k"),
            vector: Some(vec![0.5, 0.5]),
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };

        let (node, returned_record) = memory_record_to_node_owned(record);
        assert_eq!(node.id, returned_record.node_id);
        assert!(node.is_alive());
        assert_eq!(returned_record.namespace, "ns");
        assert_eq!(returned_record.key, "k");
        assert_eq!(returned_record.payload, "payload");
        assert_eq!(returned_record.vector, Some(vec![0.5, 0.5]));

        // Verify fields are on the node
        assert_eq!(
            node.get_field(FIELD_NAMESPACE),
            Some(&crate::node::FieldValue::String("ns".into()))
        );
        assert_eq!(
            node.get_field(FIELD_KEY),
            Some(&crate::node::FieldValue::String("k".into()))
        );
    }

    #[test]
    fn test_memory_record_to_node_owned_with_expiry() {
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1,
            updated_at_ms: 1,
            version: 1,
            node_id: memory_node_id("ns", "k"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: Some(999_999_999_999),
            superseded_by: None,
            superseded_at_ms: None,
        };
        let (node, _) = memory_record_to_node_owned(record);
        assert_eq!(
            node.get_field(FIELD_EXPIRES_AT_MS),
            Some(&crate::node::FieldValue::Int(999_999_999_999))
        );
    }

    #[test]
    fn test_memory_record_to_node_owned_empty_vector_not_stored() {
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 1,
            updated_at_ms: 1,
            version: 1,
            node_id: memory_node_id("ns", "k"),
            vector: Some(vec![]),
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        let (node, returned) = memory_record_to_node_owned(record);
        assert!(returned.vector.is_none());
        // Empty vector should not set HAS_VECTOR flag
        let expected = crate::node::VectorRepresentations::None;
        assert_eq!(node.vector, expected);
    }

    // ΓöÇΓöÇΓöÇ export_line_from_record / record_from_export_line ΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_export_line_roundtrip() {
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "data".into(),
            metadata: {
                let mut m = VantaMemoryMetadata::new();
                m.insert("x".into(), VantaValue::Int(1));
                m
            },
            created_at_ms: 10,
            updated_at_ms: 20,
            version: 2,
            node_id: memory_node_id("ns", "k"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };

        let line = export_line_from_record(record.clone());
        assert_eq!(line.schema_version, 1);
        assert_eq!(line.namespace, "ns");
        assert_eq!(line.key, "k");
        assert_eq!(line.version, 2);

        let recovered = record_from_export_line(line).unwrap();
        // node_id is computed deterministically, should match original
        assert_eq!(recovered.node_id, record.node_id);
        assert_eq!(recovered.namespace, record.namespace);
        assert_eq!(recovered.key, record.key);
        assert_eq!(recovered.version, record.version);
    }

    #[test]
    fn test_record_from_export_line_wrong_schema_version() {
        let line = VantaMemoryExportLine {
            schema_version: 999,
            namespace: "ns".into(),
            key: "k".into(),
            payload: "".into(),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 0,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        let err = record_from_export_line(line).unwrap_err();
        assert!(err.to_string().contains("unsupported"));
        assert!(err.to_string().contains("999"));
    }

    #[test]
    fn test_export_line_serialization_roundtrip() {
        let line = VantaMemoryExportLine {
            schema_version: 1,
            namespace: "myns".into(),
            key: "mykey".into(),
            payload: "my payload".into(),
            metadata: VantaMemoryMetadata::new(),
            vector: Some(vec![0.1, 0.2]),
            sparse_vector: None,
            created_at_ms: 100,
            updated_at_ms: 200,
            version: 5,
            expires_at_ms: None,
            superseded_by: Some("successor".into()),
            superseded_at_ms: Some(1234),
        };
        let json = serde_json::to_string(&line).unwrap();
        let deserialized: VantaMemoryExportLine = serde_json::from_str(&json).unwrap();
        // Compare fields individually since VantaMemoryExportLine lacks PartialEq
        assert_eq!(deserialized.schema_version, line.schema_version);
        assert_eq!(deserialized.namespace, line.namespace);
        assert_eq!(deserialized.key, line.key);
        assert_eq!(deserialized.payload, line.payload);
        assert_eq!(deserialized.metadata, line.metadata);
        assert_eq!(deserialized.vector, line.vector);
        assert_eq!(deserialized.created_at_ms, line.created_at_ms);
        assert_eq!(deserialized.updated_at_ms, line.updated_at_ms);
        assert_eq!(deserialized.version, line.version);
        assert_eq!(deserialized.expires_at_ms, line.expires_at_ms);
        assert_eq!(deserialized.superseded_by, line.superseded_by);
        assert_eq!(deserialized.superseded_at_ms, line.superseded_at_ms);
    }

    #[test]
    fn test_export_line_superseded_roundtrip() {
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "old".into(),
            payload: "data".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 10,
            updated_at_ms: 20,
            version: 2,
            node_id: memory_node_id("ns", "old"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: Some("new".into()),
            superseded_at_ms: Some(4321),
        };

        let line = export_line_from_record(record.clone());
        assert_eq!(line.superseded_by.as_deref(), Some("new"));
        assert_eq!(line.superseded_at_ms, Some(4321));

        let recovered = record_from_export_line(line).unwrap();
        assert_eq!(recovered.superseded_by, record.superseded_by);
        assert_eq!(recovered.superseded_at_ms, record.superseded_at_ms);
        assert_eq!(recovered.version, record.version);
    }

    #[test]
    fn test_memory_record_superseded_node_roundtrip() {
        // ADR-028: superseded fields must survive record → node → record.
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "old".into(),
            payload: "data".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 10,
            updated_at_ms: 20,
            version: 2,
            node_id: memory_node_id("ns", "old"),
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: Some("new".into()),
            superseded_at_ms: Some(4321),
        };

        let (node, _) = memory_record_to_node_owned(record.clone());
        assert_eq!(
            node.get_field(FIELD_SUPERSEDED_BY),
            Some(&crate::node::FieldValue::String("new".into()))
        );
        assert_eq!(
            node.get_field(FIELD_SUPERSEDED_AT_MS),
            Some(&crate::node::FieldValue::Int(4321))
        );

        let recovered = memory_record_from_node(&node).unwrap();
        assert_eq!(recovered.superseded_by, record.superseded_by);
        assert_eq!(recovered.superseded_at_ms, record.superseded_at_ms);
    }

    #[test]
    fn test_memory_record_backward_compat_no_superseded_fields() {
        // ADR-028: dumps written before the fields existed deserialize to None.
        let json = serde_json::json!({
            "namespace": "ns",
            "key": "k",
            "payload": "data",
            "metadata": {},
            "created_at_ms": 10,
            "updated_at_ms": 20,
            "version": 1,
            "node_id": "42",
            "vector": null,
            "sparse_vector": null,
            "expires_at_ms": null
        });
        let record: VantaMemoryRecord = serde_json::from_value(json).unwrap();
        assert_eq!(record.superseded_by, None);
        assert_eq!(record.superseded_at_ms, None);
    }

    // ΓöÇΓöÇΓöÇ matches_memory_filters ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    #[test]
    fn test_matches_memory_filters_exact() {
        let mut record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 0,
            node_id: 0,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        record
            .metadata
            .insert("color".into(), VantaValue::String("blue".into()));

        let mut filters = VantaMemoryMetadata::new();
        filters.insert("color".into(), VantaValue::String("blue".into()));
        assert!(matches_memory_filters(&record, &filters));

        let mut bad_filters = VantaMemoryMetadata::new();
        bad_filters.insert("color".into(), VantaValue::String("red".into()));
        assert!(!matches_memory_filters(&record, &bad_filters));
    }

    #[test]
    fn test_matches_memory_filters_empty() {
        let record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 0,
            node_id: 0,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        assert!(matches_memory_filters(&record, &VantaMemoryMetadata::new()));
    }

    #[test]
    fn test_matches_memory_filters_multi() {
        let mut record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 0,
            node_id: 0,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        record.metadata.insert("a".into(), VantaValue::Int(1));
        record
            .metadata
            .insert("b".into(), VantaValue::String("x".into()));

        let mut filters = VantaMemoryMetadata::new();
        filters.insert("a".into(), VantaValue::Int(1));
        filters.insert("b".into(), VantaValue::String("x".into()));
        assert!(matches_memory_filters(&record, &filters));

        let mut bad = VantaMemoryMetadata::new();
        bad.insert("a".into(), VantaValue::Int(1));
        bad.insert("b".into(), VantaValue::String("y".into()));
        assert!(!matches_memory_filters(&record, &bad));
    }

    // ΓöÇΓöÇ matches_advanced_filters ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

    fn make_record_with_meta(pairs: &[(&str, VantaValue)]) -> VantaMemoryRecord {
        let mut record = VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 0,
            node_id: 0,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        for (k, v) in pairs {
            record.metadata.insert(k.to_string(), v.clone());
        }
        record
    }

    #[test]
    fn test_advanced_filter_eq() {
        let r = make_record_with_meta(&[("color", VantaValue::String("blue".into()))]);
        let ops = vec![crate::sdk::types::VantaMemoryFilterItem {
            field: "color".into(),
            op: crate::sdk::types::VantaFilterOp::Eq,
            value: VantaValue::String("blue".into()),
        }];
        assert!(matches_advanced_filters(&r, &ops));
        let ops_fail = vec![crate::sdk::types::VantaMemoryFilterItem {
            field: "color".into(),
            op: crate::sdk::types::VantaFilterOp::Eq,
            value: VantaValue::String("red".into()),
        }];
        assert!(!matches_advanced_filters(&r, &ops_fail));
    }

    #[test]
    fn test_advanced_filter_neq() {
        let r = make_record_with_meta(&[("status", VantaValue::String("active".into()))]);
        let ops = vec![crate::sdk::types::VantaMemoryFilterItem {
            field: "status".into(),
            op: crate::sdk::types::VantaFilterOp::Neq,
            value: VantaValue::String("inactive".into()),
        }];
        assert!(matches_advanced_filters(&r, &ops));
    }

    #[test]
    fn test_advanced_filter_gt_gte_lt_lte_int() {
        let r = make_record_with_meta(&[("score", VantaValue::Int(50))]);
        let make_op = |op: crate::sdk::types::VantaFilterOp, v: i64| {
            vec![crate::sdk::types::VantaMemoryFilterItem {
                field: "score".into(),
                op,
                value: VantaValue::Int(v),
            }]
        };
        assert!(matches_advanced_filters(
            &r,
            &make_op(crate::sdk::types::VantaFilterOp::Gt, 40)
        ));
        assert!(!matches_advanced_filters(
            &r,
            &make_op(crate::sdk::types::VantaFilterOp::Gt, 60)
        ));
        assert!(matches_advanced_filters(
            &r,
            &make_op(crate::sdk::types::VantaFilterOp::Gte, 50)
        ));
        assert!(!matches_advanced_filters(
            &r,
            &make_op(crate::sdk::types::VantaFilterOp::Gte, 51)
        ));
        assert!(matches_advanced_filters(
            &r,
            &make_op(crate::sdk::types::VantaFilterOp::Lt, 60)
        ));
        assert!(!matches_advanced_filters(
            &r,
            &make_op(crate::sdk::types::VantaFilterOp::Lt, 40)
        ));
        assert!(matches_advanced_filters(
            &r,
            &make_op(crate::sdk::types::VantaFilterOp::Lte, 50)
        ));
        assert!(!matches_advanced_filters(
            &r,
            &make_op(crate::sdk::types::VantaFilterOp::Lte, 49)
        ));
    }

    #[test]
    fn test_advanced_filter_missing_field_returns_false() {
        let r = make_record_with_meta(&[]);
        let ops = vec![crate::sdk::types::VantaMemoryFilterItem {
            field: "nonexistent".into(),
            op: crate::sdk::types::VantaFilterOp::Eq,
            value: VantaValue::String("x".into()),
        }];
        assert!(!matches_advanced_filters(&r, &ops));
    }

    #[test]
    fn test_advanced_filter_multi_and_logic() {
        let r = make_record_with_meta(&[
            ("score", VantaValue::Int(75)),
            ("status", VantaValue::String("active".into())),
        ]);
        let ops = vec![
            crate::sdk::types::VantaMemoryFilterItem {
                field: "score".into(),
                op: crate::sdk::types::VantaFilterOp::Gte,
                value: VantaValue::Int(50),
            },
            crate::sdk::types::VantaMemoryFilterItem {
                field: "status".into(),
                op: crate::sdk::types::VantaFilterOp::Eq,
                value: VantaValue::String("active".into()),
            },
        ];
        assert!(matches_advanced_filters(&r, &ops));

        // Falla si uno de los filtros no coincide
        let ops_fail = vec![
            crate::sdk::types::VantaMemoryFilterItem {
                field: "score".into(),
                op: crate::sdk::types::VantaFilterOp::Gte,
                value: VantaValue::Int(80),
            },
            crate::sdk::types::VantaMemoryFilterItem {
                field: "status".into(),
                op: crate::sdk::types::VantaFilterOp::Eq,
                value: VantaValue::String("active".into()),
            },
        ];
        assert!(!matches_advanced_filters(&r, &ops_fail));
    }

    // ─── sparse vector persistence (ADR-019) ────────────────────

    fn make_sparse(entries: &[(u32, f32)]) -> SparseVector {
        let mut sparse = SparseVector::new();
        for (dim, val) in entries {
            sparse.insert(*dim, *val);
        }
        sparse
    }

    fn record_with_sparse(sparse: Option<SparseVector>) -> VantaMemoryRecord {
        VantaMemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "payload".into(),
            metadata: VantaMemoryMetadata::new(),
            created_at_ms: 100,
            updated_at_ms: 200,
            version: 1,
            node_id: memory_node_id("ns", "k"),
            vector: None,
            sparse_vector: sparse,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        }
    }

    #[test]
    fn test_sparse_roundtrip_listfloat() {
        let record = record_with_sparse(Some(make_sparse(&[(1, 0.5), (42, -1.25), (7, 3.0)])));
        let (node, returned) = memory_record_to_node_owned(record);

        // Written as interleaved ListFloat pairs in sorted dim order (BTreeMap).
        assert_eq!(
            node.get_field(SPARSE_VECTOR_EXT_KEY),
            Some(&crate::node::FieldValue::ListFloat(vec![
                1.0, 0.5, 7.0, 3.0, 42.0, -1.25
            ]))
        );
        // Record returned to caller keeps the sparse vector in memory.
        assert_eq!(
            returned.sparse_vector,
            Some(make_sparse(&[(1, 0.5), (42, -1.25), (7, 3.0)]))
        );

        // Read path reconstructs the same vector.
        let read = memory_record_from_node(&node).unwrap();
        assert_eq!(
            read.sparse_vector,
            Some(make_sparse(&[(1, 0.5), (42, -1.25), (7, 3.0)]))
        );
    }

    #[test]
    fn test_sparse_empty_roundtrip() {
        let record = record_with_sparse(Some(SparseVector::new()));
        let (node, _) = memory_record_to_node_owned(record);
        assert_eq!(
            node.get_field(SPARSE_VECTOR_EXT_KEY),
            Some(&crate::node::FieldValue::ListFloat(vec![]))
        );
        let read = memory_record_from_node(&node).unwrap();
        assert_eq!(read.sparse_vector, Some(SparseVector::new()));
    }

    #[test]
    fn test_sparse_none_not_stored() {
        let record = record_with_sparse(None);
        let (node, _) = memory_record_to_node_owned(record);
        assert_eq!(node.get_field(SPARSE_VECTOR_EXT_KEY), None);
        let read = memory_record_from_node(&node).unwrap();
        assert_eq!(read.sparse_vector, None);
    }

    #[test]
    fn test_sparse_read_legacy_json_string() {
        // Simulate a node persisted before ADR-019 (String/JSON format).
        let mut node = make_memory_node(42, "myns", "mykey");
        node.set_field(
            SPARSE_VECTOR_EXT_KEY,
            crate::node::FieldValue::String("{\"1\":0.5,\"42\":-1.25}".into()),
        );
        let record = memory_record_from_node(&node).unwrap();
        assert_eq!(
            record.sparse_vector,
            Some(make_sparse(&[(1, 0.5), (42, -1.25)]))
        );
    }

    #[test]
    fn test_sparse_read_corrupt_legacy_returns_none() {
        let mut node = make_memory_node(42, "myns", "mykey");
        node.set_field(
            SPARSE_VECTOR_EXT_KEY,
            crate::node::FieldValue::String("not-json".into()),
        );
        let record = memory_record_from_node(&node).unwrap();
        assert_eq!(record.sparse_vector, None);
    }

    #[test]
    fn test_sparse_read_corrupt_listfloat_odd_len_returns_none() {
        let mut node = make_memory_node(42, "myns", "mykey");
        node.set_field(
            SPARSE_VECTOR_EXT_KEY,
            crate::node::FieldValue::ListFloat(vec![1.0, 0.5, 2.0]),
        );
        let record = memory_record_from_node(&node).unwrap();
        assert_eq!(record.sparse_vector, None);
    }

    #[test]
    fn test_sparse_read_corrupt_listfloat_invalid_dims_return_none() {
        // AUD-023 (P2-7): dims inválidas (NaN, negativa, out-of-range, no-entera)
        // deben rechazarse con None, no saturarse silenciosamente via `as u32`.
        let bad_payloads = [
            vec![f64::NAN, 0.5],      // NaN dim -> hoy satura a 0
            vec![f64::INFINITY, 0.5], // +inf dim -> hoy satura a u32::MAX
            vec![-1.0, 0.5],          // dim negativa -> hoy satura a 0
            vec![4294967296.0, 0.5],  // > u32::MAX -> hoy satura a u32::MAX
            vec![1.5, 0.5],           // dim no-entera -> hoy trunca a 1
        ];
        for flat in bad_payloads {
            let mut node = make_memory_node(42, "myns", "mykey");
            node.set_field(
                SPARSE_VECTOR_EXT_KEY,
                crate::node::FieldValue::ListFloat(flat),
            );
            let record = memory_record_from_node(&node).unwrap();
            assert_eq!(
                record.sparse_vector, None,
                "dims inválidas deben devolver None, no saturar silencioso"
            );
        }
    }
}
