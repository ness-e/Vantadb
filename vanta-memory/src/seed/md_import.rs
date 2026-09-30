//! Markdown-import counterpart of `vantadb::cli_handlers::export_md`. Reads a
//! directory of `.md` files produced by `vanta-cli export --format md` and
//! idempotently writes them into a [`Embedded`].
//!
//! Round-trip guarantee: re-importing the same directory is a no-op
//! (projection-stable, all records report `unchanged`). This matches the
//! pattern in [`super::mod::apply_skill`].
//!
//! Schema v2 (VER-06): the importer accepts `1..=2`. v2 frontmatter carries the
//! ADR-046 semantics (validity window, confidence class/score, derivation,
//! quarantine) and round-trips them through the canonical transport
//! (`MemoryExportLine` → `record_from_export_line`, ADR-046 §D7) — v1 files are
//! normalized by the same function, so old exports keep importing.
//!
//! Informational related block: records with `superseded_by` / `derived_from`
//! are exported with a trailing `<!-- vanta:links -->` + `## Related` block of
//! `[[namespace/key]]` wikilinks. The importer strips that block so it never
//! reaches the payload, and counts links without a destination in
//! [`SeedCounts::links_unresolved`] (best-effort; never an import failure).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

use vantadb::sdk::{
    default_confidence, record_from_export_line, ConfidenceClass, Embedded, MemoryExportLine,
    MemoryMetadata, MemoryRecord, Value,
};

use super::SeedCounts;

/// MD export schema version this importer accepts (as a range: `1..=` this).
/// Bumped on breaking change of the frontmatter JSON shape (export side bumps
/// `MD_EXPORT_SCHEMA_VERSION`).
pub const MD_IMPORT_SCHEMA_VERSION: u32 = 2;

/// Marker that opens the informational related block appended by the export.
/// Keep in sync with `vantadb::cli_handlers::export_md` (separate crates).
pub(crate) const LINKS_MARKER: &str = "<!-- vanta:links -->";

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
    Vanta(#[from] vantadb::error::Error),
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
    // ── v2 (VER-06): absent in v1 files → normalization defaults ──
    #[serde(default)]
    valid_at_ms: u64,
    #[serde(default)]
    invalid_at_ms: Option<u64>,
    #[serde(default)]
    confidence_class: Option<ConfidenceClass>,
    #[serde(default = "default_confidence")]
    confidence: f32,
    #[serde(default)]
    last_validated_at_ms: Option<u64>,
    #[serde(default)]
    derived_from: Vec<String>,
    #[serde(default)]
    quarantined_at_ms: Option<u64>,
    #[serde(default)]
    quarantine_reason: Option<String>,
    #[serde(default)]
    quarantined_by: Option<String>,
    #[serde(default)]
    quarantine_review_due_ms: Option<u64>,
}

/// One parsed MD document: the record to import plus the informational
/// wikilink targets found in the trailing related block.
struct ParsedMd {
    record: MemoryRecord,
    /// `namespace/key` targets from `[[namespace/key]]` wikilinks (verbatim;
    /// informational — resolved best-effort after import).
    links: Vec<String>,
}

/// Parse one MD file: split frontmatter (between the two `---` lines) from
/// the body and return the record it describes.
fn parse_md_file(path: &Path) -> Result<ParsedMd, MdImportError> {
    let raw = fs::read_to_string(path)?;
    parse_md_str(&raw, path)
}

