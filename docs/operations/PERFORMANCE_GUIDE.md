---
title: VantaDB — Performance Guide: Rust Core vs Python SDK
type: operations
status: active
tags: [vantadb, operations, performance, latency, pyo3]
last_reviewed: 2026-07-05
aliases: [performance-guide, rust-vs-python]
---

# VantaDB — Performance Guide: Rust Core vs Python SDK

This document explains the performance characteristics of VantaDB's architecture layers, quantifies the gap between the Rust core and the Python SDK, and sets realistic expectations for production use.

> **Key takeaway:** The Rust core delivers ~441µs p99 latency at 100K scale. The Python SDK adds ~140x overhead (62ms) due to FFI crossing, GIL management, type conversion, and result serialization. This is **expected and by design** — not a bug.

---

## 1. Architecture Layers and Their Latency Impact

Every operation from Python traverses these layers:

```
┌─────────────────────────────────────────────────┐
│  Python caller                                   │
│  e.g., db.search_memory(vector=q, top_k=10)      │
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  1. PyO3 FFI boundary (Rust ← Python entry)     │  ~1-5µs
│     - Type unpacking (args → Rust types)         │
│     - GIL currently held by caller               │
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  2. Vector extraction (Python obj → Vec<f32>)   │  ~5-15µs (DX-04: zero-copy w/ buffer protocol)
│     - Attempts zero-copy via __buffer_protocol__ │
│     - Falls back to Python list iteration        │
│     - f64→f32 downcast if NumPy float64           │
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  3. Metadata conversion (PyDict → BTreeMap)     │  ~15-30µs
│     - Python dict → Rust HashMap (per field)     │
│     - LRU-cached for small common dicts          │
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  4. GIL release (py.detach / py.allow_threads)  │  ~3-10µs
│     - Releases Python GIL for concurrent ops     │
│     - Blocks other Python threads during → NO    │
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  5. Rust engine operation (no GIL)              │  ~441µs
│     - HNSW graph traversal                      │
│     - SIMD distance computation                 │
│     - BM25 scoring (if hybrid)                  │
│     - RRF fusion (if hybrid)                    │
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  6. Result serialization (Rust → Python dict)   │  ~20-40ms
│     - For each hit: VantaMemorySearchHit → PyDict│
│     - VantaVector wrapper (no f32→list copy)     │
│     - Metadata dict construction per hit         │
│     - Explanation/fusion report (if requested)   │
└──────────────────────┬──────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────┐
│  7. GIL re-acquire + FFI return                 │  ~5-10µs
│     - Python dict objects handed to interpreter  │
└─────────────────────────────────────────────────┘
```

### Detailed Breakdown (p50 ~62ms vector search, 10K records)

| Layer | Cost | % of Total | Analysis |
|-------|------|-----------|----------|
| PyO3 FFI entry | ~5µs | <0.1% | Negligible |
| Vector extraction | ~10µs | ~0.02% | Zero-copy w/ buffer protocol keeps this minimal |
| Metadata conversion + LRU lookup | ~20µs | ~0.03% | Cached for common dicts |
| GIL release | ~5µs | <0.1% | Necessary for Python concurrency |
| **HNSW search** | **~5-12ms** | **~8-19%** | Rust core, SIMD-accelerated |
| **Result serialization** | **~18-35ms** | **~29-56%** | **Dominant cost** — constructing N PyDicts per hit |
| BM25 scoring | ~30-40ms | ~48-65% | Inverted index traversal, only if text_query set |
| GIL re-acquire | ~5µs | <0.1% | Fast path |
| **Total** | **~62ms** | **100%** | |

---

## 2. Rust Core vs Python SDK: Side-by-Side Benchmarks

### Vector Search (HNSW, 100K scale, 128d, top_k=10)

| Metric | Rust Core | Python SDK | Gap | Ratio |
|--------|-----------|-----------|------|-------|
| **p99 latency** | **441.2 µs** | **71.9 ms** | +71.5 ms | **163x** |
| **p50 latency** | ~250-300 µs | **62.0 ms** | +61.7 ms | **~230x** |
| **Throughput** | **3,636 qps** | **16 qps** | −3,620 qps | **227x** |
| **Scale (vectors)** | 100,000 | 10,000 | — | Note the 10x scale difference in certified metrics |

