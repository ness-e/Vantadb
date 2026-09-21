//! Shared text utilities: code-point-safe truncation and recent-unique
//! selection (port of TDAM `MC/utils/text-utils.ts` plus the truncation that
//! previously lived privately in `core/hooks/auto_recall.rs`).
//!
//! All slicing is by code points (`char`s), never bytes — UTF-8 safe by
//! construction (TDAM's surrogate-pair-safe slicing parity).

/// Take the first `max_chars` code points of `s`.
pub fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

/// Truncate to `max_chars` code points, appending `suffix` when truncation
/// happened and fits (trailing whitespace before the suffix is trimmed).
pub fn truncate_with_suffix(s: &str, max_chars: usize, suffix: &str) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    let suffix_len = suffix.chars().count();
    if max_chars <= suffix_len {
        return truncate_chars(s, max_chars);
    }
    let head = truncate_chars(s, max_chars - suffix_len);
    format!("{}{suffix}", head.trim_end())
}

/// Up to `max` most recent unique texts, original order preserved
/// (port of TDAM `sanitize.ts:pickRecentUnique`).
pub fn pick_recent_unique(texts: &[impl AsRef<str>], max: usize) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut picked: Vec<String> = Vec::new();
    for text in texts.iter().rev() {
        if picked.len() == max {
            break;
        }
        if seen.insert(text.as_ref()) {
            picked.push(text.as_ref().to_string());
        }
    }
    picked.reverse();
    picked
}

#[cfg(test)]
mod tests {
    use super::{pick_recent_unique, truncate_chars, truncate_with_suffix};

    #[test]
    fn truncate_chars_is_code_point_safe() {
        // Multi-byte chars are never split mid-sequence.
        let s = "héllo 世界 🎉🎉";
        let out = truncate_chars(s, 8);
        assert_eq!(out, "héllo 世界".chars().take(8).collect::<String>());
        assert_eq!(out.chars().count(), 8);
        assert!(s.starts_with(&out));
    }

    #[test]
    fn truncate_chars_shorter_than_max_is_identity() {
        assert_eq!(truncate_chars("abc", 10), "abc");
        assert_eq!(truncate_chars("", 3), "");
    }

    #[test]
    fn truncate_with_suffix_appends_suffix_when_truncated() {
        let out = truncate_with_suffix("hello world", 8, "...");
        assert_eq!(out, "hello...");
        assert_eq!(out.chars().count(), 8);
    }

    #[test]
    fn truncate_with_suffix_trims_whitespace_before_suffix() {
        assert_eq!(truncate_with_suffix("hello world  ", 9, "..."), "hello...");
    }

    #[test]
    fn truncate_with_suffix_identity_when_within_budget() {
        assert_eq!(truncate_with_suffix("short", 10, "..."), "short");
    }

    #[test]
    fn truncate_with_suffix_tiny_budget_skips_suffix() {
        assert_eq!(truncate_with_suffix("abcdef", 2, "..."), "ab");
    }

    #[test]
    fn truncate_suffix_is_utf8_safe() {
        // 9 chars, budget 5 → keep 4 + suffix.
        let out = truncate_with_suffix("日本語テキストです", 5, "…");
        assert_eq!(out, "日本語テ…");
        assert_eq!(out.chars().count(), 5);
    }

    #[test]
    fn pick_recent_unique_keeps_last_occurrence_order() {
        let texts = vec!["a", "b", "a", "c", "b"];
        assert_eq!(pick_recent_unique(&texts, 10), ["a", "c", "b"]);
    }

    #[test]
    fn pick_recent_unique_respects_max_from_the_end() {
        let texts = vec!["a", "b", "c", "d"];
        assert_eq!(pick_recent_unique(&texts, 2), ["c", "d"]);
        assert!(pick_recent_unique(&[] as &[&str], 3).is_empty());
    }
}
