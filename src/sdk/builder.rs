use parking_lot::RwLock;
use std::path::Path;
use std::sync::Arc;
use tracing;
use crate::config::VantaConfig;
use crate::error::{Result, VantaError};
use crate::index::set_prefetch_mode;
use crate::storage::StorageEngine;


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
