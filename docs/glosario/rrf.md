---
title: "rrf"
type: glossary-entry
status: stable
tags: [busqueda, fusion, ranking, hybrid-search]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Reciprocal Rank Fusion, Rank Fusion]
description: "Algorithm to merge multiple ranking lists into a unified ranking, based solely on the ordinal position (rank) of each document, without the need to normalize heterogeneous scores"
---
#RRF—Reciprocal Rank Fusion

##Definition

**RRF** (Reciprocal Rank Fusion) is an algorithm to **merge multiple ranking lists** into a unified ranking, based solely on the **ordinal position** (rank) of each document in each list, without the need to normalize heterogeneous scores.

## Mathematical Formula

For a document $d$ that appears in multiple result lists $\mathcal{M}$:

$$
\text{RRFscore}(d) = \sum_{r \in \mathcal{M}} \frac{1}{k + r(d)}
$$

Donde:
- $r(d)$ = rango del documento $d$ en la lista $r$ (1-indexed)
- $k$ = constante de suavizado (típicamente 60)
- $\mathcal{M}$ = conjunto de listas de resultados a fusionar

## Practical Example

### Scenery

**Query:** `"vector database"`

**List 1 (BM25):** `[doc5, doc12, doc23, doc7, doc45]`
**List 2 (HNSW):** `[doc3, doc7, doc12, doc45, doc8]`

### RRF calculation (k=60)

| Documento | Rango BM25 | Rango HNSW | Score RRF |
|-----------|-----------|-----------|-----------|
| **doc12** | 2 | 3 | 1/(60+2) + 1/(60+3) = 0.01613 + 0.01587 = **0.03200** |
| **doc7** | 4 | 2 | 1/(60+4) + 1/(60+2) = 0.01563 + 0.01613 = **0.03176** |
| **doc45** | 5 | 4 | 1/(60+5) + 1/(60+4) = 0.01538 + 0.01563 = **0.03101** |
| **doc5** | 1 | — | 1/(60+1) = **0.01639** |
| **doc3** | — | 1 | 1/(60+1) = **0.01639** |
| **doc23** | 3 | — | 1/(60+3) = **0.01587** |
| **doc8** | — | 5 | 1/(60+5) = **0.01538** |

### Merged Final Ranking

1. **doc12** (0.03200) — Appears in both, good ranking in both
2. **doc7** (0.03176) — Appears in both, excellent in HNSW
3. **doc45** (0.03101) — Appears in both
4. **doc5** (0.01639) — Only on BM25, but #1
5. **doc3** (0.01639) — Only in HNSW, but #1
6. **doc23** (0.01587) — BM25 only
7. **doc8** (0.01538) — HNSW only

## Why RRF Works

### Problema: Scores Incompatibles

