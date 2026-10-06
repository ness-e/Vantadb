//! Physical aggregation operator (WIRE-13 registry extension).
//!
//! Blocking Volcano wrapper: drains its child at `open()`, accumulates the
//! registered aggregates per group (first-seen order, same contract as
//! `PhysicalDedup`), and streams one synthetic `UnifiedNode` per group.
//! `id = 0` for every synthetic row (they represent groups, not stored nodes).
//!
//! Leaf note: this file depends only on `query` / `node` / `error` — the
//! registry compiler + cost model live in `operator_registry.rs`, so the
//! dependency runs one way (`registry → physical_plan`, never back).

use std::collections::HashMap;

use crate::error::Result;
use crate::node::{FieldValue, UnifiedNode};
use crate::query::{AggregateFunc, PhysicalOperator};

/// Physical aggregation operator: one output row per group.
///
/// - Without `GROUP BY`: exactly one row over the whole input (COUNT 0 /
///   SUM null when the input is empty — SQL semantics).
/// - With `GROUP BY <field>`: one row per distinct field value, in first-seen
///   order. Missing values and explicit `Null` share one group.
/// - Output fields: the group key (when present) plus one field per aggregate
///   (`AggregateFunc::output_name`). SUM skips non-numeric values (no
///   coercion — type-strict, same convention as `evaluate_condition`).
pub struct PhysicalAggregate<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Aggregate functions to compute, in output order.
    funcs: Vec<AggregateFunc>,
    /// Optional group key field; `None` = single global group.
    group_by: Option<String>,
    /// Output rows materialized at `open()`.
    rows: Vec<UnifiedNode>,
    /// Emission cursor over `rows`.
    cursor: usize,
}

/// Per-aggregate accumulation state for one group.
#[derive(Default)]
struct Accumulator {
    /// `COUNT(*)`: every input row.
    count: u64,
    /// `COUNT(field)`: rows where the field is present and not `Null`.
    count_field: u64,
    /// `SUM`: integer total (saturating) while every contribution is `Int`.
    sum_int: i64,
    /// `SUM`: float total once a `Float` contribution arrives.
    sum_float: f64,
    /// Whether `sum_float` is authoritative (mixed or float-only input).
    sum_is_float: bool,
    /// Whether any numeric value contributed to `SUM`.
    sum_seen: bool,
}

impl Accumulator {
    /// Feed one input row into the per-function state.
    fn add(&mut self, node: &UnifiedNode, func: &AggregateFunc) {
        match func {
            AggregateFunc::Count => self.count += 1,
            AggregateFunc::CountField(field) => {
                if matches!(node.relational.get(field), Some(v) if *v != FieldValue::Null) {
                    self.count_field += 1;
                }
            }
            AggregateFunc::Sum(field) => {
                match node.relational.get(field) {
                    Some(FieldValue::Int(i)) => {
                        self.sum_seen = true;
                        if self.sum_is_float {
                            self.sum_float += *i as f64;
                        } else {
                            self.sum_int = self.sum_int.saturating_add(*i);
                        }
                    }
                    Some(FieldValue::Float(f)) => {
                        self.sum_seen = true;
                        if self.sum_is_float {
                            self.sum_float += *f;
                        } else {
                            self.sum_is_float = true;
                            self.sum_float = self.sum_int as f64 + *f;
                        }
                    }
                    // Missing / Null / non-numeric: ignored (no coercion).
                    _ => {}
                }
            }
        }
    }

    /// Final value for `func` from this accumulator.
    fn finish(&self, func: &AggregateFunc) -> FieldValue {
        match func {
            AggregateFunc::Count => FieldValue::Int(self.count as i64),
            AggregateFunc::CountField(_) => FieldValue::Int(self.count_field as i64),
            AggregateFunc::Sum(_) => {
                if !self.sum_seen {
                    FieldValue::Null
                } else if self.sum_is_float {
                    FieldValue::Float(self.sum_float)
                } else {
                    FieldValue::Int(self.sum_int)
                }
            }
        }
    }
}

