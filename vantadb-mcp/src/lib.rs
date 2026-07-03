//! VantaDB Model Context Protocol (MCP) Server.
//!
//! This module provides a complete MCP server implementation for VantaDB,
//! exposing tools, resources, and prompts for AI agent integration.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, span, warn, Level};
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::metadata;
use vantadb::storage::StorageEngine;

// ── Configuration ──────────────────────────────────────────────────────────

/// Tuning knobs for the MCP server.
#[derive(Clone, Debug)]
pub struct McpConfig {
    /// Max concurrent requests (default: storage engine's max_blocking_threads).
    pub max_concurrency: usize,
    /// Max payload length for memory_put (default: 1 MB).
    pub max_payload_length: usize,
    /// Max key length (default: 512).
    pub max_key_length: usize,
    /// Max namespace length (default: 256).
    pub max_namespace_length: usize,
    /// Max vector dimension (default: 16384).
    pub max_vector_dim: usize,
    /// Max LISP query length (default: 1 MB).
    pub max_query_length: usize,
    /// Per-request timeout (default: 60 s).
    pub request_timeout: Duration,
    /// Default limit for memory_list (default: 100).
    pub default_list_limit: usize,
    /// Max limit for memory_list (default: 10_000).
    pub max_list_limit: usize,
    /// Default top_k for search_memory (default: 10).
    pub default_top_k: usize,
    /// Max top_k for search_memory (default: 1000).
    pub max_top_k: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 32,
            max_payload_length: 1_048_576,
            max_key_length: 512,
            max_namespace_length: 256,
            max_vector_dim: 16_384,
            max_query_length: 1_048_576,
            request_timeout: Duration::from_secs(60),
            default_list_limit: 100,
            max_list_limit: 10_000,
            default_top_k: 10,
            max_top_k: 1000,
        }
    }
}

impl McpConfig {
    /// Build from a StorageEngine, taking max_concurrency from it.
    pub fn from_storage(storage: &StorageEngine) -> Self {
        Self {
            max_concurrency: storage.config.max_blocking_threads,
            ..Default::default()
        }
    }
}

// ── Error type ─────────────────────────────────────────────────────────────

/// Structured JSON-RPC error.
#[derive(Debug)]
pub struct McpError {
    pub code: i32,
    pub message: String,
}

impl McpError {
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: format!("Parse error: {}", msg.into()),
        }
    }

    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
        }
    }

    pub fn method_not_found(msg: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: msg.into(),
        }
    }

    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
        }
    }

    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: msg.into(),
        }
    }

    pub fn to_json(&self) -> Value {
        json!({"code": self.code, "message": self.message})
    }

    pub fn into_err<T>(self) -> Result<T, Value> {
        Err(self.to_json())
    }
}

// ── JSON-RPC wire types ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: Value,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize)]
struct RpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

// ── Request-scoped metrics ─────────────────────────────────────────────────

#[derive(Default)]
struct McpMetrics {
    requests_total: AtomicU64,
    errors_total: AtomicU64,
    active_requests: AtomicU64,
}

// ── Input validation helpers ───────────────────────────────────────────────

fn validate_identifier(value: &str, label: &str, max_len: usize) -> Result<(), McpError> {
    if value.is_empty() {
        return Err(McpError::invalid_params(format!(
            "'{}' must not be empty",
            label
        )));
    }
    if value.len() > max_len {
        return Err(McpError::invalid_params(format!(
            "'{}' exceeds maximum length of {} bytes",
            label, max_len
        )));
    }
    if value.contains('\0') {
        return Err(McpError::invalid_params(format!(
            "'{}' contains null byte",
            label
        )));
    }
    Ok(())
}

fn validate_payload(value: &str, max_len: usize) -> Result<(), McpError> {
    if value.len() > max_len {
        return Err(McpError::invalid_params(format!(
            "Payload exceeds maximum length of {} bytes",
            max_len
        )));
    }
    Ok(())
}

