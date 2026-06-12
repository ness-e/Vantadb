//! Optional local server wrapper.
//!
//! The server wraps the embedded core for local HTTP access. It is not the primary v0.1.x product
//! boundary and must not redefine behavior independently from the embedded engine.

use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use vantadb::storage::StorageEngine;

use crate::middleware::{auth_middleware, AuthState};

#[derive(Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<NodeDTO>>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeDTO {
    pub id: u64,
    pub semantic_cluster: u32,
    pub relational: std::collections::BTreeMap<String, vantadb::node::FieldValue>,
    pub hits: u32,
    pub confidence_score: f32,
}

impl From<&vantadb::node::UnifiedNode> for NodeDTO {
    fn from(n: &vantadb::node::UnifiedNode) -> Self {
        Self {
            id: n.id,
            semantic_cluster: n.semantic_cluster,
            relational: n.relational.clone(),
            hits: n.hits,
            confidence_score: n.confidence_score,
        }
    }
}

/// Shared server state injected into all route handlers.
pub struct ServerState {
    pub storage: Arc<StorageEngine>,
    pub semaphore: Arc<tokio::sync::Semaphore>,
    /// Optional API key for Bearer token authentication.
    ///
    /// Mirrors `VantaConfig::api_key`. `None` means the server runs without
    /// authentication (development / embedded-local mode).
    pub api_key: Option<Arc<str>>,
}

/// Builds the Axum router with the full security middleware stack.
///
/// # Security stack applied to `/api/v2/query` (outermost → innermost)
/// 1. `GovernorLayer` — per-IP rate limiting (only when `rpm > 0`)
/// 2. `auth_middleware` — Bearer token validation (no-op when `api_key` is `None`)
/// 3. Route handler
///
/// `/health` is always accessible without auth or rate-limit restrictions.
pub fn app(state: Arc<ServerState>, rpm: u32) -> Router {
    let auth_state = AuthState::new(state.api_key.as_ref().map(|k| k.to_string()));

    // ── Public route: exempt from auth and rate-limit ────────────────────────
    let public = Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_endpoint));

    // ── Protected route: auth middleware applied ─────────────────────────────
    // Auth middleware wraps the query route. The /health exemption is handled
    // inside auth_middleware itself, but keeping it on a separate sub-router
    // makes the intent explicit and avoids any accidental layer bleed-through.
    let protected = Router::new()
        .route("/api/v2/query", post(execute_query))
        .layer(middleware::from_fn(auth_middleware));

    // ── Apply rate limiting when configured ──────────────────────────────────
    //
    // GovernorConfigBuilder uses a `&mut self` builder pattern. The period is
    // expressed as "seconds between token replenishments". For RPM-based config:
    //   period_ms = 60_000 / rpm  → tokens replenish at rpm/min rate
    //   burst_size = rpm/10       → allows short bursts (min 1)
    //
    // Both branches produce an `axum::Router`, so Rust resolves the types
    // correctly without needing `Either` or dynamic dispatch.
    let protected = if rpm > 0 {
        let period_ms = (60_000u64 / rpm as u64).max(1);
        let burst_size = (rpm / 10).max(1);

        let governor_config = GovernorConfigBuilder::default()
            .per_millisecond(period_ms)
            .burst_size(burst_size)
            .finish()
            .expect("GovernorConfig build failed: rpm must be > 0 and period > 0");

        protected.layer(GovernorLayer::new(governor_config))
    } else {
        // rpm == 0 → rate limiting disabled (tests, embedded-local usage)
        protected
    };

    // ── Merge routes and attach shared state ────────────────────────────────
    Router::new()
        .merge(public)
        .merge(protected)
        .layer(Extension(auth_state))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

async fn health_check() -> Json<QueryResponse> {
    Json(QueryResponse {
        success: true,
        data: "OK".to_string(),
        node_id: None,
        nodes: None,
    })
}

/// Endpoint /metrics para exportar métricas Prometheus (TSK-06)
///
/// Retorna métricas en formato Prometheus text format para scraping por
/// Prometheus u otros sistemas de monitoreo. Este endpoint es público
/// y no requiere autenticación.
async fn metrics_endpoint() -> impl IntoResponse {
    let metrics_text = vantadb::metrics::export_metrics_text();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "text/plain; version=0.0.4")
        .body(metrics_text)
        .unwrap()
}

async fn execute_query(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    use vantadb::executor::{ExecutionResult, Executor};

    let _permit = match state.semaphore.clone().acquire_owned().await {
        Ok(p) => p,
        Err(_) => {
            return Json(QueryResponse {
                success: false,
                data: "Server concurrency semaphore closed".to_string(),
                node_id: None,
                nodes: None,
            });
        }
    };

    let storage = state.storage.clone();
    let query = payload.query.clone();

    let join_res = tokio::task::spawn_blocking(move || {
        let executor = Executor::new(&storage);
        executor.execute_hybrid(&query)
    })
    .await;

    let execution_result = match join_res {
        Ok(r) => r,
        Err(e) => {
            return Json(QueryResponse {
                success: false,
                data: format!("Internal server error: execution task panicked: {}", e),
                node_id: None,
                nodes: None,
            });
        }
    };

    match execution_result {
        Ok(ExecutionResult::Read(nodes)) => {
            let dtos = nodes.iter().map(NodeDTO::from).collect();
            Json(QueryResponse {
                success: true,
                data: format!("Read {} nodes.", nodes.len()),
                node_id: None,
                nodes: Some(dtos),
            })
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
        }),
        Ok(ExecutionResult::StaleContext(summary_id)) => Json(QueryResponse {
            success: true,
            data: format!(
                "STALE_CONTEXT: Confidence Score critical. Rehydration available for summary {}",
                summary_id
            ),
            node_id: Some(summary_id),
            nodes: None,
        }),
        Err(e) => Json(QueryResponse {
            success: false,
            data: format!("Execution Error: {}", e),
            node_id: None,
            nodes: None,
        }),
    }
}
