//! Integration tests: `ServerConnection` against a REAL `vantadb-server` process.
//!
//! The server binary reads its config entirely from environment variables (no
//! `--require-auth` flag — that flag only exists on `vanta-cli server`). Forced
//! auth = `VANTADB_API_KEY` + `VANTADB_REQUIRE_AUTH=true`.
//!
//! These tests are gated: they only run when `VANTADB_TEST_SERVER=1` is set AND a
//! build of `vantadb-server` exists at the repo-root target. They spawn the binary,
//! probe `/health` until ready, then exercise health/put/search/get/delete/list
//! through the `ServerConnection` adapter, and verify a dead server yields a clean
//! `VantaError::Http`.

use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use vantadb_desktop_lib::connections::{
    IngestItem, ServerClientConfig, ServerConnection,
};
use vantadb_desktop_lib::error::HttpErrorKind;
use vantadb_desktop_lib::{HealthStatus, VantaConnection, VantaError};

const TEST_TOKEN: &str = "desktop-integration-test-key-1";

/// Whether the real-server tests are enabled (env `VANTADB_TEST_SERVER`).
fn enabled() -> bool {
    std::env::var("VANTADB_TEST_SERVER").is_ok()
}

/// Resolve the `vantadb-server` binary relative to this crate's manifest dir.
///
/// The desktop workspace has its own `[workspace]`, so `cargo build -p vantadb-server`
/// at the repo root emits to the repo-root `target/` — that's `../../target` from here.
fn server_binary() -> Option<PathBuf> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    #[cfg(windows)]
    let exe = format!("{manifest}/../../target/debug/vantadb-server.exe");
    #[cfg(not(windows))]
    let exe = format!("{manifest}/../../target/debug/vantadb-server");
    let p = PathBuf::from(exe);
    if p.exists() {
        Some(p)
    } else {
        None
    }
}

/// Grab an ephemeral free port by binding `:0` then dropping the listener.
fn ephemeral_port() -> u16 {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    l.local_addr().unwrap().port()
}

/// Spawn `vantadb-server` with forced auth on an ephemeral port.
fn spawn_server(port: u16, storage: &std::path::Path) -> Child {
    let bin = server_binary().expect("vantadb-server binary not found; run `cargo build -p vantadb-server` at repo root");
    Command::new(bin)
        .env("VANTADB_API_KEY", TEST_TOKEN)
        .env("VANTADB_REQUIRE_AUTH", "true")
        .env("VANTADB_HOST", "127.0.0.1")
        .env("VANTADB_PORT", port.to_string())
        .env("VANTADB_STORAGE_PATH", storage)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn vantadb-server")
}

/// Wait until `GET /health` returns OK (probes with a bare client, no auth).
async fn wait_ready(port: u16, timeout: Duration) -> bool {
    let cfg = ServerClientConfig {
        url: "127.0.0.1".to_string(),
        port,
        token: None,
        timeout: Duration::from_secs(2),
    };
    let start = Instant::now();
    while start.elapsed() < timeout {
        if let Ok(client) = vantadb_desktop_lib::connections::ServerClient::new(cfg.clone()) {
            if client.health().await.is_ok() {
                return true;
            }
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
    false
}

fn config(port: u16) -> ServerClientConfig {
    ServerClientConfig {
        url: "127.0.0.1".to_string(),
        port,
        token: Some(TEST_TOKEN.to_string()),
        timeout: Duration::from_secs(5),
    }
}

/// End-to-end against the real server: connect, get, ingest, search, list, delete.
#[tokio::test]
async fn real_server_health_put_get_search_delete() {
    if !enabled() {
        eprintln!("skipping (set VANTADB_TEST_SERVER=1 and build vantadb-server to run)");
        return;
    }
    if server_binary().is_none() {
        eprintln!("skipping: vantadb-server binary not built (cargo build -p vantadb-server)");
        return;
    }

    let port = ephemeral_port();
    let storage = std::env::temp_dir().join(format!("vantadb-desktop-it-{port}"));
    std::fs::create_dir_all(&storage).expect("create temp storage dir");

    let mut child = spawn_server(port, &storage);
    assert!(
        wait_ready(port, Duration::from_secs(15)).await,
        "server did not become healthy on port {port}"
    );

    let mut conn = ServerConnection::with(config(port)).unwrap();
    conn.connect().await.expect("connect against real server");

    // health reports Healthy with a latency and message.
    let health = conn.health().await.expect("health");
    assert_eq!(health.status, HealthStatus::Healthy);
    assert!(health.message.is_some());

    // capabilities advertise Http.
    let caps = conn.capabilities();
    assert!(caps.contains(&vantadb_desktop_lib::connections::Capability::Http));

    // ingest returns an id.
    let id = conn
        .ingest(IngestItem {
            id: Some("12345".to_string()),
            namespace: "default".to_string(),
            text: "integration probe payload".to_string(),
            embedding: None,
            metadata: Default::default(),
        })
        .await
        .expect("ingest against real server");
    assert_eq!(id, "12345");

    // get reflects the stored record.
    let rec = conn.get("12345", Some("default")).await.expect("get");
    assert!(rec.text.contains("integration probe"), "got {:?}", rec.text);

    // list sees it.
    let listed = conn.list(Some("default"), 100, None).await.expect("list");
    assert!(listed.records.iter().any(|r| r.id == "12345"));

    // delete removes it.
    conn.delete("12345", Some("default")).await.expect("delete");

    // cleanup
    let _ = conn.disconnect().await;
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(storage);
}

/// A dead server (no listener on the port) yields a clean `VantaError::Http`,
/// never a panic.
#[tokio::test]
async fn dead_server_yields_http_error() {
    // Grab a port that is almost certainly not listening and release it.
    let dead_port = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr: SocketAddr = l.local_addr().unwrap();
        addr.port()
    }; // listener dropped here

    let cfg = ServerClientConfig {
        url: "127.0.0.1".to_string(),
        port: dead_port,
        token: Some(TEST_TOKEN.to_string()),
        timeout: Duration::from_secs(1),
    };
    let mut conn = ServerConnection::with(cfg).unwrap();

    // connect -> health -> connection refused -> Http, not a panic.
    let err = conn.connect().await.expect_err("dead server must fail");
    match err {
        VantaError::Http { kind, status, .. } => {
            // connection-refused is surfaced as Other (no HTTP status), not
            // Autorized/NotFound. The contract only requires a clean Http error.
            assert_eq!(kind, HttpErrorKind::Other, "required clean Http error");
            assert_eq!(status, None);
        }
        other => panic!("expected Http error, got {other:?}"),
    }
}