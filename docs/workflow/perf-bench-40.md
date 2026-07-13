# `perf-bench-40.yml` — PERF: Benchmarks — Python Integration

## ¿Qué hace?

Ejecuta benchmarks de rendimiento desde Python usando la rueda nativa de VantaDB (vantadb-py). Construye el wheel Python, lo instala y corre benchmarks de ingestión y búsqueda con parámetros configurables.

## ¿Cómo lo hace?

Un solo job `benchmark`:

1. Set up Rust + Python 3.11
2. Construye el wheel Python con `maturin build --release`
3. Instala el wheel en un virtualenv
4. Ejecuta `benchmarks/vantadb_local_bench.py` con parámetros (size, dim, queries)
5. Opcionalmente actualiza `BENCHMARKS.md` con `benchmarks/update_markdown.py`
6. Sube `benchmark_results.json` como artifact

Parámetros por defecto: 1000 vectores, 128 dimensiones, 100 queries.

## ¿Qué tests usa?

Usa scripts Python de benchmark: `vantadb_local_bench.py` y `update_markdown.py`.

## ¿Qué verifica?

- Rendimiento de ingestión de vectores desde Python
- Rendimiento de búsqueda (queries) desde Python
- Que el binding Python (PyO3) funciona correctamente en release

## Funcionalidad final

Medir y trackear el rendimiento del SDK Python de VantaDB. Genera un reporte JSON subido como artifact y actualiza dinámicamente BENCHMARKS.md con los resultados.

## ¿Cuándo se ejecuta?

- **Push** a `main` con cambios en: `src/**`, `vantadb-python/**`, `benchmarks/**`, `Cargo.toml`, `Cargo.lock`
- **Workflow dispatch** manual con parámetros configurables (size, queries, dim)
