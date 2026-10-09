// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEMG-09 — `code_index` MCP tool (MGR-22 slice v0: symbol chunker + file-per-node).
//!
//! Contract: `code_index` scans a Rust source tree, chunks by symbol, and writes
//! file-per-node memory records (`file:{rel}` / `sym:{rel}#{kind}:{name}`) plus
//! `defines` edges into the target namespace, so the existing read-only `code_*`
//! tools can search and traverse it. Idempotent by content hash; a changed file
//! reconciles its stale symbols (edge removed + record deleted).

use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::executor::Executor;
use vantadb::sdk::Value as SdkValue;
use vantadb::storage::StorageEngine;
use vantadb_mcp::{handle_tools_call, handle_tools_list, McpConfig, McpProfile};

fn full_config() -> McpConfig {
    McpConfig {
        profile: McpProfile::Full,
        ..Default::default()
    }
}

fn agent_config() -> McpConfig {
    McpConfig {
        profile: McpProfile::Agent,
        ..Default::default()
    }
}

fn setup_storage() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let db_path = dir.path().to_str().unwrap();
    let storage = Arc::new(StorageEngine::open(db_path).expect("Failed to open StorageEngine"));
    // MCP-01/AUD-044 pattern: ensure the text_index registry exists so
    // code_search's BM25 seeding works (mirrors code_tests.rs).
    let embedded = vantadb::Embedded::from_engine(storage.clone());
    embedded
        .ensure_indexes_current()
        .expect("startup index ensure should succeed");
    (dir, storage)
}

fn call(
    name: &str,
    args: Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let executor = Executor::new(storage);
    handle_tools_call(
        &Some(json!({ "name": name, "arguments": args })),
        &executor,
        storage,
        config,
    )
}

/// Message of a tool result (error_content text or JSON-RPC error message).
fn msg(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

/// Parse the `code_index` report JSON out of the tool response text.
fn report(res: Result<Value, Value>) -> Value {
    let text = msg(res);
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("code_index report must be JSON: {e}\n{text}"))
}

/// Write `files` (relative path → content) under a fresh temp repo root.
fn write_repo(files: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = tempdir().unwrap();
    for (rel, content) in files {
        let path = dir.path().join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }
    dir
}

/// Call `code_index` with a repo root and optional extra args.
fn index_call(
    ns: &str,
    root: &std::path::Path,
    extra: Value,
    storage: &Arc<StorageEngine>,
    cfg: &McpConfig,
) -> Value {
    let mut args = json!({"namespace": ns, "path": root.to_str().unwrap()});
    if let Some(map) = extra.as_object() {
        for (k, v) in map {
            args[k] = v.clone();
        }
    }
    report(call("code_index", args, storage, cfg))
}

/// Open a view over the storage for direct record assertions.
fn embedded(storage: &Arc<StorageEngine>) -> vantadb::Embedded {
    vantadb::Embedded::from_engine(storage.clone())
}

const SYMBOLS_RS: &str = "\
/// Greets.\n\
pub fn hello() {\n\
    println!(\"hi\");\n\
}\n\
\n\
pub struct Thing {\n\
    pub x: i32,\n\
}\n\
\n\
impl Thing {\n\
    pub fn new() -> Self {\n\
        Thing { x: 1 }\n\
    }\n\
}\n\
\n\
pub enum Color { Red, Green }\n\
\n\
pub const LIMIT: usize = 3;\n";

// ── Tool surface (WIRE-02 profile discipline) ───────────────────────────────

#[test]
fn code_index_is_listed_in_full_profile_only() {
    let full = handle_tools_list(&full_config()).expect("tools/list full");
    let full_names: Vec<&str> = full["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(
        full_names.contains(&"code_index"),
        "code_index must be listed in the full profile"
    );

    let agent = handle_tools_list(&agent_config()).expect("tools/list agent");
    let agent_names: Vec<&str> = agent["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(
        !agent_names.contains(&"code_index"),
        "code_index must NOT be listed in the agent profile (default surface stays ≤38)"
    );

    // Writer tool: excluded from every non-`full` profile.
    for (label, profile) in [("dev", McpProfile::Dev), ("memory", McpProfile::Memory)] {
        let list = handle_tools_list(&McpConfig {
            profile,
            ..Default::default()
        })
        .expect("tools/list");
        let names: Vec<&str> = list["tools"]
            .as_array()
            .expect("tools array")
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        assert!(
            !names.contains(&"code_index"),
            "code_index must NOT be listed in the {label} profile"
        );
    }
}

