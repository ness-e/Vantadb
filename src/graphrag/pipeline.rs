use crate::error::Result;
use crate::graphrag::{context, expand, retrieve, seed};
use crate::sdk::VantaEmbedded;

pub struct GraphRagPipeline {
    pub seed_k: usize,
    pub expansion_hops: usize,
    pub max_expansion_nodes: usize,
    pub retrieval_top_k: usize,
}

pub struct GraphRagNode {
    pub id: u128,
    pub content: String,
    pub score: f32,
    pub hop_distance: u32,
}

pub struct GraphRagEdge {
    pub source: u128,
    pub target: u128,
    pub label: String,
}

pub struct GraphRagResult {
    pub nodes: Vec<GraphRagNode>,
    pub edges: Vec<GraphRagEdge>,
    pub context_text: String,
    pub stats: GraphRagStats,
}

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
        }
    }

    pub fn search(
        &self,
        embedded: &VantaEmbedded,
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
        let expanded = expand::expand_nodes(
            &engine,
            &seed_ids,
            self.expansion_hops,
            self.max_expansion_nodes,
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
