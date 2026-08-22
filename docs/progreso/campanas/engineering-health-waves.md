# Engineering Health Waves — bloqueantes F0, VFY, COMP, IVF

> **Migrado desde** `docs/progreso/README.md` (split GOV-D2, 2026-08-22). Contenido histórico sin cambios salvo dedup indicado.

### 2026-07-25 — Bloqueantes de Release Fase 0: 3 completadas, 1 diferida

**Objetivo:** Cerrar los 6 items de Fase 0 que bloqueaban el release público. 3 implementadas, 1 diferida (nice-to-have pre-1.0), 2 ya completadas previamente.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `DEVOPS-15` | Optimizar default features Cargo.toml | `Cargo.toml:89` | ❌ **WONTFIX** — Analizado y NO aplicado. Reducir de 7 a 3 features (`cli`, `memmap2`, `fs2`, `sysinfo`) rompe la experiencia "it just works". 7 features mantienen UX completo. |
| `REV-014` | Dependabot PRs → develop branch | `.github/dependabot.yml` | ✅ `target-branch: develop` agregado a los 4 ecosystems (cargo, npm, github-actions, docker). |
| `DRV-125` | Tests Miri para 30+ usos unsafe en src/index/ | `src/index/distance.rs`, `search.rs`, `graph.rs`, `serialize.rs` | ✅ **21 tests Miri pre-existentes** verificados: 5 en distance.rs, 3 en graph.rs, 6 en search.rs, 7 en serialize.rs. Cubren todos los patrones unsafe. |
| `DEVOPS-10` | Windows code signing (SmartScreen) | `release-binaries-63.yml` | 🔵 DIFERIDO (ponytail). SHA256 + .zip dan integridad básica. Agregar Azure Trusted Signing cuando el release público lo requiera. Step YAML preparado en el archivo de tarea. |

**Ids:** `DEVOPS-15`, `REV-014`, `DRV-125`, `DEVOPS-10`

### 2026-07-25 — P4 Engineering Health Wave 0: DOC-20 (sitio de docs mdBook)

**Objetivo:** Unificar docs fragmentados en un mdBook con search integrado.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `DOC-20` | mdBook adoption for docs site | `docs/book/book.toml`, `docs/book/src/SUMMARY.md`, 73 `{{#include}}` stubs | ✅ `1f9f681d` — mdBook con 9 secciones (User Guides, API, Architecture, Operations, Strategy, Reference, Blog, Case Studies, Project). Cero duplicación de contenido existente. 83 páginas HTML generadas. |

**Verificación:** `mdbook build docs/book/` ✅ — 83 archivos en `docs/book/book/`, `index.html` funcional.

### 2026-07-25 — P4 Engineering Health Wave 0: WEB-03 (batching async de fsyncs WAL)

**Objetivo:** fsync paralelo para los shards de WAL — `flush_all` lanza un thread por shard.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `WEB-03` | Async WAL batching fsyncs | `src/wal_sharded.rs` | ✅ `c59e0f80` — `flush_all` fsync paralelo por shard. Short-circuit de shard único. |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo test --package vantadb --lib wal_sharded` ✅ (25/25)

<!-- movido a ARCHIVO_HISTORICO.md -->

### 2026-07-25 — P4 Engineering Health Wave 0: WEB-04 (versionado de formato de Storage)

**Objetivo:** Implementar el borrador `STORAGE_VERSIONING.md` — `validate_compat()` check basado en rangos para VantaFile/HNSW/WAL.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `WEB-04` | Storage format versioning | `src/binary_header.rs`, `src/lib.rs` | ✅ `21432104` — `VantaHeader::validate_compat()` check de rango. Magic + `format_version ≤ max_version`. Constants `VFILE_VERSION`, `VECTOR_INDEX_VERSION`, `WAL_FORMAT_VERSION` hechas pub. `STORAGE_VERSIONING.md` marcado como implementado. |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo test --package vantadb --lib binary_header` ✅

### 2026-07-25 — P4 Engineering Health Wave 0: DRV-121 (optimización CBO del Planner)

**Objetivo:** Predicate pushdown (orden por selectividad) + eliminación de filtros (identity filter con sel≥1.0 omitido).

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `DRV-121` | Planner CBO optimization | `src/planner.rs` | ✅ `21432104` — Filtros ordenados por selectividad estimada (ascendente). Identity filters con sel ≥ 1.0 omitidos. Constants `HIGH_SELECTIVITY_THRESHOLD`, imports `FieldValue`/`RelOp`. Test para eliminación de identity filters. |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo test --package vantadb --lib planner` ✅

### 2026-07-25 — P4 Engineering Health Wave 0: DRV-123 (pulido de Auto-embedding en INSERT)

**Objetivo:** Pulido del manejo de errores para auto-embedding `remote-inference` en INSERT.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `DRV-123` | Auto-embedding INSERT polish | `src/llm.rs`, `src/executor.rs` | ✅ `21432104` — `match` reemplaza `if let Ok`, `tracing::warn!` en fallo. Guard de texto vacío `!text.trim().is_empty()`. Aplicado a los paths de `node_id` y `InsertMessage`. Agregado el test `test_auto_embedding_graceful_degradation_on_insert`. |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo test --package vantadb --lib executor` ✅

