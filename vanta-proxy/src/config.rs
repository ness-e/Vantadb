//! Proxy configuration loaded from a TOML file (decision D31).

use std::path::Path;

use serde::Deserialize;

use crate::error::ProxyError;

/// Default forward timeout in seconds (TDAM parity: config.ts:10 — 600_000 ms).
pub const DEFAULT_FORWARD_TIMEOUT_SECS: u64 = 600;
/// Default listen port (TDAM parity).
pub const DEFAULT_PORT: u16 = 8096;

/// Top-level proxy configuration.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ProxyConfig {
    /// Listen address and port.
    pub server: ServerConfig,
    /// Upstream LLM endpoint.
    pub upstream: UpstreamConfig,
    /// Local auth/session store (D25/D34).
    pub auth: AuthConfig,
    /// In-band `mem:` commands (D33) — disabled by default (TDAM parity).
    pub mem_command: MemCommandConfig,
    /// L0 write-back persistence settings.
    pub writeback: WritebackConfig,
    /// Exact response cache (PRX-09 slice 1). Disabled by default (opt-in).
    pub cache: CacheConfig,
    /// Optional per-turn span export to Langfuse/OTel over OTLP-JSON
    /// (MEM-56). Disabled by default (empty endpoint).
    pub report: ReportConfig,
}

impl ProxyConfig {
    /// Load configuration from a TOML file.
    pub fn load(path: &Path) -> Result<Self, ProxyError> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| ProxyError::Config(format!("cannot read {}: {e}", path.display())))?;
        toml::from_str(&raw).map_err(|e| ProxyError::Config(format!("invalid TOML: {e}")))
    }
}

/// In-band `mem:` command interception (D33). Disabled by default so the
/// wire stays a transparent proxy unless explicitly opted in (TDAM parity).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MemCommandConfig {
    /// When false (default), `mem:*` messages are forwarded verbatim to the
    /// upstream LLM.
    pub enabled: bool,
}

/// L0 write-back queue persistence.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WritebackConfig {
    /// File holding labels of writes that exhausted retries (crash audit
    /// trail). Empty string disables persistence.
    pub persist_path: String,
}

impl Default for WritebackConfig {
    fn default() -> Self {
        Self {
            persist_path: "vanta-proxy-writeback-pending.json".to_string(),
        }
    }
}

/// Exact response cache (PRX-09 slice 1: exact-only). Disabled by default so
/// the wire stays a transparent proxy unless explicitly opted in.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    /// When false (default), every lookup misses and stores are no-ops.
    pub enabled: bool,
    /// Max entries held (FIFO eviction past the cap).
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_entries: 128,
        }
    }
}

/// Optional per-turn span export to a Langfuse/OTel endpoint (MEM-56).
/// Disabled by default so an unconfigured proxy pays zero overhead.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct ReportConfig {
    /// OTLP/HTTP JSON endpoint receiving one span per turn (e.g. Langfuse
    /// `https://cloud.langfuse.com/api/public/otel/v1/traces` or an OTel
    /// collector). Empty (default) disables export entirely.
    pub langfuse_endpoint: String,
    /// Raw value of the `Authorization` header sent with each export
    /// (e.g. Langfuse basic auth). Empty omits the header.
    pub langfuse_auth_header: String,
}

impl ReportConfig {
    /// Export is on only when an endpoint is configured.
    pub fn enabled(&self) -> bool {
        !self.langfuse_endpoint.is_empty()
    }
}

/// HTTP listener settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// Rate-limits placeholder (implemented in MEM-27); parsed but unused here.
    pub rate_limit_per_minute: u32,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: DEFAULT_PORT,
            rate_limit_per_minute: 60,
        }
    }
}

/// Local auth/session store settings (D25: RBAC local entity_*, no remote gateway).
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct AuthConfig {
    /// Path of the local VantaDB store holding the `user`/`team`/`agent`/`task`
    /// entity collections used for auth (D34) and session validation (D26).
    pub db_path: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            db_path: "vantadb_data".to_string(),
        }
    }
}

/// Upstream LLM endpoint settings.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct UpstreamConfig {
    /// Base URL of the upstream (e.g. `https://api.anthropic.com`). The wire
    /// path (`/v1/...`) is appended as received from the client.
    pub url: String,
    /// If non-empty, overrides the `Authorization` header sent upstream;
    /// otherwise the incoming header is passed through verbatim.
    pub api_key: String,
    /// Total forward timeout in seconds (D31/TDAM: default 600).
    pub forward_timeout_secs: u64,
    /// Model IDs served by `GET /v1/models` (PRX-05: derived from config,
    /// never hardcoded — a stale list breaks the Claude Code picker).
    /// Empty (default) → `{"object":"list","data":[]}`.
    pub models: Vec<String>,
}

impl UpstreamConfig {
    /// True when this upstream URL would route back into this proxy itself
    /// (loopback host + our own listen port) — a self-forwarding loop that
    /// recurses until the forward timeout (PRX-08 S2). Pure and total.
    pub fn points_at_self(&self, listen_port: u16) -> bool {
        let Some((host, port)) = split_host_port(&self.url) else {
            return false;
        };
        let loopback = matches!(host.as_str(), "127.0.0.1" | "localhost" | "::1");
        loopback && port == listen_port
    }
}

/// Split `scheme://authority/rest` into lowercase (host, port). `None` when
/// unparseable or portless (portless → cannot be a self-loop on our port).
fn split_host_port(url: &str) -> Option<(String, u16)> {
    let authority = url.split("://").nth(1)?.split('/').next()?;
    let authority = authority.split('@').next_back()?;
    if let Some(rest) = authority.strip_prefix('[') {
        // [::1]:port
        let (host, port) = rest.split_once("]:")?;
        Some((host.to_ascii_lowercase(), port.parse().ok()?))
    } else {
        let (host, port) = authority.split_once(':')?;
        Some((host.to_ascii_lowercase(), port.parse().ok()?))
    }
}

impl Default for UpstreamConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8096".to_string(),
            api_key: String::new(),
            forward_timeout_secs: DEFAULT_FORWARD_TIMEOUT_SECS,
            models: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_upstream_points_at_default_port() {
        // PRX-08 S2 RED: the shipped default IS a self-loop.
        let cfg = UpstreamConfig::default();
        assert!(cfg.points_at_self(DEFAULT_PORT));
        assert!(cfg.points_at_self(8096));
    }

    #[test]
    fn mock_upstream_on_other_port_is_not_self() {
        let cfg = UpstreamConfig {
            url: "http://127.0.0.1:51234".to_string(),
            ..UpstreamConfig::default()
        };
        assert!(!cfg.points_at_self(DEFAULT_PORT));
    }

    #[test]
    fn remote_upstream_is_not_self_even_on_same_port() {
        let cfg = UpstreamConfig {
            url: "https://api.anthropic.com".to_string(),
            ..UpstreamConfig::default()
        };
        assert!(!cfg.points_at_self(443));
    }
}
