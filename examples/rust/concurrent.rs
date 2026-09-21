// ponytail: blanket allow — unwraps with documented invariants; documented per-call.
#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Concurrent access example: insert and search from multiple threads
//! using Embedded behind an Arc reference.
//! Embedded is Send + Sync and safe to share across threads.

use std::error::Error;
use std::sync::Arc;
use std::thread;
use vantadb::config::Config;
use vantadb::{Embedded, MemoryInput, MemorySearchRequest};

fn main() -> Result<(), Box<dyn Error>> {
    let db = Arc::new(Embedded::open_with_config(Config {
        storage_path: "./examples_concurrent_data".into(),
        ..Default::default()
    })?);

    let mut handles = Vec::new();

    // ── Thread 1: insert records ──
    let db_clone = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        for i in 0..100 {
            if let Err(e) = db_clone.put(MemoryInput::new(
                "concurrent",
                format!("key-{}", i),
                format!("Concurrent record number {}", i),
            )) {
                eprintln!("Writer error: {e}");
            }
        }
        println!("Writer: inserted 100 records");
    }));

    // ── Thread 2: search repeatedly ──
    let db_clone = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        for _ in 0..10 {
            match db_clone.search(MemorySearchRequest {
                namespace: "concurrent".into(),
                query_vector: vec![0.1, 0.2, 0.3],
                top_k: 5,
                ..Default::default()
            }) {
                Ok(results) => println!("Searcher: got {} results", results.len()),
                Err(e) => eprintln!("Searcher error: {e}"),
            }
        }
    }));

    // ── Thread 3: read individual records ──
    let db_clone = Arc::clone(&db);
    handles.push(thread::spawn(move || {
        for i in 0..50 {
            match db_clone.get("concurrent", &format!("key-{}", i)) {
                Ok(Some(record)) => {
                    assert_eq!(record.payload, format!("Concurrent record number {}", i));
                }
                Ok(None) => eprintln!("Reader: key-{} not found", i),
                Err(e) => eprintln!("Reader error: {e}"),
            }
        }
        println!("Reader: verified 50 records");
    }));

    // Wait for all threads.
    for h in handles {
        h.join().expect("thread panicked");
    }

    // ── Final count ──
    let list = db.list("concurrent", Default::default())?;
    println!(
        "\nFinal record count: {} (expected 100)",
        list.records.len()
    );

    Ok(())
}
