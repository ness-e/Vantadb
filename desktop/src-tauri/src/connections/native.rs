//! Native embedded adapter (`NativeConnection`) for the desktop multi-connection
//! contract (DESKTOP-05).
//!
//! Wraps [`vantadb::VantaEmbedded`] and implements [`VantaConnection`]. Every
//! synchronous SDK call runs on the blocking thread pool via
//! `tokio::task::spawn_blocking` so the async trait never blocks the runtime.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use serde_json::Value as JsonValue;
use vantadb::VantaError as CoreVantaError;
use vantadb::{
    VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions, VantaMemoryRecord,
    VantaMemorySearchHit, VantaMemorySearchRequest, VantaValue,
};

use super::types::{
    Capability, ConnectionInfo, ConnectionStatus, HealthReport, HealthStatus, IngestItem,
    MemoryRecord, SearchQuery, SearchResult,
};
use super::VantaConnection;
use crate::error::VantaError;

/// Namespace used when the contract leaves it unspecified.
const DEFAULT_NAMESPACE: &str = "default";

/// Monotonic counter for synthesizing ids when the caller omits one.
static ID_SEQ: AtomicU64 = AtomicU64::new(0);

/// A `VantaConnection` backed by an embedded `VantaEmbedded` handle.
pub struct NativeConnection {
    id: String,
    path: PathBuf,
    db: VantaEmbedded,
}

impl NativeConnection {
    /// Open (or create) the embedded database at `path`.
    ///
    /// Fails with [`VantaError::Lock`] when another connection already holds an
    /// exclusive writer lock on the same path (the core reports `DatabaseBusy`).
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, VantaError> {
        let path = path.into();
        let db = VantaEmbedded::open(&path).map_err(map_core_error)?;
        let id = format!("native:{}", path.display());
        Ok(Self { id, path, db })
    }
}

/// Translate a core `vantadb::VantaError` into the desktop `VantaError`.
fn map_core_error(e: CoreVantaError) -> VantaError {
    match e {
        // The core reports a locked database as `DatabaseBusy` — surface it as
        // the contract's `Lock` so callers can branch on it.
        CoreVantaError::DatabaseBusy(msg) => VantaError::Lock(msg),
        other => VantaError::Native(other.to_string()),
    }
}

/// Generate a process-unique record id when the caller supplies none.
fn gen_id() -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let seq = ID_SEQ.fetch_add(1, Ordering::Relaxed);
    format!("rec_{now_ms}_{seq}")
}

/// Run a synchronous SDK call on the blocking pool and map join errors.
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, VantaError> + Send + 'static,
) -> Result<T, VantaError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| VantaError::Native(format!("blocking task failed: {e}")))?
}

/// Convert desktop `serde_json::Value` metadata into `VantaValue`.
fn to_vanta_value(v: JsonValue) -> VantaValue {
    match v {
        JsonValue::Null => VantaValue::Null,
        JsonValue::Bool(b) => VantaValue::Bool(b),
        JsonValue::Number(n) => n
            .as_i64()
            .map(VantaValue::Int)
            .or_else(|| n.as_f64().map(VantaValue::Float))
            .unwrap_or(VantaValue::Null),
        JsonValue::String(s) => VantaValue::String(s),
        JsonValue::Array(items) => {
            if items.iter().all(JsonValue::is_string) {
                VantaValue::ListString(items.iter().map(|i| i.as_str().unwrap().to_string()).collect())
            } else if items.iter().all(JsonValue::is_boolean) {
                VantaValue::ListBool(items.iter().map(|i| i.as_bool().unwrap()).collect())
            } else if items.iter().all(JsonValue::is_i64) {
                VantaValue::ListInt(items.iter().map(|i| i.as_i64().unwrap()).collect())
            } else if items.iter().all(JsonValue::is_number) {
                VantaValue::ListFloat(
                    items.iter().map(|i| i.as_f64().unwrap_or_default()).collect(),
                )
            } else {
                VantaValue::String(serde_json::to_string(&items).unwrap_or_default())
            }
        }
        JsonValue::Object(_) => VantaValue::String(serde_json::to_string(&v).unwrap_or_default()),
    }
}

/// Convert a desktop metadata map into the core's `BTreeMap<String, VantaValue>`.
fn metadata_to_vanta(
    metadata: &std::collections::HashMap<String, JsonValue>,
) -> BTreeMap<String, VantaValue> {
    metadata
        .iter()
        .map(|(k, v)| (k.clone(), to_vanta_value(v.clone())))
        .collect()
}

