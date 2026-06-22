---
type: glossary-entry
status: stable
tags: [vantadb, glosario, índices, vectorial]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
---

# ANN (Approximate Nearest Neighbor)

## Definición

**ANN** (Búsqueda Aproximada de Vecinos Más Cercanos) es una familia de algoritmos que encuentran vectores similares a una consulta sin examinar todos los vectores del dataset, sacrificando exactitud por velocidad.

## Exactitud vs Velocidad

| Método | Complejidad | Recall | Velocidad |
|--------|-------------|--------|-----------|
| **Exact (KNN)** | O(N·d) | 100% | Lento |
| **ANN (HNSW)** | O(log N·d) | ~95-99% | Rápido |

## Algoritmos ANN Principales

### 1. HNSW (Hierarchical Navigable Small World)

**Usado por:** VantaDB, Qdrant, Milvus, Weaviate

```
Capa 2:    [A] ──────── [D]
Capa 1:    [A] ── [B] ── [D]
Capa 0:    [A]-[B]-[C]-[D]-[E]-[F]
```

**Ventajas:**
- Alto recall (0.95+)
- Baja latencia
- Simple de implementar

### 2. IVF (Inverted File Index)

**Usado por:** FAISS, LanceDB

```
Centroids: [C1, C2, C3, ..., Ck]
Inverted lists:
  C1 → [v1, v5, v12, ...]
  C2 → [v2, v7, v8, ...]
  C3 → [v3, v4, v9, ...]
```

**Ventajas:**
- Compresión de memoria
- Búsqueda paralela

### 3. LSH (Locality-Sensitive Hashing)

**Usado por:** Research, sistemas legacy

```
Hash functions: h1, h2, ..., hk
Buckets:
  h1(v) = 5 → [v1, v3, v7]
  h2(v) = 2 → [v2, v5, v8]
```

## Métricas de Evaluación

### Recall@K

$$
\text{Recall@K} = \frac{|\text{Retrieved} \cap \text{Relevant}|}{|\text{Relevant}|}
$$

**Objetivo VantaDB:** ≥0.95 para K=10

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

## Implementación en VantaDB

### Parámetros HNSW

| Parámetro | Default | Efecto |
|-----------|---------|--------|
| `M` | 16 | Conexiones por nodo |
| `ef_construction` | 200 | Calidad de construcción |
| `ef_search` | 100 | Calidad de búsqueda |

### Trade-off Recall vs Latencia

```python
# Alta calidad (más lento)
db = VantaEmbedded("./data", config={
    "hnsw": {
        "ef_search": 500,  # Más candidatos
        "M": 32            # Más conexiones
    }
})
# Recall: 0.998, Latencia: 15ms

# Balanceado
db = VantaEmbedded("./data", config={
    "hnsw": {
        "ef_search": 100,
        "M": 16
    }
})
# Recall: 0.956, Latencia: 6ms

# Alta velocidad (menos preciso)
db = VantaEmbedded("./data", config={
    "hnsw": {
        "ef_search": 50,
        "M": 8
    }
})
# Recall: 0.890, Latencia: 3ms
```

## Benchmarks VantaDB (SIFT1M)

| Configuración | Recall@10 | p50 Latency | QPS |
|---------------|-----------|-------------|-----|
| ef_search=50 | 0.912 | 4.2ms | 238 |
| ef_search=100 | 0.956 | 6.1ms | 164 |
| ef_search=200 | 0.981 | 9.8ms | 102 |
| ef_search=500 | 0.998 | 15.4ms | 65 |

## Véase También

- [HNSW](HNSW.md) — Algoritmo ANN específico de VantaDB
- [Vector Similarity](Vector Similarity.md) — Métricas de distancia
- [Benchmarks](Benchmarks.md) — Evaluación de performance

---

*ANN permite búsquedas vectoriales sub-milisegundo en datasets de millones de vectores.*

