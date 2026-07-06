//! Write-ahead log initialization and recovery for crash durability.

use crate::backend::{BackendPartition, StorageBackend};
use crate::config::VantaConfig;
use crate::error::Result;
use crate::index::CPIndex;
use crate::storage::vfile::VantaFile;
use crate::wal::{WalReader, WalRecord, WalWriter};
use std::path::Path;
use std::sync::Arc;
use tracing::info;

const FLAG_TOMBSTONE: u32 = 0x8;

/// Open or skip WAL initialization based on the read-only configuration flag.
pub(crate) fn init_wal(
    data_dir: &Path,
    config: &VantaConfig,
) -> Result<Option<crate::wal_sharded::ShardedWal>> {
    if config.read_only {
        return Ok(None);
    }
    let wal_path = data_dir.join("vanta.wal");
    Ok(Some(crate::wal_sharded::ShardedWal::new(
        &wal_path,
        4,
        config.sync_mode,
    )?))
}

#[allow(dead_code)]
/// Replay WAL records to restore engine state after a crash or restart.
pub(crate) fn recover_state(
    data_dir: &Path,
    config: &VantaConfig,
    backend: &Arc<dyn StorageBackend>,
    hnsw: &mut CPIndex,
    vector_store: &mut VantaFile,
) -> Result<(u64, u64)> {
    if config.read_only {
        return Ok((0, 0));
    }
    let wal_path = data_dir.join("vanta.wal");
    if !wal_path.exists() {
        return Ok((0, 0));
    }
    let wal_replay_started = std::time::Instant::now();
    let mut wal_records_replayed = 0u64;

    if let Ok(mut reader) = WalReader::open(&wal_path) {
        while let Some(record) = reader.next_record()? {
            match record {
                WalRecord::Insert(node) => {
                    if let Ok(offset) = super::ops::write_node_to_vstore(vector_store, &node) {
                        hnsw.add(node.id, node.bitset.clone(), node.vector.clone(), offset);
                        let _ = super::ops::insert_node_to_backend(backend, &node, "default");
                    }
                    wal_records_replayed += 1;
                }
                WalRecord::Update { id, node } => {
                    if let Ok(offset) = super::ops::write_node_to_vstore(vector_store, &node) {
                        hnsw.add(id, node.bitset.clone(), node.vector.clone(), offset);
                        let _ = super::ops::insert_node_to_backend(backend, &node, "default");
                    }
                    wal_records_replayed += 1;
                }
                WalRecord::Delete { id } => {
                    let _ = backend.delete(BackendPartition::Default, &id.to_le_bytes());
                    if let Some(index_node) = hnsw.nodes.get(&id) {
                        if let Some(mut h) = vector_store.read_header(index_node.storage_offset) {
                            h.flags |= FLAG_TOMBSTONE;
                            let _ = vector_store.write_header(index_node.storage_offset, &h);
                        }
                    }
                    wal_records_replayed += 1;
                }
                WalRecord::Checkpoint { .. } => {}
            }
        }
    }
    let wal_replay_ms = wal_replay_started.elapsed().as_millis() as u64;
    if wal_records_replayed > 0 {
        info!(
            replayed = wal_records_replayed,
            duration_ms = wal_replay_ms,
            "WAL replay completed"
        );
    }
    Ok((wal_replay_ms, wal_records_replayed))
}
