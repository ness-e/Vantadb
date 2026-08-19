//! HTTP server startup and route wiring for VantaDB's CLI server.
//!
//! Builds an [`axum`] application, mounts middleware and API routes,
//! and binds to the configured address.
//!
//! ponytail: 928L but all pieces (handlers, middleware, telemetry, server startup)
//! flow through `run()` → `app()` → Router. Telemetry is verbose (tracing-subscriber
//! config) but not complex — not worth splitting.

use crate::audit::AuditEvent;
use crate::circuit_breaker::CircuitBreaker;
use crate::connection_pool::{ConnectionPool, PoolError};
use crate::error::ChainedError;
use crate::sdk::{
    VantaEmbedded, VantaMemoryFilter, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryRecord,
    VantaMemorySearchRequest,
};
use crate::VantaError;
use lru::LruCache;
use std::collections::HashMap;
use std::sync::Arc;
#[cfg(feature = "opentelemetry")]
use std::sync::OnceLock;
use std::time::Duration;
use std::time::Instant;

use axum::{
    extract::{DefaultBodyLimit, Path as AxumPath, Query, State},
    http::{header, HeaderValue, Method, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tokio::net::TcpListener;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorLayer,
};
use tracing_subscriber::EnvFilter;
#[cfg(feature = "opentelemetry")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Registry};
#[cfg(feature = "opentelemetry")]
static OTEL_PROVIDER: OnceLock<opentelemetry_sdk::trace::SdkTracerProvider> = OnceLock::new();

use crate::config::{LogFormat, RbacConfig, VantaConfig};
use crate::console;
use crate::error::Result;
use crate::metrics;
use crate::node::{FieldValue, UnifiedNode};
use crate::rbac::{Permission, Rbac};
use crate::storage::StorageEngine;

/// JSON body for a query endpoint request.
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    /// The VantaQL query string to execute.
    pub query: String,
}

/// Response envelope for the query endpoint.
#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    /// Whether the request succeeded.
    pub success: bool,
    /// Human-readable result message.
    pub data: String,
    /// Single node ID returned by write or stale-context results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<u128>,
    /// Collection of nodes returned by read results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<NodeDTO>>,
}

/// Data-transfer object for a UnifiedNode returned over HTTP.
#[derive(Serialize, Deserialize)]
pub struct NodeDTO {
    /// Unique node identifier.
    pub id: u128,
    /// Semantic cluster the node belongs to.
    pub semantic_cluster: u32,
    /// Relational key-value payload.
    pub relational: std::collections::BTreeMap<String, FieldValue>,
    /// Access / hit count tracked by the storage engine.
    pub hits: u32,
    /// Internal confidence score for staleness detection.
    pub confidence_score: f32,
}

impl From<&UnifiedNode> for NodeDTO {
    fn from(n: &UnifiedNode) -> Self {
        Self {
            id: n.id,
            semantic_cluster: n.semantic_cluster,
            relational: n.relational.clone(),
            hits: n.hits,
            confidence_score: n.confidence_score,
        }
    }
}

