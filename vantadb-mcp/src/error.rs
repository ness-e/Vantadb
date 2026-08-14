//! Error type for the MCP server.

use serde_json::{json, Value};

// ── Error type ─────────────────────────────────────────────────────────────

/// Structured JSON-RPC error.
#[derive(Debug)]
pub struct McpError {
    /// JSON-RPC error code (e.g. -32700 for parse error, -32602 for invalid params).
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
}

impl McpError {
    /// Create a parse error (-32700) with the given message.
    pub fn parse_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: format!("Parse error: {}", msg.into()),
        }
    }

    /// Create an invalid-params error (-32602) with the given message.
    pub fn invalid_params(msg: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: msg.into(),
        }
    }

    /// Create a method-not-found error (-32601) with the given message.
    pub fn method_not_found(msg: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: msg.into(),
        }
    }

    /// Create an internal-error (-32603) with the given message.
    pub fn internal_error(msg: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: msg.into(),
        }
    }

    /// Create an invalid-request error (-32600) with the given message.
    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: msg.into(),
        }
    }

    /// Serialize this error to a JSON-RPC error object.
    pub fn to_json(&self) -> Value {
        json!({"code": self.code, "message": self.message})
    }

    /// Convert this error into an `Err(Value)` result.
    pub fn into_err<T>(self) -> Result<T, Value> {
        Err(self.to_json())
    }
}
