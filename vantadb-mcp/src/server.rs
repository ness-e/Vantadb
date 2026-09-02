//! Stdio server loop and JSON-RPC request dispatch.

use crate::config::McpConfig;
use crate::error::McpError;
use crate::handlers::initialize::*;
use crate::handlers::prompts::*;
use crate::handlers::resources::*;
use crate::handlers::tools::*;
use crate::metrics::{ActiveRequestGuard, McpMetrics};
use crate::protocol::{RpcRequest, RpcResponse};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;

// ── MCP-35 discovery file + writer HTTP ────────────────────────────────────

/// Discovery file written by the writer instance — `{storage_path}/.vanta.server.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// OS PID of the writer process.
    pub pid: u32,
    /// HTTP port bound on 127.0.0.1 (ephemeral, OS-assigned).
    pub http_port: u16,
    /// Unix millis when the writer started (diagnostic).
    pub started_at: u64,
    /// Crate version (CARGO_PKG_VERSION) for mismatch diagnostics.
    pub version: String,
}

/// Guard that removes `.vanta.server.json` on Drop iff `pid == self`.
/// Holds the fs2 exclusive lock indirectly via the StorageEngine's `_lock_file`.
pub struct WriterGuard {
    path: PathBuf,
    pid: u32,
}

impl Drop for WriterGuard {
    fn drop(&mut self) {
        // Best-effort: only delete if the file still belongs to us (pid check
        // protects against PID reuse / writer restart race).
        if let Ok(content) = std::fs::read_to_string(&self.path) {
            if let Ok(info) = serde_json::from_str::<ServerInfo>(&content) {
                if info.pid == self.pid {
                    let _ = std::fs::remove_file(&self.path);
                    info!(pid = self.pid, path = %self.path.display(), "cleaned discovery file on Drop");
                } else {
                    debug!(
                        self_pid = self.pid,
                        file_pid = info.pid,
                        "skip Drop cleanup: pid mismatch (writer restarted)"
                    );
                }
            }
        }
    }
}

/// Path to the discovery file for a given storage path.
pub fn discovery_path(storage_path: &str) -> PathBuf {
    Path::new(storage_path).join(".vanta.server.json")
}

/// Check whether a PID is still alive (cross-platform via sysinfo).
#[allow(dead_code)]
pub fn is_pid_alive(pid: u32) -> bool {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    sys.process(sysinfo::Pid::from(pid as usize)).is_some()
}

