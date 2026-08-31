//! Integration: spawn/kill the MCP sidecar (DESKTOP-11).
//!
//! Gated: these tests only exercise a real child when the `vanta-cli` binary
//! (built with `--features embed-local`) exists. If it is not built, they
//! print an instructional skip and return — they NEVER build the binary
//! themselves (the repo-root workspace is invariant and expensive; instructing
//! is the contract). The binary is found via
//! [`vantadb_desktop_lib::connections::child_process::locate_binary`].

use std::time::Duration;

use vantadb_desktop_lib::connections::child_process::{locate_binary, McpSpawn};

/// Whether a built `vanta-cli` (embed-local) is available to spawn.
fn sidecar_available() -> bool {
    locate_binary().is_some()
}

/// Spawn + verify ready + clean kill, and that stderr was captured to a log.
#[tokio::test]
async fn spawn_ready_and_clean_kill_with_stderr_log() {
    if !sidecar_available() {
        eprintln!(
            "SKIP: vanta-cli binary not found. Build it first:\n  \
             cargo build --release --features embed-local --bin vanta-cli  (repo root)\n\
             or set VANTADB_CLI_BIN=<path>."
        );
        return;
    }

    // Use a per-test tempdir so concurrent CI runs do not collide on the storage lock.
    let db_path = std::env::temp_dir().join(format!(
        "vantadb-mcp-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&db_path);

    let mut sidecar = McpSpawn::spawn(db_path.clone())
        .await
        .expect("sidecar spawn must succeed when binary exists");
    assert!(sidecar.is_running(), "sidecar must be running after spawn");

    // stderr log must exist and be non-empty after the server signaled ready.
    let log = sidecar.log_path();
    assert!(log.exists(), "stderr log must exist: {}", log.display());
    let content = std::fs::read_to_string(log).unwrap_or_default();
    assert!(
        content.contains("MCP stdio server started"),
        "stderr log should contain the ready marker: {content:?}"
    );

    // Graceful shutdown within the grace window.
    sidecar
        .request_shutdown(Duration::from_secs(10))
        .await
        .expect("request_shutdown must not error");
    assert!(!sidecar.is_running(), "sidecar should no longer be running");

    let _ = std::fs::remove_dir_all(&db_path);
}

/// Drop must force-kill cleanly even without an explicit shutdown request.
#[tokio::test]
async fn drop_kills_cleanly() {
    if !sidecar_available() {
        eprintln!("SKIP: vanta-cli binary not found (see prior test).");
        return;
    }
    let db_path = std::env::temp_dir().join(format!(
        "vantadb-mcp-drop-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&db_path);
    {
        let mut sidecar = McpSpawn::spawn(db_path.clone())
            .await
            .expect("spawn must succeed when binary exists");
        assert!(sidecar.is_running());
        // dropped at end of scope → Drop kills
    }
    let _ = std::fs::remove_dir_all(&db_path);
}