### 2026-07-26 — P4 Engineering Health Wave 0: VFY-011 (ACID Fase 3 — MVCC/Isolation de Snapshot)

**Objetivo:** Snapshot isolation / MVCC para lecturas consistentes durante escrituras concurrentes.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `VFY-011` | ACID Fase 3 — Isolation de snapshot / MVCC | `src/storage/engine/ops.rs`, `src/storage/engine/mod.rs`, `src/storage/engine/init.rs`, `src/storage/ops.rs` | `Snapshot { txn_id: u64 }` struct + `get_with_snapshot()` filtrado MVCC. `active_txns: HashSet<u64>` reemplaza `active_txn_id: Mutex`. Detección de conflictos write-write vía `check_write_conflict()`. 7 tests nuevos: ciclo de vida de snapshot, visibilidad committed/uncommitted/deleted, txns concurrentes, conflictos write-write. `created_by_txn`/`deleted_by_txn` en NodeMetadata. |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo test -p vantadb --lib storage::engine::tests::ops` 62 pasaron ✅

### 2026-07-26 — DRV-122: JOINs IQL, Subqueries y Compatibilidad SQL

**Objetivo:** Implementar parser SELECT/JOIN/subquery, operador físico NestedLoopJoin, subquery filter, integración con planner.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `DRV-122` | IQL JOINs/subqueries/SQL compatibility | `src/query.rs`, `src/parser/mod.rs`, `src/executor.rs`, `src/planner.rs`, `tests/logic/joins.rs`, `Cargo.toml` | ✅ 3 fases: (1) parser SELECT/JOIN/subquery + plan types `de189a8c`, (2) operadores físicos NestedLoopJoin + SubqueryFilter `345d1939`, (3) integración con planner + 10 tests nuevos `6449469f`. 1559 tests pasan (0 fallos). |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo test --package vantadb --lib parser::tests` 97 pasaron ✅ | `cargo test --package vantadb --lib joines` 10 tests JOIN nuevos ✅

### 2026-07-26 — Fase 7: NUEVO-13 auto-tuning de ef_search HNSW

**Objetivo:** Mejorar el auto-tuning de HNSW ef_search con heuristic doubling, dampening factor 1.5x, gauge de métrica y test de integración.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `NUEVO-13` | HNSW ef_search auto-tuning (heuristic doubling) | `src/index/auto_tune.rs`, `src/metrics/core/mod.rs`, `src/metrics/core/registry.rs` | ✅ +66/-9 — Dampening 2.0→1.5x, gauge `vantadb_auto_tune_ef`, integration test `repeated_fallbacks_increase_ef`. Tests actualizados con nueva curva. |

**Verificación:** `cargo nextest run --profile audit -p vantadb` ✅ | 4 auto_tune tests + gauge test ✅

### 2026-07-26 — DRV-131: Tipos de índice faltantes más allá de HNSW — IVF Flat

