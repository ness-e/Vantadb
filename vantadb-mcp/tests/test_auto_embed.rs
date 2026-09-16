// EMB-14 — auto-embed en `memory_put`/`put_batch` (cierra FIND-99 con EMB-13).
// Contrato: put sin vector → auto-embed vía UN `embed_batch` (get lo muestra);
// vector provisto se respeta byte-exacto (no re-embed); fallo proveedor →
// guarda sin vector + `fallback:true` + `warning` (nunca error duro, nunca
// dummies persistidos).
// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

fn setup() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let storage = StorageEngine::open(dir.path().to_str().unwrap()).unwrap();
    (dir, Arc::new(storage))
}

fn put(storage: &Arc<StorageEngine>, ns: &str, key: &str, payload: &str) -> Value {
    let executor = vantadb::executor::Executor::new(storage);
    let params = Some(json!({
        "name": "memory_put",
        "arguments": {"namespace": ns, "key": key, "payload": payload}
    }));
    handle_tools_call(&params, &executor, storage, &McpConfig::default()).unwrap()
}

fn get_vector(storage: &Arc<StorageEngine>, ns: &str, key: &str) -> Value {
    let executor = vantadb::executor::Executor::new(storage);
    let params = Some(json!({
        "name": "memory_get",
        "arguments": {"namespace": ns, "key": key}
    }));
    let res = handle_tools_call(&params, &executor, storage, &McpConfig::default()).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: Value = serde_json::from_str(text).unwrap();
    val["vector"].clone()
}

/// Put sin vector nunca es error duro y siempre trae flag `fallback` visible
/// (Q5 heredado de EMB-13). El record se guarda en ambos caminos.
#[test]
fn put_sin_vector_lleva_flag_y_nunca_error_duro() {
    let (_dir, storage) = setup();
    let res = put(&storage, "emb14_ns", "k1", "el gato duerme en el sofá");
    assert!(
        res["isError"].is_null(),
        "put sin vector nunca es error duro, got: {res}"
    );
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: Value = serde_json::from_str(text).unwrap();
    // Flat+flags: el record sigue plano (paridad AUD-045/structured).
    assert_eq!(val["key"], "k1");
    assert!(
        val.get("fallback").is_some() && val["fallback"].is_boolean(),
        "put debe incluir 'fallback' bool, got: {text}"
    );
    if val["fallback"] == true {
        let w = val["warning"].as_str().unwrap_or("");
        assert!(!w.is_empty(), "fallback:true exige 'warning', got: {text}");
    }
    // El record existe vía get en ambos caminos.
    let executor = vantadb::executor::Executor::new(&storage);
    let get = Some(json!({
        "name": "memory_get",
        "arguments": {"namespace": "emb14_ns", "key": "k1"}
    }));
    let got = handle_tools_call(&get, &executor, &storage, &McpConfig::default()).unwrap();
    assert!(got["isError"].is_null());
}

