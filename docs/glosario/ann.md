---
type: glossary-entry
status: stable
tags: [vantadb, glosario, indexes, vector]
last_refined: 2026-06
links: "[[README.md]]"
---
# ANN (Approximate Nearest Neighbor)

##Definition

**ANN** (Approximate Nearest Neighbor Search) is a family of algorithms that find vectors similar to a query without examining all the vectors in the dataset, sacrificing accuracy for speed.

## Accuracy vs Speed

| Método | Complejidad | Recall | Velocidad |
|--------|-------------|--------|-----------|
| **Exact (KNN)** | O(N·d) | 100% | Lento |
| **ANN (HNSW)** | O(log N·d) | ~95-99% | Rápido |

## Main ANN Algorithms

### 1. HNSW (Hierarchical Navigable Small World)

**Used by:** VantaDB, Qdrant, Milvus, Weaviate

```
Capa 2:    [A] ──────── [D]
Capa 1:    [A] ── [B] ── [D]
Capa 0:    [A]-[B]-[C]-[D]-[E]-[F]
```

**Advantages:**
- High recall (0.95+)
- Low latency
- Simple to implement

### 2. IVF (Inverted File Index)

**Used by:** FAISS, LanceDB

```
Centroids: [C1, C2, C3, ..., Ck]
Inverted lists:
  C1 → [v1, v5, v12, ...]
  C2 → [v2, v7, v8, ...]
  C3 → [v3, v4, v9, ...]
```

**Advantages:**
- Memory compression
- Parallel search

### 3. LSH (Locality-Sensitive Hashing)

**Used by:** Research, legacy systems

```
Hash functions: h1, h2, ..., hk
Buckets:
  h1(v) = 5 → [v1, v3, v7]
  h2(v) = 2 → [v2, v5, v8]
```

## Evaluation Metrics

### Recall@K

$$
\text{Recall@K} = \frac{|\text{Retrieved} \cap \text{Relevant}|}{|\text{Relevant}|}
$$

**VantaDB Target:** ≥0.95 for K=10

### Latency

| Percentil | Descripción |
|-----------|-------------|
| **p50** | Latencia mediana |
| **p95** | 95% de queries bajo este valor |
| **p99** | 99% de queries bajo este valor |

### QPS (Queries Per Second)

$$
\text{QPS} = \frac{\text{Total queries}}{\text{Tiempo total (segundos)}}
$$

## Implementation in VantaDB

### HNSW Parameters

| Parámetro | Default | Efecto |
|-----------|---------|--------|
| `M` | 16 | Conexiones por nodo |
| `ef_construction` | 200 | Calidad de construcción |
| `ef_search` | 100 | Calidad de búsqueda |

### Recall vs Latency Trade-off

```python
# Alta calidad (más lento)
db = VantaEmbedded("./data", config={
    "hnsw": {
        "ef_search": 500,  # Más candidatos
        "M": 32            # Más conexiones
    }
})
# Recall: 0.998, Latencia: 15ms

# Balanced
db = VantaEmbedded("./data", config={
    "hnsw": {
        "ef_search": 100,
        "M": 16
    }
})
# Recall: 0.956, Latency: 6ms

# High speed (less accurate)
db = VantaEmbedded("./data", config={
    "hnsw": {
        "ef_search": 50,
        "M": 8
    }
})
# Recall: 0.890, Latency: 3ms
```

## VantaDB Benchmarks (SIFT1M)

| Configuración | Recall@10 | p50 Latency | QPS |
|---------------|-----------|-------------|-----|
| ef_search=50 | 0.912 | 4.2ms | 238 |
| ef_search=100 | 0.956 | 6.1ms | 164 |
| ef_search=200 | 0.981 | 9.8ms | 102 |
| ef_search=500 | 0.998 | 15.4ms | 65 |

## See Also

- [[hnsw]] — Algoritmo ANN específico de VantaDB
- [[vector-similarity]] — Métricas de distancia
- [[benchmarks]] — Evaluación de performance

---

*ANN allows sub-millisecond vector searches on datasets of millions of vectors.*

