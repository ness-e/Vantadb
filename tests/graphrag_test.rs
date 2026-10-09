// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use tempfile::tempdir;
use vantadb::graphrag::pipeline::GraphRagPipeline;
use vantadb::{Embedded, MemoryInput};

fn setup_test_db() -> (Embedded, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let db = Embedded::open(dir.path()).expect("open");
    (db, dir)
}

fn insert_text_node(db: &Embedded, ns: &str, key: &str, content: &str) -> u128 {
    let input = MemoryInput::new(ns, key, content);
    db.put(input).expect("put").node_id
}

fn insert_vector_node(db: &Embedded, ns: &str, key: &str, content: &str, vector: Vec<f32>) -> u128 {
    let mut input = MemoryInput::new(ns, key, content);
    input.vector = Some(vector);
    db.put(input).expect("put").node_id
}

#[test]
fn test_simple_graphrag_search() {
    let (db, _dir) = setup_test_db();

    let node_data = [
        (
            "VantaDB is an embedded vector database",
            vec![0.1_f32, 0.2, 0.3],
        ),
        (
            "HNSW enables fast approximate nearest neighbor search",
            vec![0.2, 0.3, 0.4],
        ),
        (
            "BM25 provides full-text lexical retrieval",
            vec![0.3, 0.4, 0.5],
        ),
        (
            "Hybrid search fuses vector and text results via RRF",
            vec![0.4, 0.5, 0.6],
        ),
        (
            "WAL ensures crash-consistent durability",
            vec![0.5, 0.6, 0.7],
        ),
    ];

    let mut ids = Vec::new();
    for (i, (content, vector)) in node_data.iter().enumerate() {
        ids.push(insert_vector_node(
            &db,
            "graphrag",
            &format!("n{i}"),
            content,
            vector.clone(),
        ));
    }

    db.add_edge(ids[0], ids[1], "uses", Some(1.0), None)
        .unwrap();
    db.add_edge(ids[0], ids[2], "uses", Some(0.9), None)
        .unwrap();
    db.add_edge(ids[0], ids[4], "uses", Some(0.8), None)
        .unwrap();
    db.add_edge(ids[1], ids[3], "enables", Some(1.0), None)
        .unwrap();
    db.add_edge(ids[2], ids[3], "enables", Some(1.0), None)
        .unwrap();

    let pipeline = GraphRagPipeline::new();
    let result = pipeline
        .search(&db, "graphrag", Some("vector database"), None)
        .expect("search");

    assert!(!result.nodes.is_empty(), "expected at least 1 node");
    assert!(
        !result.context_text.is_empty(),
        "context_text should be non-empty"
    );
    assert!(result.stats.seeds_found > 0, "expected seeds_found > 0");
}

#[test]
fn test_empty_result() {
    let (db, _dir) = setup_test_db();

    let pipeline = GraphRagPipeline::new();
    let result = pipeline
        .search(&db, "nonexistent", Some("anything"), None)
        .expect("search");

    assert!(result.nodes.is_empty(), "expected 0 nodes");
    assert!(result.edges.is_empty(), "expected 0 edges");
    assert!(result.context_text.is_empty(), "expected empty context");
    assert_eq!(result.stats.seeds_found, 0);
    assert_eq!(result.stats.nodes_expanded, 0);
    assert_eq!(result.stats.expansion_hops_used, 0);
}

#[test]
fn test_hybrid_fallback() {
    let (db, _dir) = setup_test_db();

    insert_text_node(&db, "hybrid", "b1", "vector database for AI agents");
    insert_text_node(&db, "hybrid", "b2", "HNSW index for fast similarity search");
    insert_text_node(
        &db,
        "hybrid",
        "b3",
        "BM25 full-text lexical retrieval engine",
    );

    let pipeline = GraphRagPipeline::new();
    let result = pipeline
        .search(&db, "hybrid", Some("vector search"), None)
        .expect("search");

    assert!(
        result.stats.seeds_found > 0,
        "expected BM25 fallback to find seeds, got {}",
        result.stats.seeds_found
    );
}

