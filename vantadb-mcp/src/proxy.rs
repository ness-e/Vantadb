//! MCP-35 proxy client — DatabaseBusy → HTTP fallback; MCP-48 writer re-election.
//!
//! Writer instances expose an ephemeral HTTP server on 127.0.0.1:0 with
//! `GET /api/v2/health` and `POST /api/v2/mcp/proxy` (generic MCP dispatcher).
//! Proxy instances detect `.vanta.server.json`, probe health with 500 ms
//! timeout + sysinfo PID liveness, then forward `tools/call` via reqwest.
//! Auth token is passed through as `Authorization: Bearer` when the writer
//! requires it (VANTADB_API_KEY env).
//!
//! ## MCP-48 — the proxy survives the writer
//!
//! A `ProxyHandle` used to be captured once at startup: when the writer died,
//! every subsequent call failed until the MCP client was restarted ("no tools"
//! in the session). [`WriterProxy`] holds the handle behind a lock and, on a
//! *transport* failure, re-elects a writer:
//!
//! 1. re-read the discovery file and re-probe (`try_proxy`) — reconnect;
//! 2. if no writer is left, [`cleanup_stale`] removes a discovery file whose
//!    PID is **provably dead** (never on a health timeout);
//! 3. with no writer left, promote this process into the writer role by
//!    opening the storage — the fs2 exclusive lock is the authority, so a
//!    promotion can never produce a second writer (`DatabaseBusy`).
//!
//! Retries are bounded (3 probes, 50/150 ms async sleeps, one promotion
//! attempt); nothing sleeps on the runtime thread and no path loops forever.
//!
//! ## Never replay a call that may have run
//!
//! A transport failure is split in two ([`ProxyOutcome`]): `NotSent` (connect
//! refused — provably never delivered, safe to replay after re-electing) and
//! `Unknown` (timeout, 5xx, undecodable body — the writer may have executed the
//! tool already). `Unknown` invalidates the handle so the *next* call
//! re-elects, but the current call is **never replayed**: duplicate mutations
//! on a write path are worse than one honest error.

use crate::server::{spawn_writer_http, WriterGuard};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};

/// Bounded re-election attempts before giving up (each attempt = one probe).
const RESOLVE_ATTEMPTS: u32 = 3;
/// Base backoff between attempts; doubled per attempt (50 ms, 100 ms).
const BACKOFF_BASE_MS: u64 = 50;

/// Discovery file shape — must match `crate::server::ServerInfo` (duplicated here
/// to avoid server↔proxy import cycle; both derive from same JSON contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub pid: u32,
    pub http_port: u16,
    pub started_at: u64,
    pub version: String,
}

pub fn discovery_path(storage_path: &str) -> PathBuf {
    Path::new(storage_path).join(".vanta.server.json")
}

/// Check whether a PID is still alive (cross-platform via sysinfo).
///
/// Targeted refresh only: `System::new_all()` enumerates every process
/// (~2-3 s on Windows) and would turn a bounded re-election probe into a
/// multi-second stall — this is on the MCP proxy hot path (MCP-48).
pub fn is_pid_alive(pid: u32) -> bool {
    let mut sys = sysinfo::System::new();
    let target = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[target]),
        true,
        sysinfo::ProcessRefreshKind::nothing(),
    );
    sys.process(target).is_some()
}

/// Outcome of one forwarded call.
///
/// The distinction is load-bearing for MCP-48 and cannot be collapsed:
/// - `WriterError` — the writer answered "no" (validation, not-found, auth…).
///   Re-electing would be wrong and slow, and the client must see the reason.
/// - `NotSent` — connect refused: the request provably never reached a writer,
///   so replaying after a re-election cannot duplicate a mutation.
/// - `Unknown` — a timeout, a 5xx or an undecodable body: the writer may have
///   executed the tool. Re-electing is fine for later calls; replaying the
///   current one is not.
#[derive(Debug)]
pub enum ProxyOutcome {
    /// Writer answered successfully.
    Ok(Value),
    /// Writer answered with a domain error (`success: false`) — not a death.
    WriterError(Value),
    /// Provably never delivered (connect failure). Replay is safe.
    NotSent(Value),
    /// Delivered or possibly executed (timeout/5xx/decode). Never replay.
    Unknown(Value),
}

