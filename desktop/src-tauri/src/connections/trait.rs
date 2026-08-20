use async_trait::async_trait;

use super::types::{
    Capability, ConnectionInfo, ExportReport, HealthReport, IngestItem, ListPage, MemoryFilterItem,
    MemoryRecord, NamespaceStatsMap, SearchQuery, SearchResult, VantaGraphNodeInfo,
    VantaGraphTraversalResult, VantaQueryResult,
};
use crate::error::VantaError;

/// A single connection to a VantaDB backend.
///
/// This is the **contract** of the desktop multi-connection architecture: every adapter
/// (native / server / HTTP / MCP / ...) implements it, and a future `ConnectionManager`
/// holds them as `Box<dyn VantaConnection>`. Only the contract lives here — no adapter
/// logic.
///
/// Object-safe: `async_trait` boxes each future, methods take only `&self`/`&mut self`,
/// and there are no generics or `Self`-by-value returns — so `&dyn VantaConnection` is a
/// valid trait object (compile-time check in tests). `Send + Sync` supertrait lets the
/// object be stored behind a Tauri `State` / shared manager.
#[async_trait]
pub trait VantaConnection: Send + Sync {
    /// Static metadata describing this connection.
    fn info(&self) -> ConnectionInfo;

    /// Which transport bridges this connection can expose.
    fn capabilities(&self) -> Vec<Capability>;

    /// Establish the connection. Idempotent: safe to call when already connected.
    async fn connect(&mut self) -> Result<(), VantaError>;

    /// Tear down the connection. Idempotent.
    async fn disconnect(&mut self) -> Result<(), VantaError>;

    /// Store a single item, returning its id (assigned or supplied).
    async fn ingest(&mut self, item: IngestItem) -> Result<String, VantaError>;

    /// Upsert a single record by key (creating or replacing), optionally
    /// pinning an absolute unix-ms expiry. Returns the stored record.
    ///
    /// Default implementation: transports without an upsert-by-key API report
    /// [`VantaError::Unsupported`] instead of guessing semantics. Native
    /// (embedded) implements it via the core `put`.
    async fn put(
        &mut self,
        item: IngestItem,
        expires_at_ms: Option<u64>,
    ) -> Result<MemoryRecord, VantaError> {
        let _ = (item, expires_at_ms);
        Err(VantaError::Unsupported(
            "put (upsert by key) is not implemented by this transport".into(),
        ))
    }

    /// Store many items. Each returned id corresponds positionally to `items`.
    async fn ingest_batch(&mut self, items: Vec<IngestItem>) -> Result<Vec<String>, VantaError>;

    /// Semantic / text search over stored memories.
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, VantaError>;

    /// Fetch a single record by id, optionally scoped to a namespace.
    async fn get(&self, id: &str, namespace: Option<&str>) -> Result<MemoryRecord, VantaError>;

    /// Fetch the record as it was at a specific version (VS-CORE-07).
    ///
    /// Default: transports without version history report
    /// [`VantaError::Unsupported`] — only native (embedded) implements it via
    /// the core `get_version`.
    async fn get_version(
        &self,
        id: &str,
        version: u64,
        namespace: Option<&str>,
    ) -> Result<MemoryRecord, VantaError> {
        let _ = (id, version, namespace);
        Err(VantaError::Unsupported(
            "get_version (version history) is not implemented by this transport".into(),
        ))
    }

    /// List every retained version of a record, ascending v1..vN (VS-CORE-07).
    ///
    /// Default: transports without version history report
    /// [`VantaError::Unsupported`] — only native (embedded) implements it via
    /// the core `versions`.
    async fn versions(
        &self,
        id: &str,
        namespace: Option<&str>,
    ) -> Result<Vec<MemoryRecord>, VantaError> {
        let _ = (id, namespace);
        Err(VantaError::Unsupported(
            "versions (version history) is not implemented by this transport".into(),
        ))
    }

    /// Delete a single record by id, optionally scoped to a namespace. Idempotent.
    async fn delete(&mut self, id: &str, namespace: Option<&str>) -> Result<(), VantaError>;

    /// Execute an IQL statement (VS-CORE-06).
    ///
    /// Default implementation: transports without an IQL endpoint report
    /// [`VantaError::Unsupported`] — only native (embedded) implements it via
    /// the core `VantaEmbedded::query`.
    async fn query(&self, query: &str) -> Result<VantaQueryResult, VantaError> {
        let _ = query;
        Err(VantaError::Unsupported(
            "query (IQL) is not implemented by this transport".into(),
        ))
    }

    /// List a page of records in a namespace, capped at `limit`.
    ///
    /// `cursor` is a zero-based offset into the namespace's stable id order
    /// (`None` starts from the beginning); pass the previous page's
    /// `next_cursor` to continue. Returns the page plus the cursor for the
    /// next page (`None` = last page).
    async fn list(
        &self,
        namespace: Option<&str>,
        limit: usize,
        cursor: Option<usize>,
    ) -> Result<ListPage, VantaError>;