#[test]
fn code_index_rejected_outside_full_profile() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[("src/a.rs", "pub fn a() {}\n")]);

    let err = msg(call(
        "code_index",
        json!({"namespace": "codeidx", "path": repo.path().to_str().unwrap()}),
        &storage,
        &agent_config(),
    ));
    assert!(
        err.contains("not in profile agent"),
        "agent profile must reject code_index with the documented message: {err}"
    );
}

// ── Symbol extraction (chunker v0) ──────────────────────────────────────────

#[test]
fn code_index_extracts_symbols_with_kinds_lines_and_payload() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[("src/lib.rs", SYMBOLS_RS)]);

    let rep = index_call("codeidx", repo.path(), json!({}), &storage, &full_config());
    assert_eq!(rep["indexed_files"], 1, "one file indexed: {rep}");
    assert_eq!(rep["symbols_indexed"], 5, "five symbols: {rep}");
    assert_eq!(rep["stale_symbols_removed"], 0);
    assert_eq!(rep["truncated"], false);

    let db = embedded(&storage);

    // Free function: payload carries the source text, metadata the range.
    let hello = db
        .get("codeidx", "sym:src/lib.rs#fn:hello")
        .expect("get hello")
        .expect("hello symbol must exist");
    assert!(
        hello.payload.contains("pub fn hello"),
        "payload must carry the symbol source: {}",
        hello.payload
    );
    assert_eq!(
        hello.metadata.get("symbol_kind"),
        Some(&SdkValue::String("fn".into()))
    );
    assert_eq!(hello.metadata.get("line_start"), Some(&SdkValue::Int(2)));
    assert_eq!(hello.metadata.get("line_end"), Some(&SdkValue::Int(4)));
    assert_eq!(
        hello.metadata.get("file_key"),
        Some(&SdkValue::String("file:src/lib.rs".into()))
    );

    // Container + method inside `impl` (name = Type::method).
    assert!(
        db.get("codeidx", "sym:src/lib.rs#struct:Thing")
            .expect("get struct")
            .is_some(),
        "struct Thing must be indexed"
    );
    let new = db
        .get("codeidx", "sym:src/lib.rs#method:Thing::new")
        .expect("get method")
        .expect("method Thing::new must be indexed");
    assert_eq!(new.metadata.get("line_start"), Some(&SdkValue::Int(11)));
    assert_eq!(new.metadata.get("line_end"), Some(&SdkValue::Int(13)));

    // Single-line items.
    assert!(
        db.get("codeidx", "sym:src/lib.rs#enum:Color")
            .expect("get enum")
            .is_some(),
        "enum Color must be indexed"
    );
    assert!(
        db.get("codeidx", "sym:src/lib.rs#const:LIMIT")
            .expect("get const")
            .is_some(),
        "const LIMIT must be indexed"
    );

    // File record: manifest metadata for reconcile + map payload.
    let file = db
        .get("codeidx", "file:src/lib.rs")
        .expect("get file")
        .expect("file record must exist");
    assert_eq!(
        file.metadata.get("kind"),
        Some(&SdkValue::String("file".into()))
    );
    assert_eq!(
        file.metadata.get("language"),
        Some(&SdkValue::String("rust".into()))
    );
    match file.metadata.get("symbols") {
        Some(SdkValue::ListString(keys)) => assert_eq!(keys.len(), 5, "manifest has 5 keys"),
        other => panic!("file metadata `symbols` must be ListString, got {other:?}"),
    }
    assert!(
        matches!(file.metadata.get("hash"), Some(SdkValue::String(h)) if !h.is_empty()),
        "content hash must be recorded"
    );
}

// ── File-per-node graph: `defines` edges ────────────────────────────────────

