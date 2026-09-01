//! HTTP server routing, RBAC middleware, telemetry, TLS, bootstrap and tests.
//!
//! REVIEW-10: extracted from `src/cli_server.rs` (5327 lines) by splitting shared
//! types into [`super::state`]. This module owns everything that *runs*: routing,
//! auth middleware, OTEL/tracing setup, TLS handshake, the bootstrap loop, and the
//! integration tests that exercise the wired router.

// Re-export items moved to `super::state` so existing call sites inside this file
// (and its inline tests) keep their unqualified paths without an edit.
#[cfg(test)]
use super::state::ConversationTrigger;
use super::state::{
    audit_auth, extract_namespace, extract_request_id, resolve_user_key, AuthIdentity, AuthState,
    NodeDTO, QueryRequest, QueryResponse, RequestId, ServerState, AUTH_ENTITY_NS,
    LONG_REQUEST_TIMEOUT, REQUEST_TIMEOUT, SERVICE_ID_HEADER, USER_KEY_HEADER,
};

use crate::audit::AuditEvent;
#[cfg(test)]
use crate::audit::AuditLogger;
use crate::circuit_breaker::CircuitBreaker;
use crate::connection_pool::{ConnectionPool, PoolError};
use crate::entity::EntityStore;
use crate::error::ChainedError;
use crate::sdk::{
    VantaEmbedded, VantaMemoryFilter, VantaMemoryInput, VantaMemoryListOptions,
    VantaMemoryListPage, VantaMemoryRecord, VantaMemorySearchHit, VantaMemorySearchRequest,
    VantaNamespaceStatsMap, VantaOperationalMetrics,
};
use crate::VantaError;
use std::sync::Arc;
#[cfg(feature = "opentelemetry")]
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use axum::{
    extract::{DefaultBodyLimit, Path as AxumPath, Query, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tokio::net::TcpListener;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};
use tracing::Instrument;
use tracing_subscriber::EnvFilter;
#[cfg(feature = "opentelemetry")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
#[cfg(feature = "opentelemetry")]
static OTEL_PROVIDER: OnceLock<opentelemetry_sdk::trace::SdkTracerProvider> = OnceLock::new();

#[cfg(test)]
use crate::config::RbacConfig;
use crate::config::{LogFormat, VantaConfig};
use crate::console;
use crate::error::Result;
use crate::metrics;
use crate::rbac::{Permission, Rbac};
use crate::storage::StorageEngine;

/// Rate limiter period: one request every `60_000 / rpm` ms, floor 1ms.
fn rate_limit_period_ms(rpm: u32) -> u64 {
    (60_000u64 / rpm as u64).max(1)
}

/// Rate limiter burst size for the given rpm.
///
/// REST-01: without an API key the server is in local dev mode and the web
/// console fires bursts of ~12 requests (grid + inspector + sidebar) — allow
/// the full rpm as burst so the UI never 429s. With an API key configured,
/// stay conservative (`rpm/10`, the AUD-021 fail-closed posture) so a remote
/// client cannot exhaust the limiter with an instant burst.
fn rate_limit_burst(rpm: u32, auth_active: bool) -> u32 {
    if auth_active {
        (rpm / 10).max(1)
    } else {
        rpm.max(1)
    }
}

/// Build the response for a rate-limit rejection (REST-01).
///
/// Same JSON `{success:false, error}` shape as the rest of the API surface.
/// tower_governor already computes the wait time and populates `retry-after`
/// / `x-ratelimit-after` on [`GovernorError::TooManyRequests`]; those headers
/// are forwarded verbatim so clients know when to retry.
fn rate_limit_error_response(err: GovernorError) -> Response {
    let (status, headers, message) = match err {
        GovernorError::TooManyRequests { wait_time, headers } => (
            StatusCode::TOO_MANY_REQUESTS,
            headers.unwrap_or_default(),
            format!("Rate limit exceeded; retry after {wait_time}s"),
        ),
        GovernorError::UnableToExtractKey => (
            StatusCode::INTERNAL_SERVER_ERROR,
            HeaderMap::new(),
            "Unable to extract rate limit key".to_string(),
        ),
        GovernorError::Other { code, msg, headers } => (
            code,
            headers.unwrap_or_default(),
            msg.unwrap_or_else(|| "Other Error".to_string()),
        ),
    };

    let mut response = (
        status,
        Json(serde_json::json!({ "success": false, "error": message })),
    )
        .into_response();
    *response.headers_mut() = headers;
    response
}

/// Build the axum Router with public and protected routes, rate-limiting, and middleware.
///
/// No CORS is configured (see [`app_with_cors`] to allow specific origins).
pub fn app(state: Arc<ServerState>, rpm: u32) -> Router {
    app_with_cors(state, rpm, &[])
}

/// Build the axum Router as in [`app`], optionally enabling CORS for the given
/// allowed origins.
///
/// An empty `allowed_origins` slice attaches **no** CORS middleware — the
/// server sends no `Access-Control-Allow-Origin` header. Only when origins are
/// provided is a [`tower_http::cors::CorsLayer`] mounted as the outermost
/// layer (so preflight `OPTIONS` are answered before auth).
pub fn app_with_cors(state: Arc<ServerState>, rpm: u32, allowed_origins: &[String]) -> Router {
    let rbac = Arc::new(Rbac::new());
    rbac.add_role("admin", vec![Permission::Admin]);
    rbac.add_role("reader", vec![Permission::Read]);
    rbac.add_role("writer", vec![Permission::Read, Permission::Write]);
    let auth_state = AuthState::new(
        state.api_key.as_ref().map(|k| k.to_string()),
        state.alt_api_key.as_ref().map(|k| k.to_string()),
        state.rbac_config.clone(),
        rbac,
        &state.trusted_proxies,
        Some(state.storage.clone()),
        state.db.audit_logger(),
    );

    let public = Router::new().route("/health", get(health_check));

    // Interactive routes: cap at REQUEST_TIMEOUT so a stuck handler can't hold
    // the connection indefinitely (DoS protection).
    let protected = Router::new()
        .route("/api/v2/query", post(execute_query))
        .route("/api/v2/health", get(health_v2))
        .route(
            "/api/v2/records",
            post(records_put).delete(records_delete_by_filter),
        )
        .route("/api/v2/records/batch", post(records_put_batch))
        .route(
            "/api/v2/records/{ns}/{key}",
            get(records_get).delete(records_delete),
        )
        .route("/api/v2/records/{ns}/{key}/versions", get(records_versions))
        .route("/api/v2/list", get(records_list))
        .route("/api/v2/search", post(records_search))
        .route("/api/v2/autocomplete", get(iql_autocomplete))
        .route("/api/v2/audit", get(audit_events))
        .route("/api/v2/graph/bfs", post(graph_bfs))
        .route("/api/v2/graph/dfs", post(graph_dfs))
        .route("/api/v2/graph/degree", post(graph_degree))
        .route("/api/v2/graph/centrality", post(graph_centrality))
        .route("/api/v2/graph/pagerank", post(graph_pagerank))
        .route("/api/v2/graph/v2/bfs", post(graph_v2_bfs))
        .route("/api/v2/graph/v2/dfs", post(graph_v2_dfs))
        .route("/api/v2/graph/v2/degree", post(graph_v2_degree))
        .route("/api/v2/maintenance/purge", post(maintenance_purge))
        .route("/api/v2/maintenance/compact", post(maintenance_compact))
        .route("/api/v2/maintenance/flush", post(maintenance_flush))
        .route("/api/v2/threads", get(threads_list).post(threads_create))
        .route(
            "/api/v2/threads/{id}",
            get(threads_get)
                .post(threads_send_message)
                .delete(threads_delete),
        )
        .route("/conversation/add", post(conversation_add))
        .route("/skill/listing", get(skill_listing))
        .route("/api/v2/skills", post(skill_create))
        .route(
            "/api/v2/skills/{skill_id}",
            put(skill_update).patch(skill_patch).delete(skill_delete),
        )
        .route("/api/v2/snapshots", get(snapshots_list))
        .route("/api/v2/snapshots/{name}", post(snapshots_create))
        .route("/metrics", get(metrics_endpoint))
        .route("/api/v2/metrics", get(metrics_v2))
        .layer(middleware::from_fn(auth_middleware))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ));

    // Long-running bulk/maintenance routes: a generous ceiling (rather than the
    // interactive timeout above) so legitimate large imports/exports/index
    // rebuilds aren't killed mid-flight. Still bounded against a wedged op.
    let long_running = Router::new()
        .route("/api/v2/export", post(export_v2))
        .route("/api/v2/import", post(import_v2))
        .route(
            "/api/v2/maintenance/rebuild-index",
            post(maintenance_rebuild_index),
        )
        .layer(middleware::from_fn(auth_middleware))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            LONG_REQUEST_TIMEOUT,
        ));

    let protected = protected.merge(long_running);

    let protected = if rpm > 0 {
        let period_ms = rate_limit_period_ms(rpm);
        // REST-01: full burst without auth (local web console), conservative
        // burst with auth (AUD-021 fail-closed posture).
        let burst_size = rate_limit_burst(rpm, state.api_key.is_some());

        // AUD-021: fail-closed. Should the governor config ever fail to build,
        // refuse to start rather than serving requests without a rate limit.
        // (The previous fall branch left `protected` unthrottled — fail-open.)
        let gc = GovernorConfigBuilder::default()
            .per_millisecond(period_ms)
            .burst_size(burst_size)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("governor config must build: period_ms and burst_size are >= 1 for rpm > 0");
        protected.layer(GovernorLayer::new(gc).error_handler(rate_limit_error_response))
    } else {
        protected
    };

    let router = Router::new().merge(public).merge(protected);

    // CORS goes outermost so preflight OPTIONS are answered before auth.
    let router = match cors_layer(allowed_origins) {
        Some(cors) => router.layer(cors),
        None => router,
    };

    router
        .layer(DefaultBodyLimit::max(1_000_000))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            circuit_breaker_middleware,
        ))
        .layer(middleware::from_fn(request_metrics_middleware))
        .layer(Extension(auth_state))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

/// Build a [`tower_http::cors::CorsLayer`] allowing the given origins.
///
/// Returns `None` (no CORS middleware) when no valid origin is configured.
/// Invalid/blank origins are skipped and the rest kept.
fn cors_layer(allowed_origins: &[String]) -> Option<tower_http::cors::CorsLayer> {
    let origins: Vec<HeaderValue> = allowed_origins
        .iter()
        .filter_map(|origin| match HeaderValue::from_str(origin.as_str()) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!("Invalid CORS origin {:?} — ignoring: {e}", origin);
                None
            }
        })
        .collect();
    if origins.is_empty() {
        return None;
    }
    Some(
        tower_http::cors::CorsLayer::new()
            .allow_origin(origins)
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]),
    )
}

/// Resolve the auth identity from headers: L3 (user-key) wins over L2
/// (service-id); neither present → bare transport identity.
///
/// Any resolution failure yields 401 (fail closed — internal state is never
/// leaked to the caller).
pub(crate) fn resolve_identity(
    req: &axum::extract::Request,
    auth: &AuthState,
) -> std::result::Result<AuthIdentity, (StatusCode, &'static str)> {
    let user_key = req
        .headers()
        .get(USER_KEY_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if let Some(user_key) = user_key {
        let Some(storage) = &auth.storage else {
            return Err((StatusCode::UNAUTHORIZED, "invalid_user_key"));
        };
        let store = EntityStore::new(storage.as_ref());
        return match resolve_user_key(&store, AUTH_ENTITY_NS, user_key) {
            Ok(Some((user_id, is_system_admin))) => Ok(AuthIdentity::User {
                user_id,
                is_system_admin,
            }),
            Ok(None) | Err(_) => Err((StatusCode::UNAUTHORIZED, "invalid_user_key")),
        };
    }

    let service_id = req
        .headers()
        .get(SERVICE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if let Some(service_id) = service_id {
        return Ok(AuthIdentity::Service {
            service_id: service_id.to_string(),
        });
    }

    Ok(AuthIdentity::Transport)
}

/// Resolve the real client IP used for rate limiting and logging.
///
/// `X-Forwarded-For` is only honored when the request's peer is one of
/// `trusted_proxies` (i.e. it actually arrived via a configured reverse proxy
/// that sets the header). Otherwise the direct TCP socket address
/// ([`ConnectInfo`]) is returned — so a client cannot spoof its recorded IP by
/// setting `X-Forwarded-For` itself. The first valid IP in the header is used
/// when a trusted proxy is present. Returns only the IP address (without the
/// source port) in every case — port would change per connection and fragment
/// rate-limiting/audit keys.
pub fn client_ip(req: &axum::extract::Request, trusted_proxies: &[std::net::IpAddr]) -> String {
    let peer = req
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0);

    if let Some(peer) = peer {
        if trusted_proxies.contains(&peer.ip()) {
            if let Some(forwarded) = req.headers().get("x-forwarded-for") {
                if let Ok(ip_str) = forwarded.to_str() {
                    for part in ip_str.split(',') {
                        let trimmed = part.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        if let Ok(ip) = trimmed.parse::<std::net::IpAddr>() {
                            return ip.to_string();
                        }
                    }
                }
            }
        }
        return peer.ip().to_string();
    }

    "unknown".to_string()
}

/// Axum middleware that validates Bearer tokens and enforces RBAC permissions.
///
/// Returns 401 instead of panicking if `AuthState` is missing from request
/// extensions (invariant violated — e.g. router misconfigured).
pub async fn auth_middleware(mut req: axum::extract::Request, next: middleware::Next) -> Response {
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

#[tracing::instrument]
async fn health_check() -> Json<QueryResponse> {
    Json(QueryResponse {
        success: true,
        data: "OK".to_string(),
        node_id: None,
        nodes: None,
    })
}

#[tracing::instrument]
async fn metrics_endpoint() -> impl IntoResponse {
    let metrics_text = metrics::export_metrics_text();
    match Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(metrics_text)
    {
        Ok(resp) => resp.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to build metrics response: {e}"),
        )
            .into_response(),
    }
}

/// JSON wire shape for `GET /api/v2/metrics` (REST-02): the operational
/// snapshot (same `VantaOperationalMetrics` shape the desktop `vanta_metrics`
/// wrapper consumes) plus per-namespace collection counts for the
/// Índices/salud surface (FEAT-02). Both fields reuse existing SDK types.
#[derive(Serialize)]
struct MetricsV2Response {
    metrics: VantaOperationalMetrics,
    namespaces: VantaNamespaceStatsMap,
}

/// `GET /api/v2/metrics` — engine metrics as JSON for the web console.
///
/// Runs under the connection pool like every `/api/v2` console op and inherits
/// the same auth, rate-limit and CORS layers as the other protected routes.
#[tracing::instrument(skip(state))]
async fn metrics_v2(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| {
        Ok(MetricsV2Response {
            metrics: db.operational_metrics(),
            namespaces: db.namespace_stats(None)?,
        })
    })
    .await
    {
        Ok(resp) => Json(resp).into_response(),
        Err(resp) => resp,
    }
}

/// Axum middleware that records HTTP request duration and status metrics.
///
/// Also captures the caller's tracing id (SRV-02): the first match of
/// `x-request-id` / `x-tracing-id` / `traceparent` is exposed to handlers via
/// request extensions (for audit correlation) and recorded on the request span.
pub async fn request_metrics_middleware(
    mut req: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let start = std::time::Instant::now();
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
    State(state): State<Arc<ServerState>>,
    req: axum::extract::Request,
    next: middleware::Next,
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

/// Build a generic 500 for a panicked execution task.
///
/// The panic detail is logged server-side; clients only get a generic message
/// to avoid leaking internal runtime details (AUDREP-32).
fn panic_error_response(panic_detail: &dyn std::fmt::Display) -> Response {
    tracing::error!("execution task panicked: {}", panic_detail);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(QueryResponse {
            success: false,
            data: "Internal server error".to_string(),
            node_id: None,
            nodes: None,
        }),
    )
        .into_response()
}