/// Write `.vanta.server.json` atomically and return its guard.
/// Caller must already hold the fs2 exclusive lock (i.e. StorageEngine is open).
fn write_discovery_file(storage_path: &str, http_port: u16) -> std::io::Result<WriterGuard> {
    let pid = std::process::id();
    let started_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let info = ServerInfo {
        pid,
        http_port,
        started_at,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    let path = discovery_path(storage_path);
    // ServerInfo is a plain Serialize struct (strings/ints) — serialization cannot fail.
    #[allow(clippy::expect_used)]
    let json = serde_json::to_string_pretty(&info).expect("ServerInfo serializes");
    // Open with create+truncate, write, sync
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;
    use std::io::Write;
    file.write_all(json.as_bytes())?;
    file.sync_all().ok(); // best-effort fsync
    Ok(WriterGuard { path, pid })
}

/// MCP-35 generic proxy endpoint: `POST /api/v2/mcp/proxy` forwards any MCP method
/// that needs storage (tools/call, resources/read, etc.) to writer's storage.
/// Loopback-only, but if VANTADB_API_KEY is set the proxy must send `Authorization: Bearer <key>`
/// and we validate it here for parity with `validate_auth_config` / auth_middleware.
async fn mcp_proxy_handler(
    axum::extract::State(state): axum::extract::State<Arc<vantadb::server::state::ServerState>>,
    headers: axum::http::HeaderMap,
    axum::Json(payload): axum::Json<Value>,
) -> axum::response::Response {
    // Parity auth: if writer has api_key, require Bearer match (also accept alt_api_key)
    if let Some(expected) = state.api_key.as_ref().or(state.alt_api_key.as_ref()) {
        let auth = headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let expected_bearer = format!("Bearer {}", expected);
        let alt_bearer = state.alt_api_key.as_ref().map(|k| format!("Bearer {}", k));
        let ok = auth == expected_bearer || alt_bearer.as_deref() == Some(auth);
        if !ok {
            let body = json!({"success": false, "error": {"code": -32001, "message": "Unauthorized: missing or invalid Bearer token for writer proxy"}});
            return (axum::http::StatusCode::UNAUTHORIZED, axum::Json(body)).into_response();
        }
    }
    let method = payload
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("tools/call");
    let params = payload.get("params").cloned();
    let storage = state.storage.clone();
    let cfg = McpConfig::from_storage(&storage);
    // Spawn blocking for tool/resource handlers that need storage
    let result = match method {
        "tools/call" => {
            // params is { name, arguments } for tools/call
            let tool_params = params.clone();
            tokio::task::spawn_blocking(move || {
                let executor = Executor::new(&storage);
                handle_tools_call(&tool_params, &executor, &storage, &cfg)
            })
            .await
        }
        "resources/read" => {
            let res_params = params.clone();
            let cfg2 = cfg.clone();
            tokio::task::spawn_blocking(move || handle_resources_read(&res_params, &storage, &cfg2))
                .await
        }
        _ => {
            let body = json!({"success": false, "error": {"code": -32601, "message": format!("proxy: method not supported: {}", method)}});
            return axum::Json(body).into_response();
        }
    };
    match result {
        Ok(Ok(v)) => axum::Json(json!({"success": true, "data": v})).into_response(),
        Ok(Err(e)) => axum::Json(json!({"success": false, "error": e})).into_response(),
        Err(e) => axum::Json(json!({"success": false, "error": {"code": -32603, "message": format!("proxy panic: {}", e)}})).into_response(),
    }
}

/// Spawn a minimal HTTP listener on 127.0.0.1:0 for the writer.
/// Returns the guard (keeps file alive) and bound port.
/// The listener serves `GET /api/v2/health` (mirrors health_v2) plus
/// the full `app_with_cors` router for proxy parity (1:1 tools via HTTP).
pub async fn spawn_writer_http(storage: Arc<StorageEngine>) -> std::io::Result<(WriterGuard, u16)> {
    // Derive the base storage path from data_dir (authoritative) — config.storage_path
    // is the default value and does not reflect the `path` argument passed to
    // `StorageEngine::open(path)` when config is None (see init.rs). `data_dir`
    // is `base_path.join("data")`, so parent is the base.
    let storage_path = if storage.data_dir.as_os_str().is_empty() {
        storage.config.storage_path.clone()
    } else {
        storage
            .data_dir
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| storage.config.storage_path.clone())
    };
    // In-memory backend or read-only never needs a discovery file/HTTP.
    if storage.config.read_only || storage_path.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "read-only or in-memory: no http needed",
        ));
    }
    if matches!(storage.config.backend_kind, vantadb::BackendKind::InMemory) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "in-memory backend: no http needed",
        ));
    }
    // Bind ephemeral loopback port BEFORE writing file (lock order: fs2 → TcpListener → file write)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let guard = write_discovery_file(&storage_path, port)?;
    // Build full axum router for proxy parity (all /api/v2/*)
    let config = storage.config.clone();
    let api_key: Option<std::sync::Arc<str>> = config.api_key.as_deref().map(std::sync::Arc::from);
    let alt_api_key: Option<std::sync::Arc<str>> =
        config.alt_api_key.as_deref().map(std::sync::Arc::from);
    let circuit_breaker = std::sync::Arc::new(vantadb::circuit_breaker::CircuitBreaker::new(
        config.circuit_breaker_failure_threshold,
        Duration::from_secs(config.circuit_breaker_open_timeout_secs),
    ));
    let pool = std::sync::Arc::new(vantadb::connection_pool::ConnectionPool::new(
        config.max_connections,
        Duration::from_millis(config.pool_acquire_timeout_ms),
    ));
    let state = std::sync::Arc::new(vantadb::server::state::ServerState {
        storage: storage.clone(),
        db: vantadb::sdk::VantaEmbedded::from_engine(storage.clone()),
        circuit_breaker,
        pool,
        api_key,
        alt_api_key,
        rbac_config: config.rbac_config.clone(),
        trusted_proxies: config.trusted_proxies.clone(),
        conversation_trigger: None,
    });
    // Ensure indexes current for health probe to succeed (same as bootstrap)
    if !config.read_only {
        let db = state.db.clone();
        let _ = tokio::task::spawn_blocking(move || db.ensure_indexes_current()).await;
    }
    let base_router = vantadb::server::router::app_with_cors(
        state.clone(),
        config.rate_limit_rpm,
        &config.allowed_origins,
    );
    let base_router =
        vantadb::server::router::mount_dashboard(base_router, config.dashboard_dir.as_deref());
    // MCP-35 proxy endpoint for 1:1 tools parity (writer side)
    let proxy_router = axum::Router::new()
        .route("/api/v2/mcp/proxy", axum::routing::post(mcp_proxy_handler))
        .with_state(state.clone());
    let router = base_router.merge(proxy_router);
    // Spawn serve task — holds listener, does not block caller
    tokio::spawn(async move {
        if let Err(e) = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        {
            warn!(error = %e, "writer http listener terminated");
        }
    });
    info!(pid = guard.pid, port, path = %guard.path.display(), "VantaDB writer active");
    // SIGTERM / Ctrl-C cleanup: remove file only if pid==self (guard's Drop already does, this is extra for signal)
    let sig_path = guard.path.clone();
    let sig_pid = guard.pid;
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            let mut sigterm =
                match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
                    Ok(s) => s,
                    Err(_) => return,
                };
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {},
                _ = sigterm.recv() => {},
            }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
        }
        // Best-effort cleanup on signal — guard's Drop is primary
        if let Ok(content) = std::fs::read_to_string(&sig_path) {
            if let Ok(info) = serde_json::from_str::<ServerInfo>(&content) {
                if info.pid == sig_pid {
                    let _ = std::fs::remove_file(&sig_path);
                    info!(pid = sig_pid, "cleaned discovery file on signal");
                }
            }
        }
    });
    Ok((guard, port))
}

