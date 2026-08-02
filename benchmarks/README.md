# VantaDB Benchmark Suite

Reproducible performance benchmarks for the VantaDB Python SDK (`vantadb_py`).

## Quick start (standalone, public path)

No Rust toolchain required — `vantadb_py` is installed from PyPI.

```bash
python -m venv .venv-bench
# Windows: .venv-bench/Scripts/activate   |   Linux/macOS: source .venv-bench/bin/activate

pip install -r benchmarks/requirements.txt

# Local benchmark (BENCH-01): ingestion + BM25 + HNSW + hybrid RRF on synthetic data
python benchmarks/vantadb_local_bench.py --size 10000 --queries 1000 --output report.json
```

The script exports a JSON report with `insert` / `rebuild` / `query_text` /
`query_vector` / `query_hybrid` keys. For a quick sanity check use small values
(`--size 1000 --queries 50`).

## Competitive benchmark (VantaDB vs LanceDB vs ChromaDB)

`competitive_bench.py` requires the extra dependencies already included in
`benchmarks/requirements.txt` (`numpy`, `h5py`, `lancedb`, `chromadb`,
`psutil`, `tabulate`). It auto-downloads the `glove-100-angular` /
`sift-128-euclidean` datasets from ann-benchmarks on first run (large, ~1 GB).

```bash
python benchmarks/competitive_bench.py --dataset glove-100-angular --size 10000 --queries 100
```

> [!NOTE]
> Dataset downloads are slow. The default dataset for quick local runs is
> synthetic; pass `--dataset` to opt into real ann-benchmarks data.

## Local development variant (maturin)

Benchmarking against an un-released source tree requires building the PyO3
bindings once:

```bash
maturin develop --manifest-path vantadb-python/Cargo.toml --release
python benchmarks/vantadb_local_bench.py --size 10000 --queries 1000 --output report.json
```

The scripts accept either install path (PyPI wheel or `maturin develop`).

## Published results

- Latest CI results: [`docs/operations/BENCHMARKS.md`](../docs/operations/BENCHMARKS.md)
- CI badge: `perf-bench-40` workflow (see GitHub Actions)

## Scripts

| Script | Purpose |
| :--- | :--- |
| `vantadb_local_bench.py` | BENCH-01: ingestion + lexical/vector/hybrid search latencies (zero-dep besides `vantadb_py`) |
| `competitive_bench.py` | VantaDB vs LanceDB vs ChromaDB (ingestion, QPS, latency, recall, RSS) |
| `batch_vs_sequential_bench.py` | `search_batch()` vs sequential `search()` FFI amortization |
| `prefetch_comparison.py` | Predictive kernel prefetch impact (SCALE-01) |
| `wasm_bench.mjs` | WASM build benchmark |
