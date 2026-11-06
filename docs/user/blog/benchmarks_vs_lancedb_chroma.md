---
title: "VantaDB vs LanceDB vs ChromaDB: Real Numbers from an Embedded Engine"
kind: howto
status: active
description: A historical vector-search benchmark snapshot, its methodology, and limits on comparisons with current releases
tags: [benchmarks, vector-database, performance]
---

# VantaDB vs LanceDB vs ChromaDB: Real Numbers from an Embedded Engine

*By the VantaDB Team*

> **Historical snapshot:** the table below is retained from the earlier article,
> whose metadata identified VantaDB 0.5.0. Its ingest notes refer to runs after
> July 2026; that conflicts with the original June publication date. The exact
> tested build is therefore not established by this article. These are
> previously reported results, not measurements of v0.9.0 performed in this
> update. Consult [the benchmark reference](../operations/BENCHMARKS.md) for
> curated runs and their declared versions.

Every vector database blog post eventually faces the same question: *"OK, but how fast is it, really?"* Latency, throughput, recall, memory — these are the numbers engineers check before they trust a storage engine with their data.

So we built a public, reproducible benchmark harness (`benchmarks/competitive_bench.py`) that pits **VantaDB** against **LanceDB** and **ChromaDB** on the same machine, the same dataset, and the same queries. This post walks through the methodology, the numbers from a real run, and the parts of an embedded engine that no QPS table can capture.

---

## 1. Methodology: The Part Everyone Skips

A benchmark is only as trustworthy as its methodology. Here is exactly what we ran, so you can reproduce it on your own hardware:

* **Dataset:** `glove-100-angular` from ann-benchmarks — 100-dimensional GloVe word embeddings, cosine metric.
* **Scale:** 10,000 vectors ingested, 100 query vectors, `top_k=10`.
* **Ground truth:** computed **locally** via exact brute-force numpy search over the 10K subset (the HDF5 file ships pre-computed neighbors for the full 1.18M dataset, which is not valid for a subset).
* **Iterations:** 3 runs per engine, median reported.
* **Ingest mode:** chunked inserts (`--batch-size 999`), so VantaDB's Ingest timer is not inflated by a hidden full HNSW rebuild. Earlier published numbers (pre-Jul-31-2026) used a single `put_batch_raw` call that double-counted index construction — **those numbers are not directly comparable** to this run.
* **Engine configuration:** LanceDB used its default IVF-PQ index; ChromaDB used its incremental HNSW (indexing happens during inserts); VantaDB used its memory-mapped HNSW with a full `rebuild_index`.

**Honesty note:** this run happened on a development laptop flagged by the harness's own health check — CPU load was ~85%, with 17 VS Code processes open. That means the absolute latencies below are inflated by a noisy machine. Run the benchmark on a quiet box for clean absolute numbers; the *relative* ordering and the recall story are the signal here. ChromaDB also completed 1 of 3 iterations on Windows (a file-lock on its cleanup path), so treat its median as a single-run estimate.

The harness is available for a new run; execution time depends on the dataset,
installed dependencies, hardware, and engine configurations:

```bash
pip install -r benchmarks/requirements.txt
python benchmarks/competitive_bench.py --dataset glove-100-angular --size 10000 --queries 100
```

---

## 2. The Table

Median of the runs described above:

| Engine   | Ingest QPS | Index Time (ms) | Query QPS | Latency p50 (ms) | Latency p99 (ms) | Recall@10 | Peak RSS (MB) | Delta RSS (MB) |
|----------|-----------:|----------------:|----------:|-----------------:|-----------------:|----------:|--------------:|---------------:|
| VantaDB  |      301.5 |         7,330.1 |     241.4 |             4.124 |             6.129 | **100.00%** |        434.4 |          204.0 |
| LanceDB  |   92,294.1 |         3,087.0 |     197.5 |             4.978 |             8.953 |     22.80% |        390.7 |           24.5 |
| ChromaDB |    2,227.6 |    N/A (inc)     |     591.1 |             1.650 |             2.744 |     95.60% |        386.5 |           32.8 |

---

## 3. Reading the Table Honestly

