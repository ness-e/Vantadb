//! Opaque cursor pagination helpers (REST-06 — API-03).
//!
//! The HTTP surface paginates with a single scheme: a `limit` plus an opaque
//! `cursor` token, and every page response carries `next_cursor` + `has_more`.
//! [`encode_cursor`]/[`decode_cursor`] keep the wire type a **string** and the
//! token opaque: clients pass a page's `next_cursor` back verbatim and must not
//! parse it. The current encoding is an internal position (decimal — the same
//! convention `next_cursor_to_js` uses in the WASM binding), so the encoding
//! can change server-side without a breaking wire change.
//!
//! The skills listing helper lives here (not in `handlers`) because it is the
//! one endpoint whose SDK options model the position internally: the handler
//! stays a humble object and never spells a positional field.

use crate::error::Result;
use crate::sdk::{SkillListOptions, SkillListPage};
use crate::skills::SkillStore;

/// Encode an internal position as the opaque wire cursor.
pub(crate) fn encode_cursor(position: usize) -> String {
    position.to_string()
}

/// Decode an opaque wire cursor into the internal position.
///
/// Invalid tokens are a client error (the handler maps this to a 400). The
/// message is static plus the raw token — it never echoes internal state.
pub(crate) fn decode_cursor(raw: &str) -> std::result::Result<usize, String> {
    raw.parse::<usize>()
        .map_err(|_| format!("invalid cursor {raw:?}: expected an opaque token from `next_cursor`"))
}

/// `has_more` + `next_cursor` after reading a positional page.
///
/// `returned` is the number of items the backend yielded *before* the handler
/// truncates to `limit` (callers fetch `limit + 1` to detect a next page);
/// `base` is the position the page started at.
pub(crate) fn page_meta(returned: usize, limit: usize, base: usize) -> (bool, Option<String>) {
    let has_more = returned > limit;
    let next_cursor = has_more.then(|| encode_cursor(base.saturating_add(limit)));
    (has_more, next_cursor)
}

/// One page of skill heads: bridges the opaque cursor (position) to the SDK
/// listing options, which model the position internally.
pub(crate) fn list_skills_page(
    store: &SkillStore<'_>,
    owner_agent: Option<String>,
    name_prefix: Option<String>,
    limit: usize,
    skip: usize,
) -> Result<SkillListPage> {
    store.list(SkillListOptions {
        owner_agent,
        name_prefix,
        limit,
        offset: skip,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_roundtrips_positions() {
        for position in [0usize, 1, 42, usize::MAX] {
            let token = encode_cursor(position);
            assert_eq!(decode_cursor(&token), Ok(position));
        }
    }

    #[test]
    fn cursor_rejects_non_tokens() {
        for raw in ["", "next", "-1", "1.5", " 3", "0x10"] {
            assert!(decode_cursor(raw).is_err(), "{raw:?} must be rejected");
        }
    }

    #[test]
    fn page_meta_signals_more_only_past_limit() {
        // A full page without an extra element is the last page.
        assert_eq!(page_meta(10, 10, 0), (false, None));
        // limit + 1 element means another page exists at base + limit.
        assert_eq!(page_meta(11, 10, 0), (true, Some(encode_cursor(10))));
        assert_eq!(page_meta(11, 10, 20), (true, Some(encode_cursor(30))));
    }
}
