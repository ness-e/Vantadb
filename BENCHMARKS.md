# VantaDB Benchmarks

We enforce strict internal recall checks on our algorithms to remain intellectually honest about our performance capabilities. Current tests target HNSW validation utilizing brute-force validation arrays.

## HNSW Validation Metrics

*   **Test Suite:** `tests/hnsw_recall.rs`
*   **Dimensions:** 64-dimensional float arrays.
*   **Vector Sample Size:** 5,000 vectors.
*   **Index configuration:** `m=24`, `ef_construction=200`, `ef_search=100`.

### Current State Matrix

| Engine Build | Algorithm | Recall@10 | Avg Query Latency | QPS Limit |
|--------------|-----------|-----------|--------------------|-----------|
| VantaDB v0.1 | HNSW | `96.80%` | ~2,392 µs | 410+ QPS |

*Note: The hardware testing environment utilized a 12-core execution framework with SIMD capabilities.*

## Future Benchmark Architectures
As we stabilize our codebase, we intend to publish objective comparison suites against established local persistence vector layers (e.g., SQLite `vec`). We prioritize honest, easily reproducible Python notebook methodologies that external engineers can replicate easily on consumer hardware.