// ── Stdio server (main entry point) ───────────────────────────────────────

/// Run the MCP server over stdin/stdout (JSON-RPC 2.0).
///
/// Supports graceful shutdown via SIGINT/Ctrl-C.  All blocking operations
/// are dispatched through a tokio blocking pool with a concurrency semaphore
/// and an optional per-request timeout.
pub async fn run_stdio_server(storage: Arc<StorageEngine>) {
    let config = McpConfig::from_storage(&storage);

    // MCP-01: a raw StorageEngine (as the server binary opens) skips the
    // `VantaEmbedded::open_with_config` index reconciliation, so
    // text_query / hybrid / text-filter searches fail on fresh DBs with
    // "text_index not found: bm25". Ensure index state at startup:
    // idempotent — no-op when counts match, rebuilds only when state is
    // missing/stale (existing DBs), writes fresh empty state for new DBs.
    if !storage.config.read_only {
        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        if let Err(e) = embedded.ensure_indexes_current() {
            error!(
                error = %e,
                "Failed to ensure index state at startup; text search may be unavailable"
            );
        }
    }

    // MCP-35 Step1: writer discovery file + ephemeral HTTP (127.0.0.1:0)
    // Order: fs2 lock already held via StorageEngine → TcpListener → file write
    // Guard lives for the whole server lifetime (Drop cleans pid==self).
    let _writer_guard: Option<WriterGuard> = if !storage.config.read_only
        && !storage.config.storage_path.is_empty()
        && !matches!(storage.config.backend_kind, vantadb::BackendKind::InMemory)
    {
        match spawn_writer_http(storage.clone()).await {
            Ok((g, port)) => {
                info!(
                    pid = g.pid,
                    port, "VantaDB writer http ready (.vanta.server.json)"
                );
                Some(g)
            }
            Err(e) => {
                debug!(error = %e, "writer http not started (in-memory or bind failed)");
                None
            }
        }
    } else {
        None
    };

    let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_concurrency));
    let metrics = Arc::new(McpMetrics::default());
    let running = Arc::new(AtomicBool::new(true));

    // Graceful shutdown on SIGINT / Ctrl-C.
    let sig_running = running.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            info!("Received SIGINT, initiating graceful shutdown");
            sig_running.store(false, Ordering::SeqCst);
        }
    });

    // Periodic metrics logging (every 30 s) — makes active_requests observable at runtime.
    let metrics_logger = metrics.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            info!(
                active = metrics_logger.active_requests.load(Ordering::Relaxed),
                total = metrics_logger.requests_total.load(Ordering::Relaxed),
                errors = metrics_logger.errors_total.load(Ordering::Relaxed),
                "MCP server metrics",
            );
        }
    });

    info!(
        max_concurrency = config.max_concurrency,
        request_timeout_ms = config.request_timeout.as_millis(),
        "MCP stdio server started"
    );

    serve_lines(
        &storage,
        &config,
        &semaphore,
        &metrics,
        &running,
        tokio::io::stdin(),
        tokio::io::stdout(),
    )
    .await;

    info!(
        total = metrics.requests_total.load(Ordering::Relaxed),
        errors = metrics.errors_total.load(Ordering::Relaxed),
        "MCP stdio server shut down"
    );
}

