---
type: glosario-entry
status: stable
tags: [concepto, ml, embeddings, vectores, alta-dimensionalidad]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Vectors, Embeddings, Vectores de Alta Dimensionalidad]
description: "Array de números de punto flotante que representa un objeto (texto, imagen, audio) en un espacio de alta dimensionalidad, capturando similitud semántica"
---

# Vectores

## Definición

En el contexto de bases de datos y ML, un **vector** es un array de números de punto flotante (típicamente `f32`) que representa un objeto (texto, imagen, audio) en un **espacio de alta dimensionalidad**. Los vectores capturan **similitud semántica**: objetos similares tienen vectores cercanos en el espacio vectorial.

## Estructura Matemática

$$
\mathbf{v} \in \mathbb{R}^d
$$

Donde $d$ es la **dimensionalidad** (típicamente 384, 768, 1024, 1536, 3072).

### Ejemplo

```
Vector de 4 dimensiones:
v = [0.12, -0.34, 0.56, 0.78]

Vector de embedding real (384d):
v = [0.021, -0.156, 0.089, ..., 0.034]  # 384 floats
```

## Cómo se Generan los Vectores

| Modelo | Dimensiones | Caso de Uso |
|--------|-------------|-------------|
| **OpenAI text-embedding-3-small** | 1536 | Texto general |
| **OpenAI text-embedding-3-large** | 3072 | Alta precisión |
| **sentence-transformers/all-MiniLM-L6-v2** | 384 | Texto, ligero |
| **BGE-M3** | 1024 | Multilingüe |
| **CLIP ViT-L/14** | 768 | Imágenes + texto |

### Proceso de Embedding

```
Texto: "El gato duerme en el sofá"
        │
        ▼
[Modelo de Embedding (ej: BERT)]
        │
        ▼
Vector: [0.12, -0.34, 0.56, ..., 0.78]  (384 floats)
```

## Métricas de Similitud Vectorial

### 1. Similitud Coseno
Mide el ángulo entre vectores (independiente de magnitud):

$$
\cos(\theta) = \frac{\mathbf{a} \cdot \mathbf{b}}{|\mathbf{a}| \cdot |\mathbf{b}|} \in [-1, 1]
$$

- **1**: Vectores idénticos en dirección
- **0**: Ortogonales (no relacionados)
- **-1**: Opuestos

### 2. Distancia Euclidiana (L2)
Distancia geométrica directa:

$$
d(\mathbf{a}, \mathbf{b}) = \sqrt{\sum_{i=1}^{d} (a_i - b_i)^2}
$$

- **0**: Vectores idénticos
- **Mayor**: Más diferentes

### 3. Producto Punto (Dot Product)
$$
\mathbf{a} \cdot \mathbf{b} = \sum_{i=1}^{d} a_i \cdot b_i
$$

## Por Qué Importa en VantaDB

VantaDB es una **base de datos vectorial** además de documental:

### Búsqueda por Similitud Semántica

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

### Almacenamiento en VantaDB

```
UnifiedNode (VantaDB)
├── key: "doc_001"
├── vector: [0.12, -0.34, ..., 0.78]  # 384 floats
├── text: "El gato duerme en el sofá"
├── metadata: {"source": "web", "date": "2026-06-12"}
└── edges: [...]  # Relaciones de grafo
```

## El Problema de la busqueda-vectorial

### Búsqueda Exhaustiva (Brute Force)

Para encontrar los K vectores más similares a una query:

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

### Solución: Búsqueda Aproximada (ANN)

Algoritmos como [HNSW](HNSW.md) encuentran vectores similares **sin comparar con todos**:

| Dataset | Brute Force | HNSW | Speedup |
|---------|-------------|------|---------|
| 100K vectores | 100 ms | 6 ms | 16x |
| 1M vectores | 1 segundo | 15 ms | 66x |
| 10M vectores | 10 segundos | 25 ms | 400x |

**Trade-off:** ANN sacrifica exactitud (recall < 100%) por velocidad.

## Optimizaciones en VantaDB

### 1. SIMD (Single Instruction, Multiple Data)

Procesar múltiples floats en una sola instrucción CPU:

```rust
// Sin SIMD: 1 operación por float
for i in 0..d {
    sum += a[i] * b[i];
}

// Con SIMD (AVX2): 8 floats por instrucción
unsafe {
    let va = _mm256_loadu_ps(a.as_ptr());
    let vb = _mm256_loadu_ps(b.as_ptr());
    let vprod = _mm256_mul_ps(va, vb);
    // ...
}
```

**Speedup:** 4-8x en distancias vectoriales.

### 2. Cuantización (SQ8)

Reducir precisión de `f32` (4 bytes) a `u8` (1 byte):

```
Original:  [0.123456, -0.789012, 0.456789, ...]  # 4 bytes/float
Cuantizado: [31, 178, 116, ...]                    # 1 byte/float
```

**Beneficio:** 4x menos memoria, 2-4x más rápido en búsquedas.

**Costo:** ~1-2% pérdida de recall.

### 3. mmap (Memory-Mapped I/O)

Cargar vectores desde disco sin copiar a RAM:

```
Disco: vector_store.vanta (10 GB)
        │
        ▼ mmap
Memoria Virtual: [punteros a disco]
        │
        ▼ page fault (solo cuando se accede)
RAM: Solo las páginas accedidas
```

**Beneficio:** Dataset > RAM posible.

## Véase También

- [HNSW](HNSW.md) — Índice ANN para busqueda-vectorial eficiente
- [Vector Similarity](Vector Similarity.md) — Métricas de distancia
- [mmap](mmap.md) — Para datasets que exceden RAM
- [BM25](BM25.md) — busqueda-lexica complementaria
- [RRF](RRF.md) — Fusión de busqueda-vectorial + léxica

---

*Los vectores son el puente entre datos no estructurados (texto, imágenes) y operaciones matemáticas de similitud.*