impl<'a> PhysicalAggregate<'a> {
    /// Create a new aggregation operator over `child`.
    pub fn new(
        child: Box<dyn PhysicalOperator + 'a>,
        funcs: Vec<AggregateFunc>,
        group_by: Option<String>,
    ) -> Self {
        Self {
            child,
            funcs,
            group_by,
            rows: Vec::new(),
            cursor: 0,
        }
    }
}

impl PhysicalOperator for PhysicalAggregate<'_> {
    // ponytail: blocking materialization — groups are accumulated in memory at
    // `open()` (group cardinality is data-dependent; the Volcano stream has no
    // spill). Upgrade path: hash-partitioned streaming if a bench ever shows
    // this as a bottleneck (Regla 9: no benchmark, no claim).
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.rows.clear();
        self.cursor = 0;

        // Drain the child into per-group accumulators (first-seen order).
        let mut order: Vec<Option<FieldValue>> = Vec::new();
        let mut groups: HashMap<Option<FieldValue>, Vec<Accumulator>> = HashMap::new();
        while let Some(node) = self.child.next()? {
            let key: Option<FieldValue> = match &self.group_by {
                Some(field) => Some(
                    node.relational
                        .get(field)
                        .cloned()
                        .unwrap_or(FieldValue::Null),
                ),
                None => None,
            };
            let states = groups.entry(key.clone()).or_insert_with(|| {
                order.push(key.clone());
                (0..self.funcs.len())
                    .map(|_| Accumulator::default())
                    .collect()
            });
            for (state, func) in states.iter_mut().zip(&self.funcs) {
                state.add(&node, func);
            }
        }
        // Note: the child is NOT closed here — `close()` propagates to it
        // (same convention as `PhysicalSort`, the other blocking operator).

        // Without GROUP BY the global aggregate always yields exactly one row
        // (SQL: COUNT over an empty input is 0, SUM is null).
        if self.group_by.is_none() && groups.is_empty() {
            order.push(None);
            groups.insert(
                None,
                (0..self.funcs.len())
                    .map(|_| Accumulator::default())
                    .collect(),
            );
        }

        // Materialize one synthetic row per group.
        for key in order {
            let states = groups.remove(&key).unwrap_or_default();
            let mut row = UnifiedNode::new(0);
            if let Some(field) = &self.group_by {
                row.relational
                    .insert(field.clone(), key.clone().unwrap_or(FieldValue::Null));
            }
            for (state, func) in states.iter().zip(&self.funcs) {
                row.relational
                    .insert(func.output_name(), state.finish(func));
            }
            self.rows.push(row);
        }
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        if self.cursor < self.rows.len() {
            let row = self.rows[self.cursor].clone();
            self.cursor += 1;
            return Ok(Some(row));
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.rows.clear();
        self.cursor = 0;
        self.child.close()
    }
}