/// Convert a `VantaValue` back into `serde_json::Value`.
fn from_vanta_value(v: &VantaValue) -> JsonValue {
    match v {
        VantaValue::String(s) => JsonValue::String(s.clone()),
        VantaValue::Int(i) => JsonValue::from(*i),
        VantaValue::Float(f) => JsonValue::from(*f),
        VantaValue::Bool(b) => JsonValue::from(*b),
        VantaValue::Null => JsonValue::Null,
        VantaValue::DateTime(dt) => JsonValue::String(dt.to_rfc3339()),
        VantaValue::ListString(items) => JsonValue::from(items.clone()),
        VantaValue::ListInt(items) => JsonValue::from(items.clone()),
        VantaValue::ListFloat(items) => JsonValue::from(items.clone()),
        VantaValue::ListBool(items) => JsonValue::from(items.clone()),
        VantaValue::ListDateTime(items) => {
            JsonValue::from(items.iter().map(|d| d.to_rfc3339()).collect::<Vec<_>>())
        }
    }
}

fn ingest_to_input(item: &IngestItem, key: &str) -> VantaMemoryInput {
    let mut input = VantaMemoryInput::new(item.namespace.clone(), key, item.text.clone());
    input.metadata = metadata_to_vanta(&item.metadata);
    input.vector = item.embedding.clone();
    input
}

fn record_to_memory(r: VantaMemoryRecord) -> MemoryRecord {
    MemoryRecord {
        id: r.key,
        namespace: r.namespace,
        text: r.payload,
        embedding: r.vector,
        metadata: r
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), from_vanta_value(v)))
            .collect(),
        created_at_ms: Some(r.created_at_ms),
    }
}

fn hit_to_result(h: VantaMemorySearchHit) -> SearchResult {
    SearchResult {
        id: h.record.key,
        namespace: h.record.namespace,
        text: h.record.payload,
        score: h.score,
        metadata: h
            .record
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), from_vanta_value(v)))
            .collect(),
    }
}

fn search_request(q: &SearchQuery) -> VantaMemorySearchRequest {
    let text_query = {
        let t = q.query.trim();
        if t.is_empty() {
            None
        } else {
            Some(t.to_string())
        }
    };
    VantaMemorySearchRequest {
        namespace: q.namespace.clone().unwrap_or_else(|| DEFAULT_NAMESPACE.into()),
        query_vector: q.embedding.clone().unwrap_or_default(),
        filters: metadata_to_vanta(&q.filters),
        text_query,
        top_k: q.top_k,
        ..Default::default()
    }
}

