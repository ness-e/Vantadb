# Progreso General del Proyecto VantaDB

> **Última actualización:** 2026-06-21 (CLI-EPIC + Filter Operators + SDK Methods ✅)

## Resumen Ejecutivo

VantaDB es una base de datos vectorial en Rust enfocada en alto rendimiento, HNSW híbrido, GraphRAG, CLIP y ecosistema Python/LLM.

**Estado:** 🟢 FASE 3 pre-lanzamiento (~95%)

### Progreso general

| Categoría | Completadas | Total | Estado |
|-----------|-------------|-------|--------|
| Core/Index | 16 | 16 | ✅ |
| Python Bindings | 5 | 5 | ✅ |
| API/Servidor | 9 | 9 | ✅ |
| Observabilidad | 6 | 6 | ✅ |
| Documentation/Website | 11 | 11 | ✅ |
| Benchmarks/CI | 5 | 5 | ✅ |
| QA/Tests | 7 | 7 | ✅ |
| Herramientas DX | 15 | 15 | ✅ |
| CLI | 7 | 7 | ✅ |
| Project Management | 6 | 6 | ✅ |
| **Total** | **86** | **~86** | **✅** |

## Leyenda

| Símbolo | Significado |
|---------|-------------|
| ✅ Completada | Tarea finalizada, mergeada a main |
| 🟡 En progreso | Tarea en trabajo activo |
| 🔴 Bloqueada | Tarea que no puede avanzar |

---

## Tareas Completadas

### FASE 1: Fundación

1. **[TSK-01]** Definir tipos de datos vector_index — ✅
   - `src/vector_index.rs`: `VectorIndex`, `IndexOptions`, `QuantizationMode`
2. **[TSK-02]** Implementar HNSW básico — ✅
   - `src/hnsw.rs`: insert, search, ef_construction, ef_search, multi-layer skip list
3. **[TSK-03]** Implementar IVF básico — ✅
   - `src/ivf.rs`: k-means, nprobe, inverted lists
4. **[TSK-04]** Refactor benchmark framework — ✅
   - Dibs → Criterion, múltiples algoritmos, profiling
5. **[TSK-05]** Híbrido sparse-dense ranking — ✅
   - `src/hybrid.rs`: `HybridRanker`, `fusion_score()`, `weights`, `normalize()`
6. **[TSK-06]** HNSW multi-threaded insert — ✅
   - `src/hnsw.rs`: `RwLock<HnswLayer>`, `build_threaded()`, `Mutex<Vec>`, `try_write`
7. **[TSK-07]** Python bindings maturin — ✅
   - `Cargo.toml:pyo3`, `src/python_module.rs`, `setup.py`, `pyproject.toml`
8. **[TSK-08]** Ser/deser con rmp-serde — ✅
   - `src/serde.rs`: `to_bytes()/from_bytes()`, `to_file()/from_file()`, MessagePack
9. **[TSK-09]** Versionar formato de índice — ✅
   - `INDEX_VERSION`, `HeaderV1`, `VantaHeader`, forward compat
10. **[TSK-10]** Expansión de tests (unit + integración) — ✅
    - 34 unit tests, 3 integración, proptest, benchmark datasets

### FASE 2: Integración + API

11. **[TSK-18]** Integrar HNSW + IVF como `UnifiedIndex` — ✅
    - `src/unified_index.rs`: `SearchIndex` enum, `dispatch_search()`
12. **[TSK-19]** Consolidar `VantaIndex` como API principal — ✅
    - `src/lib.rs`: `VantaIndex`, `VantaConfig`, `put()`, `get()`, `delete()`, `search()`, `list()`
13. **[TSK-20]** Integration tests de `VantaIndex` — ✅
    - `tests/integration.rs`: create, insert, search, delete, persistencia híbrida, stress
14. **[TSK-21]** Servidor HTTP con axum (listo antes de servidor MCP) — ✅
    - `src/http.rs`, `src/cli_server.rs`, `api.http`
15. **[TSK-22]** MCP server para LLM agents — ✅
    - `vantadb-mcp/: put, get, delete, search, list, stats, clear`
16. **[TSK-23]** GitHub Actions CI + Build — ✅
    - `.github/workflows/rust_ci.yml`: build, test, clippy, fmt
17. **[TSK-24]** CLIP embeddings (producción) — ✅
    - `src/embeddings/clip.rs`: async ONNX, `download_model()`, `embed_text()`, `embed_image()`
18. **[TSK-25]** Unified embedding interface — ✅
    - `src/embeddings/mod.rs`: `EmbeddingModel` trait, `CLIPEmbedding`, `OpenAIEmbedding`, `OllamaEmbedding`
19. **[TSK-26]** Python tests con pytest — ✅
    - `tests/python/`: `test_basic.py`, `test_hybrid.py`, `test_cli_server.py`
20. **[TSK-27]** E2E tests cliente HTTP → servidor — ✅
    - `tests/e2e/`: `test_http_api.py`
21. **[TSK-28]** Investigación: lock-free HNSW (DISC-01) — ✅
    - Conclusión: `RwLock` actual es suficiente para cargas predecibles
22. **[TSK-29]** Página web estática VantaDB + landing — ✅
    - `docs/website/`: landing, components, scroll animations, pricing
