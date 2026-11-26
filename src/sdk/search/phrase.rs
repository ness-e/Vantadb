//! Phrase-matching utilities for lexical search.
//!
//! These functions verify whether a sequence of query tokens (a "phrase")
//! appears in the correct positional order within a document's term index.
//! They are pure functions operating on pre-computed term position data.

use std::collections::BTreeMap;

/// Check that ALL phrases in a list match the document's term positions.
///
/// Returns `true` if every phrase (or the empty list) has at least one
/// occurrence where its tokens appear in order at consecutive positions.
pub fn text_positions_match_phrases<K>(
    term_positions: &BTreeMap<K, Vec<u32>>,
    phrases: &[Vec<String>],
) -> bool
where
    K: AsRef<str> + Ord,
{
    phrases
        .iter()
        .all(|phrase| text_positions_match_phrase(term_positions, phrase))
}

/// Check whether a single phrase appears in the document's term positions.
///
/// A phrase matches when the first token appears at some position, and each
/// subsequent token appears at that position + offset (i.e. consecutively).
/// A single-token phrase matches if the token has at least one position.
/// An empty phrase trivially matches.
pub fn text_positions_match_phrase<K>(
    term_positions: &BTreeMap<K, Vec<u32>>,
    phrase: &[String],
) -> bool
where
    K: AsRef<str> + Ord,
{
    let Some(first_token) = phrase.first() else {
        return true;
    };
    let Some(first_positions) = find_positions(term_positions, first_token) else {
        return false;
    };
    if phrase.len() == 1 {
        return !first_positions.is_empty();
    }

    first_positions.iter().any(|start| {
        phrase.iter().enumerate().skip(1).all(|(offset, token)| {
            let Some(positions) = find_positions(term_positions, token) else {
                return false;
            };
            positions.contains(&start.saturating_add(offset as u32))
        })
    })
}

/// Find the positions of a token in a per-document term position map.
///
/// Maps are tiny (a handful of query terms per document), so a linear scan
/// is fine. ponytail: O(n) lookup per token; switch to a HashMap keyed by
/// `&str` if phrase matching ever becomes a measurable hot path.
fn find_positions<'a, K>(
    term_positions: &'a BTreeMap<K, Vec<u32>>,
    token: &str,
) -> Option<&'a Vec<u32>>
where
    K: AsRef<str>,
{
    term_positions
        .iter()
        .find(|(key, _)| key.as_ref() == token)
        .map(|(_, positions)| positions)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── text_positions_match_phrase ───────────────────────────────────────

    #[test]
    fn empty_phrase_trivially_matches() {
        let positions = BTreeMap::<String, Vec<u32>>::new();
        let phrase: Vec<String> = vec![];
        assert!(text_positions_match_phrase(&positions, &phrase));
    }

    #[test]
    fn single_token_with_position_matches() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("hello".into(), vec![0, 5, 10]);
        let phrase = vec!["hello".to_string()];
        assert!(text_positions_match_phrase(&positions, &phrase));
    }

    #[test]
    fn single_token_with_no_positions_does_not_match() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("hello".into(), vec![]);
        let phrase = vec!["hello".to_string()];
        assert!(!text_positions_match_phrase(&positions, &phrase));
    }

    #[test]
    fn token_not_in_document_does_not_match() {
        let positions = BTreeMap::<String, Vec<u32>>::new();
        let phrase = vec!["missing".to_string()];
        assert!(!text_positions_match_phrase(&positions, &phrase));
    }

    #[test]
    fn consecutive_tokens_at_correct_position_match() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("hello".into(), vec![0, 5, 10]);
        positions.insert("world".into(), vec![1, 6, 11]);
        let phrase = vec!["hello".to_string(), "world".to_string()];
        assert!(text_positions_match_phrase(&positions, &phrase));
    }

    #[test]
    fn consecutive_tokens_at_wrong_position_do_not_match() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("hello".into(), vec![0, 5, 10]);
        positions.insert("world".into(), vec![3, 8, 13]); // offset != 1
        let phrase = vec!["hello".to_string(), "world".to_string()];
        assert!(!text_positions_match_phrase(&positions, &phrase));
    }

    #[test]
    fn three_token_phrase_at_correct_positions_matches() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("the".into(), vec![0, 10, 20]);
        positions.insert("quick".into(), vec![1, 11, 21]);
        positions.insert("fox".into(), vec![2, 12, 22]);
        let phrase = vec!["the".to_string(), "quick".to_string(), "fox".to_string()];
        assert!(text_positions_match_phrase(&positions, &phrase));
    }

    #[test]
    fn second_token_of_phrase_not_in_document_does_not_match() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("hello".into(), vec![0]);
        // "world" missing
        let phrase = vec!["hello".to_string(), "world".to_string()];
        assert!(!text_positions_match_phrase(&positions, &phrase));
    }

    #[test]
    fn overlapping_tokens_different_positions_still_match_at_one_path() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("a".into(), vec![0, 2]);
        positions.insert("b".into(), vec![1, 3]);
        positions.insert("c".into(), vec![2, 4]);
        // Two overlapping phrases: a→b→c exists at path (0,1,2)
        let phrase = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert!(text_positions_match_phrase(&positions, &phrase));
    }

    // ── text_positions_match_phrases ──────────────────────────────────────

    #[test]
    fn empty_phrase_list_trivially_matches() {
        let positions = BTreeMap::<String, Vec<u32>>::new();
        let phrases: Vec<Vec<String>> = vec![];
        assert!(text_positions_match_phrases(&positions, &phrases));
    }

    #[test]
    fn all_phrases_must_match() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("hello".into(), vec![0, 10]);
        positions.insert("world".into(), vec![1, 11]);
        positions.insert("foo".into(), vec![5, 15]);
        positions.insert("bar".into(), vec![6, 16]);
        let phrases = vec![
            vec!["hello".to_string(), "world".to_string()],
            vec!["foo".to_string(), "bar".to_string()],
        ];
        assert!(text_positions_match_phrases(&positions, &phrases));
    }

    #[test]
    fn one_failing_phrase_causes_entire_check_to_fail() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("hello".into(), vec![0]);
        positions.insert("world".into(), vec![1]);
        positions.insert("missing_from_doc".into(), vec![]);
        let phrases = vec![
            vec!["hello".to_string(), "world".to_string()],
            vec!["missing_from_doc".to_string()],
        ];
        assert!(!text_positions_match_phrases(&positions, &phrases));
    }

    #[test]
    fn phrase_list_with_mixed_results_returns_false() {
        let mut positions = BTreeMap::<String, Vec<u32>>::new();
        positions.insert("good".into(), vec![0]);
        positions.insert("day".into(), vec![1]);
        positions.insert("bad".into(), vec![]);
        let phrases = vec![
            vec!["good".to_string(), "day".to_string()],
            vec!["bad".to_string()],
        ];
        assert!(!text_positions_match_phrases(&positions, &phrases));
    }
}
