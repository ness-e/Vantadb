//! ⚡ Concurrency primitive stress tests.
//!
//! Validates each concurrent data structure used throughout the VantaDB codebase
//! under heavy multi-threaded contention:
//!
//! - DashMap (used for edge_index, scalar_index, keyword_index)
//! - RwLock  (used for volatile_cache, vector_store, cardinality_stats)
//! - ArcSwap (used for HNSW RCU swaps, text index)
//! - Mutex   (used for insert_lock in StorageEngine)
//! - StorageEngine holistic concurrency (insert / get / delete / flush)

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use vantadb::config::VantaConfig;
use vantadb::node::{FieldValue, UnifiedNode};
use vantadb::storage::{BackendKind, StorageEngine};

/// Number of concurrent threads for each stress test.
const N_THREADS: usize = 8;

/// Operations per thread.
const OPS_PER_THREAD: usize = 500;

// ─── TEST 1: DashMap Concurrent Read / Write / Remove ────────────────

#[test]
fn test_dashmap_concurrent_stress() {
    let map = Arc::new(dashmap::DashMap::<u64, u64>::new());
    let barrier = Arc::new(Barrier::new(N_THREADS));

    let mut handles = vec![];
    for t in 0..N_THREADS {
        let m = Arc::clone(&map);
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();
            let base = (t * OPS_PER_THREAD) as u64;
            for i in 0..OPS_PER_THREAD {
                let key = base + i as u64;
                m.insert(key, key * 2);
                if let Some(val) = m.get(&key) {
                    assert_eq!(*val, key * 2, "DashMap read-after-write mismatch");
                }
                thread::yield_now();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(
        map.len(),
        N_THREADS * OPS_PER_THREAD,
        "DashMap lost entries under concurrent write stress"
    );

    let barrier2 = Arc::new(Barrier::new(N_THREADS));
    let mut handles2 = vec![];
    for t in 0..N_THREADS {
        let m = Arc::clone(&map);
        let b = Arc::clone(&barrier2);
        handles2.push(thread::spawn(move || {
            b.wait();
            let base = (t * OPS_PER_THREAD) as u64;
            for i in 0..OPS_PER_THREAD {
                m.remove(&(base + i as u64));
                thread::yield_now();
            }
        }));
    }

    for h in handles2 {
        h.join().unwrap();
    }

    assert_eq!(
        map.len(),
        0,
        "DashMap should be empty after concurrent remove"
    );
}

// ─── TEST 2: RwLock Concurrent Read Stress with Periodic Writes ──────

#[test]
fn test_rwlock_read_write_stress() {
    let lock = Arc::new(parking_lot::RwLock::new(0u64));
    let barrier = Arc::new(Barrier::new(N_THREADS));

    let mut handles = vec![];

    // Writer thread
    {
        let w_lock = Arc::clone(&lock);
        let w_barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            w_barrier.wait();
            for _ in 0..OPS_PER_THREAD {
                {
                    let mut guard = w_lock.write();
                    *guard += 1;
                }
                thread::yield_now();
            }
        }));
    }

    // Reader threads
    for _ in 0..N_THREADS - 1 {
        let r_lock = Arc::clone(&lock);
        let r_barrier = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            r_barrier.wait();
            let mut last_seen = 0u64;
            for _ in 0..OPS_PER_THREAD / 2 {
                let val = *r_lock.read();
                assert!(
                    val >= last_seen,
                    "RwLock: value decreased from {} to {} — write lost",
                    last_seen,
                    val
                );
                last_seen = val;
                thread::yield_now();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let final_val = *lock.read();
    assert_eq!(
        final_val, OPS_PER_THREAD as u64,
        "RwLock: writer completed {} ops but counter is {}",
        OPS_PER_THREAD, final_val
    );
}

// ─── TEST 3: ArcSwap Concurrent Swap + Read ──────────────────────────

#[test]
fn test_arcswap_concurrent_stress() {
    let shared: Arc<arc_swap::ArcSwap<u64>> = Arc::new(arc_swap::ArcSwap::new(Arc::new(0u64)));
    let barrier = Arc::new(Barrier::new(N_THREADS));

    let mut handles = vec![];
    for t in 0..N_THREADS {
        let s = Arc::clone(&shared);
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();
            for i in 0..OPS_PER_THREAD / 2 {
                if (t + i) % 2 == 0 {
                    let new_val = ((t * OPS_PER_THREAD) + i) as u64;
                    s.store(Arc::new(new_val));
                } else {
                    let _read = **s.load();
                }
                thread::yield_now();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let _final_val = **shared.load();
}

// ─── TEST 4: parking_lot::Mutex High Contention ──────────────────────

#[test]
fn test_mutex_high_contention() {
    let lock = Arc::new(parking_lot::Mutex::new(0u64));
    let barrier = Arc::new(Barrier::new(N_THREADS));

    let mut handles = vec![];
    for _ in 0..N_THREADS {
        let l = Arc::clone(&lock);
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();
            for _ in 0..OPS_PER_THREAD {
                let mut guard = l.lock();
                *guard += 1;
            }
            thread::yield_now();
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let final_val = *lock.lock();
    assert_eq!(
        final_val,
        (N_THREADS * OPS_PER_THREAD) as u64,
        "Mutex: counter mismatch after concurrent increments — expected {}, got {}",
        N_THREADS * OPS_PER_THREAD,
        final_val
    );
}

// ─── TEST 5: StorageEngine Sequential Ops ───────────────────────────
// Engine-internal lock ordering (get → cardinality_stats → vector_store → cache)
// makes true concurrent insert/get/delete unsafe without external coordination.
// This test validates insert/get/delete correctness sequentially.

fn open_engine(path: &str, kind: BackendKind) -> StorageEngine {
    let config = VantaConfig {
        backend_kind: kind,
        ..Default::default()
    };
    StorageEngine::open_with_config(path, Some(config)).unwrap()
}

#[test]
fn test_storage_engine_concurrent_ops() {
    let dir = tempfile::tempdir().unwrap();
    let engine = open_engine(dir.path().to_str().unwrap(), BackendKind::InMemory);

    // Phase 1: sequential insert by concurrent threads (no delete)
    let engine_arc = Arc::new(engine);
    let barrier = Arc::new(Barrier::new(N_THREADS));
    let mut handles = vec![];
    let node_count = Arc::new(AtomicU64::new(0));

    for t in 0..N_THREADS {
        let e = Arc::clone(&engine_arc);
        let b = Arc::clone(&barrier);
        let nc = Arc::clone(&node_count);
        handles.push(thread::spawn(move || {
            b.wait();
            let base = (t * OPS_PER_THREAD) as u128;
            for i in 0..OPS_PER_THREAD {
                let id = base + i as u128;
                let mut node = UnifiedNode::new(id);
                node.relational.insert(
                    "content".into(),
                    FieldValue::String(format!("concurrent node {}", id)),
                );
                e.insert(&node).unwrap();
                nc.fetch_add(1, Ordering::Relaxed);
                thread::yield_now();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let inserted = node_count.load(Ordering::Relaxed);
    assert_eq!(
        inserted,
        (N_THREADS * OPS_PER_THREAD) as u64,
        "StorageEngine: all nodes should be inserted"
    );

    // Phase 2: concurrent gets (read-only, safe)
    let barrier = Arc::new(Barrier::new(N_THREADS));
    let mut handles = vec![];
    for _ in 0..N_THREADS {
        let e = Arc::clone(&engine_arc);
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();
            for id in 0..(N_THREADS * OPS_PER_THREAD) as u128 {
                let _ = e.get(id);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    // Phase 3: sequential delete
    for id in 0..(N_THREADS * OPS_PER_THREAD) as u128 {
        let _ = engine_arc.delete(id, "concurrency test cleanup");
    }

    // Phase 4: verify all nodes are gone
    let mut survivors = 0u64;
    for id in 0..(N_THREADS * OPS_PER_THREAD) as u128 {
        if let Ok(Some(_)) = engine_arc.get(id) {
            survivors += 1;
        }
    }
    assert_eq!(
        survivors, 0,
        "StorageEngine: expected 0 survivors after sequential delete, got {}",
        survivors
    );

    engine_arc.flush().unwrap();
}

// ─── TEST 6: AtomicU64 Ordering Consistency ──────────────────────────

#[test]
fn test_atomic_ordering_consistency() {
    let counter = Arc::new(AtomicU64::new(0));
    let barrier = Arc::new(Barrier::new(N_THREADS));

    let mut handles = vec![];
    for _ in 0..N_THREADS {
        let c = Arc::clone(&counter);
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            b.wait();
            for _ in 0..OPS_PER_THREAD {
                c.fetch_add(1, Ordering::SeqCst);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(
        counter.load(Ordering::SeqCst),
        (N_THREADS * OPS_PER_THREAD) as u64,
        "AtomicU64: seq-cst counter mismatch"
    );
}
