//! IQL lexer — tokens primitivos y literales (FIND-50).
//!
//! Primitivas de bajo nivel sobre `nom`: identificadores, keywords,
//! números, string literals y valores de campo tipados. Sin conocimiento
//! de statements; la gramática vive en [`super::grammar`].

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0},
    combinator::{map, map_res, not, opt, peek, recognize, verify},
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
///
/// Incluye las palabras significativas de cláusula que no son keywords de
/// statement (`MIN` de `min = <score>`, `RRF_K`/`CANDIDATE_K` de `PROFILE`,
/// `BY` de `RANK BY`/`GROUP BY`, `ASSISTANT` de `INSERT MESSAGE`): aceptarlas
/// como alias descartaba la cláusula entera (FIND-526).
pub(crate) const RESERVED_KEYWORDS: &[&str] = &[
    "AND",
    "AS",
    "ASSISTANT",
    "BY",
    "CANDIDATE_K",
    "DELETE",
    "DESC",
    "FETCH",
    "FROM",
    "GROUP",
    "INSERT",
    "JOIN",
    "LIMIT",
    "MATCH",
    "MESSAGE",
    "MIN",
    "OF",
    "OFFSET",
    "ON",
    "PROFILE",
    "RANK",
    "RELATE",
    "ROLE",
    "RRF_K",
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
///
/// La comparación es case-insensitive (FIND-526): una keyword en minúscula se
/// aceptaba como alias y la cláusula siguiente se perdía en silencio
/// (`FROM Doc where x = 1` → alias `where`, filtro descartado). El fallo sigue
/// siendo `Err::Error` — el `opt(...)` del alias lo convierte en "sin alias" y
/// el keyword lo consume su cláusula (o lo rechaza el trailing-input estricto).
pub(crate) fn non_keyword_ident(i: &str) -> IResult<&str, String> {
    verify(ident, |s: &str| {
        !RESERVED_KEYWORDS
            .iter()
            .any(|kw| kw.eq_ignore_ascii_case(s))
    })(i)
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

/// Unsigned decimal literal for unix-ms timestamps (`AS OF <ms>`, SCH-03).
/// Rejects a float tail (`.`/`e`/`E`) so `12.5` fails instead of silently
/// parsing as `12` with a dangling remainder.
pub(crate) fn parse_u64_id(i: &str) -> IResult<&str, u64> {
    map_res(
        recognize(tuple((
            digit1,
            not(peek(alt((char('.'), char('e'), char('E'))))),
        ))),
        str::parse,
    )(i)
}

/// Integer literal: optional sign + digits, rejecting a float tail (`.`, `e`,
/// `E`) so `42` parses as an integer while `3.14`/`1e5` fall through to the
/// float parser.
pub(crate) fn parse_i64(i: &str) -> IResult<&str, i64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            not(peek(alt((char('.'), char('e'), char('E'))))),
        ))),
        str::parse,
    )(i)
}

/// `f64` literal that rejects `NaN`/`±Inf`/`infinity` with a committed
/// `Failure` (FIND-522).
///
/// nom's `double`/`float` accept those spellings (case-insensitive) and
/// overflowing literals like `1e400` parse to `inf`. A non-finite value
/// degrades silently downstream (`score >= NaN` is always false → zero rows
/// without error; a NaN stored in a vector poisons the index). The failure
/// uses `ErrorKind::Float` so the executor maps it to a stable message.
pub(crate) fn parse_finite_f64(i: &str) -> IResult<&str, f64> {
    let (rest, value) = double(i)?;
    if value.is_finite() {
        Ok((rest, value))
    } else {
        Err(nom::Err::Failure(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Float,
        )))
    }
}

/// `f32` counterpart of [`parse_finite_f64`] — same rejection contract.
pub(crate) fn parse_finite_f32(i: &str) -> IResult<&str, f32> {
    let (rest, value) = float(i)?;
    if value.is_finite() {
        Ok((rest, value))
    } else {
        Err(nom::Err::Failure(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Float,
        )))
    }
}

pub(crate) fn parse_literal_field_value(i: &str) -> IResult<&str, FieldValue> {
    alt((
        map(string_literal, FieldValue::String),
        map(ws(tag("true")), |_| FieldValue::Bool(true)),
        map(ws(tag("false")), |_| FieldValue::Bool(false)),
        map(ws(tag("null")), |_| FieldValue::Null),
        // parse_i64 BEFORE double: integer literals are Int (exact across the
        // whole i64 range — including values > 2^53, which lose precision as
        // f64). parse_i64 rejects a float tail, so "3.14" and "1e5" still fall
        // through to double → Float. Integers beyond i64 also fall through to
        // double (documented fallback).
        map(ws(parse_i64), FieldValue::Int),
        // FIND-522: `nan`/`inf`/`infinity` and overflow literals (`1e400`)
        // are a committed Failure, never a stored NaN.
        map(ws(parse_finite_f64), FieldValue::Float),
    ))(i)
}

pub(crate) fn parse_vector_lit(i: &str) -> IResult<&str, Vec<f32>> {
    delimited(
        ws(char('[')),
        // FIND-522: a NaN element would be written into the index silently.
        separated_list1(ws(char(',')), ws(parse_finite_f32)),
        ws(char(']')),
    )(i)
}
