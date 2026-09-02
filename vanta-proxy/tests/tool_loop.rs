// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MEM-51 contract tests (D19 a–e): the O2 agentic memory-tool loop against
//! an upstream mock that scripts tool_use SSE responses.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::Response;
use axum::routing::post;
use axum::Router;
use futures::StreamExt;
use serde_json::{json, Value};
use vanta_proxy::config::{
    AuthConfig, MemCommandConfig, ProxyConfig, ServerConfig, UpstreamConfig,
};
use vanta_proxy::server;

const USER_KEY: &str = "sk-loop-test";

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
        .entity_set("default", "user", "usr-loop", fields)
        .unwrap();
    Arc::new(engine)
}

async fn spawn(router: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    format!("http://{addr}")
}

type Bodies = Arc<Mutex<Vec<Vec<u8>>>>;

struct Env {
    proxy_url: String,
    /// Raw request bodies captured per upstream forward, in order.
    bodies: Bodies,
    /// Engine backing the proxy state (capture assertions).
    engine: Arc<vantadb::storage::StorageEngine>,
}

/// Scripted upstream + real proxy wired to it: the n-th POST to `path`
/// returns `responses[n]` as an SSE body.
async fn setup(path: &'static str, responses: Vec<String>) -> Env {
    let bodies: Bodies = Arc::new(Mutex::new(Vec::new()));
    let hits = Arc::new(AtomicUsize::new(0));
    let b2 = bodies.clone();
    let h2 = hits.clone();
    let r2 = Arc::new(Mutex::new(responses));
    let app = Router::new().route(
        path,
        post(move |body: bytes::Bytes| {
            let bodies = b2.clone();
            let hits = h2.clone();
            let responses = r2.clone();
            async move {
                bodies.lock().unwrap().push(body.to_vec());
                let idx = hits.fetch_add(1, Ordering::SeqCst);
                let payload = responses
                    .lock()
                    .unwrap()
                    .get(idx)
                    .cloned()
                    .unwrap_or_default();
                Response::builder()
                    .status(200)
                    .header("content-type", "text/event-stream")
                    .body(Body::from(payload))
                    .unwrap()
            }
        }),
    );
    let upstream_url = spawn(app).await;
    let engine = seeded_engine();
    let cfg = ProxyConfig {
        report: Default::default(),
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream_url,
            api_key: String::new(),
            forward_timeout_secs: 600,
        },
        auth: AuthConfig::default(),
        mem_command: MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig {
            persist_path: String::new(),
        },
    };
    let proxy_url = spawn(server::router(
        server::AppState::from_engine(cfg, engine.clone()).unwrap(),
    ))
    .await;
    Env {
        proxy_url,
        bodies,
        engine,
    }
}

async fn post_json(proxy_url: &str, path: &str, body: Value) -> reqwest::Response {
    reqwest::Client::new()
        .post(format!("{proxy_url}{path}"))
        .header("content-type", "application/json")
        .header("x-vanta-user-key", USER_KEY)
        .header("x-claude-code-session-id", "sess-loop")
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// OpenAI chat-completions SSE carrying one tool call with `args`.
fn openai_tool_call_sse(name: &str, args: Value) -> String {
    let args_str = args.to_string();
    format!(
        "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        json!({"choices":[{"delta":{"role":"assistant","tool_calls":[
            {"index":0,"id":"call_1","type":"function","function":{"name":name,"arguments":""}}
        ]}}]}),
        json!({"choices":[{"delta":{"tool_calls":[
            {"index":0,"function":{"arguments":args_str}}
        ]}}]})
    )
}

fn openai_final_sse(text: &str) -> String {
    format!(
        "data: {}\ndata: [DONE]\n\n",
        json!({"choices":[{"delta":{"content":text}}]})
    )
}

