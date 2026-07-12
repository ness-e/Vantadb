//! Write-ahead log initialization and recovery for crash durability.

use crate::config::VantaConfig;
use crate::error::Result;
use std::path::Path;

/// Open or skip WAL initialization based on the read-only configuration flag.
pub(crate) fn init_wal(
    data_dir: &Path,
    config: &VantaConfig,
) -> Result<Option<crate::wal_sharded::ShardedWal>> {
    if config.read_only {
        return Ok(None);
    }
    let wal_path = data_dir.join("vanta.wal");
    let wal_buffer_size = config.wal_buffer_size.unwrap_or(64 * 1024);
    let flush_threshold = config.flush_threshold;
    Ok(Some(crate::wal_sharded::ShardedWal::new_with_buffer(
        &wal_path,
        config.wal_shards,
        config.sync_mode,
        wal_buffer_size,
        flush_threshold,
    )?))
}
