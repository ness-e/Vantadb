---
title: "Investigación Multi-Agente — VantaDB (equipo técnico, 2026-08-09)"
type: investigation-report
date: 2026-08-09
tags: [vantadb, investigation, multi-agent, release-readiness]
---

# Investigación Multi-Agente — VantaDB (2026-08-09)

## Alcance y método

Investigación coordinada de **6 sub-agentes** sobre el repositorio completo de VantaDB, seguida de **verificación externa** contra los registros públicos:

| Sub-agente | Enfoque |
|---|---|
| **vanta-worker** | Core engine: ingest, WAL, persistencia, ciclo de vida de datos |
| **vanta-arch** | Arquitectura: capas, módulos, API estable vs superficies históricas |
| **vanta-audit** | Auditoría de producción: seguridad, FFI, bloqueadores de release |
| **vanta-tuner** | Rendimiento: benchmarks, hot paths, parametrización |
| **vanta-docs** | Documentación, DX, coherencia de versionado, marketing |
| **vanta-chaos** | Robustez: recovery, corrupción, crash-safety, confianza de la suite |

**Verificación externa** (2026-08-09): crates.io (`vantadb` v0.4.0), PyPI (`vantadb-py`), npm, GitHub (`ness-e/Vantadb`).

Fuentes canónicas internas: `docs/Backlog.md`, `docs/reviews/errors-found.md`, `docs/CHANGELOG.md`, código en `src/`.

---

## 1. Qué es y cómo funciona

VantaDB es un **motor embebido de memoria persistente + búsqueda vectorial híbrida**, local-first, escrito en Rust (Apache-2.0, v0.5.0 en repo).

**Flujo de una query:**

```
VantaEmbedded.put() → StorageEngine (WAL sharded CRC32C → Fjall/RocksDB/in-memory)
  → planner clasifica la query: Hybrid / TextOnly / VectorOnly
     (BM25 lexical + HNSW vectorial + sparse opcional)
  → fusión RRF (K=60)
```

- **API estable:** `src/sdk/` (struct `VantaEmbedded`, `src/sdk/api.rs`, builder `src/sdk/builder.rs`, `src/sdk/connect.rs`). Esa es la superficie de contrato.
- **NO es API estable:** el parser IQL histórico (`src/parser/mod.rs`) — superficie legada, origen del bug ERR-016 (ver §7).
- Capas: core (`src/`) → bindings (Python `vantadb-python/`, TS `vantadb-ts/`, MCP `vantadb-mcp/`, node `vantadb-node/`, WASM `vantadb-wasm/`, HTTP `src/cli_handlers/server.rs` + crate `vantadb-server/`) → adaptadores IA (LangChain, llama_index, haystack, DSPy, CrewAI, Mem0, Letta, OpenAI, Ollama).

## 2. ¿Qué problema resuelve?

**Memoria de largo plazo local para LLMs y agentes**, sin servidores externos ni dependencias cloud:

- Durable (WAL), crash-safe (CRC32C + recovery multi-modo), híbrido (lexical + vectorial + sparse).
- Compite con Mem0, Letta, LanceDB, Chroma — con enfoque embedded-first: **"SQLite para agentes"**.
- El valor diferencial **no es** la búsqueda vectorial (commodity): es la **durabilidad crash-safe + almacenamiento híbrido + operabilidad local** en un único binario embebible.

## 3. Funcionalidades — inventario con evidencia

Estado: ✅ = implementado y con suite; ⚠️ = implementado con bugs conocidos, sin publicar o experimental; ❌ = no existe.

