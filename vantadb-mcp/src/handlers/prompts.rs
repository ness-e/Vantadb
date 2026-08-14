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
                "messages": [{"role": "user", "content": {"type": "text", "text": format!("Build an IQL query for VantaDB. Operation: {}, Target: {}, Conditions: {}. Ensure the query follows IQL syntax and is properly formatted.", operation, target, conditions)}}]
            }))
        }
        _ => McpError::invalid_params(format!("Prompt not found: {}", name)).into_err(),
    }
}
