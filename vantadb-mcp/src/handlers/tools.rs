//! MCP tool handlers.

use crate::axioms::resolve_axioms;
use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::*;
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::warn;
use vantadb::executor::{ExecutionResult, Executor};
use vantadb::storage::StorageEngine;

// ── Tools handler ─────────────────────────────────────────────────────────

/// Handle `tools/list`, returning all available MCP tool definitions.
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
                "name": "query_iql",
                "description": "Executes an IQL (Interactive Query Language) statement. Allows reading structures and inserting/mutating Nodes providing semantic context. LISP is not supported; statements must be IQL.",
                "inputSchema": {
                    "type": "object", "properties": {
                        "query": { "type": "string", "description": "IQL statement" }
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
                        "node_id": { "type": "string", "description": "Node ID to explore (decimal string; u128 ids exceed JSON number precision)" }
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
            },
            {
                "name": "rehydrate",
                "description": "Recover shadow-archived nodes that belonged to a summary node from TombstoneStorage.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "summary_id": { "type": "string", "description": "Summary node ID (u128 as string) whose archived nodes to recover" }
                    },
                    "required": ["summary_id"]
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
    let name = p["name"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing 'name' in tool call").to_json())?;
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
                parse_metadata(obj).map_err(|e| e.to_json())?
            } else {
                vantadb::sdk::VantaMemoryMetadata::new()
            };

            let input = vantadb::sdk::VantaMemoryInput {
                key: key.to_string(),
                namespace: namespace.to_string(),
                payload: payload.to_string(),
                vector,
                sparse_vector: None,
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
                parse_metadata(obj).map_err(|e| e.to_json())?
            } else {
                vantadb::sdk::VantaMemoryMetadata::new()
            };

            let options = vantadb::sdk::VantaMemoryListOptions {
                limit,
                cursor,
                #[allow(deprecated)]
                filters,
                filter_ops: None,
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

        "query_iql" => {
            let query = args["query"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'query'").to_json())?;

            let trimmed = query.trim();
            if trimmed.is_empty() {
                return Ok(error_content("Query cannot be empty"));
            }

            if query.contains('\0') {
                return Ok(error_content("Query contains invalid null bytes"));
            }

            if query.len() > config.max_query_length {
                return Ok(error_content(format!(
                    "Query exceeds maximum length of {} bytes",
                    config.max_query_length
                )));
            }

            match executor.execute_hybrid(trimmed) {
                Ok(ExecutionResult::Read(nodes)) => {
                    let records: Vec<vantadb::sdk::VantaNodeRecord> = nodes
                        .into_iter()
                        .map(|n| storage.node_to_record(n))
                        .collect();
                    Ok(text_content(serialize_content(&records)))
                }
                Ok(ExecutionResult::Write {
                    affected_nodes,
                    message,
                    node_id,
                }) => Ok(text_content(serialize_content(&json!({
                    "affected_nodes": affected_nodes,
                    "message": message,
                    "node_id": node_id.map(|id| id.to_string())
                })))),
                Ok(ExecutionResult::StaleContext(summary_id)) => {
                    Ok(text_content(serialize_content(&json!({
                        "stale_context": true,
                        "rehydration_available": true,
                        "summary_id": summary_id.to_string(),
                        "message": "Suggested Historical Recovery (Critical Confidence Score)."
                    }))))
                }
                Err(e) => Ok(error_content(format!("IQL Runtime Error: {}", e))),
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

            let distance_metric = match args["distance_metric"]
                .as_str()
                .map(|s| s.to_lowercase())
                .as_deref()
            {
                Some("cosine") => vantadb::DistanceMetric::Cosine,
                Some("euclidean") => vantadb::DistanceMetric::Euclidean,
                Some(other) => {
                    return Ok(error_content(format!(
                        "Unknown distance_metric '{}' — supported: cosine, euclidean",
                        other
                    )));
                }
                None => {
                    warn!("distance_metric not specified in search_memory — defaulting to cosine");
                    vantadb::DistanceMetric::Cosine
                }
            };

            let explain = args["explain"].as_bool().unwrap_or(false);

            let filters = if let Some(obj) = args["filters"].as_object() {
                parse_metadata(obj).map_err(|e| e.to_json())?
            } else {
                vantadb::sdk::VantaMemoryMetadata::new()
            };

            let request = vantadb::sdk::VantaMemorySearchRequest {
                namespace: namespace.to_string(),
                query_vector,
                query_sparse: None,
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

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let hits = match embedded.search_vector(&vector, k) {
                Ok(hits) => hits,
                Err(e) => {
                    return Ok(error_content(format!("Search Error: {}", e)));
                }
            };

            let mut results = Vec::new();
            for hit in hits {
                if let Ok(Some(node)) = embedded.get_node(hit.node_id) {
                    results.push(json!({
                        "id": hit.node_id.to_string(),
                        "distance": hit.distance,
                        "node": node,
                    }));
                }
            }
            Ok(text_content(serialize_content(&results)))
        }

        "get_node_neighbors" => {
            let node_id = parse_node_id(&args["node_id"]).ok_or_else(|| {
                McpError::invalid_params("Invalid or missing 'node_id'").to_json()
            })?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.get_node(node_id) {
                Ok(Some(node)) => {
                    let mut neighbors = Vec::new();
                    for edge in &node.edges {
                        if let Ok(Some(target_node)) = embedded.get_node(edge.target) {
                            neighbors.push(json!({
                                "rel": edge.label,
                                "target_id": edge.target.to_string(),
                                "target_confidence": target_node.confidence_score,
                                "target_priority": target_node.importance
                            }));
                        }
                    }
                    Ok(text_content(serialize_content(
                        &json!({"node": node, "neighbors": neighbors}),
                    )))
                }
                Ok(None) => Ok(error_content("Node not found")),
                Err(e) => Ok(error_content(format!("Get Node Error: {}", e))),
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

            let escaped_content = escape_iql_string(content);
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

        "read_axioms" => Ok(text_content(serialize_content(&resolve_axioms(storage)))),

        "collection_stats" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;

            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let metrics = embedded.operational_metrics();

            let mut total_bytes = 0usize;
            let mut vector_count = 0usize;
            let mut created_at = u64::MAX;
            let total_records = match for_each_record(&embedded, namespace, config, |record| {
                total_bytes += record.payload.len()
                    + record
                        .metadata
                        .iter()
                        .fold(0, |acc, (k, v)| acc + k.len() + format!("{:?}", v).len());
                if record.vector.is_some() {
                    vector_count += 1;
                }
                created_at = created_at.min(record.created_at_ms);
            }) {
                Ok(count) => count,
                Err(e) => return Ok(error_content(format!("Collection stats error: {}", e))),
            };
            let created_at = if total_records == 0 { 0 } else { created_at };

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
                let mut has_vector = false;
                let mut created_at = u64::MAX;
                let record_count = match for_each_record(&embedded, ns, config, |record| {
                    has_vector |= record.vector.is_some();
                    created_at = created_at.min(record.created_at_ms);
                }) {
                    Ok(count) => count,
                    Err(_) => continue,
                };
                let created_at = if record_count == 0 { 0 } else { created_at };

                collections.push(json!({
                    "name": ns,
                    "record_count": record_count,
                    "has_vector_index": has_vector,
                    "created_at": created_at,
                }));
            }

            Ok(text_content(serialize_content(&collections)))
        }

        "rehydrate" => {
            let summary_id = args["summary_id"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'summary_id'").to_json())?;
            let sid: u128 = summary_id.parse().map_err(|_| {
                McpError::invalid_params("summary_id must be a valid integer (u128)").to_json()
            })?;
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let recovered = embedded
                .recover_archived_nodes(sid)
                .map_err(|e| McpError::internal_error(e.to_string()).to_json())?;
            Ok(text_content(serialize_content(&json!({
                "recovered_count": recovered.len(),
                "summary_id": summary_id,
                "rehydration_complete": true,
            }))))
        }

        "collection_delete" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            let confirm = args["confirm"].as_str().ok_or_else(|| {
                McpError::invalid_params("Missing 'confirm' (must be 'yes')").to_json()
            })?;

            if confirm != "yes" {
                return Ok(error_content(
                    "Confirmation required: set 'confirm' to 'yes'",
                ));
            }

            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;

            let txn_id = storage.begin_transaction().map_err(|e| {
                McpError::internal_error(format!("Failed to begin transaction: {}", e)).to_json()
            })?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            // Stream only keys — never materialize the full record set. Deletes
            // run after pagination: list() recomputes the ID window per call and
            // deleting mid-stream would shift the cursor offset and skip rows.
            let mut keys: Vec<String> = Vec::new();
            let streamed = for_each_record(&embedded, namespace, config, |record| {
                keys.push(record.key.clone());
            });
            if let Err(e) = streamed {
                if let Err(abort_err) = storage.abort_transaction(txn_id) {
                    warn!(error = %abort_err, "Failed to abort transaction after collection error");
                }
                return Ok(error_content(format!("Collection delete error: {}", e)));
            }

            let total = keys.len();
            let mut failures = 0;
            let mut last_error = String::new();

            for key in &keys {
                if let Err(e) = embedded.delete(namespace, key) {
                    failures += 1;
                    last_error = format!("{}: {}", key, e);
                    warn!(error = %e, key = %key, "Failed to delete record during collection_delete");
                }
            }

            if failures > 0 {
                if let Err(abort_err) = storage.abort_transaction(txn_id) {
                    warn!(error = %abort_err, "Failed to abort transaction after partial delete");
                }
                return Ok(error_content(format!(
                    "Partial delete: {}/{} removed, last error: {}",
                    total - failures,
                    total,
                    last_error
                )));
            }

            storage.commit_transaction(txn_id).map_err(|e| {
                McpError::internal_error(format!("Failed to commit transaction: {}", e)).to_json()
            })?;

            let result = json!({
                "deleted": true,
                "records_removed": total,
            });
            Ok(text_content(serialize_content(&result)))
        }

        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}
