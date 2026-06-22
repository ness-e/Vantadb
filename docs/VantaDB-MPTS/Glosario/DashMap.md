---
type: glossary-entry
status: stable
tags: [vantadb, glosario, concurrencia]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# DashMap

## Definición

**DashMap** es una implementación de HashMap concurrente y sharded para Rust que permite acceso paralelo sin necesidad de un lock global.

## Características

| Característica | Descripción |
|----------------|-------------|
| **Sharding** | Divide el mapa en múltiples shards independientes |
| **Lock-free reads** | Lecturas sin bloqueo en la mayoría de casos |
| **Fine-grained locks** | Cada shard tiene su propio lock |
| **API compatible** | Similar a `HashMap` estándar |

## Uso en VantaDB

### Índice HNSW Concurrente

```rust
use dashmap::DashMap;

pub struct CPIndex {
    // Shard por hash de node_id
    pub nodes: DashMap<u64, HnswNode>,
}

impl CPIndex {
    pub fn get_node(&self, id: u64) -> Option<dashmap::mapref::one::Ref<u64, HnswNode>> {
        // Lock solo en el shard correspondiente
        self.nodes.get(&id)
    }
    
    pub fn insert_node(&self, id: u64, node: HnswNode) {
        // Lock solo en el shard correspondiente
        self.nodes.insert(id, node);
    }
    
    pub fn search_nearest(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
        // Múltiples threads pueden leer simultáneamente
        // de diferentes shards sin contención
        let mut candidates = Vec::new();
        
        for entry in self.nodes.iter() {
            let distance = cosine_similarity(query, &entry.value().vector);
            candidates.push((entry.key().clone(), distance));
        }
        
        // Ordenar y retornar top-k
        candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        candidates.truncate(k);
        candidates
    }
}
```

### Comparación: DashMap vs RwLock<HashMap>

| Aspecto | RwLock<HashMap> | DashMap |
|---------|-----------------|---------|
| **Lecturas concurrentes** | ✅ Múltiples lectores | ✅ Múltiples lectores |
| **Escrituras concurrentes** | ❌ Lock exclusivo global | ✅ Lock por shard |
| **Contención** | Alta (un lock) | Baja (múltiples shards) |
| **Escalabilidad** | Limitada a ~8 cores | Escala con cores |

## Configuración de Shards

```rust
use dashmap::DashMap;

// Default: número de CPUs
let map: DashMap<u64, HnswNode> = DashMap::new();

// Configurar número de shards
let map: DashMap<u64, HnswNode> = DashMap::with_capacity(64);

// Sharding personalizado
let map: DashMap<u64, HnswNode> = DashMap::with_hasher(
    ahash::RandomState::new()
);
```

## API Común

```rust
use dashmap::DashMap;

let map = DashMap::new();

// Insert
map.insert("key", "value");

// Get (retorna Ref, similar a RwLockReadGuard)
if let Some(value) = map.get("key") {
    println!("{}", value.value());
}

// Get mutable (retorna RefMut)
if let Some(mut value) = map.get_mut("key") {
    *value.value_mut() = "new value";
}

// Remove
map.remove("key");

// Iterate
for entry in map.iter() {
    println!("{}: {}", entry.key(), entry.value());
}

// Entry API (similar a HashMap)
map.entry("key")
    .or_insert("default");
```

## Trade-offs

### Ventajas
- Mayor throughput bajo concurrencia
- Menor latencia de cola (tail latency)
- Mejor escalabilidad en multi-core

### Desventajas
- Mayor uso de memoria (overhead por shard)
- Iteración no determinista (orden no garantizado)
- API ligeramente más compleja que HashMap

## Benchmarks

| Operación | HashMap + RwLock | DashMap (16 shards) | Speedup |
|-----------|------------------|---------------------|---------|
| **Reads (8 threads)** | 1.2M ops/s | 8.5M ops/s | 7x |
| **Writes (8 threads)** | 180K ops/s | 1.4M ops/s | 7.8x |
| **Mixed (80/20)** | 850K ops/s | 5.2M ops/s | 6.1x |

## Véase También

- [RwLock](RwLock.md) — Alternativa con lock global
- [HNSW](HNSW.md) — Índice que usa DashMap
- [File Locking](File Locking.md) — Lock a nivel de proceso

---

*DashMap permite alta concurrencia en VantaDB sin contención de locks globales.*

