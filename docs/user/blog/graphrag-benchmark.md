---
title: GraphRAG Benchmark Methodology and Measured Results
kind: howto
status: active
description: Reproducible GraphRAG benchmark methodology, real indexing numbers, and an honest account of what remains unmeasured.
tags: [vantadb, benchmark, graphrag]
---

# GraphRAG Benchmark Methodology and Measured Results

> **Honest status of this post:** the **indexing-phase numbers below are real** — measured in a run of `benchmarks/graphrag_bench.rs` on the development machine described in §Hardware. The **query-latency and Token Reduction numbers at benchmark scale (3,000 nodes) are still PENDING**: in that environment (Windows x86_64, `--release`), the GraphRAG search phase ended in a reproducible stack overflow (`0xC00000FD`, `STATUS_STACK_OVERFLOW`) on the large corpus. No figure was invented: cells that could not be measured are marked PENDING, not filled with claims.
>
> **Environment note (2026-08-05 re-run):** after a build fix in `src/storage/vfile.rs` the script compiles and **small corpora (20–100 nodes) complete the full query phase** (real latency and tokens, section below). The canonical 3,000-node corpus still overflowed during query in that run — which is why the production-scale numbers stay PENDING and are not asserted as current engine behavior.

## Purpose

Validate the metrics published in the initial GraphRAG design (referenced from [`docs/api/GRAPH_RAG.md`](../../api/GRAPH_RAG.md)):

| Glossary metric | Published claim | Status after this work |
|---|---|---|
| Token Reduction vs RAG | 40-60% | **PENDING** — not measurable at scale in the recorded run |
| Added latency per hop | ~25-50ms | **PENDING** — not measurable at scale in the recorded run |
| Recall improvement | +15-20% | Not measured; this run did not include labeled relational-relevance judgments |
| Max hops | 3 | Verified in code: `GraphRagPipeline::new()` uses `expansion_hops: 2`, configurable |

The goal of this work is to turn those figures into **numbers from a reproducible run** — or, where they cannot be measured, to say so explicitly. The script is ready and reproducible.

## Reproducible script

**Path:** `benchmarks/graphrag_bench.rs` (registered as the `graphrag_bench` example in `Cargo.toml`).

```bash
# Build (requires the crate to compile)
cargo build --release -p vantadb --example graphrag_bench

# Run: 3,000-node corpus, 100 queries (defaults)
cargo run --release -p vantadb --example graphrag_bench

# Params: <nodes> <queries> <edges_per_node>
cargo run --release -p vantadb --example graphrag_bench -- 3000 100 2

# Optional JSON report
$env:GRAPHRAG_BENCH_OUT="benchmarks/graphrag_bench_result.json"
cargo run --release -p vantadb --example graphrag_bench -- 3000 100 2
```

### What it measures (all measured, nothing hardcoded)

1. **Indexing phase:** inserts `n` nodes (`VantaMemoryInput` with a 32d vector + content) plus edges (`add_edge`), then `flush()`. Measures total time (s) and throughput (nodes/s).
2. **Query phase (M deterministic queries):**
   - **RAG baseline** = `GraphRagPipeline` with `expansion_hops: 0` (seeds only, no expansion, no relations section). Latency + `context_text` tokens.
   - **GraphRAG** = `expansion_hops: 2` (seeds + BFS + ranking + relations in context). Latency + tokens.
   - **Token Reduction** = `1 - Tokens_GraphRAG / Tokens_RAG`.
3. **Hop sweep:** mean latency with `hops in {1,2,3}` → per-hop delta.
4. **Graph stats:** seeds found, nodes expanded, edges in context.

### Token counting

Whitespace-split (a documented proxy — **not** a real BPE tokenizer). Token values are comparative, not billing-absolute.

### Corpus (deterministic)

