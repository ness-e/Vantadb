//! Importers that convert third-party memory exports into VantaDB's
//! [interchange format](crate::sdk::MemoryExportLine) (JSONL v2).
//!
//! Three sources are supported — [`mem0`], [`zep`] and [`letta`] — each
//! exposing `convert_str` / `convert_file`. A conversion is pure (no database
//! handle, no network, no credentials): it maps a foreign export file to
//! `MemoryExportLine`s and reports what was converted, skipped and explicitly
//! discarded. The lines are then imported through the existing transport
//! (`Conversion::into_records` + `Embedded::import_records`, or
//! `Conversion::write_jsonl` + `vanta-cli import`).
//!
//! Design rules (ADR-046 §D7, VER-05):
//! - the interchange format is the existing `MemoryExportLine` v2 — never a
//!   new format;
//! - source ids become keys (deterministic → re-import updates, never
//!   duplicates); id-less records get a prefixed content hash key;
//! - every dropped source field is counted in [`ConversionStats::discarded`]
//!   and declared in the per-source mapping table (docs/api).

pub mod letta;
pub mod mem0;
pub mod zep;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use serde_json::Value as JsonValue;
use twox_hash::XxHash3_128;

use crate::error::{Error, Result};
use crate::sdk::types::{MemoryExportLine, MemoryMetadata, MemoryRecord, Value};

/// Outcome of converting one foreign export: the v2 lines in emission order
/// plus the mapping counters.
#[derive(Debug, Clone)]
pub struct Conversion {
    /// JSONL-ready [`MemoryExportLine`]s (`schema_version = 2`).
    pub lines: Vec<MemoryExportLine>,
    /// Mapping counters for this conversion.
    pub stats: ConversionStats,
}

impl Conversion {
    /// Rebuild import-ready records from the converted lines, validating them
    /// through the canonical transport (`record_from_export_line`).
    pub fn into_records(self) -> Result<Vec<MemoryRecord>> {
        self.lines
            .into_iter()
            .map(crate::sdk::serialization::record_from_export_line)
            .collect()
    }

    /// Write the lines as JSONL — the exact input format of
    /// `vanta-cli import` / `Embedded::import_file`.
    pub fn write_jsonl(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(Error::Io)?;
            }
        }
        let file = File::create(path).map_err(Error::Io)?;
        let mut writer = BufWriter::new(file);
        for line in &self.lines {
            serde_json::to_writer(&mut writer, line).map_err(Error::serialization)?;
            writer.write_all(b"\n").map_err(Error::Io)?;
        }
        writer.flush().map_err(Error::Io)?;
        Ok(())
    }
}

/// Per-source mapping counters. `converted` + `skipped` = records seen in the
/// export; `discarded` maps a source field (or declared field group) to the
/// number of times it was dropped because the v2 format has no destination
/// for it (see the per-source mapping tables in docs/api).
///
/// Counters are **record-level**. Envelope-level transport metadata a given
/// importer ignores (wrapper keys, root export metadata) is listed explicitly
/// in `docs/api/MEMORY_INTERCHANGE_FORMAT.md`; droppable *data* at envelope
/// level is counted here too (e.g. Zep `edges` when `facts` is also present).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversionStats {
    /// Source records mapped into a v2 line.
    pub converted: u64,
    /// Source records without the required textual payload (not importable).
    pub skipped: u64,
    /// Explicit discards: source field → occurrence count.
    pub discarded: BTreeMap<String, u64>,
}

impl ConversionStats {
    /// Count one explicit discard of `field`.
    pub(crate) fn discard(&mut self, field: &str) {
        *self.discarded.entry(field.to_string()).or_insert(0) += 1;
    }

    /// Count `count` explicit discards of `field` (no-op when `count == 0`).
    pub(crate) fn discard_many(&mut self, field: &str, count: usize) {
        if count > 0 {
            *self.discarded.entry(field.to_string()).or_insert(0) += count as u64;
        }
    }
}

// ── shared helpers (used by the three importers) ─────────────────────────

/// Read a source export file as UTF-8 text.
pub(crate) fn read_source_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(Error::Io)
}

/// Parse a source export as JSON, mapping the error to the crate error type.
pub(crate) fn parse_json(source: &str) -> Result<JsonValue> {
    serde_json::from_str(source).map_err(Error::serialization)
}

