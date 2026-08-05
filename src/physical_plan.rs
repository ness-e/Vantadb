//! Physical query plan operators executed against storage.
//!
//! [`PhysicalScan`] and related operators translate logical plan nodes
//! into concrete storage reads, filtering, and projection.

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;
use crate::storage::StorageEngine;

// ─── Physical Scan Operator ──────────────────────────────────

/// Physical scan operator that iterates all nodes of a given entity type.
pub struct PhysicalScan<'a> {
    /// Storage engine reference.
    storage: &'a StorageEngine,
    /// Entity type to scan.
    entity: String,
    /// Pre-fetched nodes.
    prefetched: Vec<UnifiedNode>,
    /// Current position in the prefetched list.
    cursor: usize,
}

impl<'a> PhysicalScan<'a> {
    /// Create a new scan operator for the given entity.
    pub fn new(storage: &'a StorageEngine, entity: String) -> Self {
        Self {
            storage,
            entity,
            prefetched: Vec::new(),
            cursor: 0,
        }
    }
}

impl PhysicalOperator for PhysicalScan<'_> {
    fn open(&mut self) -> Result<()> {
        self.prefetched.clear();
        self.cursor = 0;

        let parts: Vec<&str> = self.entity.split('#').collect();
        if parts.len() == 2 {
            if let Ok(id) = parts[1].parse::<u128>() {
                self.prefetched = self.storage.get_many(&[id])?;
                return Ok(());
            }
        }

        let records = self
            .storage
            .backend
            .scan(crate::backend::BackendPartition::Default)?;
        let ids: Vec<u128> = records
            .iter()
            .filter_map(|(key_bytes, _)| {
                let arr: [u8; 16] = key_bytes.as_slice().try_into().ok()?;
                Some(u128::from_le_bytes(arr))
            })
            .collect();
        self.prefetched = self.storage.get_many(&ids)?;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while self.cursor < self.prefetched.len() {
            let node = &self.prefetched[self.cursor];
            self.cursor += 1;

            if self.storage.is_deleted(node.id)? {
                continue;
            }

            if self.entity.contains('#') || self.entity == "*" {
                return Ok(Some(node.clone()));
            }
            if let Some(crate::node::FieldValue::String(t)) = node.relational.get("type") {
                if t == &self.entity {
                    return Ok(Some(node.clone()));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.prefetched.clear();
        Ok(())
    }
}

// ─── Physical Filter Operator ────────────────────────────────

/// Physical filter operator that evaluates relational conditions.
pub struct PhysicalFilter<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Field to filter on.
    field: String,
    /// Comparison operator.
    op: crate::query::RelOp,
    /// Expected value.
    value: crate::node::FieldValue,
}

impl<'a> PhysicalFilter<'a> {
    /// Create a new filter operator wrapping a child operator.
    pub fn new(
        child: Box<dyn PhysicalOperator + 'a>,
        field: String,
        op: crate::query::RelOp,
        value: crate::node::FieldValue,
    ) -> Self {
        Self {
            child,
            field,
            op,
            value,
        }
    }
}

impl PhysicalOperator for PhysicalFilter<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while let Some(node) = self.child.next()? {
            if evaluate_condition(&node, &self.field, &self.op, &self.value) {
                return Ok(Some(node));
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}

fn evaluate_condition(
    node: &UnifiedNode,
    field: &str,
    op: &crate::query::RelOp,
    expected: &crate::node::FieldValue,
) -> bool {
    if let Some(actual) = node.relational.get(field) {
        match (actual, expected) {
            (crate::node::FieldValue::String(a), crate::node::FieldValue::String(e)) => match op {
                crate::query::RelOp::Eq => a == e,
                crate::query::RelOp::Neq => a != e,
                crate::query::RelOp::Gt => a > e,
                crate::query::RelOp::Gte => a >= e,
                crate::query::RelOp::Lt => a < e,
                crate::query::RelOp::Lte => a <= e,
            },
            (crate::node::FieldValue::Int(a), crate::node::FieldValue::Int(e)) => match op {
                crate::query::RelOp::Eq => a == e,
                crate::query::RelOp::Neq => a != e,
                crate::query::RelOp::Gt => a > e,
                crate::query::RelOp::Gte => a >= e,
                crate::query::RelOp::Lt => a < e,
                crate::query::RelOp::Lte => a <= e,
            },
            (crate::node::FieldValue::Float(a), crate::node::FieldValue::Float(e)) => match op {
                crate::query::RelOp::Eq => a == e,
                crate::query::RelOp::Neq => a != e,
                crate::query::RelOp::Gt => a > e,
                crate::query::RelOp::Gte => a >= e,
                crate::query::RelOp::Lt => a < e,
                crate::query::RelOp::Lte => a <= e,
            },
            (crate::node::FieldValue::Bool(a), crate::node::FieldValue::Bool(e)) => match op {
                crate::query::RelOp::Eq => a == e,
                crate::query::RelOp::Neq => a != e,
                _ => false,
            },
            (crate::node::FieldValue::Null, crate::node::FieldValue::Null) => match op {
                crate::query::RelOp::Eq => true,
                crate::query::RelOp::Neq => false,
                _ => false,
            },
            _ => false,
        }
    } else {
        matches!(op, crate::query::RelOp::Neq)
    }
}

// ─── Physical Text Filter Operator ────────────────────────────

/// Physical text filter operator that evaluates lexical text conditions
/// (phrase-aware) against a node's string field.
pub struct PhysicalTextFilter<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Field to filter on.
    field: String,
    /// Text query (quoted phrases preserved).
    query: String,
}

impl<'a> PhysicalTextFilter<'a> {
    /// Create a new text filter operator wrapping a child operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, field: String, query: String) -> Self {
        Self { child, field, query }
    }
}

