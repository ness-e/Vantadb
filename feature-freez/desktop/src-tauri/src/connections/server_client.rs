//! Typed HTTP client for the VantaDB server.
//!
//! The real server exposes exactly 4 endpoints:
//!   - `GET  /health`       (no auth)
//!   - `GET  /metrics`      (Bearer, if api_key configured)
//!   - `GET  /api/v2/metrics` (Bearer) — JSON metrics + per-namespace stats.
//!   - `POST /api/v2/query` (Bearer) — executes an IQL statement.
//!
//! There is NO REST per-operation API. put/get/delete/list/search are IQL
//! statements sent as the `{"query": "..."}` body of `/api/v2/query`. This
//! wrapper maps the semantic ops to IQL statements, authenticates them, and
//! treats a `success:false` envelope as a *domain* error (not transport).

use std::collections::BTreeMap;

use reqwest::header::CONTENT_TYPE;

use crate::connections::types::NamespaceStats;
use crate::connections::wire_types::{
    HealthReport, QueryRequest, QueryResponse, ServerClientConfig,
};
use crate::error::VantaError;

/// Wire envelope of `GET /api/v2/metrics` (REST-02): we only read the
/// per-namespace stats; the `metrics` half is ignored.
#[derive(serde::Deserialize)]
struct MetricsV2Envelope {
    #[serde(default)]
    namespaces: BTreeMap<String, NamespaceStats>,
}

/// Typed client over the VantaDB HTTP server.
#[derive(Clone)]
pub struct ServerClient {
    inner: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

impl ServerClient {
    /// Build a client from configuration.
    pub fn new(cfg: ServerClientConfig) -> Result<Self, VantaError> {
        let inner = reqwest::Client::builder()
            .timeout(cfg.timeout)
            .connect_timeout(cfg.timeout)
            .build()
            .map_err(|e| VantaError::Http {
                kind: crate::error::HttpErrorKind::Other,
                message: format!("failed to build reqwest client: {e}"),
                status: None,
            })?;
        Ok(Self {
            inner,
            base_url: cfg.base_url(),
            token: cfg.token,
        })
    }

    /// `GET /health` — liveness check, no auth.
    pub async fn health(&self) -> Result<HealthReport, VantaError> {
        let resp = self
            .inner
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map_err(transport)?;
        let status = resp.status();
        let body: QueryResponse = resp.json().await.map_err(|e| VantaError::Http {
            kind: crate::error::HttpErrorKind::Other,
            message: format!("invalid /health response: {e}"),
            status: Some(status.as_u16()),
        })?;
        check_success(status, &body)?;
        Ok(HealthReport {
            ok: body.success,
            data: body.data,
        })
    }

    /// `GET /metrics` — Prometheus text, Bearer. Returns raw text.
    pub async fn metrics(&self) -> Result<String, VantaError> {
        let resp = self
            .inner
            .get(format!("{}/metrics", self.base_url))
            .bearer_auth(self.token.as_deref().unwrap_or(""))
            .send()
            .await
            .map_err(transport)?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(VantaError::from_http_status(status.as_u16(), text));
        }
        resp.text().await.map_err(transport)
    }