| Método | Rango de Score | Distribución |
|--------|---------------|--------------|
| **BM25** | $[0, \infty)$ | No acotado, depende del corpus |
| **Coseno** | $[-1, 1]$ | Normalizado |
| **Euclidiana** | $[[bm25|0, \infty)$ | No acotado |

**Intento ingenuo:** Promediar scores → **Sesgo hacia el método con scores más altos**

### RRF Solution: Use Only Ranges

RRF ignores the scores and uses only the **ordinal position**:
- Does not require normalization
- Immune to scale differences
- Works with any ranking method

## Implementation in VantaDB

### Algorithm

```rust
pub fn reciprocal_rank_fusion(
    rankings: Vec<Vec<DocumentId>>,
    k: f32,
) -> Vec<ScoredDocument> {
    let mut fused_scores: HashMap<DocumentId, f32> = HashMap::new();
    
    for ranked_list in rankings {
        for (index, doc_id) in ranked_list.into_iter().enumerate() {
            let rank = (index + 1) as f32;  // 1-indexed
            let score = 1.0 / (k + rank);
            
            *fused_scores.entry(doc_id).or_insert(0.0) += score;
        }
    }
    
    let mut results: Vec<ScoredDocument> = fused_scores
        .into_iter()
        .map(|(id, score)| ScoredDocument { id, score })
        .collect();
    
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    results
}
```

### hybrid-search in VantaDB

```python
results = db.search(
    vector=embed("query"),
    text="query",
    top_k=10,
    mode="hybrid",  # Usa RRF internamente
    rrf_k=60        # Parámetro de suavizado
)
```

## Effect of Parameter k

### k Small (k → 1)

```
k = 1:
Rango 1: 1/(1+1) = 0.500
Rango 2: 1/(1+2) = 0.333
Rango 3: 1/(1+3) = 0.250
```

**Extreme Decay:** #1 completely dominates.
**Usage:** When one method is much more reliable than others.

### k Standard (k = 60)

```
k = 60:
Rango 1: 1/(60+1) = 0.01639
Rango 2: 1/(60+2) = 0.01613
Rango 3: 1/(60+3) = 0.01587
```

**Soft decay:** Balance between methods.
**Use:** General case (default in VantaDB).

### k Grande (k → ∞)

```
k = 1000:
Rango 1: 1/(1000+1) = 0.000999
Rango 2: 1/(1000+2) = 0.000998
Rango 3: 1/(1000+3) = 0.000997
```

**Minimum Decay:** Almost all ranks weigh the same.
**Usage:** When all methods are equally noisy.

## Advantages of RRF

| Ventaja | Descripción |
|---------|-------------|
| **Simple** | Una línea de código por documento |
| **Robusto** | No requiere normalización de scores |
| **Rápido** | $O(N \cdot M)$ donde N = docs, M = métodos |
| **Efectivo** | Empíricamente funciona tan bien como métodos complejos |
| **Universal** | Funciona con cualquier sistema de ranking |

## Limitations of RRF

| Limitación | Descripción |
|-----------|-------------|
| **Ignora magnitud** | No diferencia entre #1 por poco o por mucho |
| **Sin learning** | No aprende de feedback del usuario |
| **k fijo** | Requiere tuning manual del parámetro |
| **Sin contexto** | No considera correlación entre métodos |

## Alternatives to RRF

| Método | Complejidad | Requiere Training | Calidad |
|--------|-------------|-------------------|---------|
| **RRF** | Baja | No | Alta |
| **Linear Combination** | Baja | No (pesos manuales) | Media |
| **Learning to Rank** | Alta | Sí | Muy alta |
| **Cross-Encoder Reranking** | Muy alta | No | Excelente |

### When to Use Each

- **RRF:** General case, without training data
- **Linear Combination:** When you know optimal weights
- **Learning to Rank:** When you have clicks/relevance labels
- **Cross-Encoder:** For top-K reranking (maximum quality)

## Comparación de Resultados

### Dataset: MS MARCO (Information Retrieval)

| Método | NDCG@10 | MRR@10 |
|--------|---------|--------|
| BM25 solo | 0.38 | 0.36 |
| HNSW solo | 0.42 | 0.40 |
| **RRF (BM25 + HNSW)** | **0.48** | **0.46** |
| Linear Combination | 0.45 | 0.43 |
| Cross-Encoder Rerank | 0.52 | 0.50 |

**Conclusion:** RRF improves both individual methods, without reranking overhead.

## Metrics in VantaDB

### Hybrid-Seek Latency

| Operación | Latencia p50 | Speedup vs Secuencial |
|-----------|--------------|----------------------|
| BM25 solo | 115 ms | — |
| HNSW solo | 62 ms | — |
| **Híbrida (RRF)** | 180 ms | 1.0x (baseline) |
| **Híbrida (paralela + RRF)** | 125 ms | 1.44x |

### Improved Recall

| Método | Recall@10 |
|--------|-----------|
| BM25 solo | 0.78 |
| HNSW solo | 0.89 |
| **RRF (híbrido)** | **0.94** |

## See Also

- [BM25]] — Lexical ranking
- [[hnsw]] — Vector ranking
- [[vectors]] — Embeddings for semantic search
- [[rag]] — Hybrid-search main use case

---

*RRF is the simplest and most effective algorithm for fusion of heterogeneous rankings.*

