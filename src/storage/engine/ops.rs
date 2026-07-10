//! Core CRUD operations: insert, get, get_many, delete, purge, scan.

use std::sync::atomic::Ordering;
use web_time::{SystemTime, UNIX_EPOCH};

use crate::backend::{BackendPartition, BackendWriteOp};
use crate::error::Result;
use crate::node::{FieldValue, FilterBitset, UnifiedNode, VectorRepresentations};
use crate::storage::engine::StorageEngine;
use crate::storage::engine::{EvictionReason, FLAG_TOMBSTONE};
use crate::storage::ops::NodeMetadata;
use crate::wal::WalRecord;

impl StorageEngine {
    /// Insert or overwrite a node: persist to WAL, vector store, KV backend, and HNSW index.
    #[tracing::instrument(skip(self, node), level = "debug", err)]
    pub fn insert(&self, node: &UnifiedNode) -> Result<()> {
        self.check_memory_pressure()?;
        if let Ok(Some(existing_node)) = self.get(node.id) {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in &existing_node.relational {
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

            // PERF-07: remove old edges from global index
            if let Some(ref ei) = self.edge_index {
                for edge in &existing_node.edges {
                    ei.remove_edge(node.id, edge.target);
                }
            }
            // PERF-08: remove old scalar entries
            if let Some(ref si) = self.scalar_index {
                for (field, value) in &existing_node.relational {
                    si.remove(field, value, node.id);
                }
            }
        }

        {
            let mut stats = self.cardinality_stats.write();
            for (field, value) in &node.relational {
                let val_keys = value.to_cardinality_keys();
                let val_map = stats.entry(field.clone()).or_default();
                for val_key in val_keys {
                    if val_map.len() < 100 || val_map.contains_key(&val_key) {
                        *val_map.entry(val_key).or_default() += 1;
                    }
                }
            }
        }

        // PERF-07: add new edges to global index
        if let Some(ref ei) = self.edge_index {
            for edge in &node.edges {
                ei.insert(node.id, edge.target);
            }
        }
        // PERF-08: add new scalar entries
        if let Some(ref si) = self.scalar_index {
            for (field, value) in &node.relational {
                si.insert(field, value, node.id);
            }
        }

        self.ensure_writable()?;
        #[cfg(feature = "failpoints")]
        fail::fail_point!("storage_insert_fail", |_| {
            Err(crate::error::VantaError::IoError(std::io::Error::other(
                "Simulated Storage insert catastrophic I/O failure",
            )))
        });

        self.touch_activity();

        let mut active_node = node.clone();
        active_node.last_accessed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        if let Some(ref sharded) = self.wal {
            sharded.append(&crate::wal::WalRecord::Insert(active_node.clone()))?;
        }

        let mut vstore = self.vector_store.write();
        let storage_offset = Self::write_node_to_vstore(&mut vstore, &active_node)?;

        let key = active_node.id.to_le_bytes();
        let metadata = NodeMetadata {
            relational: active_node.relational.clone(),
            edges: active_node.edges.clone(),
        };
        let metadata_val = postcard::to_allocvec(&metadata)
            .map_err(|e| crate::error::VantaError::SerializationError(e.to_string()))?;
        self.backend
            .put(BackendPartition::Default, &key, &metadata_val)?;

        {
            let _guard = self
                .insert_lock
                .try_lock_for(std::time::Duration::from_millis(
                    self.config.insert_lock_timeout_ms,
                ))
                .ok_or_else(|| crate::error::VantaError::Timeout {
                    operation: "acquire insert_lock in update_node".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                })?;
            let hnsw = self.hnsw.load();
            hnsw.add(
                active_node.id,
                active_node.bitset.clone(),
                active_node.vector.clone(),
                storage_offset,
            );
        }

        if active_node.tier == crate::node::NodeTier::Hot {
            let mut cache = self.volatile_cache.write();
            cache.insert(active_node.id, active_node.clone());

            let caps = crate::hardware::HardwareCapabilities::global();
            let cache_cap_bytes = caps.total_memory / 4;
            let approx_node_size = 1536;
            let max_nodes = (cache_cap_bytes / approx_node_size) as usize;

            if cache.len() > max_nodes {
                self.emergency_maintenance_trigger
                    .store(true, Ordering::Release);
                if let Err(e) = self.evict_cold_nodes_with_reason(
                    self.config.eviction_ratio,
                    EvictionReason::Watermark,
                ) {
                    tracing::warn!("eviction failed: {e}");
                }
            }
        }

        // PERF-30: auto-flush when total node count exceeds flush_threshold
        if let Some(threshold) = self.config.flush_threshold {
            let hnsw = self.hnsw.load();
            if hnsw.nodes.len() >= threshold {
                drop(hnsw);
                if let Err(e) = self.flush() {
                    tracing::warn!("auto-flush failed: {e}");
                }
            }
        }

        Ok(())
    }

    /// Insert multiple nodes in a single batch operation.
    ///
    /// Reduces I/O and lock contention by batching WAL records, KV backend writes,
    /// and acquiring the HNSW insert lock once for all nodes.
    pub fn batch_insert(&self, nodes: &[UnifiedNode]) -> Result<()> {
        if nodes.is_empty() {
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
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut wal_records: Vec<WalRecord> = Vec::with_capacity(nodes.len());
        let mut kv_ops: Vec<BackendWriteOp> = Vec::with_capacity(nodes.len());
        let mut hnsw_entries: Vec<(u128, FilterBitset, VectorRepresentations, u64)> =
            Vec::with_capacity(nodes.len());

        let mut vstore = self.vector_store.write();
        {
            let mut stats = self.cardinality_stats.write();
            for node in nodes {
                if let Ok(Some(existing_node)) = self.get(node.id) {
                    for (field, value) in &existing_node.relational {
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
                        for edge in &existing_node.edges {
                            ei.remove_edge(node.id, edge.target);
                        }
                    }
                    if let Some(ref si) = self.scalar_index {
                        for (field, value) in &existing_node.relational {
                            si.remove(field, value, node.id);
                        }
                    }
                }

                for (field, value) in &node.relational {
                    let val_keys = value.to_cardinality_keys();
                    let val_map = stats.entry(field.clone()).or_default();
                    for val_key in val_keys {
                        if val_map.len() < 100 || val_map.contains_key(&val_key) {
                            *val_map.entry(val_key).or_default() += 1;
                        }
                    }
                }

                if let Some(ref ei) = self.edge_index {
                    for edge in &node.edges {
                        ei.insert(node.id, edge.target);
                    }
                }
                if let Some(ref si) = self.scalar_index {
                    for (field, value) in &node.relational {
                        si.insert(field, value, node.id);
                    }
                }
            }
        }

        for node in nodes {
            let mut active_node = node.clone();
            active_node.last_accessed = now_ms;
            let storage_offset = Self::write_node_to_vstore(&mut vstore, &active_node)?;
            hnsw_entries.push((
                active_node.id,
                active_node.bitset.clone(),
                active_node.vector.clone(),
                storage_offset,
            ));
            wal_records.push(WalRecord::Insert(active_node.clone()));

            let key = active_node.id.to_le_bytes();
            let metadata = NodeMetadata {
                relational: active_node.relational.clone(),
                edges: active_node.edges.clone(),
            };
            let metadata_val = postcard::to_allocvec(&metadata)
                .map_err(|e| crate::error::VantaError::SerializationError(e.to_string()))?;
            kv_ops.push(BackendWriteOp::Put {
                partition: BackendPartition::Default,
                key: key.to_vec(),
                value: metadata_val,
            });
        }
        drop(vstore);

        if let Some(ref sharded) = self.wal {
            sharded.batch_append(&wal_records)?;
        }

        self.backend.write_batch(kv_ops)?;

        {
            let _guard = self
                .insert_lock
                .try_lock_for(std::time::Duration::from_millis(
                    self.config.insert_lock_timeout_ms,
                ))
                .ok_or_else(|| crate::error::VantaError::Timeout {
                    operation: "acquire insert_lock in batch_insert".into(),
                    duration_ms: self.config.insert_lock_timeout_ms,
                })?;
            let hnsw = self.hnsw.load();
            for (id, bitset, vector, offset) in &hnsw_entries {
                hnsw.add(*id, bitset.clone(), vector.clone(), *offset);
            }
        }

        {
            let mut cache = self.volatile_cache.write();
            for node in nodes {
                if node.tier == crate::node::NodeTier::Hot {
                    cache.insert(node.id, node.clone());
                }
            }
            let caps = crate::hardware::HardwareCapabilities::global();
            let cache_cap_bytes = caps.total_memory / 4;
            let approx_node_size = 1536;
            let max_nodes = (cache_cap_bytes / approx_node_size) as usize;
            if cache.len() > max_nodes {
                self.emergency_maintenance_trigger
                    .store(true, Ordering::Release);
                if let Err(e) = self.evict_cold_nodes_with_reason(
                    self.config.eviction_ratio,
                    EvictionReason::Watermark,
                ) {
                    tracing::warn!("eviction failed: {e}");
                }
            }
        }

        // PERF-30: auto-flush when total node count exceeds flush_threshold
        if let Some(threshold) = self.config.flush_threshold {
            let hnsw = self.hnsw.load();
            if hnsw.nodes.len() >= threshold {
                drop(hnsw);
                if let Err(e) = self.flush() {
                    tracing::warn!("auto-flush failed: {e}");
                }
            }
        }

        Ok(())
    }

    /// Insert multiple SDK records in a single batch operation.
    ///
    /// Converts `VantaNodeInput` records to internal `UnifiedNode` nodes,
    /// then delegates to `batch_insert` for batched persistence.
    /// Returns the IDs of all inserted records.
    #[tracing::instrument(skip(self, records), level = "debug", err)]
    pub fn insert_batch(&self, records: &[crate::VantaNodeInput]) -> Result<Vec<u128>> {
        let nodes: Vec<UnifiedNode> = records
            .iter()
            .map(|input| {
                let mut node = UnifiedNode::new(input.id);
                if let Some(content) = &input.content {
                    node.set_field("content", FieldValue::String(content.clone()));
                }
                for (key, value) in &input.fields {
                    node.set_field(key.clone(), FieldValue::from(value.clone()));
                }
                if let Some(vector) = &input.vector {
                    if !vector.is_empty() {
                        node.vector = VectorRepresentations::Full(vector.clone());
                        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                    }
                }
                node
            })
            .collect();

        self.batch_insert(&nodes)?;

        Ok(records.iter().map(|r| r.id).collect())
    }

    /// Retrieve a node by its numeric ID, checking the volatile cache first.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn get(&self, id: u128) -> Result<Option<UnifiedNode>> {
        self.touch_activity();

        self.quantization_governor.record_access(id);

        {
            let mut cache = self.volatile_cache.write();
            if let Some(node) = cache.get_mut(&id) {
                if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                    return Ok(None);
                }
                node.hits += 1;
                node.last_accessed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                return Ok(Some(node.clone()));
            }
        }

        let key = id.to_le_bytes();
        let metadata_res = match self.backend.get(BackendPartition::Default, &key)? {
            Some(res) => res,
            None => return Ok(None),
        };

        let metadata: NodeMetadata = postcard::from_bytes(&metadata_res)
            .map_err(|e| crate::error::VantaError::SerializationError(e.to_string()))?;

        let hnsw = self.hnsw.load();
        let index_node = match hnsw.nodes.get(&id) {
            Some(n) => n,
            None => return Ok(None),
        };
        let storage_offset = index_node.storage_offset;

        let vstore = self.vector_store.read();
        let header = match vstore.read_header(storage_offset) {
            Some(h) => h,
            None => return Ok(None),
        };

        if (header.flags & FLAG_TOMBSTONE) != 0 {
            return Ok(None);
        }

        let vec_start = header.vector_offset as usize;
        let vec_end = vec_start + (header.vector_len as usize * 4);
        if vec_end > vstore.size as usize {
            return Ok(None);
        }

        let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
        // SAFETY: `vec_end > vstore.size as usize` guard above ensures the
        // slice is within mmap bounds. `vec_start` is 64-byte aligned per
        // VantaFile layout, guaranteeing f32 (4-byte) alignment.
        let f32_vec: &[f32] = unsafe {
            std::slice::from_raw_parts(vec_bytes.as_ptr() as *const f32, header.vector_len as usize)
        };

        let mut node = UnifiedNode::new(id);
        node.bitset = FilterBitset::from_u128(header.bitset);
        node.vector = VectorRepresentations::Full(f32_vec.to_vec());
        node.relational = metadata.relational;
        node.edges = metadata.edges;
        node.confidence_score = header.confidence_score;
        node.importance = header.importance;
        node.tier = if header.tier == 1 {
            crate::node::NodeTier::Hot
        } else {
            crate::node::NodeTier::Cold
        };
        node.flags = crate::node::NodeFlags(header.flags);

        Ok(Some(node))
    }