impl PhysicalOperator for PhysicalTextFilter<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while let Some(node) = self.child.next()? {
            if let Some(crate::node::FieldValue::String(value)) = node.relational.get(&self.field) {
                if crate::text_index::text_contains_query(value, &self.query) {
                    return Ok(Some(node));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}

// ─── Physical Vector Search Operator ─────────────────────────

/// Physical vector search operator using HNSW index.
pub struct PhysicalVectorSearch<'a> {
    /// Storage engine reference.
    storage: &'a StorageEngine,
    /// Text query to embed.
    #[allow(dead_code)]
    query_vec_text: String,
    /// Minimum similarity score threshold.
    min_score: f32,
    /// Result node IDs from HNSW search.
    results: Vec<u128>,
    /// Pre-fetched nodes.
    prefetched: Vec<UnifiedNode>,
    /// Current position in the prefetched list.
    cursor: usize,
}

impl<'a> PhysicalVectorSearch<'a> {
    /// Create a new vector search operator.
    pub fn new(storage: &'a StorageEngine, query_text: String, min_score: f32) -> Self {
        Self {
            storage,
            query_vec_text: query_text,
            min_score,
            results: Vec::new(),
            prefetched: Vec::new(),
            cursor: 0,
        }
    }
}

impl PhysicalOperator for PhysicalVectorSearch<'_> {
    fn open(&mut self) -> Result<()> {
        self.results.clear();
        self.prefetched.clear();
        self.cursor = 0;

        #[allow(unused_mut)]
        let mut vector: Option<Vec<f32>> = None;

        #[cfg(feature = "remote-inference")]
        {
            let provider = crate::llm::get_embedding_provider();
            if let Ok(vec) = provider.embed(&self.query_vec_text) {
                vector = Some(vec);
            }
        }

        if let Some(vec) = vector {
            let neighbors = {
                let index = self.storage.hnsw.load();
                let vs = self.storage.vector_store[0].read();
                index.search_nearest(&vec, None, None, &crate::node::ALL_BITSET, 5, Some(&vs))
            };
            for (id, score) in neighbors {
                if score >= self.min_score {
                    self.results.push(id);
                }
            }
        }

        self.prefetched = self.storage.get_many(&self.results)?;

        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while self.cursor < self.prefetched.len() {
            let node = &self.prefetched[self.cursor];
            self.cursor += 1;

            if self.storage.is_deleted(node.id)? {
                continue;
            }

            return Ok(Some(node.clone()));
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.results.clear();
        self.prefetched.clear();
        Ok(())
    }
}

// ─── Physical Project Operator ───────────────────────────────

/// Physical project operator that narrows fields on each node.
pub struct PhysicalProject<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Fields to retain.
    fields: Vec<String>,
}

impl<'a> PhysicalProject<'a> {
    /// Create a new project operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, fields: Vec<String>) -> Self {
        Self { child, fields }
    }
}

impl PhysicalOperator for PhysicalProject<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        if let Some(mut node) = self.child.next()? {
            let mut projected = std::collections::BTreeMap::new();
            for field in &self.fields {
                if let Some(val) = node.relational.remove(field) {
                    projected.insert(field.clone(), val);
                }
            }
            node.relational = projected;
            return Ok(Some(node));
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}

// ─── Physical Limit Operator ─────────────────────────────────

/// Physical limit operator that caps the number of returned nodes.
pub struct PhysicalLimit<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Maximum number of rows.
    limit: usize,
    /// Number of rows emitted so far.
    count: usize,
}

impl<'a> PhysicalLimit<'a> {
    /// Create a new limit operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, limit: usize) -> Self {
        Self {
            child,
            limit,
            count: 0,
        }
    }
}

impl PhysicalOperator for PhysicalLimit<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.count = 0;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        if self.count >= self.limit {
            return Ok(None);
        }
        if let Some(node) = self.child.next()? {
            self.count += 1;
            return Ok(Some(node));
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()
    }
}

// ─── Physical Sort Operator ──────────────────────────────────

/// Physical sort operator that sorts nodes by a relational field.
pub struct PhysicalSort<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Sort field.
    field: String,
    /// Sort descending.
    desc: bool,
    /// Buffered nodes to sort.
    nodes: Vec<UnifiedNode>,
    /// Current position in sorted nodes.
    cursor: usize,
}

impl<'a> PhysicalSort<'a> {
    /// Create a new sort operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, field: String, desc: bool) -> Self {
        Self {
            child,
            field,
            desc,
            nodes: Vec::new(),
            cursor: 0,
        }
    }
}

