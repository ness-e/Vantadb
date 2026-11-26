//! Document ingestors (MEMG-08 / MGR-25): format → chunks [`MemoryInput`].
//!
//! Each [`Ingestor`] maps one source file to memory chunks that carry
//! **mandatory provenance** in `metadata`:
//!
//! | key | value |
//! |-----|-------|
//! | `source` | origin file path, relative to the scan root (forward slashes) |
//! | `page`   | page number — `0` for formats without pagination (txt/json/csv) |
//! | `chunk`  | 0-based chunk index within the file |
//!
//! The keys are flat by design: the memory `Value` type has no object variant,
//! and nested metadata objects are rejected at the MCP boundary. The shape
//! mirrors the MGR-25 provenance precedent already used in-repo
//! (`examples/rag_pdf_chat/rag_pdf_demo.py`).
//!
//! [`scan_ingestable_sources`] walks a directory (shared traversal guard from
//! [`super::sources`]), dispatches each file to the ingestor that declares its
//! extension, and enforces [`SOURCE_CHAR_BUDGET`] over the **emitted payload**
//! (chunk overlap included): the chunk that crosses the budget is truncated to
//! fit and the scan stops — declared truncation (plan pre-mortem 3).
//!
//! Declared format order (cost/benefit, plan Task 52): **txt/json/csv** (this
//! module) → html → pdf → docx (FIND-298 — the latter three need parser
//! dependencies that the MGR-25 spec must sanction first). Source-code files
//! (rs/py/ts) are out of scope by MGR-22 — they belong to the code graph, not
//! to document ingestion.

use std::path::Path;

use crate::error::Result;
use crate::sdk::types::{MemoryInput, Value};
use crate::wiki::chunker::{chunk_text, DEFAULT_OVERLAP_CHARS, DEFAULT_TARGET_CHARS};
use crate::wiki::sources::{collect_text_files, SOURCE_CHAR_BUDGET};

/// Provenance metadata key: origin file path, relative to the scan root
/// (forward-slash separated, stable across OS).
pub const META_SOURCE: &str = "source";
/// Provenance metadata key: page number (`0` for formats without pagination;
/// a pdf ingestor will emit real page numbers).
pub const META_PAGE: &str = "page";
/// Provenance metadata key: 0-based chunk index within the file.
pub const META_CHUNK: &str = "chunk";

/// A format ingestor: maps one source file to memory chunks.
///
/// Implementors declare the file extensions they handle and (optionally)
/// override [`Ingestor::ingest`] when the format is not already plain text
/// (e.g. pdf/docx later — FIND-298).
pub trait Ingestor: Send + Sync {
    /// Lowercase file extensions handled by this ingestor (no leading dot).
    /// Matching is **exact-case**: a `NOTES.TXT` file is not matched by `txt`.
    fn extensions(&self) -> &'static [&'static str];

    /// Format → chunks. The provided implementation chunks the raw text with
    /// the wiki chunker (target [`DEFAULT_TARGET_CHARS`], overlap
    /// [`DEFAULT_OVERLAP_CHARS`]) and stamps the mandatory provenance
    /// (`source`/`page`/`chunk`) on every chunk under the deterministic key
    /// `{file}#{chunk}` — a re-scan of the same tree reuses keys, so
    /// re-ingesting updates records instead of duplicating them.
    ///
    /// Note: the write boundary (`Embedded::put`/`put_batch`) rejects keys
    /// longer than 512 bytes — a relative path long enough to push the key
    /// past that limit will make the consuming write fail (the batch is
    /// validated as a whole).
    ///
    /// Errors are per-file: callers of [`scan_ingestable_sources`] skip a
    /// failing source with a trace log, never fail the whole scan.
    fn ingest(&self, namespace: &str, file: &str, content: &str) -> Result<Vec<MemoryInput>> {
        let chunks = chunk_text(content, DEFAULT_TARGET_CHARS, DEFAULT_OVERLAP_CHARS);
        let mut inputs = Vec::with_capacity(chunks.len());
        for (chunk_index, chunk) in chunks.into_iter().enumerate() {
            let mut input = MemoryInput::new(namespace, chunk_key(file, chunk_index), chunk);
            input
                .metadata
                .insert(META_SOURCE.to_string(), Value::String(file.to_string()));
            input.metadata.insert(META_PAGE.to_string(), Value::Int(0));
            input
                .metadata
                .insert(META_CHUNK.to_string(), Value::Int(chunk_index as i64));
            inputs.push(input);
        }
        Ok(inputs)
    }
}

/// Deterministic chunk key: `{file}#{chunk_index}`.
fn chunk_key(file: &str, chunk_index: usize) -> String {
    format!("{file}#{chunk_index}")
}

/// Plain-text ingestor (`.txt`) — direct chunker over the file text.
pub struct TxtIngestor;

