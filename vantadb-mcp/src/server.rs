//! Stdio server loop and JSON-RPC request dispatch.

use crate::config::McpConfig;
use crate::error::McpError;
use crate::handlers::initialize::*;
use crate::handlers::prompts::*;
use crate::handlers::resources::*;
use crate::handlers::tools::*;
use crate::metrics::{ActiveRequestGuard, McpMetrics};
use crate::protocol::{RpcRequest, RpcResponse};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{debug, error, info, warn};
use vantadb::executor::Executor;
use vantadb::storage::StorageEngine;

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
        "tools/list" => handle_tools_list(),
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
