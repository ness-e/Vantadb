//! Statistics, health checks, and backend capability queries.

use std::collections::HashMap;

use crate::backend::{BackendPartition, StorageBackend};
use crate::config::VantaConfig;
use crate::error::{Result, VantaError};
use crate::node::FieldValue;
use crate::query::RelOp;
use crate::storage::engine::{
    engine_mmap_resident_bytes, EvictionReason, MemoryStats, StorageEngine,
};
use crate::storage::ops::NodeMetadata;

impl StorageEngine {
    /// Check that the engine is not read-only.
    #[inline]
    pub fn guard_write_allowed(config: &VantaConfig) -> Result<()> {
        if config.read_only {
            return Err(VantaError::ValidationError {
                field: "read_only".into(),
                reason: "StorageEngine is read-only; write operation rejected".into(),
            });
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn ensure_writable(&self) -> Result<()> {
        Self::guard_write_allowed(&self.config)
    }

    /// Update the last-query timestamp to the current system time.
    pub fn touch_activity(&self) {
        let now = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        self.last_query_timestamp
            .store(now, std::sync::atomic::Ordering::Release);
    }

    /// Returns the advanced tokenizer configuration if available.
    #[cfg(feature = "advanced-tokenizer")]
    pub fn advanced_tokenizer_config(&self) -> Option<&crate::tokenizer::AdvancedTokenizerConfig> {
        self.config.advanced_tokenizer_config.as_ref()
    }

    /// Returns `None` when the advanced tokenizer feature is disabled.
    #[cfg(not(feature = "advanced-tokenizer"))]
    pub fn advanced_tokenizer_config(&self) -> Option<()> {
        None
    }

    /// Returns detailed memory usage statistics for this engine instance.
    pub fn get_memory_stats(&self) -> MemoryStats {
        let hnsw = self.hnsw.load();
        let vector_store = self.vector_store.read();
        let cache = self.volatile_cache.read();

        let logical =
            hnsw.estimate_memory_bytes() as u64 + vector_store.size + (cache.len() as u64 * 1536);

        let physical = engine_mmap_resident_bytes(&hnsw, &vector_store);

        let memory_limit = self
            .config
            .memory_limit
            .unwrap_or_else(|| crate::hardware::HardwareCapabilities::global().total_memory);

        let snap = crate::metrics::operational_metrics_snapshot();

        MemoryStats {
            logical_bytes: logical,
            physical_rss: physical,
            node_count: hnsw.nodes.len() as u64,
            cache_entries: cache.len(),
            eviction_count: snap.evictions_total,
            eviction_bytes: snap.eviction_bytes_total,
            memory_limit,
            quantized_nodes: snap.current_quantized_nodes,
        }
    }

    /// Check current memory usage against the RSS threshold and trigger eviction if exceeded.
    pub fn check_memory_pressure(&self) -> Result<()> {
        let threshold = self.config.rss_threshold;
        if threshold <= 0.0 {
            return Ok(());
        }
        let stats = self.get_memory_stats();
        let effective = stats.effective_bytes();
        if effective == 0 {
            return Ok(());
        }
        let limit = self
            .config
            .memory_limit
            .unwrap_or_else(|| crate::hardware::HardwareCapabilities::global().total_memory);

        // PERF-10: Check MemoryGovernor watermarks
        let above_high_water = self
            .memory_governor
            .as_ref()
            .map(|g| g.should_evict())
            .unwrap_or(false);
        if (effective as f64) > (limit as f64 * threshold) || above_high_water {
            let reason = if self
                .memory_governor
                .as_ref()
                .map(|g| g.needs_urgent_eviction())
                .unwrap_or(false)
            {
                EvictionReason::Oom
            } else {
                EvictionReason::Watermark
            };
            tracing::warn!(
                effective_bytes = effective,
                threshold_pct = (threshold * 100.0) as u64,
                ?reason,
                "Memory pressure detected — triggering auto-eviction",
            );
            if let Err(e) = self.evict_cold_nodes_with_reason(self.config.eviction_ratio, reason) {
                tracing::warn!("eviction failed: {e}");
            }
            return Err(VantaError::ResourceLimit(format!(
                "Memory pressure: {} bytes used ({}% of {} limit, threshold {}%)",
                effective,
                (effective as f64 / limit as f64 * 100.0) as u64,
                limit,
                (threshold * 100.0) as u64,
            )));
        }

        // PERF-10: Periodic MemoryGovernor sync for used_bytes
        if let Some(ref gov) = self.memory_governor {
            gov.set_used_bytes(effective);
            if gov.should_evict() && gov.try_start_eviction() {
                let reason = if gov.needs_urgent_eviction() {
                    EvictionReason::Oom
                } else {
                    EvictionReason::Watermark
                };
                if let Err(e) =
                    self.evict_cold_nodes_with_reason(self.config.eviction_ratio, reason)
                {
                    tracing::warn!("memory-governor eviction failed: {e}");
                }
                gov.finish_eviction();
            }
        }

        Ok(())
    }

    /// Perform an emergency shutdown: flush buffers and exit the process immediately.
    pub fn emergency_shutdown(&self, reason: &str, stmt: Option<&str>) -> ! {
        println!("\n=======================================================");
        println!("[!] VANTADB SYSTEM EMERGENCY: Security Constraint Violated");
        println!("=======================================================");
        tracing::error!("Emergency shutdown reason: {}", reason);
        if let Some(s) = stmt {
            tracing::error!("Offending Transaction: {}", s);
        }

        println!("Attempting controlled flush...");
        if let Err(e) = self.flush() {
            tracing::error!("Failed to flush buffers during shutdown: {}", e);
        } else {
            println!("Buffers flushed successfully.");
        }
        std::process::exit(1);
    }

    pub(crate) fn initialize_cardinality_stats(
        backend: &dyn StorageBackend,
    ) -> HashMap<String, HashMap<String, usize>> {
        let mut stats: HashMap<String, HashMap<String, usize>> = HashMap::new();
        if let Ok(records) = backend.scan(BackendPartition::Default) {
            for (_key, val) in records {
                if let Ok(metadata) = postcard::from_bytes::<NodeMetadata>(&val) {
                    for (field, value) in metadata.relational {
                        let val_keys = value.to_cardinality_keys();
                        let val_map = stats.entry(field).or_default();
                        for val_key in val_keys {
                            if val_map.len() < 100 || val_map.contains_key(&val_key) {
                                *val_map.entry(val_key).or_default() += 1;
                            }
                        }
                    }
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
        stats
    }

    /// Estimate the selectivity of a relational filter based on cached cardinality statistics.
    pub fn get_estimated_selectivity(&self, field: &str, op: &RelOp, value: &FieldValue) -> f32 {
        let stats = self.cardinality_stats.read();
        let total_nodes = self.hnsw.load().nodes.len();
        if total_nodes == 0 {
            let val_keys = value.to_cardinality_keys();
            let val_key = val_keys
                .first()
                .cloned()
                .unwrap_or_else(|| "null".to_string());
            if let Some(val_map) = stats.get(field) {
                let freq = *val_map.get(&val_key).unwrap_or(&0);
                return match op {
                    RelOp::Eq => {
                        if freq > 0 {
                            1.0
                        } else {
                            0.0
                        }
                    }
                    RelOp::Neq => {
                        if freq > 0 {
                            0.0
                        } else {
                            1.0
                        }
                    }
                    _ => 0.5,
                };
            }
            return 1.0;
        }

        let val_keys = value.to_cardinality_keys();
        let val_key = val_keys
            .first()
            .cloned()
            .unwrap_or_else(|| "null".to_string());

        if let Some(val_map) = stats.get(field) {
            let freq = *val_map.get(&val_key).unwrap_or(&0) as f32;

            match op {
                RelOp::Eq => {
                    if freq > 0.0 {
                        freq / total_nodes as f32
                    } else if val_map.len() >= 100 {
                        1.0 / total_nodes.max(1) as f32
                    } else {
                        0.0
                    }
                }
                RelOp::Neq => {
                    let eq_sel = if freq > 0.0 {
                        freq / total_nodes as f32
                    } else if val_map.len() >= 100 {
                        1.0 / total_nodes.max(1) as f32
                    } else {
                        0.0
                    };
                    1.0 - eq_sel
                }
                RelOp::Gt | RelOp::Gte | RelOp::Lt | RelOp::Lte => 0.33,
            }
        } else {
            match op {
                RelOp::Eq => 0.0,
                RelOp::Neq => 1.0,
                _ => 0.5,
            }
        }
    }

    /// Request backend compaction.
    pub fn request_compaction(&self) {
        if !self.supports_manual_compaction() {
            tracing::info!(
                "Maintenance requested manual disk compaction, but it was skipped. \
                The active backend ({:?}) manages compaction automatically. This is expected behavior.",
                self.backend_kind()
            );
            return;
        }
        self.backend.compact();
    }

    /// Return the capabilities descriptor of the active KV backend.
    pub fn backend_capabilities(&self) -> crate::backend::BackendCapabilities {
        self.backend.capabilities()
    }

    /// Return the kind of the active KV backend (InMemory, RocksDb, or Fjall).
    pub fn backend_kind(&self) -> crate::backend::BackendKind {
        self.backend.capabilities().kind
    }

    /// Return whether the active backend supports point-in-time checkpoints.
    pub fn supports_checkpoint(&self) -> bool {
        self.backend.capabilities().supports_checkpoint
    }

    /// Return whether the active backend supports explicitly triggered compaction.
    pub fn supports_manual_compaction(&self) -> bool {
        self.backend.capabilities().supports_manual_compaction
    }
}
