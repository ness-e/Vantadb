//! Configuration system for VantaDB engine.
//!
//! Defines [`VantaConfig`] with typed fields, environment variable parsing,
//! and per-backend configuration options with fallback defaults.

use crate::backend::BackendKind;
#[cfg(feature = "advanced-tokenizer")]
use crate::tokenizer::AdvancedTokenizerConfig;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
#[cfg(feature = "hot-reload")]
use std::sync::{Arc, RwLock};
use tracing::debug;
use tracing::warn;

const DEFAULT_RSS_THRESHOLD: f64 = 0.80;

/// Log output format for the VantaDB server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Compact human-readable output (default for CLI). No targets, thread IDs,
    /// or file/line info. ANSI colors enabled for terminals.
    #[default]
    Compact,
    /// JSON-structured output (default for server). Full metadata: ISO 8601
    /// timestamp, level, target, file, line, thread_id. Suitable for log
    /// aggregators (Datadog, ELK, Grafana Loki).
    Json,
    /// Full human-readable format with targets, file/line, thread IDs, ANSI
    /// colors. Useful for debugging.
    Full,
}

impl LogFormat {
    /// Parse from an env var value (case-insensitive).
    pub fn from_env_value(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "json" | "1" | "true" => LogFormat::Json,
            "full" | "verbose" => LogFormat::Full,
            _ => LogFormat::Compact,
        }
    }
}

/// Synchronisation mode for WAL and storage writes.
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

/// Controls whether mmap vector prefetching is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrefetchMode {
    /// Default — prefetch enabled (backward compatible).
    /// In the future, may auto-detect NVMe vs HDD.
    #[default]
    Auto,
    /// Force prefetch on regardless of storage type.
    Enabled,
    /// Disable prefetch entirely (avoids syscall overhead on fast NVMe).
    Disabled,
}

impl PrefetchMode {
    /// Parse from an env var value (case-insensitive).
    pub fn from_env_value(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "disabled" | "off" | "0" | "false" => PrefetchMode::Disabled,
            "enabled" | "on" | "1" | "true" => PrefetchMode::Enabled,
            _ => PrefetchMode::Auto,
        }
    }

    /// Returns `true` if prefetch is active for this mode.
    pub fn is_prefetch_enabled(self) -> bool {
        match self {
            PrefetchMode::Disabled => false,
            PrefetchMode::Auto | PrefetchMode::Enabled => true,
        }
    }
}

/// RBAC configuration mapping API tokens to roles.
#[derive(Debug, Clone, Default)]
pub struct RbacConfig {
    /// Map of token values to role names.
    pub token_role_map: HashMap<String, String>,
}

/// Subset of [`VantaConfig`] fields that are safe to modify at runtime.
///
/// Fields that change storage layout, backend, or security posture are excluded.
/// Only tuning knobs and log-level controls are reloaded.
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Prefetch mode — safe to toggle between on/off.
    pub prefetch_mode: PrefetchMode,
    /// Log output format — can be changed at runtime for debugging.
    pub log_format: LogFormat,
    /// Rate limit in requests per minute per IP (0 = disabled).
    pub rate_limit_rpm: u32,
    /// Batch size for batch ingestion operations.
    pub batch_size: Option<usize>,
    /// WAL buffer size in bytes.
    pub wal_buffer_size: Option<usize>,
    /// Number of nodes before triggering implicit WAL flush.
    pub flush_threshold: Option<usize>,
    /// Timeout for acquiring the insert spin-lock (ms).
    pub insert_lock_timeout_ms: u64,
    /// Sync mode for durability vs throughput.
    pub sync_mode: SyncMode,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            prefetch_mode: PrefetchMode::Auto,
            log_format: LogFormat::Compact,
            rate_limit_rpm: 100,
            batch_size: None,
            wal_buffer_size: None,
            flush_threshold: None,
            insert_lock_timeout_ms: 2000,
            sync_mode: SyncMode::Periodic,
        }
    }
}

impl HotReloadConfig {
    /// Load hot-reloadable subset from a [`VantaConfig`].
    pub fn from_config(cfg: &VantaConfig) -> Self {
        Self {
            prefetch_mode: cfg.prefetch_mode,
            log_format: cfg.log_format,
            rate_limit_rpm: cfg.rate_limit_rpm,
            batch_size: cfg.batch_size,
            wal_buffer_size: cfg.wal_buffer_size,
            flush_threshold: cfg.flush_threshold,
            insert_lock_timeout_ms: cfg.insert_lock_timeout_ms,
            sync_mode: cfg.sync_mode,
        }
    }

    /// Refresh `VantaConfig` fields from this hot-reload snapshot.
    ///
    /// Returns `true` if at least one field changed.
    pub fn apply_to(&self, target: &mut VantaConfig) -> bool {
        let mut changed = false;
        macro_rules! update {
            ($field:ident) => {
                if target.$field != self.$field {
                    target.$field = self.$field.clone();
                    changed = true;
                }
            };
        }
        update!(prefetch_mode);
        update!(log_format);
        update!(rate_limit_rpm);
        update!(batch_size);
        update!(wal_buffer_size);
        update!(flush_threshold);
        update!(insert_lock_timeout_ms);
        update!(sync_mode);
        changed
    }
}