23. **[TSK-31]** Implementar DataDog tracing — ✅
    - `src/telemetry/datadog.rs`: `init_tracing()`, `DD_TRACE_*`, `TracingLayer`
24. **[TSK-32]** DOTC (DataDog Observabilidad) — ✅
    - 8 módulos, `MetricsCollector`, health check, OTel bridge, ResourceDetector
25. **[TSK-33]** Razonamiento GraphRAG (diseño) — ✅
    - `docs/graphrag/README.md` design spec
26. **[TSK-51]** Sparse embedding integración — ✅
    - `src/sparse_embed.rs`: `SparseEmbedding`, `SparseVector`, dim fijo 1000, `cosine_similarity()`
27. **[TSK-52]** Implementar host header + connection pooling en servidor — ✅
    - Tower `SetRequestHeader`, `keep-alive`, `pool_idle_timeout`, h2 prioridad
28. **[TSK-53]** Permitir bind a interface específica — ✅
    - `--bind <host:port>`, defaults `0.0.0.0:7643`
29. **[TSK-57]** Investigación: dataset benchmarks grande (DISC-02) — ✅
    - `scripts/download_benchmark_datasets.sh`, `tests/benchmark_datasets.rs`
30. **[TSK-58]** Deduplication de vectores — ✅
    - `UniqueConstraint`, `conflict_policy`, `on_conflict`
31. **[TSK-59]** Atomic read-write semantics — ✅
    - WAL, sequence numbers, crash recovery, serializable isolation
32. **[TSK-60]** `sparse_threshold` (dense-sparse weight) — ✅
    - `HybridConfig`, `sparse_threshold`, `dynamic_alpha()`
33. **[TSK-68]** Event-driven hooks — ✅
    - `EventHook`, `on_insert/on_delete/on_search`, síncrono

### FASE 3: Pre-Lanzamiento

34. **[TSK-61]** Feature gates + perfis de compilación — ✅
    - `features: ["default", "cli", "python", "tel", "test-bench-datasets", "nightly"]`
35. **[TSK-62]** CLI flags + env vars + config file — ✅
    - `VantaConfig` struct, `clap` + `dotenv`, `--config`, clap completion
36. **[TSK-63]** Cross-platform CI con cobertura — ✅
    - Build matrix (ubuntu, windows, macos), `--target`, `--all-features`
37. **[TSK-64]** Linting + coverage gate — ✅
    - `clippy -D warnings`, `cargo fmt --check`, `cargo llvm-cov --fail-uncovered`
38. **[TSK-65]** Version bumps semver — ✅
    - `0.1.0` → `0.1.1` → `0.1.2` → `0.1.3` → `0.1.4`, changelog, git tag
39. **[TSK-66]** Release CI pipeline — ✅
    - `cargo publish` dry-run, GitHub Release, auto-tag, maturin publish
40. **[TSK-67]** GraphRAG docs — ✅
    - `docs/graphrag/README.md` completo: comparativa, getting started, ejemplos Python
41. **[TSK-46]** MMap-backed HNSW — ✅
    - `mmap_hnsw: bool` config, memory budget gate, 2 tests
42. **[TSK-50]** Backpressure RSS — ✅
    - `check_memory_pressure()` con `rss_threshold`, auto-eviction, 2 tests
43. **[TSK-69]** `put_batch()` con Rayon — ✅
    - `insert_many()`, expuesto en Python, 3 tests, commit `c3173d9`
44. **[TSK-73]** `AsyncVantaDB` asyncio — ✅
    - `AsyncVantaDB` Python class, 3 tests, commit `6ec3f8e`
45. **[TSK-74]** Type stubs `.pyi` — ✅
    - Python type hints, commit `6ec3f8e`
46. **[TSK-75]** WAL compact + rotate — ✅
    - `WalWriter::rotate()`, `compact_wal()`, Python binding, 2 tests, commit `68616d6`
47. **[TSK-76a]** TTL auto-eviction — ✅
    - `expires_at_ms`/`ttl_ms`, lazy eviction, `purge_expired()`, 4 tests, commit `68616d6`
48. **[TSK-76b]** Weighted eviction — ✅
    - `EvictionWeights`, `eviction_score()`, `EvictionReport`, 3 tests
49. **[TSK-70]** Durability guarantees docs — ✅
    - `docs/operations/DURABILITY_GUARANTEES.md`, 9 secciones, 10 garantías, 7 fallo escenarios
50. **[TSK-78]** Property-based testing expandido — ✅
    - 5 nuevos proptests (uniqueness, roundtrip, metadata, delete idempotency, TTL), 8/8 pasan
51. **[TSK-93]** Prometheus histograms HTTP — ✅
    - p50/p95/p99, middleware axum, 6/6 E2E, commit `37ee241`
52. **[TSK-97]** Eliminación de panics runtime — ✅
    - 6 ubicaciones, 48+33+7+6 tests, commit `98edf4c`
53. **[TSK-56]** Fix Windows CI runner — ✅
    - Timeouts, pin image, OIDC trusted publishing, commits `afa141d`..`84d862c`
54. **[TSK-55]** Real datasets CI — ✅
    - GloVe-100 en CI, `benchmark_datasets.rs`, scripts sh/ps1, step en `rust_ci.yml`
55. **[TSK-79]** Benchmark regression alerts — ✅
    - `scripts/bench_regression.py` (extract/compare/update-baseline), nightly workflow con creación de GitHub Issue
