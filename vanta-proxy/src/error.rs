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
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
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
        };
        let body = json!({ "error": { "type": "proxy_error", "message": message } });
        (status, axum::Json(body)).into_response()
    }
}