impl PhysicalOperator for PhysicalSort<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.nodes.clear();
        self.cursor = 0;

        while let Some(node) = self.child.next()? {
            self.nodes.push(node);
        }

        let field = &self.field;
        let desc = self.desc;
        self.nodes.sort_by(|a, b| {
            let a_val = a.relational.get(field);
            let b_val = b.relational.get(field);
            let cmp = match (a_val, b_val) {
                (
                    Some(crate::node::FieldValue::String(av)),
                    Some(crate::node::FieldValue::String(bv)),
                ) => av.cmp(bv),
                (
                    Some(crate::node::FieldValue::Int(av)),
                    Some(crate::node::FieldValue::Int(bv)),
                ) => av.cmp(bv),
                (
                    Some(crate::node::FieldValue::Float(av)),
                    Some(crate::node::FieldValue::Float(bv)),
                ) => av.partial_cmp(bv).unwrap_or(std::cmp::Ordering::Equal),
                (
                    Some(crate::node::FieldValue::Bool(av)),
                    Some(crate::node::FieldValue::Bool(bv)),
                ) => av.cmp(bv),
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
                _ => std::cmp::Ordering::Equal,
            };
            if desc {
                cmp.reverse()
            } else {
                cmp
            }
        });

        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        if self.cursor < self.nodes.len() {
            let node = self.nodes[self.cursor].clone();
            self.cursor += 1;
            return Ok(Some(node));
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.nodes.clear();
        self.child.close()
    }
}

// ─── Physical Nested Loop Join Operator ────────────────────────

// ponytail: NestedLoopJoin — simplest correct. Add HashJoin when perf warrants it.
// ponytail: Merges all relational fields (left precedence on name collision).
// ponytail: No predicate pushdown yet — all WHERE filters apply post-join.

/// Strip alias prefix from a qualified field reference (e.g. `"p.name"` → `"name"`).
fn strip_alias(field_ref: &str) -> &str {
    field_ref.split('.').next_back().unwrap_or(field_ref)
}

/// Physical nested-loop join operator that iterates left entities and,
/// for each left entity, scans all right entities applying the ON condition.
pub struct PhysicalNestedLoopJoin<'a> {
    /// Left child operator.
    left_child: Box<dyn PhysicalOperator + 'a>,
    /// Right child operator.
    right_child: Box<dyn PhysicalOperator + 'a>,
    /// Left-side join field (alias-qualified, e.g. `"p.addr_id"`).
    left_field: String,
    /// Right-side join field (alias-qualified, e.g. `"a.id"`).
    right_field: String,
    /// All right-side nodes buffered during open().
    right_nodes: Vec<UnifiedNode>,
    /// Index into right_nodes for the current left row.
    right_cursor: usize,
    /// Current left node being probed.
    current_left: Option<UnifiedNode>,
}

impl<'a> PhysicalNestedLoopJoin<'a> {
    /// Create a new nested-loop join operator.
    pub fn new(
        left_child: Box<dyn PhysicalOperator + 'a>,
        right_child: Box<dyn PhysicalOperator + 'a>,
        left_field: String,
        right_field: String,
    ) -> Self {
        Self {
            left_child,
            right_child,
            left_field,
            right_field,
            right_nodes: Vec::new(),
            right_cursor: 0,
            current_left: None,
        }
    }
}

impl PhysicalOperator for PhysicalNestedLoopJoin<'_> {
    fn open(&mut self) -> Result<()> {
        self.left_child.open()?;
        self.right_child.open()?;

        // Buffer all right-side nodes (needed for repeated probing per left row)
        self.right_nodes.clear();
        while let Some(node) = self.right_child.next()? {
            self.right_nodes.push(node);
        }
        self.right_child.close()?;

        self.right_cursor = 0;
        self.current_left = None;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        let left_field = strip_alias(&self.left_field).to_string();
        let right_field = strip_alias(&self.right_field).to_string();

        loop {
            // If we don't have a current left row, fetch one
            if self.current_left.is_none() {
                match self.left_child.next()? {
                    Some(node) => {
                        self.current_left = Some(node);
                        self.right_cursor = 0;
                    }
                    None => return Ok(None), // left side exhausted
                }
            }

            let left = self.current_left.as_ref().unwrap();

            // Probe right nodes
            while self.right_cursor < self.right_nodes.len() {
                let right = &self.right_nodes[self.right_cursor];
                self.right_cursor += 1;

                // Evaluate ON condition: left_field = right_field
                let left_val = left.relational.get(&left_field);
                let right_val = right.relational.get(&right_field);

                let matches = match (left_val, right_val) {
                    (Some(lv), Some(rv)) => compare_field_values(lv, rv, &crate::query::RelOp::Eq),
                    _ => false,
                };

                if matches {
                    // Merge fields: left fields first, then right fields (left priority on conflict)
                    let mut merged = UnifiedNode::new(left.id);
                    merged.relational = left.relational.clone();
                    for (k, v) in &right.relational {
                        // Use BTreeMap entry API to avoid overwriting left fields
                        if !merged.relational.contains_key(k) {
                            merged.relational.insert(k.clone(), v.clone());
                        }
                    }
                    return Ok(Some(merged));
                }
            }

            // Exhausted right nodes for this left row; advance to next left
            self.current_left = None;
        }
    }

    fn close(&mut self) -> Result<()> {
        self.left_child.close()?;
        self.right_nodes.clear();
        self.current_left = None;
        Ok(())
    }
}

