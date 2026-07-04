---
title: "Benchmarks"
type: glossary-entry
status: stable
tags: [performance, testing, metricas]
last_refined: 2026-06
links: "[[README.md]]"
aliases: [Performance Testing, Benchmarking]
---
# Benchmarks

##Definition

**Benchmarks** are **standardized performance tests** that measure quantitative metrics (latency, throughput, recall) under controlled conditions, allowing objective comparisons and detection of regressions.

## Key Metrics in VantaDB

### Latencia

| Percentil | Significado | Objetivo |
|-----------|-------------|----------|
| **p50** (mediana) | 50% de requests más rápidos | <20ms |
| **p95** | 95% de requests más rápidos | <50ms |
| **p99** | 99% de requests más rápidos | <100ms |

### Throughput

| Operación | Métrica | Objetivo |
|-----------|---------|----------|
| **Ingesta** | ops/segundo | >100 ops/s |
| **Search** | queries/segundo | >50 qps |
| **Batch search** | queries/segundo | >200 qps |

### Recall (Precision)

$$
\text{Recall@K} = \frac{|\text{Top-K recuperados} \cap \text{Top-K reales}|}{K}
$$

| Dataset | Objetivo | Actual |
|---------|----------|--------|
| SIFT1M (10K) | ≥0.95 | 0.956 |
| SIFT1M (50K) | ≥0.95 | 1.000 |
| SIFT1M (100K) | ≥0.95 | 0.998 |

## Implementation in VantaDB

### Script de Benchmark

```python
# benchmarks/vantadb_local_bench.py
import time
from vantadb import VantaEmbedded

def benchmark_search(db, vectors, top_k=10, iterations=1000):
    latencies = []
    
    for vector in vectors[:iterations]:
        start = time.perf_counter()
        results = db.search(vector=vector, top_k=top_k)
        elapsed = time.perf_counter() - start
        latencies.append(elapsed * 1000) # ms
    
    latencies.sort()
    return {
        "p50": latencies[len(latencies) // 2],
        "p95": latencies[int(len(latencies) * 0.95)],
        "p99": latencies[int(len(latencies) * 0.99)],
        "throughput": iterations / sum(latencies) * 1000
    }
```

### Benchmark Results

```json
{
  "dataset": "SIFT1M",
  "num_vectors": 100000,
  "dimensions": 128,
  "search": {
    "p50_ms": 6.1,
    "p95_ms": 12.4,
    "p99_ms": 18.7,
    "throughput_qps": 164
  },
  "recall_at_10": 0.998
}
```

## CI/CD integration

```yaml
# .github/workflows/benchmarks.yml
name: Benchmarks
on:
  push:
    branches: [main]
  schedule:
    - cron: '0 0 * * 0'  # Semanal

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: python benchmarks/run.py --suite standard
      - uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: benchmark_results.json
```

## See Also

- [[hnsw]] — Main benchmarked component
- [[ci-cd]] — Benchmarks integrated into CI
- [[chaos-testing]] — Supplementary robustness testing

---

*Benchmarks transform subjective claims into objective and reproducible data.*