    /// `GET /api/v2/metrics` — JSON metrics + per-namespace stats (REST-02),
    /// Bearer. Returns the `namespaces` map (`count`/`expiring_soon`/`expired`
    /// per namespace); the operational `metrics` half is not part of the
    /// connection contract.
    pub async fn namespace_stats(&self) -> Result<BTreeMap<String, NamespaceStats>, VantaError> {
        let resp = self
            .inner
            .get(format!("{}/api/v2/metrics", self.base_url))
            .bearer_auth(self.token.as_deref().unwrap_or(""))
            .send()
            .await
            .map_err(transport)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(VantaError::from_http_status(status.as_u16(), body));
        }
        let envelope: MetricsV2Envelope = resp.json().await.map_err(|e| VantaError::Http {
            kind: crate::error::HttpErrorKind::Other,
            message: format!("invalid /api/v2/metrics response: {e}"),
            status: Some(status.as_u16()),
        })?;
        Ok(envelope.namespaces)
    }

    /// Execute a raw IQL statement via `POST /api/v2/query`, Bearer auth.
    ///
    /// `success:false` in the body is mapped to `VantaError::Http` with kind
    /// `Domain` — the server returns HTTP 200 for domain statement failures.
    pub async fn query(&self, statement: &str) -> Result<QueryResponse, VantaError> {
        let resp = self
            .inner
            .post(format!("{}/api/v2/query", self.base_url))
            .header(CONTENT_TYPE, "application/json")
            .bearer_auth(self.token.as_deref().unwrap_or(""))
            .json(&QueryRequest {
                query: statement.to_string(),
            })
            .send()
            .await
            .map_err(transport)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(VantaError::from_http_status(status.as_u16(), body));
        }
        let body: QueryResponse = resp.json().await.map_err(|e| VantaError::Http {
            kind: crate::error::HttpErrorKind::Other,
            message: format!("invalid /api/v2/query response: {e}"),
            status: Some(status.as_u16()),
        })?;
        check_success(status, &body)?;
        Ok(body)
    }

    // ── Semantic ops mapped to IQL statements ──────────────────────────

    /// Write/update a node. `INSERT` for a fresh id, `UPDATE` otherwise.
    ///
    /// statement: `INSERT NODE#<id> TYPE <kind> {k: "v", ...}`
    pub async fn put(
        &self,
        node_id: u128,
        kind: &str,
        fields: &[(&str, &str)],
    ) -> Result<QueryResponse, VantaError> {
        let mut stmt = format!("INSERT NODE#{node_id} TYPE {kind} {{");
        for (i, (k, v)) in fields.iter().enumerate() {
            if i > 0 {
                stmt.push_str(", ");
            }
            stmt.push_str(&format!("{k}: {:?}", v));
        }
        stmt.push('}');
        self.query(&stmt).await
    }

    /// Read a single node by id.
    ///
    /// statement: `MATCH NODE#<id>`
    pub async fn get(&self, node_id: u128) -> Result<QueryResponse, VantaError> {
        self.query(&format!("MATCH NODE#{node_id}")).await
    }

    /// Delete a node by id.
    ///
    /// statement: `DELETE NODE#<id>`
    pub async fn delete(&self, node_id: u128) -> Result<QueryResponse, VantaError> {
        self.query(&format!("DELETE NODE#{node_id}")).await
    }

    /// List nodes of a kind.
    ///
    /// statement: `FROM <kind>`
    pub async fn list(&self, kind: &str) -> Result<QueryResponse, VantaError> {
        self.query(&format!("FROM {kind}")).await
    }

    /// Search nodes of a kind by text similarity on a field.
    ///
    /// statement: `FROM <kind> WHERE <field> ~ "<text>" min = <threshold>`
    pub async fn search(
        &self,
        kind: &str,
        field: &str,
        text: &str,
        min: f32,
    ) -> Result<QueryResponse, VantaError> {
        self.query(&format!(
            "FROM {kind} WHERE {field} ~ {:?} min = {min}",
            text
        ))
        .await
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

fn transport(e: reqwest::Error) -> VantaError {
    VantaError::Http {
        kind: crate::error::HttpErrorKind::Other,
        message: format!("network error: {e}"),
        status: e.status().map(|s| s.as_u16()),
    }
}

/// Validate envelope: HTTP status AND `body.success`. A `false` body.success
/// is a domain failure, surfaced as a typed `VantaError::Http` (Domain).
fn check_success(status: reqwest::StatusCode, body: &QueryResponse) -> Result<(), VantaError> {
    if !status.is_success() {
        return Err(VantaError::from_http_status(status.as_u16(), &body.data));
    }
    if body.success {
        Ok(())
    } else {
        Err(VantaError::from_http_status(200, &body.data))
    }
}
