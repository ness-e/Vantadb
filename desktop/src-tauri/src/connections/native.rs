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
use vantadb::config::VantaConfig;
use vantadb::VantaError as CoreVantaError;
use vantadb::{
    VantaBm25TermContribution, VantaEmbedded, VantaMemoryInput, VantaMemoryListOptions,
    VantaMemoryRecord, VantaMemorySearchHit, VantaMemorySearchRequest, VantaNodeRecord,
    VantaQueryResult as CoreQueryResult, VantaSearchExplanationHit, VantaValue,
};

use super::types::{
    Bm25Term, Capability, ConnectionInfo, ConnectionStatus, ExplanationHit, HealthReport,
    HealthStatus, IngestItem, ListPage, MemoryRecord, SearchQuery, SearchResult, VantaQueryResult,
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
    /// Audit log path configured on open; `None` = audit disabled (VS-12).
    audit_log_path: Option<PathBuf>,
}

impl NativeConnection {
    /// Open (or create) the embedded database at `path`.
    ///
    /// Fails with [`VantaError::Lock`] when another connection already holds an
    /// exclusive writer lock on the same path (the core reports `DatabaseBusy`).
    ///
    /// Audit (VS-12): enabled by default at `<path>/audit.jsonl`. Use
    /// [`Self::open_with_audit`] to set a custom path or disable it.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, VantaError> {
        let path = path.into();
        Self::open_with_audit(path.clone(), Some(path.join("audit.jsonl")))
    }

    /// Open with an explicit audit-log policy (VS-12).
    ///
    /// `audit_log_path`:
    /// - `Some(p)` — write audit events to `p` (parent dirs created as needed).
    /// - `None` — audit disabled; the connection reports no audit log and
    ///   `vanta_audit_events` fails with `Unsupported`.
    pub fn open_with_audit(
        path: impl Into<PathBuf>,
        audit_log_path: Option<PathBuf>,
    ) -> Result<Self, VantaError> {
        let path = path.into();
        let config = VantaConfig {
            storage_path: path.to_string_lossy().into_owned(),
            audit_log_path: audit_log_path.clone(),
            ..Default::default()
        };
        let db = VantaEmbedded::open_with_config(config).map_err(map_core_error)?;
        let id = format!("native:{}", path.display());
        Ok(Self {
            id,
            path,
            db,
            audit_log_path,
        })
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

/// Current unix time in milliseconds.
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
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
                VantaValue::ListString(
                    items
                        .iter()
                        .map(|i| i.as_str().unwrap().to_string())
                        .collect(),
                )
            } else if items.iter().all(JsonValue::is_boolean) {
                VantaValue::ListBool(items.iter().map(|i| i.as_bool().unwrap()).collect())
            } else if items.iter().all(JsonValue::is_i64) {
                VantaValue::ListInt(items.iter().map(|i| i.as_i64().unwrap()).collect())
            } else if items.iter().all(JsonValue::is_number) {
                VantaValue::ListFloat(
                    items
                        .iter()
                        .map(|i| i.as_f64().unwrap_or_default())
                        .collect(),
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
        vector: r.vector,
        metadata: r
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), from_vanta_value(v)))
            .collect(),
        created_at_ms: Some(r.created_at_ms),
        updated_at_ms: Some(r.updated_at_ms),
        version: Some(r.version),
        node_id: Some(r.node_id.to_string()),
        sparse_vector: r.sparse_vector.map(|sv| sv.0.into_iter().collect()),
        expires_at_ms: r.expires_at_ms,
    }
}

/// Map the core `VantaQueryResult` into the desktop wire DTO (VS-CORE-06).
fn core_query_to_wire(r: CoreQueryResult) -> VantaQueryResult {
    match r {
        CoreQueryResult::Read(records) => {
            VantaQueryResult::Read(records.into_iter().map(node_record_to_memory).collect())
        }
        CoreQueryResult::Write {
            affected_nodes,
            message,
            node_id,
        } => VantaQueryResult::Write {
            affected_nodes: affected_nodes as u64,
            message,
            node_id: node_id.map(|id| id.to_string()),
        },
        CoreQueryResult::StaleContext { node_id } => VantaQueryResult::StaleContext {
            node_id: node_id.to_string(),
        },
    }
}

