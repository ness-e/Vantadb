//! Memory-record operations on `Embedded`.
//!
//! Owns the per-record CRUD surface (put / get / delete / supersede), bulk
//! import, version-history access, and TTL purge. Implementation was extracted
//! from `sdk::api` (REVIEW-12, 2026-08-30) so the SDK surface can evolve per
//! domain without 2k+-line god files.
//!
//! Ponytail note: helpers that crossed module boundaries (`usable_vector`,
//! `check_read_only`, `put_one`, `put_batch_inner`) were relocated here with
//! `pub(super)` visibility so other domain modules can reuse them when needed.

use super::super::builder::Embedded;
use super::super::serialization::{
    memory_node_id, memory_record_from_node_include_expired, memory_record_to_node_owned, now_ms,
    record_from_node, validate_confidence_fields, validate_key, validate_metadata,
    validate_namespace, DERIVED_INDEX_SCHEMA_VERSION, FIELD_CONFIDENCE_CLASS, FIELD_CREATED_AT_MS,
    FIELD_EXPIRES_AT_MS, FIELD_KEY, FIELD_NAMESPACE, FIELD_PAYLOAD, FIELD_QUARANTINED_AT_MS,
    FIELD_QUARANTINED_BY, FIELD_QUARANTINE_REASON, FIELD_QUARANTINE_REVIEW_DUE_MS,
    FIELD_UPDATED_AT_MS, FIELD_VALID_AT_MS, FIELD_VERSION,
};
use super::super::types::*;
use crate::backend::{BackendKind, BackendPartition, BackendWriteOp};
use crate::error::{Error, Result};
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use web_time::Instant;

/// Report returned by bulk import operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkImportReport {
    /// Total number of records in the stream body.
    pub total_records: usize,
    /// Number of commit batches flushed to the engine.
    pub batches_committed: usize,
    /// Number of records that entered quarantine via the write-time flag
    /// (T1, ADR-046 §D5; additive field, SCH-05 review F3).
    #[serde(default)]
    pub quarantined: u64,
    /// Duration of the import in milliseconds.
    pub duration_ms: u64,
}

/// Quarantine state carried across re-writes of an existing key (I2 sticky,
/// ADR-046 §D5): a `put` never clears a quarantine.
#[derive(Clone, Default)]
struct QuarantineState {
    at_ms: Option<u64>,
    reason: Option<String>,
    by: Option<String>,
    review_due_ms: Option<u64>,
}

impl QuarantineState {
    fn from_record(record: &MemoryRecord) -> Self {
        Self {
            at_ms: record.quarantined_at_ms,
            reason: record.quarantine_reason.clone(),
            by: record.quarantined_by.clone(),
            review_due_ms: record.quarantine_review_due_ms,
        }
    }
}

/// Validate a quarantine reason code (ADR-046 §D2): non-empty lowercase
/// snake_case. The stable set is `explicit_write` | `unreviewed_import` |
/// `derived_promotion` | `policy_match` (reserved); the format check stays
/// open so future codes don't require a schema change.
fn validate_quarantine_reason(reason: &str) -> Result<()> {
    let valid = !reason.is_empty()
        && reason.len() <= 64
        && reason
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !valid {
        return Err(Error::Validation {
            field: "quarantine_reason".into(),
            reason: "must be a non-empty lowercase snake_case code".into(),
        });
    }
    Ok(())
}

impl Embedded {
    /// Review deadline for a record entering quarantine (ADR-046 §D5d):
    /// `now + quarantine_review_default_days` (config); `None` when the
    /// configured default is `0` (deadline disabled). Read-only signal —
    /// never triggers promotion by itself (I1).
    fn quarantine_review_due_ms(&self, now: u64) -> Option<u64> {
        let days = self.config.quarantine_review_default_days;
        (days > 0).then(|| now.saturating_add(u64::from(days) * 24 * 60 * 60 * 1000))
    }

    /// Materialize the quarantine entry (T1/T1c/T1d): sets the four state
    /// fields from `reason` + `by` (stable `system:<op>` actor) and stamps the
    /// review deadline. Sticky semantics live in the callers — call this only
    /// when the record is not already quarantined (I2).
    pub(crate) fn enter_quarantine(
        &self,
        record: &mut MemoryRecord,
        reason: &str,
        by: &str,
        now: u64,
    ) {
        record.quarantined_at_ms = Some(now);
        record.quarantine_reason = Some(reason.to_string());
        record.quarantined_by = Some(by.to_string());
        record.quarantine_review_due_ms = self.quarantine_review_due_ms(now);
    }

    /// F4 (SCH-05 review): metadata-only read of an existing record's
    /// quarantine state for the raw bulk path, which replaces the node
    /// wholesale and must not clear an existing quarantine (sticky, I2).
    /// `Ok(None)` = node absent or not quarantined.
    fn existing_quarantine_fields(
        engine: &crate::storage::StorageEngine,
        node_id: u128,
    ) -> Result<Option<(i64, Option<String>, Option<String>, Option<i64>)>> {
        let Some(bytes) =
            engine.get_from_partition(BackendPartition::Default, &node_id.to_le_bytes())?
        else {
            return Ok(None);
        };
        let Ok(metadata) = crate::storage::ops::deserialize_node_payload::<
            crate::storage::ops::NodeMetadata,
        >(&bytes, "node metadata") else {
            return Ok(None);
        };
        let fields = &metadata.relational;
        let at = match fields.get(FIELD_QUARANTINED_AT_MS) {
            Some(FieldValue::Int(at)) if *at > 0 => *at,
            _ => return Ok(None),
        };
        let reason = match fields.get(FIELD_QUARANTINE_REASON) {
            Some(FieldValue::String(s)) => Some(s.clone()),
            _ => None,
        };
        let by = match fields.get(FIELD_QUARANTINED_BY) {
            Some(FieldValue::String(s)) => Some(s.clone()),
            _ => None,
        };
        let due = match fields.get(FIELD_QUARANTINE_REVIEW_DUE_MS) {
            Some(FieldValue::Int(due)) if *due > 0 => Some(*due),
            _ => None,
        };
        Ok(Some((at, reason, by, due)))
    }
}

// ── MEMG-02: outcome-loop reinforcement policy (declared, not calibrated) ──
//
// Bump/decay are bounded (saturate at 1.0 / floor at 0.0) and the positive
// direction is rate-limited per record so repeated signals cannot inflate the
// score. Values are policy — empirical calibration is VER-08 (excluded here).

/// Confidence bump applied by a positive reinforcement (`Used`), saturated
/// at 1.0.
pub(crate) const REINFORCE_CONFIDENCE_BUMP: f32 = 0.05;

/// Confidence decay applied by a negative reinforcement (`Corrected`),
/// floored at 0.0.
pub(crate) const REINFORCE_CONFIDENCE_DECAY: f32 = 0.10;

