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

fn parse_traversal(i: &str) -> IResult<&str, Traversal> {
    let (i, _) = ws(tag("SIGUE"))(i)?;
    let (i, min_depth) = ws(parse_number)(i)?;
    let (i, _) = ws(tag(".."))(i)?;
    let (i, max_depth) = ws(parse_number)(i)?;
    let (i, edge_label) = ws(string_literal)(i)?;
    Ok((i, Traversal { min_depth, max_depth, edge_label }))
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

    Ok((i, Query {
        from_entity,
        traversal,
        target_alias,
        where_clause: where_clause.map(|(_, conds)| conds),
        fetch: fetch.map(|(_, f)| f),
        rank_by: rank_by.map(|(_, f, d)| RankBy { field: f, desc: d.is_some() }),
        temperature: temperature.map(|(_, _, t)| t),
    }))
}
