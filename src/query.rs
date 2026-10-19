//! Query types, logical plan nodes, and statement builders.
//!
//! Defines [`Statement`], [`LogicalPlan`], [`LogicalOperator`], and
//! related types that represent parsed queries before execution.

use crate::node::FieldValue;
use crate::search_profile::SearchProfileConfig;
use serde::Serialize;
use std::collections::BTreeMap;

/// Top-level statement type after parsing.
///
/// Serializes to JSON with serde's default representation (externally tagged
/// enums, snake_case fields) — the canonical AST JSON projection for
/// consumers, matching the SDK [`QueryResult`](crate::sdk::QueryResult)
/// convention.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Statement {
    /// A query statement (FROM/MATCH syntax).
    Query(Query),
    /// A SELECT statement with optional JOIN/subquery.
    Select(SelectStatement),
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InsertStatement {
    /// Node ID (0 = auto-assign).
    pub node_id: u128,
    /// Entity type string.
    pub node_type: String,
    /// Relational field values.
    pub fields: BTreeMap<String, FieldValue>,
    /// Optional embedding vector.
    pub vector: Option<Vec<f32>>,
}

/// Update statement: modifies an existing node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UpdateStatement {
    /// Node ID to update.
    pub node_id: u128,
    /// Relational field values to set.
    pub fields: BTreeMap<String, FieldValue>,
    /// Optional new embedding vector.
    pub vector: Option<Vec<f32>>,
}

/// Delete statement: removes a node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeleteStatement {
    /// Node ID to delete.
    pub node_id: u128,
}

/// Relate statement: creates a directed edge between two nodes.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RelateStatement {
    /// Source node ID.
    pub source_id: u128,
    /// Target node ID.
    pub target_id: u128,
    /// Edge label.
    pub label: String,
    /// Optional edge weight.
    pub weight: Option<f32>,
}

/// Insert message statement: creates a conversational message node.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct InsertMessageStatement {
    /// Message role (system, user, assistant).
    pub msg_role: String,
    /// Message content.
    pub content: String,
    /// Thread ID this message belongs to.
    pub thread_id: u128,
}

/// A parsed query with optional traversal, filters, ranking, and projection.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
    /// Optional search profile (mode, RRF k, candidate budget) — cláusula IQL
    /// PROFILE (MEM-01).
    pub search_profile: Option<SearchProfileConfig>,
    /// Optional `AS OF <unix-ms>` valid-time point (SCH-03, ADR-046 §D3):
    /// results are narrowed to nodes whose validity window contains the
    /// timestamp (`__vanta_valid_at_ms <= T < __vanta_invalid_at_ms`). `None`
    /// = no filter (default unchanged). Version-gated: accepted from
    /// [`IQL_VERSION_MIN_AS_OF`](crate::parser::IQL_VERSION_MIN_AS_OF).
    pub as_of_ms: Option<u64>,
    /// Optional `LIMIT <n>` result cap (WIRE-12): the emitted plan keeps at
    /// most `n` rows (before post-plan filters such as `AS OF`/`ROLE`).
    /// `None` = no cap. Version-gated: accepted from
    /// [`IQL_VERSION_MIN_PAGINATION`](crate::parser::IQL_VERSION_MIN_PAGINATION).
    pub limit: Option<usize>,
    /// Optional `OFFSET <n>` rows to skip (WIRE-12): skip-then-take composes
    /// with `limit`. `None` = no skip (an explicit `0` is a no-op and emits no
    /// plan operator).
    pub offset: Option<usize>,
}

/// Graph traversal specification.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Condition {
    /// Relational field comparison.
    Relational(String, RelOp, FieldValue),
    /// Vector similarity condition (field, text_query, min_score).
    VectorSim(String, String, f32),
    /// Lexical text-match condition (field, query). The query is passed raw
    /// (quoted phrases preserved) and tokenized at execution via
    /// `text_index::query_plan` — enabling contiguous phrase matching.
    TextMatch(String, String),
}

