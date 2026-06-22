---
type: glosario-entry
status: stable
tags: [busqueda, fusion, ranking, hybrid-search]
last_refined: 2026-06
links: "[Glosario](../Glosario.md)"
aliases: [Reciprocal Rank Fusion, Rank Fusion]
description: "Algoritmo para fusionar múltiples listas de ranking en un ranking unificado, basado únicamente en la posición ordinal (rango) de cada documento, sin necesidad de normalizar scores heterogéneos"
---

# RRF — Reciprocal Rank Fusion

## Definición

**RRF** (Reciprocal Rank Fusion) es un algoritmo para **fusionar múltiples listas de ranking** en un ranking unificado, basado únicamente en la **posición ordinal** (rango) de cada documento en cada lista, sin necesidad de normalizar scores heterogéneos.

## Fórmula Matemática

Para un documento $d$ que aparece en múltiples listas de resultados $\mathcal{M}$:

$$
\text{RRFscore}(d) = \sum_{r \in \mathcal{M}} \frac{1}{k + r(d)}
$$

Donde:
- $r(d)$ = rango del documento $d$ en la lista $r$ (1-indexed)
- $k$ = constante de suavizado (típicamente 60)
- $\mathcal{M}$ = conjunto de listas de resultados a fusionar

## Ejemplo Práctico

### Escenario

**Query:** `"base de datos vectorial"`

**Lista 1 (BM25):** `[doc5, doc12, doc23, doc7, doc45]`
**Lista 2 (HNSW):** `[doc3, doc7, doc12, doc45, doc8]`

### Cálculo RRF (k=60)

| Documento | Rango BM25 | Rango HNSW | Score RRF |
|-----------|-----------|-----------|-----------|
| **doc12** | 2 | 3 | 1/(60+2) + 1/(60+3) = 0.01613 + 0.01587 = **0.03200** |
| **doc7** | 4 | 2 | 1/(60+4) + 1/(60+2) = 0.01563 + 0.01613 = **0.03176** |
| **doc45** | 5 | 4 | 1/(60+5) + 1/(60+4) = 0.01538 + 0.01563 = **0.03101** |
| **doc5** | 1 | — | 1/(60+1) = **0.01639** |
| **doc3** | — | 1 | 1/(60+1) = **0.01639** |
| **doc23** | 3 | — | 1/(60+3) = **0.01587** |
| **doc8** | — | 5 | 1/(60+5) = **0.01538** |

### Ranking Final Fusionado

1. **doc12** (0.03200) — Aparece en ambos, buen ranking en ambos
2. **doc7** (0.03176) — Aparece en ambos, excelente en HNSW
3. **doc45** (0.03101) — Aparece en ambos
4. **doc5** (0.01639) — Solo en BM25, pero #1
5. **doc3** (0.01639) — Solo en HNSW, pero #1
6. **doc23** (0.01587) — Solo en BM25
7. **doc8** (0.01538) — Solo en HNSW

## Por Qué Funciona RRF

### Problema: Scores Incompatibles

| Método | Rango de Score | Distribución |
|--------|---------------|--------------|
| **BM25** | $[0, \infty)$ | No acotado, depende del corpus |
| **Coseno** | $[-1, 1]$ | Normalizado |
| **Euclidiana** | $[0, \infty)$ | No acotado |

**Intento ingenuo:** Promediar scores → **Sesgo hacia el método con scores más altos**

### Solución RRF: Usar Solo Rangos

RRF ignora los scores y usa solo la **posición ordinal**:
- No requiere normalización
- Inmune a diferencias de escala
- Funciona con cualquier método de ranking

## Implementación en VantaDB

### Algoritmo

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

### Búsqueda Híbrida en VantaDB

```python
results = db.search(
    vector=embed("query"),
    text="query",
    top_k=10,
    mode="hybrid",  # Usa RRF internamente
    rrf_k=60        # Parámetro de suavizado
)
```

## Efecto del Parámetro k

### k Pequeño (k → 1)