fn parse_md_str(raw: &str, path: &Path) -> Result<ParsedMd, MdImportError> {
    let (fm_json, body) = split_frontmatter(raw).ok_or_else(|| {
        MdImportError::Validation(format!(
            "missing --- frontmatter delimiters in {}",
            path.display()
        ))
    })?;
    let fm: FrontmatterRead = serde_json::from_str(&fm_json)?;
    if !(1..=MD_IMPORT_SCHEMA_VERSION).contains(&fm.schema_version) {
        return Err(MdImportError::Validation(format!(
            "unsupported md-export schema_version {} (supported: 1..={})",
            fm.schema_version, MD_IMPORT_SCHEMA_VERSION
        )));
    }
    let (payload, links) = split_related_block(&body);
    let line = MemoryExportLine {
        schema_version: fm.schema_version,
        namespace: fm.namespace,
        key: fm.key,
        payload: payload.to_string(),
        metadata: metadata_from_json(&fm.metadata),
        vector: None,        // MD export does not inline the dense vector
        sparse_vector: None, // ... nor the sparse one
        created_at_ms: fm.created_at_ms,
        updated_at_ms: fm.updated_at_ms,
        version: fm.version,
        expires_at_ms: fm.expires_at_ms,
        superseded_by: fm.superseded_by,
        superseded_at_ms: fm.superseded_at_ms,
        valid_at_ms: fm.valid_at_ms,
        invalid_at_ms: fm.invalid_at_ms,
        confidence_class: fm.confidence_class.unwrap_or_default(),
        confidence: fm.confidence,
        last_validated_at_ms: fm.last_validated_at_ms,
        derived_from: fm.derived_from,
        quarantined_at_ms: fm.quarantined_at_ms,
        quarantine_reason: fm.quarantine_reason,
        quarantined_by: fm.quarantined_by,
        quarantine_review_due_ms: fm.quarantine_review_due_ms,
    };
    // Single canonical transport: ADR-046 §D7 v1/v2 normalization + validation
    // shared with the JSONL importers (no second schema implementation).
    let record = record_from_export_line(line)?;
    Ok(ParsedMd { record, links })
}

fn split_frontmatter(raw: &str) -> Option<(String, String)> {
    // Normalize CRLF → LF first: the documented git flow edits `.md` files in
    // a text editor, and a Windows editor may save every line as `\r\n`.
    // Without this, the body would keep a leading `\n` (the `\n`-then-`\r`
    // trim order does not collapse the pair) and the payload projection would
    // drift against an LF-only store.
    let normalized = raw.replace("\r\n", "\n");
    let raw = normalized.as_str();
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

/// Split the informational related block appended by the export from the
/// payload body.
///
/// Only a canonical trailing block is stripped: the marker followed by
/// `## Related` and `- [[...]]` bullets. A marker occurrence that does not open
/// that shape (e.g. literal text inside user payload) is left untouched.
///
/// ponytail: bullets are collected best-effort — non-bullet lines inside the
/// block are ignored rather than treated as payload. The block is a
/// projection (regenerated on every export), never a source of joins.
fn split_related_block(body: &str) -> (&str, Vec<String>) {
    let Some(idx) = body.rfind(LINKS_MARKER) else {
        return (body, Vec::new());
    };
    let tail = &body[idx + LINKS_MARKER.len()..];
    let Some(tail) = tail.strip_prefix("\n\n## Related\n") else {
        return (body, Vec::new());
    };
    let links = tail
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("- [[")
                .and_then(|s| s.strip_suffix("]]"))
                .map(str::to_string)
        })
        .collect();
    (body[..idx].trim_end(), links)
}

fn metadata_from_json(json: &BTreeMap<String, serde_json::Value>) -> MemoryMetadata {
    let mut out = MemoryMetadata::new();
    for (k, v) in json {
        out.insert(k.clone(), json_to_vanta(v));
    }
    out
}

fn json_to_vanta(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => {
            // Try RFC3339 round-trip; if it parses, keep as DateTime, else String.
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                Value::DateTime(dt.with_timezone(&chrono::Utc))
            } else {
                Value::String(s.clone())
            }
        }
        serde_json::Value::Array(xs) => {
            // Coerce to the most-specific list variant the array supports.
            if xs.is_empty() {
                Value::ListString(Vec::new())
            } else if xs.iter().all(|v| v.is_string()) {
                Value::ListString(
                    xs.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect(),
                )
            } else if xs.iter().all(|v| v.is_i64()) {
                Value::ListInt(xs.iter().filter_map(|v| v.as_i64()).collect())
            } else if xs.iter().all(|v| v.is_f64()) {
                Value::ListFloat(xs.iter().filter_map(|v| v.as_f64()).collect())
            } else if xs.iter().all(|v| v.is_boolean()) {
                Value::ListBool(xs.iter().filter_map(|v| v.as_bool()).collect())
            } else {
                // Mixed: stringify every element.
                Value::ListString(
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
            // Flatten to JSON string for round-trip (object shape not in Value).
            Value::String(v.to_string())
        }
    }
}

