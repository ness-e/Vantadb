//! Debug helpers for explaining search results.
//!
//! These functions produce human-readable explanations for memory-search
//! results: rank maps, BM25 term breakdowns, matched phrases, snippets, and
//! structured explanation hits. They are pure(ish) functions that take an
//! engine reference for loading text-index state.

use super::phrase;
use super::snippet;
use crate::backend::BackendPartition;
use crate::error::Result;
use crate::sdk::builder::VantaEmbedded;
use crate::sdk::types::{
    VantaBm25TermContribution, VantaMemoryRecord, VantaMemorySearchHit, VantaSearchExplanationHit,
};
use crate::storage::StorageEngine;
use std::collections::BTreeMap;

/// Build a rank map from hit list position.
///
/// Returns a map of `(namespace, key)` → 1-based rank. Higher-ranked hits
/// get smaller numbers.
pub fn rank_map(hits: &[VantaMemorySearchHit]) -> BTreeMap<(String, String), usize> {
    hits.iter()
        .enumerate()
        .map(|(index, hit)| {
            (
                (hit.record.namespace.clone(), hit.record.key.clone()),
                index + 1,
            )
        })
        .collect()
}

/// Extract hit identities as `"namespace\0key"` strings.
///
/// Only compiled in `debug_assertions` builds since all callers are test-only.
#[cfg(debug_assertions)]
pub fn hit_identities(hits: &[VantaMemorySearchHit]) -> Vec<String> {
    hits.iter()
        .map(|hit| format!("{}\0{}", hit.record.namespace, hit.record.key))
        .collect()
}

/// Build a full [`VantaSearchExplanationHit`] for one search hit.
///
/// Includes BM25 term breakdown, matched phrases, snippet, and RRF rank
/// positions from both text and vector search arms.
pub fn explain_hit(
    engine: &StorageEngine,
    hit: VantaMemorySearchHit,
    text_query: Option<&str>,
    text_ranks: &BTreeMap<(String, String), usize>,
    vector_ranks: &BTreeMap<(String, String), usize>,
) -> Result<VantaSearchExplanationHit> {
    let identity_tuple = (hit.record.namespace.clone(), hit.record.key.clone());
    let identity = format!("{}\0{}", hit.record.namespace, hit.record.key);
    let bm25_terms = if let Some(text_query) = text_query {
        bm25_terms_for_record(engine, &hit.record, text_query)?
    } else {
        Vec::new()
    };
    let matched_tokens = bm25_terms
        .iter()
        .map(|term| term.token.clone())
        .collect::<Vec<_>>();
    let matched_phrases = if let Some(text_query) = text_query {
        matched_phrases_for_record(engine, &hit.record, text_query)?
    } else {
        Vec::new()
    };
    let snippet = text_query.and_then(|query| snippet::debug_snippet(&hit.record.payload, query));

    Ok(VantaSearchExplanationHit {
        identity,
        score: hit.score,
        snippet,
        matched_tokens,
        matched_phrases,
        bm25_terms,
        rrf_text_rank: text_ranks.get(&identity_tuple).copied(),
        rrf_vector_rank: vector_ranks.get(&identity_tuple).copied(),
    })
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Compute per-term BM25 contributions for a record given a text query.
fn bm25_terms_for_record(
    engine: &StorageEngine,
    record: &VantaMemoryRecord,
    text_query: &str,
) -> Result<Vec<VantaBm25TermContribution>> {
    let query_plan = crate::text_index::query_plan(text_query);
    if query_plan.terms.is_empty() {
        return Ok(Vec::new());
    }
    let Some(namespace_stats) =
        VantaEmbedded::load_text_namespace_stats(engine, &record.namespace)?
    else {
        return Ok(Vec::new());
    };
    let Some(doc_stats) =
        VantaEmbedded::load_text_doc_stats(engine, &record.namespace, &record.key)?
    else {
        return Ok(Vec::new());
    };
    if namespace_stats.doc_count == 0 {
        return Ok(Vec::new());
    }

    let doc_count = namespace_stats.doc_count as f32;
    let avg_doc_len = if namespace_stats.total_doc_len == 0 {
        1.0
    } else {
        namespace_stats.total_doc_len as f32 / doc_count
    };
    let doc_len = doc_stats.doc_len as f32;
    let mut terms = Vec::new();

    for token in query_plan.terms {
        let Some(term_stats) =
            VantaEmbedded::load_text_term_stats(engine, &record.namespace, &token)?
        else {
            continue;
        };
        let Some(posting_value) = engine.get_from_partition(
            BackendPartition::TextIndex,
            &crate::text_index::posting_key(&record.namespace, &token, &record.key),
        )?
        else {
            continue;
        };
        let posting = crate::text_index::decode_posting(&posting_value)?;
        let df = term_stats.df as f32;
        let idf = (1.0 + ((doc_count - df + 0.5) / (df + 0.5))).ln();
        let tf = posting.tf as f32;
        let denominator = tf
            + crate::text_index::BM25_K1
                * (1.0 - crate::text_index::BM25_B
                    + crate::text_index::BM25_B * (doc_len / avg_doc_len));
        let contribution = idf * ((tf * (crate::text_index::BM25_K1 + 1.0)) / denominator);
        terms.push(VantaBm25TermContribution {
            token,
            tf: posting.tf,
            df: term_stats.df,
            doc_len: doc_stats.doc_len,
            contribution,
        });
    }

    Ok(terms)
}

/// Find which phrases matched in a record for a given text query.
fn matched_phrases_for_record(
    engine: &StorageEngine,
    record: &VantaMemoryRecord,
    text_query: &str,
) -> Result<Vec<String>> {
    let query_plan = crate::text_index::query_plan(text_query);
    if query_plan.phrases.is_empty() {
        return Ok(Vec::new());
    }

    let mut term_positions = BTreeMap::new();
    for token in query_plan.terms {
        if let Some(value) = engine.get_from_partition(
            BackendPartition::TextIndex,
            &crate::text_index::posting_key(&record.namespace, &token, &record.key),
        )? {
            let posting = crate::text_index::decode_posting(&value)?;
            term_positions.insert(token, posting.positions);
        }
    }

    Ok(query_plan
        .phrases
        .into_iter()
        .filter(|phrase| phrase::text_positions_match_phrase(&term_positions, phrase))
        .map(|phrase| phrase.join(" "))
        .collect())
}
