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
use tracing::{debug, error, info, span, warn, Level};
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

    let mut stdout = tokio::io::stdout();
    let mut lines = BufReader::new(tokio::io::stdin()).lines();

    loop {
        let line = match lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => break,
            Err(e) => {
                error!(error = %e, "stdin read error");
                break;
            }
        };
        if !running.load(Ordering::SeqCst) {
            info!("Shutdown flag set, draining remaining requests");
        }

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
                    &mut stdout,
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

        if req.jsonrpc != "2.0" {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            warn!(jsonrpc = %req.jsonrpc, "Invalid JSON-RPC version, expected 2.0");
            write_json(
                &mut stdout,
                &json!({
                    "jsonrpc": "2.0",
                    "id": req.id,
                    "error": McpError::invalid_request(format!(
                        "Invalid JSON-RPC version: {}, expected 2.0", req.jsonrpc
                    )).to_json()
                }),
            )
            .await;
            continue;
        }

        let res = dispatch_request(&req, &storage, &config, &semaphore, &metrics).await;
        let (result, error) = match res {
            Ok(val) => (Some(val), None),
            Err(err) => (None, Some(err)),
        };

        let response = RpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result,
            error,
        };

        if !running.load(Ordering::SeqCst) {
            info!("Graceful shutdown after processing in-flight request");
            break;
        }

        match serde_json::to_string(&response) {
            Ok(out) => {
                if let Err(e) = stdout.write_all(out.as_bytes()).await {
                    error!(error = %e, "Failed to write response to stdout");
                } else if let Err(e) = stdout.write_all(b"\n").await {
                    error!(error = %e, "Failed to write newline to stdout");
                } else if let Err(e) = stdout.flush().await {
                    error!(error = %e, "Failed to flush stdout");
                }
            }
            Err(e) => {
                error!(error = %e, "Failed to serialize JSON-RPC response body");
            }
        }
    }

    info!(
        total = metrics.requests_total.load(Ordering::Relaxed),
        errors = metrics.errors_total.load(Ordering::Relaxed),
        "MCP stdio server shut down"
    );
}

/// Write a JSON value to stdout, logging I/O errors instead of swallowing them.
pub(crate) async fn write_json(stdout: &mut tokio::io::Stdout, value: &Value) {
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
    let _span = span!(Level::INFO, "mcp_request", method = %req.method, id = %req.id).entered();

    let _active = ActiveRequestGuard::new(&metrics.active_requests);
    let start = Instant::now();

    let result = match req.method.as_str() {
        "initialize" => handle_initialize(),
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
