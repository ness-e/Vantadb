//! Engine administration and runtime introspection.
//!
//! Owns index rebuild (`rebuild_index`, `reindex_hnsw_from_text`), layout and
//! WAL compaction (`compact_layout`, `compact_wal`), durability checkpoints
//! (`flush`), and capability introspection (`capabilities`,
//! `operational_metrics`).
//!
//! Extracted from `sdk::api` (REVIEW-12, 2026-08-30).

use super::super::builder::VantaEmbedded;
use super::super::serialization::validate_namespace;
use super::super::types::*;
use crate::error::Result;
use web_time::Instant;

impl VantaEmbedded {
    /// Rebuild the HNSW vector index, derived indexes, and text index from scratch.
    #[tracing::instrument(skip(self), err)]
    pub fn rebuild_index(&self) -> Result<VantaIndexRebuildReport> {
        self.check_read_only()?;
        let engine = self.engine_handle()?;
        let report = engine.rebuild_vector_index()?;
        let derived = self.rebuild_derived_indexes_with_report()?;
        self.rebuild_text_index_with_report()?;
        self.rebuild_sparse_index_with_report()?;
        // MOD-04: scalar index (TTL purge candidates) is derived from backend
        // metadata — rebuild it alongside the other derived indexes so a
        // repaired DB serves `purge_expired` correctly.
        engine.rebuild_scalar_index()?;
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
                    exclude_superseded: false,
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
}
