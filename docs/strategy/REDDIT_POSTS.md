---
title: Reddit Launch Posts — VantaDB
type: marketing
status: ready-to-publish
tags: [vantadb, launch, reddit, marketing]
last_reviewed: 2026-09-02
---

# Reddit Launch Posts

> **Task:** MKT-04 — Reddit posts for r/rust, r/MachineLearning, r/LocalLLaMA
> **Effort:** 🟢 2-4h
> **Estado 2026-09-02 (MKT-04):** 3 drafts corregidos y validados con datos medibles (abajo), **NO publicados** — `status: ready-to-publish`. Publicación pendiente (owner humano tiene identidad Reddit). **✅ Claims corregidos:** "zero deps"/"pure Rust" → Fjall default es pure-Rust; feature `roaring` (default) trae `croaring-sys`/`cc` que compila C/C++; `rocksdb` backend opcional también es C++. "recall>0.998 SIFT1M" → verificado: Stress Protocol recall@10 **0.9560 @ 10K**, **0.9980 @ 10K (scaling)**, **1.0000 @ 50K**, **0.9980 @ 100K** (`docs/operations/BENCHMARKS.md` §1); SIFT1M 100K p99 **441.2 µs** balanced / **1.23 ms** high-recall (`docs/operations/BENCHMARKS.md` §5). "sub-ms core search" → verificado: core Rust p50 **1.2 ms @ 10K**, **6.1 ms @ 50K** (`docs/operations/BENCHMARKS.md` §1); Python SDK vector search p50 **62 ms @ 10K** (incluye PyO3/GIL). "100% GloVe-100-angular 1.18M" → **sin medición**; competitive bench midió subset 10K con recall@10 **24.5%** (`docs/operations/BENCHMARKS.md` §7) — claim eliminado.

---

## 1. r/rust — Technical Deep Dive

**Title:** VantaDB — Embedded hybrid (BM25 + HNSW) search engine in Rust

**Post:**

Over the past few months I've been building [VantaDB](https://github.com/ness-e/Vantadb), an embedded hybrid search engine in Rust. Think SQLite-for-vectors: a single library you link, no server, no containers.

**Why Rust?** Because every existing embedded vector store (Chroma, etc.) is Python-first with C++ under the hood, making cross-platform distribution a nightmare. Rust gives us:
- Static linking via PyO3 → `pip install` that works on Windows/Mac/Linux with zero toolchain
- `memmap2` for zero-copy HNSW graph traversal
- SIMD (AVX2/NEON) via `wide::f32x8` for distance computation
- `failpoints` for chaos-tested crash recovery

**Architecture highlights:**
- HNSW with BFS topological layout compaction (anti-locality optimization to reduce page faults)
- BM25 via custom position-aware tokenizer (not Tantivy — lighter footprint for agent memory use case)
- Volcano-style CBO planner that pushes down relational filters before graph traversal
- RRF fusion for deterministic hybrid ranking

**The numbers (verified, Stress Protocol + SIFT1M subset):**
- Recall@10: **0.9560 @ 10K**, **0.9980 @ 10K (scaling)**, **1.0000 @ 50K**, **0.9980 @ 100K** vectors, 128d Cosine (`docs/operations/BENCHMARKS.md` §1)
- Core Rust p50 latency: **1.2 ms @ 10K**, **6.1 ms @ 50K** (`docs/operations/BENCHMARKS.md` §1)
- SIFT1M 100K p99: **441.2 µs** (balanced Cosine), **1.23 ms** (high-recall Cosine) (`docs/operations/BENCHMARKS.md` §5)
- Python SDK vector search p50: **62 ms @ 10K** (includes PyO3/GIL boundary) (`docs/operations/BENCHMARKS.md` §2)
- Cross-platform Python wheels (Linux x86_64/aarch64, macOS x86_64/arm64, Windows x86_64)

**Dependencies note:** Default backend Fjall is pure-Rust. Feature `roaring` (enabled by default) pulls `croaring-sys`/`cc` which compiles C/C++; `rocksdb` feature (optional) also requires C++ toolchain.

**Code:** https://github.com/ness-e/Vantadb (Apache 2.0)

