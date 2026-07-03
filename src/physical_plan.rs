//! Physical query plan operators executed against storage.
//!
//! [`PhysicalScan`] and related operators translate logical plan nodes
//! into concrete storage reads, filtering, and projection.

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;
use crate::storage::StorageEngine;

// ─── Physical Scan Operator ──────────────────────────────────

pub struct PhysicalScan<'a> {
    storage: &'a StorageEngine,
    entity: String,
    prefetched: Vec<UnifiedNode>,
    cursor: usize,
}

impl<'a> PhysicalScan<'a> {
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
            if let Ok(id) = parts[1].parse::<u64>() {
                let raw = self
                    .storage
                    .backend
                    .get(crate::backend::BackendPartition::Default, &id.to_le_bytes())?;
                if raw.is_some() {
                    self.prefetched = self.storage.get_many(&[id])?;
                }
                return Ok(());
            }
        }

        let records = self
            .storage
            .backend
            .scan(crate::backend::BackendPartition::Default)?;
        let ids: Vec<u64> = records
            .iter()
            .filter_map(|(key_bytes, _)| {
                let arr: [u8; 8] = key_bytes.as_slice().try_into().ok()?;
                Some(u64::from_le_bytes(arr))
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

pub struct PhysicalFilter<'a> {
    child: Box<dyn PhysicalOperator + 'a>,
    field: String,
    op: crate::query::RelOp,
    value: crate::node::FieldValue,
}

impl<'a> PhysicalFilter<'a> {
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

// ─── Physical Vector Search Operator ─────────────────────────

pub struct PhysicalVectorSearch<'a> {
    storage: &'a StorageEngine,
    #[allow(dead_code)]
    query_vec_text: String,
    min_score: f32,
    results: Vec<u64>,
    prefetched: Vec<UnifiedNode>,
    cursor: usize,
}

impl<'a> PhysicalVectorSearch<'a> {
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
            let llm = crate::llm::LlmClient::new();
            if let Ok(vec) = llm.generate_embedding(&self.query_vec_text) {
                vector = Some(vec);
            }
        }

        if let Some(vec) = vector {
            let neighbors = {
                let index = self.storage.hnsw.load();
                let vs = self.storage.vector_store.read();
                index.search_nearest(&vec, None, None, 0, 5, Some(&vs))
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

pub struct PhysicalProject<'a> {
    child: Box<dyn PhysicalOperator + 'a>,
    fields: Vec<String>,
}

impl<'a> PhysicalProject<'a> {
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

pub struct PhysicalLimit<'a> {
    child: Box<dyn PhysicalOperator + 'a>,
    limit: usize,
    count: usize,
}

impl<'a> PhysicalLimit<'a> {
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

pub struct PhysicalSort<'a> {
    child: Box<dyn PhysicalOperator + 'a>,
    field: String,
    desc: bool,
    nodes: Vec<UnifiedNode>,
    cursor: usize,
}

impl<'a> PhysicalSort<'a> {
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

// ─── Physical Vector Refine Operator (Brute Force Sim Check) ───

pub struct PhysicalVectorRefine<'a> {
    child: Box<dyn PhysicalOperator + 'a>,
    #[allow(dead_code)]
    query_vec_text: String,
    min_score: f32,
    query_vector: Option<crate::node::VectorRepresentations>,
}

impl<'a> PhysicalVectorRefine<'a> {
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
            let llm = crate::llm::LlmClient::new();
            if let Ok(vec) = llm.generate_embedding(&self.query_vec_text) {
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
}
