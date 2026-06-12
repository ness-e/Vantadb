# VantaDB — HNSW & Lexical Engine Performance Benchmarks

Este documento recopila las métricas de rendimiento certificadas para **VantaDB** en su versión MVP. Cubre tanto las pruebas internas del motor en Rust (**Stress Protocol**) como la telemetría del wrapper de Python del SDK.

---

## 📊 1. Resultados de Certificación del Motor (Core Rust)

Estos resultados provienen del **Stress Protocol** (`tests/certification/stress_protocol.rs`), una suite de certificación de 7 bloques que valida la consistencia de recuperación (Recall), escalabilidad, consumo de memoria del HNSW, persistencia ante fallos y consistencia del grafo.

> [!NOTE]
> Estas métricas se ejecutan bajo la suite de certificación pesada (`heavy_certification.yml`) en un entorno controlado con soporte de instrucciones vectoriales AVX2.

### Resumen del Stress Protocol (Scale: 10K a 100K)

| Métrica | Escala / Dataset | Valor / Resultado | Estado |
| :--- | :--- | :--- | :--- |
| **Recall@10 (Block 1)** | 10K vectores, 128d, Cosine | **0.9560** | ✅ Certificado |
| **Scaling Recall (10K)** | 10K vectores, 128d | **0.9980** | ✅ Certificado |
| **Scaling Recall (50K)** | 50K vectores, 128d | **1.0000** | ✅ Certificado |
| **Scaling Recall (100K)** | 100K vectores, 128d | **0.9980** | ✅ Certificado |
| **Memory Efficiency** | Por vector (estimación HNSW) | **~1172 bytes** | ✅ Certificado |
| **Graph Consistency** | Huérfanos: 0 \| Conexiones Promedio L0 | **64.0** | ✅ Certificado |
| **Latencia p50 (10K)** | Búsqueda Vectorial HNSW | **1.2 ms** | ✅ Certificado |
| **Latencia p50 (50K)** | Búsqueda Vectorial HNSW | **6.1 ms** | ✅ Certificado |
| **Factor de Escalado** | Crecimiento de latencia (10K ➔ 50K) | **4.88x** (Sub-lineal) | ✅ Certificado |

---

## 🐍 2. Rendimiento de Operaciones del SDK (Python Wrapper)

Estas métricas representan el rendimiento del ciclo de vida del SDK de Python (`vantadb_py`) interactuando con la API persistente de base de datos (`put`, `search_memory`, `rebuild_index`).

> [!IMPORTANT]
> A diferencia de las pruebas crudas en Rust, estas métricas incluyen el costo de transición de la frontera **PyO3 (Python-Rust)** y la liberación y adquisición de locks del **GIL** (Python Global Interpreter Lock).

### Resultados del Último Benchmark Local / CI

A continuación se muestran los resultados generados de forma reproducible por la suite `benchmarks/vantadb_local_bench.py`:

<!-- BENCHMARK_METRICS_START -->
| Operación / Fase | Dataset / Configuración | Latencia p50 | Latencia p95 | Latencia p99 | Rendimiento (Throughput) |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Ingesta (`PUT`)** | 10,000 registros, 128d | **10.678 ms** | **17.490 ms** | **18.988 ms** | **95 ops/sec** |
| **Index Rebuild** | Reconstrucción híbrida (HNSW + BM25) | **93.51s** | *N/A (Lote único)* | *N/A (Lote único)* | **107 ops/sec** |
| **Búsqueda Lexical (BM25)** | 10,000 registros, `top_k=10` | **115.334 ms** | **127.139 ms** | **137.539 ms** | **9 qps** |
| **Búsqueda Vectorial (HNSW)** | 10,000 registros, `top_k=10`, 128d | **61.996 ms** | **67.065 ms** | **71.893 ms** | **16 qps** |
| **Búsqueda Híbrida (RRF)** | 10,000 registros, `top_k=10`, RRF Fusion | **179.810 ms** | **191.805 ms** | **211.059 ms** | **6 qps** |
<!-- BENCHMARK_METRICS_END -->

*(Nota: Los valores marcados con `~` son aproximaciones basadas en hardware estándar con AVX2 habilitado).*

---

## 🛠️ 3. Reproducción Local del Benchmark

Para reproducir localmente estas mediciones de rendimiento y verificar el comportamiento en tu propio hardware:

```powershell
# 1. Compilar los bindings de Python en modo release
maturin develop --manifest-path vantadb-python/Cargo.toml --release

# 2. Ejecutar la suite de benchmark parametrizada
.venv/Scripts/python benchmarks/vantadb_local_bench.py --size 10000 --dim 128 --queries 1000 --output benchmarks/vanta_benchmark_report.json
```

