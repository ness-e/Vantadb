//! Physical vector operators: HNSW search and brute-force cosine refine.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;
use crate::storage::StorageEngine;
use std::sync::Arc;

/// Embeds a query text into a vector (FIND-413).
///
/// Production wiring resolves the env-selected provider; the hook exists so
/// the failure path is testable without a live provider (no ONNX, no server).
///
/// The seam mirrors the production contract exactly (see
/// [`default_query_vector`]), so a test can reproduce any of the three cases:
///
/// * `Ok(Some(vec))` — the query was embedded.
/// * `Ok(None)` — this build has no embedding capability compiled in (neither
///   `remote-inference` nor `embed-local`): the vector arm stays empty. That is
///   a build-time fact, not a provider failure.
/// * `Err(_)` — the provider failed. The error MUST reach the user: a failed
///   embedding is NOT "0 results".
///
/// `Arc` (not `Box`): the planner hands the same embedder to several operators
/// (join sides, subqueries) without cloning the closure itself.
pub(crate) type QueryEmbedder = Arc<dyn Fn(&str) -> Result<Option<Vec<f32>>> + Send + Sync>;

/// Embed the query with the env-selected provider (FIND-413).
///
/// * `Ok(Some(vec))` — the query was embedded.
/// * `Ok(None)` — this build has no embedding capability compiled in (neither
///   `remote-inference` nor `embed-local`): the vector arm stays empty. That is
///   a build-time fact, not a provider failure.
/// * `Err(_)` — the provider failed. The error MUST reach the user: a failed
///   embedding is NOT "0 results".
#[cfg(any(feature = "remote-inference", feature = "embed-local"))]
fn default_query_vector(text: &str) -> Result<Option<Vec<f32>>> {
    Ok(Some(crate::llm::embed_query(text)?))
}

/// One-shot notice for the build without embedding capability (FIND-413).
///
/// A vector plan opens once or twice per query and the fact does not change
/// between queries: warning on every `open()` is noise, not information.
#[cfg(not(any(feature = "remote-inference", feature = "embed-local")))]
static NO_EMBEDDING_CAPABILITY_WARNED: std::sync::Once = std::sync::Once::new();

/// Embed the query with the env-selected provider — this build has no
/// embedding capability compiled in (see the feature-gated variant).
#[cfg(not(any(feature = "remote-inference", feature = "embed-local")))]
fn default_query_vector(_text: &str) -> Result<Option<Vec<f32>>> {
    // Not silent: the vector arm is empty because the build cannot embed, and
    // the user must not read that as "the query matched nothing" (FIND-413).
    NO_EMBEDDING_CAPABILITY_WARNED.call_once(|| {
        tracing::warn!(
            "vector search without an embedding provider: this build has neither the \
             `remote-inference` nor the `embed-local` feature — the vector arm is empty \
             (rebuild with one of them to run semantic search)"
        );
    });
    Ok(None)
}

// ─── Physical Vector Search Operator ─────────────────────────

/// Does `node` belong to the entity scope of `entity`?
///
/// Mirrors the scope of [`PhysicalScan`](crate::physical_plan::PhysicalScan)
/// (`scan.rs:73-92`): the entity names a TYPE (`FROM Doc`), a namespace
/// table (`FROM my_ns`, MCP-29) or a node id (`FROM Doc#42`).
///
/// FIND-553: a node with **no** `type` field is not "of another type" — the
/// scope treats it as "any type" ("sin tipo" = "cualquier tipo"). Without
/// that, a store whose records carry no `type` would silently return zero
/// rows from the vector path while the scan path returned them. A node that
/// carries a namespace is still scoped by it (namespace isolation).
pub(crate) fn node_in_entity_scope(node: &UnifiedNode, entity: &str) -> bool {
    if entity == "*" || entity.contains('#') {
        return true;
    }
    match node.relational.get("type") {
        Some(crate::node::FieldValue::String(t)) => t == entity,
        // A `type` of another shape is not a type match.
        Some(_) => false,
        None => match node
            .relational
            .get(crate::sdk::serialization::FIELD_NAMESPACE)
        {
            // MCP-29: a namespace table — the namespace decides (as in the scan).
            Some(crate::node::FieldValue::String(ns)) => {
                crate::sdk::serialization::iql_table_name_for_namespace(ns) == entity
            }
            Some(_) => false,
            // FIND-553 invariant: no `type` and no namespace → the node is not
            // of another type → it belongs to any type scope.
            None => true,
        },
    }
}