#[test]
fn code_index_creates_defines_edges_resolvable_by_code_tools() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[("src/lib.rs", SYMBOLS_RS)]);
    let cfg = full_config();
    let rep = index_call("codeidx", repo.path(), json!({}), &storage, &cfg);
    assert_eq!(rep["indexed_files"], 1);

    let db = embedded(&storage);
    let file = db
        .get("codeidx", "file:src/lib.rs")
        .expect("get file")
        .expect("file exists");
    let file_node = db
        .get_node(file.node_id)
        .expect("get file node")
        .expect("file node exists");

    // Outgoing `defines` edges (one per symbol, no reverse halves counted).
    let defines: Vec<&vantadb::sdk::EdgeRecord> = file_node
        .edges
        .iter()
        .filter(|e| e.label == "defines" && !e.reverse)
        .collect();
    assert_eq!(
        defines.len(),
        5,
        "one defines edge per symbol: {:?}",
        file_node.edges
    );

    // The existing read-only tool resolves the edge (callees of the file).
    let callees = msg(call(
        "code_callees",
        json!({"node_id": file.node_id.to_string()}),
        &storage,
        &cfg,
    ));
    let hello = db
        .get("codeidx", "sym:src/lib.rs#fn:hello")
        .expect("get hello")
        .expect("hello exists");
    assert!(
        callees.contains(&hello.node_id.to_string()),
        "code_callees(file) must list the hello symbol: {callees}"
    );

    // Reverse direction: callers of a symbol = the file node.
    let callers = msg(call(
        "code_callers",
        json!({"node_id": hello.node_id.to_string()}),
        &storage,
        &cfg,
    ));
    assert!(
        callers.contains(&file.node_id.to_string()),
        "code_callers(symbol) must list the file node: {callers}"
    );
}

// ── Idempotency + reconcile ─────────────────────────────────────────────────

#[test]
fn reindex_unchanged_repo_is_noop() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[("src/lib.rs", SYMBOLS_RS)]);
    let cfg = full_config();

    let first = index_call("codeidx", repo.path(), json!({}), &storage, &cfg);
    assert_eq!(first["indexed_files"], 1);
    assert_eq!(first["skipped_unchanged"], 0);
    assert_eq!(first["symbols_indexed"], 5);

    let second = index_call("codeidx", repo.path(), json!({}), &storage, &cfg);
    assert_eq!(second["indexed_files"], 0, "no file re-indexed: {second}");
    assert_eq!(
        second["skipped_unchanged"], 1,
        "file skipped by hash: {second}"
    );
    assert_eq!(second["symbols_indexed"], 0);
    assert_eq!(second["stale_symbols_removed"], 0);

    // No version bump on skip: the record was never re-put.
    let db = embedded(&storage);
    let file = db
        .get("codeidx", "file:src/lib.rs")
        .expect("get")
        .expect("exists");
    assert_eq!(file.version, 1, "skip must not re-put the file record");
    let hello = db
        .get("codeidx", "sym:src/lib.rs#fn:hello")
        .expect("get")
        .expect("exists");
    assert_eq!(hello.version, 1, "skip must not re-put symbol records");
}

#[test]
fn changed_file_removes_stale_symbols_and_edges() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[(
        "src/lib.rs",
        "pub fn alpha() {\n    let _ = 1;\n}\n\npub fn beta() {\n    let _ = 2;\n}\n",
    )]);
    let cfg = full_config();

    let first = index_call("codeidx", repo.path(), json!({}), &storage, &cfg);
    assert_eq!(first["symbols_indexed"], 2);

    let db = embedded(&storage);
    let beta = db
        .get("codeidx", "sym:src/lib.rs#fn:beta")
        .expect("get beta")
        .expect("beta exists");
    let beta_id = beta.node_id;
    let file = db
        .get("codeidx", "file:src/lib.rs")
        .expect("get")
        .expect("exists");
    let file_id = file.node_id;

    // Rewrite: beta removed, gamma added.
    std::fs::write(
        repo.path().join("src/lib.rs"),
        "pub fn alpha() {\n    let _ = 1;\n}\n\npub fn gamma() {\n    let _ = 3;\n}\n",
    )
    .unwrap();

    let second = index_call("codeidx", repo.path(), json!({}), &storage, &cfg);
    assert_eq!(second["indexed_files"], 1);
    assert_eq!(
        second["stale_symbols_removed"], 1,
        "beta reconciled: {second}"
    );
    assert_eq!(second["symbols_indexed"], 2);

    // Stale symbol: record gone, edge gone, node gone.
    assert!(
        db.get("codeidx", "sym:src/lib.rs#fn:beta")
            .expect("get beta after")
            .is_none(),
        "stale beta record must be deleted"
    );
    let file_after = db
        .get_node(file_id)
        .expect("get node")
        .expect("file node alive");
    let defines: Vec<u128> = file_after
        .edges
        .iter()
        .filter(|e| e.label == "defines" && !e.reverse)
        .map(|e| e.target)
        .collect();
    assert_eq!(
        defines.len(),
        2,
        "no duplicate/stale defines edges: {defines:?}"
    );
    assert!(
        !defines.contains(&beta_id),
        "edge to the removed symbol must be gone"
    );
    assert!(
        db.get("codeidx", "sym:src/lib.rs#fn:gamma")
            .expect("get gamma")
            .is_some(),
        "new symbol gamma must be indexed"
    );
}

