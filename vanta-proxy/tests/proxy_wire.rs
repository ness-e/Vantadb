//! MEM-25 contract tests: verbatim wire forwarding against a mocked upstream.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::HeaderMap;
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::StreamExt;
use serde_json::{json, Value};
use vanta_proxy::config::{ProxyConfig, ServerConfig, UpstreamConfig};
use vanta_proxy::server;

const USER_KEY: &str = "sk-wire-test";

/// In-memory engine seeded with the wire-test user (D34: auth is mandatory).
fn seeded_engine() -> std::sync::Arc<vantadb::storage::StorageEngine> {
    let config = vantadb::config::VantaConfig {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::VantaConfig::default()
    };
    let engine =
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    let mut fields: std::collections::HashMap<String, vantadb::node::FieldValue> =
        std::collections::HashMap::new();
    fields.insert(
        "user_key".to_string(),
        vantadb::node::FieldValue::String(USER_KEY.to_string()),
    );
    vantadb::entity::EntityStore::new(&engine)
        .entity_set("default", "user", "usr-wire", fields)
        .unwrap();
    std::sync::Arc::new(engine)
}

/// Captured upstream request for assertions.
#[derive(Default)]
struct Captured {
    body: Vec<u8>,
    headers: HeaderMap,
    path: String,
}

type Shared = std::sync::Arc<Mutex<Option<Captured>>>;

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

struct TestEnv {
    proxy_url: String,
    upstream_captured: Shared,
}

/// Spawn a mock upstream + the real proxy wired to it.
async fn setup(extra_upstream_routes: Router) -> TestEnv {
    let captured: Shared = std::sync::Arc::new(Mutex::new(None));
    let c1 = captured.clone();
    let c2 = captured.clone();

    let mut app = Router::new()
        .route(
            "/v1/chat/completions",
            post(|headers: HeaderMap, body: bytes::Bytes| async move {
                *c1.lock().unwrap() = Some(Captured {
                    body: body.to_vec(),
                    headers,
                    path: "/v1/chat/completions".into(),
                });
                Json(json!({ "id": "chatcmpl-1", "object": "chat.completion" }))
            }),
        )
        .route(
            "/v1/messages",
            post(|headers: HeaderMap, body: bytes::Bytes| async move {
                *c2.lock().unwrap() = Some(Captured {
                    body: body.to_vec(),
                    headers,
                    path: "/v1/messages".into(),
                });
                Json(json!({ "id": "msg_1", "type": "message" }))
            }),
        )
        .route("/health", get(|| async { Json(json!({ "up": true })) }))
        .merge(extra_upstream_routes);
    let c4 = captured.clone();
    app = app.route(
        "/v1/responses",
        post(move |headers: HeaderMap, body: bytes::Bytes| async move {
            *c4.lock().unwrap() = Some(Captured {
                body: body.to_vec(),
                headers,
                path: "/v1/responses".into(),
            });
            Json(json!({ "id": "resp_1", "object": "response" }))
        }),
    );

    let upstream_url = spawn(app).await;
    let cfg = ProxyConfig {
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream_url,
            api_key: String::new(),
            forward_timeout_secs: 600,
        },
        auth: vanta_proxy::config::AuthConfig::default(),
    };
    let state = server::AppState::from_engine(cfg, seeded_engine()).unwrap();
    let proxy_url = spawn(server::router(state)).await;
    TestEnv {
        proxy_url,
        upstream_captured: captured,
    }
}

fn cfg_with(upstream: &str, timeout_secs: u64) -> ProxyConfig {
    ProxyConfig {
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream.to_string(),
            api_key: String::new(),
            forward_timeout_secs: timeout_secs,
        },
        auth: vanta_proxy::config::AuthConfig::default(),
    }
}

fn state_with(upstream: &str, timeout_secs: u64) -> server::AppState {
    let cfg = cfg_with(upstream, timeout_secs);
    server::AppState::from_engine(cfg, seeded_engine()).unwrap()
}

async fn post_json(proxy_url: &str, path: &str, body: Value) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .post(format!("{proxy_url}{path}"))
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-key-123")
        .header("x-vanta-user-key", USER_KEY)
        .json(&body)
        .send()
        .await
        .unwrap()
}

// ── (a) /v1/chat/completions forward verbatim ────────────────────────────────
#[tokio::test]
async fn a_openai_chat_completions_forward_verbatim() {
    let env = setup(Router::new()).await;
    let proxy_host = env.proxy_url.trim_start_matches("http://").to_string();
    let payload = json!({ "model": "gpt-test", "messages": [{"role": "user", "content": "hi"}] });
    let resp = post_json(
        &env.proxy_url,
        "/claude-code/space-42/v1/chat/completions",
        payload.clone(),
    )
    .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "application/json");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "chatcmpl-1");

    let cap = env.upstream_captured.lock().unwrap();
    let cap = cap.as_ref().expect("upstream received request");
    assert_eq!(cap.path, "/v1/chat/completions");
    assert_eq!(serde_json::from_slice::<Value>(&cap.body).unwrap(), payload);
    // Authorization preserved verbatim:
    assert_eq!(cap.headers["authorization"], "Bearer test-key-123");
    assert_eq!(cap.headers["content-type"], "application/json");
    // Hop-by-hop headers must NOT be forwarded (`host` is exempt: reqwest
    // legitimately rewrites it to the upstream authority).
    for h in [
        "connection",
        "transfer-encoding",
        "keep-alive",
        "te",
        "upgrade",
    ] {
        assert!(
            cap.headers.get(h).is_none(),
            "hop-by-hop header `{h}` was forwarded"
        );
    }
    // Client's Host must have been stripped/rewritten, not passed through:
    assert_ne!(
        cap.headers.get("host").map(|v| v.as_bytes()),
        Some(proxy_host.as_bytes()),
        "client Host leaked through"
    );
}

