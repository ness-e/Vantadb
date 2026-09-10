// ponytail: blanket allow — unwraps with documented invariants; same pattern as
// pipeline.rs (neighboring test file).
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! PRX-09 slice 1: exact response cache. RED first — `vanta_proxy::cache`
//! does not exist yet, so this suite fails to compile (correct RED reason).

use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use axum::http::HeaderMap;
use axum::routing::post;
use axum::{Json, Router};
use bytes::Bytes;
use serde_json::{json, Value};
use vanta_proxy::cache::{is_cacheable_request, ExactCache};
use vanta_proxy::config::{CacheConfig, ProxyConfig};
use vanta_proxy::inject::{inject_into, Protocol};
use vantadb::entity::EntityStore;
use vantadb::node::FieldValue;
use vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata};
use vantadb::storage::StorageEngine;

const USER_KEY: &str = "sk-cache-test";
const USER_ID: &str = "usr-cache-test";

/// Upstream mock: counts hits and records every body received.
struct Upstream {
    hits: Arc<AtomicUsize>,
    bodies: Arc<Mutex<Vec<Vec<u8>>>>,
}

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

fn seeded_engine() -> Arc<StorageEngine> {
    let config = vantadb::config::VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::VantaConfig::default()
    };
    let engine = StorageEngine::open_with_config(":memory:", Some(config)).expect("engine");
    let mut fields: HashMap<String, FieldValue> = HashMap::new();
    fields.insert("user_key".into(), FieldValue::String(USER_KEY.to_string()));
    EntityStore::new(&engine)
        .entity_set("default", "user", USER_ID, fields)
        .expect("seed user");
    Arc::new(engine)
}

