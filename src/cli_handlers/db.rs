//! Database open helpers.

use crate::config::VantaConfig;
use crate::error::Result;
use crate::storage::StorageEngine;
use crate::VantaEmbedded;

/// Open a database at the given path with optional read-only mode
pub fn open_database(path: &str, read_only: bool) -> Result<StorageEngine> {
    let config = VantaConfig {
        read_only,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config))
}

/// Open the embedded VantaDB SDK with the given path and read-only mode
pub fn open_embedded(path: &str, read_only: bool) -> Result<VantaEmbedded> {
    let config = VantaConfig {
        storage_path: path.to_string(),
        read_only,
        ..Default::default()
    };
    VantaEmbedded::open_with_config(config)
}

/// Compute a deterministic node ID from namespace and key using xxHash3-128
pub fn memory_node_id(namespace: &str, key: &str) -> u128 {
    let mut hasher = twox_hash::XxHash3_128::default();
    hasher.write(namespace.as_bytes());
    hasher.write(b"\0");
    hasher.write(key.as_bytes());
    hasher.finish_128()
}
