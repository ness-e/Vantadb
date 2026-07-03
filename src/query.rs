//! Query types, logical plan nodes, and statement builders.
//!
//! Defines [`Statement`], [`LogicalPlan`], [`LogicalOperator`], and
//! related types that represent parsed queries before execution.

use crate::node::FieldValue;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Query(Query),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    Relate(RelateStatement),
    InsertMessage(InsertMessageStatement), // Conversational Primitive
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    pub node_id: u64,
    pub node_type: String,
    pub fields: BTreeMap<String, FieldValue>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    pub node_id: u64,
    pub fields: BTreeMap<String, FieldValue>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    pub node_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelateStatement {
    pub source_id: u64,
    pub target_id: u64,
    pub label: String,
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertMessageStatement {
    pub msg_role: String, // system, user, assistant
    pub content: String,
    pub thread_id: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub from_entity: String,
    pub traversal: Option<Traversal>,
    pub target_alias: String,
    pub where_clause: Option<Vec<Condition>>,
    pub fetch: Option<Vec<String>>,
    pub rank_by: Option<RankBy>,
    pub temperature: Option<f32>,
    pub owner_role: Option<String>, // RBAC
}

#[derive(Debug, Clone, PartialEq)]
pub struct Traversal {
    pub min_depth: u32,
    pub max_depth: u32,
    pub edge_label: String,
    pub target_type: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Relational(String, RelOp, FieldValue),
    VectorSim(String, String, f32), // field, text_query, min_score
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelOp {
    Eq,
    Neq,
    Gt,
    Lt,
    Gte,
    Lte,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RankBy {
    pub field: String,
    pub desc: bool,
}

// ─── Logical Plan ──────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    Scan {
        entity: String,
    },
    Traverse {
        min_depth: u32,
        max_depth: u32,
        edge_label: String,
    },
    FilterRelational {
        field: String,
        op: RelOp,
        value: FieldValue,
    },
    VectorSearch {
        field: String,
        query_vec: String,
        min_score: f32,
    },
    Project {
        fields: Vec<String>,
    },
    Sort {
        field: String,
        desc: bool,
    },
    Limit {
        top_k: usize,
    },
}

#[derive(Debug, Clone)]
pub struct LogicalPlan {
    pub operators: Vec<LogicalOperator>,
    pub temperature: f32,
    pub enforce_role: Option<String>,
}

impl Query {
    /// Convert AST into a basic Logical Plan
    pub fn into_logical_plan(self) -> LogicalPlan {
        let mut ops = Vec::new();

        ops.push(LogicalOperator::Scan {
            entity: self.from_entity,
        });

        if let Some(mut conds) = self.where_clause {
            for cond in conds.drain(..) {
                match cond {
                    Condition::Relational(f, op, v) => {
                        ops.push(LogicalOperator::FilterRelational {
                            field: f,
                            op,
                            value: v,
                        });
                    }
                    Condition::VectorSim(f, text, min) => {
                        ops.push(LogicalOperator::VectorSearch {
                            field: f,
                            query_vec: text,
                            min_score: min,
                        });
                    }
                }
            }
        }

        if let Some(trav) = self.traversal {
            ops.push(LogicalOperator::Traverse {
                min_depth: trav.min_depth,
                max_depth: trav.max_depth,
                edge_label: trav.edge_label,
            });
        }

        if let Some(rank) = self.rank_by {
            ops.push(LogicalOperator::Sort {
                field: rank.field,
                desc: rank.desc,
            });
        }

        if let Some(fetch) = self.fetch {
            ops.push(LogicalOperator::Project { fields: fetch });
        }

        LogicalPlan {
            operators: ops,
            temperature: self.temperature.unwrap_or(0.0), // 0.0 default (Exhaustive)
            enforce_role: self.owner_role,
        }
    }
}

// ── VantaDB Biological Nomenclature (Type Alias) ────────────

/// The **QueryPlanner** is VantaDB's query decision engine.
/// Technically identical to `LogicalPlan` — it decides what to scan,
/// how to filter, and which traversal strategy to execute.
pub type QueryPlanner = LogicalPlan;

/// Physical Volcano-style execution operator.
pub trait PhysicalOperator: Send + Sync {
    /// Initialize resources needed for execution.
    fn open(&mut self) -> crate::error::Result<()>;
    /// Retrieve the next UnifiedNode in the stream, or None if the stream is exhausted.
    fn next(&mut self) -> crate::error::Result<Option<crate::node::UnifiedNode>>;
    fn close(&mut self) -> crate::error::Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::FieldValue;

    // ── Statement construction ──

    #[test]
    fn test_insert_statement_defaults() {
        let s = InsertStatement {
            node_id: 1,
            node_type: "Person".into(),
            fields: BTreeMap::new(),
            vector: None,
        };
        assert_eq!(s.node_id, 1);
        assert_eq!(s.node_type, "Person");
        assert!(s.vector.is_none());
    }

    #[test]
    fn test_delete_statement() {
        let s = DeleteStatement { node_id: 42 };
        assert_eq!(s.node_id, 42);
    }

    #[test]
    fn test_relate_statement_with_weight() {
        let s = RelateStatement {
            source_id: 10,
            target_id: 20,
            label: "knows".into(),
            weight: Some(0.9),
        };
        assert_eq!(s.source_id, 10);
        assert_eq!(s.target_id, 20);
        assert_eq!(s.weight, Some(0.9));
    }

    #[test]
    fn test_relate_statement_without_weight() {
        let s = RelateStatement {
            source_id: 1,
            target_id: 2,
            label: "edge".into(),
            weight: None,
        };
        assert!(s.weight.is_none());
    }

    #[test]
    fn test_message_statement() {
        let s = InsertMessageStatement {
            msg_role: "user".into(),
            content: "hello".into(),
            thread_id: 1,
        };
        assert_eq!(s.msg_role, "user");
        assert_eq!(s.content, "hello");
    }

    // ── Statement enum ──

    #[test]
    fn test_statement_variants() {
        match Statement::Insert(InsertStatement {
            node_id: 1,
            node_type: "".into(),
            fields: BTreeMap::new(),
            vector: None,
        }) {
            Statement::Insert(_) => {}
            _ => panic!("wrong variant"),
        }
        match Statement::Delete(DeleteStatement { node_id: 1 }) {
            Statement::Delete(_) => {}
            _ => panic!("wrong variant"),
        }
        match Statement::Relate(RelateStatement {
            source_id: 1,
            target_id: 2,
            label: "".into(),
            weight: None,
        }) {
            Statement::Relate(_) => {}
            _ => panic!("wrong variant"),
        }
    }

    // ── Query construction ──

    #[test]
    fn test_query_default() {
        let q = Query {
            from_entity: "Node".into(),
            traversal: None,
            target_alias: String::new(),
            where_clause: None,
            fetch: None,
            rank_by: None,
            temperature: None,
            owner_role: None,
        };
        assert_eq!(q.from_entity, "Node");
        assert!(q.traversal.is_none());
    }

    #[test]
    fn test_query_with_traversal() {
        let t = Traversal {
            min_depth: 1,
            max_depth: 3,
            edge_label: "knows".into(),
            target_type: None,
            alias: None,
        };
        let q = Query {
            from_entity: "Person".into(),
            traversal: Some(t),
            target_alias: String::new(),
            where_clause: None,
            fetch: None,
            rank_by: None,
            temperature: None,
            owner_role: None,
        };
        assert_eq!(q.traversal.as_ref().unwrap().min_depth, 1);
        assert_eq!(q.traversal.as_ref().unwrap().max_depth, 3);
        assert_eq!(q.traversal.as_ref().unwrap().edge_label, "knows");
    }

    // ── into_logical_plan ──

    #[test]
    fn test_into_logical_plan_basic() {
        let q = Query {
            from_entity: "User".into(),
            traversal: None,
            target_alias: String::new(),
            where_clause: None,
            fetch: None,
            rank_by: None,
            temperature: None,
            owner_role: None,
        };
        let plan = q.into_logical_plan();
        assert_eq!(plan.operators.len(), 1);
        assert_eq!(
            plan.operators[0],
            LogicalOperator::Scan {
                entity: "User".into()
            }
        );
    }

    #[test]
    fn test_into_logical_plan_with_conditions() {
        let mut fields: BTreeMap<String, FieldValue> = BTreeMap::new();
        fields.insert("age".into(), FieldValue::Int(25));
        let q = Query {
            from_entity: "User".into(),
            traversal: None,
            target_alias: String::new(),
            where_clause: Some(vec![Condition::Relational(
                "age".into(),
                RelOp::Gt,
                FieldValue::Int(18),
            )]),
            fetch: None,
            rank_by: None,
            temperature: None,
            owner_role: None,
        };
        let plan = q.into_logical_plan();
        assert_eq!(plan.operators.len(), 2);
        assert!(matches!(
            plan.operators[1],
            LogicalOperator::FilterRelational { .. }
        ));
    }

    #[test]
    fn test_into_logical_plan_with_rank_and_fetch() {
        let q = Query {
            from_entity: "Item".into(),
            traversal: None,
            target_alias: String::new(),
            where_clause: None,
            fetch: Some(vec!["name".into(), "price".into()]),
            rank_by: Some(RankBy {
                field: "price".into(),
                desc: true,
            }),
            temperature: None,
            owner_role: None,
        };
        let plan = q.into_logical_plan();
        let ops: Vec<&str> = plan
            .operators
            .iter()
            .map(|o| match o {
                LogicalOperator::Sort { .. } => "sort",
                LogicalOperator::Project { .. } => "project",
                LogicalOperator::Scan { .. } => "scan",
                _ => "other",
            })
            .collect();
        assert!(ops.contains(&"sort"));
        assert!(ops.contains(&"project"));
    }

    // ── RelOp ──

    #[test]
    fn test_relop_variants() {
        assert_ne!(RelOp::Eq, RelOp::Neq);
        assert_ne!(RelOp::Gt, RelOp::Lt);
        assert_ne!(RelOp::Gte, RelOp::Lte);
    }

    // ── RankBy ──

    #[test]
    fn test_rank_by_ascending() {
        let r = RankBy {
            field: "score".into(),
            desc: false,
        };
        assert_eq!(r.field, "score");
        assert!(!r.desc);
    }

    // ── LogicalOperator ──

    #[test]
    fn test_logical_operator_partial_eq() {
        assert_eq!(
            LogicalOperator::Scan {
                entity: "Test".into()
            },
            LogicalOperator::Scan {
                entity: "Test".into()
            }
        );
        assert_ne!(
            LogicalOperator::Scan { entity: "A".into() },
            LogicalOperator::Scan { entity: "B".into() }
        );
    }

    // ── Traversal ──

    #[test]
    fn test_traversal_defaults() {
        let t = Traversal {
            min_depth: 0,
            max_depth: 0,
            edge_label: "".into(),
            target_type: None,
            alias: None,
        };
        assert_eq!(t.min_depth, 0);
        assert!(t.target_type.is_none());
        assert!(t.alias.is_none());
    }

    // ── Condition ──

    #[test]
    fn test_condition_relational() {
        let c = Condition::Relational("age".into(), RelOp::Eq, FieldValue::Int(30));
        match c {
            Condition::Relational(f, op, v) => {
                assert_eq!(f, "age");
                assert_eq!(op, RelOp::Eq);
                assert_eq!(v, FieldValue::Int(30));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn test_condition_vector_sim() {
        let c = Condition::VectorSim("description".into(), "blue shoes".into(), 0.5);
        match c {
            Condition::VectorSim(f, q, s) => {
                assert_eq!(f, "description");
                assert_eq!(q, "blue shoes");
                assert!((s - 0.5).abs() < 1e-6);
            }
            _ => panic!("wrong variant"),
        }
    }
}
