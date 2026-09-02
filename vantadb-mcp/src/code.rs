//! MCP tool handlers for the `code_*` tools (MEM-32).
//!
//! Eight query-only tools over the built-in graphrag (`src/graphrag/` +
//! `src/graph.rs` traversal primitives) — D28: no external codegraph
//! dependency. Every tool is a thin read-only wrapper; all logic lives in the
//! core SDK (no duplicated semantics).
//!
//! Tool↔primitive mapping (documented per plan pre-mortem 1):
//!
//! | Tool | Primitive | Edge direction |
//! |---|---|---|
//! | `code_search` | `VantaEmbedded::graphrag_search` (own pipeline) | n/a |
//! | `code_explore` | `get_node` + depth-1 BFS Forward/Reverse | explicit |
//! | `code_callers` | depth-1 BFS `Reverse` minus root (incoming edges) | Reverse |
//! | `code_callees` | depth-1 BFS `Forward` minus root (outgoing edges) | Forward |
//! | `code_impact` | `graph_bfs(id, max_depth, direction)` reachable subgraph | parameterized |
//! | `code_node` | `get_node` → `VantaNodeRecord` | n/a |
//! | `code_status` | `operational_metrics()` snapshot | n/a |
//! | `code_files` | **not supported** stub — the own graphrag has no
//!   file-per-node concept (D28); TDAM's file semantics are not ported | n/a |

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::{
    error_content, error_content_vanta, serialize_content, text_content, validate_identifier,
};
use serde_json::{json, Value};
use std::sync::Arc;
use vantadb::graph::TraversalDirection;
use vantadb::sdk::VantaEmbedded;
use vantadb::storage::StorageEngine;

/// Default traversal depth for `code_impact`.
const IMPACT_DEFAULT_DEPTH: usize = 3;
/// Hard cap for `code_impact` depth — keeps a runaway traversal bounded.
const IMPACT_MAX_DEPTH: usize = 10;

/// Tool definitions for `tools/list` (MEM-32).
pub(crate) fn code_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "code_search",
            "description": "GraphRAG search over the built-in pipeline (seed → expand → retrieve → context). Read-only.",
            "annotations": {
                "title": "Code Search",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Namespace to search" },
                    "query": { "type": "string", "description": "Text query used to seed the pipeline" }
                },
                "required": ["namespace", "query"]
            }
        }),
        json!({
            "name": "code_explore",
            "description": "Inspects a node plus its direct neighborhood, separating outgoing (callees) from incoming (callers) neighbors. Read-only.",
            "annotations": {
                "title": "Code Explore",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Node ID to explore (decimal string; u128 ids exceed JSON number precision)" }
                },
                "required": ["node_id"]
            }
        }),
        json!({
            "name": "code_callers",
            "description": "Lists nodes that point AT the given node (incoming edges), via reverse-edge traversal. Read-only.",
            "annotations": {
                "title": "Code Callers",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Node ID whose callers to list (decimal string)" }
                },
                "required": ["node_id"]
            }
        }),
        json!({
            "name": "code_callees",
            "description": "Lists nodes the given node points TO (outgoing edges), via forward-edge traversal. Read-only.",
            "annotations": {
                "title": "Code Callees",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Node ID whose callees to list (decimal string)" }
                },
                "required": ["node_id"]
            }
        }),
        json!({
            "name": "code_impact",
            "description": "Returns every node reachable from the given node within max_depth hops, following edges only in the requested direction (default Forward). Read-only.",
            "annotations": {
                "title": "Code Impact",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Root node ID (decimal string)" },
                    "max_depth": { "type": "number", "description": "Max hops, default 3, capped at 10" },
                    "direction": { "type": "string", "enum": ["Forward", "Reverse", "Both"], "description": "Edge direction to follow, default Forward" }
                },
                "required": ["node_id"]
            }
        }),
        json!({
            "name": "code_node",
            "description": "Fetches a single graph node by ID as a full record. Read-only.",
            "annotations": {
                "title": "Code Node",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": {
                "type": "object",
                "properties": {
                    "node_id": { "type": "string", "description": "Node ID (decimal string)" }
                },
                "required": ["node_id"]
            }
        }),
        json!({
            "name": "code_status",
            "description": "Returns an operational-metrics snapshot of the engine backing the graph (node counts, index state). Read-only.",
            "annotations": {
                "title": "Code Status",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        }),
        json!({
            "name": "code_files",
            "description": "NOT SUPPORTED: the VantaDB built-in graphrag has no file-per-node concept; always returns an error explaining this.",
            "annotations": {
                "title": "Code Files",
                "readOnlyHint": true,
                "destructiveHint": false,
                "idempotentHint": true,
                "openWorldHint": false
            },
            "inputSchema": { "type": "object", "properties": {}, "required": [] }
        }),
    ]
}