/// Anthropic messages SSE carrying one tool_use block with `input`.
fn anthropic_tool_use_sse(name: &str, input: Value) -> String {
    format!(
        "data: {}\n\ndata: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        json!({"type":"message_start"}),
        json!({"type":"content_block_start","index":0,"content_block":{
            "type":"tool_use","id":"toolu_1","name":name}}),
        json!({"type":"content_block_delta","index":0,"delta":{
            "type":"input_json_delta","partial_json":input.to_string()}})
    )
}

fn anthropic_final_sse(text: &str) -> String {
    format!(
        "data: {}\n\ndata: [DONE]\n\n",
        json!({"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":text}})
    )
}

const OPENAI_PATH: &str = "/cc/s1/v1/chat/completions";
const ANTHROPIC_PATH: &str = "/cc/s1/v1/messages";

// ── (a) OpenAI capture loop ──────────────────────────────────────────────────
#[tokio::test]
async fn a_openai_capture_executes_server_side_and_loops_to_final() {
    let env = setup(
        "/v1/chat/completions",
        vec![
            openai_tool_call_sse("vanta_memory_capture", json!({"text": "remember this"})),
            openai_final_sse("done"),
        ],
    )
    .await;

    let resp = post_json(
        &env.proxy_url,
        OPENAI_PATH,
        json!({"model":"gpt-test","messages":[{"role":"user","content":"hi"}]}),
    )
    .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.headers()["content-type"], "text/event-stream");
    let streamed = resp.text().await.unwrap();
    assert!(streamed.contains("\"done\""), "final streamed: {streamed}");

    // Exactly two upstream forwards; second carries assistant + tool result.
    assert_eq!(env.bodies.lock().unwrap().len(), 2);
    let second: Value = serde_json::from_slice(&env.bodies.lock().unwrap()[1]).unwrap();
    let messages = second["messages"].as_array().unwrap();
    let assistant = messages.iter().find(|m| m["role"] == "assistant").unwrap();
    assert_eq!(
        assistant["tool_calls"][0]["function"]["name"],
        "vanta_memory_capture"
    );
    let tool_msg = messages.iter().find(|m| m["role"] == "tool").unwrap();
    assert_eq!(tool_msg["tool_call_id"], "call_1");

    // Capture executed through WriteBack → record lands in proxy-turns.
    let db = vantadb::sdk::VantaEmbedded::from_engine(env.engine.clone());
    let mut persisted = false;
    for _ in 0..100 {
        if vanta_proxy::capture::list_turns(&db)
            .iter()
            .any(|r| r.payload.contains("remember this"))
        {
            persisted = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(persisted, "captured text persisted via D47 write path");
}

// ── (b) Anthropic search loop with synchronous recall ───────────────────────
#[tokio::test]
async fn b_anthropic_search_executes_synchronous_recall() {
    let env = setup(
        "/v1/messages",
        vec![
            anthropic_tool_use_sse("vanta_memory_search", json!({"query": "coffee prefs"})),
            anthropic_final_sse("answered"),
        ],
    )
    .await;

    let resp = post_json(
        &env.proxy_url,
        ANTHROPIC_PATH,
        json!({"model":"claude-test","max_tokens":16,"messages":[{"role":"user","content":"hi"}]}),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let streamed = resp.text().await.unwrap();
    assert!(
        streamed.contains("\"answered\""),
        "final streamed: {streamed}"
    );

    // Exactly two upstream forwards; second carries assistant + tool_result.
    assert_eq!(env.bodies.lock().unwrap().len(), 2);
    let second: Value = serde_json::from_slice(&env.bodies.lock().unwrap()[1]).unwrap();
    let messages = second["messages"].as_array().unwrap();
    // Standard Anthropic shape: assistant tool_use block + user tool_result.
    let assistant = messages.iter().find(|m| m["role"] == "assistant").unwrap();
    assert_eq!(assistant["content"][0]["type"], "tool_use");
    assert_eq!(assistant["content"][0]["input"]["query"], "coffee prefs");
    let user = messages.iter().rev().find(|m| m["role"] == "user").unwrap();
    assert_eq!(user["content"][0]["type"], "tool_result");
    assert_eq!(user["content"][0]["tool_use_id"], "toolu_1");
    // Empty store → synchronous recall ran and reported no memories.
    assert_eq!(user["content"][0]["content"], "No relevant memories found.");
}

// ── (c) request without session → byte-identical passthrough ────────────────
#[tokio::test]
async fn c_without_our_tools_passthrough_is_byte_identical() {
    let sse_payload =
        "event: message_start\ndata: {\"type\":\"message_start\"}\n\ndata: [DONE]\n\n";
    // Upstream WITHOUT our tools announced: plain non-JSON body → no inject,
    // no interceptor → raw streaming passthrough of the exact bytes.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let payload = Arc::new(sse_payload.to_string());
    tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route(
                "/v1/chat/completions",
                post(move || {
                    let body = Body::from((*payload).clone());
                    async move {
                        Response::builder()
                            .status(200)
                            .header("content-type", "text/event-stream")
                            .body(body)
                            .unwrap()
                    }
                }),
            ),
        )
        .await
        .unwrap()
    });
    let cfg = ProxyConfig {
        report: Default::default(),
        upstream: UpstreamConfig {
            url: format!("http://{addr}"),
            ..UpstreamConfig::default()
        },
        ..ProxyConfig::default()
    };
    let proxy_url = spawn(server::router(
        server::AppState::from_engine(cfg, seeded_engine()).unwrap(),
    ))
    .await;

    // No session header → verbatim path; body without our tools → same gate.
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{proxy_url}{OPENAI_PATH}"))
        .header("authorization", "Bearer k")
        .header("x-vanta-user-key", USER_KEY)
        .body("{\"messages\":[]}")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap();
    assert_eq!(bytes.as_ref(), sse_payload.as_bytes(), "byte-identical");
}

