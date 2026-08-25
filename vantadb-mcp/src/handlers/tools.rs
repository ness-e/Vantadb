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

/// MCP-17/MCP-25: stdio carries export/import payloads inline inside a single
/// JSON-RPC message. Cap them so one tool call cannot exhaust the pipe or
/// client memory; larger transfers go through file-based paths
/// (`export_namespace(path, ...)` / `bulk_import_file` on the CLI or SDK).
const MAX_TRANSFER_BYTES: usize = 10 * 1024 * 1024;

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
            "name": "memory_put_batch",
            "description": "Stores multiple memory records in a single batch operation. Each input carries namespace, key, payload and optional vector/sparse_vector/metadata/expires_at_ms. All-or-nothing: an invalid input fails the whole call before any write. Duplicate keys are UPSERTs (version bumps). Vector dimensions must match the live index.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "inputs": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "namespace": { "type": "string" },
                                "key": { "type": "string" },
                                "payload": { "type": "string" },
                                "vector": { "type": "array", "items": {"type": "number"} },
                                "sparse_vector": { "type": "object", "additionalProperties": {"type": "number"} },
                                "metadata": { "type": "object" },
                                "expires_at_ms": { "type": "number", "description": "Optional absolute Unix-ms TTL timestamp" }
                            },
                            "required": ["namespace", "key", "payload"]
                        }
                    }
                },
                "required": ["inputs"]
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
            "name": "memory_delete_by_filter",
            "description": "Batch-deletes every memory record in a namespace whose metadata matches the given filters (AND semantics, operators $eq/$neq/$gt/$gte/$lt/$lte, flat values are implicit $eq). Returns the number of records deleted. Requires at least one filter item to prevent accidental full-namespace deletion.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string" },
                    "filters": { "type": "object", "description": "Metadata filters; same shape as memory_list filters, e.g. {\"env\": \"dev\"} or {\"priority\": {\"$gt\": 1}}" }
                },
                "required": ["namespace", "filters"]
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
            "name": "memory_versions",
            "description": "Lists every retained version of a memory record, ascending (v1..vN). Empty if the key does not exist or has no history. Expired versions are included as historical data until purged.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string" },
                    "key": { "type": "string" }
                },
                "required": ["namespace", "key"]
            }
        },
        {
            "name": "memory_supersede",
            "description": "Marks an existing memory record as superseded by another existing record (durable soft-dead, recoverable). Errors if either key is missing, if old_key equals new_key, or if the old record is already superseded.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string" },
                    "old_key": { "type": "string", "description": "Key of the record to mark as superseded" },
                    "new_key": { "type": "string", "description": "Key of the record that supersedes it" }
                },
                "required": ["namespace", "old_key", "new_key"]
            }
        },
        {
            "name": "query_iql",
            "description": "Executes an IQL statement against TYPED GRAPH NODES and memory namespaces. Each memory namespace is queryable as an IQL table named by its sanitized form ('/' and '-' become '_', a leading digit/dot gets a '_' prefix; e.g. namespace 'mmd/s1/history' → 'SELECT * FROM mmd_s1_history'). Records written before this feature are visible too (no migration). Graph workflow: create nodes with 'INSERT NODE#<id> TYPE <Type> { field: value }', then query them with 'SELECT * FROM <Type>' or read a single node with 'FROM NODE#<id>'. A type/namespace collision returns the union of both. Scanning an unknown or empty name returns [] without error. LISP is not supported; statements must be IQL.",
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
            "name": "search_with_method",
            "description": "MCP-24: memory search with an explicit dense-index backend override. Same parameters as search_memory plus `method` (hnsw | ivf | flat | diskann | scann); omit `method` to keep automatic engine routing.",
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
                    "method": { "type": "string", "enum": ["hnsw", "ivf", "flat", "diskann", "scann"], "description": "Dense-index backend override (MCP-24); omit to keep automatic routing" },
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
            "name": "search_multi",
            "description": "MCP-24: run one search request across multiple namespaces and merge the results (sorted by descending score, capped at `top_k` globally). `namespaces` is required; the other parameters match search_memory. Returns a flat hit array.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespaces": { "type": "array", "items": { "type": "string" }, "description": "Namespaces to search; results are merged (must not be empty)" },
                    "query_vector": { "type": "array", "items": {"type": "number"} },
                    "text_query": { "type": "string" },
                    "top_k": { "type": "number", "description": "Global top K after merging, default 10" },
                    "distance_metric": { "type": "string", "enum": ["cosine", "euclidean"] },
                    "explain": { "type": "boolean" },
                    "filters": { "type": "object" },
                    "search_profile": { "type": "object", "properties": {
                        "mode": { "type": "string", "enum": ["keyword", "vector", "hybrid"] },
                        "rrf_k": { "type": "number", "description": "RRF k parameter (1..max_rrf_k, default core)" },
                        "candidate_k": { "type": "number", "description": "Per-channel candidate budget (1..max_candidate_k, default core)" }
                    }, "description": "Optional search profile (MEM-01): mode forces the retrieval channel (keyword/vector/hybrid); rrf_k/candidate_k tune RRF. Wire format matches the native API and the IQL PROFILE clause." }
                },
                "required": ["namespaces"]
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
            "name": "graph_page_rank",
            "description": "MCP-21: computes PageRank over the subgraph reachable from the given root node ids. Returns {scores: {\"<node_id>\": rank}} with node ids as decimal strings.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "roots": { "type": "array", "items": { "type": "string" }, "description": "Root node IDs (decimal strings)" },
                    "max_iterations": { "type": "number", "description": "Maximum iterations, default 100" },
                    "damping_factor": { "type": "number", "description": "Damping factor, default 0.85" },
                    "tolerance": { "type": "number", "description": "Convergence threshold, default 1e-6" }
                },
                "required": ["roots"]
            }
        },
        {
            "name": "graph_degree_centrality",
            "description": "MCP-21: degree centrality (incoming/outgoing edge counts) for every node in the subgraph reachable from the given roots. Returns {degrees: {\"<node_id>\": {in, out}}}.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "roots": { "type": "array", "items": { "type": "string" }, "description": "Root node IDs (decimal strings)" }
                },
                "required": ["roots"]
            }
        },
        {
            "name": "graph_traverse",
            "description": "MCP-22: multi-hop traversal (BFS or DFS) from one or more start nodes. Optional filter restricts traversal to edges whose label is in 'labels' and/or whose created_at_ms falls inside 'time_range' [from_ms, to_ms]. Returns visited node ids in traversal order.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "start": { "type": "array", "items": { "type": "string" }, "description": "Start node IDs (decimal strings)" },
                    "mode": { "type": "string", "enum": ["bfs", "dfs"], "description": "Traversal order" },
                    "max_depth": { "type": "number", "description": "Maximum hops from the start nodes" },
                    "direction": { "type": "string", "enum": ["forward", "reverse", "both"], "description": "Edge direction to follow, default forward" },
                    "filter": {
                        "type": "object",
                        "properties": {
                            "labels": { "type": "array", "items": { "type": "number" }, "description": "Only follow edges whose label id is listed" },
                            "time_range": { "type": "array", "items": { "type": "number" }, "description": "[from_ms, to_ms] inclusive window on edge creation time" }
                        },
                        "description": "Optional label/temporal edge filter"
                    }
                },
                "required": ["start", "mode", "max_depth"]
            }
        },
        {
            "name": "graph_topological_sort",
            "description": "MCP-22: topological sort of the subgraph reachable from the given roots. Errors if the subgraph contains a cycle.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "roots": { "type": "array", "items": { "type": "string" }, "description": "Root node IDs (decimal strings)" }
                },
                "required": ["roots"]
            }
        },
        {
            "name": "graph_is_dag",
            "description": "MCP-22: returns true when the subgraph reachable from the given roots is a directed acyclic graph (DAG).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "roots": { "type": "array", "items": { "type": "string" }, "description": "Root node IDs (decimal strings)" }
                },
                "required": ["roots"]
            }
        },
        {
            "name": "remove_edge",
            "description": "Removes all edges between two nodes with the given label (both directions). Node ids are u128 decimal strings (JSON numbers lose precision above 2^53).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "source_id": { "type": "string", "description": "Source node ID (decimal string)" },
                    "target_id": { "type": "string", "description": "Target node ID (decimal string)" },
                    "label": { "type": "string", "description": "Edge label to remove" }
                },
                "required": ["source_id", "target_id", "label"]
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
        },
        {
            "name": "purge_expired",
            "description": "Scans all memory records and physically deletes those whose TTL expiry has passed. Returns the number of records purged.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "compact_wal",
            "description": "Flushes, archives the current WAL file, and starts a fresh one to reclaim WAL space.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "flush",
            "description": "Flushes the WAL and memory-mapped files to disk (manual durability checkpoint).",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "compact_layout",
            "description": "Compacts the vector store file grouping nodes in BFS order from the HNSW entry point. Returns the number of bytes reclaimed.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "vacuum",
            "description": "Purges tombstoned nodes from the HNSW index. Returns a report with scanned_nodes, removed_nodes, reclaimed_bytes, duration_ms, and success.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "rebuild_index",
            "description": "MCP-20: rebuilds the HNSW vector index, derived indexes, and text index from scratch (recovery primitive for a corrupted index). Returns a report: scanned_nodes, indexed_vectors, skipped_tombstones, duration_ms, derived_rebuild_ms, index_path, success.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "audit_text_index",
            "description": "MCP-20: read-only integrity audit of the derived persistent text index (BM25 postings/stats vs canonical memory records). With deep=true also verifies posting positions, term frequencies and stats values. Returns a report; passed=true and status='ok' mean no drift.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Optional namespace filter; omit to audit all namespaces" },
                    "deep": { "type": "boolean", "description": "Run value-level deep audit (slower), default false" }
                },
                "required": []
            }
        },
        {
            "name": "repair_text_index",
            "description": "MCP-20: repairs the derived text index by rebuilding it from canonical storage. Use when audit_text_index reports drift (status='repair_recommended'). Returns a repair report with record/posting/stats counts and duration_ms.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "capabilities",
            "description": "MCP-26: introspects the engine's supported features. Returns {runtime_profile, persistence, vector_search, iql_queries, read_only} so the agent can discover what the connected database supports.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "generate_snippet",
            "description": "MCP-26: generates a text snippet from a payload, highlighting matched query terms when with_highlighting=true. Returns {snippet: \"...\"} or {snippet: null} when no query terms match.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "payload": { "type": "string", "description": "Text content to extract the snippet from" },
                    "text_query": { "type": "string", "description": "Query whose terms drive term selection/highlighting" },
                    "with_highlighting": { "type": "boolean", "description": "Wrap matched terms in markers, default false" }
                },
                "required": ["payload", "text_query"]
            }
        },
        {
            "name": "list_snapshots",
            "description": "MCP-26: lists existing physical snapshot names stored under <data_dir>/snapshots (sorted). Logical backup/restore lives in 'export'/'import'; snapshots are physical Fjall copies.",
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        },
        {
            "name": "export",
            "description": "Exports memory records as JSONL (one JSON object per line). Pass 'namespace' to export a single namespace, omit it to export all namespaces. Returns the raw JSONL as text content (max 10 MB per call). Pair with 'import' for backup/restore.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Optional namespace to export; omit to export all namespaces" }
                },
                "required": []
            }
        },
        {
            "name": "import",
            "description": "Imports records from a JSONL string as produced by the 'export' tool (one record per line, schema_version 1). Empty lines are skipped and malformed lines are counted as errors in the returned report instead of failing the call. Max 10 MB per call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "JSONL content (one record per line)" }
                },
                "required": ["content"]
            }
        },
        {
            "name": "bulk_import_file",
            "description": "Bulk-imports records from a binary .vdbdump file on the host filesystem (bypasses per-record validation for raw throughput). Returns a report with total_records, batches_committed, and duration_ms.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute path to a .vdbdump file on the host running the MCP server" }
                },
                "required": ["path"]
            }
        },
        {
            "name": "bulk_import_stream",
            "description": "Bulk-imports records from inline content: either NDJSON (one VantaMemoryInput per line: namespace, key, payload, optional metadata/vector/ttl_ms) or a raw .vdbdump payload starting with the VDBJSON magic. Bypasses per-record validation for raw throughput; imported nodes are raw engine entries NOT addressable via memory_get/memory_list — use search or re-export paths that scan the engine. Max 10 MB per call.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "NDJSON content (one record per line) or raw .vdbdump payload" }
                },
                "required": ["content"]
            }
        }
    ]);
    // MEM-07: six review-agent skill tools over SkillStore. Definitions live
    // in crate::skills so this array stays readable; the wire shape is part
    // of the public MCP API.
    let mut result = json!({ "tools": base_tools });
    if let Some(tools) = result["tools"].as_array_mut() {
        tools.extend(crate::skills::skill_tool_definitions());
        tools.extend(crate::code::code_tool_definitions());
        tools.extend(crate::wiki::wiki_tool_definitions());
        tools.extend(crate::context::context_tool_definitions());
        tools.extend(crate::scenes::scene_tool_definitions());
        tools.extend(crate::threads::thread_tool_definitions());
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

        // MCP-19: batch put — thin wrapper over the SDK put_batch (single WAL
        // batch_append + KV write_batch per chunk). The SDK validates every
        // input upfront, so a malformed record fails the whole call before any
        // write (all-or-nothing); duplicate keys are upserts with version bump.
        "memory_put_batch" => {
            let inputs_arr = args["inputs"]
                .as_array()
                .ok_or_else(|| McpError::invalid_params("Missing 'inputs' array").to_json())?;
            if inputs_arr.is_empty() {
                return Err(McpError::invalid_params("'inputs' must not be empty").to_json());
            }

            let mut inputs = Vec::with_capacity(inputs_arr.len());
            for item in inputs_arr {
                if !item.is_object() {
                    return Err(McpError::invalid_params(
                        "Each entry of 'inputs' must be an object",
                    )
                    .to_json());
                }
                let input = parse_memory_input(item, config)?;
                inputs.push(input);
            }

            // AUD-046 parity: reject batch vectors whose dim does not match the
            // live index dim — same trust-boundary check as memory_put.
            if let Some(expected) = index_vector_dim(storage) {
                for input in &inputs {
                    if let Some(vector) = &input.vector {
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
            }

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.put_batch(inputs) {
                Ok(records) => Ok(text_content(serialize_content(&records))),
                Err(e) => Ok(error_content(format!("Put Batch Error: {}", e))),
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

        // MCP-18: batch delete by metadata filter — thin wrapper over the SDK
        // delete_by_filter. Reuses the exact VantaMemoryFilter wire shape that
        // memory_list already publishes (AUD-048 parse_filter_ops).
        "memory_delete_by_filter" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;

            let filter_obj = args["filters"]
                .as_object()
                .ok_or_else(|| McpError::invalid_params("Missing 'filters' object").to_json())?;
            let filter = parse_filter_ops(filter_obj).map_err(|e| e.to_json())?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.delete_by_filter(namespace, filter) {
                Ok(count) => Ok(text_content(serialize_content(&json!({
                    "deleted_count": count
                })))),
                Err(e) => Ok(error_content(format!("Delete By Filter Error: {}", e))),
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

        // MOD-10: version history + supersession via MCP — thin wrappers over
        // the SDK (versions / supersede). Same MEM-32 error shape.
        "memory_versions" => {
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
            match embedded.versions(namespace, key) {
                Ok(records) => Ok(text_content(serialize_content(&records))),
                Err(e) => Ok(error_content(format!("Versions Error: {}", e))),
            }
        }

        "memory_supersede" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            let old_key = args["old_key"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'old_key'").to_json())?;
            let new_key = args["new_key"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'new_key'").to_json())?;

            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;
            validate_identifier(old_key, "old_key", config.max_key_length)
                .map_err(|e| e.to_json())?;
            validate_identifier(new_key, "new_key", config.max_key_length)
                .map_err(|e| e.to_json())?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.supersede(namespace, old_key, new_key) {
                Ok(()) => Ok(text_content(serialize_content(
                    &json!({ "superseded": true }),
                ))),
                Err(e) => Ok(error_content(format!("Supersede Error: {}", e))),
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

            let request = match parse_search_request(namespace, args, config, storage)? {
                ParsedSearchRequest::Ready(req) => req,
                ParsedSearchRequest::Rejected(envelope) => return Ok(envelope),
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.search(request) {
                Ok(hits) => Ok(text_content(serialize_content(&hits))),
                Err(e) => Ok(error_content(format!("Search Error: {}", e))),
            }
        }

        // MCP-24: memory search with an explicit dense-index backend override
        // (search_with_method). Same wire shape as search_memory plus `method`.
        "search_with_method" => {
            let namespace = args["namespace"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;
            let method = parse_search_method(&args["method"])?;

            let request = match parse_search_request(namespace, args, config, storage)? {
                ParsedSearchRequest::Ready(req) => req,
                ParsedSearchRequest::Rejected(envelope) => return Ok(envelope),
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.search_with_method(request, method) {
                Ok(hits) => Ok(text_content(serialize_content(&hits))),
                Err(e) => Ok(error_content(format!("Search Error: {}", e))),
            }
        }

        // MCP-24: one request across several namespaces, merged by descending
        // score and capped at `top_k` globally (SDK search_multi).
        "search_multi" => {
            let ns_arr = args["namespaces"]
                .as_array()
                .ok_or_else(|| McpError::invalid_params("Missing 'namespaces' array").to_json())?;
            if ns_arr.is_empty() {
                return Err(McpError::invalid_params("'namespaces' must not be empty").to_json());
            }
            let mut namespaces = Vec::with_capacity(ns_arr.len());
            for ns in ns_arr {
                let ns = ns.as_str().ok_or_else(|| {
                    McpError::invalid_params("'namespaces' entries must be strings").to_json()
                })?;
                validate_identifier(ns, "namespace", config.max_namespace_length)
                    .map_err(|e| e.to_json())?;
                namespaces.push(ns.to_string());
            }

            // The SDK's search_multi ignores request.namespace (it overwrites
            // it per-namespace), so a placeholder is fine here.
            let request = match parse_search_request("default", args, config, storage)? {
                ParsedSearchRequest::Ready(req) => req,
                ParsedSearchRequest::Rejected(envelope) => return Ok(envelope),
            };

            let ns_refs: Vec<&str> = namespaces.iter().map(String::as_str).collect();
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.search_multi(&ns_refs, request) {
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

        // MCP-16/MCP-23: maintenance tools — thin wrappers over the SDK
        // (purge_expired/compact_wal/flush/compact_layout). Domain errors come
        // back as Ok(error_content(...)) so the LLM client can read and
        // self-correct (MEM-32), never as a propagated JSON-RPC error.
        "purge_expired" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.purge_expired() {
                Ok(count) => Ok(text_content(serialize_content(&json!({ "purged": count })))),
                Err(e) => Ok(error_content(format!("Purge Error: {}", e))),
            }
        }

        "compact_wal" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.compact_wal() {
                Ok(()) => Ok(text_content(serialize_content(
                    &json!({ "compacted_wal": true }),
                ))),
                Err(e) => Ok(error_content(format!("Compact WAL Error: {}", e))),
            }
        }

        "flush" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.flush() {
                Ok(()) => Ok(text_content(serialize_content(&json!({ "flushed": true })))),
                Err(e) => Ok(error_content(format!("Flush Error: {}", e))),
            }
        }

        "compact_layout" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.compact_layout() {
                Ok(bytes) => Ok(text_content(serialize_content(
                    &json!({ "bytes_reclaimed": bytes }),
                ))),
                Err(e) => Ok(error_content(format!("Compact Layout Error: {}", e))),
            }
        }

        // MOD-10: vacuum — thin wrapper over the SDK. VacuumReport does not
        // derive Serialize, so the report is built as an explicit JSON object
        // (same field set as the struct, src/storage/engine/mod.rs).
        "vacuum" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.vacuum() {
                Ok(report) => Ok(text_content(serialize_content(&json!({
                    "scanned_nodes": report.scanned_nodes,
                    "removed_nodes": report.removed_nodes,
                    "reclaimed_bytes": report.reclaimed_bytes,
                    "duration_ms": report.duration_ms,
                    "success": report.success,
                })))),
                Err(e) => Ok(error_content(format!("Vacuum Error: {}", e))),
            }
        }

        // MCP-20: index recovery tools — thin wrappers over the SDK
        // (rebuild_index / audit_text_index(_deep) / repair_text_index).
        // Reports are serde-serializable; domain errors come back as
        // Ok(error_content(...)) so the LLM client can read and self-correct
        // (MEM-32), never as a propagated JSON-RPC error.
        "rebuild_index" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.rebuild_index() {
                Ok(report) => Ok(text_content(serialize_content(&json!(&report)))),
                Err(e) => Ok(error_content(format!("Rebuild Index Error: {}", e))),
            }
        }

        "audit_text_index" => {
            let namespace = match args["namespace"].as_str() {
                Some(ns) => {
                    validate_identifier(ns, "namespace", config.max_namespace_length)
                        .map_err(|e| e.to_json())?;
                    Some(ns.to_string())
                }
                None => None,
            };
            let deep = args["deep"].as_bool().unwrap_or(false);
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let result = if deep {
                embedded.audit_text_index_deep(namespace.as_deref())
            } else {
                embedded.audit_text_index(namespace.as_deref())
            };
            match result {
                Ok(report) => Ok(text_content(serialize_content(&json!(&report)))),
                Err(e) => Ok(error_content(format!("Audit Text Index Error: {}", e))),
            }
        }

        "repair_text_index" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.repair_text_index() {
                Ok(report) => Ok(text_content(serialize_content(&json!(&report)))),
                Err(e) => Ok(error_content(format!("Repair Text Index Error: {}", e))),
            }
        }

        // MCP-26: introspection/utility tools — capabilities (feature
        // introspection), generate_snippet (stateless text utility),
        // list_snapshots (physical snapshot names). Same MEM-32 error shape.
        "capabilities" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let caps = embedded.capabilities();
            Ok(text_content(serialize_content(&json!(&caps))))
        }

        "generate_snippet" => {
            let payload = args["payload"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'payload'").to_json())?;
            let text_query = args["text_query"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'text_query'").to_json())?;
            validate_payload(payload, config.max_payload_length).map_err(|e| e.to_json())?;
            let with_highlighting = args["with_highlighting"].as_bool().unwrap_or(false);

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.generate_snippet(payload, text_query, with_highlighting) {
                Some(snippet) => Ok(text_content(serialize_content(
                    &json!({ "snippet": snippet }),
                ))),
                None => Ok(text_content(serialize_content(&json!({ "snippet": null })))),
            }
        }

        "list_snapshots" => match storage.list_snapshots() {
            Ok(snapshots) => Ok(text_content(serialize_content(
                &json!({ "snapshots": snapshots }),
            ))),
            Err(e) => Ok(error_content(format!("List Snapshots Error: {}", e))),
        },

        // MCP-17: backup/restore via MCP — thin wrappers over the SDK JSONL
        // serialization (export_line_from_record / record_from_export_line +
        // import_records). Domain errors come back as Ok(error_content(...))
        // so the LLM client can read and self-correct (MEM-32), never as a
        // propagated JSON-RPC error.
        "export" => {
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let namespaces: Vec<String> = match args["namespace"].as_str() {
                Some(ns) => {
                    validate_identifier(ns, "namespace", config.max_namespace_length)
                        .map_err(|e| e.to_json())?;
                    vec![ns.to_string()]
                }
                None => match embedded.list_namespaces() {
                    Ok(ns) => ns,
                    Err(e) => return Ok(error_content(format!("Export Error: {}", e))),
                },
            };

            // ponytail: stream pages via the shared for_each_record helper
            // (bounded memory) instead of materializing every record; abort
            // once the JSONL exceeds the stdio transfer cap.
            let mut jsonl = String::new();
            let mut overflow = false;
            for ns in &namespaces {
                if overflow {
                    break;
                }
                let streamed = for_each_record(&embedded, ns, config, |record| {
                    if overflow {
                        return;
                    }
                    if let Ok(line) = serde_json::to_string(&vantadb::sdk::export_line_from_record(
                        record.clone(),
                    )) {
                        jsonl.push_str(&line);
                        jsonl.push('\n');
                        if jsonl.len() > MAX_TRANSFER_BYTES {
                            overflow = true;
                        }
                    }
                });
                if let Err(e) = streamed {
                    return Ok(error_content(format!("Export Error: {}", e)));
                }
            }
            if overflow {
                return Ok(error_content(format!(
                    "Export exceeds maximum transfer size of {} bytes — export fewer namespaces or use the CLI/SDK file export",
                    MAX_TRANSFER_BYTES
                )));
            }
            Ok(text_content(jsonl))
        }

        "import" => {
            let content = args["content"].as_str().ok_or_else(|| {
                McpError::invalid_params("Missing 'content' (JSONL string)").to_json()
            })?;
            if content.len() > MAX_TRANSFER_BYTES {
                return Ok(error_content(format!(
                    "Import content exceeds maximum transfer size of {} bytes — split the payload or use import_file via the CLI/SDK",
                    MAX_TRANSFER_BYTES
                )));
            }

            // Same per-line semantics as the core's import_file: empty lines
            // are skipped, malformed lines are counted as errors instead of
            // failing the whole call. record_from_export_line recomputes the
            // deterministic node id, which put_record_exact then validates.
            let mut records = Vec::new();
            let mut malformed = 0u64;
            let mut skipped = 0u64;
            for line in content.lines() {
                if line.trim().is_empty() {
                    skipped += 1;
                    continue;
                }
                let parsed = serde_json::from_str::<vantadb::sdk::VantaMemoryExportLine>(line)
                    .ok()
                    .and_then(|l| vantadb::sdk::record_from_export_line(l).ok());
                match parsed {
                    Some(record) => records.push(record),
                    None => malformed += 1,
                }
            }

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.import_records(records) {
                Ok(mut report) => {
                    report.skipped += skipped;
                    report.errors += malformed;
                    Ok(text_content(serialize_content(&report)))
                }
                Err(e) => Ok(error_content(format!("Import Error: {}", e))),
            }
        }

        // MCP-25: bulk ingest via MCP — thin wrappers over the SDK bulk
        // import (binary .vdbdump format, bypasses per-record validation).
        "bulk_import_file" => {
            let path = args["path"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'path'").to_json())?;
            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.bulk_import_file(path) {
                Ok(report) => Ok(text_content(serialize_content(&report))),
                Err(e) => Ok(error_content(format!(
                    "Bulk Import File Error: cannot import from '{}': {}",
                    path, e
                ))),
            }
        }

        "bulk_import_stream" => {
            let content = args["content"].as_str().ok_or_else(|| {
                McpError::invalid_params("Missing 'content' (NDJSON or .vdbdump payload)").to_json()
            })?;
            if content.len() > MAX_TRANSFER_BYTES {
                return Ok(error_content(format!(
                    "Bulk import content exceeds maximum transfer size of {} bytes — use bulk_import_file with a host-side file instead",
                    MAX_TRANSFER_BYTES
                )));
            }

            let bytes = content.as_bytes();
            let payload: Vec<u8> = if bytes.starts_with(b"VDBJSON\n") {
                // Raw .vdbdump payload — pass through as-is.
                bytes.to_vec()
            } else {
                // NDJSON (one VantaMemoryInput per line) — synthesize the
                // vdbdump header the SDK stream expects around the JSON array.
                let mut inputs = Vec::new();
                for (lineno, line) in content.lines().enumerate() {
                    if line.trim().is_empty() {
                        continue;
                    }
                    match serde_json::from_str::<vantadb::sdk::VantaMemoryInput>(line) {
                        Ok(input) => inputs.push(input),
                        Err(e) => {
                            return Ok(error_content(format!(
                                "Bulk Import Error: malformed NDJSON at line {}: {}",
                                lineno + 1,
                                e
                            )));
                        }
                    }
                }
                let body = match serde_json::to_vec(&inputs) {
                    Ok(body) => body,
                    Err(e) => return Ok(error_content(format!("Bulk Import Error: {}", e))),
                };
                let mut framed = Vec::with_capacity(17 + body.len());
                framed.extend_from_slice(b"VDBJSON\n");
                framed.push(0x01);
                framed.extend_from_slice(&(inputs.len() as u64).to_le_bytes());
                framed.extend_from_slice(&body);
                framed
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let mut reader = std::io::Cursor::new(payload);
            match embedded.bulk_import_stream(&mut reader) {
                Ok(report) => Ok(text_content(serialize_content(&report))),
                Err(e) => Ok(error_content(format!("Bulk Import Error: {}", e))),
            }
        }

        // MCP-21: GDS via MCP — thin wrappers over src/sdk/gds.rs
        // (graph_page_rank / graph_degree_centrality). Domain errors come back
        // as Ok(error_content(...)) (MEM-32), never as a propagated JSON-RPC
        // error. u128 node ids are serialized as decimal strings (JSON numbers
        // lose precision above 2^53).
        "graph_page_rank" => {
            let roots = parse_node_ids(
                args["roots"]
                    .as_array()
                    .ok_or_else(|| McpError::invalid_params("Missing 'roots' array").to_json())?,
            )?;
            let max_iterations = args["max_iterations"].as_u64().unwrap_or(100) as usize;
            let damping_factor = args["damping_factor"].as_f64().unwrap_or(0.85);
            let tolerance = args["tolerance"].as_f64().unwrap_or(1e-6);

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.graph_page_rank(&roots, max_iterations, damping_factor, tolerance) {
                Ok(scores) => {
                    let map: serde_json::Map<String, Value> = scores
                        .into_iter()
                        .map(|(id, rank)| (id.to_string(), json!(rank)))
                        .collect();
                    Ok(text_content(serialize_content(&json!({ "scores": map }))))
                }
                Err(e) => Ok(error_content(format!("Page Rank Error: {}", e))),
            }
        }

        "graph_degree_centrality" => {
            let roots = parse_node_ids(
                args["roots"]
                    .as_array()
                    .ok_or_else(|| McpError::invalid_params("Missing 'roots' array").to_json())?,
            )?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.graph_degree_centrality(&roots) {
                Ok(degrees) => {
                    let map: serde_json::Map<String, Value> = degrees
                        .into_iter()
                        .map(|(id, (inn, out))| (id.to_string(), json!({ "in": inn, "out": out })))
                        .collect();
                    Ok(text_content(serialize_content(&json!({ "degrees": map }))))
                }
                Err(e) => Ok(error_content(format!("Degree Centrality Error: {}", e))),
            }
        }

        // MCP-22: graph traversal via MCP — thin wrappers over src/sdk/graph.rs.
        // `graph_traverse` covers graph_bfs/graph_dfs plus their _filtered
        // variants (a present 'filter' object routes to the filtered SDK call).
        "graph_traverse" => {
            let start = parse_node_ids(
                args["start"]
                    .as_array()
                    .ok_or_else(|| McpError::invalid_params("Missing 'start' array").to_json())?,
            )?;
            let mode = args["mode"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'mode' (bfs or dfs)").to_json())?;
            let max_depth = args["max_depth"]
                .as_u64()
                .ok_or_else(|| McpError::invalid_params("Missing 'max_depth'").to_json())?
                as usize;
            let direction = parse_direction(&args["direction"])?;

            // Filtered variant when a filter object is present; empty labels
            // means "no label filter" in the core SDK, so an absent field
            // degrades to unfiltered behavior for that dimension.
            let filter = match args.get("filter") {
                Some(Value::Null) | None => None,
                Some(f) if !f.is_object() => {
                    return Ok(error_content(
                        "'filter' must be an object {labels, time_range}",
                    ));
                }
                Some(f) => {
                    let labels: Vec<u32> = f["labels"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_u64().map(|n| n as u32))
                                .collect()
                        })
                        .unwrap_or_default();
                    let time_range = match f.get("time_range") {
                        Some(Value::Null) | None => None,
                        Some(tr) => {
                            let pair = tr.as_array().ok_or_else(|| {
                                McpError::invalid_params("'time_range' must be [from_ms, to_ms]")
                                    .to_json()
                            })?;
                            if pair.len() != 2 {
                                return Ok(error_content(
                                    "'time_range' must have exactly two values [from_ms, to_ms]",
                                ));
                            }
                            let from = pair[0].as_u64().ok_or_else(|| {
                                McpError::invalid_params("'time_range[0]' must be ms").to_json()
                            })?;
                            let to = pair[1].as_u64().ok_or_else(|| {
                                McpError::invalid_params("'time_range[1]' must be ms").to_json()
                            })?;
                            Some((from, to))
                        }
                    };
                    Some((labels, time_range))
                }
            };

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            let result = match (mode.to_lowercase().as_str(), &filter) {
                ("bfs", None) => embedded.graph_bfs(&start, max_depth, direction),
                ("dfs", None) => embedded.graph_dfs(&start, max_depth, direction),
                ("bfs", Some((labels, time_range))) => {
                    embedded.graph_bfs_filtered(&start, max_depth, direction, labels, *time_range)
                }
                ("dfs", Some((labels, time_range))) => {
                    embedded.graph_dfs_filtered(&start, max_depth, direction, labels, *time_range)
                }
                (other, _) => {
                    return Ok(error_content(format!(
                        "Unknown mode '{}' — supported: bfs, dfs",
                        other
                    )));
                }
            };
            match result {
                Ok(visited) => {
                    let ids: Vec<String> = visited.iter().map(|id| id.to_string()).collect();
                    Ok(text_content(serialize_content(&json!({
                        "visited": ids,
                        "count": ids.len()
                    }))))
                }
                Err(e) => Ok(error_content(format!("Traversal Error: {}", e))),
            }
        }

        "graph_topological_sort" => {
            let roots = parse_node_ids(
                args["roots"]
                    .as_array()
                    .ok_or_else(|| McpError::invalid_params("Missing 'roots' array").to_json())?,
            )?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.graph_topological_sort(&roots) {
                Ok(order) => {
                    let ids: Vec<String> = order.iter().map(|id| id.to_string()).collect();
                    Ok(text_content(serialize_content(&json!({ "order": ids }))))
                }
                Err(e) => Ok(error_content(format!("Topological Sort Error: {}", e))),
            }
        }

        "graph_is_dag" => {
            let roots = parse_node_ids(
                args["roots"]
                    .as_array()
                    .ok_or_else(|| McpError::invalid_params("Missing 'roots' array").to_json())?,
            )?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.graph_is_dag(&roots) {
                Ok(is_dag) => Ok(text_content(serialize_content(
                    &json!({ "is_dag": is_dag }),
                ))),
                Err(e) => Ok(error_content(format!("Is DAG Error: {}", e))),
            }
        }

        // MOD-10: remove_edge — thin wrapper over the SDK remove_edge. u128
        // node ids travel as decimal strings (same pattern as MCP-21/22);
        // the label is validated at the boundary (mutating tool, user input).
        "remove_edge" => {
            let source_id = parse_node_id(&args["source_id"]).ok_or_else(|| {
                McpError::invalid_params("Invalid or missing 'source_id'").to_json()
            })?;
            let target_id = parse_node_id(&args["target_id"]).ok_or_else(|| {
                McpError::invalid_params("Invalid or missing 'target_id'").to_json()
            })?;
            let label = args["label"]
                .as_str()
                .ok_or_else(|| McpError::invalid_params("Missing 'label'").to_json())?;
            validate_identifier(label, "label", config.max_key_length).map_err(|e| e.to_json())?;

            let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
            match embedded.remove_edge(source_id, target_id, label) {
                Ok(()) => Ok(text_content(serialize_content(&json!({ "removed": true })))),
                Err(e) => Ok(error_content(format!("Remove Edge Error: {}", e))),
            }
        }

        "skill_list" | "skill_view" | "skill_create" | "skill_update" | "skill_patch"
        | "skill_files_write" => crate::skills::handle_skill_tool(name, args, storage, config),
        "code_search" | "code_explore" | "code_callers" | "code_callees" | "code_impact"
        | "code_node" | "code_status" | "code_files" => {
            crate::code::handle_code_tool(name, args, storage, config)
        }
        "wiki_search" | "wiki_read" | "wiki_list" | "wiki_graph" | "wiki_ingest"
        | "wiki_ingest_status" => crate::wiki::handle_wiki_tool(name, args, storage, config),
        "context_assemble" => crate::context::handle_context_tool(name, args, storage, config),
        "scene_read" | "scene_list" | "scene_query" => {
            crate::scenes::handle_scene_tool(name, args, storage, config)
        }
        "thread_create"
        | "thread_send"
        | "thread_get"
        | "thread_list"
        | "thread_delete"
        | "thread_purge_expired" => crate::threads::handle_thread_tool(name, args, storage, config),
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

/// MCP-19: parse one JSON object into a `VantaMemoryInput`, applying the same
/// validation and wire semantics as `memory_put` (identifier/payload limits,
/// sparse vector object, absolute expires_at_ms → relative ttl_ms).
fn parse_memory_input(
    obj: &Value,
    config: &McpConfig,
) -> Result<vantadb::sdk::VantaMemoryInput, Value> {
    let namespace = obj["namespace"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing 'namespace'").to_json())?;
    let key = obj["key"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing 'key'").to_json())?;
    let payload = obj["payload"]
        .as_str()
        .ok_or_else(|| McpError::invalid_params("Missing 'payload'").to_json())?;

    validate_identifier(namespace, "namespace", config.max_namespace_length)
        .map_err(|e| e.to_json())?;
    validate_identifier(key, "key", config.max_key_length).map_err(|e| e.to_json())?;
    validate_payload(payload, config.max_payload_length).map_err(|e| e.to_json())?;

    let vector = if let Some(arr) = obj["vector"].as_array() {
        Some(validate_vector(arr, config.max_vector_dim).map_err(|e| e.to_json())?)
    } else {
        None
    };

    let sparse_vector = match obj.get("sparse_vector") {
        Some(Value::Null) | None => None,
        Some(v) => {
            let sparse_obj = v.as_object().ok_or_else(|| {
                McpError::invalid_params(
                    "'sparse_vector' must be an object mapping dimension id to weight",
                )
                .to_json()
            })?;
            Some(parse_sparse_vector(sparse_obj).map_err(|e| e.to_json())?)
        }
    };

    let ttl_ms = match obj.get("expires_at_ms") {
        Some(Value::Null) | None => None,
        Some(v) => {
            let expires = v.as_u64().ok_or_else(|| {
                McpError::invalid_params("'expires_at_ms' must be an unsigned integer (Unix ms)")
                    .to_json()
            })?;
            let now_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            Some(expires.saturating_sub(now_ms))
        }
    };

    let metadata = if let Some(meta_obj) = obj["metadata"].as_object() {
        parse_metadata(meta_obj).map_err(|e| e.to_json())?
    } else {
        vantadb::sdk::VantaMemoryMetadata::new()
    };

    Ok(vantadb::sdk::VantaMemoryInput {
        key: key.to_string(),
        namespace: namespace.to_string(),
        payload: payload.to_string(),
        vector,
        sparse_vector,
        metadata,
        ttl_ms,
    })
}

/// MCP-21/22: parse an array of node ids (decimal strings, or numbers for
/// backward compat via `parse_node_id`) into u128 roots.
fn parse_node_ids(arr: &[Value]) -> Result<Vec<u128>, Value> {
    arr.iter()
        .map(|v| {
            parse_node_id(v).ok_or_else(|| {
                McpError::invalid_params(
                    "Node IDs must be decimal strings (u128 exceeds JSON number precision)",
                )
                .to_json()
            })
        })
        .collect()
}

/// MCP-22: optional traversal direction ("forward"|"reverse"|"both"), default
/// Forward — same semantics as the core `TraversalDirection`.
fn parse_direction(val: &Value) -> Result<vantadb::graph::TraversalDirection, Value> {
    match val.as_str() {
        None => Ok(vantadb::graph::TraversalDirection::Forward),
        Some(d) => match d.to_lowercase().as_str() {
            "forward" => Ok(vantadb::graph::TraversalDirection::Forward),
            "reverse" => Ok(vantadb::graph::TraversalDirection::Reverse),
            "both" => Ok(vantadb::graph::TraversalDirection::Both),
            other => Err(McpError::invalid_params(format!(
                "Unknown direction '{}' — supported: forward, reverse, both",
                other
            ))
            .to_json()),
        },
    }
}

/// MCP-24: result of parsing a shared search request.
///
/// `Ready(request)` is a fully validated request ready to run against the SDK.
/// `Rejected(envelope)` is a *domain* rejection (dimension mismatch, unsupported
/// filter) that must be surfaced to the client as `Ok(error_content(...))`
/// (MEM-32) so the LLM can read and self-correct — NOT as a propagated
/// JSON-RPC error. Param-level errors (bad types, unknown enum values) still
/// come back as `Err(Value)` (JSON-RPC invalid-params).
enum ParsedSearchRequest {
    Ready(vantadb::sdk::VantaMemorySearchRequest),
    Rejected(Value),
}

/// MCP-24: shared parsing for `search_memory` / `search_with_method` /
/// `search_multi` — one implementation, same wire shape. The caller validates
/// the target namespace(s) and passes an explicit `namespace` (the SDK's
/// `search_multi` ignores `request.namespace`, overwriting it per-namespace,
/// but the struct still needs a value).
fn parse_search_request(
    namespace: &str,
    args: &Value,
    config: &McpConfig,
    storage: &Arc<StorageEngine>,
) -> Result<ParsedSearchRequest, Value> {
    let query_vector = if let Some(arr) = args["query_vector"].as_array() {
        if arr.is_empty() {
            Vec::new()
        } else {
            validate_vector(arr, config.max_vector_dim).map_err(|e| e.to_json())?
        }
    } else {
        Vec::new()
    };

    // MCP-04: reject vector queries whose dim does not match the live index
    // dim. Without this, mismatched queries score garbage (~0.0) and silently
    // return wrong results. Text-only searches (empty query_vector) are
    // unaffected.
    if !query_vector.is_empty() {
        if let Some(expected) = index_vector_dim(storage) {
            if query_vector.len() != expected {
                return Ok(ParsedSearchRequest::Rejected(error_content(
                    vantadb::VantaError::DimensionMismatch {
                        expected,
                        got: query_vector.len(),
                    }
                    .to_string(),
                )));
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
            return Ok(ParsedSearchRequest::Rejected(error_content(format!(
                "Unknown distance_metric '{}' — supported: cosine, euclidean",
                other
            ))));
        }
        None => {
            warn!("distance_metric not specified in search request — defaulting to cosine");
            vantadb::DistanceMetric::Cosine
        }
    };

    let explain = args["explain"].as_bool().unwrap_or(false);

    // AUD-048: unified filter semantics with the CLI channel. The search
    // request (`VantaMemorySearchRequest`) is flat-only — it has no
    // `filter_ops` slot — so flat values and explicit `$eq` both fold into the
    // flat metadata (identical equality semantics). Range/inequality operators
    // ($gt/$gte/$lt/$lte/$neq) cannot be expressed in a search request; return
    // a clear error pointing at memory_list, which supports them via
    // filter_ops.
    let filters = if let Some(obj) = args["filters"].as_object() {
        let ops = parse_filter_ops(obj).map_err(|e| e.to_json())?;
        let mut flat = vantadb::sdk::VantaMemoryMetadata::new();
        for item in ops {
            if item.op == vantadb::sdk::VantaFilterOp::Eq {
                flat.insert(item.field, item.value);
            } else {
                return Ok(ParsedSearchRequest::Rejected(error_content(format!(
                    "search filters support equality only (flat values or {{\"$eq\": value}}); \
                     operator '{:?}' on field '{}' is available via memory_list filters",
                    item.op, item.field
                ))));
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
            return Ok(ParsedSearchRequest::Rejected(error_content(
                "search_profile must be an object {mode, rrf_k, candidate_k}".to_string(),
            )));
        }
        None => None,
    };

    Ok(ParsedSearchRequest::Ready(
        vantadb::sdk::VantaMemorySearchRequest {
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
        },
    ))
}

/// MCP-24: parse the optional dense-index backend override for
/// `search_with_method`. `None` keeps automatic engine routing untouched.
/// Unknown values are rejected (enum whitelist) rather than silently falling
/// back, since the input schema advertises the allowed set.
fn parse_search_method(val: &Value) -> Result<Option<vantadb::index::IndexType>, Value> {
    match val.as_str() {
        None => Ok(None),
        Some(m) => match m.to_lowercase().as_str() {
            "hnsw" => Ok(Some(vantadb::index::IndexType::Hnsw)),
            "ivf" => Ok(Some(vantadb::index::IndexType::Ivf)),
            "flat" => Ok(Some(vantadb::index::IndexType::Flat)),
            "diskann" => Ok(Some(vantadb::index::IndexType::DiskAnn)),
            "scann" => Ok(Some(vantadb::index::IndexType::Scann)),
            other => Err(McpError::invalid_params(format!(
                "Unknown method '{}' — supported: hnsw, ivf, flat, diskann, scann",
                other
            ))
            .to_json()),
        },
    }
}
