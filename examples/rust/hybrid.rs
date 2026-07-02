//! Hybrid search example: insert records with both text and vectors,
//! then search using combined BM25 lexical + HNSW vector similarity.
//! VantaDB fuses results via Reciprocal Rank Fusion (RRF).

use std::error::Error;
use vantadb::config::VantaConfig;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest, VantaValue};

fn main() -> Result<(), Box<dyn Error>> {
    let db = VantaEmbedded::open_with_config(VantaConfig {
        storage_path: "./examples_hybrid_data".into(),
        ..Default::default()
    })?;

    let docs = vec![
        (
            "hybrid-1",
            "Neural networks excel at pattern recognition in images",
        ),
        (
            "hybrid-2",
            "Transformer models revolutionized natural language processing",
        ),
        (
            "hybrid-3",
            "Vector databases store embeddings for similarity search",
        ),
        (
            "hybrid-4",
            "BM25 is a classic ranking function for full-text search",
        ),
        (
            "hybrid-5",
            "Reciprocal rank fusion combines multiple ranking signals",
        ),
    ];

    for (key, payload) in &docs {
        db.put(VantaMemoryInput {
            namespace: "hybrid_demo".into(),
            key: (*key).into(),
            payload: (*payload).into(),
            metadata: vec![("source".into(), VantaValue::String("hybrid_example".into()))]
                .into_iter()
                .collect(),
            vector: Some(vec![0.1, 0.2, 0.3, 0.4]),
            ttl_ms: None,
        })?;
    }

    println!("Inserted {} records", docs.len());

    // Hybrid search: both text_query and query_vector are set.
    let results = db.search(VantaMemorySearchRequest {
        namespace: "hybrid_demo".into(),
        query_vector: vec![0.1, 0.2, 0.3, 0.4],
        text_query: Some("vector search ranking".into()),
        top_k: 5,
        ..Default::default()
    })?;

    println!("\nHybrid search results (text + vector):");
    for hit in &results {
        println!(
            "  score={:.4}  key={}  payload={}",
            hit.score, hit.record.key, hit.record.payload
        );
    }

    // Text-only search: leave query_vector empty.
    let text_results = db.search(VantaMemorySearchRequest {
        namespace: "hybrid_demo".into(),
        query_vector: vec![],
        text_query: Some("full-text ranking".into()),
        top_k: 5,
        ..Default::default()
    })?;

    println!("\nText-only results:");
    for hit in &text_results {
        println!(
            "  score={:.4}  key={}  payload={}",
            hit.score, hit.record.key, hit.record.payload
        );
    }

    Ok(())
}