// ── (d) iteration cap D48 → cut loop, stream last response ──────────────────
#[tokio::test]
async fn d_iteration_cap_cuts_loop_and_streams_last_response() {
    // Always answers with another memory-tool call → cap must stop the loop.
    let always_tool = openai_tool_call_sse("vanta_memory_search", json!({"query": "q"}));
    let env = setup("/v1/chat/completions", vec![always_tool.clone(); 5]).await;

    let resp = post_json(&env.proxy_url, OPENAI_PATH, json!({"messages":[]})).await;
    assert_eq!(resp.status(), 200);
    // Initial forward + 3 tool executions = 4 forwards; the 4th response is
    // handed back verbatim even though it still carries tool_use.
    assert_eq!(env.bodies.lock().unwrap().len(), 4);
    let streamed = resp.text().await.unwrap();
    assert_eq!(streamed, always_tool, "last response replayed verbatim");
}

// ── (e) final response streams intact ───────────────────────────────────────
#[tokio::test]
async fn e_final_response_streaming_intact() {
    let final_body = format!(
        "data: {}\n\ndata: {}\n\ndata: [DONE]\n\n",
        json!({"choices":[{"delta":{"content":"par"}}]}),
        json!({"choices":[{"delta":{"content":"tial"}}]}),
    );
    let env = setup(
        "/v1/chat/completions",
        vec![
            openai_tool_call_sse("vanta_memory_capture", json!({"text": "note"})),
            final_body.clone(),
        ],
    )
    .await;

    let resp = post_json(&env.proxy_url, OPENAI_PATH, json!({"messages":[]})).await;
    assert_eq!(resp.headers()["content-type"], "text/event-stream");
    let mut stream = resp.bytes_stream();
    let mut collected = Vec::new();
    while let Some(chunk) = stream.next().await {
        collected.extend_from_slice(&chunk.unwrap());
    }
    assert_eq!(
        String::from_utf8(collected).unwrap(),
        final_body,
        "final SSE body intact, chunk order preserved"
    );
}
