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

/// Human-readable message for an IQL parse failure.
///
/// The grammar reports a duplicated `AS OF` clause (SCH-03 review Optional-2)
/// as `Failure(Verify)` whose input slice starts at the second `AS OF`; that
/// is the only failure emitted at an `AS OF` position. This maps it to a
/// stable message and leaves every other failure with nom's default rendering.
/// WIRE-12 adds the same treatment for malformed `LIMIT`/`OFFSET` clauses
/// (missing count or reversed/duplicated order).
///
/// WIRE-13 aggregation restrictions use `ErrorKind::Fail` (distinct from the
/// structural `Verify`) so each unsupported combination maps to a stable
/// message keyed by the failure input prefix: `GROUP BY` without aggregate or
/// with a colliding field, `AS OF`/`LIMIT`/`OFFSET` with aggregation,
/// `GROUP BY` on the FROM surface, multi-field `GROUP BY` (`,`), and mixed
/// plain fields + aggregates (statement start, `SELECT`). Any other `Fail`
/// falls back to a generic message — never a misleading specific one.
fn iql_parse_error_message(err: &nom::Err<nom::error::Error<&str>>) -> String {
    if let nom::Err::Failure(inner) = err {
        // FIND-518/FIND-519: leftover input after a parsed statement is
        // rejected by the parser (nom's "there is leftover input" kind).
        if inner.code == nom::error::ErrorKind::Eof {
            let trailing: String = inner.input.trim().chars().take(80).collect();
            return format!("unexpected trailing input after the statement: `{trailing}`");
        }
        if inner.code == nom::error::ErrorKind::Verify {
            let head = inner.input.trim_start();
            if head.starts_with("AS OF") {
                return "AS OF specified more than once".to_string();
            }
            if head.starts_with("LIMIT") || head.starts_with("OFFSET") {
                return "malformed LIMIT/OFFSET clause: expected `LIMIT <n> [OFFSET <m>]`"
                    .to_string();
            }
            // FIND-518: the `WHERE` keyword was consumed but the clause does
            // not parse — the filter is never silently dropped.
            if head.starts_with("WHERE") {
                return "malformed WHERE clause: expected at least one condition (`WHERE <cond> [AND <cond>...]`)".to_string();
            }
        }
        if inner.code == nom::error::ErrorKind::Fail {
            let head = inner.input.trim_start();
            if head.starts_with("GROUP BY") {
                return "invalid GROUP BY clause: requires at least one aggregate function (COUNT/SUM) and a group field that does not collide with an aggregate output name".to_string();
            }
            if head.starts_with("AS OF") {
                return "AS OF is not supported with aggregation queries".to_string();
            }
            if head.starts_with("LIMIT") || head.starts_with("OFFSET") {
                return "LIMIT/OFFSET are not supported with aggregation queries".to_string();
            }
            if head.starts_with("FROM") || head.starts_with("MATCH") {
                return "aggregation (COUNT/SUM/GROUP BY) is only supported in SELECT queries"
                    .to_string();
            }
            if head.starts_with(',') {
                return "only a single GROUP BY field is supported".to_string();
            }
            if head.starts_with("SELECT") {
                return "cannot mix plain fields and aggregate functions in SELECT; group keys come from GROUP BY".to_string();
            }
            return "invalid aggregation syntax".to_string();
        }
    }
    err.to_string()
}

/// RAII guard that returns the governor's admission reservation on **every**
/// exit path (FIND-555).
///
/// The reservation lives in a **global** counter (`governor::ALLOCATED_BYTES`),
/// so an early `?` used to leak it: every failed query left its reservation
/// charged forever. After ~20 failures the counter passed the 2 GiB soft limit
/// and `check_memory_pressure` rejected **every** subsequent query with
/// `Error::ResourceLimit` — including writes — leaving the engine unusable
/// until it was restarted. The old code only freed the budget on the happy
/// path (and on one explicit early return), which is why the leak survived a
/// test that only exercised that single early return.
struct AllocationGuard<'a> {
    governor: &'a crate::governor::ResourceGovernor,
    bytes: usize,
}

impl Drop for AllocationGuard<'_> {
    fn drop(&mut self) {
        self.governor.free_allocation(self.bytes);
    }
}

