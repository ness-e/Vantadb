---
type: glossary-entry
status: stable
tags: [glosario, métricas, recall, evaluación, ann]
aliases: [recall@K, recall rate, true positive rate]
---

# Recall

## Definición

El **recall** (también llamado sensibilidad o tasa de verdaderos positivos) es una métrica que mide la proporción de resultados relevantes que fueron recuperados exitosamente por el sistema.

## Fórmula

$$\text{Recall@K} = \frac{|\text{Resultados recuperados} \cap \text{Ground Truth}|}{K}$$

Donde:
- $K$ = número de resultados solicitados
- **Ground Truth** = los $K$ vecinos más cercanos reales (calculados con búsqueda exacta)

## Interpretación

| Recall | Interpretación |
|--------|----------------|
| **1.0 (100%)** | Perfecto: todos los K vecinos reales fueron recuperados |
| **0.95 (95%)** | Excelente: 95% de los vecinos reales recuperados |
| **0.90 (90%)** | Bueno: aceptable para la mayoría de aplicaciones |
| **< 0.80** | Deficiente: muchos resultados relevantes perdidos |

## Ejemplo

**Query:** Vector de búsqueda
**K:** 10 (solicitamos 10 resultados)

| Posición | Resultado Real | Recuperado | Match |
|----------|----------------|------------|-------|
| 1 | doc_42 | doc_42 | ✅ |
| 2 | doc_17 | doc_89 | ❌ |
| 3 | doc_89 | doc_17 | ✅ |
| 4 | doc_33 | doc_33 | ✅ |
| 5 | doc_56 | doc_56 | ✅ |
| 6 | doc_71 | doc_71 | ✅ |
| 7 | doc_28 | doc_28 | ✅ |
| 8 | doc_94 | doc_15 | ❌ |
| 9 | doc_15 | doc_94 | ✅ |
| 10 | doc_62 | doc_62 | ✅ |

**Cálculo:**
$$\text{Recall@10} = \frac{9}{10} = 0.90$$

## En VantaDB

### Métricas Certificadas

| Dataset | Recall@10 | Latencia p50 |
|---------|-----------|--------------|
| 10K vectores, 128d | 0.956 | 1.2 ms |
| 50K vectores, 128d | 1.000 | 6.1 ms |
| 100K vectores, 128d | 0.998 | 12.4 ms |

**Objetivo:** Recall@10 ≥ 0.95 para todos los datasets

### Validación con SIFT1M

```rust
// tests/certification/stress_protocol.rs
#[test]
fn test_hnsw_recall_sift1m() {
    let (vectors, queries, ground_truth) = load_sift1m()?;
    
    let mut total_recall = 0.0;
    for (query, gt) in queries.iter().zip(ground_truth.iter()) {
        let results = db.search(query, 10, SearchMode::Vector)?;
        let retrieved: HashSet<_> = results.iter().map(|r| r.key).collect();
        let relevant: HashSet<_> = gt.iter().cloned().collect();
        
        let recall = retrieved.intersection(&relevant).count() as f32 / 10.0;
        total_recall += recall;
    }
    
    let avg_recall = total_recall / queries.len() as f32;
    assert!(avg_recall >= 0.95, "Recall@10 debe ser >= 0.95");
}
```

## Factores que Afectan el Recall

### Parámetros [HNSW](HNSW.md)

| Parámetro | Efecto en Recall | Trade-off |
|-----------|------------------|-----------|
| `M` (conexiones) | ↑ M → ↑ Recall | ↑ Memoria |
| `ef_construction` | ↑ ef_c → ↑ Recall | ↑ Tiempo de construcción |
| `ef_search` | ↑ ef_s → ↑ Recall | ↑ Latencia |

### Configuración Recomendada

```python
# Alta precisión (recall > 0.98)
db = VantaEmbedded("./data", config={
    "hnsw": {
        "M": 32,
        "ef_construction": 400,
        "ef_search": 200
    }
})

# Balanceado (recall ~0.95)
db = VantaEmbedded("./data", config={
    "hnsw": {
        "M": 16,
        "ef_construction": 200,
        "ef_search": 100
    }
})

# Baja latencia (recall ~0.90)
db = VantaEmbedded("./data", config={
    "hnsw": {
        "M": 8,
        "ef_construction": 100,
        "ef_search": 50
    }
})
```

## Recall vs Precision

| Métrica | Fórmula | Mide |
|---------|---------|------|
| **Recall@K** | $\frac{\text{TP}}{K}$ | ¿Qué fracción de los K mejores fueron encontrados? |
| **Precision@K** | $\frac{\text{TP}}{K}$ | ¿Qué fracción de los K recuperados son relevantes? |

En busqueda-vectorial, Recall@K y Precision@K son equivalentes cuando el ground truth tiene exactamente K elementos.

## Recall vs Latencia

Existe un trade-off fundamental:

```
Recall ↑  →  ef_search ↑  →  Latencia ↑
Recall ↓  →  ef_search ↓  →  Latencia ↓
```

**Curva típica:**

```
Recall
  1.0 ┤                          ╭──────
      │                    ╭─────╯
  0.9 ┤              ╭─────╯
      │         ╭────╯
  0.8 ┤    ╭────╯
      │    │
  0.7 ┤    │
      └────┴───────────────────────
      0   50   100   200   400
                ef_search
```

## Véase También

- [busqueda-vectorial](busqueda-vectorial.md) - Contexto de uso
- [HNSW](HNSW.md) - Algoritmo que optimiza recall
- [Latencia](Latencia.md) - Métrica complementaria
- [ANN](ANN.md) - Approximate Nearest Neighbor
