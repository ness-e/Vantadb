//! Query execution engine that translates logical plans into results.
//!
//! Evaluates [`LogicalOperator`] trees against the [`StorageEngine`],
//! returning materialized [`ExecutionResult`] variants.

use crate::error::{ChainedError, Error, Result};
use crate::node::{UnifiedNode, VectorRepresentations};
use crate::parser::parse_statement;
use crate::query::{
    DeleteStatement, InsertMessageStatement, InsertStatement, LogicalOperator, LogicalPlan, Query,
    RelateStatement, SelectStatement, Statement, UpdateStatement,
};
use crate::storage::StorageEngine;
use std::sync::atomic::{AtomicU32, Ordering};

const GIB: usize = 1024 * 1024 * 1024;

/// Result of executing a statement against the storage engine.
#[derive(Debug)]
pub enum ExecutionResult {
    /// Nodes returned from a read query.
    Read(Vec<UnifiedNode>),
    /// Result of a write operation.
    Write {
        /// Number of affected nodes.
        affected_nodes: usize,
        /// Human-readable status message.
        message: String,
        /// Optional primary node ID involved.
        node_id: Option<u128>,
    },
    /// Signal that a context requires rehydration (low confidence score).
    StaleContext(u128),
}

/// Search path mode for query execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchPathMode {
    /// Standard path execution.
    Standard,
    /// Uncertain path execution (lower confidence).
    Uncertain,
}

/// Certitude Mode governs query fidelity vs latency tradeoff.
/// Asymmetric I/O quota: STRICT consumes 3x, BALANCED 1.5x, FAST 1x.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CertitudeMode {
    /// L1 only (Hamming). Lowest latency, lowest fidelity.
    Fast,
    /// L1 + L2 re-ranking (PolarQuant). Balanced.
    Balanced,
    /// L1 + L2 + L3 FP32 verification. Highest fidelity, highest I/O cost.
    Strict,
}

impl CertitudeMode {
    /// Returns the I/O quota multiplier for asymmetric penalization.
    /// Prevents inefficient agents from saturating disk bandwidth.
    pub fn io_quota_multiplier(&self) -> f32 {
        match self {
            CertitudeMode::Fast => 1.0,
            CertitudeMode::Balanced => 1.5,
            CertitudeMode::Strict => 3.0,
        }
    }
}

/// Extract line/col from a nom parse error by measuring how much input was consumed.
fn iql_error_position(input: &str, err: &nom::Err<nom::error::Error<&str>>) -> (usize, usize) {
    let consumed = match err {
        nom::Err::Incomplete(_) => input.len(),
        nom::Err::Error(e) | nom::Err::Failure(e) => input.len() - e.input.len(),
    };
    let line = input[..consumed].matches('\n').count() + 1;
    let col = consumed
        - input[..consumed]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
    (line.max(1), col.max(1))
}

/// Query executor that evaluates logical plans against the storage engine.
pub struct Executor<'a> {
    /// Reference to the storage engine.
    storage: &'a StorageEngine,
    /// Current certitude mode.
    certitude: CertitudeMode,
    /// Search path mode.
    path_mode: SearchPathMode,
    /// Tracks cumulative I/O cost of this executor session.
    /// Hardware backpressure uses this to throttle expensive agents.
    io_budget_consumed: AtomicU32,
}

