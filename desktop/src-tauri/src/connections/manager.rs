//! Thread-safe registry of live [`VantaConnection`]s (DESK-06).
//!
//! Replaces the `manager: ()` placeholder in [`AppState`](crate::AppState): holds
//! every open connection keyed by id plus the single currently-*active* one that
//! the data commands (`vanta_ingest`/`vanta_search`/…) target.
//!
//! Concurrency: `tokio::sync::RwLock` lets concurrent read ops (`search`/`get`/
//! `list`/`health`) share the guard, while mutation ops (`add`/`remove`/`set_active`
//! and the `&mut self` adapter calls) take the write path. Every alias adapter
//! already `spawn_blocking`s its SDK work, so awaiting the guard's inner call
//! never blocks the Tauri runtime — it only serializes commands onto one writer.
//!
//! ponytail: one global RwLock serializes data ops across *all* connections.
//! Acceptable while the desktop drives a single active backend at a time; a
//! per-connection lock or sharded registry is the upgrade if parallel writes to
//! several backends become hot.

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::RwLock;

use super::{
    Capability, ConnectionInfo, ExportReport, HealthReport, IngestItem, ListPage, MemoryFilterItem,
    MemoryRecord, NamespaceStatsMap, SearchQuery, SearchResult, VantaConnection,
    VantaGraphNodeInfo, VantaGraphTraversalResult, VantaQueryResult,
};
use crate::error::VantaError;

#[derive(Default)]
struct Inner {
    /// Live connections by id.
    connections: HashMap<String, Box<dyn VantaConnection>>,
    /// Id of the connection data commands currently target.
    active_id: Option<String>,
}

/// Registry + active-connection selector shared via managed Tauri state.
#[derive(Default)]
pub struct ConnectionManager {
    inner: RwLock<Inner>,
}

impl ConnectionManager {
    /// Per-connection grace period used by [`ConnectionManager::shutdown_all`].
    pub const SHUTDOWN_GRACE: std::time::Duration = std::time::Duration::from_secs(5);

