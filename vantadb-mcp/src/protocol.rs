//! JSON-RPC wire types for the MCP protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;

// ── JSON-RPC wire types ────────────────────────────────────────────────────

/// Deserialize an explicit `"id": null` as `Some(Value::Null)` instead of
/// serde's default `None`. JSON-RPC 2.0 treats a *present* null id as a
/// request and only an *absent* one as a notification — the distinction
/// must survive deserialization.
fn keep_explicit_null<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(Value::deserialize(deserializer)?))
}

#[derive(Deserialize)]
pub(crate) struct RpcRequest {
    pub(crate) jsonrpc: String,
    /// JSON-RPC message id. `None` ⇒ notification: per JSON-RPC 2.0 §4.1 the
    /// server MUST NOT reply to it (MOD-07 — required `id` used to reject
    /// notifications with a spurious -32700). An explicit `"id": null` is
    /// still a request (`Some(Value::Null)`) via [`keep_explicit_null`].
    #[serde(default, deserialize_with = "keep_explicit_null")]
    pub(crate) id: Option<Value>,
    pub(crate) method: String,
    pub(crate) params: Option<Value>,
}

#[derive(Serialize)]
pub(crate) struct RpcResponse {
    pub(crate) jsonrpc: String,
    pub(crate) id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<Value>,
}
