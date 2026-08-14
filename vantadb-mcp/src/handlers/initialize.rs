//! MCP `initialize` request handler.

use serde_json::{json, Value};
use vantadb::metadata;

// ── initialize handler ────────────────────────────────────────────────────

/// Handle the `initialize` request, returning protocol version, server info and capabilities.
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
