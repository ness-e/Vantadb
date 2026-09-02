//! Error type for the MCP server.

use serde_json::{json, Value};
use vantadb::VantaError;

// ── Error type ─────────────────────────────────────────────────────────────

/// Structured JSON-RPC error.
#[derive(Debug)]
pub struct McpError {
    /// JSON-RPC error code (e.g. -32700 for parse error, -32602 for invalid params).
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
    /// Optional structured payload (`data.code` canonical string, `retriable`,
    /// `hint`) per docs/api/ERROR_HANDLING.md §6.3. `Null` → omitted on the wire.
    pub data: Value,
}

impl McpError {
    fn plain(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: Value::Null,
        }
    }

    /// Create a parse error (-32700) with the given message.
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self::plain(-32700, format!("Parse error: {}", msg.into()))
    }

    /// Create an invalid-params error (-32602) with the given message.
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self::plain(-32602, msg.into())
    }

    /// Create an invalid-params error (-32602) carrying the canonical
    /// `VANTADB_VALIDATION_ERROR` code in `data` (ERR-MCP-01). Used by the
    /// input validators; the JSON-RPC code stays -32602 for compat.
    pub fn validation(msg: impl Into<String>) -> Self {
        let mut e = Self::invalid_params(msg);
        e.data = json!({
            "code": "VANTADB_VALIDATION_ERROR",
            "retriable": false,
        });
        e
    }

    /// Create a method-not-found error (-32601) with the given message.
    pub fn method_not_found(msg: impl Into<String>) -> Self {
        Self::plain(-32601, msg.into())
    }

    /// Create an internal-error (-32603) with the given message.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self::plain(-32603, msg.into())
    }

    /// Create an invalid-request error (-32600) with the given message.
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::plain(-32600, msg.into())
    }

    /// Build the structured error from a domain error, by reference so the
    /// tool handlers can enrich `message` (context suffixes) before rendering.
    pub fn from_domain(e: &VantaError) -> Self {
        // Mapping is driven by the canonical `VantaError::code()` (ERR-CORE-01),
        // never by re-matching variants — that would duplicate the core table
        // (docs/api/ERROR_HANDLING.md §6.2). Codes with no §6.2 row fall back
        // to -32603 internal_error; conflict variants arrive folded into
        // VANTADB_VALIDATION_ERROR (see docs/api/MCP.md -32003 note).
        let code = match e.code() {
            "VANTADB_BUSY" => -32001,
            "VANTADB_CORRUPT" => -32002,
            "VANTADB_NOT_FOUND" => -32004,
            "VANTADB_RESOURCE_LIMIT" => -32007,
            "VANTADB_TIMEOUT" => -32008,
            "VANTADB_VALIDATION_ERROR" | "VANTADB_INVALID_ARGUMENT" => -32009,
            _ => -32603,
        };
        let mut data = json!({
            "code": e.code(),
            "retriable": e.is_retriable(),
        });
        if let Some(hint) = e.recovery_hint() {
            data["hint"] = json!(hint);
        }
        Self {
            code,
            message: e.to_string(),
            data,
        }
    }

    /// Serialize this error to a JSON-RPC error object.
    pub fn to_json(&self) -> Value {
        if self.data.is_null() {
            json!({"code": self.code, "message": self.message})
        } else {
            json!({"code": self.code, "message": self.message, "data": self.data})
        }
    }

    /// Convert this error into an `Err(Value)` result.
    pub fn into_err<T>(self) -> Result<T, Value> {
        Err(self.to_json())
    }
}

// ── Domain → wire mapping (ERR-MCP-01) ─────────────────────────────────────

/// Maps every `VantaError` onto its canonical JSON-RPC code plus a `data`
/// envelope (`code`, `retriable`, `hint`) LLM clients can branch on
/// programmatically (docs/api/ERROR_HANDLING.md §6.2/§6.3).
impl From<VantaError> for McpError {
    fn from(e: VantaError) -> Self {
        Self::from_domain(&e)
    }
}
