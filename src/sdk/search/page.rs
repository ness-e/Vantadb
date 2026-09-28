//! Search pagination + post-ranking selectors (WIRE-08).
//!
//! Owns the cursor token (keyset anchor + plan fingerprint), the score-range
//! filter, group-by selection, and the MMR wiring. Every `Embedded::search*`
//! entry point funnels through [`run_search_page`] so the request-level
//! options behave identically across bindings.
//!
//! # Cursor contract
//!
//! The token is an opaque JSON string carrying:
//! - a **plan fingerprint** (namespace, query vector/text/sparse, filters,
//!   metric, explain, superseded flag, profile, range) — resuming with a
//!   different plan is rejected with a stable marker;
//! - the **rank offset** consumed so far (fetch-window budget);
//! - an **identity anchor** `(key, node_id)` — the last hit returned.
//!
//! Resume is **best-effort, not a snapshot**: a hit returned in a previous
//! page can be returned again (or skipped) when interleaved writes reorder its
//! rank across the anchor — BM25/IDF are recalculated corpus-wide on every
//! write, so the anchor's *position* can move. Guaranteed: writes that rank
//! **before** the anchor are never duplicated by the resumed page (the anchor
//! identity is skipped wherever it now sits), and deleting the anchor advances
//! from its former offset. A stronger cursor (result-set snapshot or
//! server-side session) is tracked as FIND-183 (owner: vanta-engine).
//!
//! Tokens are session-scoped (fingerprint uses a process-local hasher) and
//! must never be persisted or parsed by clients.

use super::super::builder::Embedded;
use super::super::serialization::vector_types::{
    GroupByConfig, MemorySearchHit, MemorySearchPage, MemorySearchRequest, RangeFilter,
};
use super::fusion::{EntityBoost, EntityBoostReport};
use super::mmr;
use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Stable `Display` marker for malformed / mismatched search options.
pub const SEARCH_OPTIONS_MARKER: &str = "SEARCH_OPTIONS_INVALID";

/// Stable `Display` marker for an unusable continuation cursor.
pub const CURSOR_INVALID_MARKER: &str = "SEARCH_CURSOR_INVALID";

/// Current cursor token version.
const CURSOR_VERSION: u8 = 1;

/// Upper bound for the page fetch window (deep pages + interleaved writes).
///
/// ponytail: a hard ceiling instead of an unbounded scan; pages beyond it
/// return what fits (clients see a short page = last page). Raise it or add a
/// keyset-native arm budget if deep pagination ever shows up in a profile.
pub const MAX_PAGE_WINDOW: usize = 10_000;

#[derive(Debug, Serialize, Deserialize)]
struct CursorToken {
    v: u8,
    fp: u64,
    offset: usize,
    key: String,
    /// `u128` as decimal string (same convention as the JSON boundary).
    node_id: String,
}

struct Anchor {
    key: String,
    node_id: u128,
}

/// Validate the WIRE-08 request options at the boundary (dim/NaN-style checks
/// live here, never in the inner loop).
pub(crate) fn validate_search_options(request: &MemorySearchRequest) -> Result<()> {
    if let Some(range) = &request.range {
        for (name, bound) in [
            ("min_score", range.min_score),
            ("max_score", range.max_score),
        ] {
            if let Some(value) = bound {
                if !value.is_finite() {
                    return Err(Error::InvalidInput(format!(
                        "{SEARCH_OPTIONS_MARKER}: range.{name} must be finite"
                    )));
                }
            }
        }
        if let (Some(min), Some(max)) = (range.min_score, range.max_score) {
            if min > max {
                return Err(Error::InvalidInput(format!(
                    "{SEARCH_OPTIONS_MARKER}: range.min_score ({min}) must be <= range.max_score ({max})"
                )));
            }
        }
    }
    if let Some(group_by) = &request.group_by {
        if group_by.field.trim().is_empty() {
            return Err(Error::InvalidInput(format!(
                "{SEARCH_OPTIONS_MARKER}: group_by.field must not be empty"
            )));
        }
        if group_by.group_size == 0 {
            return Err(Error::InvalidInput(format!(
                "{SEARCH_OPTIONS_MARKER}: group_by.group_size must be >= 1"
            )));
        }
    }
    if let Some(mmr) = &request.mmr {
        if !(0.0..=1.0).contains(&mmr.lambda) {
            return Err(Error::InvalidInput(format!(
                "{SEARCH_OPTIONS_MARKER}: mmr.lambda must be within [0, 1]"
            )));
        }
        if let Some(0) = mmr.fetch_k {
            return Err(Error::InvalidInput(format!(
                "{SEARCH_OPTIONS_MARKER}: mmr.fetch_k must be >= 1"
            )));
        }
    }
    if request.cursor.is_some() {
        // MMR is set-dependent and group_by counts reset per request — paging
        // them would silently duplicate/skip hits, so they are rejected.
        if request.mmr.is_some() {
            return Err(Error::InvalidInput(format!(
                "{CURSOR_INVALID_MARKER}: cursor pagination is not supported with mmr"
            )));
        }
        if request.group_by.is_some() {
            return Err(Error::InvalidInput(format!(
                "{CURSOR_INVALID_MARKER}: cursor pagination is not supported with group_by"
            )));
        }
    }
    Ok(())
}