Happy to answer questions about the HNSW implementation, the CBO planner, or why we chose Fjall over RocksDB as the default LSM backend.

---

## 2. r/MachineLearning — For ML Practitioners

**Title:** VantaDB — Open-source embedded memory for local-first AI agents (Rust + Python)

**Post:**

I built [VantaDB](https://github.com/ness-e/Vantadb) because existing options for agent memory are either:
1. **Cloud vector DBs** → latency, cost, offline-fail
2. **SQLite + extensions** → works but C++ distribution is painful cross-platform
3. **In-memory** → not persistent

VantaDB is an embedded hybrid search engine in Rust that does BM25 + HNSW + filters in one process, with Python bindings via PyO3 (pre-built wheels, no local compile step).

**Quick start:**
```python
import vantadb_py
db = vantadb_py.VantaDB("./agent_memory")
record = db.put(namespace="chat", key="msg_1", vector=[...], payload="Hello world")
results = db.search_memory(namespace="chat", query_vector=[...], text_query="hello", top_k=5)
```

**Key features:**
- Apache 2.0, fully open source
- Embedded — no server, no containers, single `pip install` (pre-built wheels)
- Hybrid search (BM25 lexical + HNSW vector + metadata filters)
- GIL-released batch search via Rayon (`search_batch` 4× speedup vs sequential)
- WAL with CRC32C + automated chaos testing (failpoint injection)
- Cross-platform wheels (Linux x86_64/aarch64, macOS x86_64/arm64, Windows x86_64)

**Performance (verified):**
- Ingestion: ~95 ops/sec @ 10K records (`docs/operations/BENCHMARKS.md` §2)
- Vector search (Python SDK): p50 **62 ms**, p99 **72 ms** @ 10K, top_k=10
- Hybrid search (RRF): p50 **180 ms**, p99 **211 ms** @ 10K
- Batch search (`search_batch`): **4× faster** than sequential, avg **2.4 ms/query** @ 5K records (`docs/operations/BENCHMARKS.md` §6)

Would love feedback from folks building local AI agent systems!

---

## 3. r/LocalLLaMA — For Local AI Community

**Title:** VantaDB — Persistent memory for local LLM agents (Rust, open source)

**Post:**

If you're running Ollama, llama.cpp, or any local LLM setup, you've probably hit the "agent memory" problem: how do you persist embeddings + text + metadata in a way that survives reboot, doesn't need a cloud API, and actually searches well?

I built [VantaDB](https://github.com/ness-e/Vantadb) for exactly this use case.

**Why not just use Chroma or SQLite?**
- Chroma is Python-first and carries significant overhead for embedded use
- SQLite + vector extensions require compiling C++ across platforms
- Both lack native hybrid search (BM25 + vectors fused at the planner level)

**VantaDB in one line:** `pip install vantadb-py`, import it, point it at a directory, done.

**Local-first design:**
- Fjall LSM storage (pure-Rust default; feature `roaring` adds C/C++ build via `croaring-sys`, `rocksdb` feature optional)
- HNSW via memmap2 (OS-managed paging, no manual cache tuning)
- BM25 tokenizer optimized for short agent messages
- GIL-released for multi-threaded batch search
- Crash recovery tested with injected failpoints at WAL/storage/HNSW levels

**Stack:** Rust core → PyO3 bindings → Python SDK (WASM/TS SDK also available)

**Performance (verified):**
- Core Rust vector search p50: **1.2 ms @ 10K**, **6.1 ms @ 50K** (`docs/operations/BENCHMARKS.md` §1)
- Python SDK vector search p50: **62 ms @ 10K** (includes FFI/GIL) (`docs/operations/BENCHMARKS.md` §2)
- SIFT1M 100K p99: **441 µs** balanced / **1.23 ms** high-recall (`docs/operations/BENCHMARKS.md` §5)
- Batch search 4× speedup via Rayon GIL release (`docs/operations/BENCHMARKS.md` §6)

**Repo:** https://github.com/ness-e/Vantadb (Apache 2.0)

Curious what local agent architectures people are running and what memory patterns you've needed!