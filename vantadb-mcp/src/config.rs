//! MCP server configuration.

use std::time::Duration;
use vantadb::storage::StorageEngine;

/// Tuning knobs for the MCP server.
#[derive(Clone, Debug)]
pub struct McpConfig {
    /// Max concurrent requests (default: storage engine's max_blocking_threads).
    pub max_concurrency: usize,
    /// Max payload length for memory_put (default: 1 MB).
    pub max_payload_length: usize,
    /// Max key length (default: 512).
    pub max_key_length: usize,
    /// Max namespace length (default: 256).
    pub max_namespace_length: usize,
    /// Max vector dimension (default: 16384).
    pub max_vector_dim: usize,
    /// Max query length (default: 1 MB).
    pub max_query_length: usize,
    /// Per-request timeout (default: 60 s).
    pub request_timeout: Duration,
    /// Default limit for memory_list (default: 100).
    pub default_list_limit: usize,
    /// Max limit for memory_list (default: 10_000).
    pub max_list_limit: usize,
    /// Default top_k for search_memory (default: 10).
    pub default_top_k: usize,
    /// Max top_k for search_memory (default: 1000).
    pub max_top_k: usize,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            max_concurrency: 32,
            max_payload_length: 1_048_576,
            max_key_length: 512,
            max_namespace_length: 256,
            max_vector_dim: 16_384,
            max_query_length: 1_048_576,
            request_timeout: Duration::from_secs(60),
            default_list_limit: 100,
            max_list_limit: 10_000,
            default_top_k: 10,
            max_top_k: 1000,
        }
    }
}

impl McpConfig {
    /// Build from a StorageEngine, taking max_concurrency from it.
    pub fn from_storage(storage: &StorageEngine) -> Self {
        Self {
            max_concurrency: storage.config.max_blocking_threads,
            ..Default::default()
        }
    }
}
