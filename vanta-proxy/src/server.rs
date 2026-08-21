//! Router assembly (TDAM parity: server.ts:307,312 — primary agent-prefixed routes).

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::json;

use crate::config::ProxyConfig;
use crate::forward::Forwarder;
use crate::handlers;

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ProxyConfig>,
    pub forwarder: Arc<Forwarder>,
}

impl AppState {
    /// Build state from a loaded configuration.
    ///
    /// # Errors
    /// Returns [`crate::error::ProxyError::Config`] if the HTTP client cannot be built.
    pub fn new(config: ProxyConfig) -> Result<Self, crate::error::ProxyError> {
        let forwarder = Forwarder::new(&config.upstream)?;
        Ok(Self {
            config: Arc::new(config),
            forwarder: Arc::new(forwarder),
        })
    }
}

/// Assemble the proxy router.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route(
            "/{agent}/{spaceId}/v1/chat/completions",
            post(handlers::openai::chat_completions),
        )
        .route(
            "/{agent}/{spaceId}/v1/messages",
            post(handlers::anthropic::messages),
        )
        .route("/v1/responses", post(handlers::responses::responses))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
