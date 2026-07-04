---
title: "Heuristic Search"
type: glossary-entry
status: stable
tags: [glosario, hnsw, busqueda, heuristica, algoritmo]
last_reviewed: 2026-07-03
aliases: [heuristic search, select_neighbors_heuristic, heuristic, búsqueda-heurística]
description: "Algoritmo de selección de vecinos en HNSW que maximiza diversidad espacial (Algorithm 4, Malkov & Yashunin 2018)"
---

# Heuristic Search

## Definición

El **heuristic search** (búsqueda heurística) en el contexto de VantaDB refiere al algoritmo de selección de vecinos definido en el paper de **[[hnsw]]** (Algorithm 4, Malkov & Yashunin 2018). Su propósito es maximizar la diversidad espacial de las conexiones en el grafo, evitando que todos los vecinos de un nodo estén en la misma región del espacio vectorial.

## Algoritmo

```
Input: candidates (candidatos ordenados por similitud), M (conexiones máximas)
Output: selected (vecinos con diversidad espacial)

selected = []
discarded = []

for candidate in candidates:
    if len(selected) >= M:
        break

    if any similarity(candidate, selected[i]) > similarity(candidate, query):
        # Candidate está muy cerca de un ya seleccionado → se descarta
        discarded.push(candidate)
    else:
        selected.push(candidate)

# Si hay slots libres, se rellenan con descartados (keepPrunedConnections)
fill_remaining(selected, discarded, M)
```

### Implementación en VantaDB

```rust
// src/index/core.rs — ~line 919
/// Select neighbors using the HNSW paper heuristic (Algorithm 4).
/// Applies spatial diversity from slot 0 — no unconditional acceptance.
/// keepPrunedConnections=true: fills limited remaining slots with discarded candidates.
///
/// Metric: cosine similarity (higher = closer). The diversity condition is:
///   reject if similarity(candidate, selected) > similarity(candidate, query)
fn select_neighbors(&self, candidates: BinaryHeap<NodeSimMin>, m: usize) -> NeighborVec {
    let sorted = candidates.into_sorted_vec();
    let mut selected: Vec<SelectedInfo> = Vec::with_capacity(m);
    let mut discarded: Vec<u64> = Vec::new();

    for ns in sorted.into_iter() {
        if selected.len() >= m { break; }
        let cand_id = ns.1;
        let sim_q_cand = ns.0;

        // Diversity check
        let too_close = selected.iter().any(|sel| {
            let sim_sel_cand = self.cosine_similarity(cand_id, sel.id);
            sim_sel_cand > sim_q_cand
        });

        if too_close {
            discarded.push(cand_id);
        } else {
            selected.push(SelectedInfo { id: cand_id, ... });
        }
    }
    // Fill remaining slots with discarded candidates
    // ...
}
```

## Por Qué Es Necesario

Sin la heurística de diversidad, el grafo HNSW tendería a conectar cada nodo con sus vecinos más cercanos, creando "clusters" densos pero con mala navegabilidad global. La heurística garantiza:

1. **Mejor recall** — el grafo cubre más regiones del espacio
2. **Menos saltos** — rutas más cortas entre nodos distantes
3. **Robustez** — el grafo funciona bien incluso con datos no uniformes

## Parámetros Relacionados

| Parámetro | Efecto en Heuristic Search |
|-----------|---------------------------|
| **M** | Máximo de vecinos por nodo — más slots = más diversidad |
| **ef_construction** | Más candidatos = mejor selección heurística |
| **keepPrunedConnections** | Si es true, rellena slots vacíos con descartados |

## Véase También

- [[hnsw]] — Algoritmo que utiliza esta heurística
- [[vector-search]] — Búsqueda por similitud vectorial
- [[ann]] — Approximate Nearest Neighbor
- [[../architecture/hnsw_index|HNSW Index Architecture]]