    /// Retrieve multiple nodes by ID in a single batch operation.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn get_many(&self, ids: &[u128]) -> Result<Vec<UnifiedNode>> {
        self.touch_activity();

        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut results: Vec<UnifiedNode> = Vec::with_capacity(ids.len());

        let ids_with_keys: Vec<(u128, Vec<u8>)> = ids
            .iter()
            .map(|id| (*id, id.to_le_bytes().to_vec()))
            .collect();

        let mut remaining_indices: Vec<usize> = Vec::new();
        {
            let mut cache = self.volatile_cache.write();
            for (i, &id) in ids.iter().enumerate() {
                self.quantization_governor.record_access(id);
                if let Some(node) = cache.get_mut(&id) {
                    if node.flags.is_set(crate::node::NodeFlags::TOMBSTONE) {
                        continue;
                    }
                    node.hits += 1;
                    node.last_accessed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    results.push(node.clone());
                } else {
                    remaining_indices.push(i);
                }
            }
        }

        if remaining_indices.is_empty() {
            return Ok(results);
        }

        let remaining_keys: Vec<&[u8]> = remaining_indices
            .iter()
            .map(|&i| ids_with_keys[i].1.as_slice())
            .collect();

        let backend_results = self
            .backend
            .get_many(BackendPartition::Default, &remaining_keys)?;

