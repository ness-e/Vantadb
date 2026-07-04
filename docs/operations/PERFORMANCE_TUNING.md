---
title: "VantaDB Performance Tuning Guide"
type: operations
status: active
tags: [vantadb, operations, performance, tuning]
last_reviewed: 2026-07-03
aliases: []
---

# VantaDB Performance Tuning Guide

This guide covers all knobs, tradeoffs, and recommended configurations for
tuning VantaDB performance across different workload profiles.

---

## 1. HNSW Parameters

VantaDB uses Hierarchical Navigable Small World graphs (`src/index/core.rs`)
for approximate nearest-neighbour (ANN) search. The `HnswConfig` struct
exposes five core parameters:

| Parameter | Default | Range | Effect |
|-----------|---------|-------|--------|
| `m` | 32 | 4–128 | Max connections per node (inner layers) |
| `m_max0` | 64 | 8–256 | Max connections per node (layer 0) |
| `ef_construction` | 200 | 64–800 | Search breadth during index build |
| `ef_search` | 100 | 32–500 | Search breadth during queries |
| `ml` | 1/ln(32) | — | Layer normalization factor (auto-computed) |

### When to Adjust Each Parameter

**`m` (connections per node)**

- **Higher (48–128)**: Improves recall at the cost of more memory and slower
  build times. Edge count grows linearly with `m`. Recommended for datasets
  where recall > 0.99 is required (e.g., high-stakes RAG, deduplication).
- **Lower (4–16)**: Reduces memory footprint and speeds up builds. Suitable
  for resource-constrained devices or datasets where ~90 % recall is
  sufficient.
- **Rule of thumb**: Double `m` ≈ double memory usage for the edge list.
  Each edge costs 8 bytes (u64 neighbour ID). With 1M vectors and `m=32`,
  expect ~256 MB for edges alone.

**`m_max0` (layer 0 connections)**

- Layer 0 contains all vectors. `m_max0` is typically 2× `m`.
- Default 64 is a safe starting point. Increase if you observe poor recall
  on dense clusters; decrease to save memory on sparse datasets.

**`ef_construction` (build quality)**

- Controls how many candidates are evaluated per node during insertion.
  Higher values produce a higher-quality graph (better recall) but slow
  down index construction significantly — O(N × log N × ef_construction).
- **Low-latency builds** (one-shot indexing): 64–100.
- **Production builds** (background rebuild): 200 (default).
- **Maximum recall**: 400–800 (build time increases 2–4×).

**`ef_search` (query quality)**

- Controls how many candidates are examined during search. Directly
  trades latency for recall.
- **Throughput-oriented**: 32–64 (faster queries, lower recall).
- **Balanced**: 100 (default).
- **High-recall**: 200–500 (slower queries, recall > 0.99).
- Can be overridden per-query — it is safe to set `ef_search` higher for
  critical queries and lower for bulk/exploratory queries.

### Parameter Relationships

```
ef_construction ≥ ef_search  (otherwise build quality limits search quality)
m ≥ 4                        (below 4 the graph becomes disconnected)
m_max0 = 2 × m               (standard heuristic, may vary by density)
ml = 1 / ln(m)               (ensures logarithmic layer distribution)
```

### Memory-per-Vector Estimate

```
edges   = m × 8 bytes             (inner layer edges per vector)
edges0  = m_max0 × 8 bytes        (layer 0 edges)
graph   ≈ (edges + edges0) / 2    (layered: ~50 % of nodes reach upper layers)
vectors = dim × 4 bytes           (f32 vector payload)

Total ≈ dim × 4 + (m + m_max0) × 4 bytes per vector
```

For 128‑dimensional vectors at defaults:
```
128 × 4 + (32 + 64) × 4 = 512 + 384 = 896 bytes/node
```

The certified benchmark (`BENCHMARKS.md`) reports ~1172 bytes/node due to
D hash-map overhead and SmallVec capacity.

---

## 2. Memory Limits

### `VANTADB_MEMORY_LIMIT`

Set via `VantaConfig::memory_limit` or the Python SDK's
`memory_limit_bytes`. This is a **budget hint** — it influences:

- Whether `mmap_hnsw` is enabled (auto-enabled on systems with < 16 GB RAM).
- Backpressure thresholds (combined with `rss_threshold`).
- The `HardwareProfile` detection (`Performance` vs `LowResource`).

```rust
// Rust: programmatic override
let config = VantaConfig::default()
    .with_memory_limit(4_096_000_000)  // 4 GB budget
    .with_rss_threshold(0.85);

// Python SDK
db = vantadb.VantaDB("./data", memory_limit_bytes=4_096_000_000)
```

### Memory Governor (Hot/Cold Tiering)

VantaDB includes a `QuantizationGovernor` (`src/vector/governor.rs`) that
tracks access frequency and automatically transitions cold vectors from
full f32 to quantized representations:

