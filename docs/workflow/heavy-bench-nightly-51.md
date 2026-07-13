# `heavy-bench-nightly-51.yml` — HEAVY: Benchmarks — Nightly Regression

## ¿Qué hace?

Suite nocturna de benchmarks de rendimiento. Ejecuta 5 benchmarks de rendimiento (HNSW, hybrid queries, stress, concurrent, high density), compara los resultados contra el baseline (commit anterior) y reporta regresiones automáticamente como issues de GitHub.

## ¿Cómo lo hace?

3 jobs:

1. **`light-benchmarks`**: ejecuta 4 benchmarks con `cargo bench`:
   - `hnsw_pure` — rendimiento del índice HNSW
   - `hybrid_queries` — queries híbridas (vector + texto)
   - `stress_test` — test de estrés
   - `bench_concurrent` — operaciones concurrentes
2. **`high-density`**: ejecuta `high_density` — benchmarks de alta densidad de vectores (timeout 6h)
3. **`analyze`** (depende de ambos): descarga resultados, extrae estimaciones de Criterion con `scripts/bench_regression.py`, compara contra el baseline usando el script, y si detecta regresiones crea un issue en GitHub automáticamente con el detalle de cada regresión (benchmark, valor anterior, valor actual, % de cambio, severidad)

## ¿Qué tests usa?

Usa **Criterion** (`cargo bench`) con 5 harnesses de benchmark.

## ¿Qué verifica?

- Rendimiento de inserción y búsqueda HNSW
- Rendimiento de queries híbridas (vector + texto)
- Comportamiento bajo estrés
- Rendimiento concurrente
- Rendimiento en alta densidad de datos
- **Regresiones** comparando contra el commit anterior

## Funcionalidad final

Detección automatizada de regresiones de rendimiento. Si un cambio introduce una degradación en cualquiera de los benchmarks, se crea un issue con el reporte detallado para que el equipo lo revise.

## ¿Cuándo se ejecuta?

- **Cada noche** (03:00 UTC) vía `schedule`
- **Workflow dispatch** manual