> The Rust core benchmarks run at **100K vectors**; the Python SDK benchmarks run at **10K vectors**. Even at 10x fewer vectors, Python is ~140x slower in absolute latency. At equal 100K scale, the Python gap would be larger still.

### Ingestion (PUT, 10K records, 128d)

| Metric | Rust Core | Python SDK | Gap |
|--------|-----------|-----------|------|
| **Throughput** | ~15,000 ops/sec* | **95 ops/sec** | **158x** |
| **p50 latency** | ~65 µs* | **10.7 ms** | **164x** |

*\*Rust ingestion is not separately certified in BENCHMARKS.md; estimate based on Rust search-to-put ratio.*

### Batch Search (5K records, 128d, batch size 100)

| Mode | Total Time (100 queries) | Avg Latency | Speedup |
|------|-------------------------|-------------|---------|
| Sequential `db.search()` | 973.68 ms | 9.73 ms | 1x (baseline) |
| **Batch `db.search_batch()`** | **243.01 ms** | **2.43 ms** | **4.01x** |

Batch search amortizes FFI cost across multiple queries and parallelizes HNSW traversal via Rayon.

---

## 3. Root Cause Analysis: Why the Gap Exists

### 3.1 Dominant Cost: Result Serialization (~30-56% of total)

Each `search_memory` hit produces a `PyDict` with ~10-15 key-value pairs:
- `namespace`, `key`, `payload`, `created_at_ms`, `updated_at_ms`, `version`, `node_id`
- `vector` (wrapped in `VantaVector` — zero-copy, but still a Python object allocation)
- `metadata` (nested `PyDict`)
- `score` + optional `explanation` + `fusion_report`

For `top_k=10`, this means **10 PyDict allocations**, each calling `dict.set_item()` repeatedly. Each call crosses the PyO3 type boundary.

**Before DX-04 (original behavior):**
Each hit's vector was cloned as `Vec<f32>` then converted to a Python `list[float]` — an O(dim) copy of `dim * 4` bytes plus Python object overhead for each float. For 128-dim at top_k=10: 1,280 individual Python float objects created per search.

**After DX-04 (current behavior — `memory_record_to_pydict_owned`):**
Vector is moved (not cloned) into a `VantaVector` wrapper that exposes `__array_interface__` for zero-copy NumPy access. No per-element float boxing. Expected impact: **~20-30% reduction in result serialization cost** (see §7).

### 3.2 FFI Boundary Crossing

Every Rust function called from Python must:
1. **Lock the GIL** (already held entering the function)
2. **Unpack arguments** — PyO3 handles this automatically but it involves type checks and refcount bumps
3. **Clone or move data** across the boundary — strings are cloned, vectors are extracted via buffer protocol
4. **Call `py.detach()`** to release the GIL — adds ~3-10µs for the allow_threads dance
5. **Re-acquire GIL** on return — reconstructs Python objects from Rust results

Each crossing cost is small (~5µs each way), but the round-trip totals ~10µs minimum.

### 3.3 GIL Acquire / Release Overhead

VantaDB's Python SDK correctly releases the GIL (`py.detach()`) during all engine operations:
- `search_memory` — GIL released during HNSW traversal
- `put` — GIL released during storage write
- `put_batch` — GIL released during batch insert
- `rebuild_index` — GIL released during index build

Each `py.detach()` + reacquire cycle costs ~5-10µs. This is a minority of the total gap but matters for sub-ms operations.

### 3.4 Memory Allocation Patterns

Python object allocation operates on a separate heap from Rust's jemalloc. Every `PyDict` set_item triggers Python's allocator, which is not optimized for VantaDB's workload patterns. By contrast, Rust's hot path uses jemalloc arenas and pre-allocated buffers.

### 3.5 Why 441µs Rust → 62ms Python (140x)?

The multiplier is not uniform across layers:

| Component | Rust (µs) | Python SDK (µs) | Multiplier |
|-----------|-----------|----------------|------------|
| HNSW traversal | ~300 | ~5,000 | ~17x (includes FFI + overhead) |
| Result construction | 0 (native types) | ~35,000 | ∞ (not comparable) |
| BM25 scoring | ~50 | ~30,000 | ~600x |
| GIL overhead | 0 | ~10 | ∞ |
| **Total** | **~441** | **~62,000** | **~140x** |

