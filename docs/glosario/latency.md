---
type: glossary-entry
status: stable
tags: [glosario, métricas, latencia, performance, percentiles]
aliases: [latency, response time, p50, p95, p99]
---

# Latencia

## Definición

La **latencia** es el tiempo que transcurre entre el inicio de una operación y su completitud. En bases de datos, se mide típicamente en milisegundos (ms) o microsegundos (µs).

## Percentiles

La latencia no es un valor único; varía según la operación. Los percentiles describen la distribución:

| Percentil | Significado | Interpretación |
|-----------|-------------|----------------|
| **p50** (mediana) | 50% de operaciones son más rápidas | Experiencia típica |
| **p95** | 95% de operaciones son más rápidas | Experiencia mayoritaria |
| **p99** | 99% de operaciones son más rápidas | Peor caso común |
| **p99.9** | 99.9% de operaciones son más rápidas | Casos extremos |

### Ejemplo de Distribución

| Operación | p50 | p95 | p99 |
|-----------|-----|-----|-----|
| busqueda-vectorial (Rust) | 1.2 ms | 3.5 ms | 8.2 ms |
| busqueda-vectorial (Python) | 62.0 ms | 67.1 ms | 71.9 ms |
| Búsqueda BM25 (Python) | 115.3 ms | 127.1 ms | 137.5 ms |
| busqueda-hibrida (Python) | 179.8 ms | 191.8 ms | 211.1 ms |

**Interpretación:**
- 50% de búsquedas vectoriales en Python toman <62ms
- 95% de búsquedas toman <67.1ms
- 99% de búsquedas toman <71.9ms
- 1% de búsquedas toman >71.9ms (outliers)

## En VantaDB

### Métricas Certificadas

#### Core Rust (sin overhead de FFI)

| Dataset | p50 | p95 | p99 |
|---------|-----|-----|-----|
| 10K vectores | 1.2 ms | 2.8 ms | 5.1 ms |
| 50K vectores | 6.1 ms | 12.3 ms | 18.7 ms |
| 100K vectores | 12.4 ms | 22.1 ms | 35.6 ms |

#### Python SDK (con overhead de FFI)

| Operación | p50 | p95 | p99 |
|-----------|-----|-----|-----|
| PUT (ingesta) | 10.7 ms | 17.5 ms | 19.0 ms |
| Search (vectorial) | 62.0 ms | 67.1 ms | 71.9 ms |
| Search (BM25) | 115.3 ms | 127.1 ms | 137.5 ms |
| Search (híbrida) | 179.8 ms | 191.8 ms | 211.1 ms |

### Componentes de la Latencia

#### busqueda-vectorial en Python

```
Total: ~62 ms
├── FFI overhead (Python → Rust): ~15 ms
├── HNSW search: ~12 ms
├── Result serialization: ~10 ms
├── FFI return (Rust → Python): ~15 ms
└── GIL acquisition: ~10 ms
```

#### busqueda-hibrida en Python

```
Total: ~180 ms
├── Vector search: ~62 ms
├── BM25 search: ~115 ms
├── RRF fusion: ~3 ms
└── Overhead: ~0 ms (paralelo)
```

### Optimización de Latencia

#### 1. Liberación del [GIL](GIL.md)

```rust
// Sin liberación de GIL (bloquea Python)
fn search(&self, query: &[f32]) -> Vec<SearchResult> {
    self.engine.search(query)  // Python bloqueado durante toda la búsqueda
}

// Con liberación de GIL (permite concurrencia)
fn search(&self, py: Python, query: &[f32]) -> Vec<SearchResult> {
    py.allow_threads(|| {
        self.engine.search(query)  // Python puede ejecutar otros threads
    })
}
```

#### 2. Batch Operations

```python
# Secuencial (lento)
for query in queries:
    db.search_memory(query_vector=query, top_k=10)
# Total: N × 62 ms

# Batch (rápido, paralelizado con Rayon)
results = db.search_batch(vectors=queries, top_k=10)
# Total: N × 2.43 ms (4.01x speedup)
```

#### 3. SIMD Acceleration

```rust
// Escalar (1 operación por ciclo)
fn cosine_distance_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

// AVX2 (8 operaciones por ciclo)
unsafe fn cosine_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = _mm256_setzero_ps();
    for i in (0..a.len()).step_by(8) {
        let va = _mm256_loadu_ps(&a[i]);
        let vb = _mm256_loadu_ps(&b[i]);
        sum = _mm256_fmadd_ps(va, vb, sum);
    }
    // Horizontal sum...
}
```

**Speedup:** 8-16x en distancias vectoriales

#### 4. Memory-Mapped Files ([mmap](mmap.md))

```rust
// Sin mmap (carga completa en RAM)
let vectors: Vec<Vec<f32>> = load_from_disk()?;  // Alto uso de RAM

// Con mmap (delegado al OS)
let mmap = Mmap::open(&file)?;
let vectors: &[f32] = unsafe { 
    std::slice::from_raw_parts(mmap.as_ptr() as *const f32, len)
};
// RAM usage: ~0 (OS maneja page faults)
```

## Objetivos de Latencia

| Caso de Uso | Objetivo p50 | Objetivo p99 |
|-------------|--------------|--------------|
| **Agentes de IA** | <100 ms | <200 ms |
| **RAG pipelines** | <50 ms | <100 ms |
| **Búsqueda interactiva** | <20 ms | <50 ms |
| **Batch processing** | <500 ms | <2000 ms |

## Medición de Latencia

### Desde Python

```python
import time

# Medir latencia de una operación
start = time.perf_counter()
results = db.search_memory(query_vector=query, top_k=10)
latency_ms = (time.perf_counter() - start) * 1000

print(f"Latencia: {latency_ms:.2f} ms")
```

### Benchmark Completo

```python
import numpy as np

latencies = []
for query in queries:
    start = time.perf_counter()
    db.search_memory(query_vector=query, top_k=10)
    latencies.append((time.perf_counter() - start) * 1000)

print(f"p50: {np.percentile(latencies, 50):.2f} ms")
print(f"p95: {np.percentile(latencies, 95):.2f} ms")
print(f"p99: {np.percentile(latencies, 99):.2f} ms")
```

## Latencia vs Throughput

| Métrica | Definición | Unidad |
|---------|------------|--------|
| **Latencia** | Tiempo por operación | ms, µs |
| **Throughput** | Operaciones por segundo | ops/sec, qps |

**Relación:**
$$\text{Throughput} = \frac{1}{\text{Latencia promedio}}$$

**Ejemplo:**
- Latencia p50: 62 ms
- Throughput: 1/0.062 = 16 queries/segundo

## Véase También

- [Recall](Recall.md) - Métrica complementaria
- [Benchmarks](Benchmarks.md) - Suite de medición
- [Memory Efficiency](Memory Efficiency.md) - Uso de recursos
- [busqueda-vectorial](busqueda-vectorial.md) - Contexto de uso
