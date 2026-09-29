---
title: vector-similarity
kind: glossary
status: stable
description: Vector Similarity refers to the mathematical metrics used to measure how similar two vectors are in a high-dimensional space. It is the basis of vector-search in hnsw
aliases: [Vector Distance Metrics]
tags: [vectores, distancia, metricas, busqueda]
links: "[[README.md]]"
---

# VectorSimilarity

## Definition

**Vector Similarity** refers to the **mathematical metrics** used to measure how similar two vectors are in a high-dimensional space. It is the basis of vector-search in [hnsw](./hnsw.md).

## Main Metrics

### 1. Cosine Similarity

Measures the **angle** between vectors (independent of magnitude).

$$
\cos(\theta) = \frac{\mathbf{a} \cdot \mathbf{b}}{|\mathbf{a}| \cdot |\mathbf{b}|}
$$

| Valor | Significado |
|-------|-------------|
| 1 | Vectores idénticos en dirección |
| 0 | Ortogonales (no relacionados) |
| -1 | Opuestos |

**Use:** Text embeddings (sentence-transformers)

### 2. Euclidean Distance (L2)

Direct **geometric** distance.

$$
d(\mathbf{a}, \mathbf{b}) = \sqrt{\sum_{i=1}^{d} (a_i - b_i)^2}
$$

| Valor | Significado |
|-------|-------------|
| 0 | Vectores idénticos |
| Mayor | Más diferentes |

**Uso:** Embeddings de imágenes (CLIP)

### 3. Dot Product

$$
\mathbf{a} \cdot \mathbf{b} = \sum_{i=1}^{d} a_i \cdot b_i
$$

**Use:** Normalized vectors (equivalent to cosine)

## Implementación en VantaDB

```rust
pub enum DistanceMetric {
    Cosine,
    Euclidean,
    DotProduct,
}

pub fn distance(a: &[f32], b: &[f32], metric: DistanceMetric) -> f32 {
    match metric {
        DistanceMetric::Cosine => cosine_distance(a, b),
        DistanceMetric::Euclidean => euclidean_distance(a, b),
        DistanceMetric::DotProduct => dot_product(a, b),
    }
}
```

### Optimización SIMD

```rust
#[cfg(target_arch = "x86_64")]
unsafe fn euclidean_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    // 8 floats por instrucción (AVX2)
    // Speedup: 4-8x vs escalar
}
```

## Comparación de Métricas

| Métrica | Rango | Invariante a | Caso de Uso |
|---------|-------|--------------|-------------|
| **Coseno** | [-1, 1] | Magnitud | Texto, semántica |
| **Euclidiana** | [0, ∞) | Nada | Imágenes, geometría |
| **Dot Product** | (-∞, ∞) | Nada | Vectores normalizados |

## See Also

- [Vectors](./vectors.md) — What is compared
- [hnsw](./hnsw.md) — Index using these metrics
- [bm25](./bm25.md) — Supplementary search (lexical)

---

*The choice of similarity metric directly affects search quality.*