/// JSON ingestor (`.json`, `.jsonl`) — structured field-aware extraction (MGR-25):
/// maps top-level scalar and list properties to typed metadata (`Value::String`,
/// `Value::Int`, `Value::Float`, `Value::Bool`, `Value::ListString`, etc.),
/// while preserving canonical indexable text in `payload`.
pub struct JsonIngestor;

/// CSV ingestor (`.csv`) — structured column-aware extraction (MGR-25):
/// parses header columns, infers scalar types per cell, and stamps them
/// onto metadata for each row chunk.
pub struct CsvIngestor;

/// Markdown ingestor (`.md`, `.markdown`) — structured section & code extractor (MGR-25):
/// extracts YAML/JSON frontmatter, hierarchical headings (`h1`..`h3`), and
/// tagged code blocks (`lang`) as filterable metadata.
pub struct MdIngestor;

impl Ingestor for TxtIngestor {
    fn extensions(&self) -> &'static [&'static str] {
        &["txt"]
    }
}

impl Ingestor for JsonIngestor {
    fn extensions(&self) -> &'static [&'static str] {
        &["json", "jsonl"]
    }

    fn ingest(&self, namespace: &str, file: &str, content: &str) -> Result<Vec<MemoryInput>> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        // Try single JSON object or array
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
            match parsed {
                serde_json::Value::Object(map) => {
                    let mut input =
                        MemoryInput::new(namespace, chunk_key(file, 0), trimmed.to_string());
                    stamp_provenance(&mut input, file, 0);
                    for (k, v) in &map {
                        if let Some(val) = json_to_sdk_value(v) {
                            input.metadata.insert(k.clone(), val);
                        }
                    }
                    return Ok(vec![input]);
                }
                serde_json::Value::Array(items) => {
                    let mut inputs = Vec::with_capacity(items.len());
                    for (idx, item) in items.into_iter().enumerate() {
                        let payload = serde_json::to_string(&item).unwrap_or_default();
                        let mut input = MemoryInput::new(namespace, chunk_key(file, idx), payload);
                        stamp_provenance(&mut input, file, idx);
                        if let serde_json::Value::Object(map) = item {
                            for (k, v) in map {
                                if let Some(val) = json_to_sdk_value(&v) {
                                    input.metadata.insert(k, val);
                                }
                            }
                        }
                        inputs.push(input);
                    }
                    return Ok(inputs);
                }
                _ => {}
            }
        }

        // Try line-by-line JSONL
        let mut inputs = Vec::new();
        for line in trimmed.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(serde_json::Value::Object(map)) =
                serde_json::from_str::<serde_json::Value>(line)
            {
                let idx = inputs.len();
                let mut input = MemoryInput::new(namespace, chunk_key(file, idx), line.to_string());
                stamp_provenance(&mut input, file, idx);
                for (k, v) in map {
                    if let Some(val) = json_to_sdk_value(&v) {
                        input.metadata.insert(k, val);
                    }
                }
                inputs.push(input);
            }
        }
        if !inputs.is_empty() {
            return Ok(inputs);
        }

        // Fallback to text chunking if not parseable as JSON objects
        TxtIngestor.ingest(namespace, file, content)
    }
}

impl Ingestor for CsvIngestor {
    fn extensions(&self) -> &'static [&'static str] {
        &["csv"]
    }

    fn ingest(&self, namespace: &str, file: &str, content: &str) -> Result<Vec<MemoryInput>> {
        let rows = parse_csv_records(content);
        if rows.is_empty() {
            return Ok(Vec::new());
        }
        let headers = &rows[0];
        let mut inputs = Vec::with_capacity(rows.len().saturating_sub(1));
        for (row_idx, row) in rows.iter().skip(1).enumerate() {
            let mut pairs = Vec::with_capacity(headers.len());
            let mut metadata_entries = Vec::with_capacity(headers.len());
            for (col_idx, header) in headers.iter().enumerate() {
                let cell = row.get(col_idx).map(String::as_str).unwrap_or("");
                pairs.push(format!("{header}: {cell}"));
                let val = parse_csv_value(cell);
                metadata_entries.push((header.clone(), val));
            }
            let payload = pairs.join(", ");
            let mut input = MemoryInput::new(namespace, chunk_key(file, row_idx), payload);
            stamp_provenance(&mut input, file, row_idx);
            for (k, v) in metadata_entries {
                input.metadata.insert(k, v);
            }
            inputs.push(input);
        }
        if inputs.is_empty() {
            return TxtIngestor.ingest(namespace, file, content);
        }
        Ok(inputs)
    }
}

