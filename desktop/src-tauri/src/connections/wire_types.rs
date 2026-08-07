//! Wire DTOs for the typed HTTP server client (server response shapes).
//!
//! > Relocated here by DESKTOP-04 (canonical owner of `connections/types.rs`, which now
//! > holds the multi-connection contract DTOs). These types were originally written by
//! > DESKTOP-08 inline in `types.rs`; preserved verbatim so its `server_client` work is
//! > not lost. Use as `crate::connections::wire_types::*`.

use serde::{Deserialize, Serialize};

/// Configuration for a connection to the VantaDB HTTP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerClientConfig {
    /// Host (without scheme/port), e.g. `127.0.0.1`.
    pub url: String,
    /// Port, e.g. `8080`.
    pub port: u16,
    /// Optional API key sent as `Authorization: Bearer <token>`.
    /// `None` = dev mode (server allows unauthenticated requests).
    pub token: Option<String>,
    /// Per-request timeout.
    #[serde(default = "default_timeout")]
    pub timeout: std::time::Duration,
}

impl Default for ServerClientConfig {
    fn default() -> Self {
        Self {
            url: "127.0.0.1".to_string(),
            port: 8080,
            token: None,
            timeout: default_timeout(),
        }
    }
}

fn default_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(10)
}

impl ServerClientConfig {
    /// Full base URL, e.g. `http://127.0.0.1:8080`.
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.url, self.port)
    }
}

/// Wire request body for `POST /api/v2/query` (matches server `QueryRequest`).
#[derive(Debug, Clone, Serialize)]
pub struct QueryRequest {
    pub query: String,
}

/// Wire response envelope for `/api/v2/query` and `/health`
/// (matches server `QueryResponse`).
#[derive(Debug, Clone, Deserialize)]
pub struct QueryResponse {
    pub success: bool,
    pub data: String,
    #[serde(default)]
    pub node_id: Option<u128>,
    #[serde(default)]
    pub nodes: Option<Vec<NodeDTO>>,
}

/// Data-transfer object for a UnifiedNode returned over HTTP
/// (matches server `NodeDTO`).
#[derive(Debug, Clone, Deserialize)]
pub struct NodeDTO {
    pub id: u128,
    #[serde(default)]
    pub semantic_cluster: u32,
    #[serde(default)]
    pub relational: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    pub hits: u64,
    #[serde(default)]
    pub confidence_score: f32,
}

/// Parsed health result (server shape).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HealthReport {
    pub ok: bool,
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_default_base_url() {
        let cfg = ServerClientConfig::default();
        assert_eq!(cfg.base_url(), "http://127.0.0.1:8080");
    }

    #[test]
    fn query_response_roundtrip_read() {
        let json = r#"{
            "success": true,
            "data": "Read 1 nodes.",
            "node_id": null,
            "nodes": [{"id": 1, "semantic_cluster": 0, "relational": {"key": "k"}, "hits": 3, "confidence_score": 0.95}]
        }"#;
        let resp: QueryResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        let nodes = resp.nodes.unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, 1);
        assert_eq!(nodes[0].relational["key"], "k");
    }

    #[test]
    fn query_response_roundtrip_write() {
        let json = r#"{"success": true, "data": "Mutated 1 nodes: inserted", "node_id": 42}"#;
        let resp: QueryResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
        assert_eq!(resp.node_id, Some(42));
        assert!(resp.nodes.is_none());
    }

    #[test]
    fn query_response_domain_failure() {
        let json = r#"{"success": false, "data": "Execution Error: node not found"}"#;
        let resp: QueryResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.success);
    }
}