/// Handle to a live writer's HTTP proxy.
#[derive(Debug, Clone)]
pub struct ProxyHandle {
    /// Base URL e.g. `http://127.0.0.1:54321`
    pub base_url: String,
    /// Reused reqwest client (rustls, json).
    pub client: reqwest::Client,
    /// Optional Bearer token copied from the proxy's env (VANTADB_API_KEY).
    pub api_key: Option<String>,
}

impl ProxyHandle {
    /// Build a ProxyHandle from ServerInfo, probing health with 500 ms timeout.
    /// Returns None if PID dead, health non-healthy, or timeout.
    pub async fn try_connect(info: ServerInfo, api_key: Option<String>) -> Option<Self> {
        // sysinfo is a syscall-heavy snapshot: never on the async task (R-1/R-7).
        let pid = info.pid;
        let alive = tokio::task::spawn_blocking(move || is_pid_alive(pid))
            .await
            .unwrap_or(false);
        if !alive {
            tracing::warn!(pid = info.pid, "stale discovery: PID not alive");
            return None;
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .ok()?;
        let base_url = format!("http://127.0.0.1:{}", info.http_port);
        let health_url = format!("{}/api/v2/health", base_url);
        let mut req = client.get(&health_url);
        if let Some(ref key) = api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        let resp = req.send().await.ok()?;
        if !resp.status().is_success() {
            tracing::warn!(status = %resp.status(), "health probe non-200");
            return None;
        }
        let body: Value = resp.json().await.ok()?;
        // health_v2 returns { status: "healthy" | "degraded", ... }
        let status = body.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status != "healthy" {
            tracing::warn!(status, "health probe degraded");
            return None;
        }

        // MCP-05: Best-effort verification of writer profile against local profile
        let config_url = format!("{}/_admin/config", base_url);
        let mut cfg_req = client.get(&config_url);
        if let Some(ref key) = api_key {
            cfg_req = cfg_req.header("Authorization", format!("Bearer {}", key));
        }
        if let Ok(cfg_resp) = cfg_req.send().await {
            if cfg_resp.status().is_success() {
                if let Ok(cfg_json) = cfg_resp.json::<Value>().await {
                    if let Some(writer_profile) =
                        cfg_json.get("mcp_profile").and_then(|v| v.as_str())
                    {
                        let local_profile = std::env::var("VANTADB_MCP_PROFILE")
                            .unwrap_or_else(|_| "brain".to_string());
                        if writer_profile != local_profile {
                            tracing::warn!(
                                writer_profile,
                                local_profile = %local_profile,
                                "MCP-05: Proxy profile mismatch with writer instance"
                            );
                        }
                    }
                }
            }
        }

        Some(Self {
            base_url,
            client,
            api_key,
        })
    }

    /// Handle for a listener this process just bound itself (promotion, MCP-48).
    /// No health probe: the socket is already listening, and probing it would
    /// only add a failure mode to the re-election path.
    fn for_local_port(port: u16, api_key: Option<String>) -> Option<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .ok()?;
        Some(Self {
            base_url: format!("http://127.0.0.1:{}", port),
            client,
            api_key,
        })
    }

    /// Forward a `tools/call` via `POST /api/v2/mcp/proxy` (generic MCP proxy).
    /// `params` is the `tools/call` params object: { name, arguments }.
    pub async fn proxy_tools_call(&self, params: &Option<Value>) -> Result<Value, Value> {
        self.proxy_mcp_call("tools/call", params).await
    }

    /// Generic MCP proxy: POST /api/v2/mcp/proxy with { method, params }
    pub async fn proxy_mcp_call(
        &self,
        method: &str,
        params: &Option<Value>,
    ) -> Result<Value, Value> {
        match self.call_classified(method, params).await {
            ProxyOutcome::Ok(v) => Ok(v),
            ProxyOutcome::WriterError(e) | ProxyOutcome::NotSent(e) | ProxyOutcome::Unknown(e) => {
                Err(e)
            }
        }
    }