fn validate_vector(array: &[Value], max_dim: usize) -> Result<Vec<f32>, McpError> {
    if array.is_empty() {
        return Err(McpError::invalid_params("Vector must not be empty"));
    }
    if array.len() > max_dim {
        return Err(McpError::invalid_params(format!(
            "Vector dimension {} exceeds maximum {}",
            array.len(),
            max_dim
        )));
    }
    let mut v = Vec::with_capacity(array.len());
    for val in array {
        let f = val
            .as_f64()
            .ok_or_else(|| McpError::invalid_params("Vector elements must be numbers"))?;
        if !f.is_finite() {
            return Err(McpError::invalid_params(
                "Vector elements must be finite numbers",
            ));
        }
        v.push(f as f32);
    }
    Ok(v)
}

fn parse_metadata(obj: &serde_json::Map<String, Value>) -> vantadb::sdk::VantaMemoryMetadata {
    let mut meta = vantadb::sdk::VantaMemoryMetadata::new();
    for (key, val) in obj {
        if let Some(s) = val.as_str() {
            meta.insert(key.clone(), vantadb::sdk::VantaValue::String(s.to_string()));
        } else if let Some(b) = val.as_bool() {
            meta.insert(key.clone(), vantadb::sdk::VantaValue::Bool(b));
        } else if let Some(i) = val.as_i64() {
            meta.insert(key.clone(), vantadb::sdk::VantaValue::Int(i));
        } else if let Some(f) = val.as_f64() {
            meta.insert(key.clone(), vantadb::sdk::VantaValue::Float(f));
        }
    }
    meta
}

/// Serialize value to JSON string; on error produces a JSON-error string rather
/// than silently returning "".
fn serialize_content(value: &impl Serialize) -> String {
    serde_json::to_string(value)
        .unwrap_or_else(|e| format!("{{\"error\":\"Serialization failed: {}\"}}", e))
}

fn text_content(text: String) -> Value {
    json!({"content": [{"type": "text", "text": text}]})
}

fn error_content(msg: impl Into<String>) -> Value {
    json!({"isError": true, "content": [{"type": "text", "text": msg.into()}]})
}

/// Collect all memory records from a namespace via cursor-paginated list.
fn collect_all_records(
    embedded: &vantadb::VantaEmbedded,
    namespace: &str,
    config: &McpConfig,
) -> Result<Vec<vantadb::sdk::VantaMemoryRecord>, String> {
    let mut all_records = Vec::new();
    let mut cursor: Option<usize> = None;
    loop {
        let options = vantadb::sdk::VantaMemoryListOptions {
            limit: config.max_list_limit,
            cursor,
            filters: vantadb::sdk::VantaMemoryMetadata::new(),
        };
        match embedded.list(namespace, options) {
            Ok(page) => {
                let count = page.records.len();
                all_records.extend(page.records);
                if count == 0 {
                    break;
                }
                if let Some(next) = page.next_cursor {
                    cursor = Some(next);
                } else {
                    break;
                }
            }
            Err(e) => return Err(format!("{}", e)),
        }
    }
    Ok(all_records)
}

// ── Stdio server (main entry point) ───────────────────────────────────────

/// Run the MCP server over stdin/stdout (JSON-RPC 2.0).
///
/// Supports graceful shutdown via SIGINT/Ctrl-C.  All blocking operations
/// are dispatched through a tokio blocking pool with a concurrency semaphore
/// and an optional per-request timeout.
pub async fn run_stdio_server(storage: Arc<StorageEngine>) {
    let config = McpConfig::from_storage(&storage);
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

    info!(
        max_concurrency = config.max_concurrency,
        request_timeout_ms = config.request_timeout.as_millis(),
        "MCP stdio server started"
    );

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        if !running.load(Ordering::SeqCst) {
            info!("Shutdown flag set, draining remaining requests");
        }

        let line = match line {
            Ok(l) => l,
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
                    &mut stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": Value::Null,
                        "error": McpError::parse_error(e.to_string()).to_json()
                    }),
                );
                continue;
            }
        };

        if req.jsonrpc != "2.0" {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            warn!("Invalid jsonrpc version: {:?}", req.jsonrpc);
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

        if let Ok(out) = serde_json::to_string(&response) {
            let _ = writeln!(stdout, "{}", out);
            let _ = stdout.flush();
        } else {
            error!("Failed to serialize JSON-RPC response body");
        }
    }

    info!(
        total = metrics.requests_total.load(Ordering::Relaxed),
        errors = metrics.errors_total.load(Ordering::Relaxed),
        "MCP stdio server shut down"
    );
}