/// Plan fingerprint: identifies the request a cursor belongs to. Excludes
/// `top_k` (page size may vary between pages) and `cursor` itself.
pub(crate) fn plan_fingerprint(request: &MemorySearchRequest) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    request.namespace.hash(&mut hasher);
    request.text_query.hash(&mut hasher);
    for value in &request.query_vector {
        value.to_bits().hash(&mut hasher);
    }
    if let Some(sparse) = &request.query_sparse {
        for (dim, value) in sparse.0.iter() {
            dim.hash(&mut hasher);
            value.to_bits().hash(&mut hasher);
        }
    }
    // BTreeMap iteration is key-sorted → stable.
    for (field, value) in &request.filters {
        field.hash(&mut hasher);
        format!("{value:?}").hash(&mut hasher);
    }
    format!("{:?}", request.distance_metric).hash(&mut hasher);
    request.explain.hash(&mut hasher);
    request.exclude_superseded.hash(&mut hasher);
    if let Some(profile) = &request.search_profile {
        format!("{profile:?}").hash(&mut hasher);
    }
    if let Some(range) = &request.range {
        range.min_score.map(f32::to_bits).hash(&mut hasher);
        range.max_score.map(f32::to_bits).hash(&mut hasher);
    }
    hasher.finish()
}

fn encode_cursor(request: &MemorySearchRequest, offset: usize, last: &MemorySearchHit) -> String {
    let token = CursorToken {
        v: CURSOR_VERSION,
        fp: plan_fingerprint(request),
        offset,
        key: last.record.key.clone(),
        node_id: last.record.node_id.to_string(),
    };
    // The token is opaque to clients; JSON keeps it dependency-free.
    serde_json::to_string(&token).unwrap_or_default()
}

fn decode_cursor(raw: &str) -> Result<CursorToken> {
    let token: CursorToken = serde_json::from_str(raw).map_err(|_| {
        Error::InvalidInput(format!(
            "{CURSOR_INVALID_MARKER}: cursor is not a valid continuation token"
        ))
    })?;
    if token.v != CURSOR_VERSION {
        return Err(Error::InvalidInput(format!(
            "{CURSOR_INVALID_MARKER}: unsupported cursor version {}",
            token.v
        )));
    }
    token.node_id.parse::<u128>().map_err(|_| {
        Error::InvalidInput(format!(
            "{CURSOR_INVALID_MARKER}: cursor carries an invalid node id"
        ))
    })?;
    Ok(token)
}

/// Index of the first hit strictly after the anchor **identity**
/// `(key, node_id)` in the current ranked list. Identity — not score — is the
/// anchor: BM25/IDF scores shift corpus-wide when a write lands, so a
/// score-anchored comparison would move the cursor and duplicate hits.
/// When the anchor record is gone (deleted between pages) the consumed
/// `fallback_offset` keeps the walk moving forward. Best-effort contract:
/// duplicates are also possible when interleaved writes reorder a
/// previously-returned hit across the anchor (see the module docs).
fn skip_past_anchor(hits: &[MemorySearchHit], anchor: &Anchor, fallback_offset: usize) -> usize {
    if let Some(index) = hits
        .iter()
        .position(|hit| hit.record.node_id == anchor.node_id && hit.record.key == anchor.key)
    {
        return index + 1;
    }
    fallback_offset.min(hits.len())
}

/// Keep hits whose score falls inside `[min_score, max_score]` (inclusive).
pub(crate) fn apply_range(hits: &mut Vec<MemorySearchHit>, range: &RangeFilter) {
    hits.retain(|hit| {
        range.min_score.is_none_or(|min| hit.score >= min)
            && range.max_score.is_none_or(|max| hit.score <= max)
    });
}

/// Canonical group key for a metadata value. Debug formatting is enough:
/// grouping is in-memory only and never persisted or compared cross-process.
fn group_key(value: Option<&crate::sdk::types::Value>) -> String {
    value.map(|value| format!("{value:?}")).unwrap_or_default()
}

