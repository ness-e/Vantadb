//! IQL grammar — statements sobre los tokens del lexer (FIND-50).
//!
//! Parsers de queries (`FROM`/`MATCH`), DML (`INSERT`/`UPDATE`/`DELETE`/
//! `RELATE`), `SELECT` con JOINs/subqueries y autocomplete. Las primitivas
//! léxicas (`ws`, `ident`, literales) viven en [`super::lexer`].

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::{map, not, opt, peek},
    multi::{many0, separated_list1},
    number::complete::float,
    sequence::{delimited, tuple},
    IResult,
};

use super::lexer::{
    ident, non_keyword_ident, parse_literal_field_value, parse_number, parse_u128_id, parse_u64_id,
    parse_vector_lit, string_literal, ws, RESERVED_KEYWORDS,
};
use super::{
    iql_supports, IQL_VERSION_MIN_AS_OF, IQL_VERSION_MIN_PAGINATION, IQL_VERSION_MIN_PROFILE,
};
use crate::node::FieldValue;
use crate::query::*;
use crate::search_profile::{SearchProfileConfig, SearchProfileMode};

pub(crate) fn parse_traversal(i: &str) -> IResult<&str, Traversal> {
    let (i, _) = ws(tag("SIGUE"))(i)?;
    let (i, min_depth) = ws(parse_number)(i)?;
    let (i, _) = ws(tag(".."))(i)?;
    let (i, max_depth) = ws(parse_number)(i)?;
    let (i, edge_label) = ws(string_literal)(i)?;
    let (i, target_type) = opt(tuple((ws(tag("TYPE")), ws(ident))))(i)?;
    // `AS <alias>` uses the non-keyword ident so a following `AS OF <ts>`
    // clause is never swallowed as an alias (SCH-03).
    let (i, alias) = opt(tuple((ws(tag("AS")), ws(non_keyword_ident))))(i)?;

    Ok((
        i,
        Traversal {
            min_depth,
            max_depth,
            edge_label,
            target_type: target_type.map(|(_, t)| t),
            alias: alias.map(|(_, a)| a),
        },
    ))
}

pub(crate) fn parse_rel_op(i: &str) -> IResult<&str, RelOp> {
    // Longest match first: `==` must be tried before `=` (otherwise the second
    // `=` is left dangling and the whole condition fails to parse).
    alt((
        map(tag("=="), |_| RelOp::Eq),
        map(tag("!="), |_| RelOp::Neq),
        map(tag(">="), |_| RelOp::Gte),
        map(tag("<="), |_| RelOp::Lte),
        map(tag(">"), |_| RelOp::Gt),
        map(tag("<"), |_| RelOp::Lt),
        map(tag("="), |_| RelOp::Eq),
    ))(i)
}

pub(crate) fn parse_condition(i: &str) -> IResult<&str, Condition> {
    alt((
        // Vector Query: p.bio ~ "rust expert", min = 0.88
        map(
            tuple((
                ws(ident),
                ws(tag("~")),
                ws(string_literal),
                ws(tag(",")),
                ws(tag("min")),
                ws(tag("=")),
                ws(float),
            )),
            |(field, _, query, _, _, _, min_score)| Condition::VectorSim(field, query, min_score),
        ),
        // Text Query (phrase): p.bio ~ "neural network" (no min suffix)
        map(
            tuple((ws(ident), ws(tag("~")), ws(string_literal))),
            |(field, _, query)| Condition::TextMatch(field, query),
        ),
        // Relational Query: p.pais = "VZLA", or numeric p.edad > 18, or
        // p.activo = true / p.campo = null. Reuse parse_literal_field_value so the
        // RHS is typed: bare integers parse as Int (i64, exact) and decimals as
        // Float, so the evaluator's same-variant branches give numeric ordering;
        // quoted strings stay String for backward compatibility.
        map(
            tuple((ws(ident), ws(parse_rel_op), ws(parse_literal_field_value))),
            |(field, op, val)| Condition::Relational(field, op, val),
        ),
    ))(i)
}

