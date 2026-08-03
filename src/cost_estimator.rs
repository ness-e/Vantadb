//! Unified semantic cost estimation for query plans (COMP-028).
//!
//! Consolidates the cost/selectivity estimation logic that was previously
//! scattered across `governor.rs`, `planner.rs`, and `sdk/search/mod.rs` into a
//! single module. All estimates are derived from data already in memory
//! (`cardinality_stats`, `hnsw.nodes.len()`) — never from scans or index walks.
//!
//! Public API surface is unchanged: [`StorageEngine::get_estimated_selectivity`]
//! keeps its signature and delegates here (see `src/storage/engine/stats.rs`).
//! `FilterStrategy` moved from `sdk/search/mod.rs` as a `pub(crate)` type and is
//! not re-exported through any public API.

use crate::index::IndexType;
use crate::node::FieldValue;
use crate::query::{LogicalOperator, LogicalPlan, RelOp};
use crate::sdk::types::VantaMemoryMetadata;
use crate::storage::StorageEngine;

/// Selectivity threshold below which **PreFilter** is chosen:
/// scan metadata → build bitset → brute-force vector search on the small subset.
/// Filters with selectivity below this value match < 1 % of rows.
const PREFILTER_THRESHOLD: f32 = 0.01;

/// Average estimated size of a materialized node row, in bytes.
///
/// Heuristic used to convert estimated row counts into byte costs. Not derived
/// from live data (no scans allowed) — a constant floor keeps estimates stable.
const AVG_NODE_BYTES: usize = 1024;

/// Default embedding dimension used for vector search byte estimates.
const DEFAULT_EMBEDDING_DIMS: usize = 128;

/// Node-count threshold above which IVF lazy-build is preferred over HNSW.
///
/// Below this, HNSW graph traversal is the cheapest correct path; at scale the
/// inverted-file (clustering) index wins on latency. Only consulted when the
/// engine has no explicit non-HNSW `index_type` configured.
const IVF_NODE_THRESHOLD: usize = 10_000;

/// Estimated cost of a single logical operator.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OperatorCost {
    /// Estimated rows produced by this operator.
    pub estimated_rows: f64,
    /// Estimated bytes materialized by this operator.
    pub estimated_bytes: usize,
}

/// Estimated cost of a full logical plan.
#[derive(Debug, Clone, Copy)]
// COMP-028: estimated_bytes feeds the OLD-21 admission guard in executor.rs;
// estimated_rows is read only by unit tests (kept for future cardinality use).
pub(crate) struct PlanCost {
    /// Estimated rows produced by the final operator.
    #[allow(dead_code)] // read only by CostEstimator unit tests today.
    pub estimated_rows: f64,
    /// Estimated peak bytes (largest intermediate materialization).
    pub estimated_bytes: usize,
}

/// 3 filtering strategies ordered by increasing selectivity.
///
/// The optimizer picks one based on the estimated joint selectivity of all
/// query filters — how many rows survive the filter before vector search.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum FilterStrategy {
    /// Highly selective (joint_sel < 1 %): pre-filter metadata first, then
    /// vector-search only the matching records (brute-force on a tiny set).
    PreFilter,
    /// Moderately selective (1 % ≤ joint_sel < 10 %): build a bitset of
    /// matching node IDs and pass it as `query_mask` during HNSW traversal
    /// so the graph walk only visits candidates that survive the filter.
    InFilter,
    /// Low selectivity (joint_sel ≥ 10 %): let HNSW see everything, then
    /// post-filter results. Current default.
    PostFilter,
}

/// Semantic cost estimator over live engine statistics (COMP-028).
///
/// All estimates are derived from data that is already in memory
/// (`cardinality_stats`, `hnsw.nodes.len()`) — no scans, no index walks.
pub(crate) struct CostEstimator<'a> {
    storage: &'a StorageEngine,
}

impl<'a> CostEstimator<'a> {
    /// Create an estimator over the given engine.
    pub(crate) fn new(storage: &'a StorageEngine) -> Self {
        Self { storage }
    }

