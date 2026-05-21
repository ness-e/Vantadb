//! Optional local server wrapper.
//!
//! The server wraps the embedded core for local HTTP access. It is not the primary v0.1.x product
//! boundary and must not redefine behavior independently from the embedded engine.

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use vantadb::storage::StorageEngine;

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

pub struct ServerState {
    pub storage: Arc<StorageEngine>,
    pub semaphore: Arc<tokio::sync::Semaphore>,
}

pub fn app(state: Arc<ServerState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/v2/query", post(execute_query)) // Upgraded to v2 to reflect NodeDTO changes
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