fn state_for(upstream_url: &str) -> vanta_proxy::server::AppState {
    let cfg = ProxyConfig {
        report: Default::default(),
        cost: Default::default(),
        server: Default::default(),
        upstream: vanta_proxy::config::UpstreamConfig {
            url: upstream_url.to_string(),
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        upstreams: Vec::new(),
        auth: Default::default(),
        mem_command: Default::default(),
        writeback: Default::default(),
        cache: CacheConfig {
            enabled: true,
            max_entries: 128,
        },
    };
    vanta_proxy::server::AppState::from_engine(cfg, seeded_engine()).unwrap()
}

fn seed_memory(db: &VantaEmbedded, session_key: &str) {
    use vanta_memory::core::abstractions::PersonaMode;
    use vanta_memory::core::persona::persona_generator::{
        persona_namespace, PersonaRecord, PERSONA_KEY,
    };
    use vanta_memory::core::scene::scene_index::upsert_scene;

    let record = PersonaRecord {
        content: "# Cache Profile\nPrefers verbose answers.".into(),
        mode: PersonaMode::First,
        generated_at_ms: 0,
        generated_at: "2026-09-09T00:00:00+00:00".into(),
    };
    db.put(VantaMemoryInput {
        namespace: persona_namespace(session_key),
        key: PERSONA_KEY.into(),
        payload: serde_json::to_string(&record).expect("persona json"),
        metadata: VantaMemoryMetadata::new(),
        vector: None,
        sparse_vector: None,
        ttl_ms: None,
    })
    .expect("seed persona");
    upsert_scene(db, session_key, "cache-runbook", "deploys", "how to deploy").expect("seed scene");
}

struct TestEnv {
    proxy_url: String,
    upstream: Upstream,
    memory: VantaEmbedded,
}

async fn setup() -> TestEnv {
    let hits = Arc::new(AtomicUsize::new(0));
    let bodies = Arc::new(Mutex::new(Vec::new()));
    let h = hits.clone();
    let b = bodies.clone();
    let upstream = Router::new().route(
        "/v1/chat/completions",
        post(move |_headers: HeaderMap, body: Bytes| {
            let h = h.clone();
            let b = b.clone();
            async move {
                h.fetch_add(1, Ordering::SeqCst);
                b.lock().unwrap().push(body.to_vec());
                Json(json!({ "id": "chatcmpl-cache-1", "choices": [] }))
            }
        }),
    );
    let upstream_url = spawn(upstream).await;
    let state = state_for(&upstream_url);
    let memory = state.memory.as_ref().clone();
    let proxy_url = spawn(vanta_proxy::server::router(state)).await;
    TestEnv {
        proxy_url,
        upstream: Upstream { hits, bodies },
        memory,
    }
}

async fn post_chat(env: &TestEnv, session: &str, body: Value) -> Vec<u8> {
    let resp = reqwest::Client::new()
        .post(format!("{}/agent/space/v1/chat/completions", env.proxy_url))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-conversation-id", session)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    resp.bytes().await.unwrap().to_vec()
}

/// PRX-09 contrato: mismo request exacto → 1 solo hit upstream,
/// ambas respuestas byte-a-byte idénticas.
#[tokio::test]
async fn exact_hit_byte_identical_upstream_once() {
    let env = setup().await;
    let payload = json!({ "model": "m", "messages": [{ "role": "user", "content": "hello" }] });

    let first = post_chat(&env, "sess-cache-hit", payload.clone()).await;
    let second = post_chat(&env, "sess-cache-hit", payload).await;

    assert_eq!(first, second, "cache hit must be byte-identical");
    assert_eq!(
        env.upstream.hits.load(Ordering::SeqCst),
        1,
        "second identical request must not reach upstream"
    );
}

/// PRX-09 + PRX-04 combinado: cambiar la memoria invalida la entrada
/// (la clave incluye el body inyectado) sin romper el prefijo estable.
#[tokio::test]
async fn memory_change_invalidates_without_breaking_prefix() {
    let env = setup().await;
    let payload = json!({ "model": "m", "messages": [{ "role": "user", "content": "hi" }] });

    let _ = post_chat(&env, "sess-inv", payload.clone()).await;
    let _ = post_chat(&env, "sess-inv", payload.clone()).await;
    assert_eq!(env.upstream.hits.load(Ordering::SeqCst), 1);

    // Memory appears → injected body changes → same raw request misses.
    seed_memory(&env.memory, "sess-inv");
    let _ = post_chat(&env, "sess-inv", payload.clone()).await;
    assert_eq!(env.upstream.hits.load(Ordering::SeqCst), 2);

    let (first_sys, second_sys) = {
        let bodies = env.upstream.bodies.lock().unwrap();
        assert_eq!(bodies.len(), 2);
        let first: Value = serde_json::from_slice(&bodies[0]).unwrap();
        let second: Value = serde_json::from_slice(&bodies[1]).unwrap();
        let sys = |v: &Value| {
            v["messages"][0]["content"]
                .as_str()
                .unwrap_or("")
                .to_string()
        };
        (sys(&first), sys(&second))
    };
    assert!(
        !first_sys.contains("<vanta-memory>"),
        "first upstream body has no memory block"
    );
    assert!(
        second_sys.contains("<vanta-memory>"),
        "after seeding, upstream body carries the stable prefix"
    );

    // Fourth identical request hits the NEW key — upstream stays at 2.
    let _ = post_chat(&env, "sess-inv", payload).await;
    assert_eq!(env.upstream.hits.load(Ordering::SeqCst), 2);
}

/// Coordinación cache↔PRX-04 a nivel unidad: la clave del cache son los
/// bytes post-inyección, y la re-inyección es estable → misma clave.
#[test]
fn cache_key_stable_under_reinjection() {
    let raw = Bytes::from_static(
        br#"{"model":"m","messages":[{"role":"system","content":"S"},{"role":"user","content":"u"}]}"#,
    );
    let once = inject_into(&raw, Protocol::OpenAI, "BLOCK")
        .expect("ok")
        .expect("first injection modifies");
    let twice = inject_into(&Bytes::from(once.clone()), Protocol::OpenAI, "BLOCK").expect("ok");
    let stable = twice.unwrap_or_else(|| once.clone());
    assert_eq!(once, stable, "PRX-04: re-inject byte-stable");

    let mut cache = ExactCache::new(CacheConfig {
        enabled: true,
        max_entries: 8,
    });
    assert!(is_cacheable_request(&once));
    assert!(!is_cacheable_request(b"not-json"));
    cache.store(
        "openai",
        "/v1/chat/completions",
        &stable,
        vanta_proxy::cache::CachedEntry {
            status: 200,
            content_type: "application/json".to_string(),
            body: b"{}".to_vec(),
        },
    );
    let hit = cache
        .lookup("openai", "/v1/chat/completions", &once)
        .expect("same post-inject bytes hit");
    assert_eq!(hit.body, b"{}".to_vec());
    assert!(cache
        .lookup("openai", "/v1/chat/completions", &raw)
        .is_none());
}
