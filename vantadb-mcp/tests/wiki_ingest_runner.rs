// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! IMPL-112-S1 — runner local de ingesta (spec FIND-112 §(d), tests 5-6 + S6 + G1).
//!
//! Contrato G1: default local sin modelo → `sources_skipped` == nº fuentes,
//! `state=ready` consultable por `run_id`; con fake runner → páginas escritas +
//! `sources_processed` > 0 + `wiki_read` legible. S6: `wiki_ingest` inputSchema
//! SIN cambios (sin `provider` en el tool input — config es server-side).
//!
//! Nota de niveles: `start_ingest` dropea el `IngestReport` (P4 best-effort —
//! el cliente pollea `wiki_ingest_status`), así que `sources_*` se aserta a
//! nivel worker (`worker::run` retorna el report) y `state`/consultabilidad a
//! nivel facade (`start_ingest` + `ingest_status`).

use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::tempdir;
use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::ingest::runner_config::{build_ingest_runner, ConcreteRunner, IngestRunnerCfg};
use vanta_memory::ingest::{worker, IngestConfig};
use vantadb::storage::StorageEngine;
use vantadb::wiki::WikiStore;
use vantadb_mcp::{handle_tools_list, ingest_status, start_ingest, McpConfig};

const NS: &str = "default";
const SLUG: &str = "s1-wiki";

/// Fake FIFO runner (patrón `wiki_async_ingest.rs::ScriptedRunner`): respuestas
/// canned con bloques `parse_file_blocks`-válidos.
struct ScriptedRunner {
    outputs: Mutex<Vec<Result<String, LlmError>>>,
}

impl LlmRunner for ScriptedRunner {
    fn run(&self, _params: &LlmRunParams) -> Result<String, LlmError> {
        self.outputs.lock().expect("poisoned").remove(0)
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

/// Poll hasta `Some`; falla tras 10s (sin sleeps fijos — lección MEM-50).
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

fn poll_ready(storage: &Arc<StorageEngine>, run_id: &str) -> Value {
    poll_until(|| {
        ingest_status(storage, run_id)
            .ok()
            .filter(|v| v["state"] == "ready" || v["state"] == "failed")
    })
}

fn test_engine() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let db_dir = tempdir().expect("tempdir");
    let storage =
        Arc::new(StorageEngine::open(db_dir.path().to_str().expect("path")).expect("engine"));
    (db_dir, storage)
}

#[test]
fn ingest_local_no_model_degrades() {
    // G1 degradado: 2 fuentes, runner local sin modelo → skipped == 2.
    let src = tempdir().expect("tempdir");
    std::fs::write(src.path().join("a.md"), "# A\ncontent a.").expect("write");
    std::fs::write(src.path().join("b.md"), "# B\ncontent b.").expect("write");

    let (_db, storage) = test_engine();
    WikiStore::new(&storage).create(NS, SLUG).expect("create");

    // Nivel worker: el report prueba sources_skipped == nº fuentes.
    // El runner sale del constructor S1 (default → degradado explícito).
    let cfg = IngestRunnerCfg::defaults();
    let s1_runner = build_ingest_runner(&cfg).expect("S1 always returns Some");
    assert!(
        matches!(s1_runner, ConcreteRunner::None),
        "default local without model degrades like NoLlm"
    );
    let store = WikiStore::new(&storage);
    let report = worker::run(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&s1_runner),
        &cfg.pipeline_config(),
    )
    .expect("degraded run completes (P4)");
    let mut skipped = report.sources_skipped.clone();
    skipped.sort();
    assert_eq!(skipped, vec!["a.md".to_string(), "b.md".to_string()]);
    assert!(report.sources_processed.is_empty());
    assert!(report.commit_report.written.is_empty());

    // Nivel facade: el default S1 (sin config → None) completa + consultable.
    store.request_ingest(NS, SLUG).expect("re-request");
    let run_id = start_ingest::<ConcreteRunner>(
        storage.clone(),
        NS,
        SLUG,
        src.path().to_path_buf(),
        None,
        IngestConfig::default(),
    )
    .expect("facade start");
    let status = poll_ready(&storage, &run_id);
    assert_eq!(status["state"], "ready", "LLM-free completes ready");
    assert_eq!(status["namespace"], NS);
    assert_eq!(status["slug"], SLUG);
}

#[test]
fn ingest_local_canned_runner_writes_pages() {
    let src = tempdir().expect("tempdir");
    std::fs::write(
        src.path().join("notes.md"),
        "# Notes\nall about local runners",
    )
    .expect("write");

    let (_db, storage) = test_engine();
    WikiStore::new(&storage).create(NS, SLUG).expect("create");

    let canned = file_block(
        "wiki/entities/local-runner.md",
        "Local runners degrade honestly without a model.",
    );
    let fake = ScriptedRunner {
        outputs: Mutex::new(vec![Ok(canned)]),
    };
    let store = WikiStore::new(&storage);
    let report = worker::run(
        &store,
        NS,
        SLUG,
        src.path(),
        Some(&fake),
        &IngestConfig::default(),
    )
    .expect("canned run");
    assert_eq!(report.sources_processed, vec!["notes.md".to_string()]);
    assert!(report.sources_skipped.is_empty());
    assert!(
        report
            .commit_report
            .written
            .iter()
            .any(|p| p == "wiki/entities/local-runner.md"),
        "canned candidate written: {:?}",
        report.commit_report.written
    );

    // Página legible vía wiki_read-equivalente (`get_page` del core store).
    let page = store
        .get_page(NS, SLUG, "wiki/entities/local-runner.md")
        .expect("get_page")
        .expect("page exists");
    assert!(
        page.content.contains("degrade honestly"),
        "page content readable after canned build"
    );

    // El genérico `start_ingest<R>` absorbe cualquier `R` (veredicto fricción
    // S2): el mismo fake por la facade completa ready.
    store.request_ingest(NS, SLUG).expect("re-request");
    let fake2 = ScriptedRunner {
        outputs: Mutex::new(vec![Ok(file_block(
            "wiki/entities/second.md",
            "Second page via facade.",
        ))]),
    };
    let run_id = start_ingest(
        storage.clone(),
        NS,
        SLUG,
        src.path().to_path_buf(),
        Some(fake2),
        IngestConfig::default(),
    )
    .expect("facade start with fake runner");
    let status = poll_ready(&storage, &run_id);
    assert_eq!(status["state"], "ready");
    let page2 = store
        .get_page(NS, SLUG, "wiki/entities/second.md")
        .expect("get_page")
        .expect("facade page exists");
    assert!(page2.content.contains("Second page via facade"));
}

#[test]
fn ingest_tool_input_schema_unchanged() {
    // S6 fijado en test: la config es server-side (operador), nunca per-call
    // del LLM (Hyrum: cada campo del schema es contrato para siempre).
    let list = handle_tools_list(&McpConfig::default()).expect("tools/list");
    let tool = list["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .find(|t| t["name"] == "wiki_ingest")
        .expect("wiki_ingest registered")
        .clone();
    let props = tool["inputSchema"]["properties"]
        .as_object()
        .expect("properties object");
    assert!(
        !props.contains_key("provider"),
        "provider must not leak into the tool input (S6)"
    );
    assert_eq!(
        tool["inputSchema"]["required"],
        json!(["namespace", "slug", "root"]),
        "required set byte-stable"
    );
}