/// Convert a `VantaNodeRecord` (IQL result) into a desktop `MemoryRecord`.
///
/// IQL nodes don't carry memory-SDK timestamps/version; those stay `None`
/// (the UI already treats them as optional). The `__vanta_*` reserved fields
/// (namespace, payload, key, ...) are stripped from metadata, mirroring the
/// memory SDK's `record_to_memory`; namespace and text are recovered from
/// them, falling back to `text`/`content` for nodes created via IQL.
fn node_record_to_memory(n: VantaNodeRecord) -> MemoryRecord {
    let metadata: std::collections::HashMap<String, JsonValue> = n
        .fields
        .iter()
        .filter(|(k, _)| !k.starts_with("__vanta_"))
        .map(|(k, v)| (k.clone(), from_vanta_value(v)))
        .collect();
    let text = ["__vanta_payload", "text", "content"]
        .into_iter()
        .find_map(|k| match n.fields.get(k) {
            Some(VantaValue::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_default();
    let namespace = match n.fields.get("__vanta_namespace") {
        Some(VantaValue::String(s)) => s.clone(),
        _ => DEFAULT_NAMESPACE.to_string(),
    };
    MemoryRecord {
        id: n.id.to_string(),
        namespace,
        text,
        vector: n.vector,
        metadata,
        created_at_ms: None,
        updated_at_ms: None,
        version: None,
        node_id: Some(n.id.to_string()),
        sparse_vector: None,
        expires_at_ms: None,
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
        explanation: h.explanation.map(explanation_to_dto),
    }
}

/// Mirror a core `VantaSearchExplanationHit` 1:1 into the desktop wire DTO
/// (`ExplanationHit`), which the frontend consumes (VS-CORE-03).
fn explanation_to_dto(h: VantaSearchExplanationHit) -> ExplanationHit {
    ExplanationHit {
        identity: h.identity,
        score: h.score,
        snippet: h.snippet,
        matched_tokens: h.matched_tokens,
        matched_phrases: h.matched_phrases,
        bm25_terms: h
            .bm25_terms
            .into_iter()
            .map(|t: VantaBm25TermContribution| Bm25Term {
                token: t.token,
                tf: t.tf,
                df: t.df,
                doc_len: t.doc_len,
                contribution: t.contribution,
            })
            .collect(),
        rrf_text_rank: h.rrf_text_rank,
        rrf_vector_rank: h.rrf_vector_rank,
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
        namespace: q
            .namespace
            .clone()
            .unwrap_or_else(|| DEFAULT_NAMESPACE.into()),
        query_vector: q.embedding.clone().unwrap_or_default(),
        filters: metadata_to_vanta(&q.filters),
        text_query,
        top_k: q.top_k,
        // The core fills `VantaMemorySearchHit.explanation` when this flag is
        // set (src/sdk/search/mod.rs), so explain mode needs no extra calls.
        explain: q.explain,
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

    async fn put(
        &mut self,
        item: IngestItem,
        expires_at_ms: Option<u64>,
    ) -> Result<MemoryRecord, VantaError> {
        let key = item.id.clone().unwrap_or_else(gen_id);
        let mut input = ingest_to_input(&item, &key);
        if let Some(expires_at_ms) = expires_at_ms {
            // Core `put` takes a *relative* ttl (expires_at = now + ttl), so
            // convert the absolute unix-ms expiry the UI edits into a ttl.
            input.ttl_ms = Some(expires_at_ms.saturating_sub(now_ms()));
        }
        let db = self.db.clone();
        blocking(move || db.put(input).map(record_to_memory).map_err(map_core_error)).await
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
        record
            .map(record_to_memory)
            .ok_or_else(|| VantaError::Native(not_found))
    }

    async fn get_version(
        &self,
        id: &str,
        version: u64,
        namespace: Option<&str>,
    ) -> Result<MemoryRecord, VantaError> {
        let ns = namespace.unwrap_or(DEFAULT_NAMESPACE).to_string();
        let key = id.to_string();
        let not_found = format!("record version not found: {ns}/{key} v{version}");
        let db = self.db.clone();
        let record =
            blocking(move || db.get_version(&ns, &key, version).map_err(map_core_error)).await?;
        record
            .map(record_to_memory)
            .ok_or_else(|| VantaError::Native(not_found))
    }

    async fn versions(
        &self,
        id: &str,
        namespace: Option<&str>,
    ) -> Result<Vec<MemoryRecord>, VantaError> {
        let ns = namespace.unwrap_or(DEFAULT_NAMESPACE).to_string();
        let key = id.to_string();
        let db = self.db.clone();
        blocking(move || {
            db.versions(&ns, &key)
                .map(|records| records.into_iter().map(record_to_memory).collect())
                .map_err(map_core_error)
        })
        .await
    }

    async fn query(&self, query: &str) -> Result<VantaQueryResult, VantaError> {
        let db = self.db.clone();
        let query = query.to_string();
        blocking(move || {
            db.query(&query)
                .map(core_query_to_wire)
                .map_err(map_core_error)
        })
        .await
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
        cursor: Option<usize>,
    ) -> Result<ListPage, VantaError> {
        let ns = namespace.unwrap_or(DEFAULT_NAMESPACE).to_string();
        let options = VantaMemoryListOptions {
            limit,
            cursor,
            ..Default::default()
        };
        let db = self.db.clone();
        blocking(move || {
            db.list(&ns, options)
                .map(|page| ListPage {
                    records: page.records.into_iter().map(record_to_memory).collect(),
                    next_cursor: page.next_cursor,
                })
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

    fn audit_log_path(&self) -> Option<PathBuf> {
        self.audit_log_path.clone()
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
            let path = std::env::temp_dir()
                .join(format!("vantadb-desktop-05-{}-{seq}", std::process::id()));
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
            metadata: [("lang".to_string(), JsonValue::from("en"))]
                .into_iter()
                .collect(),
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
                explain: false,
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
        let err = conn
            .get(&id, Some("docs"))
            .await
            .expect_err("get after delete");
        assert!(matches!(err, VantaError::Native(_)));
    }

    #[tokio::test]
    async fn put_upserts_and_sets_expiry() {
        let dir = TempDir::new();
        let mut conn = NativeConnection::open(dir.path()).expect("open");

        // create
        let rec = conn
            .put(item(Some("k1"), "first"), None)
            .await
            .expect("put create");
        assert_eq!(rec.id, "k1");
        assert_eq!(rec.text, "first");

        // upsert over the same key replaces the payload
        let rec2 = conn
            .put(item(Some("k1"), "second"), None)
            .await
            .expect("put upsert");
        assert_eq!(rec2.id, "k1");
        assert_eq!(rec2.text, "second");

        // absolute unix-ms expiry lands as ttl on the underlying core record
        let now = now_ms();
        conn.put(item(Some("k2"), "temp"), Some(now + 60_000))
            .await
            .expect("put with expiry");
        let core = conn.db.get("docs", "k2").expect("get").expect("exists");
        assert!(
            core.expires_at_ms.is_some(),
            "expiry should persist on the core record"
        );
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
        assert!(
            reopened.is_ok(),
            "path should be reopenable after disconnect"
        );
    }

    #[tokio::test]
    async fn search_explain_fills_breakdown() {
        let dir = TempDir::new();
        let mut conn = NativeConnection::open(dir.path()).expect("open");

        conn.put(
            item(Some("k1"), "the quick brown fox jumps over the lazy dog"),
            None,
        )
        .await
        .expect("put k1");
        conn.put(
            item(Some("k2"), "a red fox stalks prey inside the garden wall"),
            None,
        )
        .await
        .expect("put k2");

        let hits = conn
            .search(SearchQuery {
                query: "fox".into(),
                embedding: None,
                top_k: 5,
                namespace: Some("docs".into()),
                filters: Default::default(),
                explain: true,
            })
            .await
            .expect("explain search");
        assert_eq!(hits.len(), 2, "both fox docs should match, got: {hits:?}");

        for hit in &hits {
            let explanation = hit
                .explanation
                .as_ref()
                .expect("explain mode must fill explanation");
            assert_eq!(explanation.identity, format!("docs\0{}", hit.id));
            assert_eq!(explanation.score, hit.score);
            // Text query matched tokens are present with a BM25 breakdown.
            assert!(
                explanation.matched_tokens.contains(&"fox".to_string()),
                "matched_tokens should contain the query token: {explanation:?}"
            );
            assert!(
                !explanation.bm25_terms.is_empty(),
                "bm25_terms should be populated: {explanation:?}"
            );
            let fox_term = explanation
                .bm25_terms
                .iter()
                .find(|t| t.token == "fox")
                .expect("fox term contribution present");
            assert!(fox_term.tf >= 1);
            assert!(fox_term.df >= 1);
            assert!(fox_term.doc_len >= 1);
            assert!(fox_term.contribution > 0.0);
            // Text-only route: rrf_text_rank is populated, vector rank absent.
            assert!(
                explanation.rrf_text_rank.is_some(),
                "text route must populate rrf_text_rank: {explanation:?}"
            );
            assert_eq!(explanation.rrf_vector_rank, None);
        }

        // Regular search (explain=false) leaves explanation empty — backward compat.
        let plain = conn
            .search(SearchQuery {
                query: "fox".into(),
                embedding: None,
                top_k: 5,
                namespace: Some("docs".into()),
                filters: Default::default(),
                explain: false,
            })
            .await
            .expect("plain search");
        assert!(
            plain.iter().all(|h| h.explanation.is_none()),
            "plain search must not carry explanations"
        );
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

    // ── Audit log (VS-12) ──

    #[tokio::test]
    async fn put_and_delete_write_audit_events() {
        let dir = TempDir::new();
        let mut conn = NativeConnection::open(dir.path()).expect("open");
        let audit_path = conn
            .audit_log_path()
            .expect("default open must configure audit at <storage>/audit.jsonl");

        conn.put(item(Some("k1"), "hello audit"), None)
            .await
            .expect("put");
        conn.delete("k1", Some("docs")).await.expect("delete");

        // The core wrote one event per operation; parse the JSONL directly.
        let content = std::fs::read_to_string(&audit_path).expect("read audit log");
        let events: Vec<crate::connections::types::AuditEvent> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        let ops: Vec<&str> = events.iter().map(|e| e.op.as_str()).collect();
        assert!(ops.contains(&"put"), "expected a put event, got: {ops:?}");
        assert!(
            ops.contains(&"delete"),
            "expected a delete event, got: {ops:?}"
        );
        let put_ev = events.iter().find(|e| e.op == "put").expect("put event");
        assert_eq!(put_ev.namespace, "docs");
        assert_eq!(put_ev.key, "k1");
        assert_eq!(put_ev.outcome, "ok");
    }

    #[test]
    fn open_without_audit_reports_none() {
        let dir = TempDir::new();
        let conn = NativeConnection::open_with_audit(dir.path(), None).expect("open");
        assert!(
            conn.audit_log_path().is_none(),
            "audit disabled must report no audit log"
        );
    }

    #[test]
    fn open_with_custom_audit_path() {
        let dir = TempDir::new();
        let custom = dir.path().join("custom").join("events.jsonl");
        let conn =
            NativeConnection::open_with_audit(dir.path(), Some(custom.clone())).expect("open");
        assert_eq!(conn.audit_log_path(), Some(custom));
    }

    // ── Version history (VS-CORE-07) ──

    #[tokio::test]
    async fn version_history_roundtrip_via_trait_object() {
        let dir = TempDir::new();
        let mut conn: Box<dyn VantaConnection> =
            Box::new(NativeConnection::open(dir.path()).expect("open"));

        conn.put(item(Some("k1"), "v1 text"), None)
            .await
            .expect("put v1");
        conn.put(item(Some("k1"), "v2 text"), None)
            .await
            .expect("put v2");
        conn.put(item(Some("k1"), "v3 text"), None)
            .await
            .expect("put v3");

        // versions: ascending v1..vN with payload per version
        let all = conn.versions("k1", Some("docs")).await.expect("versions");
        assert_eq!(all.len(), 3);
        assert_eq!(all[0].version, Some(1));
        assert_eq!(all[0].text, "v1 text");
        assert_eq!(all[2].version, Some(3));
        assert_eq!(all[2].text, "v3 text");

        // get_version returns the exact historical snapshot
        let v2 = conn
            .get_version("k1", 2, Some("docs"))
            .await
            .expect("get_version");
        assert_eq!(v2.version, Some(2));
        assert_eq!(v2.text, "v2 text");

        // missing version -> not-found error
        let err = conn
            .get_version("k1", 99, Some("docs"))
            .await
            .expect_err("missing version");
        assert!(matches!(err, VantaError::Native(_)));

        // delete purges history
        conn.delete("k1", Some("docs")).await.expect("delete");
        let after = conn.versions("k1", Some("docs")).await.expect("versions");
        assert!(after.is_empty(), "history must be purged on delete");
    }
}
