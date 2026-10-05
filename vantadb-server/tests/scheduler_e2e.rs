// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! WIRE-16 — end-to-end tests for the `vantadb-server` scheduler wiring
//! (ADR-0054 T3): `POST /api/v2/conversations` → L0 capture + L1 task → the
//! memory scheduler loop (fake runner) → `l1/<thread_id>`; restart keeps L0/L1
//! with an ephemeral queue; `interval = 0` disables the loop (bridge stays);
//! a factory without a runner skips passes (P4 — nothing lost, nothing burns).
//!
//! Test (1) drives the real `run_with_hooks` bootstrap — the only call in this
//! binary, because `init_telemetry` uses `.init()` (once per process). Tests
//! (2)-(4) assemble the same production pieces (`wire_memory` + `app`)
//! directly, mirroring `vantadb-server/tests/e2e.rs`.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use vanta_memory::core::abstractions::{LlmError, LlmRunParams, LlmRunner};
use vanta_memory::core::conversation::L0Recorder;
use vanta_memory::core::record::read_session_records;
use vantadb::circuit_breaker::CircuitBreaker;
use vantadb::cli_server::{ConversationTrigger, ServerHooks};
use vantadb::config::Config;
use vantadb::connection_pool::ConnectionPool;
use vantadb::sdk::Embedded;
use vantadb::storage::StorageEngine;
use vantadb_server::scheduler::wire_memory;
use vantadb_server::server::{app, ServerState};

/// L1 extraction response shape per `vanta-memory/tests/conversation_hook.rs`.
const EXTRACTION_JSON: &str = r#"[
  {"scene_name": "UI Preferences", "message_ids": ["m1"], "memories": [
    {"content": "User prefers dark mode", "type": "preference", "priority": 80, "source_message_ids": ["m1"]}
  ]}
]"#;

/// Fake runner covering the two LLM calls an L1 pass makes (pattern proven in
/// `vanta-memory/tests/conversation_hook.rs`).
struct ScriptedRunner;

impl LlmRunner for ScriptedRunner {
    fn run(&self, params: &LlmRunParams) -> Result<String, LlmError> {
        match params.task_id.as_str() {
            "l1-extraction" => Ok(EXTRACTION_JSON.to_string()),
            "l1-conflict-detection" => {
                let judged = params
                    .prompt
                    .split("NEW MEMORIES TO JUDGE")
                    .nth(1)
                    .unwrap_or(&params.prompt);
                let record_id = judged
                    .split("\"record_id\": \"")
                    .nth(1)
                    .and_then(|rest| rest.split('"').next())
                    .unwrap_or("m_0");
                Ok(format!(
                    r#"[{{"record_id": "{record_id}", "action": "store"}}]"#
                ))
            }
            other => panic!("unexpected LLM call: {other}"),
        }
    }
}

/// Pick a free loopback port (the bootstrap binds it itself).
fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("probe port");
    listener.local_addr().expect("probe addr").port()
}

/// Probe a TCP address until it accepts a connection, or panic after timeout.
async fn wait_for_port(addr: SocketAddr, timeout: Duration) {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if tokio::time::Instant::now() >= deadline {
            panic!("Server at {addr} did not start within {timeout:?}");
        }
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            break;
        }
        tokio::task::yield_now().await;
    }
}

/// Bounded wait for an observable condition (no unbounded sleeps).
async fn wait_until(mut cond: impl FnMut() -> bool, what: &str) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    while !cond() {
        assert!(
            std::time::Instant::now() < deadline,
            "timed out waiting for {what}"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

/// Open the DB the way the production bootstrap does (writer lock, indexes).
fn open_storage(dir: &Path) -> (Arc<StorageEngine>, Embedded) {
    let path = dir.join("db");
    let storage =
        Arc::new(StorageEngine::open(path.to_str().expect("utf8 path")).expect("open storage"));
    let db = Embedded::from_engine(storage.clone());
    db.ensure_indexes_current().expect("ensure indexes");
    (storage, db)
}

/// Reopen after a "restart": the writer lock may take a moment to release
/// while a dropped task finishes unwinding.
async fn open_storage_with_retry(dir: &Path) -> (Arc<StorageEngine>, Embedded) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let path = dir.join("db");
        match StorageEngine::open(path.to_str().expect("utf8 path")) {
            Ok(storage) => {
                let storage = Arc::new(storage);
                let db = Embedded::from_engine(storage.clone());
                db.ensure_indexes_current().expect("ensure indexes");
                return (storage, db);
            }
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
            Err(e) => panic!("reopen storage after restart: {e}"),
        }
    }
}

/// Manual assembly mirroring `run_with_hooks`'s state build (telemetry
/// `.init()` is once-per-process, so only test (1) goes through the real
/// bootstrap).
fn build_state(
    storage: Arc<StorageEngine>,
    db: Embedded,
    trigger: Option<Arc<dyn ConversationTrigger>>,
) -> Arc<ServerState> {
    Arc::new(ServerState {
        storage,
        db,
        circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
        pool: Arc::new(ConnectionPool::new(10, Duration::from_millis(5000))),
        api_key: None,
        alt_api_key: None,
        jwt_secret: None,
        rbac_config: Default::default(),
        trusted_proxies: vec![],
        conversation_trigger: trigger,
    })
}

