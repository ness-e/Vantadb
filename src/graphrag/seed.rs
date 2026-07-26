use crate::error::Result;
use crate::node::DistanceMetric;
use crate::sdk::{VantaEmbedded, VantaMemorySearchRequest};

pub fn find_seeds(
    embedded: &VantaEmbedded,
    namespace: &str,
    query: Option<&str>,
    query_vector: Option<&[f32]>,
    k: usize,
) -> Result<Vec<(u128, f32)>> {
    if k == 0 {
        return Ok(Vec::new());
    }

    let request = VantaMemorySearchRequest {
        namespace: namespace.to_string(),
        query_vector: query_vector.map(|v| v.to_vec()).unwrap_or_default(),
        text_query: query.map(|q| q.to_string()),
        top_k: k,
        distance_metric: DistanceMetric::Cosine,
        ..Default::default()
    };

    let hits = embedded.search(request)?;
    let seeds: Vec<(u128, f32)> = hits
        .into_iter()
        .map(|hit| (hit.record.node_id, hit.score))
        .collect();

    Ok(seeds)
}
