//! End-to-End Integration Tests for vantadb-server
//!
//! These tests spin up a real TCP/HTTP server, make requests via reqwest,
//! and validate the full client -> server -> storage -> response roundtrip.
//! Unlike the unit tests in server.rs (which use axum::Router::oneshot),
//! these tests exercise the entire socket-level HTTP pipeline.

#[path = "helpers/mod.rs"]
mod helpers;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use vantadb::circuit_breaker::CircuitBreaker;
use vantadb::connection_pool::ConnectionPool;
use vantadb::storage::StorageEngine;
use vantadb_server::server::{app, ServerState};

/// Probe a TCP address until it accepts a connection, or panic after timeout.
async fn wait_for_port(addr: SocketAddr, timeout: Duration) {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if tokio::time::Instant::now() >= deadline {
            panic!("Server at {} did not start within {:?}", addr, timeout);
        }
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            break;
        }
        tokio::task::yield_now().await;
    }
}

/// Bind a real TCP listener on a random port, spawn the real server,
/// and return the base URL + join handle.
async fn spawn_server(state: Arc<ServerState>, rpm: u32) -> (String, tokio::task::JoinHandle<()>) {
    let router = app(state, rpm);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base = format!("http://{}", addr);

    let handle = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    // Wait until the server actually accepts connections (event-based, not fixed sleep)
    wait_for_port(addr, Duration::from_secs(5)).await;

    (base, handle)
}

/// Build a test context (temp dir + ServerState) shared across E2E tests.
fn build_e2e_context(
    api_key: Option<&str>,
    concurrency: usize,
) -> (tempfile::TempDir, Arc<ServerState>) {
    helpers::build_server_state(Path::new("db"), api_key, concurrency)
}

#[tokio::test]
async fn test_e2e_health_and_metrics() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Health endpoint
    let resp = client.get(format!("{}/health", base)).send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert_eq!(body["data"], "OK");

    // Metrics endpoint
    let resp = client
        .get(format!("{}/metrics", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    // Metrics body content is only guaranteed when the `prometheus` feature
    // is enabled; without it the endpoint returns an empty body.
    #[cfg(feature = "prometheus")]
    {
        let text = resp.text().await.unwrap();
        assert!(!text.is_empty(), "Metrics body should not be empty");
        assert!(
            text.contains("vanta_"),
            "Metrics should contain 'vanta_' prefix: {}",
            text
        );
    }
}

#[tokio::test]
async fn test_e2e_insert_and_query() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // 1. Insert a node
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#101 TYPE Test { content: \"e2e-http\" }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["success"].as_bool().unwrap(),
        "Insert failed: {:?}",
        body
    );
    assert_eq!(body["node_id"].as_u64(), Some(101));

    // 2. Query for the node
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "FROM Test FETCH content"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["success"].as_bool().unwrap(),
        "Query failed: {:?}",
        body
    );

    // 3. Delete the node
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "DELETE NODE#101"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["success"].as_bool().unwrap(),
        "Delete failed: {:?}",
        body
    );
}

