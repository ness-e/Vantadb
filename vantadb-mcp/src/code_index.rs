//! `code_index` — MGR-22 slice v0 (MEMG-09): symbol chunker + file-per-node
//! indexing into a memory namespace.
//!
//! Scans a Rust source tree, chunks by symbol (free functions, methods inside
//! `impl`/`trait` blocks, structs, enums, unions, modules, constants, statics,
//! type aliases — a `trait` declaration itself surfaces through its methods,
//! not as a container symbol), and writes **file-per-node** memory records plus
//! `defines` edges (file → symbol) into the target namespace, so the existing
//! read-only `code_*` tools (`code_search`, `code_explore`, `code_callers`,
//! `code_callees`, `code_impact`) can search and traverse the result.
//!
//! Keys are deterministic (`file:{rel}` / `sym:{rel}#{kind}:{name}`), so a
//! re-index maps every record to the same node id; the file record carries a
//! content hash (FNV-1a) in its metadata and unchanged files are skipped
//! entirely. A changed file reconciles its manifest: symbols no longer present
//! have their `defines` edge removed and their record deleted (no search
//! ghosts, no duplicate edges).
//!
//! v0 scope (stop condition of the plan, Task 53 L1531): Rust only, line-based
//! declaration scanner (no new dependencies). The multi-language tree-sitter
//! chunker, the `code_watch` watcher and the repo-map scene are specified in
//! `docs/dev/research/mgr-22-repo-map.md` and tracked by FIND-299.

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{error_content, serialize_content, text_content, validate_identifier};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use vantadb::sdk::Value as SdkValue;
use vantadb::sdk::{Embedded, MemoryInput};
use vantadb::storage::StorageEngine;

/// Default walk cap for `max_files`.
const DEFAULT_MAX_FILES: usize = 200;
/// Hard cap for `max_files` — keeps a runaway walk bounded.
const MAX_MAX_FILES: usize = 2000;
/// Files larger than this are skipped (counted in `files_skipped`).
const MAX_FILE_BYTES: u64 = 512 * 1024;
/// Cap of symbols extracted per file (rest counted down by omission).
const MAX_SYMBOLS_PER_FILE: usize = 300;
/// Cap of source characters stored as a symbol payload.
const MAX_SYMBOL_PAYLOAD_CHARS: usize = 4000;
/// Cap of symbol signatures listed in the file-record payload.
const MAX_SYMBOL_NAMES_IN_FILE_PAYLOAD: usize = 50;
/// Directories never walked (build outputs, VCS, package stores).
const DENY_DIRS: [&str; 7] = [
    "target",
    "node_modules",
    ".git",
    "dist",
    "build",
    ".next",
    "__pycache__",
];
/// Key prefix of a file record.
const KEY_FILE_PREFIX: &str = "file:";
/// Key prefix of a symbol record.
const KEY_SYMBOL_PREFIX: &str = "sym:";
/// Edge label between a file record and its symbols.
const EDGE_DEFINES: &str = "defines";

/// Tool definition for `tools/list` (full profile only).
pub(crate) fn code_index_tool_definitions() -> Vec<Value> {
    vec![json!({
        "name": "code_index",
        "description": "Indexes a Rust source tree into a namespace: chunks by symbol and writes file-per-node memory records plus `defines` edges so code_search/code_explore/code_callers/code_callees/code_impact work over the result. Idempotent by content hash; changed files reconcile stale symbols. Writes only its own `file:`/`sym:` keys in the target namespace.",
        "annotations": {
            "title": "Code Index",
            "readOnlyHint": false,
            "destructiveHint": false,
            "idempotentHint": true,
            "openWorldHint": true
        },
        "inputSchema": {
            "type": "object",
            "properties": {
                "namespace": { "type": "string", "description": "Target memory namespace (dedicated, e.g. \"code\")" },
                "path": { "type": "string", "description": "Root directory to scan (Rust files only; denylist dirs skipped)" },
                "max_files": { "type": "number", "description": "Max files to index this run (default 200, capped at 2000)" }
            },
            "required": ["namespace", "path"],
            "additionalProperties": false
        }
    })]
}

