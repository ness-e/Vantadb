use async_trait::async_trait;

use super::types::{
    Capability, ConnectionInfo, ExportReport, HealthReport, IngestItem, ListPage, MemoryFilterItem,
    MemoryRecord, SearchQuery, SearchResult, VantaQueryResult,
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

    /// Cheap liveness / latency probe.
    async fn health(&self) -> Result<HealthReport, VantaError>;

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