/// Dispatch a `tools/call` for one of the `code_*` tools.
///
/// Param errors surface as JSON-RPC invalid-params; domain errors (missing
/// node, unsupported tool) surface as `error_content` results the LLM can
/// self-correct — matching the existing MCP tool pattern.
pub(crate) fn handle_code_tool(
    name: &str,
    args: &Value,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
) -> Result<Value, Value> {
    let embedded = VantaEmbedded::from_engine(storage.clone());
    match name {
        "code_search" => {
            let namespace = required_str(args, "namespace")?;
            validate_identifier(namespace, "namespace", config.max_namespace_length)
                .map_err(|e| e.to_json())?;
            let query = required_str(args, "query")?;

            match embedded.graphrag_search(namespace, Some(query), None) {
                Ok(result) => {
                    let nodes: Vec<Value> = result
                        .nodes
                        .iter()
                        .map(|n| {
                            json!({
                                "id": n.id.to_string(),
                                "content": n.content,
                                "score": n.score,
                                "hop_distance": n.hop_distance,
                            })
                        })
                        .collect();
                    let edges: Vec<Value> = result
                        .edges
                        .iter()
                        .map(|e| {
                            json!({
                                "source": e.source.to_string(),
                                "target": e.target.to_string(),
                                "label": e.label,
                            })
                        })
                        .collect();
                    Ok(text_content(serialize_content(&json!({
                        "nodes": nodes,
                        "edges": edges,
                        "context_text": result.context_text,
                        "stats": {
                            "seeds_found": result.stats.seeds_found,
                            "nodes_expanded": result.stats.nodes_expanded,
                            "total_candidates": result.stats.total_candidates,
                            "expansion_hops_used": result.stats.expansion_hops_used,
                        },
                    }))))
                }
                Err(e) => Ok(error_content_vanta(e)),
            }
        }

        "code_explore" => {
            let node_id = required_node(args)?;
            let record = match fetch_record(&embedded, node_id)? {
                Some(record) => record,
                None => return Ok(error_content(format!("Node not found: {}", node_id))),
            };
            let outgoing = neighbors(&embedded, &[node_id], 1, TraversalDirection::Forward);
            let incoming = neighbors(&embedded, &[node_id], 1, TraversalDirection::Reverse);
            Ok(text_content(serialize_content(&json!({
                "node": record,
                "outgoing": outgoing,
                "incoming": incoming,
            }))))
        }

        "code_callers" => {
            let node_id = required_node(args)?;
            let callers = neighbors(&embedded, &[node_id], 1, TraversalDirection::Reverse);
            Ok(text_content(serialize_content(
                &json!({ "callers": callers, "count": callers.len() }),
            )))
        }

        "code_callees" => {
            let node_id = required_node(args)?;
            let callees = neighbors(&embedded, &[node_id], 1, TraversalDirection::Forward);
            Ok(text_content(serialize_content(
                &json!({ "callees": callees, "count": callees.len() }),
            )))
        }

        "code_impact" => {
            let node_id = required_node(args)?;
            let raw_depth = args["max_depth"]
                .as_u64()
                .unwrap_or(IMPACT_DEFAULT_DEPTH as u64);
            let max_depth = (raw_depth as usize).clamp(1, IMPACT_MAX_DEPTH);
            // Domain errors surface as error_content (self-correctable), not
            // as JSON-RPC protocol errors.
            let direction = match args["direction"].as_str() {
                None | Some("Forward") => TraversalDirection::Forward,
                Some("Reverse") => TraversalDirection::Reverse,
                Some("Both") => TraversalDirection::Both,
                Some(other) => {
                    return Ok(error_content(format!(
                        "Invalid direction '{}' — expected 'Forward', 'Reverse', or 'Both'",
                        other
                    )))
                }
            };
            let ids = embedded
                .graph_bfs(&[node_id], max_depth, direction)
                .map_err(|e| McpError::from(e).to_json())?;
            // bfs includes the root itself; impact reports what it REACHES.
            let reached: Vec<String> = ids
                .iter()
                .filter(|&&id| id != node_id)
                .map(|id| id.to_string())
                .collect();
            Ok(text_content(serialize_content(&json!({
                "root": node_id.to_string(),
                "direction": format!("{:?}", direction),
                "max_depth": max_depth,
                "reached": reached,
                "count": reached.len(),
            }))))
        }

        "code_node" => {
            let node_id = required_node(args)?;
            let record = match fetch_record(&embedded, node_id)? {
                Some(record) => record,
                None => return Ok(error_content(format!("Node not found: {}", node_id))),
            };
            Ok(text_content(serialize_content(&record)))
        }

        "code_status" => {
            let metrics = embedded.operational_metrics();
            let snapshot = serde_json::to_value(&metrics)
                .map_err(|e| McpError::internal_error(e.to_string()).to_json())?;
            Ok(text_content(serialize_content(&json!({
                "metrics": snapshot,
                "read_only": embedded.capabilities().read_only,
            }))))
        }

        "code_files" => Ok(error_content(
            "code_files: not supported — the VantaDB built-in graphrag has no file-per-node \
             concept (D28). Use code_search / code_explore / code_node instead.",
        )),

        _ => McpError::method_not_found(format!("Tool not found: {}", name)).into_err(),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

/// Extract a required string argument.
fn required_str<'a>(args: &'a Value, field: &str) -> Result<&'a str, Value> {
    args[field]
        .as_str()
        .ok_or_else(|| McpError::invalid_params(format!("Missing or invalid '{field}'")).to_json())
}