/// Relational comparison operator.
#[derive(Debug, Clone, PartialEq, Serialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RankBy {
    /// Field to sort by.
    pub field: String,
    /// Sort descending.
    pub desc: bool,
}

/// Aggregate function over a field (WIRE-13).
///
/// Serializes with serde's default representation like the rest of the AST:
/// `Count` as `"Count"`, `CountField("email")` as `{"CountField": "email"}`,
/// `Sum("amount")` as `{"Sum": "amount"}`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum AggregateFunc {
    /// `COUNT(*)`: number of rows in the group.
    Count,
    /// `COUNT(field)`: rows where `field` is present and not `Null`.
    CountField(String),
    /// `SUM(field)`: sum of numeric `field` values (non-numeric ignored).
    Sum(String),
}

impl AggregateFunc {
    /// Output field name of this aggregate in each result row (WIRE-13):
    /// `count`, `count_<field>`, `sum_<field>`.
    pub fn output_name(&self) -> String {
        match self {
            AggregateFunc::Count => "count".to_string(),
            AggregateFunc::CountField(field) => format!("count_{field}"),
            AggregateFunc::Sum(field) => format!("sum_{field}"),
        }
    }
}

/// A JOIN clause within a SELECT statement.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JoinClause {
    /// Right-side entity type.
    pub entity: String,
    /// Alias for the right-side entity.
    pub alias: String,
    /// Left-side field in the ON condition (alias-qualified).
    pub left_field: String,
    /// Right-side field in the ON condition (alias-qualified).
    pub right_field: String,
}

/// A scalar subquery condition in WHERE (e.g. `WHERE value > (SELECT AVG(...) FROM ...)`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SubqueryCondition {
    /// Left-side field (alias-qualified).
    pub field: String,
    /// Comparison operator.
    pub op: RelOp,
    /// The subquery SELECT statement.
    pub subquery: Box<SelectStatement>,
}

/// A SELECT-style query with optional JOINs and subqueries.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SelectStatement {
    /// Projected fields (SELECT clause).
    pub projections: Vec<String>,
    /// The FROM clause (single entity or JOIN chain).
    pub from: FromClause,
    /// Optional WHERE conditions on the outer query.
    pub where_clause: Option<Vec<Condition>>,
    /// Subquery conditions in WHERE (e.g. `field op (SELECT ...)`).
    pub subquery_conditions: Vec<SubqueryCondition>,
    /// Query temperature.
    pub temperature: Option<f32>,
    /// Optional `AS OF <unix-ms>` valid-time point (SCH-03, ADR-046 §D3).
    /// Only accepted on the top-level SELECT — subqueries with `AS OF` fail
    /// to parse (silent no-op prevention). `None` = no filter.
    pub as_of_ms: Option<u64>,
    /// Optional `LIMIT <n>` result cap (WIRE-12): same semantics as in
    /// `FROM`/`MATCH`. `None` = no cap.
    pub limit: Option<usize>,
    /// Optional `OFFSET <n>` rows to skip (WIRE-12): same semantics as in
    /// `FROM`/`MATCH`. `None` = no skip.
    pub offset: Option<usize>,
    /// Aggregate functions in the SELECT list (WIRE-13). When non-empty the
    /// query is an aggregation: the plan emits [`LogicalOperator::Aggregate`]
    /// instead of `Project`, and each output row carries the group key plus
    /// one field per aggregate ([`AggregateFunc::output_name`]). Parser
    /// invariant: when non-empty, `projections` is empty and `limit`/`offset`/
    /// `as_of_ms` are `None` (the combinations are rejected at parse time,
    /// FIND-316) — programmatic AST construction must respect it.
    pub aggregates: Vec<AggregateFunc>,
    /// Optional `GROUP BY <field>` key (WIRE-13): one output row per distinct
    /// value, in first-seen order. `None` = single global group. Requires at
    /// least one aggregate and must not collide with an aggregate output name
    /// (both rejected at parse time).
    pub group_by: Option<String>,
}

