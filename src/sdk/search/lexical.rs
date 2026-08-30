use super::super::builder::VantaEmbedded;
use super::super::serialization::{matches_memory_filters, memory_record_from_node};
use super::super::types::*;
use super::phrase;
use super::text_index;
use crate::backend::BackendPartition;
use crate::error::{ChainedError, Result, VantaError};
use crate::node::{FilterBitset, UnifiedNode};
use crate::query::RelOp;
use std::collections::BTreeMap;
use web_time::Instant;

impl VantaEmbedded {
    pub(super) fn lexical_search(
        &self,
        namespace: &str,
        query_text: &str,
        filters: &VantaMemoryMetadata,
        top_k: usize,
    ) -> Result<Vec<VantaMemorySearchHit>> {
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
        let mut doc_stats_cache: BTreeMap<u128, crate::text_index::TextDocStats> = BTreeMap::new();
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
                    VantaError::SearchError(ChainedError::msg(format!(
                        "text_query found an unreadable posting; run rebuild_index: {err}"
                    )))
                })?;
                let Some(record_key) = crate::text_index::posting_record_key(&prefix, &posting_key)
                else {
                    continue;
                };
                let doc_len = if let Some(stats) = doc_stats_cache.get(&posting.node_id) {
                    stats.doc_len
                } else {
                    let Some(stats) = Self::load_text_doc_stats(&engine, namespace, record_key)?
                    else {
                        return Err(VantaError::NotFound {
                            kind: "document_stats".into(),
                            id: "unknown".into(),
                        });
                    };
                    if stats.node_id != posting.node_id {
                        return Err(VantaError::SearchError(ChainedError::msg(
                            "text_query found posting/doc stats mismatch; run rebuild_index",
                        )));
                    }
                    let doc_len = stats.doc_len;
                    doc_stats_cache.insert(posting.node_id, stats);
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
                if let Some(record) = memory_record_from_node(node) {
                    if record.namespace == namespace && matches_memory_filters(&record, filters) {
                        hits.push(VantaMemorySearchHit {
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
            engine.cache_warmer.record_co_access(&node_ids);
        }

        crate::metrics::record_text_lexical_query(
            started.elapsed().as_millis() as u64,
            candidates_scored,
        );
        Ok(hits)
    }
    /// Build a `FilterBitset` from the node IDs of all records matching
    /// the given filters within a namespace.
    ///
    /// When shredded data is available for the filter fields this uses direct
    /// typed comparisons against the column store instead of loading full
    /// records — an order of magnitude faster for selective queries.
    pub(super) fn bitset_from_filters(
        &self,
        namespace: &str,
        filters: &VantaMemoryMetadata,
    ) -> Result<FilterBitset> {
        // ── Shredded fast path ────────────────────────────────
        // Try resolving every filter field from the shredded column store.
        // If all fields are present in shredded data we skip loading full
        // records entirely, reducing I/O for selective queries.
        if let Ok(engine) = self.engine_handle() {
            let (ids, _has_index) = self.indexed_ids_by_namespace(&engine, namespace, 0, None)?;
            let mut bitset = FilterBitset::with_capacity(ids.len());
            let mut all_resolved = true;

            for &node_id in &ids {
                let shredded = match crate::shred::ShreddedRowStore::get(node_id, &*engine.backend)
                {
                    Ok(Some(s)) => s,
                    _ => {
                        all_resolved = false;
                        break; // not all nodes have shredded data → fall back
                    }
                };

                let matched = filters.iter().all(|(field, expected)| {
                    shredded
                        .get(field)
                        .is_some_and(|s| crate::shred::matches_shredded(s, &RelOp::Eq, expected))
                });

                if matched {
                    bitset.set_bit(node_id as usize);
                }
            }

            if all_resolved {
                return Ok(bitset);
            }
        }

        // ── Fallback: load full records and use existing filtering ─
        let records = self.records_for_namespace(namespace, filters)?;
        let mut bitset = FilterBitset::with_capacity(records.len());
        for record in &records {
            bitset.set_bit(record.node_id as usize);
        }
        Ok(bitset)
    }
}