/// Map a `VantaError` to the HTTP status clients receive (ERR-027).
///
/// Client mistakes (bad IQL, missing nodes, validation) map to explicit 4xx
/// statuses; anything server-side stays a 500. Shared by the IQL endpoint and
/// the `/api/v2` console surface so both speak the same error status language.
fn vanta_error_status(e: &VantaError) -> StatusCode {
    match e {
        VantaError::IqlParseError { .. }
        | VantaError::IqlError(_)
        | VantaError::InvalidInput(_)
        | VantaError::DimensionMismatch { .. }
        | VantaError::UnsupportedOperation { .. }
        | VantaError::SchemaError(_)
        | VantaError::NoVectorForKey(_) => StatusCode::BAD_REQUEST,
        VantaError::ValidationError { .. } => StatusCode::UNPROCESSABLE_ENTITY,
        VantaError::NodeNotFound(_) | VantaError::NotFound { .. } => StatusCode::NOT_FOUND,
        VantaError::DuplicateNode(_)
        | VantaError::NodeIdCollision(_)
        | VantaError::ExecutionConflict { .. } => StatusCode::CONFLICT,
        // Storage/WAL/IO/resource failures and anything unclassified.
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Build a 4xx/5xx response for a query execution error (ERR-027).
///
/// Client mistakes (bad IQL, missing nodes, validation) map to explicit 4xx
/// statuses; anything server-side stays a 500. Proxies and monitoring can then
/// distinguish query errors from healthy traffic instead of relying on the
/// body's `success` flag.
fn query_error_response(e: &VantaError) -> Response {
    (
        vanta_error_status(e),
        Json(QueryResponse {
            success: false,
            data: format!("Execution Error: {}", e),
            node_id: None,
            nodes: None,
        }),
    )
        .into_response()
}

#[tracing::instrument(skip(state))]
async fn execute_query(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<QueryRequest>,
) -> Response {
    use crate::executor::{ExecutionResult, Executor};

    let _permit = match state.pool.acquire().await {
        Ok(p) => p,
        Err(e) => {
            let msg = match e {
                PoolError::Closed => "Server query pool closed".to_string(),
                PoolError::Timeout => "Server concurrency limit reached; retry shortly".to_string(),
            };
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                [(header::RETRY_AFTER, "1")],
                Json(QueryResponse {
                    success: false,
                    data: msg,
                    node_id: None,
                    nodes: None,
                }),
            )
                .into_response();
        }
    };

    let storage = state.storage.clone();
    let query = payload.query.clone();

    let start = Instant::now();
    let join_res = tokio::task::spawn_blocking(move || {
        let executor = Executor::new(&storage);
        executor.execute_hybrid(&query)
    })
    .await;
    // FND-07: feed the canonical query latency histogram (vanta_query_latency_ms)
    // with real server-side execution time — no-op without the prometheus feature.
    metrics::record_query_latency(start.elapsed().as_millis() as u64);

    let execution_result = match join_res {
        Ok(r) => r,
        Err(e) => return panic_error_response(&e),
    };

    match execution_result {
        Ok(ExecutionResult::Read(nodes)) => {
            let dtos: Vec<NodeDTO> = nodes.iter().map(NodeDTO::from).collect();
            Json(QueryResponse {
                success: true,
                data: format!("Read {} nodes.", nodes.len()),
                node_id: None,
                nodes: Some(dtos),
            })
            .into_response()
        }
        Ok(ExecutionResult::Write {
            affected_nodes,
            message,
            node_id,
        }) => Json(QueryResponse {
            success: true,
            data: format!("Mutated {} nodes: {}", affected_nodes, message),
            node_id,
            nodes: None,
        })
        .into_response(),
        Ok(ExecutionResult::StaleContext(summary_id)) => Json(QueryResponse {
            success: true,
            data: format!(
                "STALE_CONTEXT: Confidence Score critical. Rehydration available for summary {}",
                summary_id
            ),
            node_id: Some(summary_id),
            nodes: None,
        })
        .into_response(),
        Err(e) => query_error_response(&e),
    }
}

// ─── /api/v2 console surface (WEB-01) ───────────────────────────────────────
//
// Endpoints map 1:1 to the embedded SDK (`VantaEmbedded`) so the wire format
// is the SDK's own serde. Errors are `{success: false, error}` with the status
// from `vanta_error_status` — the same shape the auth middleware and circuit
// breaker already emit. All engine work runs under a pool permit in
// `spawn_blocking` (never on the Tokio runtime, R-2 server-mcp).

/// Error body shared by the `/api/v2` console endpoints.
fn vanta_error_response(e: &VantaError) -> Response {
    (
        vanta_error_status(e),
        Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
        })),
    )
        .into_response()
}

/// Map a connection-pool acquisition failure to a 503 (mirrors `execute_query`).
fn pool_error_response(e: PoolError) -> Response {
    let msg = match e {
        PoolError::Closed => "Server query pool closed".to_string(),
        PoolError::Timeout => "Server concurrency limit reached; retry shortly".to_string(),
    };
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [(header::RETRY_AFTER, "1")],
        Json(serde_json::json!({ "success": false, "error": msg })),
    )
        .into_response()
}

/// Run a blocking SDK operation under a connection-pool permit.
///
/// Pool, panic, and `VantaError` failures become HTTP responses; success
/// returns the raw SDK value for the handler to serialize.
async fn run_db_op<T>(
    state: &ServerState,
    op: impl FnOnce(&VantaEmbedded) -> Result<T> + Send + 'static,
) -> std::result::Result<T, Response>
where
    T: Send + 'static,
{
    let _permit = state.pool.acquire().await.map_err(pool_error_response)?;
    let db = state.db.clone();
    match tokio::task::spawn_blocking(move || op(&db)).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(vanta_error_response(&e)),
        Err(e) => Err(panic_error_response(&e)),
    }
}

/// Health report shape for `GET /api/v2/health` (mirrors the desktop
/// `HealthReport` wire contract).
#[derive(Serialize)]
struct HealthReportV2 {
    status: &'static str,
    backend: String,
    latency_ms: u64,
    checked_at_ms: u64,
    message: Option<String>,
}

/// Human label for the configured storage backend.
fn backend_label(kind: &crate::backend::BackendKind) -> &'static str {
    match kind {
        crate::backend::BackendKind::Fjall => "fjall",
        crate::backend::BackendKind::RocksDb => "rocksdb",
        crate::backend::BackendKind::InMemory => "in-memory",
    }
}

#[tracing::instrument(skip(state))]
async fn health_v2(State(state): State<Arc<ServerState>>) -> Response {
    let start = Instant::now();
    let db = state.db.clone();
    let result = tokio::task::spawn_blocking(move || db.list_namespaces()).await;
    let latency_ms = start.elapsed().as_millis() as u64;
    let checked_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let (status, message) = match result {
        Ok(Ok(_)) => ("healthy", None),
        Ok(Err(e)) => ("degraded", Some(e.to_string())),
        Err(e) => ("degraded", Some(format!("execution task panicked: {e}"))),
    };
    Json(HealthReportV2 {
        status,
        backend: backend_label(&state.db.config.backend_kind).to_string(),
        latency_ms,
        checked_at_ms,
        message,
    })
    .into_response()
}

#[tracing::instrument(skip(state))]
async fn records_put(
    State(state): State<Arc<ServerState>>,
    Json(input): Json<VantaMemoryInput>,
) -> Response {
    match run_db_op(&state, move |db| db.put(input)).await {
        Ok(record) => (StatusCode::CREATED, Json(record)).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn records_put_batch(
    State(state): State<Arc<ServerState>>,
    Json(inputs): Json<Vec<VantaMemoryInput>>,
) -> Response {
    match run_db_op(&state, move |db| db.put_batch(inputs)).await {
        Ok(records) => (StatusCode::CREATED, Json(records)).into_response(),
        Err(resp) => resp,
    }
}

/// 404 body for a missing record lookup (REST convention: GET/DELETE of a
/// nonexistent key is a client mistake, not a server fault).
fn not_found_response(key: &str) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "success": false,
            "error": format!("record not found: {key}"),
        })),
    )
        .into_response()
}

#[tracing::instrument(skip(state))]
async fn records_get(
    State(state): State<Arc<ServerState>>,
    AxumPath((ns, key)): AxumPath<(String, String)>,
) -> Response {
    let key_label = key.clone();
    match run_db_op(&state, move |db| db.get(&ns, &key)).await {
        Ok(Some(record)) => Json(record).into_response(),
        Ok(None) => not_found_response(&key_label),
        Err(resp) => resp,
    }
}

/// Query params for `GET /api/v2/records/{ns}/{key}/versions`.
#[derive(Deserialize, Debug)]
struct RecordsVersionsParams {
    /// When present, returns only that version instead of the full list.
    version: Option<u64>,
}

#[tracing::instrument(skip(state))]
async fn records_versions(
    State(state): State<Arc<ServerState>>,
    AxumPath((ns, key)): AxumPath<(String, String)>,
    Query(params): Query<RecordsVersionsParams>,
) -> Response {
    match params.version {
        Some(version) => {
            let key_label = key.clone();
            match run_db_op(&state, move |db| db.get_version(&ns, &key, version)).await {
                Ok(Some(record)) => Json(record).into_response(),
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("version {version} not found for key {key_label}"),
                    })),
                )
                    .into_response(),
                Err(resp) => resp,
            }
        }
        None => match run_db_op(&state, move |db| db.versions(&ns, &key)).await {
            Ok(records) => Json(records).into_response(),
            Err(resp) => resp,
        },
    }
}

#[tracing::instrument(skip(state))]
async fn records_delete(
    State(state): State<Arc<ServerState>>,
    AxumPath((ns, key)): AxumPath<(String, String)>,
) -> Response {
    let key_label = key.clone();
    match run_db_op(&state, move |db| db.delete(&ns, &key)).await {
        Ok(true) => Json(serde_json::json!({ "deleted": true })).into_response(),
        Ok(false) => not_found_response(&key_label),
        Err(resp) => resp,
    }
}

/// Query params for `DELETE /api/v2/records?namespace=&filter=`.
#[derive(Deserialize, Debug)]
struct DeleteByFilterParams {
    namespace: String,
    /// JSON array of `VantaMemoryFilterItem` (e.g.
    /// `[{"field":"kind","op":"Eq","value":{"String":"note"}}]`).
    filter: String,
}

#[tracing::instrument(skip(state))]
async fn records_delete_by_filter(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<DeleteByFilterParams>,
) -> Response {
    let filter: VantaMemoryFilter = match serde_json::from_str(&params.filter) {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("invalid filter JSON: {e}"),
                })),
            )
                .into_response();
        }
    };
    let ns = params.namespace;
    match run_db_op(&state, move |db| db.delete_by_filter(&ns, filter)).await {
        Ok(deleted) => Json(serde_json::json!({ "deleted": deleted })).into_response(),
        Err(resp) => resp,
    }
}

/// AUD-046: merge per-namespace pages (stable namespace-name order) for the
/// `/api/v2/list` all-namespaces fan-out. A namespace whose page still has a
/// `next_cursor` was capped at `NS_CAP` mid-listing — it is reported in the
/// returned `truncated_namespaces` so the client never sees silent truncation.
fn merge_all_namespaces_pages(
    pages: Vec<(String, VantaMemoryListPage)>,
) -> (Vec<VantaMemoryRecord>, Vec<String>) {
    let mut records = Vec::new();
    let mut truncated_namespaces = Vec::new();
    for (ns, page) in pages {
        if page.next_cursor.is_some() {
            truncated_namespaces.push(ns);
        }
        records.extend(page.records);
    }
    (records, truncated_namespaces)
}

/// Query params for `GET /api/v2/list`.
#[derive(Deserialize, Debug)]
struct ListParams {
    // Option: la consola web lista sin namespace → default a "default" (igual
    // que el bridge nativo). Un campo String requerido 400ea en axum antes del handler.
    namespace: Option<String>,
    limit: Option<usize>,
    cursor: Option<usize>,
    /// JSON array of `VantaMemoryFilterItem`.
    filter_ops: Option<String>,
}

#[tracing::instrument(skip(state))]
async fn records_list(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<ListParams>,
) -> Response {
    // Sin namespace → agregar TODOS los namespaces (orden estable: nombre asc).
    // Antes defaulteaba a "default" y el grid/paleta de la consola mostraba
    // "Sin registros" con datos presentes en otros namespaces.
    let all_namespaces = params
        .namespace
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .is_none();
    let filter_ops = match params.filter_ops.as_deref() {
        None => None,
        Some(raw) => match serde_json::from_str::<VantaMemoryFilter>(raw) {
            Ok(f) => Some(f),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("invalid filter_ops JSON: {e}"),
                    })),
                )
                    .into_response();
            }
        },
    };
    let limit = params.limit.unwrap_or(100);
    let cursor = params.cursor;
    if all_namespaces {
        // FIND-24: fan-out by namespace now respects the client's `limit`
        // (instead of NS_CAP) — list(limit=100) used to materialize
        // NS_CAP=10_000 records per namespace and slice in memory, blowing
        // past REQUEST_TIMEOUT=30s for ≥10k records total. The SDK's
        // `indexed_ids_by_namespace` early-exits at `limit`, so a per-ns
        // `limit`-sized scan is O(limit) per namespace. Cross-namespace
        // pagination walks namespaces in stable name order; the returned
        // `next_cursor` is the cumulative offset within the merged window
        // and remains backward-compatible with single-namespace clients.
        //
        // NS_CAP (10_000) was the previous fan-out ceiling per ns and has been
        // removed: the SDK now enforces the `limit` early-exit natively, so
        // there is no in-memory NS_CAP cost. Truncation at the namespace
        // boundary is still detected via `next_cursor` from each per-ns
        // `VantaMemoryListPage` (see `merge_all_namespaces_pages`).

        /// Fan-out response: same shape as `VantaMemoryListPage` plus an
        /// additive signal listing namespaces whose listing is still paginating
        /// (they may hold more records than this response contains).
        #[derive(Serialize)]
        struct AllNamespacesListPage {
            records: Vec<VantaMemoryRecord>,
            next_cursor: Option<usize>,
            /// Namespaces still paginating during the fan-out (their
            /// per-ns `VantaMemoryListPage.next_cursor` was `Some`).
            truncated_namespaces: Vec<String>,
        }

        let options_for = move |_ns: String| VantaMemoryListOptions {
            filter_ops: filter_ops.clone(),
            limit,
            cursor,
            ..Default::default()
        };
        return match run_db_op(&state, move |db| {
            let mut names: Vec<String> = db.namespace_stats(None)?.keys().cloned().collect();
            names.sort();
            let mut pages = Vec::new();
            for ns in names {
                let page = db.list(&ns, options_for(ns.clone()))?;
                pages.push((ns, page));
            }
            let (records, truncated_namespaces) = merge_all_namespaces_pages(pages);
            let start = cursor.unwrap_or(0).min(records.len());
            let end = (start + limit).min(records.len());
            let window = records[start..end].to_vec();
            let next_cursor = (end < records.len()).then_some(end);
            Ok::<_, VantaError>(AllNamespacesListPage {
                records: window,
                next_cursor,
                truncated_namespaces,
            })
        })
        .await
        {
            Ok(page) => Json(page).into_response(),
            Err(resp) => resp,
        };
    }
    let ns = params.namespace.unwrap_or_default();
    let options = VantaMemoryListOptions {
        filter_ops,
        limit,
        cursor,
        ..Default::default()
    };
    match run_db_op(&state, move |db| db.list(&ns, options)).await {
        Ok(page) => Json(page).into_response(),
        Err(resp) => resp,
    }
}

/// JSON body for `POST /api/v2/search`: the SDK search request plus optional
/// offset pagination (REST-04). `cursor`/`limit` are server-only — the core
/// `search()` is a top_k window without its own cursor, so the wire pages by
/// offset over the same score-ranked result set.
#[derive(Debug, Deserialize)]
struct SearchPageRequest {
    #[serde(flatten)]
    request: VantaMemorySearchRequest,
    /// Zero-based offset into the ranked result set.
    #[serde(default)]
    cursor: Option<usize>,
    /// Page size; defaults to `top_k`.
    #[serde(default)]
    limit: Option<usize>,
}

/// Page-shaped search response mirroring `VantaMemoryListPage` so the web
/// console paginates search the same way it paginates list (REST-04).
#[derive(Serialize)]
struct SearchPageV2 {
    records: Vec<VantaMemorySearchHit>,
    next_cursor: Option<usize>,
}

#[tracing::instrument(skip(state))]
async fn records_search(
    State(state): State<Arc<ServerState>>,
    Json(page_request): Json<SearchPageRequest>,
) -> Response {
    // La Topbar de la consola web busca sin namespace → search_all (todos los
    // namespaces, merge por score). Antes defaulteaba a "default" y la búsqueda
    // global ignoraba silenciosamente todo lo ingerido en otros namespaces.
    let mut request = page_request.request;
    let all_namespaces = request.namespace.trim().is_empty();
    // Paginación offset (REST-04): el core `search()` es una ventana top_k sin
    // cursor propio, así que el server traduce cursor/limit → top_k+1 (un extra
    // para saber si hay más página) y recorta. Los resultados se ordenan por
    // score, así que offset sobre el mismo ranking es estable entre páginas.
    let page_size = page_request.limit.unwrap_or(request.top_k.max(1));
    let cursor = page_request.cursor.unwrap_or(0);
    request.top_k = cursor.saturating_add(page_size).saturating_add(1);
    match run_db_op(&state, move |db| {
        if all_namespaces {
            db.search_all(request)
        } else {
            db.search(request)
        }
    })
    .await
    {
        Ok(hits) => {
            let start = cursor.min(hits.len());
            let end = (start + page_size).min(hits.len());
            let records = hits[start..end].to_vec();
            let next_cursor = (end < hits.len()).then_some(end);
            Json(SearchPageV2 {
                records,
                next_cursor,
            })
            .into_response()
        }
        Err(resp) => resp,
    }
}

/// Query params for `GET /api/v2/autocomplete`.
#[derive(Deserialize, Debug)]
struct AutocompleteParams {
    prefix: Option<String>,
}

#[tracing::instrument]
async fn iql_autocomplete(Query(params): Query<AutocompleteParams>) -> Json<Vec<String>> {
    let prefix = params.prefix.unwrap_or_default();
    Json(crate::parser::autocomplete_prefix(&prefix))
}

/// Query params for `GET /api/v2/audit`.
#[derive(Deserialize, Debug)]
struct AuditParams {
    namespace: Option<String>,
    op: Option<String>,
    outcome: Option<String>,
    limit: Option<usize>,
    cursor: Option<usize>,
}

/// Default page size when the caller omits `limit` (mirrors the desktop).
const AUDIT_DEFAULT_LIMIT: usize = 100;

/// A page of audit events ordered newest-first (mirrors the desktop `AuditPage`).
#[derive(Serialize)]
struct AuditPageV2 {
    events: Vec<AuditEvent>,
    next_cursor: Option<usize>,
}

/// Resolve the audit log path from the embedded config.
///
/// `None` means audit is not configured — the endpoint reports 404 rather than
/// inventing a path that would never be written (mirrors the desktop, which
/// errors with "audit log no configurado").
fn audit_log_path(state: &ServerState) -> Option<std::path::PathBuf> {
    state.db.config.audit_log_path.clone()
}