/// Read newline-delimited JSON-RPC messages from `reader`, dispatching each
/// one and writing its response to `writer`.
///
/// Split out of [`run_stdio_server`] (which owns real stdin/stdout) so tests
/// can drive the raw wire format through in-memory duplex pipes.
async fn serve_lines<R, W>(
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
    semaphore: &Arc<tokio::sync::Semaphore>,
    metrics: &Arc<McpMetrics>,
    running: &AtomicBool,
    reader: R,
    writer: W,
) where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let mut lines = BufReader::new(reader).lines();
    // MOD-08: requests are dispatched to background tasks so the reader loop
    // keeps draining stdin while one is in flight (no backpressure on a burst
    // of pipelined requests). Every response write is serialized through this
    // single lock so concurrent tasks never interleave bytes on stdout.
    let stdout = Arc::new(tokio::sync::Mutex::new(writer));
    // Track in-flight responses so shutdown drains them before returning (MOD-09).
    let mut inflight = tokio::task::JoinSet::new();

    loop {
        let line = match lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => break,
            Err(e) => {
                error!(error = %e, "stdin read error");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }

        metrics.requests_total.fetch_add(1, Ordering::Relaxed);

        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                metrics.errors_total.fetch_add(1, Ordering::Relaxed);
                warn!(error = %e, input_len = line.len(), "Failed to parse JSON-RPC");
                write_json(
                    &mut *stdout.lock().await,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": McpError::parse_error(e.to_string()).to_json()
                    }),
                )
                .await;
                continue;
            }
        };

        // MOD-07: absent `id` ⇒ notification (JSON-RPC 2.0 §4.1). Answering
        // one is forbidden and used to surface as a spurious -32700 that
        // broke strict MCP clients' handshake. The only inbound notifications
        // the MCP spec defines need no server action here —
        // `notifications/initialized` is a pure lifecycle ack and
        // `notifications/cancelled` targets request ids this server cannot
        // cancel mid-`spawn_blocking` — so known and unknown notifications
        // alike are consumed silently.
        let Some(req_id) = &req.id else {
            debug!(
                method = %req.method,
                "JSON-RPC notification received (no response emitted)"
            );
            continue;
        };
        let req_id = req_id.clone();

        if req.jsonrpc != "2.0" {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            warn!(jsonrpc = %req.jsonrpc, "Invalid JSON-RPC version, expected 2.0");
            write_json(
                &mut *stdout.lock().await,
                &json!({
                    "jsonrpc": "2.0",
                    "id": req_id,
                    "error": McpError::invalid_request(format!(
                        "Invalid JSON-RPC version: {}, expected 2.0", req.jsonrpc
                    )).to_json()
                }),
            )
            .await;
            continue;
        }

        // MOD-08: dispatch in the background so a slow `tools/call`/
        // `resources/read` never blocks reading the next line. Responses are
        // matched by JSON-RPC id, so out-of-order completion is fine.
        let (storage, config, semaphore, metrics) = (
            storage.clone(),
            config.clone(),
            semaphore.clone(),
            metrics.clone(),
        );
        let stdout = stdout.clone();
        inflight.spawn(async move {
            let res = dispatch_request(&req, &storage, &config, &semaphore, &metrics).await;
            let (result, error) = match res {
                Ok(val) => (Some(val), None),
                Err(err) => (None, Some(err)),
            };
            let response = RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result,
                error,
            };
            match serde_json::to_string(&response) {
                Ok(out) => {
                    let mut guard = stdout.lock().await;
                    if let Err(e) = guard.write_all(out.as_bytes()).await {
                        error!(error = %e, "Failed to write response to stdout");
                    } else if let Err(e) = guard.write_all(b"\n").await {
                        error!(error = %e, "Failed to write newline to stdout");
                    } else if let Err(e) = guard.flush().await {
                        error!(error = %e, "Failed to flush stdout");
                    }
                }
                Err(e) => {
                    error!(error = %e, "Failed to serialize JSON-RPC response body");
                }
            }
        });

        // MOD-09: once shutdown is signaled, stop reading new requests but keep
        // draining the in-flight responses (below) before returning — the
        // request currently being processed is never dropped.
        if !running.load(Ordering::SeqCst) {
            info!("Shutdown flag set, draining in-flight responses");
            break;
        }
    }

    // MOD-09: wait for every in-flight response to be written before exiting.
    while inflight.join_next().await.is_some() {}
}

/// Write a JSON value to `stdout`, logging I/O errors instead of swallowing them.
pub(crate) async fn write_json<W: tokio::io::AsyncWrite + Unpin>(stdout: &mut W, value: &Value) {
    match serde_json::to_string(value) {
        Ok(out) => {
            if let Err(e) = stdout.write_all(out.as_bytes()).await {
                error!(error = %e, "write_json: failed to write to stdout");
            } else if let Err(e) = stdout.write_all(b"\n").await {
                error!(error = %e, "write_json: failed to write newline to stdout");
            } else if let Err(e) = stdout.flush().await {
                error!(error = %e, "write_json: failed to flush stdout");
            }
        }
        Err(e) => {
            error!(error = %e, "write_json: serialization failed");
        }
    }
}