/// Physical vector search operator using HNSW index.
pub struct PhysicalVectorSearch<'a> {
    /// Storage engine reference.
    storage: &'a StorageEngine,
    /// Text query to embed.
    query_vec_text: String,
    /// Minimum similarity score threshold.
    min_score: f32,
    /// Entity scope of the search (FIND-553).
    ///
    /// The HNSW search runs over the WHOLE index, so without a scope the
    /// operator returned the neighbours of the entire store. `None` = no
    /// scope (every indexed row).
    entity: Option<String>,
    /// Result node IDs from HNSW search.
    results: Vec<u128>,
    /// Pre-fetched nodes.
    prefetched: Vec<UnifiedNode>,
    /// Current position in the prefetched list.
    cursor: usize,
    /// Embedding hook (test seam): `None` → the env-selected provider.
    embedder: Option<QueryEmbedder>,
}

impl<'a> PhysicalVectorSearch<'a> {
    /// Create a new vector search operator.
    pub fn new(storage: &'a StorageEngine, query_text: String, min_score: f32) -> Self {
        Self {
            storage,
            query_vec_text: query_text,
            min_score,
            entity: None,
            results: Vec::new(),
            prefetched: Vec::new(),
            cursor: 0,
            embedder: None,
        }
    }

    /// Scope the search to an entity (FIND-553).
    ///
    /// `None` → no scope (every indexed row, the historical behaviour).
    /// `Some(entity)` → only the rows of that entity are returned; see
    /// [`node_in_entity_scope`].
    pub fn with_entity(mut self, entity: Option<String>) -> Self {
        self.entity = entity;
        self
    }

    /// Inject the query embedder (test seam, FIND-413).
    ///
    /// `None` (the default) → the env-selected provider. A test injects
    /// `Ok(None)` to reproduce "this build has no embedding capability"
    /// without a live provider.
    pub(crate) fn with_embedder(mut self, embedder: QueryEmbedder) -> Self {
        self.embedder = Some(embedder);
        self
    }
}

impl PhysicalOperator for PhysicalVectorSearch<'_> {
    fn open(&mut self) -> Result<()> {
        self.results.clear();
        self.prefetched.clear();
        self.cursor = 0;

        // FIND-413: a failed embedding must reach the user as an error — it is
        // NOT "0 results".
        let vector = match &self.embedder {
            Some(embed) => embed(&self.query_vec_text)?,
            None => default_query_vector(&self.query_vec_text)?,
        };

        if let Some(vec) = vector {
            let neighbors = {
                let index = self.storage.hnsw.load();
                let vs = self.storage.vector_store[0].read();
                index.search_nearest(&vec, None, None, &crate::node::ALL_BITSET, 5, Some(&*vs))
            };
            for (id, score) in neighbors {
                if score >= self.min_score {
                    self.results.push(id);
                }
            }
        }

        self.prefetched = self.storage.get_many(&self.results)?;

        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        while self.cursor < self.prefetched.len() {
            let node = &self.prefetched[self.cursor];
            self.cursor += 1;

            if self.storage.is_deleted(node.id)? {
                continue;
            }

            // FIND-553: the vector arm is scoped by the queried entity. The
            // scope is applied AFTER the top-k truncation, so the scoped row
            // set is always a SUBSET of the unscoped one (never bigger).
            //
            // ponytail: a filtered ANN search (top-k within the entity)
            // would need per-node label bitsets in the index — the bitsets
            // are never populated on insert (see sdk/search/vector.rs), so
            // the mask path rejects every candidate.
            if let Some(entity) = &self.entity {
                if !node_in_entity_scope(node, entity) {
                    continue;
                }
            }

            return Ok(Some(node.clone()));
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.results.clear();
        self.prefetched.clear();
        Ok(())
    }
}

