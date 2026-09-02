//! HTTP middleware: auth, rate limiting, circuit breaker, request metrics.
//!
//! REVIEW-10: extracted from `routing.rs` — all middleware that wraps the
//! request/response cycle. Handlers live in `handlers.rs`.

use crate::audit::AuditEvent;
use crate::circuit_breaker::CircuitBreaker;
use crate::connection_pool::ConnectionPool;
use crate::entity::EntityStore;
use crate::rbac::{Permission, Rbac};
use crate::server::state::{
    audit_auth, client_ip as state_client_ip, extract_namespace, extract_request_id,
    resolve_identity as state_resolve_identity, resolve_user_key, AuthIdentity, AuthState,
    RequestId, AUTH_ENTITY_NS, SERVICE_ID_HEADER, USER_KEY_HEADER,
};
use crate::storage::StorageEngine;
use crate::VantaError;
use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Extension, Json,
};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use tracing;
use tracing::Instrument;

/// Re-export the state-level client_ip for internal use.
pub(crate) fn client_ip(req: &Request, trusted_proxies: &[std::net::IpAddr]) -> String {
    state_client_ip(req, trusted_proxies)
}

/// Re-export the state-level resolve_identity for internal use.
pub(crate) fn resolve_identity(
    req: &Request,
    auth: &AuthState,
) -> std::result::Result<AuthIdentity, (StatusCode, &'static str)> {
    state_resolve_identity(req, auth)
}

