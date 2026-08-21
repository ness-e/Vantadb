//! Generic Responses API subset — plain verbatim forward of `/v1/responses`.
//!
//! Deliberately minimal (research 07 §7): TDAM's Responses handling is coupled
//! to Codex/WorkBuddy agent adapters; those adapters are NOT ported here. The
//! endpoint forwards the request body untouched, so any client speaking the
//! generic OpenAI Responses shape works against a compatible upstream.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};

use crate::server::AppState;

/// POST `/v1/responses` — forward verbatim (generic subset, no agent adapters).
pub async fn responses(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> Response {
    tracing::debug!("forwarding responses (generic subset)");
    state
        .forwarder
        .forward(
            &state.config.upstream,
            axum::http::Method::POST,
            "/v1/responses",
            &headers,
            body,
        )
        .await
        .unwrap_or_else(IntoResponse::into_response)
}