/// Sanitize a raw scope component into the namespace charset
/// (`A-Z a-z 0-9 . _ / -`, max 128 bytes — `validate_namespace`), falling back
/// to `fallback` when nothing survives.
pub(crate) fn sanitize_component(raw: &str) -> String {
    let out: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | '-') {
                c
            } else {
                '-'
            }
        })
        .collect();
    if out.is_empty() {
        "default".to_string()
    } else {
        out
    }
}

/// Build a ≤128-byte namespace from a prefix and a raw scope component.
pub(crate) fn scoped_namespace(prefix: &str, scope: Option<&str>) -> String {
    let mut namespace = match scope {
        Some(raw) => format!("{prefix}/{}", sanitize_component(raw)),
        None => prefix.to_string(),
    };
    namespace.truncate(128);
    namespace
}

/// Deterministic content-hash key for source records without an id.
/// `xxh3-128(namespace, payload)` — same family as `memory_node_id`, and the
/// same idea as the MD-import content hash (MEM-39): a re-import of the same
/// content maps to the same key, so it updates instead of duplicating.
pub(crate) fn content_key(prefix: &str, namespace: &str, payload: &str) -> String {
    let mut hasher = XxHash3_128::default();
    hasher.write(namespace.as_bytes());
    hasher.write(&[0]);
    hasher.write(payload.as_bytes());
    format!("{prefix}-{:032x}", hasher.finish_128())
}

/// Parse a timestamp in any of the shapes the three exports use:
/// RFC 3339 strings, date-only strings (`%Y-%m-%d`), numeric strings, and
/// numbers in nanoseconds / milliseconds / seconds (epoch thresholds:
/// ≥1e15 ns, ≥1e12 ms, ≥1e9 s; smaller values are read as ms).
/// Returns `None` for null/invalid/unparseable input (callers fall back to
/// the documented `0` sentinel or to `created_at_ms`).
pub(crate) fn timestamp_ms(value: &JsonValue) -> Option<u64> {
    match value {
        JsonValue::Number(n) => n.as_f64().and_then(numeric_to_ms),
        JsonValue::String(s) => {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                return Some(dt.timestamp_millis().max(0) as u64);
            }
            if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                let dt = date.and_hms_opt(0, 0, 0)?;
                return Some(dt.and_utc().timestamp_millis().max(0) as u64);
            }
            s.parse::<f64>().ok().and_then(numeric_to_ms)
        }
        _ => None,
    }
}

fn numeric_to_ms(n: f64) -> Option<u64> {
    if !n.is_finite() || n < 0.0 {
        return None;
    }
    let ms = if n >= 1e15 {
        n / 1e6
    } else if n >= 1e12 {
        n
    } else if n >= 1e9 {
        n * 1e3
    } else {
        n
    };
    Some(ms.round() as u64)
}

/// Convert a JSON value into the SDK's metadata [`Value`]:
/// strings/bools/ints/floats map natively, arrays of strings become
/// `ListString` (filterable), and nested objects/other arrays are preserved
/// as JSON text.
pub(crate) fn json_to_value(v: &JsonValue) -> Value {
    match v {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(u) = n.as_u64() {
                i64::try_from(u)
                    .map(Value::Int)
                    .unwrap_or(Value::Float(u as f64))
            } else {
                Value::Float(n.as_f64().unwrap_or_default())
            }
        }
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Array(items) => {
            if items.iter().all(JsonValue::is_string) {
                Value::ListString(
                    items
                        .iter()
                        .map(|i| i.as_str().unwrap_or_default().to_string())
                        .collect(),
                )
            } else {
                Value::String(JsonValue::Array(items.clone()).to_string())
            }
        }
        JsonValue::Object(_) => Value::String(v.to_string()),
    }
}

/// Insert a source field under `key` unless it is null/empty (null → absent).
pub(crate) fn insert_meta(metadata: &mut MemoryMetadata, key: &str, value: &JsonValue) {
    match value {
        JsonValue::Null => {}
        JsonValue::Array(items) if items.is_empty() => {}
        JsonValue::Object(obj) if obj.is_empty() => {}
        _ => {
            metadata.insert(key.to_string(), json_to_value(value));
        }
    }
}

