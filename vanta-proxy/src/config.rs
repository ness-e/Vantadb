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
}

impl ProxyConfig {
    /// Load configuration from a TOML file.
    pub fn load(path: &Path) -> Result<Self, ProxyError> {
        let raw = std::fs::read_to_string(path)
            .map_err(|e| ProxyError::Config(format!("cannot read {}: {e}", path.display())))?;
        toml::from_str(&raw).map_err(|e| ProxyError::Config(format!("invalid TOML: {e}")))
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
}

impl Default for UpstreamConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:8096".to_string(),
            api_key: String::new(),
            forward_timeout_secs: DEFAULT_FORWARD_TIMEOUT_SECS,
        }
    }
}
