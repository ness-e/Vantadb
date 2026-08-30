//! Export records to a git-friendly directory of Markdown files (one file
//! per record) with JSON-in-frontmatter metadata. Round-trips with
//! `vanta-memory::seed::md_import`.
//!
//! Layout (relative to `--out-dir`):
//! ```text
//! <out-dir>/
//!   index.json                       # aggregate metadata + per-file checksums
//!   <sanitized-namespace>/<sanitized-key>.md
//! ```
//!
//! Filenames sanitize `/`, `\`, `..`, `\0` and clamp to 200 chars to be
//! filesystem-safe across Windows + Unix. See [`sanitize_component`].

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::cli_handlers::{
    create_spinner, open_embedded, print_info, print_success, print_warning,
};
use crate::error::Result;
use crate::sdk::VantaMemoryRecord;

/// Stable schema version for the MD export frontmatter. Bump on breaking change
/// of the frontmatter shape (import side refuses unknown versions).
pub const MD_EXPORT_SCHEMA_VERSION: u32 = 1;

/// Sanitize a namespace or key for use as a filesystem path component.
///
/// - Replaces `/`, `\`, `..` (path traversal) and control chars with `_`.
/// - Replaces NUL with `_` (Rust strings can carry `\0`; some FS reject it).
/// - Clamps length to 200 chars to stay under typical path limits.
/// - Empty input → `_` (every directory needs a name).
pub fn sanitize_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch == '/' || ch == '\\' || ch == '\0' || ch.is_control() {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    let trimmed = out.trim_matches('.').trim();
    if trimmed.is_empty() {
        return "_".to_string();
    }
    // Clamp to 200 chars (UTF-8 safe by char count, not bytes).
    let truncated: String = trimmed.chars().take(200).collect();
    truncated
}

/// Frontmatter payload written at the top of each MD file. JSON-encoded so
/// the format is fully described by the schema and the import path doesn't
/// need a YAML parser at runtime.
#[derive(Debug, Serialize)]
struct Frontmatter<'a> {
    schema_version: u32,
    namespace: &'a str,
    key: &'a str,
    version: u64,
    node_id: String,
    created_at_ms: u64,
    updated_at_ms: u64,
    expires_at_ms: Option<u64>,
    superseded_by: Option<&'a str>,
    superseded_at_ms: Option<u64>,
    metadata: BTreeMap<String, serde_json::Value>,
    vector_dim: Option<usize>,
}

/// Per-file entry recorded in `index.json`.
#[derive(Debug, Serialize)]
struct IndexEntry {
    namespace: String,
    key: String,
    file: String,
    hash: u64,
    bytes: u64,
}

#[derive(Debug, Serialize)]
struct IndexFile {
    schema_version: u32,
    generated_at_ms: u64,
    record_count: u64,
    records: Vec<IndexEntry>,
}

fn vector_dim_to_json(vector: Option<&Vec<f32>>) -> Option<usize> {
    vector.map(|v| v.len())
}

/// Render a single record to a Markdown string with JSON frontmatter.
fn render_record_md(record: &VantaMemoryRecord) -> String {
    let fm = Frontmatter {
        schema_version: MD_EXPORT_SCHEMA_VERSION,
        namespace: &record.namespace,
        key: &record.key,
        version: record.version,
        node_id: record.node_id.to_string(),
        created_at_ms: record.created_at_ms,
        updated_at_ms: record.updated_at_ms,
        expires_at_ms: record.expires_at_ms,
        superseded_by: record.superseded_by.as_deref(),
        superseded_at_ms: record.superseded_at_ms,
        metadata: metadata_to_json(&record.metadata),
        vector_dim: vector_dim_to_json(record.vector.as_ref()),
    };
    let fm_json = serde_json::to_string(&fm).unwrap_or_else(|_| "{}".to_string());
    let mut body = String::with_capacity(fm_json.len() + record.payload.len() + 32);
    body.push_str("---\n");
    body.push_str(&fm_json);
    body.push_str("\n---\n\n");
    body.push_str(&record.payload);
    if !record.payload.ends_with('\n') {
        body.push('\n');
    }
    body
}

fn metadata_to_json(
    metadata: &crate::sdk::VantaMemoryMetadata,
) -> BTreeMap<String, serde_json::Value> {
    let mut out = BTreeMap::new();
    for (k, v) in metadata.iter() {
        out.insert(k.clone(), metadata_value_to_json(v));
    }
    out
}