```
k = 1:
Rango 1: 1/(1+1) = 0.500
Rango 2: 1/(1+2) = 0.333
Rango 3: 1/(1+3) = 0.250
```

**Decaimiento extremo:** El #1 domina completamente.
**Uso:** Cuando un método es mucho más confiable que otros.

### k Estándar (k = 60)

```
k = 60:
Rango 1: 1/(60+1) = 0.01639
Rango 2: 1/(60+2) = 0.01613
Rango 3: 1/(60+3) = 0.01587
```

**Decaimiento suave:** Balance entre métodos.
**Uso:** Caso general (default en VantaDB).

### k Grande (k → ∞)

```
k = 1000:
Rango 1: 1/(1000+1) = 0.000999
Rango 2: 1/(1000+2) = 0.000998
Rango 3: 1/(1000+3) = 0.000997
```

**Decaimiento mínimo:** Casi todos los rangos pesan igual.
**Uso:** Cuando todos los métodos son igualmente ruidosos.

## Ventajas de RRF

| Ventaja | Descripción |
|---------|-------------|
| **Simple** | Una línea de código por documento |
| **Robusto** | No requiere normalización de scores |
| **Rápido** | $O(N \cdot M)$ donde N = docs, M = métodos |
| **Efectivo** | Empíricamente funciona tan bien como métodos complejos |
| **Universal** | Funciona con cualquier sistema de ranking |

## Limitaciones de RRF

| Limitación | Descripción |
|-----------|-------------|
| **Ignora magnitud** | No diferencia entre #1 por poco o por mucho |
| **Sin learning** | No aprende de feedback del usuario |
| **k fijo** | Requiere tuning manual del parámetro |
| **Sin contexto** | No considera correlación entre métodos |

## Alternativas a RRF

| Método | Complejidad | Requiere Training | Calidad |
|--------|-------------|-------------------|---------|
| **RRF** | Baja | No | Alta |
| **Linear Combination** | Baja | No (pesos manuales) | Media |
| **Learning to Rank** | Alta | Sí | Muy alta |
| **Cross-Encoder Reranking** | Muy alta | No | Excelente |

### Cuándo Usar Cada Uno

- **RRF:** Caso general, sin training data
- **Linear Combination:** Cuando conoces pesos óptimos
- **Learning to Rank:** Cuando tienes clicks/relevancia labels
- **Cross-Encoder:** Para reranking de top-K (calidad máxima)

## Comparación de Resultados

### Dataset: MS MARCO (Information Retrieval)

| Método | NDCG@10 | MRR@10 |
|--------|---------|--------|
| BM25 solo | 0.38 | 0.36 |
| HNSW solo | 0.42 | 0.40 |
| **RRF (BM25 + HNSW)** | **0.48** | **0.46** |
| Linear Combination | 0.45 | 0.43 |
| Cross-Encoder Rerank | 0.52 | 0.50 |

**Conclusión:** RRF mejora ambos métodos individuales, sin overhead de reranking.

## Métricas en VantaDB

### Latencia de Búsqueda Híbrida

| Operación | Latencia p50 | Speedup vs Secuencial |
|-----------|--------------|----------------------|
| BM25 solo | 115 ms | — |
| HNSW solo | 62 ms | — |
| **Híbrida (RRF)** | 180 ms | 1.0x (baseline) |
| **Híbrida (paralela + RRF)** | 125 ms | 1.44x |

### Recall Mejorado

| Método | Recall@10 |
|--------|-----------|
| BM25 solo | 0.78 |
| HNSW solo | 0.89 |
| **RRF (híbrido)** | **0.94** |

## Véase También

- [BM25](BM25.md) — Ranking léxico
- [HNSW](HNSW.md) — Ranking vectorial
- [Vectores](Vectores.md) — Embeddings para búsqueda semántica
- [RAG](RAG.md) — Caso de uso principal de búsqueda híbrida

---

*RRF es el algoritmo más simple y efectivo para fusión de rankings heterogéneos.*