/// Write a JSON value to stdout, swallowing I/O errors.
fn write_json(stdout: &mut io::Stdout, value: &Value) {
    if let Ok(out) = serde_json::to_string(value) {
        let _ = writeln!(stdout, "{}", out);
        let _ = stdout.flush();
    }
}

/// Route a parsed JSON-RPC request, enforcing concurrency limits, timeouts and
/// instrumentation.
async fn dispatch_request(
    req: &RpcRequest,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
    semaphore: &Arc<tokio::sync::Semaphore>,
    metrics: &Arc<McpMetrics>,
) -> Result<Value, Value> {
    let _span = span!(Level::INFO, "mcp_request", method = %req.method, id = %req.id).entered();

    metrics.active_requests.fetch_add(1, Ordering::Relaxed);
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

            let _permit = sem
                .acquire_owned()
                .await
                .map_err(|_| McpError::internal_error("Semaphore closed").to_json())?;

            tokio::time::timeout(
                config.request_timeout,
                tokio::task::spawn_blocking(move || {
                    let _p = _permit;
                    handle_resources_read(&params_ctx, &storage_ctx)
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
    metrics.active_requests.fetch_sub(1, Ordering::Relaxed);

    match &result {
        Ok(_) => debug!(elapsed_ms = elapsed.as_millis(), method = %req.method, "OK"),
        Err(_) => {
            metrics.errors_total.fetch_add(1, Ordering::Relaxed);
            warn!(elapsed_ms = elapsed.as_millis(), method = %req.method, "Error");
        }
    }

    result
}

// ── initialize handler ────────────────────────────────────────────────────

pub fn handle_initialize() -> Result<Value, Value> {
    Ok(json!({
        "protocolVersion": "2024-11-05",
        "serverInfo": {
            "name": metadata::MCP_SERVER_INFO_NAME,
            "version": metadata::reported_version().into_owned()
        },
        "capabilities": {
            "tools": {},
            "resources": {},
            "prompts": {}
        }
    }))
}

// ── Resources handlers ────────────────────────────────────────────────────

pub fn handle_resources_list() -> Result<Value, Value> {
    Ok(json!({
        "resources": [
            {
                "uri": "metrics://",
                "name": "Operational Metrics",
                "description": "Current operational metrics including memory usage, HNSW statistics, and storage information",
                "mimeType": "application/json"
            },
            {
                "uri": "schema://",
                "name": "Database Schema",
                "description": "Schema information for the VantaDB database including text index version and configuration",
                "mimeType": "application/json"
            }
        ]
    }))
}

pub fn handle_resources_read(
    params: &Option<Value>,
    storage: &Arc<StorageEngine>,
) -> Result<Value, Value> {
    let p = params
        .as_ref()
        .ok_or_else(|| McpError::invalid_params("Missing params").to_json())?;

    let uri = p["uri"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing 'uri'").to_json())?;

    if uri == "metrics://" {
        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        let metrics_val = embedded.operational_metrics();
        let text = serialize_content(&metrics_val);
        Ok(json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]}))
    } else if uri == "schema://" {
        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        let metrics_val = embedded.operational_metrics();
        let schema_info = json!({
            "hnsw_nodes_count": metrics_val.hnsw_nodes_count,
            "hnsw_logical_bytes": metrics_val.hnsw_logical_bytes,
            "mmap_resident_bytes": metrics_val.mmap_resident_bytes
        });
        let text = serialize_content(&schema_info);
        Ok(json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]}))
    } else if uri.starts_with("memory://") {
        let path = uri.strip_prefix("memory://").unwrap_or("");
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() != 2 {
            return McpError::invalid_params(
                "Invalid memory URI format. Expected: memory://namespace/key",
            )
            .into_err();
        }
        let namespace = parts[0];
        let key = parts[1];

        if let Err(e) = validate_identifier(namespace, "namespace", 256) {
            return e.into_err();
        }
        if let Err(e) = validate_identifier(key, "key", 512) {
            return e.into_err();
        }

        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        match embedded.get(namespace, key) {
            Ok(Some(record)) => {
                let text = serialize_content(&record);
                Ok(
                    json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]}),
                )
            }
            Ok(None) => McpError::invalid_params("Memory record not found").into_err(),
            Err(e) => McpError::internal_error(format!("Error reading memory: {}", e)).into_err(),
        }
    } else if uri.starts_with("namespace://") {
        let namespace = uri.strip_prefix("namespace://").unwrap_or("");
        if namespace.is_empty() {
            return McpError::invalid_params(
                "Invalid namespace URI format. Expected: namespace://namespace",
            )
            .into_err();
        }
        if let Err(e) = validate_identifier(namespace, "namespace", 256) {
            return e.into_err();
        }

        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        let options = vantadb::sdk::VantaMemoryListOptions {
            limit: 100,
            cursor: None,
            filters: vantadb::sdk::VantaMemoryMetadata::new(),
        };
        match embedded.list(namespace, options) {
            Ok(page) => {
                let result = json!({
                    "namespace": namespace,
                    "records": page.records,
                    "next_cursor": page.next_cursor
                });
                let text = serialize_content(&result);
                Ok(
                    json!({"contents": [{"uri": uri, "mimeType": "application/json", "text": text}]}),
                )
            }
            Err(e) => {
                McpError::internal_error(format!("Error listing namespace: {}", e)).into_err()
            }
        }
    } else {
        McpError::method_not_found("Resource not found").into_err()
    }
}

