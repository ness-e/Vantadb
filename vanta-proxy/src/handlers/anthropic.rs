//! Anthropic Messages wire handler (TDAM parity: server.ts:307).

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::inject::Protocol;
use crate::server::AppState;

/// POST `/{agent}/{spaceId}/v1/messages` — auth→session→inject→forward.
pub async fn messages(
    State(state): State<AppState>,
    Path((agent, space_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> Response {
    tracing::debug!(agent = %agent, space_id = %space_id, "messages");
    state
        .process(Protocol::Anthropic, "/v1/messages", &headers, body)
        .await
}