impl Ingestor for MdIngestor {
    fn extensions(&self) -> &'static [&'static str] {
        &["md", "markdown"]
    }

    fn ingest(&self, namespace: &str, file: &str, content: &str) -> Result<Vec<MemoryInput>> {
        let (frontmatter_meta, body) = extract_md_frontmatter(content);
        let (h1, h2, h3, headings) = extract_md_headings(&body);
        let code_langs = extract_md_code_langs(&body);

        let chunks = chunk_text(&body, DEFAULT_TARGET_CHARS, DEFAULT_OVERLAP_CHARS);
        let mut inputs = Vec::with_capacity(chunks.len());
        for (chunk_index, chunk) in chunks.into_iter().enumerate() {
            let mut input =
                MemoryInput::new(namespace, chunk_key(file, chunk_index), chunk.clone());
            stamp_provenance(&mut input, file, chunk_index);

            // Stamp frontmatter metadata
            for (k, v) in &frontmatter_meta {
                input.metadata.insert(k.clone(), v.clone());
            }

            // Stamp headings
            if let Some(h1_val) = &h1 {
                input
                    .metadata
                    .insert("h1".to_string(), Value::String(h1_val.clone()));
            }
            if let Some(h2_val) = &h2 {
                input
                    .metadata
                    .insert("h2".to_string(), Value::String(h2_val.clone()));
            }
            if let Some(h3_val) = &h3 {
                input
                    .metadata
                    .insert("h3".to_string(), Value::String(h3_val.clone()));
            }
            if !headings.is_empty() {
                input
                    .metadata
                    .insert("headings".to_string(), Value::ListString(headings.clone()));
            }

            // Stamp code langs local to this chunk or inherited from document
            let chunk_langs = extract_md_code_langs(&chunk);
            let active_langs = if !chunk_langs.is_empty() {
                chunk_langs
            } else {
                code_langs.clone()
            };
            if let Some(first_lang) = active_langs.first() {
                input
                    .metadata
                    .insert("lang".to_string(), Value::String(first_lang.clone()));
            }
            if !active_langs.is_empty() {
                input
                    .metadata
                    .insert("code_langs".to_string(), Value::ListString(active_langs));
            }

            inputs.push(input);
        }
        Ok(inputs)
    }
}

/// Stamp mandatory provenance metadata on a chunk input.
fn stamp_provenance(input: &mut MemoryInput, file: &str, chunk_index: usize) {
    input
        .metadata
        .insert(META_SOURCE.to_string(), Value::String(file.to_string()));
    input.metadata.insert(META_PAGE.to_string(), Value::Int(0));
    input
        .metadata
        .insert(META_CHUNK.to_string(), Value::Int(chunk_index as i64));
}

/// Convert JSON scalar/list value to SDK typed `Value`. Nested objects are ignored (flat metadata contract).
fn json_to_sdk_value(val: &serde_json::Value) -> Option<Value> {
    match val {
        serde_json::Value::String(s) => Some(Value::String(s.clone())),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Some(Value::Int(i))
            } else if let Some(u) = n.as_u64() {
                Some(Value::Int(u as i64))
            } else {
                n.as_f64().map(Value::Float)
            }
        }
        serde_json::Value::Bool(b) => Some(Value::Bool(*b)),
        serde_json::Value::Null => Some(Value::Null),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                Some(Value::ListString(Vec::new()))
            } else if arr.iter().all(|x| x.is_string()) {
                Some(Value::ListString(
                    arr.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect(),
                ))
            } else if arr.iter().all(|x| x.is_i64()) {
                Some(Value::ListInt(
                    arr.iter().filter_map(|x| x.as_i64()).collect(),
                ))
            } else if arr.iter().all(|x| x.is_number()) {
                Some(Value::ListFloat(
                    arr.iter().filter_map(|x| x.as_f64()).collect(),
                ))
            } else if arr.iter().all(|x| x.is_boolean()) {
                Some(Value::ListBool(
                    arr.iter().filter_map(|x| x.as_bool()).collect(),
                ))
            } else {
                Some(Value::ListString(
                    arr.iter().map(|x| x.to_string()).collect(),
                ))
            }
        }
        serde_json::Value::Object(_) => None,
    }
}

/// Parse a CSV cell into typed `Value` (`Bool`, `Int`, `Float`, or `String`).
fn parse_csv_value(cell: &str) -> Value {
    let trimmed = cell.trim();
    if trimmed.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if trimmed.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else if let Ok(i) = trimmed.parse::<i64>() {
        Value::Int(i)
    } else if let Ok(f) = trimmed.parse::<f64>() {
        Value::Float(f)
    } else {
        Value::String(trimmed.to_string())
    }
}