// ─── Physical Vector Refine Operator (Brute Force Sim Check) ───

/// Physical vector refine operator that brute-force filters by cosine similarity.
pub struct PhysicalVectorRefine<'a> {
    /// Child operator.
    child: Box<dyn PhysicalOperator + 'a>,
    /// Text query to embed.
    query_vec_text: String,
    /// Minimum similarity score.
    min_score: f32,
    /// Embedded query vector.
    query_vector: Option<crate::node::VectorRepresentations>,
    /// Embedding hook (test seam) — see [`PhysicalVectorSearch::embedder`].
    embedder: Option<QueryEmbedder>,
}

impl<'a> PhysicalVectorRefine<'a> {
    /// Create a new vector refine operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, query_text: String, min_score: f32) -> Self {
        Self {
            child,
            query_vec_text: query_text,
            min_score,
            query_vector: None,
            embedder: None,
        }
    }

    /// Inject the query embedder (test seam, FIND-413) — see
    /// [`PhysicalVectorSearch::with_embedder`].
    pub(crate) fn with_embedder(mut self, embedder: QueryEmbedder) -> Self {
        self.embedder = Some(embedder);
        self
    }
}

impl PhysicalOperator for PhysicalVectorRefine<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.query_vector = None;

        // FIND-413: same contract as the search operator — the embedding error
        // is propagated, never degraded into "no rows".
        let vector = match &self.embedder {
            Some(embed) => embed(&self.query_vec_text)?,
            None => default_query_vector(&self.query_vec_text)?,
        };
        if let Some(vec) = vector {
            self.query_vector = Some(crate::node::VectorRepresentations::Full(vec));
        }
        Ok(())
    }

    fn next(&mut self) -> Result<Option<UnifiedNode>> {
        let q_vec = match &self.query_vector {
            Some(v) => v,
            None => return self.child.next(),
        };

        while let Some(node) = self.child.next()? {
            if let Some(sim) = node.vector.cosine_similarity(q_vec) {
                if sim >= self.min_score {
                    return Ok(Some(node));
                }
            }
        }
        Ok(None)
    }

    fn close(&mut self) -> Result<()> {
        self.query_vector = None;
        self.child.close()
    }
}

// ─── Tests (FIND-413) ────────────────────────────────────────────────