fn metadata_value_to_json(v: &crate::sdk::VantaValue) -> serde_json::Value {
    use crate::sdk::VantaValue;
    match v {
        VantaValue::Null => serde_json::Value::Null,
        VantaValue::Bool(b) => serde_json::Value::Bool(*b),
        VantaValue::Int(i) => serde_json::Value::from(*i),
        VantaValue::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        VantaValue::String(s) => serde_json::Value::String(s.clone()),
        VantaValue::DateTime(dt) => serde_json::Value::String(dt.to_rfc3339()),
        VantaValue::ListString(xs) => serde_json::Value::from(xs.clone()),
        VantaValue::ListInt(xs) => serde_json::Value::from(xs.clone()),
        VantaValue::ListFloat(xs) => serde_json::Value::from(xs.clone()),
        VantaValue::ListBool(xs) => serde_json::Value::from(xs.clone()),
        VantaValue::ListDateTime(xs) => {
            serde_json::Value::from(xs.iter().map(|dt| dt.to_rfc3339()).collect::<Vec<_>>())
        }
    }
}

fn hash_bytes(data: &[u8]) -> u64 {
    let mut h = DefaultHasher::new();
    data.hash(&mut h);
    h.finish()
}

/// Export all records in the given namespaces to a directory of MD files
/// under `out_dir`. Creates `out_dir` if missing.
pub fn cmd_export_md(db_path: &str, namespace: Option<&str>, out_dir: &str) -> Result<()> {
    let spinner = create_spinner("Opening database...");
    let embedded = open_embedded(db_path, true)?;
    spinner.finish_and_clear();

    let out_path = PathBuf::from(out_dir);
    fs::create_dir_all(&out_path)?;

    let namespaces: Vec<String> = match namespace {
        Some(ns) => vec![ns.to_string()],
        None => embedded.list_namespaces()?,
    };

    const BATCH_SIZE: usize = 500;
    let mut total: u64 = 0;
    let mut index_entries: Vec<IndexEntry> = Vec::new();

    for ns in &namespaces {
        let ns_safe = sanitize_component(ns);
        let ns_dir = out_path.join(&ns_safe);
        fs::create_dir_all(&ns_dir)?;

        let mut cursor: Option<usize> = None;
        loop {
            let opts = crate::sdk::VantaMemoryListOptions {
                #[allow(deprecated)]
                filters: crate::sdk::VantaMemoryMetadata::new(),
                filter_ops: None,
                limit: BATCH_SIZE,
                cursor,
                exclude_superseded: false,
            };
            let page = embedded.list(ns, opts)?;
            if page.records.is_empty() {
                break;
            }
            for record in &page.records {
                let body = render_record_md(record);
                let key_safe = sanitize_component(&record.key);
                let file_path = ns_dir.join(format!("{key_safe}.md"));

                let bytes = body.as_bytes();
                let hash = hash_bytes(bytes);
                let file_str = format!("{ns_safe}/{key_safe}.md");

                let mut f = fs::File::create(&file_path)?;
                f.write_all(bytes)?;
                f.flush()?;

                index_entries.push(IndexEntry {
                    namespace: record.namespace.clone(),
                    key: record.key.clone(),
                    file: file_str,
                    hash,
                    bytes: bytes.len() as u64,
                });
            }
            total += page.records.len() as u64;
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
    }

    // Aggregate index with per-file hashes (collision-resistant for change
    // detection, not cryptographic — std `DefaultHasher` is enough here).
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let index = IndexFile {
        schema_version: MD_EXPORT_SCHEMA_VERSION,
        generated_at_ms: now_ms,
        record_count: total,
        records: index_entries,
    };
    let index_json =
        serde_json::to_string_pretty(&index).map_err(crate::error::VantaError::serialization)?;
    let index_path = out_path.join("index.json");
    let mut index_file = fs::File::create(&index_path)?;
    index_file.write_all(index_json.as_bytes())?;
    index_file.flush()?;

    if total == 0 {
        print_warning("No records to export");
    } else {
        print_success(&format!(
            "Exported {total} records to {} (MD, git-friendly)",
            out_path.display()
        ));
        print_info(&format!("Index: {}", index_path.display()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_separators() {
        assert_eq!(sanitize_component("a/b\\c..d"), "a_b_c..d");
        assert_eq!(sanitize_component(""), "_");
        assert_eq!(sanitize_component("..."), "_");
    }

    #[test]
    fn sanitize_clamps_long_input() {
        let raw = "x".repeat(500);
        let out = sanitize_component(&raw);
        assert_eq!(out.chars().count(), 200);
    }

    #[test]
    fn render_record_md_has_frontmatter() {
        let rec = VantaMemoryRecord {
            namespace: "agent/team".into(),
            key: "k/1".into(),
            payload: "hello".into(),
            metadata: crate::sdk::VantaMemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 1,
            node_id: 42,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
        };
        let body = render_record_md(&rec);
        assert!(body.starts_with("---\n"));
        assert!(body.contains("\"namespace\": \"agent/team\""));
        assert!(body.contains("\"schema_version\": 1"));
        assert!(body.ends_with("hello\n"));
    }
}
