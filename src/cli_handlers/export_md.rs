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
//!
//! Git-friendly (VER-06): the output is byte-stable for unchanged data — no
//! wall-clock timestamps and `index.json` records are sorted by file path, so
//! re-exporting a database with no changes produces no git diff.
//!
//! Frontmatter schema v2 (VER-06) carries the ADR-046 semantics
//! (`valid_at_ms`/`invalid_at_ms`, `confidence_class`/`confidence`,
//! `derived_from`, quarantine ×4, `superseded_by`/`superseded_at_ms`). The
//! importer accepts v1 and v2 (`1..=2`), so old exports keep round-tripping.
//! Records with relations also get an informational trailing block
//! (`<!-- vanta:links -->` + `## Related` wikilinks) that the importer strips
//! before reading the payload — see `vanta_memory::seed::md_import`.

use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::cli_handlers::{
    create_spinner, open_embedded, print_info, print_json, print_success, print_warning,
};
use crate::error::Result;
use crate::sdk::{ConfidenceClass, MemoryRecord};

/// Stable schema version for the MD export frontmatter. Bump on breaking change
/// of the frontmatter shape (import side refuses unknown versions).
///
/// v2 (VER-06): adds the ADR-046 semantics (`valid_at_ms`, `invalid_at_ms`,
/// `confidence_class`, `confidence`, `last_validated_at_ms`, `derived_from`,
/// quarantine ×4). Additive: the importer accepts `1..=2`.
pub const MD_EXPORT_SCHEMA_VERSION: u32 = 2;

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
    // ── v2 (VER-06 / ADR-046): additive, import accepts 1..=2 ──
    /// Start of the validity window; v1 files normalize to `created_at_ms`.
    valid_at_ms: u64,
    /// End of the validity window (exclusive); `None` = open-ended.
    invalid_at_ms: Option<u64>,
    /// Provenance class (`"Asserted"` | `"Derived"`).
    confidence_class: ConfidenceClass,
    /// Confidence range in `[0, 1]` (not a calibrated probability).
    confidence: f32,
    /// Last successful re-validation timestamp; `None` = never.
    last_validated_at_ms: Option<u64>,
    /// Same-namespace parent keys for `derived` records.
    derived_from: &'a [String],
    /// Quarantine entry timestamp; `None` = active (not quarantined).
    quarantined_at_ms: Option<u64>,
    /// Stable quarantine reason code (`explicit_write`, `unreviewed_import`, …).
    quarantine_reason: Option<&'a str>,
    /// Principal that applied the quarantine (`system:<op>`).
    quarantined_by: Option<&'a str>,
    /// Review deadline signal; `None` = no default deadline configured.
    quarantine_review_due_ms: Option<u64>,
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

/// Aggregate manifest written next to the exported `.md` files.
///
/// Git-friendly (VER-06): contains no wall-clock timestamps and `records` is
/// sorted by `file`, so an export of unchanged data is byte-identical across
/// runs. Nothing reads this file back (the importer walks `*.md` only); it is
/// a convenience manifest for diffs and tooling.
#[derive(Debug, Serialize)]
struct IndexFile {
    schema_version: u32,
    record_count: u64,
    records: Vec<IndexEntry>,
}

fn vector_dim_to_json(vector: Option<&Vec<f32>>) -> Option<usize> {
    vector.map(|v| v.len())
}

/// Wikilinks to related records: `superseded_by` and `derived_from` (both
/// same-namespace keys) rendered as `[[namespace/key]]`.
///
/// Informational only (VER-06): the trailing block is stripped by the importer
/// before the payload is read — it is a projection for editors (Obsidian-style
/// navigation), never a source of joins.
fn related_links(record: &MemoryRecord) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();
    let mut push = |target: &str| {
        let link = format!("[[{}/{}]]", record.namespace, target);
        if !links.contains(&link) {
            links.push(link);
        }
    };
    if let Some(next) = record.superseded_by.as_deref() {
        push(next);
    }
    for parent in &record.derived_from {
        push(parent);
    }
    links
}

/// Render a single record to a Markdown string with JSON frontmatter.
fn render_record_md(record: &MemoryRecord) -> String {
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
        valid_at_ms: record.valid_at_ms,
        invalid_at_ms: record.invalid_at_ms,
        confidence_class: record.confidence_class,
        confidence: record.confidence,
        last_validated_at_ms: record.last_validated_at_ms,
        derived_from: &record.derived_from,
        quarantined_at_ms: record.quarantined_at_ms,
        quarantine_reason: record.quarantine_reason.as_deref(),
        quarantined_by: record.quarantined_by.as_deref(),
        quarantine_review_due_ms: record.quarantine_review_due_ms,
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
    // Marker literal is duplicated in `vanta_memory::seed::md_import`
    // (different crates) — keep both in sync if it ever changes.
    let links = related_links(record);
    if !links.is_empty() {
        body.push_str("\n<!-- vanta:links -->\n\n## Related\n\n");
        for link in &links {
            body.push_str("- ");
            body.push_str(link);
            body.push('\n');
        }
    }
    body
}

fn metadata_to_json(metadata: &crate::sdk::MemoryMetadata) -> BTreeMap<String, serde_json::Value> {
    let mut out = BTreeMap::new();
    for (k, v) in metadata.iter() {
        out.insert(k.clone(), metadata_value_to_json(v));
    }
    out
}

