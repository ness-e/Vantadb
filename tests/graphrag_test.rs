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

/// EGO-15: `max_degree_per_node` caps hub fan-out during expansion.
/// A hub with 6 leaves, `expansion_hops: 1` and `max_degree_per_node: 2`
/// visits hub + 2 leaves (3 nodes); the default pipeline stays uncapped (7).
#[test]
fn test_expansion_max_degree_per_node_caps_hub() {
    let (db, _dir) = setup_test_db();

    let hub = insert_vector_node(
        &db,
        "degree",
        "hub",
        "hub central alpha node",
        vec![0.1, 0.2, 0.3],
    );
    for i in 0..6 {
        let leaf = insert_vector_node(
            &db,
            "degree",
            &format!("leaf{i}"),
            &format!("leaf number {i} beta"),
            vec![0.4 + i as f32 * 0.1, 0.5, 0.6],
        );
        db.add_edge(hub, leaf, "connects", Some(1.0), None)
            .expect("edge");
    }

    let capped = GraphRagPipeline {
        seed_k: 1,
        expansion_hops: 1,
        max_expansion_nodes: 100,
        max_degree_per_node: 2,
        ..GraphRagPipeline::new()
    };
    let result = capped
        .search(&db, "degree", Some("hub central alpha"), None)
        .expect("search");
    assert_eq!(result.stats.seeds_found, 1, "hub must be the single seed");
    assert_eq!(
        result.stats.nodes_expanded, 3,
        "hub + 2 leaves with max_degree_per_node=2, got {}",
        result.stats.nodes_expanded
    );

    let full = GraphRagPipeline {
        seed_k: 1,
        expansion_hops: 1,
        max_expansion_nodes: 100,
        ..GraphRagPipeline::new()
    };
    let result = full
        .search(&db, "degree", Some("hub central alpha"), None)
        .expect("search");
    assert_eq!(
        result.stats.nodes_expanded, 7,
        "default pipeline stays uncapped, got {}",
        result.stats.nodes_expanded
    );
}

/// EGO-15 (follow-up): `min_edge_weight` filters weak edges during
/// expansion. Hub with 6 leaves (3 strong @1.0, 3 weak @0.1),
/// `min_edge_weight: Some(0.5)` visits hub + 3 strong leaves (4 nodes);
/// `None` (default) visits all 7.
#[test]
fn test_expansion_min_edge_weight_filters_weak_edges() {
    let (db, _dir) = setup_test_db();

    let hub = insert_vector_node(
        &db,
        "weight",
        "hub",
        "hub central alpha node",
        vec![0.1, 0.2, 0.3],
    );
    for i in 0..6 {
        let leaf = insert_vector_node(
            &db,
            "weight",
            &format!("leaf{i}"),
            &format!("leaf number {i} beta"),
            vec![0.4 + i as f32 * 0.1, 0.5, 0.6],
        );
        let w = if i % 2 == 0 { 1.0 } else { 0.1 };
        db.add_edge(hub, leaf, "connects", Some(w), None)
            .expect("edge");
    }

    let filtered = GraphRagPipeline {
        seed_k: 1,
        expansion_hops: 1,
        max_expansion_nodes: 100,
        min_edge_weight: Some(0.5),
        ..GraphRagPipeline::new()
    };
    let result = filtered
        .search(&db, "weight", Some("hub central alpha"), None)
        .expect("search");
    assert_eq!(result.stats.seeds_found, 1, "hub must be the single seed");
    assert_eq!(
        result.stats.nodes_expanded, 4,
        "hub + 3 strong leaves with min_edge_weight=0.5, got {}",
        result.stats.nodes_expanded
    );

    let unfiltered = GraphRagPipeline {
        seed_k: 1,
        expansion_hops: 1,
        max_expansion_nodes: 100,
        ..GraphRagPipeline::new()
    };
    let result = unfiltered
        .search(&db, "weight", Some("hub central alpha"), None)
        .expect("search");
    assert_eq!(
        result.stats.nodes_expanded, 7,
        "default (None) keeps weak edges, got {}",
        result.stats.nodes_expanded
    );
}

