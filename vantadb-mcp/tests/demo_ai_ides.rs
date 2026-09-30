//! ICP-01 demo — cross-session project memory over the MCP surface.
//!
//! Session 1 stores a note; session 2 recalls it with a synonym query that
//! shares ZERO literal tokens with the stored text (a keyword-only path could
//! never hit it). Embeddings come from a deterministic in-process fake Ollama
//! (`POST /api/embed`) declared as a **test double**: it proves the pipeline
//! (embed-on-put EMB-14 → vector store → embed-on-query EMB-15 → similarity
//! ranking), not model quality. Offline, no tokens, no model files.
//!
//! The synonym pair is EMB-15's (`test_query_embed.rs`): el gato duerme en el
//! sofá ↔ felino descansando.
// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use axum::{routing::post, Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

/// EMB-15 pair: zero literal tokens in common with `SYNONYM_QUERY`.
const STORED: &str = "el gato duerme en el sofá";
const SYNONYM_QUERY: &str = "felino descansando";
const DISTRACTOR: &str = "la física cuántica estudia partículas subatómicas";

/// 384-dim bag-of-normalized-tokens hash. The synonym map (felino→gato,
/// descansando→duerme) is the declared double's "semantic" knowledge; the
/// architecture (hash buckets) is irrelevant to the test — determinism is.
fn embed_text(text: &str) -> Vec<f32> {
    const DIM: usize = 384;
    let mut v = vec![0.0f32; DIM];
    for token in raw_tokens(text) {
        let norm: &str = match token.as_str() {
            "felino" => "gato",
            "descansando" | "descansa" => "duerme",
            other => other,
        };
        // FNV-1a — deterministic across processes (DefaultHasher isn't).
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for b in norm.as_bytes() {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        v[(hash % DIM as u64) as usize] += 1.0;
    }
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for x in &mut v {
            *x /= norm;
        }
    }
    v
}

/// Literal (unnormalized) lowercase alphanumeric tokens.
fn raw_tokens(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

/// Fake Ollama handler: `{model,input}` → `{embeddings:[[f32]]}` (the exact
/// shape `OllamaProvider` speaks — `src/llm.rs`).
async fn embed_handler(Json(body): Json<Value>) -> Json<Value> {
    let input = body["input"].as_str().unwrap_or_default();
    Json(json!({ "embeddings": [embed_text(input)] }))
}

/// Serve the fake on 127.0.0.1:<ephemeral> from its own thread + runtime.
fn start_fake_ollama() -> String {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("fake ollama runtime");
        rt.block_on(async move {
            let app = Router::new().route("/api/embed", post(embed_handler));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
                .await
                .expect("bind fake ollama");
            let addr = listener.local_addr().expect("fake ollama addr");
            tx.send(format!("http://127.0.0.1:{}", addr.port()))
                .expect("share fake ollama addr");
            axum::serve(listener, app).await.expect("serve fake ollama");
        });
    });
    rx.recv().expect("fake ollama ready")
}

fn setup() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
    vantadb::Embedded::from_engine(storage.clone())
        .ensure_indexes_current()
        .expect("startup index ensure");
    (dir, storage)
}

fn call(storage: &Arc<StorageEngine>, name: &str, arguments: Value) -> Value {
    let executor = vantadb::executor::Executor::new(storage);
    let params = Some(json!({"name": name, "arguments": arguments}));
    handle_tools_call(&params, &executor, storage, &McpConfig::default()).unwrap()
}

fn text_of(res: &Value) -> String {
    res["content"][0]["text"].as_str().unwrap().to_string()
}

/// ICP-01 acceptance: session 1 stores (auto-embed on put) → session 2 recalls
/// by a synonym query with zero shared tokens. Fails if the recall does not
/// arrive or if the keyword-only path was the one that answered.
#[test]
fn session_one_stores_session_two_recalls_by_synonym() {
    let base = start_fake_ollama();
    // Dedicated test binary (own process): process-wide env is safe here.
    std::env::set_var("VANTADB_EMBEDDING_PROVIDER", "ollama");
    std::env::set_var("VANTADB_LLM_URL", &base);

    let (_dir, storage) = setup();

    // The point of the demo: a keyword-only path could never retrieve.
    let stored = raw_tokens(STORED);
    assert!(
        raw_tokens(SYNONYM_QUERY)
            .iter()
            .all(|t| !stored.contains(t)),
        "demo pair must share zero literal tokens (no-verbatim synonym)"
    );

    // ── Session 1: store the project note (auto-embed on put, EMB-14). ──
    let res = call(
        &storage,
        "memory_put",
        json!({"namespace": "demo-project", "key": "note-1", "payload": STORED}),
    );
    assert!(res["isError"].is_null(), "put must succeed: {res}");
    let got: Value = serde_json::from_str(&text_of(&call(
        &storage,
        "memory_get",
        json!({"namespace": "demo-project", "key": "note-1"}),
    )))
    .unwrap();
    assert!(
        got["vector"].as_array().is_some_and(|v| !v.is_empty()),
        "embed-on-put must persist a vector (provider = fake): {got}"
    );

    let res = call(
        &storage,
        "memory_put",
        json!({"namespace": "demo-project", "key": "note-2", "payload": DISTRACTOR}),
    );
    assert!(
        res["isError"].is_null(),
        "distractor put must succeed: {res}"
    );

    // ── Session 2: recall by synonym (no shared words with the note). ──
    let res = call(
        &storage,
        "search_memory",
        json!({"namespace": "demo-project", "text_query": SYNONYM_QUERY, "top_k": 5}),
    );
    assert!(res["isError"].is_null(), "search must not error: {res}");
    let hits: Value = serde_json::from_str(&text_of(&res)).unwrap();
    let keys: Vec<&str> = hits
        .as_array()
        .expect("hits array")
        .iter()
        .filter_map(|h| h["record"]["key"].as_str())
        .collect();
    assert_eq!(
        keys.first().copied(),
        Some("note-1"),
        "synonym recall must rank the stored note first, got: {keys:?}"
    );

    // Ranking works both ways: a query on the distractor's topic ranks it.
    let res = call(
        &storage,
        "search_memory",
        json!({"namespace": "demo-project", "text_query": "cuántica subatómicas", "top_k": 5}),
    );
    let hits: Value = serde_json::from_str(&text_of(&res)).unwrap();
    let keys: Vec<&str> = hits
        .as_array()
        .expect("hits array")
        .iter()
        .filter_map(|h| h["record"]["key"].as_str())
        .collect();
    assert_eq!(
        keys.first().copied(),
        Some("note-2"),
        "distractor-topic query must rank the distractor first, got: {keys:?}"
    );
}
