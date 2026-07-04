---
title: "busqueda-vectorial"
type: glossary-entry
status: stable
tags: [indice, ann, busqueda-vector, hnsw]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Hierarchical Navigable Small World, HNSW Index]
description: "Indexing algorithm for approximate nearest neighbor search (ANN) that constructs a multi-layer graph of vectors, allowing searches in logarithmic time"
---
#HNSW—Hierarchical Navigable Small World

##Definition

**HNSW** is an indexing algorithm for **approximate nearest neighbor search (ANN)** that constructs a multi-layer graph of vectors, allowing searches in **logarithmic** time ($O(\log N)$) with high recall (>0.95).

## The Problem It Solves

### Exhaustive Search (Brute Force)

```
Query: vector q
Para cada vector v_i en la base de datos:
    calcular distancia(q, v_i)
Ordenar por distancia
Retornar top-K
```

**Complexity:** $O(N \cdot d)$ where N = vectors, d = dimensions

| N (vectores) | Tiempo (384d) |
|--------------|---------------|
| 1,000 | ~1 ms |
| 100,000 | ~100 ms |
| 10,000,000 | ~10 segundos ❌ |

### Solución: Búsqueda Aproximada (ANN)

HNSW finds vectors **close to the optimal** without comparing them all:

| N (vectores) | Brute Force | HNSW | Speedup |
|--------------|-------------|------|---------|
| 100,000 | 100 ms | 6 ms | 16x |
| 1,000,000 | 1 segundo | 15 ms | 66x |

## Structure of the HNSW Graph

```
Capa 2 (más dispersa):
    [A] ────────── [D]
     │              │
     └──── [C] ─────┘

Layer 1 (intermediate):
    [A] ─── [B] ─── [D]
     │ │ │
    [E] ─── [C] ─── [F]

Layer 0 (densest, all vectors):
    [A]─[B]─[C]─[D]─[E]─[F]─[G]─[H]─[I]─[J]
```

### Key Properties

1. **Multi-layer:** Higher layers have fewer nodes (fast navigation)
2. **Small World:** Any node is reachable in a few hops
3. **Navigable:** Greedy searches converge to local optimum

## Search Algorithm

```
Input: query q, K (top-K), ef (ef search)

1. Start at the highest layer entry node
2. For each layer (top to bottom):
     Greedy search until converge
3. At layer 0:
     Search with candidate list (ef)
     Keep top-K closest
4. Return top-K
```

### Key Parameters

| Parámetro | Descripción | Valor Típico | Impacto |
|-----------|-------------|--------------|---------|
| **M** | Máximo de conexiones por nodo | 16-32 | Mayor M = más memoria, mejor recall |
| **ef_construction** | Candidatos durante construcción | 200-500 | Mayor = mejor grafo, más lento construir |
| **ef_search** | Candidatos durante búsqueda | 50-200 | Mayor = mejor recall, más lento buscar |

## Implementation in VantaDB

### Data Structure

```rust
pub struct HnswIndex {
    layers: Vec<HnswLayer>,
    entry_point: NodeId,
    max_layer: usize,
    params: HnswParams,
}

pub struct HnswLayer {
    nodes: HashMap<NodeId, HnswNode>,
}

pub struct HnswNode {
    id: NodeId,
    vector: Vec<f32>, // Or mmap pointer
    neighbors: Vec<NodeId>,
}
```

### Search in VantaDB

```python
# busqueda-vectorial
results = db.search(
    vector=[0.12, -0.34, ...],  # Query vector
    top_k=10,                    # Retornar 10 más cercanos
    ef_search=100                # Parámetro de calidad
)
```

### Index Persistence

VantaDB persists the HNSW index using **[[mmap]]**:

```
Disco: vector_store.vanta
├── Header (magic, version, params)
├── Layer 0 nodes (vectores + neighbors)
├── Layer 1 nodes
├── ...
└── Layer N nodes

Runtime: mmap() → shortcut without copying to RAM
```

**Advantage:** Instant loading (no rebuilding from scratch).

