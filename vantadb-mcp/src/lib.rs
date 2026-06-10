//! VantaDB Model Context Protocol (MCP) Server.
//!
//! This module provides a complete MCP server implementation for VantaDB,
//! exposing tools, resources, and prompts for AI agent integration.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::sync::Arc;
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::metadata;
use vantadb::storage::StorageEngine;

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

fn error_code(code: i32, message: &str) -> Result<Value, Value> {
    Err(json!({"code": code, "message": message}))
}

pub async fn run_stdio_server(storage: Arc<StorageEngine>) {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let max_threads = storage.config.max_blocking_threads;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_threads));

    // Main Stdio loop (JSON-RPC)
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let req: RpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[MCP] Error (stderr): Unparseable JSON-RPC: {}", e);
                let err_res = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    }
                });
                if let Ok(out) = serde_json::to_string(&err_res) {
                    let _ = writeln!(stdout, "{}", out);
                    let _ = stdout.flush();
                }
                continue;
            }
        };

        if req.jsonrpc != "2.0" {
            continue;
        }

        let res = match req.method.as_str() {
            "initialize" => handle_initialize(),
            "tools/list" => handle_tools_list(),
            "tools/call" => {
                let sem = semaphore.clone();
                let storage_ctx = storage.clone();
                let params_ctx = req.params.clone();

                match sem.acquire_owned().await {
                    Ok(permit) => {
                        let spawn_res = tokio::task::spawn_blocking(move || {
                            let _permit = permit;
                            let executor = Executor::new(&storage_ctx);
                            handle_tools_call(&params_ctx, &executor, &storage_ctx)
                        })
                        .await;

                        match spawn_res {
                            Ok(r) => r,
                            Err(e) => {
                                error_code(-32603, &format!("Internal error: task panicked: {}", e))
                            }
                        }
                    }
                    Err(_) => error_code(-32603, "Internal error: semaphore closed"),
                }
            }
            "resources/list" => handle_resources_list(),
            "resources/read" => {
                let sem = semaphore.clone();
                let storage_ctx = storage.clone();
                let params_ctx = req.params.clone();

                match sem.acquire_owned().await {
                    Ok(permit) => {
                        let spawn_res = tokio::task::spawn_blocking(move || {
                            let _permit = permit;
                            handle_resources_read(&params_ctx, &storage_ctx)
                        })
                        .await;

                        match spawn_res {
                            Ok(r) => r,
                            Err(e) => {
                                error_code(-32603, &format!("Internal error: task panicked: {}", e))
                            }
                        }
                    }
                    Err(_) => error_code(-32603, "Internal error: semaphore closed"),
                }
            }
            "prompts/list" => handle_prompts_list(),
            "prompts/get" => handle_prompts_get(),
            _ => error_code(-32601, "Method not found"),
        };

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
        }
    }
}

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