/// Minimum time between positive reinforcements of the same record (rate
/// window, anchored on `last_validated_at_ms`). One bump per window.
pub(crate) const REINFORCE_WINDOW_MS: u64 = 300_000; // 5 min

impl Embedded {
    /// True when a vector is entirely zeros — the HNSW core rejects
    /// zero-norm vectors under cosine similarity (AUDREP-27), so the
    /// SDK treats them like empty vectors: keep the document, skip the
    /// vector index (matches `load.test.ts` seeding `[i % 10, 0, 0, 0]`).
    pub(super) fn usable_vector(vector: &[f32]) -> bool {
        !vector.is_empty() && vector.iter().any(|x| *x != 0.0)
    }

    pub(super) fn check_read_only(&self) -> Result<()> {
        if self.config.read_only {
            return Err(Error::Validation {
                field: "read_only".into(),
                reason: "this operation is not available when VantaDB is opened read-only".into(),
            });
        }
        Ok(())
    }

    /// Resolve the effective TTL (ms) for a write: an explicit `ttl_ms` wins;
    /// otherwise the namespace ("collection") default from
    /// [`Config::memory_default_ttl_ms`](crate::config::Config) applies, if any.
    ///
    /// Only new writes consult this — existing records are never backfilled
    /// when the default is configured (or changed) later.
    fn effective_ttl_ms(&self, namespace: &str, ttl_ms: Option<u64>) -> Option<u64> {
        ttl_ms.or_else(|| self.config.memory_default_ttl_ms.get(namespace).copied())
    }

    /// Validate + materialize the confidence fields of a write (ADR-046 §D4):
    /// - range: finite `[0,1]`;
    /// - V1: `derived` requires non-empty parents; `asserted` forbids them;
    /// - D4b: `derived` + declared score ⇒ rejection;
    /// - V3: bounded derivation depth / acyclicity;
    /// - score: `clamp(min(parents) × DERIVATION_DISCOUNT, 0, 1)`.
    fn materialize_confidence(
        &self,
        input: &MemoryInput,
    ) -> Result<(ConfidenceClass, f32, Vec<String>)> {
        if let Some(value) = input.confidence {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(Error::Validation {
                    field: "confidence".into(),
                    reason: "confidence must be a finite number in [0,1]".into(),
                });
            }
        }
        let class = input.confidence_class.unwrap_or_default();
        let parents = input.derived_from.clone().unwrap_or_default();

