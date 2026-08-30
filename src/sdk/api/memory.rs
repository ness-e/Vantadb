//! Memory-record operations on `VantaEmbedded`.
//!
//! Owns the per-record CRUD surface (put / get / delete / supersede), bulk
//! import, version-history access, and TTL purge. Implementation was extracted
//! from `sdk::api` (REVIEW-12, 2026-08-30) so the SDK surface can evolve per
//! domain without 2k+-line god files.
//!
//! Ponytail note: helpers that crossed module boundaries (`usable_vector`,
//! `check_read_only`, `put_one`, `put_batch_inner`) were relocated here with
//! `pub(super)` visibility so other domain modules can reuse them when needed.

use super::super::builder::VantaEmbedded;
use super::super::serialization::{
    memory_node_id, memory_record_from_node, memory_record_to_node_owned, now_ms, validate_key,
    validate_metadata, validate_namespace, DERIVED_INDEX_SCHEMA_VERSION, FIELD_CREATED_AT_MS,
    FIELD_EXPIRES_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD, FIELD_UPDATED_AT_MS,
    FIELD_VERSION,
};
use super::super::types::*;
use crate::backend::{BackendKind, BackendPartition, BackendWriteOp};
use crate::error::{Result, VantaError};
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use web_time::Instant;

/// Report returned by bulk import operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportReport {
    pub total_records: usize,
    pub batches_committed: usize,
    pub duration_ms: u64,
}

impl VantaEmbedded {
    /// True when a vector is entirely zeros — the HNSW core rejects
    /// zero-norm vectors under cosine similarity (AUDREP-27), so the
    /// SDK treats them like empty vectors: keep the document, skip the
    /// vector index (matches `load.test.ts` seeding `[i % 10, 0, 0, 0]`).
    pub(super) fn usable_vector(vector: &[f32]) -> bool {
        !vector.is_empty() && vector.iter().any(|x| *x != 0.0)
    }

