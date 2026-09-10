// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! PRX-07 (RED): PII/secret redaction en egress — AWS keys, tokens, emails,
//! custom regex × modos block/mask/log sobre el body pre-forward.

use vanta_proxy::redact::{ApplyOutcome, RedactConfig, RedactMode, Redactor};

const AWS_KEY: &str = "AKIAIOSFODNN7EXAMPLE";

fn body_with(items: &[&str]) -> Vec<u8> {
    format!(
        "{{\"model\":\"m\",\"messages\":[{{\"role\":\"user\",\"content\":\"{}\" namely\"}}]}}",
        items.join(" ")
    )
    .into_bytes()
}

fn enabled(mode: RedactMode) -> RedactConfig {
    RedactConfig {
        enabled: true,
        mode,
        ..RedactConfig::default()
    }
}

// ── built-ins ────────────────────────────────────────────────────────────────

#[test]
fn aws_key_masked_without_echo() {
    let r = Redactor::new(&enabled(RedactMode::Mask)).expect("default config builds");
    let out = r.apply(&body_with(&[AWS_KEY]));
    let ApplyOutcome::Pass(bytes) = out else {
        panic!("mask mode must pass, not block")
    };
    let s = String::from_utf8(bytes).unwrap();
    assert!(s.contains("[REDACTED_AWS_KEY]"), "key must be masked");
    assert!(!s.contains(AWS_KEY), "secret must never echo");
}

#[test]
fn email_masked() {
    let r = Redactor::new(&enabled(RedactMode::Mask)).expect("default config builds");
    let ApplyOutcome::Pass(bytes) = r.apply(&body_with(&["contact jane.doe@example.com please"]))
    else {
        panic!("mask mode must pass")
    };
    let s = String::from_utf8(bytes).unwrap();
    assert!(s.contains("[REDACTED_EMAIL]"));
    assert!(!s.contains("jane.doe@example.com"));
}

#[test]
fn generic_tokens_masked() {
    let r = Redactor::new(&enabled(RedactMode::Mask)).expect("default config builds");
    let secrets = [
        "sk-ant-abc123XYZ456",
        "ghp_0123456789abcdef0123456789abcdef0123",
    ];
    let ApplyOutcome::Pass(bytes) = r.apply(&body_with(&secrets)) else {
        panic!("mask mode must pass")
    };
    let s = String::from_utf8(bytes).unwrap();
    assert!(s.contains("[REDACTED_TOKEN]"));
    for secret in secrets {
        assert!(!s.contains(secret), "token must never echo: {secret}");
    }
}

#[test]
fn clean_body_passes_through_untouched() {
    let r = Redactor::new(&enabled(RedactMode::Mask)).expect("default config builds");
    let clean = body_with(&["hello world, no secrets here"]);
    let ApplyOutcome::Pass(bytes) = r.apply(&clean) else {
        panic!("clean body must pass")
    };
    assert_eq!(bytes, clean, "mask must be byte-stable without findings");
}

// ── custom regex ─────────────────────────────────────────────────────────────

#[test]
fn custom_regex_pattern_matches() {
    let cfg: RedactConfig =
        toml::from_str("enabled = true\nmode = \"mask\"\npatterns = [\"ticket-\\\\d{4,}\"]\n")
            .expect("redact TOML must parse");
    let r = Redactor::new(&cfg).expect("valid regex builds");
    let ApplyOutcome::Pass(bytes) = r.apply(&body_with(&["see ticket-98765 now"])) else {
        panic!("mask mode must pass")
    };
    let s = String::from_utf8(bytes).unwrap();
    assert!(s.contains("[REDACTED_CUSTOM]"));
    assert!(!s.contains("ticket-98765"));
}

#[test]
fn invalid_regex_rejected_at_construction() {
    let cfg = RedactConfig {
        enabled: true,
        patterns: vec!["([a-z".to_string()],
        ..RedactConfig::default()
    };
    assert!(
        Redactor::new(&cfg).is_err(),
        "invalid pattern must fail closed at construction"
    );
}

// ── modos ────────────────────────────────────────────────────────────────────

#[test]
fn block_mode_reports_kinds_without_values() {
    let r = Redactor::new(&enabled(RedactMode::Block)).expect("default config builds");
    let ApplyOutcome::Block(kinds) = r.apply(&body_with(&[AWS_KEY])) else {
        panic!("block mode must block")
    };
    assert!(kinds.iter().any(|k| k == "aws_key"), "kinds: {kinds:?}");
    for k in &kinds {
        assert!(!k.contains(AWS_KEY), "kinds must never carry secret values");
    }
}