/// RFC 4180 CSV record parser with support for quotes and escaped quotes.
fn parse_csv_records(content: &str) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = content.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current_field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                current_field.push(c);
            }
        } else {
            match c {
                '"' => in_quotes = true,
                ',' => {
                    current_row.push(current_field.trim().to_string());
                    current_field.clear();
                }
                '\r' => {
                    if chars.peek() == Some(&'\n') {
                        chars.next();
                    }
                    current_row.push(current_field.trim().to_string());
                    current_field.clear();
                    if !current_row.iter().all(|s| s.is_empty()) {
                        rows.push(std::mem::take(&mut current_row));
                    } else {
                        current_row.clear();
                    }
                }
                '\n' => {
                    current_row.push(current_field.trim().to_string());
                    current_field.clear();
                    if !current_row.iter().all(|s| s.is_empty()) {
                        rows.push(std::mem::take(&mut current_row));
                    } else {
                        current_row.clear();
                    }
                }
                _ => current_field.push(c),
            }
        }
    }
    if !current_field.is_empty() || !current_row.is_empty() {
        current_row.push(current_field.trim().to_string());
        if !current_row.iter().all(|s| s.is_empty()) {
            rows.push(current_row);
        }
    }
    rows
}

/// Extract YAML or JSON frontmatter from markdown text.
fn extract_md_frontmatter(content: &str) -> (std::collections::HashMap<String, Value>, String) {
    let normalized = content.replace("\r\n", "\n");
    let mut meta = std::collections::HashMap::new();

    let lines: Vec<&str> = normalized.lines().collect();
    if lines.is_empty() || lines[0].trim() != "---" {
        return (meta, content.to_string());
    }

    let mut end_idx = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end_idx = Some(i);
            break;
        }
    }

    let Some(closing_idx) = end_idx else {
        return (meta, content.to_string());
    };

    let fm_lines = &lines[1..closing_idx];
    let fm_text = fm_lines.join("\n");
    let body_lines = &lines[closing_idx + 1..];
    let body = body_lines.join("\n").trim_start_matches('\n').to_string();

    // Try parsing as JSON first
    if let Ok(serde_json::Value::Object(map)) = serde_json::from_str::<serde_json::Value>(&fm_text)
    {
        for (k, v) in map {
            if let Some(val) = json_to_sdk_value(&v) {
                meta.insert(k, val);
            }
        }
        return (meta, body);
    }

    // Try parsing as YAML lines
    parse_yaml_into_metadata(&fm_text, &mut meta);
    (meta, body)
}

/// Parse simple YAML key-value pairs and lists into typed metadata.
fn parse_yaml_into_metadata(yaml: &str, meta: &mut std::collections::HashMap<String, Value>) {
    let mut current_list_key: Option<String> = None;
    let mut current_list_items: Vec<String> = Vec::new();

    for line in yaml.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(item) = trimmed.strip_prefix('-') {
            let item_val = item.trim().trim_matches('"').trim_matches('\'').to_string();
            current_list_items.push(item_val);
            continue;
        }

        if let Some(key) = current_list_key.take() {
            meta.insert(
                key,
                Value::ListString(std::mem::take(&mut current_list_items)),
            );
        }

        if let Some((k, v)) = trimmed.split_once(':') {
            let key = k.trim().to_string();
            let v = v.trim();
            if v.is_empty() {
                current_list_key = Some(key);
            } else if v.starts_with('[') && v.ends_with(']') {
                let inner = &v[1..v.len() - 1];
                let items: Vec<String> = inner
                    .split(',')
                    .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                meta.insert(key, Value::ListString(items));
            } else {
                let unquoted = v.trim_matches('"').trim_matches('\'');
                let val = if unquoted.eq_ignore_ascii_case("true") {
                    Value::Bool(true)
                } else if unquoted.eq_ignore_ascii_case("false") {
                    Value::Bool(false)
                } else if let Ok(i) = unquoted.parse::<i64>() {
                    Value::Int(i)
                } else if let Ok(f) = unquoted.parse::<f64>() {
                    Value::Float(f)
                } else {
                    Value::String(unquoted.to_string())
                };
                meta.insert(key, val);
            }
        }
    }

    if let Some(key) = current_list_key {
        if !current_list_items.is_empty() {
            meta.insert(key, Value::ListString(current_list_items));
        }
    }
}

/// Extract h1..h3 headings from markdown body.
fn extract_md_headings(
    body: &str,
) -> (Option<String>, Option<String>, Option<String>, Vec<String>) {
    let mut h1 = None;
    let mut h2 = None;
    let mut h3 = None;
    let mut headings = Vec::new();
    let mut in_code_block = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        if let Some(text) = trimmed.strip_prefix("### ") {
            let title = text.trim().to_string();
            if h3.is_none() {
                h3 = Some(title.clone());
            }
            headings.push(title);
        } else if let Some(text) = trimmed.strip_prefix("## ") {
            let title = text.trim().to_string();
            if h2.is_none() {
                h2 = Some(title.clone());
            }
            headings.push(title);
        } else if let Some(text) = trimmed.strip_prefix("# ") {
            let title = text.trim().to_string();
            if h1.is_none() {
                h1 = Some(title.clone());
            }
            headings.push(title);
        }
    }

    (h1, h2, h3, headings)
}

