# VantaDB Benchmarks

## HNSW Stress Protocol Results (Certified)

| Scale | Recall@10 | Lat p50 | Lat p95 | Build Time | RAM |
|-------|-----------|---------|---------|------------|-----|
| 10K   | 0.9520    | 2.65ms  | 3.24ms  | 46.66s     | 10.2 MB |
| 50K   | 0.9100    | 6.89ms  | 8.80ms  | 626.24s    | 51.1 MB |
| 100K  | 0.8860    | 9.28ms  | 10.51ms | 1447.17s   | 101.9 MB |

### Methodology

**Configuration:**
- Graph limits: `M=32`, `M_max0=64`
- Dimensionality: `128D` (dense vectors)
- Distance Metrics: `Cosine Similarity`
- Index Randomization: Seeded RNG (Deterministic Graph Topology)

**Hardware Profile:**
- CPU: 12-core Logical
- Instruction Sets: AVX2 + FMA
- Host Memory: 31GB RAM
- Environment: Windows x64 Native

**Reproducibility Command:**
To run this exact benchmark and verify the metrics locally:
```bash
cargo test --test stress_protocol -- --nocapture
```