    pub(super) fn check_read_only(&self) -> Result<()> {
        if self.config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "this operation is not available when VantaDB is opened read-only".into(),
            });
        }
        Ok(())
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
            superseded_by: None,
            superseded_at_ms: None,
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

        // Best-effort version-history snapshot (VS-CORE-07): 1 write extra,
        // post-commit, same durability class as ShreddedRowStore.
        let _ = super::super::version_history::write_snapshot(
            &engine,
            &record,
            self.config.version_history_limit,
        );

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
                    superseded_by: None,
                    superseded_at_ms: None,
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

            // ── Version history: 1 write_batch per chunk (best-effort, post-commit) ──
            // Records already carry the final version (seen_versions bump), so
            // the snapshots mirror the exact bump sequence per key.
            let _ = super::super::version_history::write_snapshot_batch(
                &engine,
                &records,
                self.config.version_history_limit,
            );

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

    /// Retrieve the record as it was at the given version (VS-CORE-07).
    ///
    /// Returns `None` if that version was never persisted (unknown key or a
    /// version already purged by the retention cap or a delete). Snapshot
    /// durability is best-effort post-commit, so a crash window can leave a
    /// version gap — degraded but never corrupt.
    #[tracing::instrument(skip(self), err)]
    pub fn get_version(
        &self,
        namespace: &str,
        key: &str,
        version: u64,
    ) -> Result<Option<VantaMemoryRecord>> {
        validate_namespace(namespace)?;
        validate_key(key)?;
        let engine = self.engine_handle()?;
        super::super::version_history::get_version(&engine, namespace, key, version)
    }

    /// List every retained version of a record, ascending (v1..vN) (VS-CORE-07).
    ///
    /// Empty if the key does not exist or has no history. Expired versions are
    /// included as historical data until purged. `get_version(vN)` of the last
    /// element matches the live record.
    #[tracing::instrument(skip(self), err)]
    pub fn versions(&self, namespace: &str, key: &str) -> Result<Vec<VantaMemoryRecord>> {
        validate_namespace(namespace)?;
        validate_key(key)?;
        let engine = self.engine_handle()?;
        super::super::version_history::versions(&engine, namespace, key)
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
            // Purge version history (VS-CORE-07) — best-effort class.
            let _ = super::super::version_history::purge_key(&engine, namespace, key);
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

    /// Mark an existing record as superseded by another existing record (ADR-028).
    ///
    /// Supersession is durable and first-class: the old record keeps its data
    /// (soft-dead, recoverable) but gains `superseded_by`/`superseded_at_ms`,
    /// and can be hidden from search/list with `exclude_superseded`.
    ///
    /// Errors if either key is missing, if `old_key == new_key`, or if the old
    /// record is already superseded (idempotency guard).
    #[tracing::instrument(skip(self), err)]
    pub fn supersede(&self, namespace: &str, old_key: &str, new_key: &str) -> Result<()> {
        self.check_read_only()?;
        validate_namespace(namespace)?;
        validate_key(old_key)?;
        validate_key(new_key)?;
        if old_key == new_key {
            return Err(VantaError::InvalidInput(
                "supersede: old_key and new_key must be different".into(),
            ));
        }

        // REVIEW-13: serialize the read-modify-write below. Without this, two
        // concurrent supersede calls can both read `old.superseded_by == None`
        // and both pass the idempotency guard, double-marking the record (the
        // engine's insert_lock only serializes the individual insert, not the
        // SDK-level read + check). The guard spans every stateful step:
        // get(old) → idempotency check → get(new) → mutate → engine.insert.
        let _guard = self.supersede_lock.lock();

        let old = self
            .get(namespace, old_key)?
            .ok_or_else(|| VantaError::NotFound {
                kind: "memory record".into(),
                id: format!("{namespace}/{old_key}"),
            })?;
        if old.superseded_by.is_some() {
            return Err(VantaError::InvalidInput(format!(
                "record '{old_key}' is already superseded by '{}'",
                old.superseded_by.as_deref().unwrap_or_default()
            )));
        }
        if self.get(namespace, new_key)?.is_none() {
            return Err(VantaError::NotFound {
                kind: "memory record".into(),
                id: format!("{namespace}/{new_key}"),
            });
        }

        let now = now_ms();
        let mut record = old;
        record.superseded_by = Some(new_key.to_string());
        record.superseded_at_ms = Some(now);
        record.updated_at_ms = now;
        record.version = record.version.saturating_add(1);

        // Reuse the put/upsert serialization path (WAL + KV + HNSW via
        // engine.insert); payload/metadata/vector are unchanged, so the
        // derived text/scalar indexes stay consistent with no index writes.
        // ponytail: two WAL appends (old marked, new untouched) — not atomic;
        // a crash between them leaves a dangling marker, which is still
        // self-consistent (old marked, new present). Full 2PC deferred to
        // ACID Phase 0, same as insert.
        let engine = self.engine_handle()?;
        let (node, record) = memory_record_to_node_owned(record);
        engine.insert(&node)?;
        // Best-effort version-history snapshot, same durability class as put_one.
        let _ = super::super::version_history::write_snapshot(
            &engine,
            &record,
            self.config.version_history_limit,
        );
        Ok(())
    }

    /// Scan all memory records and physically delete those whose expiry deadline has passed.
    #[tracing::instrument(skip(self), err)]
    pub fn purge_expired(&self) -> Result<u64> {
        self.check_read_only()?;
        let engine = self.engine_handle()?;
        let now = now_ms();
        let mut to_delete: Vec<VantaMemoryRecord> = Vec::new();

        // MOD-04: select expired candidates via the scalar index
        // (`expires_at_ms <= now`) instead of a full O(N) engine scan that
        // reads and clones every node's vector. The scalar index is maintained
        // on the write path and rebuilt at open / rebuild_index. Candidates
        // are materialized from backend metadata only (no vector, no cache) —
        // purge needs nothing beyond the relational fields.
        let candidates = engine.scalar_lookup_int_le(FIELD_EXPIRES_AT_MS, now as i64);

        for node_id in candidates {
            let Some(bytes) =
                engine.get_from_partition(BackendPartition::Default, &node_id.to_le_bytes())?
            else {
                // Deleted while we were scanning — skip.
                continue;
            };
            let Ok(metadata) = crate::storage::ops::deserialize_node_payload::<
                crate::storage::ops::NodeMetadata,
            >(&bytes, "node metadata") else {
                continue;
            };
            let fields = &metadata.relational;
            let get = |key: &str| fields.get(key);
            let namespace = match get(FIELD_NAMESPACE) {
                Some(FieldValue::String(ns)) => ns.clone(),
                _ => continue,
            };
            let key = match get(FIELD_KEY) {
                Some(FieldValue::String(k)) => k.clone(),
                _ => continue,
            };
            let expires = match get(FIELD_EXPIRES_AT_MS) {
                Some(FieldValue::Int(ms)) if *ms > 0 => *ms as u64,
                _ => continue,
            };
            if now > expires {
                let payload = match get(FIELD_PAYLOAD) {
                    Some(FieldValue::String(p)) => p.clone(),
                    _ => String::new(),
                };
                let created_at_ms = match get(FIELD_CREATED_AT_MS) {
                    Some(FieldValue::Int(ms)) if *ms >= 0 => *ms as u64,
                    _ => 0,
                };
                let updated_at_ms = match get(FIELD_UPDATED_AT_MS) {
                    Some(FieldValue::Int(ms)) if *ms >= 0 => *ms as u64,
                    _ => 0,
                };
                let version = match get(FIELD_VERSION) {
                    Some(FieldValue::Int(v)) if *v >= 0 => *v as u64,
                    _ => 0,
                };
                let mut metadata_fields = VantaFields::new();
                for (fk, fv) in fields {
                    if !fk.starts_with("__vanta_") {
                        metadata_fields.insert(fk.clone(), fv.clone().into());
                    }
                }
                to_delete.push(VantaMemoryRecord {
                    namespace,
                    key,
                    payload,
                    metadata: metadata_fields,
                    created_at_ms,
                    updated_at_ms,
                    version,
                    node_id,
                    // The delete loop only reads node_id/namespace/key/payload/
                    // metadata. Skip materializing the dense vector (full
                    // Vec<f32> clone) and the sparse vector (JSON parse) —
                    // both were dead allocations in this path.
                    vector: None,
                    sparse_vector: None,
                    expires_at_ms: Some(expires),
                    superseded_by: None,
                    superseded_at_ms: None,
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
            // Purge version history of the expired key (VS-CORE-07) — best-effort.
            let _ = super::super::version_history::purge_key(&engine, &record.namespace, &record.key);
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

    /// Bulk-import records from a binary stream.
    ///
    /// Format: 8-byte magic `VDBJSON\n`, 1-byte version `0x01`,
    /// 8-byte LE record count, then serde_json-serialized `Vec<VantaMemoryInput>`.
    ///
    /// Bypasses per-record validation (`validate_namespace`, `validate_key`,
    /// `validate_metadata`) for raw throughput. Commits to the engine in batches
    /// sized by [`VantaConfig::bulk_commit_interval`] (default: 10 000).
    pub fn bulk_import_stream<R: std::io::Read>(&self, reader: &mut R) -> Result<BulkImportReport> {
        self.check_read_only()?;
        let start = Instant::now();

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
        let imported_at_ms = now_ms();

        for chunk in records.chunks(commit_interval) {
            for input in chunk {
                let node_id = memory_node_id(&input.namespace, &input.key);
                let mut node = UnifiedNode::new(node_id);
                // Reserved fields (MCP-28): mirror `memory_record_to_node_owned`
                // so bulk-imported records are addressable via get/list/delete.
                // Without these, `memory_record_from_node` returns None and the
                // record is invisible to the memory API.
                node.set_field(FIELD_NAMESPACE, FieldValue::String(input.namespace.clone()));
                node.set_field(FIELD_KEY, FieldValue::String(input.key.clone()));
                node.set_field(FIELD_PAYLOAD, FieldValue::String(input.payload.clone()));
                node.set_field(FIELD_CREATED_AT_MS, FieldValue::Int(imported_at_ms as i64));
                node.set_field(FIELD_UPDATED_AT_MS, FieldValue::Int(imported_at_ms as i64));
                node.set_field(FIELD_VERSION, FieldValue::Int(1));
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