#[cfg(test)]
#[allow(missing_docs, clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::error::Error;
    use crate::node::{NodeFlags, VectorRepresentations};
    use crate::storage::BackendKind;
    use tempfile::tempdir;

    /// Embedding hook that always fails — stands in for a local model that is
    /// not loaded or an inference server that is down (FIND-413).
    fn broken_embedder() -> QueryEmbedder {
        Arc::new(|_text: &str| -> Result<Option<Vec<f32>>> {
            Err(Error::Embedding(
                "local model not loaded: /nonexistent/model.onnx".to_string(),
            ))
        })
    }

    /// Healthy embedding hook — deterministic vector, no network and no ONNX.
    fn healthy_embedder() -> QueryEmbedder {
        Arc::new(|_text: &str| -> Result<Option<Vec<f32>>> { Ok(Some(vec![1.0, 0.0])) })
    }

    /// Embedding hook for a build with **no embedding capability** (neither
    /// `remote-inference` nor `embed-local`): the vector arm stays empty.
    ///
    /// This is the third case of the contract, and the one the planner tests
    /// pin: it is a build-time fact, not a provider failure.
    fn no_capability_embedder() -> QueryEmbedder {
        Arc::new(|_text: &str| -> Result<Option<Vec<f32>>> { Ok(None) })
    }

    fn in_memory_storage() -> (StorageEngine, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let config = Config {
            backend_kind: BackendKind::InMemory,
            ..Default::default()
        };
        let storage = StorageEngine::open_with_config(dir.path().to_str().unwrap(), Some(config))
            .expect("open in-memory storage");
        (storage, dir)
    }

    // Prove-It (FIND-413): before the fix the operator swallowed the provider
    // error (`if let Ok(vec) = ...`) and the query returned "no results", so the
    // user diagnosed "I have no data" instead of "the provider is down".
    #[test]
    fn f413_vector_search_propagates_embedding_failure() {
        let (storage, _dir) = in_memory_storage();
        let mut op = PhysicalVectorSearch::new(&storage, "rust expert".to_string(), 0.5);
        op.embedder = Some(broken_embedder());

        let err = op
            .open()
            .expect_err("an embedding failure must be an error, not 0 hits");
        let msg = err.to_string();
        assert!(
            msg.contains("local model not loaded"),
            "the error must explain why the embedding failed: {msg}"
        );
    }

    // The good case stays distinguishable: a healthy provider with no match
    // is 0 hits, not an error.
    #[test]
    fn f413_vector_search_healthy_provider_without_match_is_zero_hits() {
        let (storage, _dir) = in_memory_storage();
        let mut op = PhysicalVectorSearch::new(&storage, "nothing matches".to_string(), 0.5);
        op.embedder = Some(healthy_embedder());

        op.open().expect("a healthy provider must not error");
        assert!(
            op.next().expect("next").is_none(),
            "no match → 0 hits, not an error"
        );
        op.close().expect("close");
    }

    // Guards the previous test against being vacuous: with a healthy provider
    // the vector arm really searches the index.
    #[test]
    fn f413_vector_search_healthy_provider_returns_matching_node() {
        let (storage, _dir) = in_memory_storage();
        let mut node = UnifiedNode::new(1);
        node.vector = VectorRepresentations::Full(vec![1.0, 0.0]);
        node.flags.set(NodeFlags::HAS_VECTOR);
        storage.insert(&node).unwrap();

        let mut op = PhysicalVectorSearch::new(&storage, "match".to_string(), -10.0);
        op.embedder = Some(healthy_embedder());
        op.open().expect("open");
        let hit = op
            .next()
            .expect("next")
            .expect("the indexed node must match");
        assert_eq!(hit.id, 1);
    }

    // The refine operator had the same swallow → same contract.
    #[test]
    fn f413_vector_refine_propagates_embedding_failure() {
        let (storage, _dir) = in_memory_storage();
        let child = crate::physical_plan::PhysicalScan::new(&storage, "*".to_string());
        let mut op = PhysicalVectorRefine::new(Box::new(child), "rust expert".to_string(), 0.5);
        op.embedder = Some(broken_embedder());

        let err = op
            .open()
            .expect_err("an embedding failure must be an error, not 0 hits");
        assert!(
            err.to_string().contains("local model not loaded"),
            "the error must explain why the embedding failed: {}",
            err
        );
    }

    // The third case of the contract (FIND-413): a build with no embedding
    // capability leaves the vector arm empty — that is a build-time fact, not
    // a provider failure, so it is neither an error nor a "0 hits" answer.
    #[test]
    fn f413_vector_search_without_embedding_capability_is_empty_arm() {
        let (storage, _dir) = in_memory_storage();
        let mut op = PhysicalVectorSearch::new(&storage, "rust expert".to_string(), 0.5);
        op.embedder = Some(no_capability_embedder());

        op.open()
            .expect("a build without embedding capability must not error");
        assert!(
            op.next().expect("next").is_none(),
            "the vector arm is empty: nothing to filter, no error"
        );
        op.close().expect("close");
    }
}
