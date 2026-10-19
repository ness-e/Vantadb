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

/// JSON ingestor (`.json`) — direct text chunking of the document; parsed /
/// field-aware extraction is not part of the minimal MGR-25 contract.
pub struct JsonIngestor;

/// CSV ingestor (`.csv`) — direct text chunking; table-preserving chunking is
/// deferred (noted in FIND-298's evaluation list).
pub struct CsvIngestor;

impl Ingestor for TxtIngestor {
    fn extensions(&self) -> &'static [&'static str] {
        &["txt"]
    }
}

impl Ingestor for JsonIngestor {
    fn extensions(&self) -> &'static [&'static str] {
        &["json"]
    }
}

impl Ingestor for CsvIngestor {
    fn extensions(&self) -> &'static [&'static str] {
        &["csv"]
    }
}

/// Built-in registry in the declared cost order: txt → json → csv.
///
/// A scan accepts a file when ANY ingestor in the registry declares its
/// extension; extend this list as formats land (html → pdf → docx, FIND-298).
pub fn default_ingestors() -> Vec<Box<dyn Ingestor>> {
    vec![
        Box::new(TxtIngestor),
        Box::new(JsonIngestor),
        Box::new(CsvIngestor),
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
}