        let mut backend_map: std::collections::HashMap<u128, Vec<u8>> =
            std::collections::HashMap::with_capacity(backend_results.len());
        for (k, v) in backend_results {
            let key_slice: [u8; 16] = k.as_slice().try_into().map_err(|_| {
                crate::error::VantaError::BackendError(format!(
                    "corrupt backend: key length {} != 16",
                    k.len()
                ))
            })?;
            backend_map.insert(u128::from_le_bytes(key_slice), v);
        }

        let hnsw = self.hnsw.load();
        let vstore = self.vector_store.read();

        for &i in &remaining_indices {
            let id = ids[i];
            let Some(metadata_bytes) = backend_map.get(&id) else {
                continue;
            };

            let metadata: NodeMetadata = match postcard::from_bytes(metadata_bytes) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let Some(index_node) = hnsw.nodes.get(&id) else {
                continue;
            };
            let storage_offset = index_node.storage_offset;

            let Some(header) = vstore.read_header(storage_offset) else {
                continue;
            };

            if (header.flags & FLAG_TOMBSTONE) != 0 {
                continue;
            }

            let vec_start = header.vector_offset as usize;
            let vec_end = vec_start + (header.vector_len as usize * 4);
            if vec_end > vstore.size as usize {
                continue;
            }

            let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
            // SAFETY: bounds verified above (`vec_end > vstore.size` guard).
            // Same alignment guarantees as the get_node path above.
            let f32_vec: &[f32] = unsafe {
                std::slice::from_raw_parts(
                    vec_bytes.as_ptr() as *const f32,
                    header.vector_len as usize,
                )
            };

            let mut node = UnifiedNode::new(id);
            node.bitset = FilterBitset::from_u128(header.bitset);
            node.vector = VectorRepresentations::Full(f32_vec.to_vec());
            node.relational = metadata.relational;
            node.edges = metadata.edges;
            node.confidence_score = header.confidence_score;
            node.importance = header.importance;
            node.tier = if header.tier == 1 {
                crate::node::NodeTier::Hot
            } else {
                crate::node::NodeTier::Cold
            };
            node.flags = crate::node::NodeFlags(header.flags);

            results.push(node);
        }