/// Route a parsed JSON-RPC request, enforcing concurrency limits, timeouts and
/// instrumentation.
pub(crate) async fn dispatch_request(
    req: &RpcRequest,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
    semaphore: &Arc<tokio::sync::Semaphore>,
    metrics: &Arc<McpMetrics>,
) -> Result<Value, Value> {
    let _active = ActiveRequestGuard::new(&metrics.active_requests);
    let start = Instant::now();

    let result = match req.method.as_str() {
        "initialize" => handle_initialize(req.params.as_ref()),
        "tools/list" => handle_tools_list(config),
        "tools/call" => {
            let sem = semaphore.clone();
            let storage_ctx = storage.clone();
            let params_ctx = req.params.clone();
            let cfg = config.clone();

            let _permit = sem
                .acquire_owned()
                .await
                .map_err(|_| McpError::internal_error("Semaphore closed").to_json())?;

            // MOD-11 (H5): the timeout drops the CLIENT response, but tokio
            // cannot cancel a `spawn_blocking` task mid-flight — the engine
            // work keeps running on the blocking pool and holds its semaphore
            // permit until it finishes. Cooperative cancellation would require
            // threading a CancellationToken through every handler (invasive,
            // regression risk), so this is a documented limitation: N hung
            // operations can saturate the pool. Acceptable for the local
            // stdio single-user server (see SKILL.md § Security).
            tokio::time::timeout(
                config.request_timeout,
                tokio::task::spawn_blocking(move || {
                    let _p = _permit;
                    let executor = Executor::new(&storage_ctx);
                    handle_tools_call(&params_ctx, &executor, &storage_ctx, &cfg)
                }),
            )
            .await
            .map_err(|_| McpError::internal_error("Request timed out").to_json())?
            .map_err(|e| McpError::internal_error(format!("Task panicked: {}", e)).to_json())?
        }
        "resources/list" => handle_resources_list(),
        "resources/read" => {
            let sem = semaphore.clone();
            let storage_ctx = storage.clone();
            let params_ctx = req.params.clone();
            let config_ctx = config.clone();

            let _permit = sem
                .acquire_owned()
                .await
                .map_err(|_| McpError::internal_error("Semaphore closed").to_json())?;

            // Same documented timeout limitation as tools/call above (MOD-11 H5).
            tokio::time::timeout(
                config.request_timeout,
                tokio::task::spawn_blocking(move || {
                    let _p = _permit;
                    handle_resources_read(&params_ctx, &storage_ctx, &config_ctx)
                }),
            )
            .await
            .map_err(|_| McpError::internal_error("Request timed out").to_json())?
            .map_err(|e| McpError::internal_error(format!("Task panicked: {}", e)).to_json())?
        }
        "prompts/list" => handle_prompts_list(),
        "prompts/get" => handle_prompts_get(req.params.as_ref()),
        _ => {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            McpError::method_not_found(format!("Method not found: {}", req.method)).into_err()
        }
    };

    let elapsed = start.elapsed();

    // `active_requests` is decremented exclusively by `ActiveRequestGuard`'s
    // Drop (see above), which runs on *every* exit path including `?`
    // early-returns and panic unwinding. No manual fetch_sub here — pairing
    // one with the guard would double-decrement the happy path and drift the
    // gauge negative by one per request.
    match &result {
        Ok(_) => debug!(elapsed_ms = elapsed.as_millis(), method = %req.method, "OK"),
        Err(_) => {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            warn!(elapsed_ms = elapsed.as_millis(), method = %req.method, "Error");
        }
    }

    result
}

// ── MCP-35 proxy dispatch (second instance) ────────────────────────────────

/// Dispatch via HTTP proxy (second N instance without lock).
/// `proxy` is the live writer's handle; local methods (initialize, tools/list)
/// are answered without HTTP for latency.
pub(crate) async fn dispatch_request_proxy(
    req: &RpcRequest,
    proxy: &crate::proxy::ProxyHandle,
    config: &McpConfig,
    semaphore: &Arc<tokio::sync::Semaphore>,
    metrics: &Arc<McpMetrics>,
) -> Result<Value, Value> {
    let _active = ActiveRequestGuard::new(&metrics.active_requests);
    let start = Instant::now();
    let result: Result<Value, Value> = match req.method.as_str() {
        "initialize" => handle_initialize(req.params.as_ref()),
        "tools/list" => handle_tools_list(config),
        "tools/call" => {
            let _permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| McpError::internal_error("Semaphore closed").to_json())?;
            // Proxy path never takes fs2 lock — reqwest only, 60s timeout on writer side
            match proxy.proxy_tools_call(&req.params).await {
                Ok(v) => Ok(v),
                Err(e) => Err(if e.get("code").is_some() {
                    e
                } else {
                    McpError::internal_error(format!("proxy error: {}", e)).to_json()
                }),
            }
        }
        "resources/list" => handle_resources_list(),
        "resources/read" => {
            let _permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| McpError::internal_error("Semaphore closed").to_json())?;
            match proxy.proxy_mcp_call("resources/read", &req.params).await {
                Ok(v) => Ok(v),
                Err(e) => Err(if e.get("code").is_some() {
                    e
                } else {
                    McpError::internal_error(format!("proxy error: {}", e)).to_json()
                }),
            }
        }
        "prompts/list" => handle_prompts_list(),
        "prompts/get" => handle_prompts_get(req.params.as_ref()),
        _ => {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            McpError::method_not_found(format!("Method not found: {}", req.method)).into_err()
        }
    };
    // Same metrics logging as dispatch_request
    match &result {
        Ok(_) => {
            debug!(elapsed_ms = start.elapsed().as_millis(), method = %req.method, "OK (proxy)")
        }
        Err(_) => {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            warn!(elapsed_ms = start.elapsed().as_millis(), method = %req.method, "Error (proxy)");
        }
    }
    result
}

