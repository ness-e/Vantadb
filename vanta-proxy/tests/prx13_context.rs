// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! PRX-13 (RED): optimización de contexto en tránsito — resize por budget
//! (performance/balanced/economy, por key) sobre el body pre-forward.
//!
//! Safe default: modo Performance no toca bodies con tools o imágenes
//! (pre-mortem: resize degrada tool_calls con imagen).

use serde_json::{json, Value};
use vanta_proxy::context::{ApplyOutcome, ContextConfig, ContextMode, ContextOptimizer};
use vanta_proxy::inject::Protocol;

fn enabled(mode: ContextMode, max_input_tokens: u64) -> ContextConfig {
    ContextConfig {
        enabled: true,
        mode,
        max_input_tokens,
        ..ContextConfig::default()
    }
}

fn optimizer(mode: ContextMode, max_input_tokens: u64) -> ContextOptimizer {
    ContextOptimizer::new(&enabled(mode, max_input_tokens))
}

/// 8 turns de texto puro (~30 tokens c/u por heurística chars/4) → muy por
/// encima de un budget de 40 tokens.
fn long_history() -> Vec<u8> {
    let mut messages = vec![json!({"role": "system", "content": "you are helpful"})];
    for i in 0..8 {
        messages.push(
            json!({"role": "user", "content": format!("question number {i} with padding text here")}),
        );
        messages.push(
            json!({"role": "assistant", "content": format!("answer number {i} with padding text here too")}),
        );
    }
    json!({"model": "gpt-4o", "messages": messages})
        .to_string()
        .into_bytes()
}

fn message_roles(body: &[u8]) -> Vec<String> {
    let v: Value = serde_json::from_slice(body).expect("must stay valid JSON");
    v["messages"]
        .as_array()
        .expect("messages array survives")
        .iter()
        .map(|m| m["role"].as_str().unwrap_or("?").to_string())
        .collect()
}

// ── default transparente ─────────────────────────────────────────────────────

#[test]
fn default_config_is_transparent() {
    let cfg = ContextConfig::default();
    assert!(!cfg.enabled);
    assert_eq!(cfg.mode, ContextMode::Performance);
    assert!(cfg.modes_by_key.is_empty());
}

#[test]
fn disabled_passes_bytes_untouched() {
    let o = ContextOptimizer::new(&ContextConfig::default());
    let raw = long_history();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    assert_eq!(out, raw, "disabled must be byte-identical");
}

#[test]
fn under_budget_passes_bytes_untouched() {
    let o = optimizer(ContextMode::Economy, 1_000_000);
    let raw = long_history();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    assert_eq!(out, raw, "under budget must be byte-identical");
}

#[test]
fn non_json_passes_through() {
    let o = optimizer(ContextMode::Economy, 1);
    let raw = b"not json at all".to_vec();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    assert_eq!(out, raw);
}

// ── resize ───────────────────────────────────────────────────────────────────

#[test]
fn over_budget_drops_oldest_keeps_system_and_last_turns() {
    let o = optimizer(ContextMode::Balanced, 40);
    let raw = long_history();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    assert!(out.len() < raw.len(), "over budget must shrink");
    let roles = message_roles(&out);
    assert_eq!(roles.first().map(String::as_str), Some("system"));
    assert_eq!(roles.last().map(String::as_str), Some("assistant"));
    // 17 mensajes originales → recortado pero con estructura viva.
    assert!(roles.len() < 17, "oldest turns must be dropped: {roles:?}");
    assert!(roles.len() >= 3, "system + last turn survive: {roles:?}");
}

#[test]
fn oversize_body_fails_open() {
    let mut cfg = enabled(ContextMode::Economy, 1);
    cfg.max_scan_bytes = 16;
    let o = ContextOptimizer::new(&cfg);
    let raw = long_history();
    assert!(raw.len() > 16);
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    assert_eq!(out, raw);
}

// ── guards tools/imágenes por modo (pre-mortem) ─────────────────────────────

