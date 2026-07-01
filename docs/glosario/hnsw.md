---
type: glosario-entry
status: stable
tags: [indice, ann, busqueda-vectorial, hnsw]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Hierarchical Navigable Small World, HNSW Index]
description: "Algoritmo de indexación para búsqueda aproximada de vecinos más cercanos (ANN) que construye un grafo multi-capa de vectores, permitiendo búsquedas en tiempo logarítmico"
---

# HNSW — Hierarchical Navigable Small World

## Definición

**HNSW** es un algoritmo de indexación para **búsqueda aproximada de vecinos más cercanos (ANN)** que construye un grafo multi-capa de vectores, permitiendo búsquedas en tiempo **logarítmico** ($O(\log N)$) con alto recall (>0.95).

## El Problema que Resuelve

### Búsqueda Exhaustiva (Brute Force)

```
Query: vector q
Para cada vector v_i en la base de datos:
    calcular distancia(q, v_i)
Ordenar por distancia
Retornar top-K
```

**Complejidad:** $O(N \cdot d)$ donde N = vectores, d = dimensiones

| N (vectores) | Tiempo (384d) |
|--------------|---------------|
| 1,000 | ~1 ms |
| 100,000 | ~100 ms |
| 10,000,000 | ~10 segundos ❌ |

### Solución: Búsqueda Aproximada (ANN)

HNSW encuentra vectores **cercanos al óptimo** sin comparar con todos:

| N (vectores) | Brute Force | HNSW | Speedup |
|--------------|-------------|------|---------|
| 100,000 | 100 ms | 6 ms | 16x |
| 1,000,000 | 1 segundo | 15 ms | 66x |

## Estructura del Grafo HNSW

```
Capa 2 (más dispersa):
    [A] ────────── [D]
     │              │
     └──── [C] ─────┘

Capa 1 (intermedia):
    [A] ─── [B] ─── [D]
     │       │       │
    [E] ─── [C] ─── [F]

Capa 0 (más densa, todos los vectores):
    [A]─[B]─[C]─[D]─[E]─[F]─[G]─[H]─[I]─[J]
```

### Propiedades Clave

1. **Multi-capa:** Capas superiores tienen menos nodos (navegación rápida)
2. **Small World:** Cualquier nodo es alcanzable en pocos saltos
3. **Navegable:** Búsquedas greedy convergen al óptimo local

## Algoritmo de Búsqueda

```
Input: query q, K (top-K), ef (ef search)

1. Iniciar en nodo de entrada de capa más alta
2. Para cada capa (de arriba a abajo):
     Greedy search hasta converger
3. En capa 0:
     Búsqueda con lista de candidatos (ef)
     Mantener top-K más cercanos
4. Retornar top-K
```

### Parámetros Clave

| Parámetro | Descripción | Valor Típico | Impacto |
|-----------|-------------|--------------|---------|
| **M** | Máximo de conexiones por nodo | 16-32 | Mayor M = más memoria, mejor recall |
| **ef_construction** | Candidatos durante construcción | 200-500 | Mayor = mejor grafo, más lento construir |
| **ef_search** | Candidatos durante búsqueda | 50-200 | Mayor = mejor recall, más lento buscar |

## Implementación en VantaDB

### Estructura de Datos

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
    vector: Vec<f32>,  // O mmap pointer
    neighbors: Vec<NodeId>,
}
```

### Búsqueda en VantaDB

```python
# busqueda-vectorial
results = db.search(
    vector=[0.12, -0.34, ...],  # Query vector
    top_k=10,                    # Retornar 10 más cercanos
    ef_search=100                # Parámetro de calidad
)
```

### Persistencia del Índice

VantaDB persiste el índice HNSW mediante **[mmap](mmap.md)**:

```
Disco: vector_store.vanta
├── Header (magic, version, params)
├── Layer 0 nodes (vectores + neighbors)
├── Layer 1 nodes
├── ...
└── Layer N nodes

Runtime: mmap() → acceso directo sin copiar a RAM
```

**Ventaja:** Carga instantánea (no reconstrucción desde cero).

## Métricas de Performance en VantaDB

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

**Factor de escalado:** 4.88x (sub-lineal, mejor que $O(N)$)

## Optimizaciones en VantaDB

### 1. SIMD para Distancias

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

Reducir `f32` (4 bytes) → `u8` (1 byte):
- **4x menos memoria**
- **2-4x más rápido** en búsquedas
- **~1-2% pérdida de recall**

## Comparación con Otros Algoritmos ANN

| Algoritmo | Estructura | Construcción | Búsqueda | Memoria | Caso de Uso |
|-----------|-----------|--------------|----------|---------|-------------|
| **HNSW** | Grafo multi-capa | Media | Rápida | Alta | General purpose |
| **IVF** | Particionamiento + centroids | Rápida | Media | Media | Datasets muy grandes |
| **PQ** | Compresión de vectores | Lenta | Rápida | Muy baja | Memoria limitada |
| **ScaNN** | Cuantización + reranking | Media | Muy rápida | Media | Google-scale |
| **FAISS** | Múltiples (IVF, PQ, HNSW) | Variable | Variable | Variable | Research |

### Por Qué VantaDB Elige HNSW

1. **Balance recall/latencia:** Mejor trade-off para datasets medianos
2. **Simple de implementar:** Sin training de centroids
3. **Persistencia directa:** Grafo se puede mmap-ear
4. **No requiere training:** Funciona immediately

## Trade-offs de HNSW

| Ventaja | Costo |
|---------|-------|
| Búsqueda rápida ($O(\log N)$) | Construcción lenta ($O(N \log N)$) |
| Alto recall (>0.95) | Memoria alta (~1KB/vector) |
| Simple de usar | Parámetros requieren tuning |
| Persistencia mmap | Reconstrucción costosa si se corrompe |

## Problemas Conocidos

### AUD-03: Concurrencia en Rebuild

**Severidad:** ⚠️ Alta

Si `rebuild_index()` se ejecuta concurrentemente con búsquedas, los lectores pueden ver un índice inconsistente.

**Mitigación:** [RwLock](RwLock.md) global o estrategia de doble buffer.

### AUD-04: Validación de SIMD

**Severidad:** ℹ️ Media

El fallback escalar y la ruta SIMD conviven sin tests de equivalencia numérica.

**Mitigación:** Tests property-based comparando SIMD vs escalar.

## Véase También

- [Vectores](Vectores.md) — Lo que HNSW indexa
- [Vector Similarity](Vector Similarity.md) — Métricas de distancia
- [mmap](mmap.md) — Persistencia del índice
- [BM25](BM25.md) — Índice complementario (léxico)
- [RRF](RRF.md) — Fusión de HNSW + BM25

---

*HNSW es el estándar de facto para busqueda-vectorial ANN en producción.*

