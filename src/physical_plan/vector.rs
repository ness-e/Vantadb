//! Physical vector operators: HNSW search and brute-force cosine refine.
//!
//! Split out of the monolithic `physical_plan` module (REVIEW-05).

use crate::error::Result;
use crate::node::UnifiedNode;
use crate::query::PhysicalOperator;
use crate::storage::StorageEngine;

// ─── Physical Vector Search Operator ─────────────────────────

/// Physical vector search operator using HNSW index.
pub struct PhysicalVectorSearch<'a> {
    /// Storage engine reference.
    storage: &'a StorageEngine,
    /// Text query to embed.
    #[allow(dead_code)]
    query_vec_text: String,
    /// Minimum similarity score threshold.
    min_score: f32,
    /// Result node IDs from HNSW search.
    results: Vec<u128>,
    /// Pre-fetched nodes.
    prefetched: Vec<UnifiedNode>,
    /// Current position in the prefetched list.
    cursor: usize,
}

impl<'a> PhysicalVectorSearch<'a> {
    /// Create a new vector search operator.
    pub fn new(storage: &'a StorageEngine, query_text: String, min_score: f32) -> Self {
        Self {
            storage,
            query_vec_text: query_text,
            min_score,
            results: Vec::new(),
            prefetched: Vec::new(),
            cursor: 0,
        }
    }
}

impl PhysicalOperator for PhysicalVectorSearch<'_> {
    fn open(&mut self) -> Result<()> {
        self.results.clear();
        self.prefetched.clear();
        self.cursor = 0;

        #[allow(unused_mut)]
        let mut vector: Option<Vec<f32>> = None;

        #[cfg(feature = "remote-inference")]
        {
            let provider = crate::llm::get_embedding_provider();
            if let Ok(vec) = provider.embed(&self.query_vec_text) {
                vector = Some(vec);
            }
        }

        if let Some(vec) = vector {
            let neighbors = {
                let index = self.storage.hnsw.load();
                let vs = self.storage.vector_store[0].read();
                index.search_nearest(&vec, None, None, &crate::node::ALL_BITSET, 5, Some(&vs))
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
    #[allow(dead_code)]
    query_vec_text: String,
    /// Minimum similarity score.
    min_score: f32,
    /// Embedded query vector.
    query_vector: Option<crate::node::VectorRepresentations>,
}

impl<'a> PhysicalVectorRefine<'a> {
    /// Create a new vector refine operator.
    pub fn new(child: Box<dyn PhysicalOperator + 'a>, query_text: String, min_score: f32) -> Self {
        Self {
            child,
            query_vec_text: query_text,
            min_score,
            query_vector: None,
        }
    }
}

impl PhysicalOperator for PhysicalVectorRefine<'_> {
    fn open(&mut self) -> Result<()> {
        self.child.open()?;
        self.query_vector = None;

        #[cfg(feature = "remote-inference")]
        {
            let provider = crate::llm::get_embedding_provider();
            if let Ok(vec) = provider.embed(&self.query_vec_text) {
                self.query_vector = Some(crate::node::VectorRepresentations::Full(vec));
            }
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
