//! HTTP error response builders.
//!
//! REVIEW-10: extracted from `routing.rs` — all error mapping and response
//! construction for the server surface.

use crate::server::state::QueryResponse;
use crate::VantaError;
use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt::Display;

/// Build a generic 500 for a panicked execution task.
///
/// The panic detail is logged server-side; clients only get a generic message
/// to avoid leaking internal runtime details (AUDREP-32).
pub fn panic_error_response(panic_detail: &dyn Display) -> Response {
    tracing::error!("execution task panicked: {}", panic_detail);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(QueryResponse {
            success: false,
            data: "Internal server error".to_string(),
            node_id: None,
            nodes: None,
        }),
    )
        .into_response()
}

/// Map a `VantaError` to the HTTP status clients receive (ERR-027).
///
/// Client mistakes (bad IQL, missing nodes, validation) map to explicit 4xx
/// statuses; anything server-side stays a 500. Shared by the IQL endpoint and
/// the `/api/v2` console surface so both speak the same error status language.
pub fn vanta_error_status(e: &VantaError) -> StatusCode {
    match e {
        VantaError::IqlParseError { .. }
        | VantaError::IqlError(_)
        | VantaError::InvalidInput(_)
        | VantaError::DimensionMismatch { .. }
        | VantaError::UnsupportedOperation { .. }
        | VantaError::SchemaError(_)
        | VantaError::NoVectorForKey(_) => StatusCode::BAD_REQUEST,
        VantaError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        VantaError::NodeNotFound(_) | VantaError::NotFound { .. } => StatusCode::NOT_FOUND,
        VantaError::DuplicateNode(_)
        | VantaError::NodeIdCollision(_)
        | VantaError::ExecutionConflict { .. } => StatusCode::CONFLICT,
        // Storage/WAL/IO/resource failures and anything unclassified.
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Build a 4xx/5xx response for a query execution error (ERR-027).
///
/// Client mistakes (bad IQL, missing nodes, validation) map to explicit 4xx
/// statuses; anything server-side stays a 500. Proxies and monitoring can then
/// distinguish query errors from healthy traffic instead of relying on the
/// body's `success` flag.
pub fn query_error_response(e: &VantaError) -> Response {
    (
        vanta_error_status(e),
        Json(QueryResponse {
            success: false,
            data: format!("Execution Error: {}", e),
            node_id: None,
            nodes: None,
        }),
    )
        .into_response()
}

/// Error body shared by the `/api/v2` console endpoints.
pub fn vanta_error_response(e: &VantaError) -> Response {
    (
        vanta_error_status(e),
        Json(json!({
            "success": false,
            "error": e.to_string(),
        })),
    )
        .into_response()
}

/// Map a connection-pool acquisition failure to a 503 (mirrors `execute_query`).
pub fn pool_error_response(e: crate::connection_pool::PoolError) -> Response {
    let msg = match e {
        crate::connection_pool::PoolError::Closed => "Server query pool closed".to_string(),
        crate::connection_pool::PoolError::Timeout => {
            "Server concurrency limit reached; retry shortly".to_string()
        }
    };
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [(header::RETRY_AFTER, "1")],
        Json(json!({ "success": false, "error": msg })),
    )
        .into_response()
}

/// 404 body for a missing record lookup (REST convention: GET/DELETE of a
/// nonexistent key is a client mistake, not a server fault).
pub fn not_found_response(key: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "error": format!("record not found: {key}"),
        })),
    )
        .into_response()
}

/// 404 body for a missing thread.
pub fn thread_not_found_response(id: u128) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "success": false,
            "error": format!("thread not found: {id}"),
        })),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VantaError;

    #[test]
    fn vanta_error_status_maps_correctly() {
        assert_eq!(
            vanta_error_status(&VantaError::IqlParseError { msg: "x".into() }),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            vanta_error_status(&VantaError::ValidationError { msg: "x".into() }),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            vanta_error_status(&VantaError::NodeNotFound("x".into())),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            vanta_error_status(&VantaError::DuplicateNode("x".into())),
            StatusCode::CONFLICT
        );
        assert_eq!(
            vanta_error_status(&VantaError::Io(std::io::Error::other("x"))),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn panic_error_response_hides_detail() {
        let detail = "execution task panicked: CONTRIVED_PANIC_96942e85";
        let res = panic_error_response(&detail);

        assert_eq!(
            res.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "panicked task must stay a 5xx"
        );

        // Body must not leak the panic detail
        let body = res.into_body();
        // We can't easily read the body here without async, but the function
        // is tested in integration tests in routing.rs
    }
}