| Funcionalidad | Estado | Evidencia (archivo) |
|---|---|---|
| SDK memory CRUD (`VantaEmbedded`) | ✅ | `src/sdk/api.rs`, `src/sdk/mod.rs` |
| Hybrid RRF 3-arm (BM25 + HNSW + sparse) | ✅ | `src/text_index.rs`, `src/index/search.rs`, `src/planner.rs` |
| HNSW + ACORN filtered search | ✅ | `src/index/graph.rs` (`HnswNode`, `HnswConfig`, `insert_hnsw`), `src/index/search.rs` |
| WAL sharded + recovery (3 modos, CRC32C) | ✅ | `src/wal.rs`, `src/wal_sharded.rs`, `src/wal_archiver.rs`, tests `tests/core/` |
| LSM multi-nivel + GC + tombstones | ✅ | `src/lsm.rs`, `src/gc.rs` |
| Graph dirigido + BFS/DFS + temporal + GDS/PageRank | ✅ | `src/graph.rs`, `src/gds.rs`, `src/index/graph.rs` |
| GraphRAG pipeline | ✅ | `examples/rust/graphrag.rs`, `tests/graphrag_test.rs`, `benchmarks/graphrag_bench.rs` |
| Snapshots / backup / restore + JSONL export/import | ✅ | `src/cli_handlers/snapshot.rs`, `src/cli_handlers/backup.rs`, tests `tests/core/snapshot_certification.rs` |
| Encryption AES-256-GCM at-rest | ✅ | `src/crypto.rs` (`Cipher`, `EncryptionStream`), `src/storage/vfile.rs`, feature `encryption` en `src/lib.rs` |
| CLI completo | ✅ | `src/cli.rs`, `src/cli_handlers/` |
| Server HTTP (RBAC + rate-limit + Auth) | ⚠️ experimental | `src/cli_handlers/server.rs`, crate `vantadb-server/` (no bloquea CI) |
| Bindings Python / TS / WASM / MCP / node | ⚠️ con bugs | `vantadb-python/`, `vantadb-ts/`, `vantadb-wasm/`, `vantadb-mcp/`, `vantadb-node/` — UAF PyO3 y OOM MCP sin fix (ver §7) |
| IVF / SCANN / DiskANN / SQ8 quant | ⚠️ implementados, **sin exposición SDK** | `src/index/ivf.rs`, `src/index/scann.rs`, `src/index/diskann.rs`, `src/vector/quantization.rs` |
| Índices derivados, prefetch mmap, SIMD | ✅ | `src/index/`, `src/sdk/search/text_index.rs`; prefetch default OFF (medición, §8) |
| IVM (incremental view materialization) / BOTV 2026 | ⚠️ en roadmap | IVM/BOTV listados como planeados para 2026; sin firma encontrada en `src/` a fecha del reporte |

## 4. Estado de completitud por módulo

**Al 100% (10):**
1. CRUD + export/import JSONL
2. Retrieval híbrido (RRF)
3. HNSW recall ≥ 0.90 (unit-gated)
4. WAL recovery + truncado/corrupción (3 modos, CRC32C)
5. LSM multi-nivel + GC + tombstones
6. Graph temporal + GDS/PageRank
7. Graph RAG pipeline
8. Snapshots / backup / restore
9. Encryption AES-256-GCM
10. CLI completo

**Parciales (con evidencia):**

- **PITR / WAL-shipping**: módulos reales (`src/wal_shipping.rs`, `src/wal_archiver.rs`) pero **huérfanos** — no son invocados por StorageEngine/SDK, solo self-tests; feature `pitr = []` vacía en Cargo.toml.
- **DiskANN**: es *purely in-memory* (no hace disk I/O real) — `src/index/diskann.rs`.
- **Arrow columnar**: solo exporta `id` + primera componente del vector — `src/sdk/serialization/`.
- **IVF / SCANN**: implementados dentro de `VecIndex` (`src/index/ivf.rs`, `src/index/scann.rs`) pero **sin exposición a SDK**.
- **TUI / MCP / server / WASM**: marcados `experimental` en Cargo.toml (no bloquean CI).
- **`src/integrations.rs`**: stubs — `search_handler` devuelve `[]`, `ollama_proxy` dice "Próximamente".
- **Hot-reload de configuración**: existe pero **sin documentación**.

**No existe:**
- Cluster / replication / distributed (solo `src/wal_shipping.rs` send-only, sin receive).

## 5. ¿Qué se puede hacer hoy?

| Ruta | Ejemplo |
|---|---|
| **Rust SDK** | `VantaEmbedded::open(path)?` → `put` / `search` híbrido con filtros |
| **Python** | `pip install vantadb-py` → `VDBMemory` para agentes |
| **TypeScript / WASM** | `import { MemoryClient } from "@vantadb/sdk"` |
| **MCP stdio** | `"vantadb-mcp"` → herramientas de memoria para agentes que hablan MCP |
| **HTTP server** | `--server` (axum) con RBAC + rate-limit en `localhost` |
| **CLI** | `vantadb memory add`, `vantadb search`, `vantadb export --jsonl` |
| **Node** | `vantadb-node/` (napi bindings) |

## 6. Lo que se ve en la red (verificación externa)

| Registro | Datos reales | Estado |
|---|---|---|
| **GitHub** `@ness-e/Vantadb` | ⭐ 2 stars, 0 forks, 1381 commits, 14 workflows | Publicado, visibilidad ~cero |
| **crates.io** `vantadb` | v0.4.0 (repo en 0.5.0) · 32 descargas · **0 dependents** | Publicado, drift |
| **PyPI** | `vantadb-py` 0.1.5 (+0.5.0) **vs core repo 0.5.0** | Publicado, drift |
| **npm** | TS SDK v0.5.0 | Publicado |

**Lectura clave:** la creencia "no he publicado" es incorrecta a medias — **está publicado pero sin distribución** (2 stars, 32 descargas en crates.io, 0 dependents). El bloqueo real = blockers de corrección + benchmarks incoherentes + docs/marketing sin terminar. Además, `llms.txt` existe pero con API Python **inventada** (rota).