/// Proxy variant of `serve_lines` — uses `dispatch_request_proxy` (HTTP) instead of local storage.
async fn serve_lines_proxy<R, W>(
    proxy: &crate::proxy::ProxyHandle,
    config: &McpConfig,
    semaphore: &Arc<tokio::sync::Semaphore>,
    metrics: &Arc<McpMetrics>,
    running: &AtomicBool,
    reader: R,
    writer: W,
) where
    R: tokio::io::AsyncRead + Unpin,
    W: tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let mut lines = BufReader::new(reader).lines();
    let stdout = Arc::new(tokio::sync::Mutex::new(writer));
    let mut inflight = tokio::task::JoinSet::new();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => break,
            Err(e) => {
                error!(error = %e, "stdin read error (proxy)");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        metrics.requests_total.fetch_add(1, Ordering::Relaxed);
        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                metrics.errors_total.fetch_add(1, Ordering::Relaxed);
                warn!(error = %e, input_len = line.len(), "Failed to parse JSON-RPC (proxy)");
                write_json(&mut *stdout.lock().await, &json!({"jsonrpc":"2.0","id": Value::Null,"error": McpError::parse_error(e.to_string()).to_json()})).await;
                continue;
            }
        };
        let Some(req_id) = &req.id else {
            debug!(method = %req.method, "notification (proxy)");
            continue;
        };
        let req_id = req_id.clone();
        if req.jsonrpc != "2.0" {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            warn!(jsonrpc = %req.jsonrpc, "Invalid JSON-RPC version (proxy)");
            write_json(&mut *stdout.lock().await, &json!({"jsonrpc":"2.0","id": req_id,"error": McpError::invalid_request(format!("Invalid JSON-RPC version: {}", req.jsonrpc)).to_json()})).await;
            continue;
        }
        let (proxy, config, semaphore, metrics) = (
            proxy.clone(),
            config.clone(),
            semaphore.clone(),
            metrics.clone(),
        );
        let stdout = stdout.clone();
        inflight.spawn(async move {
            let res = dispatch_request_proxy(&req, &proxy, &config, &semaphore, &metrics).await;
            let (result, error) = match res {
                Ok(val) => (Some(val), None),
                Err(err) => (None, Some(err)),
            };
            let response = RpcResponse {
                jsonrpc: "2.0".to_string(),
                id: req_id,
                result,
                error,
            };
            if let Ok(out) = serde_json::to_string(&response) {
                let mut guard = stdout.lock().await;
                let _ = guard.write_all(out.as_bytes()).await;
                let _ = guard.write_all(b"\n").await;
                let _ = guard.flush().await;
            }
        });
        if !running.load(Ordering::SeqCst) {
            info!("Shutdown flag set (proxy)");
            break;
        }
    }
    while inflight.join_next().await.is_some() {}
}

/// Run proxy stdio server (DatabaseBusy fallback).
pub async fn run_proxy_stdio_server(proxy: crate::proxy::ProxyHandle, config: McpConfig) {
    let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_concurrency));
    let metrics = Arc::new(McpMetrics::default());
    let running = Arc::new(AtomicBool::new(true));
    let sig_running = running.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            sig_running.store(false, Ordering::SeqCst);
        }
    });
    info!(proxy = %proxy.base_url, "MCP proxy stdio server started");
    serve_lines_proxy(
        &proxy,
        &config,
        &semaphore,
        &metrics,
        &running,
        tokio::io::stdin(),
        tokio::io::stdout(),
    )
    .await;
    info!("MCP proxy server shut down");
}