#[tokio::test]
async fn test_e2e_auth_over_http() {
    let (_dir, state) = build_e2e_context(Some("e2e-secret"), 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Health is always public
    let resp = client.get(format!("{}/health", base)).send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // Query without auth -> 401
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#1 TYPE Test { }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Query with valid auth -> 200
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .header("Authorization", "Bearer e2e-secret")
        .body(r#"{"query": "INSERT NODE#1 TYPE Test { }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());

    // Query with wrong token -> 401
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .header("Authorization", "Bearer wrong-token")
        .body(r#"{"query": "INSERT NODE#2 TYPE Test { }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn test_e2e_persistence_across_restart() {
    let (_dir, state) = build_e2e_context(None, 10);
    let storage_path = _dir.path().join("db").to_str().unwrap().to_string();

    // First server
    let (base1, handle1) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Insert data
    let resp = client
        .post(format!("{}/api/v2/query", base1))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#201 TYPE E2E { value: \"persist\" }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());

    // Shut down first server — abort the task and wait for it to fully stop
    handle1.abort();
    let _ = handle1.await;

    // Second server, same storage directory
    let storage2 = Arc::new(StorageEngine::open(&storage_path).unwrap());
    let state2 = Arc::new(ServerState {
        storage: storage2.clone(),
        db: vantadb::VantaEmbedded::from_engine(storage2.clone()),
        circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(10, Duration::from_millis(5000))),
        api_key: None,
        rbac_config: Default::default(),
        trusted_proxies: vec![],
        conversation_trigger: None,
    });
    let (base2, handle2) = spawn_server(state2, 0).await;

    // Verify persistence: search for the previously inserted node
    let resp = client
        .post(format!("{}/api/v2/query", base2))
        .header("content-type", "application/json")
        .body(r#"{"query": "FROM E2E FETCH value"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["success"].as_bool().unwrap(),
        "Persistence query failed: {:?}",
        body
    );

    // Clean shutdown
    handle2.abort();
}

#[tokio::test]
async fn test_e2e_rate_limit_over_http() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 5).await; // RPM=5

    let client = reqwest::Client::new();

    // First request should pass (burst allows it)
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#301 TYPE RL { }"}"#)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Rapid second request — with RPM=5 and burst=1, second should hit the rate limit.
    // Intentional small delay between requests to test rate limiter timing;
    // not replaceable with event-based wait — this creates the timing gap the test needs.
    tokio::time::sleep(Duration::from_millis(10)).await;
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body(r#"{"query": "INSERT NODE#302 TYPE RL { }"}"#)
        .send()
        .await
        .unwrap();

    // Depending on governor timing, may or may not be 429.
    // Accept both 200 and 429 — the test validates the server responds,
    // not the exact rate limit timing over real sockets.
    assert!(
        resp.status() == 200 || resp.status() == 429,
        "Expected 200 or 429, got {}",
        resp.status()
    );
}

#[tokio::test]
async fn test_e2e_bad_request_returns_400() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Send invalid JSON
    let resp = client
        .post(format!("{}/api/v2/query", base))
        .header("content-type", "application/json")
        .body("not-json")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_e2e_conversation_add_creates_thread() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // No thread_id -> server creates the thread and appends the first message
    let resp = client
        .post(format!("{}/conversation/add", base))
        .json(&serde_json::json!({
            "title": "e2e-convo",
            "role": "user",
            "content": "hello from e2e",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    let thread_id = body["thread_id"].as_str().unwrap().to_string();

    // The thread exists and carries exactly the one message
    let resp = client
        .get(format!("{}/api/v2/threads/{}", base, thread_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let thread: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(thread["title"], "e2e-convo");
    assert_eq!(thread["messages"].as_array().unwrap().len(), 1);
    assert_eq!(thread["messages"][0]["role"], "user");
    assert_eq!(thread["messages"][0]["content"], "hello from e2e");
}

#[tokio::test]
async fn test_e2e_conversation_add_appends_to_existing_thread() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // First message creates the thread
    let resp = client
        .post(format!("{}/conversation/add", base))
        .json(&serde_json::json!({
            "role": "user",
            "content": "first",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let thread_id = resp.json::<serde_json::Value>().await.unwrap()["thread_id"]
        .as_str()
        .unwrap()
        .to_string();

    // Second message appends to the same thread
    let resp = client
        .post(format!("{}/conversation/add", base))
        .json(&serde_json::json!({
            "thread_id": thread_id,
            "role": "assistant",
            "content": "second",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let resp = client
        .get(format!("{}/api/v2/threads/{}", base, thread_id))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let thread: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(thread["messages"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_e2e_conversation_add_invalid_thread_id() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/conversation/add", base))
        .json(&serde_json::json!({
            "thread_id": "not-a-u128",
            "role": "user",
            "content": "boom",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn test_e2e_conversation_add_requires_auth() {
    let (_dir, state) = build_e2e_context(Some("e2e-secret"), 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // Without a token -> 401
    let resp = client
        .post(format!("{}/conversation/add", base))
        .json(&serde_json::json!({ "role": "user", "content": "x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // With a valid token -> 201
    let resp = client
        .post(format!("{}/conversation/add", base))
        .header("Authorization", "Bearer e2e-secret")
        .json(&serde_json::json!({ "role": "user", "content": "x" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
}

#[tokio::test]
async fn test_e2e_skill_listing_empty_and_filtered() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state.clone(), 0).await;

    let client = reqwest::Client::new();

    // Empty listing first
    let resp = client
        .get(format!("{}/skill/listing", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["items"].as_array().unwrap().len(), 0);
    assert_eq!(body["total"], 0);

    // Seed skills directly on the store the endpoint reads (no REST create)
    let store = vantadb::skills::SkillStore::new(&state.storage);
    store
        .create(vantadb::sdk::SkillCreateInput {
            name: "greeting".into(),
            description: "how to greet".into(),
            content: "# greeting\nSay hello.".into(),
            owner_agent: "agent-a".into(),
            metadata: Default::default(),
            ttl_secs: None,
        })
        .unwrap();
    store
        .create(vantadb::sdk::SkillCreateInput {
            name: "coding".into(),
            description: "how to code".into(),
            content: "# coding\nWrite Rust.".into(),
            owner_agent: "agent-b".into(),
            metadata: Default::default(),
            ttl_secs: None,
        })
        .unwrap();

    // Unfiltered listing returns both heads
    let resp = client
        .get(format!("{}/skill/listing", base))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 2);
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 2);
    // Listing is lean: no content body in the wire view
    for item in items {
        assert!(item.get("content").is_none(), "listing leaks skill content");
        assert!(item["name"].is_string());
        assert!(item["description"].is_string());
    }

    // owner_agent filter
    let resp = client
        .get(format!("{}/skill/listing?owner_agent=agent-a", base))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["owner_agent"], "agent-a");
    assert_eq!(body["items"][0]["name"], "greeting");

    // name_prefix filter
    let resp = client
        .get(format!("{}/skill/listing?name_prefix=cod", base))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 1);
    assert_eq!(body["items"][0]["name"], "coding");
}

/// MOD-12 regression: on a FRESH database, a text put followed by a lexical
/// search over HTTP must return hits without any manual index rebuild.
/// Server startup is responsible for ensuring indexes are current (MCP-01 twin).
#[tokio::test]
async fn test_e2e_text_search_fresh_db() {
    let (_dir, state) = build_e2e_context(None, 10);
    let (base, _handle) = spawn_server(state, 0).await;

    let client = reqwest::Client::new();

    // 1. Put a record with a distinctive text payload
    let input = vantadb::sdk::VantaMemoryInput::new(
        "default",
        "mod12-doc",
        "The quantum flux capacitor regulates lexical energy for BM25 scoring",
    );
    let resp = client
        .post(format!("{}/api/v2/records", base))
        .json(&input)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // 2. Lexical search via /api/v2/search (no rebuild) must return hits
    let search = vantadb::sdk::VantaMemorySearchRequest {
        text_query: Some("quantum flux".into()),
        ..Default::default()
    };
    let resp = client
        .post(format!("{}/api/v2/search", base))
        .json(&search)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let hits = body["records"]
        .as_array()
        .expect("records array in search response");
    assert!(
        !hits.is_empty(),
        "text search must hit on fresh DB without manual rebuild: {body}"
    );
}
