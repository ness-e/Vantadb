---
title: "busqueda-vectorial"
type: glossary-entry
status: stable
tags: [glosario, búsqueda, vectorial, ann, hnsw]
last_reviewed: 2026-07-03
aliases: [vector search, ANN search, approximate nearest neighbor]
---

# busqueda-vectorial

## Definición

La **busqueda-vectorial** (o búsqueda por similitud semántica) es una técnica de recuperación de información que encuentra vectores en un espacio de alta dimensionalidad que son más similares a un vector de consulta según una métrica de distancia específica.

## Fundamentos Matemáticos

### Métricas de Distancia

| Métrica | Fórmula | Uso |
|---------|---------|-----|
| **Coseno** | $1 - \frac{A \cdot B}{\|A\| \cdot \|B\|}$ | Embeddings normalizados |
| **Euclidiana (L2)** | $\sqrt{\sum(A_i - B_i)^2}$ | Vectores no normalizados |
| **Dot Product** | $A \cdot B$ | Vectores unitarios |

### Approximate Nearest Neighbor ([ANN](ANN.md))

La búsqueda exacta K-NN tiene complejidad $O(n \cdot d)$, lo cual es prohibitivo para datasets grandes. Los algoritmos ANN sacrifican precisión por velocidad:

| Algoritmo | Complejidad | Recall Típico | Uso |
|-----------|-------------|---------------|-----|
| **Brute Force** | $O(n \cdot d)$ | 100% | Baseline |
| **[HNSW](HNSW.md)** | $O(\log n \cdot d)$ | 95-99% | VantaDB default |
| **IVF-PQ** | $O(k \cdot d)$ | 90-95% | LanceDB |

## En VantaDB

### Implementación HNSW

VantaDB utiliza **Hierarchical Navigable Small World** ([HNSW](HNSW.md)) como índice vectorial principal:

```rust
// src/index.rs - CPIndex (HNSW)
pub struct CPIndex {
    graph: Vec<Vec<Vec<u64>>>,  // Niveles → Nodos → Vecinos
    vectors: VectorStore,
    config: HnswConfig,
}

impl CPIndex {
    pub fn search_nearest(
        &self,
        query: &[f32],
        k: usize,
        ef_search: usize,
    ) -> Vec<(u64, f32)> {
        // Búsqueda greedy multi-capa
        let mut candidates = BinaryHeap::new();
        let mut results = BinaryHeap::new();
        
        // Entry point desde capa superior
        let mut current = self.entry_point;
        for level in (1..self.max_level).rev() {
            current = self.greedy_search_level(query, current, level);
        }
        
        // Búsqueda exhaustiva en capa 0
        self.search_layer_0(query, current, k, ef_search)
    }
}
```

### Parámetros de Configuración

```python
db = VantaEmbedded("./data", config={
    "hnsw": {
        "M": 16,                    # Conexiones por nodo
        "ef_construction": 200,     # Candidatos en construcción
        "ef_search": 100,           # Candidatos en búsqueda
        "metric": "cosine"          # "cosine", "euclidean", "dot"
    }
})
```

### Aceleración SIMD

VantaDB utiliza instrucciones SIMD para cálculo de distancias:

| Arquitectura | Instrucciones | Speedup |
|--------------|---------------|---------|
| x86_64 | AVX2, AVX-512 | 8-16x |
| ARM | NEON | 4x |
| Fallback | Escalar | 1x |

```rust
// src/index.rs - SIMD dispatch
#[cfg(target_arch = "x86_64")]
pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
    if is_x86_feature_detected!("avx2") {
        unsafe { cosine_distance_avx2(a, b) }
    } else {
        cosine_distance_scalar(a, b)
    }
}
```

## Métricas de Evaluación

### Recall@K

Porcentaje de los K vecinos más cercanos reales que aparecen en los K resultados:

$$\text{Recall@K} = \frac{|\text{resultados} \cap \text{ground\_truth}|}{K}$$

**Objetivo VantaDB:** Recall@10 ≥ 0.95

### Latencia p50/p99

- **p50:** Mediana de latencia (50% de queries más rápidas)
- **p99:** Percentil 99 (99% de queries más rápidas)

**Métricas Certificadas (SIFT1M):**

| Dataset | Recall@10 | Latencia p50 | Memory |
|---------|-----------|--------------|--------|
| 10K vectores | 0.956 | 1.2 ms | ~12 MB |
| 50K vectores | 1.000 | 6.1 ms | ~58 MB |
| 100K vectores | 0.998 | 12.4 ms | ~117 MB |

## Véase También

- [HNSW](HNSW.md) - Algoritmo de índice vectorial
- [ANN](ANN.md) - Approximate Nearest Neighbor
- [Vector Similarity](Vector Similarity.md) - Métricas de similitud
- [Vectores](Vectores.md) - Representaciones vectoriales
- [busqueda-hibrida](busqueda-hibrida.md) - Combinación con busqueda-lexica