#[test]
fn test_max_expansion() {
    let (db, _dir) = setup_test_db();

    let chain = [
        ("root concept", vec![0.1_f32, 0.2, 0.3]),
        ("child A", vec![0.2, 0.3, 0.4]),
        ("child B", vec![0.3, 0.4, 0.5]),
        ("child C", vec![0.4, 0.5, 0.6]),
        ("grandchild", vec![0.5, 0.6, 0.7]),
        ("great grandchild", vec![0.6, 0.7, 0.8]),
    ];

    let mut ids = Vec::new();
    for (i, (content, vector)) in chain.iter().enumerate() {
        ids.push(insert_vector_node(
            &db,
            "expand",
            &format!("e{i}"),
            content,
            vector.clone(),
        ));
    }
    for pair in ids.windows(2) {
        db.add_edge(pair[0], pair[1], "connects", Some(1.0), None)
            .unwrap();
    }

    let pipeline = GraphRagPipeline {
        seed_k: 1,
        max_expansion_nodes: 1,
        ..GraphRagPipeline::new()
    };
    let result = pipeline
        .search(&db, "expand", Some("root concept"), None)
        .expect("search");

    assert!(result.stats.seeds_found > 0, "expected seeds");
    assert!(
        result.stats.nodes_expanded <= 1,
        "max_expansion_nodes=1 should cap expansion, got {}",
        result.stats.nodes_expanded
    );
}

/// DIST-15: pins the canonical wire shape consumed by every binding
/// (Py/TS/Node/WASM). u128 ids travel as decimal strings (API-01), the
/// payload is snake_case, and the stats block is complete — a rename or a
/// dropped field breaks every binding's smoke at once.
#[test]
fn graphrag_result_serializes_with_u128_ids_as_decimal_strings() {
    let (db, _dir) = setup_test_db();

    let a = insert_vector_node(
        &db,
        "wire",
        "a",
        "vector database for agents",
        vec![0.1, 0.2, 0.3],
    );
    let b = insert_vector_node(
        &db,
        "wire",
        "b",
        "graph expansion uses edges",
        vec![0.2, 0.3, 0.4],
    );
    db.add_edge(a, b, "uses", Some(1.0), None).expect("edge");

    let result = db
        .graphrag_search("wire", Some("vector database"), None)
        .expect("graphrag search");
    let json = serde_json::to_value(&result).expect("serialize GraphRagResult");

    let nodes = json["nodes"].as_array().expect("nodes must be an array");
    assert!(!nodes.is_empty(), "expected at least one node");
    for node in nodes {
        let id = node["id"].as_str().expect("id must be a decimal string");
        assert!(id.parse::<u128>().is_ok(), "id '{id}' must parse as u128");
        assert!(node["content"].is_string(), "content must be a string");
        assert!(node["score"].is_number(), "score must be a number");
        assert!(
            node["hop_distance"].is_number(),
            "hop_distance must be a number"
        );
    }

    let edges = json["edges"].as_array().expect("edges must be an array");
    assert!(!edges.is_empty(), "expected at least one edge");
    for edge in edges {
        assert!(
            edge["source"]
                .as_str()
                .is_some_and(|s| s.parse::<u128>().is_ok()),
            "source must be a decimal string"
        );
        assert!(
            edge["target"]
                .as_str()
                .is_some_and(|s| s.parse::<u128>().is_ok()),
            "target must be a decimal string"
        );
        assert!(edge["label"].is_string(), "label must be a string");
    }

    assert!(
        json["context_text"].as_str().is_some_and(|s| !s.is_empty()),
        "context_text must be a non-empty string"
    );

    let stats = &json["stats"];
    assert!(
        stats["seeds_found"].as_u64().is_some_and(|n| n > 0),
        "seeds_found must be > 0"
    );
    assert!(stats["nodes_expanded"].is_number());
    assert!(stats["total_candidates"].is_number());
    assert!(stats["expansion_hops_used"].is_number());
}

/// DIST-15: `context_text` must be byte-identical across runs and bindings.
/// The edge list is ordered by `(source, target, label)` — not by `HashSet`
/// iteration (seeded per instance, which made two runs over the same graph
/// emit different `context_text`).
#[test]
fn graphrag_edges_are_ordered_by_source_target_label() {
    let (db, _dir) = setup_test_db();

    let a = insert_vector_node(&db, "order", "a", "root concept", vec![0.1, 0.2, 0.3]);
    let b = insert_vector_node(&db, "order", "b", "child b", vec![0.2, 0.3, 0.4]);
    let c = insert_vector_node(&db, "order", "c", "child c", vec![0.3, 0.4, 0.5]);
    db.add_edge(a, b, "uses", Some(1.0), None).expect("edge");
    db.add_edge(a, c, "uses", Some(1.0), None).expect("edge");
    db.add_edge(b, c, "uses", Some(1.0), None).expect("edge");

    let result = db
        .graphrag_search("order", Some("root concept"), None)
        .expect("graphrag search");
    assert!(!result.edges.is_empty(), "expected edges");

    let emitted: Vec<(u128, u128, String)> = result
        .edges
        .iter()
        .map(|e| (e.source, e.target, e.label.clone()))
        .collect();
    let mut expected = emitted.clone();
    expected.sort();
    assert_eq!(
        emitted, expected,
        "edges must be ordered by (source, target, label)"
    );
}