/// Idempotently import every `.md` file under `dir` (recursively). Returns a
/// [`SeedCounts`] reporting what each record did. Records whose MD projection
/// (payload, metadata, v2 semantics) is identical to the stored record are
/// skipped (`unchanged`).
///
/// Writes go through the canonical transport — `MemoryExportLine` →
/// `record_from_export_line` → [`Embedded::import_records`] (ADR-046 §D7) — so
/// the validity window, confidence, derivation and quarantine state round-trip
/// verbatim and every write is validated at the core boundary.
pub fn import_md_dir(db: &Embedded, dir: &Path) -> Result<SeedCounts, MdImportError> {
    let mut counts = SeedCounts::default();
    let mut changed: Vec<MemoryRecord> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    walk(dir, &mut |path| {
        let parsed = parse_md_file(path)?;
        links.extend(parsed.links.iter().cloned());
        let candidate = parsed.record;
        // Idempotency: compare the full MD projection against the stored
        // record. Vectors are excluded because the MD format does not carry
        // them (re-importing must not wipe an existing vector).
        if let Some(existing) = db.get(&candidate.namespace, &candidate.key)? {
            if same_projection(&existing, &candidate) {
                counts.unchanged += 1;
                return Ok(());
            }
        }
        changed.push(candidate);
        Ok(())
    })?;

    if !changed.is_empty() {
        // `import_records` validates each record (class/confidence/window/
        // node id, ADR-046 §D4b) and rebuilds derived + text indexes.
        let report = db.import_records(changed, false)?;
        counts.created += report.inserted as usize;
        counts.updated += report.updated as usize;
        if report.errors > 0 {
            return Err(MdImportError::Validation(format!(
                "{} record(s) failed validation during md import",
                report.errors
            )));
        }
    }

    // Best-effort destination check for the informational wikilinks, after all
    // imports landed (a link may target a record from another file). Counts
    // link occurrences, not distinct targets: a target repeated across the
    // block (or across files) counts once per occurrence.
    for target in &links {
        if !link_target_exists(db, target)? {
            counts.links_unresolved += 1;
        }
    }
    Ok(counts)
}

/// True when the stored record and the MD-derived candidate describe the same
/// state once the non-MD fields (vectors) are zeroed.
fn same_projection(existing: &MemoryRecord, candidate: &MemoryRecord) -> bool {
    let mut a = existing.clone();
    let mut b = candidate.clone();
    a.vector = None;
    a.sparse_vector = None;
    b.vector = None;
    b.sparse_vector = None;
    a == b
}