/// Parse a required decimal-string node id.
fn required_node(args: &Value) -> Result<u128, Value> {
    crate::validation::parse_node_id(&args["node_id"])
        .ok_or_else(|| McpError::invalid_params("Invalid or missing 'node_id'").to_json())
}

/// Fetch a node record. Engine failures are protocol errors; a missing node
/// yields `Ok(None)` so callers can respond with the domain-error
/// `error_content` shape (self-correctable), matching `memory_get`.
fn fetch_record(
    embedded: &VantaEmbedded,
    node_id: u128,
) -> Result<Option<vantadb::sdk::VantaNodeRecord>, Value> {
    embedded
        .get_node(node_id)
        .map_err(|e| McpError::from(e).to_json())
}

/// Depth-1 BFS in `direction`, excluding the root itself, each neighbor
/// serialized as a full node record.
fn neighbors(
    embedded: &VantaEmbedded,
    roots: &[u128],
    max_depth: usize,
    direction: TraversalDirection,
) -> Vec<Value> {
    let Ok(ids) = embedded.graph_bfs(roots, max_depth, direction) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for id in ids {
        if roots.contains(&id) {
            continue;
        }
        if let Ok(Some(record)) = embedded.get_node(id) {
            if let Ok(value) = serde_json::to_value(&record) {
                out.push(value);
            }
        }
    }
    out
}