/// Dispatch a `tools/call` for `code_index`.
///
/// Annotations rationale (MCP-38): `destructiveHint` stays `false` even though
/// the reconcile deletes stale symbol records — the tool only overwrites and
/// deletes records it owns (`file:`/`sym:` keys in the target namespace) as
/// part of a rebuild, matching the `rebuild_index`/`memory_put` precedent
/// (user-authored data is never destroyed).
pub(crate) fn handle_code_index(
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let namespace = required_str(args, "namespace")?;
    validate_identifier(namespace, "namespace", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    let raw_path = required_str(args, "path")?;
    let max_files = args["max_files"]
        .as_u64()
        .map(|n| n as usize)
        .unwrap_or(DEFAULT_MAX_FILES)
        .clamp(1, MAX_MAX_FILES);

    let root = Path::new(raw_path);
    if !root.exists() {
        return Ok(error_content(format!(
            "code_index: path does not exist: {raw_path}"
        )));
    }
    if !root.is_dir() {
        return Ok(error_content(format!(
            "code_index: path is not a directory: {raw_path}"
        )));
    }

    let embedded = Embedded::from_engine(storage.clone());
    index_repo(&embedded, namespace, root, max_files)
}

/// Extract a required string argument.
fn required_str<'a>(args: &'a Value, field: &str) -> Result<&'a str, Value> {
    args[field]
        .as_str()
        .ok_or_else(|| McpError::invalid_params(format!("Missing or invalid '{field}'")).to_json())
}

// ── Indexing ────────────────────────────────────────────────────────────────

/// Walk `root`, extract symbols per file, and upsert records + `defines` edges.
/// Unchanged files (same content hash) are skipped; changed files reconcile
/// their stale symbols before the rewrite.
fn index_repo(
    embedded: &Embedded,
    namespace: &str,
    root: &Path,
    max_files: usize,
) -> Result<Value, Value> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut truncated = false;
    if let Err(e) = std::fs::read_dir(root) {
        return Ok(error_content(format!(
            "code_index: cannot read directory {root:?}: {e}"
        )));
    }
    collect_rust_files(root, &mut files, &mut truncated, max_files);

    let mut indexed_files = 0usize;
    let mut skipped_unchanged = 0usize;
    let mut files_skipped = 0usize;
    let mut symbols_indexed = 0usize;
    let mut stale_symbols_removed = 0usize;
    let mut symbols_truncated = false;

    for abs in &files {
        let rel = rel_path(root, abs);
        let meta = match std::fs::metadata(abs) {
            Ok(meta) => meta,
            Err(_) => {
                files_skipped += 1;
                continue;
            }
        };
        if meta.len() > MAX_FILE_BYTES {
            files_skipped += 1;
            continue;
        }
        let Ok(source) = std::fs::read_to_string(abs) else {
            files_skipped += 1; // non-UTF-8 or unreadable
            continue;
        };

        let file_key = format!("{KEY_FILE_PREFIX}{rel}");
        let hash = fnv1a_hex(source.as_bytes());
        let existing = embedded
            .get(namespace, &file_key)
            .map_err(|e| McpError::from(e).to_json())?;
        if let Some(record) = &existing {
            if metadata_str(&record.metadata, "hash").as_deref() == Some(hash.as_str()) {
                skipped_unchanged += 1;
                continue;
            }
        }

        let mut symbols = extract_symbols(&source);
        if symbols.len() > MAX_SYMBOLS_PER_FILE {
            symbols_truncated = true;
            symbols.truncate(MAX_SYMBOLS_PER_FILE);
        }
        let mut seen: HashMap<String, usize> = HashMap::new();
        let keyed: Vec<(String, SymbolChunk)> = symbols
            .iter()
            .map(|sym| (symbol_key(&rel, sym, &mut seen), sym.clone()))
            .collect();
        let new_keys: HashSet<&str> = keyed.iter().map(|(k, _)| k.as_str()).collect();
        let old_keys: Vec<String> = existing
            .as_ref()
            .map(|r| metadata_strings(&r.metadata, "symbols"))
            .unwrap_or_default();

        // Reconcile: symbols present before but gone now lose their edge and
        // their record (deterministic keys make the old node id reusable only
        // for keys that survive).
        if let Some(record) = &existing {
            for old_key in &old_keys {
                if new_keys.contains(old_key.as_str()) {
                    continue;
                }
                if let Ok(Some(old)) = embedded.get(namespace, old_key) {
                    let _ = embedded.remove_edge(record.node_id, old.node_id, EDGE_DEFINES);
                    let _ = embedded.delete(namespace, old_key);
                    stale_symbols_removed += 1;
                }
            }
        }

        // File record (manifest: hash + symbol keys).
        let mut file_input =
            MemoryInput::new(namespace, file_key.clone(), file_payload(&rel, &keyed));
        file_input
            .metadata
            .insert("kind".into(), SdkValue::String("file".into()));
        file_input
            .metadata
            .insert("path".into(), SdkValue::String(rel.clone()));
        file_input
            .metadata
            .insert("language".into(), SdkValue::String("rust".into()));
        file_input
            .metadata
            .insert("hash".into(), SdkValue::String(hash));
        file_input
            .metadata
            .insert("symbol_count".into(), SdkValue::Int(keyed.len() as i64));
        file_input.metadata.insert(
            "symbols".into(),
            SdkValue::ListString(keyed.iter().map(|(k, _)| k.clone()).collect()),
        );
        let file_record = embedded
            .put(file_input)
            .map_err(|e| McpError::from(e).to_json())?;

        // Symbol records + `defines` edges (only for keys that are new, so a
        // surviving edge from the previous generation is not duplicated).
        let old_key_set: HashSet<&str> = old_keys.iter().map(|k| k.as_str()).collect();
        for (key, sym) in &keyed {
            let mut input = MemoryInput::new(namespace, key.clone(), sym_payload(&sym.text));
            input
                .metadata
                .insert("kind".into(), SdkValue::String("symbol".into()));
            input
                .metadata
                .insert("symbol_kind".into(), SdkValue::String(sym.kind.into()));
            input
                .metadata
                .insert("path".into(), SdkValue::String(rel.clone()));
            input
                .metadata
                .insert("line_start".into(), SdkValue::Int(sym.line_start as i64));
            input
                .metadata
                .insert("line_end".into(), SdkValue::Int(sym.line_end as i64));
            input
                .metadata
                .insert("file_key".into(), SdkValue::String(file_key.clone()));
            let record = embedded
                .put(input)
                .map_err(|e| McpError::from(e).to_json())?;
            if !old_key_set.contains(key.as_str()) {
                embedded
                    .add_edge(
                        file_record.node_id,
                        record.node_id,
                        EDGE_DEFINES,
                        Some(1.0),
                        None,
                    )
                    .map_err(|e| McpError::from(e).to_json())?;
            }
            symbols_indexed += 1;
        }
        indexed_files += 1;
    }

    Ok(text_content(serialize_content(&json!({
        "namespace": namespace,
        "root": root.to_string_lossy(),
        "indexed_files": indexed_files,
        "skipped_unchanged": skipped_unchanged,
        "symbols_indexed": symbols_indexed,
        "stale_symbols_removed": stale_symbols_removed,
        "files_skipped": files_skipped,
        "truncated": truncated,
        "symbols_truncated": symbols_truncated,
    }))))
}