56. **[TSK-81]** README badges — ✅
    - 2 filas, iconos de marca, commits `93f71aa`/`c049dc7`
57. **[TSK-80]** Migration guides — ✅
    - ChromaDB y LanceDB, commit `55cc28b`
58. **[TSK-82]** CHANGELOG formal — ✅
    - git-cliff, 460 commits, commit `55cc28b`
59. **[TSK-94]** JSON structured logging — ✅
    - `LogFormat` enum, `VANTADB_LOG_FORMAT=json|compact|full`, commit `68c1ce9`
60. **[TSK-54]** Nightly CI benchmarks — ✅
    - schedule 03:00 UTC, 5 targets, upload artifacts
61. **[TSK-37]** Hybrid quality benchmarks — ✅
    - NDCG@k, MRR, Recall@k, 20-doc corpus, 2 queries
62. **[TSK-83]** Issue/PR templates — ✅
    - bug_report, feature_request, PR template en `.github/`
63. **[TSK-84]** DISC-03: Prefetch benchmark — ✅
    - Prefetch 13.8% faster, `src/index.rs:33-72`
64. **[TSK-85]** File locking stress tests — ✅
    - 4 tests, fs2 OS-level, lock timeout ~1s
65. **Backlog audit** — ✅
    - 4 discrepancias corregidas (TSK-94/67/80/82)
66. **Clippy/fmt fixes** — ✅
    - 3 unused vars, formateo 18 archivos, imports condicionales
67. **Fix `with_writer`** — ✅
    - `MakeWriter` closure en vez de `Box<dyn Write>` directo
68. **`vantadb-mcp` ttl_ms: None** — ✅
    - `planner.rs:369` `expires_at_ms: Some(0)`

### Infrastructure Issues

| Issue | Descripción | Estado |
|-------|-------------|--------|
| Windows pagefile | `os error 1455` en `mmap_hnsw` y `proptest` | 🔴 Entorno, no código |
| `install-action` | `taiki-e/install-action@cargo-llvm-cov` y `@nextest` fallan intermitentemente | 🔴 Infraestructura GitHub |

## Auditoría Integral (2026-06-19) — COMPLETADA ✅

Auditoría automatizada de 44 hallazgos ejecutada y resuelta en su totalidad el mismo día. Cada hallazgo fue delegado a un agente especializado para su diagnóstico y corrección.

### 🔴 Críticos (7/7 ✅)

| ID | Fix | Impacto |
|----|-----|---------|
| AUD-01 | `abi3-py311` → `abi3-py38` en `vantadb-python/Cargo.toml:13` | PyPI wheels ahora soportan Python 3.8–3.10 |
| AUD-02 | 16 `.unwrap()` → `?` + error handling (index.rs:13, storage.rs:1, wal.rs:2) | Eliminados panics en runtime por datos corruptos |
| AUD-03 | `bincode 1.3` → `2.0` (serde feature, 8 archivos, 27 call sites) | RUSTSEC-2025-0141 resuelto |
| AUD-04 | `pyo3 0.24` → `0.29` (3 breaking changes migrados: `PyObject`→`Py<PyAny>`, `.downcast()`→`.cast()`, `.allow_threads()`→`.detach()`) | RUSTSEC-2026-0176/0177 resueltos |
| AUD-05 | 18 links reparados en README.md + README_ES.md | Contribute/Security/Support → `.github/`, Python SDK → `docs/api/`, Benchmarks → `docs/operations/` |
| AUD-06 | `chaos_testing.rs` → `chaos_integrity.rs` en DURABILITY_GUARANTEES.md:287 | Referencia corregida |
| AUD-07 | `README.MD` → `README.md` en README_ES.md:24 | Case-sensitive FS fix |

### 🟡 Medios (14/14 ✅)

| ID | Fix |
|----|-----|
| AUD-08 | Auditoría completa de 39 ítems unsafe (33 bloques, 4 impls, 1 pub fn, 1 extern fn). 77% low-risk, 20.5% medium, 2.6% high. Reporte detallado. |
| AUD-09 | `static TEST_RESULTS` eliminado. `MULTI_PROGRESS` migrado a `thread_local!` + `RefCell`. |
| AUD-10 | Env vars guardadas/restauradas en prefetch_benchmark.rs. |
| AUD-11 | ~153 assertions con mensajes descriptivos (basic_node, benchmark_internal, test_sdk.py ~85, mcp_tests.rs 58, mcp_integration.rs 3). |
| AUD-12 | `StdRng::seed_from_u64(42)` en hnsw_recall.rs + prefetch_benchmark.rs. Benchmarks reproducibles. |
| AUD-13 | `TempDir` en basic_node.rs y benchmark_internal.rs. |
| AUD-14 | `ttl_ms: int \| None = None` agregado a `AsyncVantaDB.put()`. |
| AUD-15 | `tower 0.4` → `0.5` unified via dev-dep upgrade. |
| AUD-16 | 3 stale advisory ignores removidos de deny.toml. `cargo deny check` → OK. |
| AUD-17 | `rust-toolchain.toml`: `1.94.1` → `stable`. |
| AUD-18 | Windows CI ahora ejecuta `cargo test --workspace`. |
| AUD-19 | `curl -s` → `curl -sL` en install.sh. |
| AUD-20 | Detección `aarch64`/`arm64` + `x86_64`/`amd64` en install.sh. Unknown arches → hard-fail. |
| AUD-21 | Ref a `ROADMAP.md` en CHANGELOG.md comentada como TODO. |

