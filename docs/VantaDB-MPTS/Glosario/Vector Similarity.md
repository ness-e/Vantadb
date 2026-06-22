---
type: glosario-entry
status: stable
tags: [vectores, distancia, metricas, busqueda]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Vector Distance Metrics, Métricas de Distancia]
---

# Vector Similarity

## Definición

**Vector Similarity** se refiere a las **métricas matemáticas** usadas para medir qué tan similares son dos vectores en un espacio de alta dimensionalidad. Es la base de la búsqueda vectorial en [HNSW](HNSW.md).

## Métricas Principales

### 1. Similitud Coseno

Mide el **ángulo** entre vectores (independiente de magnitud).

$$
\cos(\theta) = \frac{\mathbf{a} \cdot \mathbf{b}}{|\mathbf{a}| \cdot |\mathbf{b}|}
$$

| Valor | Significado |
|-------|-------------|
| 1 | Vectores idénticos en dirección |
| 0 | Ortogonales (no relacionados) |
| -1 | Opuestos |

**Uso:** Embeddings de texto (sentence-transformers)

### 2. Distancia Euclidiana (L2)

Distancia **geométrica** directa.

$$
d(\mathbf{a}, \mathbf{b}) = \sqrt{\sum_{i=1}^{d} (a_i - b_i)^2}
$$

| Valor | Significado |
|-------|-------------|
| 0 | Vectores idénticos |
| Mayor | Más diferentes |

**Uso:** Embeddings de imágenes (CLIP)

### 3. Producto Punto (Dot Product)

$$
\mathbf{a} \cdot \mathbf{b} = \sum_{i=1}^{d} a_i \cdot b_i
$$

**Uso:** Vectores normalizados (equivalente a coseno)

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

## Véase También

- [Vectores](Vectores.md) — Lo que se compara
- [HNSW](HNSW.md) — Índice que usa estas métricas
- [BM25](BM25.md) — Búsqueda complementaria (léxica)

---

*La elección de métrica de similitud afecta directamente la calidad de búsqueda.*

