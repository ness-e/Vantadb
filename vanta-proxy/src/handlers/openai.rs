//! OpenAI Chat Completions wire handler (TDAM parity: server.ts:312).

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};

use crate::server::AppState;

/// POST `/{agent}/{spaceId}/v1/chat/completions` — forward verbatim.
pub async fn chat_completions(
    State(state): State<AppState>,
    Path((agent, space_id)): Path<(String, String)>,
    headers: HeaderMap,
    body: bytes::Bytes,
) -> Response {
    tracing::debug!(agent = %agent, space_id = %space_id, "forwarding chat/completions");
    state
        .forwarder
        .forward(
            &state.config.upstream,
            axum::http::Method::POST,
            "/v1/chat/completions",
            &headers,
            body,
        )
        .await
        .unwrap_or_else(IntoResponse::into_response)
}