/// Read the audit JSONL at `path`, apply filters, and paginate newest-first.
///
/// `cursor` is a zero-based offset into the *filtered* newest-first list;
/// `next_cursor` is `Some(end)` when older events remain, `None` otherwise.
///
/// ponytail: whole-file read (fine for console-sized audit logs); a byte-offset
/// tail read is the upgrade if the log grows large.
fn read_audit_page(
    path: &std::path::Path,
    namespace: Option<&str>,
    op: Option<&str>,
    outcome: Option<&str>,
    limit: usize,
    cursor: Option<usize>,
) -> std::io::Result<AuditPageV2> {
    let content = std::fs::read_to_string(path)?;
    let mut matched: Vec<AuditEvent> = content
        .lines()
        .filter_map(|line| serde_json::from_str::<AuditEvent>(line).ok())
        .filter(|e| namespace.is_none_or(|n| e.namespace == n))
        .filter(|e| op.is_none_or(|o| e.op == o))
        .filter(|e| outcome.is_none_or(|o| e.outcome == o))
        .collect();
    matched.reverse();
    let start = cursor.unwrap_or(0).min(matched.len());
    let end = (start + limit).min(matched.len());
    let events = matched[start..end].to_vec();
    let next_cursor = (end < matched.len()).then_some(end);
    Ok(AuditPageV2 {
        events,
        next_cursor,
    })
}

#[tracing::instrument(skip(state))]
async fn audit_events(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<AuditParams>,
) -> Response {
    let Some(path) = audit_log_path(&state) else {
        return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "success": false,
                "error": "audit log no configurado",
            })),
        )
            .into_response();
    };
    let namespace = params.namespace;
    let op = params.op;
    let outcome = params.outcome;
    let limit = params.limit.unwrap_or(AUDIT_DEFAULT_LIMIT);
    let cursor = params.cursor;

    let join = tokio::task::spawn_blocking(move || {
        read_audit_page(
            &path,
            namespace.as_deref(),
            op.as_deref(),
            outcome.as_deref(),
            limit,
            cursor,
        )
    })
    .await;

    match join {
        Ok(Ok(page)) => Json(page).into_response(),
        Ok(Err(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "success": false,
                "error": format!("failed to read audit log: {e}"),
            })),
        )
            .into_response(),
        Err(e) => panic_error_response(&e),
    }
}

/// Initialise the tracing subscriber with optional OpenTelemetry and MCP support.
pub fn init_telemetry(is_mcp: bool, log_format: Option<LogFormat>) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let format = resolve_log_format(log_format);
    let is_json = matches!(format, LogFormat::Json);
    let is_full = matches!(format, LogFormat::Full);

    #[cfg(feature = "opentelemetry")]
    _init_telemetry_otel(is_mcp, is_json, is_full, env_filter);

    #[cfg(not(feature = "opentelemetry"))]
    init_telemetry_fmt(is_mcp, is_json, is_full, env_filter);
}

fn resolve_log_format(log_format: Option<LogFormat>) -> LogFormat {
    log_format.unwrap_or_else(|| {
        let legacy = std::env::var("VANTADB_LOG_JSON")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);
        if legacy {
            LogFormat::Json
        } else {
            std::env::var("VANTADB_LOG_FORMAT")
                .ok()
                .map(|v| LogFormat::from_env_value(&v))
                .unwrap_or_default()
        }
    })
}

#[cfg(not(feature = "opentelemetry"))]
fn init_telemetry_fmt(is_mcp: bool, is_json: bool, is_full: bool, env_filter: EnvFilter) {
    let stderr = || Box::new(std::io::stderr()) as Box<dyn std::io::Write + Send>;

    if is_json {
        let sub = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(false);
        if is_mcp {
            sub.with_writer(stderr).init();
        } else {
            sub.init();
        }
    } else if is_full {
        let sub = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .with_ansi(true);
        if is_mcp {
            sub.with_writer(stderr).init();
        } else {
            sub.init();
        }
    } else if is_mcp {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(stderr)
            .init();
    } else {
        crate::console::init_logging(LogFormat::Compact);
    }
}

#[cfg(feature = "opentelemetry")]
fn _init_telemetry_otel(is_mcp: bool, is_json: bool, is_full: bool, env_filter: EnvFilter) {
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_otlp::WithExportConfig;

    let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let exporter = match opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint.clone())
        .build()
    {
        Ok(exporter) => exporter,
        Err(e) => {
            eprintln!(
                "⚠️ Failed to create OTLP exporter (endpoint: {}), continuing without tracing: {e}",
                endpoint
            );
            return;
        }
    };

    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "vantadb-server".to_string());

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            opentelemetry_sdk::Resource::builder_empty()
                .with_service_name(service_name.clone())
                .build(),
        )
        .build();

    let _ = OTEL_PROVIDER.set(provider.clone());
    let tracer = provider.tracer(service_name.clone());
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    let subscriber = Registry::default().with(env_filter).with(telemetry);

    if is_mcp {
        subscriber
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
            .init();
    } else if is_json {
        subscriber
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    } else if is_full {
        subscriber
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    } else {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
    }
}

/// Shut down the OpenTelemetry tracer provider, flushing any pending spans.
#[cfg(feature = "opentelemetry")]
pub fn shutdown_telemetry() {
    if let Some(provider) = OTEL_PROVIDER.get() {
        if let Err(e) = provider.shutdown() {
            eprintln!("OTel provider shutdown error: {e}");
        }
    }
}

fn log_security_mode(config: &VantaConfig) {
    let auth_status = match (&config.api_key, config.require_auth) {
        (Some(_), true) => "Bearer token auth ✓ (forced)",
        (Some(_), false) => "Bearer token auth ✓",
        (None, true) => "ERROR: require_auth but no key configured",
        (None, false) => "No auth (dev mode)",
    };

    let rate_status = if config.rate_limit_rpm == 0 {
        "Rate limit disabled".to_string()
    } else {
        format!("Rate limit {} req/min", config.rate_limit_rpm)
    };

    let tls_status = {
        #[cfg(feature = "tls")]
        {
            if config.tls_cert_path.is_some() && config.tls_key_path.is_some() {
                "TLS ✓ (rustls)"
            } else {
                "TLS feature active but no cert/key configured — falling back to plain HTTP"
            }
        }
        #[cfg(not(feature = "tls"))]
        "Plain HTTP"
    };

    console::ok(
        "Security",
        Some(&format!(
            "{} | {} | {}",
            auth_status, rate_status, tls_status
        )),
    );
}

/// Whether `host` binds only the loopback interface (`127.0.0.0/8`,
/// `::1`, or the literal name `localhost`). Unresolvable hostnames are
/// treated as non-loopback (fail closed).
fn is_loopback_host(host: &str) -> bool {
    let h = host.trim();
    let h = h.strip_prefix('[').unwrap_or(h);
    let h = h.strip_suffix(']').unwrap_or(h);
    h.eq_ignore_ascii_case("localhost")
        || h.parse::<std::net::IpAddr>()
            .map(|ip| ip.is_loopback())
            .unwrap_or(false)
}

/// Validate that the auth configuration is consistent.
///
/// Refuse-to-start policy (FIND-07): the server does NOT start when it binds a
/// non-loopback host without an API key — an unauthenticated instance exposed
/// to the network is an accident waiting to happen. Override explicitly with
/// `--allow-insecure` (dev only), which logs a prominent WARNING instead.
/// Also returns an error if `require_auth` is set but no key is configured.
/// SRV-04: `alt_api_key` requires `api_key` to be set (rotation needs a primary).
fn validate_auth_config(config: &VantaConfig) -> Result<()> {
    if config.alt_api_key.is_some() && config.api_key.is_none() {
        return Err(VantaError::InvalidInput(
            "alt_api_key requires api_key to be set (rotation needs a primary key)".into(),
        ));
    }
    if config.require_auth && config.api_key.is_none() {
        console::error(
            "Forced authentication enabled but no API key configured",
            Some(
                "Set the VANTADB_API_KEY environment variable to provide an authentication \
                 token. Alternatively, unset VANTADB_REQUIRE_AUTH / remove --require-auth \
                 to allow unauthenticated (dev) mode.",
            ),
        );
        return Err(VantaError::InvalidInput(
            "require_auth is set but no api_key is configured".into(),
        ));
    }
    if config.api_key.is_none() && !is_loopback_host(&config.host) {
        if config.allow_insecure {
            console::warn(
                "INSECURE MODE: HTTP server exposed on non-loopback host WITHOUT authentication",
                Some(&format!(
                    "host '{}' accepts unauthenticated requests from any reachable client. \
                     Set VANTADB_API_KEY (or remove --allow-insecure) to secure this server.",
                    config.host
                )),
            );
        } else {
            console::error(
                "Refusing to start: non-loopback host without an API key",
                Some(&format!(
                    "Binding '{}' without VANTADB_API_KEY exposes an unauthenticated \
                     server to the network. Fix either way: (1) set VANTADB_API_KEY to \
                     enable Bearer auth, or (2) bind a loopback host (127.0.0.1/localhost/::1), \
                     or (3) pass --allow-insecure to override this check in dev.",
                    config.host
                )),
            );
            return Err(VantaError::InvalidInput(format!(
                "non-loopback host '{}' without api_key; set VANTADB_API_KEY, bind a \
                 loopback host, or pass --allow-insecure",
                config.host
            )));
        }
    }
    Ok(())
}

/// Mount the Vanta Studio dashboard at `/dashboard`.
///
/// With `dir`, serves static files via [`tower_http::services::ServeDir`] with
/// an SPA fallback: routes **without** a file extension (deep links) get
/// `index.html`, while real asset misses still 404. Without `dir`, `/dashboard`
/// returns a 404 with a hint telling the user to pass `--dashboard-dir` (WEB-03).
///
/// Mounted here (after `app_with_cors`) on purpose: it stays **outside** the
/// auth middleware, so `/dashboard` is public on loopback (D12) even when
/// `require_auth` guards `/api/v2/*`.
fn mount_dashboard(router: Router, dir: Option<&std::path::Path>) -> Router {
    match dir {
        Some(dir) => {
            let index = dir.join("index.html");
            let fallback = tower::service_fn(move |req: axum::http::Request<axum::body::Body>| {
                let index = index.clone();
                async move {
                    // Asset miss (path has an extension) is a real 404 — never
                    // swallow it with index.html (SPA fallback is for deep links).
                    if std::path::Path::new(req.uri().path()).extension().is_some() {
                        return Ok::<_, std::convert::Infallible>(
                            (StatusCode::NOT_FOUND, "Not found").into_response(),
                        );
                    }
                    let body = match tokio::fs::read(&index).await {
                        Ok(bytes) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], bytes)
                            .into_response(),
                        Err(_) => (
                            StatusCode::NOT_FOUND,
                            format!("index.html not found in dashboard dir: {}", index.display()),
                        )
                            .into_response(),
                    };
                    Ok::<_, std::convert::Infallible>(body)
                }
            });
            router.nest_service(
                "/dashboard",
                tower_http::services::ServeDir::new(dir).fallback(fallback),
            )
        }
        None => router
            .route("/dashboard", get(dashboard_disabled))
            .route("/dashboard/{*path}", get(dashboard_disabled)),
    }
}

/// 404 hint returned when no `--dashboard-dir` is configured (WEB-03).
async fn dashboard_disabled() -> Response {
    (
        StatusCode::NOT_FOUND,
        "Dashboard not enabled. Start the server with --dashboard-dir <path> to serve the Vanta Studio console at /dashboard.",
    )
        .into_response()
}

/// Start the HTTP (or TLS) server, binding to the address in the config.
pub async fn run(config: VantaConfig) -> Result<()> {
    init_telemetry(false, Some(config.log_format));

    console::print_banner();

    validate_auth_config(&config)?;

    console::progress("Initializing storage engine...", None);

    let storage = match StorageEngine::open_with_config(&config.storage_path, Some(config.clone()))
    {
        Ok(s) => {
            console::ok("Storage engine opened", Some(&config.storage_path));
            Arc::new(s)
        }
        Err(e) => {
            console::error("Failed to open storage engine", Some(&e.to_string()));
            return Err(e);
        }
    };

    log_security_mode(&config);

    let api_key: Option<Arc<str>> = config.api_key.as_deref().map(Arc::from);
    let alt_api_key: Option<Arc<str>> = config.alt_api_key.as_deref().map(Arc::from);
    let circuit_breaker = Arc::new(CircuitBreaker::new(
        config.circuit_breaker_failure_threshold,
        Duration::from_secs(config.circuit_breaker_open_timeout_secs),
    ));
    let pool = Arc::new(ConnectionPool::new(
        config.max_connections,
        Duration::from_millis(config.pool_acquire_timeout_ms),
    ));
    let rbac_config = config.rbac_config.clone();
    let state = Arc::new(ServerState {
        storage: storage.clone(),
        db: VantaEmbedded::from_engine(storage.clone()),
        circuit_breaker,
        pool,
        api_key,
        alt_api_key,
        rbac_config,
        trusted_proxies: config.trusted_proxies.clone(),
        conversation_trigger: None,
    });

    // MOD-12 (MCP-01 twin): a raw StorageEngine skips the
    // `VantaEmbedded::open_with_config` index reconciliation, so lexical/hybrid
    // searches fail on fresh DBs with "text_index not found". Ensure index
    // state at startup: idempotent — no-op when counts match, writes fresh
    // empty state for new DBs. Read-only engines cannot rebuild, so they are
    // skipped (same guard as `open_with_config`).
    if !config.read_only {
        if let Err(e) = state.db.ensure_indexes_current() {
            console::error(
                "Failed to ensure index state at startup; text search may be unavailable",
                Some(&e.to_string()),
            );
        }
    }

    let rpm = config.rate_limit_rpm;
    let router = app_with_cors(state, rpm, &config.allowed_origins);
    let router = mount_dashboard(router, config.dashboard_dir.as_deref());
    let addr = format!("{}:{}", config.host, config.port);

    if !serve_http_or_tls(router, addr, &config, storage.clone()).await {
        return Err(VantaError::CliError(ChainedError::msg(
            "Server exited with errors",
        )));
    }

    Ok(())
}

/// Wait for SIGINT (or SIGTERM on Unix) to trigger graceful shutdown.
pub async fn wait_for_shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();
    #[cfg(unix)]
    let mut sigterm = match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
    {
        Ok(s) => s,
        Err(e) => {
            console::error("Failed to install SIGTERM handler", Some(&e.to_string()));
            return;
        }
    };

    #[cfg(unix)]
    tokio::select! {
        _ = ctrl_c => {},
        _ = sigterm.recv() => {},
    }
    #[cfg(not(unix))]
    let _ = ctrl_c.await;
}

/// Build a rustls TLS 1.3 server config from PEM certificate and key files.
#[cfg(feature = "tls")]
pub async fn build_tls13_config(
    cert_path: &str,
    key_path: &str,
) -> std::io::Result<rustls::ServerConfig> {
    use rustls::pki_types::pem::PemObject;
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};

    let cert_bytes = tokio::fs::read(cert_path).await?;
    let key_bytes = tokio::fs::read(key_path).await?;

    let certs: Vec<CertificateDer> = CertificateDer::pem_slice_iter(&cert_bytes)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut keys: Vec<PrivateKeyDer> = PrivateKeyDer::pem_slice_iter(&key_bytes)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    if keys.len() != 1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected exactly one private key in PEM file",
        ));
    }

    let key = keys.pop().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected exactly one private key",
        )
    })?;

    // Include TLSv1.2 alongside TLSv1.3 for compatibility with legacy HTTP
    // clients (e.g. older curl, Java 8, Python <3.7) that do not support
    // TLSv1.3 exclusively.
    let mut config = rustls::ServerConfig::builder_with_protocol_versions(&[
        &rustls::version::TLS12,
        &rustls::version::TLS13,
    ])
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];
    Ok(config)
}

/// Flush storage and log the result using spawn_blocking to avoid blocking Tokio.
async fn flush_on_shutdown_async(storage: Arc<crate::storage::StorageEngine>) {
    console::warn("Flushing storage before exit...", None);
    let flush_res = tokio::task::spawn_blocking(move || storage.flush()).await;

    match flush_res {
        Ok(Err(e)) => console::error("Flush failed during shutdown", Some(&e.to_string())),
        Ok(Ok(())) => console::ok("Storage flushed", None),
        Err(e) => console::error("Flush task panicked during shutdown", Some(&e.to_string())),
    }
    #[cfg(feature = "opentelemetry")]
    shutdown_telemetry();
}

/// Returns `true` if the server completed a graceful shutdown (flush was called).
#[cfg_attr(not(feature = "tls"), allow(unused_variables))]
async fn serve_http_or_tls(
    router: axum::Router,
    addr: String,
    config: &VantaConfig,
    storage: Arc<crate::storage::StorageEngine>,
) -> bool {
    #[cfg(feature = "tls")]
    if let (Some(cert), Some(key)) = (&config.tls_cert_path, &config.tls_key_path) {
        let tls_config = match build_tls13_config(cert, key).await {
            Ok(c) => axum_server::tls_rustls::RustlsConfig::from_config(Arc::new(c)),
            Err(e) => {
                console::error("Failed to load TLS certificate/key", Some(&e.to_string()));
                flush_on_shutdown_async(storage.clone()).await;
                return false;
            }
        };

        let socket_addr: std::net::SocketAddr = match addr.parse() {
            Ok(a) => a,
            Err(e) => {
                console::error("Invalid bind address", Some(&e.to_string()));
                flush_on_shutdown_async(storage.clone()).await;
                return false;
            }
        };

        console::print_ready(&format!("https://{}", addr));

        let handle = axum_server::Handle::new();
        let handle_clone = handle.clone();
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            wait_for_shutdown_signal().await;
            console::warn("Shutting down TLS server gracefully...", None);
            flush_on_shutdown_async(storage_clone).await;
            handle_clone.graceful_shutdown(Some(Duration::from_secs(10)));
        });

        if let Err(e) = axum_server::bind_rustls(socket_addr, tls_config)
            .handle(handle)
            .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
            .await
        {
            console::error("TLS server terminated unexpectedly", Some(&e.to_string()));
            flush_on_shutdown_async(storage.clone()).await;
            return false;
        }

        flush_on_shutdown_async(storage.clone()).await;
        return true;
    }

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            console::ok("TCP listener bound", Some(&addr));
            l
        }
        Err(e) => {
            console::error("Failed to bind port", Some(&e.to_string()));
            flush_on_shutdown_async(storage.clone()).await;
            return false;
        }
    };

    console::print_ready(&addr);

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        console::warn("Shutting down HTTP server gracefully...", None);
        let _ = shutdown_tx.send(());
    });

    if let Err(e) = axum::serve(
        listener,
        router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = shutdown_rx.await;
    })
    .await
    {
        console::error("Server terminated unexpectedly", Some(&e.to_string()));
    }

    flush_on_shutdown_async(storage.clone()).await;
    true
}

