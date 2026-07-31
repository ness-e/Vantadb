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
pub(crate) const DERIVED_INDEX_SCHEMA_VERSION: u32 = 1;
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

pub fn memory_record_from_node(node: &UnifiedNode) -> Option<VantaMemoryRecord> {
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

    let vector = match &node.vector {
        VectorRepresentations::Full(vector) => Some(vector.clone()),
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

    // ponytail: iterar por referencia, no clonar todo el HashMap solo para leerlo
    for (k, v) in &metadata {
        node.set_field(k.clone(), v.clone().into());
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

    // ─── now_ms ────────────────────────────────────────────────

    #[test]
    fn test_now_ms_non_zero() {
        let t = now_ms();
        assert!(
            t > 1_700_000_000_000,
            "expected reasonable Unix ms, got {t}"
        );
    }

    // ─── memory_node_id ────────────────────────────────────────

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

    // ─── validate_namespace ────────────────────────────────────

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

    // ─── validate_key ──────────────────────────────────────────

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
        assert!(validate_key("你好").is_ok());
    }

    // ─── validate_metadata ─────────────────────────────────────

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

    // ─── namespace_index_key / namespace_index_prefix ──────────

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

    // ─── encoded_scalar_value ──────────────────────────────────

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

    // ─── payload_index_prefix / payload_index_key ──────────────

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

    // ─── node_id_bytes / decode_node_id ────────────────────────

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

    // ─── get_string_field / get_u64_field ──────────────────────

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

    // ─── memory_record_from_node ───────────────────────────────

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

    // ─── memory_record_to_node_owned ───────────────────────────

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
            expires_at_ms: None,
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
            expires_at_ms: Some(999_999_999_999),
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
            expires_at_ms: None,
        };
        let (node, returned) = memory_record_to_node_owned(record);
        assert!(returned.vector.is_none());
        // Empty vector should not set HAS_VECTOR flag
        let expected = crate::node::VectorRepresentations::None;
        assert_eq!(node.vector, expected);
    }

    // ─── export_line_from_record / record_from_export_line ─────

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
            expires_at_ms: None,
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
            created_at_ms: 0,
            updated_at_ms: 0,
            version: 0,
            expires_at_ms: None,
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
            created_at_ms: 100,
            updated_at_ms: 200,
            version: 5,
            expires_at_ms: None,
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
    }

    // ─── matches_memory_filters ────────────────────────────────

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
            expires_at_ms: None,
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
            expires_at_ms: None,
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
            expires_at_ms: None,
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

    // ── matches_advanced_filters ──────────────────────────────────────────────

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
            expires_at_ms: None,
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
}