/// Shared application state injected into every route handler.
pub struct ServerState {
    /// The underlying storage engine.
    pub storage: Arc<StorageEngine>,
    /// Embedded SDK handle sharing the storage engine — source of operations
    /// for the `/api/v2` record endpoints. Shared (not per-request) so the
    /// audit logger has a single file handle with its own mutex; per-request
    /// handles would interleave appends to the audit JSONL.
    pub db: VantaEmbedded,
    /// Circuit breaker for fast-failing when the backend is failing.
    pub circuit_breaker: Arc<CircuitBreaker>,
    /// Connection pool bounding concurrent query execution.
    pub pool: Arc<ConnectionPool>,
    /// Optional bearer token for API authentication.
    pub api_key: Option<Arc<str>>,
    /// RBAC token-to-role mapping configuration.
    pub rbac_config: RbacConfig,
    /// Reverse-proxy IPs whose `X-Forwarded-For` header is honored for client
    /// IP resolution. Empty = ignore the header (ConnectInfo is authoritative).
    pub trusted_proxies: Vec<std::net::IpAddr>,
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
        state.rbac_config.clone(),
        rbac,
        &state.trusted_proxies,
    );

    let public = Router::new().route("/health", get(health_check));

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
        .route("/api/v2/export", post(export_v2))
        .route("/api/v2/import", post(import_v2))
        .route("/api/v2/graph/bfs", post(graph_bfs))
        .route("/api/v2/graph/dfs", post(graph_dfs))
        .route("/api/v2/graph/degree", post(graph_degree))
        .route("/api/v2/graph/centrality", post(graph_centrality))
        .route("/api/v2/graph/pagerank", post(graph_pagerank))
        .route("/api/v2/maintenance/purge", post(maintenance_purge))
        .route("/api/v2/maintenance/compact", post(maintenance_compact))
        .route("/api/v2/maintenance/flush", post(maintenance_flush))
        .route(
            "/api/v2/maintenance/rebuild-index",
            post(maintenance_rebuild_index),
        )
        .route("/api/v2/threads", get(threads_list).post(threads_create))
        .route(
            "/api/v2/threads/{id}",
            get(threads_get)
                .post(threads_send_message)
                .delete(threads_delete),
        )
        .route("/api/v2/snapshots", get(snapshots_list))
        .route("/api/v2/snapshots/{name}", post(snapshots_create))
        .route("/metrics", get(metrics_endpoint))
        .layer(middleware::from_fn(auth_middleware));

    let protected = if rpm > 0 {
        let period_ms = (60_000u64 / rpm as u64).max(1);
        let burst_size = (rpm / 10).max(1);

        // AUD-021: fail-closed. Should the governor config ever fail to build,
        // refuse to start rather than serving requests without a rate limit.
        // (The previous fall branch left `protected` unthrottled — fail-open.)
        let gc = GovernorConfigBuilder::default()
            .per_millisecond(period_ms)
            .burst_size(burst_size)
            .key_extractor(SmartIpKeyExtractor)
            .finish()
            .expect("governor config must build: period_ms and burst_size are >= 1 for rpm > 0");
        protected.layer(GovernorLayer::new(gc))
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

/// Per-IP rate limiter for authentication failures.
pub struct AuthRateLimiter {
    /// Per-IP map of (failure count, first-failure instant). LRU evicts oldest.
    failures: Mutex<LruCache<String, (u32, Instant)>>,
    /// Maximum failed attempts before rate-limiting kicks in.
    max_attempts: u32,
    /// Time window in seconds.
    window_secs: u64,
}

impl AuthRateLimiter {
    /// Create a new rate limiter with the given attempt cap and window.
    pub fn new(max_attempts: u32, window_secs: u64) -> Self {
        Self {
            failures: Mutex::new(LruCache::new(std::num::NonZero::new(1000).unwrap())),
            max_attempts,
            window_secs,
        }
    }

    /// Returns `true` if the given IP has exceeded the allowed failure rate.
    pub fn is_rate_limited(&self, ip: &str) -> bool {
        let mut failures = self.failures.lock();
        let now = Instant::now();
        if let Some((count, first)) = failures.get(ip) {
            if now.duration_since(*first).as_secs() > self.window_secs {
                failures.pop(ip);
                return false;
            }
            *count >= self.max_attempts
        } else {
            false
        }
    }

    /// Record an authentication failure for the given IP.
    pub fn record_failure(&self, ip: &str) {
        let mut failures = self.failures.lock();
        let now = Instant::now();
        let (count, first) = failures.get(ip).map(|&(c, f)| (c, f)).unwrap_or((0, now));
        if now.duration_since(first).as_secs() > self.window_secs {
            failures.put(ip.to_string(), (1, now));
        } else {
            failures.put(ip.to_string(), (count + 1, first));
        }
    }

    /// Clear the failure count for the given IP.
    pub fn reset(&self, ip: &str) {
        self.failures.lock().pop(ip);
    }
}

/// Authentication and authorization state shared via middleware extensions.
#[derive(Clone)]
pub struct AuthState {
    /// Optional bearer token for API key validation.
    pub api_key: Option<Arc<str>>,
    pub(crate) token_role_map: HashMap<String, String>,
    pub(crate) rbac: Arc<Rbac>,
    pub(crate) rate_limiter: Arc<AuthRateLimiter>,
    /// Reverse-proxy IPs whose `X-Forwarded-For` header is honored for client IP
    /// resolution. Empty = the header is ignored.
    pub(crate) trusted_proxies: Vec<std::net::IpAddr>,
}

impl AuthState {
    pub(crate) fn new(
        api_key: Option<String>,
        rbac_config: RbacConfig,
        rbac: Arc<Rbac>,
        trusted_proxies: &[std::net::IpAddr],
    ) -> Self {
        Self {
            api_key: api_key.map(|k| Arc::from(k.as_str())),
            token_role_map: rbac_config.token_role_map,
            rbac,
            rate_limiter: Arc::new(AuthRateLimiter::new(5, 60)),
            trusted_proxies: trusted_proxies.to_vec(),
        }
    }
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
pub async fn auth_middleware(req: axum::extract::Request, next: middleware::Next) -> Response {
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

    let authorized = match token {
        Some(token) => {
            let token_bytes = token.as_bytes();
            let expected_bytes = expected_key.as_bytes();
            token_bytes.ct_eq(expected_bytes).into()
        }
        None => false,
    };

    if authorized {
        // Check RBAC permissions
        if let Some(token_val) = token {
            if let Some(role) = auth.token_role_map.get(token_val) {
                let is_write = matches!(req.method().as_str(), "POST" | "PUT" | "PATCH" | "DELETE");
                let permission = if is_write {
                    Permission::Write
                } else {
                    Permission::Read
                };
                if !auth.rbac.has_permission(role, &permission) {
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
        auth.rate_limiter.reset(&client_ip);
        next.run(req).await
    } else {
        auth.rate_limiter.record_failure(&client_ip);
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Unauthorized",
                "hint": "Provide a valid Bearer token in the Authorization header."
            })),
        )
            .into_response()
    }
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

/// Axum middleware that records HTTP request duration and status metrics.
pub async fn request_metrics_middleware(
    req: axum::extract::Request,
    next: middleware::Next,
) -> Response {
    let start = std::time::Instant::now();
    let method = req.method().to_string();
    let route = req.uri().path().to_string();
    let res = next.run(req).await;
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
    // La consola web lista sin namespace (HomeOverview/sidebar/grid) — el bridge
    // nativo (desktop/src-tauri/connections/native.rs) defaulta vacío a "default".
    // Alinear el wire REST para que el modo embebido se comporte igual que Tauri.
    let ns = match params.namespace.as_deref() {
        Some(n) if !n.trim().is_empty() => n.to_string(),
        _ => "default".to_string(),
    };
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
    let options = VantaMemoryListOptions {
        filter_ops,
        limit: params.limit.unwrap_or(100),
        cursor: params.cursor,
        ..Default::default()
    };
    match run_db_op(&state, move |db| db.list(&ns, options)).await {
        Ok(page) => Json(page).into_response(),
        Err(resp) => resp,
    }
}

#[tracing::instrument(skip(state))]
async fn records_search(
    State(state): State<Arc<ServerState>>,
    Json(request): Json<VantaMemorySearchRequest>,
) -> Response {
    // La Topbar de la consola web busca sin namespace — mismo default que list.
    let mut request = request;
    if request.namespace.trim().is_empty() {
        request.namespace = "default".to_string();
    }
    match run_db_op(&state, move |db| db.search(request)).await {
        Ok(hits) => Json(hits).into_response(),
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

/// Validate that the auth configuration is consistent.
///
/// Returns an error if `require_auth` is `true` but no `api_key` is configured.
fn validate_auth_config(config: &VantaConfig) -> Result<()> {
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
        rbac_config,
        trusted_proxies: config.trusted_proxies.clone(),
    });

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
    fn validate_auth_allows_no_key_without_require() {
        let cfg = VantaConfig {
            api_key: None,
            require_auth: false,
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
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
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
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
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
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
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
            rbac_config: RbacConfig::default(),
            trusted_proxies: Vec::new(),
        });
        let addr = spawn_app(app(state, 0)).await;

        // GET/DELETE of a missing record → 404 with the error shape.
        let (status, body) = raw_get(addr, "/api/v2/records/mem/missing").await;
        assert_eq!(status, 404, "get missing: {body}");
        assert!(body.contains("\"success\":false"), "error shape: {body}");
        let (status, body) = raw_delete(addr, "/api/v2/records/mem/missing").await;
        assert_eq!(status, 404, "delete missing: {body}");

        // LIST without namespace → 200 (defaults to "default", igual que el bridge
        // nativo native.rs — la consola web lista sin namespace).
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
        // every rpm > 0 because the derived period/burst are always >= 1.
        for rpm in 1..=10_000u32 {
            let period_ms = (60_000u64 / rpm as u64).max(1);
            let burst_size = (rpm / 10).max(1);
            let cfg = GovernorConfigBuilder::default()
                .per_millisecond(period_ms)
                .burst_size(burst_size)
                .key_extractor(SmartIpKeyExtractor)
                .finish();
            assert!(
                cfg.is_some(),
                "governor config must build for rpm={rpm} (period={period_ms}, burst={burst_size})"
            );
        }
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
                rbac_config: RbacConfig::default(),
                trusted_proxies: Vec::new(),
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
}
