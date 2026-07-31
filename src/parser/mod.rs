//! Historical parser surface.
//!
//! The stable embedded memory API lives in `src/sdk.rs`.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, separated_list1},
    number::complete::{double, float},
    sequence::{delimited, tuple},
    IResult, Parser,
};

use crate::node::FieldValue;
use crate::query::*;

/// Strip leading and trailing whitespace around a parser.
pub fn ws<'a, F, O, E: nom::error::ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

fn ident(i: &str) -> IResult<&str, String> {
    let (i, id) = recognize(tuple((
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"), tag("#"), tag(".")))),
    )))(i)?;
    Ok((i, id.to_string()))
}

fn parse_number(i: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(i)
}

fn string_literal(input: &str) -> IResult<&str, String> {
    let (input, _) = char('"')(input)?;
    let mut s = String::new();
    let mut chars = input.chars().peekable();
    let mut consumed = 0;

    while let Some(c) = chars.next() {
        consumed += c.len_utf8();
        if c == '"' {
            let remaining = &input[consumed..];
            return Ok((remaining, s));
        } else if c == '\\' {
            if let Some(escaped_char) = chars.next() {
                consumed += escaped_char.len_utf8();
                match escaped_char {
                    'n' => s.push('\n'),
                    'r' => s.push('\r'),
                    't' => s.push('\t'),
                    '\\' => s.push('\\'),
                    '"' => s.push('"'),
                    other => {
                        s.push('\\');
                        s.push(other);
                    }
                }
            } else {
                s.push('\\');
            }
        } else {
            s.push(c);
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

fn parse_u128_id(i: &str) -> IResult<&str, u128> {
    map_res(digit1, str::parse)(i)
}

fn parse_i64(i: &str) -> IResult<&str, i64> {
    map_res(recognize(tuple((opt(char('-')), digit1))), str::parse)(i)
}

fn parse_literal_field_value(i: &str) -> IResult<&str, FieldValue> {
    alt((
        map(string_literal, FieldValue::String),
        map(ws(tag("true")), |_| FieldValue::Bool(true)),
        map(ws(tag("false")), |_| FieldValue::Bool(false)),
        map(ws(tag("null")), |_| FieldValue::Null),
        // double BEFORE parse_i64: "3.14" should be Float(3.14), not Int(3).
        // double handles both integer and float literals; integer-only
        // strings like "42" parse as Float(42.0) — semantically correct
        // and no precision loss for values up to 2^53.
        map(ws(double), FieldValue::Float),
        map(ws(parse_i64), FieldValue::Int),
    ))(i)
}

fn parse_traversal(i: &str) -> IResult<&str, Traversal> {
    let (i, _) = ws(tag("SIGUE"))(i)?;
    let (i, min_depth) = ws(parse_number)(i)?;
    let (i, _) = ws(tag(".."))(i)?;
    let (i, max_depth) = ws(parse_number)(i)?;
    let (i, edge_label) = ws(string_literal)(i)?;
    let (i, target_type) = opt(tuple((ws(tag("TYPE")), ws(ident))))(i)?;
    let (i, alias) = opt(tuple((ws(tag("AS")), ws(ident))))(i)?;

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

fn parse_rel_op(i: &str) -> IResult<&str, RelOp> {
    alt((
        map(tag("="), |_| RelOp::Eq),
        map(tag("!="), |_| RelOp::Neq),
        map(tag(">="), |_| RelOp::Gte),
        map(tag(">"), |_| RelOp::Gt),
        map(tag("<="), |_| RelOp::Lte),
        map(tag("<"), |_| RelOp::Lt),
    ))(i)
}

fn parse_condition(i: &str) -> IResult<&str, Condition> {
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
        // Relational Query: p.pais = "VZLA"
        map(
            tuple((ws(ident), ws(parse_rel_op), ws(string_literal))),
            |(field, op, val)| Condition::Relational(field, op, FieldValue::String(val)),
        ),
    ))(i)
}

/// Parse a `FROM`/`MATCH` query statement.
pub fn parse_query(i: &str) -> IResult<&str, Query> {
    let (i, _) = ws(alt((tag("FROM"), tag("MATCH"))))(i)?;
    let (i, from_entity) = ws(ident)(i)?;

    let (i, traversal) = opt(parse_traversal)(i)?;

    let (i, target_alias) = opt(ws(ident))(i)?;
    let target_alias = target_alias.unwrap_or_else(|| "target".to_string());

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
        },
    ))
}

// ─── DML (Data Manipulation Language) ──────────────────────────

fn parse_field_assign(i: &str) -> IResult<&str, (String, FieldValue)> {
    let (i, key) = ws(ident)(i)?;
    let (i, _) = ws(char(':'))(i)?;
    let (i, val) = ws(parse_literal_field_value)(i)?;
    Ok((i, (key, val)))
}

fn parse_vector_lit(i: &str) -> IResult<&str, Vec<f32>> {
    delimited(
        ws(char('[')),
        separated_list1(ws(char(',')), ws(float)),
        ws(char(']')),
    )(i)
}