/// First non-empty string among `names`, trimmed. Numbers are stringified
/// (some exports use numeric ids).
pub(crate) fn string_field(
    obj: &serde_json::Map<String, JsonValue>,
    names: &[&str],
) -> Option<String> {
    for name in names {
        match obj.get(*name) {
            Some(JsonValue::String(s)) if !s.trim().is_empty() => {
                return Some(s.trim().to_string());
            }
            Some(JsonValue::Number(n)) => return Some(n.to_string()),
            _ => {}
        }
    }
    None
}

/// Build a v2 line with the v1/v2 normalization already applied
/// (`valid_at_ms = created_at_ms`) so a convert → import → export roundtrip is
/// byte-stable (export never re-derives from a 0 sentinel).
pub(crate) fn base_line(
    namespace: String,
    key: String,
    payload: String,
    created_at_ms: u64,
    updated_at_ms: u64,
) -> MemoryExportLine {
    MemoryExportLine {
        schema_version: crate::sdk::serialization::EXPORT_SCHEMA_VERSION,
        namespace,
        key,
        payload,
        created_at_ms,
        updated_at_ms: updated_at_ms.max(created_at_ms),
        valid_at_ms: created_at_ms,
        ..Default::default()
    }
}

/// Preserve every non-consumed source field under `<prefix>.<field>`; count
/// the declared-discard fields as explicit discards.
pub(crate) fn preserve_unknown(
    obj: &serde_json::Map<String, JsonValue>,
    consumed: &[&str],
    discarded: &[&str],
    prefix: &str,
    metadata: &mut MemoryMetadata,
    stats: &mut ConversionStats,
) {
    for (key, value) in obj {
        if consumed.contains(&key.as_str()) {
            continue;
        }
        // null → absent (same semantics as `insert_meta`): nothing to count.
        if value.is_null() {
            continue;
        }
        if discarded.contains(&key.as_str()) {
            stats.discard(key);
            continue;
        }
        insert_meta(metadata, &format!("{prefix}.{key}"), value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn timestamp_ms_accepts_rfc3339_date_only_and_epoch_shapes() {
        assert_eq!(
            timestamp_ms(&json!("2025-11-03T09:12:00+00:00")),
            Some(1_762_161_120_000)
        );
        assert_eq!(timestamp_ms(&json!("2026-01-15")), Some(1_768_435_200_000));
        assert_eq!(
            timestamp_ms(&json!(1_761_253_920u64)),
            Some(1_761_253_920_000)
        );
        assert_eq!(
            timestamp_ms(&json!(1_761_253_920_000u64)),
            Some(1_761_253_920_000)
        );
        assert_eq!(timestamp_ms(&json!("1761253920")), Some(1_761_253_920_000));
        assert_eq!(timestamp_ms(&json!(null)), None);
        assert_eq!(timestamp_ms(&json!("not a date")), None);
    }

    #[test]
    fn sanitize_component_replaces_illegal_bytes_and_falls_back() {
        assert_eq!(sanitize_component("alice"), "alice");
        assert_eq!(sanitize_component("alice@example.com"), "alice-example.com");
        assert_eq!(sanitize_component("with space"), "with-space");
        assert_eq!(sanitize_component("***"), "---");
        assert_eq!(sanitize_component(""), "default");
    }

    #[test]
    fn content_key_is_deterministic_and_namespace_scoped() {
        let a = content_key("mem0", "mem0/alice", "same payload");
        let b = content_key("mem0", "mem0/alice", "same payload");
        let c = content_key("mem0", "mem0/bob", "same payload");
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert!(a.starts_with("mem0-"));
    }

    #[test]
    fn json_to_value_preserves_string_lists_and_nested_objects() {
        assert_eq!(
            json_to_value(&json!(["a", "b"])),
            Value::ListString(vec!["a".into(), "b".into()])
        );
        assert_eq!(json_to_value(&json!(true)), Value::Bool(true));
        assert_eq!(json_to_value(&json!(7)), Value::Int(7));
        match json_to_value(&json!({"k": "v"})) {
            Value::String(raw) => assert!(raw.contains("\"k\"")),
            other => panic!("object should be a JSON string, got {other:?}"),
        }
    }
}
