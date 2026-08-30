//! Namespace listing, counting, deletion, and per-namespace statistics.
//!
//! Owns the namespace-level surface: `list_namespaces`, `list`, `count`,
//! `delete_by_filter`, and `namespace_stats`. Cursor-based pagination and
//! record filtering are shared with the memory module via
//! `VantaMemoryListOptions`.
//!
//! Extracted from `sdk::api` (REVIEW-12, 2026-08-30).

use super::super::builder::VantaEmbedded;
use super::super::serialization::{
    is_scalar_indexable, matches_memory_filters, memory_record_from_node_include_expired, now_ms,
    validate_namespace,
};
use super::super::types::*;
use crate::backend::BackendPartition;
use crate::error::Result;
use std::collections::BTreeSet;

impl VantaEmbedded {
    /// List all namespaces that contain at least one memory record.
    #[tracing::instrument(skip(self), err)]
    pub fn list_namespaces(&self) -> Result<Vec<String>> {
        let engine = self.engine_handle()?;
        let mut namespaces = std::collections::BTreeSet::new();
        let entries = engine.scan_partition(BackendPartition::NamespaceIndex)?;

        if entries.is_empty() {
            for node in engine.scan_nodes()? {
                if let Some(record) = super::super::serialization::memory_record_from_node(&node) {
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
        super::super::serialization::validate_metadata(&options.filters)?;

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
            if let Some(record) = super::super::serialization::memory_record_from_node(&node) {
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
                if let Some(record) = super::super::serialization::memory_record_from_node(&node) {
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

        // ADR-028: drop superseded records at final assembly. Pagination may
        // yield a page with fewer than `limit` records when the flag is set —
        // same guarantee as the post-filter path (a non-full page is last).
        if options.exclude_superseded {
            records.retain(|record| record.superseded_by.is_none());
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
                    exclude_superseded: false,
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
                    exclude_superseded: false,
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

    /// Compute per-namespace statistics: total record count, records expiring
    /// within the given window, and already-expired records.
    ///
    /// Performs a **single pass** over all memory records (one full scan), so
    /// callers summarizing every namespace never need N paginated `count`/
    /// `list` calls. Expired records are counted only as `expired`, never as
    /// `expiring_soon`.
    ///
    /// Semantics: `count` includes not-yet-purged expired records (records
    /// hidden by lazy TTL eviction in [`memory_record_from_node`]); use
    /// [`Self::count`] / [`Self::list`] for the read-visible subset. Records
    /// with no TTL count only toward `count`.
    ///
    /// # Arguments
    /// * `expiring_soon_window_ms` — How far into the future a TTL counts as
    ///   "expiring soon". `None` uses [`DEFAULT_EXPIRING_SOON_WINDOW_MS`]
    ///   (24 hours).
    ///
    /// # Example
    /// ```
    /// use vantadb::{VantaEmbedded, VantaMemoryInput};
    ///
    /// let dir = std::env::temp_dir().join(format!(
    ///     "vantadb-ns-stats-doctest-{}",
    ///     std::process::id()
    /// ));
    /// let db = VantaEmbedded::open(&dir).unwrap();
    /// db.put(VantaMemoryInput::new("agent", "k", "payload")).unwrap();
    ///
    /// let stats = db.namespace_stats(None).unwrap();
    /// assert_eq!(stats["agent"].count, 1);
    /// let _ = std::fs::remove_dir_all(&dir);
    /// ```
    #[tracing::instrument(skip(self), err)]
    pub fn namespace_stats(
        &self,
        expiring_soon_window_ms: Option<u64>,
    ) -> Result<VantaNamespaceStatsMap> {
        let engine = self.engine_handle()?;
        let now = now_ms();
        let window = expiring_soon_window_ms.unwrap_or(DEFAULT_EXPIRING_SOON_WINDOW_MS);
        let mut stats: VantaNamespaceStatsMap = std::collections::BTreeMap::new();

        for node in engine.scan_nodes()? {
            // Include expired (not-yet-purged) records so `expired` is observable;
            // the read-path variant would hide them via lazy TTL eviction.
            let Some(record) = memory_record_from_node_include_expired(&node) else {
                continue;
            };
            let entry = stats.entry(record.namespace).or_default();
            entry.count += 1;
            if let Some(expires_at) = record.expires_at_ms {
                if expires_at <= now {
                    entry.expired += 1;
                } else if expires_at <= now.saturating_add(window) {
                    entry.expiring_soon += 1;
                }
            }
        }

        Ok(stats)
    }
}