    /// Like [`Self::proxy_mcp_call`] but distinguishes a dead writer from a
    /// refusing one and from "we cannot know whether it ran".
    pub async fn call_classified(&self, method: &str, params: &Option<Value>) -> ProxyOutcome {
        let url = format!("{}/api/v2/mcp/proxy", self.base_url);
        let body = json!({
            "method": method,
            "params": params.clone().unwrap_or(json!(null))
        });
        let mut req = self
            .client
            .post(&url)
            .json(&body)
            .timeout(Duration::from_secs(60));
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                let value =
                    json!({"code": -32603, "message": format!("proxy request failed: {}", e)});
                // Only a connect failure proves the request never left us.
                return if e.is_connect() {
                    ProxyOutcome::NotSent(value)
                } else {
                    ProxyOutcome::Unknown(value)
                };
            }
        };
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            let value = json!({
                "code": -32603,
                "message": format!("proxy http {}: {}", status, text),
                "data": {"retriable": false}
            });
            // 4xx = the writer answered and refused (auth, rate limit, payload):
            // a live writer, not a death. 5xx = outcome unknown, do not replay.
            return if status.is_client_error() {
                ProxyOutcome::WriterError(value)
            } else {
                ProxyOutcome::Unknown(value)
            };
        }
        let resp_json: Value = match resp.json().await {
            Ok(v) => v,
            Err(e) => {
                return ProxyOutcome::Unknown(
                    json!({"code": -32603, "message": format!("proxy decode failed: {}", e)}),
                )
            }
        };
        // Writer's proxy endpoint returns { success: true, data: ... } or { success: false, error: ... }
        if resp_json.get("success") == Some(&json!(true)) {
            ProxyOutcome::Ok(resp_json.get("data").cloned().unwrap_or(json!(null)))
        } else if let Some(err) = resp_json.get("error") {
            ProxyOutcome::WriterError(err.clone())
        } else {
            // Fallback: treat whole body as success data
            ProxyOutcome::Ok(resp_json)
        }
    }
}

/// Try to establish a proxy from the discovery file at `storage_path`.
/// Returns Some(handle) if file exists, PID alive, and health 200 within 500 ms.
pub async fn try_proxy(
    storage_path: &str,
    api_key: Option<String>,
) -> Option<(ProxyHandle, ServerInfo)> {
    // `std::fs` on the Tokio path would stall a runtime worker (R-1/R-7).
    let path = discovery_path(storage_path);
    let info: ServerInfo = tokio::task::spawn_blocking(move || {
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    })
    .await
    .ok()??;
    let handle = ProxyHandle::try_connect(info.clone(), api_key).await?;
    tracing::info!(port = info.http_port, "proxy mode → {}", handle.base_url);
    Some((handle, info))
}

/// Stale cleanup, **PID-only**: removes `.vanta.server.json` only when its
/// writer is demonstrably gone (PID not alive).
///
/// Returns whether a file was removed.
///
/// MCP-48: a health-probe timeout is *not* evidence of death — the old
/// version deleted the file whenever `try_proxy` failed, which includes a slow
/// but live writer, breaking discovery for every other client. Liveness by
/// lock is owned by the storage engine; this function only tidies the service
/// directory, so it must be conservative.
///
/// An **unparseable** file is also left alone: it is not evidence of a dead
/// writer either (a partial read of a live writer's file used to delete that
/// writer's advertisement for the rest of its life). `write_discovery_file`
/// now publishes atomically, and a truly corrupt file is self-healed by the
/// next promotion, which overwrites it under the lock.
///
/// Known false positive: PID reuse — a recycled PID keeps a stale file alive.
/// That costs one stale advertisement; the fs2 lock, not the file, is what
/// actually prevents a second writer.
pub fn cleanup_stale(storage_path: &str) -> bool {
    let path = discovery_path(storage_path);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(info) = serde_json::from_str::<ServerInfo>(&content) else {
        tracing::warn!(
            path = %path.display(),
            "leaving unparseable discovery file alone: it is not evidence of a dead writer"
        );
        return false;
    };
    if is_pid_alive(info.pid) {
        tracing::warn!(
            path = %path.display(),
            pid = info.pid,
            "refusing to clean discovery file: writer PID is still alive"
        );
        return false;
    }
    tracing::warn!(
        path = %path.display(),
        pid = info.pid,
        "removed stale discovery file (writer PID is gone)"
    );
    match std::fs::remove_file(&path) {
        Ok(()) => true,
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "failed to remove stale discovery file");
            false
        }
    }
}

