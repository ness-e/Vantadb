//! Sidecar MCP spawner for the desktop app (DESKTOP-11, Fase 3).
//!
//! Locates the `vantadb-server` binary and launches it in MCP-jsonrpc-server
//! (`--mcp`) mode as a child process with:
//!   - stdin/stdout piped (reserved for the MCP JSON-RPC protocol),
//!   - stderr teed into a per-process log file under `std::env::temp_dir()`
//!     named `vantadb-mcp-<pid>.log`,
//!   - a bounded startup timeout: if the child fails to signal readiness on
//!     stderr within [`SPAWN_TIMEOUT`], it is killed and a `VantaError::Mcp`
//!     is returned.
//!
//! The child is a trust boundary — all user/path handling below avoids `.unwrap`
//! on external input, and `Drop` guarantees a clean kill.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use crate::error::VantaError;

/// How long to wait for the sidecar to signal it is ready on stderr.
pub const SPAWN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

/// Substring the sidecar's stderr emits once the MCP loop is accepting input.
/// Matches `init_telemetry` → `run_stdio_server`'s `info!("MCP stdio server started")`.
const READY_MARKER: &str = "MCP stdio server started";

#[cfg(windows)]
const EXE: &str = "vantadb-server.exe";
#[cfg(not(windows))]
const EXE: &str = "vantadb-server";

/// Resolve the `vantadb-server` binary path, best-effort.
///
/// Candidate order:
/// 1. `VANTADB_SERVER_BIN` env override (explicit — used in CI/packaging).
/// 2. Next to the current executable (Tauri-bundled sidecar in release): the
///    path the app ships alongside its own binary.
/// 3. Dev builds: `$CARGO_MANIFEST_DIR/{../,../../}/target/debug/vantadb-server`
///    (the desktop workspace has its own `[workspace]`, so the repo-root target
///    lives at `../../target`).
///
/// Returns `None` when nothing exists — callers must not `.unwrap()` this.
pub fn locate_binary() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("VANTVADB_SERVER_BIN").map(PathBuf::from) {
        if p.is_file() {
            return Some(p);
        }
    }

    // Bundled sidecar next to the running executable (Tauri pattern).
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let side = dir.join(EXE);
            if side.is_file() {
                return Some(side);
            }
        }
    }

    // Dev: relative to the desktop crate manifest.
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("debug")
        .join(EXE);
    let candidates = [
        // desktop/src-tauri/target/debug  (own workspace)
        manifest.clone(),
        // repo-root target: desktop/src-tauri/../../target/debug
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug")
            .join(EXE),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// A running sidecar MCP child process.
///
/// Owns the child, the stderr log path, and the background task that tees the
/// child's stderr into the log file. `Drop` kills the child cleanly.
pub struct McpSpawn {
    child: Child,
    log_path: PathBuf,
}

impl McpSpawn {
    /// Locate and spawn the sidecar in `--mcp` mode, waiting (bounded) for it to
    /// become ready. On failure the child is killed and a `VantaError::Mcp` is
    /// returned — never a panic.
    pub async fn spawn() -> Result<Self, VantaError> {
        let bin = locate_binary().ok_or_else(|| {
            VantaError::Mcp(
                "vantadb-server binary not found; build it (cargo build -p vantadb-server at \
                 repo root) or set VANTVADB_SERVER_BIN"
                    .into(),
            )
        })?;

        let pid = std::process::id();
        let log_path = std::env::temp_dir().join(format!("vantadb-mcp-{pid}.log"));
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| VantaError::Io(format!("open sidecar log {}: {e}", log_path.display())))?;

        let mut child = Command::new(&bin)
            .arg("--mcp")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| VantaError::Mcp(format!("failed to spawn {bin:?}: {e}")))?;

        let stderr = child
            .stderr
            .take()
            .ok_or(VantaError::Mcp("sidecar stderr not piped".into()))?;

        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
        let mut tee = log_file;
        // `take()` lets us consume the sender exactly once, on the first marker line.
        let mut ready = Some(ready_tx);
        // Detached task: tees child stderr into the log and flags readiness. It
        // runs for the child's lifetime; dropping the JoinHandle detaches it.
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = writeln!(tee, "{line}");
                // Only the startup marker counts as ready — a stray error line
                // ("Failed to open storage engine", …) must not.
                if line.contains(READY_MARKER) {
                    if let Some(tx) = ready.take() {
                        let _ = tx.send(());
                    }
                }
            }
        });

        // Bounded readiness: kill on timeout.
        match tokio::time::timeout(SPAWN_TIMEOUT, ready_rx).await {
            Ok(_) => {}
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(VantaError::Mcp(format!(
                    "sidecar did not become ready within {:?}; log: {}",
                    SPAWN_TIMEOUT,
                    log_path.display()
                )));
            }
        }

        Ok(Self { child, log_path })
    }

    /// The OS pid of the sidecar (when the child is alive).
    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }

    /// Whether the child is still running.
    pub fn is_running(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }

    /// Path of the stderr capture log for this sidecar run.
    pub fn log_path(&self) -> &Path {
        &self.log_path
    }

    /// Request a shutdown: close the child's stdin (graceful) and wait up to
    /// `grace`, then force-kill as a backstop. Graceful works on every platform
    /// because the sidecar's MCP loop reads stdin and exits on EOF — see
    /// `vantadb-mcp run_stdio_server` (`Ok(None) => break`), after which
    /// `vantadb-server`'s `main` flushes the storage engine.
    pub async fn request_shutdown(&mut self, grace: std::time::Duration) -> Result<(), VantaError> {
        // 1. Graceful stop, cross-platform: dropping the write end of the stdin
        //    pipe sends EOF to the sidecar's MCP loop, which breaks out and lets
        //    the storage engine flush (vantadb-server main.rs flushes after
        //    `run_stdio_server` returns) — never a forced kill that could drop
        //    in-flight metadata. This is the only per-process graceful path on
        //    Windows, which has no deliverable signal (see [`send_graceful_stop`]).
        drop(self.child.stdin.take());

        // 2. Unix: additionally SIGINT the child. Its `tokio::signal::ctrl_c`
        //    handler sets a drain flag checked after in-flight requests — belt
        //    and suspenders over the stdin EOF above.
        #[cfg(unix)]
        if self.is_running() {
            if let Some(pid) = self.child.id() {
                send_graceful_stop(pid);
            }
        }

        // 3. Give the child up to `grace` to exit and be reaped; if it does, we
        //    are done. Timeout falls through to the forced-kill backstop.
        if tokio::time::timeout(grace, self.child.wait()).await.is_ok() {
            return Ok(());
        }

        // Backstop: force-kill and reap (child ignored EOF/SIGINT or is hung).
        self.child
            .kill()
            .await
            .map_err(|e| VantaError::Mcp(format!("force-kill sidecar: {e}")))?;
        let _ = self.child.wait().await;
        Ok(())
    }

    /// Force-kill the sidecar immediately.
    pub async fn kill(&mut self) -> Result<(), VantaError> {
        self.child
            .kill()
            .await
            .map_err(|e| VantaError::Mcp(format!("kill sidecar: {e}")))?;
        let _ = self.child.wait().await;
        Ok(())
    }
}