### Recall: the number that actually matters

A vector store that returns wrong neighbors fast is a vector store you can't use. Recall@10 measures how many of the *true* top-10 neighbors (from exact brute-force search) each engine actually returns:

* **VantaDB: 100.00%** — a fresh HNSW rebuild over 10K vectors recovers the exact top-10.
* **ChromaDB: 95.60%** — incremental HNSW with negligible recall loss.
* **LanceDB: 22.80%** — this is the classic **IVF-PQ tradeoff**: product-quantization compresses vectors aggressively for fast, memory-slim scans, but it pays for that compression in recall. LanceDB's ingest speed (`92,294 QPS`) is the other side of that same coin.

These results describe the tested index configurations, not each engine's
maximum achievable recall. Different tuning, quantization, or search budgets
can change both recall and latency.

### Query QPS and latency: directional, not absolute

VantaDB's `241.4 QPS` beat LanceDB (`197.5 QPS`) on this run and trails ChromaDB (`591.1 QPS`). Two things matter here:

1. **The machine was loaded** (see §1), so treat all three as slower than their quiet-box numbers.
2. **The bottleneck is not isolated here.** Comparing separate Rust and Python
   harnesses does not measure FFI overhead. That attribution requires
   matched-workload profiling. Batch-search measurements are documented in
   [the benchmark reference](../operations/BENCHMARKS.md).

This vector-search run does not test crash recovery. ChromaDB's
`PersistentClient` is a persistent embedded option; an in-memory index is
not evidence that its canonical records lack persistence.

### Memory: smaller than you'd expect

LanceDB's PQ index makes its working set smaller — and it shows: `390.7 MB` peak RSS. VantaDB sits at `434.4 MB` for a full, uncompressed graph plus the LSM store. That's the price of recall-first indexing, and it still fits comfortably on a laptop alongside a quantized LLM.

---

## 4. What a QPS Table Can't Measure

Benchmarks measure *queries per second*. Agent memory workloads care about a different set of properties, and this is where the numbers stop being the whole story:

* **Durability.** VantaDB provides checksummed WAL and recovery mechanisms.
  Sync configuration and failure conditions matter; this benchmark does not
  establish crash immunity for any engine. See [WAL integrity](../../api/WAL_INTEGRITY.md).
* **Hybrid retrieval in one engine.** BM25 for exact keywords, HNSW for semantics, Reciprocal Rank Fusion to merge them — no external search index, no join of two virtual tables, no normalization math in application code. A vector-only benchmark can't even represent this workload.
* **Single-process distribution.** `pip install vantadb-py` uses a prebuilt
  wheel where available. Source builds require Rust and the C/C++ toolchain
  needed by `croaring`. VantaDB stores a database directory, not one SQLite-like
  file; LanceDB and ChromaDB also offer embedded operation.
* **Memory-mapped indexing.** Memory mapping lets the operating system manage
  resident index pages. It does not eliminate RAM requirements or establish a
  memory advantage over another engine without a matched measurement.

Long-running reliability and power-loss recovery require separate tests.

---

## 5. Reproduce It Yourself

Don't take our word for any of this. The harness is public, parameterized, and prints a health check before it runs so you can see if your machine is too noisy to trust:

```bash
git clone https://github.com/ness-e/Vantadb
cd Vantadb
pip install -r benchmarks/requirements.txt
python benchmarks/competitive_bench.py --dataset glove-100-angular --size 10000 --queries 100
```

Try `--dataset sift-128-euclidean` or bump `--size 100000` — the same harness, your hardware, your numbers. If you find a configuration where we regress, we want the bug report.

---

## Conclusion

The historical table reports recall@10 of 100% for VantaDB in the tested
configuration, with both wins and losses elsewhere. It does not establish
general superiority, equal tuning, or present-day performance for any engine.

Install the Python binding, then use the harness command above to produce a new
benchmark report:

```bash
pip install vantadb-py
```

Join the community on Discord and star the [VantaDB repository](https://github.com/ness-e/Vantadb). To understand how the engine fuses lexical and semantic search under the hood, read [How Hybrid Search Works](./how_hybrid_search_works.md).
