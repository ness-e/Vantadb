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
    State(state): State<Arc<ServerState>>,
    Json(payload): Json<QueryRequest>,
) -> Json<QueryResponse> {
    use crate::parser::parse_statement;
    use crate::executor::{Executor, ExecutionResult};

    match parse_statement(&payload.query) {
        Ok((_, statement)) => {
            let executor = Executor::new(&state.storage);
            match executor.execute_statement(statement) {
                Ok(ExecutionResult::Read(nodes)) => {
                    Json(QueryResponse {
                        success: true,
                        data: format!("Read {} nodes.", nodes.len()),
                    })
                }
                Ok(ExecutionResult::Write { affected_nodes, message }) => {
                    Json(QueryResponse {
                        success: true,
                        data: format!("Mutated {} nodes: {}", affected_nodes, message),
                    })
                }
                Err(e) => {
                    Json(QueryResponse {
                        success: false,
                        data: format!("Execution Error: {}", e),
                    })
                }
            }
        }
        Err(e) => {
            Json(QueryResponse {
                success: false,
                data: format!("Parse Error: {}", e),
            })
        }
    }
}