// ─── /api/v2 extended SDK surface (WEB-02) ─────────────────────────────────
//
// Second slice of the console API: export/import, graph traversal + GDS,
// maintenance, threads, and snapshots. Same rules as WEB-01: the wire format
// is the SDK's own serde, errors are `{success: false, error}` with the
// status from `vanta_error_status`, and all engine work runs under a pool
// permit in `spawn_blocking` via `run_db_op`.

/// Body for `POST /api/v2/export`.
#[derive(Deserialize, Debug)]
struct ExportRequest {
    /// Target path for the export file (JSONL).
    path: String,
    /// When present, exports only this namespace; otherwise exports all.
    namespace: Option<String>,
    /// Optional AND-combined filter applied to the exported records.
    filter: Option<VantaMemoryFilter>,
}

/// Body for `POST /api/v2/import`.
#[derive(Deserialize, Debug)]
struct ImportRequest {
    /// Inline records to import (export wire format). Mutually exclusive with `path`.
    records: Option<Vec<VantaMemoryRecord>>,
    /// Path to a JSONL export (default) or a `.vdbdump` bulk file (`format: "bulk"`).
    path: Option<String>,
    /// File format when `path` is set: `"jsonl"` (default) or `"bulk"`.
    format: Option<String>,
}

#[tracing::instrument(skip(state))]
async fn export_v2(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<ExportRequest>,
) -> Response {
    let namespace = req.namespace.clone();
    let filter = req.filter.clone();
    match run_db_op(&state, move |db| match namespace.as_deref() {
        Some(ns) => db.export_namespace(&req.path, ns, filter),
        None => db.export_all(&req.path),
    })
    .await
    {
        Ok(report) => Json(report).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn import_v2(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<ImportRequest>,
) -> Response {
    let records = req.records.clone();
    let path = req.path.clone();
    let format = req.format.clone();
    // The three import ops return two report types (VantaImportReport vs
    // BulkImportReport); normalize to a JSON value to keep one response path.
    match run_db_op(&state, move |db| -> Result<serde_json::Value> {
        let value = if let Some(records) = records {
            serde_json::to_value(db.import_records(records)?).map_err(VantaError::serialization)?
        } else if let Some(path) = path {
            if format.as_deref() == Some("bulk") {
                serde_json::to_value(db.bulk_import_file(&path)?)
                    .map_err(VantaError::serialization)?
            } else {
                serde_json::to_value(db.import_file(&path)?).map_err(VantaError::serialization)?
            }
        } else {
            return Err(VantaError::InvalidInput(
                "import requires `records` or `path`".into(),
            ));
        };
        Ok(value)
    })
    .await
    {
        Ok(report) => Json(report).into_response(),
        Err(resp) => resp,
    }
}

/// Direction wire enum — `TraversalDirection` (src/graph.rs) is not serde.
#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum GraphDirection {
    Forward,
    Reverse,
    Both,
}

impl From<GraphDirection> for crate::graph::TraversalDirection {
    fn from(d: GraphDirection) -> Self {
        match d {
            GraphDirection::Forward => crate::graph::TraversalDirection::Forward,
            GraphDirection::Reverse => crate::graph::TraversalDirection::Reverse,
            GraphDirection::Both => crate::graph::TraversalDirection::Both,
        }
    }
}

/// Body for `POST /api/v2/graph/bfs` and `/dfs`.
#[derive(Deserialize, Debug)]
struct GraphTraversalRequest {
    /// Node ids to start from.
    roots: Vec<u128>,
    /// Maximum hop depth from the roots.
    max_depth: usize,
    /// Edge direction: `"forward"` (default), `"reverse"`, or `"both"`.
    direction: Option<GraphDirection>,
}

/// Body for `POST /api/v2/graph/degree` and `/centrality`.
#[derive(Deserialize, Debug)]
struct GraphRootsRequest {
    /// Node ids to score.
    roots: Vec<u128>,
}

fn default_pagerank_iterations() -> usize {
    100
}
fn default_pagerank_damping() -> f64 {
    0.85
}
fn default_pagerank_tolerance() -> f64 {
    1e-6
}

/// Body for `POST /api/v2/graph/pagerank`.
#[derive(Deserialize, Debug)]
struct GraphPageRankRequest {
    /// Node ids to score.
    roots: Vec<u128>,
    #[serde(default = "default_pagerank_iterations")]
    max_iterations: usize,
    #[serde(default = "default_pagerank_damping")]
    damping: f64,
    #[serde(default = "default_pagerank_tolerance")]
    tolerance: f64,
}

#[tracing::instrument(skip(state))]
async fn graph_bfs(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphTraversalRequest>,
) -> Response {
    let roots = req.roots.clone();
    let max_depth = req.max_depth;
    let direction = req.direction.unwrap_or(GraphDirection::Forward).into();
    match run_db_op(&state, move |db| db.graph_bfs(&roots, max_depth, direction)).await {
        Ok(ids) => Json(ids).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn graph_dfs(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphTraversalRequest>,
) -> Response {
    let roots = req.roots.clone();
    let max_depth = req.max_depth;
    let direction = req.direction.unwrap_or(GraphDirection::Forward).into();
    match run_db_op(&state, move |db| db.graph_dfs(&roots, max_depth, direction)).await {
        Ok(ids) => Json(ids).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn graph_degree(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphRootsRequest>,
) -> Response {
    let roots = req.roots.clone();
    match run_db_op(&state, move |db| db.graph_degree_centrality(&roots)).await {
        Ok(scores) => Json(scores).into_response(),
        Err(resp) => resp,
    }
}

/// The GDS module exposes a single centrality op (`degree_centrality`), so
/// `/graph/centrality` maps to the same SDK call as `/graph/degree`.
#[tracing::instrument(skip(state))]
async fn graph_centrality(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphRootsRequest>,
) -> Response {
    let roots = req.roots.clone();
    match run_db_op(&state, move |db| db.graph_degree_centrality(&roots)).await {
        Ok(scores) => Json(scores).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn graph_pagerank(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphPageRankRequest>,
) -> Response {
    let roots = req.roots.clone();
    let max_iterations = req.max_iterations;
    let damping = req.damping;
    let tolerance = req.tolerance;
    match run_db_op(&state, move |db| {
        db.graph_page_rank(&roots, max_iterations, damping, tolerance)
    })
    .await
    {
        Ok(scores) => Json(scores).into_response(),
        Err(resp) => resp,
    }
}

// --- Graph v2 (REST-03): desktop-DTO wire with u128-safe string ids --------

/// Wire node — mirror of the desktop `VantaGraphNodeInfo`
/// (desktop/src-tauri/src/connections/types.rs).
#[derive(Serialize, Debug)]
struct GraphNodeDTO {
    /// Numeric node id, serialized as a string (u128 on the core side).
    id: String,
    /// Display label (content/text/__vanta_payload field, id fallback).
    label: String,
    /// Grouping key for coloring (namespace or node type), when known.
    group: Option<String>,
    /// In+out degree centrality (0 when not computed).
    degree: u64,
}

/// Wire edge — mirror of the desktop `VantaGraphEdgeInfo`.
#[derive(Serialize, Debug)]
struct GraphEdgeDTO {
    /// Source node id (string — u128 on the core side).
    source: String,
    /// Target node id (string — u128 on the core side).
    target: String,
    /// Edge label, when the backend exposes one.
    label: Option<String>,
    /// Edge weight, when the backend exposes one.
    weight: Option<f32>,
}

/// Wire traversal result — mirror of the desktop `VantaGraphTraversalResult`.
#[derive(Serialize, Debug, Default)]
struct GraphTraversalDTO {
    nodes: Vec<GraphNodeDTO>,
    edges: Vec<GraphEdgeDTO>,
}

/// Body for `POST /api/v2/graph/v2/bfs` and `/dfs`. Roots are decimal strings
/// so ids above u64::MAX survive the JSON wire (the legacy `/api/v2/graph/*`
/// endpoints take bare u128 numbers, which the browser cannot parse).
#[derive(Deserialize, Debug)]
struct GraphV2TraversalRequest {
    /// Node ids to start from (decimal u128 strings).
    roots: Vec<String>,
    /// Maximum hop depth from the roots.
    max_depth: usize,
    /// Edge direction: `"forward"` (default), `"reverse"`, or `"both"`.
    direction: Option<GraphDirection>,
    /// Cap on the returned node count (default 50).
    limit: Option<usize>,
}

/// Body for `POST /api/v2/graph/v2/degree`.
#[derive(Deserialize, Debug)]
struct GraphV2DegreeRequest {
    /// Namespace whose records are scored.
    namespace: String,
    /// Cap on the returned node count (default 50).
    limit: Option<usize>,
}

/// Parse a wire node-id string into the core's u128 id (native.rs
/// `parse_node_id`).
fn parse_node_id_str(id: &str) -> Result<u128> {
    id.parse::<u128>().map_err(|_| {
        VantaError::InvalidInput(format!(
            "invalid node id '{id}': expected a decimal u128 string"
        ))
    })
}

/// Label/group extraction mirror of native.rs `node_record_to_graph_node`.
fn node_record_to_graph_dto(n: &crate::sdk::VantaNodeRecord) -> GraphNodeDTO {
    let label = ["__vanta_payload", "text", "content"]
        .into_iter()
        .find_map(|k| match n.fields.get(k) {
            Some(crate::sdk::VantaValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| n.id.to_string());
    let group = match n.fields.get("type") {
        Some(crate::sdk::VantaValue::String(s)) => Some(s.clone()),
        _ => None,
    };
    GraphNodeDTO {
        id: n.id.to_string(),
        label,
        group,
        degree: 0,
    }
}

/// Build the wire traversal result from visited node ids, mirror of native.rs
/// `graph_traversal_result`: capped at `cap` nodes; each node's outgoing edges
/// become the edge list (source = node, target = edge target).
fn graph_traversal_dto(db: &VantaEmbedded, ids: &[u128], cap: usize) -> Result<GraphTraversalDTO> {
    let mut result = GraphTraversalDTO::default();
    for id in ids.iter().take(cap) {
        if let Some(node) = db.get_node(*id)? {
            result.nodes.push(node_record_to_graph_dto(&node));
            for edge in &node.edges {
                result.edges.push(GraphEdgeDTO {
                    source: id.to_string(),
                    target: edge.target.to_string(),
                    label: Some(edge.label.clone()),
                    weight: Some(edge.weight),
                });
            }
        }
    }
    Ok(result)
}

/// POST `/api/v2/graph/v2/bfs` — desktop `VantaGraphTraversalResult` wire with
/// u128-safe string ids (REST-03).
#[tracing::instrument(skip(state))]
async fn graph_v2_bfs(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphV2TraversalRequest>,
) -> Response {
    let roots = match req
        .roots
        .iter()
        .map(|r| parse_node_id_str(r))
        .collect::<Result<Vec<u128>>>()
    {
        Ok(roots) => roots,
        Err(e) => return vanta_error_response(&e),
    };
    let max_depth = req.max_depth;
    let direction = req.direction.unwrap_or(GraphDirection::Forward).into();
    let cap = req.limit.unwrap_or(50);
    match run_db_op(&state, move |db| {
        let ids = db.graph_bfs(&roots, max_depth, direction)?;
        graph_traversal_dto(db, &ids, cap)
    })
    .await
    {
        Ok(dto) => Json(dto).into_response(),
        Err(resp) => resp,
    }
}

/// POST `/api/v2/graph/v2/dfs` — desktop `VantaGraphTraversalResult` wire with
/// u128-safe string ids (REST-03).
#[tracing::instrument(skip(state))]
async fn graph_v2_dfs(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphV2TraversalRequest>,
) -> Response {
    let roots = match req
        .roots
        .iter()
        .map(|r| parse_node_id_str(r))
        .collect::<Result<Vec<u128>>>()
    {
        Ok(roots) => roots,
        Err(e) => return vanta_error_response(&e),
    };
    let max_depth = req.max_depth;
    let direction = req.direction.unwrap_or(GraphDirection::Forward).into();
    let cap = req.limit.unwrap_or(50);
    match run_db_op(&state, move |db| {
        let ids = db.graph_dfs(&roots, max_depth, direction)?;
        graph_traversal_dto(db, &ids, cap)
    })
    .await
    {
        Ok(dto) => Json(dto).into_response(),
        Err(resp) => resp,
    }
}

/// POST `/api/v2/graph/v2/degree` — desktop `VantaGraphNodeInfo[]` wire with
/// u128-safe string ids (REST-03). Mirrors native.rs `graph_degree`; an
/// empty/unknown namespace resolves to an empty array, not an error (GRAFO-01).
#[tracing::instrument(skip(state))]
async fn graph_v2_degree(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<GraphV2DegreeRequest>,
) -> Response {
    let ns = req.namespace;
    let cap = req.limit.unwrap_or(50);
    match run_db_op(&state, move |db| {
        let options = VantaMemoryListOptions {
            limit: cap,
            cursor: None,
            ..Default::default()
        };
        let page = db.list(&ns, options)?;
        if page.records.is_empty() {
            return Ok(Vec::new());
        }
        let node_ids: Vec<u128> = page.records.iter().map(|r| r.node_id).collect();
        let degrees = db.graph_degree_centrality(&node_ids)?;
        Ok(page
            .records
            .into_iter()
            .map(|r| GraphNodeDTO {
                id: r.node_id.to_string(),
                label: r.payload.clone(),
                group: Some(ns.clone()),
                degree: degrees
                    .get(&r.node_id)
                    .map(|(in_d, out_d)| (*in_d + *out_d) as u64)
                    .unwrap_or(0),
            })
            .collect())
    })
    .await
    {
        Ok(nodes) => Json(nodes).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn maintenance_purge(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.purge_expired()).await {
        Ok(purged) => Json(serde_json::json!({ "purged": purged })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn maintenance_compact(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.compact_layout()).await {
        Ok(freed_bytes) => Json(serde_json::json!({ "freed_bytes": freed_bytes })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn maintenance_flush(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.flush()).await {
        Ok(()) => Json(serde_json::json!({ "flushed": true })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn maintenance_rebuild_index(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.rebuild_index()).await {
        Ok(report) => Json(report).into_response(),
        Err(resp) => resp,
    }
}

/// Query params for `GET /api/v2/threads`.
#[derive(Deserialize, Debug)]
struct ThreadsListParams {
    /// Maximum number of threads to return.
    #[serde(default = "default_threads_limit")]
    limit: usize,
    /// Offset into the thread list.
    #[serde(default)]
    offset: usize,
}

fn default_threads_limit() -> usize {
    100
}

/// Body for `POST /api/v2/threads`.
#[derive(Deserialize, Debug)]
struct ThreadCreateRequest {
    /// Human-readable thread title.
    title: String,
    /// Optional time-to-live in seconds for the thread.
    ttl_secs: Option<u64>,
}

/// Body for `POST /api/v2/threads/{id}` (send a message).
#[derive(Deserialize, Debug)]
struct ThreadMessageRequest {
    /// Message role (`user`, `assistant`, ...).
    role: String,
    /// Message content.
    content: String,
}

/// 404 body for a missing thread.
fn thread_not_found_response(id: u128) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "success": false,
            "error": format!("thread not found: {id}"),
        })),
    )
        .into_response()
}

/// Wire view of a thread — `MessageThread.thread_id` is a bare `u128` that
/// serde cannot emit as a JSON number (out of u64 range), so it travels as a
/// string, consistent with `u128_serde` elsewhere in the SDK wire format.
#[derive(Serialize)]
struct ThreadDTO {
    thread_id: String,
    title: String,
    messages: Vec<crate::agentic::Message>,
    created_at: u64,
    updated_at: u64,
    metadata: std::collections::HashMap<String, String>,
}

impl From<crate::agentic::MessageThread> for ThreadDTO {
    fn from(t: crate::agentic::MessageThread) -> Self {
        Self {
            thread_id: t.thread_id.to_string(),
            title: t.title,
            messages: t.messages,
            created_at: t.created_at,
            updated_at: t.updated_at,
            metadata: t.metadata,
        }
    }
}

#[tracing::instrument(skip(state))]
async fn threads_list(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<ThreadsListParams>,
) -> Response {
    let limit = params.limit;
    let offset = params.offset;
    match run_db_op(&state, move |db| db.list_threads(limit, offset)).await {
        Ok(threads) => {
            let dtos: Vec<ThreadDTO> = threads.into_iter().map(ThreadDTO::from).collect();
            Json(dtos).into_response()
        }
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn threads_create(
    State(state): State<Arc<ServerState>>,
    Json(req): Json<ThreadCreateRequest>,
) -> Response {
    let title = req.title.clone();
    let ttl_secs = req.ttl_secs;
    match run_db_op(&state, move |db| db.create_thread(&title, ttl_secs)).await {
        Ok(thread_id) => (
            StatusCode::CREATED,
            Json(serde_json::json!({ "thread_id": thread_id.to_string() })),
        )
            .into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn threads_get(
    State(state): State<Arc<ServerState>>,
    AxumPath(thread_id): AxumPath<u128>,
) -> Response {
    match run_db_op(&state, move |db| db.get_thread(thread_id)).await {
        Ok(Some(thread)) => Json(ThreadDTO::from(thread)).into_response(),
        Ok(None) => thread_not_found_response(thread_id),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn threads_send_message(
    State(state): State<Arc<ServerState>>,
    AxumPath(thread_id): AxumPath<u128>,
    Json(req): Json<ThreadMessageRequest>,
) -> Response {
    let role = req.role.clone();
    let content = req.content.clone();
    match run_db_op(&state, move |db| {
        db.send_message(thread_id, &role, &content)
    })
    .await
    {
        Ok(()) => Json(serde_json::json!({ "sent": true })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn threads_delete(
    State(state): State<Arc<ServerState>>,
    AxumPath(thread_id): AxumPath<u128>,
) -> Response {
    match run_db_op(&state, move |db| db.delete_thread(thread_id)).await {
        Ok(()) => Json(serde_json::json!({ "deleted": true })).into_response(),
        Err(resp) => resp,
    }
}

/// Body for `POST /conversation/add` (F3 data plane): record one message in a
/// conversation. When `thread_id` is absent, a new thread is created first —
/// the agent does not need to pre-create a thread to accumulate context.
#[derive(Deserialize, Debug)]
struct ConversationAddRequest {
    /// Existing thread id (u128 as decimal string). When absent, a thread is
    /// created with `title` (defaults to `"conversation"`) and `ttl_secs`.
    thread_id: Option<String>,
    /// Human-readable thread title, used only when creating a new thread.
    title: Option<String>,
    /// Message role (`user`, `assistant`, ...).
    role: String,
    /// Message content.
    content: String,
    /// Optional time-to-live in seconds for a newly created thread.
    ttl_secs: Option<u64>,
}

#[tracing::instrument(skip(state))]
async fn conversation_add(
    State(state): State<Arc<ServerState>>,
    request_id: RequestId,
    Json(req): Json<ConversationAddRequest>,
) -> Response {
    let thread_id = match req.thread_id.as_deref() {
        Some(raw) => match raw.parse::<u128>() {
            Ok(id) => Some(id),
            Err(_) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(serde_json::json!({
                        "success": false,
                        "error": format!("invalid thread_id: {raw:?}"),
                    })),
                )
                    .into_response();
            }
        },
        None => None,
    };
    let title = req
        .title
        .clone()
        .unwrap_or_else(|| "conversation".to_string());
    let ttl_secs = req.ttl_secs;
    let role = req.role.clone();
    let content = req.content.clone();
    // SRV-02: carry the caller's tracing id into the audit event.
    let rid = request_id.0;

    match run_db_op(&state, move |db| {
        let id = match thread_id {
            Some(id) => id,
            None => db.create_thread(&title, ttl_secs)?,
        };
        db.send_message(id, &role, &content)?;
        db.audit(
            AuditEvent::memory("conversation", "threads", &id.to_string(), "ok", None)
                .with_request_id_opt(rid),
        );
        Ok(id)
    })
    .await
    {
        Ok(id) => {
            // MEM-55: fire the memory-pipeline trigger best-effort. Any error
            // is logged and swallowed — the HTTP response reflects only the
            // thread save (P4: extraction failures never fail the request).
            if let Some(trigger) = &state.conversation_trigger {
                if let Err(err) = trigger.trigger(id, &req.role, &req.content) {
                    tracing::warn!(thread = %id, %err, "conversation trigger failed; ignoring");
                }
            }
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "success": true, "thread_id": id.to_string() })),
            )
                .into_response()
        }
        Err(resp) => resp,
    }
}

/// Query params for `GET /skill/listing` (F3 data plane): head rows of the
/// skill store with optional filters — enough for prompt-injection use cases.
#[derive(Deserialize, Debug)]
struct SkillListingParams {
    /// Only list skills owned by this agent.
    owner_agent: Option<String>,
    /// Only list skills whose name starts with this prefix.
    name_prefix: Option<String>,
    /// Maximum number of items to return (default 50, capped at 200).
    limit: Option<usize>,
    /// Number of items to skip.
    offset: Option<usize>,
}

/// Lean wire view of a skill head row — skill metadata without the content
/// body (the listing is for prompt injection, not for dumping full skills).
#[derive(Serialize)]
struct SkillListingItem {
    skill_id: String,
    version: u64,
    name: String,
    owner_agent: String,
    description: String,
}

#[tracing::instrument(skip(state))]
async fn skill_listing(
    State(state): State<Arc<ServerState>>,
    Query(params): Query<SkillListingParams>,
) -> Response {
    let limit = params.limit.unwrap_or(50).min(200);
    let offset = params.offset.unwrap_or(0);
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        let store = crate::skills::SkillStore::new(&engine);
        store.list(crate::sdk::SkillListOptions {
            owner_agent: params.owner_agent,
            name_prefix: params.name_prefix,
            limit,
            offset,
        })
    })
    .await
    {
        Ok(page) => {
            let items: Vec<SkillListingItem> = page
                .items
                .into_iter()
                .map(|r| SkillListingItem {
                    skill_id: r.skill_id,
                    version: r.version,
                    name: r.name,
                    owner_agent: r.owner_agent,
                    description: r.description,
                })
                .collect();
            Json(serde_json::json!({ "items": items, "total": page.total })).into_response()
        }
        Err(resp) => resp,
    }
}

/// Query params for the mutating skill endpoints (PUT/PATCH/DELETE).
///
/// `expected_version` is the optimistic lock (MEM-06 pattern): a stale value
/// surfaces as 409 via `VantaError::ExecutionConflict`. `owner_agent` is
/// checked against the head's owner — a mismatch returns the SAME 404 as a
/// missing skill (no existence oracle for other agents' skills).
#[derive(Deserialize, Debug)]
struct SkillMutationParams {
    owner_agent: String,
    expected_version: u64,
}

/// Resolve a skill head enforcing ownership. Missing skill and foreign-owned
/// skill are indistinguishable on the wire (both `NotFound` → 404).
fn require_owned_head(
    store: &crate::skills::SkillStore<'_>,
    skill_id: &str,
    owner_agent: &str,
) -> crate::error::Result<crate::sdk::SkillRecord> {
    match store.get_head(skill_id)? {
        Some(head) if head.owner_agent == owner_agent => Ok(head),
        _ => Err(VantaError::NotFound {
            kind: "skill".into(),
            id: skill_id.into(),
        }),
    }
}

/// `POST /api/v2/skills` — create a skill (version 1). Idempotent when the
/// same `(owner_agent, name)` + content already exists (`idempotent: true`).
#[tracing::instrument(skip(state))]
async fn skill_create(
    State(state): State<Arc<ServerState>>,
    Json(input): Json<crate::sdk::SkillCreateInput>,
) -> Response {
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        crate::skills::SkillStore::new(&engine).create(input)
    })
    .await
    {
        Ok(result) => (StatusCode::CREATED, Json(result)).into_response(),
        Err(resp) => resp,
    }
}

/// `PUT /api/v2/skills/{skill_id}?owner_agent=…&expected_version=…` — full
/// update of description+content, appending a new version.
#[tracing::instrument(skip(state))]
async fn skill_update(
    State(state): State<Arc<ServerState>>,
    AxumPath(skill_id): AxumPath<String>,
    Query(params): Query<SkillMutationParams>,
    Json(input): Json<crate::sdk::SkillUpdateInput>,
) -> Response {
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        let store = crate::skills::SkillStore::new(&engine);
        require_owned_head(&store, &skill_id, &params.owner_agent)?;
        store.update(&skill_id, params.expected_version, input)
    })
    .await
    {
        Ok(result) => Json(result).into_response(),
        Err(resp) => resp,
    }
}

/// `PATCH /api/v2/skills/{skill_id}?owner_agent=…&expected_version=…` —
/// partial update; only provided fields change.
#[tracing::instrument(skip(state))]
async fn skill_patch(
    State(state): State<Arc<ServerState>>,
    AxumPath(skill_id): AxumPath<String>,
    Query(params): Query<SkillMutationParams>,
    Json(input): Json<crate::sdk::SkillPatchInput>,
) -> Response {
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        let store = crate::skills::SkillStore::new(&engine);
        require_owned_head(&store, &skill_id, &params.owner_agent)?;
        store.patch(&skill_id, params.expected_version, input)
    })
    .await
    {
        Ok(result) => Json(result).into_response(),
        Err(resp) => resp,
    }
}

/// `DELETE /api/v2/skills/{skill_id}?owner_agent=…&expected_version=…` —
/// removes every version plus the head index row.
#[tracing::instrument(skip(state))]
async fn skill_delete(
    State(state): State<Arc<ServerState>>,
    AxumPath(skill_id): AxumPath<String>,
    Query(params): Query<SkillMutationParams>,
) -> Response {
    match run_db_op(&state, move |db| {
        let engine = db.engine_handle()?;
        let store = crate::skills::SkillStore::new(&engine);
        require_owned_head(&store, &skill_id, &params.owner_agent)?;
        store.delete(&skill_id, params.expected_version)
    })
    .await
    {
        Ok(deleted) => Json(serde_json::json!({ "deleted": deleted })).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn snapshots_list(State(state): State<Arc<ServerState>>) -> Response {
    match run_db_op(&state, move |db| db.list_snapshots()).await {
        Ok(names) => Json(names).into_response(),
        Err(resp) => resp,
    }
}

/// `FsSnapshot` (storage/engine) is not serializable (`created_at` is a
/// monotonic `Instant`), so the wire shape carries name + path only.
#[tracing::instrument(skip(state))]
async fn snapshots_create(
    State(state): State<Arc<ServerState>>,
    AxumPath(name): AxumPath<String>,
) -> Response {
    let name_label = name.clone();
    match run_db_op(&state, move |db| db.create_snapshot(&name)).await {
        Ok(snapshot) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "name": name_label,
                "path": snapshot.path.to_string_lossy(),
            })),
        )
            .into_response(),
        Err(resp) => resp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VantaError;

    #[test]
    fn validate_auth_allows_key_without_require() {
        let cfg = VantaConfig {
            api_key: Some("sk-test".into()),
            require_auth: false,
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn request_timeouts_are_sane() {
        // Interactive timeout must be bounded (DoS) and non-zero; the
        // long-running ceiling must comfortably exceed it so bulk operations
        // (import/export/rebuild-index) aren't killed by the interactive cap.
        assert!(REQUEST_TIMEOUT > Duration::ZERO);
        assert!(REQUEST_TIMEOUT <= Duration::from_secs(120));
        assert!(LONG_REQUEST_TIMEOUT > REQUEST_TIMEOUT);
        assert!(LONG_REQUEST_TIMEOUT >= Duration::from_secs(300));
    }

    #[tokio::test]
    async fn slow_request_times_out_with_408() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        // A handler that runs longer than the timeout must be cut with 408.
        let router = Router::new()
            .route(
                "/slow",
                get(|| async {
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    StatusCode::OK
                }),
            )
            .layer(tower_http::timeout::TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_millis(50),
            ));

        let res = router
            .oneshot(Request::builder().uri("/slow").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::REQUEST_TIMEOUT);
    }

    #[tokio::test]
    async fn fast_request_not_timed_out() {
        use axum::body::Body;
        use axum::http::Request;
        use tower::ServiceExt;

        // A handler that completes within the timeout must pass through (200).
        let router = Router::new()
            .route("/fast", get(|| async { StatusCode::OK }))
            .layer(tower_http::timeout::TimeoutLayer::with_status_code(
                StatusCode::REQUEST_TIMEOUT,
                Duration::from_millis(500),
            ));

        let res = router
            .oneshot(Request::builder().uri("/fast").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[test]
    fn validate_auth_allows_no_key_without_require() {
        let cfg = VantaConfig {
            api_key: None,
            require_auth: false,
            host: "127.0.0.1".into(),
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn validate_auth_allows_key_with_require() {
        let cfg = VantaConfig {
            api_key: Some("sk-test".into()),
            require_auth: true,
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn validate_auth_rejects_no_key_with_require() {
        let cfg = VantaConfig {
            api_key: None,
            require_auth: true,
            ..Default::default()
        };
        let err = validate_auth_config(&cfg).unwrap_err();
        match err {
            VantaError::InvalidInput(msg) => {
                assert!(msg.contains("require_auth"), "msg: {msg}");
            }
            other => panic!("expected InvalidInput, got {other:?}"),
        }
    }

    /// FIND-07 (a): non-loopback host + no key → refuse with actionable message.
    #[test]
    fn refuse_to_start_non_loopback_without_key() {
        for host in ["0.0.0.0", "192.168.1.10", "example.com", "::"] {
            let cfg = VantaConfig {
                api_key: None,
                require_auth: false,
                allow_insecure: false,
                host: host.into(),
                ..Default::default()
            };
            let err = validate_auth_config(&cfg).unwrap_err();
            match err {
                VantaError::InvalidInput(msg) => {
                    assert!(
                        msg.contains("VANTADB_API_KEY") && msg.contains("allow-insecure"),
                        "host {host}: msg lacks remediation: {msg}"
                    );
                }
                other => panic!("expected InvalidInput for {host}, got {other:?}"),
            }
        }
    }

    /// FIND-07 (b): same non-loopback host + `--allow-insecure` → starts
    /// (with a prominent WARNING logged to console).
    #[test]
    fn allow_insecure_bypasses_non_loopback_refusal() {
        let cfg = VantaConfig {
            api_key: None,
            require_auth: false,
            allow_insecure: true,
            host: "0.0.0.0".into(),
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    /// FIND-07 (c): loopback hosts without a key start normally.
    #[test]
    fn loopback_hosts_start_normally() {
        for host in ["127.0.0.1", "localhost", "::1", "[::1]"] {
            let cfg = VantaConfig {
                api_key: None,
                require_auth: false,
                allow_insecure: false,
                host: host.into(),
                ..Default::default()
            };
            assert!(
                validate_auth_config(&cfg).is_ok(),
                "loopback host {host} must start without a key"
            );
        }
    }

    /// FIND-07: an API key makes any host acceptable regardless of the override.
    #[test]
    fn api_key_accepts_any_host() {
        let cfg = VantaConfig {
            api_key: Some("sk-test".into()),
            require_auth: false,
            allow_insecure: false,
            host: "0.0.0.0".into(),
            ..Default::default()
        };
        assert!(validate_auth_config(&cfg).is_ok());
    }

    #[test]
    fn is_loopback_host_classification() {
        assert!(is_loopback_host("127.0.0.1"));
        assert!(is_loopback_host("127.9.9.9"));
        assert!(is_loopback_host("localhost"));
        assert!(is_loopback_host("LOCALHOST"));
        assert!(is_loopback_host("::1"));
        assert!(is_loopback_host("[::1]"));
        assert!(!is_loopback_host("0.0.0.0"));
        assert!(!is_loopback_host("::"));
        assert!(!is_loopback_host("192.168.1.10"));
        assert!(!is_loopback_host("db.internal")); // unresolvable → fail closed
        assert!(!is_loopback_host(""));
    }

    #[tokio::test]
    async fn body_limit_rejects_oversized() {
        // In-memory engine so the test touches no disk.
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: None,
        });

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(
                listener,
                app(state, 0).into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        });

        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        async fn http_request(addr: std::net::SocketAddr, body: &[u8]) -> String {
            let mut request = format!(
                "POST /api/v2/query HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .into_bytes();
            request.extend_from_slice(body);

            let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
            stream.write_all(&request).await.unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).await.unwrap();
            response
        }

        // Body larger than the 1_000_000-byte DefaultBodyLimit → 413.
        let oversized = http_request(addr, &vec![b'x'; 1_000_001]).await;
        assert!(
            oversized.starts_with("HTTP/1.1 413"),
            "expected 413 for oversized body, got: {oversized}"
        );

        // A small body must not be rejected by the limit (status is whatever the
        // handler returns — auth/parse — but never 413).
        let small = http_request(addr, br#"{"query":"SELECT 1"}"#).await;
        assert!(
            !small.starts_with("HTTP/1.1 413"),
            "small body should not hit the body limit, got: {small}"
        );
    }

    /// MEM-55 fake trigger: records invocations, optionally failing (P4).
    struct RecordingTrigger {
        calls: Arc<std::sync::Mutex<Vec<(u128, String, String)>>>,
        fail: bool,
    }

    impl ConversationTrigger for RecordingTrigger {
        fn trigger(
            &self,
            thread_id: u128,
            role: &str,
            content: &str,
        ) -> std::result::Result<(), String> {
            self.calls
                .lock()
                .unwrap()
                .push((thread_id, role.to_string(), content.to_string()));
            if self.fail {
                Err("llm down".to_string())
            } else {
                Ok(())
            }
        }
    }

    async fn raw_post_conversation_add(addr: std::net::SocketAddr, body: &str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let request = format!(
            "POST /conversation/add HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes();
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.write_all(&request).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    }

    #[tokio::test]
    async fn conversation_add_fires_trigger_after_save() {
        let calls: Arc<std::sync::Mutex<Vec<(u128, String, String)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let trigger = RecordingTrigger {
            calls: calls.clone(),
            fail: false,
        };
        // Build inline: db handle must come from the SAME state as the route.
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db: db.clone(),
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: Some(Arc::new(trigger)),
        });
        let addr = spawn_app(app(state, 0)).await;

        let raw =
            raw_post_conversation_add(addr, r#"{"role":"user","content":"I prefer dark mode"}"#)
                .await;
        assert!(raw.starts_with("HTTP/1.1 201"), "got: {raw}");
        let body_start = raw.find("{\"success\"").expect("json body");
        let json: serde_json::Value =
            serde_json::from_str(raw[body_start..raw.len()].trim_end()).unwrap();
        assert_eq!(json["success"], serde_json::json!(true));
        let thread_id: u128 = json["thread_id"]
            .as_str()
            .unwrap()
            .parse()
            .expect("decimal thread id");

        // Thread actually saved...
        assert!(
            db.get_thread(thread_id).unwrap().is_some(),
            "thread must be persisted"
        );
        // ...and trigger fired once with the saved identity.
        let got = calls.lock().unwrap().clone();
        assert_eq!(
            got,
            vec![(
                thread_id,
                "user".to_string(),
                "I prefer dark mode".to_string()
            )]
        );
    }

    #[tokio::test]
    async fn conversation_add_trigger_failure_does_not_fail_response() {
        let calls: Arc<std::sync::Mutex<Vec<(u128, String, String)>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: Some(Arc::new(RecordingTrigger { calls, fail: true })),
        });
        let addr = spawn_app(app(state, 0)).await;

        let raw = raw_post_conversation_add(addr, r#"{"role":"user","content":"hi"}"#).await;
        assert!(
            raw.starts_with("HTTP/1.1 201"),
            "P4 violated — extraction failure leaked into HTTP: {raw}"
        );
        assert!(raw.contains("\"success\":true"), "got: {raw}");
    }

    /// Spawn the app router on an ephemeral port, returning its address.
    async fn spawn_app(router: Router) -> std::net::SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(
                listener,
                router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            .unwrap();
        });
        addr
    }

    /// Send GET /health with an `Origin` header and return the raw HTTP response.
    async fn raw_get_with_origin(addr: std::net::SocketAddr, origin: &str) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let request = format!(
            "GET /health HTTP/1.1\r\nHost: {addr}\r\nOrigin: {origin}\r\nConnection: close\r\n\r\n"
        );
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    }

    async fn cors_test_state() -> Arc<ServerState> {
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: None,
        })
    }

    #[tokio::test]
    async fn cors_disabled_omits_allow_origin_header() {
        // No allowed origins → no CORS headers on the response.
        let state = cors_test_state().await;
        let addr = spawn_app(app(state, 0)).await;
        let response = raw_get_with_origin(addr, "http://attacker.example.com").await;
        assert!(
            !response
                .to_lowercase()
                .contains("access-control-allow-origin"),
            "expected no CORS header, got: {response}"
        );
    }

    #[tokio::test]
    async fn cors_configured_returns_allow_origin_header() {
        // Allowed origin matching the request → header echoes the origin.
        let state = cors_test_state().await;
        let addr = spawn_app(app_with_cors(
            state,
            0,
            &["http://app.example.com".to_string()],
        ))
        .await;
        let resp = raw_get_with_origin(addr, "http://app.example.com").await;
        assert!(
            resp.to_lowercase()
                .contains("access-control-allow-origin: http://app.example.com"),
            "expected CORS allow-origin header, got: {resp}"
        );
    }

    /// Send a raw HTTP/1.1 request and return the full response text.
    async fn raw_request(addr: std::net::SocketAddr, request: String) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
        stream.write_all(request.as_bytes()).await.unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).await.unwrap();
        response
    }

    /// Build an HTTP/1.1 request with a JSON body.
    fn json_request(method: &str, path: &str, addr: std::net::SocketAddr, body: &str) -> String {
        format!(
            "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    /// Split a raw HTTP response into (status_code, body).
    fn parse_response(raw: &str) -> (u16, String) {
        let status = raw
            .lines()
            .next()
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(0);
        let body = raw
            .split_once("\r\n\r\n")
            .map(|(_, b)| b.to_string())
            .unwrap_or_default();
        (status, body)
    }

    async fn raw_get(addr: std::net::SocketAddr, path: &str) -> (u16, String) {
        parse_response(
            &raw_request(
                addr,
                format!("GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"),
            )
            .await,
        )
    }

    async fn raw_delete(addr: std::net::SocketAddr, path: &str) -> (u16, String) {
        parse_response(
            &raw_request(
                addr,
                format!("DELETE {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: close\r\n\r\n"),
            )
            .await,
        )
    }

    /// Percent-encode a query parameter value (RFC 3986 unreserved passthrough).
    fn urlencode(s: &str) -> String {
        let mut out = String::new();
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }

    #[tokio::test]
    async fn v2_records_roundtrip() {
        // In-memory engine + a temp audit log so the full /api/v2 surface
        // (records, list, health, audit) is exercised end to end.
        let dir = tempfile::tempdir().unwrap();
        let audit_path = dir.path().join("audit.jsonl");
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            audit_log_path: Some(audit_path),
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: None,
        });
        let addr = spawn_app(app(state, 0)).await;

        // PUT /api/v2/records → 201 with the stored record.
        let put_body = r#"{"namespace":"mem","key":"k1","payload":"hello world","metadata":{"kind":{"String":"note"}},"vector":null,"ttl_ms":null}"#;
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/records", addr, put_body),
            )
            .await,
        );
        assert_eq!(status, 201, "put status: {body}");
        let record: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(record["namespace"], "mem");
        assert_eq!(record["key"], "k1");
        assert_eq!(record["payload"], "hello world");

        // PUT batch → 201 with two records.
        let batch_body = concat!(
            r#"[{"namespace":"mem","key":"k2","payload":"two","metadata":{"kind":{"String":"note"}},"vector":null,"ttl_ms":null},"#,
            r#"{"namespace":"mem","key":"k3","payload":"three","metadata":{"kind":{"String":"todo"}},"vector":null,"ttl_ms":null}]"#
        );
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/records/batch", addr, batch_body),
            )
            .await,
        );
        assert_eq!(status, 201, "batch status: {body}");
        let records: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(records.as_array().unwrap().len(), 2);

        // GET record → 200 with the payload.
        let (status, body) = raw_get(addr, "/api/v2/records/mem/k1").await;
        assert_eq!(status, 200, "get status: {body}");
        let record: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(record["payload"], "hello world");

        // GET versions → 200 array; GET version=N → single record.
        let (status, body) = raw_get(addr, "/api/v2/records/mem/k1/versions").await;
        assert_eq!(status, 200, "versions status: {body}");
        let versions: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(versions.as_array().unwrap().len(), 1);
        let (status, body) = raw_get(addr, "/api/v2/records/mem/k1/versions?version=1").await;
        assert_eq!(status, 200, "get_version status: {body}");

        // LIST with cursor → 200 page with the 3 records.
        let (status, body) = raw_get(addr, "/api/v2/list?namespace=mem").await;
        assert_eq!(status, 200, "list status: {body}");
        let page: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(page["records"].as_array().unwrap().len(), 3);

        // DELETE single → 200 {deleted: true}.
        let (status, body) = raw_delete(addr, "/api/v2/records/mem/k3").await;
        assert_eq!(status, 200, "delete status: {body}");
        let deleted: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(deleted["deleted"], true);

        // DELETE by filter (kind == note) → removes k1 + k2.
        let filter = r#"[{"field":"kind","op":"Eq","value":{"String":"note"}}]"#;
        let path = format!("/api/v2/records?namespace=mem&filter={}", urlencode(filter));
        let (status, body) = raw_delete(addr, &path).await;
        assert_eq!(status, 200, "delete_by_filter status: {body}");
        let deleted: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(deleted["deleted"], 2);

        // HEALTH → 200 healthy.
        let (status, body) = raw_get(addr, "/api/v2/health").await;
        assert_eq!(status, 200, "health status: {body}");
        let health: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(health["status"], "healthy");
        assert_eq!(health["backend"], "in-memory");

        // AUTOCOMPLETE → 200 array.
        let (status, body) = raw_get(addr, "/api/v2/autocomplete?prefix=FRO").await;
        assert_eq!(status, 200, "autocomplete status: {body}");
        let completions: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(completions.is_array());

        // AUDIT → 200 with the put events written through the shared logger.
        let (status, body) = raw_get(addr, "/api/v2/audit").await;
        assert_eq!(status, 200, "audit status: {body}");
        let page: serde_json::Value = serde_json::from_str(&body).unwrap();
        let events = page["events"].as_array().unwrap();
        assert!(!events.is_empty(), "expected audit events, got: {body}");
        assert!(
            events.iter().any(|e| e["op"] == "put"),
            "expected a put event, got: {body}"
        );
    }

    #[tokio::test]
    async fn v2_list_and_search_paginate() {
        // REST-04: paginación verificable — 2 llamadas con limit N devuelven
        // N y el resto, sin duplicados entre páginas, para list Y search.
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: None,
        });
        let addr = spawn_app(app(state, 0)).await;

        // 5 records con vectores de distinta similitud al query → ranking estable.
        for (key, vec) in [
            ("k1", "[1.0,0.0,0.0]"),
            ("k2", "[0.8,0.2,0.0]"),
            ("k3", "[0.6,0.4,0.0]"),
            ("k4", "[0.4,0.6,0.0]"),
            ("k5", "[0.2,0.8,0.0]"),
        ] {
            let body = format!(
                r#"{{"namespace":"ns","key":"{key}","payload":"p-{key}","metadata":{{}},"vector":{vec},"ttl_ms":null}}"#
            );
            let (status, body) = parse_response(
                &raw_request(addr, json_request("POST", "/api/v2/records", addr, &body)).await,
            );
            assert_eq!(status, 201, "put {key}: {body}");
        }

        // LIST paginado: 2 → next_cursor → 2 → next_cursor → 1, sin duplicados.
        let mut seen: Vec<String> = Vec::new();
        let mut cursor = None;
        loop {
            let path = match cursor {
                Some(c) => format!("/api/v2/list?namespace=ns&limit=2&cursor={c}"),
                None => "/api/v2/list?namespace=ns&limit=2".to_string(),
            };
            let (status, body) = raw_get(addr, &path).await;
            assert_eq!(status, 200, "list status: {body}");
            let page: serde_json::Value = serde_json::from_str(&body).unwrap();
            let records = page["records"].as_array().unwrap();
            assert!(
                records.len() <= 2,
                "page has more than limit records: {body}"
            );
            for r in records {
                let key = r["key"].as_str().unwrap().to_string();
                assert!(!seen.contains(&key), "duplicate key {key} in {body}");
                seen.push(key);
            }
            cursor = page["next_cursor"].as_u64().map(|v| v as usize);
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(
            seen.len(),
            5,
            "list pagination must yield all 5, got: {seen:?}"
        );

        // SEARCH paginado: limit=2 + cursor offset sobre el mismo ranking.
        // Base sin llave de cierre: el cursor se concatena cuando corresponde.
        let search_base = r#"{"namespace":"ns","query_vector":[1.0,0.0,0.0],"query_sparse":null,"filters":{},"text_query":null,"top_k":5,"distance_metric":"Cosine","explain":false,"limit":2"#;
        let mut seen: Vec<String> = Vec::new();
        let mut cursor: Option<usize> = None;
        loop {
            let body = match cursor {
                Some(c) => format!(r#"{search_base},"cursor":{c}}}"#),
                None => format!(r#"{search_base}}}"#),
            };
            let (status, resp) = parse_response(
                &raw_request(addr, json_request("POST", "/api/v2/search", addr, &body)).await,
            );
            assert_eq!(status, 200, "search status: {resp}");
            let page: serde_json::Value = serde_json::from_str(&resp).unwrap();
            let records = page["records"].as_array().unwrap();
            assert!(records.len() <= 2, "page has more than limit: {resp}");
            for r in records {
                let key = r["record"]["key"].as_str().unwrap().to_string();
                assert!(!seen.contains(&key), "duplicate search key {key} in {resp}");
                seen.push(key);
            }
            cursor = page["next_cursor"].as_u64().map(|v| v as usize);
            if cursor.is_none() {
                break;
            }
        }
        assert_eq!(
            seen.len(),
            5,
            "search pagination must yield all 5, got: {seen:?}"
        );
    }

    #[tokio::test]
    async fn v2_list_and_search_without_namespace_aggregate_all() {
        // Regresión: la consola (grid MEMORIAS + búsqueda global) llama list/
        // search SIN namespace esperando ver TODOS los registros. Antes el
        // server defaulteaba a "default" y mostraba "Sin registros" con datos
        // presentes en otros namespaces.
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: None,
        });
        let addr = spawn_app(app(state, 0)).await;

        for (ns, key) in [("alpha", "a1"), ("alpha", "a2"), ("beta", "b1")] {
            let body = format!(
                r#"{{"namespace":"{ns}","key":"{key}","payload":"p-{key}","metadata":{{}},"vector":[1.0,0.0,0.0],"ttl_ms":null}}"#
            );
            let (status, resp) = parse_response(
                &raw_request(addr, json_request("POST", "/api/v2/records", addr, &body)).await,
            );
            assert_eq!(status, 201, "put {ns}/{key}: {resp}");
        }

        // LIST sin namespace → los 3 registros de ambos namespaces.
        let (status, body) = raw_get(addr, "/api/v2/list").await;
        assert_eq!(status, 200, "list all: {body}");
        let page: serde_json::Value = serde_json::from_str(&body).unwrap();
        let records = page["records"].as_array().unwrap();
        assert_eq!(
            records.len(),
            3,
            "list sin namespace debe agregar todos: {body}"
        );
        let namespaces: Vec<&str> = records
            .iter()
            .map(|r| r["namespace"].as_str().unwrap())
            .collect();
        assert!(namespaces.contains(&"alpha") && namespaces.contains(&"beta"));

        // AUD-046: la respuesta del fan-out siempre lleva la señal aditiva
        // `truncated_namespaces` — vacía cuando ningún namespace superó NS_CAP
        // (nunca truncamiento silencioso).
        let truncated: Vec<String> = page["truncated_namespaces"]
            .as_array()
            .expect("fan-out response must carry truncated_namespaces")
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert!(
            truncated.is_empty(),
            "no namespace exceeded NS_CAP here: {truncated:?}"
        );

        // LIST con namespace explícito sigue filtrando (sin cambio de contrato).
        let (status, body) = raw_get(addr, "/api/v2/list?namespace=beta").await;
        assert_eq!(status, 200, "list beta: {body}");
        let page: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(page["records"].as_array().unwrap().len(), 1);

        // SEARCH sin namespace → hits de ambos namespaces (search_all).
        let body = r#"{"namespace":"","query_vector":[1.0,0.0,0.0],"query_sparse":null,"filters":{},"text_query":null,"top_k":10,"distance_metric":"Cosine","explain":false}"#;
        let (status, resp) = parse_response(
            &raw_request(addr, json_request("POST", "/api/v2/search", addr, body)).await,
        );
        assert_eq!(status, 200, "search all: {resp}");
        let page: serde_json::Value = serde_json::from_str(&resp).unwrap();
        let hits = page["records"].as_array().unwrap();
        assert!(
            !hits.is_empty(),
            "search sin namespace debe buscar en todos: {resp}"
        );
        let hit_namespaces: Vec<&str> = hits
            .iter()
            .map(|h| h["record"]["namespace"].as_str().unwrap_or(""))
            .collect();
        assert!(
            hit_namespaces.contains(&"alpha") && hit_namespaces.contains(&"beta"),
            "search sin namespace debe cubrir ambos namespaces: {resp}"
        );
    }

    #[test]
    fn merge_all_namespaces_pages_signals_truncation() {
        // AUD-046: una página con `next_cursor` ⇒ el namespace quedó truncado
        // en NS_CAP y DEBE aparecer en la señal; sin cursor ⇒ namespace
        // completo. El merge preserva el orden estable por namespace.
        fn record(ns: &str) -> VantaMemoryRecord {
            VantaMemoryRecord {
                namespace: ns.to_string(),
                key: "k".to_string(),
                payload: String::new(),
                metadata: crate::sdk::VantaMemoryMetadata::new(),
                created_at_ms: 0,
                updated_at_ms: 0,
                version: 0,
                node_id: 0,
                vector: None,
                sparse_vector: None,
                expires_at_ms: None,
                superseded_by: None,
                superseded_at_ms: None,
            }
        }
        let pages = vec![
            (
                "alpha".to_string(),
                VantaMemoryListPage {
                    records: vec![record("alpha")],
                    next_cursor: Some(10_000),
                },
            ),
            (
                "beta".to_string(),
                VantaMemoryListPage {
                    records: vec![record("beta"), record("beta")],
                    next_cursor: None,
                },
            ),
        ];
        let (records, truncated) = merge_all_namespaces_pages(pages);
        assert_eq!(records.len(), 3);
        assert_eq!(truncated, vec!["alpha".to_string()]);
    }

    #[tokio::test]
    async fn v2_errors_map_status() {
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: None,
        });
        let addr = spawn_app(app(state, 0)).await;

        // GET/DELETE of a missing record → 404 with the error shape.
        let (status, body) = raw_get(addr, "/api/v2/records/mem/missing").await;
        assert_eq!(status, 404, "get missing: {body}");
        assert!(body.contains("\"success\":false"), "error shape: {body}");
        let (status, body) = raw_delete(addr, "/api/v2/records/mem/missing").await;
        assert_eq!(status, 404, "delete missing: {body}");

        // LIST without namespace → 200 (agrega TODOS los namespaces — el grid
        // de la consola lista sin namespace; ver v2_list_without_namespace).
        let (status, body) = raw_get(addr, "/api/v2/list").await;
        assert_eq!(status, 200, "list no namespace: {body}");

        // DELETE by filter with invalid JSON → 400.
        let (status, body) = raw_delete(addr, "/api/v2/records?namespace=mem&filter=notjson").await;
        assert_eq!(status, 400, "filter parse: {body}");

        // DELETE by filter with an empty filter → 400 (SDK guard).
        let (status, body) = raw_delete(addr, "/api/v2/records?namespace=mem&filter=%5B%5D").await;
        assert_eq!(status, 400, "empty filter: {body}");

        // AUDIT without audit_log_path → 404.
        let (status, body) = raw_get(addr, "/api/v2/audit").await;
        assert_eq!(status, 404, "audit not configured: {body}");
    }

    #[tokio::test]
    async fn audit_rotates_and_still_serves_active_file() {
        // SRV-01: writes past `audit_max_bytes` must rotate the JSONL to `.1`
        // while `GET /api/v2/audit` keeps serving the active file.
        let dir = tempfile::tempdir().unwrap();
        let audit_path = dir.path().join("audit.jsonl");
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            audit_log_path: Some(audit_path.clone()),
            audit_max_bytes: 400,
            audit_max_files: 2,
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: None,
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: None,
        });
        let addr = spawn_app(app(state, 0)).await;

        // 6 puts → ~110B audit line each → exceeds the 400B cap → rotation.
        for i in 0..6 {
            let body = format!(
                r#"{{"namespace":"mem","key":"k{i}","payload":"hello {i}","metadata":{{"kind":{{"String":"note"}}}},"vector":null,"ttl_ms":null}}"#
            );
            let (status, _) = parse_response(
                &raw_request(addr, json_request("POST", "/api/v2/records", addr, &body)).await,
            );
            assert_eq!(status, 201, "put #{i}");
        }
        assert!(
            dir.path().join("audit.jsonl.1").exists(),
            "audit must rotate to .1 after exceeding audit_max_bytes"
        );
        assert!(
            !dir.path().join("audit.jsonl.3").exists(),
            "archives beyond audit_max_files must be pruned"
        );

        // The active file still serves the audit endpoint.
        let (status, body) = raw_get(addr, "/api/v2/audit").await;
        assert_eq!(status, 200, "audit after rotation: {body}");
        let page: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            !page["events"].as_array().unwrap().is_empty(),
            "audit page must not be empty after rotation"
        );
    }

    #[tokio::test]
    async fn audit_event_carries_request_id() {
        // SRV-02: a request with `x-request-id` must surface that id in the
        // audit events it produces (auth events here — the guaranteed per-
        // request audit record).
        let dir = tempfile::tempdir().unwrap();
        let audit_path = dir.path().join("audit.jsonl");
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::InMemory,
            audit_log_path: Some(audit_path.clone()),
            ..Default::default()
        };
        let storage = Arc::new(StorageEngine::open_with_config(":memory:", Some(cfg)).unwrap());
        let db = VantaEmbedded::from_engine(storage.clone());
        let state = Arc::new(ServerState {
            storage,
            db,
            circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
            pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
            api_key: Some("test-key".into()),
            alt_api_key: None,
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
            conversation_trigger: None,
        });
        let addr = spawn_app(app(state, 0)).await;

        // Wrong token + tracing id → 401, auth_l1 err event carries the id.
        let body = r#"{"namespace":"mem","key":"k1","payload":"x"}"#;
        let request = format!(
            "POST /api/v2/records HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Request-Id: abc-123\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let (status, _) = parse_response(&raw_request(addr, request).await);
        assert_eq!(status, 401, "wrong token must be rejected");

        let content = std::fs::read_to_string(&audit_path).unwrap();
        let last = content.lines().last().unwrap();
        assert!(
            last.contains("\"request_id\":\"abc-123\""),
            "audit event must carry the request id, got: {last}"
        );

        // A request WITHOUT the tracing header records events without the
        // field (backwards-compatible JSONL).
        let (status, _) = parse_response(
            &raw_request(addr, json_request("POST", "/api/v2/records", addr, body)).await,
        );
        assert_eq!(status, 401);
        let content = std::fs::read_to_string(&audit_path).unwrap();
        let last = content.lines().last().unwrap();
        assert!(
            !last.contains("request_id"),
            "no request id header → no request_id field, got: {last}"
        );
    }

    /// Build a request with a forged `X-Forwarded-For` header and the given
    /// peer socket address.
    fn request_with_xff(peer: &std::net::SocketAddr, xff: &str) -> axum::extract::Request {
        axum::extract::Request::builder()
            .header("x-forwarded-for", xff)
            .extension(axum::extract::ConnectInfo(*peer))
            .body(axum::body::Body::empty())
            .unwrap()
    }

    #[test]
    fn client_ip_ignores_xff_without_trusted_proxy() {
        // No trusted proxy configured → a forged header must be ignored and the
        // real socket address returned. This is the AUDREP-11 regression guard:
        // a direct client cannot spoof its recorded IP.
        let peer = "198.51.100.5:4444".parse().unwrap();
        let req = request_with_xff(&peer, "203.0.113.99");
        assert_eq!(client_ip(&req, &[]), "198.51.100.5");
    }

    #[test]
    fn client_ip_uses_xff_from_trusted_proxy() {
        // Peer is a configured proxy → the X-Forwarded-For value is used.
        let proxy = "10.0.0.5:4444".parse().unwrap();
        let req = request_with_xff(&proxy, "203.0.113.99");
        assert_eq!(
            client_ip(&req, &["10.0.0.5".parse().unwrap()]),
            "203.0.113.99"
        );
    }

    #[test]
    fn client_ip_uses_first_valid_ip_in_xff() {
        let proxy = "10.0.0.5:4444".parse().unwrap();
        let req = request_with_xff(&proxy, "203.0.113.1, 198.51.100.7");
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
        let req = request_with_xff(&direct, "203.0.113.99");
        assert_eq!(
            client_ip(&req, &["10.0.0.5".parse().unwrap()]),
            "198.51.100.9"
        );
    }

    #[test]
    fn client_ip_simple_remote_addr_no_xff() {
        // Untrusted: x-forwarded-for ignored, socket addr returned.
        let peer = "198.51.100.5:4444".parse().unwrap();
        let req = request_with_xff(&peer, "203.0.113.99");
        assert_eq!(client_ip(&req, &[]), "198.51.100.5");
    }

    #[tokio::test]
    #[cfg(feature = "prometheus")]
    async fn metrics_endpoint_emits_real_prometheus_text() {
        // FND-07: /metrics must expose real, fed metrics (not placeholders).
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        let state = cors_test_state().await;
        let router = app(state, 0);

        // Run one query so the latency histogram and HTTP counters are observed.
        let q = Request::builder()
            .method("POST")
            .uri("/api/v2/query")
            .header("content-type", "application/json")
            .body(Body::from(br#"{"query":"SELECT * FROM Person"}"#.to_vec()))
            .unwrap();
        let resp = router.clone().oneshot(q).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "query must succeed");

        let req = Request::builder()
            .method("GET")
            .uri("/metrics")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK, "/metrics must be reachable");
        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
            .await
            .unwrap();
        let body = String::from_utf8(bytes.to_vec()).unwrap();

        for metric in [
            "vanta_query_latency_ms",
            "vanta_http_requests_total",
            "vanta_http_request_duration_ms",
        ] {
            assert!(
                body.contains(metric),
                "/metrics must expose `{metric}` — got: {body}"
            );
        }
        assert!(
            body.contains("vanta_query_latency_ms_count"),
            "latency histogram must have observations, got: {body}"
        );
    }

    #[tokio::test]
    async fn metrics_v2_returns_json_operational_snapshot() {
        // REST-02: /api/v2/metrics must return the operational snapshot plus
        // per-namespace counts as JSON (not Prometheus text).
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        let state = cors_test_state().await;
        // Seed one memory record so the namespaces map is non-empty.
        state
            .db
            .put(VantaMemoryInput::new("agent/a", "k1", "hello"))
            .unwrap();
        let router = app(state, 0);

        let req = Request::builder()
            .method("GET")
            .uri("/api/v2/metrics")
            .body(Body::empty())
            .unwrap();
        let resp = router.oneshot(req).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "/api/v2/metrics must be reachable"
        );

        let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert!(
            json["metrics"]["hnsw_nodes_count"].is_u64(),
            "metrics.hnsw_nodes_count must be present — got: {json}"
        );
        assert!(
            json["metrics"]["process_rss_bytes"].is_u64(),
            "metrics.process_rss_bytes must be present — got: {json}"
        );
        assert_eq!(
            json["namespaces"]["agent/a"]["count"].as_u64(),
            Some(1),
            "namespaces must include the seeded record — got: {json}"
        );
    }

    #[tokio::test]
    async fn query_error_returns_4xx_not_200() {
        // ERR-027: a failing query must surface as an explicit 4xx/5xx so
        // proxies and monitoring can distinguish client errors from success.
        use axum::body::Body;
        use axum::http::Request;
        use tower::util::ServiceExt;

        let state = cors_test_state().await;
        let router = app(state, 0);

        async fn raw_post(router: axum::Router, body: &[u8]) -> (StatusCode, String) {
            let request = Request::builder()
                .method("POST")
                .uri("/api/v2/query")
                .header("content-type", "application/json")
                .body(Body::from(body.to_vec()))
                .unwrap();
            let response = router.oneshot(request).await.unwrap();
            let status = response.status();
            let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
                .await
                .unwrap();
            (status, String::from_utf8(bytes.to_vec()).unwrap())
        }

        // Unparseable IQL → 400, not 200.
        let (status, body) = raw_post(router.clone(), br#"{"query":"NOT_VALID_IQL"}"#).await;
        assert_eq!(
            status,
            StatusCode::BAD_REQUEST,
            "parse error must be a 4xx, got {status}: {body}"
        );

        // Update of a missing node → NotFound → 404, not 200.
        let (status, body) = raw_post(
            router.clone(),
            br#"{"query":"UPDATE NODE#999 SET name = \"x\""}"#,
        )
        .await;
        assert_eq!(
            status,
            StatusCode::NOT_FOUND,
            "missing node must be a 4xx, got {status}: {body}"
        );

        // A valid read still succeeds with 200.
        let (status, body) = raw_post(router, br#"{"query":"SELECT * FROM Person"}"#).await;
        assert_eq!(
            status,
            StatusCode::OK,
            "healthy query must stay 200, got {status}: {body}"
        );
    }

    #[tokio::test]
    async fn panic_error_response_hides_detail_from_client() {
        // AUDREP-32: a panicked execution task must reach the client as a generic
        // 5xx; the panic detail is only logged server-side by the helper.
        let detail = "execution task panicked: CONTRIVED_PANIC_96942e85";
        let res = panic_error_response(&detail);

        assert_eq!(
            res.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "panicked task must stay a 5xx"
        );

        let bytes = axum::body::to_bytes(res.into_body(), 1 << 20)
            .await
            .expect("response body should be readable");
        let body = String::from_utf8(bytes.to_vec()).expect("body should be utf-8");

        assert!(
            !body.contains("CONTRIVED_PANIC_96942e85"),
            "client-visible body must not leak the panic detail, got: {body}"
        );
        assert!(
            body.contains("Internal server error"),
            "client should see the generic message, got: {body}"
        );
    }

    #[test]
    fn governor_config_always_builds_for_positive_rpm() {
        // AUD-021: the server must never fall open (serve without a rate
        // limit). The eager .expect() in app_with_cors fails closed at
        // startup instead; this test proves the fail path is unreachable for
        // every rpm > 0 because the derived period/burst are always >= 1 —
        // in both the no-auth (full burst) and auth (conservative) branches.
        for rpm in 1..=10_000u32 {
            for auth_active in [false, true] {
                let period_ms = rate_limit_period_ms(rpm);
                let burst_size = rate_limit_burst(rpm, auth_active);
                let cfg = GovernorConfigBuilder::default()
                    .per_millisecond(period_ms)
                    .burst_size(burst_size)
                    .key_extractor(SmartIpKeyExtractor)
                    .finish();
                assert!(
                    cfg.is_some(),
                    "governor config must build for rpm={rpm} auth={auth_active} (period={period_ms}, burst={burst_size})"
                );
            }
        }
    }

    #[tokio::test]
    async fn rate_limiter_allows_ui_burst_without_auth() {
        // REST-01: the web console (grid + inspector + sidebar) fires bursts
        // of ~12 requests. Without auth (dev mode) the burst size equals the
        // full rpm, so a 20-request burst on a loopback client must all pass.
        // rpm=100 would have tripped the old conservative burst of 10.
        let state = cors_test_state().await; // api_key: None -> dev mode
        let addr = spawn_app(app(state, 100)).await;

        for _ in 0..20 {
            let (status, body) = raw_get(addr, "/api/v2/health").await;
            assert_eq!(
                status, 200,
                "burst request must pass the no-auth limiter, got {status}: {body}"
            );
        }
    }

    #[tokio::test]
    async fn rate_limiter_stays_conservative_with_auth() {
        // REST-01 contract: the burst relaxation only applies without auth.
        // With an API key the conservative burst (rpm/10) stays, so a burst
        // beyond it must yield 429 with the JSON error shape and a
        // Retry-After header (AUD-021 fail-closed posture).
        let mut state = cors_test_state().await;
        Arc::get_mut(&mut state).unwrap().api_key = Some("sk-test".into());
        let addr = spawn_app(app(state, 100)).await; // conservative burst = 10

        let mut saw_429 = false;
        for _ in 0..20 {
            let request = format!(
                "GET /api/v2/health HTTP/1.1\r\nHost: {addr}\r\nAuthorization: Bearer sk-test\r\nConnection: close\r\n\r\n"
            );
            let raw = raw_request(addr, request).await;
            if raw.starts_with("HTTP/1.1 429") {
                saw_429 = true;
                assert!(
                    raw.to_lowercase().contains("retry-after:"),
                    "429 must carry Retry-After, got: {raw}"
                );
                let (_, body) = parse_response(&raw);
                let json: serde_json::Value = serde_json::from_str(&body)
                    .unwrap_or_else(|_| panic!("429 body must be JSON, got: {body}"));
                assert_eq!(json["success"], false);
                assert!(
                    json["error"]
                        .as_str()
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains("rate limit"),
                    "error must mention the rate limit, got: {json}"
                );
                break;
            }
            assert_eq!(
                raw.lines()
                    .next()
                    .map(|l| l.split_whitespace().nth(1).unwrap_or(""))
                    .unwrap_or(""),
                "200",
                "requests before the limit trips must pass, got: {raw}"
            );
        }
        assert!(saw_429, "with auth the conservative burst must trip a 429");
    }

    /// Spawn the app over an on-disk fjall DB in a temp dir (needed by
    /// maintenance/snapshot endpoints whose ops are disk-backed).
    async fn disk_test_state() -> (Arc<ServerState>, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let cfg = VantaConfig {
            backend_kind: crate::backend::BackendKind::Fjall,
            ..Default::default()
        };
        let storage = Arc::new(
            StorageEngine::open_with_config(&dir.path().to_string_lossy(), Some(cfg)).unwrap(),
        );
        let db = VantaEmbedded::from_engine(storage.clone());
        (
            Arc::new(ServerState {
                storage,
                db,
                circuit_breaker: Arc::new(CircuitBreaker::new(5, Duration::from_secs(30))),
                pool: Arc::new(ConnectionPool::new(4, Duration::from_millis(100))),
                api_key: None,
                alt_api_key: None,
                rbac_config: RbacConfig::default(),
                trusted_proxies: Vec::new(),
                conversation_trigger: None,
            }),
            dir,
        )
    }

    #[tokio::test]
    async fn v2_export_import_roundtrip() {
        let state = cors_test_state().await;
        let addr = spawn_app(app(state, 0)).await;

        // Seed a record through the REST surface so the export has data.
        let put_body = r#"{"namespace":"mem","key":"k1","payload":"hello export","metadata":{"kind":{"String":"note"}},"vector":null,"ttl_ms":null}"#;
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/records", addr, put_body),
            )
            .await,
        );
        assert_eq!(status, 201, "seed put status: {body}");

        // Export all namespaces to a JSONL file.
        let dir = tempfile::tempdir().unwrap();
        let export_path = dir.path().join("export.jsonl");
        let export_body = format!(
            r#"{{"path":"{}"}}"#,
            export_path.to_string_lossy().replace('\\', "\\\\")
        );
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/export", addr, &export_body),
            )
            .await,
        );
        assert_eq!(status, 200, "export status: {body}");
        let report: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(report["records_exported"], 1, "export report: {report}");
        assert!(export_path.exists(), "export file must exist");

        // Re-import the JSONL file.
        let import_body = format!(
            r#"{{"path":"{}"}}"#,
            export_path.to_string_lossy().replace('\\', "\\\\")
        );
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/import", addr, &import_body),
            )
            .await,
        );
        assert_eq!(status, 200, "file import status: {body}");
        let report: serde_json::Value = serde_json::from_str(&body).unwrap();
        // The record already exists (same key), so the re-import updates it.
        assert_eq!(report["updated"], 1, "import report: {report}");

        // Inline-records import path.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/import", addr, r#"{"records":[]}"#),
            )
            .await,
        );
        assert_eq!(status, 200, "records import status: {body}");

        // Missing both records and path → 400 with the error shape.
        let (status, body) = parse_response(
            &raw_request(addr, json_request("POST", "/api/v2/import", addr, r#"{}"#)).await,
        );
        assert_eq!(status, 400, "import without source must 400: {body}");
        let err: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(err["success"], false);
        assert!(
            err["error"].as_str().unwrap().contains("records"),
            "error should mention the missing source, got: {err}"
        );
    }

    #[tokio::test]
    async fn v2_threads_roundtrip() {
        let state = cors_test_state().await;
        let addr = spawn_app(app(state, 0)).await;

        // Create → 201 with a numeric thread_id.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/threads", addr, r#"{"title":"t1"}"#),
            )
            .await,
        );
        assert_eq!(status, 201, "create status: {body}");
        let created: serde_json::Value = serde_json::from_str(&body).unwrap();
        let thread_id = created["thread_id"].as_str().unwrap().to_string();

        // List → contains the thread.
        let (status, body) = raw_get(addr, "/api/v2/threads").await;
        assert_eq!(status, 200, "list status: {body}");
        let threads: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(threads.as_array().unwrap().len(), 1, "list: {threads}");

        // Send a message → 200.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    &format!("/api/v2/threads/{thread_id}"),
                    addr,
                    r#"{"role":"user","content":"hello"}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 200, "send message status: {body}");

        // Get → thread with 1 message.
        let (status, body) = raw_get(addr, &format!("/api/v2/threads/{thread_id}")).await;
        assert_eq!(status, 200, "get status: {body}");
        let thread: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(thread["title"], "t1", "thread: {thread}");
        assert_eq!(thread["messages"].as_array().unwrap().len(), 1);

        // Delete → 200, then GET → 404.
        let (status, body) = raw_delete(addr, &format!("/api/v2/threads/{thread_id}")).await;
        assert_eq!(status, 200, "delete status: {body}");
        let (status, body) = raw_get(addr, &format!("/api/v2/threads/{thread_id}")).await;
        assert_eq!(status, 404, "get after delete: {body}");
    }

    #[tokio::test]
    async fn v2_graph_roundtrip() {
        let state = cors_test_state().await;
        // Seed a small graph: 1 → 2, 2 → 3.
        state
            .db
            .insert_node(crate::sdk::VantaNodeInput::new(1))
            .unwrap();
        state
            .db
            .insert_node(crate::sdk::VantaNodeInput::new(2))
            .unwrap();
        state
            .db
            .insert_node(crate::sdk::VantaNodeInput::new(3))
            .unwrap();
        state.db.add_edge(1, 2, "next", None, None).unwrap();
        state.db.add_edge(2, 3, "next", None, None).unwrap();
        let addr = spawn_app(app(state, 0)).await;

        // BFS from node 1 reaches node 3.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    "/api/v2/graph/bfs",
                    addr,
                    r#"{"roots":[1],"max_depth":2}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 200, "bfs status: {body}");
        let ids: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            ids.as_array().unwrap().contains(&serde_json::json!(3)),
            "bfs should reach 3, got: {ids}"
        );

        // DFS same shape.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    "/api/v2/graph/dfs",
                    addr,
                    r#"{"roots":[1],"max_depth":2}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 200, "dfs status: {body}");

        // Reverse direction reaches the root's ancestors.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    "/api/v2/graph/bfs",
                    addr,
                    r#"{"roots":[3],"max_depth":2,"direction":"reverse"}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 200, "reverse bfs status: {body}");
        let ids: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            ids.as_array().unwrap().contains(&serde_json::json!(1)),
            "reverse bfs should reach 1, got: {ids}"
        );

        // Degree + centrality (same SDK op).
        for path in ["/api/v2/graph/degree", "/api/v2/graph/centrality"] {
            let (status, body) = parse_response(
                &raw_request(addr, json_request("POST", path, addr, r#"{"roots":[1]}"#)).await,
            );
            assert_eq!(status, 200, "{path} status: {body}");
            let scores: serde_json::Value = serde_json::from_str(&body).unwrap();
            assert!(
                scores.as_object().unwrap().contains_key("1"),
                "{path} should score node 1, got: {scores}"
            );
        }

        // PageRank with defaults.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/graph/pagerank", addr, r#"{"roots":[1]}"#),
            )
            .await,
        );
        assert_eq!(status, 200, "pagerank status: {body}");
        let ranks: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            ranks.as_object().unwrap().contains_key("1"),
            "pagerank should score node 1, got: {ranks}"
        );

        // Invalid direction → axum's Json rejection (422, same as malformed
        // bodies elsewhere on the server).
        let (status, _) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    "/api/v2/graph/bfs",
                    addr,
                    r#"{"roots":[1],"max_depth":1,"direction":"sideways"}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 422, "invalid direction must 422");
    }

    #[tokio::test]
    async fn v2_graph_v2_roundtrip_u128_safe() {
        // REST-03: the graph_v2 endpoints serialize node/edge ids as decimal
        // strings, so ids above u64::MAX survive the JSON wire (the legacy
        // /api/v2/graph/* endpoints return bare u128 values the browser cannot
        // parse — ERR-025 pattern).
        let state = cors_test_state().await;
        let big_id: u128 = 18_446_744_073_709_551_616; // 2^64 > u64::MAX
        state
            .db
            .insert_node(crate::sdk::VantaNodeInput::new(1))
            .unwrap();
        state
            .db
            .insert_node(crate::sdk::VantaNodeInput::new(2))
            .unwrap();
        state
            .db
            .insert_node(crate::sdk::VantaNodeInput::new(big_id))
            .unwrap();
        state.db.add_edge(1, 2, "next", None, None).unwrap();
        state.db.add_edge(1, big_id, "next", None, None).unwrap();
        let addr = spawn_app(app(state.clone(), 0)).await;

        // BFS with string roots reaches the > u64::MAX node; its id must
        // round-trip as a JSON string, never a number.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    "/api/v2/graph/v2/bfs",
                    addr,
                    r#"{"roots":["1"],"max_depth":2}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 200, "graph_v2 bfs status: {body}");
        let result: serde_json::Value = serde_json::from_str(&body).unwrap();
        let big = result["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|n| n["id"].as_str() == Some("18446744073709551616"));
        assert!(big.is_some(), "bfs must reach the u128 node, got: {result}");
        assert_eq!(
            big.unwrap()["id"],
            serde_json::Value::String("18446744073709551616".to_string()),
            "u128 id must be a string on the wire, got: {result}"
        );
        // Edges carry source/target as strings too.
        assert!(
            result["edges"].as_array().unwrap().iter().any(|e| {
                e["source"].as_str() == Some("1")
                    && e["target"].as_str() == Some("18446744073709551616")
            }),
            "edge to the u128 node must serialize string ids, got: {result}"
        );

        // DFS same wire shape.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    "/api/v2/graph/v2/dfs",
                    addr,
                    r#"{"roots":["1"],"max_depth":2}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 200, "graph_v2 dfs status: {body}");
        let result: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(
            result["nodes"]
                .as_array()
                .unwrap()
                .iter()
                .any(|n| n["id"].as_str() == Some("18446744073709551616")),
            "dfs must reach the u128 node, got: {result}"
        );

        // Degree → VantaGraphNodeInfo[] shape: string ids, label, group.
        state
            .db
            .put(VantaMemoryInput::new("mem", "k1", "hello degree"))
            .unwrap();
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    "/api/v2/graph/v2/degree",
                    addr,
                    r#"{"namespace":"mem"}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 200, "graph_v2 degree status: {body}");
        let nodes: serde_json::Value = serde_json::from_str(&body).unwrap();
        let nodes = nodes.as_array().unwrap();
        assert_eq!(nodes.len(), 1, "degree nodes: {nodes:?}");
        assert!(
            nodes[0]["id"].is_string(),
            "degree id must be a string, got: {nodes:?}"
        );
        assert_eq!(nodes[0]["label"], "hello degree", "degree: {nodes:?}");
        assert_eq!(nodes[0]["group"], "mem", "degree: {nodes:?}");
        assert!(
            nodes[0]["degree"].is_u64(),
            "degree must be numeric, got: {nodes:?}"
        );

        // Invalid root string → 400 (VantaError::InvalidInput), not 422.
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request(
                    "POST",
                    "/api/v2/graph/v2/bfs",
                    addr,
                    r#"{"roots":["not-a-number"],"max_depth":1}"#,
                ),
            )
            .await,
        );
        assert_eq!(status, 400, "invalid root must 400: {body}");
        let err: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(err["success"], false);
        assert!(
            err["error"].as_str().unwrap().contains("node id"),
            "error should name the bad id, got: {err}"
        );
    }

    #[tokio::test]
    async fn v2_maintenance_roundtrip() {
        let (state, _dir) = disk_test_state().await;
        let addr = spawn_app(app(state, 0)).await;

        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/maintenance/purge", addr, "{}"),
            )
            .await,
        );
        assert_eq!(status, 200, "purge status: {body}");
        let out: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(out["purged"], 0, "purge output: {out}");

        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/maintenance/flush", addr, "{}"),
            )
            .await,
        );
        assert_eq!(status, 200, "flush status: {body}");
        let out: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(out["flushed"], true, "flush output: {out}");

        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/maintenance/compact", addr, "{}"),
            )
            .await,
        );
        assert_eq!(status, 200, "compact status: {body}");
        let out: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(out["freed_bytes"].is_u64(), "compact output: {out}");

        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/maintenance/rebuild-index", addr, "{}"),
            )
            .await,
        );
        assert_eq!(status, 200, "rebuild-index status: {body}");
        let out: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert!(out["scanned_nodes"].is_u64(), "rebuild output: {out}");
    }

    #[tokio::test]
    async fn v2_snapshots_roundtrip() {
        let (state, _dir) = disk_test_state().await;
        let addr = spawn_app(app(state, 0)).await;

        // Empty list first.
        let (status, body) = raw_get(addr, "/api/v2/snapshots").await;
        assert_eq!(status, 200, "list status: {body}");
        let names: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(names.as_array().unwrap().len(), 0, "snapshots: {names}");

        // Create → 201 with name + path (FsSnapshot is not serializable).
        let (status, body) = parse_response(
            &raw_request(
                addr,
                json_request("POST", "/api/v2/snapshots/snap1", addr, "{}"),
            )
            .await,
        );
        assert_eq!(status, 201, "create snapshot status: {body}");
        let snap: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(snap["name"], "snap1", "snapshot: {snap}");
        assert!(snap["path"].as_str().unwrap().contains("snap1"));

        // List now contains it.
        let (status, body) = raw_get(addr, "/api/v2/snapshots").await;
        assert_eq!(status, 200, "list after create: {body}");
        let names: serde_json::Value = serde_json::from_str(&body).unwrap();
        assert_eq!(names.as_array().unwrap().len(), 1, "snapshots: {names}");
        assert_eq!(names[0], "snap1");
    }

    /// Write a minimal SPA dashboard (index.html + one asset) into `dir`.
    fn write_test_dashboard(dir: &std::path::Path) {
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("index.html"), "<html>vanta-studio-test</html>").unwrap();
        std::fs::write(dir.join("assets/app.js"), "console.log('vanta')").unwrap();
    }

    #[tokio::test]
    async fn dashboard_serves_static_files_and_spa_fallback() {
        let state = cors_test_state().await;
        let dir = std::env::temp_dir().join(format!("vanta-web03-static-{}", std::process::id()));
        write_test_dashboard(&dir);
        let addr = spawn_app(mount_dashboard(app(state, 0), Some(&dir))).await;

        // /dashboard/ → index.html
        let (status, body) = raw_get(addr, "/dashboard/").await;
        assert_eq!(status, 200, "index: {body}");
        assert!(body.contains("vanta-studio-test"), "body: {body}");

        // /dashboard without trailing slash → index.html (ServeDir "/" → index)
        let (status, body) = raw_get(addr, "/dashboard").await;
        assert_eq!(status, 200, "no-slash: {body}");
        assert!(body.contains("vanta-studio-test"), "body: {body}");

        // Deep link without extension → SPA fallback to index.html
        let (status, body) = raw_get(addr, "/dashboard/alguna-ruta-spa").await;
        assert_eq!(status, 200, "spa fallback: {body}");
        assert!(body.contains("vanta-studio-test"), "body: {body}");

        // Real asset → served as-is
        let (status, body) = raw_get(addr, "/dashboard/assets/app.js").await;
        assert_eq!(status, 200, "asset: {body}");
        assert!(body.contains("console.log('vanta')"), "body: {body}");

        // Missing asset WITH extension → real 404, never index.html
        let (status, body) = raw_get(addr, "/dashboard/assets/missing.js").await;
        assert_eq!(status, 404, "missing asset: {body}");
        assert!(
            !body.contains("vanta-studio-test"),
            "missing asset must not return index.html: {body}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn dashboard_disabled_returns_404_hint() {
        let state = cors_test_state().await;
        let addr = spawn_app(mount_dashboard(app(state, 0), None)).await;

        let (status, body) = raw_get(addr, "/dashboard").await;
        assert_eq!(status, 404, "disabled dashboard: {body}");
        assert!(
            body.contains("--dashboard-dir"),
            "hint expected, got: {body}"
        );

        let (status, body) = raw_get(addr, "/dashboard/alguna-ruta").await;
        assert_eq!(status, 404, "disabled dashboard path: {body}");
        assert!(
            body.contains("--dashboard-dir"),
            "hint expected, got: {body}"
        );
    }

    // ── Skills CRUD over HTTP (MEM-54) ─────────────────────────────────────

    use axum::body::Body;
    use axum::http::Request;

    /// Send a JSON request to `router` and return (status, parsed body).
    async fn json_oneshot(
        router: axum::Router,
        method: &str,
        uri: &str,
        body: Option<String>,
    ) -> (StatusCode, serde_json::Value) {
        use tower::util::ServiceExt;
        let builder = Request::builder().method(method).uri(uri);
        let request = match body {
            Some(b) => builder
                .header("content-type", "application/json")
                .body(Body::from(b))
                .unwrap(),
            None => builder.body(Body::empty()).unwrap(),
        };
        let response = router.oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
            .await
            .unwrap();
        let json = if bytes.is_empty() {
            serde_json::Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        };
        (status, json)
    }

    fn skill_body(owner: &str, name: &str, content: &str) -> String {
        serde_json::json!({
            "owner_agent": owner,
            "name": name,
            "description": "test skill",
            "content": content,
        })
        .to_string()
    }

    #[tokio::test]
    async fn skills_crud_roundtrip_via_http() {
        // D19 / MEM-54: create → idempotent re-create → stale-version 409 →
        // patch → update → stale delete 409 → delete → gone.
        let state = cors_test_state().await;
        let router = app(state, 0);

        // CREATE → 201, version 1.
        let (status, json) = json_oneshot(
            router.clone(),
            "POST",
            "/api/v2/skills",
            Some(skill_body("agent-a", "greet", "hello")),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "create failed: {json}");
        assert_eq!(json["idempotent"], false);
        assert_eq!(json["record"]["version"], 1);
        let skill_id = json["record"]["skill_id"].as_str().unwrap().to_string();

        // Re-create identical content → idempotent no-op.
        let (status, json) = json_oneshot(
            router.clone(),
            "POST",
            "/api/v2/skills",
            Some(skill_body("agent-a", "greet", "hello")),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(
            json["idempotent"], true,
            "expected content-hash idempotency"
        );

        let base = format!("/api/v2/skills/{skill_id}");

        // PATCH with stale expected_version → 409 conflict.
        let uri = format!("{base}?owner_agent=agent-a&expected_version=99");
        let (status, _) = json_oneshot(
            router.clone(),
            "PATCH",
            &uri,
            Some(r#"{"description":"patched"}"#.to_string()),
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT, "stale version must be 409");

        // PATCH with correct lock → new head v2, content preserved.
        let (status, json) = json_oneshot(
            router.clone(),
            "PATCH",
            &format!("{base}?owner_agent=agent-a&expected_version=1"),
            Some(r#"{"description":"patched"}"#.to_string()),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "patch failed: {json}");
        assert_eq!(json["record"]["version"], 2);
        assert_eq!(json["record"]["description"], "patched");
        assert_eq!(json["record"]["content"], "hello");

        // UPDATE full replace → v3.
        let (status, json) = json_oneshot(
            router.clone(),
            "PUT",
            &format!("{base}?owner_agent=agent-a&expected_version=2"),
            Some(r#"{"description":"replaced","content":"v2 body"}"#.to_string()),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "update failed: {json}");
        assert_eq!(json["record"]["version"], 3);
        assert_eq!(json["record"]["content"], "v2 body");
        assert_eq!(json["idempotent"], false);

        // DELETE stale → 409; DELETE current → ok and skill disappears.
        let stale = format!("{base}?owner_agent=agent-a&expected_version=1");
        let (status, _) = json_oneshot(router.clone(), "DELETE", &stale, None).await;
        assert_eq!(status, StatusCode::CONFLICT);

        let current = format!("{base}?owner_agent=agent-a&expected_version=3");
        let (status, json) = json_oneshot(router.clone(), "DELETE", &current, None).await;
        assert_eq!(status, StatusCode::OK, "delete failed: {json}");
        assert_eq!(json["deleted"], true);

        let (status, _) = json_oneshot(router, "DELETE", &current, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "deleted skill must be gone");
    }

    #[tokio::test]
    async fn skills_owner_mismatch_is_indistinguishable_from_missing() {
        // Anti-enumeration: a foreign-owned skill must return the same 404 as
        // a nonexistent one — never 403 (which would confirm existence).
        let state = cors_test_state().await;
        let router = app(state, 0);

        let (_, json) = json_oneshot(
            router.clone(),
            "POST",
            "/api/v2/skills",
            Some(skill_body("agent-owner", "secret-skill", "private")),
        )
        .await;
        let skill_id = json["record"]["skill_id"].as_str().unwrap().to_string();

        for (method, body) in [
            (
                "PUT",
                Some(r#"{"description":"x","content":"y"}"#.to_string()),
            ),
            ("PATCH", Some(r#"{"description":"x"}"#.to_string())),
            ("DELETE", None),
        ] {
            let foreign =
                format!("/api/v2/skills/{skill_id}?owner_agent=agent-attacker&expected_version=1");
            let (status, _) = json_oneshot(router.clone(), method, &foreign, body.clone()).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{method} foreign owner");

            let missing =
                "/api/v2/skills/skl-nonexistent?owner_agent=agent-attacker&expected_version=1";
            let (missing_status, _) =
                json_oneshot(router.clone(), method, missing, body.clone()).await;
            assert_eq!(
                missing_status, status,
                "{method}: missing vs foreign-owned must be indistinguishable"
            );
        }

        // The real owner can still operate the skill.
        let owned = format!("/api/v2/skills/{skill_id}?owner_agent=agent-owner&expected_version=1");
        let (status, _) = json_oneshot(
            router,
            "PATCH",
            &owned,
            Some(r#"{"description":"mine"}"#.to_string()),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "real owner must pass: {status}");
    }
}

#[cfg(test)]
#[path = "cli_server_auth_tests.rs"]
mod auth_tests;
