use nom::{
    branch::alt,
    bytes::complete::{is_not, tag},
    character::complete::{alpha1, alphanumeric1, char, multispace0, none_of},
    number::complete::f32,
    combinator::{map, recognize},
    multi::{many0, many1},
    sequence::{delimited, preceded, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum LispExpr {
    List(Vec<LispExpr>),
    Map(Vec<(LispExpr, LispExpr)>),
    Atom(String),
    Keyword(String),
    StringLiteral(String),
    Number(f32),
    Variable(String),
}

fn parse_keyword(i: &str) -> IResult<&str, LispExpr> {
    map(
        preceded(char(':'), recognize(many1(alt((alphanumeric1, tag("-")))))),
        |s: &str| LispExpr::Keyword(s.to_string()),
    )(i)
}

fn parse_variable(i: &str) -> IResult<&str, LispExpr> {
    map(
        preceded(char('?'), alpha1),
        |s: &str| LispExpr::Variable(s.to_string()),
    )(i)
}

fn parse_string(i: &str) -> IResult<&str, LispExpr> {
    let parse_str = delimited(char('"'), is_not("\""), char('"'));
    map(parse_str, |s: &str| LispExpr::StringLiteral(s.to_string()))(i)
}

fn parse_atom(i: &str) -> IResult<&str, LispExpr> {
    map(
        recognize(many1(alt((alphanumeric1, tag("-"), tag("_"))))),
        |s: &str| LispExpr::Atom(s.to_string()),
    )(i)
}

fn parse_number(i: &str) -> IResult<&str, LispExpr> {
    map(f32, |n| LispExpr::Number(n))(i)
}

fn parse_expr(i: &str) -> IResult<&str, LispExpr> {
    delimited(
        multispace0,
        alt((
            parse_list,
            parse_map,
            parse_keyword,
            parse_variable,
            parse_string,
            parse_number,
            parse_atom,
        )),
        multispace0,
    )(i)
}

fn parse_list(i: &str) -> IResult<&str, LispExpr> {
    let parse_inside = many0(parse_expr);
    map(
        delimited(char('('), parse_inside, char(')')),
        |exprs| LispExpr::List(exprs),
    )(i)
}

fn parse_map(i: &str) -> IResult<&str, LispExpr> {
    let parse_pairs = many0(tuple((parse_expr, parse_expr)));
    map(
        delimited(char('{'), parse_pairs, char('}')),
        |pairs| LispExpr::Map(pairs),
    )(i)
}

pub fn parse(input: &str) -> Result<LispExpr, String> {
    match parse_expr(input) {
        Ok((rem, expr)) if rem.trim().is_empty() => Ok(expr),
        Ok((rem, _)) => Err(format!("Unparsed trailing data: {}", rem)),
        Err(e) => Err(format!("Parse error: {}", e)),
    }
}