/// What the proxy is currently talking to.
struct Link {
    /// Live writer handle (None when lost / not resolved yet).
    handle: Option<ProxyHandle>,
    /// Set when this process promoted itself into the writer role. Keeps the
    /// fs2 lock (via the `StorageEngine`) and the discovery file (via the
    /// guard) alive for as long as the proxy lives.
    promoted: Option<PromotedWriter>,
}

struct PromotedWriter {
    storage: Arc<vantadb::storage::StorageEngine>,
    _guard: WriterGuard,
}

/// MCP-48: a proxy that re-elects its writer instead of dying with it.
pub struct WriterProxy {
    storage_path: String,
    api_key: Option<String>,
    /// Config used to re-open the storage if this process must become writer.
    promote_config: Option<vantadb::config::Config>,
    link: RwLock<Link>,
    /// Single-flight gate for re-election: two requests hitting a dead writer
    /// must not both promote (the loser would block on the lock and report a
    /// spurious failure while this process just became the writer).
    resolve_gate: Mutex<()>,
    reconnects: AtomicU64,
    resolve_failures: AtomicU64,
}

impl WriterProxy {
    /// Unresolved proxy for `storage_path`; the first call resolves the writer.
    pub fn new(
        storage_path: &str,
        api_key: Option<String>,
        promote_config: Option<vantadb::config::Config>,
    ) -> Self {
        Self {
            storage_path: storage_path.to_string(),
            api_key,
            promote_config,
            link: RwLock::new(Link {
                handle: None,
                promoted: None,
            }),
            resolve_gate: Mutex::new(()),
            reconnects: AtomicU64::new(0),
            resolve_failures: AtomicU64::new(0),
        }
    }

    /// Connect to the writer advertised by the discovery file, if any.
    pub async fn connect(
        storage_path: &str,
        api_key: Option<String>,
        promote_config: Option<vantadb::config::Config>,
    ) -> Option<Self> {
        let (handle, _info) = try_proxy(storage_path, api_key.clone()).await?;
        let proxy = Self::new(storage_path, api_key, promote_config);
        proxy.link.write().await.handle = Some(handle);
        Some(proxy)
    }