**Objetivo:** Agregar nuevos tipos de índice vectorial más allá de HNSW. Implementado IVF Flat (inverted file con k-means, sin dependencias externas).

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `DRV-131` | IVF Flat index | `src/index/ivf.rs` (NEW, 836L), `src/index/mod.rs`, `src/index/graph.rs`, `src/index/search.rs`, `src/index/serialize.rs`, `src/index/core.rs`, 6 test/bench files | ✅ IVF implementado: `IvfIndex` con k-means (Forgy + Lloyd, max 20 iter), search con nprobe, serialización v8 con backwards compat v7. 16 tests IVF. 1547 tests lib pasan. 0 clippy warnings nuevos. |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo test -p vantadb --lib` 1547 pasaron ✅ | `cargo clippy -p vantadb` limpio ✅

**Ids:** `DRV-131`

<!-- movido a ARCHIVO_HISTORICO.md -->

### 2026-07-26 — P8 Post-Launch y Enterprise: CLI-01, DEVOPS-HOMEBREW, DEVOPS-PY313, DEVEX-DEMO, DEVEX-EXAMPLES ✅

**Objetivo:** Pipeline paralelo de 5 tareas P8. 3 delegadas a `vanta-worker` (Rust/Python), 2 procesadas por `vanta-lead` (CI/CD).

| ID | Tarea | Resultado |
|----|-------|-----------|
| `CLI-01` | **CLI handlers backup/restore/doctor/stats/inspect** — Conectar 5 handlers existentes al dispatcher CLI | ✅ `src/cli.rs` + `src/bin/vanta-cli.rs`. 5 comandos nuevos conectados. 46 tests CLI pasan. Delegado a vanta-worker. |
| `DEVOPS-HOMEBREW` | **Homebrew formula** — Ya implementada (`Formula/vantadb.rb`) | ✅ Solo documentación. Placeholder SHA256 — actualizar antes del publish. |
| `DEVOPS-PY313` | **Python 3.13 wheels en CI matrix** — CI verify jobs actualizados a Python 3.13 | ✅ Verify-testpypi-install + verify-pypi-install ahora usan CPython 3.13. Build mantiene 3.11 con `abi3-py311`. |
| `DEVEX-DEMO` | **Demo app** — `examples/demo/` con Python 239L + README + requirements | ✅ Delegado a vanta-worker. Syntax check limpio. |
| `DEVEX-EXAMPLES` | **Rust examples** — 4 ejemplos existentes en `examples/rust/` | ✅ Solo documentación. basic, hybrid, graphrag, concurrent compilan limpio. |

**Verificación:** `cargo check` ✅ | Backlog.md P8 counter 13→8 | CLI 7→8 | Infra CI 2→4 | Total 90→95

<!-- movido a ARCHIVO_HISTORICO.md -->

### 2026-07-27 — P5/P6/P8 Quick Wins: 8 tareas ejecutadas

**Fuente:** Backlog P5 (Docs y Community), P6 (Launch Campaign), P8 (Post-Launch)

**Objetivo:** Ejecutar tareas rápidas de community, marketing y documentación en paralelo.

| ID | Tarea | Resultado |
|----|-------|-----------|
| `TSK-106` | Habilitar GitHub Discussions | ✅ Ya estaba habilitado (`has_discussions: true`). Sin cambios. |
| `MKT-03` | Show HN draft → v0.4.0 | ✅ Draft actualizado con APIs correctas (`put`, `search_memory`) y links a PyPI/docs |
| `NUEVO-21` | Vectara competitive research | ✅ Reporte en `docs/Investigaciones/vectara-competitive-research-2026-07-27.md`. Hallazgo: Vectara cerró self-service → gap para local-first |
| `MKT-04` | Reddit posts (3 subreddits) | ✅ 3 drafts en `docs/strategy/REDDIT_POSTS.md` (r/rust, r/MachineLearning, r/LocalLLaMA) |
| `TSK-107` | Community showcase page | ✅ 6 items actualizados: apuntan a ejemplos reales (LangGraph, AutoGen, Haystack, CrewAI, Rust hybrid, GraphRAG) |
| `COM-03` | Discord forums + AutoMod | ⚠️ Parcial: 9 threads seedeados (FAQ/Showcase/Ideas/Bug). AutoMod/stickers/emojis requieren Discord UI manual |
| `COM-04` | Discord stage + ticketing | ⚠️ Parcial: Stage channel creado. Ticketing, Server Discovery (1000+ miembros), Canny.io requieren pasos externos |
| `—` | Good first issues (18 open) | ✅ 22 issues creados (#118-#142), 3 duplicados cerrados (#136-#138). Etiquetados `good first issue` en GitHub |

**Archivos creados/modificados:**
- `docs/Investigaciones/vectara-competitive-research-2026-07-27.md` (nuevo)
- `docs/strategy/SHOW_HN_PREP.md` (actualizado)
- `docs/strategy/REDDIT_POSTS.md` (nuevo)
- `web/src/app/showcase/page.tsx` (actualizado)
- `docs/discord/todo.md` (actualizado)
- `docs/discord/server-config.md` (actualizado)

**Ids:** `TSK-106`, `MKT-03`, `NUEVO-21`, `MKT-04`, `TSK-107`, `COM-03`, `COM-04`

---

### 2026-07-27 — COMP-010: Abstracción de la función de auto-embedding

**Objetivo:** Refactorizar auto-embedding de `LlmClient` concreto (Ollama hardcodeado) a trait `EmbeddingProvider` abstracto con múltiples implementaciones.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `COMP-010` | Trait `EmbeddingProvider` + `OllamaProvider` + `OpenAIProvider` + factory | `src/llm.rs`, `src/executor.rs`, `src/physical_plan.rs` | ✅ Trait definido. OllamaProvider (default, existente). OpenAIProvider (nuevo, `/v1/embeddings`). Factory `get_embedding_provider()` lee `VANTA_EMBEDDING_PROVIDER`. 4 call sites actualizados. `LlmClient` preservado para `summarize_context()`. |

**Verificación:** `cargo check -p vantadb --features remote-inference` ✅ | `cargo test --package vantadb --lib executor` 26/26 ✅ | `cargo clippy -p vantadb --features remote-inference -- -D warnings` ✅ | `cargo fmt --check` ✅

### 2026-07-27 — COMP-008: Motor de índice enchufable (trait VecIndex)

**Objetivo:** Abstraer operaciones de index vectorial (HNSW, IVF, flat scan) detrás de un trait `VecIndex` para desbloquear múltiples backends (COMP-027).

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `COMP-008` | Trait `VecIndex` con `search`/`add`/`len`/`estimate_memory_bytes`. Implementado para `CPIndex` (HNSW) e `IvfIndex`. `vector_memory_search` actualizado a `engine.vec_index()`. Fix a `vantadb-mcp` por rotura de COMP-006. | `src/index/mod.rs`, `src/index/search.rs`, `src/index/ivf.rs`, `src/sdk/search/mod.rs`, `src/storage/engine/mod.rs`, `vantadb-mcp/src/lib.rs` | ✅ Trait definido. Ambos backends implementan `VecIndex`. `vector_memory_search` usa trait object. Workspace completo compila con `-D warnings`. **1679 tests pasan.** |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo check --benches -p vantadb` ✅ | `cargo clippy --workspace --all-targets -- -D warnings` ✅ | `cargo fmt --check` ✅ | `cargo nextest run --profile audit --workspace --build-jobs 2` ✅ 1679/1679 pasaron

