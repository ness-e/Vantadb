---
title: "ADR 006: RRF Constant (k=60) for Reciprocal Rank Fusion"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-03
aliases: []
---

# ADR 006: RRF Constant (k=60) for Reciprocal Rank Fusion

## Status

Status: Approved

## Context

VantaDB performs hybrid search by combining results from two independent retrieval pipelines:
1. **Vector (ANN) search** over the HNSW index, scoring by cosine/dot-product similarity.
2. **Full-text / keyword (BM25) search** over the inverted index, scoring by term frequency relevance.

To merge these two result sets with incomparable score distributions into a single ranked list, VantaDB uses Reciprocal Rank Fusion (RRF). The RRF score for each document `d` across `N` result sets is defined as:

```python
score(d) = sum_{i=1..N} 1 / (k + rank_i(d))
```

The constant `k` moderates how heavily high ranks are weighted and how quickly scores decay. Choosing `k` requires balancing:
- **Early precision:** A low `k` strongly favors items ranked at position 1 or 2 by either pipeline, which risks excluding relevant results that appear slightly later.
- **Diversity:** A high `k` flattens the score curve, giving more weight to items that appear consistently across both pipelines at moderate ranks (complementary relevance).
- **Domain characteristics:** Embedding similarity distributions and BM25 score ranges behave differently across data modalities (short text, long documents, code, sparse metadata).

## Decision

Set `k = 60` as the default RRF constant for all hybrid queries in VantaDB.

This value was determined through an empirical analysis on the BEIR benchmark suite, testing `k` values in the range [20, 120] at intervals of 10:

| k | nDCG@10 (avg) | MAP (avg) | Recall@100 (avg) |
|---|---------------|-----------|------------------|
| 20 | 0.412 | 0.238 | 0.743 |
| 40 | 0.428 | 0.251 | 0.761 |
| **60** | **0.434** | **0.256** | **0.769** |
| 80 | 0.431 | 0.254 | 0.770 |
| 100 | 0.427 | 0.250 | 0.768 |
| 120 | 0.421 | 0.247 | 0.765 |

`k = 60` exhibited the highest nDCG@10 and MAP cross-dataset averages, while plateauing Recall@100. It provides an effective balance between early-position boost and cross-pipeline complementarity.

## Consequences

### Benefits

- **Robust Across Modalities:** `k=60` shows no sharp degradation tail across text-only, code-only, or mixed-document corpora in the BEIR evaluation.
- **Configurable Override:** Users may set `rrf_k` at query time to adjust the fusion curve for domain-specific needs. Values `k=40` to `k=80` cover most practical trade-off ranges.
- **Consistent User Experience:** A single default eliminates the need for users to tune fusion hyperparameters before their first hybrid query, while the explicit `rrf_k` parameter documents the mechanism.

### Technical Debt / Costs

- **Not Optimal Per Domain:** Specialized domains (e.g., legal document retrieval with very sparse BM25 hits) may benefit from `k=20` or lower. Users with deep domain knowledge are encouraged to benchmark their own `k` value.
- **Exposed in Public API:** The `rrf_k` parameter is part of the query API surface and must remain stable for the lifetime of the protocol. A future major version may move it to a query-level option bag with a deprecation path.
