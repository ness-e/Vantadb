//! Top-level convenience entry point for opening a VantaDB database
//! from a path string, supporting both file-backed and in-memory engines.

use super::VantaEmbedded;
use crate::config::VantaConfig;
use crate::error::Result;
use crate::storage::BackendKind;

/// Connect to a VantaDB database.
///
/// - `path`: filesystem path (opens or creates Fjall/RocksDB backend)
/// - If path is empty or `":memory:"`, opens in-memory engine
pub fn connect(path: &str) -> Result<VantaEmbedded> {
    if path.is_empty() || path == ":memory:" {
        let config = VantaConfig {
            storage_path: ":memory:".to_string(),
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        VantaEmbedded::open_with_config(config)
    } else {
        VantaEmbedded::open(path)
    }
}