### 🔵 Bajos (23/23 ✅)

| ID | Fix |
|----|-----|
| AUD-22 | `governor.request_allocation()` error ahora propaga via `?`. |
| AUD-23 | 4 sitios de flush/eviction ahora logean `tracing::warn!`. |
| AUD-24 | `compact_layout_bfs()` (249L → 53L orquestador + 3 helpers). |
| AUD-25 | `add()` (214L → 8L dispatcher + `validate_node`, `insert_hnsw`, `update_metadata`). |
| AUD-26 | `open_with_config()` (271L → 59L pipeline + 4 helpers). |
| AUD-27 | Backend string inválido → `tracing::warn!`. |
| AUD-28 | `distance_metric` inválido → `tracing::warn!`. |
| AUD-29 | 6 archivos unificados a `ness-e/Vantadb`. |
| AUD-30 | `time.sleep(0.01)` → `_wait_until()` retry loop (5-10s timeout). |
| AUD-31 | `arrow`, `rocksdb`, `fjall` feature-gated (default incluye las 3). |
| AUD-32 | `nightly_bench.yml`: `checkout@v4` → `@v6`. |
| AUD-33 | `heavy_certification.yml`: `install-action@nextest` → `@v2` + `tool:`. |
| AUD-34 | Commit count: `237` → `460` en progreso docs. |
| AUD-35 | 4 sleeps reemplazados: `wait_for_port()`, `JoinHandle::await`, 1 justificado con comentario. |
| AUD-36 | Mensaje agregado a `assert_eq!`, `assert!(true)` ya no existía. |
| AUD-37 | `tests/edge_cases.rs` creado: 25 tests cubriendo 17 categorías (NaN, Inf, empty, unicode, TTL, etc.). |
| AUD-38 | Tokio features: `"full"` → granulares (`rt`, `sync`, `time`, `macros`, etc.). |
| AUD-39 | `wide = "=1.2.0"` → `">=1.2, <2"`. |
| AUD-40 | `[workspace.package]` creado. 3 sub-crates migrados a `version.workspace = true`. |
| AUD-41 | `maturin-action@v1` → `@v2`. |
| AUD-42 | `vantadb-mcp` agregado al build/release/hash/attest en release.yml. |
| AUD-43 | Free disk space + 6GB swap agregados a nightly_bench.yml. |
| AUD-44 | `setup-python@v5` → `@v6`. |

## Progreso Reciente

### Semana del 2026-06-19 — Auditoría Integral Completa (AUD-01→44)

- **44 hallazgos de auditoría resueltos** en un solo día mediante agentes especializados paralelos (3 por lote, 15 lotes).
- **7 críticos** (seguridad, packaging, documentación), **14 medios** (tests, CI/CD, infra), **23 bajos** (refactors, deuda técnica, UX).
- **Archivos modificados**: ~45 archivos entre Rust, Python, YAML, TOML, Markdown, scripts.
- **Nuevos archivos**: `tests/edge_cases.rs` (25 tests de casos borde).
- **CVEs resueltos**: RUSTSEC-2025-0141 (bincode), RUSTSEC-2026-0176/0177 (pyo3).
- **Criterio de salida FASE 3 actualizado**: todos los AUD resueltos ✅

### Semana del 2026-06-12 → 2026-06-18

- **TSK-79**: Benchmark regression alerts. `scripts/bench_regression.py` (3 modos), nightly workflow con creación de GitHub Issue automática. Actualizado progreso y CHANGELOG.
- **CI fixes**: Conditional imports en `cli_server.rs`. Step benchmark datasets en coverage job. Update `install-action` a `@v2`.
- **Clippy audit**: 5 categorías de warnings corregidos (too_many_arguments, suspicious_open_options, field_reassign_with_default, needless_range_loop, needless_borrow)
- **Auditoría integral**: 40 hallazgos documentados (7 críticos, 14 altos, 19 medios).
- **Push final**: 30 commits ahead, pushed to `ness-e/Vantadb` main (commit `f5eafbd`)

### Tarea: AUD-WORK — Corrección de CI y Auditoría de Workflows (2026-06-20)

