---
title: "ADR 005: HNSW Graph Parameters (M, M_max0, ef_construction, ef_search, ml)"
type: adr
status: active
tags: [vantadb, architecture, adr]
last_reviewed: 2026-07-03
aliases: []
---

# ADR 005: HNSW Graph Parameters (M, M_max0, ef_construction, ef_search, ml)

## Status

Status: Approved

## Context

VantaDB's core vector index is a Hierarchical Navigable Small World (HNSW) graph. The recall performance, index build time, memory footprint, and query latency are directly governed by five key parameters:
- `M`: Maximum number of bi-directional connections per element per layer (excluding layer 0).
- `M_max0`: Maximum number of connections per element in layer 0 (the base layer).
- `ef_construction`: Dynamic candidate list size during index construction.
- `ef_search`: Dynamic candidate list size during query execution.
- `ml` (level generation multiplier): Controls the probability distribution for assigning elements to layers.

We conducted a benchmarking campaign across standard embedding dimensionalities (384, 768, 1536) using the Deep1M, GIST-1M, and SIFT-1M datasets to determine the optimal configuration for the general-purpose case.

## Decision

Select the following HNSW parameters as VantaDB's defaults:

| Parameter        | Selected Value | Rationale |
|------------------|----------------|-----------|
| `M`              | 32             | Balances graph connectivity and memory per node |
| `M_max0`         | 64             | Doubles base-layer capacity for dense neighborhood coverage |
| `ef_construction`| 200            | High recall without excessive build-time penalty |
| `ef_search`      | 100            | Default query-time recall target; user-overridable per query |
| `ml`             | `1.0 / ln(32)` | Approximately 0.288; aligns with M=32 for optimal layer distribution |

The layer generation multiplier is computed as `ml = 1.0 / ln(M)`, which is the standard heuristic from the original HNSW paper (Malkov & Yashunin, 2016). For `M = 32`, this yields `ml ≈ 0.288`.

## Consequences

### Benefits

- **High Recall (>=0.98 @ 10-NN):** The combination of `M=32` and `ef_construction=200` consistently achieves 98%+ recall at 10-Nearest Neighbors across all benchmarked dimensionalities, meeting VantaDB's correctness target.
- **Controlled Index Build Time:** `ef_construction=200` avoids the diminishing-returns regime observed at values >=300 while remaining well below the build-time explosion at values >500.
- **User-Controllable Query-Time Tradeoff:** `ef_search=100` is a conservative default; callers may lower it (e.g., 40) for throughput-sensitive scenarios or raise it (e.g., 400) for high-recall analytics.
- **Memory Predictability:** Each edge consumes a `(u32, u32)` node-id pair. With `M=32` (average edges per layer) and `M_max0=64` (at layer 0), total memory per stored vector is approximately `(M * log_n + M_max0) * 8 bytes`, growing logarithmically with dataset size.

### Technical Debt / Costs

- **Memory Per Node Higher than Low-M Variants:** A configuration of `M=16` would yield roughly half the connections and therefore a smaller memory footprint. Users embedding on severely memory-constrained devices may need to override `M`.
- **Insert Performance at Scale:** `ef_construction=200` introduces non-trivial search cost during insertion. For bulk-load workflows, VantaDB exposes the `hnsw-build` CLI tool that can pre-construct the graph in a single pass.
- **ml Computation Coupled to M:** Changing `M` without also adjusting `ml` via the `1.0 / ln(M)` formula will produce suboptimal layer distributions. The binding is maintained in a single constant `HNSW_ML_SCALE` to prevent drift.
