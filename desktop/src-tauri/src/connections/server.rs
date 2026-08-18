//! `ServerConnection` — adapts the typed IQL client ([`ServerClient`]) to the
//! multi-connection contract ([`VantaConnection`]).
//!
//! The client owns all transport concerns (URL building, Bearer auth, envelope
//! validation, `success:false` → `Http{Domain}` mapping). This adapter only
//! maps contract DTOs (`IngestItem`/`SearchQuery`/…) to the client's
//! IQL-oriented signatures, enforces a per-op timeout, and reports
//! `Capability::Http`.
//!
//! Auth / health are validated in [`connect`](VantaConnection::connect) by
//! calling the server's `/health` (no auth) — a 401 on a later op is surfaced
//! as `VantaError::Http { kind: Unauthorized }` by the client, and a dead
//! server (connection refused) surfaces as `Http { kind: Other }` — never a
//! panic.

use std::future::Future;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;

use super::types::{
    Capability, ConnectionInfo, ConnectionStatus, HealthReport, HealthStatus, IngestItem,
    MemoryRecord, SearchQuery, SearchResult,
};
use super::wire_types::{HealthReport as WireHealthReport, NodeDTO};
use super::{ServerClient, ServerClientConfig, VantaConnection};
use crate::error::{HttpErrorKind, VantaError};

/// Adapter over the VantaDB HTTP server via the typed IQL client.
pub struct ServerConnection {
    client: ServerClient,
    cfg: ServerClientConfig,
    /// Lifecycle state reported by [`VantaConnection::info`]; set by connect/disconnect.
    connected: bool,
    /// Local counter used to assign ids when the caller supplies none.
    next_id_counter: u128,
}

impl ServerConnection {
    /// Build a connection from configuration.
    pub fn with(cfg: ServerClientConfig) -> Result<Self, VantaError> {
        let client = ServerClient::new(cfg.clone())?;
        Ok(Self {
            client,
            cfg,
            connected: false,
            next_id_counter: 0,
        })
    }

    /// Run an operation under the configured timeout, mapping elapsed time to
    /// [`VantaError::Timeout`]. The client additionally carries a reqwest
    /// timeout as a backstop.
    async fn timeout_ops<T>(
        &self,
        fut: impl Future<Output = Result<T, VantaError>>,
    ) -> Result<T, VantaError> {
        match tokio::time::timeout(self.cfg.timeout, fut).await {
            Ok(r) => r,
            Err(_) => Err(VantaError::Timeout(format!(
                "operation timed out after {:?}",
                self.cfg.timeout
            ))),
        }
    }

    fn next_id(&mut self) -> u128 {
        self.next_id_counter += 1;
        self.next_id_counter
    }

    fn node_to_record(&self, node: NodeDTO, namespace: Option<&str>) -> MemoryRecord {
        MemoryRecord {
            id: node.id.to_string(),
            namespace: namespace.unwrap_or("default").to_string(),
            text: relational_str(&node.relational, "content"),
            vector: None,
            metadata: node.relational.into_iter().collect(),
            created_at_ms: None,
            updated_at_ms: None,
            version: None,
            node_id: Some(node.id.to_string()),
            sparse_vector: None,
            expires_at_ms: None,
        }
    }
}

