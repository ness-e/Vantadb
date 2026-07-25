//! Snippet generation utilities for lexical search results.
//!
//! Extracts a short excerpt from a payload around the first matched query
//! term, with optional `<strong>` highlighting.

use std::collections::BTreeSet;

/// Folds a character to its ASCII/unaccented equivalent when possible, in lowercase.
pub(crate) fn fold_char(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'Á' | 'À' | 'Â' | 'Ä' | 'Ã' | 'Å' => 'a',
        'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
        'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' | 'Ó' | 'Ò' | 'Ô' | 'Ö' | 'Õ' | 'Ø' => 'o',
        'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
        'ñ' | 'Ñ' => 'n',
        'ç' | 'Ç' => 'c',
        'ý' | 'ÿ' | 'Ý' => 'y',
        _ => c.to_ascii_lowercase(),
    }
}

/// Folds a string into a simplified lowercase ASCII-compatible string for fuzzy matching.
pub(crate) fn fold_str(s: &str) -> String {
    s.chars().map(fold_char).collect()
}

/// Generate a snippet with `<strong>`-wrapped highlighted terms.
pub(crate) fn generate_snippet_with_highlighting(
    payload: &str,
    text_query: &str,
    with_highlighting: bool,
) -> Option<String> {
    let query_plan = crate::text_index::query_plan(text_query);
    let first_token = query_plan.terms.iter().next()?;

    if payload.len() <= 120 {
        if with_highlighting {
            return Some(highlight_terms(payload, &query_plan.terms));
        }
        return Some(payload.to_string());
    }

    let folded_payload = fold_str(payload);
    let folded_first_token = fold_str(first_token);

    let match_at = folded_payload.find(&folded_first_token).unwrap_or(0);
    let mut start = match_at.saturating_sub(48);
    let mut end = match_at
        .saturating_add(first_token.len())
        .saturating_add(72)
        .min(payload.len());
    while start > 0 && !payload.is_char_boundary(start) {
        start -= 1;
    }
    while end < payload.len() && !payload.is_char_boundary(end) {
        end += 1;
    }

    let snippet_text = payload[start..end].trim();

    if with_highlighting {
        let highlighted = highlight_terms(snippet_text, &query_plan.terms);
        let mut snippet = String::new();
        if start > 0 {
            snippet.push_str("...");
        }
        snippet.push_str(&highlighted);
        if end < payload.len() {
            snippet.push_str("...");
        }
        Some(snippet)
    } else {
        let mut snippet = String::new();
        if start > 0 {
            snippet.push_str("...");
        }
        snippet.push_str(snippet_text);
        if end < payload.len() {
            snippet.push_str("...");
        }
        Some(snippet)
    }
}

/// Debug-oriented snippet (no highlighting, plain text).
pub(crate) fn debug_snippet(payload: &str, text_query: &str) -> Option<String> {
    generate_snippet_with_highlighting(payload, text_query, false)
}

/// Wrap every occurrence of any term in `<strong>` tags (case & accent insensitive).
fn highlight_terms(text: &str, terms: &BTreeSet<String>) -> String {
    let mut result = String::new();
    let mut i = 0;
    let chars: Vec<char> = text.chars().collect();

    // Pre-fold terms for comparison
    let folded_terms: Vec<(Vec<char>, &String)> = terms
        .iter()
        .map(|t| (fold_str(t).chars().collect(), t))
        .collect();

    while i < chars.len() {
        let mut matched = false;

        for (term_folded_chars, _orig_term) in &folded_terms {
            if i + term_folded_chars.len() <= chars.len() {
                let slice_chars = &chars[i..i + term_folded_chars.len()];
                let slice_folded: Vec<char> = slice_chars.iter().map(|&c| fold_char(c)).collect();

                if slice_folded == *term_folded_chars {
                    let slice_str: String = slice_chars.iter().collect();
                    result.push_str("<strong>");
                    result.push_str(&slice_str);
                    result.push_str("</strong>");
                    i += term_folded_chars.len();
                    matched = true;
                    break;
                }
            }
        }

        if !matched {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── generate_snippet_with_highlighting ────────────────────────────────

    #[test]
    fn short_payload_no_markup() {
        let payload = "hello world";
        let result = generate_snippet_with_highlighting(payload, "hello", false);
        assert_eq!(result.as_deref(), Some("hello world"));
    }

    #[test]
    fn short_payload_with_highlighting() {
        let payload = "hello world";
        let result = generate_snippet_with_highlighting(payload, "hello", true);
        assert_eq!(result.as_deref(), Some("<strong>hello</strong> world"));
    }

    #[test]
    fn long_payload_is_truncated_around_match() {
        let payload = "The quick brown fox jumps over the lazy dog. ".repeat(5);
        let result = generate_snippet_with_highlighting(&payload, "fox", false);
        let snippet = result.expect("expected a snippet");
        assert!(snippet.contains("fox"), "snippet must contain match");
        assert!(
            snippet.len() < payload.len(),
            "snippet should be shorter than original"
        );
    }

    #[test]
    fn debug_snippet_returns_plain_text() {
        let payload = "hello world test";
        let result = debug_snippet(payload, "world");
        assert_eq!(result.as_deref(), Some("hello world test"));
    }

    #[test]
    fn debug_snippet_long_truncated() {
        let payload = "The quick brown fox jumps over the lazy dog. ".repeat(5);
        let result = debug_snippet(&payload, "fox");
        let snippet = result.expect("expected a snippet");
        assert!(!snippet.contains("<strong>"));
        assert!(snippet.contains("fox"));
    }

    #[test]
    fn non_matching_query_falls_back_to_prefix_snippet() {
        // When no token matches, the function extracts a snippet from position 0.
        let payload = "hello world this is a long payload that goes on and on";
        let result = generate_snippet_with_highlighting(payload, "zzzzz", false);
        let snippet = result.expect("expected a snippet even without match");
        assert!(snippet.contains("hello"));
    }

    #[test]
    fn multiple_terms_highlighted() {
        let payload = "quick brown fox";
        let result = generate_snippet_with_highlighting(payload, "quick fox", true);
        let s = result.expect("expected a snippet");
        assert_eq!(s, "<strong>quick</strong> brown <strong>fox</strong>");
    }

    #[test]
    fn empty_query_returns_none() {
        let payload = "hello world";
        let result = debug_snippet(payload, "");
        assert!(result.is_none());
    }

    #[test]
    fn long_payload_does_not_panic() {
        let payload = "hello world ".repeat(200);
        let result = debug_snippet(&payload, "hello");
        assert!(result.is_some());
    }

    #[test]
    fn unicode_folding_snippet_accent_match() {
        let payload = "El café molido y la crème brûlée son exquisitos";
        let result = generate_snippet_with_highlighting(payload, "cafe creme", true);
        let s = result.expect("expected a snippet");
        assert!(s.contains("<strong>café</strong>"), "Got: {}", s);
        assert!(s.contains("<strong>crème</strong>"), "Got: {}", s);
    }

    #[test]
    fn unicode_folding_snippet_unaccented_query() {
        let payload = "Un rápido zorro marrón salta sobre el perro perezoso";
        let result = generate_snippet_with_highlighting(payload, "rapido zorro", true);
        let s = result.expect("expected a snippet");
        assert!(s.contains("<strong>rápido</strong>"), "Got: {}", s);
        assert!(s.contains("<strong>zorro</strong>"), "Got: {}", s);
    }
}
