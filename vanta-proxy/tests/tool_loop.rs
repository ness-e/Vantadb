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
    let config = vantadb::config::Config {
        backend_kind: vantadb::storage::BackendKind::InMemory,
        read_only: false,
        ..vantadb::config::Config::default()
    };
    let engine =
        vantadb::storage::StorageEngine::open_with_config(":memory:", Some(config)).unwrap();
    let mut fields = std::collections::HashMap::new();
    fields.insert(
        "user_key".to_string(),
        vantadb::node::FieldValue::String(USER_KEY.to_string()),
    );
    vantadb::entity::EntityStore::new(&engine)
        .set(vantadb::entity::EntityWrite {
            namespace: "default",
            collection: "user",
            id: "usr-loop",
            fields,
        })
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
        cost: Default::default(),
        server: ServerConfig::default(),
        upstream: UpstreamConfig {
            url: upstream_url,
            api_key: String::new(),
            forward_timeout_secs: 600,
            models: Vec::new(),
        },
        upstreams: Vec::new(),
        auth: AuthConfig::default(),
        mem_command: MemCommandConfig::default(),
        writeback: vanta_proxy::config::WritebackConfig {
            persist_path: String::new(),
        },
        cache: Default::default(),
        routing: Default::default(),
        redact: Default::default(),
        context: Default::default(),
        guardrails: Default::default(),
        translate: Default::default(),
        injection: Default::default(),
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
    let db = vantadb::sdk::Embedded::from_engine(env.engine.clone());
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
        cost: Default::default(),
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

// ── WIRE-01 (f) loop e2e: sesión-1 captura → sesión-2 search con hits ───────
// El turno de sesión-1 persiste en `l1/{session}` (dual-write del job L0);
// cuando el modelo invoca `vanta_memory_search` en sesión-2, el recall
// sincrónico devuelve el turno (hits, no "No relevant memories found").
// Sin el dual-write este test da 0 hits (loop inerte).
#[tokio::test]
async fn loop_e2e_capture_then_search_hits() {
    let phrase = "xylophone-quasar-7429 prefers concise answers";
    let env = setup(
        "/v1/chat/completions",
        vec![
            openai_final_sse("ack one"),
            openai_tool_call_sse(
                "vanta_memory_search",
                json!({"query": "recall xylophone concise notes"}),
            ),
            openai_final_sse("answered"),
        ],
    )
    .await;

    // Sesión-1: request normal con session header → captura L0 (async).
    let resp = post_json(
        &env.proxy_url,
        OPENAI_PATH,
        json!({"model":"gpt-test","messages":[{"role":"user","content": phrase}]}),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Esperar el dual-write L1 (WriteBack fire-and-forget — poll, no sleep fijo).
    let db = vantadb::sdk::Embedded::from_engine(env.engine.clone());
    let mut landed = false;
    for _ in 0..100 {
        let records = vanta_memory::core::record::l1_reader::read_session_records(&db, "sess-loop")
            .unwrap_or_default();
        if records.iter().any(|r| r.content.contains(phrase)) {
            landed = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(landed, "session-1 turn must land in l1 before session-2");

    // Sesión-2 (misma sesión): el modelo invoca search → hits con el turno.
    let resp = post_json(
        &env.proxy_url,
        OPENAI_PATH,
        json!({"model":"gpt-test","messages":[{"role":"user","content":"what do you recall?"}]}),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Tres forwards: s1 + s2 round-1 (tool) + s2 round-2 (con el resultado).
    assert_eq!(env.bodies.lock().unwrap().len(), 3);
    let rerequest = env.bodies.lock().unwrap()[2].clone();
    let second: Value = serde_json::from_slice(&rerequest).unwrap();
    let tool_msg = second["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "tool")
        .expect("re-request carries the tool result");
    let content = tool_msg["content"].as_str().unwrap_or("");
    assert!(
        content.contains("xylophone-quasar-7429"),
        "search must hit the captured turn, got: {content}"
    );
    assert!(
        !content.contains("No relevant memories found"),
        "search must not report empty: {content}"
    );
}

// ── WIRE-01 (f2) boundary LLM08: el search no mezcla sesiones ───────────────
// Un turno de sess-A es invisible para el search de sess-B (records sin
// tenant quedan session-only bajo el scope Agent default).
#[tokio::test]
async fn loop_e2e_search_stays_session_scoped() {
    async fn post_session(proxy_url: &str, session: &str, body: Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{proxy_url}{OPENAI_PATH}"))
            .header("content-type", "application/json")
            .header("x-vanta-user-key", USER_KEY)
            .header("x-claude-code-session-id", session)
            .json(&body)
            .send()
            .await
            .unwrap()
    }

    let phrase = "xylophone-quasar-7429 stays in session A";
    let env = setup(
        "/v1/chat/completions",
        vec![
            openai_final_sse("ack A"),
            openai_tool_call_sse(
                "vanta_memory_search",
                json!({"query": "recall xylophone concise notes"}),
            ),
            openai_final_sse("answered B"),
        ],
    )
    .await;

    let resp = post_session(
        &env.proxy_url,
        "sess-A",
        json!({"model":"gpt-test","messages":[{"role":"user","content": phrase}]}),
    )
    .await;
    assert_eq!(resp.status(), 200);

    let db = vantadb::sdk::Embedded::from_engine(env.engine.clone());
    let mut landed = false;
    for _ in 0..100 {
        let records = vanta_memory::core::record::l1_reader::read_session_records(&db, "sess-A")
            .unwrap_or_default();
        if records.iter().any(|r| r.content.contains(phrase)) {
            landed = true;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert!(landed, "sess-A turn must land in l1");

    // Sess-B busca lo mismo → sin hits (vacío, no filtración de A).
    let resp = post_session(
        &env.proxy_url,
        "sess-B",
        json!({"model":"gpt-test","messages":[{"role":"user","content":"what do you recall?"}]}),
    )
    .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(env.bodies.lock().unwrap().len(), 3);
    let rerequest = env.bodies.lock().unwrap()[2].clone();
    let second: Value = serde_json::from_slice(&rerequest).unwrap();
    let tool_msg = second["messages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["role"] == "tool")
        .expect("re-request carries the tool result");
    assert_eq!(
        tool_msg["content"].as_str().unwrap_or(""),
        "No relevant memories found.",
        "sess-B must never see sess-A turns"
    );
}
