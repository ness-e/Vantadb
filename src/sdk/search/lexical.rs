use super::super::builder::Embedded;
use super::super::serialization::{matches_memory_filters, record_from_node};
use super::super::types::*;
use super::phrase;
use super::text_index;
use crate::backend::BackendPartition;
use crate::error::{ChainedError, Error, Result};
use crate::node::{FilterBitset, UnifiedNode};
use std::collections::BTreeMap;
use std::collections::HashMap;
use web_time::Instant;

impl Embedded {
    pub(super) fn lexical_search(
        &self,
        namespace: &str,
        query_text: &str,
        filters: &MemoryMetadata,
        top_k: usize,
    ) -> Result<Vec<MemorySearchHit>> {
        let started = Instant::now();
        let engine = self.engine_handle()?;
        text_index::ensure_text_index_query_ready(&engine)?;

        if top_k == 0 {
            crate::metrics::record_text_lexical_query(0, 0);
            return Ok(Vec::new());
        }

        let query_plan = crate::text_index::query_plan(query_text);
        if query_plan.terms.is_empty() {
            crate::metrics::record_text_lexical_query(0, 0);
            return Ok(Vec::new());
        }

        let Some(namespace_stats) = Self::load_text_namespace_stats(&engine, namespace)? else {
            crate::metrics::record_text_lexical_query(started.elapsed().as_millis() as u64, 0);
            return Ok(Vec::new());
        };
        if namespace_stats.doc_count == 0 {
            crate::metrics::record_text_lexical_query(started.elapsed().as_millis() as u64, 0);
            return Ok(Vec::new());
        }

        let doc_count = namespace_stats.doc_count as f32;
        let avg_doc_len = if namespace_stats.total_doc_len == 0 {
            1.0
        } else {
            namespace_stats.total_doc_len as f32 / doc_count
        };
        let mut scores: BTreeMap<u128, f32> = BTreeMap::new();
        let mut candidate_positions: BTreeMap<u128, BTreeMap<&str, Vec<u32>>> = BTreeMap::new();
        let mut doc_stats_cache: HashMap<String, crate::text_index::TextDocStats> = HashMap::new();
        let mut candidates_scored = 0u64;

        for token in &query_plan.terms {
            let Some(term_stats) = Self::load_text_term_stats(&engine, namespace, token)? else {
                continue;
            };
            if term_stats.df == 0 {
                continue;
            }

            let df = term_stats.df as f32;
            let idf = (1.0 + ((doc_count - df + 0.5) / (df + 0.5))).ln();
            let prefix = crate::text_index::posting_prefix(namespace, token);
            for entry in engine.scan_partition_prefix_iter(BackendPartition::TextIndex, &prefix)? {
                let (posting_key, posting_value) = entry?;
                if crate::text_index::is_internal_key(&posting_key) {
                    continue;
                }
                let posting = crate::text_index::decode_posting(&posting_value).map_err(|err| {
                    Error::Search(ChainedError::msg(format!(
                        "text_query found an unreadable posting; run rebuild_index: {err}"
                    )))
                })?;
                let Some(record_key) = crate::text_index::posting_record_key(&prefix, &posting_key)
                else {
                    continue;
                };
                let doc_len = if let Some(stats) = doc_stats_cache.get(record_key) {
                    stats.doc_len
                } else {
                    let Some(stats) = Self::load_text_doc_stats(&engine, namespace, record_key)?
                    else {
                        return Err(Error::NotFound {
                            kind: "document_stats".into(),
                            id: "unknown".into(),
                        });
                    };
                    if stats.node_id != posting.node_id {
                        return Err(Error::Search(ChainedError::msg(
                            "text_query found posting/doc stats mismatch; run rebuild_index",
                        )));
                    }
                    let doc_len = stats.doc_len;
                    doc_stats_cache.insert(record_key.to_string(), stats);
                    doc_len
                };

                let tf = posting.tf as f32;
                let doc_len = doc_len as f32;
                let denominator = tf
                    + crate::text_index::BM25_K1
                        * (1.0 - crate::text_index::BM25_B
                            + crate::text_index::BM25_B * (doc_len / avg_doc_len));
                let contribution = idf * ((tf * (crate::text_index::BM25_K1 + 1.0)) / denominator);
                scores
                    .entry(posting.node_id)
                    .and_modify(|score| *score += contribution)
                    .or_insert(contribution);
                candidate_positions
                    .entry(posting.node_id)
                    .or_default()
                    .insert(token.as_str(), posting.positions);
                candidates_scored += 1;
            }
        }

        let mut hits = Vec::new();
        let node_ids: Vec<u128> = scores.keys().copied().collect();
        let mut node_map: std::collections::HashMap<u128, UnifiedNode> =
            std::collections::HashMap::with_capacity(node_ids.len());
        for n in engine.get_many(&node_ids)? {
            node_map.insert(n.id, n);
        }
        for (node_id, score) in scores {
            let positions_match = candidate_positions
                .get(&node_id)
                .map(|positions| {
                    phrase::text_positions_match_phrases(positions, &query_plan.phrases)
                })
                .unwrap_or(query_plan.phrases.is_empty());
            if !positions_match {
                continue;
            }
            if let Some(node) = node_map.get(&node_id) {
                if let Some(record) = record_from_node(node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        hits.push(MemorySearchHit {
                            record,
                            score,
                            explanation: None,
                        });
                    }
                }
            }
        }

        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(a.record.key.cmp(&b.record.key))
                .then(a.record.node_id.cmp(&b.record.node_id))
        });
        hits.truncate(top_k);

        // Feed search results into cache warmer co-access tracking
        if hits.len() >= 2 {
            let node_ids: Vec<u128> = hits.iter().map(|h| h.record.node_id).collect();
            engine.cache.warmer.record_co_access(&node_ids);
        }

        crate::metrics::record_text_lexical_query(
            started.elapsed().as_millis() as u64,
            candidates_scored,
        );
        Ok(hits)
    }
    /// Build a `FilterBitset` query mask from the given namespace and equality filters.
    ///
    /// Constructs a Bloom-like filter bitset in O(filters.len()) by projecting
    /// `("__ns__", namespace)` and each exact equality filter using [`metadata_hash_bit`].
    pub(super) fn bitset_from_filters(
        &self,
        namespace: &str,
        filters: &MemoryMetadata,
    ) -> Result<FilterBitset> {
        let mut bitset = FilterBitset::new();
        bitset.set_bit(super::super::serialization::metadata_hash_bit(
            "__ns__", namespace,
        ));
        for (field, value) in filters {
            let repr = super::super::serialization::value_canonical_repr(value);
            bitset.set_bit(super::super::serialization::metadata_hash_bit(field, &repr));
        }
        Ok(bitset)
    }
}