fn body_with_tools() -> Vec<u8> {
    json!({
        "model": "gpt-4o",
        "messages": [
            {"role": "system", "content": "you are helpful"},
            {"role": "user", "content": "question one with padding text here"},
            {"role": "assistant", "content": "thinking", "tool_calls": [
                {"id": "c1", "type": "function", "function": {"name": "f", "arguments": "{}"}}
            ]},
            {"role": "tool", "tool_call_id": "c1", "content": "tool result with padding text here"},
            {"role": "user", "content": "final question with padding text here"}
        ],
        "tools": [{"type": "function", "function": {"name": "f"}}]
    })
    .to_string()
    .into_bytes()
}

fn body_with_image() -> Vec<u8> {
    json!({
        "model": "gpt-4o",
        "messages": [
            {"role": "system", "content": "you are helpful"},
            {"role": "user", "content": "old question one with padding text"},
            {"role": "assistant", "content": "old answer one with padding text"},
            {"role": "user", "content": [
                {"type": "text", "text": "describe this with padding text here"},
                {"type": "image_url", "image_url": {"url": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUg"}}
            ]},
            {"role": "assistant", "content": "seen the image with padding text"},
            {"role": "user", "content": "follow up question with padding text"}
        ]
    })
    .to_string()
    .into_bytes()
}

#[test]
fn performance_mode_skips_bodies_with_tools() {
    let o = optimizer(ContextMode::Performance, 1);
    let raw = body_with_tools();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    assert_eq!(out, raw, "performance never touches tool bodies");
}

#[test]
fn performance_mode_skips_bodies_with_images() {
    let o = optimizer(ContextMode::Performance, 1);
    let raw = body_with_image();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    assert_eq!(out, raw, "performance never touches image bodies");
}

#[test]
fn balanced_mode_trims_text_but_keeps_images() {
    let o = optimizer(ContextMode::Balanced, 40);
    let raw = body_with_image();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    let s = String::from_utf8(out).expect("stays UTF-8 JSON");
    assert!(s.contains("image_url"), "balanced keeps image blocks");
    assert!(s.len() < raw.len(), "balanced still trims text turns");
}

#[test]
fn economy_mode_drops_image_blocks() {
    let o = optimizer(ContextMode::Economy, 1_000_000);
    let raw = body_with_image();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "any-key");
    let s = String::from_utf8(out).expect("stays UTF-8 JSON");
    assert!(!s.contains("image_url"), "economy drops image blocks");
    assert!(!s.contains("iVBORw0KGgo"), "image payload must not survive");
}

// ── por key ──────────────────────────────────────────────────────────────────

#[test]
fn mode_override_by_key_wins_over_global() {
    let mut cfg = enabled(ContextMode::Performance, 1);
    cfg.modes_by_key
        .insert("usr-eco".to_string(), ContextMode::Economy);
    let o = ContextOptimizer::new(&cfg);
    assert_eq!(o.mode_for("usr-eco"), ContextMode::Economy);
    assert_eq!(o.mode_for("other"), ContextMode::Performance);
    // La key en economy sí recorta historia de texto aunque el global sea safe.
    let raw = long_history();
    let ApplyOutcome::Pass(out) = o.apply(&raw, Protocol::OpenAI, "usr-eco");
    assert!(out.len() < raw.len());
    // El global Performance no toca bodies con tools…
    let tools = body_with_tools();
    let ApplyOutcome::Pass(same) = o.apply(&tools, Protocol::OpenAI, "other");
    assert_eq!(same, tools);
    // …pero la key en economy sí los recorta.
    let ApplyOutcome::Pass(trimmed) = o.apply(&tools, Protocol::OpenAI, "usr-eco");
    assert!(trimmed.len() < tools.len());
}

// ── wire: hook pre-forward (entre 5a y 5b) ──────────────────────────────────

use std::collections::HashMap;
use std::sync::Arc;

use axum::routing::post;
use axum::{Json, Router};
use vanta_proxy::config::{
    AuthConfig, CostConfig, MemCommandConfig, ProxyConfig, ServerConfig, UpstreamConfig,
};
use vanta_proxy::server;
use vantadb::entity::EntityStore;
use vantadb::node::FieldValue;

const USER_KEY: &str = "sk-test";
const USER_ID: &str = "usr-test";

fn seeded_engine() -> Arc<vantadb::storage::StorageEngine> {
    let config = vantadb::config::Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::Config::default()
    };
    let engine = vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config))
        .expect("engine");
    let mut fields: HashMap<String, FieldValue> = HashMap::new();
    fields.insert("user_key".into(), FieldValue::String(USER_KEY.to_string()));
    EntityStore::new(&engine)
        .entity_set("default", "user", USER_ID, fields)
        .expect("seed user");
    Arc::new(engine)
}

