//! Configuration system for VantaDB engine.
//!
//! Defines [`VantaConfig`] with typed fields, environment variable parsing,
//! and per-backend configuration options with fallback defaults.

use crate::backend::BackendKind;
#[cfg(feature = "advanced-tokenizer")]
use crate::tokenizer::AdvancedTokenizerConfig;
use std::env;
use std::str::FromStr;
use tracing::debug;
use tracing::warn;

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
    pub fn from_env_value(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "disabled" | "off" | "0" | "false" => PrefetchMode::Disabled,
            "enabled" | "on" | "1" | "true" => PrefetchMode::Enabled,
            _ => PrefetchMode::Auto,
        }
    }

    pub fn is_prefetch_enabled(self) -> bool {
        match self {
            PrefetchMode::Disabled => false,
            PrefetchMode::Auto | PrefetchMode::Enabled => true,
        }
    }
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
    pub advanced_tokenizer_config: Option<AdvancedTokenizerConfig>,
}

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
            rss_threshold: 0.80,
            eviction_weight_hits: 1.0,
            eviction_weight_confidence: 2.0,
            eviction_weight_importance: 3.0,
            eviction_weight_recency: 1.0,
            eviction_ratio: 0.20,
            backend_kind: {
                let v = match env::var("VANTA_BACKEND").ok().as_deref() {
                    Some("rocksdb") => BackendKind::RocksDb,
                    Some("memory") => BackendKind::InMemory,
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
                let v = parse_env_or("VANTADB_MAX_BLOCKING_THREADS", 16usize);
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

    /// Sets the prefetch mode for mmap vector pages during HNSW search.
    pub fn with_prefetch_mode(mut self, mode: PrefetchMode) -> Self {
        self.prefetch_mode = mode;
        self
    }
}

#[cfg(test)]
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

        assert_eq!(PrefetchMode::from_env_value("disabled"), PrefetchMode::Disabled);
        assert_eq!(PrefetchMode::from_env_value("off"), PrefetchMode::Disabled);
        assert_eq!(PrefetchMode::from_env_value("0"), PrefetchMode::Disabled);
        assert_eq!(PrefetchMode::from_env_value("false"), PrefetchMode::Disabled);

        assert_eq!(PrefetchMode::from_env_value("enabled"), PrefetchMode::Enabled);
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
        assert!((cfg.rss_threshold - 0.80).abs() < 1e-9);
        assert!((cfg.eviction_weight_hits - 1.0).abs() < 1e-9);
        assert!((cfg.eviction_weight_confidence - 2.0).abs() < 1e-9);
        assert!((cfg.eviction_weight_importance - 3.0).abs() < 1e-9);
        assert!((cfg.eviction_weight_recency - 1.0).abs() < 1e-9);
        assert!((cfg.eviction_ratio - 0.20).abs() < 1e-9);
        assert_eq!(cfg.backend_kind, BackendKind::Fjall);
        assert_eq!(cfg.max_blocking_threads, 16);
        assert_eq!(cfg.sync_mode, SyncMode::Periodic);
        assert_eq!(cfg.api_key, None);
        assert_eq!(cfg.rate_limit_rpm, 100);
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
        let cfg = VantaConfig::default()
            .with_eviction_weights(0.5, 1.5, 2.5, 3.5);
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
        let cfg = VantaConfig::default()
            .with_eviction_weights(0.1, 0.2, 0.3, 0.4);
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
    fn test_with_tls() {
        let cfg = VantaConfig::default()
            .with_tls("cert.pem".into(), "key.pem".into());
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
        let cfg = VantaConfig::default()
            .with_prefetch_mode(PrefetchMode::Disabled);
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
        assert_eq!(cfg.tls_cert_path, Some("crt.pem".into()));
        assert_eq!(cfg.log_format, LogFormat::Json);
        assert_eq!(cfg.prefetch_mode, PrefetchMode::Disabled);
    }
}