/// MEM0-03 / EGO-15: DEFAULT_MAX_DEGREE_PER_NODE is 128 by default, capping hub fan-out.
#[test]
fn test_default_max_degree_per_node_caps_large_hub() {
    use vantadb::graphrag::pipeline::DEFAULT_MAX_DEGREE_PER_NODE;
    assert_eq!(DEFAULT_MAX_DEGREE_PER_NODE, 128);

    let (db, _dir) = setup_test_db();
    let hub = insert_vector_node(
        &db,
        "large_hub",
        "hub",
        "central mega hub for graph testing",
        vec![0.1, 0.2, 0.3],
    );

    // Create 150 leaves
    for i in 0..150 {
        let leaf = insert_vector_node(
            &db,
            "large_hub",
            &format!("leaf_{i}"),
            &format!("leaf content {i}"),
            vec![0.1 + (i as f32 * 0.001), 0.2, 0.3],
        );
        db.add_edge(hub, leaf, "links_to", Some(1.0), None)
            .expect("edge");
    }

    let default_pipeline = GraphRagPipeline::new();
    assert_eq!(default_pipeline.max_expansion_nodes, 100);
    assert_eq!(default_pipeline.max_degree_per_node, 128);

    let pipeline = GraphRagPipeline::new().with_max_expansion_nodes(256);
    assert_eq!(pipeline.max_degree_per_node, 128);

    let result = pipeline
        .search(&db, "large_hub", Some("central mega hub"), None)
        .expect("search");

    assert_eq!(result.stats.seeds_found, 1);
    // 1 hub seed + 128 capped leaves = 129
    assert_eq!(
        result.stats.nodes_expanded, 129,
        "default pipeline must cap fan-out to 128 leaves + 1 seed = 129"
    );
}

/// MEM0-02 / FIND-608: Min-Max calibration prevents RRF seed scores (~0.016)
/// from being crushed by hop_boost.
#[test]
fn test_min_max_seed_score_calibration() {
    let (db, _dir) = setup_test_db();

    // Insert seed node
    let seed_id = insert_vector_node(
        &db,
        "minmax",
        "seed",
        "the exact matched query seed",
        vec![0.1, 0.2, 0.3],
    );

    // Insert 5 neighbor nodes connected to seed, giving them high degree
    for i in 0..5 {
        let neighbor = insert_vector_node(
            &db,
            "minmax",
            &format!("neighbor_{i}"),
            &format!("neighbor node {i}"),
            vec![0.9, 0.9, 0.9],
        );
        db.add_edge(seed_id, neighbor, "neighbor_of", Some(1.0), None)
            .expect("edge");
    }

    let pipeline = GraphRagPipeline::new();
    let result = pipeline
        .search(&db, "minmax", Some("exact matched query"), None)
        .expect("search");

    assert!(!result.nodes.is_empty());
    // The seed should be the #1 ranked candidate despite neighbors having hop boost
    assert_eq!(
        result.nodes[0].id, seed_id,
        "calibrated seed node must rank #1"
    );
    // Score should be high (norm_seed=1.0 -> 0.6*1.0 + 0.3*1.0 + degree factor > 0.9)
    assert!(
        result.nodes[0].score >= 0.9,
        "calibrated seed score should be >= 0.9, got {}",
        result.nodes[0].score
    );
}

/// GR-03: extract_content falls back to FIELD_PAYLOAD ("__vanta_payload").
#[test]
fn test_extract_content_fallback_to_field_payload() {
    let (db, _dir) = setup_test_db();

    let node_id = insert_text_node(&db, "payload_fallback", "k1", "my secret payload content");

    let pipeline = GraphRagPipeline::new();
    let result = pipeline
        .search(&db, "payload_fallback", Some("secret payload"), None)
        .expect("search");

    assert!(!result.nodes.is_empty());
    let hit = result
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .expect("found node");
    assert_eq!(hit.content, "my secret payload content");
}

/// NOT-01 / FIND-609: LabelIntern persistence across database close and reopen.
#[test]
fn test_label_intern_disk_persistence() {
    let dir = tempfile::tempdir().expect("tempdir");
    let label1 = "discovered_relation";
    let label2 = "sub_task_of";
    let id1;
    let id2;

    {
        let db = Embedded::open(dir.path()).expect("open");
        let a = insert_text_node(&db, "labels", "a", "entity A");
        let b = insert_text_node(&db, "labels", "b", "entity B");
        let c = insert_text_node(&db, "labels", "c", "entity C");

        let engine = db.engine_handle().unwrap();
        id1 = engine.intern_label(label1);
        id2 = engine.intern_label(label2);

        db.add_edge(a, b, label1, Some(1.0), None)
            .expect("add edge 1");
        db.add_edge(b, c, label2, Some(0.8), None)
            .expect("add edge 2");

        // Verify labels resolve in initial session
        assert_eq!(engine.resolve_label(id1), Some(label1.to_string()));
        assert_eq!(engine.resolve_label(id2), Some(label2.to_string()));
        db.close().expect("close");
    }

    // Reopen database from disk and assert labels are preserved
    {
        let db = Embedded::open(dir.path()).expect("reopen");
        let engine = db.engine_handle().unwrap();
        assert_eq!(
            engine.resolve_label(id1),
            Some(label1.to_string()),
            "label1 must survive reopen"
        );
        assert_eq!(
            engine.resolve_label(id2),
            Some(label2.to_string()),
            "label2 must survive reopen"
        );
    }
}
