// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
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

#[test]
fn embed_texts_basic() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();

    // test handle_tools_list contains embed_texts
    let list = handle_tools_list(&cfg).unwrap();
    let tools = list["tools"].as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert!(
        names.contains(&"embed_texts"),
        "tools/list missing embed_texts: {:?}",
        names
    );
    // check inputSchema
    let embed_tool = tools.iter().find(|t| t["name"] == "embed_texts").unwrap();
    assert_eq!(embed_tool["inputSchema"]["required"][0], "texts");

    // call embed_texts
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": ["hola", "hello world"]
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    // should be text_content with JSON
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(val["count"], 2);
    assert_eq!(val["dim"], 384);
    let embeddings = val["embeddings"].as_array().unwrap();
    assert_eq!(embeddings.len(), 2);
    assert_eq!(embeddings[0].as_array().unwrap().len(), 384);
    assert!(val["next_cursor"].is_null());
    assert_eq!(val["truncated"], false);
    // EMB-13 Q5: el flag es siempre visible (nunca silencioso).
    assert!(
        val.get("fallback").is_some() && val["fallback"].is_boolean(),
        "embed_texts debe incluir 'fallback' bool, got: {}",
        text
    );
    if val["fallback"] == true {
        let w = val["warning"].as_str().unwrap_or("");
        assert!(
            !w.is_empty(),
            "fallback:true debe traer 'warning' no vacío, got: {}",
            text
        );
    }
}

#[test]
fn embed_texts_with_model_param() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": ["hola mundo"],
            "model": "multilingual-e5-small"
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: serde_json::Value = serde_json::from_str(text).unwrap();
    assert_eq!(val["count"], 1);
    assert_eq!(val["model"], "multilingual-e5-small");
    // EMB-13: `model` se ecorea pero NO selecciona proveedor (EMB-17).
    assert!(
        val.get("fallback").is_some() && val["fallback"].is_boolean(),
        "embed_texts debe incluir 'fallback' bool, got: {}",
        text
    );
}

#[test]
fn embed_texts_budgeting_truncation() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    // set small budget to force truncation
    let cfg = McpConfig {
        max_embed_tokens: 20, // each text ~7 tokens (len 28/4), => 2 fit (14), third would exceed
        max_embed_batch_size: 2,
        ..Default::default()
    };
    // Create 5 texts, each ~28 chars => ~7 tokens
    let texts: Vec<String> = (0..5)
        .map(|i| format!("text number {} with some words", i))
        .collect();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": texts.clone(),
            "cursor": 0
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: serde_json::Value = serde_json::from_str(text).unwrap();
    // should be truncated (batch size 2)
    assert_eq!(val["truncated"], true);
    assert_eq!(val["count"], 2);
    assert_eq!(val["next_cursor"], 2);
    // EMB-13: budgeting intacto + flag presente en cada página.
    assert!(
        val.get("fallback").is_some() && val["fallback"].is_boolean(),
        "página 1 debe incluir 'fallback' bool, got: {}",
        text
    );
    // second page
    let params2 = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": texts.clone(),
            "cursor": 2
        }
    }));
    let res2 = handle_tools_call(&params2, &executor, &storage, &cfg).unwrap();
    let text2 = res2["content"][0]["text"].as_str().unwrap();
    let val2: serde_json::Value = serde_json::from_str(text2).unwrap();
    assert_eq!(val2["count"], 2);
    assert_eq!(val2["next_cursor"], 4);
    assert!(
        val2.get("fallback").is_some() && val2["fallback"].is_boolean(),
        "página 2 debe incluir 'fallback' bool, got: {}",
        text2
    );
}

#[test]
fn embed_texts_rejects_empty() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": []
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg);
    assert!(res.is_err());
}

#[test]
fn embed_texts_rejects_missing_texts() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "model": "foo"
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg);
    assert!(res.is_err());
}

// ── EMB-13: contrato Q5 + señal ──────────────────────────────────────────

/// Q5: sin modelo el fallback es visible (`fallback:true` + `warning`),
/// nunca silencioso y nunca error duro.
#[test]
fn embed_texts_fallback_flag_visible() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": { "texts": ["hola mundo"] }
    }));
    // Nunca error duro: siempre Ok con flag.
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(
        val.get("fallback").is_some() && val["fallback"].is_boolean(),
        "'fallback' bool siempre presente, got: {}",
        text
    );
    #[cfg(not(any(feature = "embed-local", feature = "remote-inference")))]
    {
        // Compilado sin `vantadb::llm` → dummy avisado directo.
        assert_eq!(val["fallback"], true);
        let w = val["warning"].as_str().unwrap_or("");
        assert!(
            w.to_lowercase().contains("fallback") || w.to_lowercase().contains("unavailable"),
            "'warning' debe explicar el fallback, got: {}",
            text
        );
    }
    #[cfg(any(feature = "embed-local", feature = "remote-inference"))]
    {
        // Con features: consistencia flag↔warning en ambos caminos.
        if val["fallback"] == true {
            let w = val["warning"].as_str().unwrap_or("");
            assert!(!w.is_empty(), "fallback:true exige warning, got: {}", text);
        } else {
            assert!(
                val.get("warning").is_none() || val["warning"].is_null(),
                "fallback:false no debe traer warning, got: {}",
                text
            );
        }
    }
}

fn cosine(a: &[serde_json::Value], b: &[serde_json::Value]) -> f32 {
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        let x = x.as_f64().unwrap_or(0.0) as f32;
        let y = y.as_f64().unwrap_or(0.0) as f32;
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na <= 1e-9 || nb <= 1e-9 {
        return 0.0;
    }
    dot / (na.sqrt() * nb.sqrt())
}

/// Señal semántica real cuando hay modelo: el par sinónimo supera a los
/// impares. Sin modelo (fallback) pasa graceful — la evidencia numérica
/// vive en el cat test MCP del Step 3, no en este unit test.
#[test]
fn embed_texts_real_signal_si_hay_modelo() {
    let (_dir, storage) = setup();
    let executor = vantadb::executor::Executor::new(&storage);
    let cfg = McpConfig::default();
    let params = Some(json!({
        "name": "embed_texts",
        "arguments": {
            "texts": [
                "el gato duerme en el sofá",
                "un felino descansa en el sillón",
                "la física cuántica estudia partículas subatómicas"
            ]
        }
    }));
    let res = handle_tools_call(&params, &executor, &storage, &cfg).unwrap();
    let text = res["content"][0]["text"].as_str().unwrap();
    let val: serde_json::Value = serde_json::from_str(text).unwrap();
    if val
        .get("fallback")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
    {
        eprintln!("EMB-13: sin modelo (fallback:true) — señal verificada en cat test Step 3");
        return;
    }
    let embs = val["embeddings"].as_array().unwrap();
    assert_eq!(embs.len(), 3);
    let a0 = embs[0].as_array().unwrap();
    let a1 = embs[1].as_array().unwrap();
    let a2 = embs[2].as_array().unwrap();
    let sim_par = cosine(a0, a1);
    let sim_imp0 = cosine(a0, a2);
    let sim_imp1 = cosine(a1, a2);
    eprintln!(
        "EMB-13 señal: par={:.4} impar0={:.4} impar1={:.4}",
        sim_par, sim_imp0, sim_imp1
    );
    assert!(
        sim_par > sim_imp0 && sim_par > sim_imp1,
        "el par sinónimo debe superar a los impares: par={} imp0={} imp1={}",
        sim_par,
        sim_imp0,
        sim_imp1
    );
}