// ── Prompts handlers ──────────────────────────────────────────────────────

pub fn handle_prompts_list() -> Result<Value, Value> {
    Ok(json!({
        "prompts": [
            {
                "name": "search_memory",
                "description": "Optimized prompt for searching memory records with hybrid vector and text search",
                "arguments": [
                    { "name": "namespace", "description": "Target namespace for search", "required": true },
                    { "name": "query", "description": "Search query (text or vector)", "required": true },
                    { "name": "filters", "description": "Optional metadata filters", "required": false }
                ]
            },
            {
                "name": "analyze_namespace",
                "description": "Analyze the content and structure of a namespace",
                "arguments": [
                    { "name": "namespace", "description": "Namespace to analyze", "required": true }
                ]
            },
            {
                "name": "summarize_context",
                "description": "Generate a summary of context from memory records",
                "arguments": [
                    { "name": "namespace", "description": "Source namespace", "required": true },
                    { "name": "limit", "description": "Number of records to include", "required": false }
                ]
            },
            {
                "name": "query_builder",
                "description": "Build IQL queries for VantaDB",
                "arguments": [
                    { "name": "operation", "description": "Operation type (SELECT, INSERT, UPDATE, DELETE)", "required": true },
                    { "name": "target", "description": "Target (nodes, memory, etc.)", "required": true },
                    { "name": "conditions", "description": "Query conditions", "required": false }
                ]
            }
        ]
    }))
}

