# FASE 1-3 — Fundación, Integración y Pre-Lanzamiento

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

## Tareas Completadas

### FASE 1: Fundación

1. **[TSK-01]** Definir tipos de datos de vector_index — ✅
- `src/vector_index.rs`: `VectorIndex`, `IndexOptions`, `QuantizationMode`
2. **[TSK-02]** Implementar HNSW básico — ✅
- `src/hnsw.rs`: insert, search, ef_construction, ef_search, multi-layer skip list
3. **[TSK-03]** Implementar IVF básico — ✅
- `src/ivf.rs`: k-means, nprobe, inverted lists
4. **[TSK-04]** Refactorizar framework de benchmarks — ✅
- Dibs → Criterion, múltiples algoritmos, profiling
5. **[TSK-05]** Ranking híbrido sparse-dense — ✅
- `src/hybrid.rs`: `HybridRanker`, `fusion_score()`, `weights`, `normalize()`
6. **[TSK-06]** Inserción HNSW multi-threaded — ✅
- `src/hnsw.rs`: `RwLock<HnswLayer>`, `build_threaded()`, `Mutex<Vec>`, `try_write`
7. **[TSK-07]** Bindings Python con maturin — ✅
- `Cargo.toml:pyo3`, `src/python_module.rs`, `setup.py`, `pyproject.toml`
8. **[TSK-08]** Ser/deser con rmp-serde — ✅
   - `src/serde.rs`: `to_bytes()/from_bytes()`, `to_file()/from_file()`, MessagePack
9. **[TSK-09]** Versionar formato de índice — ✅
- `INDEX_VERSION`, `HeaderV1`, `VantaHeader`, forward compat
10. **[TSK-10]** Expansión de tests (unit + integration) — ✅
- 34 unit tests, 3 integration, proptest, benchmark datasets

### FASE 2: Integración + API

11. **[TSK-18]** Integrar HNSW + IVF como `UnifiedIndex` — ✅
- `src/unified_index.rs`: `SearchIndex` enum, `dispatch_search()`
12. **[TSK-19]** Consolidar `VantaIndex` como API principal — ✅
- `src/lib.rs`: `VantaIndex`, `VantaConfig`, `put()`, `get()`, `delete()`, `search()`, `list()`
13. **[TSK-20]** Tests de integración de `VantaIndex` — ✅
- `tests/integration.rs`: create, insert, search, delete, hybrid persistence, stress
14. **[TSK-21]** Servidor HTTP con axum (listo antes del servidor MCP) — ✅
- `src/http.rs`, `src/cli_server.rs`, `api.http`
15. **[TSK-22]** Servidor MCP para agentes LLM — ✅
- `vantadb-mcp/: put, get, delete, search, list, stats, clear`
16. **[TSK-23]** GitHub Actions CI + Build — ✅
- `.github/workflows/rust_ci.yml`: build, test, clippy, fmt
17. **[TSK-24]** Embeddings CLIP (producción) — ✅
- `src/embeddings/clip.rs`: async ONNX, `download_model()`, `embed_text()`, `embed_image()`
18. **[TSK-25]** Interfaz de embedding unificada — ✅
- `src/embeddings/mod.rs`: `EmbeddingModel` trait, `CLIPEmbedding`, `OpenAIEmbedding`, `OllamaEmbedding`
19. **[TSK-26]** Tests Python con pytest — ✅
- `tests/python/`: `test_basic.py`, `test_hybrid.py`, `test_cli_server.py`
20. **[TSK-27]** Tests E2E cliente HTTP → servidor — ✅
- `tests/e2e/`: `test_http_api.py`
21. **[TSK-28]** Investigación: lock-free HNSW (DISC-01) — ✅
    - Conclusión: el `RwLock` actual es suficiente para cargas predecibles
