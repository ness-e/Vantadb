use crate::config::VantaConfig;
use crate::error::{Result, VantaError};
use crate::graphrag::pipeline::{GraphRagPipeline, GraphRagResult};
use crate::index::set_prefetch_mode;
use crate::storage::StorageEngine;
use parking_lot::RwLock;
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
    #[tracing::instrument(skip(path), err)]
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let config = VantaConfig {
            storage_path: path.as_ref().to_string_lossy().into_owned(),
            ..Default::default()
        };
        Self::open_with_config(config)
    }

    /// Open a VantaDB database with a fully custom configuration.
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
}