## Performance Metrics in VantaDB

### Recall vs Latencia (SIFT1M, 128d)

| ef_search | Recall@10 | Latencia p50 | Latencia p99 |
|-----------|-----------|--------------|--------------|
| 50 | 0.92 | 3 ms | 12 ms |
| 100 | 0.96 | 6 ms | 18 ms |
| 200 | 0.98 | 12 ms | 35 ms |

### Escalabilidad

| Dataset | Recall@10 | Latencia p50 | Memory |
|---------|-----------|--------------|--------|
| 10K vectores | 0.956 | 1.2 ms | ~12 MB |
| 50K vectores | 1.000 | 6.1 ms | ~58 MB |
| 100K vectores | 0.998 | 12.4 ms | ~117 MB |

**Scaling factor:** 4.88x (sub-linear, better than $O(N)$)

## Optimizations in VantaDB

### 1. SIMD for Distances

```rust
#[cfg(target_arch = "x86_64")]
unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    // 8 floats por instrucción (AVX2)
    // Speedup: 4-8x vs escalar
}
```

### 2. Layout Cache-Friendly

```rust
// ANTES: Vec<Vec<f32>> → cache misses
// DESPUÉS: Vec<f32> plano → localidad espacial
struct FlatVectors {
    data: Vec<f32>,      // Todos los vectores contiguos
    dim: usize,
}

impl FlatVectors {
    fn get(&self, idx: usize) -> &[f32] {
        let start = idx * self.dim;
        &self.data[start..start + self.dim]
    }
}
```

### 3. Cuantización SQ8 (Roadmap)

Reduce `f32` (4 bytes) → `u8` (1 byte):
- **4x less memory**
- **2-4x faster** in searches
- **~1-2% recall loss**

## Comparison with Other ANN Algorithms

| Algoritmo | Estructura | Construcción | Búsqueda | Memoria | Caso de Uso |
|-----------|-----------|--------------|----------|---------|-------------|
| **HNSW** | Grafo multi-capa | Media | Rápida | Alta | General purpose |
| **IVF** | Particionamiento + centroids | Rápida | Media | Media | Datasets muy grandes |
| **PQ** | Compresión de vectores | Lenta | Rápida | Muy baja | Memoria limitada |
| **ScaNN** | Cuantización + reranking | Media | Muy rápida | Media | Google-scale |
| **FAISS** | Múltiples (IVF, PQ, HNSW) | Variable | Variable | Variable | Research |

### Why VantaDB Chooses HNSW

1. **Balance recall/latency:** Best trade-off for medium datasets
2. **Simple to implement:** No centroid training
3. **Direct persistence:** Graph can be mmap-eared
4. **No training required:** Works immediately

## HNSW Trade-offs

| Ventaja | Costo |
|---------|-------|
| Búsqueda rápida ($O(\log N)$) | Construcción lenta ($O(N \log N)$) |
| Alto recall (>0.95) | Memoria alta (~1KB/vector) |
| Simple de usar | Parámetros requieren tuning |
| Persistencia mmap | Reconstrucción costosa si se corrompe |

## Known Issues

### AUD-03: Concurrency in Rebuild

**Severity:** ⚠️ High

If `rebuild_index()` is run concurrently with lookups, readers may see an inconsistent index.

**Mitigation:** [[rwlock]] global or double buffer strategy.

### AUD-04: Validación de SIMD

**Severity:** ℹ️ Medium

The scalar fallback and the SIMD route coexist without numerical equivalence tests.

**Mitigation:** Property-based tests comparing SIMD vs scalar.

## See Also

- [[vectors]] — What HNSW indexes
- [[vector-similarity]] — Distance metrics
- [[mmap]] — Index persistence
- [[bm25]] — Supplementary index (lexicon)
- [[rrf]] — HNSW + BM25 merger

### Related Implementation Documentation
- [[../architecture/hnsw_index|HNSW Index Architecture]]

---

*HNSW is the de facto standard for ANN vector search in production.*