    /// Current role, for logs and tests: `writer-promoted` once this process
    /// owns the storage, else `proxy`.
    pub async fn role(&self) -> &'static str {
        if self.link.read().await.promoted.is_some() {
            "writer-promoted"
        } else {
            "proxy"
        }
    }

    /// Whether a writer is currently reached (false = detached, re-election due).
    pub async fn connected(&self) -> bool {
        self.link.read().await.handle.is_some()
    }

    /// Number of successful re-elections (reconnects + promotions).
    pub fn reconnects(&self) -> u64 {
        self.reconnects.load(Ordering::Relaxed)
    }

    /// Number of resolutions that ended with no writer at all.
    pub fn resolve_failures(&self) -> u64 {
        self.resolve_failures.load(Ordering::Relaxed)
    }

    /// Current writer base URL, if any (log/diagnostic surface).
    pub async fn base_url(&self) -> Option<String> {
        self.link
            .read()
            .await
            .handle
            .as_ref()
            .map(|h| h.base_url.clone())
    }

    /// Forward one MCP method, re-electing the writer when the current one is
    /// unreachable. Domain errors from a live writer pass through untouched.
    ///
    /// Replay policy: a call is retried **only** when it provably never reached
    /// the writer (`NotSent`). `Unknown` (timeout / 5xx / undecodable body) may
    /// already have mutated storage, so it is reported as-is and the *next*
    /// call does the re-election.
    pub async fn call(&self, method: &str, params: &Option<Value>) -> Result<Value, Value> {
        let current = self.link.read().await.handle.clone();
        if let Some(handle) = current {
            match handle.call_classified(method, params).await {
                ProxyOutcome::Ok(v) => return Ok(v),
                ProxyOutcome::WriterError(e) => return Err(e),
                ProxyOutcome::NotSent(e) => {
                    tracing::warn!(
                        base_url = %handle.base_url,
                        error = %e,
                        "MCP-48: writer unreachable (request never sent) — re-electing"
                    );
                    self.invalidate(&handle.base_url).await;
                }
                ProxyOutcome::Unknown(e) => {
                    tracing::warn!(
                        base_url = %handle.base_url,
                        error = %e,
                        "MCP-48: writer outcome unknown — invalidating, NOT replaying"
                    );
                    self.invalidate(&handle.base_url).await;
                    return Err(outcome_unknown(&e.to_string()));
                }
            }
        }

        match self.resolve_with_backoff().await {
            Some(handle) => match handle.call_classified(method, params).await {
                ProxyOutcome::Ok(v) => Ok(v),
                ProxyOutcome::WriterError(e) => Err(e),
                ProxyOutcome::NotSent(e) => {
                    // Do not keep a known-bad handle installed: the next call
                    // would burn a round-trip before re-electing.
                    self.invalidate(&handle.base_url).await;
                    Err(writer_unavailable(&e.to_string()))
                }
                ProxyOutcome::Unknown(e) => {
                    self.invalidate(&handle.base_url).await;
                    Err(outcome_unknown(&e.to_string()))
                }
            },
            None => {
                self.resolve_failures.fetch_add(1, Ordering::Relaxed);
                Err(writer_unavailable(
                    "no writer reachable and none could be elected",
                ))
            }
        }
    }

    /// Drop the cached handle iff it is still the one that just failed (a
    /// concurrent call may have installed a fresh one already).
    async fn invalidate(&self, base_url: &str) {
        let mut link = self.link.write().await;
        if link.handle.as_ref().map(|h| h.base_url.as_str()) == Some(base_url) {
            link.handle = None;
        }
    }

    /// Bounded: N probes with async backoff, then at most one promotion.
    /// Single-flight: only one re-election runs at a time, so two requests
    /// hitting a dead writer cannot both try to take the lock.
    async fn resolve_with_backoff(&self) -> Option<ProxyHandle> {
        let _single_flight = self.resolve_gate.lock().await;
        // Another task may have re-elected while we waited for the gate.
        if let Some(handle) = self.link.read().await.handle.clone() {
            return Some(handle);
        }
        for attempt in 0..RESOLVE_ATTEMPTS {
            if attempt > 0 {
                let ms = BACKOFF_BASE_MS * (1 << (attempt - 1)) as u64;
                // `tokio::time::sleep` yields the thread — a std sleep here
                // would freeze the MCP stdout loop (concurrency-async R-2/R-3).
                tokio::time::sleep(Duration::from_millis(ms)).await;
            }
            if let Some(handle) = self.resolve_once().await {
                return Some(handle);
            }
        }
        self.promote().await
    }

    /// One discovery pass: reconnect to an advertised writer, else leave the
    /// stale file removed (only if its PID is dead) for the promotion step.
    async fn resolve_once(&self) -> Option<ProxyHandle> {
        if self.storage_path.is_empty() {
            return None;
        }
        if let Some((handle, info)) = try_proxy(&self.storage_path, self.api_key.clone()).await {
            self.link.write().await.handle = Some(handle.clone());
            let n = self.reconnects.fetch_add(1, Ordering::Relaxed) + 1;
            tracing::info!(
                reconnect = n,
                base_url = %handle.base_url,
                pid = info.pid,
                "MCP-48: proxy reconnected to writer"
            );
            return Some(handle);
        }
        // No writer answered. Tidy the service directory only if the writer is
        // provably gone — a live PID keeps its file (it may just be slow).
        // `std::fs` off the async task (R-1/R-7).
        let cleanup_path = self.storage_path.clone();
        let _ = tokio::task::spawn_blocking(move || cleanup_stale(&cleanup_path)).await;
        None
    }

    /// Become the writer. The fs2 exclusive lock is the authority: if another
    /// writer holds it, `StorageEngine::open_with_config` returns
    /// `DatabaseBusy` and we stay a proxy (never a second writer).
    ///
    /// Deliberately *not* gated on "no discovery file exists": a file that
    /// could not be removed (sharing violation, recycled PID) would otherwise
    /// disable promotion forever. Trying anyway costs at most one
    /// `file_lock_timeout_ms` in the pathological case.
    async fn promote(&self) -> Option<ProxyHandle> {
        if self.storage_path.is_empty() {
            return None;
        }
        // Already the writer: if our own listener died, restart it on the
        // retained engine instead of going through discovery — nobody else can
        // take the lock we hold, so a discovery round would dead-end.
        if let Some(storage) = self.local_storage().await {
            tracing::warn!("MCP-48: local writer listener unreachable — restarting it");
            let (guard, port) = spawn_writer_http(storage.clone()).await.ok()?;
            let handle = ProxyHandle::for_local_port(port, self.local_api_key())?;
            {
                let mut link = self.link.write().await;
                link.promoted = Some(PromotedWriter {
                    storage,
                    _guard: guard,
                });
                link.handle = Some(handle.clone());
            }
            return Some(handle);
        }

        let path = self.storage_path.clone();
        let mut cfg = self.promote_config.clone().unwrap_or_default();
        cfg.storage_path = path.clone();
        // The Bearer token must be the one the storage config enforces, else the
        // self-proxy would be rejected by our own writer auth.
        let writer_api_key = cfg.api_key.clone().or_else(|| self.api_key.clone());
        // Disk I/O + lock retry loop: never on the async runtime thread.
        let opened = tokio::task::spawn_blocking(move || {
            vantadb::storage::StorageEngine::open_with_config(&path, Some(cfg))
        })
        .await;
        let storage = match opened {
            Ok(Ok(storage)) => Arc::new(storage),
            Ok(Err(vantadb::Error::DatabaseBusy(msg))) => {
                tracing::debug!(msg = %msg, "MCP-48: promotion blocked by a live writer lock");
                return None;
            }
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "MCP-48: promotion could not open storage");
                return None;
            }
            Err(e) => {
                tracing::warn!(error = %e, "MCP-48: promotion task failed");
                return None;
            }
        };
        let (guard, port) = match spawn_writer_http(storage.clone()).await {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(error = %e, "MCP-48: promotion could not start writer http");
                return None;
            }
        };
        let handle = ProxyHandle::for_local_port(port, writer_api_key)?;
        {
            let mut link = self.link.write().await;
            link.promoted = Some(PromotedWriter {
                storage,
                _guard: guard,
            });
            link.handle = Some(handle.clone());
        }
        let n = self.reconnects.fetch_add(1, Ordering::Relaxed) + 1;
        tracing::info!(
            reconnect = n,
            port,
            pid = std::process::id(),
            "MCP-48: proxy promoted itself to writer (lock acquired)"
        );
        Some(handle)
    }

    /// Flush the promoted writer's storage on shutdown (no-op when a proxy).
    pub async fn flush_if_promoted(&self) {
        let storage = {
            let link = self.link.read().await;
            link.promoted.as_ref().map(|p| p.storage.clone())
        };
        let Some(storage) = storage else {
            return;
        };
        match tokio::task::spawn_blocking(move || storage.flush()).await {
            Ok(Ok(())) => tracing::info!("promoted writer flushed"),
            Ok(Err(e)) => tracing::error!(error = %e, "promoted writer flush failed"),
            Err(e) => tracing::error!(error = %e, "promoted writer flush task failed"),
        }
    }

    /// Storage of the writer this process promoted itself into, if any.
    async fn local_storage(&self) -> Option<Arc<vantadb::storage::StorageEngine>> {
        self.link
            .read()
            .await
            .promoted
            .as_ref()
            .map(|p| p.storage.clone())
    }

    /// Bearer token our own promoted writer enforces (parity auth).
    fn local_api_key(&self) -> Option<String> {
        self.promote_config
            .as_ref()
            .and_then(|c| c.api_key.clone())
            .or_else(|| self.api_key.clone())
    }
}

