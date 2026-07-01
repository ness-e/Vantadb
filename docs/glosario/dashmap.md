---
type: glossary-entry
status: stable
tags: [vantadb, glosario, concurrencia]
last_refined: 2026-06
links: "[[README.md]]"
---
#DashMap

##Definition

**DashMap** is a concurrent and sharded HashMap implementation for Rust that allows parallel access without the need for a global lock.

## Characteristics

| Característica | Descripción |
|----------------|-------------|
| **Sharding** | Divide el mapa en múltiples shards independientes |
| **Lock-free reads** | Lecturas sin bloqueo en la mayoría de casos |
| **Fine-grained locks** | Cada shard tiene su propio lock |
| **API compatible** | Similar a `HashMap` estándar |

## Usage in VantaDB

### Concurrent HNSW Index

```rust
use dashmap::DashMap;

pub struct CPIndex {
    // Shard by hash of node_id
    pub nodes: DashMap<u64, HnswNode>,
}

impl CPIndex {
    pub fn get_node(&self, id: u64) -> Option<dashmap::mapref::one::Ref<u64, HnswNode>> {
        // Lock only on the corresponding shard
        self.nodes.get(&id)
    }
    
    pub fn insert_node(&self, id: u64, node: HnswNode) {
        // Lock only on the corresponding shard
        self.nodes.insert(id, node);
    }
    
    pub fn search_nearest(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        // Multiple threads can read simultaneously
        // from different shards without contention
        let mut candidates = Vec::new();
        
        for entry in self.nodes.iter() {
            let distance = cosine_similarity(query, &entry.value().vector);
            candidates.push((entry.key().clone(), distance));
        }
        
        // Sort and return top-k
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates.truncate(k);
        candidates
    }
}
```

### Comparison: DashMap vs RwLock<HashMap>

| Aspecto | RwLock<HashMap> | DashMap |
|---------|-----------------|---------|
| **Lecturas concurrentes** | ✅ Múltiples lectores | ✅ Múltiples lectores |
| **Escrituras concurrentes** | ❌ Lock exclusivo global | ✅ Lock por shard |
| **Contención** | Alta (un lock) | Baja (múltiples shards) |
| **Escalabilidad** | Limitada a ~8 cores | Escala con cores |

## Shard Configuration

```rust
use dashmap::DashMap;

// Default: number of CPUs
let map: DashMap<u64, HnswNode> = DashMap::new();

// Configure number of shards
let map: DashMap<u64, HnswNode> = DashMap::with_capacity(64);

// Custom sharding
let map: DashMap<u64, HnswNode> = DashMap::with_hasher(
    ahash::RandomState::new()
);
```

## Common API

```rust
use dashmap::DashMap;

let map = DashMap::new();

//Insert
map.insert("key", "value");

// Get (returns Ref, similar to RwLockReadGuard)
if let Some(value) = map.get("key") {
    println!("{}", value.value());
}

// Get mutable (returns RefMut)
if let Some(mut value) = map.get_mut("key") {
    *value.value_mut() = "new value";
}

//Remove
map.remove("key");

// Iterate
for entry in map.iter() {
    println!("{}: {}", entry.key(), entry.value());
}

// Entry API (similar to HashMap)
map.entry("key")
    .or_insert("default");
```

## Trade-offs

### Advantages
- Higher throughput under concurrency
- Lower tail latency
- Better scalability in multi-core

### Disadvantages
- Greater memory usage (overhead per shard)
- Non-deterministic iteration (order not guaranteed)
- Slightly more complex API than HashMap

## Benchmarks

| Operación | HashMap + RwLock | DashMap (16 shards) | Speedup |
|-----------|------------------|---------------------|---------|
| **Reads (8 threads)** | 1.2M ops/s | 8.5M ops/s | 7x |
| **Writes (8 threads)** | 180K ops/s | 1.4M ops/s | 7.8x |
| **Mixed (80/20)** | 850K ops/s | 5.2M ops/s | 6.1x |

## See Also

- [[rwlock]] — Alternative with global lock
- [[hnsw]] — Index used by DashMap
- [[file-locking]] — Lock at the process level

---

*DashMap enables high concurrency in VantaDB without global lock contention.*

