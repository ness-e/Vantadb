use super::builder::VantaEmbedded;
use super::serialization::{
    is_scalar_indexable, matches_memory_filters, memory_node_id, memory_record_from_node,
    memory_record_to_node_owned, now_ms, validate_key, validate_metadata, validate_namespace,
    DERIVED_INDEX_SCHEMA_VERSION, FIELD_CREATED_AT_MS, FIELD_EXPIRES_AT_MS, FIELD_KEY,
    FIELD_NAMESPACE, FIELD_PAYLOAD, FIELD_UPDATED_AT_MS, FIELD_VERSION,
};
use super::types::*;
use crate::backend::{BackendKind, BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use crate::executor::Executor;
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use tracing;
use web_time::Instant;

impl VantaEmbedded {
    /// True when a vector is entirely zeros — the HNSW core rejects
    /// zero-norm vectors under cosine similarity (AUDREP-27), so the
    /// SDK treats them like empty vectors: keep the document, skip the
    /// vector index (matches `load.test.ts` seeding `[i % 10, 0, 0, 0]`).
    fn usable_vector(vector: &[f32]) -> bool {
        !vector.is_empty() && vector.iter().any(|x| *x != 0.0)
    }

    fn check_read_only(&self) -> Result<()> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "this operation is not available when VantaDB is opened read-only".into(),
            });
        }
        Ok(())
    }

    /// Insert or update a node directly. The `input` provides id, content, vector, and fields.
    #[tracing::instrument(skip(self), err)]
    pub fn insert_node(&self, input: VantaNodeInput) -> Result<()> {
        self.check_read_only()?;
        let engine = self.engine_handle()?;
        let mut node = UnifiedNode::new(input.id);

        if let Some(content) = input.content {
            node.set_field("content", FieldValue::String(content));
        }

        for (key, value) in input.fields {
            node.set_field(key, value.into());
        }

        if let Some(vector) = input.vector.filter(|v| Self::usable_vector(v)) {
            node.vector = VectorRepresentations::Full(vector);
            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        }

        engine.insert(&node)
    }

    /// Retrieve a node by its numeric id. Returns `None` if the id does not exist.
    #[tracing::instrument(skip(self), err)]
    pub fn get_node(&self, id: u128) -> Result<Option<VantaNodeRecord>> {
        let engine = self.engine_handle()?;
        engine
            .get(id)
            .map(|node| node.map(|n| engine.node_to_record(n)))
    }

    /// Delete a node by its numeric id. The `reason` string is recorded for auditing.
    #[tracing::instrument(skip(self), err)]
    pub fn delete_node(&self, id: u128, reason: &str) -> Result<()> {
        self.check_read_only()?;
        self.engine_handle()?.delete(id, reason)
    }

    /// Purge tombstoned nodes from the HNSW index (vacuum).
    ///
    /// Scans all HNSW nodes and removes those flagged as tombstones.
    /// Returns a [`VacuumReport`] with counts and timing.
    pub fn vacuum(&self) -> Result<crate::storage::engine::VacuumReport> {
        self.check_read_only()?;
        self.engine_handle()?.vacuum()
    }

    /// Run the segment optimizer pipeline (vacuum → merge → reindex).
    ///
    /// Each phase is logged independently; a phase failure does not abort
    /// subsequent phases.
    pub fn pipeline(
        &self,
        mode: crate::storage::engine::PipelineMode,
    ) -> Result<crate::storage::engine::PipelineReport> {
        self.check_read_only()?;
        self.engine_handle()?.run_pipeline(mode)
    }

    /// Return the current segment optimizer configuration.
    pub fn optimizer_config(&self) -> crate::storage::engine::SegmentOptimizerConfig {
        self.config.segment_optimizer
    }

    /// Override the segment optimizer configuration.
    ///
    /// The new config takes effect on the next pipeline invocation.
    pub fn set_optimizer_config(&mut self, cfg: crate::storage::engine::SegmentOptimizerConfig) {
        self.config.segment_optimizer = cfg;
    }

    /// Shared logic for inserting/updating a single memory record.
    /// Used by both `put()` and `put_batch()`.
    fn put_one(&self, input: VantaMemoryInput) -> Result<VantaMemoryRecord> {
        self.check_read_only()?;
        validate_namespace(&input.namespace)?;
        validate_key(&input.key)?;
        validate_metadata(&input.metadata)?;

        let engine = self.engine_handle()?;
        let node_id = memory_node_id(&input.namespace, &input.key);
        let existing = match engine.get(node_id)? {
            Some(node) => match memory_record_from_node(&node) {
                Some(record) if record.namespace == input.namespace && record.key == input.key => {
                    Some(record)
                }
                _ => {
                    return Err(VantaError::NodeIdCollision(memory_node_id(
                        &input.namespace,
                        &input.key,
                    )));
                }
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
            vector: input.vector.filter(|v| Self::usable_vector(v)),
            sparse_vector: input.sparse_vector,
            expires_at_ms,
        };
        let (node, record) = memory_record_to_node_owned(record);

        // Persist the node first (WAL + KV + HNSW)
        engine.insert(&node)?;

        // Best-effort JSON shredding — if this fails the record still works
        // via the existing derived-index / PostFilter paths.
        if !record.metadata.is_empty() {
            let _ = crate::shred::ShreddedRowStore::put(
                record.node_id,
                &record.metadata,
                &*engine.backend,
            );
        }

        self.replace_derived_indexes(&engine, existing.as_ref(), Some(&record))?;

        Ok(record)
    }

    /// Insert or update a persistent memory record.
    /// Returns the created/updated record with system-assigned timestamps and version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantadb::config::VantaConfig;
    /// use vantadb::{BackendKind, VantaEmbedded, VantaMemoryInput};
    ///
    /// let db = VantaEmbedded::open_with_config(VantaConfig {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// })
    /// .expect("open in-memory database");
    ///
    /// let record = db
    ///     .put(VantaMemoryInput::new("docs", "greeting", "Hello, VantaDB!"))
    ///     .expect("put record");
    ///
    /// assert_eq!(record.namespace, "docs");
    /// assert_eq!(record.key, "greeting");
    /// assert_eq!(record.payload, "Hello, VantaDB!");
    /// assert_eq!(record.version, 1);
    ///
    /// db.close().expect("close database");
    /// ```
    #[tracing::instrument(skip(self, input), err)]
    pub fn put(&self, input: VantaMemoryInput) -> Result<VantaMemoryRecord> {
        let (namespace, key) = (input.namespace.clone(), input.key.clone());
        let res = self.put_one(input);
        self.audit(crate::audit::AuditEvent::new(
            "put",
            &namespace,
            &key,
            if res.is_ok() { "ok" } else { "err" },
            None,
        ));
        res
    }

    /// Insert or update multiple namespace-scoped persistent memory records.
    ///
    /// Uses `batch_insert_with_opts()` internally — a single WAL `batch_append`,
    /// KV `write_batch`, and HNSW lock acquisition across all nodes in a chunk.
    /// Skips the per-node existence check (caller guarantees fresh inserts or
    /// uses `put()` for individual UPSERTS).
    #[tracing::instrument(skip(self, inputs), err)]
    pub fn put_batch(&self, inputs: Vec<VantaMemoryInput>) -> Result<Vec<VantaMemoryRecord>> {
        let (namespace, key) = inputs
            .first()
            .map(|i| (i.namespace.clone(), i.key.clone()))
            .unwrap_or_else(|| ("N/A".to_string(), "N/A".to_string()));
        let res = self.put_batch_inner(inputs);
        self.audit(crate::audit::AuditEvent::new(
            "put_batch",
            &namespace,
            &key,
            if res.is_ok() { "ok" } else { "err" },
            None,
        ));
        res
    }

    fn put_batch_inner(&self, inputs: Vec<VantaMemoryInput>) -> Result<Vec<VantaMemoryRecord>> {
        use crate::storage::engine::{BatchInsertOptions, InsertMode};

        for input in &inputs {
            validate_namespace(&input.namespace)?;
            validate_key(&input.key)?;
            validate_metadata(&input.metadata)?;
        }

        let engine = self.engine_handle()?;
        let batch_size = self.config.batch_size.unwrap_or(1000);
        let mut all_results: Vec<VantaMemoryRecord> = Vec::with_capacity(inputs.len());
        let mut rebuild_needed = false;
        // Track versions for keys seen earlier in this batch (in-batch dedup,
        // mirrors put_one's UPSERT semantics). Persisted before the chunk loop
        // so duplicate keys split across chunks still bump correctly.
        let mut seen_versions: HashMap<u128, u64> = HashMap::with_capacity(inputs.len());

        for chunk in inputs.chunks(batch_size) {
            let timestamp = now_ms();
            let mut nodes: Vec<UnifiedNode> = Vec::with_capacity(chunk.len());
            let mut records: Vec<VantaMemoryRecord> = Vec::with_capacity(chunk.len());

            for input in chunk {
                let node_id = memory_node_id(&input.namespace, &input.key);
                // Existing record: in-batch duplicate wins (already bumped), else
                // consult the engine like put_one (pre-existing records from
                // earlier batches should also increment, not reset to 1).
                let existing = if let Some(v) = seen_versions.get(&node_id) {
                    Some((*v, timestamp))
                } else {
                    match engine.get(node_id)? {
                        Some(node) => match memory_record_from_node(&node) {
                            Some(record)
                                if record.namespace == input.namespace
                                    && record.key == input.key =>
                            {
                                Some((record.version, record.created_at_ms))
                            }
                            _ => {
                                return Err(VantaError::NodeIdCollision(memory_node_id(
                                    &input.namespace,
                                    &input.key,
                                )));
                            }
                        },
                        None => None,
                    }
                };
                let (prev_version, prev_created_at_ms) = existing
                    .map(|(v, c)| (Some(v), Some(c)))
                    .unwrap_or((None, None));
                let created_at_ms = prev_created_at_ms.unwrap_or(timestamp);
                let version = prev_version.map(|v| v.saturating_add(1)).unwrap_or(1);
                seen_versions.insert(node_id, version);

                let record = VantaMemoryRecord {
                    namespace: input.namespace.clone(),
                    key: input.key.clone(),
                    payload: input.payload.clone(),
                    metadata: input.metadata.clone(),
                    created_at_ms,
                    updated_at_ms: timestamp,
                    version,
                    node_id,
                    vector: input.vector.clone().filter(|v| Self::usable_vector(v)),
                    sparse_vector: input.sparse_vector.clone(),
                    expires_at_ms: input.ttl_ms.map(|ttl| timestamp.saturating_add(ttl)),
                };
                let (node, record) = memory_record_to_node_owned(record);
                nodes.push(node);
                records.push(record);
            }

            // ── Single batch insert: WAL batch_append + KV write_batch ──
            // Auto mode — rebuild only if the chunk exceeds the incremental
            // threshold (default: 1000 nodes). In-memory backends (WASM,
            // `:memory:`) have no filesystem to persist a rebuilt index to,
            // so insert incrementally instead — rebuild_vector_index() calls
            // fs (mmap/persist) which fails on wasm32 with "IO error:
            // operation not supported on this platform" (load.test.ts).
            let use_auto = self.config.backend_kind != BackendKind::InMemory;
            let opts = BatchInsertOptions {
                skip_existing_check: true,
                skip_wal: false,
                insert_mode: if use_auto {
                    InsertMode::Auto
                } else {
                    InsertMode::Incremental
                },
                ..Default::default()
            };
            let chunk_needs_rebuild = opts.needs_rebuild(chunk.len());
            engine.batch_insert_with_opts(&nodes, opts)?;
            rebuild_needed = rebuild_needed || chunk_needs_rebuild;

            // ── Post-processing (same as put_one but without derived indexes for batch) ──
            for record in &records {
                if !record.metadata.is_empty() {
                    let _ = crate::shred::ShreddedRowStore::put(
                        record.node_id,
                        &record.metadata,
                        &*engine.backend,
                    );
                }
            }

            // ponytail: no `replace_derived_indexes` for batch — derived index update
            // for UPSERTS requires per-node `engine.get()` to diff old vs new. For
            // fresh-insert workloads (common case) there is nothing to diff. Add
            // a second pass with existence checks when UPSERT-batch support is needed.

            all_results.extend(records);
        }

        // ── HNSW index: rebuild from scratch when any chunk used Rebuild mode ──
        if rebuild_needed {
            engine.rebuild_vector_index()?;
        }

        // Derived + text indexes: put_batch writes nodes directly (no per-node
        // replace_derived_indexes), so rebuild them in one pass. Without this,
        // list/count/text-search return 0 for batch-inserted records because the
        // empty NamespaceIndex/TextIndex partitions are read as authoritative.
        // ponytail: full rebuild per batch is O(total nodes); switch to
        // incremental per-record index ops if batch-heavy workloads need it.
        self.rebuild_derived_indexes_with_report()?;
        self.rebuild_text_index_with_report()?;
        self.rebuild_sparse_index_with_report()?;

        Ok(all_results)
    }

    /// Retrieve a single memory record by namespace and key.
    /// Returns `None` if the record does not exist or has expired.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantadb::config::VantaConfig;
    /// use vantadb::{BackendKind, VantaEmbedded, VantaMemoryInput};
    ///
    /// let db = VantaEmbedded::open_with_config(VantaConfig {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// })
    /// .expect("open in-memory database");
    ///
    /// db.put(VantaMemoryInput::new("docs", "greeting", "Hello, VantaDB!"))
    ///     .expect("put record");
    ///
    /// let record = db
    ///     .get("docs", "greeting")
    ///     .expect("get record")
    ///     .expect("record should exist");
    /// assert_eq!(record.payload, "Hello, VantaDB!");
    ///
    /// // Unknown keys return `None` instead of an error.
    /// assert!(db.get("docs", "missing").expect("get missing").is_none());
    ///
    /// db.close().expect("close database");
    /// ```
    #[tracing::instrument(skip(self), err)]
    pub fn get(&self, namespace: &str, key: &str) -> Result<Option<VantaMemoryRecord>> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let node_id = memory_node_id(namespace, key);
        let Some(node) = self.engine_handle()?.get(node_id)? else {
            return Ok(None);
        };

        match memory_record_from_node(&node) {
            Some(record) if record.namespace == namespace && record.key == key => Ok(Some(record)),
            Some(_record) => Err(VantaError::NodeIdCollision(memory_node_id(namespace, key))),
            None => Ok(None),
        }
    }

    /// Delete a memory record by namespace and key.
    /// Returns `true` if a record was actually deleted, `false` if it did not exist.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantadb::config::VantaConfig;
    /// use vantadb::{BackendKind, VantaEmbedded, VantaMemoryInput};
    ///
    /// let db = VantaEmbedded::open_with_config(VantaConfig {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// })
    /// .expect("open in-memory database");
    ///
    /// db.put(VantaMemoryInput::new("docs", "greeting", "Hello, VantaDB!"))
    ///     .expect("put record");
    ///
    /// assert!(db.delete("docs", "greeting").expect("delete existing"));
    /// // Deleting again returns `false` because the record is gone.
    /// assert!(!db.delete("docs", "greeting").expect("delete missing"));
    /// assert!(db.get("docs", "greeting").expect("get after delete").is_none());
    ///
    /// db.close().expect("close database");
    /// ```
    #[tracing::instrument(skip(self), err)]
    pub fn delete(&self, namespace: &str, key: &str) -> Result<bool> {
        self.check_read_only()?;
        validate_namespace(namespace)?;
        validate_key(key)?;

        let Some(existing) = self.get(namespace, key)? else {
            return Ok(false);
        };

        let node_id = memory_node_id(namespace, key);
        let engine = self.engine_handle()?;
        let res = engine.delete(node_id, "memory delete");
        if res.is_ok() {
            self.replace_derived_indexes(&engine, Some(&existing), None)?;
        }
        self.audit(crate::audit::AuditEvent::new(
            "delete",
            namespace,
            key,
            if res.is_ok() { "ok" } else { "err" },
            Some("memory delete".to_string()),
        ));
        res?;
        Ok(true)
    }

    /// Insert or update a record with exact fields (used internally by import).
    pub(crate) fn put_record_exact(&self, record: VantaMemoryRecord) -> Result<VantaMemoryRecord> {
        self.check_read_only()?;
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
            Some(node) => match memory_record_from_node(&node) {
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
                if let Some(record) = memory_record_from_node(&node) {
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
    ///
    /// Paginates directly over candidate IDs instead of loading all records into memory,
    /// preventing OOM on namespaces with 100K+ records. Records are returned in
    /// namespace-index order (stable by insertion/ID), not key-sorted.
    #[tracing::instrument(skip(self), err)]
    #[allow(deprecated)] // options.filters is legacy path kept for backward compatibility
    pub fn list(
        &self,
        namespace: &str,
        options: VantaMemoryListOptions,
    ) -> Result<VantaMemoryListPage> {
        validate_namespace(namespace)?;
        validate_metadata(&options.filters)?;

        let engine = self.engine_handle()?;
        let limit = options.limit;
        let cursor = options.cursor.unwrap_or(0);

        // ERR-033: `limit = 0` means "return no records", not "return one".
        // Short-circuit before any scan so a zero-limit request is cheap and
        // never triggers the full-scan fallback below.
        if limit == 0 {
            return Ok(VantaMemoryListPage {
                records: Vec::new(),
                next_cursor: None,
            });
        }

        let (candidate_ids, has_index_entries) = if let Some(ops) = &options.filter_ops {
            if let Some(eq_op) = ops
                .iter()
                .find(|op| op.op == crate::sdk::types::VantaFilterOp::Eq)
            {
                if is_scalar_indexable(&eq_op.value) {
                    self.indexed_ids_by_filter(&engine, namespace, &eq_op.field, &eq_op.value)?
                } else {
                    self.indexed_ids_by_namespace(&engine, namespace)?
                }
            } else {
                self.indexed_ids_by_namespace(&engine, namespace)?
            }
        } else if let Some((field, value)) = options.filters.iter().next() {
            if is_scalar_indexable(value) {
                self.indexed_ids_by_filter(&engine, namespace, field, value)?
            } else {
                self.indexed_ids_by_namespace(&engine, namespace)?
            }
        } else {
            self.indexed_ids_by_namespace(&engine, namespace)?
        };

        // Deduplicate IDs (prefix scan may return duplicates)
        let mut seen = BTreeSet::new();
        let unique_ids: Vec<u128> = candidate_ids
            .into_iter()
            .filter(|id| seen.insert(*id))
            .collect();

        // Fetch only the window of IDs for this page — not all records
        let window_ids: Vec<u128> = unique_ids
            .iter()
            .skip(cursor)
            .take(limit)
            .copied()
            .collect();
        let mut records: Vec<VantaMemoryRecord> = Vec::with_capacity(window_ids.len());
        for node in engine.get_many(&window_ids)? {
            if let Some(record) = memory_record_from_node(&node) {
                let matches = if let Some(ops) = &options.filter_ops {
                    crate::sdk::serialization::matches_advanced_filters(&record, ops)
                } else {
                    matches_memory_filters(&record, &options.filters)
                };
                if record.namespace == namespace && matches {
                    records.push(record);
                }
            }
        }

        // Fallback: full scan when no derived index exists (paginated — skip to cursor)
        if records.is_empty() && !has_index_entries {
            crate::metrics::record_derived_full_scan_fallback();
            let mut skipped = 0usize;
            for node in engine.scan_nodes()? {
                if let Some(record) = memory_record_from_node(&node) {
                    let matches = if let Some(ops) = &options.filter_ops {
                        crate::sdk::serialization::matches_advanced_filters(&record, ops)
                    } else {
                        matches_memory_filters(&record, &options.filters)
                    };
                    if record.namespace == namespace && matches {
                        if skipped < cursor {
                            skipped += 1;
                            continue;
                        }
                        records.push(record);
                        if records.len() >= limit {
                            break;
                        }
                    }
                }
            }
        }

        let end_cursor = cursor.saturating_add(limit);
        // A trailing cursor is only valid when this page was actually FULL after
        // the post-filter/dedup pass. `unique_ids.len()` is the pre-filter candidate
        // count, which can exceed the real remaining rows (dedup, filters, TTL) —
        // basing has-more on it emits a phantom cursor at an empty page and loops a
        // client forever. Invariant: a page with fewer than `limit` records is last.
        let page_full = records.len() == limit;
        let next_cursor = (page_full && end_cursor < unique_ids.len()).then_some(end_cursor);

        Ok(VantaMemoryListPage {
            records,
            next_cursor,
        })
    }

    /// Rebuild the HNSW vector index, derived indexes, and text index from scratch.
    #[tracing::instrument(skip(self), err)]
    pub fn rebuild_index(&self) -> Result<VantaIndexRebuildReport> {
        self.check_read_only()?;
        let report = self.engine_handle()?.rebuild_vector_index()?;
        let derived = self.rebuild_derived_indexes_with_report()?;
        self.rebuild_text_index_with_report()?;
        self.rebuild_sparse_index_with_report()?;
        let mut report: VantaIndexRebuildReport = report.into();
        report.derived_rebuild_ms = derived.duration_ms;
        Ok(report)
    }

    /// Rebuild the HNSW vector index from stored vectors, paginating through
    /// memory records via the SDK's `list()` cursor API to prevent OOM on
    /// datasets with 100K+ records. Processes records in batches capped at
    /// `page_size` (default: 1000, max: 1000).
    ///
    /// This is a safe alternative to unbounded `list()` enumeration: instead of
    /// loading all record IDs into memory at once, it walks pages of records
    /// using cursor-based pagination, then delegates the vector index rebuild
    /// to the low-level engine which streams nodes directly from the vector store.
    #[tracing::instrument(skip(self), err)]
    pub fn reindex_hnsw_from_text(
        &self,
        namespace: &str,
        page_size: Option<usize>,
    ) -> Result<VantaIndexRebuildReport> {
        self.check_read_only()?;
        validate_namespace(namespace)?;

        let batch_size = page_size.unwrap_or(1000).max(1).min(1000);
        let started = Instant::now();

        // Phase 1: Paginate through all records using cursor-based list()
        // to safely enumerate the namespace without OOM.
        let mut total_found = 0u64;
        let mut cursor = None;
        loop {
            let page = self.list(
                namespace,
                VantaMemoryListOptions {
                    #[allow(deprecated)]
                    filters: VantaMemoryMetadata::new(),
                    filter_ops: None,
                    limit: batch_size,
                    cursor,
                },
            )?;
            if page.records.is_empty() {
                break;
            }
            total_found += page.records.len() as u64;
            match page.next_cursor {
                Some(c) => cursor = Some(c),
                None => break,
            }
        }

        // Phase 2: Delegate the actual HNSW rebuild to the engine, which
        // streams nodes directly from the vector store (no OOM risk).
        let rebuild_ms = started.elapsed().as_millis() as u64;
        let engine = self.engine_handle()?;
        let report = engine.rebuild_vector_index()?;

        let mut vanta_report: VantaIndexRebuildReport = report.into();
        vanta_report.derived_rebuild_ms = rebuild_ms;

        // If the enumeration phase found records, ensure the engine agreed
        if total_found > 0 && vanta_report.scanned_nodes == 0 {
            tracing::warn!(
                namespace = namespace,
                total_found = total_found,
                "reindex_hnsw_from_text: list() found records but engine scanned zero nodes"
            );
        }

        tracing::info!(
            namespace = namespace,
            total_found = total_found,
            scanned_nodes = vanta_report.scanned_nodes,
            duration_ms = vanta_report.duration_ms + rebuild_ms,
            "reindex_hnsw_from_text completed"
        );

        Ok(vanta_report)
    }

    /// Compact the vector store file, grouping nodes in BFS order from the HNSW entry point.
    #[tracing::instrument(skip(self), err)]
    pub fn compact_layout(&self) -> Result<u64> {
        self.check_read_only()?;
        self.engine_handle()?.compact_layout_bfs()
    }

    /// Flush WAL and memory-mapped files to disk.
    #[tracing::instrument(skip(self), err)]
    pub fn flush(&self) -> Result<()> {
        self.check_read_only()?;
        self.engine_handle()?.flush()
    }

    /// Compact the WAL: flush, archive the current WAL file, and start a fresh one.
    #[tracing::instrument(skip(self), err)]
    pub fn compact_wal(&self) -> Result<()> {
        self.check_read_only()?;
        self.engine_handle()?.compact_wal()
    }

    /// Scan all memory records and physically delete those whose expiry deadline has passed.
    #[tracing::instrument(skip(self), err)]
    pub fn purge_expired(&self) -> Result<u64> {
        self.check_read_only()?;
        let engine = self.engine_handle()?;
        let now = now_ms();
        let mut to_delete: Vec<VantaMemoryRecord> = Vec::new();

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
                let payload = match node.get_field(FIELD_PAYLOAD) {
                    Some(crate::node::FieldValue::String(p)) => p.clone(),
                    _ => String::new(),
                };
                let created_at_ms = match node.get_field(FIELD_CREATED_AT_MS) {
                    Some(crate::node::FieldValue::Int(ms)) if *ms >= 0 => *ms as u64,
                    _ => 0,
                };
                let updated_at_ms = match node.get_field(FIELD_UPDATED_AT_MS) {
                    Some(crate::node::FieldValue::Int(ms)) if *ms >= 0 => *ms as u64,
                    _ => 0,
                };
                let version = match node.get_field(FIELD_VERSION) {
                    Some(crate::node::FieldValue::Int(v)) if *v >= 0 => *v as u64,
                    _ => 0,
                };
                let mut metadata = VantaFields::new();
                for (fk, fv) in &node.relational {
                    if !fk.starts_with("__vanta_") {
                        metadata.insert(fk.clone(), fv.clone().into());
                    }
                }
                to_delete.push(VantaMemoryRecord {
                    namespace,
                    key,
                    payload,
                    metadata,
                    created_at_ms,
                    updated_at_ms,
                    version,
                    node_id: node.id,
                    // The delete loop only reads node_id/namespace/key/payload/
                    // metadata. Skip materializing the dense vector (full
                    // Vec<f32> clone) and the sparse vector (JSON parse) —
                    // both were dead allocations in this path.
                    vector: None,
                    sparse_vector: None,
                    expires_at_ms: Some(expires),
                });
            }
        }

        let count = to_delete.len() as u64;
        if count == 0 {
            return Ok(0);
        }

        let mut all_ops = Vec::new();
        let mut total_payload_entries = 0u64;
        let mut total_posting = 0u64;
        let mut doc_stats_delta: i64 = 0;
        let mut term_deltas: BTreeMap<(String, String), i64> = BTreeMap::new();
        let mut namespace_deltas: BTreeMap<String, (i64, i64)> = BTreeMap::new();

        for record in &to_delete {
            engine.delete(record.node_id, "purge_expired")?;
            all_ops.extend(Self::derived_delete_ops(record)?);
            total_payload_entries += record.metadata.len() as u64;

            let terms = crate::text_index::record_terms(&record.payload);
            all_ops.extend(crate::text_index::posting_delete_ops(
                &record.namespace,
                &record.key,
                &record.payload,
            ));
            all_ops.push(crate::text_index::doc_stats_delete_op(
                &record.namespace,
                &record.key,
            ));
            doc_stats_delta -= 1;
            total_posting += crate::text_index::posting_count(&record.payload);

            for token in terms.token_counts.keys() {
                *term_deltas
                    .entry((record.namespace.clone(), token.clone()))
                    .or_default() -= 1;
            }
            let ns_delta = namespace_deltas
                .entry(record.namespace.clone())
                .or_insert((0, 0));
            ns_delta.0 -= 1;
            ns_delta.1 -= i64::from(terms.doc_len);
        }

        let mut term_stats_delta: i64 = 0;
        for ((namespace, token), delta) in term_deltas {
            if delta == 0 {
                continue;
            }
            let existing = Self::load_text_term_stats(&engine, &namespace, &token)?
                .map(|stats| stats.df)
                .unwrap_or(0);
            let next = Self::checked_stats_value(existing as i128 + delta as i128, "df")?;
            match (existing == 0, next == 0) {
                (true, false) => term_stats_delta += 1,
                (false, true) => term_stats_delta -= 1,
                _ => {}
            }
            if next == 0 {
                all_ops.push(crate::text_index::term_stats_delete_op(&namespace, &token));
            } else {
                all_ops.push(crate::text_index::term_stats_put_op(
                    &namespace, &token, next,
                )?);
            }
        }

        let mut namespace_stats_delta: i64 = 0;
        for (namespace, (doc_delta, len_delta)) in namespace_deltas {
            if doc_delta == 0 && len_delta == 0 {
                continue;
            }
            let existing = Self::load_text_namespace_stats(&engine, &namespace)?.unwrap_or(
                crate::text_index::TextNamespaceStats {
                    doc_count: 0,
                    total_doc_len: 0,
                },
            );
            let next_doc_count = Self::checked_stats_value(
                existing.doc_count as i128 + doc_delta as i128,
                "doc_count",
            )?;
            let next_total_doc_len = Self::checked_stats_value(
                existing.total_doc_len as i128 + len_delta as i128,
                "total_doc_len",
            )?;
            match (existing.doc_count == 0, next_doc_count == 0) {
                (true, false) => namespace_stats_delta += 1,
                (false, true) => namespace_stats_delta -= 1,
                _ => {}
            }
            if next_doc_count == 0 {
                all_ops.push(crate::text_index::namespace_stats_delete_op(&namespace));
            } else {
                all_ops.push(crate::text_index::namespace_stats_put_op(
                    &namespace,
                    &crate::text_index::TextNamespaceStats {
                        doc_count: next_doc_count,
                        total_doc_len: next_total_doc_len,
                    },
                )?);
            }
        }

        for op in &all_ops {
            match op {
                BackendWriteOp::Put {
                    partition: BackendPartition::TextIndex,
                    key,
                    value,
                } => {
                    if crate::text_index::is_term_stats_key(key) {
                        if let Some((ns, token)) = Self::parse_term_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_term_stats(value) {
                                let mut cache = engine.text_stats_cache.write();
                                cache.insert((ns, token), stats);
                                // ponytail: watermark eviction — drop first half if over limit
                                if cache.len() > crate::config::MAX_TEXT_STATS_CACHE {
                                    let keys: Vec<_> =
                                        cache.keys().take(cache.len() / 2).cloned().collect();
                                    for k in keys {
                                        cache.remove(&k);
                                    }
                                }
                            }
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_namespace_stats(value) {
                                let mut cache = engine.text_ns_cache.write();
                                cache.insert(ns, stats);
                                // ponytail: watermark eviction — drop first half if over limit
                                if cache.len() > crate::config::MAX_TEXT_NS_CACHE {
                                    let keys: Vec<_> =
                                        cache.keys().take(cache.len() / 2).cloned().collect();
                                    for k in keys {
                                        cache.remove(&k);
                                    }
                                }
                            }
                        }
                    }
                }
                BackendWriteOp::Delete {
                    partition: BackendPartition::TextIndex,
                    key,
                } => {
                    if crate::text_index::is_term_stats_key(key) {
                        if let Some((ns, token)) = Self::parse_term_stats_key(key) {
                            let mut cache = engine.text_stats_cache.write();
                            cache.remove(&(ns, token));
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            let mut cache = engine.text_ns_cache.write();
                            cache.remove(&ns);
                        }
                    }
                }
                _ => {}
            }
        }

        engine.write_backend_batch(all_ops)?;

        if let Some(mut state) = Self::load_derived_index_state(&engine)? {
            if state.schema_version == DERIVED_INDEX_SCHEMA_VERSION {
                state.record_count = state.record_count.saturating_sub(count);
                state.namespace_entries = state.namespace_entries.saturating_sub(count);
                state.payload_entries = state.payload_entries.saturating_sub(total_payload_entries);
                Self::write_derived_index_state(&engine, &state)?;
            }
        }

        if let Some(mut state) = Self::load_text_index_state(&engine)? {
            if Self::text_index_state_matches_spec(&state) {
                state.record_count = state.record_count.saturating_sub(count);
                state.posting_entries = state.posting_entries.saturating_sub(total_posting);
                state.doc_stats_entries =
                    Self::apply_u64_delta(state.doc_stats_entries, doc_stats_delta);
                state.term_stats_entries =
                    Self::apply_u64_delta(state.term_stats_entries, term_stats_delta);
                state.namespace_stats_entries =
                    Self::apply_u64_delta(state.namespace_stats_entries, namespace_stats_delta);
                Self::write_text_index_state(&engine, &state)?;
            }
        }

        crate::metrics::record_text_postings_written(total_posting);

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
    ///
    /// Automatically creates a reverse edge on the target node, enabling
    /// bidirectional traversal queries.
    #[tracing::instrument(skip(self), err)]
    pub fn add_edge(
        &self,
        source_id: u128,
        target_id: u128,
        label: &str,
        weight: Option<f32>,
        created_at_ms: Option<u64>,
    ) -> Result<()> {
        self.check_read_only()?;
        crate::metrics::record_graph_op("add_edge");
        let engine = self.engine_handle()?;
        let label_id = engine.intern_label(label);
        let w = weight.unwrap_or(1.0);
        // Both the forward and reverse edge share the same logical creation time.
        let ts = created_at_ms.unwrap_or_else(now_ms);

        let mut source = engine
            .get(source_id)?
            .ok_or(VantaError::NodeNotFound(source_id))?;
        source.edges.push(crate::node::Edge {
            target: target_id,
            label_id,
            weight: w,
            reverse: false,
            created_at_ms: ts,
        });
        engine.insert(&source)?;

        let mut target = engine
            .get(target_id)?
            .ok_or(VantaError::NodeNotFound(target_id))?;
        target.edges.push(crate::node::Edge {
            target: source_id,
            label_id,
            weight: w,
            reverse: true,
            created_at_ms: ts,
        });
        engine.insert(&target)
    }

    /// Remove all edges between two nodes with the given label (both directions).
    #[tracing::instrument(skip(self), err)]
    pub fn remove_edge(&self, source_id: u128, target_id: u128, label: &str) -> Result<()> {
        self.check_read_only()?;
        crate::metrics::record_graph_op("remove_edge");
        let engine = self.engine_handle()?;
        let label_id = engine.intern_label(label);

        let mut source = engine
            .get(source_id)?
            .ok_or(VantaError::NodeNotFound(source_id))?;
        source
            .edges
            .retain(|e| !(e.target == target_id && e.label_id == label_id));
        engine.insert(&source)?;

        let mut target = engine
            .get(target_id)?
            .ok_or(VantaError::NodeNotFound(target_id))?;
        target
            .edges
            .retain(|e| !(e.target == source_id && e.label_id == label_id));
        engine.insert(&target)
    }

    /// Execute an IQL query.
    #[tracing::instrument(skip(self), err)]
    pub fn query(&self, query: &str) -> Result<VantaQueryResult> {
        let engine = self.engine_handle()?;
        let executor = Executor::new(&engine);
        let result = executor.execute_hybrid(query)?;
        Ok(match result {
            crate::executor::ExecutionResult::Read(nodes) => VantaQueryResult::Read(
                nodes
                    .into_iter()
                    .map(|n| engine.node_to_record(n))
                    .collect(),
            ),
            crate::executor::ExecutionResult::Write {
                affected_nodes,
                message,
                node_id,
            } => VantaQueryResult::Write {
                affected_nodes,
                message,
                node_id,
            },
            crate::executor::ExecutionResult::StaleContext(node_id) => {
                VantaQueryResult::StaleContext { node_id }
            }
        })
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
        let hnsw = engine.hnsw.load();
        // ERR-028: mirror the AUDREP-55 up-front guard using the index's own
        // metric (matches `search_nearest` exactly) so the legacy K-NN path
        // also reports InvalidInput instead of a silent empty result.
        if hnsw.config.distance_metric == crate::node::DistanceMetric::Cosine
            && crate::index::f32_l2_norm(vector) < f32::EPSILON
        {
            return Err(crate::error::VantaError::InvalidInput(
                "zero-norm cosine query vector is undefined; use a non-zero vector \
                 or the euclidean distance metric (AUDREP-55, ERR-028)"
                    .into(),
            ));
        }
        let results = {
            // ponytail: search reads from L0 only. Multi-level search
            // will need a segment-merged view.
            let vs = engine.vector_store[0].read();
            hnsw.search_nearest(
                vector,
                None,
                None,
                &crate::node::ALL_BITSET,
                top_k,
                Some(&*vs),
            )
        };
        Ok(results
            .into_iter()
            .map(|(node_id, distance)| VantaSearchHit { node_id, distance })
            .collect())
    }

    // ── REC-002: delete_by_filter ──────────────────────────────────────────

    /// Delete all records in a namespace that match the given metadata filter.
    ///
    /// Iterates through all records in the namespace using cursor-based pagination
    /// and deletes each record that satisfies every filter item. Returns the total
    /// number of records deleted.
    ///
    /// # Behaviour
    /// - Returns an error if `namespace` is empty or invalid.
    /// - Returns an error if `filter` is empty (to avoid accidental full-namespace wipe).
    /// - Operation is **not atomic**: if the process is interrupted mid-execution,
    ///   some records may be deleted while others are not. The WAL does not support
    ///   batch rollback.
    ///
    /// # Performance
    /// V1 uses full paginated scan. For namespaces with millions of records,
    /// execution time scales linearly. Optimise to direct scan in V2 if needed.
    #[tracing::instrument(skip(self, filter), err)]
    pub fn delete_by_filter(&self, namespace: &str, filter: VantaMemoryFilter) -> Result<u64> {
        self.check_read_only()?;
        validate_namespace(namespace)?;
        if filter.is_empty() {
            return Err(crate::error::VantaError::InvalidInput(
                "delete_by_filter requires at least one filter item to prevent accidental \
                 full-namespace deletion. Use delete() to remove individual records."
                    .into(),
            ));
        }

        const PAGE_SIZE: usize = 500;
        let mut cursor: Option<usize> = None;
        let mut deleted: u64 = 0;
        let mut keys_to_delete: Vec<String> = Vec::new();

        // Phase 1: collect all matching keys via paginated scan.
        loop {
            let page = self.list(
                namespace,
                VantaMemoryListOptions {
                    #[allow(deprecated)]
                    filters: VantaMemoryMetadata::new(),
                    filter_ops: Some(filter.clone()),
                    limit: PAGE_SIZE,
                    cursor,
                },
            )?;
            for record in &page.records {
                keys_to_delete.push(record.key.clone());
            }
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }

        // Phase 2: delete collected keys.
        for key in &keys_to_delete {
            if self.delete(namespace, key)? {
                deleted += 1;
                if deleted % 1000 == 0 {
                    tracing::info!(namespace, deleted, "delete_by_filter: progress checkpoint");
                }
            }
        }
        self.audit(crate::audit::AuditEvent::new(
            "delete_by_filter",
            namespace,
            "N/A",
            "ok",
            Some(format!("{deleted} records")),
        ));
        Ok(deleted)
    }

    // ── REC-003: count ────────────────────────────────────────────────────

    /// Count records in a namespace, optionally filtered by metadata.
    ///
    /// Without a filter, this is an O(n) scan over namespace index entries.
    /// With a filter, records are evaluated in-memory after retrieval.
    ///
    /// # Arguments
    /// * `namespace` — Namespace to count records in.
    /// * `filter` — Optional list of filter items (AND-combined). Pass `None`
    ///   to count all records in the namespace.
    #[tracing::instrument(skip(self, filter), err)]
    pub fn count(&self, namespace: &str, filter: Option<VantaMemoryFilter>) -> Result<u64> {
        validate_namespace(namespace)?;

        const PAGE_SIZE: usize = 1000;
        let mut cursor: Option<usize> = None;
        let mut total: u64 = 0;

        loop {
            let page = self.list(
                namespace,
                VantaMemoryListOptions {
                    #[allow(deprecated)]
                    filters: VantaMemoryMetadata::new(),
                    filter_ops: filter.clone(),
                    limit: PAGE_SIZE,
                    cursor,
                },
            )?;
            total += page.records.len() as u64;
            cursor = page.next_cursor;
            if cursor.is_none() {
                break;
            }
        }
        Ok(total)
    }

    // ── REC-004: similar_to_key ───────────────────────────────────────────

    /// Find records similar to an existing record identified by `key`.
    ///
    /// Retrieves the vector stored under `key` in `namespace` and performs a
    /// K-NN vector similarity search against the HNSW index. Results are
    /// post-filtered to the same namespace and the source record itself is
    /// excluded from the output.
    ///
    /// # Errors
    /// * [`VantaError::NotFound`] if `key` does not exist in `namespace`.
    /// * [`VantaError::NoVectorForKey`] if the record exists but carries no vector.
    ///
    /// # Notes
    /// `search_vector()` queries the global HNSW index (all namespaces). Results
    /// are post-filtered to `namespace` to preserve namespace isolation semantics.
    #[tracing::instrument(skip(self), err)]
    pub fn similar_to_key(
        &self,
        namespace: &str,
        key: &str,
        top_k: usize,
    ) -> Result<Vec<VantaMemorySearchHit>> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let record =
            self.get(namespace, key)?
                .ok_or_else(|| crate::error::VantaError::NotFound {
                    kind: "memory record".into(),
                    id: format!("{namespace}/{key}"),
                })?;

        let vector = record.vector.ok_or_else(|| {
            crate::error::VantaError::NoVectorForKey(format!("{namespace}/{key}"))
        })?;

        // Search top_k + 1 to account for the source record itself being in results.
        let raw_hits = self.search_vector(&vector, top_k + 1)?;

        let engine = self.engine_handle()?;
        let raw_ids: Vec<u128> = raw_hits.iter().map(|h| h.node_id).collect();
        let nodes = engine.get_many(&raw_ids)?;

        let hits: Vec<VantaMemorySearchHit> = raw_hits
            .into_iter()
            .zip(nodes)
            .filter_map(|(hit, node)| {
                crate::sdk::serialization::memory_record_from_node(&node).and_then(|r| {
                    if r.namespace == namespace && r.key != key {
                        Some(VantaMemorySearchHit {
                            record: r,
                            score: 1.0 - hit.distance,
                            explanation: None,
                        })
                    } else {
                        None
                    }
                })
            })
            .take(top_k)
            .collect();

        Ok(hits)
    }
}