/// Parse the optional `AS OF <unix-ms>` valid-time clause (SCH-03,
/// ADR-046 §D3), version-gated by [`IQL_VERSION_MIN_AS_OF`].
///
/// Strictness: when the keyword pair is consumed the timestamp is required —
/// a malformed clause is a parse error, never a silent filter drop. Accepted
/// in canonical positions (right after the table spec, after the optional
/// clauses, and — since WIRE-12 — after pagination) so
/// `FROM x AS OF t WHERE ...`, `FROM x WHERE ... AS OF t` and
/// `FROM x LIMIT n AS OF t` all work.
fn parse_as_of_clause(i: &str) -> IResult<&str, Option<u64>> {
    if !iql_supports(IQL_VERSION_MIN_AS_OF) {
        return Ok((i, None));
    }
    let (i, keyword) = opt(tuple((ws(tag("AS")), ws(tag("OF")))))(i)?;
    if keyword.is_none() {
        return Ok((i, None));
    }
    let (i, ts) = ws(parse_u64_id)(i)?;
    Ok((i, Some(ts)))
}

/// Merge the two `AS OF` parse positions (early — right after the table spec —
/// and late — after the last optional clause) into the final clause value,
/// rejecting a **duplicated** clause: keeping the first and silently dropping
/// the second would misreport the query's time scope (SCH-03 review
/// Optional-2). The failure input points at the duplicate clause so the
/// reported line/col is where the user must look; `Executor::execute_hybrid`
/// maps this `Failure(Verify)` at an `AS OF` position to a stable message
/// (`iql_parse_error_message`).
fn merge_as_of_clauses(
    early: Option<u64>,
    late: Option<u64>,
    late_input: &str,
) -> Result<Option<u64>, nom::Err<nom::error::Error<&str>>> {
    if early.is_some() && late.is_some() {
        return Err(nom::Err::Failure(nom::error::Error::new(
            late_input,
            nom::error::ErrorKind::Verify,
        )));
    }
    Ok(early.or(late))
}

/// Parse one optional `LIMIT <n>` / `OFFSET <n>` clause (WIRE-12).
///
/// Strictness (same rule as `AS OF`): once the keyword is consumed the count
/// is required — a malformed clause is a parse error, never a silent drop.
/// The keyword is only recognized at a clause boundary (the following
/// character must not continue an identifier), so aliases like `LIMITED`
/// stay valid.
fn parse_pagination_clause<'a>(
    i: &'a str,
    keyword: &'static str,
) -> IResult<&'a str, Option<usize>> {
    let clause_start = i;
    let mut keyword_parser = ws(tuple((
        tag::<&str, &str, nom::error::Error<&str>>(keyword),
        not(peek(nom::character::complete::satisfy(|c: char| {
            c.is_alphanumeric() || c == '_'
        }))),
    )));
    let Ok((i, _)) = keyword_parser(i) else {
        return Ok((clause_start, None));
    };
    match ws(parse_number)(i) {
        Ok((i, n)) => Ok((i, Some(n as usize))),
        Err(_) => Err(nom::Err::Failure(nom::error::Error::new(
            clause_start,
            nom::error::ErrorKind::Verify,
        ))),
    }
}

/// Parse the optional `LIMIT <n>` / `OFFSET <n>` pagination clauses (WIRE-12),
/// version-gated by [`IQL_VERSION_MIN_PAGINATION`].
///
/// Canonical order is `LIMIT` then `OFFSET` (SQL). Reversed or duplicated
/// pagination keywords are rejected as a `Failure` — they would otherwise be
/// left as trailing input and silently ignored by callers.
fn parse_pagination(i: &str) -> IResult<&str, (Option<usize>, Option<usize>)> {
    if !iql_supports(IQL_VERSION_MIN_PAGINATION) {
        return Ok((i, (None, None)));
    }
    let (i, limit) = parse_pagination_clause(i, "LIMIT")?;
    let (i, offset) = parse_pagination_clause(i, "OFFSET")?;
    if ws(tag::<&str, &str, nom::error::Error<&str>>("LIMIT"))(i).is_ok()
        || ws(tag::<&str, &str, nom::error::Error<&str>>("OFFSET"))(i).is_ok()
    {
        return Err(nom::Err::Failure(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Verify,
        )));
    }
    Ok((i, (limit, offset)))
}

