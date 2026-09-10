// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! PRX-06 slice 1 (RED): task-aware routing por tier — tier→modelo/upstream.
//!
//! El clasificador CC (`classify_cc_request`) existe huérfano: solo Sidequery
//! se usa (bypass). Estos tests mapean Main/Fork/Sidequery → modelo + upstream.

use serde_json::json;
use vanta_proxy::config::ProxyConfig;
use vanta_proxy::inject::Protocol;
use vanta_proxy::routing::{tier_of, Tier, TierRoutingConfig};

fn anthropic_main() -> Vec<u8> {
    // Marker on last message → Main.
    serde_json::to_vec(&json!({
        "model": "claude-opus",
        "messages": [
            {"role": "user", "content": "hi"},
            {"role": "user", "content": [{"type": "text", "text": "latest",
                "cache_control": {"type": "ephemeral"}}]},
        ],
    }))
    .unwrap()
}

fn anthropic_fork() -> Vec<u8> {
    // Marker on n-2 → Fork.
    serde_json::to_vec(&json!({
        "model": "claude-opus",
        "messages": [
            {"role": "user", "content": "old"},
            {"role": "user", "content": [{"type": "text", "text": "cached",
                "cache_control": {"type": "ephemeral"}}]},
            {"role": "user", "content": "new tail"},
        ],
    }))
    .unwrap()
}

fn routing_cfg() -> TierRoutingConfig {
    toml::from_str(
        "enabled = true\n\
         mode = \"enforce\"\n\
         fork_model = \"claude-haiku\"\n\
         fork_upstream = 1\n",
    )
    .expect("routing TOML must parse")
}

// ── tier_of ────────────────────────────────────────────────────────────────

#[test]
fn tier_of_maps_cc_kinds_on_anthropic_only() {
    assert_eq!(
        tier_of(Protocol::Anthropic, &anthropic_main()),
        Some(Tier::Main)
    );
    assert_eq!(
        tier_of(Protocol::Anthropic, &anthropic_fork()),
        Some(Tier::Fork)
    );
    // Non-Anthropic never routes (CC classifier is Anthropic-only).
    assert_eq!(tier_of(Protocol::OpenAI, &anthropic_fork()), None);
    assert_eq!(tier_of(Protocol::Responses, &anthropic_fork()), None);
    // Garbage → None (fail-open, transparent proxy).
    assert_eq!(tier_of(Protocol::Anthropic, b"not json"), None);
}

// ── resolve: tier → modelo/upstream ────────────────────────────────────────

#[test]
fn fork_resolves_to_cheap_model_and_second_upstream() {
    let cfg = routing_cfg();
    let d = cfg
        .resolve(Tier::Fork, "sk-user", 2)
        .expect("enabled → Some");
    assert_eq!(d.model_override.as_deref(), Some("claude-haiku"));
    assert_eq!(d.upstream_index, 1);
    assert!(!d.shadow);
}

#[test]
fn main_tier_keeps_primary_upstream_and_no_model_override() {
    let cfg = routing_cfg();
    let d = cfg
        .resolve(Tier::Main, "sk-user", 2)
        .expect("enabled → Some");
    assert_eq!(d.model_override, None);
    assert_eq!(d.upstream_index, 0);
}

#[test]
fn disabled_routing_resolves_to_none() {
    let cfg = TierRoutingConfig::default();
    assert!(!cfg.enabled);
    assert_eq!(cfg.resolve(Tier::Fork, "sk-user", 2), None);
}

#[test]
fn bypass_key_skips_routing() {
    let mut cfg = routing_cfg();
    cfg.bypass_keys = vec!["sk-admin".to_string()];
    assert_eq!(cfg.resolve(Tier::Fork, "sk-admin", 2), None);
    assert!(cfg.resolve(Tier::Fork, "sk-user", 2).is_some());
}

#[test]
fn shadow_mode_reports_but_marks_shadow() {
    let mut cfg = routing_cfg();
    cfg.mode = vanta_proxy::routing::RoutingMode::Shadow;
    let d = cfg
        .resolve(Tier::Fork, "sk-user", 2)
        .expect("shadow → Some");
    assert!(d.shadow);
    assert_eq!(d.model_override.as_deref(), Some("claude-haiku"));
}

#[test]
fn upstream_index_clamps_to_available_list() {
    let cfg = routing_cfg(); // fork_upstream = 1, single upstream configured
    let d = cfg
        .resolve(Tier::Fork, "sk-user", 1)
        .expect("enabled → Some");
    assert_eq!(d.upstream_index, 0, "out-of-range clamps to primary");
}

// ── rewrite + TOML compat ──────────────────────────────────────────────────

