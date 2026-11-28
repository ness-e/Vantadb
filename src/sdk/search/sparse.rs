use super::super::builder::Embedded;
use super::super::serialization::impl_sparse_index::{
    decode_sparse_posting, sparse_posting_prefix,
};
use super::super::serialization::{
    matches_memory_filters, record_from_node, SPARSE_INDEX_SCHEMA_VERSION,
};
use super::super::types::*;
use crate::backend::BackendPartition;
use crate::error::{Error, Result};
use crate::node::UnifiedNode;
use crate::storage::StorageEngine;
use std::collections::BTreeMap;

/// Check that the persistent sparse index matches the key layout this build
/// queries, or fail loudly.
///
/// `FIND-830`, ronda P2-01 (`R1`): a **read-only** open never runs
/// `ensure_indexes_current` (`src/sdk/builder.rs:244`), and `read_only` is a
/// public option (`Config::with_read_only`). A base written by the previous
/// sparse-key layout keeps its `v1` postings while the lookup scans `v2`, so an
/// unguarded search answers an **empty hit list** — indistinguishable from "no
/// matches", which is the silent failure `FIND-513` already removed from the
/// text index (`ensure_text_index_query_ready`, `src/sdk/search/text_index.rs`).
///
/// Option chosen over a brute-force fallback (`records_for_namespace`): the
/// rebuild is one writable open away, falling back would add a second scoring
/// path that no test exercises and that would hide a misconfigured deployment;
/// the text index already answers the same situation with an error, and one
/// behaviour for both indexes is what an operator can reason about.
pub(crate) fn ensure_sparse_index_query_ready(engine: &StorageEngine) -> Result<()> {
    let state = Embedded::load_sparse_index_state(engine).map_err(|_| Error::NotFound {
        kind: "sparse_index_state".into(),
        id: "sparse".into(),
    })?;
    let Some(state) = state else {
        return Err(Error::NotFound {
            kind: "sparse_index".into(),
            id: "sparse".into(),
        });
    };
    if state.schema_version != SPARSE_INDEX_SCHEMA_VERSION {
        return Err(Error::Validation {
            field: "sparse_index_schema".into(),
            reason: format!(
                "sparse search requires sparse_index schema v{SPARSE_INDEX_SCHEMA_VERSION}; \
                 this base carries v{} (a different key layout) — reopen it writable to rebuild \
                 the index",
                state.schema_version
            ),
        });
    }
    Ok(())
}

