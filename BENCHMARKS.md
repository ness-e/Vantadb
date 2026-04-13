# VantaDB — HNSW Engine Benchmarks

All benchmarks use the internal **Stress Protocol** (`tests/certification/stress_protocol.rs`), a 7-block certification suite that validates recall, scaling, memory, persistence, edge cases, graph consistency, and latency.

## Methodology

### Dataset

- **Type:** Synthetic L2-normalized random vectors
- **Dimensions:** 128
- **Seed:** 2024 (deterministic, reproducible)
- **Similarity:** Cosine similarity

### HNSW Configuration

| Scale | M  | M_max0 | ef_construction | ef_search |
|-------|----|--------|-----------------|-----------|
| 10K   | 32 | 64     | 200             | 100       |
| 50K   | 32 | 64     | 400             | 200       |
| 100K  | 32 | 64     | 500             | 300       |

### Hardware

- **CPU:** 12-core, AVX2
- **RAM:** 31 GB
- **OS:** Windows 11

### Reproduction

```bash
cargo test --test stress_protocol -- --nocapture
```

## Results (Certified — April 2026)

| Scale | Recall@10 | Lat p50  | Lat p95   | Build Time | RAM      |
|-------|-----------|----------|-----------|------------|----------|
| 10K   | 0.9520    | 2.65 ms  | 3.24 ms   | 46.66s     | 10.2 MB  |
| 50K   | 0.9100    | 6.89 ms  | 8.80 ms   | 626.24s    | 51.1 MB  |
| 100K  | 0.8860    | 9.28 ms  | 10.51 ms  | 1447.17s   | 101.9 MB |

### Additional Validations

- **Ground Truth (50K):** Recall@10 = 0.9660 (brute-force verified)
- **Persistence:** Zero recall loss after serialize/deserialize cycle
- **Graph Integrity:** 0 orphan nodes, avg L0 connectivity = 51.3
- **Memory:** Linear growth (~1060 bytes/vector)

## Limitations

- Results are on synthetic random data, **not** a standard benchmark dataset (SIFT1M, etc.)
- Build times are single-threaded (no parallel insertion yet)
- External competitive benchmarks (FAISS, HNSWlib) pending — see Roadmap