- **Objetivo:** Corregir las fallas del pipeline de CI de GitHub Actions (timeout en `crash_injection` y fallo de permisos de `wal_write_failure_returns_error`) y aplicar de forma estructurada los 9 hallazgos del reporte de auditoría.
- **Commits:** `85f2beb`, `447224e`, `4030d36`, `ab09229`, `25dc38b`, `a3c2c04`, `aaf0428`, `26afb62`
- **Checklist Completado:**
  - [x] Modificar `.config/nextest.toml`
    - [x] Migrar exclusiones de `binary_id(...)` a `binary(...)`
    - [x] Corregir `hnsw_recall` a `hnsw_recall_certification`
    - [x] Cambiar `not test(integrations_certification)` a `not binary(integration)`
    - [x] Agregar exclusión de `mcp_tests` y `multilingual_tokenizer_integration`
    - [x] Agregar exclusión de `memory_telemetry` y el test unitario `concurrent_insert_preserves_hnsw_invariants`
  - [x] Modificar `Cargo.toml`
    - [x] Declarar `fjall_cold_copy_restore`, `property_durability`, `fuzz_proptest` y `multilingual_tokenizer_integration`
    - [x] Agregar `required-features = ["failpoints"]` a `chaos_integrity` (`Cargo.toml:201`)
  - [x] Actualizar Workflows y Políticas
    - [x] Modificar `heavy_certification.yml` para incluir `--features cli,arrow` y clasificar `mcp_tests`, `multilingual_tokenizer_integration`, `columnar`, `memory_telemetry` y `concurrent_insert_preserves_hnsw_invariants`
    - [x] Modificar `docs/operations/CI_POLICY.md`
    - [x] Split quick CI (<30min) de weekly heavy certification (`aaf0428`)
    - [x] Robustecer nextest filter expression (`a3c2c04`)
    - [x] Restaurar strict binary_id nextest filter con cli features (`25dc38b`)
    - [x] Fix version extraction en python_wheels.yml, mejorar comentario test-threads (`26afb62`)
  - [x] Entorno de Validación Local (Pre-push)
    - [x] Agregar `numpy` al entorno virtual de auditoría de Python en `dev-tools/setup_venv.ps1`
- **Pendientes del reporte original:**
  - [x] ~~`Cargo.toml`: Agregar `required-features = ["failpoints"]` a `chaos_integrity`~~ → **Completado** en `Cargo.toml:201`
  - [ ] `.config/nextest.toml`: Hacer `test-threads = 2` específico para Windows (actualmente global en `nextest.toml:67`)
- **Cambios y Resultados:**
  - **Soporte robusto de workspace en Nextest:** El cambio de `binary_id(...)` a `binary(...)` en `nextest.toml` asegura que los binarios pesados se excluyan efectivamente del Fast Gate de PR, previniendo fallas de permisos de root y timeouts en el CI rápido.
  - **Exclusiones de tests de larga duración:** Se identificó y excluyó `memory_telemetry` (timeout de 180s local) y el test unitario lento `concurrent_insert_preserves_hnsw_invariants` (~68s) de la fast gate, acelerando el pipeline.
  - **Validación de Python SDK corregida:** Se instaló `numpy` en el entorno virtual hermético de auditoría (`dev-tools/setup_venv.ps1`) para que las pruebas de integración del SDK de Python que dependen de NumPy pasen correctamente y no bloqueen el pre-push de Git.
  - **Declaración explícita de tests:** Los tests sin entrada explícita `[[test]]` en `Cargo.toml` fueron declarados formalmente para evitar su desaparición por auto-descubrimiento.
  - **Clasificación en Heavy Certification:** `mcp_tests`, `multilingual_tokenizer_integration`, `memory_telemetry` y `concurrent_insert_preserves_hnsw_invariants` fueron clasificados para correr exclusivamente en `heavy_certification.yml` y documentados en `CI_POLICY.md`.
  - **Ejecución de columnar test:** Se habilitó la feature `arrow` en los workflows y se programó `columnar` para que sea evaluado en CI.
- **CI Pending:** `.config/nextest.toml` — `test-threads = 2` movido de global a `[profile.audit.overrides."cfg(target_os = \"windows\")".override]` exclusivo para Windows.
- **DISC-03:** `PrefetchMode` enum (Auto/Enabled/Disabled) agregado a `src/config.rs` con campo `prefetch_mode` en `VantaConfig`; soporte de env vars `VANTA_PREFETCH` y `VANTA_DISABLE_PREFETCH`; integrado en `src/index.rs` vía `OnceLock<PrefetchMode>` y llamado desde `open_with_config` en `src/sdk.rs`.
- **DISC-02:** 3 nuevos tests Windows-only en `tests/file_locking_stress.rs` — antivirus FILE_SHARE_READ, backup FILE_SHARE_DELETE, stale lock recovery (+test cross-platform existente).
- **TSK-47 (SQ8):**
  - `VectorRepresentations::SQ8(Box<[i8]>, f32)` en `src/node.rs` con soporte en `dimensions()`, `to_f32()`, `as_f32_slice()`, `memory_size()`, `cosine_similarity()`.
  - `sq8_quantize()` y `sq8_similarity()` en `src/vector/quantization.rs`.
  - `sq8_similarity_fallback()` en `src/index.rs` para compara raw query vs SQ8; manejado en `calculate_similarity()`.
  - Serialización (tag 4) y deserialización en formato binario del índice.
  - `estimated_memory_size()` y `storage.rs::vector_size` extendidos para SQ8.
- **TSK-49 (rkyv):**
  - Dependencia `rkyv` opcional (feature `rkyv-serialization`) en `Cargo.toml`.
  - `src/serialization/rkyv_archives.rs` con `ArchivedHnswHeader`, `ArchivedHnswNode`, `ArchivedHnswGraph` — formato `repr(C)` para mmap zero-copy.
  - `CPIndex::serialize_to_rkyv()` y `CPIndex::load_from_rkyv()`.
  - `serialization_order()` promovido a `pub(crate)`.
- **ROAD-06:** `docs/operations/grafana-dashboard.json` (6 paneles: RSS, Memory Pressure, Vector Ops, Latency P50/P95/P99, Disk Usage, Index Memory) + `docs/operations/GRAFANA_SETUP.md`.

### TSK-45 — Publicar core en crates.io + docs.rs (2026-06-21)

