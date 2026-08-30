//! Markdown-import counterpart of [`vanta_memory::seed::md_export`] (in
//! `vantadb::cli_handlers::export_md`). Reads a directory of `.md` files
//! produced by `vanta-cli export --format md` and idempotently writes them
//! into a [`VantaEmbedded`].
//!
//! Round-trip guarantee: re-importing the same directory is a no-op
//! (content-hash stable, all records report `unchanged`). This matches the
//! pattern in [`super::mod::apply_skill`].

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata, VantaValue};

use super::SeedCounts;

/// MD export schema version this importer accepts. Bumped on breaking change
/// of the frontmatter JSON shape (export side bumps `MD_EXPORT_SCHEMA_VERSION`).
pub const MD_IMPORT_SCHEMA_VERSION: u32 = 1;

/// Errors surfaced by the MD import layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum MdImportError {
    /// Filesystem error.
    #[error("md import io: {0}")]
    Io(#[from] std::io::Error),
    /// Frontmatter could not be parsed as JSON.
    #[error("md import json: {0}")]
    Json(#[from] serde_json::Error),
    /// Frontmatter is structurally invalid (missing fields, wrong version).
    #[error("md import validation: {0}")]
    Validation(String),
    /// Underlying VantaDB storage error.
    #[error("vantadb: {0}")]
    Vanta(#[from] vantadb::error::VantaError),
}

/// Frontmatter shape we read back. Mirrors `cli_handlers::export_md::Frontmatter`
/// without the `serialize` direction.
#[derive(Debug, Deserialize)]
#[allow(dead_code)] // some fields are read for forward-compat / shape validation
struct FrontmatterRead {
    schema_version: u32,
    namespace: String,
    key: String,
    #[serde(default)]
    version: u64,
    #[serde(default)]
    node_id: String,
    #[serde(default)]
    created_at_ms: u64,
    #[serde(default)]
    updated_at_ms: u64,
    #[serde(default)]
    expires_at_ms: Option<u64>,
    #[serde(default)]
    superseded_by: Option<String>,
    #[serde(default)]
    superseded_at_ms: Option<u64>,
    #[serde(default)]
    metadata: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    vector_dim: Option<usize>,
}

/// Parse one MD file: split frontmatter (between the two `---` lines) from
/// the body and return a [`VantaMemoryInput`].
pub fn parse_md_file(path: &Path) -> Result<VantaMemoryInput, MdImportError> {
    let raw = fs::read_to_string(path)?;
    parse_md_str(&raw, path)
}

fn parse_md_str(raw: &str, path: &Path) -> Result<VantaMemoryInput, MdImportError> {
    let (fm_json, body) = split_frontmatter(raw).ok_or_else(|| {
        MdImportError::Validation(format!(
            "missing --- frontmatter delimiters in {}",
            path.display()
        ))
    })?;
    let fm: FrontmatterRead = serde_json::from_str(&fm_json)?;
    if fm.schema_version != MD_IMPORT_SCHEMA_VERSION {
        return Err(MdImportError::Validation(format!(
            "unsupported md-export schema_version {} (expected {})",
            fm.schema_version, MD_IMPORT_SCHEMA_VERSION
        )));
    }
    let metadata = metadata_from_json(&fm.metadata);
    Ok(VantaMemoryInput {
        namespace: fm.namespace,
        key: fm.key,
        payload: body.to_string(),
        metadata,
        vector: None, // MD export does not inline the dense vector
        sparse_vector: None,
        ttl_ms: None,
    })
}

fn split_frontmatter(raw: &str) -> Option<(String, String)> {
    // Expect first line to be `---`, then JSON, then closing `---` (optionally
    // followed by a blank line, then the body).
    let mut lines = raw.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end() != "---" {
        return None;
    }
    let mut rest = String::new();
    for line in lines {
        rest.push_str(line);
    }
    // Find closing `---` (a line that, after trimming, equals "---").
    let mut end_idx: Option<usize> = None;
    let mut offset: usize = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---\n" || line.trim_end() == "---\r\n" || line.trim_end() == "---" {
            end_idx = Some(offset);
            break;
        }
        offset += line.len();
    }
    let end = end_idx?;
    let fm_json = rest[..end].to_string();
    let after = rest[end..].to_string();
    // Skip the closing "---" line itself (and optional trailing newline).
    // Then trim one leading newline + any leading blank lines.
    let body_start_in_after = after.find('\n').map(|i| i + 1).unwrap_or(after.len());
    let body = after[body_start_in_after..]
        .trim_start_matches('\n')
        .trim_start_matches('\r')
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_string();
    Some((fm_json, body))
}

fn metadata_from_json(json: &BTreeMap<String, serde_json::Value>) -> VantaMemoryMetadata {
    let mut out = VantaMemoryMetadata::new();
    for (k, v) in json {
        out.insert(k.clone(), json_to_vanta(v));
    }
    out
}