/// The FROM clause of a SELECT — either a single entity or a JOIN of two sub-clauses.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum FromClause {
    /// Scan a single entity type with an alias.
    Single {
        /// Entity type.
        entity: String,
        /// Alias used in projections / conditions.
        alias: String,
    },
    /// A JOIN between two FROM clauses.
    Join {
        /// Left side (another FROM clause).
        left: Box<FromClause>,
        /// Right side (another FROM clause).
        right: Box<FromClause>,
        /// The ON condition fields.
        left_field: String,
        right_field: String,
    },
}

impl SelectStatement {
    /// Convert a SELECT statement into a LogicalPlan.
    pub fn into_logical_plan(self) -> LogicalPlan {
        let mut ops = Vec::new();

        match self.from {
            FromClause::Single { entity, alias: _ } => {
                ops.push(LogicalOperator::Scan { entity });
            }
            FromClause::Join {
                left,
                right,
                left_field,
                right_field,
            } => {
                let left_plan = SelectStatement::from_clause_to_plan(*left);
                let right_plan = SelectStatement::from_clause_to_plan(*right);
                ops.push(LogicalOperator::Join {
                    left_plan: Box::new(left_plan),
                    right_plan: Box::new(right_plan),
                    left_field,
                    right_field,
                });
            }
        }

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
                    Condition::TextMatch(f, query) => {
                        ops.push(LogicalOperator::TextFilter { field: f, query });
                    }
                }
            }
        }

        for subq in self.subquery_conditions {
            let subq_plan = subq.subquery.into_logical_plan();
            ops.push(LogicalOperator::SubqueryFilter {
                field: subq.field,
                op: subq.op,
                subquery_plan: Box::new(subq_plan),
            });
        }

        if !self.aggregates.is_empty() {
            // WIRE-13: aggregation replaces projection — the operator builds
            // the output fields (group key + aggregates). LIMIT/OFFSET and
            // AS OF are rejected at parse time for aggregation queries
            // (FIND-316), so no pagination operators are emitted here.
            // Programmatic AST construction must respect the same invariant —
            // those fields would otherwise be silently ignored.
            debug_assert!(
                self.limit.is_none()
                    && self.offset.is_none()
                    && self.as_of_ms.is_none()
                    && self.projections.is_empty(),
                "aggregation queries must not set limit/offset/as_of_ms/projections"
            );
            ops.push(LogicalOperator::Aggregate {
                funcs: self.aggregates,
                group_by: self.group_by,
            });
        } else {
            if !self.projections.is_empty() {
                ops.push(LogicalOperator::Project {
                    fields: self.projections,
                });
            }

            push_pagination(&mut ops, self.limit, self.offset);
        }

        LogicalPlan {
            operators: ops,
            temperature: self.temperature.unwrap_or(0.0),
            enforce_role: None,
            search_profile: None,
        }
    }

    fn from_clause_to_plain_plan(clause: FromClause) -> LogicalPlan {
        let ops = match clause {
            FromClause::Single { entity, alias: _ } => {
                vec![LogicalOperator::Scan { entity }]
            }
            FromClause::Join {
                left,
                right,
                left_field,
                right_field,
            } => {
                let left_plan = SelectStatement::from_clause_to_plain_plan(*left);
                let right_plan = SelectStatement::from_clause_to_plain_plan(*right);
                vec![LogicalOperator::Join {
                    left_plan: Box::new(left_plan),
                    right_plan: Box::new(right_plan),
                    left_field,
                    right_field,
                }]
            }
        };
        LogicalPlan {
            operators: ops,
            temperature: 0.0,
            enforce_role: None,
            search_profile: None,
        }
    }

    fn from_clause_to_plan(clause: FromClause) -> LogicalPlan {
        Self::from_clause_to_plain_plan(clause)
    }
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
    /// Lexical text filter on a field (phrase-aware).
    TextFilter {
        /// Field name.
        field: String,
        /// Text query (quoted phrases preserved).
        query: String,
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
    /// Skip the first `skip` rows of the child stream (WIRE-12). Registry
    /// extension (C2S6 pattern: variant + physical file + one register line):
    /// compiled post-chain by `OperatorRegistry`, after the built-in `Limit`.
    /// `Query`/`SelectStatement` composition widens the emitted `Limit`
    /// window by the skip so the net effect is skip-then-take (see
    /// `push_pagination`).
    Offset {
        /// Number of leading rows to skip.
        skip: usize,
    },
    /// Aggregate rows into groups (WIRE-13). Registry extension (C2S6 pattern:
    /// variant + physical file + one register line): compiled post-chain by
    /// `OperatorRegistry`. Emits one synthetic `UnifiedNode` per group
    /// (`id = 0`), carrying the group key plus one field per aggregate
    /// ([`AggregateFunc::output_name`]). Without `group_by` it emits exactly
    /// one row over the whole input (SQL global-aggregate semantics).
    Aggregate {
        /// Aggregate functions to compute, in output order.
        funcs: Vec<AggregateFunc>,
        /// Optional group key field; `None` = single global group.
        group_by: Option<String>,
    },
    /// Deduplicate consecutive rows by a relational field (C2S6 extension
    /// exemplar: compiles/costs through `OperatorRegistry` without touching
    /// the proven `planner` / `executor` matches; test-constructed, H2
    /// solo-physical authorized — no IQL producer yet).
    Dedup {
        /// Field whose first-seen value wins; rows missing it share one key.
        field: String,
    },
    /// A JOIN between two sub-plans.
    Join {
        /// Left-side sub-plan.
        left_plan: Box<LogicalPlan>,
        /// Right-side sub-plan.
        right_plan: Box<LogicalPlan>,
        /// Left-side join field (alias-qualified).
        left_field: String,
        /// Right-side join field (alias-qualified).
        right_field: String,
    },
    /// A scalar subquery filter: `field op (SELECT ...)`.
    SubqueryFilter {
        /// Field to compare (alias-qualified).
        field: String,
        /// Comparison operator.
        op: RelOp,
        /// The subquery plan to execute.
        subquery_plan: Box<LogicalPlan>,
    },
}