**Ids:** `COMP-008`

### 2026-07-27 — COMP-014: FreshHNSW (Background Repair de Enlaces Huérfanos)

| Tarea | Logro | Archivos | Estado |
|-------|-------|----------|--------|
| `COMP-014` | FreshHNSW: `repair_orphan_links()` de tres fases (snapshot→scan→repair) evita deadlock de DashMap. `FreshHnswReport`, `PipelineMode::FreshHnswOnly`, fase de pipeline entre Vacuum y Merge. 4 tests (empty, no-orphans, after-delete, multi-layer). | `src/index/graph.rs`, `src/storage/engine/mod.rs`, `src/storage/engine/maintenance.rs` | ✅ 4/4 tests (0.49s), cargo check ok |

### 2026-07-27 — COMP-013: Pipeline de optimización de segmentos (Vacuum/Merge/Index)

**Objetivo:** Construir pipeline formal de optimización de segmentos en background (Vacuum → Merge → Index). Ya existían piezas sueltas (`compact_layout_bfs`, `trigger_compaction`, `rebuild_vector_index`) pero sin orquestación.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `COMP-013` | Pipeline orquestado con `PipelineMode` (Full/VacuumOnly/MergeOnly/IndexOnly), `vacuum()` purga tombstones del HNSW, `merge_segments()` compactación BFS condicional, `run_pipeline()` orquestación secuencial con tolerancia a fallos por fase | `src/storage/engine/mod.rs`, `src/storage/engine/maintenance.rs`, `src/config.rs`, `src/sdk/api.rs`, `src/storage/engine/tests/maintenance.rs` | ✅ `PipelineMode`, `VacuumReport`, `MergeReport`, `PipelineReport`, `SegmentOptimizerConfig` en `VantaConfig`. SDK expone `vacuum()`, `pipeline()`, `optimizer_config()`, `set_optimizer_config()`. 77 tests de mantenimiento pasando. |

**Verificación:** `cargo check -p vantadb` ✅ | `cargo test -p vantadb -- maintenance` 77/77 ✅

### 2026-07-27 — COMP-009: Importación masiva binaria

**Objetivo:** Protocolo binario de importación masiva 5-10x más rápido que `put_batch()`, con bypass de validación por registro y batch commit.

| ID | Tarea | Archivos | Resultado |
|----|-------|----------|-----------|
| `COMP-009` | Formato `.vdbdump` (magic `VDBJSON\n` + version + count + serde_json body), `bulk_import_stream()` bypass validación, `bulk_import_file()`, `bulk_commit_interval` en config. Python: `VantaDB.bulk_import()` + `VantaDB.bulk_import_bytes()`. WASM: `VantaDB.bulk_import()` + `VantaDB.bulk_import_bytes()` | `src/sdk/api.rs`, `src/config.rs`, `vantadb-python/src/lib.rs`, `vantadb-python/src/convert.rs`, `vantadb-python/vantadb_py/__init__.py`, `vantadb-wasm/src/lib.rs`, `docs/Backlog.md`, `.opencode/skills/campaign-executor/tasks/COMP-009.md` | ✅ `BulkImportReport` struct + bulk_import_stream + bulk_import_file. Python async wrappers. WASM Uint8Array binding. 3 tests pasando. `bulk_commit_interval` configurable |

**Verificación:** `cargo check` (workspace completo) ✅ | `cargo test -p vantadb -- tests::test_bulk_` 3/3 ✅

---