22. **[TSK-29]** Sitio web estático de VantaDB + landing — ✅
- `docs/website/`: landing, components, scroll animations, pricing
23. **[TSK-31]** Implementar tracing de DataDog — ✅
- `src/telemetry/datadog.rs`: `init_tracing()`, `DD_TRACE_*`, `TracingLayer`
24. **[TSK-32]** DOTC (DataDog Observability) — ✅
- 8 módulos, `MetricsCollector`, health check, puente OTel, ResourceDetector
25. **[TSK-33]** Razonamiento GraphRAG (Layout) — ✅
- `docs/graphrag/README.md` design spec
26. **[TSK-51]** Integración de sparse embedding — ✅
- `src/sparse_embed.rs`: `SparseEmbedding`, `SparseVector`, dim fija 1000, `cosine_similarity()`
27. **[TSK-52]** Implementar host header + connection pooling en el servidor — ✅
- Tower `SetRequestHeader`, `keep-alive`, `pool_idle_timeout`, h2 priority
28. **[TSK-53]** Permitir bind a interfaz específica — ✅
- `--bind <host:port>`, defaults `0.0.0.0:7643`
29. **[TSK-57]** Investigación: dataset de benchmark grande (DISC-02) — ✅
- `scripts/download_benchmark_datasets.sh`, `tests/benchmark_datasets.rs`
30. **[TSK-58]** Deduplicación de vectores — ✅
- `UniqueConstraint`, `conflict_policy`, `on_conflict`
31. **[TSK-59]** Semántica atómica read-write — ✅
- WAL, sequence numbers, crash recovery, serializable isolation
32. **[TSK-60]** `sparse_threshold` (peso dense-sparse) — ✅
- `HybridConfig`, `sparse_threshold`, `dynamic_alpha()`
33. **[TSK-68]** Hooks basados en eventos — ✅
- `EventHook`, `on_insert/on_delete/on_search`, síncrono

### FASE 3: Pre-Lanzamiento

34. **[TSK-61]** Feature gates + perfiles de build — ✅
- `features: ["default", "cli", "python", "tel", "test-bench-datasets", "nightly"]`
35. **[TSK-62]** Flags CLI + env vars + archivo de config — ✅
- `VantaConfig` struct, `clap` + `dotenv`, `--config`, clap completion
36. **[TSK-63]** CI multiplataforma con cobertura — ✅
- Build matrix (ubuntu, windows, macos), `--target`, `--all-features`
37. **[TSK-64]** Linting + cobertura gate — ✅
- `clippy -D warnings`, `cargo fmt --check`, `cargo llvm-cov --fail-uncovered`
38. **[TSK-65]** Version bumps semver — ✅
- `0.1.0` → `0.1.1` → `0.1.2` → `0.1.3` → `0.1.4`, changelog, git tag
39. **[TSK-66]** Pipeline de CI de release — ✅
- `cargo publish` dry-run, GitHub Release, auto-tag, maturin publish
40. **[TSK-67]** Docs de GraphRAG — ✅
- archivo completo `docs/graphrag/README.md`: comparison, getting started, Python examples
41. **[TSK-46]** HNSW con MMap — ✅
- `mmap_hnsw: bool` config, memory budget gate, 2 tests
42. **[TSK-50]** Backpressure RSS — ✅
- `check_memory_pressure()` con `rss_threshold`, auto-eviction, 2 tests
43. **[TSK-69]** `put_batch()` con Rayon — ✅
- `insert_many()`, expuesto en Python, 3 tests, commit `c3173d9`
44. **[TSK-73]** `AsyncVantaDB` asyncio — ✅
- clase `AsyncVantaDB` Python, 3 tests, commit `6ec3f8e`
45. **[TSK-74]** Type stubs `.pyi` — ✅
- Python type hints, commit `6ec3f8e`
46. **[TSK-75]** Compactación + rotación WAL — ✅
- `WalWriter::rotate()`, `compact_wal()`, binding Python, 2 tests, commit `68616d6`
47. **[TSK-76a]** Auto-eviction TTL — ✅
- `expires_at_ms`/`ttl_ms`, lazy eviction, `purge_expired()`, 4 tests, commit `68616d6`
48. **[TSK-76b]** Eviction ponderada — ✅
- `EvictionWeights`, `eviction_score()`, `EvictionReport`, 3 tests
49. **[TSK-70]** Docs de garantías de durabilidad — ✅
- `docs/operations/DURABILITY_GUARANTEES.md`, 9 secciones, 10 garantías, 7 escenarios de fallo
50. **[TSK-78]** Property-based testing expandido — ✅
- 5 nuevos proptests (uniqueness, roundtrip, metadata, delete idempotency, TTL), 8/8 pass
51. **[TSK-93]** Histogramas Prometheus HTTP — ✅
- p50/p95/p99, axum middleware, 6/6 E2E, commit `37ee241`
52. **[TSK-97]** Eliminación de panics en runtime — ✅
- Remover `unwrap()` de APIs públicas, `std::panic::catch_unwind` en C FFI, commit `62cfd6bb`
53. **[WEB-01]** Centralización de documentación (Monorepo) — ✅
- Unificación total de `web/docs/` → `docs/web/`, integración del backlog web en el raíz, eliminación de artefactos de migración (`plan/`).
54. **[WEB-14a]** Rediseño del Hero (Swiss Typographic Grid) — ✅
- Rediseñado SwissHero.tsx y swiss-hero.css siguiendo el manifiesto de diseño suizo.
- Implementado dibujo del grid de 1px usando SVG con stroke-dashoffset y stagger animado en GSAP.
- Eliminada animación de typewriter en subtítulo, mostrando texto inmediatamente en Outfit a tamaño display.
- Agregada interactividad de click-to-copy con feedback visual en el comando de instalación.
- Removidos todos los inline styles de SwissHero.
54. **[TSK-56]** Fix runner Windows CI — ✅
- Timeouts, pin image, OIDC trusted publishing, commits `afa141d`..`84d862c`
55. **[TSK-55]** Datasets reales en CI — ✅
- GloVe-100 en CI, `benchmark_datasets.rs`, scripts sh/ps1, step en `rust_ci.yml`
55. **[TSK-79]** Alertas de regresión de benchmarks — ✅
- `scripts/bench_regression.py` (extract/compare/update-baseline), workflow nocturno con creación de GitHub Issue
56. **[TSK-81]** Badges del README — ✅
    - 2 filas, iconos de marca, commits `93f71aa`/`c049dc7`
