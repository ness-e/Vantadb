//! Mem0 → VantaDB importer.
//!
//! Accepts the shapes a Mem0 user can export without credentials:
//! - a JSON array of memory objects (`get_all()` results serialized),
//! - `{"results": [...]}` (the OSS `get_all` envelope),
//! - `{"data": [...]}` (the official CLI `--agent` envelope).
//!
//! Mapping table and declared discards: `docs/api/MEMORY_INTERCHANGE_FORMAT.md`.

use serde_json::Value as JsonValue;

use super::{
    base_line, content_key, insert_meta, parse_json, read_source_file, scoped_namespace,
    string_field, timestamp_ms, Conversion, ConversionStats,
};
use crate::error::{Error, Result};
use crate::sdk::types::Value;
use std::path::Path;

/// Source fields consumed by the mapping (everything else is preserved under
/// the `mem0.` metadata prefix, except the search-time fields below).
const CONSUMED: &[&str] = &[
    "memory",
    "text",
    "content",
    "data",
    "id",
    "memory_id",
    "uuid",
    "created_at",
    "updated_at",
    "metadata",
    "categories",
    "hash",
    "user_id",
    "agent_id",
    "run_id",
    "app_id",
];

/// Retrieval-time fields with no v2 destination: discarded and counted.
const SEARCH_TIME: &[&str] = &["score", "relevance", "selection_rank"];

/// Payload aliases accepted (documented: `memory` is canonical; the CLI's
/// import format also accepts `text`/`content`, and some dumps use `data`).
const PAYLOAD_ALIASES: &[&str] = &["memory", "text", "content", "data"];
const ID_ALIASES: &[&str] = &["id", "memory_id", "uuid"];
const SCOPE_FIELDS: &[&str] = &["user_id", "agent_id", "run_id", "app_id"];

/// Convert a Mem0 memories export (JSON text) into v2 interchange lines.
pub fn convert_str(source: &str) -> Result<Conversion> {
    let root = parse_json(source)?;
    let items = memory_items(&root)?;
    let mut stats = ConversionStats::default();
    let mut lines = Vec::with_capacity(items.len());

    for item in items {
        let Some(obj) = item.as_object() else {
            stats.skipped += 1;
            continue;
        };
        let Some(payload) = string_field(obj, PAYLOAD_ALIASES) else {
            stats.skipped += 1;
            continue;
        };

        let id = string_field(obj, ID_ALIASES);
        let scope = string_field(obj, SCOPE_FIELDS);
        let namespace = scoped_namespace("mem0", scope.as_deref());
        let key = match &id {
            Some(id) => id.clone(),
            None => content_key("mem0", &namespace, &payload),
        };

        let created_at_ms = obj.get("created_at").and_then(timestamp_ms).unwrap_or(0);
        let updated_at_ms = obj
            .get("updated_at")
            .and_then(timestamp_ms)
            .unwrap_or(created_at_ms);

        let mut line = base_line(namespace, key, payload, created_at_ms, updated_at_ms);
        line.metadata
            .insert("source".to_string(), Value::String("mem0".to_string()));
        if let Some(id) = &id {
            line.metadata
                .insert("source_id".to_string(), Value::String(id.clone()));
        }
        for field in SCOPE_FIELDS {
            if let Some(value) = obj.get(*field) {
                insert_meta(&mut line.metadata, field, value);
            }
        }
        for field in ["hash", "categories"] {
            if let Some(value) = obj.get(field) {
                insert_meta(&mut line.metadata, &format!("mem0.{field}"), value);
            }
        }
        if let Some(meta) = obj.get("metadata") {
            insert_meta(&mut line.metadata, "mem0.metadata", meta);
        }
        for (key, value) in obj {
            if CONSUMED.contains(&key.as_str()) {
                continue;
            }
            if SEARCH_TIME.contains(&key.as_str()) {
                stats.discard(key);
                continue;
            }
            insert_meta(&mut line.metadata, &format!("mem0.{key}"), value);
        }

        lines.push(line);
        stats.converted += 1;
    }

    Ok(Conversion { lines, stats })
}

/// Convert a Mem0 export file into v2 interchange lines.
pub fn convert_file(path: impl AsRef<Path>) -> Result<Conversion> {
    convert_str(&read_source_file(path.as_ref())?)
}

fn memory_items(root: &JsonValue) -> Result<Vec<&JsonValue>> {
    if let Some(array) = root.as_array() {
        return Ok(array.iter().collect());
    }
    if let Some(obj) = root.as_object() {
        for key in ["results", "data", "memories"] {
            if let Some(array) = obj.get(key).and_then(JsonValue::as_array) {
                return Ok(array.iter().collect());
            }
        }
    }
    Err(Error::Validation {
        field: "root".into(),
        reason: "expected a JSON array of memories or an object with a `results`, `data` or `memories` array".into(),
    })
}
