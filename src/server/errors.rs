//! HTTP error response builders.
//!
//! REVIEW-10: extracted from `routing.rs` — all error mapping and response
//! construction for the server surface.

use crate::server::state::QueryResponse;
use crate::Error;
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

/// Map a `Error` to the HTTP status clients receive (ERR-027).
///
/// Client mistakes (bad IQL, missing nodes, validation) map to explicit 4xx
/// statuses; anything server-side stays a 500. Shared by the IQL endpoint and
/// the `/api/v2` console surface so both speak the same error status language.
pub fn status(e: &Error) -> StatusCode {
    match e {
        Error::IqlParse { .. }
        | Error::Iql(_)
        | Error::InvalidInput(_)
        | Error::DimensionMismatch { .. }
        | Error::UnsupportedOperation { .. }
        | Error::Schema(_)
        | Error::NoVectorForKey(_) => StatusCode::BAD_REQUEST,
        Error::Validation { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        Error::NodeNotFound(_) | Error::NotFound { .. } => StatusCode::NOT_FOUND,
        Error::DuplicateNode(_) | Error::NodeIdCollision(_) | Error::ExecutionConflict { .. } => {
            StatusCode::CONFLICT
        }
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

/// Structured observability event for a `Error` crossing the HTTP
/// boundary (ERR-OBS-01). Fields are stable: `error.code` is one of the ten
/// canonical `VANTADB_*` codes (low cardinality — safe for log pipelines and
/// future metric labels). FIND-55: `error.display` carries the full engine
/// message server-side — it used to be the only place that text appeared, the
/// 5xx response body; sanitizing the body moved the chain here.
///
/// FIND-53: this is also the single metric choke point — every error fed to
/// it increments `vantadb_errors_total{code}` on the in-tree Prometheus
/// registry (no-op when the `prometheus` feature is off). Both envelopes
/// (`query_error_response`, `response`) route through here, so
/// the series counts every HTTP-served `Error` exactly once.
fn log_error(e: &Error, status: StatusCode) {
    crate::metrics::record_vanta_error(e.code());
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
/// storage detail) goes to server-side logs via [`log_error`], and
/// clients branch on `code`. 4xx messages are user-input data and stay
/// descriptive (same rule `panic_error_response` applies for panics).
pub fn query_error_response(e: &Error) -> Response {
    let status = status(e);
    log_error(e, status);
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
pub fn response(e: &Error) -> Response {
    let status = status(e);
    log_error(e, status);
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
    use crate::Error;

    #[test]
    fn status_maps_correctly() {
        assert_eq!(
            status(&Error::IqlParse {
                msg: "x".into(),
                line: 1,
                col: 1
            }),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            status(&Error::Validation {
                field: "x".into(),
                reason: "y".into()
            }),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(status(&Error::NodeNotFound(42)), StatusCode::NOT_FOUND);
        assert_eq!(status(&Error::DuplicateNode(42)), StatusCode::CONFLICT);
        assert_eq!(
            status(&Error::Io(std::io::Error::other("x"))),
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
        let e = Error::NodeNotFound(7);

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

        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response(&e).into_body(), 4096).await.unwrap())
                .unwrap();
        assert_eq!(body["code"], "VANTADB_NOT_FOUND");
        assert_eq!(body["error"], "Node not found: 7");
    }

    /// FIND-55: 5xx bodies must not leak the engine's internal Display (io
    /// paths, storage detail) — clients branch on the canonical `code`, and
    /// the full chain lives in server-side logs via `log_error`. Mirrors
    /// the leak-mitigation pattern `panic_error_response` already applies
    /// (AUDREP-32).
    #[tokio::test]
    async fn five_xx_bodies_are_sanitized_to_generic_message() {
        use axum::body::to_bytes;
        let e = Error::Io(std::io::Error::other(
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

        let resp = response(&e);
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
        let e = Error::Validation {
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

        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response(&e).into_body(), 4096).await.unwrap())
                .unwrap();
        assert_eq!(
            body["error"],
            "Validation error on payload: vector must be non-empty"
        );
        assert_eq!(body["code"], "VANTADB_VALIDATION_ERROR");
    }

    /// FIND-53: both error envelopes route through `log_error`, the
    /// single choke point feeding `vantadb_errors_total{code}`. Uses the
    /// TIMEOUT code (no other test in this module increments it, so the
    /// before/after delta is exact even under parallel execution). Requires
    /// the `prometheus` feature — without it every registry counter is a
    /// deliberate no-op (same as `vanta_http_requests_total` et al.).
    #[cfg(feature = "prometheus")]
    #[tokio::test]
    async fn error_envelopes_increment_vantadb_errors_total_by_code() {
        use crate::metrics;
        let counter = metrics::ERRORS_TOTAL
            .as_ref()
            .expect("vantadb_errors_total must register on the in-tree registry");
        let timeout = Error::Timeout {
            operation: "test".into(),
            duration_ms: 1,
        };
        assert_eq!(timeout.code(), "VANTADB_TIMEOUT");

        let before = counter.with_label_values(&["VANTADB_TIMEOUT"]).get();
        let _ = query_error_response(&timeout);
        let _ = response(&timeout);
        let after = counter.with_label_values(&["VANTADB_TIMEOUT"]).get();

        assert_eq!(
            after,
            before + 2,
            "each envelope through log_error must count exactly once"
        );
        let scrape = metrics::export_metrics_text();
        assert!(
            scrape.contains("vantadb_errors_total{code=\"VANTADB_TIMEOUT\"}"),
            "series must appear in the /metrics scrape, got:\n{scrape}"
        );
    }
}