/// Unified configuration for VantaDB.
///
/// Consolidates engine, LLM, and server settings. Loads from environment
/// variables with sensible defaults and allows programmatic overrides.
#[derive(Debug, Clone)]
pub struct VantaConfig {
    /// Directory path for persistent storage.
    pub storage_path: String,
    /// Host address to bind the HTTP server.
    pub host: String,
    /// Port number for the HTTP server.
    pub port: u16,
    /// Base URL for the LLM inference endpoint.
    pub llm_url: String,
    /// Model name for LLM inference.
    pub llm_model: String,
    /// Model name for LLM summarisation.
    pub llm_summarize_model: String,
    /// Optional memory limit in bytes.
    pub memory_limit: Option<u64>,
    /// If true, the engine operates in read-only mode.
    pub read_only: bool,
    /// If true, force mmap-based vector storage.
    pub force_mmap: bool,
    /// Whether to use memory-mapped storage for the HNSW index.
    /// When true, vectors in the HNSW index use zero-copy MmapFull representations
    /// instead of heap-allocated Full vectors. Enabled automatically on systems
    /// with <16GB RAM or LowResource profile. Set `force_mmap` to override.
    pub mmap_hnsw: bool,
    /// Prefetch mode for mmap vector pages during HNSW search.
    /// Controls whether `madvise(MADV_WILLNEED)` / `PrefetchVirtualMemory`
    /// is issued for unvisited neighbor pages in the hot search loop.
    /// Default: `Auto` (prefetch enabled, backward compatible).
    pub prefetch_mode: PrefetchMode,
    /// RSS threshold (0.0–1.0) that triggers backpressure rejection.
    /// When the effective memory usage exceeds this fraction of the memory limit,
    /// write operations return `VantaError::ResourceLimit`.
    /// Set to 0.0 to disable backpressure entirely.
    pub rss_threshold: f64,
    /// Weight for hit count in eviction scoring (default: 1.0).
    pub eviction_weight_hits: f64,
    /// Weight for confidence score in eviction scoring (default: 2.0).
    pub eviction_weight_confidence: f64,
    /// Weight for importance score in eviction scoring (default: 3.0).
    pub eviction_weight_importance: f64,
    /// Weight for recency (last_accessed) in eviction scoring (default: 1.0).
    pub eviction_weight_recency: f64,
    /// Fraction of hot nodes to evict when memory pressure triggers (default: 0.20).
    pub eviction_ratio: f64,
    /// The chosen key-value storage backend.
    pub backend_kind: BackendKind,
    /// Maximum number of blocking threads for the async runtime.
    pub max_blocking_threads: usize,
    /// Write synchronisation mode for durability vs. throughput.
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
    /// Batch size for batch ingestion operations (default: 1000).
    /// Configured via `VANTADB_BATCH_SIZE`.
    pub batch_size: Option<usize>,
    /// WAL buffer size in bytes (default: 65536 / 64KB).
    /// Configured via `VANTADB_WAL_BUFFER_SIZE`.
    pub wal_buffer_size: Option<usize>,
    /// Number of nodes before triggering implicit WAL flush (default: 10000).
    /// Configured via `VANTADB_FLUSH_THRESHOLD`.
    pub flush_threshold: Option<usize>,
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
    /// Log output format. Configured via `VANTADB_LOG_FORMAT` env var.
    /// Values: `compact` (default), `json`, `full`.
    /// Also respects legacy `VANTADB_LOG_JSON=1/true` for backward compat.
    pub log_format: LogFormat,
    /// Timeout for acquiring the insert spin-lock (default: 2000 ms).
    /// Configured via `VANTADB_INSERT_LOCK_TIMEOUT_MS`.
    pub insert_lock_timeout_ms: u64,
    /// Timeout for acquiring the process-level file lock (default: 1000 ms).
    /// Configured via `VANTADB_FILE_LOCK_TIMEOUT_MS`.
    pub file_lock_timeout_ms: u64,
    #[cfg(feature = "advanced-tokenizer")]
    /// Advanced tokenizer configuration for multilingual text processing.
    pub advanced_tokenizer_config: Option<AdvancedTokenizerConfig>,
    /// RBAC configuration mapping API tokens to roles.
    pub rbac_config: RbacConfig,
    /// Optional AES-256-GCM encryption key (hex-encoded 32-byte value).
    ///
    /// When set, storage files are transparently encrypted at rest using this
    /// key. Requires the `encryption` feature. Configured via
    /// `VANTADB_ENCRYPTION_KEY` environment variable.
    pub encryption_key: Option<String>,
    /// Number of WAL shards for reduced mutex contention (default: 4).
    /// Each shard has its own append lock; workloads hash node IDs across shards.
    /// Set to 0 to disable WAL, or 1 for single-file (legacy) behaviour.
    /// Configured via `VANTADB_WAL_SHARDS`.
    pub wal_shards: usize,
    /// Hot-reloadable config snapshot.
    ///
    /// When `cfg(feature = "hot-reload")` is enabled, a background watcher
    /// thread monitors the config file and atomically swaps this value.
    /// Read via [`VantaConfig::hot_reload()`].
    #[cfg(feature = "hot-reload")]
    pub hot_reload_config: Arc<RwLock<HotReloadConfig>>,
}