#[cfg(test)]
#[allow(missing_docs)]
mod tests {
    use super::*;
    use crate::error::Result as CrateResult;
    use crate::node::{FieldValue, UnifiedNode};
    use crate::query::AggregateFunc;

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
        fn open(&mut self) -> CrateResult<()> {
            self.nodes = self.saved.clone();
            self.cursor = 0;
            Ok(())
        }
        fn next(&mut self) -> CrateResult<Option<UnifiedNode>> {
            if self.cursor < self.nodes.len() {
                let node = self.nodes[self.cursor].clone();
                self.cursor += 1;
                Ok(Some(node))
            } else {
                Ok(None)
            }
        }
        fn close(&mut self) -> CrateResult<()> {
            self.nodes.clear();
            Ok(())
        }
    }

    fn node_with(id: u128, key: &str, val: FieldValue) -> UnifiedNode {
        let mut node = UnifiedNode::new(id);
        node.relational.insert(key.into(), val);
        node
    }

    fn aggregate(
        child: MockScan,
        funcs: Vec<AggregateFunc>,
        group_by: Option<&str>,
    ) -> PhysicalAggregate<'static> {
        PhysicalAggregate::new(Box::new(child), funcs, group_by.map(String::from))
    }

    fn drain(op: &mut dyn PhysicalOperator) -> Vec<UnifiedNode> {
        let mut out = Vec::new();
        while let Some(node) = op.next().unwrap() {
            out.push(node);
        }
        out
    }

    #[test]
    fn aggregate_counts_all_rows_without_group_by() {
        let child = MockScan::new(vec![
            UnifiedNode::new(1),
            UnifiedNode::new(2),
            UnifiedNode::new(3),
        ]);
        let mut op = aggregate(child, vec![AggregateFunc::Count], None);
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(rows.len(), 1, "global aggregate yields one row");
        assert_eq!(rows[0].id, 0, "synthetic rows carry id 0");
        assert_eq!(rows[0].relational.get("count"), Some(&FieldValue::Int(3)));
    }

    #[test]
    fn aggregate_count_field_ignores_missing_and_null() {
        let mut missing = UnifiedNode::new(2);
        missing
            .relational
            .insert("other".into(), FieldValue::Int(1));
        let child = MockScan::new(vec![
            node_with(1, "email", FieldValue::String("a@x".into())),
            missing,
            node_with(3, "email", FieldValue::Null),
            node_with(4, "email", FieldValue::String("b@x".into())),
        ]);
        let mut op = aggregate(child, vec![AggregateFunc::CountField("email".into())], None);
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].relational.get("count_email"),
            Some(&FieldValue::Int(2)),
            "missing and explicit null do not count"
        );
    }

    #[test]
    fn aggregate_sum_int_accumulates_as_int() {
        let child = MockScan::new(vec![
            node_with(1, "amount", FieldValue::Int(10)),
            node_with(2, "amount", FieldValue::Int(20)),
            node_with(3, "amount", FieldValue::Int(30)),
        ]);
        let mut op = aggregate(child, vec![AggregateFunc::Sum("amount".into())], None);
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(
            rows[0].relational.get("sum_amount"),
            Some(&FieldValue::Int(60))
        );
    }

    #[test]
    fn aggregate_sum_mixed_int_and_float_promotes_to_float() {
        let child = MockScan::new(vec![
            node_with(1, "amount", FieldValue::Int(10)),
            node_with(2, "amount", FieldValue::Float(2.5)),
        ]);
        let mut op = aggregate(child, vec![AggregateFunc::Sum("amount".into())], None);
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(
            rows[0].relational.get("sum_amount"),
            Some(&FieldValue::Float(12.5))
        );
    }

    #[test]
    fn aggregate_sum_skips_non_numeric_values() {
        let mut missing = UnifiedNode::new(4);
        missing
            .relational
            .insert("other".into(), FieldValue::Int(1));
        let child = MockScan::new(vec![
            node_with(1, "amount", FieldValue::Int(10)),
            node_with(2, "amount", FieldValue::String("n/a".into())),
            node_with(3, "amount", FieldValue::Bool(true)),
            missing,
        ]);
        let mut op = aggregate(child, vec![AggregateFunc::Sum("amount".into())], None);
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(
            rows[0].relational.get("sum_amount"),
            Some(&FieldValue::Int(10)),
            "only numeric contributions count"
        );
    }

    #[test]
    fn aggregate_sum_all_non_numeric_is_null() {
        let child = MockScan::new(vec![
            node_with(1, "amount", FieldValue::String("n/a".into())),
            node_with(2, "amount", FieldValue::Null),
        ]);
        let mut op = aggregate(child, vec![AggregateFunc::Sum("amount".into())], None);
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(
            rows[0].relational.get("sum_amount"),
            Some(&FieldValue::Null),
            "no numeric contribution -> null (SQL)"
        );
    }

    #[test]
    fn aggregate_group_by_emits_first_seen_order() {
        let child = MockScan::new(vec![
            node_with(1, "category", FieldValue::String("b".into())),
            node_with(2, "category", FieldValue::String("a".into())),
            node_with(3, "category", FieldValue::String("b".into())),
        ]);
        let mut op = aggregate(child, vec![AggregateFunc::Count], Some("category"));
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(rows.len(), 2, "one row per distinct key");
        assert_eq!(
            rows[0].relational.get("category"),
            Some(&FieldValue::String("b".into())),
            "first-seen order preserved"
        );
        assert_eq!(rows[0].relational.get("count"), Some(&FieldValue::Int(2)));
        assert_eq!(
            rows[1].relational.get("category"),
            Some(&FieldValue::String("a".into()))
        );
        assert_eq!(rows[1].relational.get("count"), Some(&FieldValue::Int(1)));
    }

    #[test]
    fn aggregate_group_by_missing_field_shares_null_group() {
        let mut missing = UnifiedNode::new(1);
        missing
            .relational
            .insert("other".into(), FieldValue::Int(1));
        let child = MockScan::new(vec![
            missing,
            node_with(2, "category", FieldValue::Null),
            node_with(3, "category", FieldValue::String("x".into())),
        ]);
        let mut op = aggregate(child, vec![AggregateFunc::Count], Some("category"));
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(rows.len(), 2, "missing and null collapse into one group");
        assert_eq!(rows[0].relational.get("category"), Some(&FieldValue::Null));
        assert_eq!(rows[0].relational.get("count"), Some(&FieldValue::Int(2)));
    }

    #[test]
    fn aggregate_without_group_by_empty_input_yields_one_row() {
        let child = MockScan::new(vec![]);
        let mut op = aggregate(
            child,
            vec![AggregateFunc::Count, AggregateFunc::Sum("amount".into())],
            None,
        );
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(
            rows.len(),
            1,
            "global aggregate over empty input is one row"
        );
        assert_eq!(rows[0].relational.get("count"), Some(&FieldValue::Int(0)));
        assert_eq!(
            rows[0].relational.get("sum_amount"),
            Some(&FieldValue::Null)
        );
    }

    #[test]
    fn aggregate_group_by_empty_input_yields_zero_rows() {
        let child = MockScan::new(vec![]);
        let mut op = aggregate(child, vec![AggregateFunc::Count], Some("category"));
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert!(rows.is_empty(), "no groups over empty input");
    }

    #[test]
    fn aggregate_multiple_funcs_share_one_row() {
        let child = MockScan::new(vec![
            node_with(1, "amount", FieldValue::Int(5)),
            node_with(2, "amount", FieldValue::Int(7)),
        ]);
        let mut op = aggregate(
            child,
            vec![
                AggregateFunc::Count,
                AggregateFunc::CountField("amount".into()),
                AggregateFunc::Sum("amount".into()),
            ],
            None,
        );
        op.open().unwrap();
        let rows = drain(&mut op);
        op.close().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].relational.get("count"), Some(&FieldValue::Int(2)));
        assert_eq!(
            rows[0].relational.get("count_amount"),
            Some(&FieldValue::Int(2))
        );
        assert_eq!(
            rows[0].relational.get("sum_amount"),
            Some(&FieldValue::Int(12))
        );
    }

    #[test]
    fn aggregate_reopen_resets_accumulators() {
        let child = MockScan::new(vec![node_with(1, "amount", FieldValue::Int(5))]);
        let mut op = aggregate(child, vec![AggregateFunc::Count], None);
        op.open().unwrap();
        assert_eq!(
            op.next().unwrap().unwrap().relational.get("count"),
            Some(&FieldValue::Int(1))
        );
        op.close().unwrap();
        op.open().unwrap();
        assert_eq!(
            op.next().unwrap().unwrap().relational.get("count"),
            Some(&FieldValue::Int(1)),
            "count resets on reopen"
        );
        op.close().unwrap();
    }
}
