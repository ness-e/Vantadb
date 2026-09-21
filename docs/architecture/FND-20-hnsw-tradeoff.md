---
title: "FND-20: HNSW Trade-off (ef_search/M: recall vs latency) and why not IVF/FAISS"
type: architecture
status: active
tags: [vantadb, architecture, hnsw, indexing, faiss, showhn]
last_reviewed: 2026-08-16
aliases: []
---

# FND-20: HNSW Trade-off (ef_search/M: recall vs latency) and why not IVF/FAISS

> Technical note for the Show HN audience. A developer asks: *"Why HNSW?
> Why not FAISS / IVF / exact search?"* This document answers with the
> parameters actually shipped in `src/index/`, cited file:line, and the
> benchmark evidence from the certification suite. No numbers are invented;
> every figure below comes from code defaults or the certified stress
> protocol.

## 1. TL;DR

- VantaDB ships an HNSW graph index (`IndexType::Hnsw` is the default,
  `src/index/mod.rs:27-30`) with `M=32`, `M_max0=64`, `ef_construction=100`,
  `ef_search=100`, `ml = 1/ln(32) ≈ 0.288` (`src/index/graph.rs:255-269`).
- Below 10,000 nodes VantaDB **does not use the graph at all**: it falls back
  to an exact brute-force scan (`flat_threshold = Some(10000)`,
  `src/index/graph.rs:236-240`, `src/index/search/neighbors.rs:57-62`). For
  the typical local-first dataset the engine is exact for free.
- The certified stress protocol reports **Recall@10 = 0.9560** at 10K×128d
  with p50 latency ≈ 1.2 ms and ≈ 1172 bytes/node
  (`docs/operations/BENCHMARKS.md:27-30`,
  `docs/operations/PERFORMANCE_TUNING.md:454-458`).
- FAISS is an ANN **library**; VantaDB is an embedded **database**. The
  comparison is not "HNSW vs FAISS" but "bringing your own FAISS + storage +
  WAL + filtering + bindings" vs "opening a file". For 10K–1M local vectors
  the second is the defensible choice.

## 2. Current parameters (source of truth: code)

`HnswConfig::default()` — `src/index/graph.rs:255-269`:

| Parameter | Default | Meaning |
|-----------|---------|---------|
| `m` | `32` | Max bi-directional edges per node in layers > 0 |
| `m_max0` | `64` | Max edges per node in layer 0 (base layer), 2× `m` |
| `ef_construction` | `100` | Candidate list size during index build |
| `ef_search` | `100` | Candidate list size during query |
| `ml` | `1.0 / (32_f64).ln()` ≈ 0.288 | Layer assignment multiplier (`1/ln(M)`) |
| `distance_metric` | `Cosine` | Default metric |
| `flat_threshold` | `Some(10000)` | Exact-scan below this node count |
| `index_type` | `Hnsw` | Pluggable backend (see §5) |
| `auto_tune` | `false` | Adaptive `ef_search` is opt-in |

### How `ef_search` is applied at query time

`search_nearest` resolves the effective candidate list as
`ef_search = max(config.ef_search, top_k)`, i.e. recall is never below what
`top_k` demands; and when `auto_tune` is enabled it further takes
`max(static_ef, tuned_ef, top_k)` — `src/index/search/nearest.rs:71-77`.

The auto-tuner (`src/index/auto_tune.rs:11-53`) starts `ef_search` at 50,
bounded to `[10, 2000]`, grows it ×1.5 when a query falls back to brute
force, and halves it after 10 consecutive successful graph searches. It is
off by default so latency is deterministic; enable it (`auto_tune: true`)
for workloads where recall pressure varies.

### Documented drift (flagged, not papered over)

- ADR 005 (`docs/architecture/adr/005_hnsw_parameters.md:35`) records
  `ef_construction = 200`.
- `PERFORMANCE_TUNING.md:27` records `ef_construction = 400`.
- **Code ships `ef_construction = 100`** (`src/index/graph.rs:260`).

The code is the source of truth for this note. The gap is small
(`100 → 200` buys marginal build quality at ~2× build cost) and the
certified recall targets are met at the shipped value; the docs should be
reconciled in a follow-up.

## 3. The trade-off: ef_search/M vs recall vs latency

### ef_search (query-time)

`ef_search` is the beam width of the greedy graph descent on the base
layer (`src/index/search/nearest.rs:130-145`). It trades **latency directly
for recall**:

- **Cost**: every extra candidate costs one distance computation + one heap
  operation. Latency grows roughly linearly with `ef_search`.
- **Benefit**: recall grows steeply at first, then flattens — the classic
  ANN curve. Doubling `ef_search` from 50 → 100 buys most of the remaining
  recall; 100 → 200 buys the tail.
- **Why 100 is the default**: at `ef_search=100`, `top_k=10`, the certified
  protocol reaches Recall@10 = 0.956 at ~1.2 ms p50 (10K×128d)
  (`docs/operations/PERFORMANCE_TUNING.md:454-457`). Lower values (32–64)
  trade recall for throughput; higher values (200–500) push recall > 0.99
  for high-stakes queries. Because `ef_search` is per-query state, callers
  can raise it only for critical searches — a property FAISS's
  `index.search()` also exposes via its `efSearch` parameter, and one of the
  few places the two APIs are equivalent.