        match class {
            ConfidenceClass::Asserted => {
                if !parents.is_empty() {
                    return Err(Error::Validation {
                        field: "derived_from".into(),
                        reason: "asserted records cannot declare derived_from parents".into(),
                    });
                }
                Ok((
                    class,
                    input.confidence.unwrap_or_else(default_confidence),
                    Vec::new(),
                ))
            }
            ConfidenceClass::Derived => {
                if parents.is_empty() {
                    return Err(Error::Validation {
                        field: "derived_from".into(),
                        reason: "derived records require a non-empty derived_from parent list (V1)"
                            .into(),
                    });
                }
                if input.confidence.is_some() {
                    return Err(Error::Validation {
                        field: "confidence".into(),
                        reason: "derived score is computed from parents; declared scores are not allowed on derived records".into(),
                    });
                }
                self.validate_derivation_chain(&input.namespace, &input.key, &parents)?;

                let mut min_parent = f32::INFINITY;
                for parent_key in &parents {
                    let parent = self.get(&input.namespace, parent_key)?.ok_or_else(|| {
                        Error::Validation {
                            field: "derived_from".into(),
                            reason: format!(
                                "parent '{parent_key}' not found in namespace '{}'",
                                input.namespace
                            ),
                        }
                    })?;
                    min_parent = min_parent.min(parent.confidence);
                }
                let score = (min_parent * DERIVATION_DISCOUNT).clamp(0.0, 1.0);
                Ok((class, score, parents))
            }
        }
    }

    /// V3 (ADR-046 §D4): reject derivation cycles (including self-reference)
    /// and chains deeper than [`MAX_DERIVATION_DEPTH`]. Walks existing
    /// ancestors only — direct parent existence is enforced by the caller.
    fn validate_derivation_chain(
        &self,
        namespace: &str,
        key: &str,
        parents: &[String],
    ) -> Result<()> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut frontier: Vec<(String, usize)> =
            parents.iter().map(|p| (p.clone(), 1usize)).collect();
        while let Some((parent_key, depth)) = frontier.pop() {
            if parent_key == key {
                return Err(Error::Validation {
                    field: "derived_from".into(),
                    reason: format!("derivation cycle detected: '{key}' is its own ancestor"),
                });
            }
            if depth > MAX_DERIVATION_DEPTH {
                return Err(Error::Validation {
                    field: "derived_from".into(),
                    reason: format!(
                        "derivation chain exceeds MAX_DERIVATION_DEPTH ({MAX_DERIVATION_DEPTH})"
                    ),
                });
            }
            if !visited.insert(parent_key.clone()) {
                continue;
            }
            if let Some(parent) = self.get(namespace, &parent_key)? {
                for grandparent in parent.derived_from {
                    frontier.push((grandparent, depth + 1));
                }
            }
        }
        Ok(())
    }

    /// `valid_at_ms` materialization (ADR-046 §D8): an absent value defaults to
    /// `created_at_ms`; `Some(0)` is rejected because `0` is the v1 "unset"
    /// sentinel — an explicit epoch-0 validity is not representable.
    fn materialize_valid_at(&self, input: Option<u64>, created_at_ms: u64) -> Result<u64> {
        match input {
            None => Ok(created_at_ms),
            Some(0) => Err(Error::Validation {
                field: "valid_at_ms".into(),
                reason: "must be greater than 0; omit the field to default to created_at_ms".into(),
            }),
            Some(value) => Ok(value),
        }
    }

    /// Resolve the existing record for a write to `(namespace, key)`.
    ///
    /// A physically present node whose record is hidden by lazy TTL eviction
    /// (`record_from_node` → `None`) is **purged on write** (DUR-03 / H-023):
    /// the expired record is logically absent — `get` hides it, `delete`
    /// reports nothing, `list`/search exclude it — so a write must not
    /// collide with it. Purging first gives the same observable outcome as
    /// the documented `purge_expired()`-then-`put` workaround: fresh version 1
    /// and `created_at_ms` reset (the single-record purge is a superset of the
    /// sweeper's cleanup — it also removes sparse index entries, which
    /// `purge_expired` does not).
    ///
    /// Returns the existing live record (if any) **plus the read guard that
    /// stabilizes its generation**: the caller must hold it across the
    /// subsequent insert + index replacement, so a concurrent purge cannot
    /// remove the generation's stats in between (a second decrement would
    /// drive the text df negative). The guard is `None` for a fresh insert —
    /// there is no generation to protect.
    ///
    /// [`Error::NodeIdCollision`] is returned when the deterministic id is
    /// occupied by a different key or a non-memory node; inside an active
    /// transaction the expired case keeps that same error (see
    /// `purge_expired_record`).
    fn resolve_existing_for_write(
        &self,
        engine: &crate::storage::StorageEngine,
        namespace: &str,
        key: &str,
    ) -> Result<(
        Option<MemoryRecord>,
        Option<parking_lot::RwLockReadGuard<'_, ()>>,
    )> {
        let node_id = memory_node_id(namespace, key);
        loop {
            let Some(node) = engine.get(node_id)? else {
                return Ok((None, None));
            };
            match record_from_node(&node) {
                Some(record) if record.namespace == namespace && record.key == key => {
                    // Live record: take the read guard and re-verify under it
                    // (a purge may have won the race since the peek).
                    let guard = self.purge_lock.read();
                    let Some(node) = engine.get(node_id)? else {
                        drop(guard);
                        continue;
                    };
                    match record_from_node(&node) {
                        Some(record) if record.namespace == namespace && record.key == key => {
                            return Ok((Some(record), Some(guard)));
                        }
                        Some(_) => return Err(Error::NodeIdCollision(node_id)),
                        None => {
                            // Expired between the peek and the guard: retry as
                            // the purge-on-write case.
                            drop(guard);
                            if self
                                .purge_expired_record(engine, namespace, key, node_id)?
                                .is_none()
                            {
                                return Ok((None, None));
                            }
                            continue;
                        }
                    }
                }
                Some(_) => return Err(Error::NodeIdCollision(node_id)),
                None => match memory_record_from_node_include_expired(&node) {
                    Some(expired) if expired.namespace == namespace && expired.key == key => {
                        if self
                            .purge_expired_record(engine, namespace, key, node_id)?
                            .is_none()
                        {
                            return Ok((None, None));
                        }
                        continue;
                    }
                    _ => return Err(Error::NodeIdCollision(node_id)),
                },
            }
        }
    }

    /// Physically remove one expired record and its derived entries, under the
    /// `purge_lock` **write** guard. Uses the same primitives as `delete_inner`
    /// — node delete (KV + HNSW + shred), derived/text/sparse index
    /// replacement to `None`, version-history purge — plus a re-check: if a
    /// concurrent purge already removed the node, or a concurrent write
    /// refreshed it, the cleanup must not run again (a second stats decrement
    /// would drive the text df negative).
    ///
    /// Returns `Ok(Some(record))` when the node was refreshed into a live
    /// record of this key (the caller should upsert over it), `Ok(None)` when
    /// the expired record was purged or the node vanished, and
    /// [`Error::NodeIdCollision`] when the id now holds a foreign node.
    ///
    /// Inside an active transaction `engine.delete` only buffers the node
    /// delete while the index cleanup would apply immediately — an abort
    /// would leave the node present with its stats already gone — so this
    /// refuses to purge and the caller keeps the pre-DUR-03 collision error.
    fn purge_expired_record(
        &self,
        engine: &crate::storage::StorageEngine,
        namespace: &str,
        key: &str,
        node_id: u128,
    ) -> Result<Option<MemoryRecord>> {
        let _guard = self.purge_lock.write();
        if engine.txn.has_active() {
            return Err(Error::NodeIdCollision(node_id));
        }
        let Some(node) = engine.get(node_id)? else {
            return Ok(None);
        };
        match record_from_node(&node) {
            Some(record) if record.namespace == namespace && record.key == key => {
                return Ok(Some(record));
            }
            Some(_) => return Err(Error::NodeIdCollision(node_id)),
            None => {}
        }
        let Some(current) = memory_record_from_node_include_expired(&node) else {
            // Present but not a parseable memory record of this key: a foreign
            // node — never silently overwrite it.
            return Err(Error::NodeIdCollision(node_id));
        };
        if current.namespace != namespace || current.key != key {
            return Err(Error::NodeIdCollision(node_id));
        }
        engine.delete(node_id, "ttl_rewrite")?;
        self.replace_derived_indexes(engine, Some(&current), None)?;
        // Best-effort class, same as `delete_inner` (VS-CORE-07).
        let _ = super::super::version_history::purge_key(engine, &current.namespace, &current.key);
        Ok(None)
    }

    /// Shared logic for inserting/updating a single memory record.
    /// Used by both `put()` and `put_batch()`.
    fn put_one(&self, input: MemoryInput) -> Result<MemoryRecord> {
        self.check_read_only()?;
        validate_namespace(&input.namespace)?;
        validate_key(&input.key)?;
        validate_metadata(&input.metadata)?;

        let engine = self.engine_handle()?;
        let node_id = memory_node_id(&input.namespace, &input.key);
        // The guard stabilizes a live generation across insert + index
        // replacement (DUR-03 race hardening); `None` for fresh inserts.
        let (existing, _generation_guard) =
            self.resolve_existing_for_write(&engine, &input.namespace, &input.key)?;

        let timestamp = now_ms();
        let created_at_ms = existing
            .as_ref()
            .map(|r| r.created_at_ms)
            .unwrap_or(timestamp);
        let version = existing
            .as_ref()
            .map(|r| r.version.saturating_add(1))
            .unwrap_or(1);
        let expires_at_ms = self
            .effective_ttl_ms(&input.namespace, input.ttl_ms)
            .map(|ttl| timestamp.saturating_add(ttl));

        let (confidence_class, confidence, derived_from) = self.materialize_confidence(&input)?;
        let valid_at_ms = self.materialize_valid_at(input.valid_at_ms, created_at_ms)?;
        let mut quarantine = existing
            .as_ref()
            .map(QuarantineState::from_record)
            .unwrap_or_default();
        // T1 (ADR-046 §D5, MGR-13 §3.2): an explicit `quarantine: true` write
        // enters the quarantine state; sticky — an existing quarantine always
        // wins (I2), only T2/T4 exit.
        let quarantine_entered = quarantine.at_ms.is_none() && input.quarantine;
        if quarantine_entered {
            quarantine.at_ms = Some(timestamp);
            quarantine.reason = Some("explicit_write".into());
            quarantine.by = Some("system:put".into());
            quarantine.review_due_ms = self.quarantine_review_due_ms(timestamp);
        }

        let record = MemoryRecord {
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
            valid_at_ms,
            invalid_at_ms: None,
            confidence_class,
            confidence,
            last_validated_at_ms: None,
            derived_from,
            quarantined_at_ms: quarantine.at_ms,
            quarantine_reason: quarantine.reason,
            quarantined_by: quarantine.by,
            quarantine_review_due_ms: quarantine.review_due_ms,
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

        if quarantine_entered {
            self.audit(crate::audit::AuditEvent::new(
                "quarantine_enter",
                &record.namespace,
                &record.key,
                "ok",
                Some("explicit_write".to_string()),
            ));
        }

        Ok(record)
    }

    /// Insert or update a persistent memory record.
    /// Returns the created/updated record with system-assigned timestamps and version.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vantadb::config::Config;
    /// use vantadb::{BackendKind, Embedded, MemoryInput};
    ///
    /// let db = Embedded::open_with_config(Config {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// })
    /// .expect("open in-memory database");
    ///
    /// let record = db
    ///     .put(MemoryInput::new("docs", "greeting", "Hello, VantaDB!"))
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
    pub fn put(&self, input: MemoryInput) -> Result<MemoryRecord> {
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
    pub fn put_batch(&self, inputs: Vec<MemoryInput>) -> Result<Vec<MemoryRecord>> {
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

    fn put_batch_inner(&self, inputs: Vec<MemoryInput>) -> Result<Vec<MemoryRecord>> {
        use crate::storage::engine::{BatchInsertOptions, InsertMode};

        for input in &inputs {
            validate_namespace(&input.namespace)?;
            validate_key(&input.key)?;
            validate_metadata(&input.metadata)?;
        }

        let engine = self.engine_handle()?;
        let batch_size = self.config.batch_size.unwrap_or(1000);
        let mut all_results: Vec<MemoryRecord> = Vec::with_capacity(inputs.len());
        let mut rebuild_needed = false;
        // Track versions + quarantine state for keys seen earlier in this
        // batch (in-batch dedup, mirrors put_one's UPSERT semantics). Persisted
        // before the chunk loop so duplicate keys split across chunks still
        // bump correctly.
        let mut seen: HashMap<u128, (u64, QuarantineState)> = HashMap::with_capacity(inputs.len());

        for chunk in inputs.chunks(batch_size) {
            let timestamp = now_ms();
            let mut nodes: Vec<UnifiedNode> = Vec::with_capacity(chunk.len());
            let mut records: Vec<MemoryRecord> = Vec::with_capacity(chunk.len());
            // SCH-05 review F3: T1 entries of this chunk, audited post-commit.
            let mut entered_quarantine: Vec<(String, String)> = Vec::new();

            for input in chunk {
                let node_id = memory_node_id(&input.namespace, &input.key);
                // Existing record: in-batch duplicate wins (already bumped), else
                // consult the engine like put_one (pre-existing records from
                // earlier batches should also increment, not reset to 1).
                let (prev_version, prev_created_at_ms, mut quarantine) = if let Some((v, q)) =
                    seen.get(&node_id)
                {
                    (Some(*v), Some(timestamp), q.clone())
                } else {
                    // NOTE: the guard is intentionally dropped at the end of
                    // this match — the batch path finishes with full index
                    // rebuilds, so it does not need the generation pinned.
                    match self.resolve_existing_for_write(&engine, &input.namespace, &input.key)? {
                        (Some(record), _guard) => (
                            Some(record.version),
                            Some(record.created_at_ms),
                            QuarantineState::from_record(&record),
                        ),
                        (None, _guard) => (None, None, QuarantineState::default()),
                    }
                };
                let created_at_ms = prev_created_at_ms.unwrap_or(timestamp);
                let version = prev_version.map(|v| v.saturating_add(1)).unwrap_or(1);

                // T1 (ADR-046 §D5): same write-time flag semantics as put_one
                // (sticky wins; entry only when not already quarantined).
                let quarantine_entered = quarantine.at_ms.is_none() && input.quarantine;
                if quarantine_entered {
                    quarantine.at_ms = Some(timestamp);
                    quarantine.reason = Some("explicit_write".into());
                    quarantine.by = Some("system:put_batch".into());
                    quarantine.review_due_ms = self.quarantine_review_due_ms(timestamp);
                    entered_quarantine.push((input.namespace.clone(), input.key.clone()));
                }

                let (confidence_class, confidence, derived_from) =
                    self.materialize_confidence(input)?;
                let valid_at_ms = self.materialize_valid_at(input.valid_at_ms, created_at_ms)?;
                seen.insert(node_id, (version, quarantine.clone()));

                let record = MemoryRecord {
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
                    expires_at_ms: self
                        .effective_ttl_ms(&input.namespace, input.ttl_ms)
                        .map(|ttl| timestamp.saturating_add(ttl)),
                    superseded_by: None,
                    superseded_at_ms: None,
                    valid_at_ms,
                    invalid_at_ms: None,
                    confidence_class,
                    confidence,
                    last_validated_at_ms: None,
                    derived_from,
                    quarantined_at_ms: quarantine.at_ms,
                    quarantine_reason: quarantine.reason,
                    quarantined_by: quarantine.by,
                    quarantine_review_due_ms: quarantine.review_due_ms,
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

            // SCH-05 review F3: audit each T1 entry post-commit (mirrors
            // put_one; the write-time quarantine flag is a domain event).
            for (ns, key) in &entered_quarantine {
                self.audit(crate::audit::AuditEvent::new(
                    "quarantine_enter",
                    ns,
                    key,
                    "ok",
                    Some("explicit_write".to_string()),
                ));
            }

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
    /// use vantadb::config::Config;
    /// use vantadb::{BackendKind, Embedded, MemoryInput};
    ///
    /// let db = Embedded::open_with_config(Config {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// })
    /// .expect("open in-memory database");
    ///
    /// db.put(MemoryInput::new("docs", "greeting", "Hello, VantaDB!"))
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
    pub fn get(&self, namespace: &str, key: &str) -> Result<Option<MemoryRecord>> {
        validate_namespace(namespace)?;
        validate_key(key)?;

        let node_id = memory_node_id(namespace, key);
        let Some(node) = self.engine_handle()?.get(node_id)? else {
            return Ok(None);
        };

        match record_from_node(&node) {
            Some(record) if record.namespace == namespace && record.key == key => Ok(Some(record)),
            Some(_record) => Err(Error::NodeIdCollision(memory_node_id(namespace, key))),
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
    ) -> Result<Option<MemoryRecord>> {
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
    pub fn versions(&self, namespace: &str, key: &str) -> Result<Vec<MemoryRecord>> {
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
    /// use vantadb::config::Config;
    /// use vantadb::{BackendKind, Embedded, MemoryInput};
    ///
    /// let db = Embedded::open_with_config(Config {
    ///     storage_path: ":memory:".into(),
    ///     backend_kind: BackendKind::InMemory,
    ///     ..Default::default()
    /// })
    /// .expect("open in-memory database");
    ///
    /// db.put(MemoryInput::new("docs", "greeting", "Hello, VantaDB!"))
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
        Ok(self.delete_inner(namespace, key)?.is_some())
    }

    /// Shared delete logic (VER-02): performs the delete and returns the
    /// removed record (`None` when the key did not exist).
    fn delete_inner(&self, namespace: &str, key: &str) -> Result<Option<MemoryRecord>> {
        self.check_read_only()?;
        validate_namespace(namespace)?;
        validate_key(key)?;

        let Some(existing) = self.get(namespace, key)? else {
            return Ok(None);
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
        Ok(Some(existing))
    }

    /// Delete a record and emit a **purge certificate** (VER-02).
    ///
    /// The certificate inventories every purge surface (store, JSON-shredded
    /// metadata, HNSW/vector store, derived/payload, text, sparse, version
    /// history, WAL tombstone), records the exact evidence per surface, and
    /// carries a deterministic `sha256` integrity hash plus a reference to the
    /// VER-01 WAL hash-chain. Declared limits (physical media, backups,
    /// archived WAL segments, exports, audit-log retention, parametric
    /// unlearning) are always listed — the certificate never overstates the
    /// purge.
    ///
    /// When the key does not exist, the certificate still reports the keyless
    /// surfaces and marks record-dependent ones `not-assessed` (status
    /// `not_found`) instead of claiming a clean purge.
    ///
    /// # Durability (cross-process verification)
    ///
    /// The certificate reflects the state of this live engine handle. A
    /// verifier that opens the database **read-only** (e.g.
    /// `vanta-cli certificate verify`) does not replay the WAL, so callers
    /// that intend to verify the certificate after this process exits must
    /// persist the purge first — call [`Embedded::flush`] or
    /// [`Embedded::close`] after this method. The CLI delete path already
    /// closes the database before returning.
    #[tracing::instrument(skip(self), err)]
    pub fn delete_certified(
        &self,
        namespace: &str,
        key: &str,
    ) -> Result<crate::attestation::PurgeCertificate> {
        let deleted = self.delete_inner(namespace, key)?;
        let engine = self.engine_handle()?;
        let node_id = memory_node_id(namespace, key);
        crate::attestation::build_certificate(
            &engine,
            deleted.as_ref(),
            namespace,
            key,
            node_id,
            "memory delete",
        )
    }

    /// Verify a stored purge certificate (VER-02): schema + integrity hash,
    /// then a re-scan of the surfaces that are re-checkable without the
    /// deleted record.
    ///
    /// Fails when the certificate is structurally invalid (claimless/partial
    /// surface inventory, unknown status, empty declared limits), when it was
    /// edited/corrupted (hash mismatch), or when a surface it claimed clean
    /// now holds entries / live residues remain while the certificate does not
    /// attest a `not_found` report.
    ///
    /// Verification runs against the **live database handle**; a certificate
    /// emitted by an unflushed process can yield a false negative until that
    /// process flushes/closes (see [`Embedded::delete_certified`] §Durability).
    /// The certificate is not bound to a database instance — verification
    /// matches by namespace/key/node_id against whichever database is opened.
    #[tracing::instrument(skip(self, certificate_json), err)]
    pub fn verify_purge_certificate(
        &self,
        certificate_json: &str,
    ) -> Result<crate::attestation::PurgeCertificateVerification> {
        let certificate: crate::attestation::PurgeCertificate =
            serde_json::from_str(certificate_json).map_err(Error::serialization)?;
        let engine = self.engine_handle()?;
        crate::attestation::verify_certificate(&engine, &certificate)
    }

    /// Insert or update a record with exact fields (used internally by import).
    ///
    /// Raw transport choke point (SDK Rust / HTTP `import` records / WASM
    /// `import_records`): re-validates the confidence boundary + validity
    /// window even though the JSONL path (`record_from_export_line`) already
    /// validated them — a hostile `records` payload must not persist
    /// out-of-range scores, inconsistent classes or inverted windows
    /// (ADR-046 §D4/§D7).
    pub(crate) fn put_record_exact(&self, mut record: MemoryRecord) -> Result<MemoryRecord> {
        self.check_read_only()?;
        validate_namespace(&record.namespace)?;
        validate_key(&record.key)?;
        validate_metadata(&record.metadata)?;
        validate_confidence_fields(
            record.confidence_class,
            &record.derived_from,
            record.confidence,
        )?;
        if let Some(invalid_at) = record.invalid_at_ms {
            if record.valid_at_ms > invalid_at {
                return Err(Error::Validation {
                    field: "invalid_at_ms".into(),
                    reason: format!(
                        "valid_at_ms ({}) must be <= invalid_at_ms ({invalid_at})",
                        record.valid_at_ms
                    ),
                });
            }
        }

        let expected_node_id = memory_node_id(&record.namespace, &record.key);
        if record.node_id != expected_node_id {
            return Err(Error::Validation {
                field: "node_id".into(),
                reason: format!("node_id does not match deterministic namespace/key hash for namespace='{}' key='{}'", record.namespace, record.key),
            });
        }

        let engine = self.engine_handle()?;
        // See `put_one`: hold the generation guard across insert + replace.
        let (previous, _generation_guard) =
            self.resolve_existing_for_write(&engine, &record.namespace, &record.key)?;

        // F4 (SCH-05 review): sticky on the raw transport — an incoming record
        // that carries no quarantine state must not clear an existing one
        // (closes the I2 bypass via the agent-facing `import` tool). Only the
        // explicit T2/T4 ops leave quarantine.
        if record.quarantined_at_ms.is_none() {
            if let Some(prev) = previous.as_ref() {
                if prev.quarantined_at_ms.is_some() {
                    record.quarantined_at_ms = prev.quarantined_at_ms;
                    record.quarantine_reason = prev.quarantine_reason.clone();
                    record.quarantined_by = prev.quarantined_by.clone();
                    record.quarantine_review_due_ms = prev.quarantine_review_due_ms;
                }
            }
        }

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
            return Err(Error::InvalidInput(
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
            .ok_or_else(|| Error::NotFound {
                kind: "memory record".into(),
                id: format!("{namespace}/{old_key}"),
            })?;
        if old.superseded_by.is_some() {
            return Err(Error::InvalidInput(format!(
                "record '{old_key}' is already superseded by '{}'",
                old.superseded_by.as_deref().unwrap_or_default()
            )));
        }
        if self.get(namespace, new_key)?.is_none() {
            return Err(Error::NotFound {
                kind: "memory record".into(),
                id: format!("{namespace}/{new_key}"),
            });
        }

        let now = now_ms();
        let mut record = old;
        // O2 (ADR-046 §D3-2/§D3-3): `invalid_at = now` would invert the window
        // (`valid_at > invalid_at`) for a record whose validity starts in the
        // future. Guard instead of writing an inconsistent state; superseding
        // a not-yet-valid record is rejected explicitly (no silent clamp, no
        // D3-3 divergence).
        if record.valid_at_ms > now {
            return Err(Error::Validation {
                field: "valid_at_ms".into(),
                reason: format!(
                    "cannot supersede a record whose validity starts in the future (valid_at_ms {} > now {now})",
                    record.valid_at_ms
                ),
            });
        }
        record.superseded_by = Some(new_key.to_string());
        record.superseded_at_ms = Some(now);
        // ADR-046 §D3-3: 0.8.0 keeps the validity window aligned with the
        // supersession event (divergence only via a retroactive setter, v1.0).
        record.invalid_at_ms = Some(now);
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

    /// MEMG-02: report the outcome of a recalled memory and feed it back into
    /// the record's confidence — the write-side of the outcome loop.
    ///
    /// The host declares the outcome explicitly ([`ReinforceOutcome`]); the
    /// engine never infers it. Declared policy (see `docs/api/scores.md`
    /// §Reinforcement):
    ///
    /// - `Used` — `confidence = min(1.0, confidence + 0.05)` and
    ///   `last_validated_at_ms = now` (successful re-validation), rate-limited
    ///   to one bump per record per 5-minute window anchored on the previous
    ///   stamp; inside the window the call is a no-op (audited as
    ///   `used_rate_limited`, never silent).
    /// - `Corrected` — `confidence = max(0.0, confidence - 0.10)`; failures do
    ///   NOT stamp `last_validated_at_ms` (success-only, MGR-12 §3.3).
    /// - `Unused` — neutral: no score change, no stamp; the audit event is the
    ///   record of the host's declaration.
    ///
    /// Interaction notes: a `Corrected` does NOT reset the positive rate-limit
    /// anchor — a `Used` shortly after a correction is still limited by the
    /// previous validation stamp (the window counts successful validations).
    /// Quarantine state is not inspected: quarantined records are unreachable
    /// through default recall/list (excluded by default), so a host can only
    /// reinforce one by explicit key.
    ///
    /// Applies to `asserted` records only: a `derived` score is computed from
    /// its parents (`min × DERIVATION_DISCOUNT`, ADR-046 §D4) and mutating it
    /// would break determinism (V4) — derived records are rejected explicitly.
    ///
    /// State-only change (same class as `quarantine_apply`): `version` does not
    /// change and no version-history snapshot is written (a reinforcement per
    /// turn would flood the 32-entry history); `updated_at_ms` is refreshed and
    /// the operation is audited as `memory_reinforce`. Callers that never
    /// invoke this op observe exactly the previous behavior — `put` still
    /// leaves `last_validated_at_ms = None`.
    #[tracing::instrument(skip(self), err)]
    pub fn reinforce(
        &self,
        namespace: &str,
        key: &str,
        outcome: ReinforceOutcome,
    ) -> Result<MemoryRecord> {
        self.check_read_only()?;
        validate_namespace(namespace)?;
        validate_key(key)?;
        // REVIEW-13 pattern: serialize the read-modify-write (the engine's
        // insert_lock only serializes the individual insert, not the SDK-level
        // get + mutate) — same rationale as `supersede`/`quarantine_*`.
        let _guard = self.supersede_lock.lock();

        let Some(mut record) = self.get(namespace, key)? else {
            return Err(Error::NotFound {
                kind: "memory record".into(),
                id: format!("{namespace}/{key}"),
            });
        };
        if record.confidence_class == ConfidenceClass::Derived {
            return Err(Error::InvalidInput(format!(
                "record '{key}' is derived — its confidence is computed from its parents (min × DERIVATION_DISCOUNT) and cannot be reinforced; reinforce the parents instead"
            )));
        }

        let now = now_ms();
        let mut reason = outcome.as_wire_str().to_string();
        let mut changed = false;
        match outcome {
            ReinforceOutcome::Used => {
                let in_window = record
                    .last_validated_at_ms
                    .is_some_and(|prev| now.saturating_sub(prev) < REINFORCE_WINDOW_MS);
                if in_window {
                    // Rate-limited: no score change, no re-stamp. Never
                    // silent — the audit reason distinguishes it.
                    reason = "used_rate_limited".to_string();
                } else {
                    record.confidence = (record.confidence + REINFORCE_CONFIDENCE_BUMP).min(1.0);
                    record.last_validated_at_ms = Some(now);
                    changed = true;
                }
            }
            ReinforceOutcome::Corrected => {
                record.confidence = (record.confidence - REINFORCE_CONFIDENCE_DECAY).max(0.0);
                changed = true;
            }
            ReinforceOutcome::Unused => {
                // Neutral: nothing to write; the audit event records the
                // explicit declaration.
            }
        }

        if changed {
            record.updated_at_ms = now;
            // Same write path as the quarantine state ops: WAL + KV + HNSW via
            // engine.insert. Payload/metadata/vector are unchanged, and the
            // confidence fields are not indexed, so the derived text/scalar
            // indexes stay consistent with no index writes.
            let engine = self.engine_handle()?;
            let (node, record) = memory_record_to_node_owned(record);
            engine.insert(&node)?;
            self.audit(crate::audit::AuditEvent::new(
                "memory_reinforce",
                namespace,
                key,
                "ok",
                Some(reason),
            ));
            return Ok(record);
        }

        self.audit(crate::audit::AuditEvent::new(
            "memory_reinforce",
            namespace,
            key,
            "ok",
            Some(reason),
        ));
        Ok(record)
    }

    /// T1d (ADR-046 §D5, MGR-13 §3.2): quarantine an existing record post-hoc
    /// (`quarantine_apply`).
    ///
    /// Sets the four quarantine state fields with the caller's `reason` code
    /// (default `explicit_write`), the `system:quarantine_apply` applier and
    /// the review deadline (config, §D5d). Idempotent: a record already
    /// quarantined is returned as-is (sticky, I2). The state change does not
    /// bump `version` (state ≠ content; no history snapshot — the audit event
    /// is the evidence) and is audited as `quarantine_enter`.
    #[tracing::instrument(skip(self), err)]
    pub fn quarantine_apply(
        &self,
        namespace: &str,
        key: &str,
        reason: Option<&str>,
    ) -> Result<MemoryRecord> {
        self.check_read_only()?;
        let reason = reason.unwrap_or("explicit_write");
        validate_quarantine_reason(reason)?;
        // REVIEW-13 pattern: serialize the read-modify-write below (same
        // rationale as `supersede` — the engine's insert_lock only serializes
        // the individual insert, not the SDK-level get + mutate).
        let _guard = self.supersede_lock.lock();

        let Some(mut record) = self.get(namespace, key)? else {
            return Err(Error::NotFound {
                kind: "memory record".into(),
                id: format!("{namespace}/{key}"),
            });
        };
        if record.quarantined_at_ms.is_some() {
            return Ok(record); // sticky: already quarantined (I2)
        }
        let now = now_ms();
        self.enter_quarantine(&mut record, reason, "system:quarantine_apply", now);
        record.updated_at_ms = now;

        let engine = self.engine_handle()?;
        let (node, record) = memory_record_to_node_owned(record);
        engine.insert(&node)?;
        self.audit(crate::audit::AuditEvent::new(
            "quarantine_enter",
            namespace,
            key,
            "ok",
            Some(reason.to_string()),
        ));
        Ok(record)
    }

    /// T2 (ADR-046 §D5, MGR-13 §3.2): promote a quarantined record back to
    /// active (`quarantine_promote`).
    ///
    /// Explicit, audited act — the only exits are T2 and T4 (I1: nothing
    /// promotes by clock/TTL). Clears the four quarantine fields and bumps
    /// `updated_at_ms`; `version` does **not** change and no version-history
    /// snapshot is written (state ≠ content; the audit event is the evidence).
    /// Errors when the record is missing or not quarantined.
    #[tracing::instrument(skip(self), err)]
    pub fn quarantine_promote(&self, namespace: &str, key: &str) -> Result<MemoryRecord> {
        self.check_read_only()?;
        let _guard = self.supersede_lock.lock();

        let Some(mut record) = self.get(namespace, key)? else {
            return Err(Error::NotFound {
                kind: "memory record".into(),
                id: format!("{namespace}/{key}"),
            });
        };
        if record.quarantined_at_ms.is_none() {
            return Err(Error::InvalidInput(format!(
                "record '{key}' is not quarantined (nothing to promote)"
            )));
        }
        let now = now_ms();
        record.quarantined_at_ms = None;
        record.quarantine_reason = None;
        record.quarantined_by = None;
        record.quarantine_review_due_ms = None;
        record.updated_at_ms = now;

        let engine = self.engine_handle()?;
        let (node, record) = memory_record_to_node_owned(record);
        engine.insert(&node)?;
        self.audit(crate::audit::AuditEvent::new(
            "quarantine_promote",
            namespace,
            key,
            "ok",
            None,
        ));
        Ok(record)
    }

    /// T4 (ADR-046 §D5, MGR-13 §3.2): reject (delete) a quarantined record
    /// (`quarantine_reject`; destructive — the call itself is the confirmation).
    ///
    /// Errors when the record is missing or not quarantined. The underlying
    /// delete and the rejection are both audited (`delete` +
    /// `quarantine_reject`).
    #[tracing::instrument(skip(self), err)]
    pub fn quarantine_reject(&self, namespace: &str, key: &str) -> Result<bool> {
        self.check_read_only()?;
        // F6 (SCH-05 review): serialize get→check→delete against concurrent
        // promote/apply (same REVIEW-13 rationale as `supersede`: the engine's
        // insert_lock only serializes individual writes, not this check-then-act).
        let _guard = self.supersede_lock.lock();
        let Some(record) = self.get(namespace, key)? else {
            return Err(Error::NotFound {
                kind: "memory record".into(),
                id: format!("{namespace}/{key}"),
            });
        };
        if record.quarantined_at_ms.is_none() {
            return Err(Error::InvalidInput(format!(
                "record '{key}' is not quarantined (nothing to reject)"
            )));
        }
        let deleted = self.delete(namespace, key)?;
        self.audit(crate::audit::AuditEvent::new(
            "quarantine_reject",
            namespace,
            key,
            if deleted { "ok" } else { "err" },
            None,
        ));
        Ok(deleted)
    }

    /// Scan all memory records and physically delete those whose expiry deadline has passed.
    #[tracing::instrument(skip(self), err)]
    pub fn purge_expired(&self) -> Result<u64> {
        self.check_read_only()?;
        let engine = self.engine_handle()?;
        // DUR-03: serialize against the purge-on-write path and against
        // upserts that hold the read guard (a second stats decrement for the
        // same generation would drive the text df negative).
        let _guard = self.purge_lock.write();
        // DUR-03 review (round 2): inside an active transaction `engine.delete`
        // only buffers the node delete while the index cleanup would apply
        // immediately — an abort would leave the node present with its stats
        // already gone. Skip the sweep; the next one (post-txn) purges.
        if engine.txn.has_active() {
            return Ok(0);
        }
        let now = now_ms();
        let mut to_delete: Vec<MemoryRecord> = Vec::new();

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
                let mut metadata_fields = Fields::new();
                for (fk, fv) in fields {
                    if !fk.starts_with("__vanta_") {
                        metadata_fields.insert(fk.clone(), fv.clone().into());
                    }
                }
                to_delete.push(MemoryRecord {
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
                    ..Default::default()
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
            let _ =
                super::super::version_history::purge_key(&engine, &record.namespace, &record.key);
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
                                let mut guard = engine.cache.text_stats.write();
                                guard.insert((ns, token), stats);
                                // ponytail: watermark eviction — drop first half if over limit
                                if guard.len() > crate::config::MAX_TEXT_STATS_CACHE {
                                    let keys: Vec<_> =
                                        guard.keys().take(guard.len() / 2).cloned().collect();
                                    for k in keys {
                                        guard.remove(&k);
                                    }
                                }
                            }
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            if let Ok(stats) = crate::text_index::decode_namespace_stats(value) {
                                let mut guard = engine.cache.text_ns.write();
                                guard.insert(ns, stats);
                                // ponytail: watermark eviction — drop first half if over limit
                                if guard.len() > crate::config::MAX_TEXT_NS_CACHE {
                                    let keys: Vec<_> =
                                        guard.keys().take(guard.len() / 2).cloned().collect();
                                    for k in keys {
                                        guard.remove(&k);
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
                            let mut guard = engine.cache.text_stats.write();
                            guard.remove(&(ns, token));
                        }
                    } else if crate::text_index::is_namespace_stats_key(key) {
                        if let Some(ns) = Self::parse_namespace_stats_key(key) {
                            let mut guard = engine.cache.text_ns.write();
                            guard.remove(&ns);
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
    /// 8-byte LE record count, then serde_json-serialized `Vec<MemoryInput>`.
    ///
    /// Bypasses per-record validation (`validate_namespace`, `validate_key`,
    /// `validate_metadata`) for raw throughput. Commits to the engine in batches
    /// sized by [`Config::bulk_commit_interval`](crate::Config::bulk_commit_interval) (default: 10 000).
    pub fn bulk_import_stream<R: std::io::Read>(&self, reader: &mut R) -> Result<BulkImportReport> {
        self.check_read_only()?;
        let start = Instant::now();

        // ── Header ──
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        if &magic != b"VDBJSON\n" {
            return Err(Error::Validation {
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
            return Err(Error::Validation {
                field: "version".into(),
                reason: format!("unsupported format version: {}", version[0]),
            });
        }

        let mut raw_count = [0u8; 8];
        reader.read_exact(&mut raw_count)?;
        let total = u64::from_le_bytes(raw_count) as usize;

        // ── Body: serde_json-serialized Vec<MemoryInput> ──
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;

        let records: Vec<MemoryInput> =
            serde_json::from_slice(&buf).map_err(|e| Error::Validation {
                field: "body".into(),
                reason: format!("JSON deserialization failed: {}", e),
            })?;

        if records.len() != total {
            return Err(Error::Validation {
                field: "count".into(),
                reason: format!("declared {} records but got {}", total, records.len()),
            });
        }

        let engine = self.engine_handle()?;
        let commit_interval = self.config.bulk_commit_interval.unwrap_or(10_000);
        let mut batches = 0usize;
        let mut quarantined = 0u64;
        let imported_at_ms = now_ms();

        for chunk in records.chunks(commit_interval) {
            for input in chunk {
                let node_id = memory_node_id(&input.namespace, &input.key);
                let mut node = UnifiedNode::new(node_id);
                // Reserved fields (MCP-28): mirror `memory_record_to_node_owned`
                // so bulk-imported records are addressable via get/list/delete.
                // Without these, `record_from_node` returns None and the
                // record is invisible to the memory API.
                node.set_field(FIELD_NAMESPACE, FieldValue::String(input.namespace.clone()));
                node.set_field(FIELD_KEY, FieldValue::String(input.key.clone()));
                node.set_field(FIELD_PAYLOAD, FieldValue::String(input.payload.clone()));
                node.set_field(FIELD_CREATED_AT_MS, FieldValue::Int(imported_at_ms as i64));
                node.set_field(FIELD_UPDATED_AT_MS, FieldValue::Int(imported_at_ms as i64));
                node.set_field(FIELD_VERSION, FieldValue::Int(1));

                // v2 projection (ADR-046 §D2/§D6): the raw bulk path bypasses
                // the validated put path, so declared `derived` records (which
                // need parent lookups + score derivation) are rejected
                // explicitly instead of being silently downgraded.
                if input.confidence_class == Some(ConfidenceClass::Derived)
                    || input
                        .derived_from
                        .as_ref()
                        .is_some_and(|parents| !parents.is_empty())
                {
                    return Err(Error::Validation {
                        field: "confidence_class".into(),
                        reason: "derived records are not supported by bulk import; use put_batch"
                            .into(),
                    });
                }
                if let Some(value) = input.confidence {
                    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                        return Err(Error::Validation {
                            field: "confidence".into(),
                            reason: "confidence must be a finite number in [0,1]".into(),
                        });
                    }
                }
                // Boundary alignment with the validated put path (N2): `Some(0)`
                // is rejected there (`valid_at_ms` must be > 0; omit for
                // default); the raw bulk path must not silently reinterpret it.
                if input.valid_at_ms == Some(0) {
                    return Err(Error::Validation {
                        field: "valid_at_ms".into(),
                        reason: "must be greater than 0; omit the field to default to the import timestamp"
                            .into(),
                    });
                }
                let valid_at_ms = input.valid_at_ms.unwrap_or(imported_at_ms);
                node.set_field(FIELD_VALID_AT_MS, FieldValue::Int(valid_at_ms as i64));
                node.set_field(
                    FIELD_CONFIDENCE_CLASS,
                    FieldValue::String(ConfidenceClass::Asserted.as_wire_str().to_string()),
                );
                node.confidence_score = input.confidence.unwrap_or_else(default_confidence);
                // T1/F4 (ADR-046 §D5, SCH-05 review): the raw bulk path honors
                // the write-time flag AND preserves an existing quarantine when
                // the incoming record carries none (sticky, I2 — only T2/T4
                // leave quarantine). The metadata-only point read per unflagged
                // record is the price of closing the bypass;
                // ponytail: batch the existence checks if bulk-heavy workloads
                // ever show this read in a profile.
                if input.quarantine {
                    node.set_field(
                        FIELD_QUARANTINED_AT_MS,
                        FieldValue::Int(imported_at_ms as i64),
                    );
                    node.set_field(
                        FIELD_QUARANTINE_REASON,
                        FieldValue::String("explicit_write".to_string()),
                    );
                    node.set_field(
                        FIELD_QUARANTINED_BY,
                        FieldValue::String("system:bulk_import".to_string()),
                    );
                    if let Some(due) = self.quarantine_review_due_ms(imported_at_ms) {
                        node.set_field(FIELD_QUARANTINE_REVIEW_DUE_MS, FieldValue::Int(due as i64));
                    }
                    quarantined += 1;
                } else if let Some((at, reason, by, due)) =
                    Self::existing_quarantine_fields(&engine, node_id)?
                {
                    node.set_field(FIELD_QUARANTINED_AT_MS, FieldValue::Int(at));
                    if let Some(reason) = reason {
                        node.set_field(FIELD_QUARANTINE_REASON, FieldValue::String(reason));
                    }
                    if let Some(by) = by {
                        node.set_field(FIELD_QUARANTINED_BY, FieldValue::String(by));
                    }
                    if let Some(due) = due {
                        node.set_field(FIELD_QUARANTINE_REVIEW_DUE_MS, FieldValue::Int(due));
                    }
                }
                if let Some(ref v) = input.vector {
                    node.vector = VectorRepresentations::Full(v.clone());
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }
                for (k, v) in &input.metadata {
                    let fv = match v {
                        Value::String(s) => FieldValue::String(s.clone()),
                        Value::Int(i) => FieldValue::Int(*i),
                        Value::Float(f) => FieldValue::Float(*f),
                        Value::Bool(b) => FieldValue::Bool(*b),
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
        // SCH-05 review F3: import ops are audited (the bulk path was silent).
        self.audit(crate::audit::AuditEvent::new(
            "bulk_import",
            "N/A",
            "N/A",
            "ok",
            (quarantined > 0).then(|| format!("{quarantined} quarantined")),
        ));
        Ok(BulkImportReport {
            total_records: total,
            batches_committed: batches,
            quarantined,
            duration_ms,
        })
    }

    /// Convenience: bulk-import from a binary file in bulk format.
    ///
    /// The path goes through the same export-base sandbox as the other
    /// file import/export ops (WIRE-09) — the HTTP server passes a
    /// user-supplied path here (`import_v2` with `format: "bulk"`).
    pub fn bulk_import_file(&self, path: &str) -> Result<BulkImportReport> {
        let resolved = self.resolve_export_path(std::path::Path::new(path))?;
        let mut file = std::fs::File::open(&resolved)?;
        self.bulk_import_stream(&mut file)
    }
}