/// Spawn a real TCP server for a manually assembled state.
async fn spawn_server(state: Arc<ServerState>) -> (String, tokio::task::JoinHandle<()>) {
    let router = app(state, 0);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let base = format!("http://{addr}");
    let handle = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("serve");
    });
    wait_for_port(addr, Duration::from_secs(5)).await;
    (base, handle)
}

/// POST one conversation turn and return the thread id (decimal string).
async fn post_turn(
    client: &reqwest::Client,
    base: &str,
    thread_id: Option<&str>,
    content: &str,
) -> String {
    let mut body = serde_json::json!({ "role": "user", "content": content });
    if let Some(id) = thread_id {
        body["thread_id"] = serde_json::json!(id);
    }
    let resp = client
        .post(format!("{base}/api/v2/conversations"))
        .json(&body)
        .send()
        .await
        .expect("post conversation");
    assert_eq!(resp.status(), 201, "conversation add must succeed");
    let parsed: serde_json::Value = resp.json().await.expect("json");
    parsed["thread_id"].as_str().expect("thread_id").to_string()
}

// ──────────────────── (1) full path through the real bootstrap ────────────────────

#[tokio::test]
async fn e2e_capture_flows_to_l1_via_run_with_hooks() {
    let dir = tempfile::tempdir().expect("tempdir");
    let port = free_port();
    let config = Config {
        storage_path: dir.path().join("db").to_string_lossy().into_owned(),
        host: "127.0.0.1".to_string(),
        port,
        ..Default::default()
    };

    let (db_tx, db_rx) = tokio::sync::oneshot::channel::<Embedded>();
    let mut hooks = ServerHooks::default();
    hooks.on_storage_ready = Some(Box::new(move |hooks: &mut ServerHooks, db: Embedded| {
        let _ = db_tx.send(db.clone());
        wire_memory(hooks, db, 20, || Some(ScriptedRunner));
    }));

    let handle = tokio::spawn(async move {
        let _ = vantadb::cli_server::run_with_hooks(config, hooks).await;
    });
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().expect("addr");
    wait_for_port(addr, Duration::from_secs(10)).await;

    let db = db_rx
        .await
        .expect("run_with_hooks must share its db via on_storage_ready");

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}");
    let thread_id = post_turn(&client, &base, None, "I prefer dark mode").await;

    // Verification (1): L0 captured + the scheduler loop processed the queued
    // L1 task into l1/<thread_id>.
    wait_until(
        || {
            read_session_records(&db, &thread_id)
                .map(|r| !r.is_empty())
                .unwrap_or(false)
        },
        "l1 memories written by the scheduler loop",
    )
    .await;
    let records = read_session_records(&db, &thread_id).expect("read l1");
    assert!(
        records.iter().any(|r| r.content.contains("dark mode")),
        "expected extracted memory in l1/{thread_id}, got: {records:?}"
    );
    let l0 = L0Recorder::new(db.clone())
        .read_messages(&thread_id)
        .expect("read l0");
    assert_eq!(l0.len(), 1, "L0 capture must survive (LLM-free)");

    handle.abort();
    let _ = handle.await;
}

// ──────────────────── (2) restart: L0/L1 persist, queue is ephemeral ────────────────────

#[tokio::test]
async fn e2e_restart_persists_l0_and_queue_is_ephemeral() {
    let dir = tempfile::tempdir().expect("tempdir");
    let client = reqwest::Client::new();

    // ── instance A: production pieces, manual assembly ──
    let (storage_a, db_a) = open_storage(dir.path());
    let mut hooks_a = ServerHooks::default();
    let queue_a = wire_memory(&mut hooks_a, db_a.clone(), 20, || Some(ScriptedRunner));
    assert_eq!(queue_a.queue_depth(), (0, 0), "fresh queue is empty");
    let state_a = build_state(
        storage_a.clone(),
        db_a.clone(),
        hooks_a.conversation_trigger.take(),
    );
    let (base_a, handle_a) = spawn_server(state_a).await;

    let thread_id = post_turn(&client, &base_a, None, "I prefer dark mode").await;
    wait_until(
        || {
            read_session_records(&db_a, &thread_id)
                .map(|r| !r.is_empty())
                .unwrap_or(false)
        },
        "instance A processed the capture",
    )
    .await;

    // ── stop A: abort + graceful join + drop every handle (lock released) ──
    handle_a.abort();
    let _ = handle_a.await;
    for service in hooks_a.background_services.drain(..) {
        service.shutdown().await;
    }
    drop(db_a);
    drop(storage_a);

    // ── restart: reopen the same path; persisted L0/L1 survive ──
    let (storage_b, db_b) = open_storage_with_retry(dir.path()).await;
    let l0 = L0Recorder::new(db_b.clone())
        .read_messages(&thread_id)
        .expect("read l0");
    assert_eq!(l0.len(), 1, "L0 must survive restart");
    assert!(
        !read_session_records(&db_b, &thread_id)
            .expect("read l1")
            .is_empty(),
        "L1 must survive restart"
    );

    // ── instance B: fresh (ephemeral) queue; a new capture flows again ──
    let mut hooks_b = ServerHooks::default();
    let queue_b = wire_memory(&mut hooks_b, db_b.clone(), 20, || Some(ScriptedRunner));
    assert_eq!(
        queue_b.queue_depth(),
        (0, 0),
        "queue is ephemeral: a restart starts empty"
    );
    let state_b = build_state(
        storage_b.clone(),
        db_b.clone(),
        hooks_b.conversation_trigger.take(),
    );
    let (base_b, handle_b) = spawn_server(state_b).await;

    post_turn(
        &client,
        &base_b,
        Some(&thread_id),
        "second turn after restart",
    )
    .await;
    wait_until(
        || {
            queue_b.queue_depth() == (0, 0)
                && L0Recorder::new(db_b.clone())
                    .read_messages(&thread_id)
                    .map(|m| m.len() >= 2)
                    .unwrap_or(false)
        },
        "instance B drained the new capture",
    )
    .await;
    let l0_b = L0Recorder::new(db_b.clone())
        .read_messages(&thread_id)
        .expect("read l0 b");
    assert_eq!(l0_b.len(), 2, "both turns persisted in L0 across restart");

    handle_b.abort();
    let _ = handle_b.await;
    for service in hooks_b.background_services.drain(..) {
        service.shutdown().await;
    }
}