| Tier | Representation | Bytes/Dim | Recall Impact |
|------|---------------|-----------|---------------|
| **Hot** (f32 Full) | `VectorRepresentations::Full` | 4 | None (baseline) |
| **Warm** (SQ8) | `VectorRepresentations::SQ8` | 1 | ~0.5–2 % drop |
| **Cold** (MMap) | `VectorRepresentations::MmapFull` | 0 (OS page cache) | None when paged in |

Configuration constants in `QuantizationConfig`:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `cold_threshold_ticks` | 100 | Ticks without access before SQ8 demotion |
| `tick_interval_ms` | 1000 | Governor tick interval (1s) |
| `hot_threshold` | 5 | Hits per tick to promote back to f32 |

### Quantization Options

Beyond the governor's auto-tiering, vectors can be stored pre-quantized:

| Scheme | Codec | Bytes/Dim | Use Case |
|--------|-------|-----------|----------|
| f32 Full | Raw float32 | 4 | Maximum precision, hot data |
| SQ8 | 8-bit scalar | 1 | Balanced memory/accuracy, warm data |
| TurboQuant | 3-bit packed | 0.5 | Large-scale approximate retrieval |
| RaBitQ | 1-bit (POPCNT) | 0.015 | Ultra-fast pruning / pre-filter |

Quantization is selected automatically at the node level. The
`VectorRepresentations` enum (`src/node.rs:30`) supports all four schemes.

### Eviction Scoring

When the memory governor triggers eviction, each node is scored by:

```
score = (hits × weight_hits)
      + (confidence × weight_confidence)
      + (importance × weight_importance)
      + (recency × weight_recency)
```

Default weights penalise low-confidence, low-importance, seldom-accessed
nodes. Adjust via:

```rust
config.with_eviction_weights(
    hits: 0.5,        // reduce hit-count influence
    confidence: 3.0,   // increase confidence weight
    importance: 4.0,   // increase importance weight
    recency: 2.0,      // medium recency weight
);
```

The `eviction_ratio` (default 0.20) controls what fraction of hot nodes
are evicted per pressure event. Lower values = gentler but more frequent
evictions.

### Memory vs Performance Tradeoffs

- **Full f32 + no limit**: Best recall, highest memory. Suitable for
  datasets < 1M vectors on servers with ≥ 32 GB RAM.
- **MMap HNSW + SQ8 cold**: Good balance for 1M–10M vectors. Hot nodes
  stay in f32; cold nodes are quantized or paged from disk.
- **Quantized-only (SQ8/Turbo)**: Minimal memory footprint. Accept 1–3 %
  recall loss for 5–10× memory reduction.

---

## 3. Backend Selection

VantaDB supports three storage backends via the `StorageBackend` trait.

| Feature | Fjall (default) | RocksDB | InMemory |
|---------|-----------------|---------|----------|
| Language | 100% Rust | C++ (C bindings) | Rust-only |
| Build time | ~30 s | ~5–10 min | ~20 s |
| Dependencies | Zero | CMake, Clang, libstdc++ | None |
| Memory safety | Safe Rust | `unsafe` bindings | Safe Rust |
| WAL + crash recovery | ✅ | ✅ | ❌ (ephemeral) |
| MVCC | ✅ | ✅ | ✅ (in-process) |
| LSM-tree | ✅ | ✅ | ❌ (BTreeMap) |
| WASM compat | ❌ | ❌ | ✅ |
| Concurrent access | Process-level lock | Process-level lock | Single-process |

### Selection Guidance

- **Fjall** (env `VANTA_BACKEND=fjall`, default):
  For embedded/local-first applications. Pure Rust, fast compilation, no
  system dependencies. Use unless you have a specific need for RocksDB.

- **RocksDB** (env `VANTA_BACKEND=rocksdb`):
  For extreme write throughput (> 100K ops/sec) or legacy infrastructure
  that depends on RocksDB tooling. Requires C++ build toolchain.
  Enables `supports_checkpoint` and `supports_manual_compaction` for
  advanced operational workflows.

- **InMemory** (env `VANTA_BACKEND=memory`):
  For testing, ephemeral data, CI pipelines, and WASM targets. All data
  is lost on process exit. Enables the fastest possible throughput with
  zero durability guarantees.

### Migration Between Backends

Backends are **not interchangeable at runtime** — data must be exported
and re-imported:

```bash
# 1. Export from source
vanta-cli export --namespace default --out ./backup.jsonl --db ./fjall_data

# 2. Re-open with new backend
vanta-cli import --in ./backup.jsonl --db ./rocksdb_data
```

---

## 4. Sync Modes

The `SyncMode` enum (`src/config.rs:44`) controls WAL fsync behaviour.

