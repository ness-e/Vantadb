// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! MCP-48: the proxy must re-elect a writer instead of dying with it.
//!
//! Contract reproduced here end-to-end, in-process, without restarting anything:
//!
//! 1. a live writer serves `tools/call` and the proxy is connected to it;
//! 2. the writer dies (PID gone, TCP refused);
//! 3. the **next** operation re-elects — it promotes this process to writer
//!    (fs2 lock is free) and the write lands in real storage;
//! 4. the state is observable: role flips, the reconnect counter moves, and
//!    a fresh `.vanta.server.json` advertises the new writer.
//!
//! Plus the safety halves, which matter more than the happy path:
//! - `cleanup_stale` never deletes a discovery file whose PID is alive
//!   (a slow writer is not a dead writer);
//! - with a live writer holding the fs2 lock, an unreachable advertised port
//!   does **not** promote (no double writer) and fails closed with a
//!   retriable `-32001`.

use serde_json::{json, Value};
use std::time::{Duration, Instant};
use vantadb_mcp::{cleanup_stale, discovery_path, ServerInfo, WriterProxy};

fn write_discovery(path: &str, pid: u32, port: u16) {
    let info = ServerInfo {
        pid,
        http_port: port,
        started_at: 0,
        version: "0.9.0".into(),
    };
    std::fs::write(
        discovery_path(path),
        serde_json::to_string_pretty(&info).unwrap(),
    )
    .unwrap();
}

/// Minimal writer stand-in: raw TCP, canned routes, dies on the oneshot.
/// `proxy_status` lets a test make the writer *refuse* (`401`) instead of die.
async fn spawn_fake_writer_with(proxy_status: u16) -> (u16, tokio::sync::oneshot::Sender<()>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut rx => break,
                acc = listener.accept() => {
                    let Ok((mut sock, _)) = acc else { break };
                    tokio::spawn(async move {
                        let mut data = Vec::new();
                        let mut buf = [0u8; 1024];
                        loop {
                            let n = sock.read(&mut buf).await.unwrap_or(0);
                            if n == 0 {
                                return;
                            }
                            data.extend_from_slice(&buf[..n]);
                            if data.windows(4).any(|w| w == b"\r\n\r\n") {
                                break;
                            }
                        }
                        let req = String::from_utf8_lossy(&data);
                        let (status, body) = if req.contains("/api/v2/health") {
                            ("200 OK", r#"{"status":"healthy"}"#)
                        } else if proxy_status == 200 {
                            ("200 OK", r#"{"success":true,"data":{"fake":true}}"#)
                        } else {
                            (
                                "401 Unauthorized",
                                r#"{"success":false,"error":{"code":-32005,"message":"Unauthorized"}}"#,
                            )
                        };
                        let resp = format!(
                            "HTTP/1.1 {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                            status,
                            body.len(),
                            body
                        );
                        let _ = sock.write_all(resp.as_bytes()).await;
                        let _ = sock.shutdown().await;
                    });
                }
            }
        }
    });
    (port, tx)
}

async fn spawn_fake_writer() -> (u16, tokio::sync::oneshot::Sender<()>) {
    spawn_fake_writer_with(200).await
}

