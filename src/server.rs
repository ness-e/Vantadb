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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes: Option<Vec<NodeDTO>>,
}

#[derive(Serialize, Deserialize)]
pub struct NodeDTO {
    pub id: u64,
    pub semantic_cluster: u32,
    pub relational: std::collections::BTreeMap<String, crate::node::FieldValue>,
    pub hits: u32,
    pub trust_score: f32,
}

impl From<&crate::node::UnifiedNode> for NodeDTO {
    fn from(n: &crate::node::UnifiedNode) -> Self {
        Self {
            id: n.id,
            semantic_cluster: n.semantic_cluster,
            relational: n.relational.clone(),
            hits: n.hits,
            trust_score: n.trust_score,
        }
    }
}

pub struct ServerState {
    pub storage: Arc<StorageEngine>,
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
    use crate::executor::{Executor, ExecutionResult};

    let executor = Executor::new(&state.storage);
    match executor.execute_hybrid(&payload.query).await {
        Ok(ExecutionResult::Read(nodes)) => {
            let dtos = nodes.iter().map(NodeDTO::from).collect();
            Json(QueryResponse {
                success: true,
                data: format!("Read {} nodes.", nodes.len()),
                node_id: None,
                nodes: Some(dtos),
            })
        }
        Ok(ExecutionResult::Write { affected_nodes, message, node_id }) => {
            Json(QueryResponse {
                success: true,
                data: format!("Mutated {} nodes: {}", affected_nodes, message),
                node_id,
                nodes: None,
            })
        }
        Ok(ExecutionResult::StaleContext(summary_id)) => {
            Json(QueryResponse {
                success: true,
                data: format!("STALE_CONTEXT: TrustScore critical. Rehydration available for summary {}", summary_id),
                node_id: Some(summary_id),
                nodes: None,
            })
        }
        Err(e) => {
            Json(QueryResponse {
                success: false,
                data: format!("Execution Error: {}", e),
                node_id: None,
                nodes: None,
            })
        }
    }
}