        Ok(results)
    }

    /// Mark a node as deleted: write tombstone, remove from cache and backend.
    #[tracing::instrument(skip(self), level = "debug", err)]
    pub fn delete(&self, id: u128, _reason: &str) -> Result<()> {
        self.check_memory_pressure()?;
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

            // PERF-07: cascade — remove all edges referencing this node
            if let Some(ref ei) = self.edge_index {
                ei.remove_all_for_node(id);
            }
            // PERF-08: remove node from scalar index
            if let Some(ref si) = self.scalar_index {
                si.remove_node(id);
            }
        }

        self.ensure_writable()?;
        if let Some(ref sharded) = self.wal {
            sharded.append(&crate::wal::WalRecord::Delete { id })?;
        }

        let hnsw = self.hnsw.load();
        let offset = hnsw.nodes.get(&id).map(|n| n.storage_offset);

        if let Some(offset) = offset {
            let mut vstore = self.vector_store.write();
            if let Some(mut header) = vstore.read_header(offset) {
                header.flags |= FLAG_TOMBSTONE;
                vstore.write_header(offset, &header)?;
            }
        }

        hnsw.nodes.remove(&id);

        // PERF-23: If we just removed the entry point, promote a replacement
        {
            let mut ep = hnsw.entry_point.lock();
            if *ep == id {
                *ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
            }
        }

        self.volatile_cache.write().remove(&id);

        let key = id.to_le_bytes();
        self.backend.delete(BackendPartition::Default, &key)?;

        Ok(())
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
        }

        // Phase 2: WAL batch append
        let wal_records: Vec<WalRecord> = ids.iter().map(|&id| WalRecord::Delete { id }).collect();
        if let Some(ref sharded) = self.wal {
            sharded.batch_append(&wal_records)?;
        }

        // Phase 3: HNSW node removal + vector store tombstone marking
        {
            let hnsw = self.hnsw.load();
            {
                let mut vstore = self.vector_store.write();
                for &id in ids {
                    if let Some(offset) = hnsw.nodes.get(&id).map(|n| n.storage_offset) {
                        if let Some(mut header) = vstore.read_header(offset) {
                            header.flags |= FLAG_TOMBSTONE;
                            vstore.write_header(offset, &header)?;
                        }
                    }
                }
            }
            for &id in ids {
                hnsw.nodes.remove(&id);
                let mut ep = hnsw.entry_point.lock();
                if *ep == id {
                    *ep = hnsw.find_new_entry_point().unwrap_or(u128::MAX);
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

    /// Insert a node into a specific backend column family and update the HNSW index.
    pub fn insert_to_cf(&self, node: &UnifiedNode, cf_name: &str) -> Result<()> {
        self.ensure_writable()?;
        let partition = crate::storage::ops::partition_from_cf_name(cf_name)?;
        let key = node.id.to_le_bytes();
        let val = postcard::to_allocvec(node)
            .map_err(|e| crate::error::VantaError::SerializationError(e.to_string()))?;
        self.backend.put(partition, &key, &val)?;

        let mut vstore = self.vector_store.write();
        let storage_offset = Self::write_node_to_vstore(&mut vstore, node)?;
        self.refresh_index(node, storage_offset)?;
        Ok(())
    }

    /// Return all currently readable nodes from the primary backend partition.
    pub fn scan_nodes(&self) -> Result<Vec<UnifiedNode>> {
        let (nodes, _) = self.scan_nodes_page("", usize::MAX)?;
        Ok(nodes)
    }

    /// Paginated scan: returns a page of nodes and the next cursor.
    pub fn scan_nodes_page(
        &self,
        cursor: &str,
        limit: usize,
    ) -> Result<(Vec<UnifiedNode>, String)> {
        let cursor_id: u128 = cursor.parse().unwrap_or(0);
        let entries = self.backend.scan(BackendPartition::Default)?;

        let raw_nodes = {
            let hnsw = self.hnsw.load();
            let vstore = self.vector_store.read();

            let mut collected = Vec::with_capacity(entries.len().min(limit));
            for (key, value) in entries {
                if collected.len() >= limit {
                    break;
                }
                if key.len() != std::mem::size_of::<u128>() {
                    continue;
                }

                let id = u128::from_le_bytes(
                    key.as_slice().try_into().expect("key slice fits [u8; 16]"),
                );
                if id <= cursor_id {
                    continue;
                }

                let metadata: NodeMetadata = match postcard::from_bytes(&value) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let index_node = match hnsw.nodes.get(&id) {
                    Some(n) => n,
                    None => continue,
                };
                let storage_offset = index_node.storage_offset;

                let header = match vstore.read_header(storage_offset) {
                    Some(h) => h,
                    None => continue,
                };

                if (header.flags & FLAG_TOMBSTONE) != 0 {
                    continue;
                }

                let vec_start = header.vector_offset as usize;
                let vec_end = vec_start + (header.vector_len as usize * 4);
                if vec_end > vstore.size as usize {
                    continue;
                }

                let vec_bytes = &vstore.mmap_bytes()[vec_start..vec_end];
                // SAFETY: bounds verified above. Same alignment guarantees as
                // the other `from_raw_parts` paths in this module.
                let f32_vec: Vec<f32> = unsafe {
                    std::slice::from_raw_parts(
                        vec_bytes.as_ptr() as *const f32,
                        header.vector_len as usize,
                    )
                }
                .to_vec();

                collected.push((id, metadata, header, f32_vec));
            }
            collected
        };

        let mut nodes = Vec::with_capacity(raw_nodes.len());
        let mut last_id = 0u128;
        for (id, metadata, header, f32_vec) in raw_nodes {
            last_id = id;
            let mut node = UnifiedNode::new(id);
            node.bitset = FilterBitset::from_u128(header.bitset);
            node.vector = VectorRepresentations::Full(f32_vec);
            node.relational = metadata.relational;
            node.edges = metadata.edges;
            node.confidence_score = header.confidence_score;
            node.importance = header.importance;
            node.tier = if header.tier == 1 {
                crate::node::NodeTier::Hot
            } else {
                crate::node::NodeTier::Cold
            };
            node.flags = crate::node::NodeFlags(header.flags);
            nodes.push(node);
        }

        let next_cursor = if nodes.len() == limit && limit > 0 {
            last_id.to_string()
        } else {
            String::new()
        };

        Ok((nodes, next_cursor))
    }
}