/// Vector provisto se respeta byte-exacto en TODA config: con vector presente
/// no hay re-embed (ni siquiera con modelo real), `fallback:false` sin warning.
#[test]
fn put_vector_provisto_se_respeta_byte_exact() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    // Base dim 3 con vector explícito.
    let seed = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "emb14_keep", "key": "seed", "payload": "s",
            "vector": [1.0, 0.0, 0.0]
        }
    }));
    handle_tools_call(&seed, &executor, &storage, &cfg).unwrap();
    // Put con vector provisto distinto pero misma dim.
    let mine = vec![0.25, 0.5, 0.75];
    let params = Some(json!({
        "name": "memory_put",
        "arguments": {
            "namespace": "emb14_keep", "key": "mine", "payload": "un texto cualquiera",
            "vector": mine
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: Value = serde_json::from_str(text).unwrap();
    assert_eq!(
        val["fallback"], false,
        "vector provisto → sin fallback, got: {text}"
    );
    assert!(
        val.get("warning").is_none() || val["warning"].is_null(),
        "fallback:false no trae warning, got: {text}"
    );
    // Get devuelve EXACTAMENTE lo provisto: prueba de no-re-embed.
    let stored = get_vector(&storage, "emb14_keep", "mine");
    let arr = stored.as_array().expect("vector debe persistir");
    let got: Vec<f32> = arr.iter().map(|v| v.as_f64().unwrap() as f32).collect();
    assert_eq!(
        got,
        vec![0.25f32, 0.5, 0.75],
        "el vector provisto no se re-embebe"
    );
}

/// Batch sin vectores: UN `embed_batch` implícito; envelope `{records, fallback,
/// warning?}` coherente. Sin modelo → todo null + aviso; con modelo → todo lleno.
#[test]
fn put_batch_sin_vectores_flag_coherente() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let batch = Some(json!({
        "name": "memory_put_batch",
        "arguments": {"inputs": [
            {"namespace": "emb14_batch", "key": "a", "payload": "el gato duerme"},
            {"namespace": "emb14_batch", "key": "b", "payload": "un felino descansa"},
            {"namespace": "emb14_batch", "key": "c", "payload": "física cuántica"}
        ]}
    }));
    let res = handle_tools_call(&batch, &executor, &storage, &McpConfig::default()).unwrap();
    assert!(
        res["isError"].is_null(),
        "batch sin vectores no es error: {res}"
    );
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: Value = serde_json::from_str(text).unwrap();
    let records = val["records"].as_array().expect("envelope con 'records'");
    assert_eq!(records.len(), 3);
    assert!(
        val.get("fallback").is_some() && val["fallback"].is_boolean(),
        "batch debe incluir 'fallback' bool, got: {text}"
    );
    #[cfg(not(any(feature = "embed-local", feature = "remote-inference")))]
    {
        // Sin features llm: todo guardado sin vector + aviso (mirror EMB-13).
        assert_eq!(val["fallback"], true);
        let w = val["warning"].as_str().unwrap_or("");
        assert!(
            w.to_lowercase().contains("fallback") || w.to_lowercase().contains("unavailable"),
            "'warning' debe explicar el fallback, got: {text}"
        );
        for r in records {
            assert!(
                r["vector"].is_null(),
                "sin proveedor el vector queda null: {r}"
            );
        }
    }
    #[cfg(any(feature = "embed-local", feature = "remote-inference"))]
    {
        // Con features: coherencia flag↔vectores en ambos caminos.
        if val["fallback"] == true {
            let w = val["warning"].as_str().unwrap_or("");
            assert!(!w.is_empty(), "fallback:true exige warning, got: {text}");
            for r in records {
                assert!(r["vector"].is_null(), "fallback → vector null: {r}");
            }
        } else {
            let dims: Vec<usize> = records
                .iter()
                .map(|r| r["vector"].as_array().expect("vector lleno").len())
                .collect();
            assert!(
                dims.windows(2).all(|w| w[0] == w[1]) && dims[0] > 0,
                "con modelo todos los vectores llenos y misma dim: {dims:?}"
            );
        }
    }
}

/// Aviso explícito sin modelo: el flag y el warning nombran la causa.
/// Sin features es determinista; con features verifica coherencia.
#[test]
fn fallback_aviso_explicito_sin_modelo() {
    let (_dir, storage) = setup();
    let res = put(&storage, "emb14_warn", "k", "texto para avisar");
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: Value = serde_json::from_str(text).unwrap();
    #[cfg(not(any(feature = "embed-local", feature = "remote-inference")))]
    {
        assert_eq!(val["fallback"], true);
        let w = val["warning"].as_str().unwrap_or("");
        assert!(
            w.to_lowercase().contains("fallback") || w.to_lowercase().contains("unavailable"),
            "'warning' debe explicar el fallback, got: {text}"
        );
        assert!(
            get_vector(&storage, "emb14_warn", "k").is_null(),
            "fallback guarda sin vector"
        );
    }
    #[cfg(any(feature = "embed-local", feature = "remote-inference"))]
    {
        if val["fallback"] == true {
            assert!(!val["warning"].as_str().unwrap_or("").is_empty());
            assert!(get_vector(&storage, "emb14_warn", "k").is_null());
        } else {
            assert!(get_vector(&storage, "emb14_warn", "k").is_array());
        }
    }
}

fn cosine(a: &[Value], b: &[Value]) -> f32 {
    let (mut dot, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b.iter()) {
        let (x, y) = (
            x.as_f64().unwrap_or(0.0) as f32,
            y.as_f64().unwrap_or(0.0) as f32,
        );
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na <= 1e-9 || nb <= 1e-9 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// Señal semántica real cuando hay modelo: put sin vector deja vectores con
/// señal (par sinónimo > impares). Sin modelo pasa graceful — la evidencia
/// numérica vive en el cat test del Step 3, no aquí.
#[test]
fn auto_embed_senal_real_si_hay_modelo() {
    let (_dir, storage) = setup();
    for (k, p) in [
        ("s0", "el gato duerme en el sofá"),
        ("s1", "un felino descansa en el sillón"),
        ("s2", "la física cuántica estudia partículas subatómicas"),
    ] {
        let res = put(&storage, "emb14_sig", k, p);
        assert!(res["isError"].is_null());
    }
    let v: Vec<Value> = ["s0", "s1", "s2"]
        .iter()
        .map(|k| get_vector(&storage, "emb14_sig", k))
        .collect();
    if v.iter().any(|x| !x.is_array()) {
        eprintln!("EMB-14: sin modelo (vectores null) — señal verificada en cat test Step 3");
        return;
    }
    let (a0, a1, a2) = (
        v[0].as_array().unwrap(),
        v[1].as_array().unwrap(),
        v[2].as_array().unwrap(),
    );
    let (par, imp0, imp1) = (cosine(a0, a1), cosine(a0, a2), cosine(a1, a2));
    eprintln!("EMB-14 señal: par={par:.4} impar0={imp0:.4} impar1={imp1:.4}");
    assert!(
        par > imp0 && par > imp1,
        "el par sinónimo debe superar a los impares: par={par} imp0={imp0} imp1={imp1}"
    );
}