### M / M_max0 (construction-time)

`M` is the graph's out-degree. It is **locked at build time** and trades
**memory + build cost for recall ceiling**:

- **Cost**: edges dominate index memory. Each directed edge is a `u128` id
  in the inline `neighbor_lists` (`src/index/graph.rs:145-158`). The
  estimate is `≈ dim × 4 + (M + M_max0) × 4` bytes/vector —
  `docs/operations/PERFORMANCE_TUNING.md:87-93`: for 128d at defaults,
  896 B/node formula, ~1172 B/node measured including DashMap/SmallVec
  overhead (line 95-96).
- **Benefit**: a denser graph is a better navigable small world; recall at a
  *given* `ef_search` improves with `M`, so `M` and `ef_search` are
  substitutes. `M=32, M_max0=64` is the sweet spot for the 10K–1M target:
  recall ≥ 0.95 at the default `ef_search`, ~896 B/vector, sub-ms to
  low-ms latency. `M=16` halves edge memory at the cost of a lower recall
  ceiling; `M=64`+ raises the ceiling but linearly increases memory and
  build time.
- **Rule of thumb**: never ship `ef_construction < ef_search` — build
  quality caps search quality (`docs/operations/PERFORMANCE_TUNING.md:73`).

### The flat_threshold: exact search where it is free

`use_flat_search()` returns true when `nodes.len() <= flat_threshold`
(`src/index/search/neighbors.rs:57-62`), routing the query to an exact O(n)
scan (`src/index/search/nearest.rs:56-64`). The default threshold of 10,000
means:

- Datasets ≤ 10K get **100% recall, zero ANN approximation** — the regime
  where a flat scan is faster than any graph (no entry-point descent, no
  visited-set allocation).
- HNSW only starts being used where it actually pays: > 10K nodes.

This is a strong, honest answer to "why approximate search at all?": for
the smallest local datasets VantaDB doesn't approximate.

## 4. Memory: HNSW in RAM vs IVF on disk

| | HNSW (shipped) | IVF (available, opt-in) |
|---|---|---|
| Search cost | `O(ef_search · log N)` distance evals | `O(nlist + nprobe · N/nlist)` — probe `nprobe` of `nlist` inverted lists |
| Build | Incremental per insert, no training pass | **k-means over the whole dataset** (Forgy init + Lloyd, ≤ 20 iters, `src/index/ivf.rs:79-228`) |
| Memory | Edges in RAM (`≈ (M+M_max0) · 8 B` per node) + vectors | Centroids + inverted lists; vectors can be spilled |
| Inserts after build | O(1) per insert, graph stays navigable | Degrades recall until rebuild; VantaDB rebuilds lazily on next search (`src/index/search/nearest.rs:42-47`) |
| Latency profile | Sub-linear scaling, flat at small N | Probe-dependent; must visit full inverted lists per centroid |

IVF's advantage is memory at **million+ scale with many clusters** — that
is precisely not the local-first workload (10K–1M). IVF also has a
qualitative disadvantage for an embedded, mutable database: it needs a
training pass and periodic rebuilds as the dataset mutates, whereas HNSW
accepts inserts incrementally and stays navigable
(`src/index/ivf.rs:79-228` documents the training cost; `nearest.rs:42-47`
documents the lazy rebuild).

Other backends exist behind the same `VecIndex` trait
(`src/index/mod.rs:50-90`): `Flat` (exact scan, `mod.rs:33-34`), `DiskAnn`
(a Vamana-style graph, in-memory, `mod.rs:36`), and `Scann` (SQ8 scalar
quantization + re-rank, `mod.rs:37-38`). All are opt-in via `IndexType`
(`mod.rs:27-39`); HNSW remains the default because it is the best
average-case answer for the target workload, and the flat threshold covers
the exact-search corner.

## 5. Why HNSW for local-first

1. **Latency target**: sub-ms to low-ms on 10K–1M vectors. Certified p50
   ≈ 1.2 ms @ 10K, ≈ 6.1 ms @ 50K, scaling sub-linearly (~4.88× from 10K to
   50K vs 5× linear) — `docs/operations/PERFORMANCE_TUNING.md:456-470`.
   Rust-core p99 ≈ 441 µs @ 100K (`docs/operations/PERFORMANCE_GUIDE.md:104`).
2. **No training pass**: IVF needs k-means over the full dataset before the
   first query. HNSW builds incrementally — the first `put` is already
   searchable. For a local-first DB that must feel like a file, not a
   cluster, this is decisive.
3. **Incremental mutation**: inserts/deletes don't invalidate the index
   (certified persistence round-trip keeps recall within 0.01,
   `tests/certification/stress_protocol.rs:420-431`).