#[test]
fn rewrite_model_swaps_model_field_only() {
    let out = vanta_proxy::routing::rewrite_model(&anthropic_main(), "claude-haiku");
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["model"], "claude-haiku");
    assert_eq!(v["messages"].as_array().unwrap().len(), 2);
}

#[test]
fn legacy_toml_without_routing_parses_disabled() {
    let cfg: ProxyConfig = toml::from_str(
        "[server]\nhost = \"127.0.0.1\"\nport = 8096\n\
         [upstream]\nurl = \"https://api.anthropic.com\"\napi_key = \"k\"\n",
    )
    .expect("legacy TOML must parse");
    assert!(!cfg.routing.enabled);
}

#[test]
fn full_toml_with_routing_table_parses() {
    let cfg: ProxyConfig = toml::from_str(
        "[upstream]\nurl = \"https://api.anthropic.com\"\n\
         [routing]\nenabled = true\nmode = \"enforce\"\n\
         fork_model = \"claude-haiku\"\n\
         bypass_keys = [\"sk-admin\"]\n",
    )
    .expect("routing TOML must parse");
    assert!(cfg.routing.enabled);
    assert_eq!(cfg.routing.fork_model.as_deref(), Some("claude-haiku"));
    assert_eq!(cfg.routing.bypass_keys, vec!["sk-admin"]);
}

// ── slice 2 e2e: tier→upstream/modelo sobre el wire ────────────────────────

use std::sync::{Arc, Mutex};
use vanta_proxy::routing::RoutingMode;

const USER_KEY: &str = "sk-prx06-test";

fn seeded_engine() -> Arc<vantadb::storage::StorageEngine> {
    let config = vantadb::config::VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::VantaConfig::default()
    };
    let engine =
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    let mut fields = std::collections::HashMap::new();
    fields.insert(
        "user_key".to_string(),
        vantadb::node::FieldValue::String(USER_KEY.to_string()),
    );
    vantadb::entity::EntityStore::new(&engine)
        .entity_set("default", "user", "usr-prx06", fields)
        .unwrap();
    Arc::new(engine)
}

fn upstream_cfg(url: String) -> vanta_proxy::config::UpstreamConfig {
    vanta_proxy::config::UpstreamConfig {
        url,
        api_key: String::new(),
        forward_timeout_secs: 600,
        models: Vec::new(),
    }
}

/// Mock upstream que captura bodies y responde JSON fijo.
async fn spawn_mock(captured: Arc<Mutex<Vec<Vec<u8>>>>) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = axum::Router::new().route(
        "/v1/messages",
        axum::routing::post(move |body: bytes::Bytes| {
            let captured = captured.clone();
            async move {
                captured.lock().unwrap().push(body.to_vec());
                axum::http::Response::builder()
                    .status(200)
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"id":"msg_1","type":"message"}"#))
                    .unwrap()
            }
        }),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn spawn_proxy(url_a: String, url_b: String) -> String {
    let engine = seeded_engine();
    let cfg = ProxyConfig {
        server: Default::default(),
        upstream: upstream_cfg(url_a.clone()),
        upstreams: vec![upstream_cfg(url_a), upstream_cfg(url_b)],
        auth: Default::default(),
        mem_command: Default::default(),
        writeback: vanta_proxy::config::WritebackConfig {
            persist_path: String::new(),
        },
        cache: Default::default(),
        report: Default::default(),
        cost: Default::default(),
        routing: TierRoutingConfig {
            enabled: true,
            mode: RoutingMode::Enforce,
            fork_model: Some("claude-haiku".to_string()),
            fork_upstream: 1,
            ..Default::default()
        },
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = vanta_proxy::server::router(
        vanta_proxy::server::AppState::from_engine(cfg, engine).unwrap(),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn post_anthropic(proxy_url: &str, body: Vec<u8>) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("{proxy_url}/cc/s1/v1/messages"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .body(body)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn fork_routes_to_cheap_upstream_with_model_override() {
    let hit_a: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let hit_b: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let url_a = spawn_mock(hit_a.clone()).await;
    let url_b = spawn_mock(hit_b.clone()).await;
    let proxy = spawn_proxy(url_a, url_b).await;

    let resp = post_anthropic(&proxy, anthropic_fork()).await;
    assert_eq!(resp.status(), 200);

    // Fork → upstream B (índice 1) con modelo reescrito…
    assert_eq!(hit_a.lock().unwrap().len(), 0, "primary must not see fork");
    assert_eq!(hit_b.lock().unwrap().len(), 1);
    let seen: serde_json::Value = serde_json::from_slice(&hit_b.lock().unwrap()[0]).unwrap();
    assert_eq!(seen["model"], "claude-haiku");

    // …mientras Main sigue al primario con modelo intacto.
    let resp = post_anthropic(&proxy, anthropic_main()).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(hit_a.lock().unwrap().len(), 1);
    assert_eq!(hit_b.lock().unwrap().len(), 1, "no re-route of main");
    let seen: serde_json::Value = serde_json::from_slice(&hit_a.lock().unwrap()[0]).unwrap();
    assert_eq!(seen["model"], "claude-opus");
}

// ── slice 3 e2e: `/v1/responses` dentro del tool-loop ───────────────────────

/// Responses SSE con un function_call nuestro (shape sintético realista:
/// `response.output_item.added` + `function_call_arguments.delta`).
fn responses_tool_call_sse(name: &str, args: serde_json::Value) -> String {
    use serde_json::json;
    format!(
        "data: {}\n\ndata: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        json!({"type":"response.output_item.added","output_index":0,
            "item":{"type":"function_call","id":"fc_1","call_id":"call_1",
                "name":name,"arguments":""}}),
        json!({"type":"response.function_call_arguments.delta","item_id":"fc_1",
            "output_index":0,"delta":args.to_string()}),
        json!({"type":"response.completed","response":{"id":"resp_1"}}),
    )
}

fn responses_final_sse(text: &str) -> String {
    use serde_json::json;
    format!(
        "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        json!({"type":"response.output_text.delta","item_id":"msg_1",
            "output_index":0,"delta":text}),
        json!({"type":"response.completed","response":{"id":"resp_2"}}),
    )
}