/// Parse a `FROM`/`MATCH` query statement.
pub fn parse_query(i: &str) -> IResult<&str, Query> {
    let (i, _) = ws(alt((tag("FROM"), tag("MATCH"))))(i)?;
    let (i, from_entity) = ws(ident)(i)?;

    let (i, traversal) = opt(parse_traversal)(i)?;

    let (i, target_alias) = opt(ws(non_keyword_ident))(i)?;
    let target_alias = target_alias.unwrap_or_else(|| "target".to_string());

    let (i, as_of_early) = parse_as_of_clause(i)?;

    let (i, where_clause) = opt(tuple((
        ws(tag("WHERE")),
        separated_list1(ws(tag("AND")), parse_condition),
    )))(i)?;

    let (i, fetch) = opt(tuple((
        ws(tag("FETCH")),
        separated_list1(ws(char(',')), ws(ident)),
    )))(i)?;

    let (i, rank_by) = opt(tuple((ws(tag("RANK BY")), ws(ident), opt(ws(tag("DESC"))))))(i)?;

    let (i, temperature) = opt(tuple((ws(tag("WITH")), ws(tag("TEMPERATURE")), ws(float))))(i)?;

    let (i, owner_role) = opt(tuple((ws(tag("ROLE")), ws(string_literal))))(i)?;

    let (i, search_profile) = if iql_supports(IQL_VERSION_MIN_PROFILE) {
        opt(tuple((
            ws(tag("PROFILE")),
            ws(parse_profile_mode),
            opt(tuple((ws(tag("rrf_k")), ws(parse_number)))),
            opt(tuple((ws(tag("candidate_k")), ws(parse_number)))),
        )))(i)?
    } else {
        (i, None)
    };

    let as_of_late_start = i;
    let (i, as_of_late) = parse_as_of_clause(i)?;
    let (i, (limit, offset)) = parse_pagination(i)?;
    let as_of_tail_start = i;
    let (i, as_of_tail) = parse_as_of_clause(i)?;
    let as_of_ms = merge_as_of_clauses(
        merge_as_of_clauses(as_of_early, as_of_late, as_of_late_start)?,
        as_of_tail,
        as_of_tail_start,
    )?;

    Ok((
        i,
        Query {
            from_entity,
            traversal,
            target_alias,
            where_clause: where_clause.map(|(_, conds)| conds),
            fetch: fetch.map(|(_, f)| f),
            rank_by: rank_by.map(|(_, f, d)| RankBy {
                field: f,
                desc: d.is_some(),
            }),
            temperature: temperature.map(|(_, _, t)| t),
            owner_role: owner_role.map(|(_, r)| r),
            search_profile: search_profile.map(|(_, mode, rrf, cand)| SearchProfileConfig {
                mode,
                rrf_k: rrf.map(|(_, n)| n as usize),
                candidate_k: cand.map(|(_, n)| n as usize),
            }),
            as_of_ms,
            limit,
            offset,
        },
    ))
}

/// Parse el modo de un perfil de búsqueda: keyword | vector | hybrid (MEM-01).
pub(crate) fn parse_profile_mode(i: &str) -> IResult<&str, SearchProfileMode> {
    alt((
        map(tag("keyword"), |_| SearchProfileMode::Keyword),
        map(tag("vector"), |_| SearchProfileMode::Vector),
        map(tag("hybrid"), |_| SearchProfileMode::Hybrid),
    ))(i)
}

// ─── DML (Data Manipulation Language) ──────────────────────────

pub(crate) fn parse_field_assign(i: &str) -> IResult<&str, (String, FieldValue)> {
    let (i, key) = ws(ident)(i)?;
    let (i, _) = ws(char(':'))(i)?;
    let (i, val) = ws(parse_literal_field_value)(i)?;
    Ok((i, (key, val)))
}

