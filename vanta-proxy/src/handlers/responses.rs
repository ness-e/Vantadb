//! Generic Responses API subset - plain verbatim forward of `/v1/responses`.
//!
//! Deliberately minimal (research 07 §7): TDAM's Responses handling is coupled
//! to Codex/WorkBuddy agent adapters; those adapters are NOT ported here. The
//! endpoint forwards the request body untouched, so any client speaking the
//! generic OpenAI Responses shape works against a compatible upstream.

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::inject::Protocol;
use crate::server::AppState;

/// POST `/v1/responses` - auth→session→inject→forward (generic subset).
///
/// No `{space_id}` segment in this route (TDAM parity) - the empty string is
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

/// POST `/{agent}/{space_id}/v1/responses` - same generic subset with the
/// prefixed shape (API-05 parity with messages/models); the `space_id`
/// segment keys the limiter/reports instead of the empty legacy key.
pub async fn responses_prefixed(
    State(state): State<AppState>,
    Path((agent, space_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> Response {
    tracing::debug!(agent = %agent, space_id = %space_id, "responses (prefixed)");
    state
        .process(
            Protocol::Responses,
            "/v1/responses",
            &headers,
            body,
            &space_id,
        )
        .await
}