/// Recursively collect `.rs` files under `dir` in deterministic (sorted)
/// order, skipping symlinks and [`DENY_DIRS`]. Sets `truncated` when the
/// `max_files` cap stops further collection.
fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>, truncated: &mut bool, max_files: usize) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        let Ok(meta) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if DENY_DIRS.contains(&name) {
                    continue;
                }
            }
            collect_rust_files(&path, out, truncated, max_files);
        } else if meta.is_file() && path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if out.len() >= max_files {
                *truncated = true;
                continue;
            }
            out.push(path);
        }
    }
}

/// Path of `abs` relative to `root`, with `/` separators (stable keys on all
/// platforms).
fn rel_path(root: &Path, abs: &Path) -> String {
    abs.strip_prefix(root)
        .unwrap_or(abs)
        .to_string_lossy()
        .replace('\\', "/")
}

/// FNV-1a 64-bit, hex-encoded. Stable across runs and toolchains (unlike
/// `DefaultHasher`), dependency-free, non-cryptographic (content addressing
/// only — not a security primitive).
fn fnv1a_hex(bytes: &[u8]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// Read a string metadata value.
fn metadata_str(meta: &vantadb::sdk::MemoryMetadata, key: &str) -> Option<String> {
    match meta.get(key) {
        Some(SdkValue::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Read a `ListString` metadata value (empty when absent/another type).
fn metadata_strings(meta: &vantadb::sdk::MemoryMetadata, key: &str) -> Vec<String> {
    match meta.get(key) {
        Some(SdkValue::ListString(values)) => values.clone(),
        _ => Vec::new(),
    }
}

/// File-record payload: path plus a bounded signature list (the symbols carry
/// the searchable source text).
fn file_payload(rel: &str, keyed: &[(String, SymbolChunk)]) -> String {
    let mut names: Vec<String> = keyed
        .iter()
        .take(MAX_SYMBOL_NAMES_IN_FILE_PAYLOAD)
        .map(|(_, sym)| format!("{} {}", sym.kind, sym.name))
        .collect();
    if keyed.len() > names.len() {
        names.push(format!("… (+{} more)", keyed.len() - names.len()));
    }
    format!("{rel} (rust)\nsymbols: {}", names.join(", "))
}

/// Symbol payload: source text capped at [`MAX_SYMBOL_PAYLOAD_CHARS`].
fn sym_payload(text: &str) -> String {
    if text.chars().count() <= MAX_SYMBOL_PAYLOAD_CHARS {
        text.to_string()
    } else {
        let mut out: String = text.chars().take(MAX_SYMBOL_PAYLOAD_CHARS).collect();
        out.push_str("\n…[truncated]");
        out
    }
}

/// Deterministic symbol key `sym:{rel}#{kind}:{name}`; repeated `kind:name`
/// pairs in one file get a `~2`, `~3`, … suffix in file order.
fn symbol_key(rel: &str, sym: &SymbolChunk, seen: &mut HashMap<String, usize>) -> String {
    let base = format!("{KEY_SYMBOL_PREFIX}{rel}#{}:{}", sym.kind, sym.name);
    let count = seen.entry(base.clone()).or_insert(0);
    *count += 1;
    if *count == 1 {
        base
    } else {
        format!("{base}~{count}")
    }
}

// ── Symbol chunker (v0: line-based Rust declaration scanner) ────────────────

/// One extracted symbol chunk.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SymbolChunk {
    /// Symbol name (methods are `Type::method`).
    pub name: String,
    /// `fn` | `method` | `struct` | `enum` | `union` | `mod` |
    /// `const` | `static` | `type`.
    pub kind: &'static str,
    /// 1-based first line.
    pub line_start: usize,
    /// 1-based last line.
    pub line_end: usize,
    /// Verbatim source span (trimmed).
    pub text: String,
}

struct ParsedHeader {
    kind: &'static str,
    name: String,
    is_container: bool,
}

/// Extract Rust symbols from `source`.
///
/// v0 heuristic (declared ceiling, FIND-299): a line-based scanner with a
/// code mask that ignores comments and string/char literals. It captures
/// top-level items plus `fn`s directly inside `impl`/`trait` blocks; nested
/// modules and multi-line `impl ... for` headers are out of scope.
pub(crate) fn extract_symbols(source: &str) -> Vec<SymbolChunk> {
    let bytes = source.as_bytes();
    let mask = code_mask(bytes);
    let line_starts = line_starts(bytes);
    let mut out = Vec::new();
    let mut depth: i64 = 0;

    for k in 0..line_starts.len() {
        let ls = line_starts[k];
        let le = line_bounds(&line_starts, bytes.len(), k).1;
        if depth == 0 {
            let text = masked_text(bytes, &mask, ls, le);
            if let Some(header) = parse_item(&text) {
                let (end_off, _is_block) = find_item_end(bytes, &mask, ls);
                let end_off = end_off.min(bytes.len());
                let end_line = line_of(&line_starts, end_off.saturating_sub(1).max(ls));
                if header.is_container {
                    extract_methods_in_span(
                        source,
                        bytes,
                        &mask,
                        &line_starts,
                        k,
                        end_off,
                        &header.name,
                        &mut out,
                    );
                } else {
                    let text = source.get(ls..end_off).unwrap_or("").trim_end().to_string();
                    out.push(SymbolChunk {
                        name: header.name,
                        kind: header.kind,
                        line_start: k + 1,
                        line_end: end_line,
                        text,
                    });
                }
            }
        }
        for i in ls..le {
            if mask[i] {
                match bytes[i] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
            }
        }
    }
    out
}

/// Extract `fn`s directly inside an `impl`/`trait` block (relative depth 1),
/// naming them `{container}::{fn}`.
#[allow(clippy::too_many_arguments)]
fn extract_methods_in_span(
    source: &str,
    bytes: &[u8],
    mask: &[bool],
    line_starts: &[usize],
    header_line: usize,
    region_end: usize,
    container: &str,
    out: &mut Vec<SymbolChunk>,
) {
    let mut depth: i64 = 0;
    let mut seen_open = false;
    for k in header_line..line_starts.len() {
        let ls = line_starts[k];
        if ls >= region_end {
            break;
        }
        let (_, le) = line_bounds(line_starts, bytes.len(), k);
        let le = le.min(region_end);
        if seen_open && depth == 1 && k > header_line {
            let text = masked_text(bytes, mask, ls, le);
            if let Some(name) = parse_fn_name(&text) {
                let (end_off, _is_block) = find_item_end(bytes, mask, ls);
                let end_off = end_off.min(region_end);
                let end_line = line_of(line_starts, end_off.saturating_sub(1).max(ls));
                out.push(SymbolChunk {
                    name: format!("{container}::{name}"),
                    kind: "method",
                    line_start: k + 1,
                    line_end: end_line,
                    text: source.get(ls..end_off).unwrap_or("").trim_end().to_string(),
                });
            }
        }
        for i in ls..le {
            if mask[i] {
                match bytes[i] {
                    b'{' => {
                        depth += 1;
                        seen_open = true;
                    }
                    b'}' => depth -= 1,
                    _ => {}
                }
            }
        }
    }
}

/// Byte offsets of every line start (1-based lines = index + 1).
fn line_starts(bytes: &[u8]) -> Vec<usize> {
    let mut starts = vec![0usize];
    for (i, b) in bytes.iter().enumerate() {
        if *b == b'\n' {
            starts.push(i + 1);
        }
    }
    starts
}

/// `(start, end)` byte offsets of line `k` (end excludes the newline).
fn line_bounds(starts: &[usize], len: usize, k: usize) -> (usize, usize) {
    let start = starts[k];
    let end = starts.get(k + 1).copied().unwrap_or(len);
    (start, end)
}

/// 1-based line number containing `offset`.
fn line_of(starts: &[usize], offset: usize) -> usize {
    match starts.binary_search(&offset) {
        Ok(i) => i + 1,
        Err(i) => i.max(1),
    }
}

/// Mask of code bytes: `false` inside line/block comments, string literals,
/// raw strings and char literals.
fn code_mask(bytes: &[u8]) -> Vec<bool> {
    let mut mask = vec![true; bytes.len()];
    let len = bytes.len();
    let mut i = 0usize;
    while i < len {
        let b = bytes[i];
        // Line comment.
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'/' {
            let start = i;
            while i < len && bytes[i] != b'\n' {
                i += 1;
            }
            mask[start..i].fill(false);
            continue;
        }
        // Block comment (Rust nesting).
        if b == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
            let start = i;
            let mut nest = 1usize;
            i += 2;
            while i < len && nest > 0 {
                if bytes[i] == b'/' && i + 1 < len && bytes[i + 1] == b'*' {
                    nest += 1;
                    i += 2;
                } else if bytes[i] == b'*' && i + 1 < len && bytes[i + 1] == b'/' {
                    nest -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            mask[start..i.min(len)].fill(false);
            continue;
        }
        // String prefix forms: r"…", r#"…"#, b"…", br#"…"#, c"…".
        if (b == b'r' || b == b'b' || b == b'c') && i + 1 < len {
            let mut j = i + 1;
            let mut hashes = 0usize;
            if b == b'r' || bytes[i + 1] == b'r' {
                if bytes[j] == b'r' {
                    j += 1;
                }
                while j < len && bytes[j] == b'#' {
                    hashes += 1;
                    j += 1;
                }
                if j < len && bytes[j] == b'"' {
                    let start = i;
                    i = skip_raw_string(bytes, j + 1, hashes);
                    mask[start..i.min(len)].fill(false);
                    continue;
                }
                if b == b'r' {
                    mask[i] = true;
                    i += 1;
                    continue;
                }
            }
            if (b == b'b' || b == b'c') && bytes[i + 1] == b'"' {
                let start = i;
                i = skip_normal_string(bytes, i + 1);
                mask[start..i.min(len)].fill(false);
                continue;
            }
            if b == b'b' && bytes[i + 1] == b'\'' {
                let start = i;
                i = skip_char(bytes, i + 1);
                mask[start..i.min(len)].fill(false);
                continue;
            }
        }
        if b == b'"' {
            let start = i;
            i = skip_normal_string(bytes, i);
            mask[start..i.min(len)].fill(false);
            continue;
        }
        if b == b'\'' {
            // Char literal (`'x'`, `'\n'`) vs lifetime (`'a`).
            let next = i + 1;
            if next < len && bytes[next] == b'\\' {
                let end = skip_char(bytes, i);
                if end > i + 1 {
                    mask[i..end.min(len)].fill(false);
                    i = end;
                    continue;
                }
            } else if next + 1 < len && bytes[next + 1] == b'\'' {
                mask[i..(i + 3).min(len)].fill(false);
                i += 3;
                continue;
            }
            mask[i] = true;
            i += 1;
            continue;
        }
        i += 1;
    }
    mask
}

/// Body of a `"…"` string starting at `open` (index of the quote); returns the
/// index just past the closing quote (or the line end on an unterminated
/// string, to stay conservative).
fn skip_normal_string(bytes: &[u8], open: usize) -> usize {
    let len = bytes.len();
    let mut i = open + 1;
    while i < len {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => return i + 1,
            b'\n' => return i, // unterminated on this line: stop
            _ => i += 1,
        }
    }
    len
}

/// Body of a raw string: starts after the opening quote; closes on
/// `"` followed by `hashes` `#`s.
fn skip_raw_string(bytes: &[u8], mut i: usize, hashes: usize) -> usize {
    let len = bytes.len();
    while i < len {
        if bytes[i] == b'"' {
            let mut matched = 0usize;
            while matched < hashes && i + 1 + matched < len && bytes[i + 1 + matched] == b'#' {
                matched += 1;
            }
            if matched == hashes {
                return i + 1 + hashes;
            }
        }
        i += 1;
    }
    len
}

/// Body of a char literal whose opening quote sits at `open`; returns the
/// index just past the closing quote.
fn skip_char(bytes: &[u8], open: usize) -> usize {
    let len = bytes.len();
    let mut i = open + 1;
    if i < len && bytes[i] == b'\\' {
        i += 1;
        if i < len && bytes[i] == b'u' && i + 1 < len && bytes[i + 1] == b'{' {
            // `'\u{1F600}'`
            while i < len && bytes[i] != b'}' {
                i += 1;
            }
            i += 1;
        } else {
            i += 1;
        }
    } else if i < len {
        i += 1;
    }
    if i < len && bytes[i] == b'\'' {
        i + 1
    } else {
        open + 1 // not a char literal (lifetime): consume only the quote
    }
}

/// Reconstruct the code text of `[start, end)` ignoring masked bytes.
fn masked_text(bytes: &[u8], mask: &[bool], start: usize, end: usize) -> String {
    let buf: Vec<u8> = (start..end.min(mask.len()))
        .filter(|&k| mask[k])
        .map(|k| bytes[k])
        .collect();
    String::from_utf8_lossy(&buf).into_owned()
}

/// Scan from `from` for the item terminator: `;` at zero paren/bracket depth,
/// or the `}` matching the first `{`. Returns `(end offset, is_block)`.
fn find_item_end(bytes: &[u8], mask: &[bool], from: usize) -> (usize, bool) {
    let mut parens: i64 = 0;
    let mut brackets: i64 = 0;
    let mut depth: i64 = 0;
    let mut saw_open = false;
    let mut i = from;
    while i < bytes.len() {
        if mask[i] {
            match bytes[i] {
                b'(' => parens += 1,
                b')' => parens -= 1,
                b'[' => brackets += 1,
                b']' => brackets -= 1,
                b'{' => {
                    depth += 1;
                    saw_open = true;
                }
                b'}' => {
                    depth -= 1;
                    if saw_open && depth == 0 {
                        return (i + 1, true);
                    }
                }
                b';' if depth == 0 && parens <= 0 && brackets <= 0 => {
                    return (i + 1, false);
                }
                _ => {}
            }
        }
        i += 1;
    }
    (bytes.len(), saw_open)
}

/// Parse an item header from a masked line. Returns `None` for non-items.
fn parse_item(line: &str) -> Option<ParsedHeader> {
    let rest = strip_modifiers(line.trim())?;
    if let Some(name) = keyword_name(rest, "fn ") {
        return Some(ParsedHeader {
            kind: "fn",
            name,
            is_container: false,
        });
    }
    for (kw, kind) in [
        ("struct ", "struct"),
        ("enum ", "enum"),
        ("union ", "union"),
        ("mod ", "mod"),
        ("type ", "type"),
        ("const ", "const"),
        ("static ", "static"),
    ] {
        if let Some(rest_kw) = rest.strip_prefix(kw) {
            let name_src = if kind == "static" {
                rest_kw.strip_prefix("mut ").unwrap_or(rest_kw)
            } else {
                rest_kw
            };
            if let Some(name) = take_ident(name_src) {
                return Some(ParsedHeader {
                    kind,
                    name,
                    is_container: false,
                });
            }
            return None;
        }
    }
    if rest.starts_with("impl") {
        let head = rest.strip_prefix("impl")?;
        let name = impl_self_type(head)?;
        return Some(ParsedHeader {
            kind: "impl",
            name,
            is_container: true,
        });
    }
    if let Some(rest_kw) = rest.strip_prefix("trait ") {
        if let Some(name) = take_ident(rest_kw) {
            return Some(ParsedHeader {
                kind: "trait",
                name,
                is_container: true,
            });
        }
    }
    None
}

/// Name of a `fn` header (masked line), if the line declares one.
fn parse_fn_name(line: &str) -> Option<String> {
    let rest = strip_modifiers(line.trim())?;
    keyword_name(rest, "fn ")
}

/// Strip visibility and item modifiers (`pub`, `pub(crate)`, `async`,
/// `unsafe`, `const fn`, `extern "…"`) from a header. Returns `None` when the
/// line does not look like an item head at all.
fn strip_modifiers(line: &str) -> Option<&str> {
    let mut r = line.trim_start();
    if r.starts_with('#') || r.is_empty() {
        return None; // attribute line or empty
    }
    for _ in 0..8 {
        if r.starts_with("pub ") {
            r = r[4..].trim_start();
        } else if r.starts_with("pub(") {
            let close = r.find(')')?;
            r = r[close + 1..].trim_start();
        } else if r.starts_with("async ") {
            r = r[6..].trim_start();
        } else if r.starts_with("unsafe ") {
            r = r[7..].trim_start();
        } else if r.starts_with("const fn ") {
            // Strip only `const`, keep `fn` as the item keyword.
            r = r[6..].trim_start();
        } else if r.starts_with("extern ") {
            let after = r[7..].trim_start();
            if let Some(stripped) = after.strip_prefix('"') {
                let close = stripped.find('"')?;
                r = stripped[close + 1..].trim_start();
            } else {
                r = after;
            }
        } else {
            break;
        }
    }
    Some(r)
}

/// Identifier after `kw` in a header (stops at `<`, `(`, `:`, whitespace).
fn keyword_name(rest: &str, kw: &str) -> Option<String> {
    take_ident(rest.strip_prefix(kw)?)
}

/// Take a leading Rust identifier.
fn take_ident(s: &str) -> Option<String> {
    let ident: String = s
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if ident.is_empty() {
        None
    } else {
        Some(ident)
    }
}

/// Self type of an `impl` header: `impl<T> Foo<T> {` → `Foo`,
/// `impl fmt::Display for MyErr {` → `MyErr`,
/// `impl Trait for &mut Bar {` → `Bar`,
/// `impl<T> Foo<T> where T: Clone {` → `Foo`.
fn impl_self_type(head: &str) -> Option<String> {
    let mut rest = head.trim_start();
    // Skip generics `<…>`.
    if rest.starts_with('<') {
        let mut depth = 0i64;
        let mut end = None;
        for (i, c) in rest.char_indices() {
            match c {
                '<' => depth += 1,
                '>' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(i + c.len_utf8());
                        break;
                    }
                }
                _ => {}
            }
        }
        rest = rest.get(end?..).unwrap_or("").trim_start();
    }
    let mut target = match rest.rfind(" for ") {
        Some(pos) => rest[pos + 5..].trim(),
        None => rest,
    };
    // Drop a trailing where-clause or the opening brace.
    if let Some(pos) = target.find(" where ") {
        target = target[..pos].trim_end();
    }
    if let Some(pos) = target.find('{') {
        target = target[..pos].trim_end();
    }
    // Strip references, pointers, lifetimes and modifiers: `&'a mut Foo` → `Foo`.
    let mut t = target;
    loop {
        let stripped = t
            .strip_prefix('&')
            .or_else(|| t.strip_prefix('*'))
            .or_else(|| {
                t.strip_prefix('\'').map(|r| {
                    let cut = r
                        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .unwrap_or(r.len());
                    &r[cut..]
                })
            })
            .or_else(|| t.strip_prefix("mut "))
            .or_else(|| t.strip_prefix("const "))
            .or_else(|| t.strip_prefix("dyn "));
        match stripped {
            Some(r) => t = r.trim_start(),
            None => break,
        }
    }
    // Type name: cut generic arguments, take the last path segment, keep the ident.
    let t = t.split('<').next().unwrap_or(t);
    let t = t.rsplit("::").next().unwrap_or(t);
    let name: String = t
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(chunks: &[SymbolChunk]) -> Vec<(&str, &str)> {
        chunks.iter().map(|c| (c.kind, c.name.as_str())).collect()
    }

    #[test]
    fn extract_symbols_captures_top_level_items() {
        let src = "/// doc\npub fn hello() {\n    println!(\"hi\");\n}\n\npub struct Thing {\n    pub x: i32,\n}\n\npub enum Color { Red }\npub const LIMIT: usize = 3;\n";
        let chunks = extract_symbols(src);
        assert_eq!(
            names(&chunks),
            vec![
                ("fn", "hello"),
                ("struct", "Thing"),
                ("enum", "Color"),
                ("const", "LIMIT"),
            ]
        );
        assert_eq!(chunks[0].line_start, 2);
        assert_eq!(chunks[0].line_end, 4);
        assert!(chunks[0].text.contains("pub fn hello"));
    }

    #[test]
    fn braces_in_strings_comments_and_chars_do_not_confuse_spans() {
        let src = "pub fn f() {\n    let s = \"} not a brace {\";\n    // } comment {\n    let c = '}';\n    let r = r#\"{\"json\": {}}\"#;\n}\npub fn g() {}\n";
        let chunks = extract_symbols(src);
        assert_eq!(names(&chunks), vec![("fn", "f"), ("fn", "g")]);
        assert_eq!(chunks[0].line_end, 6, "string/comment braces ignored");
        assert_eq!(chunks[1].line_start, 7);
    }

    #[test]
    fn lifetimes_do_not_swallow_declarations() {
        let src = "pub struct Holder<'a> {\n    inner: &'a str,\n}\npub fn borrow<'a>(s: &'a str) -> &'a str { s }\n";
        let chunks = extract_symbols(src);
        assert_eq!(names(&chunks), vec![("struct", "Holder"), ("fn", "borrow")]);
        assert_eq!(chunks[1].line_end, 4);
    }

    #[test]
    fn impl_and_trait_blocks_yield_typed_methods() {
        let src = "pub trait Greet {\n    fn greet(&self);\n}\n\nimpl Greet for Foo {\n    fn greet(&self) {}\n    pub fn extra(&mut self) -> usize {\n        1\n    }\n}\n";
        let chunks = extract_symbols(src);
        assert_eq!(
            names(&chunks),
            vec![
                ("method", "Greet::greet"),
                ("method", "Foo::greet"),
                ("method", "Foo::extra"),
            ]
        );
        assert_eq!(chunks[0].line_start, 2, "trait required method");
        assert_eq!(chunks[2].line_start, 7);
        assert_eq!(chunks[2].line_end, 9);
    }

    #[test]
    fn impl_containers_resolve_through_where_clauses_and_references() {
        let src = "impl<T> Foo<T>\nwhere\n    T: Clone,\n{\n    fn m(&self) {}\n}\n\nimpl Trait for &mut Bar {\n    fn n(&self) {}\n}\n";
        let chunks = extract_symbols(src);
        assert_eq!(
            names(&chunks),
            vec![("method", "Foo::m"), ("method", "Bar::n")],
            "where-clauses and reference self types must not corrupt the container name"
        );
    }

    #[test]
    fn modifiers_and_visibility_are_stripped() {
        let src = "pub(crate) async fn a() {}\nunsafe fn b() {}\nconst fn c() {}\npub const X: u8 = 1;\nstatic mut Y: u8 = 0;\npub type T = u8;\npub union U { f: u8 }\n";
        let chunks = extract_symbols(src);
        assert_eq!(
            names(&chunks),
            vec![
                ("fn", "a"),
                ("fn", "b"),
                ("fn", "c"),
                ("const", "X"),
                ("static", "Y"),
                ("type", "T"),
                ("union", "U"),
            ]
        );
    }

    #[test]
    fn semicolons_inside_types_do_not_end_the_item_early() {
        let src = "pub fn f(x: [u8; 3]) -> usize { x.len() }\n";
        let chunks = extract_symbols(src);
        assert_eq!(names(&chunks), vec![("fn", "f")]);
        assert_eq!(
            chunks[0].line_end, 1,
            "`;` inside `[...]` is not the terminator"
        );
    }

    #[test]
    fn empty_and_comment_only_sources_yield_nothing() {
        assert!(extract_symbols("").is_empty());
        assert!(extract_symbols("// just a comment\n/* block { } */\n").is_empty());
    }

    #[test]
    fn unterminated_block_is_clamped_to_eof() {
        let src = "pub fn f() {\n    let x = 1;\n";
        let chunks = extract_symbols(src);
        assert_eq!(names(&chunks), vec![("fn", "f")]);
        assert_eq!(chunks[0].line_end, 2, "clamped to the last content line");
    }

    #[test]
    fn duplicate_symbol_names_get_stable_suffixes() {
        let mut seen = HashMap::new();
        let chunk = SymbolChunk {
            name: "x".into(),
            kind: "fn",
            line_start: 1,
            line_end: 1,
            text: String::new(),
        };
        assert_eq!(symbol_key("a.rs", &chunk, &mut seen), "sym:a.rs#fn:x");
        assert_eq!(symbol_key("a.rs", &chunk, &mut seen), "sym:a.rs#fn:x~2");
        assert_eq!(symbol_key("a.rs", &chunk, &mut seen), "sym:a.rs#fn:x~3");
    }

    #[test]
    fn fnv1a_matches_known_vectors() {
        assert_eq!(fnv1a_hex(b""), "cbf29ce484222325");
        assert_eq!(fnv1a_hex(b"a"), "af63dc4c8601ec8c");
    }
}