pub(crate) fn parse_insert(i: &str) -> IResult<&str, InsertStatement> {
    let (i, _) = ws(tag("INSERT"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, node_id) = ws(parse_u128_id)(i)?;
    let (i, _) = ws(tag("TYPE"))(i)?;
    let (i, node_type) = ws(ident)(i)?;

    let (i, fields) = delimited(
        ws(char('{')),
        opt(separated_list1(ws(char(',')), ws(parse_field_assign))),
        ws(char('}')),
    )(i)?;
    let fields = fields.unwrap_or_default().into_iter().collect();

    let (i, vector) = opt(tuple((ws(tag("VECTOR")), ws(parse_vector_lit))))(i)?;

    Ok((
        i,
        InsertStatement {
            node_id,
            node_type,
            fields,
            vector: vector.map(|(_, v)| v),
        },
    ))
}

pub(crate) fn parse_update_field_expr(i: &str) -> IResult<&str, (String, FieldValue)> {
    let (i, key) = ws(ident)(i)?;
    let (i, _) = ws(char('='))(i)?;
    let (i, val) = ws(parse_literal_field_value)(i)?;
    Ok((i, (key, val)))
}

pub(crate) fn parse_update(i: &str) -> IResult<&str, UpdateStatement> {
    let (i, _) = ws(tag("UPDATE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, node_id) = ws(parse_u128_id)(i)?;
    let (i, _) = ws(tag("SET"))(i)?;

    let (i, vector_only) = opt(tuple((ws(tag("VECTOR")), ws(parse_vector_lit))))(i)?;

    if let Some((_, vec)) = vector_only {
        return Ok((
            i,
            UpdateStatement {
                node_id,
                fields: std::collections::BTreeMap::new(),
                vector: Some(vec),
            },
        ));
    }

    let (i, parsed_fields) = separated_list1(ws(char(',')), ws(parse_update_field_expr))(i)?;
    let fields = parsed_fields.into_iter().collect();

    Ok((
        i,
        UpdateStatement {
            node_id,
            fields,
            vector: None,
        },
    ))
}

pub(crate) fn parse_delete(i: &str) -> IResult<&str, DeleteStatement> {
    let (i, _) = ws(tag("DELETE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, node_id) = ws(parse_u128_id)(i)?;
    Ok((i, DeleteStatement { node_id }))
}

pub(crate) fn parse_relate(i: &str) -> IResult<&str, RelateStatement> {
    let (i, _) = ws(tag("RELATE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, source_id) = ws(parse_u128_id)(i)?;
    let (i, _) = ws(tag("--\""))(i)?;
    let (i, label) = ws(take_while1(|c| c != '"'))(i)?;
    let (i, _) = ws(tag("\"-->"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, target_id) = ws(parse_u128_id)(i)?;

    let (i, weight) = opt(tuple((ws(tag("WEIGHT")), ws(float))))(i)?;

    Ok((
        i,
        RelateStatement {
            source_id,
            target_id,
            label: label.to_string(),
            weight: weight.map(|(_, w)| w),
        },
    ))
}

pub(crate) fn parse_insert_message(i: &str) -> IResult<&str, InsertMessageStatement> {
    let (i, _) = ws(tag("INSERT"))(i)?;
    let (i, _) = ws(tag("MESSAGE"))(i)?;

    let (i, msg_role) = alt((
        map(ws(tag("SYSTEM")), |_| "system".to_string()),
        map(ws(tag("USER")), |_| "user".to_string()),
        map(ws(tag("ASSISTANT")), |_| "assistant".to_string()),
    ))(i)?;

    let (i, content) = ws(string_literal)(i)?;

    let (i, _) = ws(tag("TO"))(i)?;
    let (i, _) = ws(tag("THREAD#"))(i)?;
    let (i, thread_id) = ws(parse_u128_id)(i)?;

    Ok((
        i,
        InsertMessageStatement {
            msg_role,
            content,
            thread_id,
        },
    ))
}

// ─── SELECT / JOIN / Subquery ──────────────────────────────────

pub(crate) fn parse_join_on(i: &str) -> IResult<&str, (String, String)> {
    let (i, _) = ws(tag("ON"))(i)?;
    let (i, left_field) = ws(ident)(i)?;
    let (i, _) = ws(tag("="))(i)?;
    let (i, right_field) = ws(ident)(i)?;
    Ok((i, (left_field, right_field)))
}

pub(crate) fn parse_join_clause(i: &str) -> IResult<&str, JoinClause> {
    let (i, _) = ws(tag("JOIN"))(i)?;
    let (i, entity) = ws(ident)(i)?;
    let (i, alias) = ws(ident)(i)?;
    let (i, (left_field, right_field)) = parse_join_on(i)?;
    Ok((
        i,
        JoinClause {
            entity,
            alias,
            left_field,
            right_field,
        },
    ))
}

pub(crate) fn parse_subquery_condition_inner(i: &str) -> IResult<&str, SubqueryCondition> {
    let (i, field) = ws(ident)(i)?;
    let (i, op) = ws(parse_rel_op)(i)?;
    let (i, _) = ws(tag("("))(i)?;
    let (i, subquery) = parse_select(i)?;
    // `AS OF` inside a subquery would be silently dropped (sub-plans carry no
    // valid-time filter) — fail fast instead (SCH-03).
    if subquery.as_of_ms.is_some() {
        return Err(nom::Err::Failure(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Verify,
        )));
    }
    let (i, _) = ws(tag(")"))(i)?;
    Ok((
        i,
        SubqueryCondition {
            field,
            op,
            subquery: Box::new(subquery),
        },
    ))
}

/// Parse a single WHERE item — either a regular condition or a subquery condition.
pub(crate) fn parse_where_item(i: &str) -> IResult<&str, WhereItem> {
    // Peek ahead: if after field + op we see '(', it's a subquery.
    // We try subquery first; if it fails with a recoverable error, fall back to
    // a regular condition. A `Failure` (e.g. `AS OF` inside the subquery,
    // SCH-03) propagates so the malformed query never silently drops its
    // WHERE clause.
    match parse_subquery_condition_inner(i) {
        Ok((rest, subq)) => return Ok((rest, WhereItem::Subquery(subq))),
        Err(err @ nom::Err::Failure(_)) => return Err(err),
        Err(_) => {}
    }
    let (rest, cond) = parse_condition(i)?;
    Ok((rest, WhereItem::Condition(cond)))
}

/// A single WHERE item — either a relational/vector condition or a subquery comparison.
#[derive(Debug, Clone, PartialEq)]
pub enum WhereItem {
    /// Regular condition (relational or vector).
    Condition(Condition),
    /// Subquery comparison (e.g. `field op (SELECT ...)`).
    Subquery(SubqueryCondition),
}

/// Parse a `SELECT` query with optional JOINs and subqueries.
pub fn parse_select(i: &str) -> IResult<&str, SelectStatement> {
    let (i, _) = ws(tag("SELECT"))(i)?;

    // Projections: comma-separated identifiers, or "*" for all
    let (i, projections) =
        if let Ok((rest, _)) = ws(tag::<&str, &str, nom::error::Error<&str>>("*"))(i) {
            (rest, Vec::new())
        } else {
            separated_list1(ws(char(',')), ws(ident))(i)?
        };

    let (i, _) = ws(tag("FROM"))(i)?;
    let (i, from_entity) = ws(ident)(i)?;
    let (i, from_alias) = opt(ws(non_keyword_ident))(i)?;
    // Default alias matches `parse_query` (and its docs): "target".
    let from_alias = from_alias.unwrap_or_else(|| "target".to_string());

    // Parse zero or more JOIN clauses
    let (i, join_clauses) = many0(parse_join_clause)(i)?;

    // `AS OF` right after the table spec (`SELECT * FROM ns AS OF t WHERE ...`).
    let (i, as_of_early) = parse_as_of_clause(i)?;

    // Build FromClause tree from JOINs
    let from = if join_clauses.is_empty() {
        FromClause::Single {
            entity: from_entity,
            alias: from_alias,
        }
    } else {
        let mut current = FromClause::Single {
            entity: from_entity,
            alias: from_alias,
        };
        for jc in join_clauses {
            current = FromClause::Join {
                left: Box::new(current),
                right: Box::new(FromClause::Single {
                    entity: jc.entity,
                    alias: jc.alias,
                }),
                left_field: jc.left_field,
                right_field: jc.right_field,
            };
        }
        current
    };

    // WHERE clause with mixed regular and subquery conditions
    let (i, where_items) = opt(tuple((
        ws(tag("WHERE")),
        separated_list1(ws(tag("AND")), parse_where_item),
    )))(i)?;

    let (i, temperature) = opt(tuple((ws(tag("WITH")), ws(tag("TEMPERATURE")), ws(float))))(i)?;

    let as_of_late_start = i;
    let (i, as_of_late) = parse_as_of_clause(i)?;
    let (i, (limit, offset)) = parse_pagination(i)?;
    let as_of_tail_start = i;
    let (i, as_of_tail) = parse_as_of_clause(i)?;
    let as_of_ms = merge_as_of_clauses(
        merge_as_of_clauses(as_of_early, as_of_late, as_of_late_start)?,
        as_of_tail,
        as_of_tail_start,
    )?;

    // Split where_items into regular conditions and subquery conditions
    let (where_conds, subq_conds) = match where_items {
        Some((_, items)) => {
            let mut conds = Vec::new();
            let mut subqs = Vec::new();
            for item in items {
                match item {
                    WhereItem::Condition(c) => conds.push(c),
                    WhereItem::Subquery(s) => subqs.push(s),
                }
            }
            (Some(conds), subqs)
        }
        None => (None, Vec::new()),
    };

    Ok((
        i,
        SelectStatement {
            projections,
            from,
            where_clause: where_conds,
            subquery_conditions: subq_conds,
            temperature: temperature.map(|(_, _, t)| t),
            as_of_ms,
            limit,
            offset,
        },
    ))
}

// ─── Entry Point ───────────────────────────────────────────────

/// WIRE-12 strictness: once `LIMIT`/`OFFSET` was consumed, a non-empty
/// remainder is a parse error. Without this, a clause written after the
/// pagination keywords (`FROM x LIMIT 5 WHERE ...`) would be left as trailing
/// input and silently ignored by the executor (FIND-312 class), returning a
/// page *without* the trailing clause — more misleading than the pre-WIRE-12
/// all-rows no-op. Applied at the statement entry point (not inside
/// `parse_select`) so subqueries — where the caller consumes the closing `)` —
/// keep parsing. A trailing statement terminator (`;`) is tolerated, matching
/// the lenient behavior of non-paginated statements.
fn reject_trailing_after_pagination<'a>(
    rest: &'a str,
    stmt: &Statement,
) -> Result<(), nom::Err<nom::error::Error<&'a str>>> {
    let paginated = match stmt {
        Statement::Query(q) => q.limit.is_some() || q.offset.is_some(),
        Statement::Select(s) => s.limit.is_some() || s.offset.is_some(),
        _ => false,
    };
    if paginated && !rest.trim().trim_end_matches(';').trim().is_empty() {
        return Err(nom::Err::Failure(nom::error::Error::new(
            rest,
            nom::error::ErrorKind::Verify,
        )));
    }
    Ok(())
}

/// Parse any supported VantaQL statement (query, insert, update, delete, relate).
pub fn parse_statement(i: &str) -> IResult<&str, Statement> {
    let (rest, stmt) = alt((
        map(parse_insert_message, Statement::InsertMessage), // Must be before parse_insert to prevent shadowing
        map(parse_insert, Statement::Insert),
        map(parse_update, Statement::Update),
        map(parse_delete, Statement::Delete),
        map(parse_relate, Statement::Relate),
        map(parse_select, Statement::Select), // Must be before parse_query (SELECT would match as alias)
        map(parse_query, Statement::Query),
    ))(i)?;
    reject_trailing_after_pagination(rest, &stmt)?;
    Ok((rest, stmt))
}

// ─── Autocomplete (VS-CORE-06) ──────────────────────────────────

/// Multi-token keywords the grammar parses as a single clause. Offered as one
/// completion on top of [`RESERVED_KEYWORDS`].
pub(crate) const EXTRA_AUTOCOMPLETE_KEYWORDS: &[&str] = &["RANK BY", "WITH TEMPERATURE", "AS OF"];

pub(crate) fn is_keyword_token(token: &str) -> bool {
    RESERVED_KEYWORDS
        .iter()
        .any(|k| k.eq_ignore_ascii_case(token))
}

/// Autocomplete candidates for an IQL editor prefix (VS-CORE-06).
///
/// Returns the completions for the token being typed at the end of `prefix`:
/// IQL keywords (single and multi-word, case-insensitive prefix match) plus
/// identifier tokens already present in the typed text (entity / alias / field
/// names) that extend the current token. Sorted and deduplicated.
///
/// Token-level shim over the statement grammar: `parse_statement` rejects
/// partial input, so completion matches the token stream against the same
/// keyword table the parser uses instead of parsing the prefix.
pub fn autocomplete_prefix(prefix: &str) -> Vec<String> {
    let current = prefix.split_whitespace().next_back().unwrap_or_default();
    let current_lower = current.to_ascii_lowercase();

    let mut out: Vec<String> = RESERVED_KEYWORDS
        .iter()
        .chain(EXTRA_AUTOCOMPLETE_KEYWORDS)
        .filter(|kw| kw.to_ascii_lowercase().starts_with(&current_lower))
        .map(|s| s.to_string())
        .collect();

    // Identifier tokens already present in the statement that extend the
    // current token (e.g. reusing the entity name while typing a WHERE
    // condition). Skip when the current token is empty — at a clause boundary
    // the full keyword list is the useful suggestion, not a dump of every
    // identifier.
    if !current.is_empty() {
        for token in prefix.split_whitespace() {
            if token != current
                && !is_keyword_token(token)
                && token.to_ascii_lowercase().starts_with(&current_lower)
            {
                out.push(token.to_string());
            }
        }
    }

    out.sort();
    out.dedup();
    out
}