/// Report returned by bulk import operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportReport {
    pub total_records: usize,
    pub batches_committed: usize,
    pub duration_ms: u64,
}

impl VantaEmbedded {
    /// Bulk-import records from a binary stream.
    ///
    /// Format: 8-byte magic `VDBJSON\n`, 1-byte version `0x01`,
    /// 8-byte LE record count, then serde_json-serialized `Vec<VantaMemoryInput>`.
    ///
    /// Bypasses per-record validation (`validate_namespace`, `validate_key`,
    /// `validate_metadata`) for raw throughput. Commits to the engine in batches
    /// sized by [`VantaConfig::bulk_commit_interval`] (default: 10 000).
    pub fn bulk_import_stream<R: std::io::Read>(&self, reader: &mut R) -> Result<BulkImportReport> {
        self.check_read_only()?;
        let start = web_time::Instant::now();

        // ── Header ──
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        if &magic != b"VDBJSON\n" {
            return Err(VantaError::ValidationError {
                field: "header".into(),
                reason: format!(
                    "invalid magic bytes: expected VDBJSON\\n, got {:?}",
                    std::str::from_utf8(&magic).unwrap_or("??")
                ),
            });
        }

        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;
        if version[0] != 0x01 {
            return Err(VantaError::ValidationError {
                field: "version".into(),
                reason: format!("unsupported format version: {}", version[0]),
            });
        }

        let mut raw_count = [0u8; 8];
        reader.read_exact(&mut raw_count)?;
        let total = u64::from_le_bytes(raw_count) as usize;

        // ── Body: serde_json-serialized Vec<VantaMemoryInput> ──
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let records: Vec<VantaMemoryInput> =
            serde_json::from_slice(&buf).map_err(|e| VantaError::ValidationError {
                field: "body".into(),
                reason: format!("JSON deserialization failed: {}", e),
            })?;

        if records.len() != total {
            return Err(VantaError::ValidationError {
                field: "count".into(),
                reason: format!("declared {} records but got {}", total, records.len()),
            });
        }

        let engine = self.engine_handle()?;
        let commit_interval = self.config.bulk_commit_interval.unwrap_or(10_000);
        let mut batches = 0usize;

        for chunk in records.chunks(commit_interval) {
            for input in chunk {
                let node_id = memory_node_id(&input.namespace, &input.key);
                let mut node = UnifiedNode::new(node_id);
                node.set_field("payload", FieldValue::String(input.payload.clone()));
                if let Some(ref v) = input.vector {
                    node.vector = VectorRepresentations::Full(v.clone());
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }
                for (k, v) in &input.metadata {
                    let fv = match v {
                        VantaValue::String(s) => FieldValue::String(s.clone()),
                        VantaValue::Int(i) => FieldValue::Int(*i),
                        VantaValue::Float(f) => FieldValue::Float(*f),
                        VantaValue::Bool(b) => FieldValue::Bool(*b),
                        // DateTime and list variants are not supported in bulk import.
                        _ => continue,
                    };
                    node.set_field(k, fv);
                }
                if let Some(ttl) = input.ttl_ms {
                    let expires_at_ms = now_ms().saturating_add(ttl);
                    node.set_field(FIELD_EXPIRES_AT_MS, FieldValue::Int(expires_at_ms as i64));
                }
                engine.insert(&node)?;
            }
            batches += 1;
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        Ok(BulkImportReport {
            total_records: total,
            batches_committed: batches,
            duration_ms,
        })
    }

    /// Convenience: bulk-import from a binary file in bulk format.
    pub fn bulk_import_file(&self, path: &str) -> Result<BulkImportReport> {
        let mut file = std::fs::File::open(path)?;
        self.bulk_import_stream(&mut file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::VantaConfig;

    fn make_embedded(read_only: bool) -> VantaEmbedded {
        let config = VantaConfig {
            storage_path: ":memory:".into(),
            read_only,
            ..Default::default()
        };
        VantaEmbedded::test_empty(config)
    }

    // ── capabilities ──

    #[test]
    fn test_capabilities_default() {
        let e = make_embedded(false);
        let caps = e.capabilities();
        assert_eq!(caps.runtime_profile, VantaRuntimeProfile::Performance);
        assert!(caps.persistence);
        assert!(caps.vector_search);
        assert!(caps.iql_queries);
        assert!(!caps.read_only);
    }

    #[test]
    fn test_capabilities_read_only() {
        let e = make_embedded(true);
        let caps = e.capabilities();
        assert!(caps.read_only);
    }

    #[test]
    fn test_capabilities_clone() {
        let e = make_embedded(false);
        let caps = e.capabilities();
        let cloned = caps.clone();
        assert_eq!(caps, cloned);
    }

    // ── check_read_only ──

    #[test]
    fn test_check_read_only_passes() {
        let e = make_embedded(false);
        assert!(e.check_read_only().is_ok());
    }

    #[test]
    fn test_check_read_only_errors() {
        let e = make_embedded(true);
        let err = e.check_read_only().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("read-only") || msg.contains("read_only"),
            "got: {msg}"
        );
    }

    #[test]
    fn test_put_blocked_when_read_only() {
        let e = VantaEmbedded::open_with_config(VantaConfig {
            storage_path: ":memory:".into(),
            backend_kind: crate::storage::BackendKind::InMemory,
            read_only: true,
            ..Default::default()
        })
        .expect("open read-only in-memory database");

        let err = e
            .put(VantaMemoryInput::new("docs", "greeting", "Hello, VantaDB!"))
            .unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("read-only") || msg.contains("read_only"),
            "put should be blocked on read-only db, got: {msg}"
        );

        // Nothing was written: a read still sees nothing.
        assert!(e.get("docs", "greeting").expect("get").is_none());
    }

    // ── search_vector (early returns) ──

    #[test]
    fn test_search_vector_empty_input() {
        let e = make_embedded(false);
        let result = e.search_vector(&[], 10).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_search_vector_zero_topk() {
        let e = make_embedded(false);
        let result = e.search_vector(&[0.1, 0.2], 0).unwrap();
        assert!(result.is_empty());
    }

    // ── engine-dependent methods return NotInitialized ──

    #[test]
    fn test_insert_node_no_engine() {
        let e = make_embedded(false);
        let input = VantaNodeInput {
            id: 1,
            content: Some("test".into()),
            vector: None,
            fields: VantaFields::new(),
        };
        let err = e.insert_node(input).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_get_node_no_engine() {
        let e = make_embedded(false);
        let err = e.get_node(42).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_delete_node_no_engine() {
        let e = make_embedded(false);
        let err = e.delete_node(42, "test").unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_put_no_engine() {
        let e = make_embedded(false);
        let input = VantaMemoryInput::new("ns", "k", "payload");
        let err = e.put(input).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_put_batch_no_engine() {
        let e = make_embedded(false);
        let inputs = vec![VantaMemoryInput::new("ns", "k", "payload")];
        let err = e.put_batch(inputs).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_get_no_engine() {
        let e = make_embedded(false);
        let err = e.get("ns", "k").unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_delete_memory_no_engine() {
        let e = make_embedded(false);
        let err = e.delete("ns", "k").unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_list_namespaces_no_engine() {
        let e = make_embedded(false);
        let err = e.list_namespaces().unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_list_no_engine() {
        let e = make_embedded(false);
        let opts = VantaMemoryListOptions::default();
        let err = e.list("ns", opts).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_rebuild_index_no_engine() {
        let e = make_embedded(false);
        let err = e.rebuild_index().unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_reindex_hnsw_from_text_no_engine() {
        let e = make_embedded(false);
        let err = e.reindex_hnsw_from_text("ns", Some(100)).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_compact_layout_no_engine() {
        let e = make_embedded(false);
        let err = e.compact_layout().unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_flush_no_engine() {
        let e = make_embedded(false);
        let err = e.flush().unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_compact_wal_no_engine() {
        let e = make_embedded(false);
        let err = e.compact_wal().unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_purge_expired_no_engine() {
        let e = make_embedded(false);
        let err = e.purge_expired().unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_add_edge_no_engine() {
        let e = make_embedded(false);
        let err = e.add_edge(1, 2, "label", None, None).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    // ── add_edge temporal (COMP-021) ──

    fn make_embedded_real() -> VantaEmbedded {
        let config = VantaConfig {
            storage_path: ":memory:".into(),
            backend_kind: crate::storage::BackendKind::InMemory,
            ..Default::default()
        };
        VantaEmbedded::open_with_config(config).expect("open in-memory engine")
    }

    fn insert_node_input(id: u128) -> VantaNodeInput {
        VantaNodeInput {
            id,
            content: Some("n".into()),
            vector: None,
            fields: VantaFields::new(),
        }
    }

    #[test]
    fn test_add_edge_with_timestamp_persists_both_nodes() {
        let e = make_embedded_real();
        e.insert_node(insert_node_input(1)).unwrap();
        e.insert_node(insert_node_input(2)).unwrap();

        let ts = 1_700_000_000_000;
        e.add_edge(1, 2, "references", Some(0.5), Some(ts)).unwrap();

        let engine = e.engine_handle().unwrap();
        let n1 = engine.get(1).unwrap().unwrap();
        let n2 = engine.get(2).unwrap().unwrap();

        let fwd = n1
            .edges
            .iter()
            .find(|x| x.target == 2)
            .expect("forward edge");
        assert!(!fwd.reverse);
        assert_eq!(fwd.created_at_ms, ts, "forward edge must keep explicit ts");

        let rev = n2
            .edges
            .iter()
            .find(|x| x.target == 1)
            .expect("reverse edge");
        assert!(rev.reverse);
        assert_eq!(rev.created_at_ms, ts, "reverse edge must share the ts");
    }

    #[test]
    fn test_add_edge_default_timestamp_now() {
        let e = make_embedded_real();
        e.insert_node(insert_node_input(1)).unwrap();
        e.insert_node(insert_node_input(2)).unwrap();

        e.add_edge(1, 2, "references", None, None).unwrap();

        let engine = e.engine_handle().unwrap();
        let n1 = engine.get(1).unwrap().unwrap();
        let fwd = n1
            .edges
            .iter()
            .find(|x| x.target == 2)
            .expect("forward edge");
        assert!(
            fwd.created_at_ms > 0,
            "default created_at_ms must be now, got {}",
            fwd.created_at_ms
        );
    }

    #[test]
    fn test_query_no_engine() {
        let e = make_embedded(false);
        let err = e.query("GET *").unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_operational_metrics_default() {
        let e = make_embedded(false);
        let m = e.operational_metrics();
        // No engine handle available, metrics are a snapshot with defaults
        assert_eq!(m.hnsw_nodes_count, 0);
    }

    // ── VantaNodeInput helpers (via serialization re-export) ──

    #[test]
    fn test_node_input_default_fields() {
        let input = VantaNodeInput {
            id: 1,
            content: None,
            vector: Some(vec![1.0, 2.0]),
            fields: VantaFields::new(),
        };
        assert_eq!(input.id, 1);
        assert!(input.content.is_none());
        assert_eq!(input.vector.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_node_input_with_content() {
        let input = VantaNodeInput {
            id: 99,
            content: Some("hello".into()),
            vector: None,
            fields: [("lang".into(), VantaValue::String("en".into()))].into(),
        };
        assert_eq!(input.content.as_deref(), Some("hello"));
        assert_eq!(
            input.fields.get("lang").unwrap(),
            &VantaValue::String("en".into())
        );
    }

    // ── list() pagination cursor (AUDREP-30) ──

    #[test]
    fn test_list_no_trailing_cursor_when_post_filter_exhausts_page() {
        let e = make_embedded_real();
        // 10 records are all present in the derived index for `lang == "en"`,
        // but the combined advanced filter additionally requires `rank`, a field
        // no record carries. matches_advanced_filters() returns false for a
        // missing field, so post-filter yields zero records while the pre-filter
        // candidate count (unique_ids.len() == 10) suggests more pages exist.
        for i in 0..10u32 {
            e.put(VantaMemoryInput {
                namespace: "ns".into(),
                key: format!("k{i}"),
                payload: format!("payload{i}"),
                metadata: [("lang".into(), VantaValue::String("en".into()))].into(),
                vector: None,
                sparse_vector: None,
                ttl_ms: None,
            })
            .unwrap();
        }

        let filter_ops: VantaMemoryFilter = vec![
            // Candidate lookup: feeds the derived payload index (first Eq field).
            VantaMemoryFilterItem {
                field: "lang".into(),
                op: VantaFilterOp::Eq,
                value: VantaValue::String("en".into()),
            },
            // Excludes every record: `rank` is absent on all of them.
            VantaMemoryFilterItem {
                field: "rank".into(),
                op: VantaFilterOp::Neq,
                value: VantaValue::String("top".into()),
            },
        ];
        let opts = VantaMemoryListOptions {
            #[allow(deprecated)]
            filters: VantaMemoryMetadata::new(),
            filter_ops: Some(filter_ops),
            limit: 4,
            cursor: None,
        };

        let page = e.list("ns", opts).unwrap();
        assert!(
            page.records.is_empty(),
            "post-filter should yield zero records, got {}",
            page.records.len()
        );
        assert!(
            page.next_cursor.is_none(),
            "empty page emitted trailing cursor {:?} — pre-filter count ({}) \
             overestimates remaining rows, which loops a client forever",
            page.next_cursor,
            10,
        );
    }

    /// ERR-033: `limit = 0` must return zero records — the core previously
    /// clamped to `max(1)`, so a zero-limit list returned one record instead.
    #[test]
    fn test_list_zero_limit_returns_no_records() {
        let e = make_embedded_real();
        for i in 0..3u32 {
            e.put(VantaMemoryInput {
                namespace: "ns".into(),
                key: format!("k{i}"),
                payload: format!("payload{i}"),
                metadata: VantaMemoryMetadata::new(),
                vector: None,
                sparse_vector: None,
                ttl_ms: None,
            })
            .unwrap();
        }

        let opts = VantaMemoryListOptions {
            #[allow(deprecated)]
            filters: VantaMemoryMetadata::new(),
            filter_ops: None,
            limit: 0,
            cursor: None,
        };
        let page = e.list("ns", opts).unwrap();
        assert!(
            page.records.is_empty(),
            "limit=0 must return no records, got {}",
            page.records.len()
        );
        assert!(
            page.next_cursor.is_none(),
            "zero-limit page has no next cursor"
        );
    }

    /// ERR-026: a list-valued metadata filter must narrow by equality, not
    /// fail. The derived payload index only stores flattened scalar entries,
    /// so `list()` must fall back to a namespace scan for non-scalar filter
    /// values and let `matches_memory_filters` do the strict equality check.
    #[test]
    fn test_list_filters_by_list_metadata() {
        let e = make_embedded_real();
        e.put(VantaMemoryInput {
            namespace: "ns".into(),
            key: "k1".into(),
            payload: "p1".into(),
            metadata: [(
                "tags".into(),
                VantaValue::ListString(vec!["a".into(), "b".into()]),
            )]
            .into(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })
        .unwrap();
        e.put(VantaMemoryInput {
            namespace: "ns".into(),
            key: "k2".into(),
            payload: "p2".into(),
            metadata: [("tags".into(), VantaValue::ListString(vec!["x".into()]))].into(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })
        .unwrap();
        e.put(VantaMemoryInput {
            namespace: "ns".into(),
            key: "k3".into(),
            payload: "p3".into(),
            metadata: VantaMemoryMetadata::new(),
            vector: None,
            sparse_vector: None,
            ttl_ms: None,
        })
        .unwrap();

        let opts = VantaMemoryListOptions {
            #[allow(deprecated)]
            filters: [(
                "tags".into(),
                VantaValue::ListString(vec!["a".into(), "b".into()]),
            )]
            .into(),
            filter_ops: None,
            limit: 10,
            cursor: None,
        };
        let page = e.list("ns", opts).unwrap();
        let keys: Vec<_> = page.records.iter().map(|r| r.key.as_str()).collect();
        assert_eq!(
            keys,
            vec!["k1"],
            "list filter must return exactly the record with matching list metadata, got {keys:?}"
        );
    }

    // ── bulk_import ──

    #[test]
    fn test_bulk_import_stream_invalid_magic() {
        let e = make_embedded(false);
        // 8 bytes that don't match "VDBJSON\n"
        let mut bad_data: &[u8] = b"BADMAGIC";
        let err = e.bulk_import_stream(&mut bad_data).unwrap_err();
        assert!(err.to_string().contains("magic"), "got: {:?}", err);
    }

    #[test]
    fn test_bulk_import_stream_empty_no_engine() {
        // Valid header with count=0 but engine not initialized
        let mut buf = Vec::new();
        buf.extend_from_slice(b"VDBJSON\n");
        buf.push(0x01);
        buf.extend_from_slice(&0u64.to_le_bytes());
        // Body: empty JSON array
        buf.extend_from_slice(b"[]");

        let e = make_embedded(false);
        let mut cursor = std::io::Cursor::new(&buf);
        let err = e.bulk_import_stream(&mut cursor).unwrap_err();
        assert!(err.to_string().contains("initialized"), "got: {:?}", err);
    }

    #[test]
    fn test_bulk_import_stream_count_mismatch() {
        // Write header claiming 5 records but body has 0 (empty JSON)
        let mut buf = Vec::new();
        buf.extend_from_slice(b"VDBJSON\n");
        buf.push(0x01);
        buf.extend_from_slice(&5u64.to_le_bytes()); // count = 5
                                                    // Empty JSON array
        buf.extend_from_slice(b"[]");

        let e = make_embedded(false);
        let mut cursor = std::io::Cursor::new(&buf);
        let err = e.bulk_import_stream(&mut cursor).unwrap_err();
        assert!(err.to_string().contains("count"), "got: {:?}", err);
    }
}