/// Parse an environment variable with a fallback default.
fn parse_env_or<T: FromStr>(key: &str, default: T) -> T {
    match env::var(key) {
        Ok(val) => match val.parse::<T>() {
            Ok(v) => v,
            Err(_) => {
                warn!("Invalid value for {}: \"{}\" — using default", key, val);
                default
            }
        },
        Err(env::VarError::NotPresent) => default,
        Err(env::VarError::NotUnicode(_)) => {
            warn!("Non-Unicode value for {} — using default", key);
            default
        }
    }
}

impl Default for VantaConfig {
    fn default() -> Self {
        Self {
            storage_path: {
                let v =
                    env::var("VANTADB_STORAGE_PATH").unwrap_or_else(|_| "vantadb_data".to_string());
                debug!(val = %v, "VANTADB_STORAGE_PATH");
                v
            },
            host: {
                let v = env::var("VANTADB_HOST")
                    .or_else(|_| env::var("HOST"))
                    .unwrap_or_else(|_| "127.0.0.1".to_string());
                debug!(val = %v, "VANTADB_HOST");
                v
            },
            port: {
                let v = parse_env_or("VANTADB_PORT", 8080u16);
                debug!(val = v, "VANTADB_PORT");
                v
            },
            llm_url: {
                let v = env::var("VANTA_LLM_URL")
                    .unwrap_or_else(|_| "http://localhost:11434".to_string());
                debug!(val = %v, "VANTA_LLM_URL");
                v
            },
            llm_model: {
                let v = env::var("VANTA_LLM_MODEL").unwrap_or_else(|_| "all-minilm".to_string());
                debug!(val = %v, "VANTA_LLM_MODEL");
                v
            },
            llm_summarize_model: {
                let v =
                    env::var("VANTA_LLM_SUMMARIZE_MODEL").unwrap_or_else(|_| "llama3".to_string());
                debug!(val = %v, "VANTA_LLM_SUMMARIZE_MODEL");
                v
            },
            memory_limit: None,
            read_only: false,
            force_mmap: false,
            mmap_hnsw: true,
            prefetch_mode: {
                let raw = env::var("VANTA_PREFETCH").ok();
                let mode = raw.as_deref().map(PrefetchMode::from_env_value);
                let disable = env::var("VANTA_DISABLE_PREFETCH")
                    .ok()
                    .map(|v| v == "1" || v == "true");
                let v = match (mode, disable) {
                    (Some(m), _) => {
                        if let Some(ref val) = raw {
                            let trimmed = val.trim().to_lowercase();
                            let known = [
                                "auto", "disabled", "off", "0", "false", "enabled", "on", "1",
                                "true",
                            ];
                            if m == PrefetchMode::Auto && !known.contains(&trimmed.as_str()) {
                                warn!(
                                    "Unrecognized VANTA_PREFETCH=\"{}\" — expected \"enabled\", \"disabled\", or \"auto\". Using default: Auto",
                                    val
                                );
                            }
                        }
                        m
                    }
                    (_, Some(true)) => PrefetchMode::Disabled,
                    _ => PrefetchMode::Auto,
                };
                debug!(?v, "VANTA_PREFETCH");
                v
            },
            rss_threshold: DEFAULT_RSS_THRESHOLD,
            eviction_weight_hits: 1.0,
            eviction_weight_confidence: 2.0,
            eviction_weight_importance: 3.0,
            eviction_weight_recency: 1.0,
            eviction_ratio: 0.20,
            backend_kind: {
                let v = match env::var("VANTA_BACKEND").ok().as_deref() {
                    Some("rocksdb") => BackendKind::RocksDb,
                    Some("memory") => BackendKind::InMemory,
                    Some("fjall") => BackendKind::Fjall,
                    Some(other) => {
                        warn!(
                            "Unrecognized VANTA_BACKEND=\"{}\" — expected \"rocksdb\" or \"memory\". Using default: Fjall",
                            other
                        );
                        BackendKind::Fjall
                    }
                    None => BackendKind::Fjall,
                };
                debug!(?v, "VANTA_BACKEND");
                v
            },
            max_blocking_threads: {
                let default = std::thread::available_parallelism()
                    .map(|n| n.get() * 2)
                    .unwrap_or(16);
                let v = parse_env_or("VANTADB_MAX_BLOCKING_THREADS", default);
                debug!(val = v, "VANTADB_MAX_BLOCKING_THREADS");
                v
            },
            sync_mode: SyncMode::default(),
            insert_lock_timeout_ms: {
                let v = parse_env_or("VANTADB_INSERT_LOCK_TIMEOUT_MS", 2000u64);
                debug!(val = v, "VANTADB_INSERT_LOCK_TIMEOUT_MS");
                v
            },
            file_lock_timeout_ms: {
                let v = parse_env_or("VANTADB_FILE_LOCK_TIMEOUT_MS", 1000u64);
                debug!(val = v, "VANTADB_FILE_LOCK_TIMEOUT_MS");
                v
            },
            api_key: {
                let v = env::var("VANTADB_API_KEY").ok();
                debug!(present = v.is_some(), "VANTADB_API_KEY");
                v
            },
            rate_limit_rpm: {
                let v = parse_env_or("VANTADB_RATE_LIMIT_RPM", 100u32);
                debug!(val = v, "VANTADB_RATE_LIMIT_RPM");
                v
            },
            batch_size: {
                let v = parse_env_or::<u32>("VANTADB_BATCH_SIZE", 0)
                    .try_into()
                    .ok()
                    .and_then(|n: usize| if n > 0 { Some(n) } else { None });
                debug!(val = ?v, "VANTADB_BATCH_SIZE");
                v
            },
            wal_buffer_size: {
                let v = parse_env_or::<u32>("VANTADB_WAL_BUFFER_SIZE", 0)
                    .try_into()
                    .ok()
                    .and_then(|n: usize| if n > 0 { Some(n) } else { None });
                debug!(val = ?v, "VANTADB_WAL_BUFFER_SIZE");
                v
            },
            flush_threshold: {
                let v = parse_env_or::<u32>("VANTADB_FLUSH_THRESHOLD", 0)
                    .try_into()
                    .ok()
                    .and_then(|n: usize| if n > 0 { Some(n) } else { None });
                debug!(val = ?v, "VANTADB_FLUSH_THRESHOLD");
                v
            },
            tls_cert_path: {
                let v = env::var("VANTADB_TLS_CERT").ok();
                debug!(present = v.is_some(), "VANTADB_TLS_CERT");
                v
            },
            tls_key_path: {
                let v = env::var("VANTADB_TLS_KEY").ok();
                debug!(present = v.is_some(), "VANTADB_TLS_KEY");
                v
            },
            log_format: {
                let v = {
                    let legacy = env::var("VANTADB_LOG_JSON")
                        .map(|v| v == "1" || v == "true")
                        .unwrap_or(false);
                    if legacy {
                        LogFormat::Json
                    } else {
                        match env::var("VANTADB_LOG_FORMAT") {
                            Ok(raw) => {
                                let trimmed = raw.trim().to_lowercase();
                                let parsed = LogFormat::from_env_value(&raw);
                                if parsed == LogFormat::Compact && trimmed != "compact" {
                                    warn!(
                                        "Unrecognized VANTADB_LOG_FORMAT=\"{}\" — expected \"compact\", \"json\", or \"full\". Using default: Compact",
                                        raw
                                    );
                                }
                                parsed
                            }
                            Err(_) => LogFormat::Compact,
                        }
                    }
                };
                debug!(?v, "VANTADB_LOG_FORMAT");
                v
            },
            #[cfg(feature = "advanced-tokenizer")]
            advanced_tokenizer_config: None,
            encryption_key: {
                let v = env::var("VANTADB_ENCRYPTION_KEY").ok();
                if v.is_some() {
                    debug!("VANTADB_ENCRYPTION_KEY is set (value not logged)");
                }
                v
            },
            wal_shards: parse_env_or("VANTADB_WAL_SHARDS", 4usize),
            rbac_config: RbacConfig::default(),
            #[cfg(feature = "hot-reload")]
            hot_reload_config: Arc::new(RwLock::new(HotReloadConfig::default())),
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

    /// Sets whether to use memory-mapped HNSW index storage.
    pub fn with_mmap_hnsw(mut self, val: bool) -> Self {
        self.mmap_hnsw = val;
        self
    }

    /// Sets the RSS threshold that triggers backpressure (0.0–1.0).
    /// Set to 0.0 to disable.
    pub fn with_rss_threshold(mut self, threshold: f64) -> Self {
        self.rss_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Sets eviction scoring weights.
    pub fn with_eviction_weights(
        mut self,
        hits: f64,
        confidence: f64,
        importance: f64,
        recency: f64,
    ) -> Self {
        self.eviction_weight_hits = hits;
        self.eviction_weight_confidence = confidence;
        self.eviction_weight_importance = importance;
        self.eviction_weight_recency = recency;
        self
    }

    /// Sets the fraction of hot nodes to evict when memory pressure triggers.
    pub fn with_eviction_ratio(mut self, ratio: f64) -> Self {
        self.eviction_ratio = ratio.clamp(0.0, 1.0);
        self
    }

    /// Returns the eviction weights as a struct.
    pub fn eviction_weights(&self) -> crate::node::EvictionWeights {
        crate::node::EvictionWeights {
            hits: self.eviction_weight_hits,
            confidence: self.eviction_weight_confidence,
            importance: self.eviction_weight_importance,
            recency: self.eviction_weight_recency,
        }
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

    /// Sets the batch size for batch ingestion operations.
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = Some(size);
        self
    }

    /// Sets the WAL buffer size in bytes.
    pub fn with_wal_buffer_size(mut self, size: usize) -> Self {
        self.wal_buffer_size = Some(size);
        self
    }

    /// Sets the number of nodes before triggering an implicit WAL flush.
    pub fn with_flush_threshold(mut self, threshold: usize) -> Self {
        self.flush_threshold = Some(threshold);
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

    /// Sets the log output format.
    pub fn with_log_format(mut self, format: LogFormat) -> Self {
        self.log_format = format;
        self
    }

    /// Sets the encryption key for at-rest AES-256-GCM encryption.
    ///
    /// The key should be a hex-encoded 32-byte (64 hex char) value.
    /// Requires the `encryption` feature to have any effect.
    pub fn with_encryption(mut self, key: String) -> Self {
        self.encryption_key = Some(key);
        self
    }

    /// Sets the RBAC configuration for token-to-role mapping.
    pub fn with_rbac_config(mut self, config: RbacConfig) -> Self {
        self.rbac_config = config;
        self
    }

    /// Sets the prefetch mode for mmap vector pages during HNSW search.
    pub fn with_prefetch_mode(mut self, mode: PrefetchMode) -> Self {
        self.prefetch_mode = mode;
        self
    }

    /// Sets the number of WAL shards for reduced mutex contention.
    pub fn with_wal_shards(mut self, shards: usize) -> Self {
        self.wal_shards = shards;
        self
    }

    /// Returns a reference to the hot-reloadable config snapshot.
    #[cfg(feature = "hot-reload")]
    pub fn hot_reload(&self) -> Arc<RwLock<HotReloadConfig>> {
        Arc::clone(&self.hot_reload_config)
    }

    /// Spawn a background watcher thread that monitors `path` for file changes
    /// and atomically applies reloaded fields to `config`.
    ///
    /// Only safe-to-reload fields (see [`HotReloadConfig`]) are applied.
    /// Changes to storage paths, backend, TLS, or API keys are ignored.
    ///
    /// The thread exits when the returned [`watch::Sender`] is dropped.
    #[cfg(feature = "hot-reload")]
    pub fn watch_config<C: Fn() + Send + 'static>(
        config: Arc<RwLock<Self>>,
        path: impl Into<std::path::PathBuf> + Send + 'static,
        on_reload: C,
    ) -> std::io::Result<std::sync::mpsc::Sender<()>> {
        use notify::event::ModifyKind;
        use notify::{Config, Event, EventKind, RecommendedWatcher, Watcher};
        use std::sync::mpsc;

        let path = path.into();
        let (tx, rx) = mpsc::channel::<()>();

        let mut watcher = RecommendedWatcher::new(
            move |res: notify::Result<Event>| {
                if let Ok(event) = res {
                    if matches!(event.kind, EventKind::Modify(ModifyKind::Data(_))) {
                        let source_path = event.paths.first().cloned();
                        if let Some(source) = source_path {
                            if source == path {
                                // Re-read config from the file
                                let content = match std::fs::read_to_string(&source) {
                                    Ok(c) => c,
                                    Err(e) => {
                                        warn!("hot-reload: failed to read config: {e}");
                                        return;
                                    }
                                };
                                let parsed: serde_json::Value = match serde_json::from_str(&content)
                                {
                                    Ok(v) => v,
                                    Err(e) => {
                                        warn!("hot-reload: failed to parse config: {e}");
                                        return;
                                    }
                                };
                                match apply_hot_reload_from_value(&config, &parsed) {
                                    Ok(changed) => {
                                        if changed {
                                            on_reload();
                                        }
                                    }
                                    Err(e) => {
                                        warn!("hot-reload: failed to apply config: {e}");
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        watcher
            .watch(
                path.parent().unwrap_or(std::path::Path::new(".")),
                notify::RecursiveMode::NonRecursive,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        std::thread::spawn(move || {
            // block until shutdown signal then drop watcher
            let _ = rx.recv();
            drop(watcher);
            debug!("hot-reload: watcher shut down");
        });

        Ok(tx)
    }
}

/// Parse a `serde_json::Value` / `toml::Value` and apply hot-reloadable fields.
#[cfg(feature = "hot-reload")]
fn apply_hot_reload_from_value(
    config: &Arc<RwLock<VantaConfig>>,
    value: &serde_json::Value,
) -> Result<bool, String> {
    use serde_json::Value;

    let mut hot = HotReloadConfig::default();

    macro_rules! set_str_enum {
        ($field:ident, $key:literal, $parser:path) => {
            if let Some(v) = value.get($key).and_then(|v| v.as_str()) {
                let parsed = $parser(v);
                if parsed != $field {
                    $field = parsed;
                }
            }
        };
    }

    macro_rules! set_u32 {
        ($field:ident, $key:literal) => {
            if let Some(v) = value.get($key).and_then(|v| v.as_u64()) {
                let parsed = v as u32;
                if parsed != $field {
                    $field = parsed;
                }
            }
        };
    }

    macro_rules! set_u64 {
        ($field:ident, $key:literal) => {
            if let Some(v) = value.get($key).and_then(|v| v.as_u64()) {
                if v != $field {
                    $field = v;
                }
            }
        };
    }

    macro_rules! set_opt_usize {
        ($field:ident, $key:literal) => {
            if let Some(v) = value.get($key) {
                let parsed = match v {
                    Value::Null => None,
                    Value::Number(n) => n.as_u64().map(|n| n as usize),
                    _ => None,
                };
                if parsed != $field {
                    $field = parsed;
                }
            }
        };
    }

    set_str_enum!(hot.prefetch_mode, "prefetch_mode", PrefetchMode::from_env_value);
    set_str_enum!(hot.log_format, "log_format", LogFormat::from_env_value);
    set_u32!(hot.rate_limit_rpm, "rate_limit_rpm");
    set_opt_usize!(hot.batch_size, "batch_size");
    set_opt_usize!(hot.wal_buffer_size, "wal_buffer_size");
    set_opt_usize!(hot.flush_threshold, "flush_threshold");
    set_u64!(hot.insert_lock_timeout_ms, "insert_lock_timeout_ms");

    // sync_mode is a string in config files
    if let Some(v) = value.get("sync_mode").and_then(|v| v.as_str()) {
        let parsed = match v.to_lowercase().as_str() {
            "always" => SyncMode::Always,
            "never" => SyncMode::Never,
            _ => SyncMode::Periodic,
        };
        if parsed != hot.sync_mode {
            hot.sync_mode = parsed;
        }
    }

    let mut guard = config.write().map_err(|e| e.to_string())?;
    let changed = hot.apply_to(&mut guard);
    if changed {
        debug!("hot-reload: applied config changes");
    }
    Ok(changed)
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;

    // ── LogFormat ──────────────────────────────────────────────

    #[test]
    fn test_log_format_default() {
        assert_eq!(LogFormat::default(), LogFormat::Compact);
    }

    #[test]
    fn test_log_format_from_env_value() {
        assert_eq!(LogFormat::from_env_value("json"), LogFormat::Json);
        assert_eq!(LogFormat::from_env_value("JSON"), LogFormat::Json);
        assert_eq!(LogFormat::from_env_value("1"), LogFormat::Json);
        assert_eq!(LogFormat::from_env_value("true"), LogFormat::Json);
        assert_eq!(LogFormat::from_env_value("full"), LogFormat::Full);
        assert_eq!(LogFormat::from_env_value("verbose"), LogFormat::Full);
        assert_eq!(LogFormat::from_env_value("FULL"), LogFormat::Full);
        assert_eq!(LogFormat::from_env_value("compact"), LogFormat::Compact);
        assert_eq!(LogFormat::from_env_value("garbage"), LogFormat::Compact);
        assert_eq!(LogFormat::from_env_value(""), LogFormat::Compact);
    }

    // ── SyncMode ───────────────────────────────────────────────

    #[test]
    fn test_sync_mode_default() {
        assert_eq!(SyncMode::default(), SyncMode::Periodic);
    }

    // ── PrefetchMode ───────────────────────────────────────────

    #[test]
    fn test_prefetch_mode_default() {
        assert_eq!(PrefetchMode::default(), PrefetchMode::Auto);
    }

    #[test]
    fn test_prefetch_mode_from_env_value() {
        assert_eq!(PrefetchMode::from_env_value("auto"), PrefetchMode::Auto);
        assert_eq!(PrefetchMode::from_env_value("AUTO"), PrefetchMode::Auto);
        assert_eq!(PrefetchMode::from_env_value("unknown"), PrefetchMode::Auto);

        assert_eq!(
            PrefetchMode::from_env_value("disabled"),
            PrefetchMode::Disabled
        );
        assert_eq!(PrefetchMode::from_env_value("off"), PrefetchMode::Disabled);
        assert_eq!(PrefetchMode::from_env_value("0"), PrefetchMode::Disabled);
        assert_eq!(
            PrefetchMode::from_env_value("false"),
            PrefetchMode::Disabled
        );

        assert_eq!(
            PrefetchMode::from_env_value("enabled"),
            PrefetchMode::Enabled
        );
        assert_eq!(PrefetchMode::from_env_value("on"), PrefetchMode::Enabled);
        assert_eq!(PrefetchMode::from_env_value("1"), PrefetchMode::Enabled);
        assert_eq!(PrefetchMode::from_env_value("true"), PrefetchMode::Enabled);
    }

    #[test]
    fn test_prefetch_mode_is_enabled() {
        assert!(PrefetchMode::Auto.is_prefetch_enabled());
        assert!(PrefetchMode::Enabled.is_prefetch_enabled());
        assert!(!PrefetchMode::Disabled.is_prefetch_enabled());
    }

    // ── VantaConfig defaults ───────────────────────────────────

    #[test]
    fn test_vanta_config_default_values() {
        let cfg = VantaConfig::default();
        assert_eq!(cfg.storage_path, "vantadb_data");
        assert_eq!(cfg.host, "127.0.0.1".to_string());
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.llm_url, "http://localhost:11434");
        assert_eq!(cfg.llm_model, "all-minilm");
        assert_eq!(cfg.llm_summarize_model, "llama3");
        assert_eq!(cfg.memory_limit, None);
        assert!(!cfg.read_only);
        assert!(!cfg.force_mmap);
        assert!(cfg.mmap_hnsw);
        assert_eq!(cfg.prefetch_mode, PrefetchMode::Auto);
        assert!((cfg.rss_threshold - DEFAULT_RSS_THRESHOLD).abs() < 1e-9);
        assert!((cfg.eviction_weight_hits - 1.0).abs() < 1e-9);
        assert!((cfg.eviction_weight_confidence - 2.0).abs() < 1e-9);
        assert!((cfg.eviction_weight_importance - 3.0).abs() < 1e-9);
        assert!((cfg.eviction_weight_recency - 1.0).abs() < 1e-9);
        assert!((cfg.eviction_ratio - 0.20).abs() < 1e-9);
        assert_eq!(cfg.backend_kind, BackendKind::Fjall);
        let expected_threads = std::thread::available_parallelism()
            .map(|n| n.get() * 2)
            .unwrap_or(16);
        assert_eq!(cfg.max_blocking_threads, expected_threads);
        assert_eq!(cfg.sync_mode, SyncMode::Periodic);
        assert_eq!(cfg.api_key, None);
        assert_eq!(cfg.rate_limit_rpm, 100);
        assert_eq!(cfg.batch_size, None);
        assert_eq!(cfg.wal_buffer_size, None);
        assert_eq!(cfg.flush_threshold, None);
        assert_eq!(cfg.tls_cert_path, None);
        assert_eq!(cfg.tls_key_path, None);
        assert_eq!(cfg.log_format, LogFormat::Compact);
        assert_eq!(cfg.insert_lock_timeout_ms, 2000);
        assert_eq!(cfg.file_lock_timeout_ms, 1000);
    }

    // ── Builder methods ───────────────────────────────────────

    #[test]
    fn test_with_storage_path() {
        let cfg = VantaConfig::default().with_storage_path("/tmp/vanta".into());
        assert_eq!(cfg.storage_path, "/tmp/vanta");
    }

    #[test]
    fn test_with_memory_limit() {
        let cfg = VantaConfig::default().with_memory_limit(4_096_000_000);
        assert_eq!(cfg.memory_limit, Some(4_096_000_000));
    }

    #[test]
    fn test_with_read_only() {
        let cfg = VantaConfig::default().with_read_only(true);
        assert!(cfg.read_only);
    }

    #[test]
    fn test_with_force_mmap() {
        let cfg = VantaConfig::default().with_force_mmap(true);
        assert!(cfg.force_mmap);
    }

    #[test]
    fn test_with_mmap_hnsw() {
        let cfg = VantaConfig::default().with_mmap_hnsw(false);
        assert!(!cfg.mmap_hnsw);
    }

    #[test]
    fn test_with_rss_threshold() {
        let cfg = VantaConfig::default().with_rss_threshold(0.5);
        assert!((cfg.rss_threshold - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_with_rss_threshold_clamps() {
        let cfg = VantaConfig::default().with_rss_threshold(1.5);
        assert!((cfg.rss_threshold - 1.0).abs() < 1e-9);
        let cfg = VantaConfig::default().with_rss_threshold(-0.5);
        assert!((cfg.rss_threshold - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_with_rss_threshold_zero_disables() {
        let cfg = VantaConfig::default().with_rss_threshold(0.0);
        assert!((cfg.rss_threshold - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_with_eviction_weights() {
        let cfg = VantaConfig::default().with_eviction_weights(0.5, 1.5, 2.5, 3.5);
        assert!((cfg.eviction_weight_hits - 0.5).abs() < 1e-9);
        assert!((cfg.eviction_weight_confidence - 1.5).abs() < 1e-9);
        assert!((cfg.eviction_weight_importance - 2.5).abs() < 1e-9);
        assert!((cfg.eviction_weight_recency - 3.5).abs() < 1e-9);
    }

    #[test]
    fn test_with_eviction_ratio() {
        let cfg = VantaConfig::default().with_eviction_ratio(0.5);
        assert!((cfg.eviction_ratio - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_with_eviction_ratio_clamps() {
        let cfg = VantaConfig::default().with_eviction_ratio(1.5);
        assert!((cfg.eviction_ratio - 1.0).abs() < 1e-9);
        let cfg = VantaConfig::default().with_eviction_ratio(-0.5);
        assert!((cfg.eviction_ratio - 0.0).abs() < 1e-9);
    }

    #[test]
    fn test_eviction_weights_struct() {
        let cfg = VantaConfig::default().with_eviction_weights(0.1, 0.2, 0.3, 0.4);
        let w = cfg.eviction_weights();
        assert!((w.hits - 0.1).abs() < 1e-9);
        assert!((w.confidence - 0.2).abs() < 1e-9);
        assert!((w.importance - 0.3).abs() < 1e-9);
        assert!((w.recency - 0.4).abs() < 1e-9);
    }

    #[test]
    fn test_with_backend() {
        let cfg = VantaConfig::default().with_backend(BackendKind::RocksDb);
        assert_eq!(cfg.backend_kind, BackendKind::RocksDb);
        let cfg = VantaConfig::default().with_backend(BackendKind::InMemory);
        assert_eq!(cfg.backend_kind, BackendKind::InMemory);
    }

    #[test]
    fn test_with_max_blocking_threads() {
        let cfg = VantaConfig::default().with_max_blocking_threads(32);
        assert_eq!(cfg.max_blocking_threads, 32);
    }

    #[test]
    fn test_with_sync_mode() {
        let cfg = VantaConfig::default().with_sync_mode(SyncMode::Always);
        assert_eq!(cfg.sync_mode, SyncMode::Always);
    }

    #[test]
    fn test_with_api_key() {
        let cfg = VantaConfig::default().with_api_key(Some("sk-test".into()));
        assert_eq!(cfg.api_key, Some("sk-test".into()));
        let cfg = VantaConfig::default().with_api_key(None);
        assert_eq!(cfg.api_key, None);
    }

    #[test]
    fn test_with_rate_limit_rpm() {
        let cfg = VantaConfig::default().with_rate_limit_rpm(0);
        assert_eq!(cfg.rate_limit_rpm, 0);
    }

    #[test]
    fn test_with_batch_size() {
        let cfg = VantaConfig::default().with_batch_size(500);
        assert_eq!(cfg.batch_size, Some(500));
    }

    #[test]
    fn test_with_wal_buffer_size() {
        let cfg = VantaConfig::default().with_wal_buffer_size(131072);
        assert_eq!(cfg.wal_buffer_size, Some(131072));
    }

    #[test]
    fn test_with_flush_threshold() {
        let cfg = VantaConfig::default().with_flush_threshold(5000);
        assert_eq!(cfg.flush_threshold, Some(5000));
    }

    #[test]
    fn test_with_tls() {
        let cfg = VantaConfig::default().with_tls("cert.pem".into(), "key.pem".into());
        assert_eq!(cfg.tls_cert_path, Some("cert.pem".into()));
        assert_eq!(cfg.tls_key_path, Some("key.pem".into()));
    }

    #[test]
    fn test_with_log_format() {
        let cfg = VantaConfig::default().with_log_format(LogFormat::Json);
        assert_eq!(cfg.log_format, LogFormat::Json);
    }

    #[test]
    fn test_with_prefetch_mode() {
        let cfg = VantaConfig::default().with_prefetch_mode(PrefetchMode::Disabled);
        assert_eq!(cfg.prefetch_mode, PrefetchMode::Disabled);
    }

    #[test]
    fn test_from_env_equals_default() {
        // `from_env()` delegates to `default()`, which reads env vars.
        // In an isolated test environment with no preset vars, both yield
        // the same values. To verify the delegation itself:
        // `from_env()` calls `Self::default()` — structural equality check.
        let cfg_default = VantaConfig::default();
        let cfg_from_env = VantaConfig::from_env();
        // Basic structural fields (env-independent) match
        assert_eq!(cfg_default.memory_limit, cfg_from_env.memory_limit);
        assert_eq!(cfg_default.read_only, cfg_from_env.read_only);
        assert_eq!(cfg_default.force_mmap, cfg_from_env.force_mmap);
        assert_eq!(cfg_default.backend_kind, cfg_from_env.backend_kind);
        assert_eq!(cfg_default.sync_mode, cfg_from_env.sync_mode);
        assert_eq!(cfg_default.prefetch_mode, cfg_from_env.prefetch_mode);
        assert_eq!(cfg_default.log_format, cfg_from_env.log_format);
        assert_eq!(cfg_default.rate_limit_rpm, cfg_from_env.rate_limit_rpm);
        assert_eq!(cfg_default.batch_size, cfg_from_env.batch_size);
        assert_eq!(cfg_default.wal_buffer_size, cfg_from_env.wal_buffer_size);
        assert_eq!(cfg_default.flush_threshold, cfg_from_env.flush_threshold);
    }

    // ── Builder chaining ───────────────────────────────────────

    #[test]
    fn test_builder_chaining() {
        let cfg = VantaConfig::default()
            .with_storage_path("/data/vanta".into())
            .with_memory_limit(8_000_000_000)
            .with_read_only(true)
            .with_force_mmap(true)
            .with_mmap_hnsw(false)
            .with_rss_threshold(0.70)
            .with_eviction_weights(0.5, 1.0, 1.5, 2.0)
            .with_eviction_ratio(0.15)
            .with_backend(BackendKind::RocksDb)
            .with_max_blocking_threads(64)
            .with_sync_mode(SyncMode::Always)
            .with_api_key(Some("sk-chained".into()))
            .with_rate_limit_rpm(200)
            .with_batch_size(500)
            .with_wal_buffer_size(131072)
            .with_flush_threshold(5000)
            .with_tls("crt.pem".into(), "key.pem".into())
            .with_log_format(LogFormat::Json)
            .with_prefetch_mode(PrefetchMode::Disabled);

        assert_eq!(cfg.storage_path, "/data/vanta");
        assert_eq!(cfg.memory_limit, Some(8_000_000_000));
        assert!(cfg.read_only);
        assert!(cfg.force_mmap);
        assert!(!cfg.mmap_hnsw);
        assert!((cfg.rss_threshold - 0.70).abs() < 1e-9);
        assert!((cfg.eviction_weight_hits - 0.5).abs() < 1e-9);
        assert!((cfg.eviction_ratio - 0.15).abs() < 1e-9);
        assert_eq!(cfg.backend_kind, BackendKind::RocksDb);
        assert_eq!(cfg.max_blocking_threads, 64);
        assert_eq!(cfg.sync_mode, SyncMode::Always);
        assert_eq!(cfg.api_key, Some("sk-chained".into()));
        assert_eq!(cfg.rate_limit_rpm, 200);
        assert_eq!(cfg.batch_size, Some(500));
        assert_eq!(cfg.wal_buffer_size, Some(131072));
        assert_eq!(cfg.flush_threshold, Some(5000));
        assert_eq!(cfg.tls_cert_path, Some("crt.pem".into()));
        assert_eq!(cfg.log_format, LogFormat::Json);
        assert_eq!(cfg.prefetch_mode, PrefetchMode::Disabled);
    }
}