fn metadata_value_to_json(v: &crate::sdk::Value) -> serde_json::Value {
    use crate::sdk::Value;
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::Value::from(*i),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::DateTime(dt) => serde_json::Value::String(dt.to_rfc3339()),
        Value::ListString(xs) => serde_json::Value::from(xs.clone()),
        Value::ListInt(xs) => serde_json::Value::from(xs.clone()),
        Value::ListFloat(xs) => serde_json::Value::from(xs.clone()),
        Value::ListBool(xs) => serde_json::Value::from(xs.clone()),
        Value::ListDateTime(xs) => {
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
pub fn cmd_export_md(
    db_path: &str,
    namespace: Option<&str>,
    out_dir: &str,
    json_output: bool,
) -> Result<()> {
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
            let opts = crate::sdk::MemoryListOptions {
                #[allow(deprecated)]
                filters: crate::sdk::MemoryMetadata::new(),
                filter_ops: None,
                limit: BATCH_SIZE,
                cursor,
                exclude_superseded: false,
                as_of_ms: None,
                valid_window: None,
                include_quarantined: true,
                min_confidence: None,
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
    // Sorted by file path and free of volatile fields so re-exporting an
    // unchanged database is byte-identical (git-friendly, VER-06).
    index_entries.sort_by(|a, b| a.file.cmp(&b.file));
    let index = IndexFile {
        schema_version: MD_EXPORT_SCHEMA_VERSION,
        record_count: total,
        records: index_entries,
    };
    let index_json =
        serde_json::to_string_pretty(&index).map_err(crate::error::Error::serialization)?;
    let index_path = out_path.join("index.json");
    let mut index_file = fs::File::create(&index_path)?;
    index_file.write_all(index_json.as_bytes())?;
    index_file.flush()?;

    if json_output {
        return print_json(&serde_json::json!({
            "exported": total,
            "out": out_path.display().to_string(),
            "format": "md",
            "index": index_path.display().to_string(),
        }));
    }

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
        let rec = MemoryRecord {
            namespace: "agent/team".into(),
            key: "k/1".into(),
            payload: "hello".into(),
            metadata: crate::sdk::MemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 1,
            node_id: 42,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            ..Default::default()
        };
        let body = render_record_md(&rec);
        assert!(body.starts_with("---\n"));
        // Frontmatter is compact JSON (no spaces) — parse it instead of
        // string-matching whitespace-sensitive literals.
        let fm_end = body.find("\n---\n").expect("closing frontmatter delimiter");
        let fm_json = &body["---\n".len()..fm_end];
        let fm: serde_json::Value =
            serde_json::from_str(fm_json).expect("frontmatter must be valid JSON");
        assert_eq!(fm["namespace"], "agent/team");
        assert_eq!(fm["schema_version"], 2);
        assert_eq!(fm["key"], "k/1");
        assert!(body.ends_with("hello\n"));
    }

    #[test]
    fn render_record_md_emits_v2_semantics() {
        let rec = MemoryRecord {
            namespace: "agent/team".into(),
            key: "k2".into(),
            payload: "hello".into(),
            metadata: crate::sdk::MemoryMetadata::new(),
            created_at_ms: 1000,
            updated_at_ms: 2000,
            version: 3,
            node_id: 42,
            vector: None,
            sparse_vector: None,
            expires_at_ms: None,
            superseded_by: None,
            superseded_at_ms: None,
            valid_at_ms: 1500,
            invalid_at_ms: Some(2500),
            confidence: 0.5,
            last_validated_at_ms: Some(3000),
            quarantined_at_ms: Some(4000),
            quarantine_reason: Some("explicit_write".into()),
            quarantined_by: Some("system:test".into()),
            quarantine_review_due_ms: Some(5000),
            ..Default::default()
        };
        let body = render_record_md(&rec);
        let fm_end = body.find("\n---\n").expect("closing frontmatter delimiter");
        let fm: serde_json::Value =
            serde_json::from_str(&body["---\n".len()..fm_end]).expect("frontmatter JSON");
        assert_eq!(fm["valid_at_ms"], 1500);
        assert_eq!(fm["invalid_at_ms"], 2500);
        assert_eq!(fm["confidence_class"], "Asserted");
        assert_eq!(fm["confidence"], 0.5);
        assert_eq!(fm["last_validated_at_ms"], 3000);
        assert_eq!(fm["derived_from"], serde_json::json!([]));
        assert_eq!(fm["quarantined_at_ms"], 4000);
        assert_eq!(fm["quarantine_reason"], "explicit_write");
        assert_eq!(fm["quarantined_by"], "system:test");
        assert_eq!(fm["quarantine_review_due_ms"], 5000);
    }

    #[test]
    fn render_record_md_emits_related_wikilinks() {
        let rec = MemoryRecord {
            namespace: "agent/team".into(),
            key: "old".into(),
            payload: "body".into(),
            superseded_by: Some("new".into()),
            derived_from: vec!["parent-a".into(), "new".into()],
            ..Default::default()
        };
        let body = render_record_md(&rec);
        assert!(body.contains("\n<!-- vanta:links -->\n\n## Related\n\n"));
        assert!(body.contains("- [[agent/team/new]]\n"));
        assert!(body.contains("- [[agent/team/parent-a]]\n"));
        // Deduplicated: a target repeated across superseded_by/derived_from
        // renders once.
        assert_eq!(body.matches("- [[agent/team/new]]").count(), 1);
        assert!(body.ends_with("- [[agent/team/parent-a]]\n"));
    }

    #[test]
    fn render_record_md_omits_related_block_without_relations() {
        let rec = MemoryRecord {
            namespace: "ns".into(),
            key: "k".into(),
            payload: "body".into(),
            ..Default::default()
        };
        let body = render_record_md(&rec);
        assert!(!body.contains("<!-- vanta:links -->"));
        assert!(!body.contains("## Related"));
        assert!(body.ends_with("body\n"));
    }
}
