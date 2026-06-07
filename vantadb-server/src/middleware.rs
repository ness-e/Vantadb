//! Security middleware for the VantaDB HTTP server.
//!
//! Provides Bearer token authentication as an Axum middleware function.
//! Rate limiting is configured directly in [`crate::server::app`] using
//! `tower-governor`'s `GovernorLayer`.
//!
//! # Authentication behaviour
//! - [`AuthState::api_key`] is `None` → all requests pass through (dev mode).
//! - `Authorization: Bearer <token>` absent or wrong → `401 Unauthorized`.
//! - `/health` endpoint is always exempt (Docker / Fly.io liveness probe).

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

/// Shared authentication context injected as an Axum [`Extension`].
///
/// Clone is cheap — the inner `Arc<str>` is reference-counted.
#[derive(Clone)]
pub struct AuthState {
    /// The expected Bearer token value, or `None` when auth is disabled.
    pub api_key: Option<Arc<str>>,
}

impl AuthState {
    /// Creates a new [`AuthState`] from an optional key string.
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            api_key: api_key.map(|k| Arc::from(k.as_str())),
        }
    }
}

/// Axum middleware that enforces Bearer token authentication.
///
/// # Behaviour
/// - `/health` → always exempt (liveness / readiness probe compatibility).
/// - No API key configured ([`AuthState::api_key`] is `None`) → transparent pass-through.
/// - Missing or invalid `Authorization: Bearer <token>` → `401 Unauthorized` with JSON body.
pub async fn auth_middleware(
    axum::extract::Extension(auth): axum::extract::Extension<AuthState>,
    req: Request,
    next: Next,
) -> Response {
    // Health probe is always exempt — required for Docker/Fly.io liveness checks.
    if req.uri().path() == "/health" {
        return next.run(req).await;
    }

    // No API key configured → development / embedded mode, no auth enforced.
    let Some(expected_key) = &auth.api_key else {
        return next.run(req).await;
    };

    // Extract and validate the Authorization header.
    let authorized = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|token| token == expected_key.as_ref())
        .unwrap_or(false);

    if authorized {
        next.run(req).await
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "Unauthorized",
                "hint": "Provide a valid Bearer token in the Authorization header."
            })),
        )
            .into_response()
    }
}
