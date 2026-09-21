//! Tolerant JSON parsing for LLM responses (port of TDAM
//! `offload/local-llm/parsers/json-utils.ts` + `sanitizeJsonForParse`).
//!
//! LLMs frequently wrap JSON in markdown code fences, surround it with prose,
//! emit trailing commas, or leak raw control characters (newlines/tabs) into
//! string values. [`extract_json`] walks cheap repair strategies in order and
//! returns the first that deserializes.

use serde::de::DeserializeOwned;

/// Parse `raw` as JSON `T`, tolerating code fences, surrounding prose,
/// trailing commas and unescaped control characters inside strings.
///
/// Strategy order (cheapest first, TDAM json-utils.ts:12-57):
/// 1. Direct parse.
/// 2. Markdown code fence (```json … ```).
/// 3. First `{`/`[` … last `}`/`]` slice.
/// 4. Control-character sanitize + trailing-comma repair on the slice.
/// 5. Sanitize + trailing-comma repair on the whole string.
pub fn extract_json<T: DeserializeOwned>(raw: &str) -> Option<T> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(v) = try_parse(trimmed) {
        return Some(v);
    }

    if let Some(inner) = strip_code_fence(trimmed) {
        if let Some(v) = try_parse(inner) {
            return Some(v);
        }
    }

    for (open, close) in [('{', '}'), ('[', ']')] {
        if let Some(candidate) = slice_between(trimmed, open, close) {
            if let Some(v) = try_parse(&candidate) {
                return Some(v);
            }
            let sanitized = sanitize_json_for_parse(&candidate);
            if let Some(v) = try_parse(&sanitized) {
                return Some(v);
            }
            let fixed = fix_trailing_commas(&sanitized);
            if let Some(v) = try_parse(&fixed) {
                return Some(v);
            }
        }
    }

    let fixed = fix_trailing_commas(&sanitize_json_for_parse(trimmed));
    try_parse(&fixed)
}

/// Escape unescaped control characters (U+0000–U+001F) inside JSON string
/// literals so the JSON parses (port of TDAM `sanitizeJsonForParse`,
/// sanitize.ts:316-334). String-aware: escape sequences and `"` boundaries are
/// respected; structural whitespace outside strings is untouched.
pub fn sanitize_json_for_parse(s: &str) -> String {
    // Phase 1: escape control chars inside string literals.
    let escaped = escape_control_chars_in_json_strings(s);
    if serde_json::from_str::<serde_json::Value>(&escaped).is_ok() {
        return escaped;
    }
    // Phase 2: strip rare control chars globally, preserving \t \n \r.
    escaped
        .chars()
        .filter(|c| !matches!(*c, '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}'))
        .collect()
}

/// Walk a JSON text and escape U+0000–U+001F that appear inside string
/// literals. Already-escaped sequences are copied verbatim (TDAM
/// `escapeControlCharsInJsonStrings`, sanitize.ts:345-405).
fn escape_control_chars_in_json_strings(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if in_string {
            if ch == '\\' {
                // Copy the escape and its target verbatim.
                out.push(ch);
                if let Some(next) = chars.next() {
                    out.push(next);
                }
                continue;
            }
            if ch == '"' {
                out.push(ch);
                in_string = false;
                continue;
            }
            let code = ch as u32;
            if code <= 0x1f {
                let short = match code {
                    0x08 => Some("\\b"),
                    0x09 => Some("\\t"),
                    0x0a => Some("\\n"),
                    0x0c => Some("\\f"),
                    0x0d => Some("\\r"),
                    _ => None,
                };
                if let Some(short) = short {
                    out.push_str(short);
                } else {
                    out.push_str(&format!("\\u{code:04x}"));
                }
                continue;
            }
            out.push(ch);
        } else if ch == '"' {
            out.push(ch);
            in_string = true;
        } else {
            out.push(ch);
        }
    }
    out
}

/// Remove commas directly before `}`/`]` (string-aware, so `"a, }"` inside a
/// string literal is left intact).
pub fn fix_trailing_commas(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match c {
            '"' => {
                in_string = true;
                out.push(c);
            }
            ',' => {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                    i += 1; // drop the comma, keep following whitespace
                    continue;
                }
                out.push(c);
            }
            _ => out.push(c),
        }
        i += 1;
    }
    out
}

fn try_parse<T: DeserializeOwned>(s: &str) -> Option<T> {
    serde_json::from_str(s).ok()
}

/// Extract the first ```…``` (optionally ```json) block, ignoring trailing
/// prose after the closing fence.
fn strip_code_fence(s: &str) -> Option<&str> {
    let start = s.find("```")?;
    let after_open = &s[start + 3..];
    let after_open = after_open.strip_prefix("json").unwrap_or(after_open);
    let after_open = after_open.trim_start();
    let end = after_open.find("```")?;
    Some(after_open[..end].trim())
}

/// Slice from the first `open` char to the last `close` char.
fn slice_between(s: &str, open: char, close: char) -> Option<String> {
    let first = s.find(open)?;
    let last = s.rfind(close)?;
    if last <= first {
        return None;
    }
    Some(s[first..=last].to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{extract_json, fix_trailing_commas, sanitize_json_for_parse};

    fn parse(raw: &str) -> Option<Value> {
        extract_json(raw)
    }

    #[test]
    fn direct_parse() {
        let v = parse(r#"{"a": 1}"#).expect("direct parse");
        assert_eq!(v["a"], 1);
    }

    #[test]
    fn code_fence_parse() {
        let v = parse("Here is the result:\n```json\n{\"a\": 2}\n```\nHope that helps!")
            .expect("fence parse");
        assert_eq!(v["a"], 2);
    }

    #[test]
    fn fence_without_lang_tag() {
        let v = parse("```\n[1, 2, 3]\n```").expect("bare fence parse");
        assert_eq!(v.as_array().map(|a| a.len()), Some(3));
    }

    #[test]
    fn slice_between_braces_skips_prose() {
        let v = parse("Sure! The data is: {\"a\": 3} and that's it.").expect("slice parse");
        assert_eq!(v["a"], 3);
    }

    #[test]
    fn trailing_commas_repaired() {
        let v = parse(r#"{"a": [1, 2,], "b": {"c": 3,},}"#).expect("repaired");
        assert_eq!(v["a"][1], 2);
        assert_eq!(v["b"]["c"], 3);
    }

    #[test]
    fn trailing_comma_inside_string_not_touched() {
        let fixed = fix_trailing_commas(r#"{"a": "x, }", "b": 1,}"#);
        assert_eq!(fixed, r#"{"a": "x, }", "b": 1}"#);
    }

    #[test]
    fn control_chars_inside_strings_escaped() {
        // Raw newline + tab inside a string value must be escaped (TDAM sanitize.ts).
        let v = parse("{\"a\": \"line1\nline2\tend\"}").expect("control chars escaped");
        assert_eq!(v["a"], "line1\nline2\tend");
    }

    #[test]
    fn control_char_escape_does_not_touch_escapes() {
        let s = sanitize_json_for_parse(r#"{"a": "\\n", "b": "\n"}"#);
        assert_eq!(s, r#"{"a": "\\n", "b": "\n"}"#);
    }

    #[test]
    fn control_chars_outside_strings_left_alone() {
        // Structural whitespace (newlines between values) is untouched.
        let v = parse("{\n  \"a\": 1\n}").expect("structural whitespace");
        assert_eq!(v["a"], 1);
    }

    #[test]
    fn empty_input_is_none() {
        assert!(parse("   ").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn non_json_is_none() {
        assert!(parse("no json here at all").is_none());
    }
}
