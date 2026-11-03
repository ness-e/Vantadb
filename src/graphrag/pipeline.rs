use crate::error::Result;
pub use crate::graphrag::expand::DEFAULT_MAX_DEGREE_PER_NODE;
use crate::graphrag::{context, expand, retrieve, seed};
use crate::sdk::Embedded;
use serde::Serialize;

pub struct GraphRagPipeline {
    pub seed_k: usize,
    pub expansion_hops: usize,
    pub max_expansion_nodes: usize,
    pub retrieval_top_k: usize,
    /// EGO-15 / MEM0-03: max edges followed out of each node during expansion
    /// (default: `DEFAULT_MAX_DEGREE_PER_NODE` = 128). Caps hub fan-out (`usize::MAX` = uncapped).
    pub max_degree_per_node: usize,
    /// EGO-15 (follow-up): skip edges below this weight during expansion
    /// (`None` = keep all edges, the pre-EGO-15 behavior).
    pub min_edge_weight: Option<f32>,
}

/// DIST-15: canonical wire shape for the bindings (Py/TS/Node/WASM).
/// `id`/`source`/`target` serialize as decimal strings (API-01) so u128 ids
/// above 2^53 survive JSON; the field set is pinned by
/// `tests/graphrag_test.rs::graphrag_result_serializes_with_u128_ids_as_decimal_strings`.
#[derive(Serialize)]
pub struct GraphRagNode {
    #[serde(with = "crate::sdk::types::u128_serde")]
    pub id: u128,
    pub content: String,
    pub score: f32,
    pub hop_distance: u32,
}

#[derive(Serialize)]
pub struct GraphRagEdge {
    #[serde(with = "crate::sdk::types::u128_serde")]
    pub source: u128,
    #[serde(with = "crate::sdk::types::u128_serde")]
    pub target: u128,
    pub label: String,
}

#[derive(Serialize)]
pub struct GraphRagResult {
    pub nodes: Vec<GraphRagNode>,
    pub edges: Vec<GraphRagEdge>,
    pub context_text: String,
    pub stats: GraphRagStats,
}

#[derive(Serialize)]
pub struct GraphRagStats {
    pub seeds_found: usize,
    pub nodes_expanded: usize,
    pub total_candidates: usize,
    pub expansion_hops_used: usize,
}

impl Default for GraphRagPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphRagPipeline {
    pub fn new() -> Self {
        Self {
            seed_k: 10,
            expansion_hops: 2,
            max_expansion_nodes: 100,
            retrieval_top_k: 20,
            max_degree_per_node: DEFAULT_MAX_DEGREE_PER_NODE,
            min_edge_weight: None,
        }
    }

    pub fn with_seed_k(mut self, seed_k: usize) -> Self {
        self.seed_k = seed_k;
        self
    }

    pub fn with_expansion_hops(mut self, expansion_hops: usize) -> Self {
        self.expansion_hops = expansion_hops;
        self
    }

    pub fn with_max_expansion_nodes(mut self, max_expansion_nodes: usize) -> Self {
        self.max_expansion_nodes = max_expansion_nodes;
        self
    }

    pub fn with_retrieval_top_k(mut self, retrieval_top_k: usize) -> Self {
        self.retrieval_top_k = retrieval_top_k;
        self
    }

    pub fn with_max_degree_per_node(mut self, max_degree_per_node: usize) -> Self {
        self.max_degree_per_node = max_degree_per_node;
        self
    }

    pub fn with_min_edge_weight(mut self, min_edge_weight: Option<f32>) -> Self {
        self.min_edge_weight = min_edge_weight;
        self
    }

    pub fn search(
        &self,
        embedded: &Embedded,
        namespace: &str,
        query: Option<&str>,
        query_vector: Option<&[f32]>,
    ) -> Result<GraphRagResult> {
        let seed_scores = seed::find_seeds(embedded, namespace, query, query_vector, self.seed_k)?;

        let engine = embedded.engine_handle()?;

        if seed_scores.is_empty() {
            return Ok(GraphRagResult {
                nodes: Vec::new(),
                edges: Vec::new(),
                context_text: String::new(),
                stats: GraphRagStats {
                    seeds_found: 0,
                    nodes_expanded: 0,
                    total_candidates: 0,
                    expansion_hops_used: 0,
                },
            });
        }

        let seed_ids: Vec<u128> = seed_scores.iter().map(|(id, _)| *id).collect();
        let expanded = expand::expand_nodes_capped(
            &engine,
            &seed_ids,
            self.expansion_hops,
            self.max_expansion_nodes,
            self.max_degree_per_node,
            self.min_edge_weight,
        )?;

        let top_nodes =
            retrieve::retrieve_and_rank(&engine, &seed_scores, &expanded, self.retrieval_top_k)?;

        let top_ids: std::collections::HashSet<u128> = top_nodes.iter().map(|n| n.id).collect();

        let expanded_set: std::collections::HashSet<u128> =
            expanded.visited.iter().copied().collect();

        let edges = retrieve::collect_edges(&engine, &top_ids, &expanded_set)?;

        let context_text = context::generate_context(&top_nodes, &edges);

        Ok(GraphRagResult {
            nodes: top_nodes,
            edges,
            context_text,
            stats: GraphRagStats {
                seeds_found: seed_scores.len(),
                nodes_expanded: expanded.visited.len(),
                total_candidates: expanded.visited.len() + seed_scores.len(),
                expansion_hops_used: expanded.max_hops_reached,
            },
        })
    }
}
