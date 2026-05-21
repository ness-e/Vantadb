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
    Collapse(CollapseStatement),           // Phase 32B: Consistency Records
}

#[derive(Debug, Clone, PartialEq)]
pub struct CollapseStatement {
    pub zone_id: u64,
    pub index: usize,
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
