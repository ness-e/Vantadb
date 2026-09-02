//! MCP-35 proxy client — DatabaseBusy → HTTP fallback.
//!
//! Writer instances expose an ephemeral HTTP server on 127.0.0.1:0 with
//! `GET /api/v2/health` and `POST /api/v2/mcp/proxy` (generic MCP dispatcher).
//! Proxy instances detect `.vanta.server.json`, probe health with 500 ms
//! timeout + sysinfo PID liveness, then forward `tools/call` via reqwest.
//! Auth token is passed through as `Authorization: Bearer` when the writer
//! requires it (VANTADB_API_KEY env).

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Discovery file shape — must match `crate::server::ServerInfo` (duplicated here
/// to avoid server↔proxy import cycle; both derive from same JSON contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub pid: u32,
    pub http_port: u16,
    pub started_at: u64,
    pub version: String,
}

pub fn discovery_path(storage_path: &str) -> PathBuf {
    Path::new(storage_path).join(".vanta.server.json")
}

pub fn is_pid_alive(pid: u32) -> bool {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    sys.process(sysinfo::Pid::from(pid as usize)).is_some()
}

/// Handle to a live writer's HTTP proxy.
#[derive(Debug, Clone)]
pub struct ProxyHandle {
    /// Base URL e.g. `http://127.0.0.1:54321`
    pub base_url: String,
    /// Reused reqwest client (rustls, json).
    pub client: reqwest::Client,
    /// Optional Bearer token copied from the proxy's env (VANTADB_API_KEY).
    pub api_key: Option<String>,
}

impl ProxyHandle {
    /// Build a ProxyHandle from ServerInfo, probing health with 500 ms timeout.
    /// Returns None if PID dead, health non-healthy, or timeout.
    pub async fn try_connect(info: ServerInfo, api_key: Option<String>) -> Option<Self> {
        if !is_pid_alive(info.pid) {
            tracing::warn!(pid = info.pid, "stale discovery: PID not alive");
            return None;
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(500))
            .build()
            .ok()?;
        let base_url = format!("http://127.0.0.1:{}", info.http_port);
        let health_url = format!("{}/api/v2/health", base_url);
        let mut req = client.get(&health_url);
        if let Some(ref key) = api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        let resp = req.send().await.ok()?;
        if !resp.status().is_success() {
            tracing::warn!(status = %resp.status(), "health probe non-200");
            return None;
        }
        let body: Value = resp.json().await.ok()?;
        // health_v2 returns { status: "healthy" | "degraded", ... }
        let status = body.get("status").and_then(|v| v.as_str()).unwrap_or("");
        if status != "healthy" {
            tracing::warn!(status, "health probe degraded");
            return None;
        }
        Some(Self {
            base_url,
            client,
            api_key,
        })
    }

    /// Forward a `tools/call` via `POST /api/v2/mcp/proxy` (generic MCP proxy).
    /// `params` is the `tools/call` params object: { name, arguments }.
    pub async fn proxy_tools_call(&self, params: &Option<Value>) -> Result<Value, Value> {
        self.proxy_mcp_call("tools/call", params).await
    }

    /// Generic MCP proxy: POST /api/v2/mcp/proxy with { method, params }
    pub async fn proxy_mcp_call(
        &self,
        method: &str,
        params: &Option<Value>,
    ) -> Result<Value, Value> {
        let url = format!("{}/api/v2/mcp/proxy", self.base_url);
        let body = json!({
            "method": method,
            "params": params.clone().unwrap_or(json!(null))
        });
        let mut req = self
            .client
            .post(&url)
            .json(&body)
            .timeout(Duration::from_secs(60));
        if let Some(ref key) = self.api_key {
            req = req.header("Authorization", format!("Bearer {}", key));
        }
        let resp = req.send().await.map_err(
            |e| json!({"code": -32603, "message": format!("proxy request failed: {}", e)}),
        )?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(
                json!({"code": -32603, "message": format!("proxy http {}: {}", status, text)}),
            );
        }
        let resp_json: Value = resp.json().await.map_err(
            |e| json!({"code": -32603, "message": format!("proxy decode failed: {}", e)}),
        )?;
        // Writer's proxy endpoint returns { success: true, data: ... } or { success: false, error: ... }
        if resp_json.get("success") == Some(&json!(true)) {
            Ok(resp_json.get("data").cloned().unwrap_or(json!(null)))
        } else if let Some(err) = resp_json.get("error") {
            Err(err.clone())
        } else {
            // Fallback: treat whole body as success data
            Ok(resp_json)
        }
    }
}

/// Try to establish a proxy from the discovery file at `storage_path`.
/// Returns Some(handle) if file exists, PID alive, and health 200 within 500 ms.
pub async fn try_proxy(
    storage_path: &str,
    api_key: Option<String>,
) -> Option<(ProxyHandle, ServerInfo)> {
    let path = discovery_path(storage_path);
    let content = std::fs::read_to_string(&path).ok()?;
    let info: ServerInfo = serde_json::from_str(&content).ok()?;
    let handle = ProxyHandle::try_connect(info.clone(), api_key).await?;
    tracing::info!(
        pid = handle.base_url,
        port = info.http_port,
        "proxy mode → {}",
        handle.base_url
    );
    Some((handle, info))
}

/// Stale cleanup: remove `.vanta.server.json` best-effort, log warn.
pub fn cleanup_stale(storage_path: &str) {
    let path = discovery_path(storage_path);
    if path.exists() {
        match std::fs::remove_file(&path) {
            Ok(_) => tracing::warn!(path = %path.display(), "removed stale discovery file"),
            Err(e) => {
                tracing::warn!(path = %path.display(), error = %e, "failed to remove stale discovery file")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_pid_alive_self() {
        let pid = std::process::id();
        assert!(is_pid_alive(pid), "own pid must be alive");
    }

    #[test]
    fn is_pid_alive_dead() {
        // Use a very high PID that definitely doesn't exist
        assert!(!is_pid_alive(999_999), "bogus pid must be dead");
    }

    #[tokio::test]
    async fn stale_pid_cleanup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        // Write a discovery file with dead PID
        let info = ServerInfo {
            pid: 999_999,
            http_port: 12345,
            started_at: 0,
            version: "0.5.0".into(),
        };
        let disc = discovery_path(&path);
        std::fs::write(&disc, serde_json::to_string_pretty(&info).unwrap()).unwrap();
        assert!(disc.exists());
        // try_proxy should fail due to dead PID (health not probed)
        let res = try_proxy(&path, None).await;
        assert!(res.is_none(), "dead PID should not connect");
        // cleanup
        cleanup_stale(&path);
        assert!(!disc.exists(), "stale file should be removed");
    }

    #[tokio::test]
    async fn health_timeout_fallback() {
        // No file -> try_proxy returns None quickly (no health to probe)
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let res = try_proxy(&path, None).await;
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn discovery_file_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        let info = ServerInfo {
            pid: std::process::id(),
            http_port: 0,
            started_at: 12345,
            version: "0.5.0".into(),
        };
        let disc = discovery_path(&path);
        std::fs::write(&disc, serde_json::to_string_pretty(&info).unwrap()).unwrap();
        let content = std::fs::read_to_string(&disc).unwrap();
        let parsed: ServerInfo = serde_json::from_str(&content).unwrap();
        assert_eq!(parsed.pid, info.pid);
        assert_eq!(parsed.version, "0.5.0");
        let _ = std::fs::remove_file(&disc);
    }
}