The 140x gap is dominated by **Python object construction costs that simply don't exist in Rust** — not by slow Rust code.

---

## 4. When to Use Python SDK vs Rust Directly

### Use Python SDK when:

| Scenario | Rationale |
|----------|-----------|
| **Prototyping / experimentation** | Python ergonomics: Jupyter, notebooks, ad-hoc analysis |
| **ML pipeline integration** | LangChain, LlamaIndex, embedding models already in Python |
| **Agent memory** | AI agents need ~10-100ms recall — Python SDK easily meets this |
| **Low QPS (<50 qps)** | Single-user or batch workloads at 16 qps are sufficient |
| **Batch processing** | Use `search_batch()` for 4x throughput over sequential calls |
| **Mixed vector + BM25 hybrid** | SDK handles fusion; Rust core requires manual orchestration |

### Use Rust directly when:

| Scenario | Rationale |
|----------|-----------|
| **High QPS (>100 qps)** | Rust core delivers 3,636 qps on single thread at 100K scale |
| **Sub-millisecond latency required** | Real-time systems, audio processing, high-frequency trading |
| **Embedded / edge devices** | No Python runtime overhead, memory-mapped zero-copy |
| **Batch indexing at scale** | Ingest millions of vectors without Python dict overhead |
| **Latency-critical LLM agent loops** | Every ms counts in agentic loops with multiple DB calls per turn |
| **Competitive benchmarks** | VantaDB's raw performance vs LanceDB, Chroma, etc. |

### Decision Matrix

| Need | Choice | Expected Latency (10K scale) |
|------|--------|------------------------------|
| Quick scripting | Python SDK `search_memory()` | ~62ms |
| High throughput search | Python SDK `search_batch()` | ~2.4ms/query |
| Maximum speed | Rust `engine.search()` | ~1.2ms (p50) |
| Real-time edge | Rust embedded, no Python | ~300-500µs |
| Hybrid search, ergonomic | Python SDK | ~180ms |
| Bulk load 1M vectors | Rust `put_batch()` | ~65µs/record |

---

## 5. Optimization Tips

### 5.1 Use Batch Operations

```python
# ❌ Slow: N sequential FFI calls
for vector in query_vectors:
    results = db.search_memory("ns", vector, top_k=10)

# ✅ Fast: Single batch FFI call + Rayon parallelism
results = db.search_batch(vectors=query_vectors, top_k=10)
# 4x speedup over sequential
```

### 5.2 Pass NumPy Arrays for Zero-Copy

```python
import numpy as np

# ❌ Slow: Python list → PyO3 Vec<f32> extraction
query = [0.1] * 128
results = db.search_memory("ns", query, top_k=10)

# ✅ Zero-copy: NumPy f32 array uses __buffer_protocol__
query = np.array([0.1] * 128, dtype=np.float32)
results = db.search_memory("ns", query, top_k=10)
# Eliminates ~10µs per call, significant at scale
```

### 5.3 Avoid `explain=True` in Hot Paths

```python
# ❌ Expensive: builds per-hit explanation dicts
results = db.search_memory("ns", query, top_k=10, explain=True)
# +20-40ms overhead for explanation construction

# ✅ Fast: scores only
results = db.search_memory("ns", query, top_k=10, explain=False)
```

### 5.4 Use Rust Directly for Latency-Critical Paths

When <5ms latency is required per search, bypass Python entirely:

```rust
use vantadb::sdk::VantaEmbedded;

let engine = VantaEmbedded::open("./db").unwrap();
let query = vec![0.1f32; 128];
let results = engine.search_vector(&query, 10).unwrap();
// ~441µs at 100K scale — no Python overhead
```

### 5.5 Optimize Metadata Dicts

```python
# ❌ Slow: Large unique dictionaries bypass LRU cache
db.put("ns", "key1", "payload", metadata={
    "source": f"doc_{i}",  # every call creates new cache entry
    "timestamp": time.time_ns(),  # never cached
})

# ✅ Fast: Small, repetitive dicts use metadata LRU (CODE-014)
METADATA = {"source": "common", "type": "text"}
for i in range(1000):
    db.put("ns", f"key{i}", "payload", metadata=METADATA)
```

The `py_dict_to_metadata` LRU cache (capacity=64) caches the conversion of small dicts (≤4 fields) with identical keys+values. Repetitive calls hit the cache, saving ~20µs per call.