pub fn handle_prompts_get(params: Option<&Value>) -> Result<Value, Value> {
    let p = params.ok_or_else(|| McpError::invalid_params("Missing params").to_json())?;
    let name = p["name"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing 'name'").to_json())?;

    let args = p.get("arguments");

    match name {
        "search_memory" => {
            let namespace = args
                .and_then(|a| a["namespace"].as_str())
                .unwrap_or("default");
            let query = args.and_then(|a| a["query"].as_str()).unwrap_or("");
            Ok(json!({
                "description": "Optimized prompt for searching memory records with hybrid vector and text search",
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Search the VantaDB memory in namespace '{}' for: '{}'. Use hybrid search combining vector similarity and lexical matching. Apply any specified filters and return the top K results with confidence scores.", namespace, query)}}]
            }))
        }
        "analyze_namespace" => {
            let namespace = args
                .and_then(|a| a["namespace"].as_str())
                .unwrap_or("default");
            Ok(json!({
                "description": "Analyze the content and structure of a namespace",
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Analyze the VantaDB namespace '{}'. List all records, examine metadata patterns, identify clusters, and provide insights about the namespace structure and content distribution.", namespace)}}]
            }))
        }
        "summarize_context" => {
            let namespace = args
                .and_then(|a| a["namespace"].as_str())
                .unwrap_or("default");
            let limit = args.and_then(|a| a["limit"].as_u64()).unwrap_or(10);
            Ok(json!({
                "description": "Generate a summary of context from memory records",
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Retrieve the last {} records from namespace '{}' and generate a comprehensive summary of the context, identifying key themes, relationships, and important information.", limit, namespace)}}]
            }))
        }
        "query_builder" => {
            let operation = args
                .and_then(|a| a["operation"].as_str())
                .unwrap_or("SELECT");
            let target = args.and_then(|a| a["target"].as_str()).unwrap_or("nodes");
            let conditions = args.and_then(|a| a["conditions"].as_str()).unwrap_or("");
            Ok(json!({
                "description": "Build IQL queries for VantaDB",
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Build an IQL query for VantaDB. Operation: {}, Target: {}, Conditions: {}. Ensure the query follows VantaLISP/IQL syntax and is properly formatted.", operation, target, conditions)}}]
            }))
        }
        _ => McpError::invalid_params(format!("Prompt not found: {}", name)).into_err(),
    }
}

// ── Tools handler ─────────────────────────────────────────────────────────

pub fn handle_tools_list() -> Result<Value, Value> {
    Ok(json!({
        "tools": [
            {
                "name": "memory_put",
                "description": "Inserts or updates a memory record in a namespace with payload, vector, and optional metadata.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string", "description": "Target namespace" },
                        "key": { "type": "string", "description": "Unique key for the record" },
                        "payload": { "type": "string", "description": "Text content of the memory" },
                        "vector": { "type": "array", "items": {"type": "number"}, "description": "Optional embedding vector" },
                        "metadata": { "type": "object", "description": "Optional metadata key-value pairs" }
                    },
                    "required": ["namespace", "key", "payload"]
                }
            },
            {
                "name": "memory_get",
                "description": "Retrieves a memory record by namespace and key.",
                "inputSchema": {
                    "type": "object", "properties": {
                        "namespace": { "type": "string" }, "key": { "type": "string" }
                    }, "required": ["namespace", "key"]
                }
            },
            {
                "name": "memory_delete",
                "description": "Deletes a memory record by namespace and key.",
                "inputSchema": {
                    "type": "object", "properties": {
                        "namespace": { "type": "string" }, "key": { "type": "string" }
                    }, "required": ["namespace", "key"]
                }
            },
            {
                "name": "memory_list",
                "description": "Lists memory records in a namespace with optional pagination and filters.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string" },
                        "limit": { "type": "number", "description": "Max records, default 100" },
                        "cursor": { "type": "number", "description": "Optional pagination cursor" },
                        "filters": { "type": "object", "description": "Optional metadata filters" }
                    },
                    "required": ["namespace"]
                }
            },
            {
                "name": "memory_list_namespaces",
                "description": "Lists all available namespaces in the database.",
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "query_lisp",
                "description": "Executes VantaLISP code. Allows reading structures and inserting/mutating Nodes providing semantic context.",
                "inputSchema": {
                    "type": "object", "properties": {
                        "query": { "type": "string", "description": "VantaLISP program or statement" }
                    }, "required": ["query"]
                }
            },
            {
                "name": "search_semantic",
                "description": "Raw semantic vector search directly in the HNSW index.",
                "inputSchema": {
                    "type": "object", "properties": {
                        "vector": { "type": "array", "items": {"type": "number"}, "description": "F32 query vector" },
                        "k": { "type": "number", "description": "Top K neighbors" }
                    }, "required": ["vector", "k"]
                }
            },
            {
                "name": "search_memory",
                "description": "Performs memory search in a given namespace supporting optional text queries, filters, distance metric, and explain.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string" },
                        "query_vector": { "type": "array", "items": {"type": "number"} },
                        "text_query": { "type": "string" },
                        "top_k": { "type": "number", "description": "Top K hits, default 10" },
                        "distance_metric": { "type": "string", "enum": ["cosine", "euclidean"] },
                        "explain": { "type": "boolean" },
                        "filters": { "type": "object" }
                    },
                    "required": ["namespace"]
                }
            },
            {
                "name": "get_node_neighbors",
                "description": "Inspects neighbors or lineage of a node.",
                "inputSchema": {
                    "type": "object", "properties": {
                        "node_id": { "type": "number", "description": "Node ID to explore" }
                    }, "required": ["node_id"]
                }
            },
            {
                "name": "inject_context",
                "description": "Injects external state or context connecting it to a specific thread for subsequent consolidation.",
                "inputSchema": {
                    "type": "object", "properties": {
                        "content": { "type": "string", "description": "Context content" },
                        "thread_id": { "type": "number", "description": "Thread ID it belongs to" }
                    }, "required": ["content", "thread_id"]
                }
            },
            {
                "name": "read_axioms",
                "description": "Returns the active Devil's Advocate Axioms (Iron Axioms) in the database.",
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "collection_stats",
                "description": "Returns statistics for a namespace/collection including record count, byte size, vector index info, and creation time.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string", "description": "Target namespace" }
                    },
                    "required": ["namespace"]
                }
            },
            {
                "name": "collection_list",
                "description": "Lists all collections with metadata including record count, vector index status, and creation time.",
                "inputSchema": { "type": "object", "properties": {}, "required": [] }
            },
            {
                "name": "collection_delete",
                "description": "Deletes an entire namespace/collection and all its records. Requires 'confirm' set to 'yes'.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string", "description": "Target namespace to delete" },
                        "confirm": { "type": "string", "description": "Must be 'yes' to confirm deletion" }
                    },
                    "required": ["namespace", "confirm"]
                }
            }
        ]
    }))
}