| Mode | fsync | Durability | Throughput Impact |
|------|-------|------------|-------------------|
| `Always` | Every write | Maximum (ACID) | 10–100× slower |
| `Periodic` (default) | Every ~5 s | High | Baseline |
| `Never` | Never | Low (OS page cache) | Fastest |

### When to Use Each Mode

- **`Always`**: Financial/transactional workloads where losing a single
  write is unacceptable. On SATA SSDs, expect 10–100 ms per write.
- **`Periodic`**: General-purpose use. Balances safety and speed.
  A crash may lose the last ~5 seconds of writes.
- **`Never`**: Bulk ingestion, caching layers, disposable data.
  Maximum throughput. Use with `InMemory` backend for zero-persist
  workloads.

### Async WAL

VantaDB currently uses synchronous WAL writes. Async WAL (batching
fsyncs across multiple mutations) is not yet implemented but is the
recommended path for future high-throughput optimisations.

---

## 5. Query Optimization

### Bitset Filters

Each `UnifiedNode` stores a `u128` bitset field
(`src/node.rs: bitset: u128`). During search, a `query_mask` is AND-ed
against this bitset — only nodes where
`(node.bitset & query_mask) == query_mask` are returned.

```python
# Python: tag nodes with bitset categories
db.put("doc1", vector=[...], bitset=0b0011)   # category A + B
db.put("doc2", vector=[...], bitset=0b0001)   # category A only

# Search filtered to category A only
results = db.search(query="...", bitset_filter=0b0001)
```

Bitset filtering is evaluated **during HNSW traversal** (inside the hot
loop in `search_layer` at `src/index/core.rs:768`), so filtered-out nodes
are never returned. This is far more efficient than post-filtering.

**Performance impact**: Near-zero when the filter is selective. The bitset
check is a single `u128` AND + compare — essentially free. Use bitsets to
implement tenant isolation, document type routing, or any categorical
filter.

### Hybrid Search Tuning

Hybrid search fuses BM25 lexical results with HNSW vector results using
Reciprocal Rank Fusion (`src/planner.rs`).

Key constants defined in `src/planner.rs`:

| Constant | Default | Effect |
|----------|---------|--------|
| `RRF_K` | 60 | RRF smoothing constant. Higher = more weight to lower ranks |
| `CANDIDATE_MULTIPLIER` | 4 | Per-arm candidate budget = `top_k × 4` |
| `MIN_CANDIDATE_BUDGET` | 32 | Minimum candidates per arm |
| `MAX_CANDIDATE_BUDGET` | 256 | Maximum candidates per arm |

**RRF K tuning**:
- **Low K (10–30)**: Emphasises top-ranked results from each arm. Higher
  precision but may miss relevant mid-ranked documents. Good for exact-match
  queries.
- **High K (60–120)**: Smoother blending, more weight to consensus results.
  Better for exploratory/ambiguous queries.

**Candidate budget tuning**:
- Higher `CANDIDATE_MULTIPLIER` (6–8) improves recall at the cost of more
  work per query. Useful when `top_k` is small but recall is critical.
- Lower (2–3) improves throughput for large `top_k` values.
- Clamped by `MAX_CANDIDATE_BUDGET` to prevent unbounded scans.

### Batch Operations vs Individual

The Python SDK exposes `search_batch()` which amortises PyO3 FFI and
HNSW GIL overhead:

| Method | 100 Queries (5K records) | Speedup |
|--------|-------------------------|---------|
| `db.search()` sequential | 973.68 ms | 1× (baseline) |
| `db.search_batch()` | 243.01 ms | **4.01×** |

Always batch independent queries when throughput matters.

---

## 6. Hardware Considerations

### CPU: SIMD Support

VantaDB auto-detects the available instruction set at startup
(`src/hardware/mod.rs`) and selects the fastest vector paths:

| Instruction Set | Detection | Rel. Performance |
|----------------|-----------|------------------|
| AVX-512 | `std::is_x86_feature_detected!("avx512f")` | 1.5–2× vs AVX2 |
| AVX2 | `std::is_x86_feature_detected!("avx2")` | 3–5× vs scalar |
| NEON | `std::arch::is_aarch64_feature_detected!("neon")` | 3–5× vs scalar |
| Scalar fallback | None detected | Baseline |

All SIMD paths use the `wide` crate (f32x8) for automatic vectorisation
of dot products and distance computations.

**Build for your CPU**: Compile with `-C target-cpu=native` to enable all
instructions:

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

For CI/packaged builds that must run on older hardware, omit this flag;
VantaDB will detect available instructions at runtime and fall back
gracefully.

### RAM

| Dataset Size | Recommended RAM | Memory Limit | Notes |
|-------------|----------------|--------------|-------|
| < 100K vectors | 2–4 GB | None needed | Fully in-memory f32 |
| 100K–1M vectors | 8–16 GB | 4096 MB | MMap HNSW recommended |
| 1M–10M vectors | 16–64 GB | 8192–16384 MB | Enable SQ8 for cold data |
| 10M–100M+ vectors | 64 GB+ | 32768+ MB | Quantization required |

