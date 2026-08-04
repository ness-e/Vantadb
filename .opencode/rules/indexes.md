# Indexes (Vector & Text) — Reglas

> **Scope:** `src/index/` completo (`graph.rs` HNSW, `diskann.rs`, `ivf.rs`, `scann.rs`, `flat.rs`, `distance.rs`, `neighbor_index.rs`, `auto_tune`, `serialize`, `search`, `refresh`, `stats`), `text_index.rs`, `tokenizer.rs`, `src/vector/` (`quantization.rs`, `transform.rs`, `governor.rs`)
> **No tocar aquí:** durabilidad/WAL (`durability.md`), concurrencia/async (`concurrency-async.md`), API pública (`api-contract.md`)
> **Status:** 🟢 Vigente
> **Fuentes:** OLD-004, VFY-004, PERF-09, INV-007

## Reglas

### R-1: Métricas competitivas honestas — sin placeholders ni números inventados

- **Must:** publicar solo números generados por ejecución real del harness (`benchmarks/competitive_bench.py`, 751 L — datasets glove-100-angular / sift-128-euclidean, ChromaDB/LanceDB vs VantaDB) con hardware, versiones y fecha declarados en el JSON contrato.
- **Must not:** publicar números inventados ni placeholders (los valores del JSON de ejemplo del INV-007 son ilustrativos) en el web, docs o PRs; tampoco comparar runs de hardware distinto.
- **Por qué:** la credibilidad de los benchmarks comparativos depende de reproducibilidad y transparencia; los números falsos erosionan confianza y rompen el contrato JSON con la web (INV-007 §6).

<!-- Referencias cruzadas: → ver core-engine.md, concurrency-async.md, query-dsl.md -->
