//! Basic CRUD example: create an Embedded instance, add memory records with
//! vectors and metadata, search by vector similarity, and print results.

use std::error::Error;
use vantadb::config::Config;
use vantadb::{Embedded, MemoryInput, MemorySearchRequest, Value};

fn main() -> Result<(), Box<dyn Error>> {
    let db = Embedded::open_with_config(Config {
        storage_path: "./examples_basic_data".into(),
        ..Default::default()
    })?;

    let records = vec![
        ("doc-1", "The quick brown fox jumps over the lazy dog"),
        ("doc-2", "A fast brown fox leaps over a sleepy hound"),
        (
            "doc-3",
            "Vector databases store embeddings for similarity search",
        ),
    ];

    for (key, payload) in &records {
        db.put(MemoryInput {
            namespace: "demo".into(),
            key: (*key).into(),
            payload: (*payload).into(),
            metadata: vec![("source".into(), Value::String("example".into()))]
                .into_iter()
                .collect(),
            vector: Some(vec![0.1, 0.2, 0.3]),
            sparse_vector: None,
            ttl_ms: None,
        })?;
    }

    println!("Inserted {} records", records.len());

    for hit in db.search(MemorySearchRequest {
        namespace: "demo".into(),
        query_vector: vec![0.1, 0.2, 0.3],
        top_k: 3,
        ..Default::default()
    })? {
        println!(
            "  score={:.4}  key={}  payload={}",
            hit.score, hit.record.key, hit.record.payload
        );
    }

    if let Some(record) = db.get("demo", "doc-1")? {
        println!("Retrieved: key={} payload={}", record.key, record.payload);
    }

    Ok(())
}