/// Retriable error for a proxy with no writer: `-32001 VANTADB_BUSY`, exactly
/// what a client would get from a `DatabaseBusy` on the direct path
/// (`docs/api/ERROR_HANDLING.md` §6.2), so retry guidance stays uniform.
fn writer_unavailable(detail: &str) -> Value {
    json!({
        "code": -32001,
        "message": format!("VantaDB writer unavailable: {}", detail),
        "data": {"code": "VANTADB_BUSY", "retriable": true}
    })
}

/// The writer may have executed the call (timeout / 5xx / undecodable body).
/// Not retriable: a retry could apply the same mutation twice. The hint tells
/// the operator what to check instead.
fn outcome_unknown(detail: &str) -> Value {
    json!({
        "code": -32603,
        "message": format!("VantaDB writer outcome unknown (not replayed): {}", detail),
        "data": {
            "retriable": false,
            "hint": "the call may have been executed; verify state before retrying"
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_pid_alive_self() {
        let pid = std::process::id();
        assert!(is_pid_alive(pid), "own pid must be alive");
    }

    #[test]
    fn is_pid_alive_dead() {
        // Use a very high PID that definitely doesn't exist
        assert!(!is_pid_alive(999_999), "bogus pid must be dead");
    }

    #[tokio::test]
    async fn stale_pid_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        // Write a discovery file with dead PID
        let info = ServerInfo {
            pid: 999_999,
            http_port: 12345,
            started_at: 0,
            version: "0.5.0".into(),
        };
        let disc = discovery_path(&path);
        std::fs::write(&disc, serde_json::to_string_pretty(&info).unwrap()).unwrap();
        assert!(disc.exists());
        // try_proxy should fail due to dead PID (health not probed)
        let res = try_proxy(&path, None).await;
        assert!(res.is_none(), "dead PID should not connect");
        // cleanup
        assert!(cleanup_stale(&path), "dead PID must be cleaned");
        assert!(!disc.exists(), "stale file should be removed");
    }

    /// MCP-48 regression: a live PID keeps its discovery file even when the
    /// probe fails (slow writer ≠ dead writer).
    #[tokio::test]
    async fn live_pid_is_not_cleaned() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let info = ServerInfo {
            pid: std::process::id(),
            http_port: 1,
            started_at: 0,
            version: "0.9.0".into(),
        };
        let disc = discovery_path(&path);
        std::fs::write(&disc, serde_json::to_string_pretty(&info).unwrap()).unwrap();
        assert!(!cleanup_stale(&path), "live PID must not be cleaned");
        assert!(disc.exists(), "live writer keeps its discovery file");
    }

    #[tokio::test]
    async fn health_timeout_fallback() {
        // No file -> try_proxy returns None quickly (no health to probe)
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let res = try_proxy(&path, None).await;
        assert!(res.is_none());
    }

    /// A connect failure proves the request never reached a writer, so replay
    /// after re-election cannot duplicate a mutation.
    #[tokio::test]
    async fn connect_failure_is_not_sent() {
        let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);
        let handle = ProxyHandle::for_local_port(port, None).unwrap();
        let outcome = handle
            .call_classified("tools/call", &Some(json!({"name": "memory_list"})))
            .await;
        assert!(
            matches!(outcome, ProxyOutcome::NotSent(_)),
            "connect refusal must be NotSent, got: {outcome:?}"
        );
    }

    /// An unparseable discovery file is not evidence of a dead writer: a
    /// partial read of a live writer's file must never delete it.
    #[tokio::test]
    async fn unparseable_discovery_is_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let disc = discovery_path(&path);
        std::fs::write(&disc, "{\"pid\":").unwrap();
        assert!(!cleanup_stale(&path), "unparseable is not provably dead");
        assert!(disc.exists(), "must not delete what it cannot classify");
    }

    #[tokio::test]
    async fn discovery_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let info = ServerInfo {
            pid: std::process::id(),
            http_port: 0,
            started_at: 12345,
            version: "0.5.0".into(),
        };
        let disc = discovery_path(&path);
        std::fs::write(&disc, serde_json::to_string_pretty(&info).unwrap()).unwrap();
        let content = std::fs::read_to_string(&disc).unwrap();
        let parsed: ServerInfo = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.pid, info.pid);
        assert_eq!(parsed.version, "0.5.0");
        let _ = std::fs::remove_file(&disc);
    }

    /// A handle with no discovery path cannot re-elect: it fails closed with
    /// the retriable BUSY envelope instead of hanging or looping.
    #[tokio::test]
    async fn detached_proxy_fails_closed_retriable() {
        let proxy = WriterProxy::new("", None, None);
        let started = std::time::Instant::now();
        let err = proxy
            .call("tools/call", &Some(json!({"name": "memory_list"})))
            .await
            .expect_err("no writer");
        assert_eq!(err["code"], json!(-32001));
        assert_eq!(err["data"]["retriable"], json!(true));
        assert_eq!(proxy.role().await, "proxy");
        assert_eq!(proxy.reconnects(), 0);
        assert!(started.elapsed() < Duration::from_secs(2), "bounded retry");
    }
}