pub fn handle_resources_read(params: &Option<Value>, storage: &Arc<StorageEngine>) -> Result<Value, Value> {
    let p = params
        .as_ref()
        .ok_or_else(|| json!({"code": -32602, "message": "Missing params"}))?;
    let uri = p["uri"].as_str().ok_or_else(|| json!({"code": -32602, "message": "Missing 'uri'"}))?;

    if uri == "metrics://" {
        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        let metrics = embedded.operational_metrics();
        let content = serde_json::to_string(&metrics).unwrap_or_default();
        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": content
            }]
        }))
    } else if uri == "schema://" {
        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        let metrics = embedded.operational_metrics();
        let schema_info = json!({
            "hnsw_nodes_count": metrics.hnsw_nodes_count,
            "hnsw_logical_bytes": metrics.hnsw_logical_bytes,
            "mmap_resident_bytes": metrics.mmap_resident_bytes
        });
        let content = serde_json::to_string(&schema_info).unwrap_or_default();
        Ok(json!({
            "contents": [{
                "uri": uri,
                "mimeType": "application/json",
                "text": content
            }]
        }))
    } else if uri.starts_with("memory://") {
        // Parse memory://namespace/key
        let path = uri.strip_prefix("memory://").unwrap_or("");
        let parts: Vec<&str> = path.splitn(2, '/').collect();
        if parts.len() != 2 {
            return error_code(-32602, "Invalid memory URI format. Expected: memory://namespace/key");
        }
        let namespace = parts[0];
        let key = parts[1];

        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        match embedded.get(namespace, key) {
            Ok(Some(record)) => {
                let content = serde_json::to_string(&record).unwrap_or_default();
                Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": content
                    }]
                }))
            }
            Ok(None) => error_code(-32602, "Memory record not found"),
            Err(e) => error_code(-32603, &format!("Error reading memory: {}", e)),
        }
    } else if uri.starts_with("namespace://") {
        // Parse namespace://namespace
        let namespace = uri.strip_prefix("namespace://").unwrap_or("");
        if namespace.is_empty() {
            return error_code(-32602, "Invalid namespace URI format. Expected: namespace://namespace");
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
                let content = serde_json::to_string(&result).unwrap_or_default();
                Ok(json!({
                    "contents": [{
                        "uri": uri,
                        "mimeType": "application/json",
                        "text": content
                    }]
                }))
            }
            Err(e) => error_code(-32603, &format!("Error listing namespace: {}", e)),
        }
    } else {
        error_code(-32601, "Resource not found")
    }
}

pub fn handle_prompts_list() -> Result<Value, Value> {
    Ok(json!({
        "prompts": [
            {
                "name": "search_memory",
                "description": "Optimized prompt for searching memory records with hybrid vector and text search",
                "arguments": [
                    {
                        "name": "namespace",
                        "description": "Target namespace for search",
                        "required": true
                    },
                    {
                        "name": "query",
                        "description": "Search query (text or vector)",
                        "required": true
                    },
                    {
                        "name": "filters",
                        "description": "Optional metadata filters",
                        "required": false
                    }
                ]
            },
            {
                "name": "analyze_namespace",
                "description": "Analyze the content and structure of a namespace",
                "arguments": [
                    {
                        "name": "namespace",
                        "description": "Namespace to analyze",
                        "required": true
                    }
                ]
            },
            {
                "name": "summarize_context",
                "description": "Generate a summary of context from memory records",
                "arguments": [
                    {
                        "name": "namespace",
                        "description": "Source namespace",
                        "required": true
                    },
                    {
                        "name": "limit",
                        "description": "Number of records to include",
                        "required": false
                    }
                ]
            },
            {
                "name": "query_builder",
                "description": "Build IQL queries for VantaDB",
                "arguments": [
                    {
                        "name": "operation",
                        "description": "Operation type (SELECT, INSERT, UPDATE, DELETE)",
                        "required": true
                    },
                    {
                        "name": "target",
                        "description": "Target (nodes, memory, etc.)",
                        "required": true
                    },
                    {
                        "name": "conditions",
                        "description": "Query conditions",
                        "required": false
                    }
                ]
            }
        ]
    }))
}

