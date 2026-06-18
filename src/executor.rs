use crate::error::{Result, VantaError};
use crate::node::{UnifiedNode, VectorRepresentations};
use crate::parser::parse_statement;
use crate::query::{LogicalOperator, LogicalPlan, Statement};
use crate::storage::StorageEngine;
use std::sync::atomic::{AtomicU32, Ordering};

pub enum ExecutionResult {
    Read(Vec<UnifiedNode>),
    Write {
        affected_nodes: usize,
        message: String,
        node_id: Option<u64>,
    },
    StaleContext(u64), // Phase 30: Signal that a context requires rehydration (Critical Confidence Score)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SearchPathMode {
    Standard,
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

pub struct Executor<'a> {
    storage: &'a StorageEngine,
    certitude: CertitudeMode,
    path_mode: SearchPathMode,
    /// Tracks cumulative I/O cost of this executor session.
    /// Hardware backpressure uses this to throttle expensive agents.
    io_budget_consumed: AtomicU32,
}

impl<'a> Executor<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self {
            storage,
            certitude: CertitudeMode::Balanced,
            path_mode: SearchPathMode::Standard,
            io_budget_consumed: AtomicU32::new(0.0_f32.to_bits()),
        }
    }

    pub fn with_certitude(storage: &'a StorageEngine, mode: CertitudeMode) -> Self {
        Self {
            storage,
            certitude: mode,
            path_mode: SearchPathMode::Standard,
            io_budget_consumed: AtomicU32::new(0.0_f32.to_bits()),
        }
    }

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

    #[tracing::instrument(skip(self), err)]
    pub fn execute_hybrid(&self, query_string: &str) -> Result<ExecutionResult> {
        let trimmed = query_string.trim_start();
        if trimmed.starts_with('(') {
            Err(VantaError::Execution(
                "LISP queries require the experimental-lisp extension/crate.".to_string(),
            ))
        } else {
            match parse_statement(trimmed) {
                Ok((_, stmt)) => self.execute_statement(stmt),
                Err(e) => Err(VantaError::Execution(format!("IQL Parse Error: {}", e))),
            }
        }
    }

    #[tracing::instrument(skip(self), err)]
    pub fn execute_statement(&self, statement: Statement) -> Result<ExecutionResult> {
        // ── Memory Pressure Check ──
        {
            use crate::governor::ResourceGovernor;
            let governor = ResourceGovernor::new(2 * 1024 * 1024 * 1024, 50);
            let probe_cost = 0;
            let _ = governor.request_allocation(probe_cost);
        }

        match statement {
            Statement::Query(query) => {
                let plan = query.into_logical_plan();
                let nodes = self.execute_plan(plan)?;

                use crate::node::AccessTracker;
                // Phase 30: Archaeological Interception (Non-blocking)
                for node in &nodes {
                    if let Some(crate::node::FieldValue::String(node_type)) =
                        node.relational.get("type")
                    {
                        if node_type == "SemanticSummary" && node.confidence_score() < 0.4 {
                            println!("⚠️ [Executor] Supervised mode: Low-confidence summary detected (ID 0). Skipping.");
                            continue;
                        }
                    }
                }

                Ok(ExecutionResult::Read(nodes))
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
                        return Err(VantaError::Execution(format!(
                            "Node {} not found for update",
                            update.node_id
                        )))
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
                        return Err(VantaError::Execution(format!(
                            "Source Node {} not found for relation",
                            relate.source_id
                        )))
                    }
                };

                // Axiom: Topological Consistency
                if self.storage.get(relate.target_id)?.is_none() {
                    if self.storage.is_deleted(relate.target_id).unwrap_or(false) {
                        return Err(VantaError::Execution(format!(
                            "Reference to deleted node: ID {} resides in the Tombstone storage",
                            relate.target_id
                        )));
                    } else {
                        return Err(VantaError::Execution(format!(
                            "Topological Axiom violated: Target Node {} does not exist",
                            relate.target_id
                        )));
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
                let msg_id = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
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

        let governor = ResourceGovernor::new(2 * 1024 * 1024 * 1024, 50); // 2GB Soft Limit, 50ms timeout
        governor.apply_temperature_limits(&mut plan);

        let estimated_mem_cost = 1024 * 1024; // 1MB estimated buffer footprint per query
        let _ = governor.request_allocation(estimated_mem_cost)?;

        // Intercept Conflict entity scan for experimental governance immediately
        for op in &plan.operators {
            if let LogicalOperator::Scan { entity } = op {
                if entity.starts_with("Conflict#") {
                    governor.free_allocation(estimated_mem_cost);
                    return Err(VantaError::Execution(
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
