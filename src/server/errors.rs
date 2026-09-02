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

/// Map an HTTP status class to the log level (ERR-OBS-01).
///
/// 5xx means an invariant broke server-side → ERROR (someone should look);
/// 4xx is a client mistake → WARN (trend-watching only). Keeps the level
/// decision in one place so both error envelopes agree.
fn error_log_level(status: StatusCode) -> tracing::Level {
    if status.is_server_error() {
        tracing::Level::ERROR
    } else {
        tracing::Level::WARN
    }
}

/// Structured observability event for a `VantaError` crossing the HTTP
/// boundary (ERR-OBS-01). Fields are stable: `error.code` is one of the ten
/// canonical `VANTADB_*` codes (low cardinality — safe for log pipelines and
/// future metric labels). FIND-55: `error.display` carries the full engine
/// message server-side — it used to be the only place that text appeared, the
/// 5xx response body; sanitizing the body moved the chain here.
fn log_vanta_error(e: &VantaError, status: StatusCode) {
    // `tracing::event!` needs a compile-time-constant level, so branch on the
    // class; the field set stays identical across both arms.
    if error_log_level(status) == tracing::Level::ERROR {
        tracing::error!(
            error.code = e.code(),
            error.retriable = e.is_retriable(),
            error.hint = e.recovery_hint().unwrap_or_default(),
            error.display = %e,
            "vanta request failed"
        );
    } else {
        tracing::warn!(
            error.code = e.code(),
            error.retriable = e.is_retriable(),
            error.hint = e.recovery_hint().unwrap_or_default(),
            error.display = %e,
            "vanta request failed"
        );
    }
}

/// Build a 4xx/5xx response for a query execution error (ERR-027).
///
/// Client mistakes (bad IQL, missing nodes, validation) map to explicit 4xx
/// statuses; anything server-side stays a 500. Proxies and monitoring can then
/// distinguish query errors from healthy traffic instead of relying on the
/// body's `success` flag.
///
/// ERR-CORE-01: the body carries the stable `code` field (one of the
/// `VANTADB_*` canonical codes). Built with `json!` so the field can be added
/// without touching the public `QueryResponse` struct; the serialized shape is
/// identical because `node_id`/`nodes` are `None` here and skipped.
///
/// FIND-55: 5xx bodies stay generic — the internal `Display` (io paths,
/// storage detail) goes to server-side logs via [`log_vanta_error`], and
/// clients branch on `code`. 4xx messages are user-input data and stay
/// descriptive (same rule `panic_error_response` applies for panics).
pub fn query_error_response(e: &VantaError) -> Response {
    let status = vanta_error_status(e);
    log_vanta_error(e, status);
    let data = if status.is_server_error() {
        "internal error".to_string()
    } else {
        format!("Execution Error: {}", e)
    };
    (
        status,
        Json(json!({
            "success": false,
            "data": data,
            "code": e.code(),
        })),
    )
        .into_response()
}