impl Embedded {
    /// Sparse-dot search over the derived sparse-vector inverted index.
    ///
    /// The `SparseIndex` partition stores one posting per (namespace, dim,
    /// record). A query only walks the posting lists for the dims it contains,
    /// so cost is O(sum of matching posting-list lengths) instead of the old
    /// O(records in namespace) brute-force scan (`NUEVO-22`).
    pub(super) fn sparse_memory_search(
        &self,
        namespace: &str,
        query_sparse: &crate::node::SparseVector,
        filters: &MemoryMetadata,
        top_k: usize,
    ) -> Result<Vec<MemorySearchHit>> {
        if query_sparse.is_empty() || top_k == 0 {
            return Ok(Vec::new());
        }
        let engine = self.engine_handle()?;
        ensure_sparse_index_query_ready(&engine)?;

        let mut scores: BTreeMap<u128, f32> = BTreeMap::new();
        for (dim, query_weight) in query_sparse.0.iter() {
            let prefix = sparse_posting_prefix(namespace, *dim);
            for entry in
                engine.scan_partition_prefix_iter(BackendPartition::SparseIndex, &prefix)?
            {
                let (_posting_key, posting_value) = entry?;
                let posting = decode_sparse_posting(&posting_value)?;
                scores
                    .entry(posting.node_id)
                    .and_modify(|score| *score += query_weight * posting.weight)
                    .or_insert(query_weight * posting.weight);
            }
        }

        let mut hits = Vec::with_capacity(top_k);
        let node_ids: Vec<u128> = scores.keys().copied().collect();
        if !node_ids.is_empty() {
            let mut node_map: std::collections::HashMap<u128, UnifiedNode> =
                std::collections::HashMap::with_capacity(node_ids.len());
            for n in engine.get_many(&node_ids)? {
                node_map.insert(n.id, n);
            }
            for (node_id, score) in scores {
                if let Some(node) = node_map.get(&node_id) {
                    if let Some(record) = record_from_node(node) {
                        if record.namespace == namespace && matches_memory_filters(&record, filters)
                        {
                            hits.push(MemorySearchHit {
                                record,
                                score,
                                explanation: None,
                            });
                        }
                    }
                }
            }
        }

        super::fusion::sort_hits(&mut hits);
        hits.truncate(top_k);
        Ok(hits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sdk::Embedded;

    /// `FIND-830`, ronda P2-01 (`R1`): un open **read-only** se salta
    /// `ensure_indexes_current` (`src/sdk/builder.rs:244`), así que una base
    /// escrita por el layout anterior de claves sparse (`v1`) conserva sus
    /// postings viejos mientras la búsqueda escanea `v2`. Antes del fix eso
    /// devolvía una lista de hits **vacía**: un "no hay coincidencias"
    /// indistinguible de la verdad para toda consulta sparse.
    ///
    /// Tiene que fallar en voz alta, como ya hace el índice de texto con
    /// `ensure_text_index_query_ready` (`tests/text_index_recovery.rs:741-753`
    /// pina lo mismo para texto).
    #[test]
    fn sparse_search_on_a_read_only_open_of_an_old_index_fails_loudly() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("db").to_string_lossy().to_string();
        let open = |read_only: bool| {
            Embedded::open_with_config(crate::config::Config {
                storage_path: path.clone(),
                read_only,
                ..Default::default()
            })
            .expect("open db")
        };
        let search = |db: &Embedded| {
            db.search(MemorySearchRequest {
                namespace: "ns1".into(),
                query_sparse: Some(crate::node::SparseVector(BTreeMap::from([(3u32, 1.0f32)]))),
                top_k: 10,
                ..Default::default()
            })
        };

        let db = open(false);
        let mut input = crate::sdk::MemoryInput::new("ns1", "invoice-0001", "hello world");
        input.sparse_vector = Some(crate::node::SparseVector(BTreeMap::from([(3u32, 1.0f32)])));
        db.put(input).expect("put");
        assert_eq!(
            search(&db).expect("writable search answers").len(),
            1,
            "the index must answer before the downgrade, or the test proves nothing"
        );

        // The layout the previous version wrote: its keys are `v1\0…` and its
        // state says v1. Nothing rebuilds it on a read-only open, so the `v2`
        // prefix the lookup scans matches no key at all.
        let engine = db.engine_handle().unwrap();
        let node_id = db
            .get("ns1", "invoice-0001")
            .expect("get")
            .expect("record")
            .node_id;
        let mut ops = Vec::new();
        for (key, _) in engine
            .scan_partition(crate::backend::BackendPartition::SparseIndex)
            .unwrap()
        {
            ops.push(crate::backend::BackendWriteOp::Delete {
                partition: crate::backend::BackendPartition::SparseIndex,
                key,
            });
        }
        ops.push(crate::backend::BackendWriteOp::Put {
            partition: crate::backend::BackendPartition::SparseIndex,
            key: b"v1\0ns1\0\x03\0\0\0\0invoice-0001".to_vec(),
            value: crate::sdk::serialization::impl_sparse_index::sparse_posting_value(node_id, 1.0)
                .expect("posting value"),
        });
        engine.write_backend_batch(ops).expect("rewrite to v1");
        let mut state = Embedded::load_sparse_index_state(&engine)
            .unwrap()
            .expect("sparse state");
        state.schema_version = crate::sdk::serialization::SPARSE_INDEX_SCHEMA_VERSION - 1;
        Embedded::write_sparse_index_state(&engine, &state).unwrap();
        drop(engine);
        drop(db);

        let db = open(true);
        let error = search(&db).expect_err("a stale sparse index must not answer an empty list");
        let message = error.to_string();
        assert!(
            message.contains("sparse_index schema"),
            "the error must name the index and its version: {message}"
        );
        assert!(
            message.contains("reopen it writable"),
            "the error must say how to fix it: {message}"
        );
    }
}