fn state_with_context(upstream: &str, context: ContextConfig) -> server::AppState {
    let cfg = ProxyConfig {
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream.to_string(),
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        upstreams: Vec::new(),
        auth: AuthConfig::default(),
        mem_command: MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig::default(),
        cache: Default::default(),
        report: Default::default(),
        cost: CostConfig::default(),
        routing: Default::default(),
        redact: Default::default(),
        context,
        guardrails: Default::default(),
        translate: Default::default(),
    };
    server::AppState::from_engine(cfg, seeded_engine()).unwrap()
}

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

async fn post_messages(proxy_url: &str, raw: Vec<u8>) -> reqwest::Response {
    let body: Value = serde_json::from_slice(&raw).unwrap();
    reqwest::Client::new()
        .post(format!("{proxy_url}/agent/space/v1/chat/completions"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", "s-context")
        .json(&body)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn economy_trims_long_history_before_upstream() {
    let seen = Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let seen_clone = seen.clone();
    let upstream = spawn(Router::new().route(
        "/v1/chat/completions",
        post(move |body: bytes::Bytes| async move {
            *seen_clone.lock().unwrap() = body.to_vec();
            (axum::http::StatusCode::OK, Json(json!({})))
        }),
    ))
    .await;
    let proxy = spawn(server::router(state_with_context(
        &upstream,
        ContextConfig {
            enabled: true,
            mode: ContextMode::Economy,
            max_input_tokens: 40,
            ..ContextConfig::default()
        },
    )))
    .await;

    let raw = long_history();
    let resp = post_messages(&proxy, raw.clone()).await;
    assert_eq!(resp.status().as_u16(), 200);
    let forwarded: Value =
        serde_json::from_slice(&seen.lock().unwrap().clone()).expect("upstream got JSON");
    let msgs = forwarded["messages"].as_array().expect("messages survive");
    assert!(msgs.len() < 17, "history must be trimmed: {}", msgs.len());
    assert_eq!(msgs[0]["role"], "system");
    assert_eq!(msgs.last().unwrap()["role"], "assistant");
}

#[tokio::test]
async fn disabled_default_forwards_history_verbatim() {
    let seen = Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let seen_clone = seen.clone();
    let upstream = spawn(Router::new().route(
        "/v1/chat/completions",
        post(move |body: bytes::Bytes| async move {
            *seen_clone.lock().unwrap() = body.to_vec();
            (axum::http::StatusCode::OK, Json(json!({})))
        }),
    ))
    .await;
    let proxy = spawn(server::router(state_with_context(
        &upstream,
        ContextConfig::default(),
    )))
    .await;

    // Sin sesión el pipeline es verbatim (D29 solo-sesiones); con sesión y
    // context disabled, los mensajes originales viajan intactos.
    let raw = long_history();
    let before: Value = serde_json::from_slice(&raw).unwrap();
    let resp = post_messages(&proxy, raw).await;
    assert_eq!(resp.status().as_u16(), 200);
    let forwarded: Value =
        serde_json::from_slice(&seen.lock().unwrap().clone()).expect("upstream got JSON");
    assert_eq!(
        forwarded["messages"], before["messages"],
        "disabled context must not touch turns"
    );
}
#[test]
fn toml_parses_modes_and_overrides() {
    let cfg: ContextConfig = toml::from_str(
        "enabled = true\nmode = \"economy\"\nmax_input_tokens = 4000\n\
         [modes_by_key]\n\"usr-1\" = \"balanced\"\n",
    )
    .expect("context TOML must parse");
    assert!(cfg.enabled);
    assert_eq!(cfg.mode, ContextMode::Economy);
    assert_eq!(cfg.max_input_tokens, 4000);
    assert_eq!(cfg.modes_by_key.get("usr-1"), Some(&ContextMode::Balanced));
}
