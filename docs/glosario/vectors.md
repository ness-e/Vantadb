---
title: "Vectores"
type: glossary-entry
status: stable
tags: [concept, ml, embeddings, vectores, alta-dimensionalidad]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Vectors, Embeddings, High Dimensional Vectors]
description: "Array of floating point numbers representing an object (text, image, audio) in a high-dimensional space, capturing semantic similarity"
---
# Vectores

## Definition

In the context of databases and ML, a **vector** is an array of floating point numbers (typically `f32`) that represents an object (text, image, audio) in a **high-dimensional space**. Vectors capture **semantic similarity**: similar objects have nearby vectors in the vector space.

## Mathematical Structure

$$
\mathbf{v} \in \mathbb{R}^d
$$

Where $d$ is the **dimensionality** (typically 384, 768, 1024, 1536, 3072).

### Example

```
Vector de 4 dimensiones:
v = [0.12, -0.34, 0.56, 0.78]

Actual embedding vector (384d):
v = [0.021, -0.156, 0.089, ..., 0.034] #384 floats
```

## How Vectors are Generated

| Modelo | Dimensiones | Caso de Uso |
|--------|-------------|-------------|
| **OpenAI text-embedding-3-small** | 1536 | Texto general |
| **OpenAI text-embedding-3-large** | 3072 | Alta precisión |
| **sentence-transformers/all-MiniLM-L6-v2** | 384 | Texto, ligero |
| **BGE-M3** | 1024 | Multilingüe |
| **CLIP ViT-L/14** | 768 | Imágenes + texto |

### Embedding Process

```
Texto: "El gato duerme en el sofá"
        │
        ▼
[Modelo de Embedding (ej: BERT)]
        │
        ▼
Vector: [0.12, -0.34, 0.56, ..., 0.78]  (384 floats)
```

## Vector Similarity Metrics

### 1. Cosine Similarity
Measures the angle between vectors (independent of magnitude):

$$
\cos(\theta) = \frac{\mathbf{a} \cdot \mathbf{b}}{|\mathbf{a}| \cdot |\mathbf{b}|} \in [-1, 1]
$$

- **1**: Vectors identical in direction
- **0**: Orthogonal (not related)
- **-1**: Opposites

### 2. Distancia Euclidiana (L2)
Distancia geométrica directa:

$$
d(\mathbf{a}, \mathbf{b}) = \sqrt{\sum_{i=1}^{d} (a_i - b_i)^2}
$$

- **0**: Identical vectors
- **Major**: More different

### 3. Dot Product
$$
\mathbf{a} \cdot \mathbf{b} = \sum _{i=1}^{d} a_i \cdot b_i
$$

##Why it Matters in VantaDB

VantaDB is a **vector database** as well as a documentary:

### Semantic Similarity Search

```python
# Usuario pregunta: "¿Cómo configuro mi router?"
query_vector = embed("¿Cómo configuro mi router?")

# VantaDB busca documentos con vectores similares
results = db.search(
    vector=query_vector,
    top_k=5
)
# Retorna: "Guía de instalación de router", "Configuración WiFi", etc.
```

### Storage in VantaDB

```
UnifiedNode (VantaDB)
├── key: "doc_001"
├── vector: [0.12, -0.34, ..., 0.78]  # 384 floats
├── text: "El gato duerme en el sofá"
├── metadata: {"source": "web", "date": "2026-06-12"}
└── edges: [...]  # Relaciones de grafo
```

## The Vector-Search Problem

### Búsqueda Exhaustiva (Brute Force)

To find the K most similar vectors to a query:

```
Para cada vector en la base de datos:
    Calcular similitud con query
Ordenar por similitud
Retornar top-K
```

**Complejidad:** $O(N \cdot d)$ donde N = número de vectores, d = dimensiones

| Dataset | Tiempo de Búsqueda |
|---------|-------------------|
| 1,000 vectores | ~1 ms |
| 100,000 vectores | ~100 ms |
| 10,000,000 vectores | ~10 segundos ❌ |

### Solution: Approximate Search (ANN)

Algorithms like [[hnsw]] find similar vectors **without comparing them all**:

| Dataset | Brute Force | HNSW | Speedup |
|---------|-------------|------|---------|
| 100K vectores | 100 ms | 6 ms | 16x |
| 1M vectores | 1 segundo | 15 ms | 66x |
| 10M vectores | 10 segundos | 25 ms | 400x |

**Trade-off:** ANN sacrifices accuracy (recall < 100%) for speed.

## Optimizations in VantaDB

### 1. SIMD (Single Instruction, Multiple Data)

Process multiple floats in a single CPU instruction:

```rust
// Sin SIMD: 1 operación por float
for i in 0..d {
    sum += a[i] * b[i];
}

// With SIMD (AVX2): 8 floats per instruction
unsafe {
    let va = _mm256_loadu_ps(a.as_ptr());
    let vb = _mm256_loadu_ps(b.as_ptr());
    let vprod = _mm256_mul_ps(va, vb);
    //...
}
```

**Speedup:** 4-8x en distancias vectoriales.

### 2. Quantization (SQ8)

Reduce precision from `f32` (4 bytes) to `u8` (1 byte):

```
Original:  [0.123456, -0.789012, 0.456789, ...]  # 4 bytes/float
Cuantizado: [31, 178, 116, ...]                    # 1 byte/float
```

**Benefit:** 4x less memory, 2-4x faster searches.

**Cost:** ~1-2% recall loss.

### 3. mmap (Memory-Mapped I/O)

Load vectors from disk without copying to RAM:

```
Disco: vector_store.vanta (10 GB)
        │
        ▼ mmap
Memoria Virtual: [punteros a disco]
        │
        ▼ page fault (solo cuando se accede)
RAM: Solo las páginas accedidas
```

**Benefit:** Dataset > RAM possible.

## See Also

- [[hnsw]] — Índice ANN para busqueda-vectorial eficiente
- [[vector-similarity]] — Métricas de distancia
- [[mmap]] — Para datasets que exceden RAM
- [[bm25]] — busqueda-lexica complementaria
- [[rrf]] — Fusión de busqueda-vectorial + léxica

---

*Vectors are the bridge between unstructured data (text, images) and mathematical similarity operations.*

