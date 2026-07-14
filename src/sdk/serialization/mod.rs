//! Wire-format helpers for reading and writing VantaDB memory records
//! to/from internal node representations and JSONL export lines.

#[cfg(test)]
use super::builder::VantaEmbedded;
use super::types::*;
use crate::error::{Result, VantaError};
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
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
const EXPORT_SCHEMA_VERSION: u32 = 1;
const DERIVED_INDEX_SCHEMA_VERSION: u32 = 1;
pub(crate) const DERIVED_INDEX_STATE_KEY: &[u8] = b"derived_index_state";
pub(crate) const TEXT_INDEX_STATE_KEY: &[u8] = b"text_index_state";

pub(crate) mod conversions;
pub mod graph_types;
pub(crate) mod impl_export;
pub(crate) mod impl_index;
pub(crate) mod impl_rebuild;
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

pub fn memory_record_from_node(node: UnifiedNode) -> Option<VantaMemoryRecord> {
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

/// Convert a `VantaMemoryRecord` into a JSONL export line with schema version.
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
}