// ── (b) /v1/messages forward verbatim ────────────────────────────────────────
#[tokio::test]
async fn b_anthropic_messages_forward_verbatim() {
    let env = setup(Router::new()).await;
    let payload = json!({ "model": "claude-test", "max_tokens": 8, "messages": [] });
    let resp = post_json(
        &env.proxy_url,
        "/codebuddy/sp1/v1/messages",
        payload.clone(),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "msg_1");

    let cap = env.upstream_captured.lock().unwrap();
    let cap = cap.as_ref().expect("upstream received request");
    assert_eq!(cap.path, "/v1/messages");
    assert_eq!(serde_json::from_slice::<Value>(&cap.body).unwrap(), payload);
    assert_eq!(cap.headers["authorization"], "Bearer test-key-123");
}

// ── (c) /v1/responses generic subset forward verbatim ────────────────────────
#[tokio::test]
async fn c_responses_generic_subset_forward_verbatim() {
    let env = setup(Router::new()).await;
    let payload = json!({ "model": "gpt-test", "input": "hello" });
    let resp = post_json(&env.proxy_url, "/v1/responses", payload.clone()).await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"], "resp_1");

    let cap = env.upstream_captured.lock().unwrap();
    let cap = cap.as_ref().expect("upstream received request");
    assert_eq!(cap.path, "/v1/responses");
    assert_eq!(serde_json::from_slice::<Value>(&cap.body).unwrap(), payload);
}

// ── (d) upstream timeout → 504 ───────────────────────────────────────────────
#[tokio::test]
async fn d_upstream_timeout_maps_to_504() {
    let slow = Router::new().route(
        "/v1/chat/completions",
        post(|| async {
            tokio::time::sleep(Duration::from_millis(1500)).await;
            Json(json!({}))
        }),
    );
    let upstream_url = spawn(slow).await;
    // Proxy timeout (1s) < upstream delay (1.5s) → 504 GATEWAY_TIMEOUT.
    let state = state_with(&upstream_url, 1);
    let proxy_url = spawn(server::router(state)).await;
    let resp = post_json(&proxy_url, "/agent/s1/v1/chat/completions", json!({"a": 1})).await;
    assert_eq!(resp.status(), 504);
    let body: Value = resp.json().await.unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .is_some_and(|m| m.contains("timeout")));
}

// ── (e) upstream down → 502 ──────────────────────────────────────────────────
#[tokio::test]
async fn e_upstream_down_maps_to_502() {
    // Bind then immediately drop: port is closed.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_addr = listener.local_addr().unwrap();
    drop(listener);

    let state = state_with(&format!("http://{dead_addr}"), 5);
    let proxy_url = spawn(server::router(state)).await;
    let resp = post_json(&proxy_url, "/agent/s1/v1/chat/completions", json!({"a": 1})).await;
    assert_eq!(resp.status(), 502);
    let body: Value = resp.json().await.unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .is_some_and(|m| m.contains("unreachable")));
}

// ── (f) /health ──────────────────────────────────────────────────────────────
#[tokio::test]
async fn f_health_ok() {
    let env = setup(Router::new()).await;
    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/health", env.proxy_url))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");
}

// ── (g) streaming SSE passthrough without buffering ──────────────────────────
#[tokio::test]
async fn g_sse_streaming_passthrough_no_buffering() {
    // Upstream emits event 1 immediately, event 2 after 900ms.
    let sse = Router::new().route(
        "/v1/messages",
        post(|| async {
            let body = Body::from_stream(futures::stream::unfold(0u8, |n| async move {
                if n == 0 {
                    Some((
                        Ok::<bytes::Bytes, std::convert::Infallible>(bytes::Bytes::from(
                            "event: message_start\ndata: {\"type\":\"message_start\"}\n\n",
                        )),
                        1,
                    ))
                } else {
                    tokio::time::sleep(Duration::from_millis(900)).await;
                    if n == 1 {
                        Some((
                            Ok(bytes::Bytes::from(
                                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n",
                            )),
                            2,
                        ))
                    } else {
                        None
                    }
                }
            }));
            axum::http::Response::builder()
                .status(200)
                .header("content-type", "text/event-stream")
                .body(body)
                .unwrap()
        }),
    );
    let upstream_url = spawn(sse).await;
    let state = state_with(&upstream_url, 600);
    let proxy_url = spawn(server::router(state)).await;

    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{proxy_url}/claude-code/s1/v1/messages"))
        .header("accept", "text/event-stream")
        .header("x-vanta-user-key", USER_KEY)
        .body("{}")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "text/event-stream");

    let start = Instant::now();
    let mut stream = resp.bytes_stream();
    let first = stream.next().await.expect("first chunk").unwrap();
    let first_elapsed = start.elapsed();

    // If the proxy buffered the whole body, the first chunk would only arrive
    // after the 900ms gap. Generous margin keeps this deterministic.
    assert!(
        first_elapsed < Duration::from_millis(500),
        "SSE was buffered: first chunk took {first_elapsed:?}"
    );
    assert!(std::str::from_utf8(&first)
        .unwrap()
        .contains("message_start"));

    let second = stream.next().await.expect("second chunk").unwrap();
    assert!(
        std::str::from_utf8(&second)
            .unwrap()
            .contains("message_stop"),
        "chunks must arrive in order"
    );
}
