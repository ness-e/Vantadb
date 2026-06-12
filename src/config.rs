use crate::backend::BackendKind;
#[cfg(feature = "advanced-tokenizer")]
use crate::tokenizer::AdvancedTokenizerConfig;
use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    /// Forces fsync/fdatasync on every write operation to the WAL and storage backend.
    ///
    /// WARNING: This mode guarantees maximum durability (ACID compliance) but carries a
    /// critical performance penalty. On standard SATA SSDs and HDDs, it can degrade write
    /// throughput by 10x to 100x (latencies up to 10-100ms per write) compared to `Periodic`.
    /// Only use this mode for transactional workloads where data durability is strictly
    /// prioritized over ingest speed (e.g., financial transactions).
    Always,
    /// Flushes data periodically to disk (default). Combines high throughput with
    /// reasonable durability guarantees.
    #[default]
    Periodic,
    /// Disables explicit flushing to disk. Relies entirely on the OS page cache.
    /// Provides maximum performance but risks losing the last few writes in case of a crash.
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
    /// Optional Bearer token for HTTP API authentication.
    ///
    /// When set via `VANTADB_API_KEY`, the server requires
    /// `Authorization: Bearer <token>` on all protected endpoints.
    /// If `None`, the server runs without authentication (development mode).
    pub api_key: Option<String>,
    /// Maximum HTTP requests per minute per remote IP for the rate limiter.
    ///
    /// Configured via `VANTADB_RATE_LIMIT_RPM`. Set to `0` to disable rate
    /// limiting entirely (useful for tests and embedded-local usage).
    pub rate_limit_rpm: u32,
    /// Path to the PEM-encoded TLS certificate file.
    ///
    /// Requires the `tls` feature. Configured via `VANTADB_TLS_CERT`.
    /// If `None` while the `tls` feature is active, the server falls back
    /// to plain HTTP and logs a warning.
    pub tls_cert_path: Option<String>,
    /// Path to the PEM-encoded TLS private key file.
    ///
    /// Requires the `tls` feature. Configured via `VANTADB_TLS_KEY`.
    pub tls_key_path: Option<String>,
    #[cfg(feature = "advanced-tokenizer")]
    pub advanced_tokenizer_config: Option<AdvancedTokenizerConfig>,
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
            backend_kind: match env::var("VANTA_BACKEND").ok().as_deref() {
                Some("rocksdb") => BackendKind::RocksDb,
                Some("memory") => BackendKind::InMemory,
                _ => BackendKind::Fjall,
            },
            max_blocking_threads: env::var("VANTADB_MAX_BLOCKING_THREADS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(16),
            sync_mode: SyncMode::default(),
            api_key: env::var("VANTADB_API_KEY").ok(),
            rate_limit_rpm: env::var("VANTADB_RATE_LIMIT_RPM")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            tls_cert_path: env::var("VANTADB_TLS_CERT").ok(),
            tls_key_path: env::var("VANTADB_TLS_KEY").ok(),
            #[cfg(feature = "advanced-tokenizer")]
            advanced_tokenizer_config: None,
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

    /// Sets the API key for Bearer token authentication.
    ///
    /// When `None`, the server runs in unauthenticated mode.
    pub fn with_api_key(mut self, key: Option<String>) -> Self {
        self.api_key = key;
        self
    }

    /// Sets the rate limit in requests per minute per IP.
    ///
    /// Use `0` to disable rate limiting.
    pub fn with_rate_limit_rpm(mut self, rpm: u32) -> Self {
        self.rate_limit_rpm = rpm;
        self
    }

    /// Sets the TLS certificate and key paths for HTTPS.
    ///
    /// Requires the `tls` feature to have any effect.
    pub fn with_tls(mut self, cert_path: String, key_path: String) -> Self {
        self.tls_cert_path = Some(cert_path);
        self.tls_key_path = Some(key_path);
        self
    }

    /// Sets the advanced tokenizer configuration for multilingual text processing.
    ///
    /// Requires the `advanced-tokenizer` feature to have any effect.
    #[cfg(feature = "advanced-tokenizer")]
    pub fn with_advanced_tokenizer_config(
        mut self,
        config: Option<AdvancedTokenizerConfig>,
    ) -> Self {
        self.advanced_tokenizer_config = config;
        self
    }
}