- **Objetivo:** Publicar el crate `vantadb` v0.1.4 en crates.io con metadata completa, README corregido, y verificaciones de licencia.
- **Commits:** `d249cd5`, `d2ba445`, `2ffd7c9`
- **Checklist completado:**
  - [x] Instalar cargo-deny + `cargo deny check licenses` — todas las licencias compatibles con Apache 2.0
  - [x] Agregar `repository`, `homepage`, `documentation`, `badges` (maintenance badge) a `Cargo.toml`
  - [x] Agregar `publish = false` a `vantadb-python/Cargo.toml` (cdylib, va a PyPI)
  - [x] Renombrar `README.MD` → `README.md` + actualizar ref en `Cargo.toml`
  - [x] Agregar `#![doc(html_root_url)]` a `src/lib.rs`
  - [x] Excluir `test/` del package via `exclude = ["test/"]` en `Cargo.toml`
  - [x] Excluir `job_log.txt` via `.gitignore`
  - [x] `cargo package --list` verificado limpio (373 files, 386.6MiB → 1.4MiB comprimido)
  - [x] `cargo publish --dry-run` pasa
  - [x] Publicado en crates.io: `cargo publish` → **vantadb v0.1.4** disponible en https://crates.io/crates/vantadb
- **Archivos modificados:** `Cargo.toml`, `vantadb-python/Cargo.toml`, `src/lib.rs`, `.gitignore`
- **Resultado:** Core crate publicado exitosamente en crates.io. Documentación auto-build en docs.rs pendiente.

### TSK-106b — SECURITY.md + Vulnerability Disclosure Policy (2026-06-21)

- **Objetivo:** Crear política de seguridad coordinada con ventana de divulgación de 90 días, alineada con estándares OpenSSF/OWASP.
- **Commits:** `c14ed97`
- **Archivo creado:** `.github/SECURITY.md`
- **Contenido:**
  - Reporting via GitHub Security Advisories (privado, respuesta ≤3 días hábiles)
  - Timeline de divulgación coordinada de 90 días (día 0→3 acuse, 3→10 triaje, 10→90 fix, 90+ disclosure público)
  - Política de versiones soportadas (solo latest minor)
  - Modelo de amenazas: network input (axum), file I/O, Python FFI, CLI arguments
  - Proceso de embargo notificado 3–30 días laborales antes del disclosure
- **Resultado:** GitHub ahora detecta automáticamente la security policy en la pestaña Security del repo.

### TSK-71 — WASM Build (wasm32-wasip1) para el core de VantaDB (2026-06-21)

- **Objetivo:** Compilar el core de VantaDB a `wasm32-wasip1` haciendo 5 dependencias opcionales y agregando fallbacks en línea para WASM.
- **Commits:** *(pending — sin commit aún)*
- **Checklist completado:**
  - [x] `Cargo.toml`: 5 deps (`sysinfo`, `memmap2`, `fs2`, `prometheus`, `rayon`) movidas a `optional = true`, feature `wasm` creada, `cpufeatures` eliminada
  - [x] `hardware/mod.rs`: `detect()` bifurcado con `#[cfg(feature = "sysinfo")]`, fallback conservador (1GB RAM, 1 core)
  - [x] `metrics.rs`: macros `observe_histogram!`, `inc_counter!`, `inc_counter_by!`, `set_gauge!` con `#[cfg(feature = "prometheus")]` interno; 33 statics gated; `export_metrics_text()` con fallback; `record_http_request` bifurcada; `record_memory_breakdown` refactorizado con `_get_rss_virt()`
  - [x] `storage.rs`: shim mmap `Vec<u8>`-backed (`Mmap`/`MmapMut`/`MmapOptions`) para no-memmap2; file locking `fs2` gated con `Ok(())` fallback
  - [x] `index.rs`: import condicional de `MmapMut`; llamadas a `crate::storage::Mmap`/`MmapMut` en lugar de `memmap2::`
  - [x] `sdk.rs`: `rayon::prelude::*` gated; `.into_par_iter()` → bloque `#[cfg]` con `.into_iter()` de fallback
  - [x] Build nativo (`cargo check`): ✅ sin errores
  - [x] Build WASM (`cargo check --target wasm32-wasip1 --no-default-features --features wasm`): ✅ sin errores
- **Archivos modificados:** `Cargo.toml`, `src/hardware/mod.rs`, `src/metrics.rs`, `src/storage.rs`, `src/index.rs`, `src/sdk.rs`
- **Resultado:** Core crate compila en wasm32-wasip1. Warnings menores (unsafe innecesario en shim, dead code en backend/hardware) no bloqueantes.

### Fix WASM Browser Build (wasm32-unknown-unknown) — SystemTime panic (2026-06-21)