// ──────────────────── (3) disabled: no loop, bridge stays ────────────────────

#[tokio::test]
async fn e2e_disabled_interval_keeps_bridge_and_spawns_no_loop() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (storage, db) = open_storage(dir.path());
    let mut hooks = ServerHooks::default();
    let queue = wire_memory(&mut hooks, db.clone(), 0, || Some(ScriptedRunner));
    assert!(
        hooks.background_services.is_empty(),
        "interval 0 must not spawn the loop"
    );
    assert!(
        hooks.conversation_trigger.is_some(),
        "bridge stays wired (L0 is never lost)"
    );

    let state = build_state(storage, db.clone(), hooks.conversation_trigger.take());
    let (base, handle) = spawn_server(state).await;
    let client = reqwest::Client::new();
    let thread_id = post_turn(&client, &base, None, "I prefer dark mode").await;

    wait_until(
        || {
            L0Recorder::new(db.clone())
                .read_messages(&thread_id)
                .map(|m| m.len() == 1)
                .unwrap_or(false)
        },
        "L0 capture",
    )
    .await;
    // Negative assertion: with no loop, the L1 task stays queued and no L1 is
    // written. Two snapshots separated by a bounded quiet window (longer than
    // several would-be 20ms ticks) prove nothing consumed it.
    let snapshot = queue.queue_depth();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        queue.queue_depth(),
        snapshot,
        "queue must not be consumed without a loop"
    );
    assert_eq!(queue.queue_depth(), (0, 1), "the L1 task stays queued");
    assert!(
        read_session_records(&db, &thread_id)
            .expect("read l1")
            .is_empty(),
        "no loop → no l1"
    );

    handle.abort();
    let _ = handle.await;
}

// ──────────────────── (4) without runner: skip, never burn ────────────────────

#[tokio::test]
async fn e2e_without_runner_skips_pass_and_keeps_queue() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (storage, db) = open_storage(dir.path());
    let mut hooks = ServerHooks::default();
    // Same policy as production: no real engine configured → `None` (skip).
    let queue = wire_memory(&mut hooks, db.clone(), 20, || None::<ScriptedRunner>);
    assert_eq!(
        hooks.background_services.len(),
        1,
        "the loop is spawned; the PASS is what skips"
    );

    let state = build_state(storage, db.clone(), hooks.conversation_trigger.take());
    let (base, handle) = spawn_server(state).await;
    let client = reqwest::Client::new();
    let thread_id = post_turn(&client, &base, None, "I prefer dark mode").await;

    wait_until(
        || {
            L0Recorder::new(db.clone())
                .read_messages(&thread_id)
                .map(|m| m.len() == 1)
                .unwrap_or(false)
        },
        "L0 capture",
    )
    .await;
    // Negative assertion: passes run every 20ms but skip (factory `None`), so
    // the task must survive queued and nothing must be claimed/dead-lettered.
    let snapshot = queue.queue_depth();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        queue.queue_depth(),
        snapshot,
        "skipping passes must not consume the task"
    );
    assert_eq!(queue.queue_depth(), (0, 1), "task stays queued (P4)");
    assert_eq!(queue.pending_count(), 0, "nothing claimed/retried");
    assert!(
        read_session_records(&db, &thread_id)
            .expect("read l1")
            .is_empty(),
        "skipped passes write no l1"
    );

    handle.abort();
    let _ = handle.await;
    for service in hooks.background_services.drain(..) {
        service.shutdown().await;
    }
}
