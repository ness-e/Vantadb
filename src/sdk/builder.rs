use crate::agentic::thread::ThreadStore;
use crate::config::VantaConfig;
use crate::error::{Result, VantaError};
use crate::graphrag::pipeline::{GraphRagPipeline, GraphRagResult};
use crate::index::set_prefetch_mode;
use crate::storage::StorageEngine;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing;

/// Stable embedded database handle used by SDKs and bindings.
#[derive(Clone)]
pub struct VantaEmbedded {
    engine: Arc<RwLock<Option<Arc<StorageEngine>>>>,
    pub(crate) config: VantaConfig,
}

impl std::fmt::Debug for VantaEmbedded {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let is_open = self.engine.read().is_some();
        f.debug_struct("VantaEmbedded")
            .field("config", &self.config)
            .field("is_open", &is_open)
            .finish()
    }
}

impl VantaEmbedded {
    /// Wrap an existing engine handle in a VantaEmbedded instance.
    /// Copies the engine's config for use as the embedded config.
    #[tracing::instrument(skip(engine))]
    pub fn from_engine(engine: Arc<StorageEngine>) -> Self {
        let config = engine.config.clone();
        Self {
            engine: Arc::new(RwLock::new(Some(engine))),
            config,
        }
    }

    /// Open a VantaDB database at the given path with default configuration.
    ///
    /// # Examples
    ///
    /// Opens a persistent database in a temporary directory. The directory is
    /// removed after the engine is closed so the example leaves no files behind.
    ///
    /// ```rust
    /// use vantadb::VantaEmbedded;
    ///
    /// let path = std::env::temp_dir().join(format!(
    ///     "vantadb-open-example-{}",
    ///     std::process::id()
    /// ));
    /// let db = VantaEmbedded::open(&path).expect("open database");
    /// db.close().expect("close database");
    /// let _ = std::fs::remove_dir_all(&path);
    /// ```
    #[tracing::instrument(skip(path), err)]
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let config = VantaConfig {
            storage_path: path.as_ref().to_string_lossy().into_owned(),
            ..Default::default()
        };
        Self::open_with_config(config)
    }

    /// Open a VantaDB database with a fully custom configuration.
    ///
    /// # Examples
    ///
    /// Opens an in-memory database by setting `BackendKind::InMemory` as the
    /// backend and `":memory:"` as the storage path:
    ///
    /// ```rust
    /// use vantadb::config::VantaConfig;
    /// use vantadb::{BackendKind, VantaEmbedded};
    ///
    /// let config = VantaConfig {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// };
    /// let db = VantaEmbedded::open_with_config(config).expect("open database");
    /// db.close().expect("close database");
    /// ```
    #[tracing::instrument(skip(config), err)]
    pub fn open_with_config(config: VantaConfig) -> Result<Self> {
        let final_config = config.clone();
        set_prefetch_mode(config.prefetch_mode);

        let engine = StorageEngine::open_with_config(
            &final_config.storage_path,
            Some(final_config.clone()),
        )?;
        let embedded = Self {
            engine: Arc::new(RwLock::new(Some(Arc::new(engine)))),
            config: final_config,
        };
        if !embedded.config.read_only {
            embedded.ensure_indexes_current()?;
        }
        Ok(embedded)
    }

    pub(crate) fn engine_handle(&self) -> Result<Arc<StorageEngine>> {
        self.engine.read().clone().ok_or(VantaError::NotInitialized)
    }

    /// Create an empty handle (no engine) for tests.
    /// Produces `NotInitialized` errors on any engine-dependent operation.
    #[doc(hidden)]
    pub fn test_empty(config: VantaConfig) -> Self {
        Self {
            engine: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// Run the GraphRAG pipeline: seed → expand → retrieve → generate context.
    ///
    /// Uses the default pipeline configuration (seed_k=10, hops=2, max=100, top_k=20).
    /// For custom settings, construct [`GraphRagPipeline`] directly.
    pub fn graphrag_search(
        &self,
        namespace: &str,
        query: Option<&str>,
        query_vector: Option<&[f32]>,
    ) -> Result<GraphRagResult> {
        let pipeline = GraphRagPipeline::new();
        pipeline.search(self, namespace, query, query_vector)
    }

    // ── Agentic Threads ──

    /// Create a new conversation thread.
    ///
    /// Returns the thread's numeric ID. Pass `ttl_secs` for auto-expiry.
    pub fn create_thread(&self, title: &str, ttl_secs: Option<u64>) -> Result<u128> {
        let engine = self.engine_handle()?;
        let store = ThreadStore::new(&engine);
        store.create_thread(title, HashMap::new(), ttl_secs, None)
    }

    /// Append a message to a thread.
    pub fn send_message(&self, thread_id: u128, role: &str, content: &str) -> Result<()> {
        let engine = self.engine_handle()?;
        let store = ThreadStore::new(&engine);
        store.send_message(thread_id, role, content, HashMap::new(), None)
    }

    /// Retrieve a thread by its ID.
    pub fn get_thread(&self, thread_id: u128) -> Result<Option<crate::agentic::MessageThread>> {
        let engine = self.engine_handle()?;
        let store = ThreadStore::new(&engine);
        store.get_thread(thread_id)
    }

    /// List threads with pagination.
    pub fn list_threads(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<crate::agentic::MessageThread>> {
        let engine = self.engine_handle()?;
        let store = ThreadStore::new(&engine);
        store.list_threads(limit, offset)
    }

    /// Delete a thread by its ID.
    pub fn delete_thread(&self, thread_id: u128) -> Result<()> {
        let engine = self.engine_handle()?;
        let store = ThreadStore::new(&engine);
        store.delete_thread(thread_id)
    }

    /// Purge threads whose TTL has expired.
    ///
    /// Returns the number of threads removed.
    pub fn purge_expired_threads(&self) -> Result<usize> {
        let engine = self.engine_handle()?;
        let store = ThreadStore::new(&engine);
        store.purge_expired_threads()
    }

    /// Recover archived (shadow-archived) nodes that belonged to a summary node.
    ///
    /// Scans the TombstoneStorage partition for nodes with a `belonged_to`
    /// edge targeting `summary_id`, re-activates them, and inserts them
    /// back into the active store.
    #[tracing::instrument(skip(self), err)]
    pub fn recover_archived_nodes(
        &self,
        summary_id: u128,
    ) -> Result<Vec<crate::sdk::VantaNodeRecord>> {
        let engine = self.engine_handle()?;
        let nodes = engine.recover_archived_nodes(summary_id)?;
        Ok(nodes
            .into_iter()
            .map(|n| engine.node_to_record(n))
            .collect())
    }

    /// Flush and close the embedded engine handle.
    #[tracing::instrument(skip(self), err)]
    pub fn close(&self) -> Result<()> {
        if let Err(e) = self.flush() {
            tracing::warn!("flush failed: {e}");
        }
        let mut guard = self.engine.write();
        *guard = None;
        Ok(())
    }

    // ── Filesystem Snapshots ──

    /// Create an instant filesystem snapshot via hard links (Unix) or copy (Windows).
    ///
    /// All data files in the storage directory are hard-linked into
    /// `<data_dir>/snapshots/<name>`, giving an O(1) point-in-time image.
    pub fn create_snapshot(&self, name: &str) -> Result<crate::storage::FsSnapshot> {
        let engine = self.engine_handle()?;
        engine.create_snapshot(name)
    }

    /// List all existing snapshot names.
    pub fn list_snapshots(&self) -> Result<Vec<String>> {
        let engine = self.engine_handle()?;
        engine.list_snapshots()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_empty_embedded() -> VantaEmbedded {
        VantaEmbedded::test_empty(VantaConfig::default())
    }

    // ── Debug ──

    #[test]
    fn test_debug_impl_closed() {
        let e = make_empty_embedded();
        let d = format!("{:?}", e);
        assert!(d.contains("VantaEmbedded"), "got: {d}");
        assert!(d.contains("is_open"), "got: {d}");
        assert!(d.contains("false"), "got: {d}");
    }

    #[test]
    fn test_debug_impl_contains_config() {
        let e = make_empty_embedded();
        let d = format!("{:?}", e);
        assert!(d.contains("config"), "got: {d}");
    }

    // ── engine_handle ──

    #[test]
    fn test_engine_handle_none_errors() {
        let e = make_empty_embedded();
        let result = e.engine_handle();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("initialized"), "got: {err}");
    }

    // ── close ──

    #[test]
    fn test_close_on_empty_ok() {
        let e = make_empty_embedded();
        // close on an already-None engine should not panic
        assert!(e.close().is_ok());
    }

    #[test]
    fn test_close_then_engine_handle_fails() {
        let e = make_empty_embedded();
        let _ = e.close();
        let result = e.engine_handle();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("initialized"), "got: {err}");
    }

    // ── VantaConfig defaults used by builder ──

    #[test]
    fn test_default_config_values() {
        let cfg = VantaConfig::default();
        assert!(!cfg.read_only);
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.host, "127.0.0.1");
    }

    // ── recover_archived_nodes ──

    #[test]
    fn test_recover_archived_nodes_empty() {
        let dir = tempfile::tempdir().unwrap();
        let embedded = VantaEmbedded::open(dir.path()).unwrap();
        let result = embedded.recover_archived_nodes(42);
        assert!(
            result.is_ok(),
            "recover_archived_nodes should succeed on empty DB"
        );
        let nodes = result.unwrap();
        assert!(nodes.is_empty(), "no archived nodes to recover");
    }

    #[test]
    fn test_recover_archived_nodes_with_data() {
        let dir = tempfile::tempdir().unwrap();
        let embedded = VantaEmbedded::open(dir.path()).unwrap();
        let engine = embedded.engine_handle().unwrap();

        // Insert an archived node directly into TombstoneStorage
        let belonged_to_id = engine.intern_label("belonged_to");
        let mut archived = crate::node::UnifiedNode::new(100);
        archived.edges.push(crate::node::Edge {
            target: 1,
            label_id: belonged_to_id,
            weight: 1.0,
            reverse: false,
        });
        let data = postcard::to_allocvec(&archived)
            .map_err(|e| format!("serialization: {e}"))
            .unwrap();
        engine
            .put_to_partition(
                crate::storage::BackendPartition::TombstoneStorage,
                b"archived_100",
                &data,
            )
            .expect("put archived node");

        // Recover via the SDK method
        let nodes = embedded.recover_archived_nodes(1).unwrap();
        assert_eq!(nodes.len(), 1, "should recover 1 node");
        assert_eq!(nodes[0].id, 100, "recovered node id should match");
    }
}
