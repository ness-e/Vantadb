//! MCP prompt handlers.

use crate::error::McpError;
use serde_json::{json, Value};

// ── Prompts handlers ──────────────────────────────────────────────────────

/// Handle `prompts/list`, returning the available prompt templates.
pub fn handle_prompts_list() -> Result<Value, Value> {
    Ok(json!({
        "prompts": [
            {
                "name": "search_memory",
                "description": "Recall-first memory search: memory_recall, then hybrid search_memory with deterministic temporal ranges",
                "arguments": [
                    { "name": "namespace", "description": "Target namespace for search", "required": true },
                    { "name": "query", "description": "Search query (text or vector)", "required": true },
                    { "name": "filters", "description": "Optional metadata filters", "required": false }
                ]
            },
            {
                "name": "analyze_namespace",
                "description": "Analyze a namespace for structure AND vigencia: clusters plus TTL, supersession and curation signals",
                "arguments": [
                    { "name": "namespace", "description": "Namespace to analyze", "required": true }
                ]
            },
            {
                "name": "summarize_context",
                "description": "Summarize context honouring supersession and TTL: superseded records are history, not current state",
                "arguments": [
                    { "name": "namespace", "description": "Source namespace", "required": true },
                    { "name": "limit", "description": "Number of records to include", "required": false }
                ]
            },
            {
                "name": "query_builder",
                "description": "Build IQL queries with honest temporal rules: no server-side time-travel WHERE on memory records",
                "arguments": [
                    { "name": "operation", "description": "Operation type (SELECT, INSERT, UPDATE, DELETE)", "required": true },
                    { "name": "target", "description": "Target (nodes, memory, etc.)", "required": true },
                    { "name": "conditions", "description": "Query conditions", "required": false }
                ]
            }
        ]
    }))
}

/// Handle `prompts/get`, returning the expanded prompt for a given template name.
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
                "description": "Recall-first memory search: memory_recall, then hybrid search_memory with deterministic temporal ranges",
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Recall-first search in VantaDB namespace '{}' for: '{}'. Step 1: call `memory_recall` with this query (scope agent, top_k 5). Step 2: if the question carries a temporal expression (e.g. 'yesterday at 2pm'), translate it to a DETERMINISTIC [from_ms, to_ms] range — never guess; an unresolvable expression falls back to the last 30 days and you must say so. Step 3: run `search_memory` (hybrid vector + text) and, for time-bounded questions, page `memory_list` keeping only records whose created_at_ms falls inside the range (`search_memory` filters are equality-only). Only inject recalled hits into context when recalled is non-empty; when nothing is recalled, say so instead of filling the gap.", namespace, query)}}]
            }))
        }
        "analyze_namespace" => {
            let namespace = args
                .and_then(|a| a["namespace"].as_str())
                .unwrap_or("default");
            Ok(json!({
                "description": "Analyze a namespace for structure AND vigencia: clusters plus TTL, supersession and curation signals",
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Analyze the VantaDB namespace '{}' for structure AND vigencia. List records, examine metadata patterns and identify clusters; then check curation signals: records past their TTL (`purge_expired` candidates), superseded records (follow the `memory_supersede` chain via `memory_versions`), pending approval-inbox items, and near-duplicates to merge. Report what is current, what is obsolete, and what needs a human decision.", namespace)}}]
            }))
        }
        "summarize_context" => {
            let namespace = args
                .and_then(|a| a["namespace"].as_str())
                .unwrap_or("default");
            let limit = args.and_then(|a| a["limit"].as_u64()).unwrap_or(10);
            Ok(json!({
                "description": "Summarize context honouring supersession and TTL: superseded records are history, not current state",
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Retrieve the last {} records from namespace '{}' and summarize key themes, relationships and important information. Exclude superseded records from the 'current state' (check `memory_versions` when a record looks replaced); flag records past their TTL as expired, not valid. Quote record keys so the summary stays traceable.", limit, namespace)}}]
            }))
        }
        "query_builder" => {
            let operation = args
                .and_then(|a| a["operation"].as_str())
                .unwrap_or("SELECT");
            let target = args.and_then(|a| a["target"].as_str()).unwrap_or("nodes");
            let conditions = args.and_then(|a| a["conditions"].as_str()).unwrap_or("");
            Ok(json!({
                "description": "Build IQL queries with honest temporal rules: no server-side time-travel WHERE on memory records",
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Build an IQL query for VantaDB. Operation: {}, Target: {}, Conditions: {}. Rules: IQL is not Cypher and not LISP (LINK does not exist — use RELATE); UPDATE uses SET field = value. Temporal conditions on memory records have NO server-side WHERE filter — express them as a deterministic [from_ms, to_ms] range and apply client-side over created_at_ms from `memory_list` pages (graph edge windows use `graph_traverse` time_range).", operation, target, conditions)}}]
            }))
        }
        _ => McpError::invalid_params(format!("Prompt not found: {}", name)).into_err(),
    }
}