// ─── Physical Subquery Filter Operator ──────────────────────────

/// Physical subquery filter that evaluates a scalar subquery during `open()`
/// then filters child rows against the computed scalar value.
pub struct PhysicalSubqueryFilter<'a> {
    /// Child operator whose rows are filtered.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Subquery plan to evalute once during open().
    subquery_plan: Box<dyn PhysicalOperator + 'a>,
    /// Field to compare (alias-qualified).
    field: String,
    /// Comparison operator.
    op: crate::query::RelOp,
    /// Scalar value obtained by executing the subquery.
    scalar_value: Option<crate::node::FieldValue>,
}

impl<'a> PhysicalSubqueryFilter<'a> {
    /// Create a new subquery filter operator.
    pub fn new(
        child: Box<dyn PhysicalOperator + 'a>,
        subquery_plan: Box<dyn PhysicalOperator + 'a>,
        field: String,
        op: crate::query::RelOp,
    ) -> Self {
        Self {
            child,
            subquery_plan,
            field,
            op,
            scalar_value: None,
        }
    }
}

impl PhysicalOperator for PhysicalSubqueryFilter<'_> {
    fn open(&mut self) -> Result<()> {
        // Execute the subquery plan to get the scalar value
        self.subquery_plan.open()?;
        let mut subq_nodes: Vec<UnifiedNode> = Vec::new();
        while let Some(node) = self.subquery_plan.next()? {
            subq_nodes.push(node);
        }
        self.subquery_plan.close()?;

        // Take the first field value from the first result node as the scalar
        self.scalar_value = subq_nodes
            .first()
            .and_then(|n| n.relational.values().next().cloned());

        // Open child for iteration
        self.child.open()?;
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        let sv = match &self.scalar_value {
            Some(v) => v,
            None => {
                // Subquery returned no rows — compare against Null
                // For Eq, no rows = no match. For Neq, all rows match.
                // Simplest: treat empty as Null.
                return self.child.next(); // pass through, no scalar to compare
            }
        };

        let field_name = strip_alias(&self.field);
        while let Some(node) = self.child.next()? {
            if let Some(actual) = node.relational.get(field_name) {
                if compare_field_values(actual, sv, &self.op) {
                    return Ok(Some(node));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.child.close()?;
        Ok(())
    }
}

/// Compare two field values using the given relational operator.
fn compare_field_values(
    a: &crate::node::FieldValue,
    b: &crate::node::FieldValue,
    op: &crate::query::RelOp,
) -> bool {
    match (a, b) {
        (crate::node::FieldValue::String(a), crate::node::FieldValue::String(e)) => match op {
            crate::query::RelOp::Eq => a == e,
            crate::query::RelOp::Neq => a != e,
            crate::query::RelOp::Gt => a > e,
            crate::query::RelOp::Gte => a >= e,
            crate::query::RelOp::Lt => a < e,
            crate::query::RelOp::Lte => a <= e,
        },
        (crate::node::FieldValue::Int(a), crate::node::FieldValue::Int(e)) => match op {
            crate::query::RelOp::Eq => a == e,
            crate::query::RelOp::Neq => a != e,
            crate::query::RelOp::Gt => a > e,
            crate::query::RelOp::Gte => a >= e,
            crate::query::RelOp::Lt => a < e,
            crate::query::RelOp::Lte => a <= e,
        },
        (crate::node::FieldValue::Float(a), crate::node::FieldValue::Float(e)) => match op {
            crate::query::RelOp::Eq => a == e,
            crate::query::RelOp::Neq => a != e,
            crate::query::RelOp::Gt => a > e,
            crate::query::RelOp::Gte => a >= e,
            crate::query::RelOp::Lt => a < e,
            crate::query::RelOp::Lte => a <= e,
        },
        (crate::node::FieldValue::Bool(a), crate::node::FieldValue::Bool(e)) => match op {
            crate::query::RelOp::Eq => a == e,
            crate::query::RelOp::Neq => a != e,
            _ => false,
        },
        (crate::node::FieldValue::Null, crate::node::FieldValue::Null) => match op {
            crate::query::RelOp::Eq => true,
            crate::query::RelOp::Neq => false,
            _ => false,
        },
        _ => false,
    }
}

// ─── Physical Vector Refine Operator (Brute Force Sim Check) ───

/// Physical vector refine operator that brute-force filters by cosine similarity.
pub struct PhysicalVectorRefine<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Text query to embed.
    #[allow(dead_code)]
    query_vec_text: String,
    /// Minimum similarity score.
    min_score: f32,
    /// Embedded query vector.
    query_vector: Option<crate::node::VectorRepresentations>,
}

impl<'a> PhysicalVectorRefine<'a> {
    /// Create a new vector refine operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, query_text: String, min_score: f32) -> Self {
        Self {
            child,
            query_vec_text: query_text,
            min_score,
            query_vector: None,
        }
    }
}