/// Best-effort graceful stop signal. The sidecar's MCP loop shuts down on
/// SIGINT (vantadb-mcp `run_stdio_server` -> `tokio::signal::ctrl_c`), so
/// SIGINT — not SIGTERM — is the signal that reaches its flush path.
///
/// Unix-only: Windows has no per-process signal (`GenerateConsoleCtrlEvent`
/// only delivers Ctrl-C to processes sharing the caller's console and, per
/// Microsoft Learn, CTRL_C cannot be limited to a process group). Windows
/// graceful shutdown in [`McpSpawn::request_shutdown`] uses stdin EOF instead.
#[cfg(unix)]
fn send_graceful_stop(pid: u32) {
    // SAFETY: `pid` is `Child::id()` of the process this struct spawned and
    // still owns (guarded by `is_running()`); `libc::kill` only delivers the
    // signal to that pid. A race (child already exited) surfaces as a harmless
    // ESRCH, which is ignored — the forced-kill backstop still applies.
    unsafe { libc::kill(pid as i32, libc::SIGINT) };
}

impl Drop for McpSpawn {
    fn drop(&mut self) {
        // Clean kill — `start_kill` is sync/non-blocking and safe in Drop.
        let _ = self.child.start_kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_suffix_matches_platform() {
        #[cfg(windows)]
        assert_eq!(EXE, "vantadb-server.exe");
        #[cfg(not(windows))]
        assert_eq!(EXE, "vantadb-server");
    }

    /// Unit-level: `locate_binary` never panics and either finds the built
    /// binary or returns None (documented skip). If it returns Some, it must be
    /// an existing file.
    #[test]
    fn locate_binary_returns_existing_file_or_none() {
        match locate_binary() {
            Some(p) => assert!(p.is_file(), "resolved binary must exist: {}", p.display()),
            None => eprintln!(
                "skipping binary-location assertion (vantadb-server not built; \
                 build it at the repo root target)"
            ),
        }
    }
}