    /// Export records in a namespace to a JSONL file, optionally filtered by
    /// AND-combined metadata items (VS-CORE-04).
    ///
    /// `None` (or an empty filter) exports the full namespace. Default
    /// implementation: transports without a file-export endpoint report
    /// [`VantaError::Unsupported`] — only native (embedded) implements it via
    /// the core `export_namespace`.
    async fn export_namespace(
        &self,
        path: &str,
        namespace: &str,
        filter: Option<Vec<MemoryFilterItem>>,
    ) -> Result<ExportReport, VantaError> {
        let _ = (path, namespace, filter);
        Err(VantaError::Unsupported(
            "export_namespace is not implemented by this transport".into(),
        ))
    }

    /// Delete all records in a namespace matching an AND-combined metadata
    /// filter (VS-CORE-05). Returns the number of deleted records.
    ///
    /// The core rejects an empty filter to prevent accidental full-namespace
    /// deletion; that error propagates to the UI unchanged. Default
    /// implementation: transports without batch-delete report
    /// [`VantaError::Unsupported`] — only native (embedded) implements it via
    /// the core `delete_by_filter`.
    async fn delete_by_filter(
        &mut self,
        namespace: &str,
        filter: Vec<MemoryFilterItem>,
    ) -> Result<u64, VantaError> {
        let _ = (namespace, filter);
        Err(VantaError::Unsupported(
            "delete_by_filter (batch delete) is not implemented by this transport".into(),
        ))
    }

    /// Breadth-first graph traversal from root node ids (GRAFO-01).
    ///
    /// `roots` are node ids (u128, string-serialized on the wire). `direction`
    /// is `"Forward"` / `"Reverse"` / `"Both"` (core `TraversalDirection`).
    /// `limit` caps the number of nodes/edges returned (default 50). Default
    /// implementation: transports without graph traversal report
    /// [`VantaError::Unsupported`] — only native (embedded) implements it via
    /// the core `VantaEmbedded::graph_bfs`.
    async fn graph_bfs(
        &self,
        roots: Vec<String>,
        max_depth: usize,
        direction: String,
        limit: Option<usize>,
    ) -> Result<VantaGraphTraversalResult, VantaError> {
        let _ = (roots, max_depth, direction, limit);
        Err(VantaError::Unsupported(
            "graph_bfs (graph traversal) is not implemented by this transport".into(),
        ))
    }

    /// Depth-first graph traversal from root node ids (GRAFO-01).
    ///
    /// Same contract as [`Self::graph_bfs`]; only native (embedded) implements
    /// it via the core `VantaEmbedded::graph_dfs`.
    async fn graph_dfs(
        &self,
        roots: Vec<String>,
        max_depth: usize,
        direction: String,
        limit: Option<usize>,
    ) -> Result<VantaGraphTraversalResult, VantaError> {
        let _ = (roots, max_depth, direction, limit);
        Err(VantaError::Unsupported(
            "graph_dfs (graph traversal) is not implemented by this transport".into(),
        ))
    }

    /// Degree centrality (in+out counts) for every node in `namespace`
    /// (GRAFO-01). Returns nodes up to `limit` (default 50) with their
    /// `degree` populated; an empty/unknown namespace returns an empty list,
    /// not an error. Only native (embedded) implements it via the core
    /// `VantaEmbedded::graph_degree_centrality`.
    async fn graph_degree(
        &self,
        namespace: &str,
        limit: Option<usize>,
    ) -> Result<Vec<VantaGraphNodeInfo>, VantaError> {
        let _ = (namespace, limit);
        Err(VantaError::Unsupported(
            "graph_degree (degree centrality) is not implemented by this transport".into(),
        ))
    }

    /// Cheap liveness / latency probe.
    async fn health(&self) -> Result<HealthReport, VantaError>;

    /// Per-namespace record statistics (VS-CORE-02).
    ///
    /// `expiring_soon_window_ms` defaults to the core's 24h window when
    /// `None`. `count` includes expired (not-yet-purged) records so the
    /// `expired` bucket is observable. Default implementation: transports
    /// without a stats endpoint report [`VantaError::Unsupported`] — native
    /// implements it via the core `namespace_stats`, server via
    /// `/api/v2/metrics`.
    async fn namespace_stats(
        &self,
        expiring_soon_window_ms: Option<u64>,
    ) -> Result<NamespaceStatsMap, VantaError> {
        let _ = expiring_soon_window_ms;
        Err(VantaError::Unsupported(
            "namespace_stats is not implemented by this transport".into(),
        ))
    }

    /// Path of this transport's audit log (VS-12).
    ///
    /// `None` means the transport has no audit log (e.g. a server connection,
    /// or a native connection opened with audit disabled). Transports that
    /// write one override this; the default keeps existing impls unchanged.
    fn audit_log_path(&self) -> Option<std::path::PathBuf> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Compile-time proof of object safety: taking `&dyn VantaConnection` compiles only
    // if the trait is dyn-compatible. This function is never called; its mere existence
    // (compiled as part of the crate) is the check.
    #[allow(dead_code)]
    fn assert_object_safe(_conn: &dyn VantaConnection) {}
}
