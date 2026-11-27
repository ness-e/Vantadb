// ponytail: blanket allow - unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! FIND-782: the discovery probe must not pay the whole request budget on a
//! **closed** loopback port — and must still pay it for a writer that is alive
//! but silent.
//!
//! FIND-757 measured the split of a failing re-election (~1.95 s): ~77 % is the
//! 500 ms health timeout paid three times (`RESOLVE_ATTEMPTS`), ~9 % the six
//! `is_pid_alive` calls. The root cause is the `connect` phase: on Windows a
//! closed loopback port answers WSAECONNREFUSED after **~2.0 s** (5/5 runs,
//! 2019-2191 ms), so a raw request always burns its whole 500 ms before the HTTP
//! exchange exists.
//!
//! The fix bounds the *connect* phase of the **probe client** only
//! (`HEALTH_CONNECT_TIMEOUT_MS`), leaving both budgets that matter untouched:
//!
//! - the 500 ms **response** budget — a live but slow writer keeps all of it;
//! - the **call** client (`POST /api/v2/mcp/proxy`), which keeps HEAD's builder
//!   so the timing of the replay path does not move under this ticket.
//!
//! ## What these tests can and cannot see
//!
//! Both assertions are wall-clock *observables* of a mechanism (`try_proxy`
//! returns `Option`, so the duration is the only signal), in the `FIND-739`
//! sense: each is paired with a state assertion and neither can be met by the
//! old shape.
//!
//! The closed-port one is **Windows-shaped**: on Linux/macOS a closed port
//! answers RST at once, so the old shape would also come in at ~30 ms and a
//! green there proves nothing. The alive-but-silent one is platform
//! independent (the kernel completes the handshake in both cases) and is what
//! stops the tempting simplification — merging the two clients into a single
//! 150 ms timeout — from passing.

use std::time::{Duration, Instant};
use vantadb_mcp::{discovery_path, try_proxy, ServerInfo};

/// Ceiling for the *probe* on a closed port. The old shape cannot go under
/// ~500 ms (it exhausted the request timeout by construction on Windows); the
/// fixed shape measures 155-197 ms.
const PROBE_CEILING: Duration = Duration::from_millis(400);
/// A silent writer must keep the full 500 ms reply budget: anything under this
/// means the deadline leaked into the reply phase.
const REPLY_BUDGET_FLOOR: Duration = Duration::from_millis(400);

fn closed_port() -> u16 {
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);
    port
}

fn write_discovery(path: &str, pid: u32, port: u16) {
    let info = ServerInfo {
        pid,
        http_port: port,
        started_at: 0,
        version: "0.10.0".into(),
    };
    std::fs::write(
        discovery_path(path),
        serde_json::to_string_pretty(&info).unwrap(),
    )
    .unwrap();
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort_unstable();
    v[v.len() / 2]
}

#[tokio::test]
async fn closed_port_probe_fails_on_the_connect_deadline() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_string_lossy().to_string();
    let port = closed_port();

    // The shape FIND-757 measured: the advertised PID is **alive** (this
    // process) and the advertised port is closed, so `try_connect` cannot
    // short-circuit on a dead PID — every probe reaches the health request.
    write_discovery(&path, std::process::id(), port);

    let mut samples = Vec::new();
    for _ in 0..5 {
        let started = Instant::now();
        let connected = try_proxy(&path, None).await;
        samples.push(started.elapsed());
        assert!(
            connected.is_none(),
            "a closed port must never yield a handle"
        );
    }
    let med = median(samples.clone());
    println!("FIND-782 try_proxy live-PID/closed-port: {samples:?} median {med:?}");

    assert!(
        med < PROBE_CEILING,
        "the probe paid the request timeout instead of failing on the connect \
         deadline: median {med:?} (samples {samples:?}) is over {PROBE_CEILING:?}; \
         a closed loopback port refuses in ~2 s on Windows, so only a connect \
         deadline keeps this under the budget"
    );

    // A probe that fails is not evidence of death (MCP-48): the discovery file
    // of a live PID must survive it, still advertising the same port.
    let disc: ServerInfo =
        serde_json::from_str(&std::fs::read_to_string(discovery_path(&path)).unwrap())
            .expect("discovery of a live PID must survive a failed probe");
    assert_eq!(disc.pid, std::process::id());
    assert_eq!(disc.http_port, port, "advertisement left untouched");
}

/// R3 (FIND-782): the connect deadline must **not** leak into the reply phase.
///
/// A listener that never calls `accept()` still has its handshakes completed by
/// the kernel, so the connect is instantaneous and the request sits in the
/// socket buffer unanswered — the exact shape of "slow but alive". `try_proxy`
/// must then spend the 500 ms reply budget, not the 150 ms connect deadline.
/// Merging the two clients into one short-timeout client is the obvious
/// simplification that would break it, and it leaves every other test green.
///
/// The listener is held until the assertion so the port stays open.
#[tokio::test]
async fn alive_but_silent_writer_keeps_the_reply_budget() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().to_string_lossy().to_string();
    let silent = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = silent.local_addr().unwrap().port();
    write_discovery(&path, std::process::id(), port);

    let started = Instant::now();
    let connected = try_proxy(&path, None).await;
    let elapsed = started.elapsed();
    drop(silent);

    assert!(
        connected.is_none(),
        "a writer that never answers must not yield a handle"
    );
    assert!(
        elapsed >= REPLY_BUDGET_FLOOR,
        "the reply budget was cut short ({elapsed:?} < {REPLY_BUDGET_FLOOR:?}): the \
         connect deadline leaked into the answer phase, which is what a slow but \
         alive writer needs"
    );
    // Coarse net for the opposite regression (waiting forever / not timing out).
    assert!(
        elapsed < Duration::from_secs(3),
        "the probe did not honour its request budget: {elapsed:?}"
    );
}