/// Extract code block languages from markdown body.
fn extract_md_code_langs(body: &str) -> Vec<String> {
    let mut langs = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("```") {
            let lang = rest.split_whitespace().next().unwrap_or("");
            if !lang.is_empty() && !langs.contains(&lang.to_string()) {
                langs.push(lang.to_string());
            }
        }
    }
    langs
}

/// Built-in registry in the declared cost order: txt → json → csv → md.
///
/// A scan accepts a file when ANY ingestor in the registry declares its
/// extension; extend this list as formats land (html → pdf → docx, FIND-298).
pub fn default_ingestors() -> Vec<Box<dyn Ingestor>> {
    vec![
        Box::new(TxtIngestor),
        Box::new(JsonIngestor),
        Box::new(CsvIngestor),
        Box::new(MdIngestor),
    ]
}

/// Scan `root` for files handled by `ingestors` and produce memory chunks with
/// mandatory provenance (`source`/`page`/`chunk`).
///
/// Semantics:
/// - traversal mirrors the wiki scanner (canonicalized root, containment
///   guard, deterministic order, unreadable/non-UTF-8 files skipped);
/// - each accepted file is dispatched to the first ingestor declaring its
///   extension; a per-file ingestion error skips that file with a trace log;
/// - [`SOURCE_CHAR_BUDGET`] bounds the **sum of emitted payload characters**
///   (chunk overlap included); the chunk that crosses the budget is truncated
///   to fit and the scan stops (declared truncation).
///
/// Only a missing/invalid `root` errors — partial ingestion is the documented
/// contract (mirrors [`super::sources::scan_local_sources`]).
pub fn scan_ingestable_sources(
    root: &Path,
    namespace: &str,
    ingestors: &[Box<dyn Ingestor>],
) -> Result<Vec<MemoryInput>> {
    let mut inputs = Vec::new();
    let mut budget = SOURCE_CHAR_BUDGET;
    collect_text_files(
        root,
        &|ext| ingestors.iter().any(|i| i.extensions().contains(&ext)),
        &mut |file, content| {
            let ext = extension_of(&file);
            let Some(ingestor) = ingestors.iter().find(|i| i.extensions().contains(&ext)) else {
                return true; // accept() already filtered; defensive only
            };
            match ingestor.ingest(namespace, &file, &content) {
                Ok(chunks) => {
                    for chunk in chunks {
                        let len = chunk.payload.chars().count();
                        if len >= budget {
                            // Budget exhausted: keep the cut that fits and stop
                            // the scan. `take` is the identity when the chunk
                            // fits exactly (`len == budget`).
                            let truncated = len > budget;
                            let mut chunk = chunk;
                            if truncated {
                                chunk.payload = chunk.payload.chars().take(budget).collect();
                            }
                            budget = 0;
                            inputs.push(chunk);
                            tracing::debug!(
                                file = %file,
                                truncated,
                                "ingest: budget exhausted — scan stopped"
                            );
                            return false;
                        }
                        budget -= len;
                        inputs.push(chunk);
                    }
                    true
                }
                Err(e) => {
                    tracing::warn!(file = %file, error = %e, "ingest: source skipped");
                    true
                }
            }
        },
    )?;
    Ok(inputs)
}

