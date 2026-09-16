// EMB-17 RED: `model` switch por nombre (Q2) — desconocido→error con lista,
// conocido→ese modelo, sin-archivos→error con download hint.
// Patrón `test_embed_texts.rs`: graceful por cfg (default sin engine vs features).
// ponytail: blanket allow - unwraps con invariantes documentadas; documentado per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
use serde_json::json;
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

fn setup() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
    (dir, Arc::new(storage))
}

fn call_embed_texts(
    storage: &Arc<StorageEngine>,
    args: serde_json::Value,
) -> Result<serde_json::Value, serde_json::Value> {
    let executor = vantadb::executor::Executor::new(storage);
    let cfg = McpConfig::default();
    let params = Some(json!({ "name": "embed_texts", "arguments": args }));
    match handle_tools_call(&params, &executor, storage, &cfg) {
        Ok(res) => {
            let text = res["content"][0]["text"].as_str().unwrap().to_string();
            Ok(serde_json::from_str(&text).unwrap())
        }
        Err(e) => Err(e),
    }
}

/// EMB-17.2: id desconocido → `invalid_params` con lista válida (nunca silencio).
#[test]
fn emb17_unknown_model_rejected_with_valid_list() {
    let (_dir, storage) = setup();
    let err = call_embed_texts(
        &storage,
        json!({ "texts": ["hola"], "model": "no-existe-xyz" }),
    )
    .expect_err("unknown model debe ser Err, no Ok-ecoreo silencioso");
    let msg = err.to_string();
    assert!(
        msg.to_lowercase().contains("unknown embedding model"),
        "debe decir 'unknown embedding model', got: {}",
        msg
    );
    for id in ["multilingual-e5-small", "all-MiniLM-L6-v2"] {
        assert!(
            msg.contains(id),
            "la lista válida debe incluir {id}, got: {}",
            msg
        );
    }
}

/// EMB-17.1: id conocido → ese modelo (ecoreo canónico + dim del manifest + flag).
#[test]
fn emb17_known_model_echo_and_dim() {
    let (_dir, storage) = setup();
    let val = call_embed_texts(
        &storage,
        json!({ "texts": ["hola mundo"], "model": "multilingual-e5-small" }),
    )
    .expect("known model no debe ser Err");
    assert_eq!(val["model"], "multilingual-e5-small");
    assert_eq!(val["dim"], 384);
    assert_eq!(val["dimensions"], 384);
    assert!(
        val.get("fallback").is_some() && val["fallback"].is_boolean(),
        "flag 'fallback' siempre presente, got: {}",
        val
    );
    #[cfg(not(any(feature = "embed-local", feature = "remote-inference")))]
    {
        assert_eq!(val["fallback"], true);
    }
    #[cfg(any(feature = "embed-local", feature = "remote-inference"))]
    {
        if val["fallback"] == true {
            assert!(!val["warning"].as_str().unwrap_or("").is_empty());
        }
    }
}

/// EMB-17.3: id conocido sin archivos en disco → error con download hint (nunca fallback con dim errónea).
/// `bge-base-en-v1.5` (768d) no está entre los 3 en disco (plan §verificación real).
#[test]
fn emb17_missing_files_model_errors_with_download_hint() {
    let (_dir, storage) = setup();
    // Sin features no hay engine: el path conocido degrada a fallback avisado (Q5, precedente EMB-13).
    // Con features el contrato exige error con hint (archivos ausentes ≠ "sin modelo").
    #[cfg(not(any(feature = "embed-local", feature = "remote-inference")))]
    {
        let val = call_embed_texts(
            &storage,
            json!({ "texts": ["hola"], "model": "bge-base-en-v1.5" }),
        )
        .expect("sin features: conocido degrada a fallback, no Err");
        assert_eq!(val["fallback"], true);
    }
    #[cfg(any(feature = "embed-local", feature = "remote-inference"))]
    {
        let err = call_embed_texts(
            &storage,
            json!({ "texts": ["hola"], "model": "bge-base-en-v1.5" }),
        )
        .expect_err("con features y sin archivos debe ser Err con download hint");
        let msg = err.to_string();
        assert!(
            msg.contains("bge-base-en-v1.5"),
            "debe nombrar el modelo pedido, got: {}",
            msg
        );
        assert!(
            msg.to_lowercase().contains("download"),
            "debe sugerir download.py, got: {}",
            msg
        );
    }
}