Este script exportará un reporte detallado con paridad de esquema en [benchmarks/vanta_benchmark_report.json](file:///c:/Users/Eros/VantaDB%20Proyect/VantaDB/benchmarks/vanta_benchmark_report.json).

---

## ⚠️ Limitaciones y Consideraciones Técnicas

1. **Datos Sintéticos:** Las pruebas actuales se realizan sobre vectores aleatorios normalizados L2 (distribución uniforme). El comportamiento real del índice puede variar ligeramente con datasets de producción reales (ej. texto/embeddings reales de RAG).
2. **Construcción Monohilo:** La fase de ingesta y construcción del grafo HNSW actualmente es monohilo en la API del SDK.
3. **Métricas de Distancia:** La versión actual está altamente optimizada para **Distancia Coseno**.

---

## 🚀 4. Impacto del Prefetching Predictivo del Kernel (SCALE-01)

Este benchmark compara la latencia de las consultas sobre un dataset persistido bajo `VantaFile` con y sin la optimización de prefetch predictivo del kernel (`madvise(MADV_WILLNEED)` en Unix y `PrefetchVirtualMemory` en Windows).

<!-- PREFETCH_BENCHMARK_START -->
| Métrica | Sin Prefetch (A) | Con Prefetch (B) | Mejora (%) |
| :--- | :--- | :--- | :--- |
| **Latencia Media** | 38.870 ms | 38.539 ms | **0.9%** |
| **Latencia p50** | 36.639 ms | 36.007 ms | **1.7%** |
| **Latencia p95** | 52.592 ms | 51.537 ms | **2.0%** |
| **Latencia p99** | 57.057 ms | 57.776 ms | **-1.3%** |
| **Throughput (QPS)** | 25.7 qps | 25.9 qps | **+0.9%** |
<!-- PREFETCH_BENCHMARK_END -->

---

## 🚀 5. Impacto de Optimización de Bucle y Distancias HNSW (Fase 2)

Este benchmark documenta el rendimiento y los tiempos de construcción en base al dataset estándar **SIFT1M** (escalas de 10K y 100K) tras aplicar las mejoras de la Fase 2:

1. **Caché en pila O(M²) de select_neighbors:** Eliminación absoluta de las búsquedas redundantes en `HashMap` (`self.nodes.get`) en el bucle de diversidad.
2. **Euclideana al Cuadrado en Travesía:** Supresión de `.sqrt()` en el hot path.
3. **Carga SIMD vmovups:** Carga vectorial alineada/unalineada contigua mediante `try_from` en registros `f32x8`.

### Resultados Comparativos de Construcción y Búsqueda (SIFT1M)

| Escala | Configuración | Métrica | Construcción (Antes) | Construcción (Ahora) | Aceleración (Speedup) | Latencia p99 | QPS Promedio |
| :--- | :--- | :---: | :---: | :---: | :---: | :---: | :---: |
| **100K** | Balanced Cos | Cosine | 139.4s | **63.7s** | **2.18x** | 441.2 µs | 3,636 |
| **100K** | High Recall Cos | Cosine | 390.8s | **182.2s** | **2.14x** | 1,231.8 µs | 1,379 |
| **100K** | Balanced L2 | Euclidean | 191.4s | **68.4s** | **2.80x** | 671.4 µs | 3,270 |
| **100K** | High Recall L2 | Euclidean | 462.2s | **194.5s** | **2.37x** | 1,183.6 µs | 1,353 |
| **100K** | High Recall L2 Mmap | Mmap Euclidean | 411.2s | **189.8s** | **2.16x** | 1,094.8 µs | 1,438 |

*Certificación en hardware: AMD Ryzen 12-Core @ 3.5GHz, compilación con `-C target-cpu=native`.*

---

## 🚀 6. Rendimiento de Búsqueda por Lotes (`search_batch`) en Python SDK

La búsqueda por lotes (`search_batch()`) en el SDK amortiza los costos de frontera FFI de PyO3 al realizar la transferencia en una sola llamada desde el entorno Python, liberando de forma eager el **GIL** y ejecutando la travesía del índice HNSW en paralelo a nivel multinúcleo con **Rayon**.

### Resultados del Micro-Benchmark Comparativo (5,000 registros, 128d, Batch Size: 100)

| Modo de Búsqueda | Tiempo Total (100 consultas) | Latencia Media por Consulta | Factor de Aceleración (Speedup) | Reducción de Latencia |
| :--- | :---: | :---: | :---: | :---: |
| **Secuencial (`db.search()`)** | 973.68 ms | 9.73 ms | *Línea Base* | *N/A* |
| **Por Lotes (`db.search_batch()`)** | **243.01 ms** | **2.43 ms** | **4.01x más rápido** | **75.0%** |

*Nota: Estos resultados demuestran la paridad con el escalado multinúcleo en CPUs de desarrollo estándar, permitiendo que aplicaciones RAG o LLM con alto volumen de consultas concurrentes no se bloqueen por el GIL.*

















## 🚀 7. Competitive Benchmark vs LanceDB & Chroma
Este benchmark compara **VantaDB** directamente contra **LanceDB** y **ChromaDB** en ingesta, latencias, precisión (Recall) y huella de memoria en reposo.

* **Fecha de ejecución**: 2026-06-06 15:43:40
* **Configuración del Dataset**:
  * **Nombre**: `glove-100-angular`
  * **Tamaño Ingestado**: 10000 registros
  * **Dimensión de Vectores**: 100
  * **Consultas Evaluadas**: 100
  * **Métrica**: `cosine`
  * **Vecinos (Top-K)**: 10

### Tabla Comparativa

| Engine   |   Ingest QPS | Index Time (ms)   |   Query QPS |   Latency p50 (ms) |   Latency p99 (ms) | Recall@10   |   Peak RSS (MB) |   Delta RSS (MB) |
|----------|--------------|-------------------|-------------|--------------------|--------------------|-------------|-----------------|------------------|
| VantaDB  |        598.3 | 16039.9           |        24.3 |             39.74  |             58.245 | 24.50%      |           236.5 |             91.7 |
| LanceDB  |     114583   | 602.2             |       320.5 |              2.653 |              6.98  | 13.90%      |           344.2 |             97.2 |
| ChromaDB |       3886   | N/A (Inc)         |       978.6 |              0.941 |              3.349 | 24.10%      |           253.5 |             39.1 |

*Nota: LanceDB e incremental-HNSW de ChromaDB usan sus wrappers de C/C++ nativos integrados en Python. VantaDB corre a través de sus bindings FFI de PyO3 (`vantadb_py`) consumiendo el core de Rust mapeado en memoria (`mmap`).*