#[test]
fn log_mode_forwards_bytes_unchanged() {
    let r = Redactor::new(&enabled(RedactMode::Log)).expect("default config builds");
    let raw = body_with(&[AWS_KEY]);
    let ApplyOutcome::Pass(bytes) = r.apply(&raw) else {
        panic!("log mode must pass")
    };
    assert_eq!(bytes, raw, "log mode never mutates the wire");
    assert!(!r.scan(&raw).is_empty(), "log mode still reports findings");
}

#[test]
fn disabled_is_transparent() {
    let r = Redactor::new(&RedactConfig::default()).expect("default config builds");
    assert!(!RedactConfig::default().enabled);
    let raw = body_with(&[AWS_KEY]);
    let ApplyOutcome::Pass(bytes) = r.apply(&raw) else {
        panic!("disabled must pass")
    };
    assert_eq!(bytes, raw);
    assert!(r.scan(&raw).is_empty());
}

// ── DoS bound ────────────────────────────────────────────────────────────────

#[test]
fn oversize_body_fails_open() {
    let cfg = RedactConfig {
        enabled: true,
        max_scan_bytes: 16,
        ..RedactConfig::default()
    };
    let r = Redactor::new(&cfg).expect("config builds");
    let raw = body_with(&[AWS_KEY]);
    assert!(raw.len() > 16);
    let ApplyOutcome::Pass(bytes) = r.apply(&raw) else {
        panic!("oversize must fail open")
    };
    assert_eq!(bytes, raw);
}

// ── wire: hook pre-forward (paso 5a) ─────────────────────────────────────────

use std::collections::HashMap;
use std::sync::Arc;

use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use vanta_proxy::config::{
    AuthConfig, CostConfig, MemCommandConfig, ProxyConfig, ServerConfig, UpstreamConfig,
};
use vanta_proxy::server;
use vantadb::entity::EntityStore;
use vantadb::node::FieldValue;

const USER_KEY: &str = "sk-test";
const USER_ID: &str = "usr-test";

fn seeded_engine() -> Arc<vantadb::storage::StorageEngine> {
    let config = vantadb::config::VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::VantaConfig::default()
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

fn state_with_redact(upstream: &str, redact: RedactConfig) -> server::AppState {
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
        redact,
        context: Default::default(),
        guardrails: Default::default(),
    };
    server::AppState::from_engine(cfg, seeded_engine()).unwrap()
}

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

fn key_message() -> Value {
    json!({
        "model": "gpt-4o",
        "messages": [{ "role": "user", "content": format!("deploy with {AWS_KEY} now") }]
    })
}

async fn post_messages(proxy_url: &str, body: Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("{proxy_url}/agent/space/v1/chat/completions"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-vanta-session", "s-redact")
        .json(&body)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn block_mode_answers_422_kinds_only_without_forwarding() {
    let upstream = spawn(Router::new().route(
        "/v1/chat/completions",
        post(move |body: bytes::Bytes| async move {
            let _ = body;
            (axum::http::StatusCode::OK, Json(json!({})))
        }),
    ))
    .await;
    let proxy = spawn(server::router(state_with_redact(
        &upstream,
        RedactConfig {
            enabled: true,
            mode: RedactMode::Block,
            ..RedactConfig::default()
        },
    )))
    .await;

    let resp = post_messages(&proxy, key_message()).await;
    assert_eq!(resp.status().as_u16(), 422);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["type"], "redaction_blocked");
    let raw = body.to_string();
    assert!(raw.contains("aws_key"), "kinds must be reported: {raw}");
    assert!(!raw.contains(AWS_KEY), "secret must never echo in 422");
}

#[tokio::test]
async fn mask_mode_scrubs_body_before_upstream() {
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
    let proxy = spawn(server::router(state_with_redact(
        &upstream,
        RedactConfig {
            enabled: true,
            mode: RedactMode::Mask,
            ..RedactConfig::default()
        },
    )))
    .await;

    let resp = post_messages(&proxy, key_message()).await;
    assert_eq!(resp.status().as_u16(), 200);
    let forwarded = String::from_utf8(seen.lock().unwrap().clone()).unwrap();
    assert!(forwarded.contains("[REDACTED_AWS_KEY]"));
    assert!(
        !forwarded.contains(AWS_KEY),
        "upstream must never see the secret"
    );
}
