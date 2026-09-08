//! IQL lexer — tokens primitivos y literales (FIND-50).
//!
//! Primitivas de bajo nivel sobre `nom`: identificadores, keywords,
//! números, string literals y valores de campo tipados. Sin conocimiento
//! de statements; la gramática vive en [`super::grammar`].

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize, verify},
    multi::{many0, separated_list1},
    number::complete::{double, float},
    sequence::{delimited, tuple},
    IResult, Parser,
};

use crate::node::FieldValue;

/// Strip leading and trailing whitespace around a parser.
pub fn ws<'a, F, O, E: nom::error::ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Parser<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub(crate) fn ident(i: &str) -> IResult<&str, String> {
    let (i, id) = recognize(tuple((
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_"), tag("#"), tag(".")))),
    )))(i)?;
    Ok((i, id.to_string()))
}

/// Keywords reservadas del lenguaje. No pueden usarse como alias en `parse_query`,
/// porque un `opt(ident)` sin guarda consumiría la cláusula siguiente como alias
/// (ej. `FROM Person WHERE ...` se comía `WHERE` y descartaba el filtro).
pub(crate) const RESERVED_KEYWORDS: &[&str] = &[
    "AND",
    "AS",
    "DELETE",
    "DESC",
    "FETCH",
    "FROM",
    "INSERT",
    "JOIN",
    "LIMIT",
    "MATCH",
    "MESSAGE",
    "ON",
    "PROFILE",
    "RANK",
    "RELATE",
    "ROLE",
    "SELECT",
    "SET",
    "SIGUE",
    "SYSTEM",
    "TEMPERATURE",
    "TIMEOUT",
    "TIPO",
    "TO",
    "TYPE",
    "UPDATE",
    "USER",
    "VECTOR",
    "WEIGHT",
    "WHERE",
    "WITH",
];

/// `ident` que falla si el token es una keyword reservada. Se usa para aliases
/// opcionales: deja el keyword sin consumir para que lo tome la cláusula real.
pub(crate) fn non_keyword_ident(i: &str) -> IResult<&str, String> {
    verify(ident, |s: &str| !RESERVED_KEYWORDS.contains(&s))(i)
}

pub(crate) fn parse_number(i: &str) -> IResult<&str, u32> {
    map_res(digit1, str::parse)(i)
}

pub(crate) fn string_literal(input: &str) -> IResult<&str, String> {
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

pub(crate) fn parse_u128_id(i: &str) -> IResult<&str, u128> {
    map_res(digit1, str::parse)(i)
}

pub(crate) fn parse_i64(i: &str) -> IResult<&str, i64> {
    map_res(recognize(tuple((opt(char('-')), digit1))), str::parse)(i)
}

pub(crate) fn parse_literal_field_value(i: &str) -> IResult<&str, FieldValue> {
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

pub(crate) fn parse_vector_lit(i: &str) -> IResult<&str, Vec<f32>> {
    delimited(
        ws(char('[')),
        separated_list1(ws(char(',')), ws(float)),
        ws(char(']')),
    )(i)
}
