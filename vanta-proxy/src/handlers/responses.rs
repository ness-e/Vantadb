//! Generic Responses API subset — plain verbatim forward of `/v1/responses`.
//!
//! Deliberately minimal (research 07 §7): TDAM's Responses handling is coupled
//! to Codex/WorkBuddy agent adapters; those adapters are NOT ported here. The
//! endpoint forwards the request body untouched, so any client speaking the
//! generic OpenAI Responses shape works against a compatible upstream.

use axum::extract::State;
use axum::http::HeaderMap;
use axum::response::Response;

use crate::inject::Protocol;
use crate::server::AppState;

/// POST `/v1/responses` — auth→session→inject→forward (generic subset).
///
/// No `{spaceId}` segment in this route (TDAM parity) — the empty string is
/// the limiter/report space key.
pub async fn responses(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> Response {
    tracing::debug!("responses (generic subset)");
    state
        .process(Protocol::Responses, "/v1/responses", &headers, body, "")
        .await
}
