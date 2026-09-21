//! Scene name normalizer (MEM-14, F4).
//!
//! Defensive engineering layer that runs *after* the L2 LLM emits scene names
//! and *before* any block is written (TDAM `filename-normalizer.ts` — port
//! WITHOUT the `.md` extension: the record store has no filesystem).
//!
//! Even though the prompt forbids spaces and punctuation, LLMs occasionally
//! produce names like `Daily Rhythm in Shanghai`. Such names break Markdown
//! navigation refs, shell tools, and URL/path encoding consumers. This module
//! canonicalizes them to the TDAM allowed set: ASCII alphanumerics, CJK
//! ideographs, hyphen, underscore, dot.
//!
//! This also resolves the MEM-12 key-collision debt: `sanitize_key` maps `/`
//! to `_`, so `a/b` and `a_b` used to collide as keys. The normalizer strips
//! `/` (TDAM drop list), so `a/b` → `ab` ≠ `a_b` — no collision.
//!
//! Source: `docs/research/tdam/02-scene-persona.md` + TDAM
//! `filename-normalizer.ts` (195).

/// Characters dropped from scene names (TDAM drop list: quotes, brackets,
/// punctuation known to break shells/markdown).
fn is_drop_char(ch: char) -> bool {
    matches!(
        ch,
        '(' | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '<'
            | '>'
            | '\''
            | '"'
            | '`'
            | ','
            | ';'
            | ':'
            | '!'
            | '?'
            | '*'
            | '|'
            | '/'
            | '\\'
            | '='
            | '&'
            | '%'
            | '$'
            | '#'
            | '@'
            | '^'
            | '~'
            | '+'
    )
}

fn is_whitespace(ch: char) -> bool {
    // Space, tab, NBSP, full-width space (TDAM `\s\u00A0\u3000`), plus CR/LF.
    matches!(ch, ' ' | '\t' | '\u{00A0}' | '\u{3000}' | '\r' | '\n')
}

fn is_separator(ch: char) -> bool {
    matches!(ch, '-' | '_' | '.')
}

/// Normalize a single scene name to its canonical form.
///
/// Rules (TDAM `normalizeSceneFilename` minus the `.md` extension and minus
/// the directory-component strip — the record store has no filesystem paths,
/// so a `/` is just a character in the TDAM drop list, not a path separator):
/// - Whitespace runs (spaces, tabs, NBSP, full-width space) → single hyphen.
/// - Strip quotes, brackets, and ASCII punctuation that breaks shell/markdown
///   (including `/` and `\`).
/// - Collapse consecutive separators (`-`, `_`, `.`).
/// - Trim leading/trailing separators.
/// - Fall back to `"scene"` if the result becomes empty.
/// - CJK ideographs are preserved (they are not in the drop list).
///
/// Examples:
/// - `"Daily Rhythm in Shanghai"` → `"Daily-Rhythm-in-Shanghai"`
/// - `"日常生活 健康管理"` → `"日常生活-健康管理"`
/// - `"Coffee (Yirgacheffe)"` → `"Coffee-Yirgacheffe"`
/// - `"  spaced  "` → `"spaced"`
/// - `""` → `"scene"`
/// - `"a/b"` → `"ab"` (no key collision with `"a_b"`)
pub fn normalize_scene_name(name: &str) -> String {
    if name.is_empty() {
        return "scene".to_string();
    }

    // Whitespace runs → '-' while dropping forbidden punctuation in one pass
    // (single trailing whitespace never appends a separator).
    let mut step = String::with_capacity(name.len());
    let mut ws_run = false;
    for ch in name.chars() {
        if is_whitespace(ch) {
            ws_run = true;
            continue;
        }
        if is_drop_char(ch) {
            continue;
        }
        if ws_run && !step.is_empty() {
            step.push('-');
        }
        ws_run = false;
        step.push(ch);
    }

    // Collapse consecutive separators.
    let mut out = String::with_capacity(step.len());
    let mut prev = None;
    for ch in step.chars() {
        if is_separator(ch) && prev == Some(ch) {
            continue;
        }
        out.push(ch);
        prev = Some(ch);
    }

    // Trim leading/trailing separators; fall back to "scene".
    let trimmed = out.trim_matches(|c| c == '-' || c == '_' || c == '.');
    if trimmed.is_empty() {
        "scene".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Return whether a scene name already matches its normalized form.
pub fn is_normalized_scene_name(name: &str) -> bool {
    normalize_scene_name(name) == name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_becomes_hyphen() {
        assert_eq!(
            normalize_scene_name("Daily Rhythm in Shanghai"),
            "Daily-Rhythm-in-Shanghai"
        );
        assert_eq!(
            normalize_scene_name("日常生活 健康管理"),
            "日常生活-健康管理"
        );
    }

    #[test]
    fn punctuation_and_slashes_dropped() {
        assert_eq!(
            normalize_scene_name("Coffee (Yirgacheffe)"),
            "Coffee-Yirgacheffe"
        );
        assert_eq!(
            normalize_scene_name("a/b"),
            "ab",
            "slash dropped, no key collision with a_b"
        );
        assert_eq!(normalize_scene_name("a_b"), "a_b");
        assert_ne!(normalize_scene_name("a/b"), normalize_scene_name("a_b"));
    }

    #[test]
    fn separators_collapse_and_trim() {
        assert_eq!(normalize_scene_name("a--b__c..d"), "a-b_c.d");
        assert_eq!(normalize_scene_name("--a--"), "a");
        assert_eq!(normalize_scene_name("  spaced  "), "spaced");
    }

    #[test]
    fn empty_and_separator_only_fall_back() {
        assert_eq!(normalize_scene_name(""), "scene");
        assert_eq!(normalize_scene_name("---"), "scene");
        assert_eq!(normalize_scene_name("   "), "scene");
    }

    #[test]
    fn slashes_are_dropped_not_paths() {
        // No filesystem in the record store: '/' is a drop character.
        assert_eq!(normalize_scene_name("dir/sub/My Scene"), "dirsubMy-Scene");
        assert_eq!(normalize_scene_name("dir\\sub\\My Scene"), "dirsubMy-Scene");
    }

    #[test]
    fn cjk_preserved() {
        assert_eq!(normalize_scene_name("已经规范"), "已经规范");
        assert!(is_normalized_scene_name("已经规范"));
    }

    #[test]
    fn identity_check() {
        assert!(is_normalized_scene_name("Daily-Rhythm-in-Shanghai"));
        assert!(!is_normalized_scene_name("Daily Rhythm in Shanghai"));
        assert!(!is_normalized_scene_name(""));
    }
}
