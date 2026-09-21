// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-44 — E2e ingest→tools wiki_* roundtrip (P31 Task 2).
//!
//! Single cross-crate test chaining the full integration: temp `.md`
//! fixtures → `vanta_memory::ingest::worker::run` (scripted LLM runner) →
//! MCP tools `wiki_search` / `wiki_read` / `wiki_graph` find and read what
//! the ingest wrote. Both ends share the core `WikiStore` through this
//! test's single `StorageEngine` (dev-dependency direction:
//! vantadb-mcp → vanta-memory, verified cycle-free via cargo tree).

use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tempfile::tempdir;
use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::ingest::{worker, IngestConfig};
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;
use vantadb::wiki::{WikiState, WikiStore};
use vantadb_mcp::{handle_tools_call, McpConfig};

const NS: &str = "default";
const SLUG: &str = "team-wiki";

/// Scripted FIFO runner: one extraction call expected (single small source).
struct ScriptedRunner {
    outputs: Mutex<Vec<Result<String, LlmError>>>,
}

impl LlmRunner for ScriptedRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        let mut queue = self.outputs.lock().expect("poisoned");
        queue.remove(0)
    }
}

fn file_block(path: &str, body: &str) -> String {
    format!(
        "<<<FILE path=\"{path}\">>>\n---\ntype: entity\ntitle: {}\n---\n{body}\n<<<END>>>",
        path.rsplit('/')
            .next()
            .unwrap_or("page")
            .trim_end_matches(".md")
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

#[test]
fn e2e_ingest_then_wiki_tools_roundtrip() {
    // 1. Temp .md fixture — the only local source.
    let src = tempdir().expect("tempdir");
    std::fs::write(
        src.path().join("notes.md"),
        "# Team Notes\nRedis persistence keeps memory safe across restarts.",
    )
    .expect("write source");

    // 2. Shared storage + wiki lifecycle: pending → created for the worker.
    let db_dir = tempdir().expect("tempdir");
    let storage =
        Arc::new(StorageEngine::open(db_dir.path().to_str().expect("path")).expect("engine"));
    let store = WikiStore::new(&storage);
    store.create(NS, SLUG).expect("create");

    // 3. Ingest via vanta-memory worker with a scripted runner emitting two
    //    linked pages ([[Redis]] wikilink → graph edge under test).
    let runner = ScriptedRunner {
        outputs: Mutex::new(vec![Ok([
            file_block(
                "wiki/entities/redis.md",
                "Redis is an in-memory data store.",
            ),
            file_block(
                "wiki/concepts/persistence.md",
                "Persistence snapshots memory to disk. See [[Redis]].",
            ),
        ]
        .join("\n"))]),
    };
    let report = worker::run(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&runner),
        &IngestConfig::default(),
    )
    .expect("ingest run");

    let redis_path = vantadb::wiki::canonical_path("entities", "redis");
    let persistence_path = vantadb::wiki::canonical_path("concepts", "persistence");
    assert!(
        report.commit_report.written.contains(&redis_path)
            && report.commit_report.written.contains(&persistence_path),
        "both pages written: {:?}",
        report.commit_report.written
    );
    assert_eq!(
        store.get(NS, SLUG).expect("get").expect("exists").state,
        WikiState::Ready,
        "build completed ready"
    );

    let executor = Executor::new(&storage);
    let cfg = McpConfig::default();
    let call = |name: &str, args: Value| {
        handle_tools_call(
            &Some(json!({ "name": name, "arguments": args })),
            &executor,
            &storage,
            &cfg,
        )
    };

    // 4. wiki_search finds terms from the ingested files ("memory" appears
    //    in both pages' content).
    let text = msg(call(
        "wiki_search",
        json!({"namespace": NS, "slug": SLUG, "query": "memory"}),
    ));
    assert!(!text.contains("Error"), "search on ready wiki: {text}");
    let v: Value = serde_json::from_str(&text).expect("json");
    let results = v["results"].as_array().expect("results array");
    assert_eq!(results.len(), 2, "both pages match: {text}");
    assert!(
        results
            .iter()
            .any(|r| r["path"] == persistence_path.as_str()),
        "persistence page ranked: {text}"
    );

    // 5. wiki_read returns the merged content of an ingested page.
    let text = msg(call(
        "wiki_read",
        json!({"namespace": NS, "slug": SLUG, "path": redis_path}),
    ));
    let v: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(v["locked"], true, "managed page locked: {text}");
    assert!(
        v["content"]
            .as_str()
            .unwrap()
            .contains("in-memory data store"),
        "merged content readable: {text}"
    );

    // 6. wiki_graph connects the pages via the [[Redis]] wikilink.
    let text = msg(call(
        "wiki_graph",
        json!({"namespace": NS, "slug": SLUG, "root_path": persistence_path}),
    ));
    let v: Value = serde_json::from_str(&text).expect("json");
    assert!(v["visited"].as_u64().unwrap_or(0) >= 2, "connected: {text}");
    let edges = v["edges"].as_array().expect("edges array");
    assert!(
        edges
            .iter()
            .any(|e| e["from"] == persistence_path.as_str() && e["to"] == redis_path.as_str()),
        "[[Redis]] edge present: {text}"
    );
}
