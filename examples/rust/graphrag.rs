//! Knowledge graph example: insert nodes with content and vectors,
//! connect them with directed edges, then traverse using BFS.
//! Demonstrates the low-level Node/Graph API.

use std::error::Error;
use vantadb::config::VantaConfig;
use vantadb::{VantaEmbedded, VantaFields, VantaNodeInput};

fn main() -> Result<(), Box<dyn Error>> {
    let db = VantaEmbedded::open_with_config(VantaConfig {
        storage_path: "./examples_graph_data".into(),
        ..Default::default()
    })?;

    // ── Insert graph nodes ──
    let nodes = vec![
        (
            1,
            "VantaDB is an embedded vector database",
            vec![0.1, 0.2, 0.3],
        ),
        (
            2,
            "HNSW enables fast approximate nearest neighbor search",
            vec![0.2, 0.3, 0.4],
        ),
        (
            3,
            "BM25 provides full-text lexical retrieval",
            vec![0.3, 0.4, 0.5],
        ),
        (
            4,
            "Hybrid search fuses vector and text results via RRF",
            vec![0.4, 0.5, 0.6],
        ),
        (
            5,
            "WAL ensures crash-consistent durability",
            vec![0.5, 0.6, 0.7],
        ),
    ];

    for (id, content, vector) in &nodes {
        db.insert_node(VantaNodeInput {
            id: *id,
            content: Some((*content).into()),
            vector: Some(vector.clone()),
            fields: VantaFields::new(),
        })?;
    }

    // ── Add edges to build a knowledge graph ──
    // VantaDB -> HNSW -> Hybrid Search
    // VantaDB -> BM25  -> Hybrid Search
    // VantaDB -> WAL
    db.add_edge(1, 2, "uses", Some(1.0))?;
    db.add_edge(1, 3, "uses", Some(0.9))?;
    db.add_edge(1, 5, "uses", Some(0.8))?;
    db.add_edge(2, 4, "enables", Some(1.0))?;
    db.add_edge(3, 4, "enables", Some(1.0))?;

    println!("Inserted {} nodes with {} edges", nodes.len(), 5);

    // ── BFS traversal from VantaDB (node 1) ──
    let visited = db.graph_bfs(&[1], 3)?;
    println!("\nBFS from node 1 (max_depth=3):");
    for id in &visited {
        if let Some(record) = db.get_node(*id)? {
            let content = record
                .fields
                .get("content")
                .and_then(|v| {
                    if let vantadb::VantaValue::String(s) = v {
                        Some(s.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("(no content)");
            println!("  node {}: {}", id, content);
        }
    }

    Ok(())
}