/// MCP-35 Step3: try to open writer, fallback to proxy on DatabaseBusy, stale cleanup + retry once.
/// Called by the binary's MCP entry point (main.rs) instead of raw `StorageEngine::open`.
pub async fn run_stdio_server_auto(
    storage_path: &str,
    vanta_config: Option<vantadb::config::VantaConfig>,
) -> Result<(), vantadb::VantaError> {
    let cfg = vanta_config.unwrap_or_default();
    // Clone cfg with correct storage_path (priority to explicit param)
    let mut cfg_with_path = cfg.clone();
    if !storage_path.is_empty() {
        cfg_with_path.storage_path = storage_path.to_string();
    }
    let path = cfg_with_path.storage_path.clone();
    match vantadb::storage::StorageEngine::open_with_config(&path, Some(cfg_with_path.clone())) {
        Ok(storage) => {
            let storage = Arc::new(storage);
            run_stdio_server(storage.clone()).await;
            tracing::info!("MCP server exited, flushing storage (auto)...");
            if let Err(e) = storage.flush() {
                tracing::error!("Flush failed: {}", e);
            } else {
                tracing::info!("Storage flushed");
            }
            Ok(())
        }
        Err(vantadb::VantaError::DatabaseBusy(msg)) => {
            tracing::warn!(msg = %msg, "DatabaseBusy — trying proxy fallback");
            let api_key = cfg_with_path
                .api_key
                .clone()
                .or_else(|| std::env::var("VANTADB_API_KEY").ok());
            // Try proxy (500ms health + PID alive)
            if let Some((handle, _info)) = crate::proxy::try_proxy(&path, api_key.clone()).await {
                // Proxy mode — no lock, HTTP only
                let mcp_cfg = McpConfig {
                    max_concurrency: cfg_with_path.max_blocking_threads,
                    ..Default::default()
                };
                run_proxy_stdio_server(handle, mcp_cfg).await;
                Ok(())
            } else {
                // Stale: cleanup + retry once (Step3)
                crate::proxy::cleanup_stale(&path);
                match vantadb::storage::StorageEngine::open_with_config(&path, Some(cfg_with_path))
                {
                    Ok(storage) => {
                        let storage = Arc::new(storage);
                        run_stdio_server(storage.clone()).await;
                        tracing::info!("MCP server exited after stale retry, flushing...");
                        if let Err(e) = storage.flush() {
                            tracing::error!("Flush failed: {}", e);
                        } else {
                            tracing::info!("Storage flushed");
                        }
                        Ok(())
                    }
                    Err(e @ vantadb::VantaError::DatabaseBusy(_)) => {
                        // Final error with hint pid/port if file still exists
                        let hint = std::fs::read_to_string(discovery_path(&path))
                            .ok()
                            .and_then(|c| serde_json::from_str::<ServerInfo>(&c).ok())
                            .map(|info| {
                                format!(
                                    " another writer still active pid={} port={}",
                                    info.pid, info.http_port
                                )
                            })
                            .unwrap_or_default();
                        Err(vantadb::VantaError::DatabaseBusy(format!("{};{}", e, hint)))
                    }
                    Err(e) => Err(e),
                }
            }
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Drive [`serve_lines`] with in-memory duplex pipes: write `input`,
    /// signal EOF, and return everything the server wrote back.
    async fn serve_lines_capture(input: &str) -> String {
        serve_lines_capture_with(input, AtomicBool::new(true)).await
    }

    /// Like [`serve_lines_capture`] but with an explicit `running` flag, so
    /// tests can exercise the shutdown path (MOD-09).
    async fn serve_lines_capture_with(input: &str, running: AtomicBool) -> String {
        let dir = tempfile::tempdir().expect("tempdir");
        let storage = Arc::new(
            StorageEngine::open(dir.path().to_str().expect("utf8 temp path"))
                .expect("open storage"),
        );
        let config = McpConfig::from_storage(&storage);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_concurrency));
        let metrics = Arc::new(McpMetrics::default());

        let (mut client_write, server_in) = tokio::io::duplex(1024 * 1024);
        let (server_out, mut client_read) = tokio::io::duplex(1024 * 1024);

        // Queue the whole input into the duplex buffer and signal EOF BEFORE
        // driving the server inline. No `tokio::spawn` here: the loop under
        // test is driven directly so its spawned background tasks run on the
        // same runtime and are drained before `serve_lines` returns.
        client_write
            .write_all(input.as_bytes())
            .await
            .expect("write input");
        client_write.shutdown().await.expect("shutdown input");
        drop(client_write); // EOF → the server loop breaks

        serve_lines(
            &storage, &config, &semaphore, &metrics, &running, server_in, server_out,
        )
        .await;

        use tokio::io::AsyncReadExt;
        let mut out = Vec::new();
        client_read
            .read_to_end(&mut out)
            .await
            .expect("read server output");
        String::from_utf8(out).expect("utf8 server output")
    }

    /// MOD-07 regression: a JSON-RPC notification carries no `id` and the
    /// server MUST NOT answer it (JSON-RPC 2.0 §4.1). These used to fail
    /// deserialization and come back as a spurious -32700 parse error,
    /// breaking strict MCP clients' handshake (`notifications/initialized`
    /// is mandatory per the MCP lifecycle spec).
    #[tokio::test]
    async fn notification_without_id_is_not_answered() {
        for method in [
            "notifications/initialized",
            "notifications/cancelled",
            "notifications/some_unknown_notification",
        ] {
            let line = format!("{{\"jsonrpc\":\"2.0\",\"method\":\"{method}\"}}\n");
            let out = serve_lines_capture(&line).await;
            assert!(
                out.is_empty(),
                "{method} must not produce any response, got: {out}"
            );
        }
    }

    /// Control: requests WITH an id are still answered normally.
    #[tokio::test]
    async fn request_with_id_still_answered() {
        let out =
            serve_lines_capture("{\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"tools/list\"}\n").await;
        let v: serde_json::Value =
            serde_json::from_str(out.trim()).expect("response should be JSON");
        assert_eq!(v["id"], 7);
        assert!(v["result"]["tools"].is_array(), "got: {out}");
        assert!(v["error"].is_null(), "got: {out}");
    }

    /// Control: malformed JSON still yields a -32700 parse error with null id.
    #[tokio::test]
    async fn malformed_json_still_parse_error() {
        let out = serve_lines_capture("{not json}\n").await;
        assert!(out.contains("-32700"), "expected -32700, got: {out}");
    }

    /// JSON-RPC allows `"id": null`; explicit null is still a request and
    /// must be answered (only an ABSENT id makes it a notification).
    #[tokio::test]
    async fn explicit_null_id_is_a_request_not_a_notification() {
        let out =
            serve_lines_capture("{\"jsonrpc\":\"2.0\",\"id\":null,\"method\":\"tools/list\"}\n")
                .await;
        let v: serde_json::Value =
            serde_json::from_str(out.trim()).expect("response should be JSON");
        assert!(v["id"].is_null(), "got: {out}");
        assert!(v["result"]["tools"].is_array(), "got: {out}");
    }

    /// MOD-09 regression: when shutdown is signaled while a request is in
    /// flight, the in-flight response must still be written before the loop
    /// returns. The old loop `break`-ed right after building the response,
    /// discarding it.
    #[tokio::test]
    async fn in_flight_response_written_on_shutdown() {
        let out = serve_lines_capture_with(
            "{\"jsonrpc\":\"2.0\",\"id\":9,\"method\":\"tools/list\"}\n",
            AtomicBool::new(false),
        )
        .await;
        let v: serde_json::Value =
            serde_json::from_str(out.trim()).expect("response should be JSON");
        assert_eq!(
            v["id"], 9,
            "in-flight response must be written on shutdown, got: {out}"
        );
        assert!(v["result"]["tools"].is_array(), "got: {out}");
    }

    /// MCP-36: protocol negotiation — 2025-06-18 is the latest stable.
    #[tokio::test]
    async fn initialize_negotiates_2025_06_18() {
        let out = serve_lines_capture(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1.0\"}}}\n",
        )
        .await;
        let v: serde_json::Value =
            serde_json::from_str(out.trim()).expect("response should be JSON");
        assert_eq!(v["result"]["protocolVersion"], "2025-06-18", "got: {out}");
    }

    #[tokio::test]
    async fn initialize_echoes_2024_11_05_for_old_clients() {
        let out = serve_lines_capture(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{},\"clientInfo\":{\"name\":\"test\",\"version\":\"1.0\"}}}\n",
        )
        .await;
        let v: serde_json::Value =
            serde_json::from_str(out.trim()).expect("response should be JSON");
        assert_eq!(v["result"]["protocolVersion"], "2024-11-05", "got: {out}");
    }

    /// MOD-08 regression: pipelined requests must all be answered even though
    /// the loop now dispatches them to background tasks — none may be dropped.
    #[tokio::test]
    async fn pipelined_requests_all_answered() {
        let input = [
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"resources/list\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"prompts/list\"}\n",
        ]
        .concat();
        let out = serve_lines_capture(&input).await;
        let mut ids: Vec<i64> = out
            .lines()
            .map(|l| {
                serde_json::from_str::<serde_json::Value>(l)
                    .expect("line should be JSON")
                    .get("id")
                    .and_then(serde_json::Value::as_i64)
                    .expect("id should be a number")
            })
            .collect();
        ids.sort_unstable();
        assert_eq!(
            ids,
            vec![1, 2, 3],
            "all pipelined requests must be answered: {out}"
        );
    }
}
