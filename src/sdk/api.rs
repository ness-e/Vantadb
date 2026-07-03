use super::builder::VantaEmbedded;
use super::serialization::{
    memory_node_id, memory_record_from_node, memory_record_to_node_owned, now_ms, validate_key,
    validate_metadata, validate_namespace, FIELD_EXPIRES_AT_MS, FIELD_KEY, FIELD_NAMESPACE,
};
use super::types::*;
use crate::backend::BackendPartition;
use crate::error::{Result, VantaError};
use crate::executor::Executor;
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use tracing;

impl VantaEmbedded {
    /// Insert or update a node directly. The `input` provides id, content, vector, and fields.
    #[tracing::instrument(skip(self), err)]
    pub fn insert_node(&self, input: VantaNodeInput) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut node = UnifiedNode::new(input.id);

        if let Some(content) = input.content {
            node.set_field("content", FieldValue::String(content));
        }

        for (key, value) in input.fields {
            node.set_field(key, value.into());
        }

        if let Some(vector) = input.vector.filter(|vector| !vector.is_empty()) {
            node.vector = VectorRepresentations::Full(vector);
            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        }

        engine.insert(&node)
    }

    /// Retrieve a node by its numeric id. Returns `None` if the id does not exist.
    #[tracing::instrument(skip(self), err)]
    pub fn get_node(&self, id: u64) -> Result<Option<VantaNodeRecord>> {
        self.engine_handle()?
            .get(id)
            .map(|node| node.map(Into::into))
    }

    /// Delete a node by its numeric id. The `reason` string is recorded for auditing.
    #[tracing::instrument(skip(self), err)]
    pub fn delete_node(&self, id: u64, reason: &str) -> Result<()> {
        self.engine_handle()?.delete(id, reason)
    }

    /// Insert or update a persistent memory record.
    /// Returns the created/updated record with system-assigned timestamps and version.
    #[tracing::instrument(skip(self, input), err)]
    pub fn put(&self, input: VantaMemoryInput) -> Result<VantaMemoryRecord> {
        validate_namespace(&input.namespace)?;
        validate_key(&input.key)?;
        validate_metadata(&input.metadata)?;

        let engine = self.engine_handle()?;
        let node_id = memory_node_id(&input.namespace, &input.key);
        let existing = match engine.get(node_id)? {
            Some(node) => match memory_record_from_node(node) {
                Some(record) if record.namespace == input.namespace && record.key == input.key => {
                    Some(record)
                }
                _ => None,
            },
            None => None,
        };

        let timestamp = now_ms();
        let created_at_ms = existing
            .as_ref()
            .map(|r| r.created_at_ms)
            .unwrap_or(timestamp);
        let version = existing
            .as_ref()
            .map(|r| r.version.saturating_add(1))
            .unwrap_or(1);
        let expires_at_ms = input.ttl_ms.map(|ttl| timestamp.saturating_add(ttl));

        let record = VantaMemoryRecord {
            namespace: input.namespace,
            key: input.key,
            payload: input.payload,
            metadata: input.metadata,
            created_at_ms,
            updated_at_ms: timestamp,
            version,
            node_id,
            vector: input.vector.filter(|v| !v.is_empty()),
            expires_at_ms,
        };
        let (node, record) = memory_record_to_node_owned(record);
        engine.insert(&node)?;
        self.replace_derived_indexes(&engine, existing.as_ref(), Some(&record))?;

        Ok(record)
    }

    /// Insert or update multiple namespace-scoped persistent memory records in parallel.
    #[tracing::instrument(skip(self, inputs), err)]
    pub fn put_batch(&self, inputs: Vec<VantaMemoryInput>) -> Result<Vec<VantaMemoryRecord>> {
        #[cfg(feature = "rayon")]
        use rayon::prelude::*;

        for input in &inputs {
            validate_namespace(&input.namespace)?;
            validate_key(&input.key)?;
            validate_metadata(&input.metadata)?;
        }

        let results: Vec<Result<VantaMemoryRecord>> = {
            #[cfg(feature = "rayon")]
            {
                inputs.into_par_iter()
            }
            #[cfg(not(feature = "rayon"))]
            {
                inputs.into_iter()
            }
        }
        .map(|input| {
            let engine = self.engine_handle()?;
            let node_id = memory_node_id(&input.namespace, &input.key);
            let existing = match engine.get(node_id)? {
                Some(node) => match memory_record_from_node(node) {
                    Some(record)
                        if record.namespace == input.namespace && record.key == input.key =>
                    {
                        Some(record)
                    }
                    _ => {
                        return Err(VantaError::NodeIdCollision(
                            memory_node_id(&input.namespace, &input.key),
                        ));
                    }
                },
                None => None,
            };

            let timestamp = now_ms();
            let created_at_ms = existing
                .as_ref()
                .map(|record| record.created_at_ms)
                .unwrap_or(timestamp);
            let version = existing
                .as_ref()
                .map(|record| record.version.saturating_add(1))
                .unwrap_or(1);
            let expires_at_ms = input.ttl_ms.map(|ttl| timestamp.saturating_add(ttl));

            let record = VantaMemoryRecord {
                namespace: input.namespace,
                key: input.key,
                payload: input.payload,
                metadata: input.metadata,
                created_at_ms,
                updated_at_ms: timestamp,
                version,
                node_id,
                vector: input.vector.filter(|v| !v.is_empty()),
                expires_at_ms,
            };
            let (node, record) = memory_record_to_node_owned(record);
            engine.insert(&node)?;
            self.replace_derived_indexes(&engine, existing.as_ref(), Some(&record))?;
            Ok(record)
        })
        .collect();

        results.into_iter().collect()
    }

    /// Retrieve a single memory record by namespace and key.
    /// Returns `None` if the record does not exist or has expired.
    #[tracing::instrument(skip(self), err)]
    pub fn get(&self, namespace: &str, key: &str) -> Result<Option<VantaMemoryRecord>> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let node_id = memory_node_id(namespace, key);
        let Some(node) = self.engine_handle()?.get(node_id)? else {
            return Ok(None);
        };

        match memory_record_from_node(node) {
            Some(record) if record.namespace == namespace && record.key == key => Ok(Some(record)),
            Some(_record) => Err(VantaError::NodeIdCollision(memory_node_id(namespace, key))),
            None => Ok(None),
        }
    }

    /// Delete a memory record by namespace and key.
    /// Returns `true` if a record was actually deleted, `false` if it did not exist.
    #[tracing::instrument(skip(self), err)]
    pub fn delete(&self, namespace: &str, key: &str) -> Result<bool> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let Some(existing) = self.get(namespace, key)? else {
            return Ok(false);
        };

        let node_id = memory_node_id(namespace, key);
        let engine = self.engine_handle()?;
        engine.delete(node_id, "memory delete")?;
        self.replace_derived_indexes(&engine, Some(&existing), None)?;
        Ok(true)
    }

    /// Insert or update a record with exact fields (used internally by import).
    pub(crate) fn put_record_exact(&self, record: VantaMemoryRecord) -> Result<VantaMemoryRecord> {
        validate_namespace(&record.namespace)?;
        validate_key(&record.key)?;
        validate_metadata(&record.metadata)?;

        let expected_node_id = memory_node_id(&record.namespace, &record.key);
        if record.node_id != expected_node_id {
            return Err(VantaError::ValidationError {
                field: "node_id".into(),
                reason: format!("node_id does not match deterministic namespace/key hash for namespace='{}' key='{}'", record.namespace, record.key),
            });
        }

        let engine = self.engine_handle()?;
        let previous = match engine.get(record.node_id)? {
            Some(node) => match memory_record_from_node(node) {
                Some(previous)
                    if previous.namespace == record.namespace && previous.key == record.key =>
                {
                    Some(previous)
                }
                _ => {
                    return Err(VantaError::NodeIdCollision(record.node_id));
                }
            },
            None => None,
        };

        let (node, record) = memory_record_to_node_owned(record);
        engine.insert(&node)?;
        self.replace_derived_indexes(&engine, previous.as_ref(), Some(&record))?;

        Ok(record)
    }

    /// List all namespaces that contain at least one memory record.
    #[tracing::instrument(skip(self), err)]
    pub fn list_namespaces(&self) -> Result<Vec<String>> {
        let engine = self.engine_handle()?;
        let mut namespaces = std::collections::BTreeSet::new();
        let entries = engine.scan_partition(BackendPartition::NamespaceIndex)?;

        if entries.is_empty() {
            for node in engine.scan_nodes()? {
                if let Some(record) = memory_record_from_node(node) {
                    namespaces.insert(record.namespace);
                }
            }
        } else {
            for (key, _value) in entries {
                if let Some(separator) = key.iter().position(|byte| *byte == 0) {
                    if let Ok(namespace) = String::from_utf8(key[..separator].to_vec()) {
                        namespaces.insert(namespace);
                    }
                }
            }
        }

        Ok(namespaces.into_iter().collect())
    }

    /// List memory records in a namespace with optional filters, cursor-based pagination, and limit.
    #[tracing::instrument(skip(self), err)]
    pub fn list(
        &self,
        namespace: &str,
        options: VantaMemoryListOptions,
    ) -> Result<VantaMemoryListPage> {
        validate_namespace(namespace)?;
        validate_metadata(&options.filters)?;

        let records = self.records_for_namespace(namespace, &options.filters)?;

        let start = options.cursor.unwrap_or(0).min(records.len());
        let limit = options.limit.max(1);
        let end = start.saturating_add(limit).min(records.len());
        let next_cursor = (end < records.len()).then_some(end);

        Ok(VantaMemoryListPage {
            records: records[start..end].to_vec(),
            next_cursor,
        })
    }

    /// Rebuild the HNSW vector index, derived indexes, and text index from scratch.
    #[tracing::instrument(skip(self), err)]
    pub fn rebuild_index(&self) -> Result<VantaIndexRebuildReport> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "rebuild_index is not available when VantaDB is opened read-only".into(),
            });
        }
        let report = self.engine_handle()?.rebuild_vector_index()?;
        let derived = self.rebuild_derived_indexes_with_report()?;
        self.rebuild_text_index_with_report()?;
        let mut report: VantaIndexRebuildReport = report.into();
        report.derived_rebuild_ms = derived.duration_ms;
        Ok(report)
    }

    /// Compact the vector store file, grouping nodes in BFS order from the HNSW entry point.
    #[tracing::instrument(skip(self), err)]
    pub fn compact_layout(&self) -> Result<u64> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "compact_layout is not available when VantaDB is opened read-only".into(),
            });
        }
        self.engine_handle()?.compact_layout_bfs()
    }

    /// Flush WAL and memory-mapped files to disk.
    #[tracing::instrument(skip(self), err)]
    pub fn flush(&self) -> Result<()> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "flush is not available when VantaDB is opened read-only".into(),
            });
        }
        self.engine_handle()?.flush()
    }

    /// Compact the WAL: flush, archive the current WAL file, and start a fresh one.
    #[tracing::instrument(skip(self), err)]
    pub fn compact_wal(&self) -> Result<()> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "compact_wal is not available when VantaDB is opened read-only".into(),
            });
        }
        self.engine_handle()?.compact_wal()
    }

    /// Scan all memory records and physically delete those whose expiry deadline has passed.
    #[tracing::instrument(skip(self), err)]
    pub fn purge_expired(&self) -> Result<u64> {
        if self.config.read_only {
            return Err(VantaError::Execution(
                "purge_expired is not available when VantaDB is opened read-only".to_string(),
            ));
        }
        let engine = self.engine_handle()?;
        let now = now_ms();
        let mut to_delete: Vec<(String, String, u64)> = Vec::new();

        for node in engine.scan_nodes()? {
            if !node.is_alive() {
                continue;
            }
            let namespace = match node.get_field(FIELD_NAMESPACE) {
                Some(crate::node::FieldValue::String(ns)) => ns.clone(),
                _ => continue,
            };
            let key = match node.get_field(FIELD_KEY) {
                Some(crate::node::FieldValue::String(k)) => k.clone(),
                _ => continue,
            };
            let expires = match node.get_field(FIELD_EXPIRES_AT_MS) {
                Some(crate::node::FieldValue::Int(ms)) if *ms > 0 => *ms as u64,
                _ => continue,
            };
            if now > expires {
                to_delete.push((namespace, key, node.id));
            }
        }

        let count = to_delete.len() as u64;
        for (_, _, node_id) in &to_delete {
            engine.delete(*node_id, "purge_expired")?;
            self.replace_derived_indexes(&engine, None, None)?;
        }

        Ok(count)
    }

    /// Return stable runtime capabilities.
    #[tracing::instrument(skip(self))]
    pub fn capabilities(&self) -> VantaCapabilities {
        VantaCapabilities {
            runtime_profile: VantaRuntimeProfile::Performance,
            persistence: true,
            vector_search: true,
            iql_queries: true,
            read_only: self.config.read_only,
        }
    }

    /// Add a directed edge between two nodes.
    #[tracing::instrument(skip(self), err)]
    pub fn add_edge(
        &self,
        source_id: u64,
        target_id: u64,
        label: &str,
        weight: Option<f32>,
    ) -> Result<()> {
        let engine = self.engine_handle()?;
        let mut node = engine
            .get(source_id)?
            .ok_or(VantaError::NodeNotFound(source_id))?;
        node.edges.push(crate::node::Edge {
            target: target_id,
            label: label.to_string(),
            weight: weight.unwrap_or(1.0),
        });
        engine.insert(&node)
    }

    /// Execute an IQL query.
    #[tracing::instrument(skip(self), err)]
    pub fn query(&self, query: &str) -> Result<VantaQueryResult> {
        let engine = self.engine_handle()?;
        let executor = Executor::new(&engine);
        let result = executor.execute_hybrid(query)?;
        Ok(result.into())
    }

    /// Snapshot of current process-level operational metrics.
    #[tracing::instrument(skip(self))]
    pub fn operational_metrics(&self) -> VantaOperationalMetrics {
        if let Ok(engine) = self.engine_handle() {
            let stats = engine.get_memory_stats();
            crate::metrics::record_memory_breakdown(
                stats.node_count,
                stats.logical_bytes,
                stats.physical_rss,
                stats.cache_entries as u64,
                0,
            );
        }
        crate::metrics::operational_metrics_snapshot().into()
    }

    /// K-NN vector search across all nodes via HNSW index.
    #[tracing::instrument(skip(self, vector), err)]
    pub fn search_vector(&self, vector: &[f32], top_k: usize) -> Result<Vec<VantaSearchHit>> {
        if vector.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }
        let engine = self.engine_handle()?;
        let results = {
            let hnsw = engine.hnsw.load();
            let vs = engine.vector_store.read();
            hnsw.search_nearest(vector, None, None, u128::MAX, top_k, Some(&*vs))
        };
        Ok(results
            .into_iter()
            .map(|(node_id, distance)| VantaSearchHit { node_id, distance })
            .collect())
    }
}
