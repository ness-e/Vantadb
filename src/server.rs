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
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<u64>,
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
        node_id: None,
    })
}

async fn execute_query(
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    use crate::executor::{Executor, ExecutionResult};

    let executor = Executor::new(&state.storage);
    match executor.execute_hybrid(&payload.query).await {
        Ok(ExecutionResult::Read(nodes)) => {
            Json(QueryResponse {
                success: true,
                data: format!("Read {} nodes.", nodes.len()),
                node_id: None,
            })
        }
        Ok(ExecutionResult::Write { affected_nodes, message, node_id }) => {
            Json(QueryResponse {
                success: true,
                data: format!("Mutated {} nodes: {}", affected_nodes, message),
                node_id,
            })
        }
        Ok(ExecutionResult::StaleContext(summary_id)) => {
            Json(QueryResponse {
                success: true,
                data: format!("STALE_CONTEXT: TrustScore critical. Rehydration available for summary {}", summary_id),
                node_id: Some(summary_id),
            })
        }
        Err(e) => {
            Json(QueryResponse {
                success: false,
                data: format!("Execution Error: {}", e),
                node_id: None,
            })
        }
    }
}
