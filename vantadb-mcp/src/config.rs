//! MCP server configuration.

use std::time::Duration;
use vantadb::storage::StorageEngine;

/// Tool surface profile — controls which tools are exposed via `tools/list`.
///
/// Profiles are selected via the `VANTADB_MCP_PROFILE` environment variable:
/// - `memory` (≤20 tools): Core memory CRUD + search + list only. For memory-only agents.
/// - `dev` (≤35 tools): Memory + graph + collections + maintenance + introspection. Recommended for Cursor (cap ~40).
/// - `full` (76 tools): All tools including code, wiki, skills, threads, scenes, context. Default for compat.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum McpProfile {
    /// Full tool surface (76 tools) — all tools including code, wiki, skills, threads, scenes, context.
    /// Default for backward compatibility.
    #[default]
    Full,
    /// Developer profile (~35 tools) — memory, graph, collections, key maintenance, axioms.
    /// Recommended for Cursor (cap ~40 tools).
    Dev,
    /// Memory-only profile (~18 tools) — core memory CRUD + search + IQL + collections + capabilities.
    Memory,
}

impl std::str::FromStr for McpProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "full" => Ok(McpProfile::Full),
            "dev" => Ok(McpProfile::Dev),
            "memory" => Ok(McpProfile::Memory),
            other => Err(format!(
                "Invalid VANTADB_MCP_PROFILE: '{other}'. Valid values: full, dev, memory"
            )),
        }
    }
}

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
    /// Max explicit `rrf_k` in search_memory.search_profile (default: 100).
    pub max_rrf_k: usize,
    /// Max explicit `candidate_k` in search_memory.search_profile (default: 10_000).
    pub max_candidate_k: usize,
    /// Max bytes for a single skill resource file in skill_files_write (default: 5 MB).
    pub max_skill_resource_bytes: usize,
    /// Max total bytes for a skill — content plus all resource files (default: 50 MB).
    pub max_skill_total_bytes: usize,
    /// Tool surface profile (default: Full). Set via VANTADB_MCP_PROFILE env var.
    pub profile: McpProfile,
    /// MCP-39: byte budget for one-shot list/search tool responses.
    /// Default 40 KB (80% of OpenCode's 50 KB cap; safe under Claude Code's
    /// 25k token limit ~100 KB). Tunable via `VANTADB_MCP_BYTE_BUDGET` env var.
    pub byte_budget: usize,
    /// MCP-39: floor for `byte_budget` (default 1 KB) — below this the
    /// envelope overhead alone would not fit and the clamp rejects.
    pub min_byte_budget: usize,
    /// MCP-39: ceiling for `byte_budget` (default 1 MB) — above this the
    /// server is asked to render responses larger than typical MCP clients
    /// can display.
    pub max_byte_budget: usize,
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
            max_rrf_k: 100,
            max_candidate_k: 10_000,
            max_skill_resource_bytes: 5_000_000,
            max_skill_total_bytes: 50_000_000,
            profile: McpProfile::default(),
            byte_budget: 40 * 1024,
            min_byte_budget: 1024,
            max_byte_budget: 1024 * 1024,
        }
    }
}

impl McpConfig {
    /// Build from a StorageEngine, taking max_concurrency from it.
    /// Reads `VANTADB_MCP_PROFILE` env var for tool surface profile.
    /// Reads `VANTADB_MCP_BYTE_BUDGET` env var, clamped to
    /// `[min_byte_budget, max_byte_budget]`.
    pub fn from_storage(storage: &StorageEngine) -> Self {
        let profile = std::env::var("VANTADB_MCP_PROFILE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or_default();
        let mut config = Self {
            max_concurrency: storage.config.max_blocking_threads,
            profile,
            ..Default::default()
        };
        if let Ok(raw) = std::env::var("VANTADB_MCP_BYTE_BUDGET") {
            if let Ok(parsed) = raw.parse::<usize>() {
                config.byte_budget = parsed.clamp(config.min_byte_budget, config.max_byte_budget);
            }
        }
        config
    }
}