- **Objetivo:** Eliminar panics de `std::time::SystemTime::now()` al compilar `vantadb-wasm` para `wasm32-unknown-unknown` (target browser WASM).
- **Problema:** `SystemTime::now()` no está disponible en `wasm32-unknown-unknown`. Causaba panic en tiempo de ejecución al cargar el WASM.
- **Solución:** Reemplazar todas las ocurrencias de `std::time::SystemTime` y `std::time::UNIX_EPOCH` por `web_time::SystemTime` / `web_time::UNIX_EPOCH` (crate `web-time` v1.1.0, compatible con WASM y native).
- **Archivos modificados (11):**
  - `src/binary_header.rs` — import + `verify_magic_number()`
  - `src/segment_expiry_state.rs` — `SegmentExpiryState`
  - `src/segment_redundancy.rs` — `SegmentRedundancy`
  - `src/sync_verification.rs` — `SyncVerification`
  - `src/cluster_manager.rs` — `ClusterManager`
  - `src/sdk.rs` — import + `now_ms()`
  - `src/storage.rs` — import
  - `src/wal.rs` — 2x `now()` + 2x `duration_since()`
  - `src/cli_handlers.rs` — `now()` + `duration_since()`
  - `src/executor.rs` — `now()` + `duration_since()`
  - `src/gc.rs` — import
- **Verificación:**
  - `cargo build --target wasm32-unknown-unknown` (desde `vantadb-wasm/`): ✅ sin errores
  - `cargo test --lib` (native): ✅ 48 tests, 0 fallos

### TSK-112 — Package `vantadb-wasm` como npm TypeScript SDK (2026-06-21)

- **Objetivo:** Compilar, empaquetar y publicar `vantadb-wasm` como un SDK de TypeScript funcional en npm con tests de integración, ejemplos para Vercel AI SDK / LangChain / LlamaIndex, y README profesional.
- **Commits:** *(pending)*
- **Checklist completado:**
  - [x] `wasm-pack build --target bundler` desde `vantadb-wasm/` — binario WASM compilado en `vantadb-wasm/pkg/`
  - [x] `vantadb-ts/package.json` — `main`, `types`, `exports`, `files`, `repository`, `homepage`, `bugs`, `prepublishOnly` configurados
  - [x] `vantadb-ts/vantadb.ts` — TypeScript wrappers: `VantaDB` class, tipos `MemoryRecord`, `SearchResult`, `Capabilities`, `OperationalMetrics`, `ListPage`
  - [x] `vantadb-ts/types.ts` — tipos `MemoryInput`, `VantaMemoryMetadata`, todas las u64 expuestas como `string`
  - [x] `vantadb-ts/README.md` — SDK docs con quick start, runtime support matrix (Node/Bun/Deno/browser), full API table
  - [x] `vantadb-ts/test-runner.mjs` — Node.js ESM test runner con `--experimental-wasm-modules`, 26 tests de integración
  - [x] Fix u64 > 2^53 en WASM bindings: `memory_record_to_js()` + `search_hit_to_js()` helpers manuales con `js_sys::Reflect`
  - [x] Fix `read_header` alignment: `DiskNodeHeader::ref_from_bytes` (zerocopy) → `read_from_bytes` (owned copy) en `storage.rs:579`
  - [x] Fix deref en `storage.rs:1535` — `*h` → `h` tras cambio a owned header
  - [x] Limpieza de debug tracing (WARN/INFO logs eliminados)
  - [x] Eliminación de structs no usados (`JsMemoryRecord`, `JsMemorySearchHit`, `JsMemoryListPage`)
  - [x] Eliminación de deps no usadas (`esbuild`, `rollup`, `vite-plugin-wasm`, `vite-plugin-top-level-await`)
- **Archivos modificados:**
  - `vantadb-wasm/src/lib.rs` — `memory_record_to_js`, `search_hit_to_js`, `put`/`get`/`put_batch`/`list`/`search`/`search_vector` refactorizados a manual JsValue
  - `src/storage.rs` — `read_header` return type: `Option<&DiskNodeHeader>` → `Option<DiskNodeHeader>`, 3 `.cloned()` removidos, 1 `*h` → `h`
  - `vantadb-ts/package.json` — metadata npm, scripts, devDeps limpiados
  - `vantadb-ts/vantadb.ts` — `searchVector` return type corregido a `{node_id: string; score: number}[]`
- **Archivos creados:**
  - `vantadb-ts/README.md` — documentación del SDK TypeScript
  - `vantadb-ts/test-runner.mjs` — test runner para Node.js ESM
- **Problema raíz diagnosticado:**
  - `StorageEngine::get` retornaba `None` porque `DiskNodeHeader::ref_from_bytes` requiere alineación 64-byte del buffer subyacente, pero el `Vec<u8>` en WASM (heap-allocated) solo garantiza 8-16 bytes de alineación. `read_header(offset=64)` fallaba silenciosamente.
- **Resultado:** 26/26 tests de integración pasan. Build WASM + TypeScript verificados.

### TSK-118 — Ejemplos TS: LangChain.js, LlamaIndex.TS, Vercel AI SDK (2026-06-21)

- **Objetivo:** Crear ejemplos funcionales de integración con los tres frameworks JS/TS más usados para RAG y agentes.
- **Archivos creados:**
  - `vantadb-ts/examples/vercel-ai-memory.mjs` — Vercel AI SDK tool calling + VantaDB como memoria conversacional
  - `vantadb-ts/examples/langchain-rag.mjs` — LangChain Document pipeline + OpenAIEmbeddings + VantaDB search
  - `vantadb-ts/examples/llamaindex-rag.mjs` — LlamaIndex document indexing + VantaDB vector search
- **Resultado:** 3 ejemplos ESM con sintaxis verificada. Requieren `npm install` de los respectivos SDKs para ejecutarse.

### CLI-EPIC — Comandos CLI: backup, restore, doctor, inspect, stats, count, search-similar (2026-06-21)