/// Resolve `[[namespace/key]]` best-effort: namespaces and keys may both
/// contain `/`, so try every split point and accept the first pair that names
/// an existing record.
fn link_target_exists(db: &Embedded, target: &str) -> Result<bool, MdImportError> {
    for (i, _) in target.match_indices('/') {
        let (ns, rest) = target.split_at(i);
        let key = &rest[1..];
        if !ns.is_empty() && !key.is_empty() && db.get(ns, key)?.is_some() {
            return Ok(true);
        }
    }
    Ok(false)
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
    use vantadb::config::Config;
    use vantadb::storage::BackendKind;

    fn open_db() -> Embedded {
        Embedded::open_with_config(Config {
            backend_kind: BackendKind::InMemory,
            read_only: false,
            ..Config::default()
        })
        .expect("open in-memory db")
    }

    fn write_md(dir: &Path, rel: &str, content: &str) {
        let full = dir.join(rel);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("mkdir parent");
        }
        fs::write(&full, content).expect("write md");
    }

    #[test]
    fn split_frontmatter_basic() {
        let raw = "---\n{\"schema_version\":2}\n---\n\nbody text\n";
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
        assert!(matches!(a, Value::String(s) if s == "alpha"));
        let n = meta.get("n").unwrap();
        assert!(matches!(n, Value::Int(7)));
        let b = meta.get("b").unwrap();
        assert!(matches!(b, Value::Bool(true)));
    }

    #[test]
    fn split_related_block_strips_canonical_block() {
        let body = "hello\n\n<!-- vanta:links -->\n\n## Related\n\n- [[ns/a]]\n- [[ns/b]]\n";
        let (payload, links) = split_related_block(body);
        assert_eq!(payload, "hello");
        assert_eq!(links, vec!["ns/a".to_string(), "ns/b".to_string()]);
    }

    #[test]
    fn split_related_block_leaves_payload_marker_untouched() {
        // Marker without the canonical `## Related` shape → payload verbatim.
        let body = "payload mentions <!-- vanta:links --> mid-text";
        let (payload, links) = split_related_block(body);
        assert_eq!(payload, body);
        assert!(links.is_empty());
    }

    #[test]
    fn import_md_dir_accepts_v2_and_preserves_semantics() {
        let db = open_db();
        let tmp = tempfile::tempdir().expect("tempdir");
        let md = "---\n{\"schema_version\":2,\"namespace\":\"ns\",\"key\":\"k1\",\
\"version\":2,\"node_id\":\"0\",\"created_at_ms\":1000,\"updated_at_ms\":2000,\
\"expires_at_ms\":null,\"superseded_by\":\"k2\",\"superseded_at_ms\":1800,\
\"metadata\":{\"author\":\"alice\"},\"vector_dim\":null,\"valid_at_ms\":1200,\
\"invalid_at_ms\":1800,\"confidence_class\":\"Asserted\",\"confidence\":0.5,\
\"last_validated_at_ms\":1500,\"derived_from\":[],\"quarantined_at_ms\":1700,\
\"quarantine_reason\":\"explicit_write\",\"quarantined_by\":\"system:test\",\
\"quarantine_review_due_ms\":1900}\n---\n\nbody v2\n";
        write_md(tmp.path(), "ns/k1.md", md);

        let counts = import_md_dir(&db, tmp.path()).expect("import v2");
        assert_eq!(counts.created, 1, "fresh v2 import creates");
        assert_eq!(counts.updated, 0);
        assert_eq!(counts.unchanged, 0);

        let got = db.get("ns", "k1").expect("get").expect("exists");
        assert_eq!(got.payload, "body v2");
        assert_eq!(got.valid_at_ms, 1200);
        assert_eq!(got.invalid_at_ms, Some(1800));
        assert_eq!(got.confidence, 0.5);
        assert_eq!(got.last_validated_at_ms, Some(1500));
        assert_eq!(got.superseded_by.as_deref(), Some("k2"));
        assert_eq!(got.superseded_at_ms, Some(1800));
        assert_eq!(got.quarantined_at_ms, Some(1700));
        assert_eq!(got.quarantine_reason.as_deref(), Some("explicit_write"));
        assert_eq!(got.quarantined_by.as_deref(), Some("system:test"));
        assert_eq!(got.quarantine_review_due_ms, Some(1900));
        assert!(matches!(got.metadata.get("author"), Some(Value::String(s)) if s == "alice"));
    }

    #[test]
    fn import_md_dir_normalizes_crlf_files() {
        // O1 (review P2-01): a `.md` saved by a Windows editor with CRLF line
        // endings must import with a clean payload (no leading `\n`/`\r`) and
        // stay idempotent against the LF-only store projection.
        let db = open_db();
        let tmp = tempfile::tempdir().expect("tempdir");
        let md = "---\r\n{\"schema_version\":2,\"namespace\":\"ns\",\"key\":\"k\",\
\"version\":1,\"node_id\":\"0\",\"created_at_ms\":1000,\"updated_at_ms\":1000,\
\"metadata\":{},\"valid_at_ms\":1000,\"confidence_class\":\"Asserted\",\
\"confidence\":1.0,\"derived_from\":[]}\r\n---\r\n\r\nwindows payload\r\n";
        write_md(tmp.path(), "ns/k.md", md);

        let counts = import_md_dir(&db, tmp.path()).expect("import crlf");
        assert_eq!(counts.created, 1);
        let got = db.get("ns", "k").expect("get").expect("exists");
        assert_eq!(
            got.payload, "windows payload",
            "CRLF file: payload must not keep leading/trailing newlines"
        );

        let again = import_md_dir(&db, tmp.path()).expect("re-import crlf");
        assert_eq!(again.unchanged, 1, "CRLF source stays idempotent");
    }

    #[test]
    fn import_md_dir_accepts_v1_and_normalizes() {
        let db = open_db();
        let tmp = tempfile::tempdir().expect("tempdir");
        // v1 frontmatter: no v2 fields at all.
        let md = "---\n{\"schema_version\":1,\"namespace\":\"ns\",\"key\":\"legacy\",\
\"version\":1,\"node_id\":\"0\",\"created_at_ms\":1000,\"updated_at_ms\":1000,\
\"expires_at_ms\":null,\"superseded_by\":\"newer\",\"superseded_at_ms\":1500,\
\"metadata\":{},\"vector_dim\":null}\n---\n\nlegacy body\n";
        write_md(tmp.path(), "ns/legacy.md", md);

        let counts = import_md_dir(&db, tmp.path()).expect("import v1");
        assert_eq!(counts.created, 1);

        let got = db.get("ns", "legacy").expect("get").expect("exists");
        // ADR-046 §D7 v1 normalization: valid_at = created_at, invalid_at =
        // superseded_at, class Asserted, confidence D_a = 1.0.
        assert_eq!(got.valid_at_ms, 1000);
        assert_eq!(got.invalid_at_ms, Some(1500));
        assert_eq!(got.confidence, 1.0);
        assert_eq!(got.confidence_class, ConfidenceClass::Asserted);
        assert_eq!(got.last_validated_at_ms, None);
        assert!(got.derived_from.is_empty());
    }

    #[test]
    fn import_md_dir_rejects_unknown_schema_version() {
        let db = open_db();
        let tmp = tempfile::tempdir().expect("tempdir");
        let md = "---\n{\"schema_version\":3,\"namespace\":\"ns\",\"key\":\"future\"}\n---\n\nx\n";
        write_md(tmp.path(), "ns/future.md", md);

        let err = import_md_dir(&db, tmp.path()).expect_err("v3 must be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("unsupported md-export schema_version 3"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn import_md_dir_strips_related_block_and_counts_unresolved_links() {
        let db = open_db();
        // Dangling link target: `ghost` does not exist.
        let tmp = tempfile::tempdir().expect("tempdir");
        let md = "---\n{\"schema_version\":2,\"namespace\":\"ns\",\"key\":\"k\",\
\"version\":1,\"node_id\":\"0\",\"created_at_ms\":1000,\"updated_at_ms\":1000,\
\"metadata\":{},\"valid_at_ms\":1000,\"confidence_class\":\"Asserted\",\
\"confidence\":1.0,\"derived_from\":[]}\n---\n\nreal payload\n\
\n<!-- vanta:links -->\n\n## Related\n\n- [[ns/ghost]]\n";
        write_md(tmp.path(), "ns/k.md", md);

        let counts = import_md_dir(&db, tmp.path()).expect("import");
        assert_eq!(counts.created, 1);
        assert_eq!(counts.links_unresolved, 1, "ghost has no destination");

        let got = db.get("ns", "k").expect("get").expect("exists");
        assert_eq!(got.payload, "real payload", "links never reach the payload");
    }

    #[test]
    fn import_md_dir_resolves_existing_wikilink() {
        let db = open_db();
        db.put(vantadb::sdk::MemoryInput::new("ns", "target", "here"))
            .expect("seed target");
        let tmp = tempfile::tempdir().expect("tempdir");
        let md = "---\n{\"schema_version\":2,\"namespace\":\"ns\",\"key\":\"k\",\
\"version\":1,\"node_id\":\"0\",\"created_at_ms\":1000,\"updated_at_ms\":1000,\
\"metadata\":{},\"valid_at_ms\":1000,\"confidence_class\":\"Asserted\",\
\"confidence\":1.0,\"derived_from\":[]}\n---\n\nbody\n\
\n<!-- vanta:links -->\n\n## Related\n\n- [[ns/target]]\n";
        write_md(tmp.path(), "ns/k.md", md);

        let counts = import_md_dir(&db, tmp.path()).expect("import");
        assert_eq!(counts.links_unresolved, 0);
    }

    #[test]
    fn import_md_dir_v2_idempotent_then_updates_on_frontmatter_edit() {
        let db = open_db();
        let tmp = tempfile::tempdir().expect("tempdir");
        let base = "---\n{\"schema_version\":2,\"namespace\":\"ns\",\"key\":\"k\",\
\"version\":1,\"node_id\":\"0\",\"created_at_ms\":1000,\"updated_at_ms\":1000,\
\"metadata\":{},\"valid_at_ms\":1000,\"confidence_class\":\"Asserted\",\
\"confidence\":1.0,\"derived_from\":[]}\n---\n\nstable\n";
        write_md(tmp.path(), "ns/k.md", base);
        import_md_dir(&db, tmp.path()).expect("first");

        let redo = import_md_dir(&db, tmp.path()).expect("re-import");
        assert_eq!(redo.created, 0);
        assert_eq!(redo.updated, 0);
        assert_eq!(redo.unchanged, 1);

        // A frontmatter-only edit (confidence) must propagate: the projection
        // hash is not payload-only.
        let edited = base.replace("\"confidence\":1.0", "\"confidence\":0.25");
        write_md(tmp.path(), "ns/k.md", &edited);
        let changed = import_md_dir(&db, tmp.path()).expect("re-import edited");
        assert_eq!(changed.updated, 1, "frontmatter edit propagates");
        let got = db.get("ns", "k").expect("get").expect("exists");
        assert_eq!(got.confidence, 0.25);
    }
}
