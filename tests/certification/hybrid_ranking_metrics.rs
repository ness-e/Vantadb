//! TSK-37: Hybrid ranking quality metrics — NDCG@k, MRR, Recall@k
//!
//! Creates a small labeled corpus, runs hybrid (BM25 + vector) search,
//! and computes information retrieval metrics against known relevance judgments.

use tempfile::tempdir;
use vantadb::{VantaEmbedded, VantaMemoryInput, VantaMemorySearchRequest};

fn ndcg_at_k(ranked: &[String], relevant: &[String], k: usize) -> f64 {
    let k = k.min(ranked.len());
    if k == 0 {
        return 0.0;
    }
    let mut dcg = 0.0;
    for i in 0..k {
        let rel = if relevant.contains(&ranked[i]) {
            1.0
        } else {
            0.0
        };
        dcg += (2.0f64.powf(rel) - 1.0) / (i as f64 + 2.0).log2();
    }
    let mut idcg = 0.0;
    for i in 0..k.min(relevant.len()) {
        idcg += 1.0 / (i as f64 + 2.0).log2();
    }
    if idcg > 0.0 {
        dcg / idcg
    } else {
        0.0
    }
}

fn mrr(ranked: &[String], relevant: &[String]) -> f64 {
    for (i, id) in ranked.iter().enumerate() {
        if relevant.contains(id) {
            return 1.0 / (i as f64 + 1.0);
        }
    }
    0.0
}

fn recall_at_k(ranked: &[String], relevant: &[String], k: usize) -> f64 {
    if relevant.is_empty() {
        return 0.0;
    }
    let k = k.min(ranked.len());
    let found = ranked[..k]
        .iter()
        .filter(|id| relevant.contains(id))
        .count();
    found as f64 / relevant.len() as f64
}

#[test]
fn test_hybrid_ranking_metrics() {
    let dir = tempdir().expect("tempdir");
    let db = VantaEmbedded::open(dir.path()).expect("open");

    // 20-document corpus with known relevance for hybrid queries
    let corpus: Vec<(&str, &str, Vec<f32>)> = vec![
        (
            "doc_0",
            "Transformer architecture for natural language processing",
            vec![0.90, 0.10, 0.10, 0.10],
        ),
        (
            "doc_1",
            "Attention mechanisms in deep transformer models",
            vec![0.85, 0.15, 0.10, 0.10],
        ),
        (
            "doc_2",
            "BERT pretraining with masked language modeling",
            vec![0.80, 0.20, 0.15, 0.10],
        ),
        (
            "doc_3",
            "GPT autoregressive language model transformer",
            vec![0.75, 0.10, 0.20, 0.10],
        ),
        (
            "doc_4",
            "Self-attention and multi-head attention explained",
            vec![0.70, 0.30, 0.10, 0.10],
        ),
        (
            "doc_5",
            "Deep convolutional networks for computer vision",
            vec![0.10, 0.90, 0.10, 0.10],
        ),
        (
            "doc_6",
            "Neural machine translation using transformers",
            vec![0.70, 0.20, 0.30, 0.10],
        ),
        (
            "doc_7",
            "Recurrent neural networks for sequence modeling",
            vec![0.20, 0.80, 0.10, 0.10],
        ),
        (
            "doc_8",
            "Reinforcement learning with policy gradients",
            vec![0.10, 0.10, 0.90, 0.10],
        ),
        (
            "doc_9",
            "Generative adversarial networks for image synthesis",
            vec![0.10, 0.10, 0.10, 0.90],
        ),
        (
            "doc_10",
            "Data preprocessing with Python pandas library",
            vec![0.15, 0.10, 0.85, 0.20],
        ),
        (
            "doc_11",
            "SQL database optimization and query tuning",
            vec![0.10, 0.15, 0.10, 0.85],
        ),
        (
            "doc_12",
            "Rust systems programming for performance",
            vec![0.10, 0.10, 0.20, 0.30],
        ),
        (
            "doc_13",
            "Web development with React and TypeScript",
            vec![0.20, 0.10, 0.10, 0.20],
        ),
        (
            "doc_14",
            "Docker container orchestration with Kubernetes",
            vec![0.15, 0.15, 0.70, 0.15],
        ),
        (
            "doc_15",
            "Transformer-based recommendation systems",
            vec![0.65, 0.15, 0.15, 0.15],
        ),
        (
            "doc_16",
            "Efficient fine-tuning of large language models",
            vec![0.60, 0.20, 0.20, 0.15],
        ),
        (
            "doc_17",
            "Model quantization and pruning for edge deployment",
            vec![0.50, 0.30, 0.10, 0.20],
        ),
        (
            "doc_18",
            "Vector databases for semantic search applications",
            vec![0.40, 0.30, 0.30, 0.30],
        ),
        (
            "doc_19",
            "Approximate nearest neighbor search algorithms",
            vec![0.35, 0.25, 0.35, 0.30],
        ),
    ];

    for (key, text, vector) in &corpus {
        let mut input = VantaMemoryInput::new("metrics", *key, *text);
        input.vector = Some(vector.clone());
        db.put(input).expect("put doc");
    }

    // ── Query 1: "transformer attention" ──
    // Relevant: docs touching transformers, attention, or LLMs
    let relevant_1: Vec<String> = vec![
        "doc_0", "doc_1", "doc_2", "doc_3", "doc_4", "doc_6", "doc_15", "doc_16",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let mut req = VantaMemorySearchRequest::default();
    req.namespace = "metrics".to_string();
    req.query_vector = vec![0.85, 0.15, 0.10, 0.10];
    req.text_query = Some("transformer attention".to_string());
    req.top_k = 10;
    let results = db.search(req).expect("search query 1");
    let ranked_1: Vec<String> = results.into_iter().map(|h| h.record.key).collect();

    let ndcg_1 = ndcg_at_k(&ranked_1, &relevant_1, 10);
    let mrr_1 = mrr(&ranked_1, &relevant_1);
    let recall_1 = recall_at_k(&ranked_1, &relevant_1, 10);

    assert!(ndcg_1 > 0.5, "Query1 NDCG@10 > 0.5 (got {})", ndcg_1);
    assert!(mrr_1 > 0.5, "Query1 MRR > 0.5 (got {})", mrr_1);
    assert!(recall_1 > 0.5, "Query1 Recall@10 > 0.5 (got {})", recall_1);

    // ── Query 2: "deep learning" ──
    // Relevant: docs about deep learning, RNNs, CNNs
    let relevant_2: Vec<String> = vec!["doc_1", "doc_5", "doc_7"]
        .into_iter()
        .map(String::from)
        .collect();

    let mut req2 = VantaMemorySearchRequest::default();
    req2.namespace = "metrics".to_string();
    req2.query_vector = vec![0.20, 0.80, 0.10, 0.10];
    req2.text_query = Some("deep learning".to_string());
    req2.top_k = 10;
    let results2 = db.search(req2).expect("search query 2");
    let ranked_2: Vec<String> = results2.into_iter().map(|h| h.record.key).collect();

    let ndcg_2 = ndcg_at_k(&ranked_2, &relevant_2, 10);
    let mrr_2 = mrr(&ranked_2, &relevant_2);
    let recall_2 = recall_at_k(&ranked_2, &relevant_2, 10);

    assert!(ndcg_2 > 0.4, "Query2 NDCG@10 > 0.4 (got {})", ndcg_2);
    assert!(mrr_2 > 0.3, "Query2 MRR > 0.3 (got {})", mrr_2);
    assert!(recall_2 > 0.3, "Query2 Recall@10 > 0.3 (got {})", recall_2);
}
