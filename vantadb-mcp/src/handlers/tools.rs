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
    let base_tools = json!([
        {
            "name": "memory_put",
            "description": "Inserts or updates a memory record in a namespace with payload, vector, optional sparse vector, optional metadata, and optional TTL.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Target namespace" },
                    "key": { "type": "string", "description": "Unique key for the record" },
                    "payload": { "type": "string", "description": "Text content of the memory" },
                    "vector": { "type": "array", "items": {"type": "number"}, "description": "Optional embedding vector" },
                    "sparse_vector": { "type": "object", "additionalProperties": {"type": "number"}, "description": "Optional sparse term-weight vector, e.g. {\"0\": 0.5, \"7\": 1.25} (dimension id -> weight)" },
                    "metadata": { "type": "object", "description": "Optional metadata key-value pairs" },
                    "expires_at_ms": { "type": "number", "description": "Optional absolute Unix-ms timestamp after which the record expires (TTL)" }
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
            "description": "Performs memory search in a given namespace supporting optional text queries, filters, distance metric, explain, and a search profile.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string" },
                    "query_vector": { "type": "array", "items": {"type": "number"} },
                    "text_query": { "type": "string" },
                    "top_k": { "type": "number", "description": "Top K hits, default 10" },
                    "distance_metric": { "type": "string", "enum": ["cosine", "euclidean"] },
                    "explain": { "type": "boolean" },
                    "filters": { "type": "object" },
                    "search_profile": { "type": "object", "properties": {
                        "mode": { "type": "string", "enum": ["keyword", "vector", "hybrid"] },
                        "rrf_k": { "type": "number", "description": "RRF k parameter (1..max_rrf_k, default core)" },
                        "candidate_k": { "type": "number", "description": "Per-channel candidate budget (1..max_candidate_k, default core)" }
                    }, "description": "Optional search profile (MEM-01): mode forces the retrieval channel (keyword/vector/hybrid); rrf_k/candidate_k tune RRF. Wire format matches the native API and the IQL PROFILE clause." }
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
    ]);
    // MEM-07: six review-agent skill tools over SkillStore. Definitions live
    // in crate::skills so this array stays readable; the wire shape is part
    // of the public MCP API.
    let mut result = json!({ "tools": base_tools });
    if let Some(tools) = result["tools"].as_array_mut() {
        tools.extend(crate::skills::skill_tool_definitions());
    }
    Ok(result)
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

            // AUD-046: reject vector puts whose dim does not match the live
            // index dim, mirroring the search-side check (MCP-04). Without
            // this, a mismatched vector is silently accepted into the HNSW
            // index — `vector_count` rises but the node never surfaces in
            // search, corrupting the index with mixed dims. An empty index
            // (first vector put) has no dim yet and defines it.
            if let Some(vector) = &vector {
                if let Some(expected) = index_vector_dim(storage) {
                    if vector.len() != expected {
                        return Ok(error_content(
                            vantadb::VantaError::DimensionMismatch {
                                expected,
                                got: vector.len(),
                            }
                            .to_string(),
                        ));
                    }
                }
            }

            // AUD-045: accept the sparse_vector object (dimension id -> weight,
            // e.g. {"0": 0.5}). Passed as JSON object, mirroring the core's
            // SparseVector(BTreeMap<u32, f32>). An absent/invalid value is
            // rejected explicitly rather than silently dropped.
            let sparse_vector = match args.get("sparse_vector") {
                Some(Value::Null) | None => None,
                Some(v) => {
                    let obj = v.as_object().ok_or_else(|| {
                        McpError::invalid_params(
                            "'sparse_vector' must be an object mapping dimension id to weight, e.g. {\"0\": 0.5}",
                        )
                        .to_json()
                    })?;
                    Some(parse_sparse_vector(obj).map_err(|e| e.to_json())?)
                }
            };

            // AUD-045: accept an absolute expires_at_ms (Unix ms) and convert to
            // the SDK input's relative ttl_ms. An already-expired timestamp
            // saturates to 0 (expire immediately) rather than overflowing.
            let ttl_ms = match args.get("expires_at_ms") {
                Some(Value::Null) | None => None,
                Some(v) => {
                    let expires = v.as_u64().ok_or_else(|| {
                        McpError::invalid_params(
                            "'expires_at_ms' must be an unsigned integer (Unix ms)",
                        )
                        .to_json()
                    })?;
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis() as u64)
                        .unwrap_or(0);
                    Some(expires.saturating_sub(now_ms))
                }
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
                sparse_vector,
                metadata,
                ttl_ms,
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

            let filter_ops = if let Some(obj) = args["filters"].as_object() {
                // AUD-048: unified filter semantics with the CLI channel —
                // accepts both flat values (implicit $eq) and operator
                // objects ($eq/$gt/$gte/$lt/$lte/$neq). Routed through the
                // core's `filter_ops` slot, which already supports operators.
                let ops = parse_filter_ops(obj).map_err(|e| e.to_json())?;
                if ops.is_empty() {
                    None
                } else {
                    Some(ops)
                }
            } else {
                None
            };

            let options = vantadb::sdk::VantaMemoryListOptions {
                limit,
                cursor,
                #[allow(deprecated)]
                filters: vantadb::sdk::VantaMemoryMetadata::new(),
                filter_ops,
                exclude_superseded: false,
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

            // MCP-04: reject vector queries whose dim does not match the live
            // index dim. Without this check, mismatched queries score garbage
            // (all distances ~0.0) and silently return wrong results. Text-only
            // searches (empty query_vector) are unaffected.
            if !query_vector.is_empty() {
                if let Some(expected) = index_vector_dim(storage) {
                    if query_vector.len() != expected {
                        return Ok(error_content(
                            vantadb::VantaError::DimensionMismatch {
                                expected,
                                got: query_vector.len(),
                            }
                            .to_string(),
                        ));
                    }
                }
            }

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

            // AUD-048: unified filter semantics with the CLI channel. The
            // search request (`VantaMemorySearchRequest`) is flat-only — it
            // has no `filter_ops` slot — so flat values and explicit `$eq`
            // both fold into the flat metadata (identical equality semantics).
            // Range/inequality operators ($gt/$gte/$lt/$lte/$neq) cannot be
            // expressed in a search request; return a clear error pointing at
            // memory_list, which supports them via filter_ops.
            let filters = if let Some(obj) = args["filters"].as_object() {
                let ops = parse_filter_ops(obj).map_err(|e| e.to_json())?;
                let mut flat = vantadb::sdk::VantaMemoryMetadata::new();
                for item in ops {
                    if item.op == vantadb::sdk::VantaFilterOp::Eq {
                        flat.insert(item.field, item.value);
                    } else {
                        return Ok(error_content(format!(
                            "search_memory filters support equality only (flat values or {{\"$eq\": value}}); \
                             operator '{:?}' on field '{}' is available via memory_list filters",
                            item.op, item.field
                        )));
                    }
                }
                flat
            } else {
                vantadb::sdk::VantaMemoryMetadata::new()
            };

            // MEM-02: passthrough del SearchProfileConfig. La forma de wire es
            // EXACTAMENTE la forma serde de SearchProfileConfig (src/sdk/types.rs),
            // la misma que deserializa la API nativa y la cláusula IQL PROFILE →
            // paridad de shape entre canales (D13/D19). Ausente o {} → None
            // (modo Hybrid + constantes core).
            let search_profile = match args.get("search_profile") {
                Some(Value::Object(obj)) => {
                    Some(validate_search_profile(obj, config).map_err(|e| e.to_json())?)
                }
                Some(_) => {
                    return Ok(error_content(
                        "search_profile must be an object {mode, rrf_k, candidate_k}".to_string(),
                    ));
                }
                None => None,
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
                exclude_superseded: false,
                search_profile,
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

            // MCP-04: reject queries whose dim does not match the live index
            // dim (trust boundary — validated here, not deeper in the engine).
            // A 3-dim query against a 4-dim index would otherwise return
            // garbage distances (all ~0.0) with success.
            if let Some(expected) = index_vector_dim(storage) {
                if vector.len() != expected {
                    return Ok(error_content(
                        vantadb::VantaError::DimensionMismatch {
                            expected,
                            got: vector.len(),
                        }
                        .to_string(),
                    ));
                }
            }

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let hits = match embedded.search_vector(&vector, k) {
                Ok(hits) => hits,
                Err(e) => {
                    return Ok(error_content(format!("Search Error: {}", e)));
                }
            };

            // MCP-03: `VantaSearchHit.distance` carries the raw HNSW score —
            // cosine SIMILARITY (identical → 1.0, orthogonal → 0.0) and the
            // negated euclidean distance. The `distance` field exposed here is
            // documented in docs/api/MCP.md as "lower is more similar", so
            // invert the cosine score (1 - similarity) into a real distance.
            // The core SDK keeps its score semantics for other consumers
            // (search_memory, WASM), so the conversion happens at this
            // serialization boundary.
            let metric = storage.vec_index().config.distance_metric;
            let mut results = Vec::new();
            for hit in hits {
                if let Ok(Some(node)) = embedded.get_node(hit.node_id) {
                    let distance = match metric {
                        vantadb::DistanceMetric::Cosine => 1.0 - hit.distance,
                        vantadb::DistanceMetric::Euclidean => -hit.distance,
                        vantadb::DistanceMetric::SparseDot => hit.distance,
                    };
                    results.push(json!({
                        "id": hit.node_id.to_string(),
                        "distance": distance,
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
            // AUD-050: a present-but-wrong-typed thread_id (e.g. string "200")
            // used to surface as "Missing 'thread_id'" — misleading, since the
            // field IS present. Distinguish absence from bad type explicitly.
            let thread_id = match args.get("thread_id") {
                Some(Value::Null) | None => {
                    return Err(McpError::invalid_params("Missing 'thread_id'").to_json());
                }
                Some(v) => v.as_u64().ok_or_else(|| {
                    McpError::invalid_params(format!(
                        "'thread_id' must be a numeric id (integer), got {}",
                        json_value_type_name(v)
                    ))
                    .to_json()
                })?,
            };

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

        "skill_list" | "skill_view" | "skill_create" | "skill_update" | "skill_patch"
        | "skill_files_write" => crate::skills::handle_skill_tool(name, args, storage, config),
        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}

/// Vector dimension of the live HNSW index, if it holds any vectors.
///
/// The index dim is not stored in config (HnswConfig has no `dim` field), so it
/// is derived from the first node that carries a vector. An empty index returns
/// `None` and dimension validation is skipped — there is nothing to compare
/// against yet, and an empty result set is the correct answer anyway.
fn index_vector_dim(storage: &Arc<StorageEngine>) -> Option<usize> {
    storage
        .vec_index()
        .nodes
        .iter()
        .find_map(|entry| entry.value().vector_slice().map(|v| v.len()))
}