// ── Integration with the existing graphrag search ──────────────────────────

#[test]
fn code_search_seeds_from_indexed_namespace() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[(
        "src/zoo.rs",
        "pub fn zebra_quokka() {\n    let marker = \"unique-wombat-token\";\n    let _ = marker;\n}\n",
    )]);
    let cfg = full_config();
    let rep = index_call("codeidx", repo.path(), json!({}), &storage, &cfg);
    assert_eq!(rep["indexed_files"], 1);

    let text = msg(call(
        "code_search",
        json!({"namespace": "codeidx", "query": "unique-wombat-token"}),
        &storage,
        &cfg,
    ));
    let search: Value = serde_json::from_str(&text).expect("code_search JSON");
    let symbol = embedded(&storage)
        .get("codeidx", "sym:src/zoo.rs#fn:zebra_quokka")
        .expect("get symbol")
        .expect("symbol exists");
    let nodes = search["nodes"].as_array().expect("nodes array");
    assert!(
        nodes
            .iter()
            .any(|n| n["id"].as_str() == Some(&symbol.node_id.to_string())),
        "graphrag must seed from the indexed symbol record: {text}"
    );
    assert_eq!(search["stats"]["seeds_found"], 1, "one seed: {text}");
    // The `defines` edge expands the search to the file node (traversal works).
    // NOTE: graphrag's `content` stays empty for memory records (it reads
    // `content|payload|text|description`, not the reserved `__vanta_payload`);
    // the payload is read via `code_node` (pre-existing gap → FIND-300).
    let file = embedded(&storage)
        .get("codeidx", "file:src/zoo.rs")
        .expect("get file")
        .expect("file exists");
    assert!(
        nodes
            .iter()
            .any(|n| n["id"].as_str() == Some(&file.node_id.to_string())),
        "expansion over `defines` must reach the file node: {text}"
    );
}

// ── Argument validation + walk shaping ─────────────────────────────────────

#[test]
fn code_index_rejects_invalid_root() {
    let (_dir, storage) = setup_storage();
    let cfg = full_config();

    let _ghost = tempdir().unwrap();
    let missing_path = _ghost.path().join("definitely-not-there");
    let missing = msg(call(
        "code_index",
        json!({"namespace": "codeidx", "path": missing_path.to_str().unwrap()}),
        &storage,
        &cfg,
    ));
    assert!(
        missing.contains("does not exist") || missing.contains("not found"),
        "nonexistent root must produce an actionable error: {missing}"
    );

    let (_fdir, file_path) = {
        let d = tempdir().unwrap();
        let p = d.path().join("plain.txt");
        std::fs::write(&p, "not a dir").unwrap();
        (d, p)
    };
    let not_dir = msg(call(
        "code_index",
        json!({"namespace": "codeidx", "path": file_path.to_str().unwrap()}),
        &storage,
        &cfg,
    ));
    assert!(
        not_dir.contains("not a directory"),
        "file-as-root must produce an actionable error: {not_dir}"
    );

    let no_path = msg(call(
        "code_index",
        json!({"namespace": "codeidx"}),
        &storage,
        &cfg,
    ));
    assert!(
        no_path.contains("path"),
        "missing path must be reported: {no_path}"
    );
}

#[test]
fn code_index_skips_denylist_dirs_and_non_rust_files() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[
        ("src/keep.rs", "pub fn keep() {}\n"),
        ("target/junk.rs", "pub fn junk() {}\n"),
        ("node_modules/dep.rs", "pub fn dep() {}\n"),
        ("docs/readme.md", "# not rust\n"),
    ]);

    let rep = index_call("codeidx", repo.path(), json!({}), &storage, &full_config());
    assert_eq!(rep["indexed_files"], 1, "only src/keep.rs: {rep}");

    let db = embedded(&storage);
    assert!(
        db.get("codeidx", "file:src/keep.rs")
            .expect("get")
            .is_some(),
        "kept file indexed"
    );
    assert!(
        db.get("codeidx", "file:target/junk.rs")
            .expect("get")
            .is_none(),
        "target/ must be skipped"
    );
    assert!(
        db.get("codeidx", "file:node_modules/dep.rs")
            .expect("get")
            .is_none(),
        "node_modules/ must be skipped"
    );
}