/// Mock scripted para `/v1/responses`: la n-ésima llamada devuelve scripts[n].
async fn spawn_responses_mock(captured: Arc<Mutex<Vec<Vec<u8>>>>, scripts: Vec<String>) -> String {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let hits = Arc::new(AtomicUsize::new(0));
    let scripts = Arc::new(Mutex::new(scripts));
    let app = axum::Router::new().route(
        "/v1/responses",
        axum::routing::post(move |body: bytes::Bytes| {
            let captured = captured.clone();
            let hits = hits.clone();
            let scripts = scripts.clone();
            async move {
                captured.lock().unwrap().push(body.to_vec());
                let idx = hits.fetch_add(1, Ordering::SeqCst);
                let payload = scripts
                    .lock()
                    .unwrap()
                    .get(idx)
                    .cloned()
                    .unwrap_or_default();
                axum::http::Response::builder()
                    .status(200)
                    .header("content-type", "text/event-stream")
                    .body(axum::body::Body::from(payload))
                    .unwrap()
            }
        }),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn responses_with_memory_tool_executes_server_side_and_loops() {
    use serde_json::json;
    let captured: Arc<Mutex<Vec<Vec<u8>>>> = Arc::new(Mutex::new(Vec::new()));
    let upstream = spawn_responses_mock(
        captured.clone(),
        vec![
            responses_tool_call_sse("vanta_memory_capture", json!({"text": "remember this"})),
            responses_final_sse("done"),
        ],
    )
    .await;

    // Proxy single-upstream apuntando al mock (routing default-off).
    let engine = seeded_engine();
    let cfg = ProxyConfig {
        server: Default::default(),
        upstream: upstream_cfg(upstream),
        upstreams: Vec::new(),
        auth: Default::default(),
        mem_command: Default::default(),
        writeback: vanta_proxy::config::WritebackConfig {
            persist_path: String::new(),
        },
        cache: Default::default(),
        report: Default::default(),
        cost: Default::default(),
        routing: Default::default(),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = vanta_proxy::server::router(
        vanta_proxy::server::AppState::from_engine(cfg, engine).unwrap(),
    );
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let proxy = format!("http://{addr}");

    let resp = reqwest::Client::new()
        .post(format!("{proxy}/v1/responses"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-claude-code-session-id", "sess-prx06-resp")
        .json(&json!({
            "model": "gpt-test",
            "instructions": "sys",
            "input": [{"role": "user", "content": "hi"}],
            "tools": [{"type": "function", "name": "vanta_memory_capture"}],
            "stream": true,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let streamed = resp.text().await.unwrap();
    assert!(streamed.contains("done"), "final streamed: {streamed}");

    // Dos forwards: el segundo lleva function_call + function_call_output en `input`.
    assert_eq!(captured.lock().unwrap().len(), 2);
    let second: serde_json::Value = serde_json::from_slice(&captured.lock().unwrap()[1]).unwrap();
    let input = second["input"].as_array().unwrap();
    let call = input
        .iter()
        .find(|i| i["type"] == "function_call")
        .expect("history carries the executed call");
    assert_eq!(call["call_id"], "call_1");
    let output = input
        .iter()
        .find(|i| i["type"] == "function_call_output")
        .expect("history carries the synthesized result");
    assert_eq!(output["call_id"], "call_1");
    assert_eq!(output["output"], "Memory captured.");
}