**RAM per vector (estimated)**:

- HNSW graph edges: ~896 bytes (128d, default params)
- HNSW DashMap overhead: ~60 bytes per entry
- VantaFile vector payload: `dim × 4` bytes
- Text index (BM25): varies with content, ~text_length × 2 bytes

**Tip**: If the HNSW index fits in memory but vector payloads do not,
enable `mmap_hnsw` (default on) to keep graph edges hot and page vector
data from disk on demand.

### Storage: SSD vs HDD

| Storage | WAL fsync | MMap load | Query Latency Impact |
|---------|-----------|-----------|---------------------|
| NVMe SSD | ~50 µs | Excellent | Baseline |
| SATA SSD | ~500 µs | Good | +10–30 % |
| HDD | ~10 ms | Poor | +200–500 % |

- **Always use SSD** for production. HDDs introduce catastrophic latency
  during mmap page faults and WAL fsync.
- For cold data on HDD, consider setting `prefetch_mode=Enabled` to
  pre-load pages into cache.
- For fast NVMe, set `prefetch_mode=Disabled` to avoid unnecessary
  `madvise` syscalls.

### Recommended Specs by Dataset Size

| Dataset | CPU | RAM | Storage | Profile |
|---------|-----|-----|---------|---------|
| Dev / < 10K | Any | 2 GB | Any | `LowResource` |
| Small (< 100K) | 4+ cores / SSE | 4–8 GB | SATA SSD | `Performance` |
| Medium (100K–1M) | 8+ cores / AVX2 | 16 GB | NVMe | `Performance` |
| Large (1M–10M) | 16+ cores / AVX-512 | 32–64 GB | NVMe | `Enterprise` |
| Extreme (10M+) | 32+ cores / AVX-512 | 128 GB+ | NVMe RAID | `Enterprise` |

---

## 7. Benchmarking

### Running Benchmarks

VantaDB uses Cargo's built-in benchmark harness:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench -- "stress_protocol"

# Run with CPU-native optimisations (recommended for meaningful results)
RUSTFLAGS="-C target-cpu=native" cargo bench

# Python SDK benchmarks
maturin develop --manifest-path vantadb-python/Cargo.toml --release
python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000
```

### Understanding Results

The **Stress Protocol** (`tests/certification/stress_protocol.rs`) is a
7-block certification suite. Key metrics:

| Metric | 10K | 50K | 100K | Meaning |
|--------|-----|-----|------|---------|
| Recall@10 | 0.956 | — | — | Fraction of true nearest neighbours returned |
| Scaling recall | 0.998 | 1.000 | 0.998 | Recall consistency across dataset sizes |
| p50 latency | 1.2 ms | 6.1 ms | — | Median query latency |
| Throughput (QPS) | ~833 | ~164 | — | Queries per second (single-threaded) |
| Memory/node | ~1172 B | ~1172 B | — | Bytes per vector in HNSW |

### Expected Performance by Dataset Size

| Scale | Build Time (M=32/ef=200) | p50 Query (ef=100) | QPS (single-threaded) |
|-------|--------------------------|-------------------|----------------------|
| 10K × 128d | ~2 s | ~1 ms | ~1000 |
| 50K × 128d | ~12 s | ~6 ms | ~166 |
| 100K × 128d | ~64 s | ~12 ms | ~83 |
| 1M × 128d | ~15 min | ~100 ms | ~10 |

**Scaling factor**: Latency grows sub-linearly with dataset size
(certified ~4.88× from 10K to 50K, compared to 5× linear).

### Python SDK Overhead

Python benchmarks include PyO3 boundary crossing and GIL overhead.
Expect 2–5× higher latency compared to pure Rust:

| Operation | Rust Core | Python SDK | Overhead |
|-----------|-----------|------------|----------|
| Vector search (10K) | 1.2 ms | 62 ms | ~50× |
| Lexical search (10K) | ~2 ms | 115 ms | ~57× |
| Insert (single) | ~100 µs | 10.7 ms | ~107× |

Overhead is dominated by FFI serialisation, not HNSW traversal. Use
`search_batch()` to amortise this cost (4× throughput improvement).

### Interpreting Latency Breakdown

When optimising, profile each phase independently:

1. **HNSW traversal**: Dominates for vector-only queries. Tune `ef_search`.
2. **BM25 scoring**: Dominates for lexical/text queries. Tune tokenizer.
3. **RRF fusion**: Negligible (< 100 µs) for typical candidate sizes.
4. **FFI serialisation**: Dominates Python SDK overhead. Use `search_batch()`.
5. **Mmap page fault**: Cold-start latency. Warm up with a few probe queries.