/// Axum middleware that validates Bearer tokens and enforces RBAC permissions.
///
/// Returns 401 instead of panicking if `AuthState` is missing from request
/// extensions (invariant violated — e.g. router misconfigured).
pub async fn auth_middleware(mut req: Request, next: Next) -> Response {
    // Health endpoint is always public
    if req.uri().path() == "/health" {
        return next.run(req).await;
    }

    let auth = match req.extensions().get::<AuthState>() {
        Some(a) => a.clone(),
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Unauthorized: authentication state not available",
                })),
            )
                .into_response();
        }
    };

    // SRV-02: correlate audit auth events with the caller's tracing id.
    let request_id = req
        .extensions()
        .get::<RequestId>()
        .and_then(|r| r.0.clone());

    // No API key configured — allow all (dev mode), but surface it so the
    // silent auth bypass is visible (not rate-limited: tracing::rate_limited is
    // unstable and unavailable in the pinned tracing 0.1.44).
    let Some(expected_key) = &auth.api_key else {
        tracing::warn!(
            method = %req.method(),
            path = %req.uri().path(),
            "no API key configured; allowing unauthenticated request (dev mode)"
        );
        return next.run(req).await;
    };

    // Extract client IP for rate limiting (respects X-Forwarded-For)
    let client_ip = client_ip(&req, &auth.trusted_proxies);

    // Check rate limiting before processing auth
    if auth.rate_limiter.is_rate_limited(&client_ip) {
        audit_auth(
            &auth,
            AuditEvent::auth("l1", "auth", "N/A", "err", Some("rate_limited".into()))
                .with_request_id_opt(request_id.clone()),
        );
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "success": false,
                "error": "Too many authentication failures. Try again later.",
            })),
        )
            .into_response();
    }

    let token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    // SRV-04: accept either primary or alt API key for zero-downtime rotation.
    let authorized = match token {
        Some(token) => {
            let token_bytes = token.as_bytes();
            let primary_ok = expected_key.as_bytes().ct_eq(token_bytes).into();
            let alt_ok = auth
                .alt_api_key
                .as_ref()
                .map(|alt| alt.as_bytes().ct_eq(token_bytes).into())
                .unwrap_or(false);
            primary_ok || alt_ok
        }
        None => false,
    };

    if !authorized {
        auth.rate_limiter.record_failure(&client_ip);
        let reason = if token.is_some() {
            "invalid_token"
        } else {
            "missing_token"
        };
        audit_auth(
            &auth,
            AuditEvent::auth("l1", "auth", "N/A", "err", Some(reason.into()))
                .with_request_id_opt(request_id.clone()),
        );
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Unauthorized",
                "hint": "Provide a valid Bearer token in the Authorization header."
            })),
        )
            .into_response();
    }

    // L1 passed → resolve L2/L3 identity (MEM-05). Resolution is
    // deny-by-default: any failure fails closed with 401 and never leaks
    // internal state.
    let identity = match resolve_identity(&req, &auth) {
        Ok(identity) => identity,
        Err((status, reason)) => {
            audit_auth(
                &auth,
                AuditEvent::auth("l3", "auth", "N/A", "err", Some(reason.into()))
                    .with_request_id_opt(request_id.clone()),
            );
            return (
                status,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Unauthorized",
                    "hint": "Provide a valid x-vanta-user-key header."
                })),
            )
                .into_response();
        }
    };

    // Coarse transport RBAC applies only to bare Bearer (L1) identities —
    // service (L2) and user (L3) identities authorize downstream via their
    // resolved principal (PermissionChecker).
    if identity == AuthIdentity::Transport {
        if let Some(token_val) = token {
            if let Some(role) = auth.token_role_map.get(token_val) {
                // SRV-05: namespace-scoped RBAC for record/search endpoints.
                // Extract namespace from path/query for /api/v2/records/* and /api/v2/search.
                let path = req.uri().path();
                let is_record_endpoint = path.starts_with("/api/v2/records")
                    || path.starts_with("/api/v2/search")
                    || path.starts_with("/api/v2/list");
                let namespace = if is_record_endpoint {
                    extract_namespace(req.uri().path(), req.uri().query())
                } else {
                    None
                };
                let is_write = matches!(req.method().as_str(), "POST" | "PUT" | "PATCH" | "DELETE");
                let permitted = if let Some(ns) = namespace {
                    auth.rbac.can_access_namespace(role, &ns, is_write)
                } else {
                    // Fallback to global permissions for non-record endpoints or when ns not found
                    let permission = if is_write {
                        Permission::Write
                    } else {
                        Permission::Read
                    };
                    auth.rbac.has_permission(role, &permission)
                };
                if !permitted {
                    auth.rate_limiter.reset(&client_ip);
                    return (
                        StatusCode::FORBIDDEN,
                        Json(serde_json::json!({
                            "success": false,
                            "error": "Forbidden: insufficient permissions for this operation",
                        })),
                    )
                        .into_response();
                }
            }
        }
    }
    auth.rate_limiter.reset(&client_ip);

    // Audit identity-establishing outcomes (L2/L3). Bare L1 successes are the
    // transport baseline and are not recorded — avoids flooding the log with
    // one `auth_l1 ok` per request.
    match &identity {
        AuthIdentity::Service { service_id } => {
            audit_auth(
                &auth,
                AuditEvent::auth("l2", "auth", service_id, "ok", None)
                    .with_request_id_opt(request_id.clone()),
            );
        }
        AuthIdentity::User {
            user_id,
            is_system_admin,
        } => {
            audit_auth(
                &auth,
                AuditEvent::auth(
                    "l3",
                    "auth",
                    user_id,
                    "ok",
                    Some(format!("is_system_admin={is_system_admin}")),
                )
                .with_request_id_opt(request_id),
            );
        }
        AuthIdentity::Transport => {}
    }

    req.extensions_mut().insert(identity);
    next.run(req).await
}