    /// Estimate the selectivity of a relational filter based on cached
    /// cardinality statistics.
    ///
    /// COMP-028: implementation of `StorageEngine::get_estimated_selectivity`
    /// (src/storage/engine/stats.rs), which delegates here. Logic is a 1:1 move
    /// — the heuristic itself is unchanged.
    pub(crate) fn selectivity(&self, field: &str, op: &RelOp, value: &FieldValue) -> f32 {
        let stats = self.storage.cardinality_stats.read();
        let total_nodes = self.storage.hnsw.load().nodes.len();
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

    /// Estimate the joint selectivity of all query filters against the
    /// engine's cardinality statistics, then pick the best filtering strategy.
    ///
    /// COMP-028: moved from `sdk/search/mod.rs` (logic unchanged).
    pub(crate) fn select_filter_strategy(&self, filters: &VantaMemoryMetadata) -> FilterStrategy {
        if filters.is_empty() {
            return FilterStrategy::PostFilter;
        }

        let mut joint_selectivity = 1.0f32;
        for (field, value) in filters.iter() {
            let fv = FieldValue::from(value.clone());
            let sel = self.selectivity(field, &RelOp::Eq, &fv);
            joint_selectivity *= sel;
        }

        if joint_selectivity < PREFILTER_THRESHOLD {
            FilterStrategy::PreFilter
        } else if joint_selectivity < crate::planner::HIGH_SELECTIVITY_THRESHOLD {
            FilterStrategy::InFilter
        } else {
            FilterStrategy::PostFilter
        }
    }

    /// Estimate the cost of a single operator given the incoming row count.
    ///
    /// Rows flow top-down through the operator chain; `in_rows` is the output
    /// row estimate of the previous operator (0 for the first operator).
    fn estimate_operator(&self, op: &LogicalOperator, in_rows: f64) -> OperatorCost {
        match op {
            LogicalOperator::Scan { .. } => {
                let rows = self.storage.hnsw.load().nodes.len() as f64;
                // Floor at 1 row so a scan always yields a positive byte cost.
                let bytes = (rows.max(1.0) * AVG_NODE_BYTES as f64) as usize;
                OperatorCost {
                    estimated_rows: rows,
                    estimated_bytes: bytes,
                }
            }
            LogicalOperator::FilterRelational {
                field,
                op: rel_op,
                value,
            } => {
                let sel = self.selectivity(field, rel_op, value) as f64;
                let rows = in_rows * sel;
                OperatorCost {
                    estimated_rows: rows,
                    estimated_bytes: (rows * AVG_NODE_BYTES as f64) as usize,
                }
            }
            LogicalOperator::VectorSearch { .. } => {
                // No top-k on the operator itself (Limit follows); assume a
                // default candidate pool capped by the index size.
                let rows = (self.storage.hnsw.load().nodes.len() as f64).min(100.0);
                let bytes = (rows * DEFAULT_EMBEDDING_DIMS as f64 * 4.0) as usize;
                OperatorCost {
                    estimated_rows: rows,
                    estimated_bytes: bytes,
                }
            }
            LogicalOperator::Limit { top_k } => {
                let rows = in_rows.min(*top_k as f64);
                OperatorCost {
                    estimated_rows: rows,
                    estimated_bytes: (rows * AVG_NODE_BYTES as f64) as usize,
                }
            }
            LogicalOperator::Traverse { max_depth, .. } => {
                // BFS fanout heuristic: each level expands by ~max_depth edges.
                let rows = in_rows * (*max_depth as f64).powi(2);
                OperatorCost {
                    estimated_rows: rows,
                    estimated_bytes: (rows * AVG_NODE_BYTES as f64) as usize,
                }
            }
            // Sort/Project pass rows through; Join/SubqueryFilter are estimated
            // as pass-through of the current row count (sub-plan costs are not
            // materialized here — OLD-21 will refine this).
            LogicalOperator::Sort { .. }
            | LogicalOperator::Project { .. }
            | LogicalOperator::Join { .. }
            | LogicalOperator::SubqueryFilter { .. } => OperatorCost {
                estimated_rows: in_rows,
                estimated_bytes: (in_rows * AVG_NODE_BYTES as f64) as usize,
            },
        }
    }

    /// Choose which vector index backend to route a search through (OLD-21).
    ///
    /// Heuristic, not an optimizer:
    /// - An explicitly configured non-HNSW `index_type` (Ivf/Flat/DiskAnn/…) is
    ///   always honored — a user who asked for IVF gets IVF.
    /// - Otherwise: Flat when `nodes <= flat_threshold` (matches
    ///   [`CPIndex::use_flat_search`](crate::index::graph::CPIndex)), IVF once
    ///   the dataset is large enough to amortize clustering, HNSW in between.
    ///
    /// The engine's [`search_nearest`](crate::index::search) performs the actual
    /// routing identically (flat_threshold + config.index_type); this is the
    /// single authority for the decision so callers can record/EXPLAIN it.
    pub(crate) fn select_index_strategy(&self) -> IndexType {
        let index = self.storage.hnsw.load();
        if index.config.index_type != IndexType::Hnsw {
            return index.config.index_type;
        }
        let nodes = index.nodes.len();
        if index.config.flat_threshold.is_some_and(|t| nodes <= t) {
            return IndexType::Flat;
        }
        if nodes >= IVF_NODE_THRESHOLD {
            return IndexType::Ivf;
        }
        IndexType::Hnsw
    }

    /// Estimate the cost of a full logical plan by chaining operator estimates
    /// in plan order. Rows flow between operators; bytes are accounted by the
    /// peak (largest intermediate materialization) operator.
    pub(crate) fn estimate_plan(&self, plan: &LogicalPlan) -> PlanCost {
        let mut rows = 0.0f64;
        let mut peak_bytes = 0usize;
        for op in &plan.operators {
            let oc = self.estimate_operator(op, rows);
            rows = oc.estimated_rows;
            peak_bytes = peak_bytes.max(oc.estimated_bytes);
        }
        PlanCost {
            estimated_rows: rows,
            estimated_bytes: peak_bytes,
        }
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::backend::BackendKind;
    use crate::config::VantaConfig;
    use crate::index::graph::CPIndex;
    use crate::node::{UnifiedNode, VectorRepresentations};
    use crate::sdk::types::VantaValue;

    /// Open an empty in-memory engine for estimation tests.
    fn in_memory_engine() -> StorageEngine {
        let config = VantaConfig {
            backend_kind: BackendKind::InMemory,
            ..VantaConfig::default()
        };
        StorageEngine::open_with_config(":memory:", Some(config)).expect("open in-memory engine")
    }

    fn node_with_color(id: u128, color: &str) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.vector = VectorRepresentations::Full(vec![0.1, 0.2, 0.3]);
        node.relational
            .insert("color".into(), FieldValue::String(color.into()));
        node
    }

    #[test]
    fn test_estimate_plan_scan_bytes_positive() {
        let engine = in_memory_engine();
        engine.insert(&node_with_color(1, "red")).expect("insert");
        let plan = LogicalPlan {
            operators: vec![LogicalOperator::Scan {
                entity: "test".into(),
            }],
            temperature: 0.0,
            enforce_role: None,
        };
        let cost = CostEstimator::new(&engine).estimate_plan(&plan);
        assert!(
            cost.estimated_bytes > 0,
            "plan with >= 1 operator must have positive bytes, got {}",
            cost.estimated_bytes
        );
        assert_eq!(cost.estimated_rows, 1.0);
    }

    #[test]
    fn test_selectivity_eq_freq_over_total() {
        let engine = in_memory_engine();
        // 10 nodes, 3 with color=red → selectivity 0.3.
        for i in 0..10 {
            let color = if i < 3 { "red" } else { "blue" };
            engine.insert(&node_with_color(i, color)).expect("insert");
        }
        let est = CostEstimator::new(&engine);
        let sel = est.selectivity("color", &RelOp::Eq, &FieldValue::String("red".into()));
        assert!((sel - 0.3).abs() < 1e-6, "expected 3/10 = 0.3, got {sel}");
    }

    #[test]
    fn test_select_filter_strategy_bands() {
        let engine = in_memory_engine();
        // 1000 nodes: 1 rare → sel 0.001 → PreFilter; 50 common → 0.05 → InFilter;
        // 949 bulk → 0.949 → PostFilter.
        for i in 0..1000 {
            let color = match i {
                0 => "rare",
                1..=50 => "common",
                _ => "bulk",
            };
            engine.insert(&node_with_color(i, color)).expect("insert");
        }
        let est = CostEstimator::new(&engine);

        // joint_sel < 0.01 → PreFilter
        let mut filters = VantaMemoryMetadata::new();
        filters.insert("color".into(), VantaValue::String("rare".into()));
        assert_eq!(
            est.select_filter_strategy(&filters),
            FilterStrategy::PreFilter
        );

        // 0.01 ≤ joint_sel < 0.10 → InFilter
        let mut filters = VantaMemoryMetadata::new();
        filters.insert("color".into(), VantaValue::String("common".into()));
        assert_eq!(
            est.select_filter_strategy(&filters),
            FilterStrategy::InFilter
        );

        // joint_sel ≥ 0.10 → PostFilter
        let mut filters = VantaMemoryMetadata::new();
        filters.insert("color".into(), VantaValue::String("bulk".into()));
        assert_eq!(
            est.select_filter_strategy(&filters),
            FilterStrategy::PostFilter
        );

        // empty filters → PostFilter
        let empty = VantaMemoryMetadata::new();
        assert_eq!(
            est.select_filter_strategy(&empty),
            FilterStrategy::PostFilter
        );
    }

    #[test]
    fn test_estimate_operator_limit_trims_rows() {
        let engine = in_memory_engine();
        let est = CostEstimator::new(&engine);
        let cost = est.estimate_operator(&LogicalOperator::Limit { top_k: 5 }, 100.0);
        assert_eq!(cost.estimated_rows, 5.0, "Limit trims rows to top_k");
    }

    /// Build a CPIndex with `n` nodes (level-0 only, fast for tests) using `cfg`.
    fn index_with_n(n: usize, mut cfg: crate::index::graph::HnswConfig) -> std::sync::Arc<CPIndex> {
        cfg.m = 8;
        cfg.m_max0 = 8;
        cfg.ef_construction = 4;
        cfg.ef_search = 8;
        cfg.ml = 1.0 / (8_f64).ln();
        let idx = CPIndex::new_with_config(cfg);
        for i in 0..n as u128 {
            idx.add_with_level(
                i,
                crate::node::FilterBitset::new(),
                crate::node::VectorRepresentations::Full(vec![0.1, 0.2, 0.3]),
                0,
                0,
            );
        }
        std::sync::Arc::new(idx)
    }

    #[test]
    fn test_select_index_strategy_small_dataset_flat() {
        let engine = in_memory_engine();
        // 0 nodes <= flat_threshold (default 10_000) → Flat.
        assert_eq!(
            CostEstimator::new(&engine).select_index_strategy(),
            IndexType::Flat
        );
    }

    #[test]
    fn test_select_index_strategy_medium_dataset_defaults_to_hnsw() {
        let engine = in_memory_engine();
        // Flat disabled, 100 nodes < IVF_NODE_THRESHOLD → HNSW.
        let cfg = crate::index::graph::HnswConfig {
            flat_threshold: None,
            ..Default::default()
        };
        engine.hnsw.store(index_with_n(100, cfg));
        assert_eq!(
            CostEstimator::new(&engine).select_index_strategy(),
            IndexType::Hnsw
        );
    }

    #[test]
    fn test_select_index_strategy_large_dataset_ivf() {
        let engine = in_memory_engine();
        // 10_000 nodes, flat disabled → at/above IVF_NODE_THRESHOLD → IVF.
        let cfg = crate::index::graph::HnswConfig {
            flat_threshold: None,
            ..Default::default()
        };
        engine.hnsw.store(index_with_n(IVF_NODE_THRESHOLD, cfg));
        assert_eq!(
            CostEstimator::new(&engine).select_index_strategy(),
            IndexType::Ivf
        );
    }

    #[test]
    fn test_select_index_strategy_respects_explicit_config() {
        let engine = in_memory_engine();
        // Explicit IVF on a tiny dataset overrides the flat/hnsw heuristics.
        let cfg = crate::index::graph::HnswConfig {
            index_type: IndexType::Ivf,
            ..Default::default()
        };
        engine.hnsw.store(index_with_n(3, cfg));
        assert_eq!(
            CostEstimator::new(&engine).select_index_strategy(),
            IndexType::Ivf
        );
    }
}