/// Wait until nothing accepts on `port` (writer fully gone, not just closing).
async fn wait_port_closed(port: u16) {
    for _ in 0..100 {
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_err()
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("port {port} never closed");
}

fn tools_call(name: &str, arguments: Value) -> Option<Value> {
    Some(json!({ "name": name, "arguments": arguments }))
}

/// The contract: writer alive → proxy → writer killed → next op re-elects,
/// no process restart, state observable.
#[test]
fn proxy_re_elects_writer_without_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_string_lossy().to_string();
    let path_for_run = path.clone();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(async move {
        let (fake_port, kill) = spawn_fake_writer().await;
        // The writer advertised by discovery is this process (live PID).
        write_discovery(&path_for_run, std::process::id(), fake_port);

        let proxy = WriterProxy::connect(&path_for_run, None, None)
            .await
            .expect("proxy must connect to the live writer");
        assert_eq!(proxy.role().await, "proxy");
        assert_eq!(proxy.reconnects(), 0, "no re-election yet");

        let v = proxy
            .call(
                "tools/call",
                &tools_call(
                    "memory_put",
                    json!({"namespace": "ns1", "key": "before", "payload": "x"}),
                ),
            )
            .await
            .expect("writer forwards the call");
        assert_eq!(v["fake"], json!(true), "first call goes to the writer");

        // ── writer dies: nothing listens, and the PID is gone ──────────────
        kill.send(()).unwrap();
        wait_port_closed(fake_port).await;
        write_discovery(&path_for_run, 999_999, fake_port);

        // ── next operation re-elects (promotes to writer), no restart ─────
        let v = proxy
            .call(
                "tools/call",
                &tools_call(
                    "memory_put",
                    json!({"namespace": "ns1", "key": "after", "payload": "survives the writer"}),
                ),
            )
            .await
            .expect("post-mortem write must succeed via the re-elected writer");
        assert_eq!(v["fake"], Value::Null, "must NOT be the dead fake writer");
        assert!(
            v["content"].is_array(),
            "expected a real tool result, got: {v}"
        );
        assert_eq!(proxy.role().await, "writer-promoted");
        assert_eq!(proxy.reconnects(), 1, "re-election is observable");

        // The promoted writer is discoverable by every other client…
        let disc: ServerInfo =
            serde_json::from_str(&std::fs::read_to_string(discovery_path(&path_for_run)).unwrap())
                .expect("promoted writer must advertise itself");
        assert_eq!(disc.pid, std::process::id());
        assert_ne!(disc.http_port, fake_port);

        // …and the write is readable through it (real storage, not a mock).
        let listed = proxy
            .call(
                "tools/call",
                &tools_call("memory_list", json!({"namespace": "ns1"})),
            )
            .await
            .expect("memory_list through the promoted writer");
        let text = listed.to_string();
        assert!(text.contains("after"), "write lost: {text}");
    });

    // Abort the promoted listener + drop its StorageEngine so the fjall files
    // close; otherwise the temp dir is locked on Windows (FIND-732 class).
    rt.shutdown_timeout(Duration::from_secs(5));
    for _ in 0..20 {
        if std::fs::remove_dir_all(&path).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Pre-mortem (a): a live writer that stops answering must NOT be promoted
/// over — the fs2 lock stays the authority and we fail closed, retriable.
///
/// FIND-739: the old assert was `elapsed < 5 s`, i.e. it measured the wall
/// clock (5.05 s, 50 ms over) while claiming to verify a *bounded* retry. It
/// conflated two different clocks: the retry mechanism (3 probes, 50/150 ms
/// backoff, one promotion attempt — `RESOLVE_ATTEMPTS`), and the promotion's
/// own lock patience, `Config::file_lock_timeout_ms` (1000 ms by default),
/// which the test never configured.
///
/// The contract is asserted on the mechanism now:
/// - `resolve_failures() == 1` — exactly one resolution ran, and it ended with
///   no writer (counter observable since MCP-48);
/// - no reconnect, no promotion, no handle installed, role still `proxy`;
/// - a second open of the same path is `DatabaseBusy` — the fs2 lock, not the
///   discovery file, is what makes a second writer impossible;
/// - the live-PID discovery file survives.
///
/// Lock patience is pinned to 50 ms in the config this proxy promotes with, so
/// the only clock left in the path is the probe/backoff budget. The remaining
/// wall-clock assert is a **coarse regression net** for the MCP-48 class
/// (`System::new_all()` made this same call take 11.17 s), not the contract.
#[tokio::test]
async fn live_writer_lock_blocks_promotion() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_string_lossy().to_string();

    // A real writer: owns the storage and therefore the fs2 exclusive lock.
    let storage = std::sync::Arc::new(vantadb::storage::StorageEngine::open(&path).unwrap());

    // Discovery advertises a live PID at a port that refuses connections:
    // the shape of a writer that is up but too slow to answer.
    let dead_probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let dead_port = dead_probe.local_addr().unwrap().port();
    drop(dead_probe);
    write_discovery(&path, std::process::id(), dead_port);

    // FIND-739: pin the lock patience instead of guessing at it. 50 ms covers
    // the 5/10/20 ms steps of `acquire_db_lock` before it exits inside the
    // 40 ms sleep (the ladder is 5+10+20+40 > 50), so the promotion really
    // walks the DatabaseBusy path; it just doesn't drag the default 1000 ms of
    // patience into a bound that claims to measure retries.
    let config = vantadb::config::Config {
        file_lock_timeout_ms: 50,
        ..Default::default()
    };
    let proxy = WriterProxy::new(&path, None, Some(config));
    let started = Instant::now();
    let err = proxy
        .call(
            "tools/call",
            &tools_call(
                "memory_put",
                json!({"namespace": "ns", "key": "k", "payload": "p"}),
            ),
        )
        .await
        .expect_err("unreachable writer must surface an error, not a promotion");
    let elapsed = started.elapsed();

    assert_eq!(
        err.get("code").and_then(|c| c.as_i64()),
        Some(-32001),
        "expected retriable BUSY, got: {err}"
    );
    assert_eq!(
        err.get("data").and_then(|d| d.get("retriable")),
        Some(&json!(true)),
        "client must be told it may retry: {err}"
    );

    // ── The mechanism, not the clock (FIND-739) ────────────────────────────
    // One resolution ran to completion and elected nobody. `resolve_failures`
    // counts failed *resolutions*, not probes, so it cannot pin
    // `RESOLVE_ATTEMPTS` by itself — the lower bound below does.
    assert_eq!(
        proxy.resolve_failures(),
        1,
        "exactly one resolution, ending with no writer (elapsed {elapsed:?})"
    );
    // The backoff before the last probe is 50 + 100 = 150 ms (proxy.rs
    // `BACKOFF_BASE_MS`), so any resolution that keeps its 3 probes cannot
    // finish faster than that. Dropping to 1 probe (or 0) would still fail
    // closed and satisfy every state assertion below — this is what catches it.
    assert!(
        elapsed >= Duration::from_millis(150),
        "retry budget skipped, not bounded: {elapsed:?} is under the 150 ms of \
         backoff that 3 probes must pay"
    );
    assert_eq!(proxy.reconnects(), 0, "nothing was elected");
    assert_eq!(
        proxy.role().await,
        "proxy",
        "must not promote over a live lock"
    );
    assert!(
        !proxy.connected().await,
        "a failed resolution must not leave a writer handle installed"
    );

    // The discovery file of a live PID survives the failed re-election —
    // "intact" read back, not just "the path exists".
    let disc: ServerInfo =
        serde_json::from_str(&std::fs::read_to_string(discovery_path(&path)).unwrap())
            .expect("live-PID discovery must survive, still parseable");
    assert_eq!(
        disc.pid,
        std::process::id(),
        "live writer keeps its advertisement"
    );
    assert_eq!(disc.http_port, dead_port, "advertisement left untouched");

    // Why those two writers stay impossible: the lock, not the discovery file,
    // is the authority. This open uses the **production default** patience
    // (1000 ms) on purpose — it runs after `elapsed` is captured, so it costs
    // ~1 s without contaminating the bound, and keeps the default covered.
    let probe_path = path.clone();
    let opened =
        tokio::task::spawn_blocking(move || vantadb::storage::StorageEngine::open(&probe_path))
            .await
            .expect("lock probe task");
    assert!(
        matches!(opened, Err(vantadb::Error::DatabaseBusy(_))),
        "a held fs2 lock must surface as DatabaseBusy, never as a second writer"
    );

    // ── Coarse net, not the contract (FIND-739) ────────────────────────────
    // With the lock patience pinned, this call is dominated by 3 probes +
    // 50/150 ms backoff: measured 1.90-2.40 s over 9 runs (FIND-757). The
    // scenario matters — discovery advertises a **live PID at a closed port**,
    // so every probe pays the 500 ms health timeout and `try_connect` never
    // short-circuits on a dead PID. That timeout is the cost (~1.5 s, ~77 %);
    // the six `is_pid_alive` calls are measured at 24-34 ms each ⇒ ~0.17 s
    // (~9 %). FIND-757: the "~300 ms per Windows targeted refresh" this
    // comment used to claim was never measured in isolation, and is wrong by
    // an order of magnitude. The 10 s net is ~4x the measured worst, ~2x the
    // loaded number reported for the old shape (MCP-44: 5.05 s, which also
    // paid the 1000 ms of patience this test no longer pays), and ~2x below
    // the regression it exists for: `System::new_all()` per PID check costs
    // ~3.5 s on Windows (**estimate — not re-measured, that call is gone**)
    // and this path does six checks per resolution (2 per probe × 3 probes)
    // ⇒ ~21 s, 11.17 s when MCP-48 measured it with fewer checks. It is a
    // net, not the contract — the mechanism asserts above are what must never
    // go red.
    assert!(
        elapsed < Duration::from_secs(10),
        "re-election regressed: {elapsed:?} for 3 probes + backoff with a 50 ms lock patience \
         (measured 1.93-2.10 s loaded; the `System::new_all()` class is ~21 s via 6 checks)"
    );

    // The writer storage is untouched and still the single writer.
    let db = vantadb::Embedded::from_engine(storage.clone());
    db.put(vantadb::sdk::MemoryInput {
        namespace: "ns".into(),
        key: "direct".into(),
        payload: "still alive".into(),
        ..Default::default()
    })
    .expect("writer storage still writable");
    let list = db
        .list(
            "ns",
            vantadb::sdk::MemoryListOptions {
                limit: 10,
                ..Default::default()
            },
        )
        .expect("writer storage still readable");
    assert!(list.records.iter().any(|r| r.key == "direct"));

    // The advertised port was a lie, not a death: once discovery points at a
    // reachable writer again, the very same proxy reconnects (no restart).
    let (real_port, kill) = spawn_fake_writer().await;
    write_discovery(&path, std::process::id(), real_port);
    let v = proxy
        .call(
            "tools/call",
            &tools_call("memory_list", json!({"namespace": "ns"})),
        )
        .await
        .expect("reconnect to the live writer");
    assert_eq!(v["fake"], json!(true));
    assert_eq!(proxy.reconnects(), 1);
    assert_eq!(proxy.role().await, "proxy");

    kill.send(()).unwrap();
}

/// Conservative stale cleanup: only a proven-dead PID is removed. Timeout
/// eviction and "unparseable ⇒ delete" are exactly the bugs MCP-48 removes.
#[tokio::test]
async fn stale_discovery_cleanup_is_pid_only() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_string_lossy().to_string();
    let disc = discovery_path(&path);

    // Live PID (this process) → refuse.
    write_discovery(&path, std::process::id(), 1);
    assert!(!cleanup_stale(&path), "must refuse to clean a live PID");
    assert!(disc.exists());

    // Dead PID → remove.
    write_discovery(&path, 999_999, 1);
    assert!(cleanup_stale(&path), "dead PID must be cleaned");
    assert!(!disc.exists());

    // Partial/corrupt file → NOT removed: a truncated read of a live writer's
    // file must never delete that writer's advertisement.
    std::fs::write(&disc, "{\"pid\":12").unwrap();
    assert!(!cleanup_stale(&path), "unparseable is not provably dead");
    assert!(disc.exists(), "corrupt discovery must be left alone");
    let _ = std::fs::remove_file(&disc);

    // Nothing to clean → no-op, not an error.
    assert!(!cleanup_stale(&path));
}

/// Pre-mortem (b) again — a writer that answers `401` is alive and refusing,
/// not dead: no re-election, no replay, and the client is not told to retry.
#[tokio::test]
async fn writer_refusal_is_not_a_death() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_string_lossy().to_string();
    let (port, kill) = spawn_fake_writer_with(401).await;
    write_discovery(&path, std::process::id(), port);

    let proxy = WriterProxy::connect(&path, None, None)
        .await
        .expect("health is fine: the writer is alive");
    let err = proxy
        .call(
            "tools/call",
            &tools_call(
                "memory_put",
                json!({"namespace": "ns", "key": "k", "payload": "p"}),
            ),
        )
        .await
        .expect_err("401 must surface as an error");
    assert_ne!(
        err.get("code").and_then(|c| c.as_i64()),
        Some(-32001),
        "a live writer refusing is not VANTADB_BUSY: {err}"
    );
    assert_eq!(
        err.get("data").and_then(|d| d.get("retriable")),
        Some(&json!(false)),
        "auth cannot be retried away: {err}"
    );
    assert_eq!(proxy.reconnects(), 0, "no re-election over a live writer");
    assert_eq!(proxy.role().await, "proxy");
    kill.send(()).unwrap();
}
