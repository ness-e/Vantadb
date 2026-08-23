//! MEM-52 — Fachada productiva de ingest wiki (P33 Task 3).
//!
//! Contrato D19: el disparo async (mismo `start_ingest` que llama el tool
//! `wiki_ingest`) retorna run_id inmediato → estado consultable por run_id
//! hasta ready → páginas disponibles vía tools wiki_* (handle_tools_call).
//! El runner scripted se inyecta en la fachada porque JSON-RPC no transporta
//! runners; el wrapper del tool (runner=None) se cubre con el test de
//! degradado + registro.

use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::tempdir;
use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::ingest::IngestConfig;
use vantadb::storage::StorageEngine;
use vantadb::wiki::WikiStore;
use vantadb_mcp::{
    handle_tools_call, handle_tools_list, ingest_status, start_ingest, McpConfig, NoLlm,
};

const NS: &str = "default";
const SLUG: &str = "facade-wiki";

/// Scripted FIFO runner (patrón MEM-44): una llamada de extracción esperada.
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

/// Message of a tool result.
fn msg(res: Result<Value, Value>) -> String {
    match res {
        Ok(v) => v["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        Err(v) => v["message"].as_str().unwrap_or_default().to_string(),
    }
}

/// Poll until `f` returns Some; fails the test after 10s (poll-loop, no fixed
/// sleeps — lección MEM-50).
fn poll_until<T>(mut f: impl FnMut() -> Option<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(v) = f() {
            return v;
        }
        assert!(Instant::now() < deadline, "timed out polling ingest status");
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn tools_list_registers_ingest_tools() {
    let list = handle_tools_list().expect("tools/list");
    let names: Vec<&str> = list["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert!(names.contains(&"wiki_ingest"), "missing wiki_ingest");
    assert!(
        names.contains(&"wiki_ingest_status"),
        "missing wiki_ingest_status"
    );
}

#[test]
fn d19_async_ingest_run_id_then_ready_then_pages_readable() {
    // 1. Fuente local + storage compartido + wiki creado.
    let src = tempdir().expect("tempdir");
    std::fs::write(
        src.path().join("notes.md"),
        "# Facade Notes\nFacade ingestion keeps memory durable across restarts.",
    )
    .expect("write source");

    let db_dir = tempdir().expect("tempdir");
    let storage =
        Arc::new(StorageEngine::open(db_dir.path().to_str().expect("path")).expect("engine"));
    WikiStore::new(&storage).create(NS, SLUG).expect("create");

    // 2. Disparo async por la fachada del tool con runner scripted: retorna
    //    run_id INMEDIATO mientras el build corre en su hilo.
    let runner = ScriptedRunner {
        outputs: Mutex::new(vec![Ok(file_block(
            "wiki/entities/durability.md",
            "Durability snapshots memory to disk.",
        ))]),
    };
    let run_id = start_ingest(
        storage.clone(),
        NS,
        SLUG,
        src.path().to_path_buf(),
        Some(runner),
        IngestConfig::default(),
    )
    .expect("start_ingest returns run_id immediately");
    assert!(!run_id.is_empty(), "run_id non-empty");

    // 3. Estado consultable por run_id (MEM-31) hasta ready.
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let call_tool = |name: &str, args: Value| {
        handle_tools_call(
            &Some(json!({ "name": name, "arguments": args })),
            &executor,
            &storage,
            &cfg,
        )
    };
    let final_status = poll_until(|| {
        let text = msg(call_tool("wiki_ingest_status", json!({ "run_id": run_id })));
        let v: Value = serde_json::from_str(&text).ok()?;
        (v["state"] == "ready" || v["state"] == "failed").then_some(v)
    });
    assert_eq!(
        final_status["state"], "ready",
        "build ready: {final_status}"
    );
    assert_eq!(final_status["namespace"], NS);
    assert_eq!(final_status["slug"], SLUG);

    // 4. Páginas disponibles para wiki_read (mismo storage que los tools).
    let page_path = vantadb::wiki::canonical_path("entities", "durability");
    let text = msg(call_tool(
        "wiki_read",
        json!({ "namespace": NS, "slug": SLUG, "path": page_path }),
    ));
    let v: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(v["locked"], true, "managed page locked: {text}");
    assert!(
        v["content"].as_str().unwrap().contains("snapshots memory"),
        "page content readable after async build: {text}"
    );
}

#[test]
fn degraded_start_without_llm_completes_ready_with_skips() {
    let src = tempdir().expect("tempdir");
    std::fs::write(src.path().join("a.md"), "# A\ncontent a.").expect("write source");

    let db_dir = tempdir().expect("tempdir");
    let storage =
        Arc::new(StorageEngine::open(db_dir.path().to_str().expect("path")).expect("engine"));
    WikiStore::new(&storage).create(NS, SLUG).expect("create");

    let run_id = start_ingest::<NoLlm>(
        storage.clone(),
        NS,
        SLUG,
        src.path().to_path_buf(),
        None,
        IngestConfig::default(),
    )
    .expect("degraded start");

    let final_status = poll_until(|| {
        let text = msg(ingest_status(&storage, &run_id)
            .map(text_content_like)
            .map_err(|e| json!({"message": e})));
        serde_json::from_str::<Value>(&text)
            .ok()
            .filter(|v| v["state"] == "ready" || v["state"] == "failed")
    });
    assert_eq!(final_status["state"], "ready", "LLM-free completes ready");
}

/// msg() shape for a plain JSON value (status returns a Value, not tool envelope).
fn text_content_like(v: Value) -> Value {
    serde_json::json!({ "content": [{ "text": v.to_string() }] })
}
