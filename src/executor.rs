//! Query execution engine that translates logical plans into results.
//!
//! Evaluates [`LogicalOperator`] trees against the [`StorageEngine`],
//! returning materialized [`ExecutionResult`] variants.

use crate::error::{Result, VantaError};
use crate::node::{UnifiedNode, VectorRepresentations};
use crate::parser::parse_statement;
use crate::query::{LogicalOperator, LogicalPlan, Statement};
use crate::storage::StorageEngine;
use std::sync::atomic::{AtomicU32, Ordering};

const GIB: usize = 1024 * 1024 * 1024;
const MIB: usize = 1024 * 1024;

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
        node_id: Option<u64>,
    },
    /// Signal that a context requires rehydration (low confidence score).
    StaleContext(u64),
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
            Err(VantaError::IqlError(
                "LISP queries require the experimental-lisp extension/crate.".to_string(),
            ))
        } else {
            match parse_statement(trimmed) {
                Ok((_, stmt)) => self.execute_statement(stmt),
                Err(e) => Err(VantaError::IqlParseError {
                    msg: e.to_string(),
                    line: 0,
                    col: 0,
                }),
            }
        }
    }

    /// Execute a pre-parsed statement against the storage engine.
    #[tracing::instrument(skip(self), err)]
    pub fn execute_statement(&self, statement: Statement) -> Result<ExecutionResult> {
        // ── Memory Pressure Check ──
        {
            use crate::governor::ResourceGovernor;
            let governor = ResourceGovernor::new(2 * GIB, 50);
            let probe_cost = 0;
            governor.request_allocation(probe_cost)?;
        }

        match statement {
            Statement::Query(query) => {
                let plan = query.into_logical_plan();
                let nodes = self.execute_plan(plan)?;

                use crate::node::AccessTracker;
                // Phase 30: Archaeological Interception (Non-blocking)
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

                Ok(ExecutionResult::Read(filtered_nodes))
            }
            Statement::Insert(insert) => {
                let mut node = UnifiedNode::new(insert.node_id);
                // Newly inserted nodes are immediately Hot: they just arrived and are
                // the highest-priority candidates for volatile_cache residence.
                node.tier = crate::node::NodeTier::Hot;
                node.set_field("type", crate::node::FieldValue::String(insert.node_type));

                // Copy all provided fields
                for (k, v) in insert.fields.clone() {
                    node.set_field(&k, v);
                }

                // Auto-Embedding Logic: If VECTOR is not provided in IQL, but "text" field exists!
                #[cfg(feature = "remote-inference")]
                if insert.vector.is_none() {
                    if let Some(crate::node::FieldValue::String(text)) = insert.fields.get("text") {
                        let llm = crate::llm::LlmClient::new();
                        // Request vectors to local Ollama inference bridge
                        if let Ok(vec) = llm.generate_embedding(text) {
                            node.vector = VectorRepresentations::Full(vec);
                            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                        }
                    }
                }
                #[cfg(not(feature = "remote-inference"))]
                if insert.vector.is_none() && insert.fields.contains_key("text") {
                    tracing::warn!("LLM feature disabled: skipping automatic embedding generation");
                }
                if insert.vector.is_some() {
                    if let Some(vec) = insert.vector {
                        node.vector = VectorRepresentations::Full(vec);
                        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                    }
                }

                self.storage.insert(&node)?;
                Ok(ExecutionResult::Write {
                    affected_nodes: 1,
                    message: format!("Node {} inserted.", insert.node_id),
                    node_id: Some(insert.node_id),
                })
            }
            Statement::Update(update) => {
                let mut node = match self.storage.get(update.node_id)? {
                    Some(n) => n,
                    None => {
                        return Err(VantaError::NotFound {
                            kind: "node".into(),
                            id: update.node_id.to_string(),
                        })
                    }
                };
                for (k, v) in update.fields {
                    node.set_field(k, v);
                }
                if let Some(vec) = update.vector {
                    node.vector = VectorRepresentations::Full(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }

                self.storage.insert(&node)?;
                Ok(ExecutionResult::Write {
                    affected_nodes: 1,
                    message: format!("Node {} updated.", node.id),
                    node_id: Some(node.id),
                })
            }
            Statement::Delete(delete) => {
                self.storage.delete(delete.node_id, "IQL Manual Deletion")?;
                Ok(ExecutionResult::Write {
                    affected_nodes: 1,
                    message: format!("Node {} deleted.", delete.node_id),
                    node_id: Some(delete.node_id),
                })
            }
            Statement::Relate(relate) => {
                let mut node = match self.storage.get(relate.source_id)? {
                    Some(n) => n,
                    None => {
                        return Err(VantaError::NotFound {
                            kind: "source_node".into(),
                            id: relate.source_id.to_string(),
                        })
                    }
                };

                // Axiom: Topological Consistency
                if self.storage.get(relate.target_id)?.is_none() {
                    if self.storage.is_deleted(relate.target_id).unwrap_or(false) {
                        return Err(VantaError::NotFound {
                            kind: "tombstone_node".into(),
                            id: relate.target_id.to_string(),
                        });
                    } else {
                        return Err(VantaError::NotFound {
                            kind: "target_node".into(),
                            id: relate.target_id.to_string(),
                        });
                    }
                }

                if let Some(w) = relate.weight {
                    node.add_weighted_edge(relate.target_id, relate.label, w);
                } else {
                    node.add_edge(relate.target_id, relate.label);
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
            Statement::InsertMessage(msg) => {
                // Syntactic Sugar for Chat Threads: Creates a node and relates it.
                // Normally we'd use a UUID generator, but for MVP we use a timestamp-based ID or random
                let msg_id = web_time::SystemTime::now()
                    .duration_since(web_time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_micros() as u64;
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

                // Embed directly via LLM since it's a message
                #[cfg(feature = "remote-inference")]
                {
                    let llm = crate::llm::LlmClient::new();
                    if let Ok(vec) = llm.generate_embedding(&msg.content) {
                        node.vector = VectorRepresentations::Full(vec);
                        node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                    }
                }

                // Now create relationship: MESSAGE -> belongs_to -> THREAD
                node.add_edge(msg.thread_id, "belongs_to_thread".to_string());

                // Node is saved (Atomic write for State + Edge)
                self.storage.insert(&node)?;

                Ok(ExecutionResult::Write {
                    affected_nodes: 2,
                    message: format!(
                        "Message {} inserted and linked to Thread {}.",
                        msg_id, msg.thread_id
                    ),
                    node_id: Some(msg_id),
                })
            }
        }
    }

    /// Evaluates the Logical Plan over the underlying storage engine
    #[tracing::instrument(skip(self), err)]
    pub fn execute_plan(&self, mut plan: LogicalPlan) -> Result<Vec<UnifiedNode>> {
        use crate::governor::ResourceGovernor;

        let governor = ResourceGovernor::new(2 * GIB, 50); // 2GB Soft Limit, 50ms timeout
        governor.apply_temperature_limits(&mut plan);

        let estimated_mem_cost = MIB; // 1MB estimated buffer footprint per query
        governor.request_allocation(estimated_mem_cost)?;

        // Intercept Conflict entity scan for experimental governance immediately
        for op in &plan.operators {
            if let LogicalOperator::Scan { entity } = op {
                if entity.starts_with("Conflict#") {
                    governor.free_allocation(estimated_mem_cost);
                    return Err(VantaError::IqlError(
                        "Conflict entity scan requires the experimental-governance extension/crate."
                            .to_string(),
                    ));
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
    use crate::config::VantaConfig;
    use crate::node::FieldValue;
    use crate::query::{DeleteStatement, InsertStatement, RelateStatement, UpdateStatement};
    use std::collections::BTreeMap;
    use tempfile::tempdir;

    fn setup_storage() -> (StorageEngine, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = VantaConfig {
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
        assert!(matches!(err, VantaError::IqlError(_)));
        assert!(err.to_string().contains("LISP"));
    }

    #[test]
    fn test_execute_hybrid_parse_error() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);
        let err = ex.execute_hybrid("NOT_VALID_IQL").unwrap_err();
        assert!(matches!(err, VantaError::IqlParseError { .. }));
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
        assert_eq!(node.edges[0].label, "knows");
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
}
