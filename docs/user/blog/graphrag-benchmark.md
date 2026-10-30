---
title: GraphRAG Benchmark
kind: reference
status: active
description: Measured GraphRAG vs plain RAG latency and context cost on a deterministic corpus
tags: [vantadb, benchmarks, graphrag]
---

# GraphRAG Benchmark

Measured with `benchmarks/graphrag_bench.rs` (MKT-16):

```powershell
cargo run --release --example graphrag_bench -- 3000 100
```

## Method

Deterministic corpus (seeded LCG, no external RNG): 3000 nodes in 8 topic
clusters, dim 32, chain + hub + sparse cross-cluster edges. Fresh temp DB,
10 warmup queries, median aggregation. Token counting is a whitespace-split
proxy (**not** a real tokenizer — treat token numbers as approximate).

> **History:** on 2026-08-05 this bench hit a stack overflow in the engine
> search phase on Windows x86_64 release (AUDIT-04 class) — only index timing
> was real, query metrics were PENDING. Re-run 2026-10-30: the overflow is
> **resolved** (engine fixes since August); all cells below are measured.

## Results (2026-10-30, Windows x86_64, 12 cores)

Index: 3000 nodes + edges in **30.968 s** (97 nodes/s).

| Metric | RAG baseline (hops=0) | GraphRAG (hops=2) |
|---|---|---|
| p50 latency | 20.383 ms | 22.489 ms |
| p95 latency | 26.231 ms | 26.895 ms |
| mean latency | 17.476 ms | 18.841 ms (**+1.365 ms**) |
| avg tokens (whitespace proxy) | 76.8 | 610.5 |

Per-hop latency (mean): hops=1 → 16.434 ms · hops=2 → 17.488 ms · hops=3 → 18.028 ms (+0.797 ms per hop, 1→3).

Graph stats: seeds_found 10.00/10 · nodes_expanded 100.00 · context edges (sample) 92.

## Reading the token number honestly

`token_reduction` prints **−6.955 (−695%)**: GraphRAG returns ~8× *more*
tokens than plain RAG. That is expected — expansion adds related context;
the benefit is recall quality (more relevant nodes surfaced), not token
savings. Do not cite this metric as "reduction"; cite it as expansion cost
(+534 tokens/query avg) against the +1.4 ms latency delta.

## Reproduce

```powershell
cargo run --release --example graphrag_bench -- 3000 100
# JSON report: GRAPHRAG_BENCH_OUT=path.json (schema_version 1)
```

Small smoke: `cargo run --release --example graphrag_bench -- 20 5`.