## 7. Veredicto: ¿está justificado no publicar/avanzar?

Sí hay que **arreglar antes de anunciar**. No es "falta de features": la infra está presente. Son **bugs reales no resueltos** con ubicación exacta:

| ID | Tipo | Severidad | Evidencia |
|---|---|---|---|
| **ERR-022** | `top_k`/`k` sin clamp → alloc gigante → **crash/OOM del proceso** | Crítica (DoS) | `src/index/search.rs:522-601` (alloc `HashSet::with_capacity(ef*3)`), `vantadb-mcp/src/lib.rs:1301`, `vantadb-python/src/lib.rs` (858, 1246), `vantadb-wasm/src/lib.rs` (736-738) |
| **ERR-021** | MCP OOM: `collection_stats/list/delete` materializan tablas completas via `collect_all_records`; streaming eliminado | Crítica (OOM) | `vantadb-mcp/src/lib.rs:333-365, 1401, 1430, 1499` |
| **ERR-016** | Parser consume `WHERE`/`RANK` como alias → **pérdida silenciosa de filtro** en queries | Crítica (data loss) | `src/parser/mod.rs:174-175` + `src/index/executor.rs:160` — ✅ **resuelto** 2026-08-09 (`non_keyword_ident` + 3 tests) |
| **UAF** | use-after-free en `__array_interface__` (PyO3) | Crítica (bloquea PyPI) | `vantadb-python/` bindings FFI |
| **ERR-010** | Raza checkpoint↔snapshot (corrupción/duplicación) | Resuelto | `src/storage/engine/maintenance.rs` — fix en v0.4.0 |
| **ERR-035/036** | Read-lock global del HNSW retenido durante todo `search_nearest` → writers quedan congelados por queries | Media (contención) | `src/physical_plan.rs` (289-290), `src/storage/engine/ops.rs:507,1234` |

**Escalas combinadas (6 sub-agentes):** producción-ready **5/10** (audit) · confianza de suite **3/5** (chaos). Conclusión: 3 blockers crash/OOM (ERR-022, ERR-021) + 1 data loss (ERR-016) + 1 UAF — el resto es polish de docs y benchmark.

## 8. Configuración / DX / UX / Rendimiento

### Configuración
- ✅ `src/config.rs` (builder + env vars presentes, ~1300 líneas).
- ⚠️ **No existe** `.vanta/config.toml`.
- ⚠️ Hot-reload de config JSON **no documentado**.
- ✅ `hardware_profiles` (auto-detection de recursos) destacable.

### DX de desarrollo
- ✅ Justfile cross-platform, CONTRIBUTING/CLA.
- ⚠️ **QUICKSTART desactualizado** (dice v0.4.x; repo es 0.5.0).
- ⚠️ Wheel local stale (`dist/vantadb_py-0.1.5`); badge PyPI 0.1.5 vs core 0.5.0 = **drift de versionado**.

### UX / docs
- ✅ 8 docs API, 13 ADRs, 30+ ops docs, TEST_MAP.
- ⚠️ CHANGELOG muerto (ultima entrada 0.5.0); falta `[Unreleased]` (ERR-050).
- ⚠️ Mojibake en `vantadb-python/README.md` y `docs/DESIGN_RULES.md`.
- ⚠️ `llms.txt` con API inventada; `docs/README.md` con wikilinks que no se renderizan en GitHub.
- ⚠️ Límite u64 (ERR-023) sin documentar.
- ⚠️ Quickstart de pip OK (~5 min), pero CLI para usuario final de a pie exige toolchain Rust (no hay binario distribuido).

### Rendimiento (clave — los números publicados mienten)
- ✅ Core SIFT1M@100K → 3636 QPS, p99 441 µs.
- ⚠️ **`benches/hnsw_pure.rs` mide brute-force** (`flat_threshold=10000`) — no HNSW real (ERR-019).
- ⚠️ **Doble verdad publicada**: JSON local vs tabla de CI no coinciden.
- ⚠️ Comparativo repo: 598 ingest QPS vs 114K QPS (LanceDB) (**190×**); 24 query QPS vs 906 (Chroma) — resultado fuera de proporción por el benchmark roto.
- ⚠️ `criterion_baseline.json` vacío.
- ⚠️ Hot path top-3: ERR-036 (write-lock en `get()`), ERR-042 (`read_header` duplicada), ERR-048/047 (2× hashes, copias inline).
- ✅ Prefetch mmap: micro-bench propio dice **no aporta** (≤2% peor p99) → default `OFF` correcto.

## 9. ¿Qué falta para estar "completo"?