/// A logical query plan containing an ordered list of operators.
#[derive(Debug, Clone, PartialEq)]
pub struct LogicalPlan {
    /// Ordered list of logical operators.
    pub operators: Vec<LogicalOperator>,
    /// Query temperature (0.0 = deterministic/exhaustive).
    pub temperature: f32,
    /// RBAC role to enforce during execution.
    pub enforce_role: Option<String>,
    /// Optional search profile (MEM-01): mode/RRF k/candidate budget.
    pub search_profile: Option<SearchProfileConfig>,
}

/// Push the pagination operators for `LIMIT`/`OFFSET` (WIRE-12).
///
/// `OFFSET` compiles as a post-chain registry extension (`PhysicalOffset`,
/// applied by `optimize_and_compile` AFTER the built-in `Limit`), while SQL
/// pagination is skip-then-take. When both clauses are present the emitted
/// `Limit` window is therefore widened by the offset — the extension then
/// trims the front, netting "skip `offset`, take `limit`". Without the
/// widening, `Limit` would cap the stream before the skip and drop valid rows.
fn push_pagination(ops: &mut Vec<LogicalOperator>, limit: Option<usize>, offset: Option<usize>) {
    let offset = offset.filter(|skip| *skip > 0);
    if let Some(limit) = limit {
        let window = match offset {
            Some(skip) => limit.saturating_add(skip),
            None => limit,
        };
        ops.push(LogicalOperator::Limit { top_k: window });
    }
    if let Some(skip) = offset {
        ops.push(LogicalOperator::Offset { skip });
    }
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
                    Condition::TextMatch(f, query) => {
                        ops.push(LogicalOperator::TextFilter { field: f, query });
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

        push_pagination(&mut ops, self.limit, self.offset);

        LogicalPlan {
            operators: ops,
            temperature: self.temperature.unwrap_or(0.0), // 0.0 default (Exhaustive)
            enforce_role: self.owner_role,
            search_profile: self.search_profile,
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
            search_profile: None,
            as_of_ms: None,
            limit: None,
            offset: None,
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
            search_profile: None,
            as_of_ms: None,
            limit: None,
            offset: None,
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
            search_profile: None,
            as_of_ms: None,
            limit: None,
            offset: None,
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
            search_profile: None,
            as_of_ms: None,
            limit: None,
            offset: None,
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
            search_profile: None,
            as_of_ms: None,
            limit: None,
            offset: None,
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

    // ── Pagination plan emission (WIRE-12) ──

    fn paginated_query(limit: Option<usize>, offset: Option<usize>) -> Query {
        Query {
            from_entity: "Doc".into(),
            traversal: None,
            target_alias: "target".into(),
            where_clause: None,
            fetch: None,
            rank_by: None,
            temperature: None,
            owner_role: None,
            search_profile: None,
            as_of_ms: None,
            limit,
            offset,
        }
    }

    #[test]
    fn into_logical_plan_emits_limit_only() {
        let plan = paginated_query(Some(5), None).into_logical_plan();
        assert!(plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Limit { top_k: 5 })));
        assert!(!plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Offset { .. })));
    }

    #[test]
    fn into_logical_plan_widens_limit_window_when_offset_present() {
        // OFFSET compiles post-chain through the registry (planner applies
        // extensions AFTER the built-in Limit), so the emitted Limit window is
        // widened by the skip: Limit{5+2} then Offset{2} = skip 2, take 5.
        let plan = paginated_query(Some(5), Some(2)).into_logical_plan();
        assert!(plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Limit { top_k: 7 })));
        assert!(plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Offset { skip: 2 })));
    }

    #[test]
    fn into_logical_plan_emits_offset_only_without_limit() {
        let plan = paginated_query(None, Some(3)).into_logical_plan();
        assert!(!plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Limit { .. })));
        assert!(plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Offset { skip: 3 })));
    }

    #[test]
    fn into_logical_plan_omits_zero_offset_and_keeps_limit() {
        let plan = paginated_query(Some(4), Some(0)).into_logical_plan();
        assert!(plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Limit { top_k: 4 })));
        assert!(!plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Offset { .. })));
    }

    // ── Aggregation (WIRE-13) ──

    #[test]
    fn aggregate_func_output_names_are_deterministic() {
        assert_eq!(AggregateFunc::Count.output_name(), "count");
        assert_eq!(
            AggregateFunc::CountField("email".into()).output_name(),
            "count_email"
        );
        assert_eq!(
            AggregateFunc::Sum("amount".into()).output_name(),
            "sum_amount"
        );
    }

    fn aggregate_select(aggregates: Vec<AggregateFunc>, group_by: Option<&str>) -> SelectStatement {
        SelectStatement {
            projections: vec![],
            from: FromClause::Single {
                entity: "Invoice".into(),
                alias: "target".into(),
            },
            where_clause: None,
            subquery_conditions: vec![],
            temperature: None,
            as_of_ms: None,
            limit: None,
            offset: None,
            aggregates,
            group_by: group_by.map(String::from),
        }
    }

    #[test]
    fn into_logical_plan_emits_aggregate_without_project() {
        let plan =
            aggregate_select(vec![AggregateFunc::Count], Some("category")).into_logical_plan();
        assert!(plan.operators.iter().any(|op| matches!(
            op,
            LogicalOperator::Aggregate { funcs, group_by }
                if funcs == &vec![AggregateFunc::Count]
                    && group_by.as_deref() == Some("category")
        )));
        assert!(
            !plan
                .operators
                .iter()
                .any(|op| matches!(op, LogicalOperator::Project { .. })),
            "aggregate output fields are built by the operator, not Project"
        );
        assert!(!plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Limit { .. })));
    }

    #[test]
    fn into_logical_plan_plain_select_still_emits_project() {
        // Control: the aggregate branch must not change non-aggregate SELECT.
        let mut select = aggregate_select(vec![], None);
        select.projections = vec!["name".into()];
        let plan = select.into_logical_plan();
        assert!(plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Project { .. })));
        assert!(!plan
            .operators
            .iter()
            .any(|op| matches!(op, LogicalOperator::Aggregate { .. })));
    }
}
