//! HTTP router building, CORS, and dashboard mounting.
//!
//! REVIEW-10: extracted from `routing.rs` — pure router construction without
//! middleware or handler logic. This module owns the route topology:
//! public endpoints, protected endpoints, long-running endpoints, CORS, and
//! the optional SPA dashboard.

use crate::server::state::{AuthState, ServerState, LONG_REQUEST_TIMEOUT, REQUEST_TIMEOUT};
use axum::{
    extract::DefaultBodyLimit,
    http::{header, HeaderMap, HeaderValue, Method},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post, put},
    Extension, Json, Router,
};
use std::sync::Arc;
use tower_governor::{
    governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor, GovernorError,
    GovernorLayer,
};
use tower_http::cors::CorsLayer;
use tracing;

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
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            headers.unwrap_or_default(),
            format!("Rate limit exceeded; retry after {wait_time}s"),
        ),
        GovernorError::UnableToExtractKey => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
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

/// Build a [`tower_http::cors::CorsLayer`] allowing the given origins.
///
/// Returns `None` (no CORS middleware) when no valid origin is configured.
/// Invalid/blank origins are skipped and the rest kept.
fn cors_layer(allowed_origins: &[String]) -> Option<CorsLayer> {
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
        CorsLayer::new()
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
    use crate::rbac::{Permission, Rbac};

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

    let public = Router::new().route("/health", get(super::handlers::health_check));

    // Interactive routes: cap at REQUEST_TIMEOUT so a stuck handler can't hold
    // the connection indefinitely (DoS protection).
    let protected = Router::new()
        .route("/api/v2/query", post(super::handlers::execute_query))
        .route("/api/v2/health", get(super::handlers::health_v2))
        .route(
            "/api/v2/records",
            post(super::handlers::records_put).delete(super::handlers::records_delete_by_filter),
        )
        .route(
            "/api/v2/records/batch",
            post(super::handlers::records_put_batch),
        )
        .route(
            "/api/v2/records/{ns}/{key}",
            get(super::handlers::records_get).delete(super::handlers::records_delete),
        )
        .route(
            "/api/v2/records/{ns}/{key}/versions",
            get(super::handlers::records_versions),
        )
        .route("/api/v2/list", get(super::handlers::records_list))
        .route("/api/v2/search", post(super::handlers::records_search))
        .route(
            "/api/v2/autocomplete",
            get(super::handlers::iql_autocomplete),
        )
        .route("/api/v2/audit", get(super::handlers::audit_events))
        .route("/api/v2/graph/bfs", post(super::handlers::graph_bfs))
        .route("/api/v2/graph/dfs", post(super::handlers::graph_dfs))
        .route("/api/v2/graph/degree", post(super::handlers::graph_degree))
        .route(
            "/api/v2/graph/centrality",
            post(super::handlers::graph_centrality),
        )
        .route(
            "/api/v2/graph/pagerank",
            post(super::handlers::graph_pagerank),
        )
        .route("/api/v2/graph/v2/bfs", post(super::handlers::graph_v2_bfs))
        .route("/api/v2/graph/v2/dfs", post(super::handlers::graph_v2_dfs))
        .route(
            "/api/v2/graph/v2/degree",
            post(super::handlers::graph_v2_degree),
        )
        .route(
            "/api/v2/maintenance/purge",
            post(super::handlers::maintenance_purge),
        )
        .route(
            "/api/v2/maintenance/compact",
            post(super::handlers::maintenance_compact),
        )
        .route(
            "/api/v2/maintenance/flush",
            post(super::handlers::maintenance_flush),
        )
        .route(
            "/api/v2/threads",
            get(super::handlers::threads_list).post(super::handlers::threads_create),
        )
        .route(
            "/api/v2/threads/{id}",
            get(super::handlers::threads_get)
                .post(super::handlers::threads_send_message)
                .delete(super::handlers::threads_delete),
        )
        .route("/conversation/add", post(super::handlers::conversation_add))
        .route("/skill/listing", get(super::handlers::skill_listing))
        .route("/api/v2/skills", post(super::handlers::skill_create))
        .route(
            "/api/v2/skills/{skill_id}",
            put(super::handlers::skill_update)
                .patch(super::handlers::skill_patch)
                .delete(super::handlers::skill_delete),
        )
        .route("/api/v2/snapshots", get(super::handlers::snapshots_list))
        .route(
            "/api/v2/snapshots/{name}",
            post(super::handlers::snapshots_create),
        )
        .route("/metrics", get(super::handlers::metrics_endpoint))
        .route("/api/v2/metrics", get(super::handlers::metrics_v2))
        .layer(middleware::from_fn(super::middleware::auth_middleware))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            REQUEST_TIMEOUT,
        ));

    // Long-running bulk/maintenance routes: a generous ceiling (rather than the
    // interactive timeout above) so legitimate large imports/exports/index
    // rebuilds aren't killed mid-flight. Still bounded against a wedged op.
    let long_running = Router::new()
        .route("/api/v2/export", post(super::handlers::export_v2))
        .route("/api/v2/import", post(super::handlers::import_v2))
        .route(
            "/api/v2/maintenance/rebuild-index",
            post(super::handlers::maintenance_rebuild_index),
        )
        .layer(middleware::from_fn(super::middleware::auth_middleware))
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
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
            super::middleware::circuit_breaker_middleware,
        ))
        .layer(middleware::from_fn(
            super::middleware::request_metrics_middleware,
        ))
        .layer(Extension(auth_state))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
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
pub fn mount_dashboard(router: Router, dir: Option<&std::path::Path>) -> Router {
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
                            (axum::http::StatusCode::NOT_FOUND, "Not found").into_response(),
                        );
                    }
                    let body = match tokio::fs::read(&index).await {
                        Ok(bytes) => ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], bytes)
                            .into_response(),
                        Err(_) => (
                            axum::http::StatusCode::NOT_FOUND,
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
            .route("/dashboard", get(super::handlers::dashboard_disabled))
            .route(
                "/dashboard/{*path}",
                get(super::handlers::dashboard_disabled),
            ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limit_period_ms_is_sane() {
        assert_eq!(rate_limit_period_ms(1), 60_000);
        assert_eq!(rate_limit_period_ms(60), 1_000);
        assert_eq!(rate_limit_period_ms(60_000), 1);
        assert_eq!(rate_limit_period_ms(100_000), 1); // floor
    }

    #[test]
    fn rate_limit_burst_dev_mode_full() {
        // No auth → full rpm as burst
        assert_eq!(rate_limit_burst(100, false), 100);
        assert_eq!(rate_limit_burst(10, false), 10);
        assert_eq!(rate_limit_burst(1, false), 1);
    }

    #[test]
    fn rate_limit_burst_auth_mode_conservative() {
        // With auth → rpm/10, floor 1
        assert_eq!(rate_limit_burst(100, true), 10);
        assert_eq!(rate_limit_burst(50, true), 5);
        assert_eq!(rate_limit_burst(9, true), 1); // floor
        assert_eq!(rate_limit_burst(1, true), 1);
    }

    #[test]
    fn cors_layer_none_when_empty() {
        assert!(cors_layer(&[]).is_none());
        assert!(cors_layer(&["".to_string()]).is_none());
    }

    #[test]
    fn cors_layer_some_when_valid() {
        let layer = cors_layer(&["http://localhost:3000".to_string()]);
        assert!(layer.is_some());
    }

    #[test]
    fn cors_layer_skips_invalid_origin() {
        let layer = cors_layer(&["not-a-url".to_string(), "http://valid.com".to_string()]);
        assert!(layer.is_some());
    }
}
