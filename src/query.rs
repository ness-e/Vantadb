//! Query types, logical plan nodes, and statement builders.
//!
//! Defines [`Statement`], [`LogicalPlan`], [`LogicalOperator`], and
//! related types that represent parsed queries before execution.

use crate::node::FieldValue;
use std::collections::BTreeMap;

/// Top-level statement type after parsing.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A query statement.
    Query(Query),
    /// An insert statement.
    Insert(InsertStatement),
    /// An update statement.
    Update(UpdateStatement),
    /// A delete statement.
    Delete(DeleteStatement),
    /// A relate (edge creation) statement.
    Relate(RelateStatement),
    /// A conversational message insert.
    InsertMessage(InsertMessageStatement),
}

/// Insert statement: creates a new node.
#[derive(Debug, Clone, PartialEq)]
pub struct InsertStatement {
    /// Node ID (0 = auto-assign).
    pub node_id: u64,
    /// Entity type string.
    pub node_type: String,
    /// Relational field values.
    pub fields: BTreeMap<String, FieldValue>,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
}

/// Update statement: modifies an existing node.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateStatement {
    /// Node ID to update.
    pub node_id: u64,
    /// Relational field values to set.
    pub fields: BTreeMap<String, FieldValue>,
    /// Optional new embedding vector.
    pub vector: Option<Vec<f32>>,
}

/// Delete statement: removes a node.
#[derive(Debug, Clone, PartialEq)]
pub struct DeleteStatement {
    /// Node ID to delete.
    pub node_id: u64,
}

/// Relate statement: creates a directed edge between two nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct RelateStatement {
    /// Source node ID.
    pub source_id: u64,
    /// Target node ID.
    pub target_id: u64,
    /// Edge label.
    pub label: String,
    /// Optional edge weight.
    pub weight: Option<f32>,
}

/// Insert message statement: creates a conversational message node.
#[derive(Debug, Clone, PartialEq)]
pub struct InsertMessageStatement {
    /// Message role (system, user, assistant).
    pub msg_role: String,
    /// Message content.
    pub content: String,
    /// Thread ID this message belongs to.
    pub thread_id: u64,
}

/// A parsed query with optional traversal, filters, ranking, and projection.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    /// Entity type to search from.
    pub from_entity: String,
    /// Optional graph traversal.
    pub traversal: Option<Traversal>,
    /// Target alias for the result.
    pub target_alias: String,
    /// Optional WHERE conditions.
    pub where_clause: Option<Vec<Condition>>,
    /// Fields to fetch (projection).
    pub fetch: Option<Vec<String>>,
    /// Optional ranking specification.
    pub rank_by: Option<RankBy>,
    /// Query temperature (0.0 = deterministic).
    pub temperature: Option<f32>,
    /// RBAC owner role filter.
    pub owner_role: Option<String>,
}

/// Graph traversal specification.
#[derive(Debug, Clone, PartialEq)]
pub struct Traversal {
    /// Minimum traversal depth.
    pub min_depth: u32,
    /// Maximum traversal depth.
    pub max_depth: u32,
    /// Edge label to follow.
    pub edge_label: String,
    /// Target type filter.
    pub target_type: Option<String>,
    /// Alias for traversed nodes.
    pub alias: Option<String>,
}

/// A query condition (relational or vector similarity).
#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    /// Relational field comparison.
    Relational(String, RelOp, FieldValue),
    /// Vector similarity condition (field, text_query, min_score).
    VectorSim(String, String, f32),
}

/// Relational comparison operator.
#[derive(Debug, Clone, PartialEq)]
pub enum RelOp {
    /// Equals.
    Eq,
    /// Not equals.
    Neq,
    /// Greater than.
    Gt,
    /// Less than.
    Lt,
    /// Greater than or equal.
    Gte,
    /// Less than or equal.
    Lte,
}

/// Ranking specification for query results.
#[derive(Debug, Clone, PartialEq)]
pub struct RankBy {
    /// Field to sort by.
    pub field: String,
    /// Sort descending.
    pub desc: bool,
}

// ─── Logical Plan ──────────────────────────────────────────

/// A logical operator node in the query plan.
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    /// Full scan of an entity type.
    Scan {
        /// Entity type name.
        entity: String,
    },
    /// Graph traversal.
    Traverse {
        /// Minimum depth.
        min_depth: u32,
        /// Maximum depth.
        max_depth: u32,
        /// Edge label to follow.
        edge_label: String,
    },
    /// Relational field filter.
    FilterRelational {
        /// Field name.
        field: String,
        /// Comparison operator.
        op: RelOp,
        /// Expected value.
        value: FieldValue,
    },
    /// Vector similarity search.
    VectorSearch {
        /// Field name.
        field: String,
        /// Text query to embed.
        query_vec: String,
        /// Minimum similarity score.
        min_score: f32,
    },
    /// Field projection (narrowing).
    Project {
        /// Fields to retain.
        fields: Vec<String>,
    },
    /// Sort by a field.
    Sort {
        /// Sort field.
        field: String,
        /// Sort descending.
        desc: bool,
    },
    /// Limit the result set.
    Limit {
        /// Maximum rows.
        top_k: usize,
    },
}

/// A logical query plan containing an ordered list of operators.
#[derive(Debug, Clone)]
pub struct LogicalPlan {
    /// Ordered list of logical operators.
    pub operators: Vec<LogicalOperator>,
    /// Query temperature (0.0 = deterministic/exhaustive).
    pub temperature: f32,
    /// RBAC role to enforce during execution.
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
    /// Release resources held by this operator.
    fn close(&mut self) -> crate::error::Result<()>;
}

#[cfg(test)]
#[allow(missing_docs)]
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