/// Keep at most `group_size` hits per metadata value, in rank order, until
/// `top_k` hits were selected.
pub(crate) fn apply_group_by(
    hits: Vec<MemorySearchHit>,
    group_by: &GroupByConfig,
    top_k: usize,
) -> Vec<MemorySearchHit> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    let mut selected = Vec::new();
    for hit in hits {
        if selected.len() >= top_k {
            break;
        }
        let key = group_key(hit.record.metadata.get(&group_by.field));
        let count = counts.entry(key).or_insert(0);
        if *count < group_by.group_size {
            *count += 1;
            selected.push(hit);
        }
    }
    selected
}

/// Shared pipeline for every `search*` entry point.
pub(crate) fn run_search_page(
    db: &Embedded,
    request: MemorySearchRequest,
    method: Option<crate::index::IndexType>,
    boost: Option<&EntityBoost>,
) -> Result<(MemorySearchPage, EntityBoostReport)> {
    validate_search_options(&request)?;
    let top_k = request.top_k;
    if top_k == 0 {
        return Ok((
            MemorySearchPage {
                hits: Vec::new(),
                next_cursor: None,
            },
            EntityBoostReport::default(),
        ));
    }

    let cursor = request.cursor.as_deref().map(decode_cursor).transpose()?;
    if let Some(token) = &cursor {
        if token.fp != plan_fingerprint(&request) {
            return Err(Error::InvalidInput(format!(
                "{CURSOR_INVALID_MARKER}: cursor does not belong to this search request (plan fingerprint mismatch)"
            )));
        }
    }

    // Effective fetch window: deep pages need `offset + top_k` ranked hits;
    // MMR needs its candidate window. Everything downstream keeps using
    // `top_k` as the page size.
    let mut window = top_k;
    if let Some(mmr_cfg) = &request.mmr {
        window = window.max(mmr::resolve_fetch_k(top_k, mmr_cfg));
    }
    if let Some(token) = &cursor {
        window = window.max(token.offset.saturating_add(top_k));
    }

    // Fetch → post-process → page. The window grows (doubling, bounded by
    // MAX_PAGE_WINDOW) when a selector can shorten the page below `top_k`
    // while the ranked list is not exhausted:
    // - cursor resume: writes interleaved *before* the anchor shift the window;
    // - `range.max_score`: the filter eats the head of the ranking, so the
    //   first `top_k` fetched hits may all fall outside the band;
    // - `group_by`: saturated groups skip candidates.
    // The PRE-selector ranked length distinguishes "list exhausted" from
    // "selector consumed the page".
    let selectors_can_shorten = cursor.is_some()
        || request
            .range
            .as_ref()
            .is_some_and(|range| range.max_score.is_some())
        || request.group_by.is_some();
    let (mut hits, boost_report) = loop {
        let mut effective = request.clone();
        effective.top_k = window;
        effective.cursor = None;
        let (mut fetched, report) = db.search_impl(effective, method, boost)?;
        let ranked_len = fetched.len();

        if request.exclude_superseded {
            // ADR-028: drop superseded records at final assembly — no index change.
            fetched.retain(|hit| hit.record.superseded_by.is_none());
        }
        if let Some(range) = &request.range {
            apply_range(&mut fetched, range);
        }
        if let Some(mmr_cfg) = &request.mmr {
            fetched = mmr::rerank(fetched, mmr_cfg, top_k);
        }
        if let Some(group_by) = &request.group_by {
            fetched = apply_group_by(fetched, group_by, top_k);
        }
        if let Some(token) = &cursor {
            let anchor = Anchor {
                key: token.key.clone(),
                node_id: token.node_id.parse().unwrap_or(u128::MAX),
            };
            let start = skip_past_anchor(&fetched, &anchor, token.offset);
            fetched.drain(..start.min(fetched.len()));
        }

        let page_full = fetched.len() >= top_k;
        let exhausted = ranked_len < window;
        if page_full || exhausted || window >= MAX_PAGE_WINDOW || !selectors_can_shorten {
            break (fetched, report);
        }
        window = window.saturating_mul(2).min(MAX_PAGE_WINDOW);
    };

    // Emit a cursor only when the page is full (a short page is the last page —
    // `MemoryListPage` convention).
    let next_offset = cursor
        .as_ref()
        .map(|token| token.offset)
        .unwrap_or_default();
    let full = hits.len() >= top_k;
    hits.truncate(top_k);
    let next_cursor = if full && request.mmr.is_none() && request.group_by.is_none() {
        hits.last()
            .map(|last| encode_cursor(&request, next_offset.saturating_add(hits.len()), last))
    } else {
        None
    };

    Ok((MemorySearchPage { hits, next_cursor }, boost_report))
}