impl<'a> Executor<'a> {
    /// Create a new executor with default settings (Balanced, Standard).
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self {
            storage,
            certitude: CertitudeMode::Balanced,
            path_mode: SearchPathMode::Standard,
            io_budget_consumed: AtomicU32::new(0.0_f32.to_bits()),
        }
    }

    /// Create an executor with a specific certitude mode.
    pub fn with_certitude(storage: &'a StorageEngine, mode: CertitudeMode) -> Self {
        Self {
            storage,
            certitude: mode,
            path_mode: SearchPathMode::Standard,
            io_budget_consumed: AtomicU32::new(0.0_f32.to_bits()),
        }
    }

    /// Set the search path mode.
    pub fn with_path_mode(mut self, path: SearchPathMode) -> Self {
        self.path_mode = path;
        self
    }

    /// Track I/O cost with asymmetric penalization based on CertitudeMode.
    fn consume_io(&self, base_cost: f32) {
        let penalty = base_cost * self.certitude.io_quota_multiplier();
        let mut current_bits = self.io_budget_consumed.load(Ordering::Acquire);
        loop {
            let current = f32::from_bits(current_bits);
            let next = current + penalty;
            match self.io_budget_consumed.compare_exchange_weak(
                current_bits,
                next.to_bits(),
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(b) => current_bits = b,
            }
        }
    }

    /// Returns the cumulative I/O budget consumed by this executor.
    pub fn io_consumed(&self) -> f32 {
        f32::from_bits(self.io_budget_consumed.load(Ordering::Acquire))
    }

    /// Inserts a pre-built UnifiedNode directly into storage.
    pub fn insert_node(&self, node: &crate::node::UnifiedNode) -> crate::error::Result<()> {
        self.storage.insert(node)
    }

    /// Parse and execute a hybrid (IQL) query string.
    #[tracing::instrument(skip(self), err)]
    pub fn execute_hybrid(&self, query_string: &str) -> Result<ExecutionResult> {
        let trimmed = query_string.trim_start();
        if trimmed.starts_with('(') {
            Err(Error::Iql(ChainedError::msg(
                "LISP query execution is not supported (archived 2024-06). Use IQL syntax instead - see docs/api/IQL.md.",
            )))
        } else {
            match parse_statement(trimmed) {
                Ok((_, stmt)) => self.execute_statement(stmt),
                Err(e) => {
                    let (line, col) = iql_error_position(trimmed, &e);
                    Err(Error::IqlParse {
                        msg: e.to_string(),
                        line,
                        col,
                    })
                }
            }
        }
    }

    /// Execute a pre-parsed statement against the storage engine.
    #[tracing::instrument(skip(self), err)]
    pub fn execute_statement(&self, statement: Statement) -> Result<ExecutionResult> {
        Self::check_memory_pressure()?;

        match statement {
            Statement::Select(select) => self.execute_select(select),
            Statement::Query(query) => self.execute_query(query),
            Statement::Insert(insert) => self.execute_insert(insert),
            Statement::Update(update) => self.execute_update(update),
            Statement::Delete(delete) => self.execute_delete(delete),
            Statement::Relate(relate) => self.execute_relate(relate),
            Statement::InsertMessage(msg) => self.execute_insert_message(msg),
        }
    }

    /// Admission probe: rejects the statement when the process is under memory pressure.
    fn check_memory_pressure() -> Result<()> {
        use crate::governor::ResourceGovernor;
        let governor = ResourceGovernor::new(2 * GIB, 50);
        let probe_cost = 0;
        governor.request_allocation(probe_cost)?;
        Ok(())
    }

    /// SELECT path: logical plan → Volcano execution → read results.
    fn execute_select(&self, select: SelectStatement) -> Result<ExecutionResult> {
        let plan = select.into_logical_plan();
        let nodes = self.execute_plan(plan)?;
        Ok(ExecutionResult::Read(nodes))
    }

    /// QUERY path: logical plan → execution → archaeological interception filter.
    fn execute_query(&self, query: Query) -> Result<ExecutionResult> {
        let plan = query.into_logical_plan();
        let nodes = self.execute_plan(plan)?;
        Ok(ExecutionResult::Read(
            Self::filter_low_confidence_summaries(nodes),
        ))
    }

    /// Phase 30: Archaeological Interception (non-blocking).
    /// Drops low-confidence `SemanticSummary` nodes from read results.
    fn filter_low_confidence_summaries(nodes: Vec<UnifiedNode>) -> Vec<UnifiedNode> {
        use crate::node::AccessStats;
        let mut filtered_nodes = Vec::with_capacity(nodes.len());
        for node in nodes {
            let is_low_confidence_summary =
                if let Some(crate::node::FieldValue::String(node_type)) =
                    node.relational.get("type")
                {
                    node_type == "SemanticSummary" && node.confidence_score() < 0.4
                } else {
                    false
                };

            if is_low_confidence_summary {
                tracing::warn!(
                    "[Executor] Supervised mode: Low-confidence summary detected (ID {}). Skipping.",
                    node.id
                );
            } else {
                filtered_nodes.push(node);
            }
        }
        filtered_nodes
    }

    /// INSERT path: builds a Hot node from fields + vector/embed logic, then stores it.
    fn execute_insert(&self, insert: InsertStatement) -> Result<ExecutionResult> {
        let InsertStatement {
            node_id,
            node_type,
            fields,
            vector,
        } = insert;
        let mut node = UnifiedNode::new(node_id);
        // Newly inserted nodes are immediately Hot: they just arrived and are
        // the highest-priority candidates for volatile_cache residence.
        node.tier = crate::node::NodeTier::Hot;
        node.set_field("type", crate::node::FieldValue::String(node_type));

        // Copy all provided fields
        for (k, v) in fields.clone() {
            node.set_field(&k, v);
        }

        self.auto_embed_insert(&mut node, &vector, &fields, node_id);
        Self::apply_explicit_vector(&mut node, vector);

        self.storage.insert(&node)?;
        Ok(ExecutionResult::Write {
            affected_nodes: 1,
            message: format!("Node {node_id} inserted."),
            node_id: Some(node_id),
        })
    }

    /// Auto-Embedding Logic: if VECTOR is not provided in IQL but a "text" field exists,
    /// embed it via the remote provider (graceful degradation on failure).
    #[cfg(feature = "remote-inference")]
    fn auto_embed_insert(
        &self,
        node: &mut UnifiedNode,
        vector: &Option<Vec<f32>>,
        fields: &std::collections::BTreeMap<String, crate::node::FieldValue>,
        node_id: u128,
    ) {
        if vector.is_none() {
            if let Some(crate::node::FieldValue::String(text)) = fields.get("text") {
                if !text.trim().is_empty() {
                    let provider = crate::llm::get_embedding_provider();
                    match provider.embed(text) {
                        Ok(vec) => {
                            node.vector = VectorRepresentations::Full(vec);
                            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                        }
                        Err(e) => {
                            tracing::warn!("Auto-embedding failed for INSERT node {node_id}: {e}")
                        }
                    }
                }
            }
        }
    }

    /// Without `remote-inference` there is no provider: warn once, insert without a vector.
    #[cfg(not(feature = "remote-inference"))]
    fn auto_embed_insert(
        &self,
        _node: &mut UnifiedNode,
        vector: &Option<Vec<f32>>,
        fields: &std::collections::BTreeMap<String, crate::node::FieldValue>,
        _node_id: u128,
    ) {
        if vector.is_none() && fields.contains_key("text") {
            tracing::warn!("LLM feature disabled: skipping automatic embedding generation");
        }
    }

    /// Applies an explicit VECTOR when present (shared by INSERT/UPDATE paths).
    fn apply_explicit_vector(node: &mut UnifiedNode, vector: Option<Vec<f32>>) {
        if let Some(vec) = vector {
            node.vector = VectorRepresentations::Full(vec);
            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
        }
    }

    /// UPDATE path: merges fields (+ optional vector) into the stored node.
    fn execute_update(&self, update: UpdateStatement) -> Result<ExecutionResult> {
        let mut node = match self.storage.get(update.node_id)? {
            Some(n) => n,
            None => {
                return Err(Error::NotFound {
                    kind: "node".into(),
                    id: update.node_id.to_string(),
                })
            }
        };
        for (k, v) in update.fields {
            node.set_field(k, v);
        }
        Self::apply_explicit_vector(&mut node, update.vector);

        self.storage.insert(&node)?;
        Ok(ExecutionResult::Write {
            affected_nodes: 1,
            message: format!("Node {} updated.", node.id),
            node_id: Some(node.id),
        })
    }

    /// DELETE path: tombstones the node with an audit reason.
    fn execute_delete(&self, delete: DeleteStatement) -> Result<ExecutionResult> {
        self.storage.delete(delete.node_id, "IQL Manual Deletion")?;
        Ok(ExecutionResult::Write {
            affected_nodes: 1,
            message: format!("Node {} deleted.", delete.node_id),
            node_id: Some(delete.node_id),
        })
    }

    /// RELATE path: attaches a (weighted) edge after topological-consistency checks.
    fn execute_relate(&self, relate: RelateStatement) -> Result<ExecutionResult> {
        let mut node = match self.storage.get(relate.source_id)? {
            Some(n) => n,
            None => {
                return Err(Error::NotFound {
                    kind: "source_node".into(),
                    id: relate.source_id.to_string(),
                })
            }
        };

        // Axiom: Topological Consistency
        if self.storage.get(relate.target_id)?.is_none() {
            if self.storage.is_deleted(relate.target_id).unwrap_or(false) {
                return Err(Error::NotFound {
                    kind: "tombstone_node".into(),
                    id: relate.target_id.to_string(),
                });
            } else {
                return Err(Error::NotFound {
                    kind: "target_node".into(),
                    id: relate.target_id.to_string(),
                });
            }
        }

        let label_id = self.storage.intern_label(&relate.label);
        if let Some(w) = relate.weight {
            node.add_weighted_edge(relate.target_id, label_id, w);
        } else {
            node.add_edge(relate.target_id, label_id);
        }
        self.storage.insert(&node)?;
        Ok(ExecutionResult::Write {
            affected_nodes: 1,
            message: format!(
                "Edge related from {} to {}.",
                relate.source_id, relate.target_id
            ),
            node_id: Some(relate.source_id),
        })
    }

    /// INSERT-MESSAGE path: syntactic sugar for chat threads (message node + belongs_to edge).
    fn execute_insert_message(&self, msg: InsertMessageStatement) -> Result<ExecutionResult> {
        // Normally we'd use a UUID generator, but for MVP we use a timestamp-based ID or random
        let msg_id = web_time::SystemTime::now()
            .duration_since(web_time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros();
        let mut node = UnifiedNode::new(msg_id);
        node.set_field(
            "type",
            crate::node::FieldValue::String("Message".to_string()),
        );
        node.set_field(
            "role",
            crate::node::FieldValue::String(msg.msg_role.clone()),
        );
        node.set_field(
            "content",
            crate::node::FieldValue::String(msg.content.clone()),
        );

        self.auto_embed_message(&mut node, &msg.content, msg_id);

        // Now create relationship: MESSAGE -> belongs_to -> THREAD
        let belongs_to_id = self.storage.intern_label("belongs_to_thread");
        node.add_edge(msg.thread_id, belongs_to_id);

        // Node is saved (Atomic write for State + Edge)
        self.storage.insert(&node)?;

        Ok(ExecutionResult::Write {
            affected_nodes: 2,
            message: format!(
                "Message {msg_id} inserted and linked to Thread {}.",
                msg.thread_id
            ),
            node_id: Some(msg_id),
        })
    }

    /// Embeds a chat message directly via the LLM (graceful degradation on failure).
    #[cfg(feature = "remote-inference")]
    fn auto_embed_message(&self, node: &mut UnifiedNode, content: &str, node_id: u128) {
        if !content.trim().is_empty() {
            let provider = crate::llm::get_embedding_provider();
            match provider.embed(content) {
                Ok(vec) => {
                    node.vector = VectorRepresentations::Full(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }
                Err(e) => tracing::warn!("Auto-embedding failed for InsertMessage {node_id}: {e}"),
            }
        }
    }

    /// Without `remote-inference` there is no provider: insert the message without a vector.
    #[cfg(not(feature = "remote-inference"))]
    fn auto_embed_message(&self, _node: &mut UnifiedNode, _content: &str, _node_id: u128) {}

    /// Evaluates the Logical Plan over the underlying storage engine
    #[tracing::instrument(skip(self), err)]
    pub fn execute_plan(&self, mut plan: LogicalPlan) -> Result<Vec<UnifiedNode>> {
        use crate::governor::ResourceGovernor;

        let governor = ResourceGovernor::new(2 * GIB, 50); // 2GB Soft Limit, 50ms timeout
        governor.apply_temperature_limits(&mut plan);

        // OLD-21: derive the admission budget from the semantic cost estimator
        // instead of a fixed 1MB heuristic. The plan is estimated AFTER
        // temperature limits are applied so hot systems are accounted correctly.
        let estimated_mem_cost = governor
            .estimate_plan_cost(self.storage, &plan)
            .estimated_bytes;
        governor.request_allocation(estimated_mem_cost)?;

        // Intercept Conflict# entity scans (governance framework was archived 2024-06)
        for op in &plan.operators {
            if let LogicalOperator::Scan { entity } = op {
                if entity.starts_with("Conflict#") {
                    governor.free_allocation(estimated_mem_cost);
                    return Err(Error::Iql(ChainedError::msg(
                        "Conflict# entity scans are not supported (governance framework archived 2024-06).",
                    )));
                }
            }
        }

        // Compile logical plan into dynamic physical Volcano plan using Cost-Based Optimizer
        let mut physical_op = crate::planner::optimize_and_compile(&plan, self.storage)?;

        let mut results = Vec::new();
        physical_op.open()?;

        while let Some(node) = physical_op.next()? {
            self.consume_io(1.0); // Track I/O units for Volcano step execution

            // Agented RBAC (Role-Based Access Control) Graph pruning
            if let Some(required_role) = &plan.enforce_role {
                let mut role_match = false;
                if let Some(crate::node::FieldValue::String(node_role)) =
                    node.relational.get("_owner_role")
                {
                    if node_role == required_role {
                        role_match = true;
                    }
                }
                if !role_match && required_role != "admin" {
                    continue; // Prune branch (Sub-graph isolation enforced)
                }
            }

            results.push(node);
        }

        physical_op.close()?;
        governor.free_allocation(estimated_mem_cost);
        Ok(results)
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::backend::BackendKind;
    use crate::config::Config;
    use crate::node::FieldValue;
    #[cfg(feature = "remote-inference")]
    use crate::query::InsertMessageStatement;
    use crate::query::{DeleteStatement, InsertStatement, RelateStatement, UpdateStatement};
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    fn setup_storage() -> (StorageEngine, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = Config {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("Failed to open StorageEngine");
        (storage, dir)
    }

    // ── Construction ──

    #[test]
    fn test_new_executor_defaults() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);
        assert_eq!(ex.certitude, CertitudeMode::Balanced);
        assert_eq!(ex.path_mode, SearchPathMode::Standard);
        assert_eq!(ex.io_consumed(), 0.0);
    }

    #[test]
    fn test_with_certitude() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::with_certitude(&storage, CertitudeMode::Strict);
        assert_eq!(ex.certitude, CertitudeMode::Strict);
    }

    #[test]
    fn test_with_path_mode_changes_mode() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage).with_path_mode(SearchPathMode::Uncertain);
        assert_eq!(ex.path_mode, SearchPathMode::Uncertain);
    }

    // ── CertitudeMode ──

    #[test]
    fn test_certitude_io_multiplier() {
        assert_eq!(CertitudeMode::Fast.io_quota_multiplier(), 1.0);
        assert_eq!(CertitudeMode::Balanced.io_quota_multiplier(), 1.5);
        assert_eq!(CertitudeMode::Strict.io_quota_multiplier(), 3.0);
    }

    // ── I/O tracking ──

    #[test]
    fn test_io_consumed_starts_zero() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::with_certitude(&storage, CertitudeMode::Fast);
        assert_eq!(ex.io_consumed(), 0.0);
    }

    #[test]
    fn test_consume_io_increases_budget() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::with_certitude(&storage, CertitudeMode::Fast);
        ex.consume_io(5.0);
        assert_eq!(ex.io_consumed(), 5.0);
    }

    #[test]
    fn test_consume_io_accumulates() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::with_certitude(&storage, CertitudeMode::Fast);
        ex.consume_io(1.0);
        ex.consume_io(2.0);
        assert_eq!(ex.io_consumed(), 3.0);
    }

    #[test]
    fn test_consume_io_applies_multiplier() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::with_certitude(&storage, CertitudeMode::Strict);
        ex.consume_io(10.0);
        assert_eq!(ex.io_consumed(), 30.0); // 10 * 3.0
    }

    #[test]
    fn test_consume_io_fast_mode() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::with_certitude(&storage, CertitudeMode::Fast);
        ex.consume_io(7.0);
        assert_eq!(ex.io_consumed(), 7.0); // 7 * 1.0
    }

    // ── insert_node ──

    #[test]
    fn test_insert_node_stores_in_storage() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);
        let node = UnifiedNode::new(42);
        ex.insert_node(&node).unwrap();
        let fetched = storage.get(42).unwrap().unwrap();
        assert_eq!(fetched.id, 42);
    }

    // ── execute_hybrid ──

    #[test]
    fn test_execute_hybrid_rejects_lisp() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);
        let err = ex.execute_hybrid("(match ...)").unwrap_err();
        assert!(matches!(err, Error::Iql(_)));
        assert!(err.to_string().contains("LISP"));
    }

    #[test]
    fn test_execute_hybrid_parse_error() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);
        let err = ex.execute_hybrid("NOT_VALID_IQL").unwrap_err();
        assert!(matches!(err, Error::IqlParse { .. }));
    }

    // ── execute_statement: Insert ──

    #[test]
    fn test_execute_insert_statement() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let mut fields = BTreeMap::new();
        fields.insert("name".into(), FieldValue::String("alice".into()));

        let stmt = Statement::Insert(InsertStatement {
            node_id: 10,
            node_type: "Person".into(),
            fields,
            vector: Some(vec![0.1, 0.2, 0.3]),
        });

        let result = ex.execute_statement(stmt).unwrap();
        match result {
            ExecutionResult::Write {
                affected_nodes,
                message,
                node_id,
            } => {
                assert_eq!(affected_nodes, 1);
                assert_eq!(node_id, Some(10));
                assert!(message.contains("inserted"));
            }
            _ => panic!("expected Write result"),
        }

        let node = storage.get(10).unwrap().unwrap();
        assert_eq!(
            node.get_field("name"),
            Some(&FieldValue::String("alice".into()))
        );
    }

    #[test]
    fn test_execute_insert_without_vector_emits_warning() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let mut fields = BTreeMap::new();
        fields.insert("text".into(), FieldValue::String("hello".into()));

        let stmt = Statement::Insert(InsertStatement {
            node_id: 20,
            node_type: "Message".into(),
            fields,
            vector: None,
        });

        // Without remote-inference feature, it inserts with a warning but no vector
        let result = ex.execute_statement(stmt).unwrap();
        match result {
            ExecutionResult::Write { affected_nodes, .. } => {
                assert_eq!(affected_nodes, 1);
            }
            _ => panic!("expected Write result"),
        }

        let node = storage.get(20).unwrap().unwrap();
        assert!(node.vector.is_none());
    }

    // ── execute_statement: Update ──

    #[test]
    fn test_execute_update_statement() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        // Insert first
        let insert = Statement::Insert(InsertStatement {
            node_id: 30,
            node_type: "Person".into(),
            fields: BTreeMap::new(),
            vector: None,
        });
        ex.execute_statement(insert).unwrap();

        // Update
        let mut fields = BTreeMap::new();
        fields.insert("age".into(), FieldValue::Int(25));
        let update = Statement::Update(UpdateStatement {
            node_id: 30,
            fields,
            vector: None,
        });
        let result = ex.execute_statement(update).unwrap();
        match result {
            ExecutionResult::Write {
                affected_nodes,
                message,
                ..
            } => {
                assert_eq!(affected_nodes, 1);
                assert!(message.contains("updated"));
            }
            _ => panic!("expected Write result"),
        }

        let node = storage.get(30).unwrap().unwrap();
        assert_eq!(node.get_field("age"), Some(&FieldValue::Int(25)));
    }

    #[test]
    fn test_execute_update_nonexistent_errors() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let update = Statement::Update(UpdateStatement {
            node_id: 999,
            fields: BTreeMap::new(),
            vector: None,
        });
        let err = ex.execute_statement(update).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    // ── execute_statement: Delete ──

    #[test]
    fn test_execute_delete_statement() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let insert = Statement::Insert(InsertStatement {
            node_id: 40,
            node_type: "Temp".into(),
            fields: BTreeMap::new(),
            vector: None,
        });
        ex.execute_statement(insert).unwrap();

        let delete = Statement::Delete(DeleteStatement { node_id: 40 });
        let result = ex.execute_statement(delete).unwrap();
        match result {
            ExecutionResult::Write {
                affected_nodes,
                message,
                ..
            } => {
                assert_eq!(affected_nodes, 1);
                assert!(message.contains("deleted"));
            }
            _ => panic!("expected Write result"),
        }

        assert!(storage.get(40).unwrap().is_none());
    }

    // ── execute_statement: Relate ──

    #[test]
    fn test_execute_relate_statement() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        // Insert two nodes
        ex.execute_statement(Statement::Insert(InsertStatement {
            node_id: 50,
            node_type: "A".into(),
            fields: BTreeMap::new(),
            vector: None,
        }))
        .unwrap();
        ex.execute_statement(Statement::Insert(InsertStatement {
            node_id: 51,
            node_type: "B".into(),
            fields: BTreeMap::new(),
            vector: None,
        }))
        .unwrap();

        // Relate them
        let relate = Statement::Relate(RelateStatement {
            source_id: 50,
            target_id: 51,
            label: "knows".into(),
            weight: Some(0.9),
        });
        let result = ex.execute_statement(relate).unwrap();
        match result {
            ExecutionResult::Write {
                affected_nodes,
                message,
                ..
            } => {
                assert_eq!(affected_nodes, 1);
                assert!(message.contains("related"));
            }
            _ => panic!("expected Write result"),
        }

        let node = storage.get(50).unwrap().unwrap();
        assert_eq!(node.edges.len(), 1);
        assert_eq!(node.edges[0].target, 51);
        let knows_id = storage.intern_label("knows");
        assert_eq!(node.edges[0].label_id, knows_id);
        assert_eq!(node.edges[0].weight, 0.9);
    }

    #[test]
    fn test_execute_relate_missing_source_errors() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let relate = Statement::Relate(RelateStatement {
            source_id: 999,
            target_id: 1,
            label: "x".into(),
            weight: None,
        });
        let err = ex.execute_statement(relate).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_execute_relate_missing_target_errors() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        ex.execute_statement(Statement::Insert(InsertStatement {
            node_id: 60,
            node_type: "A".into(),
            fields: BTreeMap::new(),
            vector: None,
        }))
        .unwrap();

        let relate = Statement::Relate(RelateStatement {
            source_id: 60,
            target_id: 999,
            label: "x".into(),
            weight: None,
        });
        let err = ex.execute_statement(relate).unwrap_err();
        assert!(err.to_string().contains("not found") || err.to_string().contains("Tombstone"));
    }

    // ── Auto-embedding (remote-inference feature) ──

    #[cfg(feature = "remote-inference")]
    #[test]
    fn test_auto_embedding_graceful_degradation_on_insert() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let mut fields = BTreeMap::new();
        fields.insert("text".into(), FieldValue::String("hello world".into()));

        let stmt = Statement::Insert(InsertStatement {
            node_id: 100,
            node_type: "Message".into(),
            fields,
            vector: None,
        });

        // No Ollama running — auto-embedding fails gracefully. Node still inserted.
        let result = ex.execute_statement(stmt).unwrap();
        match result {
            ExecutionResult::Write { affected_nodes, .. } => {
                assert_eq!(affected_nodes, 1);
            }
            _ => panic!("expected Write result"),
        }

        let node = storage.get(100).unwrap().unwrap();
        // Auto-embedding failed (no Ollama), so no vector was set
        assert!(node.vector.is_none());
    }

    #[cfg(feature = "remote-inference")]
    #[test]
    fn test_auto_embedding_graceful_degradation_on_insert_message() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let stmt = Statement::InsertMessage(InsertMessageStatement {
            thread_id: 200,
            msg_role: "user".into(),
            content: "hello world".into(),
        });

        // No Ollama running — auto-embedding fails gracefully. Node + edge still inserted.
        let result = ex.execute_statement(stmt).unwrap();
        match result {
            ExecutionResult::Write {
                affected_nodes,
                message,
                ..
            } => {
                assert_eq!(affected_nodes, 2);
                assert!(message.contains("inserted"));
            }
            _ => panic!("expected Write result"),
        }
    }

    #[cfg(feature = "remote-inference")]
    #[test]
    fn test_auto_embedding_skipped_when_vector_provided() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let mut fields = BTreeMap::new();
        fields.insert("text".into(), FieldValue::String("hello world".into()));

        let stmt = Statement::Insert(InsertStatement {
            node_id: 101,
            node_type: "Message".into(),
            fields,
            vector: Some(vec![0.1, 0.2, 0.3]),
        });

        // Vector is explicitly provided — auto-embedding is skipped.
        let result = ex.execute_statement(stmt).unwrap();
        match result {
            ExecutionResult::Write { .. } => {}
            _ => panic!("expected Write result"),
        }

        let node = storage.get(101).unwrap().unwrap();
        assert!(!node.vector.is_none());
        assert!(node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR));
    }

    #[cfg(feature = "remote-inference")]
    #[test]
    fn test_auto_embedding_skipped_on_empty_text() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let mut fields = BTreeMap::new();
        fields.insert("text".into(), FieldValue::String("".into()));
        fields.insert("name".into(), FieldValue::String("no-text".into()));

        let stmt = Statement::Insert(InsertStatement {
            node_id: 110,
            node_type: "Message".into(),
            fields,
            vector: None,
        });

        // Empty text — auto-embedding is skipped without calling the LLM.
        let result = ex.execute_statement(stmt).unwrap();
        match result {
            ExecutionResult::Write { affected_nodes, .. } => {
                assert_eq!(affected_nodes, 1);
            }
            _ => panic!("expected Write result"),
        }

        let node = storage.get(110).unwrap().unwrap();
        assert!(
            node.vector.is_none(),
            "empty text should not trigger LLM call"
        );
        assert_eq!(
            node.get_field("name"),
            Some(&FieldValue::String("no-text".into()))
        );
    }

    #[cfg(feature = "remote-inference")]
    #[test]
    fn test_auto_embedding_skipped_on_no_text_field() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let mut fields = BTreeMap::new();
        fields.insert("title".into(), FieldValue::String("hello".into()));

        let stmt = Statement::Insert(InsertStatement {
            node_id: 111,
            node_type: "Note".into(),
            fields,
            vector: None,
        });

        // No "text" field — auto-embedding is skipped entirely.
        let result = ex.execute_statement(stmt).unwrap();
        match result {
            ExecutionResult::Write { affected_nodes, .. } => {
                assert_eq!(affected_nodes, 1);
            }
            _ => panic!("expected Write result"),
        }

        let node = storage.get(111).unwrap().unwrap();
        assert!(node.vector.is_none());
    }

    #[cfg(feature = "remote-inference")]
    #[test]
    fn test_auto_embedding_skipped_on_empty_message_content() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let stmt = Statement::InsertMessage(InsertMessageStatement {
            thread_id: 300,
            msg_role: "user".into(),
            content: "".into(),
        });

        // Empty message content — auto-embedding is skipped without LLM call.
        let result = ex.execute_statement(stmt).unwrap();
        match result {
            ExecutionResult::Write {
                affected_nodes,
                message,
                ..
            } => {
                assert_eq!(affected_nodes, 2);
                assert!(message.contains("inserted"));
            }
            _ => panic!("expected Write result"),
        }
    }

    // ── execute_statement: dispatcher por variante (C2S5, sin cambio semántico) ──

    #[test]
    fn test_execute_select_statement_reads() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        ex.execute_statement(Statement::Insert(InsertStatement {
            node_id: 70,
            node_type: "Person".into(),
            fields: BTreeMap::new(),
            vector: None,
        }))
        .unwrap();

        let select = Statement::Select(crate::query::SelectStatement {
            projections: vec![],
            from: crate::query::FromClause::Single {
                entity: "*".into(),
                alias: "n".into(),
            },
            where_clause: None,
            subquery_conditions: vec![],
            temperature: None,
        });
        let result = ex.execute_statement(select).unwrap();
        match result {
            ExecutionResult::Read(nodes) => {
                assert!(nodes.iter().any(|n| n.id == 70));
            }
            _ => panic!("expected Read result"),
        }
    }

    #[test]
    fn test_execute_query_statement_reads() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        ex.execute_statement(Statement::Insert(InsertStatement {
            node_id: 71,
            node_type: "Person".into(),
            fields: BTreeMap::new(),
            vector: None,
        }))
        .unwrap();

        let query = Statement::Query(crate::query::Query {
            from_entity: "*".into(),
            traversal: None,
            target_alias: "n".into(),
            where_clause: None,
            fetch: None,
            rank_by: None,
            temperature: None,
            owner_role: None,
            search_profile: None,
        });
        let result = ex.execute_statement(query).unwrap();
        match result {
            ExecutionResult::Read(nodes) => {
                assert!(nodes.iter().any(|n| n.id == 71));
            }
            _ => panic!("expected Read result"),
        }
    }

    #[test]
    fn test_execute_insert_message_links_thread() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        let stmt = Statement::InsertMessage(crate::query::InsertMessageStatement {
            thread_id: 400,
            msg_role: "user".into(),
            content: "hola".into(),
        });
        let result = ex.execute_statement(stmt).unwrap();
        let msg_id = match result {
            ExecutionResult::Write {
                affected_nodes,
                message,
                node_id,
            } => {
                assert_eq!(affected_nodes, 2);
                assert!(message.contains("inserted"));
                node_id.expect("message node id")
            }
            _ => panic!("expected Write result"),
        };

        let node = storage.get(msg_id).unwrap().unwrap();
        assert_eq!(
            node.get_field("type"),
            Some(&FieldValue::String("Message".into()))
        );
        assert_eq!(node.edges.len(), 1);
        assert_eq!(node.edges[0].target, 400);
        assert_eq!(
            node.edges[0].label_id,
            storage.intern_label("belongs_to_thread")
        );
    }

    // ── Admission guard (OLD-21): cost-aware budget must be returned on error ──

    #[test]
    #[serial_test::serial]
    fn test_execute_plan_frees_admission_on_error() {
        use crate::governor::ALLOCATED_BYTES;
        use std::sync::atomic::Ordering;

        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        // The Conflict# scan hits the experimental-governance early-return AFTER
        // `request_allocation`; the guard MUST `free_allocation` the same bytes
        // or the in-flight counter leaks and eventually OOMs real queries.
        let plan = LogicalPlan {
            operators: vec![LogicalOperator::Scan {
                entity: "Conflict#missing-extension".to_string(),
            }],
            temperature: 0.0,
            enforce_role: None,
            search_profile: None,
        };

        ALLOCATED_BYTES.store(0, Ordering::SeqCst);
        let result = ex.execute_plan(plan);
        assert!(result.is_err(), "Conflict scan must be rejected");
        assert_eq!(
            ALLOCATED_BYTES.load(Ordering::SeqCst),
            0,
            "admission budget must be freed on the error path"
        );
        ALLOCATED_BYTES.store(0, Ordering::SeqCst);
    }
}
