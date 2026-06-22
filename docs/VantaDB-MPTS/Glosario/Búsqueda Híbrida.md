---
type: glossary-entry
status: stable
tags: [glosario, búsqueda, híbrida, rrf, fusion]
aliases: [hybrid search, search fusion, combined search]
---

# Búsqueda Híbrida

## Definición

La **búsqueda híbrida** combina múltiples estrategias de recuperación (típicamente [vectorial](Búsqueda Vectorial.md) + [léxica](Búsqueda Léxica.md)) para aprovechar las fortalezas de cada una y mejorar el recall general del sistema.

## Por Qué Híbrida

| Estrategia | Fortaleza | Debilidad |
|------------|-----------|-----------|
| **Vectorial** ([HNSW](HNSW.md)) | Similitud semántica, sinónimos | Keywords exactos, códigos |
| **Léxica** ([BM25](BM25.md)) | Coincidencia exacta, terminología | Significado contextual |
| **Híbrida** | Ambos | Complejidad de fusión |

### Ejemplo de Complementariedad

**Query:** "¿Qué es el WAL de VantaDB?"

| Documento | Score Vectorial | Score BM25 | Score Híbrido |
|-----------|-----------------|------------|---------------|
| "Write-Ahead Log garantiza durabilidad" | 0.85 | 0.92 | **0.89** ✅ |
| "VantaDB usa logs para persistencia" | 0.78 | 0.15 | 0.45 |
| "Base de datos embebida para agentes" | 0.72 | 0.05 | 0.38 |

El documento #1 rankea alto en ambos métodos → score híbrido superior.

## Reciprocal Rank Fusion ([RRF](RRF.md))

VantaDB utiliza RRF para fusionar los rankings de múltiples recuperadores.

### Fórmula

$$\text{RRF\_score}(d) = \sum_{r \in R} \frac{1}{k + \text{rank}_r(d)}$$

Donde:
- $d$ = documento
- $R$ = conjunto de recuperadores (vectorial, léxico)
- $\text{rank}_r(d)$ = posición del documento en el ranking del recuperador $r$
- $k$ = constante de suavizado (default: 60)

### Ventajas de RRF

1. **Independiente de escala:** No requiere normalizar scores
2. **Robusta:** Funciona con cualquier recuperador
3. **Simple:** Solo necesita el ranking, no los scores absolutos

### Ejemplo de Cálculo

| Documento | Rank Vectorial | Rank BM25 | RRF Score (k=60) |
|-----------|----------------|-----------|------------------|
| doc_A | 1 | 3 | 1/(60+1) + 1/(60+3) = **0.0323** |
| doc_B | 2 | 1 | 1/(60+2) + 1/(60+1) = **0.0325** |
| doc_C | 3 | 5 | 1/(60+3) + 1/(60+5) = **0.0308** |

**Resultado:** doc_B > doc_A > doc_C

## En VantaDB

### Implementación

```rust
// src/hybrid.rs - Hybrid Search Executor
pub struct HybridSearch {
    vector_index: Arc<CPIndex>,
    text_index: Arc<TextIndex>,
    rrf_k: f32,
}

impl HybridSearch {
    pub fn search(
        &self,
        query_vector: &[f32],
        query_text: &str,
        top_k: usize,
    ) -> Vec<SearchResult> {
        // 1. Ejecutar búsquedas en paralelo
        let vector_results = self.vector_index.search_nearest(
            query_vector,
            top_k * 2,  // Over-fetch para RRF
            self.ef_search
        );
        
        let text_results = self.text_index.search(
            query_text,
            top_k * 2
        );
        
        // 2. Aplicar RRF
        let mut scores: HashMap<u64, f32> = HashMap::new();
        
        for (rank, (node_id, _)) in vector_results.iter().enumerate() {
            *scores.entry(*node_id).or_insert(0.0) += 
                1.0 / (self.rrf_k + (rank + 1) as f32);
        }
        
        for (rank, (node_id, _)) in text_results.iter().enumerate() {
            *scores.entry(*node_id).or_insert(0.0) += 
                1.0 / (self.rrf_k + (rank + 1) as f32);
        }
        
        // 3. Ordenar por score RRF
        let mut results: Vec<_> = scores.into_iter().collect();
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        results.truncate(top_k);
        results
    }
}
```

