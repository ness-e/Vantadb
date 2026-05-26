use crate::backend::BackendKind;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    Always,
    #[default]
    Periodic,
    Never,
}

/// Unified configuration for VantaDB.
///
/// Consolidates engine, LLM, and server settings. Loads from environment
/// variables with sensible defaults and allows programmatic overrides.
#[derive(Debug, Clone)]
pub struct VantaConfig {
    pub storage_path: String,
    pub host: String,
    pub port: u16,
    pub llm_url: String,
    pub llm_model: String,
    pub llm_summarize_model: String,
    pub memory_limit: Option<u64>,
    pub read_only: bool,
    pub force_mmap: bool,
    pub backend_kind: BackendKind,
    pub max_blocking_threads: usize,
    pub sync_mode: SyncMode,
}

impl Default for VantaConfig {
    fn default() -> Self {
        Self {
            storage_path: env::var("VANTADB_STORAGE_PATH")
                .unwrap_or_else(|_| "vantadb_data".to_string()),
            host: env::var("VANTADB_HOST")
                .or_else(|_| env::var("HOST"))
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("VANTADB_PORT")
                .or_else(|_| env::var("PORT"))
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            llm_url: env::var("VANTA_LLM_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string()),
            llm_model: env::var("VANTA_LLM_MODEL").unwrap_or_else(|_| "all-minilm".to_string()),
            llm_summarize_model: env::var("VANTA_LLM_SUMMARIZE_MODEL")
                .unwrap_or_else(|_| "llama3".to_string()),
            memory_limit: None,
            read_only: false,
            force_mmap: false,
            backend_kind: BackendKind::Fjall,
            max_blocking_threads: env::var("VANTADB_MAX_BLOCKING_THREADS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(16),
            sync_mode: SyncMode::default(),
        }
    }
}

impl VantaConfig {
    /// Creates a configuration from environment variables.
    pub fn from_env() -> Self {
        Self::default()
    }

    /// Overrides the storage path.
    pub fn with_storage_path(mut self, path: String) -> Self {
        self.storage_path = path;
        self
    }

    /// Overrides the memory limit.
    pub fn with_memory_limit(mut self, limit: u64) -> Self {
        self.memory_limit = Some(limit);
        self
    }

    /// Sets the engine to read-only mode.
    pub fn with_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Forces the use of MMap for the vector index.
    pub fn with_force_mmap(mut self, force: bool) -> Self {
        self.force_mmap = force;
        self
    }

    /// Selects the KV backend.
    pub fn with_backend(mut self, kind: BackendKind) -> Self {
        self.backend_kind = kind;
        self
    }

    /// Sets the maximum number of blocking threads.
    pub fn with_max_blocking_threads(mut self, max: usize) -> Self {
        self.max_blocking_threads = max;
        self
    }

    /// Sets the sync mode.
    pub fn with_sync_mode(mut self, sync_mode: SyncMode) -> Self {
        self.sync_mode = sync_mode;
        self
    }
}