#[test]
fn code_index_caps_max_files_and_reports_truncation() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[
        ("a.rs", "pub fn a() {}\n"),
        ("b.rs", "pub fn b() {}\n"),
        ("c.rs", "pub fn c() {}\n"),
    ]);

    let rep = index_call(
        "codeidx",
        repo.path(),
        json!({"max_files": 2}),
        &storage,
        &full_config(),
    );
    assert_eq!(
        rep["indexed_files"], 2,
        "max_files must bound the walk: {rep}"
    );
    assert_eq!(rep["truncated"], true, "truncation must be reported: {rep}");
}

// ── Declared caps (trust-boundary hygiene) ──────────────────────────────────

#[test]
fn code_index_skips_oversized_and_non_utf8_files() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[("src/ok.rs", "pub fn ok() {}\n")]);
    // > 512 KB cap.
    let big = "// filler\n".repeat(64 * 1024);
    std::fs::write(repo.path().join("src/big.rs"), big).unwrap();
    // Invalid UTF-8 bytes.
    std::fs::write(repo.path().join("src/bin.rs"), [0xFF, 0xFE, 0x00, 0x41]).unwrap();

    let rep = index_call("codeidx", repo.path(), json!({}), &storage, &full_config());
    assert_eq!(rep["indexed_files"], 1, "only src/ok.rs: {rep}");
    assert_eq!(
        rep["files_skipped"], 2,
        "oversized + non-UTF-8 counted: {rep}"
    );
}

#[test]
fn code_index_declares_symbol_truncation_and_payload_cap() {
    let (_dir, storage) = setup_storage();
    // One fn with a >4000-char body, one file with 301 fns (> 300 cap).
    let body = "    let _ = 1;\n".repeat(400);
    let huge_fn = format!("pub fn huge() {{\n{body}}}\n");
    let mut many = String::new();
    for i in 0..301 {
        many.push_str(&format!("pub fn f{i}() {{}}\n"));
    }
    let repo = write_repo(&[
        ("src/huge.rs", huge_fn.as_str()),
        ("src/many.rs", many.as_str()),
    ]);

    let rep = index_call("codeidx", repo.path(), json!({}), &storage, &full_config());
    assert_eq!(rep["indexed_files"], 2, "{rep}");
    assert_eq!(
        rep["symbols_truncated"], true,
        "symbol cap must be declared, never silent: {rep}"
    );

    let db = embedded(&storage);
    let huge = db
        .get("codeidx", "sym:src/huge.rs#fn:huge")
        .expect("get huge")
        .expect("huge exists");
    assert!(
        huge.payload.ends_with("…[truncated]"),
        "payload must carry the truncation marker"
    );
    assert_eq!(
        huge.payload.chars().count(),
        4000 + "\n…[truncated]".chars().count(),
        "payload capped at 4000 chars + marker"
    );
    assert!(
        db.get("codeidx", "sym:src/many.rs#fn:f299")
            .expect("get f299")
            .is_some(),
        "symbols up to the cap are kept"
    );
    assert!(
        db.get("codeidx", "sym:src/many.rs#fn:f300")
            .expect("get f300")
            .is_none(),
        "symbols beyond the cap are not written (declared via symbols_truncated)"
    );
}

#[test]
fn code_index_skips_symlinked_entries() {
    let (_dir, storage) = setup_storage();
    let repo = write_repo(&[("src/real.rs", "pub fn real() {}\n")]);
    let link = repo.path().join("src/link.rs");
    #[cfg(windows)]
    let created =
        std::os::windows::fs::symlink_file(repo.path().join("src/real.rs"), &link).is_ok();
    #[cfg(unix)]
    let created = std::os::unix::fs::symlink(repo.path().join("src/real.rs"), &link).is_ok();
    if !created {
        eprintln!(
            "code_index_skips_symlinked_entries: symlink creation not permitted in this \
             environment; the real-file assertion still holds"
        );
    }

    let rep = index_call("codeidx", repo.path(), json!({}), &storage, &full_config());
    assert_eq!(
        rep["indexed_files"], 1,
        "a symlinked entry must not add a second file: {rep}"
    );
    let db = embedded(&storage);
    assert!(
        db.get("codeidx", "file:src/link.rs")
            .expect("get link")
            .is_none(),
        "symlink must not be indexed as a file"
    );
}