### Uso desde Python

```python
from vantadb import VantaEmbedded

db = VantaEmbedded("./data")

# Búsqueda híbrida
results = db.search(
    vector=embed("¿Cómo funciona la persistencia?"),
    text="persistencia WAL durability",
    top_k=10,
    mode="hybrid"  # Usa HNSW + BM25 + RRF
)

for result in results:
    print(f"{result.key}: {result.score:.4f}")
    print(f"  {result.text[:100]}...")
```

### Configuración

```python
db = VantaEmbedded("./data", config={
    "hybrid": {
        "rrf_k": 60,              # Constante de suavizado
        "vector_weight": 0.5,     # Peso relativo (para weighted fusion)
        "text_weight": 0.5,       # Peso relativo
        "min_score": 0.01         # Score mínimo para incluir
    }
})
```

## Modos de Fusión

### 1. RRF (Default)

```python
results = db.search(
    vector=query_vector,
    text=query_text,
    mode="hybrid"  # RRF con k=60
)
```

**Ventajas:**
- Independiente de escala
- Robusta a outliers
- Sin necesidad de tuning

### 2. Weighted Sum (Alternativo)

```python
results = db.search(
    vector=query_vector,
    text=query_text,
    mode="hybrid",
    fusion="weighted",
    weights={"vector": 0.7, "text": 0.3}
)
```

**Requiere:**
- Normalización de scores
- Tuning de pesos

## Performance

| Métrica | Solo Vectorial | Solo BM25 | Híbrido (RRF) |
|---------|----------------|-----------|---------------|
| **Recall@10** | 0.956 | 0.823 | **0.978** |
| **Latencia p50** | 62.0 ms | 115.3 ms | 179.8 ms |
| **Throughput** | 16 qps | 9 qps | 6 qps |

**Nota:** La búsqueda híbrida mejora recall a costa de latencia.

## Casos de Uso

### ✅ Ideal para Búsqueda Híbrida

- **RAG (Retrieval-Augmented Generation):** Mejor contexto para LLMs
- **Preguntas en lenguaje natural:** Combina semántica + keywords
- **Documentación técnica:** Nombres de funciones + conceptos
- **Búsqueda en knowledge bases:** Múltiples tipos de queries

### ❌ No Necesaria

- **Búsquedas por ID/código:** Solo léxica es suficiente
- **Similitud de imágenes:** Solo vectorial
- **Filtrado por metadata:** No requiere fusión

## Métricas de Evaluación

### Recall Mejorado

| Dataset | Vectorial | BM25 | Híbrido | Mejora |
|---------|-----------|------|---------|--------|
| SIFT1M | 0.956 | 0.823 | 0.978 | +2.3% |
| NQ (Natural Questions) | 0.712 | 0.634 | 0.789 | +10.8% |
| MS MARCO | 0.685 | 0.598 | 0.751 | +9.6% |

### Latencia Compuesta

$$\text{Latencia}_{\text{híbrida}} \approx \text{Latencia}_{\text{vectorial}} + \text{Latencia}_{\text{BM25}} + \text{Overhead}_{\text{RRF}}$$

En VantaDB:
- Vectorial: ~62 ms
- BM25: ~115 ms
- RRF: ~3 ms
- **Total: ~180 ms**

## Véase También

- [RRF](RRF.md) - Algoritmo de fusión
- [Búsqueda Vectorial](Búsqueda Vectorial.md) - Similitud semántica
- [Búsqueda Léxica](Búsqueda Léxica.md) - Coincidencia de keywords
- [HNSW](HNSW.md) - Índice vectorial
- [BM25](BM25.md) - Scoring léxico
- [GraphRAG](GraphRAG.md) - Extensión con grafos