/// The diagnostic the `ROLE` prune emits when it prunes **every** record the
/// plan produced (FIND-559).
///
/// `None` when the prune was partial, and `None` when the plan produced
/// nothing at all: a partial prune is ordinary filtering, and an empty plan
/// says nothing about the prune — that is a **different** case (the `AS OF`
/// filter, which runs *after* the plan, emptying the result).
///
/// Informative only: the caller logs it. It never becomes an `Error` and it
/// never changes the result set.
fn role_prune_notice(
    required_role: &str,
    scanned: usize,
    pruned: usize,
    untagged: usize,
) -> Option<String> {
    if scanned == 0 || pruned < scanned {
        return None;
    }
    let cause = if untagged == pruned {
        "none of them carries the '_owner_role' field".to_string()
    } else {
        format!("none of them carries the role '{required_role}'")
    };
    Some(format!(
        "ROLE '{required_role}' pruned {pruned} of {scanned} scanned records: {cause}. \
         Tag the records with '_owner_role' in their metadata or drop the clause."
    ))
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
                        msg: iql_parse_error_message(&e),
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
    /// SELECT path: logical plan → Volcano execution → valid-time filter.
    fn execute_select(&self, select: SelectStatement) -> Result<ExecutionResult> {
        let as_of_ms = select.as_of_ms;
        let plan = select.into_logical_plan();
        let nodes = self.execute_plan(plan)?;
        Ok(ExecutionResult::Read(Self::filter_valid_at(
            nodes, as_of_ms,
        )))
    }

    /// QUERY path: logical plan → execution → archaeological interception
    /// filter + valid-time filter (`AS OF`, SCH-03).
    fn execute_query(&self, query: Query) -> Result<ExecutionResult> {
        let as_of_ms = query.as_of_ms;
        let plan = query.into_logical_plan();
        let nodes = self.execute_plan(plan)?;
        let nodes = Self::filter_low_confidence_summaries(nodes);
        Ok(ExecutionResult::Read(Self::filter_valid_at(
            nodes, as_of_ms,
        )))
    }

    /// Phase 30: Archaeological Interception (non-blocking).
    /// Drops low-confidence `SemanticSummary` nodes from read results.
    fn filter_low_confidence_summaries(nodes: Vec<UnifiedNode>) -> Vec<UnifiedNode> {
        use crate::node::AccessStats;
        let mut filtered_nodes = Vec::with_capacity(nodes.len());
        for node in nodes {
            let is_low_confidence_summary =
                if let Some(crate::node::FieldValue::String(node_type)) =
                    node.relational.get(crate::physical_plan::FIELD_TYPE)
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

    /// Valid-time filter for IQL `AS OF T` (SCH-03, ADR-046 §D3).
    ///
    /// Memory nodes carry `__vanta_valid_at_ms` (v2; always written) with a v1
    /// fallback to `__vanta_created_at_ms` (ADR-046 §D7 normalization; `0` is
    /// the v1 "unset" sentinel). Nodes without any validity metadata (plain
    /// graph nodes) are not time-scoped and always pass — `AS OF` only narrows
    /// records that declare a validity window. `None` = no filter (default).
    fn filter_valid_at(nodes: Vec<UnifiedNode>, as_of_ms: Option<u64>) -> Vec<UnifiedNode> {
        let Some(t_ms) = as_of_ms else {
            return nodes;
        };
        nodes
            .into_iter()
            .filter(|node| Self::node_is_valid_at(node, t_ms))
            .collect()
    }

    /// ADR-046 §D3 predicate for a raw node: `valid_at <= t < invalid_at`.
    fn node_is_valid_at(node: &UnifiedNode, t_ms: u64) -> bool {
        fn u64_field(node: &UnifiedNode, field: &str) -> Option<u64> {
            match node.relational.get(field) {
                Some(crate::node::FieldValue::Int(value)) if *value >= 0 => Some(*value as u64),
                _ => None,
            }
        }
        let valid_at = u64_field(node, crate::sdk::serialization::FIELD_VALID_AT_MS)
            .filter(|value| *value > 0)
            .or_else(|| u64_field(node, crate::sdk::serialization::FIELD_CREATED_AT_MS));
        let Some(valid_at) = valid_at else {
            return true;
        };
        let invalid_at = u64_field(node, crate::sdk::serialization::FIELD_INVALID_AT_MS);
        valid_at <= t_ms && invalid_at.is_none_or(|invalid| invalid > t_ms)
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
                            let cfg = crate::config::Config::default().llm_cfg();
                            if Self::is_synthetic_local_fallback(
                                &cfg.embedding_provider,
                                &cfg.local_model_path,
                            ) {
                                // Fallback avisado, nunca silencioso (FIND-485):
                                // un vector sintético persistido ensucia el índice
                                // con vecinos falsos — peor que un nodo sin vector.
                                tracing::warn!(
                                    "Auto-embedding for INSERT node {node_id} produced a synthetic \
                                     vector (local model files missing under '{}'): the node is \
                                     stored WITHOUT a vector — a synthetic vector would pollute \
                                     the index with false neighbours",
                                    cfg.local_model_path
                                );
                            } else {
                                node.vector = VectorRepresentations::Full(vec);
                                node.flags.set(crate::node::NodeFlags::HAS_VECTOR);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Auto-embedding failed for INSERT node {node_id}: {e}")
                        }
                    }
                }
            }
        }
    }

    /// `true` when the vector the provider just returned is a SYNTHETIC one
    /// (the deterministic dummy), i.e. the local provider could not load the
    /// model files.
    ///
    /// Parity with the MCP path (`vantadb-mcp/src/handlers/tools.rs`): a local
    /// provider without model files never persists vectors. A remote provider
    /// (ollama/openai) computes a real vector on the server, so it is kept.
    #[cfg(all(feature = "remote-inference", feature = "embed-local"))]
    fn is_synthetic_local_fallback(provider_name: &str, model_dir: &str) -> bool {
        if !crate::embedding_health::routes_to_local(provider_name) {
            return false;
        }
        !crate::embedding_health::model_files_present(model_dir)
    }

    /// This build has no local provider: every vector comes from a server, so
    /// none of them is synthetic.
    #[cfg(all(feature = "remote-inference", not(feature = "embed-local")))]
    fn is_synthetic_local_fallback(_provider_name: &str, _model_dir: &str) -> bool {
        false
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
        // FIND-555: the reservation is global, so it must be returned on
        // EVERY exit path — the guard does it on drop, including the `?`
        // early returns below.
        let _reservation = AllocationGuard {
            governor: &governor,
            bytes: estimated_mem_cost,
        };

        // Intercept Conflict# entity scans (governance framework was archived 2024-06)
        for op in &plan.operators {
            if let LogicalOperator::Scan { entity } = op {
                if entity.starts_with("Conflict#") {
                    return Err(Error::Iql(ChainedError::msg(
                        "Conflict# entity scans are not supported (governance framework archived 2024-06).",
                    )));
                }
            }
        }

        // Compile logical plan into dynamic physical Volcano plan using Cost-Based Optimizer
        let mut physical_op = crate::planner::optimize_and_compile(&plan, self.storage)?;

        let mut results = Vec::new();
        // FIND-559: bookkeeping of the `ROLE` prune. The prune keeps its
        // fail-closed semantics (a record without `_owner_role` is discarded);
        // these counters only exist so the executor can explain an empty
        // result. Zero cost when the plan carries no `ROLE`.
        let (mut role_scanned, mut role_pruned, mut role_untagged) = (0usize, 0usize, 0usize);
        physical_op.open()?;

        while let Some(node) = physical_op.next()? {
            self.consume_io(1.0); // Track I/O units for Volcano step execution

            // Agented RBAC (Role-Based Access Control) Graph pruning
            if let Some(required_role) = &plan.enforce_role {
                role_scanned += 1;
                let owner_role = node.relational.get(crate::physical_plan::FIELD_OWNER_ROLE);
                let role_match = matches!(
                    owner_role,
                    Some(crate::node::FieldValue::String(node_role)) if node_role == required_role
                );
                if !role_match && required_role != "admin" {
                    // FIND-559: count what the prune discards (once per
                    // query, never once per record).
                    role_pruned += 1;
                    if owner_role.is_none() {
                        role_untagged += 1;
                    }
                    continue; // Prune branch (Sub-graph isolation enforced)
                }
            }

            results.push(node);
        }

        physical_op.close()?;

        // FIND-559: **one** notice per query, and only when the `ROLE` prune
        // ate every record the plan produced — the case that used to return
        // zero rows with no hint of why. Informative: it is not an `Error`
        // and it does not change the result. A plan that produced nothing
        // says nothing about the prune (that is the `AS OF` case, which runs
        // after the plan).
        if let Some(required_role) = &plan.enforce_role {
            if let Some(notice) =
                role_prune_notice(required_role, role_scanned, role_pruned, role_untagged)
            {
                tracing::warn!("{notice}");
            }
        }

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

    // ── execute_hybrid: AS OF / valid-time (SCH-03) ──

    /// Node shaped like a time-scoped record: `type` + optional validity
    /// window fields (the exact reserved names the record write path uses).
    fn insert_validity_node(
        storage: &StorageEngine,
        id: u128,
        node_type: &str,
        valid_at_ms: Option<u64>,
        invalid_at_ms: Option<u64>,
    ) {
        let mut node = UnifiedNode::new(id);
        node.set_field("type", FieldValue::String(node_type.to_string()));
        if let Some(valid) = valid_at_ms {
            node.set_field(
                crate::sdk::serialization::FIELD_VALID_AT_MS,
                FieldValue::Int(valid as i64),
            );
        }
        if let Some(invalid) = invalid_at_ms {
            node.set_field(
                crate::sdk::serialization::FIELD_INVALID_AT_MS,
                FieldValue::Int(invalid as i64),
            );
        }
        storage.insert(&node).expect("insert validity node");
    }

    fn read_ids(ex: &Executor<'_>, query: &str) -> Vec<u128> {
        match ex.execute_hybrid(query).expect("execute") {
            ExecutionResult::Read(nodes) => {
                let mut ids: Vec<u128> = nodes.iter().map(|n| n.id).collect();
                ids.sort_unstable();
                ids
            }
            other => panic!("expected Read, got {other:?}"),
        }
    }

    #[test]
    fn as_of_filters_nodes_by_valid_window_with_known_t() {
        let (storage, _dir) = setup_storage();
        insert_validity_node(&storage, 1, "Doc", Some(1000), Some(2000)); // [1000, 2000)
        insert_validity_node(&storage, 2, "Doc", Some(3000), None); // [3000, ∞)
        let ex = Executor::new(&storage);

        // Default (no AS OF) returns everything — zero breaking change.
        assert_eq!(read_ids(&ex, "FROM Doc"), vec![1, 2]);
        // Closed-open window: T < start excluded, T == start included,
        // T == end excluded, open end included.
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 999"), Vec::<u128>::new());
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 1000"), vec![1]);
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 1999"), vec![1]);
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 2000"), Vec::<u128>::new());
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 3000"), vec![2]);
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 999999"), vec![2]);
    }

    #[test]
    fn as_of_after_where_also_applies_and_includes_unscoped_nodes() {
        let (storage, _dir) = setup_storage();
        insert_validity_node(&storage, 1, "Doc", Some(1000), Some(2000));
        // Plain graph node: no validity metadata → not time-scoped, always passes.
        let mut plain = UnifiedNode::new(9);
        plain.set_field("type", FieldValue::String("Doc".to_string()));
        storage.insert(&plain).expect("insert plain node");
        let ex = Executor::new(&storage);

        assert_eq!(read_ids(&ex, "FROM Doc AS OF 1500"), vec![1, 9]);
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 2500"), vec![9]);
        // Late position (after WHERE) parses too.
        assert_eq!(
            read_ids(&ex, r#"FROM Doc WHERE type = "Doc" AS OF 1500"#),
            vec![1, 9]
        );
    }

    #[test]
    fn select_as_of_filters_namespace_scan_records() {
        let (storage, _dir) = setup_storage();
        // v1 memory-shaped node: no `__vanta_valid_at_ms`, created_at fallback.
        let mut v1 = UnifiedNode::new(7);
        v1.set_field(
            crate::sdk::serialization::FIELD_NAMESPACE,
            FieldValue::String("Ns".to_string()),
        );
        v1.set_field(
            crate::sdk::serialization::FIELD_CREATED_AT_MS,
            FieldValue::Int(500),
        );
        storage.insert(&v1).expect("insert v1 node");
        insert_validity_node(&storage, 8, "Other", Some(100), Some(200));
        let ex = Executor::new(&storage);

        // v1 normalization: valid := created_at (ADR §D7) — AS OF 499 excludes,
        // AS OF 500 includes (inclusive start).
        assert_eq!(
            read_ids(&ex, "SELECT * FROM Ns AS OF 499"),
            Vec::<u128>::new()
        );
        assert_eq!(read_ids(&ex, "SELECT * FROM Ns AS OF 500"), vec![7]);
        assert_eq!(read_ids(&ex, "SELECT * FROM Ns"), vec![7]);
    }

    // ── execute_hybrid: FIND-552 — la proyeccion no puede anular los
    // post-filtros (valid-time, confianza, ROLE) ─────────────────────────

    /// FIND-552: `AS OF` debe filtrar aunque la query proyecte campos
    /// (`FETCH`). La proyeccion borraba `__vanta_valid_at_ms` /
    /// `__vanta_created_at_ms` y el predicado temporal se degradaba a
    /// "siempre valido" para todas las filas.
    #[test]
    fn as_of_still_filters_when_the_query_projects_fields() {
        let (storage, _dir) = setup_storage();
        insert_validity_node(&storage, 1, "Doc", Some(1000), Some(2000));
        insert_validity_node(&storage, 2, "Doc", Some(3000), None);
        // Nodo sin metadatos de validez: no esta acotado en el tiempo ⇒
        // siempre pasa (no se inventa la semantica de un nodo legacy).
        let mut unscoped = UnifiedNode::new(9);
        unscoped.set_field("type", FieldValue::String("Doc".to_string()));
        unscoped.set_field("name", FieldValue::String("legacy".to_string()));
        storage.insert(&unscoped).expect("insert unscoped node");
        let ex = Executor::new(&storage);

        // Control: sin proyeccion el filtro temporal ya funciona.
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 1000"), vec![1, 9]);
        // Con proyeccion: el filtro temporal debe seguir aplicando.
        assert_eq!(read_ids(&ex, "FROM Doc AS OF 1000 FETCH name"), vec![1, 9]);
    }

    /// FIND-552: el filtro arqueologico de confianza (`SemanticSummary` con
    /// `confidence_score < 0.4`) debe seguir aplicando con proyeccion: la
    /// proyeccion borraba `type` y el filtro dejasaba de detectar nada.
    #[test]
    fn low_confidence_summaries_are_filtered_when_the_query_projects_fields() {
        let (storage, _dir) = setup_storage();
        let mut summary = UnifiedNode::new(1);
        summary.set_field("type", FieldValue::String("SemanticSummary".to_string()));
        summary.set_field("name", FieldValue::String("resumen".to_string()));
        summary.confidence_score = 0.1;
        storage
            .insert(&summary)
            .expect("insert low-confidence summary");

        let mut healthy = UnifiedNode::new(2);
        healthy.set_field("type", FieldValue::String("Doc".to_string()));
        healthy.set_field("name", FieldValue::String("doc".to_string()));
        storage.insert(&healthy).expect("insert healthy doc");

        let ex = Executor::new(&storage);

        // Control: sin proyeccion el resumen de baja confianza ya se descarta.
        assert_eq!(read_ids(&ex, "FROM SemanticSummary"), Vec::<u128>::new());
        // Con proyeccion: debe seguir descartandose.
        assert_eq!(
            read_ids(&ex, "FROM SemanticSummary FETCH name"),
            Vec::<u128>::new()
        );
        // Y el nodo sano no se ve afectado por la proyeccion.
        assert_eq!(read_ids(&ex, "FROM Doc FETCH name"), vec![2]);
    }

    /// FIND-552 / FIND-559: la poda por `ROLE` lee `_owner_role`, que la
    /// proyeccion tambien borraba ⇒ con proyeccion `ROLE` devolvia 0 filas.
    #[test]
    fn role_prune_still_applies_when_the_query_projects_fields() {
        let (storage, _dir) = setup_storage();
        let mut editor = UnifiedNode::new(1);
        editor.set_field("type", FieldValue::String("Doc".to_string()));
        editor.set_field("name", FieldValue::String("a".to_string()));
        editor.set_field("_owner_role", FieldValue::String("editor".to_string()));
        storage.insert(&editor).expect("insert editor node");

        let mut viewer = UnifiedNode::new(2);
        viewer.set_field("type", FieldValue::String("Doc".to_string()));
        viewer.set_field("name", FieldValue::String("b".to_string()));
        viewer.set_field("_owner_role", FieldValue::String("viewer".to_string()));
        storage.insert(&viewer).expect("insert viewer node");

        let ex = Executor::new(&storage);

        // Control: sin proyeccion la poda por rol deja solo al editor.
        assert_eq!(read_ids(&ex, r#"FROM Doc ROLE "editor""#), vec![1]);
        // Con proyeccion: la poda debe seguir aplicando.
        assert_eq!(
            read_ids(&ex, r#"FROM Doc FETCH name ROLE "editor""#),
            vec![1]
        );
    }

    // ── FIND-559: la poda de `ROLE` es fail-closed, pero avisa ──

    /// Captures the WARN-level tracing output produced by `f`.
    ///
    /// The `ROLE` prune reports through `tracing::warn!` (informative: never
    /// an `Error`, never a change of the result), so the notice is only
    /// observable in the log.
    fn capture_warnings<F: FnOnce()>(f: F) -> String {
        use std::io::Write;
        use std::sync::{Arc, Mutex};

        #[derive(Clone, Default)]
        struct CaptureWriter(Arc<Mutex<Vec<u8>>>);

        impl Write for CaptureWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0
                    .lock()
                    .expect("capture buffer")
                    .extend_from_slice(buf);
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CaptureWriter {
            type Writer = CaptureWriter;
            fn make_writer(&'a self) -> Self::Writer {
                self.clone()
            }
        }

        let sink = CaptureWriter::default();
        let subscriber = tracing_subscriber::fmt()
            .with_writer(sink.clone())
            .with_max_level(tracing::Level::WARN)
            .with_ansi(false)
            .finish();
        tracing::subscriber::with_default(subscriber, f);
        let bytes = sink.0.lock().expect("capture buffer").clone();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// FIND-559: a record without `_owner_role` is **pruned** (fail-closed) —
    /// the owner's decision of 2026-10-21 keeps that semantics. What changes
    /// is only the silence around it, never the rows.
    #[test]
    fn role_prune_keeps_the_fail_closed_result() {
        let (storage, _dir) = setup_storage();
        let mut tagged = UnifiedNode::new(1);
        tagged.set_field("type", FieldValue::String("Doc".to_string()));
        tagged.set_field("_owner_role", FieldValue::String("user".to_string()));
        storage.insert(&tagged).expect("insert tagged node");

        let mut untagged = UnifiedNode::new(2);
        untagged.set_field("type", FieldValue::String("Doc".to_string()));
        storage.insert(&untagged).expect("insert untagged node");

        let ex = Executor::new(&storage);

        // The tagged record is visible; the untagged one is pruned.
        assert_eq!(read_ids(&ex, r#"FROM Doc ROLE "user""#), vec![1]);
    }

    /// FIND-559: when the `ROLE` prune eats **every** scanned record the
    /// executor says so — once per query, with the counts and the field that
    /// is missing. Without the notice, `ROLE "user"` over an untagged
    /// database returned zero rows with no hint of why.
    #[test]
    fn role_prune_warns_once_when_it_prunes_every_scanned_record() {
        let (storage, _dir) = setup_storage();
        for id in 1..=3u128 {
            let mut node = UnifiedNode::new(id);
            node.set_field("type", FieldValue::String("Doc".to_string()));
            storage.insert(&node).expect("insert untagged node");
        }

        let ex = Executor::new(&storage);
        let warnings = capture_warnings(|| {
            let ids = read_ids(&ex, r#"FROM Doc ROLE "user""#);
            assert!(ids.is_empty(), "the fail-closed prune still holds");
        });

        assert!(
            warnings.contains("_owner_role"),
            "the notice must name the missing field: {warnings}"
        );
        assert!(
            warnings.contains("pruned 3 of 3"),
            "the notice must carry the counts: {warnings}"
        );
        assert_eq!(
            warnings.matches("pruned").count(),
            1,
            "the notice is emitted once per query, not once per record: {warnings}"
        );
    }

    /// FIND-559: a **partial** prune is ordinary filtering, not a diagnostic —
    /// no notice.
    #[test]
    fn role_prune_does_not_warn_when_it_prunes_only_part_of_the_records() {
        let (storage, _dir) = setup_storage();
        let mut tagged = UnifiedNode::new(1);
        tagged.set_field("type", FieldValue::String("Doc".to_string()));
        tagged.set_field("_owner_role", FieldValue::String("user".to_string()));
        storage.insert(&tagged).expect("insert tagged node");

        let mut untagged = UnifiedNode::new(2);
        untagged.set_field("type", FieldValue::String("Doc".to_string()));
        storage.insert(&untagged).expect("insert untagged node");

        let ex = Executor::new(&storage);
        let warnings = capture_warnings(|| {
            let ids = read_ids(&ex, r#"FROM Doc ROLE "user""#);
            assert_eq!(ids, vec![1]);
        });

        assert!(
            !warnings.contains("_owner_role"),
            "a partial prune is not a diagnostic: {warnings}"
        );
    }

    /// FIND-559: a query **without** `ROLE` never prunes ⇒ no notice (and no
    /// overhead on that path).
    #[test]
    fn role_prune_does_not_warn_without_a_role_clause() {
        let (storage, _dir) = setup_storage();
        let mut node = UnifiedNode::new(1);
        node.set_field("type", FieldValue::String("Doc".to_string()));
        storage.insert(&node).expect("insert untagged node");

        let ex = Executor::new(&storage);
        let warnings = capture_warnings(|| {
            let ids = read_ids(&ex, "FROM Doc");
            assert_eq!(ids, vec![1]);
        });

        assert!(
            !warnings.contains("_owner_role"),
            "no ROLE clause ⇒ no prune ⇒ no notice: {warnings}"
        );
    }

    /// FIND-559: the notice policy itself — the notice exists **only** when
    /// the prune ate 100 % of the records the plan produced.
    ///
    /// `scanned == 0` is the case where the plan produced nothing (an empty
    /// scan, a `WHERE` that matched nothing): there is nothing to say about
    /// the `ROLE` prune, and it is a different case from the `AS OF` filter
    /// emptying the result (that one runs *after* the plan).
    #[test]
    fn role_prune_notice_policy() {
        // 100 % pruned ⇒ notice.
        let notice = super::role_prune_notice("user", 3, 3, 3)
            .expect("a prune that ate everything must be reported");
        assert!(notice.contains("_owner_role"), "notice: {notice}");
        // Partial prune ⇒ no notice.
        assert!(
            super::role_prune_notice("user", 3, 1, 1).is_none(),
            "a partial prune is not a diagnostic"
        );
        // Nothing scanned ⇒ no notice.
        assert!(
            super::role_prune_notice("user", 0, 0, 0).is_none(),
            "an empty plan says nothing about the prune"
        );
    }

    /// FIND-552 invariant: a query **without** projection is untouched by the
    /// allowlist — it returns every field, exactly as before the fix.
    #[test]
    fn query_without_projection_still_returns_every_field() {
        let (storage, _dir) = setup_storage();
        let mut node = UnifiedNode::new(1);
        node.set_field("type", FieldValue::String("Doc".to_string()));
        node.set_field("name", FieldValue::String("n".to_string()));
        node.set_field("secret", FieldValue::String("s".to_string()));
        storage.insert(&node).expect("insert node");
        let ex = Executor::new(&storage);

        let rows = match ex.execute_hybrid("FROM Doc").expect("execute") {
            ExecutionResult::Read(rows) => rows,
            other => panic!("expected Read, got {other:?}"),
        };
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0].relational.contains_key("secret"),
            "no projection ⇒ no field is dropped"
        );
        assert!(rows[0].relational.contains_key("name"));
    }

    /// SCH-03 review Optional-2: a duplicated `AS OF` is rejected at parse
    /// time with a stable message and a position pointing at the duplicate.
    #[test]
    fn test_duplicate_as_of_reports_clear_message() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);
        let err = ex.execute_hybrid("FROM Doc AS OF 1 AS OF 2").unwrap_err();
        match err {
            Error::IqlParse { msg, line, col } => {
                assert_eq!(msg, "AS OF specified more than once");
                assert_eq!(line, 1);
                assert_eq!(col, 17, "position must point at the duplicate clause");
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }
    }

    /// WIRE-12: malformed pagination clauses get a stable message and the
    /// reported position points at the offending clause.
    #[test]
    fn test_malformed_pagination_reports_clear_message() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);
        let err = ex.execute_hybrid("FROM Doc LIMIT abc").unwrap_err();
        match err {
            Error::IqlParse { msg, line, col } => {
                assert_eq!(
                    msg,
                    "malformed LIMIT/OFFSET clause: expected `LIMIT <n> [OFFSET <m>]`"
                );
                assert_eq!(line, 1);
                assert_eq!(col, 9, "position must point at the malformed clause");
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }
        // Reversed order is rejected with the same stable message.
        let err = ex.execute_hybrid("FROM Doc OFFSET 2 LIMIT 5").unwrap_err();
        assert!(matches!(err, Error::IqlParse { .. }));
    }

    /// WIRE-12: a clause after the pagination keywords is rejected at parse
    /// time instead of silently returning a page without it (FIND-312 class).
    #[test]
    fn pagination_rejects_trailing_clause_instead_of_silently_dropping_it() {
        let (storage, _dir) = setup_storage();
        insert_five_ranked_nodes(&storage);
        let ex = Executor::new(&storage);
        let err = ex
            .execute_hybrid("FROM Doc LIMIT 5 WHERE n > 1")
            .unwrap_err();
        assert!(matches!(err, Error::IqlParse { .. }));
    }

    // ── execute_hybrid: LIMIT / OFFSET pagination (WIRE-12) ──

    /// Node shaped like a rankable graph record: `type` + integer field `n`.
    fn insert_ranked_node(storage: &StorageEngine, id: u128, n: i64) {
        let mut node = UnifiedNode::new(id);
        node.set_field("type", FieldValue::String("Doc".to_string()));
        node.set_field("n", FieldValue::Int(n));
        storage.insert(&node).expect("insert ranked node");
    }

    fn insert_five_ranked_nodes(storage: &StorageEngine) {
        for id in 1..=5u128 {
            insert_ranked_node(storage, id, id as i64);
        }
    }

    #[test]
    fn limit_caps_scan_results() {
        let (storage, _dir) = setup_storage();
        insert_five_ranked_nodes(&storage);
        let ex = Executor::new(&storage);
        // Scan order is backend-defined; the cap is what LIMIT guarantees.
        assert_eq!(read_ids(&ex, "FROM Doc LIMIT 2").len(), 2);
        assert_eq!(read_ids(&ex, "FROM Doc").len(), 5, "control: no cap");
    }

    #[test]
    fn offset_skips_leading_rows() {
        let (storage, _dir) = setup_storage();
        insert_five_ranked_nodes(&storage);
        let ex = Executor::new(&storage);
        assert_eq!(read_ids(&ex, "FROM Doc OFFSET 2").len(), 3);
        assert_eq!(read_ids(&ex, "FROM Doc OFFSET 10"), Vec::<u128>::new());
    }

    #[test]
    fn limit_offset_composes_skip_then_take() {
        let (storage, _dir) = setup_storage();
        insert_five_ranked_nodes(&storage);
        let ex = Executor::new(&storage);
        // RANK BY makes the order deterministic: [1,2,3,4,5] by n; skip 1, take 2.
        assert_eq!(
            read_ids(&ex, "FROM Doc RANK BY n LIMIT 2 OFFSET 1"),
            vec![2, 3]
        );
        // Skip 3, take 10 → the tail.
        assert_eq!(
            read_ids(&ex, "FROM Doc RANK BY n LIMIT 10 OFFSET 3"),
            vec![4, 5]
        );
    }

    #[test]
    fn rank_by_applies_before_limit() {
        let (storage, _dir) = setup_storage();
        insert_ranked_node(&storage, 1, 3);
        insert_ranked_node(&storage, 2, 1);
        insert_ranked_node(&storage, 3, 2);
        let ex = Executor::new(&storage);
        // Ascending by n: ids [2(n=1), 3(n=2), 1(n=3)] → top 2 = {2,3}.
        assert_eq!(read_ids(&ex, "FROM Doc RANK BY n LIMIT 2"), vec![2, 3]);
        // Descending by n: ids [1(n=3), 3(n=2), 2(n=1)] → top 2 = {1,3}.
        assert_eq!(read_ids(&ex, "FROM Doc RANK BY n DESC LIMIT 2"), vec![1, 3]);
    }

    #[test]
    fn limit_zero_returns_empty() {
        let (storage, _dir) = setup_storage();
        insert_five_ranked_nodes(&storage);
        let ex = Executor::new(&storage);
        assert_eq!(read_ids(&ex, "FROM Doc LIMIT 0"), Vec::<u128>::new());
    }

    #[test]
    fn limit_applies_after_where_filters() {
        let (storage, _dir) = setup_storage();
        insert_five_ranked_nodes(&storage);
        let ex = Executor::new(&storage);
        // Filter first (n > 1 → ids 2..5), then cap.
        assert_eq!(read_ids(&ex, "FROM Doc WHERE n > 1 LIMIT 2").len(), 2);
        assert_eq!(
            read_ids(&ex, "FROM Doc WHERE n > 4 LIMIT 2"),
            vec![5],
            "filter leaves one row — cap must not invent rows"
        );
    }

    #[test]
    fn select_path_honors_limit_offset() {
        let (storage, _dir) = setup_storage();
        insert_five_ranked_nodes(&storage);
        let ex = Executor::new(&storage);
        assert_eq!(read_ids(&ex, "SELECT * FROM Doc LIMIT 2").len(), 2);
        // SELECT path has no RANK BY; OFFSET alone skips without a cap.
        assert_eq!(read_ids(&ex, "SELECT * FROM Doc OFFSET 4").len(), 1);
    }

    /// Documented interaction: the valid-time filter runs AFTER the plan
    /// (executor.rs `filter_valid_at`), so a LIMIT page can shrink below N
    /// when `AS OF` drops rows — the short page is the end of the walk
    /// (same convention as IQL.md §Page-completeness).
    #[test]
    fn as_of_filter_runs_after_limit_short_page_is_documented() {
        let (storage, _dir) = setup_storage();
        insert_ranked_node(&storage, 1, 1);
        insert_ranked_node(&storage, 2, 2);
        insert_ranked_node(&storage, 3, 3);
        // Node 2 is not valid at t=1500; nodes 1 and 3 are.
        {
            let mut n2 = UnifiedNode::new(2);
            n2.set_field("type", FieldValue::String("Doc".to_string()));
            n2.set_field("n", FieldValue::Int(2));
            n2.set_field(
                crate::sdk::serialization::FIELD_VALID_AT_MS,
                FieldValue::Int(1000),
            );
            n2.set_field(
                crate::sdk::serialization::FIELD_INVALID_AT_MS,
                FieldValue::Int(1200),
            );
            storage.insert(&n2).expect("re-insert node 2 with window");
        }
        let ex = Executor::new(&storage);
        // Limit takes raw rows 1,2 first; AS OF then drops 2 → short page [1].
        assert_eq!(
            read_ids(&ex, "FROM Doc RANK BY n LIMIT 2 AS OF 1500"),
            vec![1]
        );
        // With a wider window both valid rows come back.
        assert_eq!(
            read_ids(&ex, "FROM Doc RANK BY n LIMIT 3 AS OF 1500"),
            vec![1, 3]
        );
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
            as_of_ms: None,
            limit: None,
            offset: None,
            aggregates: vec![],
            group_by: None,
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
            as_of_ms: None,
            limit: None,
            offset: None,
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

    // ── Admission guard (FIND-555): the budget must be returned on EVERY path ──

    /// The reservation lives in a **global** counter, so a failing query that
    /// does not return it grows the counter until the engine rejects every
    /// subsequent query — including writes — with `Error::ResourceLimit`.
    /// The RAII guard is what makes that impossible; this pins its contract.
    #[test]
    #[serial_test::serial]
    fn test_allocation_guard_frees_on_every_exit() {
        use crate::governor::{ResourceGovernor, ALLOCATED_BYTES};
        use std::sync::atomic::Ordering as AtomicOrdering;

        let gov = ResourceGovernor::new(2 * GIB, 50);
        ALLOCATED_BYTES.store(0, AtomicOrdering::SeqCst);
        gov.request_allocation(4096)
            .expect("the reservation must be granted");

        {
            let _reservation = AllocationGuard {
                governor: &gov,
                bytes: 4096,
            };
            assert!(
                ALLOCATED_BYTES.load(AtomicOrdering::SeqCst) >= 4096,
                "the guard must hold the reservation while it is alive"
            );
        }
        assert_eq!(
            ALLOCATED_BYTES.load(AtomicOrdering::SeqCst),
            0,
            "the reservation must be freed when the guard is dropped"
        );

        // FIND-572: freeing more than was reserved must not wrap the counter
        // (a wrapped counter rejects every later query with ResourceLimit).
        gov.free_allocation(4096);
        assert_eq!(
            ALLOCATED_BYTES.load(AtomicOrdering::SeqCst),
            0,
            "an over-free must saturate at zero, never wrap"
        );
        ALLOCATED_BYTES.store(0, AtomicOrdering::SeqCst);
    }

    // ── execute_hybrid: aggregations (WIRE-13) ──

    /// Node shaped like an ERP invoice: `type` + `category` + integer `amount`.
    fn insert_invoice(storage: &StorageEngine, id: u128, category: &str, amount: i64) {
        let mut node = UnifiedNode::new(id);
        node.set_field("type", FieldValue::String("Invoice".to_string()));
        node.set_field("category", FieldValue::String(category.to_string()));
        node.set_field("amount", FieldValue::Int(amount));
        storage.insert(&node).expect("insert invoice");
    }

    fn read_rows(ex: &Executor<'_>, query: &str) -> Vec<UnifiedNode> {
        match ex.execute_hybrid(query).expect("execute") {
            ExecutionResult::Read(nodes) => nodes,
            other => panic!("expected Read, got {other:?}"),
        }
    }

    #[test]
    fn execute_hybrid_select_count_star_aggregates_all_rows() {
        let (storage, _dir) = setup_storage();
        insert_invoice(&storage, 1, "a", 10);
        insert_invoice(&storage, 2, "b", 20);
        insert_invoice(&storage, 3, "a", 30);
        let ex = Executor::new(&storage);

        let rows = read_rows(&ex, "SELECT COUNT(*) FROM Invoice");
        assert_eq!(rows.len(), 1, "global aggregate yields one row");
        assert_eq!(rows[0].relational.get("count"), Some(&FieldValue::Int(3)));
    }

    #[test]
    fn execute_hybrid_select_group_by_category_with_sum() {
        let (storage, _dir) = setup_storage();
        insert_invoice(&storage, 1, "a", 10);
        insert_invoice(&storage, 2, "b", 20);
        insert_invoice(&storage, 3, "a", 30);
        let ex = Executor::new(&storage);

        let rows = read_rows(
            &ex,
            "SELECT COUNT(*), SUM(amount) FROM Invoice GROUP BY category",
        );
        assert_eq!(rows.len(), 2, "one row per category");
        // First-seen order: "a" appears first.
        assert_eq!(
            rows[0].relational.get("category"),
            Some(&FieldValue::String("a".into()))
        );
        assert_eq!(rows[0].relational.get("count"), Some(&FieldValue::Int(2)));
        assert_eq!(
            rows[0].relational.get("sum_amount"),
            Some(&FieldValue::Int(40))
        );
        assert_eq!(
            rows[1].relational.get("category"),
            Some(&FieldValue::String("b".into()))
        );
        assert_eq!(
            rows[1].relational.get("sum_amount"),
            Some(&FieldValue::Int(20))
        );
    }

    #[test]
    fn execute_hybrid_where_then_aggregate_filters_before_grouping() {
        let (storage, _dir) = setup_storage();
        insert_invoice(&storage, 1, "a", 10);
        insert_invoice(&storage, 2, "a", 30);
        let ex = Executor::new(&storage);

        let rows = read_rows(&ex, "SELECT SUM(amount) FROM Invoice WHERE amount > 10");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].relational.get("sum_amount"),
            Some(&FieldValue::Int(30)),
            "WHERE applies before aggregation"
        );
    }

    #[test]
    fn execute_hybrid_count_field_ignores_missing_and_null() {
        let (storage, _dir) = setup_storage();
        insert_invoice(&storage, 1, "a", 10); // has `amount`
                                              // Node without `amount` and node with explicit null: neither counts.
        let mut bare = UnifiedNode::new(2);
        bare.set_field("type", FieldValue::String("Invoice".to_string()));
        storage.insert(&bare).expect("insert bare");
        let mut null_amount = UnifiedNode::new(3);
        null_amount.set_field("type", FieldValue::String("Invoice".to_string()));
        null_amount.set_field("amount", FieldValue::Null);
        storage.insert(&null_amount).expect("insert null");
        let ex = Executor::new(&storage);

        let rows = read_rows(&ex, "SELECT COUNT(amount) FROM Invoice");
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].relational.get("count_amount"),
            Some(&FieldValue::Int(1)),
            "only the row with a present, non-null amount counts"
        );
    }

    #[test]
    fn execute_hybrid_join_then_group_by_aggregates_merged_rows() {
        // Aggregation wraps the full chain (JOIN included): group by a merged
        // field across the joined rows.
        let (storage, _dir) = setup_storage();
        for (id, addr) in [(1u128, 10i64), (2, 10), (3, 20)] {
            let mut n = UnifiedNode::new(id);
            n.set_field("type", FieldValue::String("Person".to_string()));
            n.set_field("addr_id", FieldValue::Int(addr));
            storage.insert(&n).expect("insert person");
        }
        for (id, city) in [(10u128, "ccs"), (20, "mcy")] {
            let mut n = UnifiedNode::new(id);
            n.set_field("type", FieldValue::String("Address".to_string()));
            n.set_field("city", FieldValue::String(city.to_string()));
            // The join condition `p.addr_id = a.id` resolves the relational
            // field `id` (not the node id).
            n.set_field("id", FieldValue::Int(id as i64));
            storage.insert(&n).expect("insert address");
        }
        let ex = Executor::new(&storage);

        let rows = read_rows(
            &ex,
            "SELECT COUNT(*) FROM Person p JOIN Address a ON p.addr_id = a.id GROUP BY city",
        );
        assert_eq!(rows.len(), 2, "one row per joined city");
        assert_eq!(
            rows[0].relational.get("city"),
            Some(&FieldValue::String("ccs".into()))
        );
        assert_eq!(rows[0].relational.get("count"), Some(&FieldValue::Int(2)));
        assert_eq!(
            rows[1].relational.get("city"),
            Some(&FieldValue::String("mcy".into()))
        );
        assert_eq!(rows[1].relational.get("count"), Some(&FieldValue::Int(1)));
    }

    #[test]
    fn aggregation_rejects_trailing_clause_instead_of_silently_dropping_it() {
        // WIRE-13 (review C-1): `GROUP BY c WHERE ...` would otherwise run the
        // WHERE-less aggregate (wrong values) — reject loud at parse time
        // instead of silently returning a result without the clause.
        let (storage, _dir) = setup_storage();
        insert_invoice(&storage, 1, "a", 10);
        let ex = Executor::new(&storage);
        let err = ex
            .execute_hybrid("SELECT COUNT(*) FROM Invoice GROUP BY category WHERE amount > 15")
            .unwrap_err();
        assert!(matches!(err, Error::IqlParse { .. }));
    }

    /// WIRE-13: aggregation restrictions fail loud with stable messages and
    /// positions pointing at the offending clause (never silent wrong results).
    #[test]
    fn execute_hybrid_aggregation_restrictions_report_clear_messages() {
        let (storage, _dir) = setup_storage();
        let ex = Executor::new(&storage);

        // LIMIT with aggregation: position points at the LIMIT clause.
        let err = ex
            .execute_hybrid("SELECT COUNT(*) FROM Invoice LIMIT 5")
            .unwrap_err();
        match err {
            Error::IqlParse { msg, line, col } => {
                assert_eq!(
                    msg,
                    "LIMIT/OFFSET are not supported with aggregation queries"
                );
                assert_eq!(line, 1);
                assert_eq!(col, 29, "position must point at the LIMIT clause");
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }

        // AS OF with aggregation: position points at the AS OF clause.
        let err = ex
            .execute_hybrid("SELECT COUNT(*) FROM Invoice AS OF 5")
            .unwrap_err();
        match err {
            Error::IqlParse { msg, line, col } => {
                assert_eq!(msg, "AS OF is not supported with aggregation queries");
                assert_eq!(line, 1);
                assert_eq!(col, 29, "position must point at the AS OF clause");
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }

        // Mixed plain fields + aggregates: position points at the statement.
        let err = ex
            .execute_hybrid("SELECT name, COUNT(*) FROM Invoice")
            .unwrap_err();
        match err {
            Error::IqlParse { msg, line, col } => {
                assert_eq!(
                    msg,
                    "cannot mix plain fields and aggregate functions in SELECT; group keys come from GROUP BY"
                );
                assert_eq!((line, col), (1, 1));
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }

        // GROUP BY without an aggregate.
        let err = ex
            .execute_hybrid("SELECT * FROM Invoice GROUP BY category")
            .unwrap_err();
        match err {
            Error::IqlParse { msg, line, col } => {
                assert_eq!(
                    msg,
                    "invalid GROUP BY clause: requires at least one aggregate function (COUNT/SUM) and a group field that does not collide with an aggregate output name"
                );
                assert_eq!((line, col), (1, 22));
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }

        // GROUP BY on the FROM/MATCH surface (aggregation is a SELECT surface).
        let err = ex
            .execute_hybrid("FROM Invoice GROUP BY category")
            .unwrap_err();
        match err {
            Error::IqlParse { msg, .. } => {
                assert_eq!(
                    msg,
                    "aggregation (COUNT/SUM/GROUP BY) is only supported in SELECT queries"
                );
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }
    }

    // ── execute_hybrid: FIND-518 / FIND-519 strictness ─────────────

    /// FIND-518: un `WHERE` mal formado es un error de parseo — la query NO
    /// puede ejecutarse como un escaneo sin filtro (que materializaba todos
    /// los nodos en RAM antes de filtrar).
    #[test]
    fn malformed_where_is_a_parse_error_not_an_unfiltered_scan() {
        let (storage, _dir) = setup_storage();
        insert_ranked_node(&storage, 1, 1);
        let ex = Executor::new(&storage);
        let err = ex.execute_hybrid("FROM Doc WHERE n =").unwrap_err();
        match err {
            Error::IqlParse { msg, line, col } => {
                assert!(msg.contains("WHERE"), "message must mention WHERE: {msg}");
                assert_eq!(
                    (line, col),
                    (1, 9),
                    "position must point at the WHERE clause"
                );
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }
    }

    /// FIND-519: el trailing input es un error de parseo, no un descarte
    /// silencioso.
    #[test]
    fn trailing_input_is_a_parse_error() {
        let (storage, _dir) = setup_storage();
        insert_ranked_node(&storage, 1, 1);
        let ex = Executor::new(&storage);
        let err = ex.execute_hybrid("FROM Doc p garbage").unwrap_err();
        match err {
            Error::IqlParse { msg, line, col } => {
                assert!(
                    msg.contains("trailing"),
                    "message must mention the trailing input: {msg}"
                );
                assert_eq!(
                    (line, col),
                    (1, 11),
                    "position must point at the trailing text"
                );
            }
            other => panic!("expected IqlParse, got {other:?}"),
        }
    }
}