57. **[TSK-80]** Guías de migración — ✅
- ChromaDB y LanceDB, commit `55cc28b`
58. **[TSK-82]** CHANGELOG formal — ✅
- git-cliff, 460 commits, commit `55cc28b`
59. **[TSK-94]** Logging estructurado JSON — ✅
- `LogFormat` enum, `VANTADB_LOG_FORMAT=json|compact|full`, commit `68c1ce9`
60. **[TSK-54]** Benchmarks CI nocturnos — ✅
- schedule 03:00 UTC, 5 targets, upload artifacts
61. **[TSK-37]** Benchmarks de calidad híbrida — ✅
- NDCG@k, MRR, Recall@k, corpus de 20 docs, 2 queries
62. **[TSK-83]** Templates de Issue/PR — ✅
- bug_report, feature_request, PR template en `.github/`
63. **[TSK-84]** DISC-03: benchmark de prefetch — ✅
- Prefetch 13.8% más rápido, `src/index.rs:33-72`
64. **[TSK-85]** Stress tests de file locking — ✅
- 4 tests, fs2 OS-level, lock timeout ~30s
65. **Auditoría de backlog** — ✅
- 4 discrepancias corregidas (TSK-94/67/80/82)
66. **Fixes de Clippy/fmt** — ✅
- 3 unused vars, formato de 18 archivos, imports condicionales
67. **Fix de `with_writer`** — ✅
- `MakeWriter` closure en vez de `Box<dyn Write>` directo
68. **`vantadb-mcp` ttl_ms: None** — ✅
- `planner.rs:369` `expires_at_ms: Some(0)`
69. **COMP-025: JSON shredding** — ✅
- **Fase 1:** Inferencia de esquema + almacenamiento columnar + integración de filtros (equality fast path). `ShreddedSchema`, `ShreddedRowStore`, `infer_field_type`. 8 tests.
- **Fase 2:** Filtros de comparación tipados (Gt/Lt/Gte/Lte/Neq). `matches_shredded` con 6 operadores en I64/F64/Bool/String. 5 tests adicionales (13 total). Test de integración con 3 nodes.
- **Total:** 13 tests, `src/shred/mod.rs`, commit TBD

### Problemas de Infraestructura

| Problema | Descripción | Estado |
|-------|-------------|--------|
| Windows pagefile | `os error 1455` en `mmap_hnsw` y `proptest` | 🔴 Ambiental, no de código |
| `install-action` | `taiki-e/install-action@cargo-llvm-cov` y `@nextest` fallan intermitentemente | 🔴 Infraestructura GitHub |
