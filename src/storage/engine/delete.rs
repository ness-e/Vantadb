//! Delete, delete_batch, purge, and tombstone bookkeeping.

use std::sync::atomic::Ordering;

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::lsm::unpack_offset;
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{BufferedWrite, FLAG_TOMBSTONE};
use crate::wal::WalRecord;

impl StorageEngine {
    /// Mark a node as deleted: write tombstone, remove from cache and backend.
    ///
    /// # ACID note
    /// WAL is appended before any store I/O. If vector-store tombstone or backend delete
    /// fails mid-operation, the WAL record creates a phantom on recovery replay.
    /// Unlike insert(), there is no compensation action because WAL is the commit point.
    /// Full saga/2PC is deferred to ACID Phase 0.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn delete(&self, id: u128, _reason: &str) -> Result<()> {
        self.check_memory_pressure()?;
        // Inside transaction ΓåÆ buffer in the txn's write set; stats, indexes
        // and store writes are applied only at commit (ERR-013).
        {
            let active = self.active_txns.lock();
            if !active.is_empty() {
                if active.len() == 1 {
                    let txn_id = active.iter().next().copied().ok_or_else(|| {
                        crate::error::VantaError::generic_error(
                            "active transaction set corrupted: len()==1 but no txn id".to_string(),
                        )
                    })?;
                    drop(active);
                    let mut buffers = self.txn_buffers.lock();
                    buffers
                        .entry(txn_id)
                        .or_default()
                        .push(BufferedWrite::Delete(id));
                    return Ok(());
                }
                return Err(crate::error::VantaError::InvalidInput(
                    "Multiple active transactions; use delete_in_txn() instead".into(),
                ));
            }
        }

        // Non-transaction path: cardinality/index updates + WAL + stores.
        self.apply_delete_stats(id);

