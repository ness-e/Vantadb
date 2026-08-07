use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Transport / backend a connection speaks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    Native,
    Http,
    Mcp,
    Node,
    Python,
    Wasm,
}

/// Lifecycle status of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Error,
}

/// Reported health of a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// A single unit to ingest into the store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IngestItem {
    /// Optional explicit id; if absent the backend assigns one.
    #[serde(default)]
    pub id: Option<String>,
    /// Namespace/collection the record belongs to.
    #[serde(default = "default_namespace")]
    pub namespace: String,
    /// The text content.
    pub text: String,
    /// Optional precomputed embedding. Use finite floats — JSON has no NaN/Inf.
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
    /// Arbitrary record metadata (`serde_json::Value` so any JSON-able value roundtrips).
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Input for [`crate::connections::VantaConnection::search`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Semantic / natural-language query text.
    pub query: String,
    /// Optional explicit query vector (takes precedence over `query` text when present).
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
    /// Maximum number of results to return.
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    /// Restrict search to a single namespace.
    #[serde(default)]
    pub namespace: Option<String>,
    /// Metadata filters applied post-search.
    #[serde(default)]
    pub filters: HashMap<String, serde_json::Value>,
}

/// A single hit returned by [`crate::connections::VantaConnection::search`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub namespace: String,
    pub text: String,
    /// Relevance score. Higher is better, semantics are backend-defined.
    pub score: f32,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// A stored memory record returned by `get` / `list`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: String,
    pub namespace: String,
    pub text: String,
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
    /// Creation time as unix milliseconds.
    #[serde(default)]
    pub created_at_ms: Option<u64>,
}

/// Result of [`crate::connections::VantaConnection::health`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    /// Backend engine the report describes (e.g. `"fjall"` for native embedded).
    #[serde(default = "default_backend")]
    pub backend: String,
    /// Round-trip latency in milliseconds.
    pub latency_ms: u64,
    /// Time the check was performed, unix milliseconds.
    pub checked_at_ms: u64,
    /// Optional human-readable diagnostic detail.
    #[serde(default)]
    pub message: Option<String>,
}

/// Static metadata describing a connection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub name: String,
    /// Transport bridge the connection uses.
    pub via: Capability,
    pub status: ConnectionStatus,
    #[serde(default)]
    pub description: Option<String>,
}

fn default_namespace() -> String {
    "default".to_string()
}

fn default_top_k() -> usize {
    10
}

fn default_backend() -> String {
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use std::fmt::Debug;

    /// JSON roundtrip: serialize → deserialize must yield an equal value.
    fn rt<T: Serialize + DeserializeOwned + PartialEq + Debug>(v: &T) -> T {
        let json = serde_json::to_string(v).expect("serialize");
        serde_json::from_str(&json).expect("deserialize")
    }

    fn json<T: Serialize>(v: &T) -> String {
        serde_json::to_string(v).expect("serialize")
    }

    #[test]
    fn capability_roundtrips() {
        for cap in [Capability::Native, Capability::Http, Capability::Mcp, Capability::Node, Capability::Python, Capability::Wasm] {
            assert_eq!(rt(&cap), cap);
        }
        assert_eq!(json(&Capability::Http), r#""http""#);
    }

    #[test]
    fn connection_status_roundtrips() {
        for s in [ConnectionStatus::Connected, ConnectionStatus::Disconnected, ConnectionStatus::Error] {
            assert_eq!(rt(&s), s);
        }
    }

    #[test]
    fn health_status_roundtrips() {
        for s in [HealthStatus::Healthy, HealthStatus::Degraded, HealthStatus::Unhealthy] {
            assert_eq!(rt(&s), s);
        }
    }

    #[test]
    fn ingest_item_roundtrip() {
        let item = IngestItem {
            id: Some("k1".into()),
            namespace: "mem".into(),
            text: "hello world".into(),
            embedding: Some(vec![0.5, -1.25, 3.0]),
            metadata: [("lang".to_string(), serde_json::Value::from("en"))].into_iter().collect(),
        };
        assert_eq!(rt(&item), item);
    }

    #[test]
    fn ingest_item_defaults_deserialize_absent_fields() {
        // Fields with `#[serde(default)]` must deserialize when absent in JSON.
        let json = r#"{"text":"hi"}"#;
        let item: IngestItem = serde_json::from_str(json).expect("deserialize");
        assert_eq!(item.namespace, "default");
        assert!(item.id.is_none());
        assert!(item.embedding.is_none());
        assert!(item.metadata.is_empty());
    }

    #[test]
    fn search_query_roundtrip() {
        let q = SearchQuery {
            query: "cats".into(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
            top_k: 5,
            namespace: Some("mem".into()),
            filters: [("scope".into(), serde_json::Value::from("test"))].into_iter().collect(),
        };
        assert_eq!(rt(&q), q);
    }

    #[test]
    fn search_result_roundtrip() {
        let r = SearchResult {
            id: "r1".into(),
            namespace: "mem".into(),
            text: "cats".into(),
            score: 0.987,
            metadata: [("k".to_string(), serde_json::Value::from(42))].into_iter().collect(),
        };
        assert_eq!(rt(&r), r);
    }

    #[test]
    fn memory_record_roundtrip() {
        let rec = MemoryRecord {
            id: "r1".into(),
            namespace: "mem".into(),
            text: "cats".into(),
            embedding: None,
            metadata: HashMap::new(),
            created_at_ms: Some(1_700_000_000_000),
        };
        assert_eq!(rt(&rec), rec);
    }

    #[test]
    fn health_report_roundtrip() {
        let h = HealthReport {
            status: HealthStatus::Healthy,
            backend: "fjall".into(),
            latency_ms: 12,
            checked_at_ms: 1_700_000_000_000,
            message: Some("ok".into()),
        };
        assert_eq!(rt(&h), h);
    }

    #[test]
    fn connection_info_roundtrip() {
        let c = ConnectionInfo {
            id: "c1".into(),
            name: "local".into(),
            via: Capability::Native,
            status: ConnectionStatus::Connected,
            description: Some("embedded".into()),
        };
        assert_eq!(rt(&c), c);
    }
}