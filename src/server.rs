use axum::{
    routing::{get, post},
    Router, Json, extract::State,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::storage::StorageEngine;

#[derive(Serialize, Deserialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: String, // Simplified response message
}

pub struct ServerState {
    pub storage: Arc<StorageEngine>,
}

pub fn app(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/query", post(execute_query))
        .with_state(state)
}

async fn health_check() -> Json<QueryResponse> {
    Json(QueryResponse {
        success: true,
        data: "OK".to_string(),
    })
}

async fn execute_query(
    State(_state): State<Arc<ServerState>>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    // Scaffold: Normally parses payload.query string into LogicalPlan, then Executor.
    // For now we map success for the ping.
    Json(QueryResponse {
        success: true,
        data: format!("Executed: {}", payload.query),
    })
}
