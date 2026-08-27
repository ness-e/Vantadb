//! MCP `initialize` request handler.

use serde_json::{json, Value};
use vantadb::metadata;

/// Latest stable MCP protocol version this server advertises.
pub const LATEST_PROTOCOL_VERSION: &str = "2025-06-18";

/// All protocol versions this server understands (latest first).
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2025-06-18", "2024-11-05"];

// ── initialize handler ────────────────────────────────────────────────────

/// Handle the `initialize` request, returning protocol version, server info and capabilities.
///
/// Negotiation: if `params.protocolVersion` is a supported version, echo it
/// back; otherwise default to `LATEST_PROTOCOL_VERSION`. This keeps old
/// clients (2024-11-05) working while new clients get 2025-06-18
/// (structured output, annotations). Unknown versions fall forward to latest
/// rather than erroring — forward-compatible per MCP spec guidance.
pub fn handle_initialize(params: Option<&Value>) -> Result<Value, Value> {
    let requested = params
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str());
    let negotiated = match requested {
        Some(v) if SUPPORTED_PROTOCOL_VERSIONS.contains(&v) => v,
        _ => LATEST_PROTOCOL_VERSION,
    };
    Ok(json!({
        "protocolVersion": negotiated,
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
