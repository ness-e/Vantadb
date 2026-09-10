//! Typed proxy errors mapped to HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ProxyError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("upstream unreachable: {0}")]
    UpstreamUnreachable(String),
    #[error("upstream timeout")]
    UpstreamTimeout,
    #[error("forward failed: {0}")]
    Forward(String),
    /// D34: request without a valid `x-vanta-user-key` — always rejected.
    #[error("unauthorized: missing or invalid user key")]
    Unauthorized,
    /// Malformed request against the local session state machine.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// Local VantaDB storage failure while resolving auth/session/memory.
    #[error("local storage error: {0}")]
    Storage(String),
    /// PRX-03: virtual key over budget with enforcement on. `key` is the
    /// resolved D34 `user_id` (identity, never the raw secret header).
    #[error(
        "budget exceeded for virtual key `{key}`: spent ${spent_usd:.6} of ${budget_usd:.6} budget"
    )]
    BudgetExceeded {
        key: String,
        spent_usd: f64,
        budget_usd: f64,
    },
    /// PRX-07: egress redaction blocked the request in `Block` mode.
    /// `kinds` are labels only (`aws_key`, `email`, …) — never matched
    /// values, so 422 responses can't echo secrets.
    #[error("request blocked by egress redaction: {kinds:?}")]
    RedactionBlocked { kinds: Vec<String> },
    /// PRX-10: per-key model allowlist denied the request. `key` is the
    /// resolved D34 `user_id` (identity, never the raw secret header);
    /// `model` is the requested model name — neither echoes secrets.
    #[error("model `{model}` not allowed for virtual key `{key}`")]
    GuardrailBlocked { key: String, model: String },
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        // Budget shape is its own `type` (no Retry-After: not a window).
        if let ProxyError::BudgetExceeded {
            key,
            spent_usd,
            budget_usd,
        } = &self
        {
            let body = json!({ "error": {
                "type": "budget_exceeded",
                "message": self.to_string(),
                "key": key,
                "spent_usd": spent_usd,
                "budget_usd": budget_usd,
            } });
            return (StatusCode::TOO_MANY_REQUESTS, axum::Json(body)).into_response();
        }
        // Redaction shape is its own `type` (kinds only, never values).
        if let ProxyError::RedactionBlocked { kinds } = &self {
            let body = json!({ "error": {
                "type": "redaction_blocked",
                "message": self.to_string(),
                "kinds": kinds,
            } });
            return (StatusCode::UNPROCESSABLE_ENTITY, axum::Json(body)).into_response();
        }
        // Guardrail shape is its own `type` (identity + model, never secrets).
        if let ProxyError::GuardrailBlocked { key, model } = &self {
            let body = json!({ "error": {
                "type": "guardrail_blocked",
                "message": self.to_string(),
                "key": key,
                "model": model,
            } });
            return (StatusCode::FORBIDDEN, axum::Json(body)).into_response();
        }
        let (status, message) = match &self {
            ProxyError::UpstreamTimeout => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            ProxyError::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            ProxyError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ProxyError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ProxyError::UpstreamUnreachable(_) | ProxyError::Forward(_) => {
                // Contract: upstream down → 502 with clear typed message.
                (StatusCode::BAD_GATEWAY, self.to_string())
            }
            ProxyError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            // Defensive: the early returns above serve the `budget_exceeded`
            // and `redaction_blocked` shapes; these arms only satisfy
            // exhaustiveness.
            ProxyError::BudgetExceeded { .. } => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            ProxyError::RedactionBlocked { .. } => {
                (StatusCode::UNPROCESSABLE_ENTITY, self.to_string())
            }
            ProxyError::GuardrailBlocked { .. } => (StatusCode::FORBIDDEN, self.to_string()),
        };
        let body = json!({ "error": { "type": "proxy_error", "message": message } });
        (status, axum::Json(body)).into_response()
    }
}