    /// Empty registry with no active connection.
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Inner::default()),
        }
    }

    fn no_active() -> VantaError {
        VantaError::Unsupported("no active connection; call vanta_connect first".into())
    }

    fn missing(id: &str) -> VantaError {
        VantaError::Other(format!("connection not found: {id}"))
    }

    /// Register `conn`, connect it (validating health), store it, and make it
    /// the active connection. Returns its static info.
    pub async fn add(
        &self,
        mut conn: Box<dyn VantaConnection>,
    ) -> Result<ConnectionInfo, VantaError> {
        conn.connect().await?;
        let info = conn.info();
        let id = info.id.clone();
        let mut inner = self.inner.write().await;
        inner.connections.insert(id.clone(), conn);
        inner.active_id = Some(id.clone());
        Ok(info)
    }

    /// Remove a connection by id, disconnecting it and releasing any backend
    /// resources (e.g. the native path lock). Clears active if it was the target.
    pub async fn remove(&self, id: &str) -> Result<(), VantaError> {
        let mut inner = self.inner.write().await;
        let taken = inner.connections.remove(id);
        if inner.active_id.as_deref() == Some(id) {
            inner.active_id = inner.connections.keys().next().cloned();
        }
        drop(inner);
        match taken {
            Some(mut conn) => conn.disconnect().await,
            None => Err(Self::missing(id)),
        }
    }

    /// Mark `id` as the active connection.
    pub async fn set_active(&self, id: &str) -> Result<(), VantaError> {
        let mut inner = self.inner.write().await;
        if !inner.connections.contains_key(id) {
            return Err(Self::missing(id));
        }
        inner.active_id = Some(id.to_string());
        Ok(())
    }

    /// Id of the currently-active connection.
    pub async fn active_id(&self) -> Result<String, VantaError> {
        let inner = self.inner.read().await;
        inner.active_id.clone().ok_or_else(Self::no_active)
    }

    /// Snapshot of every registered connection as `(id, info)`.
    pub async fn list_connections(&self) -> Vec<(String, ConnectionInfo)> {
        let inner = self.inner.read().await;
        inner
            .connections
            .iter()
            .map(|(k, v)| (k.clone(), v.info()))
            .collect()
    }

    /// Static info of the active connection.
    pub async fn active_info(&self) -> Result<ConnectionInfo, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        inner
            .connections
            .get(&id)
            .map(|c| c.info())
            .ok_or_else(|| Self::missing(&id))
    }

    /// Live health probe of the active connection.
    pub async fn health(&self) -> Result<HealthReport, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.health().await
    }

    /// Path of the active connection's audit log, when the transport has one
    /// (VS-12). `None` = the transport writes no audit log.
    pub async fn audit_log_path(&self) -> Result<Option<PathBuf>, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        Ok(conn.audit_log_path())
    }

    /// Store a single item on the active connection, returning its id.
    pub async fn ingest(&self, item: IngestItem) -> Result<String, VantaError> {
        let id = self.active_id().await?;
        let mut inner = self.inner.write().await;
        let conn = inner
            .connections
            .get_mut(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.ingest(item).await
    }

    /// Upsert a single record by key on the active connection, returning the
    /// stored record.
    pub async fn put(
        &self,
        item: IngestItem,
        expires_at_ms: Option<u64>,
    ) -> Result<MemoryRecord, VantaError> {
        let id = self.active_id().await?;
        let mut inner = self.inner.write().await;
        let conn = inner
            .connections
            .get_mut(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.put(item, expires_at_ms).await
    }

    /// Store many items on the active connection, returning ids positionally.
    pub async fn ingest_batch(&self, items: Vec<IngestItem>) -> Result<Vec<String>, VantaError> {
        let id = self.active_id().await?;
        let mut inner = self.inner.write().await;
        let conn = inner
            .connections
            .get_mut(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.ingest_batch(items).await
    }

    /// Search the active connection.
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.search(query).await
    }

    /// Fetch a single record by key on the active connection.
    pub async fn get(
        &self,
        key: &str,
        namespace: Option<&str>,
    ) -> Result<MemoryRecord, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.get(key, namespace).await
    }

    /// Fetch a record as it was at a specific version on the active connection.
    pub async fn get_version(
        &self,
        key: &str,
        version: u64,
        namespace: Option<&str>,
    ) -> Result<MemoryRecord, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.get_version(key, version, namespace).await
    }

    /// List every retained version of a record on the active connection.
    pub async fn versions(
        &self,
        key: &str,
        namespace: Option<&str>,
    ) -> Result<Vec<MemoryRecord>, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.versions(key, namespace).await
    }

    /// Execute an IQL statement on the active connection (VS-CORE-06).
    ///
    /// Read-only dispatch: IQL `SELECT`/`FETCH` only. Transports without an
    /// IQL endpoint report [`VantaError::Unsupported`] via the trait default.
    pub async fn query(&self, query: &str) -> Result<VantaQueryResult, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.query(query).await
    }

    /// Per-namespace record statistics on the active connection (VS-CORE-02).
    ///
    /// `expiring_soon_window_ms` defaults to the core's 24h window when
    /// `None`. Transports without a stats endpoint report `Unsupported`.
    pub async fn namespace_stats(
        &self,
        expiring_soon_window_ms: Option<u64>,
    ) -> Result<NamespaceStatsMap, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.namespace_stats(expiring_soon_window_ms).await
    }

    /// Delete a record by key on the active connection. Idempotent.
    pub async fn delete(&self, key: &str, namespace: Option<&str>) -> Result<(), VantaError> {
        let id = self.active_id().await?;
        let mut inner = self.inner.write().await;
        let conn = inner
            .connections
            .get_mut(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.delete(key, namespace).await
    }

    /// Delete every record matching an AND-combined metadata filter on the
    /// active connection (VS-CORE-05), returning the number deleted.
    ///
    /// The core rejects an empty filter to prevent accidental full-namespace
    /// deletion — that error propagates unchanged. Transports without
    /// batch-delete report `Unsupported`.
    pub async fn delete_by_filter(
        &self,
        namespace: &str,
        filter: Vec<MemoryFilterItem>,
    ) -> Result<u64, VantaError> {
        let id = self.active_id().await?;
        let mut inner = self.inner.write().await;
        let conn = inner
            .connections
            .get_mut(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.delete_by_filter(namespace, filter).await
    }

    /// Breadth-first graph traversal on the active connection (GRAFO-01).
    ///
    /// `direction` is `"Forward"` / `"Reverse"` / `"Both"`; `limit` caps the
    /// result (default 50). Transports without graph traversal report
    /// `Unsupported` via the trait default.
    pub async fn graph_bfs(
        &self,
        roots: Vec<String>,
        max_depth: usize,
        direction: String,
        limit: Option<usize>,
    ) -> Result<VantaGraphTraversalResult, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.graph_bfs(roots, max_depth, direction, limit).await
    }

    /// Depth-first graph traversal on the active connection (GRAFO-01).
    pub async fn graph_dfs(
        &self,
        roots: Vec<String>,
        max_depth: usize,
        direction: String,
        limit: Option<usize>,
    ) -> Result<VantaGraphTraversalResult, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.graph_dfs(roots, max_depth, direction, limit).await
    }

    /// Degree centrality (in+out) for every node in `namespace` on the active
    /// connection (GRAFO-01). Empty/unknown namespace → empty list.
    pub async fn graph_degree(
        &self,
        namespace: &str,
        limit: Option<usize>,
    ) -> Result<Vec<VantaGraphNodeInfo>, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.graph_degree(namespace, limit).await
    }

    /// List a page of records on the active connection.
    pub async fn list_records(
        &self,
        namespace: Option<&str>,
        limit: Option<usize>,
        cursor: Option<usize>,
    ) -> Result<ListPage, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.list(namespace, limit.unwrap_or(100), cursor).await
    }

    /// Export a namespace (optionally filtered) to a JSONL file on the active
    /// connection (VS-CORE-04). `None` (or empty) exports the full namespace.
    pub async fn export_namespace(
        &self,
        namespace: &str,
        path: &str,
        filter: Option<Vec<MemoryFilterItem>>,
    ) -> Result<ExportReport, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.export_namespace(path, namespace, filter).await
    }

    /// Clone of the active native connection's embedded-SDK handle (MEM-53).
    ///
    /// The memory-pipeline commands run sync vanta-memory APIs over it on the
    /// blocking pool, so the handle is cloned out and the read guard released.
    /// Non-native active connections (server/subprocess) fail with
    /// [`VantaError::Unsupported`].
    pub async fn active_embedded(&self) -> Result<vantadb::VantaEmbedded, VantaError> {
        let id = self.active_id().await?;
        let inner = self.inner.read().await;
        let conn = inner
            .connections
            .get(&id)
            .ok_or_else(|| Self::missing(&id))?;
        conn.as_native()
            .map(|native| native.db().clone())
            .ok_or_else(|| {
                VantaError::Unsupported(
                    "the memory pipeline requires a native (embedded) connection".into(),
                )
            })
    }

    /// Tear down every registered connection on app shutdown (DESKTOP-20).
    ///
    /// Order: non-native connections (server / subprocess-backed) are
    /// disconnected first, then native (embedded) last so its `close` flushes
    /// pending writes. Each `disconnect` is bounded by `grace`; a hung adapter
    /// times out and is dropped, and any sidecar it owns is force-killed by
    /// `McpSpawn`'s `Drop` — no orphaned children survive.
    ///
    /// Returns one `(id, result)` per connection; the registry is left empty.
    /// Idempotent: calling again on an empty registry returns `vec![]`.
    pub async fn shutdown_all(
        &self,
        grace: std::time::Duration,
    ) -> Vec<(String, Result<(), VantaError>)> {
        let mut inner = self.inner.write().await;
        let connections = std::mem::take(&mut inner.connections);
        inner.active_id = None;
        drop(inner);

        let mut conns: Vec<(String, Box<dyn VantaConnection>)> = connections.into_iter().collect();
        // Native last: its `disconnect` flushes the embedded store. Stable sort
        // keeps insertion order within each group.
        conns.sort_by_key(|(_, c)| c.info().via != Capability::Native);

        let mut results = Vec::with_capacity(conns.len());
        for (id, mut conn) in conns {
            let res = match tokio::time::timeout(grace, conn.disconnect()).await {
                Ok(res) => res,
                Err(_) => Err(VantaError::Other(format!(
                    "disconnect timed out after {grace:?}"
                ))),
            };
            results.push((id, res));
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connections::Capability;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let seq = SEQ.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("vantadb-desktop-06-{}-{seq}", std::process::id()));
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

    fn item(key: &str, text: &str) -> IngestItem {
        IngestItem {
            id: Some(key.to_string()),
            namespace: "docs".into(),
            text: text.into(),
            embedding: None,
            metadata: Default::default(),
        }
    }

    /// E2E contract (DESK-06): connect native → ingest 3 items → search →
    /// ordered results; plus get/list/delete against the registry.
    #[tokio::test]
    async fn e2e_native_connect_ingest_search_ordered() {
        let dir = TempDir::new();
        let manager = ConnectionManager::new();

        // connect native and make it active
        let info = manager
            .add(Box::new(
                crate::connections::native::NativeConnection::open(dir.path()).unwrap(),
            ))
            .await
            .unwrap();
        assert_eq!(info.via, Capability::Native);

        // ingest 3 items
        let ids = manager
            .ingest_batch(vec![
                item("k1", "the quick brown fox jumps over the lazy dog"),
                item("k2", "a red fox stalks prey inside the garden wall"),
                item("k3", "vector databases power semantic search engines"),
            ])
            .await
            .unwrap();
        assert_eq!(
            ids,
            vec!["k1".to_string(), "k2".to_string(), "k3".to_string()]
        );

        // get roundtrip
        let rec = manager.get("k1", Some("docs")).await.unwrap();
        assert!(rec.text.contains("fox"));

        // put upsert roundtrip (same key + same text keeps search assertions valid)
        let rec = manager
            .put(
                item("k1", "the quick brown fox jumps over the lazy dog"),
                None,
            )
            .await
            .unwrap();
        assert_eq!(rec.id, "k1");

        // search: both fox docs returned, ordered by non-increasing score
        let hits = manager
            .search(SearchQuery {
                query: "fox".into(),
                embedding: None,
                top_k: 5,
                namespace: Some("docs".into()),
                filters: Default::default(),
                explain: false,
            })
            .await
            .unwrap();
        assert!(
            hits.iter().any(|h| h.id == "k1"),
            "k1 should match, got: {hits:?}"
        );
        assert!(hits.iter().any(|h| h.id == "k2"));
        let scores: Vec<f32> = hits.iter().map(|h| h.score).collect();
        assert!(
            scores.windows(2).all(|w| w[0] >= w[1]),
            "results must be ordered by descending score: {scores:?}"
        );

        // list caps at limit
        let listed = manager
            .list_records(Some("docs"), Some(2), None)
            .await
            .unwrap();
        assert!(listed.records.len() == 2);

        // delete + list registry
        manager.delete("k1", Some("docs")).await.unwrap();
        assert_eq!(manager.list_connections().await.len(), 1);
    }

    /// IQL roundtrip (VS-CORE-06): INSERT → FROM (Read with mapped record) →
    /// DELETE → FROM (empty Read) on the native connection.
    #[tokio::test]
    async fn e2e_native_iql_query_roundtrip() {
        let dir = TempDir::new();
        let manager = ConnectionManager::new();
        manager
            .add(Box::new(
                crate::connections::native::NativeConnection::open(dir.path()).unwrap(),
            ))
            .await
            .unwrap();

        // INSERT — Write result carries the affected count + node id (string).
        let res = manager
            .query(r#"INSERT NODE#100 TYPE Person {name: "Ada", content: "hello iql"}"#)
            .await
            .unwrap();
        match res {
            VantaQueryResult::Write {
                affected_nodes,
                node_id,
                ..
            } => {
                assert_eq!(affected_nodes, 1);
                assert_eq!(node_id.as_deref(), Some("100"));
            }
            other => panic!("expected Write, got {other:?}"),
        }

        // SELECT by type — the node maps into a MemoryRecord (text + metadata).
        let res = manager.query(r#"FROM Person"#).await.unwrap();
        match res {
            VantaQueryResult::Read(records) => {
                assert_eq!(records.len(), 1, "expected 1 Person, got {records:?}");
                assert_eq!(records[0].node_id.as_deref(), Some("100"));
                assert_eq!(records[0].text, "hello iql");
                assert_eq!(
                    records[0].metadata.get("name").and_then(|v| v.as_str()),
                    Some("Ada")
                );
            }
            other => panic!("expected Read, got {other:?}"),
        }

        // DELETE — idempotent Write.
        let res = manager.query(r#"DELETE NODE#100"#).await.unwrap();
        assert!(
            matches!(
                res,
                VantaQueryResult::Write {
                    affected_nodes: 1,
                    ..
                }
            ),
            "expected Write(1), got {res:?}"
        );

        // The type no longer matches anything.
        let res = manager.query(r#"FROM Person"#).await.unwrap();
        match res {
            VantaQueryResult::Read(records) => assert!(records.is_empty()),
            other => panic!("expected empty Read, got {other:?}"),
        }
    }

    /// Graph roundtrip (GRAFO-01): put records → RELATE edges between their
    /// deterministic node ids → bfs/dfs return nodes + edges; degree centrality
    /// covers every node in the namespace; unknown namespace → empty list.
    #[tokio::test]
    async fn e2e_native_graph_roundtrip() {
        let dir = TempDir::new();
        let manager = ConnectionManager::new();
        manager
            .add(Box::new(
                crate::connections::native::NativeConnection::open(dir.path()).unwrap(),
            ))
            .await
            .unwrap();

        // Put three records; capture their deterministic graph node ids.
        let mut node_ids = Vec::new();
        for key in ["k1", "k2", "k3"] {
            let rec = manager
                .put(item(key, &format!("record {key}")), None)
                .await
                .unwrap();
            node_ids.push(rec.node_id.expect("put returns node_id"));
        }
        let (n1, n2, n3) = (&node_ids[0], &node_ids[1], &node_ids[2]);

        // Relate n1 → n2 and n2 → n3 (both endpoints must already exist).
        manager
            .query(&format!(r#"RELATE NODE#{n1} --"knows"--> NODE#{n2}"#))
            .await
            .unwrap();
        manager
            .query(&format!(r#"RELATE NODE#{n2} --"knows"--> NODE#{n3}"#))
            .await
            .unwrap();

        // BFS from n1 reaches all three, carrying nodes + edges.
        let res = manager
            .graph_bfs(vec![n1.clone()], 5, "Forward".into(), None)
            .await
            .unwrap();
        assert_eq!(res.nodes.len(), 3, "bfs should visit 3 nodes: {res:?}");
        assert_eq!(res.edges.len(), 2, "bfs should carry 2 edges: {res:?}");
        assert!(res.nodes.iter().any(|n| &n.id == n1), "n1 in result");
        assert!(res.nodes.iter().any(|n| &n.id == n3), "n3 in result");
        // Labels recover the record payload (memory SDK puts `content`).
        let n1_dto = res.nodes.iter().find(|n| &n.id == n1).unwrap();
        assert_eq!(n1_dto.label, "record k1");
        // Edges connect n1→n2 and n2→n3 with the related label.
        assert!(res
            .edges
            .iter()
            .any(|e| &e.source == n1 && &e.target == n2 && e.label.as_deref() == Some("knows")));
        assert!(res.edges.iter().any(|e| &e.source == n2 && &e.target == n3));

        // DFS also reaches all three.
        let res = manager
            .graph_dfs(vec![n1.clone()], 5, "Forward".into(), None)
            .await
            .unwrap();
        assert_eq!(res.nodes.len(), 3, "dfs should visit 3 nodes: {res:?}");

        // Degree centrality covers every node in the namespace: n1 has 1 out,
        // n2 has 1 in + 1 out = 2, n3 has 1 in.
        let degrees = manager.graph_degree("docs", None).await.unwrap();
        assert_eq!(degrees.len(), 3, "degree covers all records: {degrees:?}");
        let d1 = degrees.iter().find(|n| &n.id == n1).unwrap();
        let d2 = degrees.iter().find(|n| &n.id == n2).unwrap();
        let d3 = degrees.iter().find(|n| &n.id == n3).unwrap();
        assert_eq!(d1.degree, 1);
        assert_eq!(d2.degree, 2);
        assert_eq!(d3.degree, 1);
        assert_eq!(d1.group.as_deref(), Some("docs"));

        // Unknown namespace → empty list, not an error.
        let empty = manager.graph_degree("missing-ns", None).await.unwrap();
        assert!(empty.is_empty());

        // limit caps the result.
        let limited = manager
            .graph_bfs(vec![n1.clone()], 5, "Forward".into(), Some(2))
            .await
            .unwrap();
        assert!(limited.nodes.len() <= 2);
    }

    /// Cursor roundtrip (VS-CORE-01): page 1 → next_cursor → page 2 with no
    /// overlap, and a full page is followed by a final page with no cursor.
    #[tokio::test]
    async fn list_records_paginates_by_cursor_without_overlap() {
        let dir = TempDir::new();
        let manager = ConnectionManager::new();
        manager
            .add(Box::new(
                crate::connections::native::NativeConnection::open(dir.path()).unwrap(),
            ))
            .await
            .unwrap();

        let keys: Vec<String> = (0..5).map(|i| format!("k{i}")).collect();
        manager
            .ingest_batch(
                keys.iter()
                    .map(|k| item(k, &format!("payload for {k}")))
                    .collect(),
            )
            .await
            .unwrap();

        // Page 1: first 2 records + a cursor to continue.
        let p1 = manager
            .list_records(Some("docs"), Some(2), None)
            .await
            .unwrap();
        assert_eq!(p1.records.len(), 2);
        let cursor = p1.next_cursor.expect("page 1 is full, must carry a cursor");

        // Page 2: next 2 records, disjoint from page 1.
        let p2 = manager
            .list_records(Some("docs"), Some(2), Some(cursor))
            .await
            .unwrap();
        assert_eq!(p2.records.len(), 2);
        for r in &p2.records {
            assert!(
                !p1.records.iter().any(|x| x.id == r.id),
                "page 2 overlaps page 1: {}",
                r.id
            );
        }

        // Page 3: remaining 1 record; a short page means this was the last.
        let p3 = manager
            .list_records(
                Some("docs"),
                Some(2),
                Some(p2.next_cursor.expect("page 2 is full")),
            )
            .await
            .unwrap();
        assert_eq!(p3.records.len(), 1);
        assert_eq!(p3.next_cursor, None, "a short page is the last page");
    }

    /// shutdown_all empties the registry and disconnects every backend; a
    /// second call is a no-op. Contract (DESKTOP-20): no connections left and
    /// no errors from a healthy native backend.
    #[tokio::test]
    async fn shutdown_all_empties_registry_and_disconnects() {
        let dir = TempDir::new();
        let manager = ConnectionManager::new();
        manager
            .add(Box::new(
                crate::connections::native::NativeConnection::open(dir.path()).unwrap(),
            ))
            .await
            .unwrap();

        let results = manager
            .shutdown_all(std::time::Duration::from_secs(5))
            .await;
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_ok(), "disconnect should succeed");
        assert!(manager.list_connections().await.is_empty());
        assert!(manager.active_id().await.is_err());

        // Idempotent: empty registry → no results.
        assert!(manager
            .shutdown_all(std::time::Duration::from_secs(1))
            .await
            .is_empty());
    }

    /// Audit-log path follows the active connection (VS-12): a default native
    /// open exposes `<storage>/audit.jsonl`; an audit-disabled open exposes none.
    #[tokio::test]
    async fn audit_log_path_follows_active_connection() {
        let dir = TempDir::new();
        let manager = ConnectionManager::new();
        manager
            .add(Box::new(
                crate::connections::native::NativeConnection::open(dir.path()).expect("open"),
            ))
            .await
            .expect("add");
        assert_eq!(
            manager.audit_log_path().await.expect("active connection"),
            Some(dir.path().join("audit.jsonl"))
        );

        // A native connection opened with audit disabled reports None.
        let dir2 = TempDir::new();
        let manager2 = ConnectionManager::new();
        manager2
            .add(Box::new(
                crate::connections::native::NativeConnection::open_with_audit(dir2.path(), None)
                    .expect("open"),
            ))
            .await
            .expect("add");
        assert_eq!(
            manager2.audit_log_path().await.expect("active connection"),
            None
        );
    }
}