#[async_trait]
impl VantaConnection for NativeConnection {
    fn info(&self) -> ConnectionInfo {
        ConnectionInfo {
            id: self.id.clone(),
            name: format!("native:{}", self.path.display()),
            via: Capability::Native,
            status: ConnectionStatus::Connected,
            description: Some("embedded fjall backend".to_string()),
        }
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::Native]
    }

    async fn connect(&mut self) -> Result<(), VantaError> {
        // The constructor already acquires the database handle.
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), VantaError> {
        let db = self.db.clone();
        blocking(move || db.close().map_err(map_core_error)).await
    }

    async fn ingest(&mut self, item: IngestItem) -> Result<String, VantaError> {
        let key = item.id.clone().unwrap_or_else(gen_id);
        let input = ingest_to_input(&item, &key);
        let db = self.db.clone();
        blocking(move || db.put(input).map(|_| key).map_err(map_core_error)).await
    }

    async fn ingest_batch(&mut self, items: Vec<IngestItem>) -> Result<Vec<String>, VantaError> {
        if items.is_empty() {
            return Ok(Vec::new());
        }
        let keys: Vec<String> = items
            .iter()
            .map(|it| it.id.clone().unwrap_or_else(gen_id))
            .collect();
        let inputs: Vec<VantaMemoryInput> = items
            .iter()
            .zip(&keys)
            .map(|(it, key)| ingest_to_input(it, key))
            .collect();
        let db = self.db.clone();
        blocking(move || db.put_batch(inputs).map(|_| keys).map_err(map_core_error)).await
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, VantaError> {
        let request = search_request(&query);
        let db = self.db.clone();
        blocking(move || {
            db.search(request)
                .map(|hits| hits.into_iter().map(hit_to_result).collect())
                .map_err(map_core_error)
        })
        .await
    }

    async fn get(&self, id: &str, namespace: Option<&str>) -> Result<MemoryRecord, VantaError> {
        let ns = namespace.unwrap_or(DEFAULT_NAMESPACE).to_string();
        let key = id.to_string();
        let not_found = format!("record not found: {ns}/{key}");
        let db = self.db.clone();
        let record = blocking(move || db.get(&ns, &key).map_err(map_core_error)).await?;
        record.map(record_to_memory).ok_or_else(|| VantaError::Native(not_found))
    }

    async fn delete(&mut self, id: &str, namespace: Option<&str>) -> Result<(), VantaError> {
        let ns = namespace.unwrap_or(DEFAULT_NAMESPACE).to_string();
        let key = id.to_string();
        let db = self.db.clone();
        blocking(move || db.delete(&ns, &key).map(|_| ()).map_err(map_core_error)).await
    }

    async fn list(
        &self,
        namespace: Option<&str>,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, VantaError> {
        let ns = namespace.unwrap_or(DEFAULT_NAMESPACE).to_string();
        let options = VantaMemoryListOptions {
            limit,
            ..Default::default()
        };
        let db = self.db.clone();
        blocking(move || {
            db.list(&ns, options)
                .map(|page| page.records.into_iter().map(record_to_memory).collect())
                .map_err(map_core_error)
        })
        .await
    }

    async fn health(&self) -> Result<HealthReport, VantaError> {
        let started = Instant::now();
        let db = self.db.clone();
        // Prove the engine is alive: listing namespaces touches the backend.
        blocking(move || db.list_namespaces().map(|_| ()).map_err(map_core_error)).await?;
        let latency_ms = started.elapsed().as_millis() as u64;
        let checked_at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        Ok(HealthReport {
            status: HealthStatus::Healthy,
            backend: "fjall".to_string(),
            latency_ms,
            checked_at_ms,
            message: Some("backend=fjall".to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    static DIR_SEQ: AtomicU64 = AtomicU64::new(0);

    /// Unique temp dir per test, cleaned up on drop.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            let seq = DIR_SEQ.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "vantadb-desktop-05-{}-{seq}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&path);
            std::fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn item(id: Option<&str>, text: &str) -> IngestItem {
        IngestItem {
            id: id.map(str::to_string),
            namespace: "docs".into(),
            text: text.into(),
            embedding: None,
            metadata: [("lang".to_string(), JsonValue::from("en"))].into_iter().collect(),
        }
    }

    #[tokio::test]
    async fn crud_roundtrip_via_trait_object() {
        let dir = TempDir::new();
        let mut conn: Box<dyn VantaConnection> =
            Box::new(NativeConnection::open(dir.path()).expect("open"));

        // ingest assigns an id when none is supplied
        let id = conn
            .ingest(item(None, "the quick brown fox jumps over the lazy dog"))
            .await
            .expect("ingest");
        assert!(!id.is_empty());

        // get roundtrip
        let rec = conn.get(&id, Some("docs")).await.expect("get");
        assert_eq!(rec.text, "the quick brown fox jumps over the lazy dog");
        assert_eq!(rec.metadata["lang"], JsonValue::from("en"));

        // search finds it via BM25 text query
        let hits = conn
            .search(SearchQuery {
                query: "fox".into(),
                embedding: None,
                top_k: 10,
                namespace: Some("docs".into()),
                filters: Default::default(),
            })
            .await
            .expect("search");
        assert!(
            hits.iter().any(|h| h.id == id),
            "search should return the ingested record, got: {hits:?}"
        );

        // delete + idempotent second delete
        conn.delete(&id, Some("docs")).await.expect("delete");
        conn.delete(&id, Some("docs")).await.expect("delete again");

        // get after delete -> not-found error
        let err = conn.get(&id, Some("docs")).await.expect_err("get after delete");
        assert!(matches!(err, VantaError::Native(_)));
    }

    #[tokio::test]
    async fn second_open_same_path_locks() {
        let dir = TempDir::new();
        let mut first = NativeConnection::open(dir.path()).expect("first open");

        // A second handle on the same path must fail with the lock variant.
        let second = NativeConnection::open(dir.path());
        assert!(
            matches!(&second, Err(VantaError::Lock(_))),
            "expected lock error, got: {:?}",
            second.as_ref().err()
        );

        // After the holder disconnects, the path is available again.
        first.disconnect().await.expect("disconnect");
        let reopened = NativeConnection::open(dir.path());
        assert!(reopened.is_ok(), "path should be reopenable after disconnect");
    }

    #[tokio::test]
    async fn health_reports_fjall_backend() {
        let dir = TempDir::new();
        let conn = NativeConnection::open(dir.path()).expect("open");
        let report = conn.health().await.expect("health");
        assert_eq!(report.status, HealthStatus::Healthy);
        assert_eq!(report.message.as_deref(), Some("backend=fjall"));
    }

    #[test]
    fn capabilities_and_info() {
        let dir = TempDir::new();
        let conn = NativeConnection::open(dir.path()).expect("open");
        assert!(conn.capabilities().contains(&Capability::Native));
        assert_eq!(conn.info().via, Capability::Native);
    }
}
