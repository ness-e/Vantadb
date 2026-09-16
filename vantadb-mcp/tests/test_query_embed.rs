// EMB-15 — embed de query con el MISMO proveedor + prueba sinónimos.
// Contrato: `search_memory` texto-solo embebe la query con `embed_query` del
// proveedor activo; `memory_recall` deja de pasar hook `None`; sin proveedor →
// keyword honesto + aviso (nunca error duro, nunca dummies); ranking RRF intacto.
// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
use serde_json::{json, Value};
use std::sync::Arc;
use tempfile::tempdir;
use vantadb::storage::StorageEngine;
use vantadb_mcp::*;

fn setup() -> (tempfile::TempDir, Arc<StorageEngine>) {
    let dir = tempdir().unwrap();
    let storage = Arc::new(StorageEngine::open(dir.path().to_str().unwrap()).unwrap());
    // MCP-01: StorageEngine::open solo deja el registro text_index ausente;
    // el server real corre ensure_indexes_current() al arrancar.
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

fn get_vector(storage: &Arc<StorageEngine>, ns: &str, key: &str) -> Value {
    let res = call(storage, "memory_get", json!({"namespace": ns, "key": key}));
    let val: Value = serde_json::from_str(&text_of(&res)).unwrap();
    val["vector"].clone()
}

/// ¿Hay modelo REAL disponible? Espejo barato de `is_local_model_available` +
/// sonda remota. Sin modelo los tests de señal hacen skip graceful (patrón
/// EMB-13/14); con modelo deben PROBAR semántica (RED real pre-GREEN).
#[cfg(any(feature = "embed-local", feature = "remote-inference"))]
fn real_model_available() -> bool {
    let provider =
        std::env::var("VANTADB_EMBEDDING_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
    if provider == "ollama" || provider == "openai" {
        // Remoto: solo real si el servidor responde (sin servidor → fallback avisado).
        return vantadb::llm::get_embedding_provider()
            .embed_query("hola")
            .is_ok();
    }
    let dir = std::env::var("VANTADB_LOCAL_MODEL")
        .unwrap_or_else(|_| "embeddings/models/multilingual-e5-small/onnx".to_string());
    let base = std::path::Path::new(&dir);
    let has_onnx = base.join("model.onnx").exists()
        || base.join("onnx/model.onnx").exists()
        || std::path::Path::new("embeddings/models/multilingual-e5-small/onnx/model.onnx").exists();
    let has_tok = base.join("tokenizer.json").exists()
        || base.join("../tokenizer.json").exists()
        || std::path::Path::new("embeddings/models/multilingual-e5-small/tokenizer.json").exists();
    has_onnx && has_tok
}

#[cfg(not(any(feature = "embed-local", feature = "remote-inference")))]
fn real_model_available() -> bool {
    false
}

const D0: &str = "el gato duerme en el sofá";
const D1: &str = "un felino reposa en el sillón";
const D2: &str = "la física cuántica estudia partículas subatómicas";
/// Cero palabras comunes con D0 ({felino, descansando} ∩ {gato, duerme, sofá} = ∅):
/// keyword-only JAMÁS recupera D0; solo el vector con señal lo hace.
const QUERY_SINONIMO: &str = "felino descansando";

fn seed_search_corpus(storage: &Arc<StorageEngine>) {
    for (k, p) in [("d0", D0), ("d1", D1), ("d2", D2)] {
        let res = call(
            storage,
            "memory_put",
            json!({"namespace": "emb15_search", "key": k, "payload": p}),
        );
        assert!(res["isError"].is_null(), "seed put no es error: {res}");
    }
}

/// Query texto-solo con CERO palabras comunes con D0 debe recuperarlo cuando
/// hay modelo (paridad de espacios: mismo proveedor que EMB-14 usó al guardar).
/// Sin modelo → skip graceful (la evidencia numérica vive en el cat test).
#[test]
fn search_texto_solo_sinonimo_recupera_par() {
    let (_dir, storage) = setup();
    seed_search_corpus(&storage);
    if !real_model_available() || get_vector(&storage, "emb15_search", "d0").is_null() {
        eprintln!(
            "EMB-15: sin modelo real (vectores null) — sinónimos verificados en cat test Step 3"
        );
        return;
    }
    let res = call(
        &storage,
        "search_memory",
        json!({"namespace": "emb15_search", "text_query": QUERY_SINONIMO, "top_k": 5}),
    );
    assert!(
        res["isError"].is_null(),
        "search texto-solo no es error: {res}"
    );
    let hits: Value = serde_json::from_str(&text_of(&res)).unwrap();
    let keys: Vec<&str> = hits
        .as_array()
        .expect("hits array")
        .iter()
        .filter_map(|h| h["record"]["key"].as_str())
        .collect();
    assert!(
        keys.contains(&"d0"),
        "la query sin palabras comunes debe recuperar D0 por semántica, got keys: {keys:?}"
    );
    eprintln!(
        "EMB-15 señal search: keys={keys:?} (query={QUERY_SINONIMO:?} sin palabras comunes con D0)"
    );
}

/// `query_vector` provisto se respeta: con vector válido + texto la búsqueda
/// vectorial funciona SIN proveedor (no hay re-embed que lo pise). Determinista
/// en toda config — guard rail pre y post GREEN.
#[test]
fn search_vector_provisto_no_se_reembebe() {
    let (_dir, storage) = setup();
    let seed = call(
        &storage,
        "memory_put",
        json!({"namespace": "emb15_keep", "key": "s", "payload": "x", "vector": [1.0, 0.0, 0.0]}),
    );
    assert!(seed["isError"].is_null());
    let res = call(
        &storage,
        "search_memory",
        json!({"namespace": "emb15_keep", "query_vector": [1.0, 0.0, 0.0], "text_query": "cualquier texto", "top_k": 5}),
    );
    assert!(
        res["isError"].is_null(),
        "vector provisto + texto no es error: {res}"
    );
    let hits: Value = serde_json::from_str(&text_of(&res)).unwrap();
    let keys: Vec<&str> = hits
        .as_array()
        .expect("hits array")
        .iter()
        .filter_map(|h| h["record"]["key"].as_str())
        .collect();
    assert!(
        keys.contains(&"s"),
        "el vector provisto debe seguir mandando, got: {keys:?}"
    );
}

/// Sin proveedor el recall sigue honestamente en keyword: sin error, modo
/// avisado. Determinista en toda config (default y ollama-sin-servidor).
#[test]
fn recall_sin_proveedor_modo_keyword_sin_error() {
    let (_dir, storage) = setup();
    if real_model_available() {
        eprintln!("EMB-15: con modelo real este guard lo cubre recall_sinonimo_con_modelo; skip");
        return;
    }
    let res = call(
        &storage,
        "memory_recall",
        json!({"query": "felino descansando", "top_k": 5}),
    );
    assert!(
        res["isError"].is_null(),
        "recall sin modelo no es error duro: {res}"
    );
    let val: Value = serde_json::from_str(&text_of(&res)).unwrap();
    assert_eq!(
        val["effective_mode"], "keyword",
        "sin proveedor el modo avisado es keyword, got: {val}"
    );
}

/// Siembra L1 con vectores vía hook `embed_query` en-test (semántica query,
/// igual que el hook prod). Solo corre con features; sin ellas → skip.
#[cfg(any(feature = "embed-local", feature = "remote-inference"))]
fn seed_l1(storage: &Arc<StorageEngine>) {
    use vanta_memory::core::abstractions::{
        DedupAction, DedupDecision, ExtractedMemory, MemoryType,
    };
    use vanta_memory::core::record::l1_writer::write_memory;
    let db = vantadb::Embedded::from_engine(storage.clone());
    let hook: vanta_memory::core::record::EmbedFn =
        Arc::new(|t: &str| vantadb::llm::get_embedding_provider().embed_query(t).ok());
    for (idx, content) in [D0, D1, D2].iter().enumerate() {
        let mem = ExtractedMemory {
            content: content.to_string(),
            memory_type: MemoryType::Episodic,
            priority: 80,
            source_message_ids: vec![],
            scene_name: "test".to_string(),
            metadata: Value::Null,
        };
        let dec = DedupDecision {
            record_id: String::new(),
            action: DedupAction::Store,
            target_ids: vec![],
            merged_content: None,
            merged_type: None,
            merged_priority: None,
            merged_timestamps: None,
        };
        write_memory(
            &db,
            "mcp",
            "s1",
            &mem,
            &dec,
            1_700_000_000_000,
            idx,
            Some(&hook),
        )
        .expect("seed L1");
    }
}

/// Recall con hook real: la query sin palabras comunes recupera D0 y el modo
/// es honesto (hybrid/embedding, NO keyword). Pre-GREEN falla (hook `None` →
/// keyword-only, D0 invisible). Sin modelo → skip graceful.
#[test]
fn recall_sinonimo_con_modelo() {
    let (_dir, _storage) = setup();
    if !real_model_available() {
        eprintln!("EMB-15: sin modelo real — recall-sinónimo verificado en cat test Step 3");
        return;
    }
    #[cfg(any(feature = "embed-local", feature = "remote-inference"))]
    {
        seed_l1(&_storage);
        let res = call(
            &_storage,
            "memory_recall",
            json!({"query": QUERY_SINONIMO, "top_k": 5}),
        );
        assert!(res["isError"].is_null(), "recall no es error: {res}");
        let val: Value = serde_json::from_str(&text_of(&res)).unwrap();
        assert_ne!(
            val["effective_mode"], "keyword",
            "con modelo el recall debe usar embeddings, got: {val}"
        );
        let contents: Vec<&str> = val["recalled"]
            .as_array()
            .expect("recalled array")
            .iter()
            .filter_map(|m| m["content"].as_str())
            .collect();
        assert!(
            contents.iter().any(|c| c.contains("gato")),
            "D0 (gato) debe aparecer por sinonimia felino↔gato, got: {contents:?}"
        );
        eprintln!(
            "EMB-15 señal recall: mode={} recalled={contents:?}",
            val["effective_mode"]
        );
    }
    #[cfg(not(any(feature = "embed-local", feature = "remote-inference")))]
    {
        unreachable!("gated por real_model_available()==false sin features");
    }
}
