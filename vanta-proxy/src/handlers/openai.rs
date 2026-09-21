//! OpenAI Chat Completions wire handler (TDAM parity: server.ts:312).

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::inject::Protocol;
use crate::server::AppState;

/// POST `/{agent}/{spaceId}/v1/chat/completions` — auth→session→inject→forward.
pub async fn chat_completions(
    State(state): State<AppState>,
    Path((agent, space_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> Response {
    tracing::debug!(agent = %agent, space_id = %space_id, "chat/completions");
    state
        .process(
            Protocol::OpenAI,
            "/v1/chat/completions",
            &headers,
            body,
            &space_id,
        )
        .await
}