fn parse_insert(i: &str) -> IResult<&str, InsertStatement> {
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

fn parse_update_field_expr(i: &str) -> IResult<&str, (String, FieldValue)> {
    let (i, key) = ws(ident)(i)?;
    let (i, _) = ws(char('='))(i)?;
    let (i, val) = ws(parse_literal_field_value)(i)?;
    Ok((i, (key, val)))
}

fn parse_update(i: &str) -> IResult<&str, UpdateStatement> {
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

fn parse_delete(i: &str) -> IResult<&str, DeleteStatement> {
    let (i, _) = ws(tag("DELETE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, node_id) = ws(parse_u128_id)(i)?;
    Ok((i, DeleteStatement { node_id }))
}

fn parse_relate(i: &str) -> IResult<&str, RelateStatement> {
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

fn parse_insert_message(i: &str) -> IResult<&str, InsertMessageStatement> {
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

fn parse_join_on(i: &str) -> IResult<&str, (String, String)> {
    let (i, _) = ws(tag("ON"))(i)?;
    let (i, left_field) = ws(ident)(i)?;
    let (i, _) = ws(tag("="))(i)?;
    let (i, right_field) = ws(ident)(i)?;
    Ok((i, (left_field, right_field)))
}

fn parse_join_clause(i: &str) -> IResult<&str, JoinClause> {
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

fn parse_subquery_condition_inner(i: &str) -> IResult<&str, SubqueryCondition> {
    let (i, field) = ws(ident)(i)?;
    let (i, op) = ws(parse_rel_op)(i)?;
    let (i, _) = ws(tag("("))(i)?;
    let (i, subquery) = parse_select(i)?;
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
fn parse_where_item(i: &str) -> IResult<&str, WhereItem> {
    // Peek ahead: if after field + op we see '(', it's a subquery.
    // We try subquery first; if it fails, fall back to regular condition.
    if let Ok((rest, subq)) = parse_subquery_condition_inner(i) {
        return Ok((rest, WhereItem::Subquery(subq)));
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
    let (i, from_alias) = opt(ws(ident))(i)?;
    let from_alias = from_alias.unwrap_or_else(|| from_entity.clone());

    // Parse zero or more JOIN clauses
    let (i, join_clauses) = many0(parse_join_clause)(i)?;

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
        },
    ))
}

// ─── Entry Point ───────────────────────────────────────────────

/// Parse any supported VantaQL statement (query, insert, update, delete, relate).
pub fn parse_statement(i: &str) -> IResult<&str, Statement> {
    alt((
        map(parse_insert_message, Statement::InsertMessage), // Must be before parse_insert to prevent shadowing
        map(parse_insert, Statement::Insert),
        map(parse_update, Statement::Update),
        map(parse_delete, Statement::Delete),
        map(parse_relate, Statement::Relate),
        map(parse_select, Statement::Select), // Must be before parse_query (SELECT would match as alias)
        map(parse_query, Statement::Query),
    ))(i)
}

#[cfg(test)]
#[allow(unused_imports, dead_code)]
mod tests {
    use super::*;
    use crate::node::FieldValue;

    // ─── Helper ──────────────────────────────────────────────────

    fn parse_ok<T: std::fmt::Debug>(result: IResult<&str, T>) -> T {
        result.expect("parse should succeed").1
    }

    fn parse_err<T: std::fmt::Debug>(result: IResult<&str, T>) {
        assert!(result.is_err(), "expected parse error, got {:?}", result);
    }

    // ─── Basic parser functions ─────────────────────────────────

    #[test]
    fn test_ws_identity() {
        let mut parser = ws(tag::<&str, &str, nom::error::Error<&str>>("hello"));
        assert_eq!(parser("  hello  ").unwrap(), ("", "hello"));
        assert_eq!(parser("hello").unwrap(), ("", "hello"));
        assert_eq!(parser("  hello world").unwrap(), ("world", "hello"));
    }

    #[test]
    fn test_ident_simple() {
        assert_eq!(ident("foo").unwrap().1, "foo");
        assert_eq!(ident("_private").unwrap().1, "_private");
        assert_eq!(ident("abc123").unwrap().1, "abc123");
        assert_eq!(ident("foo.bar").unwrap().1, "foo.bar");
        assert_eq!(ident("foo#bar").unwrap().1, "foo#bar");
        assert_eq!(ident("Usuario#usr45").unwrap().1, "Usuario#usr45");
        assert_eq!(ident("Persona.nombre").unwrap().1, "Persona.nombre");
    }

    #[test]
    fn test_ident_fails_on_digit_start() {
        assert!(ident("123abc").is_err());
        assert!(ident("").is_err());
    }

    #[test]
    fn test_ident_consumes_til_non_match() {
        assert_eq!(
            ident("hello world").unwrap(),
            (" world", "hello".to_string())
        );
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("42").unwrap().1, 42);
        assert_eq!(parse_number("0").unwrap().1, 0);
        assert_eq!(parse_number("999").unwrap().1, 999);
        assert_eq!(parse_number("42abc").unwrap().1, 42);
        assert!(parse_number("abc").is_err());
        assert!(parse_number("").is_err());
    }

    #[test]
    fn test_parse_u128_id() {
        assert_eq!(parse_u128_id("0").unwrap().1, 0u128);
        assert_eq!(parse_u128_id("42").unwrap().1, 42u128);
        assert_eq!(
            parse_u128_id("12345678901234567890").unwrap().1,
            12345678901234567890u128
        );
        assert!(parse_u128_id("").is_err());
        assert!(parse_u128_id("abc").is_err());
    }

    #[test]
    fn test_parse_i64() {
        assert_eq!(parse_i64("42").unwrap().1, 42i64);
        assert_eq!(parse_i64("-42").unwrap().1, -42i64);
        assert_eq!(parse_i64("0").unwrap().1, 0i64);
        assert!(parse_i64("").is_err());
        assert!(parse_i64("abc").is_err());
    }

    // ─── String literal ─────────────────────────────────────────

    #[test]
    fn test_string_literal_basic() {
        assert_eq!(string_literal(r#""hello""#).unwrap().1, "hello");
        assert_eq!(string_literal(r#""""#).unwrap().1, "");
        assert_eq!(string_literal(r#""hello world""#).unwrap().1, "hello world");
    }

    #[test]
    fn test_string_literal_escapes() {
        assert_eq!(
            string_literal(r#""hello\nworld""#).unwrap().1,
            "hello\nworld"
        );
        assert_eq!(
            string_literal(r#""hello\tworld""#).unwrap().1,
            "hello\tworld"
        );
        assert_eq!(
            string_literal(r#""hello\rworld""#).unwrap().1,
            "hello\rworld"
        );
        assert_eq!(
            string_literal(r#""hello\\world""#).unwrap().1,
            "hello\\world"
        );
        assert_eq!(
            string_literal(r#""hello\"world""#).unwrap().1,
            "hello\"world"
        );
    }

    #[test]
    fn test_string_literal_unknown_escape() {
        // Unknown escape sequences preserve the backslash
        assert_eq!(string_literal(r#""hello\x""#).unwrap().1, "hello\\x");
        assert_eq!(string_literal(r#""\x""#).unwrap().1, "\\x");
    }

    #[test]
    fn test_string_literal_unterminated() {
        assert!(string_literal(r#""hello"#).is_err());
        assert!(string_literal(r#""""#).is_ok()); // this is valid: empty string
    }

    #[test]
    fn test_string_literal_no_quote() {
        assert!(string_literal("hello").is_err());
        assert!(string_literal("").is_err());
    }

    #[test]
    fn test_string_literal_trailing_backslash() {
        // Trailing backslash at end of input
        assert!(string_literal(r#""hello\"#).is_err());
    }

    // ─── RelOp ──────────────────────────────────────────────────

    #[test]
    fn test_parse_rel_op_eq() {
        assert_eq!(parse_rel_op("=").unwrap().1, RelOp::Eq);
        assert_eq!(parse_rel_op("!=").unwrap().1, RelOp::Neq);
        assert_eq!(parse_rel_op(">=").unwrap().1, RelOp::Gte);
        assert_eq!(parse_rel_op(">").unwrap().1, RelOp::Gt);
        assert_eq!(parse_rel_op("<=").unwrap().1, RelOp::Lte);
        assert_eq!(parse_rel_op("<").unwrap().1, RelOp::Lt);
    }

    #[test]
    fn test_parse_rel_op_unknown() {
        // "==" matches "=" then returns Eq (nom alt tries in order, first match wins)
        assert_eq!(parse_rel_op("==").unwrap().1, RelOp::Eq);
        assert!(parse_rel_op("~").is_err());
        assert!(parse_rel_op("").is_err());
    }

    #[test]
    fn test_parse_rel_op_neq_precedence() {
        // "!=" should match fully, not just "!"
        assert_eq!(parse_rel_op("!= ").unwrap().1, RelOp::Neq);
    }

    // ─── Literal field values ───────────────────────────────────

    #[test]
    fn test_parse_literal_field_value_string() {
        assert_eq!(
            parse_literal_field_value(r#""hello""#).unwrap().1,
            FieldValue::String("hello".to_string())
        );
    }

    #[test]
    fn test_parse_literal_field_value_bool() {
        assert_eq!(
            parse_literal_field_value("true").unwrap().1,
            FieldValue::Bool(true)
        );
        assert_eq!(
            parse_literal_field_value("false").unwrap().1,
            FieldValue::Bool(false)
        );
    }

    #[test]
    fn test_parse_literal_field_value_null() {
        assert_eq!(
            parse_literal_field_value("null").unwrap().1,
            FieldValue::Null
        );
    }

    #[test]
    fn test_parse_literal_field_value_int() {
        // All numeric literals parse as Float (double alt takes precedence over parse_i64)
        assert_eq!(
            parse_literal_field_value("42").unwrap().1,
            FieldValue::Float(42.0)
        );
        assert_eq!(
            parse_literal_field_value("-7").unwrap().1,
            FieldValue::Float(-7.0)
        );
    }

    #[test]
    #[allow(clippy::approx_constant)] // 3.14 is intentional here, not an approximation
    fn test_parse_literal_field_value_float() {
        // double now comes BEFORE parse_i64 in alt(), so "3.14" → Float(3.14)
        let val = parse_literal_field_value("3.14").unwrap().1;
        assert_eq!(val, FieldValue::Float(3.14));

        // Integer literals still parse via double → Float (e.g. 42 → Float(42.0))
        let val2 = parse_literal_field_value("42").unwrap().1;
        assert_eq!(val2, FieldValue::Float(42.0));

        // Verify the double parser works directly
        let (remaining, f) = double::<&str, nom::error::Error<&str>>("1.5").unwrap();
        assert!((f - 1.5).abs() < 1e-10);
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_parse_literal_field_value_empty() {
        assert!(parse_literal_field_value("").is_err());
    }

    #[test]
    fn test_parse_literal_field_value_bool_ws() {
        // Boolean parsing includes ws wrapper
        assert_eq!(
            parse_literal_field_value("  true  ").unwrap().1,
            FieldValue::Bool(true)
        );
    }

    // ─── Conditions ─────────────────────────────────────────────

    #[test]
    fn test_parse_condition_relational() {
        let (_, cond) = parse_condition(r#"pais = "VZLA""#).unwrap();
        assert_eq!(
            cond,
            Condition::Relational(
                "pais".to_string(),
                RelOp::Eq,
                FieldValue::String("VZLA".to_string())
            )
        );
    }

    #[test]
    fn test_parse_condition_relational_neq() {
        let (_, cond) = parse_condition(r#"edad != "18""#).unwrap();
        assert_eq!(
            cond,
            Condition::Relational(
                "edad".to_string(),
                RelOp::Neq,
                FieldValue::String("18".to_string())
            )
        );
    }

    #[test]
    fn test_parse_condition_vector_sim() {
        let (_, cond) = parse_condition(r#"bio ~ "rust expert", min = 0.88"#).unwrap();
        assert_eq!(
            cond,
            Condition::VectorSim("bio".to_string(), "rust expert".to_string(), 0.88)
        );
    }

    #[test]
    fn test_parse_condition_vector_sim_trailing_ws() {
        // ws(float) consumes trailing whitespace via multispace0
        let (_, cond) = parse_condition(r#"bio ~ "data", min = 0.5   "#).unwrap();
        assert_eq!(
            cond,
            Condition::VectorSim("bio".to_string(), "data".to_string(), 0.5)
        );
    }

    #[test]
    fn test_parse_condition_invalid() {
        assert!(parse_condition("").is_err());
        assert!(parse_condition("foo").is_err());
    }

    // ─── Traversal ──────────────────────────────────────────────

    #[test]
    fn test_parse_traversal_basic() {
        let (_, trav) = parse_traversal(r#"SIGUE 1..3 "amigo""#).unwrap();
        assert_eq!(
            trav,
            Traversal {
                min_depth: 1,
                max_depth: 3,
                edge_label: "amigo".to_string(),
                target_type: None,
                alias: None,
            }
        );
    }

    #[test]
    fn test_parse_traversal_with_target_type() {
        let (_, trav) = parse_traversal(r#"SIGUE 2..5 "conoce" TYPE Persona"#).unwrap();
        assert_eq!(trav.target_type, Some("Persona".to_string()));
        assert_eq!(trav.min_depth, 2);
        assert_eq!(trav.max_depth, 5);
        assert_eq!(trav.edge_label, "conoce");
    }

    #[test]
    fn test_parse_traversal_with_alias() {
        let (_, trav) = parse_traversal(r#"SIGUE 1..2 "edge" AS alias"#).unwrap();
        assert_eq!(trav.alias, Some("alias".to_string()));
    }

    #[test]
    fn test_parse_traversal_full() {
        let (_, trav) = parse_traversal(r#"SIGUE 1..3 "friend" TYPE Person AS p"#).unwrap();
        assert_eq!(
            trav,
            Traversal {
                min_depth: 1,
                max_depth: 3,
                edge_label: "friend".to_string(),
                target_type: Some("Person".to_string()),
                alias: Some("p".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_traversal_invalid() {
        assert!(parse_traversal("").is_err());
        assert!(parse_traversal("SIGUE").is_err());
        assert!(parse_traversal("NOT_SIGUE").is_err());
    }

    // ─── Field assign ───────────────────────────────────────────

    #[test]
    fn test_parse_field_assign_string() {
        let (_, (k, v)) = parse_field_assign(r#"nombre: "Eros""#).unwrap();
        assert_eq!(k, "nombre");
        assert_eq!(v, FieldValue::String("Eros".to_string()));
    }

    #[test]
    fn test_parse_field_assign_int() {
        // All numeric literals parse as Float (double alt takes precedence)
        let (_, (k, v)) = parse_field_assign("edad: 28").unwrap();
        assert_eq!(k, "edad");
        assert_eq!(v, FieldValue::Float(28.0));
    }

    #[test]
    fn test_parse_field_assign_bool() {
        let (_, (k, v)) = parse_field_assign("activo: true").unwrap();
        assert_eq!(k, "activo");
        assert_eq!(v, FieldValue::Bool(true));
    }

    #[test]
    fn test_parse_field_assign_null() {
        let (_, (k, v)) = parse_field_assign("campo: null").unwrap();
        assert_eq!(k, "campo");
        assert_eq!(v, FieldValue::Null);
    }

    #[test]
    fn test_parse_field_assign_missing_colon() {
        assert!(parse_field_assign("nombre").is_err());
    }

    // ─── Vector literal ─────────────────────────────────────────

    #[test]
    fn test_parse_vector_lit_basic() {
        assert_eq!(
            parse_vector_lit("[0.1, -0.4, 0.9]").unwrap().1,
            vec![0.1, -0.4, 0.9]
        );
    }

    #[test]
    fn test_parse_vector_lit_single() {
        assert_eq!(parse_vector_lit("[0.5]").unwrap().1, vec![0.5]);
    }

    #[test]
    fn test_parse_vector_lit_empty() {
        assert!(parse_vector_lit("[]").is_err()); // separated_list1 requires at least one
    }

    #[test]
    fn test_parse_vector_lit_no_brackets() {
        assert!(parse_vector_lit("0.1, 0.2").is_err());
    }

    // ─── Parse Query ────────────────────────────────────────────

    #[test]
    fn test_parse_query_simple_from() {
        let (_, q) = parse_query("FROM Usuario").unwrap();
        assert_eq!(q.from_entity, "Usuario");
        assert_eq!(q.target_alias, "target");
        assert!(q.traversal.is_none());
        assert!(q.where_clause.is_none());
        assert!(q.fetch.is_none());
        assert!(q.rank_by.is_none());
        assert!(q.temperature.is_none());
        assert!(q.owner_role.is_none());
    }

    #[test]
    fn test_parse_query_match_keyword() {
        let (_, q) = parse_query("MATCH Node").unwrap();
        assert_eq!(q.from_entity, "Node");
    }

    #[test]
    fn test_parse_query_with_alias() {
        let (_, q) = parse_query("FROM Person p").unwrap();
        assert_eq!(q.from_entity, "Person");
        assert_eq!(q.target_alias, "p");
    }

    #[test]
    fn test_parse_query_with_traversal() {
        let (_, q) = parse_query(r#"FROM Usuario SIGUE 1..3 "amigo""#).unwrap();
        assert!(q.traversal.is_some());
        let trav = q.traversal.unwrap();
        assert_eq!(trav.min_depth, 1);
        assert_eq!(trav.max_depth, 3);
        assert_eq!(trav.edge_label, "amigo");
    }

    #[test]
    fn test_parse_query_with_where_single() {
        // Must provide explicit alias before WHERE, otherwise opt(ident) consumes "WHERE" as alias
        let (_, q) = parse_query(r#"FROM Person p WHERE edad = "25""#).unwrap();
        let conds = q.where_clause.unwrap();
        assert_eq!(conds.len(), 1);
        assert_eq!(
            conds[0],
            Condition::Relational(
                "edad".to_string(),
                RelOp::Eq,
                FieldValue::String("25".to_string())
            )
        );
    }

    #[test]
    fn test_parse_query_with_where_multiple() {
        let (_, q) = parse_query(r#"FROM Person p WHERE pais = "VZLA" AND edad = "30""#).unwrap();
        let conds = q.where_clause.unwrap();
        assert_eq!(conds.len(), 2);
    }

    #[test]
    fn test_parse_query_with_where_vector_sim() {
        let (_, q) = parse_query(r#"FROM Item i WHERE bio ~ "great", min = 0.75"#).unwrap();
        let conds = q.where_clause.unwrap();
        assert_eq!(conds.len(), 1);
        match &conds[0] {
            Condition::VectorSim(field, query, min) => {
                assert_eq!(field, "bio");
                assert_eq!(query, "great");
                assert!((*min - 0.75).abs() < 1e-5);
            }
            _ => panic!("expected VectorSim"),
        }
    }

    #[test]
    fn test_parse_query_with_fetch() {
        let (_, q) = parse_query(r#"FROM Person p FETCH nombre, email"#).unwrap();
        assert_eq!(
            q.fetch,
            Some(vec!["nombre".to_string(), "email".to_string()])
        );
    }

    #[test]
    fn test_parse_query_with_rank_by() {
        let (_, q) = parse_query(r#"FROM Person p RANK BY relevancia DESC"#).unwrap();
        let rank = q.rank_by.unwrap();
        assert_eq!(rank.field, "relevancia");
        assert!(rank.desc);
    }

    #[test]
    fn test_parse_query_with_rank_by_asc() {
        let (_, q) = parse_query(r#"FROM Person p RANK BY score"#).unwrap();
        let rank = q.rank_by.unwrap();
        assert_eq!(rank.field, "score");
        assert!(!rank.desc);
    }

    #[test]
    fn test_parse_query_with_temperature() {
        let (_, q) = parse_query(r#"FROM Person p WITH TEMPERATURE 0.5"#).unwrap();
        assert!((q.temperature.unwrap() - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_parse_query_with_role() {
        let (_, q) = parse_query(r#"FROM Person p ROLE "admin""#).unwrap();
        assert_eq!(q.owner_role, Some("admin".to_string()));
    }

    #[test]
    fn test_parse_query_full() {
        let input = r#"
            FROM Usuario#usr45
            SIGUE 1..3 "amigo" Persona
            WHERE Persona.pais = "VZLA" AND Persona.bio ~ "rust", min = 0.88
            FETCH Persona.nombre, Persona.email
            RANK BY Persona.relevancia DESC
            WITH TEMPERATURE 0.5
            ROLE "admin"
        "#;
        let (_, q) = parse_query(input).unwrap();
        assert_eq!(q.from_entity, "Usuario#usr45");
        assert!(q.traversal.is_some());
        assert_eq!(q.where_clause.as_ref().map(|c| c.len()), Some(2));
        assert_eq!(
            q.fetch,
            Some(vec![
                "Persona.nombre".to_string(),
                "Persona.email".to_string()
            ])
        );
        assert!(q.rank_by.unwrap().desc);
        assert!((q.temperature.unwrap() - 0.5).abs() < 1e-5);
        assert_eq!(q.owner_role, Some("admin".to_string()));
    }

    #[test]
    fn test_parse_query_default_alias() {
        // When no target alias is given and no ident follows, it defaults to "target"
        // NOTE: "WHERE" would be consumed as alias; this tests the truly-no-token case
        let (_, q) = parse_query("FROM Node").unwrap();
        assert_eq!(q.target_alias, "target");
    }

    #[test]
    fn test_parse_query_invalid_keyword() {
        assert!(parse_query("SELECT * FROM foo").is_err());
        assert!(parse_query("FROM").is_err());
    }

    #[test]
    fn test_parse_query_extra_tokens_remain() {
        // With an explicit alias, remaining tokens are left unconsumed
        let (remaining, _) = parse_query("FROM Node p extra_stuff").unwrap();
        assert_eq!(remaining.trim(), "extra_stuff");
    }

    // ─── Insert Statement ───────────────────────────────────────

    #[test]
    fn test_parse_insert_basic() {
        let input = r#"INSERT NODE#101 TYPE Usuario { nombre: "Eros", edad: 28 }"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Insert(ins) => {
                assert_eq!(ins.node_id, 101);
                assert_eq!(ins.node_type, "Usuario");
                assert_eq!(ins.fields.len(), 2);
                assert_eq!(
                    ins.fields.get("nombre").unwrap(),
                    &FieldValue::String("Eros".to_string())
                );
                // All numeric literals parse as Float (double alt takes precedence)
                assert_eq!(ins.fields.get("edad").unwrap(), &FieldValue::Float(28.0));
                assert!(ins.vector.is_none());
            }
            _ => panic!("expected Insert"),
        }
    }

    #[test]
    fn test_parse_insert_with_vector() {
        let input =
            r#"INSERT NODE#42 TYPE Item { name: "test", price: 99 } VECTOR [0.1, -0.4, 0.9]"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Insert(ins) => {
                assert_eq!(ins.node_id, 42);
                assert_eq!(ins.node_type, "Item");
                assert_eq!(ins.fields.len(), 2);
                let vec = ins.vector.unwrap();
                assert!((vec[0] - 0.1).abs() < 1e-5);
                assert!((vec[1] + 0.4).abs() < 1e-5);
            }
            _ => panic!("expected Insert"),
        }
    }

    #[test]
    fn test_parse_insert_empty_fields() {
        let input = r#"INSERT NODE#7 TYPE Empty {}"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Insert(ins) => {
                assert_eq!(ins.node_id, 7);
                assert!(ins.fields.is_empty());
            }
            _ => panic!("expected Insert"),
        }
    }

    #[test]
    fn test_parse_insert_node_id_zero() {
        let input = r#"INSERT NODE#0 TYPE Root { label: "root" }"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Insert(ins) => {
                assert_eq!(ins.node_id, 0);
            }
            _ => panic!("expected Insert"),
        }
    }

    // ─── Update Statement ───────────────────────────────────────

    #[test]
    fn test_parse_update_fields() {
        let input = r#"UPDATE NODE#101 SET nombre = "Eros Dev", activo = true"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Update(upd) => {
                assert_eq!(upd.node_id, 101);
                assert_eq!(upd.fields.len(), 2);
                assert_eq!(
                    upd.fields.get("nombre").unwrap(),
                    &FieldValue::String("Eros Dev".to_string())
                );
                assert_eq!(upd.fields.get("activo").unwrap(), &FieldValue::Bool(true));
                assert!(upd.vector.is_none());
            }
            _ => panic!("expected Update"),
        }
    }

    #[test]
    fn test_parse_update_vector_only() {
        let input = r#"UPDATE NODE#42 SET VECTOR [0.5, -0.2]"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Update(upd) => {
                assert_eq!(upd.node_id, 42);
                assert!(upd.fields.is_empty());
                let vec = upd.vector.unwrap();
                assert!((vec[0] - 0.5).abs() < 1e-5);
            }
            _ => panic!("expected Update"),
        }
    }

    #[test]
    fn test_parse_update_single_field() {
        let input = r#"UPDATE NODE#5 SET name = "new""#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Update(upd) => {
                assert_eq!(upd.node_id, 5);
                assert_eq!(upd.fields.len(), 1);
            }
            _ => panic!("expected Update"),
        }
    }

    // ─── Delete Statement ───────────────────────────────────────

    #[test]
    fn test_parse_delete() {
        let input = "DELETE NODE#5";
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Delete(del) => {
                assert_eq!(del.node_id, 5);
            }
            _ => panic!("expected Delete"),
        }
    }

    #[test]
    fn test_parse_delete_large_id() {
        let input = "DELETE NODE#999999999";
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Delete(del) => {
                assert_eq!(del.node_id, 999999999);
            }
            _ => panic!("expected Delete"),
        }
    }

    // ─── Relate Statement ───────────────────────────────────────

    #[test]
    fn test_parse_relate_basic() {
        let input = r#"RELATE NODE#1 --"amigo"--> NODE#2"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Relate(rel) => {
                assert_eq!(rel.source_id, 1);
                assert_eq!(rel.target_id, 2);
                assert_eq!(rel.label, "amigo");
                assert!(rel.weight.is_none());
            }
            _ => panic!("expected Relate"),
        }
    }

    #[test]
    fn test_parse_relate_with_weight() {
        let input = r#"RELATE NODE#1 --"amigo"--> NODE#2 WEIGHT 0.95"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Relate(rel) => {
                assert_eq!(rel.source_id, 1);
                assert_eq!(rel.target_id, 2);
                assert_eq!(rel.label, "amigo");
                assert!((rel.weight.unwrap() - 0.95).abs() < 1e-5);
            }
            _ => panic!("expected Relate"),
        }
    }

    #[test]
    fn test_parse_relate_large_ids() {
        let input = r#"RELATE NODE#100 --"edge"--> NODE#200 WEIGHT 0.5"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Relate(rel) => {
                assert_eq!(rel.source_id, 100);
                assert_eq!(rel.target_id, 200);
            }
            _ => panic!("expected Relate"),
        }
    }

    // ─── Insert Message Statement ───────────────────────────────

    #[test]
    fn test_parse_insert_message_system() {
        let input = r#"INSERT MESSAGE SYSTEM "hello world" TO THREAD#200"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::InsertMessage(msg) => {
                assert_eq!(msg.msg_role, "system");
                assert_eq!(msg.content, "hello world");
                assert_eq!(msg.thread_id, 200);
            }
            _ => panic!("expected InsertMessage"),
        }
    }

    #[test]
    fn test_parse_insert_message_user() {
        let input = r#"INSERT MESSAGE USER "what is rust?" TO THREAD#1"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::InsertMessage(msg) => {
                assert_eq!(msg.msg_role, "user");
                assert_eq!(msg.content, "what is rust?");
            }
            _ => panic!("expected InsertMessage"),
        }
    }

    #[test]
    fn test_parse_insert_message_assistant() {
        let input = r#"INSERT MESSAGE ASSISTANT "I can help" TO THREAD#5"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::InsertMessage(msg) => {
                assert_eq!(msg.msg_role, "assistant");
            }
            _ => panic!("expected InsertMessage"),
        }
    }

    #[test]
    fn test_parse_insert_message_escaped() {
        let input = r#"INSERT MESSAGE SYSTEM "Hello \"world\" \\ test" TO THREAD#200"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::InsertMessage(msg) => {
                assert_eq!(msg.content, "Hello \"world\" \\ test");
            }
            _ => panic!("expected InsertMessage"),
        }
    }

    // ─── parse_statement dispatcher ─────────────────────────────

    #[test]
    fn test_parse_statement_insert_message_before_insert() {
        // InsertMessage must be parsed BEFORE Insert (starts with "INSERT" too)
        let input = r#"INSERT MESSAGE SYSTEM "hi" TO THREAD#1"#;
        let (_, stmt) = parse_statement(input).unwrap();
        assert!(matches!(stmt, Statement::InsertMessage(_)));
    }

    #[test]
    fn test_parse_statement_query() {
        let (_, stmt) = parse_statement("FROM Node").unwrap();
        assert!(matches!(stmt, Statement::Query(_)));
    }

    #[test]
    fn test_parse_statement_invalid() {
        assert!(parse_statement("").is_err());
        assert!(parse_statement("GARBAGE").is_err());
        assert!(parse_statement("   ").is_err());
    }

    // ─── Edge cases ─────────────────────────────────────────────

    #[test]
    fn test_parse_query_empty_string() {
        assert!(parse_query("").is_err());
    }

    #[test]
    fn test_parse_query_whitespace_only() {
        assert!(parse_query("   ").is_err());
    }

    #[test]
    fn test_parse_condition_vs_statement() {
        // Parsing a condition directly shouldn't consume statement-level keywords
        let (_, cond) = parse_condition(r#"field ~ "query", min = 0.5"#).unwrap();
        assert_eq!(
            cond,
            Condition::VectorSim("field".to_string(), "query".to_string(), 0.5)
        );
    }

    #[test]
    fn test_ws_on_empty() {
        let mut parser = ws(tag::<&str, &str, nom::error::Error<&str>>("x"));
        assert!(parser("").is_err());
    }

    #[test]
    fn test_ident_with_special_chars() {
        assert_eq!(ident("a.b.c").unwrap().1, "a.b.c");
        assert_eq!(ident("a#b#c").unwrap().1, "a#b#c");
        assert_eq!(ident("a_b_c").unwrap().1, "a_b_c");
    }

    #[test]
    fn test_parse_number_zero() {
        assert_eq!(parse_number("0abc").unwrap().1, 0);
        assert_eq!(parse_number("0").unwrap().1, 0);
    }

    #[test]
    fn test_parse_vector_lit_negative() {
        let vec = parse_vector_lit("[-1.5, 0.0, 2.5]").unwrap().1;
        assert!((vec[0] + 1.5).abs() < 1e-5);
        assert!((vec[1] - 0.0).abs() < 1e-5);
        assert!((vec[2] - 2.5).abs() < 1e-5);
    }

    #[test]
    #[allow(clippy::approx_constant)] // 3.14 is intentional here, not an approximation
    fn test_parse_field_assign_float() {
        // double now comes before parse_i64: "3.14" → Float(3.14)
        let (_, (k, v)) = parse_field_assign("price: 3.14").unwrap();
        assert_eq!(k, "price");
        assert_eq!(v, FieldValue::Float(3.14));
    }

    #[test]
    fn test_string_literal_with_spaces_and_symbols() {
        assert_eq!(
            string_literal(r#""hello world! @#$%""#).unwrap().1,
            "hello world! @#$%"
        );
    }

    #[test]
    fn test_parse_i64_negative_zero() {
        assert_eq!(parse_i64("-0").unwrap().1, 0i64);
    }

    #[test]
    fn test_parse_rel_op_gte() {
        assert_eq!(parse_rel_op(">=42").unwrap().1, RelOp::Gte);
    }

    #[test]
    fn test_parse_rel_op_lte() {
        assert_eq!(parse_rel_op("<=42").unwrap().1, RelOp::Lte);
    }

    #[test]
    fn test_parse_query_mixed_conditions() {
        let input = r#"FROM Person p WHERE name = "Alice" AND bio ~ "developer", min = 0.9"#;
        let (_, q) = parse_query(input).unwrap();
        let conds = q.where_clause.unwrap();
        assert_eq!(conds.len(), 2);
        assert!(matches!(conds[0], Condition::Relational(..)));
        assert!(matches!(conds[1], Condition::VectorSim(..)));
    }

    #[test]
    fn test_parse_statement_multiple_calls() {
        // Verify parse_statement is re-usable (no internal state)
        let stmt1 = parse_statement("DELETE NODE#1").unwrap().1;
        let stmt2 = parse_statement("DELETE NODE#2").unwrap().1;
        assert!(matches!(stmt1, Statement::Delete(_)));
        assert!(matches!(stmt2, Statement::Delete(_)));
    }

    // ─── SELECT / JOIN / Subquery ──────────────────────────────

    #[test]
    fn test_parse_select_basic() {
        let input = r#"SELECT * FROM Person"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Select(sel) => {
                assert!(sel.projections.is_empty());
                assert!(matches!(sel.from, crate::query::FromClause::Single { .. }));
                assert!(sel.where_clause.is_none());
                assert!(sel.subquery_conditions.is_empty());
            }
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn test_parse_select_with_alias() {
        let input = r#"SELECT name, age FROM Person p"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Select(sel) => {
                assert_eq!(sel.projections, vec!["name", "age"]);
                match &sel.from {
                    crate::query::FromClause::Single { entity, alias } => {
                        assert_eq!(entity, "Person");
                        assert_eq!(alias, "p");
                    }
                    _ => panic!("expected Single from clause"),
                }
            }
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn test_parse_select_single_field() {
        let input = r#"SELECT name FROM Person"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Select(sel) => {
                assert_eq!(sel.projections, vec!["name"]);
            }
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn test_parse_select_join() {
        let input = r#"SELECT * FROM Person p JOIN Address a ON p.addr_id = a.id"#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Select(sel) => match &sel.from {
                crate::query::FromClause::Join {
                    left,
                    right,
                    left_field,
                    right_field,
                } => {
                    assert_eq!(left_field, "p.addr_id");
                    assert_eq!(right_field, "a.id");
                    match (&**left, &**right) {
                        (
                            crate::query::FromClause::Single {
                                entity: e1,
                                alias: a1,
                            },
                            crate::query::FromClause::Single {
                                entity: e2,
                                alias: a2,
                            },
                        ) => {
                            assert_eq!(e1, "Person");
                            assert_eq!(a1, "p");
                            assert_eq!(e2, "Address");
                            assert_eq!(a2, "a");
                        }
                        _ => panic!("expected Single/Single join children"),
                    }
                }
                _ => panic!("expected Join from clause"),
            },
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn test_parse_select_where_relational() {
        let input = r#"SELECT * FROM Person p WHERE name = "Alice""#;
        let (_, stmt) = parse_statement(input).unwrap();
        match stmt {
            Statement::Select(sel) => {
                let conds = sel.where_clause.expect("expected WHERE conditions");
                assert_eq!(conds.len(), 1);
                assert!(
                    matches!(&conds[0], crate::query::Condition::Relational(..)),
                    "expected relational condition"
                );
            }
            _ => panic!("expected Select"),
        }
    }

    #[test]
    fn test_parse_statement_select_is_dispatched() {
        let (_, stmt) = parse_statement("SELECT * FROM Node").unwrap();
        assert!(matches!(stmt, Statement::Select(_)));
    }
}