pub fn handle_prompts_get() -> Result<Value, Value> {
    Ok(json!({
        "error": {
            "code": -32601,
            "message": "Prompts are listed but individual prompt retrieval is not yet implemented. Use the prompt templates from prompts/list directly."
        }
    }))
}

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
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string", "description": "Target namespace" },
                        "key": { "type": "string", "description": "Record key to retrieve" }
                    },
                    "required": ["namespace", "key"]
                }
            },
            {
                "name": "memory_delete",
                "description": "Deletes a memory record by namespace and key.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string", "description": "Target namespace" },
                        "key": { "type": "string", "description": "Record key to delete" }
                    },
                    "required": ["namespace", "key"]
                }
            },
            {
                "name": "memory_list",
                "description": "Lists memory records in a namespace with optional pagination and filters.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string", "description": "Target namespace" },
                        "limit": { "type": "number", "description": "Maximum number of records to return, default is 100" },
                        "cursor": { "type": "number", "description": "Optional cursor for pagination" },
                        "filters": { "type": "object", "description": "Optional metadata key-value filters" }
                    },
                    "required": ["namespace"]
                }
            },
            {
                "name": "memory_list_namespaces",
                "description": "Lists all available namespaces in the database.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            },
            {
                "name": "query_lisp",
                "description": "Executes VantaLISP code. Allows reading structures and inserting/mutating Nodes providing semantic context.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string", "description": "VantaLISP program or statement" }
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "search_semantic",
                "description": "Raw semantic vector search directly in the HNSW index.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "vector": { "type": "array", "items": {"type": "number"}, "description": "F32 query vector" },
                        "k": { "type": "number", "description": "Top K neighbors" }
                    },
                    "required": ["vector", "k"]
                }
            },
            {
                "name": "search_memory",
                "description": "Performs memory search in a given namespace supporting optional text queries, filters, distance metric, and explain.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "namespace": { "type": "string", "description": "Target namespace for search" },
                        "query_vector": { "type": "array", "items": {"type": "number"}, "description": "Optional query vector" },
                        "text_query": { "type": "string", "description": "Optional lexical query string" },
                        "top_k": { "type": "number", "description": "Top K hits to return, default is 10" },
                        "distance_metric": { "type": "string", "enum": ["cosine", "euclidean"], "description": "Distance metric, default is cosine" },
                        "explain": { "type": "boolean", "description": "If true, returns explainable ranking details, default is false" },
                        "filters": { "type": "object", "description": "Optional metadata key-value filters" }
                    },
                    "required": ["namespace"]
                }
            },
            {
                "name": "get_node_neighbors",
                "description": "Inspects neighbors or lineage of a node (Volatile or Archived).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_id": { "type": "number", "description": "Node ID to explore" }
                    },
                    "required": ["node_id"]
                }
            },
            {
                "name": "inject_context",
                "description": "Injects external state or context connecting it to a specific thread for subsequent consolidation (Vector Compaction).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "content": { "type": "string", "description": "Context content" },
                        "thread_id": { "type": "number", "description": "Thread ID it belongs to" }
                    },
                    "required": ["content", "thread_id"]
                }
            },
            {
                "name": "read_axioms",
                "description": "Returns the active Devil's Advocate Axioms (Iron Axioms) in the database.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        ]
    }))
}