impl PhysicalOperator for PhysicalVectorRefine<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.query_vector = None;

        #[cfg(feature = "remote-inference")]
        {
            let provider = crate::llm::get_embedding_provider();
            if let Ok(vec) = provider.embed(&self.query_vec_text) {
                self.query_vector = Some(crate::node::VectorRepresentations::Full(vec));
            }
        }
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        let q_vec = match &self.query_vector {
            Some(v) => v,
            None => return self.child.next(),
        };

        while let Some(node) = self.child.next()? {
            if let Some(sim) = node.vector.cosine_similarity(q_vec) {
                if sim >= self.min_score {
                    return Ok(Some(node));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.query_vector = None;
        self.child.close()
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::node::{FieldValue, UnifiedNode};
    use crate::query::RelOp;

    fn node_with_field(key: &str, val: FieldValue) -> UnifiedNode {
        let mut node = UnifiedNode::new(1);
        node.relational.insert(key.into(), val);
        node
    }

    // ── evaluate_condition ──

    #[test]
    fn test_evaluate_condition_eq_string() {
        let node = node_with_field("name", FieldValue::String("alice".into()));
        assert!(evaluate_condition(
            &node,
            "name",
            &RelOp::Eq,
            &FieldValue::String("alice".into())
        ));
        assert!(!evaluate_condition(
            &node,
            "name",
            &RelOp::Eq,
            &FieldValue::String("bob".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_neq_string() {
        let node = node_with_field("name", FieldValue::String("alice".into()));
        assert!(evaluate_condition(
            &node,
            "name",
            &RelOp::Neq,
            &FieldValue::String("bob".into())
        ));
        assert!(!evaluate_condition(
            &node,
            "name",
            &RelOp::Neq,
            &FieldValue::String("alice".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_gt_int() {
        let node = node_with_field("age", FieldValue::Int(30));
        assert!(evaluate_condition(
            &node,
            "age",
            &RelOp::Gt,
            &FieldValue::Int(20)
        ));
        assert!(!evaluate_condition(
            &node,
            "age",
            &RelOp::Gt,
            &FieldValue::Int(30)
        ));
        assert!(!evaluate_condition(
            &node,
            "age",
            &RelOp::Gt,
            &FieldValue::Int(40)
        ));
    }

    #[test]
    fn test_evaluate_condition_gte_int() {
        let node = node_with_field("age", FieldValue::Int(30));
        assert!(evaluate_condition(
            &node,
            "age",
            &RelOp::Gte,
            &FieldValue::Int(30)
        ));
        assert!(evaluate_condition(
            &node,
            "age",
            &RelOp::Gte,
            &FieldValue::Int(20)
        ));
        assert!(!evaluate_condition(
            &node,
            "age",
            &RelOp::Gte,
            &FieldValue::Int(40)
        ));
    }

    #[test]
    fn test_evaluate_condition_lt_float() {
        let node = node_with_field("price", FieldValue::Float(10.5));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Lt,
            &FieldValue::Float(20.0)
        ));
        assert!(!evaluate_condition(
            &node,
            "price",
            &RelOp::Lt,
            &FieldValue::Float(5.0)
        ));
    }

    #[test]
    fn test_evaluate_condition_lte_float() {
        let node = node_with_field("price", FieldValue::Float(10.0));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Lte,
            &FieldValue::Float(10.0)
        ));
        assert!(evaluate_condition(
            &node,
            "price",
            &RelOp::Lte,
            &FieldValue::Float(15.0)
        ));
    }

    #[test]
    fn test_evaluate_condition_bool_eq() {
        let node = node_with_field("active", FieldValue::Bool(true));
        assert!(evaluate_condition(
            &node,
            "active",
            &RelOp::Eq,
            &FieldValue::Bool(true)
        ));
        assert!(!evaluate_condition(
            &node,
            "active",
            &RelOp::Eq,
            &FieldValue::Bool(false)
        ));
    }

    #[test]
    fn test_evaluate_condition_bool_neq() {
        let node = node_with_field("active", FieldValue::Bool(true));
        assert!(evaluate_condition(
            &node,
            "active",
            &RelOp::Neq,
            &FieldValue::Bool(false)
        ));
    }

    #[test]
    fn test_evaluate_condition_bool_non_relational() {
        let node = node_with_field("active", FieldValue::Bool(true));
        assert!(!evaluate_condition(
            &node,
            "active",
            &RelOp::Gt,
            &FieldValue::Bool(false)
        ));
    }

    #[test]
    fn test_evaluate_condition_null_eq() {
        let node = node_with_field("empty", FieldValue::Null);
        assert!(evaluate_condition(
            &node,
            "empty",
            &RelOp::Eq,
            &FieldValue::Null
        ));
        assert!(!evaluate_condition(
            &node,
            "empty",
            &RelOp::Neq,
            &FieldValue::Null
        ));
    }

    #[test]
    fn test_evaluate_condition_missing_field_neq() {
        let node = UnifiedNode::new(1);
        assert!(evaluate_condition(
            &node,
            "missing",
            &RelOp::Neq,
            &FieldValue::String("x".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_missing_field_eq() {
        let node = UnifiedNode::new(1);
        assert!(!evaluate_condition(
            &node,
            "missing",
            &RelOp::Eq,
            &FieldValue::String("x".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_type_mismatch() {
        let node = node_with_field("val", FieldValue::Int(42));
        assert!(!evaluate_condition(
            &node,
            "val",
            &RelOp::Eq,
            &FieldValue::String("42".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_string_gte() {
        let node = node_with_field("name", FieldValue::String("banana".into()));
        assert!(evaluate_condition(
            &node,
            "name",
            &RelOp::Gte,
            &FieldValue::String("apple".into())
        ));
        assert!(evaluate_condition(
            &node,
            "name",
            &RelOp::Gte,
            &FieldValue::String("banana".into())
        ));
        assert!(!evaluate_condition(
            &node,
            "name",
            &RelOp::Gte,
            &FieldValue::String("cherry".into())
        ));
    }

    #[test]
    fn test_evaluate_condition_negative_float() {
        let node = node_with_field("balance", FieldValue::Float(-5.0));
        assert!(evaluate_condition(
            &node,
            "balance",
            &RelOp::Lt,
            &FieldValue::Float(0.0)
        ));
        assert!(evaluate_condition(
            &node,
            "balance",
            &RelOp::Eq,
            &FieldValue::Float(-5.0)
        ));
    }

    #[test]
    fn test_relop_ordering_consistent() {
        let a = FieldValue::Int(50);
        let test_values = [0i64, 50, 100];
        for &v in &test_values {
            let node = node_with_field("x", FieldValue::Int(v));
            let gt = evaluate_condition(&node, "x", &RelOp::Gt, &a);
            let lt = evaluate_condition(&node, "x", &RelOp::Lt, &a);
            if v == 50 {
                assert!(
                    !gt && !lt,
                    "Gt and Lt should both be false for equal v={}",
                    v
                );
            } else {
                assert_ne!(gt, lt, "Gt and Lt should be opposites for v={}", v);
            }
        }
    }

    // ── PhysicalOperator trait object safety ──

    #[test]
    fn test_physical_operator_trait_satisfied() {
        fn _is_send_sync<T: Send + Sync>() {}
        _is_send_sync::<PhysicalFilter>();
        _is_send_sync::<PhysicalProject>();
        _is_send_sync::<PhysicalLimit>();
        _is_send_sync::<PhysicalSort>();
    }

    // ── MockScan helper — controlled child operator ──────────────────────

    /// Mock operator that yields a fixed set of nodes for testing parent operators.
    struct MockScan {
        nodes: Vec<UnifiedNode>,
        saved: Vec<UnifiedNode>,
        cursor: usize,
    }

    impl MockScan {
        fn new(nodes: Vec<UnifiedNode>) -> Self {
            let saved = nodes.clone();
            Self {
                nodes,
                saved,
                cursor: 0,
            }
        }
    }

    impl PhysicalOperator for MockScan {
        fn open(&mut self) -> Result<()> {
            self.nodes = self.saved.clone();
            self.cursor = 0;
            Ok(())
        }

        fn next(&mut self) -> Result<Option<UnifiedNode>> {
            if self.cursor < self.nodes.len() {
                let node = self.nodes[self.cursor].clone();
                self.cursor += 1;
                Ok(Some(node))
            } else {
                Ok(None)
            }
        }

        fn close(&mut self) -> Result<()> {
            self.nodes.clear();
            Ok(())
        }
    }

    fn bool_node(id: u128, active: bool) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.relational
            .insert("active".into(), FieldValue::Bool(active));
        node
    }

    fn int_node(id: u128, val: i64) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.relational.insert("val".into(), FieldValue::Int(val));
        node
    }

    fn string_node(id: u128, name: &str) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.relational
            .insert("name".into(), FieldValue::String(name.into()));
        node
    }

    // ── PhysicalFilter ──────────────────────────────────────────────────

    #[test]
    fn test_physical_filter_matches() {
        let child = MockScan::new(vec![
            bool_node(1, true),
            bool_node(2, false),
            bool_node(3, true),
        ]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "active".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        filter.open().unwrap();
        let r1 = filter.next().unwrap().expect("node 1 matches");
        assert_eq!(r1.id, 1);
        let r2 = filter.next().unwrap().expect("node 3 matches");
        assert_eq!(r2.id, 3);
        assert!(filter.next().unwrap().is_none(), "no more matches");
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_no_match() {
        let child = MockScan::new(vec![bool_node(1, false), bool_node(2, false)]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "active".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        filter.open().unwrap();
        assert!(filter.next().unwrap().is_none(), "no nodes match");
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_empty_child() {
        let child = MockScan::new(vec![]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "active".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        filter.open().unwrap();
        assert!(filter.next().unwrap().is_none());
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_neq() {
        let child = MockScan::new(vec![string_node(1, "alice"), string_node(2, "bob")]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "name".into(),
            RelOp::Neq,
            FieldValue::String("bob".into()),
        );
        filter.open().unwrap();
        assert_eq!(filter.next().unwrap().unwrap().id, 1, "alice != bob");
        assert!(filter.next().unwrap().is_none());
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_int_gt() {
        let child = MockScan::new(vec![int_node(1, 10), int_node(2, 20), int_node(3, 30)]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "val".into(),
            RelOp::Gt,
            FieldValue::Int(15),
        );
        filter.open().unwrap();
        assert_eq!(filter.next().unwrap().unwrap().id, 2);
        assert_eq!(filter.next().unwrap().unwrap().id, 3);
        assert!(filter.next().unwrap().is_none());
        filter.close().unwrap();
    }

    #[test]
    fn test_physical_filter_open_close_cycle() {
        let child = MockScan::new(vec![bool_node(1, true)]);
        let mut filter = PhysicalFilter::new(
            Box::new(child),
            "active".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        filter.open().unwrap();
        assert!(filter.next().unwrap().is_some());
        filter.close().unwrap();
        // Can re-open
        filter.open().unwrap();
        assert!(filter.next().unwrap().is_some());
        filter.close().unwrap();
    }

    // ── PhysicalProject ─────────────────────────────────────────────────

    #[test]
    fn test_physical_project_narrows_fields() {
        let mut node = UnifiedNode::new(1);
        node.relational.insert("a".into(), FieldValue::Int(1));
        node.relational
            .insert("b".into(), FieldValue::String("x".into()));
        node.relational.insert("c".into(), FieldValue::Float(3.0));

        let child = MockScan::new(vec![node]);
        let mut project = PhysicalProject::new(Box::new(child), vec!["a".into(), "c".into()]);
        project.open().unwrap();
        let result = project.next().unwrap().expect("got projected node");
        assert_eq!(result.relational.len(), 2, "only 2 fields retained");
        assert!(result.relational.contains_key("a"));
        assert!(result.relational.contains_key("c"));
        assert!(!result.relational.contains_key("b"), "b removed");
        assert!(project.next().unwrap().is_none());
        project.close().unwrap();
    }

    #[test]
    fn test_physical_project_empty_fields() {
        let mut node = UnifiedNode::new(1);
        node.relational.insert("a".into(), FieldValue::Int(1));

        let child = MockScan::new(vec![node]);
        let mut project = PhysicalProject::new(Box::new(child), vec![]);
        project.open().unwrap();
        let result = project.next().unwrap().expect("got node with no fields");
        assert!(result.relational.is_empty(), "all fields removed");
        project.close().unwrap();
    }

    #[test]
    fn test_physical_project_preserves_only_requested_fields() {
        let mut node = UnifiedNode::new(1);
        node.relational.insert("keep".into(), FieldValue::Int(42));
        node.relational
            .insert("drop".into(), FieldValue::String("gone".into()));

        let child = MockScan::new(vec![node]);
        let mut project = PhysicalProject::new(Box::new(child), vec!["keep".into()]);
        project.open().unwrap();
        let result = project.next().unwrap().unwrap();
        assert_eq!(result.relational.get("keep"), Some(&FieldValue::Int(42)));
        assert!(!result.relational.contains_key("drop"));
        project.close().unwrap();
    }

    #[test]
    fn test_physical_project_empty_child() {
        let child = MockScan::new(vec![]);
        let mut project = PhysicalProject::new(Box::new(child), vec!["a".into()]);
        project.open().unwrap();
        assert!(project.next().unwrap().is_none());
        project.close().unwrap();
    }

    // ── PhysicalLimit ───────────────────────────────────────────────────

    #[test]
    fn test_physical_limit_caps_results() {
        let child = MockScan::new(vec![int_node(1, 1), int_node(2, 2), int_node(3, 3)]);
        let mut limit = PhysicalLimit::new(Box::new(child), 2);
        limit.open().unwrap();
        assert_eq!(limit.next().unwrap().unwrap().id, 1);
        assert_eq!(limit.next().unwrap().unwrap().id, 2);
        assert!(limit.next().unwrap().is_none(), "limit stops at 2");
        limit.close().unwrap();
    }

    #[test]
    fn test_physical_limit_zero() {
        let child = MockScan::new(vec![int_node(1, 1)]);
        let mut limit = PhysicalLimit::new(Box::new(child), 0);
        limit.open().unwrap();
        assert!(limit.next().unwrap().is_none(), "zero limit yields nothing");
        limit.close().unwrap();
    }

    #[test]
    fn test_physical_limit_exact_count() {
        let child = MockScan::new(vec![int_node(1, 1), int_node(2, 2)]);
        let mut limit = PhysicalLimit::new(Box::new(child), 2);
        limit.open().unwrap();
        assert!(limit.next().unwrap().is_some());
        assert!(limit.next().unwrap().is_some());
        assert!(limit.next().unwrap().is_none());
        limit.close().unwrap();
    }

    #[test]
    fn test_physical_limit_more_than_available() {
        let child = MockScan::new(vec![int_node(1, 1)]);
        let mut limit = PhysicalLimit::new(Box::new(child), 10);
        limit.open().unwrap();
        assert!(limit.next().unwrap().is_some(), "one node available");
        assert!(limit.next().unwrap().is_none(), "no more despite limit=10");
        limit.close().unwrap();
    }

    #[test]
    fn test_physical_limit_empty_child() {
        let child = MockScan::new(vec![]);
        let mut limit = PhysicalLimit::new(Box::new(child), 5);
        limit.open().unwrap();
        assert!(limit.next().unwrap().is_none());
        limit.close().unwrap();
    }

    // ── PhysicalSort ────────────────────────────────────────────────────

    #[test]
    fn test_physical_sort_ascending() {
        let child = MockScan::new(vec![int_node(3, 30), int_node(1, 10), int_node(2, 20)]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), false);
        sort.open().unwrap();
        assert_eq!(sort.next().unwrap().unwrap().id, 1, "lowest first");
        assert_eq!(sort.next().unwrap().unwrap().id, 2);
        assert_eq!(sort.next().unwrap().unwrap().id, 3, "highest last");
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_descending() {
        let child = MockScan::new(vec![int_node(3, 30), int_node(1, 10), int_node(2, 20)]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), true);
        sort.open().unwrap();
        assert_eq!(sort.next().unwrap().unwrap().id, 3, "highest first");
        assert_eq!(sort.next().unwrap().unwrap().id, 2);
        assert_eq!(sort.next().unwrap().unwrap().id, 1, "lowest last");
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_by_string() {
        let child = MockScan::new(vec![
            string_node(2, "banana"),
            string_node(1, "apple"),
            string_node(3, "cherry"),
        ]);
        let mut sort = PhysicalSort::new(Box::new(child), "name".into(), false);
        sort.open().unwrap();
        assert_eq!(sort.next().unwrap().unwrap().id, 1, "apple first");
        assert_eq!(sort.next().unwrap().unwrap().id, 2, "banana second");
        assert_eq!(sort.next().unwrap().unwrap().id, 3, "cherry third");
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_missing_field_none_first_asc() {
        let mut n1 = UnifiedNode::new(1);
        n1.relational.insert("val".into(), FieldValue::Int(10));
        let mut n2 = UnifiedNode::new(2);
        n2.relational.insert("x".into(), FieldValue::Int(99));
        let child = MockScan::new(vec![n1, n2]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), false);
        sort.open().unwrap();
        // Nodes without the field sort first (None < Some)
        assert_eq!(
            sort.next().unwrap().unwrap().id,
            2,
            "node2 lacks 'val' → first"
        );
        assert_eq!(sort.next().unwrap().unwrap().id, 1, "node1 has val=10");
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_empty_child() {
        let child = MockScan::new(vec![]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), false);
        sort.open().unwrap();
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_open_close_cycle() {
        let child = MockScan::new(vec![int_node(2, 20), int_node(1, 10)]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), true);
        sort.open().unwrap();
        assert_eq!(sort.next().unwrap().unwrap().id, 2);
        assert_eq!(sort.next().unwrap().unwrap().id, 1);
        assert!(sort.next().unwrap().is_none());
        sort.close().unwrap();
    }

    #[test]
    fn test_physical_sort_deterministic_within_equal_keys() {
        let child = MockScan::new(vec![int_node(2, 30), int_node(1, 10), int_node(3, 20)]);
        let mut sort = PhysicalSort::new(Box::new(child), "val".into(), false);
        sort.open().unwrap();
        let ids: Vec<u128> = std::iter::from_fn(|| sort.next().unwrap().map(|n| n.id)).collect();
        assert_eq!(ids, vec![1, 3, 2], "sorted by val ascending → id order");
        sort.close().unwrap();
    }

    // ── PhysicalVectorRefine (passthrough when no embedding) ────────────

    #[test]
    fn test_physical_vector_refine_passthrough_no_embedding() {
        // Without `remote-inference` feature, query_vector stays None
        // so next() passes through all child nodes unfiltered.
        let child = MockScan::new(vec![bool_node(1, true), bool_node(2, false)]);
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5);
        refine.open().unwrap();
        let r1 = refine.next().unwrap().expect("passthrough node 1");
        assert_eq!(r1.id, 1);
        let r2 = refine.next().unwrap().expect("passthrough node 2");
        assert_eq!(r2.id, 2);
        assert!(refine.next().unwrap().is_none());
        refine.close().unwrap();
    }

    #[test]
    fn test_physical_vector_refine_empty_child() {
        let child = MockScan::new(vec![]);
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5);
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_none());
        refine.close().unwrap();
    }

    #[test]
    fn test_physical_vector_refine_open_close_cycle() {
        let child = MockScan::new(vec![bool_node(1, true)]);
        let mut refine = PhysicalVectorRefine::new(Box::new(child), "query".into(), 0.5);
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_some());
        refine.close().unwrap();
        // Re-open
        refine.open().unwrap();
        assert!(refine.next().unwrap().is_some());
        refine.close().unwrap();
    }

    // ── Open/Close lifecycle on every operator ──────────────────────────

    #[test]
    fn test_all_operators_support_close_without_open() {
        // Verify calling close() on a fresh operator does not panic.
        let mut f = PhysicalFilter::new(
            Box::new(MockScan::new(vec![])),
            "x".into(),
            RelOp::Eq,
            FieldValue::Bool(true),
        );
        f.close().unwrap();

        let mut p = PhysicalProject::new(Box::new(MockScan::new(vec![])), vec![]);
        p.close().unwrap();

        let mut l = PhysicalLimit::new(Box::new(MockScan::new(vec![])), 0);
        l.close().unwrap();

        let mut s = PhysicalSort::new(Box::new(MockScan::new(vec![])), "x".into(), false);
        s.close().unwrap();

        let mut r = PhysicalVectorRefine::new(Box::new(MockScan::new(vec![])), "q".into(), 0.0);
        r.close().unwrap();
    }
}