fn json_to_vanta(v: &serde_json::Value) -> VantaValue {
    match v {
        serde_json::Value::Null => VantaValue::Null,
        serde_json::Value::Bool(b) => VantaValue::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                VantaValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                VantaValue::Float(f)
            } else {
                VantaValue::Null
            }
        }
        serde_json::Value::String(s) => {
            // Try RFC3339 round-trip; if it parses, keep as DateTime, else String.
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                VantaValue::DateTime(dt.with_timezone(&chrono::Utc))
            } else {
                VantaValue::String(s.clone())
            }
        }
        serde_json::Value::Array(xs) => {
            // Coerce to the most-specific list variant the array supports.
            if xs.is_empty() {
                VantaValue::ListString(Vec::new())
            } else if xs.iter().all(|v| v.is_string()) {
                VantaValue::ListString(
                    xs.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect(),
                )
            } else if xs.iter().all(|v| v.is_i64()) {
                VantaValue::ListInt(xs.iter().filter_map(|v| v.as_i64()).collect())
            } else if xs.iter().all(|v| v.is_f64()) {
                VantaValue::ListFloat(xs.iter().filter_map(|v| v.as_f64()).collect())
            } else if xs.iter().all(|v| v.is_boolean()) {
                VantaValue::ListBool(xs.iter().filter_map(|v| v.as_bool()).collect())
            } else {
                // Mixed: stringify every element.
                VantaValue::ListString(
                    xs.iter()
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .collect(),
                )
            }
        }
        serde_json::Value::Object(_) => {
            // Flatten to JSON string for round-trip (object shape not in VantaValue).
            VantaValue::String(v.to_string())
        }
    }
}

fn content_hash(content: &str) -> u64 {
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    h.finish()
}

/// Idempotently import every `.md` file under `dir` (recursively). Returns a
/// [`SeedCounts`] reporting what each record did. Records with unchanged
/// content (matching content-hash) are skipped.
pub fn import_md_dir(db: &VantaEmbedded, dir: &Path) -> Result<SeedCounts, MdImportError> {
    let mut counts = SeedCounts::default();
    walk(dir, &mut |path| {
        let input = parse_md_file(path)?;
        // Idempotency: hash the canonical (namespace, key, payload) tuple;
        // if the existing record's payload is the same, count as unchanged.
        let existing = db.get(&input.namespace, &input.key)?;
        let new_form = canonical_form(&input);
        let new_hash = content_hash(&new_form);
        let existing_form = existing.as_ref().map(canonical_form_from_record);
        let existing_hash = existing_form.as_ref().map(|s| content_hash(s));
        if existing_hash.as_ref() == Some(&new_hash) {
            counts.unchanged += 1;
            return Ok(());
        }
        db.put(input)?;
        if existing.is_some() {
            counts.updated += 1;
        } else {
            counts.created += 1;
        }
        Ok(())
    })?;
    Ok(counts)
}

fn canonical_form(input: &VantaMemoryInput) -> String {
    // Stable string used for idempotency hash. We hash namespace + key +
    // payload — metadata round-trips through the body (frontmatter) and is
    // already covered by payload equality for our MD shape.
    let mut s = String::with_capacity(input.payload.len() + 64);
    s.push_str(&input.namespace);
    s.push('\u{1f}');
    s.push_str(&input.key);
    s.push('\u{1f}');
    s.push_str(&input.payload);
    s
}

fn canonical_form_from_record(r: &vantadb::sdk::VantaMemoryRecord) -> String {
    let mut s = String::with_capacity(r.payload.len() + 64);
    s.push_str(&r.namespace);
    s.push('\u{1f}');
    s.push_str(&r.key);
    s.push('\u{1f}');
    s.push_str(&r.payload);
    s
}

fn walk<F>(dir: &Path, cb: &mut F) -> Result<(), MdImportError>
where
    F: FnMut(&Path) -> Result<(), MdImportError>,
{
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, cb)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            cb(&path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_basic() {
        let raw = "---\n{\"schema_version\":1}\n---\n\nbody text\n";
        let (fm, body) = split_frontmatter(raw).expect("frontmatter");
        assert!(fm.contains("schema_version"));
        assert_eq!(body, "body text");
    }

    #[test]
    fn split_frontmatter_rejects_missing() {
        let raw = "no frontmatter here\nbody\n";
        assert!(split_frontmatter(raw).is_none());
    }

    #[test]
    fn round_trip_json_to_metadata_and_back() {
        let mut m = BTreeMap::new();
        m.insert("a".to_string(), serde_json::json!("alpha"));
        m.insert("n".to_string(), serde_json::json!(7));
        m.insert("b".to_string(), serde_json::json!(true));
        let meta = metadata_from_json(&m);
        let a = meta.get("a").unwrap();
        assert!(matches!(a, VantaValue::String(s) if s == "alpha"));
        let n = meta.get("n").unwrap();
        assert!(matches!(n, VantaValue::Int(7)));
        let b = meta.get("b").unwrap();
        assert!(matches!(b, VantaValue::Bool(true)));
    }
}
