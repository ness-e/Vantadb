//! Top-level convenience entry point for opening a VantaDB database
//! from a path string, supporting both file-backed and in-memory engines.

use super::Embedded;
use crate::config::Config;
use crate::error::Result;
use crate::storage::BackendKind;

/// Connect to a VantaDB database.
///
/// - `path`: filesystem path (opens or creates Fjall/RocksDB backend)
/// - If path is empty or `":memory:"`, opens in-memory engine
pub fn connect(path: &str) -> Result<Embedded> {
    if path.is_empty() || path == ":memory:" {
        let config = Config {
            storage_path: ":memory:".to_string(),
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        Embedded::open_with_config(config)
    } else {
        Embedded::open(path)
    }
}
