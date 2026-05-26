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
| **Ingesta (`PUT`)** | 10K registros, 128d (con payload y metadata) | *N/D* | *N/D* | *N/D* | **~5,400 ops/sec** |
| **Index Rebuild** | Reconstrucción híbrida (HNSW + BM25) | **~2.1s** | *N/D* | *N/D* | N/D |
| **Búsqueda Lexical (BM25)** | 10K registros, `top_k=10` | *N/D* | *N/D* | *N/D* | **~830 queries/sec** |
| **Búsqueda Vectorial (HNSW)** | 10K registros, `top_k=10`, 128d | *N/D* | *N/D* | *N/D* | **~830 queries/sec** |
| **Búsqueda Híbrida (RRF)** | 10K registros, `top_k=10`, RRF Fusion | *N/D* | *N/D* | *N/D* | **~450 queries/sec** |
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
