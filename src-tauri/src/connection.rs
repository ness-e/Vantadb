//! NativeConnection — the contract between the VantaDB core and the Tauri
//! frontend (DESKTOP-03/04/05).
//!
//! The trait abstracts *how* the UI reaches the core. The MVP ships one
//! implementation: [`InProcessConnection`], which reuses `vantadb` directly in
//! this process (no HTTP server, no WASM bridge). Future server-backed
//! connections (desktop → remote `vantadb-server`) plug in behind the same
//! trait without touching the frontend.
//!
//! Error handling is explicit — no unwrap/expect; every method returns a
//! [`ConnectionError`] carrying the VantaDB error chain (DESKTOP-26).

use serde::{Deserialize, Serialize};
use vantadb::config::VantaConfig;
use vantadb::sdk::VantaMemoryInput;
use vantadb::{BackendKind, VantaEmbedded};

/// Which transport a connection uses. Mirrored to the UI for status display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionKind {
    /// Core running inside this process (in-memory or on-disk).
    InProcess,
}

/// Health-check payload returned by [`NativeConnection::ping`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub ok: bool,
    pub connection_kind: ConnectionKind,
    pub core_version: String,
    pub message: String,
}

impl Health {
    fn healthy(kind: ConnectionKind) -> Self {
        Self {
            ok: true,
            connection_kind: kind,
            core_version: env!("CARGO_PKG_VERSION").to_string(),
            message: "pong".to_string(),
        }
    }
}

/// Error type surfacing the core's `VantaError` across the FFI/commands
/// boundary. `thiserror` keeps the mapping explicit (DESKTOP-026 error
/// contract): the UI always receives a `Result<_, String>` from a Tauri
/// command, so unknowns degrade to a descriptive message instead of panics.
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    /// Reserved for connection lifecycle (DESKTOP-20); not constructed in the
    /// in-process MVP because the core always opens eagerly.
    #[allow(dead_code)]
    #[error("connection not open: {0}")]
    NotOpen(String),
    #[error("core error: {0}")]
    Core(#[from] vantadb::VantaError),
}

/// The contract between the Rust core and the desktop UI.
pub trait NativeConnection: Send + Sync {
    /// Connection transport kind (display + routing).
    fn kind(&self) -> ConnectionKind;

    /// Health check — the MVP gate ("ping responds").
    fn ping(&self) -> Result<Health, ConnectionError>;

    /// Upsert a memory record: `put(ns, key, payload)`.
    fn put(&self, namespace: &str, key: &str, payload: &str) -> Result<String, ConnectionError>;

    /// Read a single memory record by namespace/key.
    fn get(&self, namespace: &str, key: &str) -> Result<Option<String>, ConnectionError>;

    /// Delete a memory record. Returns `true` if it existed.
    fn delete(&self, namespace: &str, key: &str) -> Result<bool, ConnectionError>;
}

/// In-process connection: `vantadb` linked directly into this binary.
///
/// Opens a VantaDB engine on `storage_path` with an optional backend kind.
/// `:memory:` + [BackendKind::InMemory] is the zero-footprint default used by
/// the demo so no data directory or extra features are required to compile.
#[derive(Clone)]
pub struct InProcessConnection {
    db: vantadb::VantaEmbedded,
}

impl InProcessConnection {
    /// Open an in-process engine. `storage_path` may be `":memory:"`.
    pub fn open(
        storage_path: &str,
        backend: Option<BackendKind>,
    ) -> Result<Self, ConnectionError> {
        let backend_kind = backend.unwrap_or(BackendKind::Fjall);
        let config = VantaConfig {
            storage_path: storage_path.to_string(),
            backend_kind,
            ..Default::default()
        };
        let db = VantaEmbedded::open_with_config(config)?;
        Ok(Self { db })
    }

    /// Convenience: in-memory engine (no disk). Used by the demo and tests.
    pub fn open_in_memory() -> Result<Self, ConnectionError> {
        Self::open(":memory:", Some(BackendKind::InMemory))
    }
}

impl NativeConnection for InProcessConnection {
    fn kind(&self) -> ConnectionKind {
        ConnectionKind::InProcess
    }

    fn ping(&self) -> Result<Health, ConnectionError> {
        Ok(Health::healthy(self.kind()))
    }

    fn put(&self, namespace: &str, key: &str, payload: &str) -> Result<String, ConnectionError> {
        let input = VantaMemoryInput::new(namespace, key, payload);
        let rec = self.db.put(input)?;
        Ok(format!("{}:{} v{}", rec.namespace, rec.key, rec.version))
    }

    fn get(&self, namespace: &str, key: &str) -> Result<Option<String>, ConnectionError> {
        Ok(self.db.get(namespace, key)?.map(|r| r.payload))
    }

    fn delete(&self, namespace: &str, key: &str) -> Result<bool, ConnectionError> {
        Ok(self.db.delete(namespace, key)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn conn() -> InProcessConnection {
        InProcessConnection::open_in_memory().expect("open in-memory")
    }

    #[test]
    fn ping_responds_healthy() {
        let h = conn().ping().expect("ping");
        assert!(h.ok);
        assert_eq!(h.message, "pong");
        assert_eq!(h.connection_kind, ConnectionKind::InProcess);
    }

    #[test]
    fn put_get_roundtrip() {
        let c = conn();
        let msg = c.put("doc", "greeting", "hello world").expect("put");
        assert!(msg.contains("doc:greeting"));
        assert_eq!(c.get("doc", "greeting").expect("get").as_deref(), Some("hello world"));
    }

    #[test]
    fn get_missing_is_none() {
        assert!(conn().get("doc", "absent").expect("get").is_none());
    }

    #[test]
    fn delete_returns_existence() {
        let c = conn();
        c.put("doc", "k", "v").expect("put");
        assert!(c.delete("doc", "k").expect("delete"));
        assert!(!c.delete("doc", "k").expect("delete"));
        assert!(c.get("doc", "k").expect("get").is_none());
    }
}