        // WAL + apply inside a single insert_lock critical section (ERR-010) ΓÇö
        // see insert()/delete().
        self.ensure_writable()?;
        {
            let _guard = self
                .insert_lock
                .try_lock_for(std::time::Duration::from_millis(
                    self.config.insert_lock_timeout_ms,
                ))
                .ok_or_else(|| crate::error::VantaError::Timeout {
                    operation: "acquire insert_lock in delete (WAL + apply)".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                })?;
            if let Some(ref sharded) = self.wal {
                sharded.append(&crate::wal::WalRecord::Delete { id })?;
            }
            self.apply_delete_inner(id, false)?;
            self.backend
                .delete(BackendPartition::Default, &id.to_le_bytes())
        }
    }

    /// Apply cardinality stats + edge/scalar index removal for a delete.
    ///
    /// Called from the non-transactional `delete()` path and from
    /// `commit_transaction` when a buffered delete is actually committed.
    /// It is intentionally NOT called while a delete is being buffered into a
    /// transaction ΓÇö an abort would otherwise leave the counters deflated for
    /// records that were never removed (ERR-013).
    pub(crate) fn apply_delete_stats(&self, id: u128) {
        if let Ok(Some(node)) = self.get(id) {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in node.relational {
                let val_keys = value.to_cardinality_keys();
                if let Some(val_map) = stats.get_mut(&field) {
                    for val_key in val_keys {
                        if let Some(count) = val_map.get_mut(&val_key) {
                            if *count > 0 {
                                *count -= 1;
                            }
                        }
                    }
                    val_map.retain(|_, &mut v| v > 0);
                }
            }
            // ponytail: drop the field with fewest entries if total pairs > global cap
            let total: usize = stats.values().map(|m| m.len()).sum();
            if total > crate::config::MAX_CARDINALITY_PAIRS {
                if let Some(min_field) = stats
                    .iter()
                    .min_by_key(|(_, m)| m.len())
                    .map(|(k, _)| k.clone())
                {
                    stats.remove(&min_field);
                }
            }

            // PERF-07: cascade ΓÇö remove all edges referencing this node
            if let Some(ref ei) = self.edge_index {
                ei.remove_all_for_node(id);
            }
            // PERF-08: remove node from scalar index
            if let Some(ref si) = self.scalar_index {
                si.remove_node(id);
            }
        }
    }

    /// Apply a delete to the stores (vstore tombstone, HNSW, cache).
    ///
    /// Does NOT remove the backend metadata entry ΓÇö callers that need
    /// physical removal (non-transactional deletes) must call
    /// `backend.delete()` separately alongside `stamp_deleted_in_backend`.
    /// Transactional deletes leave the metadata stamp so MVCC snapshots
    /// can still read the tombstone, and GC can later reclaim it.
    ///
    /// Does NOT write to WAL ΓÇö the caller is responsible for WAL logging.
    /// Does NOT check active_txns or ensure_writable.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub(crate) fn apply_delete(&self, id: u128) -> Result<()> {
        self.apply_delete_inner(id, true)
    }

    /// Shared delete-apply body. When `acquire` is false the caller already
    /// holds `insert_lock` (delete()'s ERR-010 critical section), so the HNSW
    /// removal must not re-acquire the non-reentrant lock.
    pub(crate) fn apply_delete_inner(&self, id: u128, acquire: bool) -> Result<()> {
        let packed = {
            let hnsw = self.hnsw.load();
            hnsw.nodes.get(&id).map(|n| n.storage_offset)
        };

        // PERF-23: vector store tombstone
        if let Some(packed) = packed {
            let (seg_id, local_off) = unpack_offset(packed);
            if let Some(vs) = self.vector_store.get(seg_id as usize) {
                let mut vstore = vs.write();
                if let Some(mut header) = vstore.read_header(local_off) {
                    header.flags |= FLAG_TOMBSTONE;
                    vstore.write_header(local_off, &header)?;
                }
            }
        }

        if acquire {
            let _guard = self
                .insert_lock
                .try_lock_for(std::time::Duration::from_millis(
                    self.config.insert_lock_timeout_ms,
                ))
                .ok_or_else(|| crate::error::VantaError::Timeout {
                    operation: "acquire insert_lock in apply_delete".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                })?;
            self.remove_hnsw_entry(id);
        } else {
            self.remove_hnsw_entry(id);
        }

        self.volatile_cache.write().remove(&id);

        Ok(())
    }

    /// Remove a node from the HNSW graph and promote a replacement entry point
    /// if the removed node was the entry point. Caller must hold `insert_lock`.
    fn remove_hnsw_entry(&self, id: u128) {
        let hnsw = self.hnsw.load();
        hnsw.nodes.remove(&id);

        // PERF-23: If we just removed the entry point, promote a replacement
        if hnsw.entry_point.load(Ordering::Relaxed) == id {
            let new_ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
            hnsw.entry_point.store(new_ep, Ordering::Relaxed);
        }
    }

    /// Delete multiple nodes in a single batch operation.
    ///
    /// Reduces I/O and lock contention by batching WAL records, KV backend writes,
    /// and processing HNSW removal for all nodes under one guard.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn delete_batch(&self, ids: &[u128]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        self.check_memory_pressure()?;
        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(crate::error::VantaError::IoError(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });

        self.touch_activity();

        // Phase 1: cardinality stats update, edge / scalar index removal
        {
            let mut stats = self.cardinality_stats.write();
            for &id in ids {
                if let Ok(Some(node)) = self.get(id) {
                    for (field, value) in &node.relational {
                        let val_keys = value.to_cardinality_keys();
                        if let Some(val_map) = stats.get_mut(field.as_str()) {
                            for val_key in val_keys {
                                if let Some(count) = val_map.get_mut(&val_key) {
                                    if *count > 0 {
                                        *count -= 1;
                                    }
                                }
                            }
                            val_map.retain(|_, &mut v| v > 0);
                        }
                    }
                    if let Some(ref ei) = self.edge_index {
                        ei.remove_all_for_node(id);
                    }
                    if let Some(ref si) = self.scalar_index {
                        si.remove_node(id);
                    }
                }
            }
            // ponytail: drop the field with fewest entries if total pairs > global cap
            let total: usize = stats.values().map(|m| m.len()).sum();
            if total > crate::config::MAX_CARDINALITY_PAIRS {
                if let Some(min_field) = stats
                    .iter()
                    .min_by_key(|(_, m)| m.len())
                    .map(|(k, _)| k.clone())
                {
                    stats.remove(&min_field);
                }
            }
        }

        // Phase 2-3: WAL batch append + HNSW removal under one insert_lock guard
        // (ERR-010) ΓÇö see batch_insert()/flush().
        let _guard = self
            .insert_lock
            .try_lock_for(std::time::Duration::from_millis(
                self.config.insert_lock_timeout_ms,
            ))
            .ok_or_else(|| crate::error::VantaError::Timeout {
                operation: "acquire insert_lock in delete_batch".into(),
                duration_ms: self.config.insert_lock_timeout_ms,
            })?;

        // Phase 2: WAL batch append
        let wal_records: Vec<WalRecord> = ids.iter().map(|&id| WalRecord::Delete { id }).collect();
        if let Some(ref sharded) = self.wal {
            sharded.batch_append(wal_records)?;
        }

        // Phase 3: HNSW node removal + vector store tombstone marking
        {
            let hnsw = self.hnsw.load();
            for &id in ids {
                if let Some(packed) = hnsw.nodes.get(&id).map(|n| n.storage_offset) {
                    let (seg_id, local_off) = unpack_offset(packed);
                    if let Some(vs) = self.vector_store.get(seg_id as usize) {
                        let mut vstore = vs.write();
                        if let Some(mut header) = vstore.read_header(local_off) {
                            header.flags |= FLAG_TOMBSTONE;
                            vstore.write_header(local_off, &header)?;
                        }
                    }
                }
            }
            for &id in ids {
                hnsw.nodes.remove(&id);
                if hnsw.entry_point.load(Ordering::Relaxed) == id {
                    let new_ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
                    hnsw.entry_point.store(new_ep, Ordering::Relaxed);
                }
            }
        }

        // Phase 4: backend batch delete
        {
            let mut kv_ops: Vec<BackendWriteOp> = Vec::with_capacity(ids.len());
            for &id in ids {
                let key = id.to_le_bytes();
                kv_ops.push(BackendWriteOp::Delete {
                    partition: BackendPartition::Default,
                    key: key.to_vec(),
                });
            }
            self.backend.write_batch(kv_ops)?;
        }

        // Phase 5: volatile cache removal
        {
            let mut cache = self.volatile_cache.write();
            for &id in ids {
                cache.remove(&id);
            }
        }

        Ok(())
    }

    /// Permanently remove all traces of a node from all backend partitions.
    pub fn purge_permanent(&self, id: u128) -> Result<()> {
        self.ensure_writable()?;
        let key = id.to_le_bytes();
        self.backend.write_batch(vec![
            BackendWriteOp::Delete {
                partition: BackendPartition::Default,
                key: key.to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::TombstoneStorage,
                key: key.to_vec(),
            },
            BackendWriteOp::Delete {
                partition: BackendPartition::Tombstones,
                key: key.to_vec(),
            },
        ])
    }

    /// Check whether a node has been marked as deleted in the tombstones partition.
    pub fn is_deleted(&self, id: u128) -> Result<bool> {
        let key = id.to_le_bytes();
        match self.backend.get(BackendPartition::Tombstones, &key)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}