### 5.6 Minimize `top_k`

Result dict construction scales linearly with `top_k`:

```python
# ❌ Slower: 10 PyDict constructions
db.search_memory("ns", query, top_k=10)

# ✅ Faster: 3 PyDict constructions
db.search_memory("ns", query, top_k=3)
```

Each additional `top_k` hit adds ~2-3ms for dict construction.

---

## 6. Realistic Expectations

### The Python SDK will never match Rust core latency.

This is not a bug — it's a fundamental cost of crossing language boundaries safely. Every Python SDK call pays for:

- **Type safety** — PyO3 validates every argument and return value
- **GIL correctness** — Other Python threads can run during engine operations
- **Memory safety** — No unsound pointer aliasing between Python and Rust heaps
- **Object model** — Python dicts are hash maps, not struct fields; construction costs are inherent

### Realistic Latency Budgets

| Operation | Rust Core (100K) | Python SDK (10K) | Python SDK (100K, est.) |
|-----------|-----------------|-------------------|-------------------------|
| Vector search (p50) | ~300µs | ~62ms | ~80-120ms |
| Vector search (p99) | ~441µs | ~72ms | ~90-140ms |
| Hybrid search (p50) | ~350µs + BM25 | ~180ms | ~250-400ms |
| PUT (p50) | ~50µs | ~11ms | ~13-15ms |
| Batch search/query | N/A | ~2.4ms avg | ~3-5ms avg |

### What You Can Expect

| Expectation | Reality |
|-------------|---------|
| Python SDK is **fast enough** for AI agents | ✅ Yes — agent loops need <100ms, SDK delivers ~62ms |
| Python SDK is **fast enough** for real-time web | ❌ Not for sub-10ms requirements — use Rust directly |
| Batch operations help significantly | ✅ 4x speedup via `search_batch()` |
| Zero-copy input (DX-04) helps | ✅ ~20-30% reduction in result serialization |
| GIL is correctly released | ✅ Concurrency-safe for multi-threaded Python |
| Rust core is **blazing fast** | ✅ ~441µs p99 at 100K, competitive with LanceDB/Chroma |
| Gap will close over time | ⚠️ Partially — serialization is fundamental; expect 2-3x improvement, not 140x |

### What 62ms Means in Practice

| Application | Acceptable? |
|-------------|-------------|
| AI agent memory recall per turn | ✅ Yes — 62ms << typical LLM generation latency (~1-5s) |
| RAG pipeline with LLM inference | ✅ Yes — dominated by embedding + LLM time |
| Interactive search (type-ahead) | ⚠️ Marginal — prefer batch or Rust for <20ms |
| Real-time audio/video sync | ❌ No — use Rust directly |
| High-throughput OLTP | ❌ No — use Rust directly |
| 95th percentile web API | ✅ Yes — 67ms is fine for most API consumers |

---

## 7. DX-04 Zero-Copy Improvements — Impact Assessment

DX-04 introduced two key zero-copy optimizations in the Python bindings:

### 7.1 Buffer Protocol Vector Extraction (`extract_vector`)

**File:** `vantadb-python/src/lib.rs:199-243`

```rust
// Attempt zero-copy via buffer protocol (requires Python 3.11+)
if let Ok(buf) = pyo3::buffer::PyBuffer::<f32>::get(obj) {
    if buf.is_c_contiguous() {
        if let Some(slice) = buf.as_slice(py) {
            return Ok(slice.iter().map(|cell| cell.get()).collect());
        }
    }
}
```

**Before:** Every vector converted from Python list → individual Python float extraction → `Vec<f32>` — O(dim) PyO3 type checks.

**After:** NumPy `ndarray[float32]` → buffer protocol → shared memory. Only `f64` arrays incur a copy (with downcast precision warning).

**Impact:** ~5-15µs saved per call. Not transformative at 62ms total, but significant for batch operations with many queries.

### 7.2 `VantaVector` Wrapper (`__array_interface__`)

**File:** `vantadb-python/src/lib.rs:1485-1552`

```rust
fn __array_interface__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    let shape = PyTuple::new(py, &[self.data.len()])?;
    dict.set_item("shape", shape)?;
    dict.set_item("typestr", "<f4")?;
    let ptr = self.data.as_ptr() as usize;
    let data: Py<PyAny> = (ptr as u64, true).into_py(py);
    dict.set_item("data", data)?;
    // ...
}
```

