---
title: ANN (Approximate Nearest Neighbor)
kind: glossary
status: stable
description: "##Definition"
tags: [vantadb, glosario, indexes, vector]
type: glossary-entry
last_reviewed: "2026-09-15"
links: "[[README.md]]"
---

# ANN (Approximate Nearest Neighbor)

##Definition

**ANN** (Approximate Nearest Neighbor Search) is a family of algorithms that find vectors similar to a query without examining all the vectors in the dataset, sacrificing accuracy for speed.

## Accuracy vs Speed

| M├⌐todo | Complejidad | Recall | Velocidad |
|--------|-------------|--------|-----------|
| **Exact (KNN)** | O(N┬╖d) | 100% | Lento |
| **ANN (HNSW)** | O(log N┬╖d) | ~95-99% | R├ípido |

## Main ANN Algorithms

### 1. HNSW (Hierarchical Navigable Small World)

**Used by:** VantaDB, Qdrant, Milvus, Weaviate

```
Capa 2:    [A] ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ [D]
Capa 1:    [A] ΓöÇΓöÇ [B] ΓöÇΓöÇ [D]
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
  C1 ΓåÆ [v1, v5, v12, ...]
  C2 ΓåÆ [v2, v7, v8, ...]
  C3 ΓåÆ [v3, v4, v9, ...]
```

**Advantages:**
- Memory compression
- Parallel search

### 3. LSH (Locality-Sensitive Hashing)

**Used by:** Research, legacy systems

```
Hash functions: h1, h2, ..., hk
Buckets:
  h1(v) = 5 ΓåÆ [v1, v3, v7]
  h2(v) = 2 ΓåÆ [v2, v5, v8]
```

## Evaluation Metrics

### Recall@K

$$
\text{Recall@K} = \frac{|\text{Retrieved} \cap \text{Relevant}|}{|\text{Relevant}|}
$$

**VantaDB Target:** ΓëÑ0.95 for K=10

### Latency

| Percentil | Descripci├│n |
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

| Par├ímetro | Default | Efecto |
|-----------|---------|--------|
| `M` | 16 | Conexiones por nodo |
| `ef_construction` | 200 | Calidad de construcci├│n |
| `ef` | 100 | Calidad de b├║squeda |

### Recall vs Latency Trade-off

```python
import vantadb

# HNSW params (M, ef_construction, ef) live in the Rust engine config,
# not the constructor. ef is auto-tuned at runtime.
db = vantadb.Client("./data")
# Alta calidad (m├ís lento) ΓÇö Recall: 0.998, Latencia: 15ms
# Balanced ΓÇö Recall: 0.956, Latency: 6ms
# High speed (less accurate) ΓÇö Recall: 0.890, Latency: 3ms
```

## VantaDB Benchmarks (SIFT1M)

| Configuraci├│n | Recall@10 | p50 Latency | QPS |
|---------------|-----------|-------------|-----|
| ef=50 | 0.912 | 4.2ms | 238 |
| ef=100 | 0.956 | 6.1ms | 164 |
| ef=200 | 0.981 | 9.8ms | 102 |
| ef=500 | 0.998 | 15.4ms | 65 |

## See Also

- [hnsw](./hnsw.md) ΓÇö Algoritmo ANN espec├¡fico de VantaDB
- [vector-similarity](./vector-similarity.md) ΓÇö M├⌐tricas de distancia
- [benchmarks](./benchmarks.md) ΓÇö Evaluaci├│n de performance

---

*ANN allows sub-millisecond vector searches on datasets of millions of vectors.*