- **Seeded LCG** — no external RNG; same run ⇒ same corpus.
- 8 thematic clusters, dim=32, normalized vectors with noise around the centroid → synthetic vector-cluster and lexical signal, not model-generated semantic embeddings.
- Edges: forward DAG (intra-cluster chain + inter-cluster hub + misc forward) — never references a lower id, strictly acyclic.
- DB: `tempdir()` — removed on exit.

## Run results (2026-08-05)

**Hardware (measured by the script):**

```text
os=windows arch=x86_64 cpus=12 cpu=Intel64 Family 6 Model 154 Stepping 4, GenuineIntel
RAM: 31 GB · AVX2 · profile PERFORMANCE
```

### Indexing phase — REAL NUMBERS

| Corpus | Index time (s) | Throughput (nodes/s) |
|---|---|---|
| 20 nodes | 1.569 | 12.7 |
| 100 nodes | 10.749 | 9.3 |
| 300 nodes | 26.809 | 11.2 |
| 800 nodes | 86.016 | 9.3 |
| 3000 nodes | 174.308* | 17.2 |

*The 20–800 values come from the aborted earlier run; **the 3,000-node value (174.308 s / 17.2 nodes/s) was reported in the 2026-08-05 re-run**. These historical values are preserved without recomputation. The recorded evidence does not isolate a cause for differences between runs.*

**Honest reading:** the table describes that historical per-node CRUD workload,
not current v0.9.0 throughput or a general ingest limit. It is **not** an
optimized ingest benchmark; see `benchmarks/competitive_bench.py` for batched ingest.

### Query phase — PENDING AT SCALE

Recorded reproduction of the failure (re-measured 2026-08-05):

```text
=== GraphRAG benchmark ===
corpus: 3000 nodes, 8 topics, dim=32 | queries: 100
index: 3000 nodes + edges in 174.308s (17 nodes/s)
thread '<unknown>' has overflowed its stack     # exit code -1073741571 / 0xC00000FD
```

- **Trigger:** the canonical run (3,000 nodes / 100 queries / `edges_per_node=2`) overflowed on the **first query** (`GraphRagPipeline::search`), after indexing completed. With the same build, **small corpora (20 and 100 nodes) complete the query phase** — so the failure scaled with graph/context size in that environment.
- **Not attributed:** whether this reflects current engine behavior is unverified; no reproduction was run for this post beyond the recorded log above.

> **To readers:** do not use the empty cells. The glossary's "40-60% Token Reduction" claim **still lacks a productive-scale run**. This post says so explicitly rather than inventing one.

### Small corpora complete the run (real, 2026-08-05)

```text
# 100 nodes / 10 queries / edges_per_node=2
rag_baseline  : p50=0.866 p95=2.397 mean=1.014 ms
graphrag      : p50=1.418 p95=4.091 mean=1.693 ms
per_hop_delta (1->3): 0.726 ms
rag_tokens_avg   : 108.4
graphrag_tokens_avg: 812.0
token_reduction  : -6.491 (-649.08%)
seeds_found_avg   : 10.00   nodes_expanded_avg: 82.50
```

> **These numbers are NOT production-representative.** On small, densely connected corpora, a 2-hop expansion reaches almost the whole graph (`nodes_expanded_avg: 82.5` of 100), so `context_text` grows versus baseline and `token_reduction` comes out **negative**. They are useful only as a pipeline smoke test; the production number needs the 3,000-node corpus.

## Scope / non-goals

- **Recall improvement (+15-20%):** NOT measured — this run did not include labeled relational-relevance judgments.
- **Embeddings:** no real embedding model; vectors are synthetic clusters. The benchmark measures the GraphRAG pipeline (graph structure + ranking), not embedding quality.
- **LanceDB/Chroma comparison:** out of scope; see `benchmarks/competitive_bench.py` for pure vector comparison and `docs/user/operations/BENCHMARKS.md` for curated numbers.

---

*Last updated: 2026-08-05 · Script: `benchmarks/graphrag_bench.rs` · Registered: `Cargo.toml [[example]] graphrag_bench`*