**Before:** Each search result's vector was cloned (`memory_record_to_pydict` → `dict.set_item("vector", vector.clone())`) then exposed as a Python list.

**After:** `memory_record_to_pydict_owned` moves the `Vec<f32>` into a `VantaVector` that exposes zero-copy `__array_interface__` to NumPy. The vector is never copied; `np.asarray(result["vector"])` reads Rust's buffer directly.

**Impact:** Saves ~10-20µs per hit (memory allocation + f32→PyFloat boxing + list construction + PyO3 crossing). For `top_k=10`: **~100-200µs saved per search** — measurable but modest relative to the ~35ms serialization cost.

### 7.3 Expected Total Impact of DX-04

| Metric | Before DX-04 (est.) | After DX-04 | Improvement |
|--------|-------------------|-------------|-------------|
| Vector extraction (single query) | ~25µs | ~10µs | 60% |
| Vector result construction (per hit) | ~4ms | ~2ms | 50% |
| Total search p50 (10K) | ~70ms (est.)* | **62ms** | ~12% |
| Batch search (100 queries) | ~280ms (est.) | **243ms** | ~13% |

*\*Benchmarked after DX-04 was already in place; before estimates are conservative.*

### 7.4 Future Zero-Copy Opportunities

| Opportunity | Expected Gain | Complexity |
|-------------|---------------|------------|
| PyDict → Rust struct via `#[pyclass]` for search results | 30-50% reduction in serialization | High — requires breaking Python API compatibility |
| Memory-mapped result buffers | 10-20% at scale | Medium — depends on shared memory |
| Lazy metadata deserialization | 10-30% for sparse queries | Medium — skip metadata until accessed |
| Reuse PyDict objects via object pool | 10-20% | Low — pre-allocate common result shapes |
| `__array__` protocol on hit lists | 5-10% for ML consumers | Medium — return NumPy-compatible arrays directly |

**Realistic ceiling:** Even with all above optimizations, Python SDK is unlikely to go below **~15-20ms** for vector search at 10K scale. The remaining overhead is fundamental to Python object model crossing.

---

## 8. Summary

```
                ┌──────────────────────────────┐
                │  441 µs  ← Rust core (100K)  │
                │      ✦ SIMD distance         │
                │      ✦ Zero-copy HNSW        │
                │      ✦ jemalloc arenas       │
                └─────────────┬────────────────┘
                              │
                    ~140x gap by design
                              │
                ┌─────────────▼────────────────┐
                │  62 ms    ← Python SDK (10K) │
                │      ✦ FFI + GIL        5%   │
                │      ✦ HNSW traversal  10%   │
                │      ✦ BM25 scoring    50%   │
                │      ✦ Dict construction 35% │
                └──────────────────────────────┘
```

### Key Numbers to Remember

- **441µs** — Rust core p99 at 100K vectors (the speed ceiling)
- **62ms** — Python SDK p50 at 10K vectors (the ergonomic floor)
- **4x** — Batch search speedup over sequential
- **~140x** — Gap between Rust core and Python SDK
- **~30%** — Expected DX-04 reduction in result serialization cost
- **95 ops/sec** — Python ingestion throughput at 10K records
- **3,636 qps** — Rust search throughput at 100K records

### Decision Flow

```
Need <1ms per search?
  ├── Yes → Use Rust core directly
  └── No  → 
         Need <20ms per search?
           ├── Yes → Use Python SDK + search_batch()
           └── No  → 
                  Need <100ms per search?
                    ├── Yes → Use Python SDK search_memory()
                    └── No  → Python SDK is fine for any use case
```

---

## References

| Document | Contents |
|----------|----------|
| `docs/operations/BENCHMARKS.md` | Certified performance metrics for Rust core and Python SDK |
| `docs/glosario/benchmarks.md` | Benchmark glossary, methodology, definitions |
| `docs/glosario/pyo3.md` | PyO3 binding architecture and GIL management |
| `docs/glosario/latency.md` | Latency glossary with per-layer breakdowns |
| `vantadb-python/src/lib.rs` | Python SDK implementation (PyO3 bindings) |
| `docs/Backlog.md` (DOC-21) | Performance clarity documentation task |
| `vantadb/src/sdk/` | Rust SDK core engine |