4. **No server, no separate index daemon**: the graph lives in the process,
   serialized portably (endianness-aware, `src/index/serialize/bytes.rs`).
5. **The exact corner is covered for free**: ≤ 10K vectors → exact scan
   (§3).

## 6. Why not FAISS

This is the question the Show HN will get. The honest answer has three
parts.

### 6.1 FAISS is a library, not a database

FAISS gives you `index.add()` / `index.search()` and serialization. It does
not give you:

- a write-ahead log, crash recovery, or transactions — you must build them,
- filtered search as part of the index traversal (you post-filter, or
  maintain your own id filters),
- a storage engine, metadata, or a query language,
- multi-language bindings beyond its own (Python/C++ primarily; the rest
  are community wrappers),
- a durability guarantee of any kind. `write_index` is a snapshot you
  manage yourself.

VantaDB embeds HNSW **inside** a storage engine with WAL + CRC32C
auto-healing, ACID transactions, metadata/filter bitsets evaluated during
traversal (ACORN path, `src/index/search/nearest.rs:139`), mmap-backed
vector storage, and PyO3/napi-rs bindings. "Why not FAISS?" is answered by
the architecture: *using FAISS means also building the other 80% of a
database around it* — which is exactly what this project did, with HNSW as
the index core because it is the right algorithm for this scale.

### 6.2 Where FAISS is better — and why it doesn't apply

| FAISS strength | Why it doesn't move the needle for local-first |
|---|---|
| **GPU search** (`faiss-gpu`, IVF/PQ batched) | The target is an embedded process on a laptop/edge box; no GPU, and for ≤ 1M vectors CPU HNSW is already sub-ms to low-ms |
| **Scale-out to 100M+ vectors** | Requires a serving layer (Milvus, Qdrant, pgvector…) around FAISS — a server deployment, the opposite of local-first |
| **Mature quantization (PQ/OPQ)** | VantaDB ships scalar (SQ8/3-bit/1-bit) and mmap tiering (`docs/operations/PERFORMANCE_TUNING.md:150-162`); PQ is a documented future path (`docs/architecture/PQ_FEASIBILITY.md`) |
| **Battle-tested at billion scale** | True; irrelevant when the workload is 10K–1M in one process |

### 6.3 The defensible conclusion

> "FAISS is the right tool when you are building a *search service* and
> will write the storage, durability, filtering, and bindings yourself, or
> when you need GPU/billion-scale. VantaDB is the right tool when you want
> an *embedded vector database*: open a file, put records, search them, and
> get the same guarantees as SQLite — HNSW chosen precisely because it
> needs no training pass, accepts incremental inserts, and hits
> recall ≥ 0.95 with sub-ms latency at the scale that fits in one process."

## 7. Evidence (certified, no invented numbers)

`docs/operations/BENCHMARKS.md:27-30` — Stress Protocol
(`tests/certification/stress_protocol.rs`):

| Metric | Value | Config |
|--------|-------|--------|
| Recall@10 | **0.9560** | 10K × 128d, Cosine, `ef_search=100` |
| Scaling recall | **0.998 / 1.000 / 0.998** | 10K / 50K / 100K |
| p50 latency | **1.2 ms / 6.1 ms** | 10K / 50K (`PERFORMANCE_TUNING.md:456`) |
| Throughput | **~833 / ~164 QPS** | 10K / 50K, single-threaded (`PERFORMANCE_TUNING.md:457`) |
| Memory | **~1172 B/node** | measured, DashMap + SmallVec included |
| Recall gate | ≥ 0.95 required | `stress_protocol.rs:235-236`, `hnsw_recall.rs:133` |

Build-time reference (`M=32, ef=200`): ~2 s @ 10K, ~12 s @ 50K, ~64 s @ 100K
(`docs/operations/PERFORMANCE_TUNING.md:462-467`).

## 8. References

- `src/index/graph.rs:227-269` — `HnswConfig` + defaults
- `src/index/search/nearest.rs:42-77,130-145` — backend routing + `ef_search` resolution
- `src/index/search/neighbors.rs:57-62` — `use_flat_search`
- `src/index/auto_tune.rs:11-53` — adaptive `ef_search`
- `src/index/mod.rs:27-90,100-139` — `IndexType`, `VecIndex`, `create_index`
- `src/index/ivf.rs:79-228` — IVF k-means build
- `src/index/serialize/bytes.rs:36-37` — serialized config (portable)
- `docs/operations/BENCHMARKS.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/PERFORMANCE_GUIDE.md`
- `docs/architecture/adr/005_hnsw_parameters.md` (drift flagged in §2)
- External: [HNSW vs FAISS comparison (Vectroid)](https://www.vectroid.com/resources/hnsw-vs-faiss-comprehensive-comparison),
  [How vector search works: IVF and HNSW (Medium)](https://medium.com/@arthurpro/how-vector-search-actually-works-ivf-and-hnsw-c96a0900f11d),
  [Faiss vs HNSWlib (Zilliz)](https://zilliz.com/blog/faiss-vs-hnswlib-choosing-the-right-tool-for-vector-search)