/// File extension of a forward-slash relative path (no dot; `""` when none).
fn extension_of(file: &str) -> &str {
    Path::new(file)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wiki::chunker::DEFAULT_TARGET_CHARS;
    use std::fs;

    fn temp_root() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        fs::write(p, content).expect("write");
    }

    fn provenance(input: &MemoryInput) -> (String, i64, i64) {
        let source = match input.metadata.get(META_SOURCE) {
            Some(Value::String(s)) => s.clone(),
            other => panic!("missing `source` metadata: {other:?}"),
        };
        let page = match input.metadata.get(META_PAGE) {
            Some(Value::Int(i)) => *i,
            other => panic!("missing `page` metadata: {other:?}"),
        };
        let chunk = match input.metadata.get(META_CHUNK) {
            Some(Value::Int(i)) => *i,
            other => panic!("missing `chunk` metadata: {other:?}"),
        };
        (source, page, chunk)
    }

    // ── (a) proveniencia obligatoria por formato ──

    #[test]
    fn txt_ingestor_chunks_with_mandatory_provenance() {
        let inputs = TxtIngestor
            .ingest("docs", "notes/a.txt", "hello world")
            .expect("ingest");
        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].payload, "hello world");
        assert_eq!(inputs[0].key, "notes/a.txt#0");
        assert_eq!(provenance(&inputs[0]), ("notes/a.txt".to_string(), 0, 0));
    }

    #[test]
    fn json_and_csv_ingestors_use_the_same_provenance_contract() {
        let json = JsonIngestor
            .ingest("docs", "data/b.json", r#"{"k": 1}"#)
            .expect("ingest json");
        assert_eq!(provenance(&json[0]), ("data/b.json".to_string(), 0, 0));

        let csv = CsvIngestor
            .ingest("docs", "tables/c.csv", "a,b\n1,2\n")
            .expect("ingest csv");
        assert_eq!(provenance(&csv[0]), ("tables/c.csv".to_string(), 0, 0));
    }

    // ── (b) chunking con overlap ──

    #[test]
    fn large_text_chunks_with_overlap_and_covers_all_content() {
        // Six 2_500-char paragraphs (blank-line separated) → two chunks; the
        // second opens with the 400-char tail of the first (chunker overlap).
        let paragraphs: Vec<String> = (0..6)
            .map(|i| format!("{}{i}", "p".repeat(2_499)))
            .collect();
        let text = paragraphs.join("\n\n");

        let inputs = TxtIngestor
            .ingest("docs", "big.txt", &text)
            .expect("ingest");
        assert_eq!(inputs.len(), 2, "12k target splits the 15k text in two");
        let first = &inputs[0].payload;
        let second = &inputs[1].payload;
        let first_len = first.chars().count();
        assert!(first_len <= DEFAULT_TARGET_CHARS);
        let tail: String = first.chars().skip(first_len.saturating_sub(400)).collect();
        assert!(
            second.starts_with(&tail),
            "second chunk must open with the 400-char overlap tail"
        );
        // Every paragraph is covered by at least one chunk (no data loss).
        for (i, para) in paragraphs.iter().enumerate() {
            let marker = format!("{}{i}", "p".repeat(10));
            assert!(
                inputs.iter().any(|c| c.payload.contains(&marker)),
                "paragraph {i} missing from the emitted chunks: {para:.20}…"
            );
        }
        assert_eq!(provenance(&inputs[1]).2, 1, "chunk index increments");
    }

    // ── (c) dispatch por extensión + skip de no soportados/vacíos ──

    #[test]
    fn unknown_extension_and_empty_content_are_skipped() {
        let root = temp_root();
        write(root.path(), "keep.txt", "real content");
        write(root.path(), "skip.bin", "not handled");
        write(root.path(), "empty.txt", "");

        let inputs =
            scan_ingestable_sources(root.path(), "docs", &default_ingestors()).expect("scan");

        assert_eq!(inputs.len(), 1);
        assert_eq!(inputs[0].key, "keep.txt#0");
    }

    #[test]
    fn dispatch_picks_ingestor_by_extension_from_the_registry() {
        struct FooIngestor;
        impl Ingestor for FooIngestor {
            fn extensions(&self) -> &'static [&'static str] {
                &["foo"]
            }
        }
        let root = temp_root();
        write(root.path(), "a.foo", "foo content");
        write(root.path(), "b.txt", "txt content");

        let registry: Vec<Box<dyn Ingestor>> = vec![Box::new(FooIngestor)];
        let inputs = scan_ingestable_sources(root.path(), "docs", &registry).expect("scan");

        assert_eq!(inputs.len(), 1, "only the registered extension is scanned");
        assert_eq!(inputs[0].key, "a.foo#0");
    }

    #[test]
    fn subdirectory_paths_stay_forward_slash_relative() {
        let root = temp_root();
        write(root.path(), "sub/dir/page.txt", "nested");

        let inputs =
            scan_ingestable_sources(root.path(), "docs", &default_ingestors()).expect("scan");

        assert_eq!(inputs.len(), 1);
        let (source, _, _) = provenance(&inputs[0]);
        assert_eq!(source, "sub/dir/page.txt");
        assert!(!inputs[0].key.contains('\\'), "keys are OS-stable");
        assert_eq!(inputs[0].key, "sub/dir/page.txt#0");
    }

    #[test]
    fn scan_is_deterministic_across_runs() {
        let root = temp_root();
        for i in 0..5 {
            write(root.path(), &format!("f{i}.txt"), &format!("content {i}"));
        }
        let first = scan_ingestable_sources(root.path(), "docs", &default_ingestors())
            .expect("scan 1")
            .into_iter()
            .map(|i| i.key)
            .collect::<Vec<_>>();
        let second = scan_ingestable_sources(root.path(), "docs", &default_ingestors())
            .expect("scan 2")
            .into_iter()
            .map(|i| i.key)
            .collect::<Vec<_>>();
        assert_eq!(first, second);
        assert!(first.iter().any(|k| k == "f0.txt#0"));
    }

    #[test]
    fn nonexistent_root_is_a_clear_error() {
        let err = scan_ingestable_sources(
            Path::new("Z:/definitely/not/here"),
            "docs",
            &default_ingestors(),
        )
        .expect_err("scan must fail");
        assert!(
            matches!(err, crate::error::Error::InvalidInput(_)),
            "{err:?}"
        );
    }

    // ── MGR-25: Ingestores Estructurados Nativos (JSON/JSONL, CSV, Markdown) ──

    #[test]
    fn test_mgr25_json_ingestor_typed_metadata() {
        let json_content = r#"{
            "title": "VantaDB Paper",
            "version": 2,
            "score": 9.85,
            "published": true,
            "tags": ["database", "rust", "hnsw"]
        }"#;

        let inputs = JsonIngestor
            .ingest("docs", "papers/paper.json", json_content)
            .expect("ingest json");
        assert_eq!(inputs.len(), 1);
        let input = &inputs[0];
        assert_eq!(provenance(input), ("papers/paper.json".to_string(), 0, 0));
        assert_eq!(
            input.metadata.get("title"),
            Some(&Value::String("VantaDB Paper".into()))
        );
        assert_eq!(input.metadata.get("version"), Some(&Value::Int(2)));
        assert_eq!(input.metadata.get("score"), Some(&Value::Float(9.85)));
        assert_eq!(input.metadata.get("published"), Some(&Value::Bool(true)));
        assert_eq!(
            input.metadata.get("tags"),
            Some(&Value::ListString(vec![
                "database".into(),
                "rust".into(),
                "hnsw".into()
            ]))
        );

        // JSONL (line-delimited JSON)
        let jsonl_content = "{\"id\": 1, \"name\": \"alpha\"}\n{\"id\": 2, \"name\": \"beta\"}\n";
        let jsonl_inputs = JsonIngestor
            .ingest("docs", "items.jsonl", jsonl_content)
            .expect("ingest jsonl");
        assert_eq!(jsonl_inputs.len(), 2);
        assert_eq!(jsonl_inputs[0].key, "items.jsonl#0");
        assert_eq!(jsonl_inputs[0].metadata.get("id"), Some(&Value::Int(1)));
        assert_eq!(
            jsonl_inputs[0].metadata.get("name"),
            Some(&Value::String("alpha".into()))
        );
        assert_eq!(jsonl_inputs[1].key, "items.jsonl#1");
        assert_eq!(jsonl_inputs[1].metadata.get("id"), Some(&Value::Int(2)));
        assert_eq!(
            jsonl_inputs[1].metadata.get("name"),
            Some(&Value::String("beta".into()))
        );
    }

    #[test]
    fn test_mgr25_csv_ingestor_typed_columns() {
        let csv_content = "name,age,score,active\nAlice,30,95.5,true\nBob,25,88.0,false\n";
        let inputs = CsvIngestor
            .ingest("docs", "tables/users.csv", csv_content)
            .expect("ingest csv");
        assert_eq!(inputs.len(), 2);

        let row0 = &inputs[0];
        assert_eq!(row0.key, "tables/users.csv#0");
        assert_eq!(provenance(row0), ("tables/users.csv".to_string(), 0, 0));
        assert_eq!(
            row0.metadata.get("name"),
            Some(&Value::String("Alice".into()))
        );
        assert_eq!(row0.metadata.get("age"), Some(&Value::Int(30)));
        assert_eq!(row0.metadata.get("score"), Some(&Value::Float(95.5)));
        assert_eq!(row0.metadata.get("active"), Some(&Value::Bool(true)));

        let row1 = &inputs[1];
        assert_eq!(row1.key, "tables/users.csv#1");
        assert_eq!(provenance(row1), ("tables/users.csv".to_string(), 0, 1));
        assert_eq!(
            row1.metadata.get("name"),
            Some(&Value::String("Bob".into()))
        );
        assert_eq!(row1.metadata.get("age"), Some(&Value::Int(25)));
        assert_eq!(row1.metadata.get("score"), Some(&Value::Float(88.0)));
        assert_eq!(row1.metadata.get("active"), Some(&Value::Bool(false)));
    }

    #[test]
    fn test_mgr25_md_ingestor_headings_frontmatter_code_blocks() {
        let md_content = r#"---
title: Vanta Architecture
version: 1
draft: false
rating: 4.9
tags:
  - storage
  - vector
---

# VantaDB Architecture

Welcome to VantaDB.

## Storage Layer

Underlying storage uses Fjall/RocksDB engines.

```rust
fn init_engine() -> Result<()> {
    Ok(())
}
```

### Ingestion Details

Fast streaming ingestion.
"#;

        let inputs = MdIngestor
            .ingest("docs", "arch.md", md_content)
            .expect("ingest md");
        assert_eq!(inputs.len(), 1);
        let doc = &inputs[0];
        assert_eq!(provenance(doc), ("arch.md".to_string(), 0, 0));

        // Frontmatter fields
        assert_eq!(
            doc.metadata.get("title"),
            Some(&Value::String("Vanta Architecture".into()))
        );
        assert_eq!(doc.metadata.get("version"), Some(&Value::Int(1)));
        assert_eq!(doc.metadata.get("draft"), Some(&Value::Bool(false)));
        assert_eq!(doc.metadata.get("rating"), Some(&Value::Float(4.9)));
        assert_eq!(
            doc.metadata.get("tags"),
            Some(&Value::ListString(vec!["storage".into(), "vector".into()]))
        );

        // Hierarchical headings
        assert_eq!(
            doc.metadata.get("h1"),
            Some(&Value::String("VantaDB Architecture".into()))
        );
        assert_eq!(
            doc.metadata.get("h2"),
            Some(&Value::String("Storage Layer".into()))
        );
        assert_eq!(
            doc.metadata.get("h3"),
            Some(&Value::String("Ingestion Details".into()))
        );

        // Code blocks
        assert_eq!(
            doc.metadata.get("lang"),
            Some(&Value::String("rust".into()))
        );
        assert_eq!(
            doc.metadata.get("code_langs"),
            Some(&Value::ListString(vec!["rust".into()]))
        );
    }

    #[test]
    fn test_mgr25_relational_filtering_in_embedded() {
        use crate::sdk::types::{FilterOp, MemoryFilterItem, MemoryListOptions};
        use crate::storage::StorageEngine;
        use std::sync::Arc;

        let temp = temp_root();
        let engine = Arc::new(
            StorageEngine::open(temp.path().to_str().expect("path")).expect("open engine"),
        );
        let embedded = crate::sdk::Embedded::from_engine(engine);
        let ns = "test_mgr25_filters";

        // Ingest CSV rows
        let csv =
            "name,age,score,active\nAlice,30,95.5,true\nBob,18,72.0,false\nCharlie,25,88.5,true\n";
        let csv_inputs = CsvIngestor
            .ingest(ns, "users.csv", csv)
            .expect("csv ingest");
        for input in csv_inputs {
            embedded.put(input).expect("put csv row");
        }

        // Ingest Markdown
        let md = "---\nlang_meta: rust\nactive: true\n---\n# Guide\n```rust\nfn f(){}\n```\n";
        let md_inputs = MdIngestor.ingest(ns, "guide.md", md).expect("md ingest");
        for input in md_inputs {
            embedded.put(input).expect("put md");
        }

        // Filter 1: Int filter (age >= 25)
        let filter_int = vec![MemoryFilterItem {
            field: "age".to_string(),
            op: FilterOp::Gte,
            value: Value::Int(25),
        }];
        let res_int = embedded
            .list(
                ns,
                MemoryListOptions {
                    filter_ops: Some(filter_int),
                    ..Default::default()
                },
            )
            .expect("list int filter");
        // Alice (30) and Charlie (25) match
        assert_eq!(res_int.records.len(), 2);

        // Filter 2: Float filter (score > 90.0)
        let filter_float = vec![MemoryFilterItem {
            field: "score".to_string(),
            op: FilterOp::Gt,
            value: Value::Float(90.0),
        }];
        let res_float = embedded
            .list(
                ns,
                MemoryListOptions {
                    filter_ops: Some(filter_float),
                    ..Default::default()
                },
            )
            .expect("list float filter");
        // Only Alice (95.5) matches
        assert_eq!(res_float.records.len(), 1);
        assert_eq!(
            res_float.records[0].metadata.get("name"),
            Some(&Value::String("Alice".into()))
        );

        // Filter 3: Bool filter (active == true)
        let filter_bool = vec![MemoryFilterItem {
            field: "active".to_string(),
            op: FilterOp::Eq,
            value: Value::Bool(true),
        }];
        let res_bool = embedded
            .list(
                ns,
                MemoryListOptions {
                    filter_ops: Some(filter_bool),
                    ..Default::default()
                },
            )
            .expect("list bool filter");
        // Alice, Charlie, and guide.md match
        assert_eq!(res_bool.records.len(), 3);

        // Filter 4: String filter (lang == "rust")
        let filter_str = vec![MemoryFilterItem {
            field: "lang".to_string(),
            op: FilterOp::Eq,
            value: Value::String("rust".into()),
        }];
        let res_str = embedded
            .list(
                ns,
                MemoryListOptions {
                    filter_ops: Some(filter_str),
                    ..Default::default()
                },
            )
            .expect("list str filter");
        assert_eq!(res_str.records.len(), 1);
        assert_eq!(res_str.records[0].key, "guide.md#0");
    }
}