/// Read a text field from a node's `relational` map, tolerating both the flat
/// string shape the mock returns (`"content": "x"`) and the typed-shape the real
/// server returns (`"content": {"String": "x"}` / `{"Number": n}` / …).
fn relational_str(relational: &serde_json::Map<String, serde_json::Value>, key: &str) -> String {
    let Some(v) = relational.get(key) else {
        return String::new();
    };
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(obj) => {
            // Typed FieldValue: exactly one variant key ("String"/"Number"/"Bool"/…).
            if let Some(s) = obj.get("String").and_then(|s| s.as_str()) {
                s.to_string()
            } else if let Some(n) = obj.get("Number") {
                n.to_string()
            } else if let Some(b) = obj.get("Bool") {
                b.to_string()
            } else if let Some(arr) = obj.get("List") {
                serde_json::to_string(arr).unwrap_or_default()
            } else {
                serde_json::to_string(&obj).unwrap_or_default()
            }
        }
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[async_trait]
impl VantaConnection for ServerConnection {
    fn info(&self) -> ConnectionInfo {
        ConnectionInfo {
            id: format!("server-{}", self.cfg.base_url()),
            name: format!("VantaDB Server ({})", self.cfg.base_url()),
            via: Capability::Http,
            status: if self.connected {
                ConnectionStatus::Connected
            } else {
                ConnectionStatus::Disconnected
            },
            description: Some("HTTP connection to vantadb-server".into()),
        }
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::Http]
    }

    async fn connect(&mut self) -> Result<(), VantaError> {
        let report = self.client.health().await?;
        if report.ok {
            self.connected = true;
            Ok(())
        } else {
            self.connected = false;
            Err(VantaError::Http {
                kind: HttpErrorKind::Domain,
                message: report.data,
                status: Some(200),
            })
        }
    }

    async fn disconnect(&mut self) -> Result<(), VantaError> {
        self.connected = false;
        Ok(())
    }

    async fn ingest(&mut self, item: IngestItem) -> Result<String, VantaError> {
        let id = match &item.id {
            Some(s) => s.parse::<u128>().map_err(|_| {
                VantaError::Other(format!(
                    "server connection requires numeric node ids, got {s:?}"
                ))
            })?,
            None => self.next_id(),
        };

        // `content` is the text field the IQL search (`WHERE content ~ ...`) reads.
        let mut fields: Vec<(String, String)> =
            vec![("content".to_string(), item.text.clone())];
        for (k, v) in item.metadata.iter() {
            fields.push((k.clone(), serde_json::to_string(v).map_err(VantaError::from)?));
        }
        let refs: Vec<(&str, &str)> = fields
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        let resp = self
            .timeout_ops(self.client.put(id, &item.namespace, &refs))
            .await?;
        Ok(resp
            .node_id
            .map(|n| n.to_string())
            .unwrap_or_else(|| id.to_string()))
    }

    async fn ingest_batch(&mut self, items: Vec<IngestItem>) -> Result<Vec<String>, VantaError> {
        let mut ids = Vec::with_capacity(items.len());
        for item in items {
            ids.push(self.ingest(item).await?);
        }
        Ok(ids)
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, VantaError> {
        let kind = query.namespace.as_deref().unwrap_or("default");
        let resp = self
            .timeout_ops(self.client.search(kind, "content", &query.query, 0.0))
            .await?;
        let namespace = query.namespace.unwrap_or_else(|| "default".to_string());
        Ok(resp
            .nodes
            .unwrap_or_default()
            .into_iter()
            .map(|n| SearchResult {
                id: n.id.to_string(),
                namespace: namespace.clone(),
                text: relational_str(&n.relational, "content"),
                score: n.confidence_score,
                metadata: n.relational.into_iter().collect(),
            })
            .collect())
    }

    async fn get(&self, id: &str, namespace: Option<&str>) -> Result<MemoryRecord, VantaError> {
        let nid = id
            .parse::<u128>()
            .map_err(|_| VantaError::Other(format!("server connection requires numeric ids, got {id:?}")))?;
        let resp = self.timeout_ops(self.client.get(nid)).await?;
        let node = resp
            .nodes
            .unwrap_or_default()
            .into_iter()
            .next()
            .ok_or_else(|| VantaError::Http {
                kind: HttpErrorKind::NotFound,
                message: format!("node {id} not found"),
                status: Some(404),
            })?;
        Ok(self.node_to_record(node, namespace))
    }

    async fn delete(&mut self, id: &str, _namespace: Option<&str>) -> Result<(), VantaError> {
        let nid = id
            .parse::<u128>()
            .map_err(|_| VantaError::Other(format!("server connection requires numeric ids, got {id:?}")))?;
        self.timeout_ops(self.client.delete(nid)).await?;
        Ok(())
    }

    async fn list(&self, namespace: Option<&str>, limit: usize) -> Result<Vec<MemoryRecord>, VantaError> {
        let kind = namespace.unwrap_or("default");
        let resp = self.timeout_ops(self.client.list(kind)).await?;
        Ok(resp
            .nodes
            .unwrap_or_default()
            .into_iter()
            .take(limit)
            .map(|n| self.node_to_record(n, namespace))
            .collect())
    }

    async fn health(&self) -> Result<HealthReport, VantaError> {
        let start = Instant::now();
        let report: WireHealthReport = self.client.health().await?;
Ok(HealthReport {
            status: if report.ok {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            },
            backend: "http".to_string(),
            latency_ms: start.elapsed().as_millis() as u64,
            checked_at_ms: now_ms(),
            message: Some(report.data),
        })
    }
}
