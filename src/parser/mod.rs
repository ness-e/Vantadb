use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::{many0, separated_list1},
    sequence::{delimited, tuple},
    number::complete::float,
    IResult, Parser,
};

pub mod lisp;

use crate::query::*;
use crate::node::FieldValue;

pub fn ws<'a, F, O, E: nom::error::ParseError<&'a str>>(mut inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
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

fn string_literal(i: &str) -> IResult<&str, String> {
    delimited(char('"'), take_while1(|c| c != '"'), char('"'))
        .map(|s: &str| s.to_string())
        .parse(i)
}

fn parse_u64_id(i: &str) -> IResult<&str, u64> {
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
        map(ws(parse_i64), FieldValue::Int),
        map(ws(float), |f: f32| FieldValue::Float(f as f64)),
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

    Ok((i, Traversal { 
        min_depth, 
        max_depth, 
        edge_label,
        target_type: target_type.map(|(_, t)| t),
        alias: alias.map(|(_, a)| a),
    }))
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
            |(field, _, query, _, _, _, min_score)| Condition::VectorSim(field, query, min_score)
        ),
        // Relational Query: p.pais = "VZLA"
        map(
            tuple((
                ws(ident),
                ws(parse_rel_op),
                ws(string_literal),
            )),
            |(field, op, val)| Condition::Relational(field, op, FieldValue::String(val))
        )
    ))(i)
}

pub fn parse_query(i: &str) -> IResult<&str, Query> {
    let (i, _) = ws(tag("FROM"))(i)?;
    let (i, from_entity) = ws(ident)(i)?;
    
    let (i, traversal) = opt(parse_traversal)(i)?;
    
    let (i, target_alias) = opt(ws(ident))(i)?;
    let target_alias = target_alias.unwrap_or_else(|| "target".to_string());

    let (i, where_clause) = opt(tuple((
        ws(tag("WHERE")),
        separated_list1(ws(tag("AND")), parse_condition)
    )))(i)?;
    
    let (i, fetch) = opt(tuple((
        ws(tag("FETCH")),
        separated_list1(ws(char(',')), ws(ident))
    )))(i)?;

    let (i, rank_by) = opt(tuple((
        ws(tag("RANK BY")),
        ws(ident),
        opt(ws(tag("DESC")))
    )))(i)?;

    let (i, temperature) = opt(tuple((
        ws(tag("WITH")),
        ws(tag("TEMPERATURE")),
        ws(float),
    )))(i)?;

    let (i, owner_role) = opt(tuple((
        ws(tag("ROLE")),
        ws(string_literal),
    )))(i)?;

    Ok((i, Query {
        from_entity,
        traversal,
        target_alias,
        where_clause: where_clause.map(|(_, conds)| conds),
        fetch: fetch.map(|(_, f)| f),
        rank_by: rank_by.map(|(_, f, d)| RankBy { field: f, desc: d.is_some() }),
        temperature: temperature.map(|(_, _, t)| t),
        owner_role: owner_role.map(|(_, r)| r),
    }))
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
    let (i, node_id) = ws(parse_u64_id)(i)?;
    let (i, _) = ws(tag("TYPE"))(i)?;
    let (i, node_type) = ws(ident)(i)?;
    
    let (i, fields) = delimited(
        ws(char('{')),
        opt(separated_list1(ws(char(',')), ws(parse_field_assign))),
        ws(char('}')),
    )(i)?;
    let fields = fields.unwrap_or_default().into_iter().collect();

    let (i, vector) = opt(tuple((
        ws(tag("VECTOR")),
        ws(parse_vector_lit)
    )))(i)?;

    Ok((i, InsertStatement {
        node_id,
        node_type,
        fields,
        vector: vector.map(|(_, v)| v),
    }))
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
    let (i, node_id) = ws(parse_u64_id)(i)?;
    let (i, _) = ws(tag("SET"))(i)?;

    let (i, vector_only) = opt(tuple((ws(tag("VECTOR")), ws(parse_vector_lit))))(i)?;

    if let Some((_, vec)) = vector_only {
        return Ok((i, UpdateStatement {
            node_id,
            fields: std::collections::BTreeMap::new(),
            vector: Some(vec),
        }));
    }

    let (i, parsed_fields) = separated_list1(ws(char(',')), ws(parse_update_field_expr))(i)?;
    let fields = parsed_fields.into_iter().collect();

    Ok((i, UpdateStatement {
        node_id,
        fields,
        vector: None,
    }))
}

fn parse_delete(i: &str) -> IResult<&str, DeleteStatement> {
    let (i, _) = ws(tag("DELETE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, node_id) = ws(parse_u64_id)(i)?;
    Ok((i, DeleteStatement { node_id }))
}

fn parse_relate(i: &str) -> IResult<&str, RelateStatement> {
    let (i, _) = ws(tag("RELATE"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, source_id) = ws(parse_u64_id)(i)?;
    let (i, _) = ws(tag("--\""))(i)?;
    let (i, label) = ws(take_while1(|c| c != '"'))(i)?;
    let (i, _) = ws(tag("\"-->"))(i)?;
    let (i, _) = ws(tag("NODE#"))(i)?;
    let (i, target_id) = ws(parse_u64_id)(i)?;
    
    let (i, weight) = opt(tuple((ws(tag("WEIGHT")), ws(float))))(i)?;

    Ok((i, RelateStatement {
        source_id,
        target_id,
        label: label.to_string(),
        weight: weight.map(|(_, w)| w),
    }))
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
    let (i, thread_id) = ws(parse_u64_id)(i)?;

    Ok((i, InsertMessageStatement {
        msg_role,
        content,
        thread_id,
    }))
}

// ─── Entry Point ───────────────────────────────────────────────

pub fn parse_statement(i: &str) -> IResult<&str, Statement> {
    alt((
        map(parse_insert_message, Statement::InsertMessage), // Must be before parse_insert to prevent shadowing
        map(parse_insert, Statement::Insert),
        map(parse_update, Statement::Update),
        map(parse_delete, Statement::Delete),
        map(parse_relate, Statement::Relate),
        map(parse_query, Statement::Query),
    ))(i)
}
