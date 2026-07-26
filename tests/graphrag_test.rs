use tempfile::tempdir;
use vantadb::graphrag::pipeline::GraphRagPipeline;
use vantadb::{VantaEmbedded, VantaMemoryInput};

fn setup_test_db() -> (VantaEmbedded, tempfile::TempDir) {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");
    (db, dir)
}

fn insert_text_node(db: &VantaEmbedded, ns: &str, key: &str, content: &str) -> u128 {
    let input = VantaMemoryInput::new(ns, key, content);
    db.put(input).expect("put").node_id
}

fn insert_vector_node(
    db: &VantaEmbedded,
    ns: &str,
    key: &str,
    content: &str,
    vector: Vec<f32>,
) -> u128 {
    let mut input = VantaMemoryInput::new(ns, key, content);
    input.vector = Some(vector);
    db.put(input).expect("put").node_id
}

#[test]
fn test_simple_graphrag_search() {
    let (db, _dir) = setup_test_db();

    let node_data = vec![
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

    db.add_edge(ids[0], ids[1], "uses", Some(1.0)).unwrap();
    db.add_edge(ids[0], ids[2], "uses", Some(0.9)).unwrap();
    db.add_edge(ids[0], ids[4], "uses", Some(0.8)).unwrap();
    db.add_edge(ids[1], ids[3], "enables", Some(1.0)).unwrap();
    db.add_edge(ids[2], ids[3], "enables", Some(1.0)).unwrap();

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

    let chain = vec![
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
        db.add_edge(pair[0], pair[1], "connects", Some(1.0))
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