pub fn handle_tools_call(
    params: &Option<Value>,
    executor: &Executor<'_>,
    storage: &Arc<StorageEngine>,
) -> Result<Value, Value> {
    let p = params
        .as_ref()
        .ok_or_else(|| json!({"code": -32602, "message": "Missing params"}))?;
    let name = p["name"].as_str().unwrap_or("");
    let args = &p["arguments"];

    match name {
        "memory_put" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'namespace'"}))?
                .to_string();
            let key = args["key"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'key'"}))?
                .to_string();
            let payload = args["payload"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'payload'"}))?
                .to_string();

            let vector = if let Some(arr) = args["vector"].as_array() {
                let mut v = Vec::with_capacity(arr.len());
                for val in arr {
                    v.push(val.as_f64().unwrap_or(0.0) as f32);
                }
                Some(v)
            } else {
                None
            };

            let mut metadata = vantadb::sdk::VantaMemoryMetadata::new();
            if let Some(obj) = args["metadata"].as_object() {
                for (key, val) in obj {
                    if let Some(s) = val.as_str() {
                        metadata
                            .insert(key.clone(), vantadb::sdk::VantaValue::String(s.to_string()));
                    } else if let Some(b) = val.as_bool() {
                        metadata.insert(key.clone(), vantadb::sdk::VantaValue::Bool(b));
                    } else if let Some(i) = val.as_i64() {
                        metadata.insert(key.clone(), vantadb::sdk::VantaValue::Int(i));
                    } else if let Some(f) = val.as_f64() {
                        metadata.insert(key.clone(), vantadb::sdk::VantaValue::Float(f));
                    }
                }
            }

            let input = vantadb::sdk::VantaMemoryInput {
                key,
                namespace,
                payload,
                vector,
                metadata,
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.put(input) {
                Ok(record) => {
                    let content = serde_json::to_string(&record).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("Put Error: {}", e)}]}),
                ),
            }
        }
        "memory_get" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'namespace'"}))?
                .to_string();
            let key = args["key"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'key'"}))?
                .to_string();

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.get(&namespace, &key) {
                Ok(Some(record)) => {
                    let content = serde_json::to_string(&record).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Ok(None) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": "Record not found"}]}),
                ),
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("Get Error: {}", e)}]}),
                ),
            }
        }
        "memory_delete" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'namespace'"}))?
                .to_string();
            let key = args["key"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'key'"}))?
                .to_string();

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.delete(&namespace, &key) {
                Ok(deleted) => {
                    let content = serde_json::to_string(&json!({"deleted": deleted})).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("Delete Error: {}", e)}]}),
                ),
            }
        }
        "memory_list" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'namespace'"}))?
                .to_string();
            let limit = args["limit"].as_u64().unwrap_or(100) as usize;
            let cursor = args["cursor"].as_u64().map(|c| c as usize);

            let mut filters = vantadb::sdk::VantaMemoryMetadata::new();
            if let Some(obj) = args["filters"].as_object() {
                for (key, val) in obj {
                    if let Some(s) = val.as_str() {
                        filters
                            .insert(key.clone(), vantadb::sdk::VantaValue::String(s.to_string()));
                    } else if let Some(b) = val.as_bool() {
                        filters.insert(key.clone(), vantadb::sdk::VantaValue::Bool(b));
                    } else if let Some(i) = val.as_i64() {
                        filters.insert(key.clone(), vantadb::sdk::VantaValue::Int(i));
                    } else if let Some(f) = val.as_f64() {
                        filters.insert(key.clone(), vantadb::sdk::VantaValue::Float(f));
                    }
                }
            }

            let options = vantadb::sdk::VantaMemoryListOptions {
                limit,
                cursor,
                filters,
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.list(&namespace, options) {
                Ok(page) => {
                    let result = json!({
                        "records": page.records,
                        "next_cursor": page.next_cursor
                    });
                    let content = serde_json::to_string(&result).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("List Error: {}", e)}]}),
                ),
            }
        }
        "memory_list_namespaces" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.list_namespaces() {
                Ok(namespaces) => {
                    let content = serde_json::to_string(&namespaces).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("List Namespaces Error: {}", e)}]}),
                ),
            }
        }
        "query_lisp" => {
            let query = args["query"].as_str().unwrap_or("");
            match executor.execute_hybrid(query) {
                Ok(ExecutionResult::Read(nodes)) => {
                    let content = serde_json::to_string(&nodes).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Ok(ExecutionResult::Write {
                    affected_nodes,
                    message,
                    node_id,
                }) => {
                    let content = serde_json::to_string(&json!({
                        "affected_nodes": affected_nodes,
                        "message": message,
                        "node_id": node_id
                    }))
                    .unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Ok(ExecutionResult::StaleContext(summary_id)) => {
                    let content = serde_json::to_string(&json!({
                        "stale_context": true,
                        "rehydration_available": true,
                        "summary_id": summary_id,
                        "message": "Suggested Historical Recovery (Critical Confidence Score)."
                    }))
                    .unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("LISP Runtime Error: {}", e)}]}),
                ),
            }
        }
        "search_memory" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'namespace'"}))?
                .to_string();

            let query_vector = if let Some(arr) = args["query_vector"].as_array() {
                let mut v = Vec::with_capacity(arr.len());
                for val in arr {
                    v.push(val.as_f64().unwrap_or(0.0) as f32);
                }
                v
            } else {
                Vec::new()
            };

            let text_query = args["text_query"].as_str().map(String::from);
            let top_k = args["top_k"].as_u64().unwrap_or(10) as usize;

            let distance_metric = match args["distance_metric"].as_str() {
                Some("euclidean") => vantadb::DistanceMetric::Euclidean,
                _ => vantadb::DistanceMetric::Cosine,
            };

            let explain = args["explain"].as_bool().unwrap_or(false);

            let mut filters = vantadb::sdk::VantaMemoryMetadata::new();
            if let Some(obj) = args["filters"].as_object() {
                for (key, val) in obj {
                    if let Some(s) = val.as_str() {
                        filters
                            .insert(key.clone(), vantadb::sdk::VantaValue::String(s.to_string()));
                    } else if let Some(b) = val.as_bool() {
                        filters.insert(key.clone(), vantadb::sdk::VantaValue::Bool(b));
                    } else if let Some(i) = val.as_i64() {
                        filters.insert(key.clone(), vantadb::sdk::VantaValue::Int(i));
                    } else if let Some(f) = val.as_f64() {
                        filters.insert(key.clone(), vantadb::sdk::VantaValue::Float(f));
                    }
                }
            }

            let request = vantadb::sdk::VantaMemorySearchRequest {
                namespace,
                query_vector,
                filters,
                text_query,
                top_k,
                distance_metric,
                explain,
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.search(request) {
                Ok(hits) => {
                    let content = serde_json::to_string(&hits).unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": content}]}))
                }
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("Search Error: {}", e)}]}),
                ),
            }
        }
        "search_semantic" => {
            let vec_arr = args["vector"]
                .as_array()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'vector' array"}))?;
            let mut vector: Vec<f32> = Vec::with_capacity(vec_arr.len());
            for v in vec_arr {
                vector.push(v.as_f64().unwrap_or(0.0) as f32);
            }
            let k = args["k"].as_i64().unwrap_or(5) as usize;

            let mut results = Vec::new();
            let index = storage.hnsw.read();
            let vs = storage.vector_store.read();
            let neighbors = index.search_nearest(&vector, None, None, 0, k, Some(&vs));
            for (id, distance) in neighbors {
                if let Ok(Some(node)) = storage.get(id) {
                    results.push(json!({"id": id, "distance": distance, "node": node}));
                }
            }
            let content = serde_json::to_string(&results).unwrap_or_default();
            Ok(json!({"content": [{"type": "text", "text": content}]}))
        }
        "get_node_neighbors" => {
            let node_id = args["node_id"]
                .as_u64()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'node_id"}))?;

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
                let content = serde_json::to_string(&json!({"node": node, "neighbors": neighbors}))
                    .unwrap_or_default();
                Ok(json!({"content": [{"type": "text", "text": content}]}))
            } else {
                Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": "Node not found"}]}),
                )
            }
        }
        "inject_context" => {
            let content = args["content"]
                .as_str()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'content'"}))?;
            let thread_id = args["thread_id"]
                .as_u64()
                .ok_or_else(|| json!({"code": -32602, "message": "Missing 'thread_id'"}))?;

            let query = format!(
                "INSERT MESSAGE SYSTEM \"{}\" TO THREAD#{}",
                content, thread_id
            );
            match executor.execute_hybrid(&query) {
                Ok(ExecutionResult::Write {
                    affected_nodes,
                    message,
                    ..
                }) => {
                    let out = serde_json::to_string(&json!({
                        "affected_nodes": affected_nodes,
                        "message": message,
                        "status": "Context Anchored"
                    }))
                    .unwrap_or_default();
                    Ok(json!({"content": [{"type": "text", "text": out}]}))
                }
                Ok(_) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": "Unexpected read result for insert"}]}),
                ),
                Err(e) => Ok(
                    json!({"isError": true, "content": [{"type": "text", "text": format!("Execution Error: {}", e)}]}),
                ),
            }
        }
        "read_axioms" => {
            let axioms = json!([
                {"id": 1, "name": "Topological Axiom", "description": "References (edges) to orphan nodes or nodes in Tombstone storage are not allowed."},
                {"id": 2, "name": "Confidence Constraint", "description": "Divergent vector mutations with high historical Confidence Score are rejected."},
                {"id": 3, "name": "Immortal Axiom", "description": "Maintenance: Nodes marked as PINNED evade degradation by Data Decay."},
                {"id": 4, "name": "Resource Allocation", "description": "Maintenance: 5% of memory reserved for nodes with semantic priority >= 0.8."}
            ]);
            let content = serde_json::to_string(&axioms).unwrap_or_default();
            Ok(json!({"content": [{"type": "text", "text": content}]}))
        }
        _ => error_code(-32601, "Tool not found"),
    }
}