/// Axum middleware that records HTTP request duration and status metrics.
///
/// Also captures the caller's tracing id (SRV-02): the first match of
/// `x-request-id` / `x-tracing-id` / `traceparent` is exposed to handlers via
/// request extensions (for audit correlation) and recorded on the request span.
pub async fn request_metrics_middleware(mut req: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = req.method().to_string();
    let route = req.uri().path().to_string();
    let request_id = extract_request_id(req.headers());
    if let Some(id) = &request_id {
        req.extensions_mut().insert(RequestId(Some(id.clone())));
    }
    let span = tracing::info_span!("http_request", request_id = tracing::field::Empty);
    if let Some(id) = &request_id {
        span.record("request_id", id.as_str());
    }
    let res = next.run(req).instrument(span).await;
    let status = res.status();
    metrics::record_http_request(&method, &route, status.as_u16(), start);
    res
}

/// Fast-fail requests while the circuit breaker is open.
///
/// When the breaker allows the request, records success/failure from the
/// resulting status code (>=500 trips the breaker). Returns `503` with a
/// `Retry-After` header while open.
pub async fn circuit_breaker_middleware(
    State(state): State<Arc<crate::server::state::ServerState>>,
    req: Request,
    next: Next,
) -> Response {
    if !state.circuit_breaker.allow_request() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            [(
                header::RETRY_AFTER,
                state.circuit_breaker.retry_after_secs().to_string(),
            )],
            Json(serde_json::json!({
                "success": false,
                "error": "Service temporarily unavailable: circuit breaker open",
            })),
        )
            .into_response();
    }

    let res = next.run(req).await;
    if res.status().is_server_error() {
        state.circuit_breaker.record_failure();
    } else {
        state.circuit_breaker.record_success();
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;
    use crate::server::state::ServerState;

    #[test]
    fn client_ip_ignores_xff_without_trusted_proxy() {
        // No trusted proxy configured → a forged header must be ignored and the
        // real socket address returned. This is the AUDREP-11 regression guard:
        // a direct client cannot spoof its recorded IP.
        let peer = "198.51.100.5:4444".parse().unwrap();
        let req = Request::builder()
            .header("x-forwarded-for", "203.0.113.99")
            .extension(axum::extract::ConnectInfo(peer))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(client_ip(&req, &[]), "198.51.100.5");
    }

    #[test]
    fn client_ip_uses_xff_from_trusted_proxy() {
        // Peer is a configured proxy → the X-Forwarded-For value is used.
        let proxy = "10.0.0.5:4444".parse().unwrap();
        let req = Request::builder()
            .header("x-forwarded-for", "203.0.113.99")
            .extension(axum::extract::ConnectInfo(proxy))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            client_ip(&req, &["10.0.0.5".parse().unwrap()]),
            "203.0.113.99"
        );
    }

    #[test]
    fn client_ip_uses_first_valid_ip_in_xff() {
        let proxy = "10.0.0.5:4444".parse().unwrap();
        let req = Request::builder()
            .header("x-forwarded-for", "203.0.113.1, 198.51.100.7")
            .extension(axum::extract::ConnectInfo(proxy))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            client_ip(&req, &["10.0.0.5".parse().unwrap()]),
            "203.0.113.1"
        );
    }

    #[test]
    fn client_ip_ignores_xff_from_untrusted_peer_with_proxy_list() {
        // The list of trusted proxies is non-empty, but this request's peer is
        // NOT one of them, so X-Forwarded-For must still be ignored.
        let direct = "198.51.100.9:5555".parse().unwrap();
        let req = Request::builder()
            .header("x-forwarded-for", "203.0.113.99")
            .extension(axum::extract::ConnectInfo(direct))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(
            client_ip(&req, &["10.0.0.5".parse().unwrap()]),
            "198.51.100.9"
        );
    }

    #[test]
    fn client_ip_simple_remote_addr_no_xff() {
        // Untrusted: x-forwarded-for ignored, socket addr returned.
        let peer = "198.51.100.5:4444".parse().unwrap();
        let req = Request::builder()
            .header("x-forwarded-for", "203.0.113.99")
            .extension(axum::extract::ConnectInfo(peer))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(client_ip(&req, &[]), "198.51.100.5");
    }
}
