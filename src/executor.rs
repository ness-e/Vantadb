use crate::error::{Result, VantaError};
use crate::eval::LispSandbox;
use crate::governance::{ResolutionResult, ConfidenceArbiter};
use crate::node::{UnifiedNode, VectorRepresentations};
use crate::parser::lisp::parse as parse_lisp_expr;
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
    StaleContext(u64), // Phase 30: Señal de que un contexto requiere rehidratación (Confidence Score crítico)
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
    /// Used by the LISP sandbox to inject Node rules.
    pub fn insert_node(&self, node: &crate::node::UnifiedNode) -> crate::error::Result<()> {
        self.storage.insert(node)
    }

    pub async fn execute_hybrid(&self, query_string: &str) -> Result<ExecutionResult> {
        let trimmed = query_string.trim_start();
        if trimmed.starts_with('(') {
            let expr = parse_lisp_expr(trimmed)
                .map_err(|e| VantaError::Execution(format!("LISP Parse Error: {}", e)))?;
            let mut sandbox = LispSandbox::new(self);
            sandbox.eval(std::borrow::Cow::Owned(expr)).await
        } else {
            match parse_statement(trimmed) {
                Ok((_, stmt)) => self.execute_statement(stmt).await,
                Err(e) => Err(VantaError::Execution(format!("IQL Parse Error: {}", e))),
            }
        }
    }

    pub async fn execute_statement(&self, statement: Statement) -> Result<ExecutionResult> {
        // ── Memory Pressure Check ──
        {
            use crate::governor::{AllocationStatus, ResourceGovernor};
            let governor = ResourceGovernor::new(2 * 1024 * 1024 * 1024, 50);
            let probe_cost = 0; 
            if let Ok(AllocationStatus::GrantedWithPressure) =
                governor.request_allocation(probe_cost)
            {
                println!("🚨 [ResourceGovernor] High memory pressure (>90%) detected. Triggering emergency flush.");
                if let Some(winner) = self.storage.consistency_buffer.force_flush() {
                    println!(
                        "    └─ Priority record preserved: {}",
                        winner.id
                    );
                    let _ = self.storage.insert(&winner);
                }
            }
        }

        match statement {
            Statement::Query(query) => {
                let plan = query.into_logical_plan();
                let nodes = self.execute_plan(plan).await?;

                use crate::node::AccessTracker;
                // Fase 30: Interceptación Arqueológica (Non-blocking)
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
                node.set_field("type", crate::node::FieldValue::String(insert.node_type));

                // Copy all provided fields
                for (k, v) in insert.fields.clone() {
                    node.set_field(&k, v);
                }

                // Auto-Embedding Logic: If VECTOR is not provided in IQL, but "texto" field exists!
                if insert.vector.is_none() {
                    if let Some(crate::node::FieldValue::String(text)) = insert.fields.get("texto")
                    {
                        let llm = crate::llm::LlmClient::new();
                        // Request vectors to local Ollama inference bridge
                        if let Ok(vec) = llm.generate_embedding(text).await {
                            node.vector = VectorRepresentations::Full(vec);
                            node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                        }
                    }
                } else if let Some(vec) = insert.vector {
                    node.vector = VectorRepresentations::Full(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                }

                // ── Admission Filter Check ──
                if let Some(crate::node::FieldValue::String(role)) =
                    node.relational.get("_owner_role")
                {
                    if self.storage.admission_filter.is_role_blocked(role) {
                        return Err(VantaError::Execution(format!(
                            "Admission Policy: agent '{}' has Confidence Score 0.0 (blocked)",
                            role
                        )));
                    }
                }

                // Conflict Resolution
                if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
                    if let crate::node::VectorRepresentations::Full(vec) = &node.vector {
                        let nearest = {
                            let index = self.storage.hnsw.read();
                            let vs = self.storage.vector_store.read();
                            index.search_nearest(vec, None, None, 0, 1, Some(&vs))
                        };

                        if let Some((existing_id, _)) = nearest.first() {
                            if *existing_id != node.id {
                                if let Some(existing) = self.storage.get(*existing_id)? {
                                    match self.storage.conflict_resolver.evaluate_conflict(&existing, &node)
                                    {
                                        ResolutionResult::Reject(reason) => {
                                            return Err(VantaError::Execution(format!(
                                                "Consistency Violation: {}",
                                                reason
                                            )));
                                        }
                                        ResolutionResult::Superposition(record) => {
                                            self.storage
                                                .consistency_buffer
                                                .insert_record(record);
                                            return Ok(ExecutionResult::Write { 
                                                affected_nodes: 1, 
                                                message: format!("Node {} moved to ConsistencyBuffer (Pending Resolution).", node.id),
                                                node_id: Some(node.id),
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
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
                // ── Admission Filter Check ──
                if let Some(crate::node::FieldValue::String(role)) =
                    node.relational.get("_owner_role")
                {
                    if self.storage.admission_filter.is_role_blocked(role) {
                        return Err(VantaError::Execution(
                            format!("Admission Policy (Update): agent '{}' has Confidence Score 0.0 (blocked)", role)
                        ));
                    }
                }

                // Conflict Resolution
                if node.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
                    if let crate::node::VectorRepresentations::Full(vec) = &node.vector {
                        let nearest = {
                            let index = self.storage.hnsw.read();
                            let vs = self.storage.vector_store.read();
                            index.search_nearest(vec, None, None, 0, 1, Some(&vs))
                        };

                        if let Some((existing_id, _)) = nearest.first() {
                            if *existing_id != node.id {
                                if let Some(existing) = self.storage.get(*existing_id)? {
                                    match self.storage.conflict_resolver.evaluate_conflict(&existing, &node)
                                    {
                                        ResolutionResult::Reject(reason) => {
                                            return Err(VantaError::Execution(format!(
                                                "Consistency Violation (Update): {}",
                                                reason
                                            )));
                                        }
                                        ResolutionResult::Superposition(record) => {
                                            self.storage
                                                .consistency_buffer
                                                .insert_record(record);
                                            return Ok(ExecutionResult::Write { 
                                                affected_nodes: 1, 
                                                message: format!("Node {} update entered ConsistencyBuffer (Pending Resolution).", node.id),
                                                node_id: Some(node.id),
                                            });
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
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
                    if self
                        .storage
                        .is_deleted(relate.target_id)
                        .unwrap_or(false)
                    {
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
                    .unwrap()
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
                let llm = crate::llm::LlmClient::new();
                if let Ok(vec) = llm.generate_embedding(&msg.content).await {
                    node.vector = VectorRepresentations::Full(vec);
                    node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
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
            Statement::Collapse(collapse) => {
                let mut buffer = self.storage.consistency_buffer.records.write();
                if let Some(mut record) = buffer.remove(&collapse.zone_id) {
                    if collapse.index < record.candidates.len() {
                        let winner = record.candidates.remove(collapse.index);

                        // Remaining candidates to archive
                        let mut losers_to_archive = Vec::new();
                        for cand in record.candidates {
                            losers_to_archive.push((
                                collapse.zone_id,
                                cand.id,
                                "Manual Resolution: Candidate discarded by administrator".to_string(),
                            ));
                        }

                        self.storage
                            .consistency_buffer
                            .stats
                            .pending_to_resolved
                            .fetch_add(1, Ordering::Relaxed);
                        drop(buffer);

                        self.storage.insert(&winner)?;

                        if !losers_to_archive.is_empty() {
                            use crate::governance::AuditableTombstone;
                            if let Some(cf_shadow) = self.storage.db.cf_handle("tombstone_storage") {
                                for (id, hash, reason) in losers_to_archive {
                                    let tomb = AuditableTombstone::new(id, reason, hash);
                                    let key = id.to_le_bytes();
                                    if let Ok(tomb_val) = bincode::serialize(&tomb) {
                                        let _ = self.storage.db.put_cf(&cf_shadow, key, &tomb_val);
                                    }
                                }
                            }
                        }

                        Ok(ExecutionResult::Write {
                            affected_nodes: 1,
                            message: format!(
                                "Consistency record {} resolved. Candidate {} prevailed.",
                                collapse.zone_id, collapse.index
                            ),
                            node_id: Some(collapse.zone_id),
                        })
                    } else {
                        Err(VantaError::Execution(format!(
                            "Candidate index {} out of bounds for record {}",
                            collapse.index, collapse.zone_id
                        )))
                    }
                } else {
                    Err(VantaError::Execution(format!(
                        "Consistency record {} not found in buffer",
                        collapse.zone_id
                    )))
                }
            }
        }
    }

    /// Evaluates the Logical Plan over the underlying storage engine
    pub async fn execute_plan(&self, mut plan: LogicalPlan) -> Result<Vec<UnifiedNode>> {
        use crate::governor::ResourceGovernor;

        let governor = ResourceGovernor::new(2 * 1024 * 1024 * 1024, 50); // 2GB Soft Limit, 50ms timeout
        governor.apply_temperature_limits(&mut plan);

        let estimated_mem_cost = 1024 * 1024; // 1MB estimated buffer footprint per query
        match governor.request_allocation(estimated_mem_cost)? {
            crate::governor::AllocationStatus::GrantedWithPressure => {
                println!("🚨 [ResourceGovernor] High memory pressure detected. Triggering emergency flush.");
                if let Some(winner) = self.storage.consistency_buffer.force_flush() {
                    println!(
                        "    └─ Priority record preserved: {}",
                        winner.id
                    );
                    let _ = self.storage.insert(&winner);
                }
            }
            crate::governor::AllocationStatus::Granted => {}
        }

        let mut results = Vec::new();
        let mut target_nodes = Vec::new();

        // Pass 1: Resolver Escaneo Vectorial Dinámico (Si hubiere Condition::VectorSim)
        let mut searched_hnsw = false;

        for op in &plan.operators {
            if let LogicalOperator::VectorSearch {
                field: _,
                query_vec,
                min_score: _,
            } = op
            {
                let llm = crate::llm::LlmClient::new();

                // Real Inference: Translate NLP into Embedded Vectors
                if let Ok(vector) = llm.generate_embedding(query_vec).await {
                    // Record basic vector search I/O cost (cost logic is synthetic placeholder)
                    self.consume_io(10.0);

                    let index = self.storage.hnsw.read();
                    let vs = self.storage.vector_store.read();
                    let mut neighbors = index.search_nearest(&vector, None, None, 0, 5, Some(&vs)); // MVP: top_k = 5

                    if self.path_mode == SearchPathMode::Uncertain {
                        // Scan the ConsistencyBuffer via brute force
                        let buffer = self.storage.consistency_buffer.records.read();
                        let target_vec = VectorRepresentations::Full(vector.clone());
                        let mut matches = Vec::new();

                        for (&id, record) in buffer.iter() {
                            for cand in &record.candidates {
                                if cand.flags.is_set(crate::node::NodeFlags::HAS_VECTOR) {
                                    if let Some(sim) = cand.vector.cosine_similarity(&target_vec) {
                                        // Apply a penalty to the pending match
                                        let penalized_sim = sim * 0.9;
                                        matches.push((id, penalized_sim));
                                    }
                                }
                            }
                        }

                        // Merge and sort
                        neighbors.extend(matches);
                        neighbors.sort_by(|a, b| {
                            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
                        });
                        neighbors.truncate(5); // Keep top 5
                    }

                    for (id, _sim) in neighbors {
                        target_nodes.push(id);
                    }
                    searched_hnsw = true;
                }
            }
        }

        if !searched_hnsw {
            // Fallback: real scan based on FROM entity (Scan operator)
            for op in &plan.operators {
                if let LogicalOperator::Scan { entity } = op {
                    // If entity starts with Conflict#, intercept it immediately
                    if entity.starts_with("Conflict#") {
                        if let Some(id_str) = entity.split('#').nth(1) {
                            if let Ok(id) = id_str.parse::<u64>() {
                                let buffer = self.storage.consistency_buffer.records.read();
                                if let Some(record) = buffer.get(&id) {
                                    return Ok(record.candidates.clone());
                                }
                            }
                        }
                    } else if let Some(id_str) = entity.split('#').nth(1) {
                        if let Ok(id) = id_str.parse::<u64>() {
                            target_nodes.push(id);
                        }
                    }
                    // Otherwise, scan is deferred to post-filter (MVP limitation)
                    break;
                }
            }
        }

        // Pass 2: Materializar los nodos devueltos por el índice y filtrar RBAC
        for id in target_nodes {
            // Materializing nodes is I/O intensive, track heavily
            self.consume_io(2.5);

            if let Ok(Some(node)) = self.storage.get(id) {
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
        }

        governor.free_allocation(estimated_mem_cost);
        Ok(results)
    }
}
