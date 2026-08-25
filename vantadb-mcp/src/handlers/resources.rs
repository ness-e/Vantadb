//! MCP resource handlers.

use crate::config::McpConfig;
use crate::error::McpError;
use crate::validation::*;
use serde_json::{json, Value};
use std::sync::Arc;
use vantadb::storage::StorageEngine;

// ── Resources handlers ────────────────────────────────────────────────────

/// Handle `resources/list`, returning the available operational-metrics resource.
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
                "description": "Database schema information including HNSW configuration and text index version",
                "mimeType": "application/json"
            }
        ]
    }))
}

/// Handle `resources/read`, serving metrics, memory records or namespace listings.
pub fn handle_resources_read(
    params: &Option<Value>,
    storage: &Arc<StorageEngine>,
    config: &McpConfig,
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
        let schema = build_schema_resource(storage);
        let text = serialize_content(&schema);
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

        if let Err(e) = validate_identifier(namespace, "namespace", config.max_namespace_length) {
            return e.into_err();
        }
        if let Err(e) = validate_identifier(key, "key", config.max_key_length) {
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
        if let Err(e) = validate_identifier(namespace, "namespace", config.max_namespace_length) {
            return e.into_err();
        }

        let embedded = vantadb::VantaEmbedded::from_engine(storage.clone());
        // MOD-11 (H7): page size aligned to the memory_list default instead
        // of a hardcoded 100. The resource returns the FIRST page with its
        // `next_cursor`; full pagination (up to config.max_list_limit) lives
        // in the memory_list tool, which accepts the cursor — documented in
        // SKILL.md § Available MCP Resources.
        let options = vantadb::sdk::VantaMemoryListOptions {
            limit: config.default_list_limit,
            cursor: None,
            #[allow(deprecated)]
            filters: vantadb::sdk::VantaMemoryMetadata::new(),
            filter_ops: None,
            exclude_superseded: false,
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

/// Build the `schema://` resource payload: active HNSW configuration and
/// text index schema/tokenizer version.
pub(crate) fn build_schema_resource(storage: &Arc<StorageEngine>) -> Value {
    let index = storage.vec_index();
    let hnsw_config = serde_json::to_value(index.config.clone()).unwrap_or_else(|_| json!({}));
    let text_spec = vantadb::TextIndexSpec::default();
    json!({
        "vector_index": {
            "type": "HNSW",
            "format_version": vantadb::VECTOR_INDEX_VERSION,
            "config": hnsw_config
        },
        "text_index": {
            "schema_version": text_spec.schema_version,
            "tokenizer": {
                "name": text_spec.tokenizer.name,
                "version": text_spec.tokenizer.version
            },
            "key_format": text_spec.key_format
        }
    })
}