### Bloqueadores pre-publish (orden de ejecución)
1. **ERR-022** — clamp de `top_k` en los 3 bindings (Python/MCP/WASM) + hard cap en core.
2. **ERR-021** — MCP: restaurar streaming con `take(n)` + paginación, no cargar la tabla entera.
3. **UAF PyO3** — fix de `__array_interface__` (bloquea PyPI).
4. **ERR-016** — ya resuelto; confirmar guard y tests en CI.
5. **ERR-035/036** — quitar read-lock global del hot path (snapshot mmap sin guard).
6. `cargo semver-checks` + `cargo deny audit` verdes + `release-plz publish` v0.5.0 **consistente** en crates.io y PyPI.
7. Reparar benchmarks publicados: sello de máquina+commit, `hnsw_pure` HNSW real, `criterion_baseline.json`.
8. Regenerar CHANGELOG (git-cliff), arreglar `llms.txt`, corregir mojibake, documentar límite `u64`.

### Roadmap post-lanzamiento
- WAL async (10-100× ingest; hoy sync por lote).
- PITR + wal-shipping en SDK (hoy huérfano).
- Query planner con optimizaciones reales (hoy router + heurística).
- Algoritmos IVF/SCANN/DiskANN expuestos por SDK.
- Desktop Tauri (consola admin, 8/9 done) + admin web.

## 10. Estado del backlog

- **~113 tareas activas.** P0-P4 cerradas.
- **P15 ERR** (50 hallazgos, revisión multi-agente 2026-08-08: **5 críticos**):
  - `ERR-022` (clamp top_k) — crash/OOM — pendiente
  - `ERR-021` (MCP OOM) — pendiente
  - `ERR-035` (read-lock global) — pendiente
  - `ERR-016` (alias WHERE) — **resuelto**
  - `ERR-010` (raza checkpoint) — **resuelto** (v0.4.0)
- **ERS 004 / AUD-016..AUD-021** pendientes (auditorías: RUSTSEC-2026-0002 etc.).
- **P9** OLD-852 PGWire; **P8** BIZ-01b; **P6** LEG-01 (marca humana); **P5** 3 tareas de docs; **P14** REVIEW 3 god modules.
- **P12 desktop** — ADMIN ✅ (2026-08-08), queda DESKTOP (Tauri, DESKTOP-12..27).

### Conclusión y prioridades

El veredicto combinado: **la infraestructura está madura pero hay bugs críticos reales sin arreglar** — no es un tema de "falta de features". Los 4 bloques a resolver con **precisión de archivo y esfuerzo estimado**:

| # | Blocker | Archivo exacto | Esfuerzo |
|---|---|---|---|
| 1 | Clamp `top_k` (ERR-022) | `src/index/search.rs:522-601` + `vantadb-mcp/src/lib.rs:1301` + Python `lib.rs:858,1246` + WASM `lib.rs:736-738` | ≤ 1 día |
| 2 | MCP streaming/paginación (ERR-021) | `vantadb-mcp/src/lib.rs:333-365, 1401, 1430, 1499` | 1-2 días |
| 3 | UAF `__array_interface__` PyO3 | `vantadb-python/` FFI (pyo3 array interface) | 1-2 días |
| 4 | Quitar read-lock global (ERR-035/036) | `src/physical_plan.rs:289-290`, `src/storage/engine/ops.rs:507,1234` | 1-3 días |

Tras eso: release 0.5.0 coherente + benchmarks verídicos + recomponer docs/marketing. El resto (IVF/SCANN expuesto, WAL async, diskann real, desktop) es iteración post-launch.

---

## Fuentes

- `docs/Backlog.md` — estado de backlog, P0-P15, prioridades.
- `docs/reviews/errors-found.md` — catálogo ERR (ERR-010, ERR-016, ERR-021, ERR-022, ERR-035/036, ERR-050…).
- `docs/CHANGELOG.md` — historia de versiones.
- Código: `src/` (`sdk/`, `parser/mod.rs`, `storage/engine/maintenance.rs`, `index/` (`graph.rs`, `search.rs`, `ivf.rs`, `scann.rs`, `diskann.rs`), `graph.rs`, `gds.rs`, `lsm.rs`, `gc.rs`, `crypto.rs`, `storage/vfile.rs`, `wal.rs`, `wal_sharded.rs`, `wal_archiver.rs`, `wal_shipping.rs`, `integrations.rs`, `cli.rs`, `cli_handlers/`, `vector/quantization.rs`), bindings `vantadb-python/`, `vantadb-mcp/`, `vantadb-node/`, `vantadb-ts/`, `vantadb-wasm/`, `vantadb-server/`.
- Bench: `benchmarks/hnsw_pure.rs`, `benchmarks/graphrag_bench.rs`, `criterion_baseline.json`.
- Registros externos: crates.io (`vantadb`), GitHub `@ness-e/Vantadb`, PyPI (`vantadb-py`), npm (`@vantadb/sdk`).