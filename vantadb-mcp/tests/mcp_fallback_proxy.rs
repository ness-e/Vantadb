//! MCP-35 fallback proxy E2E: 2× server same DB
//!
//! Verifies the full flow from the task contract:
//! - writer writes `.vanta.server.json` with pid/port/version
//! - health 200
//! - Drop removes file when pid==self
//! - proxy detects DatabaseBusy via file + health 500ms + PID alive
//! - proxy `memory_put` via HTTP is visible in writer storage
//! - stale cleanup when PID dead + retry open

use serde_json::json;
use std::sync::Arc;
use vantadb::storage::StorageEngine;
use vantadb_mcp::{cleanup_stale, discovery_path, is_pid_alive, try_proxy, ServerInfo};

#[tokio::test]
async fn writer_writes_discovery_and_health() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap().to_string();
    let storage = Arc::new(StorageEngine::open(&path).expect("writer open"));
    let (guard, port) = vantadb_mcp::spawn_writer_http(storage.clone())
        .await
        .expect("spawn_writer_http");
    // file exists
    let disc = discovery_path(&path);
    assert!(disc.exists(), "discovery file must exist after writer open");
    let content = std::fs::read_to_string(&disc).unwrap();
    let info: ServerInfo = serde_json::from_str(&content).unwrap();
    assert_eq!(info.pid, std::process::id());
    assert_eq!(info.http_port, port);
    assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    // health 200
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .build()
        .unwrap();
    let resp = client
        .get(format!("http://127.0.0.1:{}/api/v2/health", port))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "health must be 200");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body.get("status").and_then(|v| v.as_str()), Some("healthy"));
    // Drop must remove if pid==self
    drop(guard);
    // WriterGuard Drop is sync; give fs a moment
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    assert!(
        !disc.exists(),
        "discovery file must be removed on Drop when pid==self"
    );
}

#[tokio::test]
async fn proxy_put_visible_in_writer() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap().to_string();
    let storage = Arc::new(StorageEngine::open(&path).expect("writer open"));
    let (guard, port) = vantadb_mcp::spawn_writer_http(storage.clone())
        .await
        .expect("spawn");
    // Establish proxy handle via try_proxy (uses 500ms health + PID alive)
    let (handle, info) = try_proxy(&path, None).await.expect("proxy should connect");
    assert_eq!(info.http_port, port);
    assert_eq!(handle.base_url, format!("http://127.0.0.1:{}", port));
    // Proxy tools/call memory_put
    let params = Some(
        json!({"name":"memory_put","arguments":{"namespace":"ns1","key":"k1","payload":"hello proxy"}}),
    );
    let _res = handle
        .proxy_tools_call(&params)
        .await
        .expect("proxy put via http");
    // Check writer storage sees it via VantaEmbedded (high-level API)
    let db = vantadb::VantaEmbedded::from_engine(storage.clone());
    let list = db
        .list(
            "ns1",
            vantadb::sdk::VantaMemoryListOptions {
                limit: 100,
                ..Default::default()
            },
        )
        .expect("list after proxy put");
    assert!(
        !list.records.is_empty(),
        "writer must see proxy's put, list empty"
    );
    assert!(
        list.records.iter().any(|r| r.key == "k1"),
        "k1 must be in list"
    );
    drop(guard);
}

#[tokio::test]
async fn stale_cleanup_allows_retry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_str().unwrap().to_string();
    // Write stale file with dead PID
    let stale = vantadb_mcp::ServerInfo {
        pid: 999_999,
        http_port: 12345,
        started_at: 0,
        version: "0.5.0".into(),
    };
    let disc = discovery_path(&path);
    std::fs::write(&disc, serde_json::to_string_pretty(&stale).unwrap()).unwrap();
    assert!(disc.exists());
    // is_pid_alive must be false for dead
    assert!(!is_pid_alive(999_999));
    // try_proxy should fail (dead PID)
    let res = try_proxy(&path, None).await;
    assert!(res.is_none(), "dead PID should not proxy");
    // cleanup stale
    cleanup_stale(&path);
    assert!(!disc.exists(), "stale file must be removed");
    // Now open should succeed as writer (no contention)
    let storage = StorageEngine::open(&path).expect("open after stale cleanup");
    // storage.data_dir should be <path>/data
    assert!(storage.data_dir.ends_with("data"));
    // New writer should write fresh file
    let storage = Arc::new(storage);
    let (guard, _port) = vantadb_mcp::spawn_writer_http(storage.clone())
        .await
        .expect("new writer");
    assert!(disc.exists());
    drop(guard);
}
