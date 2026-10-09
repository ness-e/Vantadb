//! HTTP middleware: auth, rate limiting, circuit breaker, request metrics.
//!
//! REVIEW-10: extracted from `routing.rs` — all middleware that wraps the
//! request/response cycle. Handlers live in `handlers.rs`.

use crate::audit::AuditEvent;
use crate::metrics;
use crate::rbac::{AccessMode, Permission};
use crate::server::state::{
    audit_auth, body_namespaces_for_route, client_ip as state_client_ip, extract_namespace,
    extract_request_id, is_body_namespace_route, resolve_identity as state_resolve_identity,
    AuthIdentity, AuthState, RequestId,
};
use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use std::time::Instant;
use subtle::ConstantTimeEq;
use tracing::Instrument;

/// Re-export the state-level client_ip for internal use.
pub fn client_ip(req: &Request, trusted_proxies: &[std::net::IpAddr]) -> String {
    state_client_ip(req, trusted_proxies)
}

/// Re-export the state-level resolve_identity for internal use.
pub fn resolve_identity(
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
    let key_ok = match token {
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

    // SRV-06: HS256 JWT fallback (ADR-039) — offline verification, only when a
    // secret is configured. A valid JWT grants the same L1 transport identity
    // as a valid API key; failures fall through to the generic 401 below
    // (no api-key-vs-JWT oracle, same body and hint).
    let jwt_ok = match (token, auth.jwt_secret.as_deref()) {
        (Some(t), Some(secret)) => super::jwt::verify_jwt(t, secret).is_ok(),
        _ => false,
    };
    let authorized = key_ok || jwt_ok;

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
    // MEMG-04: clone the mapped role out first — the body read below needs
    // `&mut req`, which the borrowed `token` would otherwise block.
    let transport_role: Option<String> = if identity == AuthIdentity::Transport {
        token.and_then(|t| auth.token_role_map.get(t)).cloned()
    } else {
        None
    };
    if let Some(role) = transport_role.as_deref() {
        // `admin` bypasses namespace scoping by definition
        // (`Permission::Admin` short-circuits `can_access_namespace`) — skip
        // extraction and the body read.
        if !auth.rbac.has_permission(role, &Permission::Admin) {
            // SRV-05: namespace-scoped RBAC for record/search/list endpoints
            // (namespace in path or query).
            let path = req.uri().path().to_string();
            let query = req.uri().query().map(str::to_string);
            let is_scoped_route = path.starts_with("/api/v2/records")
                || path.starts_with("/api/v2/search")
                || path.starts_with("/api/v2/list");
            let mut namespaces: Vec<String> = if is_scoped_route {
                extract_namespace(&path, query.as_deref())
                    .into_iter()
                    .collect()
            } else {
                Vec::new()
            };
            // MEMG-04: surfaces whose namespace travels in the JSON body
            // (records put/batch, search, export, import). Body-declared
            // namespaces join the check as a union — a matching `?namespace=`
            // must never mask a foreign namespace in the body.
            //
            // Fail closed on empty declarations: when the body declares no
            // namespace (blank/missing field, empty body, all-namespaces
            // export, path import), the handler may treat the operation as
            // "all namespaces" (e.g. `search_all`) — so the coarse GLOBAL
            // permission is additionally required, on top of any path/query
            // namespace check. A scoped credential cannot ride `?namespace=own`
            // while the empty body escalates the operation to all namespaces;
            // global roles keep their coarse semantics.
            let mut require_coarse = false;
            if is_body_namespace_route(&path, req.method().as_str()) {
                match read_body_json(&mut req).await {
                    Ok(Some(value)) => {
                        let body_namespaces = body_namespaces_for_route(&path, &value);
                        if body_namespaces.is_empty() {
                            require_coarse = true;
                        } else {
                            for ns in body_namespaces {
                                if !namespaces.contains(&ns) {
                                    namespaces.push(ns);
                                }
                            }
                        }
                    }
                    Ok(None) => require_coarse = true,
                    Err(resp) => return resp,
                }
            }
            // `POST /api/v2/search` is a read operation (no mutation): map it
            // to `AccessMode::Read` so a least-privilege scoped role with
            // only `namespace_read` can search its own namespace. All other
            // write methods keep the write mapping (DELETE-is-write is the
            // compat behavior; strict per-action separation is FIND-301).
            let is_search_post =
                req.method() == axum::http::Method::POST && path == "/api/v2/search";
            let mode = if is_search_post {
                AccessMode::Read
            } else {
                match req.method().as_str() {
                    "POST" | "PUT" | "PATCH" | "DELETE" => AccessMode::Write,
                    _ => AccessMode::Read,
                }
            };
            let is_write = matches!(mode, AccessMode::Write);
            // MEMG-10: the *request* action label for the audit (the
            // enforcement mode below keeps the compat mapping — DELETE is
            // still covered by write; strict per-action enforcement is
            // FIND-301). `scope` names which check ran.
            let action = if is_search_post {
                "read"
            } else {
                match req.method().as_str() {
                    "DELETE" => "delete",
                    "POST" | "PUT" | "PATCH" => "write",
                    _ => "read",
                }
            };
            let enforced = if is_write { "write" } else { "read" };
            let coarse_ok = {
                let permission = if is_write {
                    Permission::Write
                } else {
                    Permission::Read
                };
                auth.rbac.has_permission(role, &permission)
            };
            // Fail closed on the first namespace the role cannot access.
            let denied_ns: Option<&str> = namespaces
                .iter()
                .find(|ns| !auth.rbac.can_access_namespace(role, ns, mode))
                .map(String::as_str);
            // MEMG-04: an empty body declaration on a body-declared route
            // may make the handler operate across ALL namespaces — the coarse
            // global permission is required too, and when that is what denied
            // the request it is audited under `scope=global`.
            let denied_by_coarse = require_coarse && !coarse_ok;
            let (permitted, scope) = if namespaces.is_empty() {
                // No namespace anywhere: coarse check only (non-scoped
                // routes, or an ambiguous body).
                (coarse_ok, "global")
            } else if denied_ns.is_none() && denied_by_coarse {
                (false, "global")
            } else {
                (denied_ns.is_none(), "namespace")
            };
            if !permitted {
                auth.rate_limiter.reset(&client_ip);
                // MGR-04/MEMG-10: RBAC denials were silent — audit them
                // (metadata only: role + namespace + action; never the token).
                audit_auth(
                    &auth,
                    AuditEvent::auth(
                        "rbac",
                        denied_ns.unwrap_or("N/A"),
                        role,
                        "denied",
                        Some(format!("action={action};enforced={enforced};scope={scope}")),
                    )
                    .with_request_id_opt(request_id.clone()),
                );
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

/// Max bytes buffered to inspect a body for namespace extraction. Matches the
/// router's `DefaultBodyLimit` (1 MB) plus headroom, so a body the handler
/// would accept is never rejected by this gate.
const BODY_NAMESPACE_READ_LIMIT: usize = 1_100_000;

/// MEMG-04: read the request body (bounded) as JSON and **restore** the body
/// so downstream handlers still receive it.
///
/// `Ok(None)` = empty body. `Err(response)` = the gate itself rejects the
/// request (oversized → 413; malformed JSON → 400) — only role-bearing
/// credentials on body-namespace routes reach this function.
async fn read_body_json(
    req: &mut Request,
) -> std::result::Result<Option<serde_json::Value>, Response> {
    let placeholder = Request::new(axum::body::Body::empty());
    let (parts, body) = std::mem::replace(req, placeholder).into_parts();
    let bytes = match axum::body::to_bytes(body, BODY_NAMESPACE_READ_LIMIT).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return Err((
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(serde_json::json!({
                    "success": false,
                    "error": "request body too large to enforce namespace scoping",
                })),
            )
                .into_response())
        }
    };
    let value = if bytes.is_empty() {
        None
    } else {
        match serde_json::from_slice::<serde_json::Value>(&bytes) {
            Ok(value) => Some(value),
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": "invalid JSON body on a namespace-scoped route",
                    })),
                )
                    .into_response())
            }
        }
    };
    *req = Request::from_parts(parts, axum::body::Body::from(bytes));
    Ok(value)
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

    #[test]
    fn client_ip_ignores_xff_without_trusted_proxy() {
        // No trusted proxy configured → a forged header must be ignored and the
        // real socket address returned. This is the AUDREP-11 regression guard:
        // a direct client cannot spoof its recorded IP.
        let peer: std::net::SocketAddr = "198.51.100.5:4444".parse().unwrap();
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
        let proxy: std::net::SocketAddr = "10.0.0.5:4444".parse().unwrap();
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
        let proxy: std::net::SocketAddr = "10.0.0.5:4444".parse().unwrap();
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
        let direct: std::net::SocketAddr = "198.51.100.9:5555".parse().unwrap();
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
        let peer: std::net::SocketAddr = "198.51.100.5:4444".parse().unwrap();
        let req = Request::builder()
            .header("x-forwarded-for", "203.0.113.99")
            .extension(axum::extract::ConnectInfo(peer))
            .body(axum::body::Body::empty())
            .unwrap();
        assert_eq!(client_ip(&req, &[]), "198.51.100.5");
    }
}