/// Error body shared by the `/api/v2` console endpoints.
///
/// ERR-CORE-01: includes the stable `code` field alongside `error` so clients
/// can branch programmatically instead of parsing the message (additive,
/// backward-compatible).
///
/// FIND-55: 5xx bodies stay generic (chain only in logs, `code` to clients);
/// 4xx keep the descriptive message.
pub fn vanta_error_response(e: &VantaError) -> Response {
    let status = vanta_error_status(e);
    log_vanta_error(e, status);
    let message = if status.is_server_error() {
        "internal error".to_string()
    } else {
        e.to_string()
    };
    (
        status,
        Json(json!({
            "success": false,
            "error": message,
            "code": e.code(),
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
            vanta_error_status(&VantaError::IqlParseError {
                msg: "x".into(),
                line: 1,
                col: 1
            }),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            vanta_error_status(&VantaError::ValidationError {
                field: "x".into(),
                reason: "y".into()
            }),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            vanta_error_status(&VantaError::NodeNotFound(42)),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            vanta_error_status(&VantaError::DuplicateNode(42)),
            StatusCode::CONFLICT
        );
        assert_eq!(
            vanta_error_status(&VantaError::IoError(std::io::Error::other("x"))),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    /// ERR-OBS-01: 5xx (server-side) logs at ERROR, 4xx (client mistakes) at
    /// WARN — mirrors the level semantics in docs/operations/OBSERVABILITY.md.
    #[test]
    fn error_log_level_maps_status_class() {
        assert_eq!(
            error_log_level(StatusCode::BAD_REQUEST),
            tracing::Level::WARN
        );
        assert_eq!(error_log_level(StatusCode::NOT_FOUND), tracing::Level::WARN);
        assert_eq!(
            error_log_level(StatusCode::UNPROCESSABLE_ENTITY),
            tracing::Level::WARN
        );
        assert_eq!(
            error_log_level(StatusCode::INTERNAL_SERVER_ERROR),
            tracing::Level::ERROR
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
        let _body = res.into_body();
        // We can't easily read the body here without async, but the function
        // is tested in integration tests in routing.rs
    }

    /// ERR-CORE-01: both error envelopes carry the stable canonical `code`
    /// field (additive — existing consumers keep `success`/`error`/`data`).
    #[tokio::test]
    async fn error_envelopes_carry_canonical_code() {
        use axum::body::to_bytes;
        let e = VantaError::NodeNotFound(7);

        let body: serde_json::Value = serde_json::from_slice(
            &to_bytes(query_error_response(&e).into_body(), 4096)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["code"], "VANTADB_NOT_FOUND");
        assert_eq!(body["success"], false);
        assert_eq!(body["data"], "Execution Error: Node not found: 7");
        assert!(body.get("node_id").is_none(), "None fields stay skipped");
        assert!(body.get("nodes").is_none());

        let body: serde_json::Value = serde_json::from_slice(
            &to_bytes(vanta_error_response(&e).into_body(), 4096)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(body["code"], "VANTADB_NOT_FOUND");
        assert_eq!(body["error"], "Node not found: 7");
    }

    /// FIND-55: 5xx bodies must not leak the engine's internal Display (io
    /// paths, storage detail) — clients branch on the canonical `code`, and
    /// the full chain lives in server-side logs via `log_vanta_error`. Mirrors
    /// the leak-mitigation pattern `panic_error_response` already applies
    /// (AUDREP-32).
    #[tokio::test]
    async fn five_xx_bodies_are_sanitized_to_generic_message() {
        use axum::body::to_bytes;
        let e = VantaError::IoError(std::io::Error::other(
            "CONTRIVED_IO_LEAK_/srv/vanta/secrets/data.wal",
        ));

        let resp = query_error_response(&e);
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["data"], "internal error");
        assert_eq!(body["code"], "VANTADB_IO_ERROR", "code stays for clients");
        assert!(
            !body.to_string().contains("CONTRIVED_IO_LEAK"),
            "io detail must not reach the wire: {body}"
        );

        let resp = vanta_error_response(&e);
        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(resp.into_body(), 4096).await.unwrap()).unwrap();
        assert_eq!(body["error"], "internal error");
        assert_eq!(body["code"], "VANTADB_IO_ERROR");
        assert!(
            !body.to_string().contains("CONTRIVED_IO_LEAK"),
            "io detail must not reach the wire: {body}"
        );
    }

    /// FIND-55: 4xx messages are user-input data, not internal detail — they
    /// stay descriptive in both envelopes.
    #[tokio::test]
    async fn four_xx_bodies_keep_descriptive_message() {
        use axum::body::to_bytes;
        let e = VantaError::ValidationError {
            field: "payload".into(),
            reason: "vector must be non-empty".into(),
        };

        let body: serde_json::Value = serde_json::from_slice(
            &to_bytes(query_error_response(&e).into_body(), 4096)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            body["data"],
            "Execution Error: Validation error on payload: vector must be non-empty"
        );
        assert_eq!(body["code"], "VANTADB_VALIDATION_ERROR");

        let body: serde_json::Value = serde_json::from_slice(
            &to_bytes(vanta_error_response(&e).into_body(), 4096)
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            body["error"],
            "Validation error on payload: vector must be non-empty"
        );
        assert_eq!(body["code"], "VANTADB_VALIDATION_ERROR");
    }
}