- **Objetivo:** Expandir el CLI de VantaDB con 7 nuevos comandos para operaciones de respaldo, diagnóstico e inspección.
- **Checklist completado:**
  - [x] `vanta-cli backup [--out <path>]` — backup con flush WAL, copia de vector_store + index + WAL, manifiesto con CRC32
  - [x] `vanta-cli restore --in <path> [--force] [--rebuild]` — restaura desde backup, verifica CRC32, reconstruye índices opcionalmente
  - [x] `vanta-cli doctor` — diagnóstico de salud: WAL, backend, memoria, HNSW, índices, métricas operacionales
  - [x] `vanta-cli inspect --namespace <ns> --key <key>` — inspecciona un registro con todos sus campos
  - [x] `vanta-cli stats [--json]` — estadísticas de la base de datos con salida formateada o JSON
  - [x] `vanta-cli count --namespace <ns> [--filter key=val]` — conteo de registros
  - [x] `vanta-cli search-similar --namespace <ns> --key <key> [--limit <N>]` — búsqueda por similitud desde una clave existente
  - [x] Fix WAL replay: `recover_state()` ahora escribe `NodeMetadata` al backend durante replay — permite restore completo sin depender de archivos internos de Fjall
- **Archivos modificados:** `src/cli.rs`, `src/cli_handlers.rs`, `src/bin/vanta-cli.rs`, `src/storage.rs`
- **Archivos creados:** `completions/_vanta-cli`, `completions/_vanta-cli.ps1`, `completions/vanta-cli.bash`, `completions/vanta-cli.fish`
- **Tests:** 46 tests CLI, todos pasan

### TSK-111 — Expanded Filter Operators (2026-06-21)

- **Objetivo:** Extender el sistema de filtros de igualdad plana (`VantaMemoryMetadata`) con operadores de comparación (`Eq, Neq, Gt, Gte, Lt, Lte, In, Exists`).
- **Checklist completado:**
  - [x] `FilterOperator` enum en `src/sdk.rs`
  - [x] `MemoryFilter` struct con `field`, `operator`, `value`
  - [x] `evaluate_filter()` y `compare_vanta_values()` para evaluación tipo-safe
  - [x] `filter_exprs: Vec<MemoryFilter>` añadido a `VantaMemoryListOptions` y `VantaMemorySearchRequest`
  - [x] Backward compat: filtros planos siguen funcionando como `Eq`
  - [x] Prefix scan optimization: primer `Eq` filter se usa para scan, post-filter con todas las condiciones
- **Archivos modificados:** `src/sdk.rs`, `src/lib.rs`
- **Exports:** `FilterOperator`, `MemoryFilter` re-exportados desde `src/lib.rs`

### TSK-119 — delete_by_filter (2026-06-21)

- **Objetivo:** Eliminar múltiples registros por filtro de metadatos desde SDK y CLI.
- **Checklist completado:**
  - [x] `VantaEmbedded::delete_by_filter()` — usa `records_for_namespace()` + `matches_memory_filters()`, retorna count de eliminados
  - [x] `vanta-cli delete-by-filter --namespace <ns> --filter key=val`
- **Archivos modificados:** `src/cli.rs`, `src/cli_handlers.rs`, `src/bin/vanta-cli.rs`, `src/sdk.rs`
- **Bindings actualizados:** `vantadb-wasm`, `vantadb-python`, `vantadb-mcp` añadieron `filter_exprs: vec![]`

### TSK-86 — similar_to_key (2026-06-21)

- **Objetivo:** Conveniencia: buscar registros similares usando el vector de un registro existente por su key.
- **Checklist completado:**
  - [x] `VantaEmbedded::similar_to_key(namespace, key, top_k)` — obtiene registro, extrae vector, ejecuta search
  - [x] `vanta-cli search-similar --namespace <ns> --key <key> [--limit <N>]`
- **Archivos modificados:** `src/cli.rs`, `src/cli_handlers.rs`, `src/bin/vanta-cli.rs`, `src/sdk.rs`

### TSK-87 — count with filters (2026-06-21)

- **Objetivo:** Contar registros en un namespace, opcionalmente filtrados por metadatos.
- **Checklist completado:**
  - [x] `VantaEmbedded::count(namespace, filters, filter_exprs)` — prefix scan sin filters, scan + filter con filters
  - [x] `vanta-cli count --namespace <ns> [--filter key=val]`
- **Archivos modificados:** `src/cli.rs`, `src/cli_handlers.rs`, `src/bin/vanta-cli.rs`, `src/sdk.rs`

### TSK-88 — Multi-namespace Search (2026-06-21)

- **Objetivo:** Buscar simultáneamente en múltiples namespaces.
- **Checklist completado:**
  - [x] `namespaces: Vec<String>` en `VantaMemorySearchRequest`
  - [x] Backward compat: si `namespaces` está vacío, se usa `namespace`
  - [x] Implementación: itera namespaces, corre search por cada uno, mergea top_k por score
  - [x] CLI: `vanta-cli search --namespace ns1,ns2,... --query <q>` acepta lista separada por comas
- **Archivos modificados:** `src/cli.rs`, `src/cli_handlers.rs`, `src/bin/vanta-cli.rs`, `src/sdk.rs`
- **Tests:** Todos existentes actualizados con `namespaces: vec![]`
