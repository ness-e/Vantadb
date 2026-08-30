//! Graph-node and edge operations on `VantaEmbedded`.
//!
//! Owns direct node CRUD (`insert_node`, `get_node`, `delete_node`), edge
//! operations (`add_edge`, `remove_edge`), IQL execution (`query`), snapshot
//! export/restore (`collect_graph_nodes`, `restore_graph_nodes`), and the
//! segment-optimizer entry points (`vacuum`, `pipeline`, `optimizer_config`,
//! `set_optimizer_config` — colocated here because they share the engine
//! handle with node operations and are rarely used outside the graph path).
//!
//! Extracted from `sdk::api` (REVIEW-12, 2026-08-30).

use super::super::builder::VantaEmbedded;
use super::super::types::*;
use crate::backend::BackendPartition;
use crate::error::{Result, VantaError};
use crate::executor::Executor;
use crate::node::{FieldValue, UnifiedNode, VectorRepresentations};
use crate::sdk::serialization::now_ms;

impl VantaEmbedded {
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

    /// Collect every live non-memory-record node as an SDK record (CORE-02).
    ///
    /// Nodes that carry [`FIELD_NAMESPACE`] belong to the memory-record layer
    /// and are exported by the memory snapshot (`collect_all_deduped`); every
    /// other live node is a graph node (created via `insert_node`, `add_edge`
    /// or IQL INSERT/RELATE). The WASM binding persists these alongside
    /// `db_state.json` so the graph store survives an OPFS/IDB reopen.
    pub fn collect_graph_nodes(&self) -> Result<Vec<VantaNodeRecord>> {
        let engine = self.engine_handle()?;
        let ids: Vec<u128> = engine
            .backend
            .scan(BackendPartition::Default)?
            .iter()
            .filter_map(|(key_bytes, _)| {
                let arr: [u8; 16] = key_bytes.as_slice().try_into().ok()?;
                Some(u128::from_le_bytes(arr))
            })
            .collect();
        let mut out = Vec::new();
        for mut node in engine.get_many(&ids)? {
            if engine.is_deleted(node.id)? {
                continue;
            }
            if node
                .relational
                .contains_key(super::super::serialization::FIELD_NAMESPACE)
            {
                continue; // memory record — owned by the memory snapshot
            }
            out.push(engine.node_to_record(std::mem::take(&mut node)));
        }
        Ok(out)
    }

    /// Restore graph nodes previously exported by [`VantaEmbedded::collect_graph_nodes`]
    /// (CORE-02). Edge labels are re-interned into the fresh engine's label
    /// table; weights, direction (`reverse`) and creation timestamps are
    /// preserved so traversal behaves identically after restore. Returns the
    /// number of nodes restored.
    pub fn restore_graph_nodes(&self, records: Vec<VantaNodeRecord>) -> Result<usize> {
        self.check_read_only()?;
        crate::metrics::record_graph_op("restore_graph_nodes");
        let engine = self.engine_handle()?;
        let mut restored = 0usize;
        for record in records {
            let mut node = UnifiedNode::new(record.id);
            for (key, value) in record.fields {
                node.set_field(&key, value.into());
            }
            if let Some(vector) = record.vector.filter(|v| Self::usable_vector(v)) {
                node.vector = VectorRepresentations::Full(vector);
                node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
            }
            for edge in record.edges {
                let label_id = engine.intern_label(&edge.label);
                node.edges.push(crate::node::Edge {
                    target: edge.target,
                    label_id,
                    weight: edge.weight,
                    reverse: edge.reverse,
                    created_at_ms: edge.created_at_ms,
                });
            }
            node.tier = match record.tier {
                VantaStorageTier::Hot => crate::node::NodeTier::Hot,
                VantaStorageTier::Cold => crate::node::NodeTier::Cold,
            };
            node.confidence_score = record.confidence_score;
            node.importance = record.importance;
            node.hits = record.hits;
            node.last_accessed = record.last_accessed;
            node.epoch = record.epoch;
            engine.insert(&node)?;
            restored += 1;
        }
        Ok(restored)
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
}