/// Dispatch a `tools/call` request, validating inputs against config limits.
pub fn handle_tools_call(
    params: &Option<Value>,
    executor: &Executor<'_>,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let p = params
        .as_ref()
        .ok_or_else(|| McpError::invalid_params("Missing params").to_json())?;
    let name = p["name"].as_str().unwrap_or("");
    let args = &p["arguments"];

    match name {
        "memory_put" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            let key = args["key"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'key'").to_json())?;
            let payload = args["payload"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'payload'").to_json())?;

            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;
            validate_identifier(key, "key", config.max_key_length).map_err(|e| e.to_json())?;
            validate_payload(payload, config.max_payload_length).map_err(|e| e.to_json())?;

            let vector = if let Some(arr) = args["vector"].as_array() {
                Some(validate_vector(arr, config.max_vector_dim).map_err(|e| e.to_json())?)
            } else {
                None
            };

            let metadata = if let Some(obj) = args["metadata"].as_object() {
                parse_metadata(obj)
            } else {
                vantadb::sdk::VantaMemoryMetadata::new()
            };

            let input = vantadb::sdk::VantaMemoryInput {
                key: key.to_string(),
                namespace: namespace.to_string(),
                payload: payload.to_string(),
                vector,
                metadata,
                ttl_ms: None,
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.put(input) {
                Ok(record) => Ok(text_content(serialize_content(&record))),
                Err(e) => Ok(error_content(format!("Put Error: {}", e))),
            }
        }

        "memory_get" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            let key = args["key"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'key'").to_json())?;

            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;
            validate_identifier(key, "key", config.max_key_length).map_err(|e| e.to_json())?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.get(namespace, key) {
                Ok(Some(record)) => Ok(text_content(serialize_content(&record))),
                Ok(None) => Ok(error_content("Record not found")),
                Err(e) => Ok(error_content(format!("Get Error: {}", e))),
            }
        }

        "memory_delete" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            let key = args["key"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'key'").to_json())?;

            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;
            validate_identifier(key, "key", config.max_key_length).map_err(|e| e.to_json())?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.delete(namespace, key) {
                Ok(deleted) => Ok(text_content(serialize_content(
                    &json!({"deleted": deleted}),
                ))),
                Err(e) => Ok(error_content(format!("Delete Error: {}", e))),
            }
        }

        "memory_list" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;

            let raw_limit = args["limit"]
                .as_u64()
                .unwrap_or(config.default_list_limit as u64);
            let limit = (raw_limit as usize).min(config.max_list_limit);
            let cursor = args["cursor"].as_u64().map(|c| c as usize);

            let filters = if let Some(obj) = args["filters"].as_object() {
                parse_metadata(obj)
            } else {
                vantadb::sdk::VantaMemoryMetadata::new()
            };

            let options = vantadb::sdk::VantaMemoryListOptions {
                limit,
                cursor,
                filters,
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.list(namespace, options) {
                Ok(page) => {
                    let result = json!({"records": page.records, "next_cursor": page.next_cursor});
                    Ok(text_content(serialize_content(&result)))
                }
                Err(e) => Ok(error_content(format!("List Error: {}", e))),
            }
        }

        "memory_list_namespaces" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.list_namespaces() {
                Ok(namespaces) => Ok(text_content(serialize_content(&namespaces))),
                Err(e) => Ok(error_content(format!("List Namespaces Error: {}", e))),
            }
        }

        "query_lisp" => {
            let query = args["query"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'query'").to_json())?;

            if query.len() > config.max_query_length {
                return Ok(error_content(format!(
                    "Query exceeds maximum length of {} bytes",
                    config.max_query_length
                )));
            }

            match executor.execute_hybrid(query) {
                Ok(ExecutionResult::Read(nodes)) => Ok(text_content(serialize_content(&nodes))),
                Ok(ExecutionResult::Write {
                    affected_nodes,
                    message,
                    node_id,
                }) => Ok(text_content(serialize_content(&json!({
                    "affected_nodes": affected_nodes,
                    "message": message,
                    "node_id": node_id
                })))),
                Ok(ExecutionResult::StaleContext(summary_id)) => {
                    Ok(text_content(serialize_content(&json!({
                        "stale_context": true,
                        "rehydration_available": true,
                        "summary_id": summary_id,
                        "message": "Suggested Historical Recovery (Critical Confidence Score)."
                    }))))
                }
                Err(e) => Ok(error_content(format!("LISP Runtime Error: {}", e))),
            }
        }

        "search_memory" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;

            let query_vector = if let Some(arr) = args["query_vector"].as_array() {
                if arr.is_empty() {
                    Vec::new()
                } else {
                    validate_vector(arr, config.max_vector_dim).map_err(|e| e.to_json())?
                }
            } else {
                Vec::new()
            };

            let text_query = args["text_query"].as_str().map(String::from);
            let raw_top_k = args["top_k"]
                .as_u64()
                .unwrap_or(config.default_top_k as u64);
            let top_k = (raw_top_k as usize).min(config.max_top_k);

            let distance_metric = match args["distance_metric"].as_str() {
                Some("euclidean") => vantadb::DistanceMetric::Euclidean,
                _ => vantadb::DistanceMetric::Cosine,
            };

            let explain = args["explain"].as_bool().unwrap_or(false);

            let filters = if let Some(obj) = args["filters"].as_object() {
                parse_metadata(obj)
            } else {
                vantadb::sdk::VantaMemoryMetadata::new()
            };

            let request = vantadb::sdk::VantaMemorySearchRequest {
                namespace: namespace.to_string(),
                query_vector,
                filters,
                text_query,
                top_k,
                distance_metric,
                explain,
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.search(request) {
                Ok(hits) => Ok(text_content(serialize_content(&hits))),
                Err(e) => Ok(error_content(format!("Search Error: {}", e))),
            }
        }

        "search_semantic" => {
            let vec_arr = args["vector"]
                .as_array()
                .ok_or_else(|| McpError::invalid_params("Missing 'vector' array").to_json())?;
            let vector =
                validate_vector(vec_arr, config.max_vector_dim).map_err(|e| e.to_json())?;
            let k = args["k"].as_u64().unwrap_or(5) as usize;

            let mut results = Vec::new();
            let index = storage.hnsw.load();
            let vs = storage.vector_store.read();
            let neighbors = index.search_nearest(&vector, None, None, 0, k, Some(&vs));
            for (id, distance) in neighbors {
                if let Ok(Some(node)) = storage.get(id) {
                    results.push(json!({"id": id, "distance": distance, "node": node}));
                }
            }
            Ok(text_content(serialize_content(&results)))
        }

        "get_node_neighbors" => {
            let node_id = args["node_id"]
                .as_u64()
                .ok_or_else(|| McpError::invalid_params("Missing 'node_id'").to_json())?;

            if let Ok(Some(node)) = storage.get(node_id) {
                let mut neighbors = Vec::new();
                for edge in &node.edges {
                    if let Ok(Some(target_node)) = storage.get(edge.target) {
                        neighbors.push(json!({
                            "rel": edge.label,
                            "target_id": edge.target,
                            "target_confidence": target_node.confidence_score,
                            "target_priority": target_node.importance
                        }));
                    }
                }
                Ok(text_content(serialize_content(
                    &json!({"node": node, "neighbors": neighbors}),
                )))
            } else {
                Ok(error_content("Node not found"))
            }
        }

        "inject_context" => {
            let content = args["content"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'content'").to_json())?;
            let thread_id = args["thread_id"]
                .as_u64()
                .ok_or_else(|| McpError::invalid_params("Missing 'thread_id'").to_json())?;

            if content.len() > config.max_payload_length {
                return Ok(error_content(format!(
                    "Content exceeds maximum length of {} bytes",
                    config.max_payload_length
                )));
            }

            let escaped_content = content.replace('\\', "\\\\").replace('"', "\\\"");
            let query = format!(
                "INSERT MESSAGE SYSTEM \"{}\" TO THREAD#{}",
                escaped_content, thread_id
            );

            match executor.execute_hybrid(&query) {
                Ok(ExecutionResult::Write {
                    affected_nodes,
                    message,
                    ..
                }) => Ok(text_content(serialize_content(&json!({
                    "affected_nodes": affected_nodes,
                    "message": message,
                    "status": "Context Anchored"
                })))),
                Ok(_) => Ok(error_content("Unexpected read result for insert")),
                Err(e) => Ok(error_content(format!("Execution Error: {}", e))),
            }
        }

        "read_axioms" => {
            let axioms = json!([
                {"id": 1, "name": "Topological Axiom", "description": "References (edges) to orphan nodes or nodes in Tombstone storage are not allowed."},
                {"id": 2, "name": "Confidence Constraint", "description": "Divergent vector mutations with high historical Confidence Score are rejected."},
                {"id": 3, "name": "Immortal Axiom", "description": "Maintenance: Nodes marked as PINNED evade degradation by Data Decay."},
                {"id": 4, "name": "Resource Allocation", "description": "Maintenance: 5% of memory reserved for nodes with semantic priority >= 0.8."}
            ]);
            Ok(text_content(serialize_content(&axioms)))
        }

        "collection_stats" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;

            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let metrics = embedded.operational_metrics();

            let records = match collect_all_records(&embedded, namespace, config) {
                Ok(r) => r,
                Err(e) => return Ok(error_content(format!("Collection stats error: {}", e))),
            };

            let total_records = records.len();
            let total_bytes: usize = records.iter().map(|r| {
                r.payload.len()
                    + r.metadata.iter().fold(0, |acc, (k, v)| {
                        acc + k.len() + format!("{:?}", v).len()
                    })
            }).sum();
            let vector_count = records.iter().filter(|r| r.vector.is_some()).count();
            let created_at = records.iter().map(|r| r.created_at_ms).min().unwrap_or(0);

            let result = json!({
                "total_records": total_records,
                "total_bytes": total_bytes,
                "has_vector_index": metrics.hnsw_nodes_count > 0,
                "vector_count": vector_count,
                "created_at": created_at,
            });
            Ok(text_content(serialize_content(&result)))
        }

        "collection_list" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());

            let namespaces = match embedded.list_namespaces() {
                Ok(ns) => ns,
                Err(e) => return Ok(error_content(format!("List collections error: {}", e))),
            };

            let mut collections = Vec::new();
            for ns in &namespaces {
                let records = match collect_all_records(&embedded, ns, config) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

                let record_count = records.len();
                let has_vector = records.iter().any(|r| r.vector.is_some());
                let created_at = records.iter().map(|r| r.created_at_ms).min().unwrap_or(0);

                collections.push(json!({
                    "name": ns,
                    "record_count": record_count,
                    "has_vector_index": has_vector,
                    "created_at": created_at,
                }));
            }

            Ok(text_content(serialize_content(&collections)))
        }

        "collection_delete" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            let confirm = args["confirm"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'confirm' (must be 'yes')").to_json())?;

            if confirm != "yes" {
                return Ok(error_content("Confirmation required: set 'confirm' to 'yes'"));
            }

            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());

            let records = match collect_all_records(&embedded, namespace, config) {
                Ok(r) => r,
                Err(e) => return Ok(error_content(format!("Collection delete error: {}", e))),
            };

            let records_removed = records.len();
            for record in &records {
                if let Err(e) = embedded.delete(namespace, &record.key) {
                    warn!(error = %e, key = %record.key, "Failed to delete record during collection_delete");
                }
            }

            let result = json!({
                "deleted": true,
                "records_removed": records_removed,
            });
            Ok(text_content(serialize_content(&result)))
        }

        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}
