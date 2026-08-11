---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-08-07
verified_by: "Historial de verificación: docs/progreso/BACKLOG_HISTORY.md"
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks — organized by execution order.
> **Execution state lives in:** `docs/plans/YYYY-MM-DD-<campaign>.md` (plan file) + task files — per campaign-executor RULES.md §2. This file is the task catalog; the plan file is the execution state.
> **Completed tasks moved to:** `docs/progreso/README.md`
> **Verification method:** All items cross-checked against actual codebase (Jul 27, 2026). 8 tareas ejecutadas en sesión: TSK-106, MKT-03, NUEVO-21, MKT-04, TSK-107, DISC-02, DISC-03, Good first issues (18 creadas).
> **Sync 2026-08-06:** 30 tareas ejecutadas por el plan `docs/plans/2026-08-05-backlog-validation-actions.md` tachadas y migradas a `docs/progreso/README.md`: AUDIT-01/03/04, DEBT-01, TECH-01..08, AUDIT-05/08, NUEVO-01, MKT-10/16, AUD-001..011 (AUD-010 fusionada en TECH-04/ADR-012), GH-123/141.
> **Sync 2026-08-07:** 214 filas completadas eliminadas del backlog (210 IDs únicos) — migradas/verificadas en `docs/progreso/README.md` y `docs/progreso/BACKLOG_HISTORY.md`. Quedan 35 tareas activas (ver Exec Summary).
> **Total open items:** ~20 activas — previas (DISC-01..03, LEG-01, BIZ-01b, OLD-01, DESKTOP-15..27, REVIEW-04, [ADMIN-XX pending W4]) + P15 residuales (ERR-006/007/008/009/015/026/031/032/033/036/037/042/043/044/045/047/048/049) + P16 residuales (PERF-07/08/09, CI-01, REVIEW-05). P15/P16 principales ejecutadas por plan `docs/plans/archive/2026-08-09-backlog-pipeline.md` (49/49 ✅). Origen: investigación multi-agente 2026-08-09 → `docs/Investigaciones/investigacion-equipo-2026-08-09.md`
> **Sync 2026-08-09:** plan `docs/plans/archive/2026-08-09-backlog-pipeline.md` archivado — 49/49 tareas delegables completadas (Wave 0-3: RELEASE-01/02/03, SEC-01, 24 ERR, 7 FEAT, REVISAR-01, COV-001/003/004, PERF-01/04/06, DOC-02..08). RELEASE-02 verificado live: 0.5.0 publicado (crates.io/PyPI/npm/GitHub 2026-08-01). Filas completadas eliminadas de P15/P16; residuales siguen activas. Task 50 COM-02/03 (humana) queda en la tabla.
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## Exec Summary

| Phase | Items | Est. Effort | Priority |
|-------|-------|-------------|----------|
| **P0** 🚀 Release Blockers | 0 — ✅ 3/3 ejecutadas (plan 2026-08-09: RELEASE-01 semver-checks, RELEASE-02 publish 0.5.0 verificado live, RELEASE-03 artefactos) | — | ✅ Cerrada |
| **P1** 🛡️ Security & Critical | 0 — ✅ 1/1 ejecutada (SEC-01 UAF `__array_interface__` fix) | — | ✅ Cerrada |
| **P2** ⚡ Quick Wins Técnicos | 0 | — | ✅ Cerrado |
| **P3** 🧪 Test Coverage (core SDKs) | 4 (COV-001..004) | ~1-2 días | 🟢 Alta |
| **P4** 🔧 Engineering Health | 0 | — | ✅ Cerrado |
| **P5** 📖 Docs & Community | 3 (DISC-01..03) | ~1-2 semanas | 🟡 Media |
| **P6** 🚀 Launch Campaign | 1 (LEG-01) | ~1-2 semanas | 🟡 Media |
| **P7** 🌐 WASM & Performance | 0 | — | ✅ Cerrado |
| **P8** 🔮 Post-Launch & Enterprise | 1 (BIZ-01b) | ~3-5 semanas | 🔵 Futuro |
| **P9** 📚 Old Docs Rescue (reference) | 1 (OLD-01) | — | 📖 Referencia |
| **P10** 🏗️ Competitive Features (catalog) | 0 | — | 🗺️ Roadmap |
| **P11** 🐛 GitHub Issues | 0 | — | ✅ Cerrado |
| **P12** 🖥️ DESKTOP App (Tauri) + Consola Admin | 25 (DESKTOP-12..27 + ADMIN-01..09) | ~4-6 semanas | 🔵 Futuro |
| **P13** 🔎 AUDREP — Audit Report 2025 | 0 | — | ✅ Cerrado |
| **P14** 🔍 REVIEW items | 2 (REVIEW-04, REVIEW-05) | 📆 Backlog | 🟡 Media |
| **P15** 🔍 ERR items (revisión multi-agente 2026-08-08) | 18 residuales (ERR-006/007/008/009/015/026/031/032/033/036/037/042/043/044/045/047/048/049) | 📆 Backlog | 🟡 Media (36 ejecutadas por plan 2026-08-09) |
| **P16** 🧩 Completitud de Features (investigación 2026-08-09) | 5 residuales (PERF-07/08/09, CI-01, REVIEW-05) | 📆 Backlog | 🟢 Baja (19 ejecutadas por plan 2026-08-09) |

> **Historial de items removidos/completados:** ver `docs/progreso/BACKLOG_HISTORY.md`.
> **Nuevo 2026-08-04:** Fase 12 DESKTOP (26 tareas, app Tauri multi-connection sobre las 6 integraciones) + `DEBT-01` (gate docs-coverage roto, Fase 4) + `TECH-01..08` (hallazgos de investigación DESKTOP-01b: 2 bugs reales, 1 batch stale-docs, 1 ADR env-naming, 4 features/decisiones, todos en Phase 4).
> **Nuevo 2026-08-07:** Fase 7 ADMIN (ADMIN-01..09, consola administrativa centralizada: datos/métricas/KPIs/SOPs/telemetría/procesos/conexiones) sobre la infra DESKTOP-02..11 ya completada. Fuente de KPIs/SOPs: investigación de mercado (Grafana VectorDB Observability, Milvus, Qdrant, Weaviate, Zilliz/VectorDBBench) + snapshot de métricas ya existente en core (`src/metrics/core/snapshot.rs`, 112 líneas). Tareas DESKTOP-12..27 (drivers/MCP/empaquetado) se ejecutan después del core del dashboard.
> **Nuevo 2026-08-08:** P15 ERR — 50 hallazgos de la revisión multi-agente por capas (6 sub-agentes: vanta-audit/arch/engine/worker/tuner/docs + verificación manual). Origen completo en `docs/reviews/errors-found.md`. Top 3 a atacar primero: ERR-010 (raza checkpoint/persistencia), ERR-021 (OOM MCP), ERR-022 (top_k sin clamp → alloc gigante).
> **Nuevo 2026-08-09:** Fase 3 reabierta con COV-001..004 — medición de coverage multi-agente (3 sub-agentes 2026-08-09): Rust root 81.40% ✅ gate / TS 0% medible (bloqueado import-time por `vite-plugin-wasm`↔`vitest`) / Python wrapper 69% (core PyO3 invisible). Detalle en `docs/reviews/coverage-2026-08-09.md` (pendiente de generar). COV-004 es decisión de política ADR, no código.
> **Nuevo 2026-08-09:** P16 Completitud de Features + P0/P1 reabiertos — investigación multi-agente (5 sub-agentes: audit/worker/docs×2/tuner) que barrió features declaradas → parciales/huérfanas (PITR/WAL-shipping standalone-vacío, DiskANN "in-memory", Arrow 1-comp, IVF/SCANN sin SDK, integrations stubs), releases (semver-checks, publish 0.5.0, artifactos de ejecución), seguridad (UAF `__array_interface__`), rendimiento (benchmarks publicados no confiables) y docs (CHANGELOG muerto / llms.txt inventado / mojibake). Origen completo: `docs/Investigaciones/investigacion-equipo-2026-08-09.md`. **Top 3 a atacar primero:** RELEASE-01 (semver-checks), SEC-01 (UAF), PERF-01 (sellar benchmarks).
> **Nuevo 2026-08-09 (audit-reports archive):** 4 reportes `audit-full-2026-07-*`/`08-04` archivados tras verificación contra código. Resueltos desde entonces: archive.rs fsync (AUDREP-04/35) ✅, deny.toml RUSTSEC-2024-0436 ✅, tests.rs god file dividido ✅, `.playwright-cli` → .gitignore ✅, JSON-LD en web ✅, prefers-reduced-motion ✅, metrics registry test ✅, pyi stubs ✅. Hallazgos pendientes incorporados como tareas nuevas: `PERF-07` (sparse JSON hot path), `PERF-08` (WASM serialización completa), `REVIEW-05` (god files restantes), `CI-01` (pre-commit-config). C1 UAF en `types.rs:365-380` (VantaSearchHit) sigue vigente → cubierto por `SEC-01`.
> **Nuevo 2026-08-09 (batch 2 audit-reports archive):** 4 reportes más archivados (audit-full-20260808, deps-01, inv-001, inv-024). Resueltos/verificados: AUD-012..015 (clippy 5 errores, tests INV-024, prune canonical, cap over-capacity) ✅ commit `9d3c05a2`; INV-024 H-1 (panic sq8 dims) ✅ NV-01 clamp; INV-024 M-1 (alineación `vector_offset`) ✅ `vfile.rs:739` central guard; inv-001 RUSTSEC sin acción ✅; deps-01 duplicación legítima trackeada en `ERR-007` ✅. Pendientes ya en backlog: AUD-016..021 (sección Hallazgos pendientes), `ERR-009` (Miri), `SEC-01`/`AUD-019` (__array_interface__). Único gap → `PERF-09` creado (cold-start "zero-copy" engañoso, `_force_copy` muerto).
> **Nuevo 2026-08-09 (batch 3 audit-reports archive):** `audit-full-2025-07-27.md` (vantadb-audit-report, auditoría multi-agente sobre `develop@63b0101d`) archivado → `docs/audit-reports/archive/`. Reporte íntegramente procesado: fue la fuente de **P13 AUDREP-01..62 + DEPS-01 + NV-01..05** (verificado 2026-08-05, todos resueltos 2026-08-05..08, commits en `docs/avance/historial/snapshot-2026-08-07.md`). 6 hallazgos fueron corregidos antes del ticketeo (CRIT-01, CRIT-06, CRIT-09, ALTO-01, CRIT-10-prometheus, MED-15) y los residuales vivos ya están recapturados como tareas activas (SEC-01, ERR-021, PERF-07/08/09, CI-01, REVIEW-05). **No se crean tareas nuevas — archivo cerrado.**

---

## ✅ Definition of Ready / Done

> **DoR + DoD del proyecto (VantaDB-specific) viven en:** `.opencode/references/definition-of-done.md` — secciones "VantaDB — Definition of Ready" y "VantaDB — Project-specific DoD commands". Referencia única; no duplicar aquí.

---

## Phase 0: 🚀 Release Blockers

> Items que bloquean un release público seguro. Resolver antes de cualquier publicación.
> **Ejecutada 2026-08-09 por plan `docs/plans/archive/2026-08-09-backlog-pipeline.md` — 3/3 ✅.** RELEASE-02 publish 0.5.0 verificado live (crates.io/PyPI/npm/GitHub ya en 0.5.0 desde 2026-08-01). RELEASE-03 artefactos limpiados (gitignored).

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `RELEASE-01` | **Gate `cargo semver-checks` en CI** — *ejecutado* (`1e2a58cc`: job dedicado semver-checks en release/ci). | `.github/workflows/release.yml`, `ci-rust-10.yml` | 🟢 | 🔴 | ✅ Completado |
| `RELEASE-02` | **Publish coordinado 0.5.0** — *verificado live*: crates.io `vantadb 0.5.0`, PyPI `vantadb_py 0.5.0`, npm `vantadb 0.5.0`, GitHub Release + tag `v0.5.0` (2026-08-01). | `release.yml`, `release-wheels-60.yml`, `release-npm-61.yml` | 🟢 | 🔴 | ✅ Completado |
| `RELEASE-03` | **Limpiar artefactos de ejecución** — *ejecutado*: `_audit04_repro_db/`, `benchmarks/_probe_db/`, `chroma_db`, `.pyc`, `data_comp_bench/` limpiados/ignored. | raíz repo, `.gitignore` | 🟢 | 🟡 | ✅ Completado |
| `CI-01` | **`.pre-commit-config.yaml` con prettier + formatters** — causó el certify FAILED de 2026-07-24 (L3 prettier `NbAccordion.tsx:20` sin hook). Registrar prettier (web/), ruff (python), cargo fmt en pre-commit XS effort | `.pre-commit-config.yaml` | 🟢 | 🟢 | 📝 Pendiente |

---

## Phase 1: 🛡️ Security & Critical

> Investigaciones de seguridad y dependencias críticas.
> **Ejecutada 2026-08-09** — SEC-01 resuelto por plan `2026-08-09-backlog-pipeline.md` (Wave 0).

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `SEC-01` | **UAF real en `__array_interface__`** — *resuelto* (`241f30a3`+`9fac19d0`: copia del buffer + test numpy; AUD-019 superseded). | `vantadb-python/src/types.rs:365-380`, test numpy | 🟠 | 🔴 | ✅ Completado |


> **Items previos resueltos (9):** ver `docs/progreso/BACKLOG_HISTORY.md` (P1).

---

> **Phase 2: ⚡ Quick Wins Técnicos** — **31 items removidos:** ver `docs/progreso/BACKLOG_HISTORY.md` (P2). No quedan items activos en P2.
> ⚠️ **Nota DRV-014 (tradeoff WAL batch):** ver ADR `docs/architecture/adr/DRV-014-wal-batch-tradeoff.md` — el fix original fue revertido deliberadamente por `cae92db3` (batch-append por shard, 3-5× speedup). Tradeoff de performance, no deuda pendiente.

---

> **Phase 3: 🧪 Test Coverage (core SDKs)** — reabierta 2026-08-09 con COV-001..004 tras medición multi-agente (Rust 81.40% root ✅ / TS 0% bloqueado / Python 69%). Items previos (14) removidos: ver `docs/progreso/BACKLOG_HISTORY.md` (P3).

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| `COV-001` | **Python: smoke test async de `AsyncVantaDB`** — cubre las 37 líneas faltantes (`flush`, `purge_expired`, `query`, `graph_*`, `put`, `delete`, `export_*`); solo ejercita el path sync hoy. Gate: coverage wrapper ≥85% | `vantadb-python/vantadb_py/__init__.py`, `vantadb-python/tests/` | 🟢 | 🟢 |
| `COV-002` | **TS: destrabar medición de coverage** — resolver incompatibilidad `vite-plugin-wasm@3.6.0` ↔ `vitest@4.1.10` (virtual module `__vite-plugin-wasm-helper` no resuelto en Node, ver vitest-dev/vitest#6723) o reportar con c8 desde `test-runner.mjs`. Runtime de `src/` (vantadb.ts, native.ts, errors.ts, guards.ts) está 0% medible. 25/26 tests ya pasan vía runner alterno | `vantadb-ts/vitest.config.ts`, `src/` | 🟡 | 🟢 |
| `COV-003` | **Rust: tests del binario CLI** — `src/cli_handlers/*` (crud 396, search 383, diagnostics 367, server 271, migrate 238), `src/bin/vanta-cli.rs`, `src/sdk/gds.rs` ≈2,500 ln al 0%. Asserts en subcomandos. Root coverage 81.40% → ~88% | `src/cli_handlers/`, `src/bin/`, `tests/` | 🟡 | 🟢 |
| `COV-004` | **ADR: política del gate de coverage en CI** — ¿root crate (81.40%, hoy pasa) vs `--workspace` (72.76%, incluye bindings python/wasm/server/mcp que el gate 100% by design)? Si se adopta `--workspace`, migrar medición de bindings a sus runners nativos (pytest/browser). Documentar decisión | `.github/workflows/ci-rust-10.yml`, ADR en `docs/architecture/adr/` | 📖 | 🟡 |

---

## Phase 4: 🔧 Engineering Health & Architecture

> Investigaciones de salud de ingeniería — rendimiento, concurrencia, arquitectura.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
| `PERF-01` | **Sellar + resync benchmarks publicados** — las claims del README (ej. "100k docs en 0.6s") son del desarrollo inicial y no se reproducen con el código actual (`cargo bench` con 100k docs/docs mix >60s). Re-validar cifras, actualizar README/QUICKSTART/docs con metodología y HW, o retirar claims no soportadas | `benches/`, `README.md`, `docs/QUICKSTART.md`, `docs/benchmarks/` | 🟡 | 🔴 |
| `PERF-02` | **Baseline riguroso post-publicación**: `criterion` con perfiles fijos + `critcmp` para regresiones en CI (candidate), dataset sintético determinístico guardado junto a benches | `benches/` (candidates), `.github/workflows/heavy-bench-nightly-51.yml` | 🟡 | 🟡 |
| `PERF-03` | **Bench competitivo de SDKs** — dejar de afirmar superioridad sin números: comparar (medir) hnsw_pure vs Qdrant/Chroma/Milvus-frugal en el mismo HW; publicar tabla honesta. Implica mantener `data_comp_bench/`, luego integrar a `docs/benchmarks/` | `data_comp_bench/`, `docs/benchmarks/` | 🟠 | 🟡 |
| `PERF-04` | **Prefetch default OFF** — el auto-indexado prefetch en `engine` oculta la latencia real (`fnv1a` eager en put). Si el feature es real, documentar flag y mantener OFF por defecto (tal cual salió: 0.5.0) | `src/index/hnsw.rs` (prefetch), docs | 🟢 | 🟢 |
| `PERF-05` | **WAL async roadmap** — batch-append por shard ya da 3-5× (ADR DRV-014); roadmap: `io_uring`/`aio` + fsync group commit. No bloquea release | `src/storage/wal.rs`, ADR | 🔴 | 🟡 |
| `PERF-06` | **`VANTADB_MEMORY_LIMIT` env var** — hoy el flag `--memory-limit` se parsea como int sin sufijos (KB/MB/GB); añadir parse humano estilo `500MB`/`1g` | `src/config.rs`, `src/cli.rs` | 🟢 | 🟡 |
| `PERF-07` | **Sparse JSON parseado en cada read/write del hot path** — `memory_record_from_node` (`src/sdk/serialization/mod.rs:271-279`) hace `serde_json::from_str` en cada read y `to_string` en cada write (L335-338) aunque el caller no use sparse; `.ok()` traga errores de parse → degradación silenciosa a `None`. Cachear/streaming del JSON sparse o saltar si `sparse_vector` no está presente | `src/sdk/serialization/mod.rs:271-279, 335-338`, `src/sdk/api.rs:751` (3er consumidor) | 🟢 | 🟡 |
| `PERF-08` | **WASM serialización completa en persist + search hot path** — `serde_wasm_bindgen::to_value` serializa TODOS los records en cada `persist` (`vantadb-wasm/src/lib.rs:750`, H3-SER-001) y en resultados de search (H3-SER-002); datasets >100MB bloquean el event loop por segundos. Plan: persistencia diferencial (delta de cambios) + `Float32Array` zero-copy para vectores de search | `vantadb-wasm/src/lib.rs:439,447,750,997` | 🟠 | 🟡 |
| `PERF-09` | **Cold-start "zero-copy" engañoso** — `deserialize_from_bytes(data, _force_copy)` (INV-024 L-3): el parámetro `_force_copy` está muerto y la ruta cold-start SIEMPRE copia todos los vectores al heap, aunque el log emite "loaded zero-copy index" (`src/index/serialize.rs:613`). Decidir con vanta-tuner: habilitar MmapFull real en cold-start, o corregir el log/comentario | `src/index/serialize.rs:238, 611-613` | 🟠 | 🟡 |

> **Items previos completados (10):** ver `docs/progreso/BACKLOG_HISTORY.md` (P4) — movidos a `docs/progreso/README.md`.

---

## Phase 5: 📖 Docs & Community

> Preparación de documentación pública, comunidad, y onboarding.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|----------|------|-------------|
| `DISC-01` | **Configurar Discord: reaction roles, autorole, logging, welcome DM, onboarding** | `docs/discord/todo.md` + assets SVG + server activo | 🟡 2-3d | 🟢 | ⚠️ Docs + assets OK. Config pendiente |
| `DISC-02` | **Discord: AutoMod, stickers/emojis, forums seed** | — | 🟢 4-6h | 🟢 | ⚠️ Forums seedeado (9 threads: FAQ/Showcase/Ideas/Bug). AutoMod/stickers/emojis requieren Discord UI manual — no API-accessible |
| `DISC-03` | **Discord: ticketing system, stage channel, Server Discovery, Canny.io** | — | 🟢 4-6h | 🟢 | ⏸️ **ICEBOX 2026-08-05** — Server Discovery exige 1000+ miembros; Canny.io SaaS externo; ticketing requiere bot externo (Ticket Tool/Helper.gg). Nada accionable hoy. Dependencias documentadas en `docs/discord/todo.md`. No cuenta como activa. |

---

## Phase 6: 🚀 Launch Campaign

> Todo lo necesario para el Show HN y marketing de lanzamiento.

### 👤 Tareas Humanas (no-delegables a agentes)

> Requieren identidad legal, pago, o acceso manual a dashboards externos. `owner: human`. No ingresan al flujo de agentes. (Sección creada 2026-08-05 — LEG-01 movido aquí.)

| ID | Descripción | Estimación real | Prioridad | Estado Real |
|----|-------------|-----------------|-----------|-------------|
| `LEG-01` | **Registrar trademark "VantaDB" (USPTO + EUIPO)** — Requiere abogado, pago (~$250-350/clase USPTO, ~€850 EUIPO), identidad legal. Estimación original "2-4h" irreal. | semanas, $2-5K | 🔴 | ❌ No iniciado — mover a `docs/strategy/GO_TO_MARKET.md` cuando exista |

| ID | Descripción | Esfuerzo | Prio | Estado Real |
|----|-------------|----------|------|-------------|

---

## Phase 7: 🌐 WASM & Performance

> **Items removidos (5):** ver `docs/progreso/BACKLOG_HISTORY.md` (P7).

---

## Phase 8: 🔮 Post-Launch & Enterprise

> Features para después del lanzamiento público.

| ID | Descripción | Esfuerzo | Prio |
|----|-------------|----------|------|
| `BIZ-01b` | **Enterprise features: encryption + RBAC ya en crate principal. Audit/replication/enterprise crate separado no existen** | 🟡 3-5d | 🟡 ⏳ |


### 🔍 Investigaciones Post-Consolidación

> Items agregados 2026-07-29 tras verificación de 19 hallazgos de 4 sub-agentes contra código actual. **Sin implementación — solo investigación + propuesta.**

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|

### 🌐 Web Frontend — Auditorías

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|

### ⚙️ CI & Tooling — Auditorías

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|

### 🚀 Implementaciones derivadas de Investigaciones (agregadas 2026-08-04)

> Items creados tras la verificación de 13 INV contra código real (sub-agentes, 2026-08-04). Cada fila referencia su doc de investigación.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### 🚨 Hallazgos de Auditoría Doc↔Código (agregados 2026-08-04)

> Items creados tras la auditoría multi-agente completa (7 sub-agentes + verificación dirigida del padre) que comparó docs vs código real. Cada tarea sigue el proceso **Investigación → Análisis → Implementación**. El backlog indica qué cada fase debe entregar.

| ID | Descripción (Investigación → Análisis → Implementación) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

> **Items removidos (1):** ver `docs/progreso/BACKLOG_HISTORY.md` (P8).

---

## Phase 9: 📚 Old Docs Rescue — Reference Catalog

> Recuperado de `VANTADB DOC OLD` (~280 archivos .md analizados vía 21 sub-agentes).
> **Total:** 21 items, **13 activos** (8 ✅ removidos a progreso). **Estado:** 1 ⚠️ parcial, 1 ❌ pendiente, 2 ❌ justificado.
> **Referencia completa:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7.
> **Items removidos a progreso (8):** ver `docs/progreso/BACKLOG_HISTORY.md` (P9).

### 🔴 Alta — Features perdidas con alto valor de mercado

> **8 items ✅ removidos a progreso:** ver `docs/progreso/BACKLOG_HISTORY.md` (P9).

| ID | Feature | Esfuerzo | Estado | Dependencias | Prioridad |
|----|---------|----------|--------|--------------|-----------|
| `OLD-01` | **PGWire (PostgreSQL wire protocol)** — Compatibilidad con psql, pgAdmin, ecosistema PG | 🟠 2-3 sem | ❌ No implementado | Ninguna | 🗺️ Roadmap |

---

## Phase 10: 🏗️ Competitive Features — Catalog

> **Fuente:** Análisis de 27 archivos de `VANTADB DOC OLD/` (9 vector DBs + 8 graph DBs + 10 arquitectura).
> **Total:** 30 items, **18 activos.** 12 ✅ implementados removidos a progreso: ver `docs/progreso/BACKLOG_HISTORY.md` (P10).
> **Reportes completos:** `docs/audit-reports/competitive-features-consolidated-report.md`, `docs/audit-reports/deep-analysis-{vector,graph,arch}.md`

### 🔴 Alta — Features competitivas críticas para adopción

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|

### 🟠 Media-Alta — Features competitivas importantes

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|

### 🟡 Medio — Features de madurez y ecosistema

| ID | Feature | Esfuerzo | Estado | Dependencias |
|----|---------|----------|--------|--------------|

---

## Phase 11: 🐙 GitHub Issues — Backlog de Issues Abiertos

> Issues abiertos en `ness-e/Vantadb` convertidos a tareas del backlog (2026-08-01).
> **Total:** 14 tareas (issues #119–#144). Todos son `good first issue`.
> ⚠️ **Recordatorio obligatorio:** al completar cada tarea, **revisar el resultado contra el issue y cerrarlo en GitHub** (`gh issue close <NUM> --repo ness-e/Vantadb`), confirmando que se cumple su Definition of Done.
> Estado real se actualiza en la columna **Estado**.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

---

## Phase 12: 🖥️ DESKTOP — App de Escritorio Tauri Multi-Connection

> **Objetivo:** App de escritorio Tauri v2 en `desktop/` que conecta la UI con VantaDB a través de **cualquiera de las 6 integraciones** (crate nativa, `vantadb-server` HTTP, `vantadb-mcp` stdio, `vantadb-node` napi, `vantadb-python` PyO3, `vantadb-ts`/`vantadb-wasm` webview), individualmente o varias simultáneas, para máxima compatibilidad/rendimiento/seguridad.
> **Base de decisión:** `docs/Investigaciones/DESKTOP-01-tauri-plataforma-desktop.md` (✅ SÍ — Tauri v2, vía nativa óptima) + **`docs/Investigaciones/DESKTOP-01b-investigacion-6-integraciones-arquitectura.md`** (investigación completa de las 6 integraciones + arquitectura multi-connection de vanta-arch: trait `VantaConnection` + `ConnectionManager`, default = crate `vantadb` embebida, regla "un escritor por path de DB").
> **Contexto de integraciones (investigado 2026-08-04):** server = HTTP REST `/api/v2/query` (IQL) en `127.0.0.1:8080`, auth Bearer, sin streaming; MCP = JSON-RPC 2.0 **solo stdio** (15 tools), proceso es la DB; node = addon napi-rs (Tauri no puede `require()`, solo sidecar); python = PyO3, sin CLI (requiere driver script); ts/wasm = snapshot JSON in-memory, read-only/demo.
> **Regla de fragmentación:** 1 tarea = 1 concepto; nunca mezclar 2 integraciones en una tarea. `desktop/` usa `[workspace]` vacío en `src-tauri/Cargo.toml` → desacoplado del workspace raíz (no toca CI/versiones de core).
> **Scoping 2026-08-05 (plan Task 54 — MVP multi-connection recortado):** incluir 13 tareas (DESKTOP-02/03/04/05/08-14/19/20/24/26), defer 4 a Fase 7 futura (DESKTOP-15/16/17/18 Node/Python — valor marginal, empaquetado frágil). `desktop/` aún NO existe — **build pendiente desde cero**, solo scoping realizado.

### Fase 0 — Scaffold

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### Fase 1 — Trait + adaptador nativo + UI mínima

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### Fase 2 — Adaptador Server (HTTP)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### Fase 3 — Adaptador MCP (stdio)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-12` | **Cliente rmcp** — Dep `rmcp` (`client`, `transport-child-process`); `TokioChildProcess` + init handshake, `list_tools`, `call_tool` con params serde_json. **DoD:** conecta al binario real; `list_tools` devuelve las 15 tools (memory_put/get/delete/list, search_memory, query_lisp, collection_*...). | `src/connections/mcp_client.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-13` | **`McpConnection`** — Mapea las 15 tools al trait (`memory_put`→ingest, `memory_get`→get, `memory_delete`→delete, `memory_list`/`collection_list`→list, `search_semantic`/`search_memory`→search, `query_lisp`+`collection_*` fuera del trait MVP); `VantaError::Mcp`. **DoD:** integración real: ingest + search vía MCP en temp dir; tool inexistente → error mapeado. | `src/connections/mcp.rs` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-14` | **Healthcheck/reconnect/UI MCP** — Healthcheck por tool trivial; `close` = `graceful_shutdown` con kill-timeout; selector "MCP" en UI. **DoD:** desconectar mata el proceso (verificado); reconectar funciona. | `src/connections/mcp.rs`, `ConnectionSelector.tsx` | 🟢 | 🔵 | ❌ Desde cero |

### Fase 4 — Node y Python (opcionales, feature-gate)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-15` | **JSON-RPC cliente + driver Node** — Framing newline-delimited, ids incrementales, mapa de respuestas pendientes, timeout (`connections/jsonrpc.rs`, compartido con Python); driver `drivers/node/driver.js` que `require('vantadb_native')` y sirve stdio. **DoD:** `node driver.js` responde put/search por stdio (test manual con pipe). | `src/connections/jsonrpc.rs`, `drivers/node/driver.js` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-16` | **`NodeConnection`** — Spawn node (dev: sistema; release: sidecar `externalBin`), IPC vía jsonrpc, mapeo `VantaError::Node`, capabilities. **DoD:** integración: put/search vía node en temp dir. | `src/connections/node.rs`, `tauri.conf.json` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-17` | **Driver Python + decisión runtime** — `drivers/python/python_driver.py` (import `vantadb_py`, JSON-RPC stdio); **decisión documentada aquí**: runtime python del sistema vs bundled (MVP = sistema primero). **DoD:** driver responde put/search con python local (si hay runtime; si no, skip documentado). | `drivers/python/python_driver.py` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-18` | **`PythonConnection`** — Spawn python + driver, reusa `jsonrpc.rs`, mapeo `VantaError::Python`. **DoD:** integración opcional (skippable sin runtime python). | `src/connections/python.rs` | 🟡 | 🔵 | ❌ Desde cero |

### Fase 5 — ConnectionManager multi-connection

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-19` | **ConnectionManager completo** — Registry multi (`HashMap<id, Arc<dyn VantaConnection>>`), active, `path_holders` (regla 1-escritor por path → `VantaError::Lock` + hint), routing por `connection` id, capability gate (write sobre read-only → `Unsupported`). **DoD:** Nativa + Server (paths distintos) conectadas simultáneamente; conectar segunda vía sobre el mismo path → rechazada con hint. | `src/connections/manager.rs` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-20` | ~~**Lifecycle shutdown_all** — `shutdown_all` en `RunEvent::ExitRequested`: orden webview → subprocesos → nativa última (flush); timeout configurable + kill forzoso.~~ ✅ 2026-08-08 — commit `45f8bed8` (`shutdown_all(grace)` + `RunEvent::ExitRequested`, manager.rs + lib.rs, test) | `src/lib.rs`, `src/connections/manager.rs` | 🟢 | 🔵 | ✅ |
| `DESKTOP-21` | **UI multi-connection** — Selector con N vías conectadas, switch de activa, health badge por vía, warning de conflicto de path. **DoD:** UI muestra 2 vías vivas; la op va a la activa; warning al intentar conflicto. | `ConnectionSelector.tsx`, `ConnectionPanel.tsx` | 🟡 | 🔵 | ❌ Desde cero |

### Fase 6 — Streaming, config, empaquetado, CI, tests, docs

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `DESKTOP-22` | **Eventos Tauri (streaming)** — `vanta://connection-state` (obligatorio) + `ingress-progress`/`search-progress` (flag `progress`); listeners en frontend. **DoD:** ingest batch de 1000 items emite progreso sin bloquear la UI. | `src/lib.rs`, `desktop/src/hooks/*` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-23` | **Persistencia de config** — JSON en `app_config_dir`, load en setup, save atómico (`temp + rename`), defaults, vías guardadas. **DoD:** reiniciar la app conserva vías guardadas y vía activa. | `src/config.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-24` | **Empaquetado** — Bundle NSIS/MSI (Windows primero), `externalBin` (node + `vantadb-server.exe` + runtime python si procede), identifier, icons, auto-update opcional. **DoD:** instalador produce una app que conecta nativo + server + node sin entorno de dev. | `tauri.conf.json`, `src-tauri/build.rs` | 🔴 | 🔵 | ❌ Desde cero |
| `DESKTOP-25` | **CI GitHub Actions** — Build Windows (tauri-action), `cargo test` en `src-tauri` (workspace desacoplado), `npm build` frontend, artefacto instalador; matrix por features de vías (con/sin server/mcp). **DoD:** pipeline verde; artefacto instalador subido. | `.github/workflows/desktop.yml` | 🟡 | 🔵 | ❌ Desde cero |
| `DESKTOP-26` | **Tests** — Unit: tipos, mapping de errores, framing jsonrpc; integración por adaptador (mock HTTP, MCP real, nativa temp); **contrato de errores**: misma op en N vías → mismo shape `VantaError`. **DoD:** `cargo test` + integraciones en CI. | `src/**/*_tests.rs` | 🟢 | 🔵 | ❌ Desde cero |
| `DESKTOP-27` | **Docs + ADR** — README desktop, `ARCHITECTURE.md` (modelo conexión), **ADR** multi-connection + regla 1-escritor (siguiente número libre en `docs/architecture/adr/`), guía de usuario por vía, actualizar DESKTOP-01 con decisiones. **DoD:** ADR revisado por vanta-arch; guía cubre las 6 vías. | `docs/desktop/*`, `docs/architecture/adr/ADR-0XX.md` | 🟢 | 🔵 | ❌ Desde cero |

### Fase 7 — Consola Administrativa (ADMIN)

> **Contexto 2026-08-07:** el usuario dirige el desktop hacia una **consola administrativa de VantaDB** (dashboard con métricas/KPIs/SOPs/telemetría/procesos/conexiones y explorador de datos), no solo a un MVP multi-connection. Fuente de datos: snapshot de métricas ya existente en core (`src/metrics/core/snapshot.rs` — `OperationalMetricsSnapshot` con ~72 campos: startup_ms, WAL replay, ANN rebuild, text index repairs, hybrid/planner, index routing, evictions, quantized nodes, memoria) + command `vanta_health` ya implementado (DESKTOP-03). Estilo: **modo claro reutilizando el design system de `web/`** (cream `#FBF9F5`, ink, neon `#FF5A45`), NO el tema oscuro propietario actual de `App.css`.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ADMIN-01` | ~~**Command `vanta_metrics` IPC** — Exponer `OperationalMetricsSnapshot` como comando Tauri.~~ ✅ 2026-08-08 — commit `d77559f3` (`vanta_metrics` en metrics.rs usa `VantaOperationalMetrics` ya `Serialize`; 37 campos incl. `derived_prefix_scans`) | `src/commands/metrics.rs`, `src/lib.rs` | 🟢 | 🔵 | ✅ |
| `ADMIN-02` | ~~**Métricas vivas (delta entre snapshots)** — Frontend calcula deltas comparando snapshots consecutivos.~~ ✅ 2026-08-08 — convergido en ADMIN-04 (`b62fff7c`): grid de ADMIN-04 incluye deltas imports/queries/scans + RSS + poll 4s (contrato cubierto; código propio eliminado por duplicación) | `desktop/src/hooks/useMetrics.ts`, `src-tauri/src/commands/metrics.rs` | 🟡 | 🔵 | ✅ |
| `ADMIN-03` | ~~**Migrar UI al design system web (modo claro)** — Reemplazar tema oscuro de `App.css` por tokens de `web/globals.css`.~~ ✅ 2026-08-08 — commit `847ab080` (App.css reescrito con tokens cream/ink/neon, sombra dura, radius 0; `ConnectionSelector.tsx` eliminado) | `desktop/src/App.tsx`, `desktop/src/App.css`, `desktop/tailwind.config.js`, `desktop/src/components/*` | 🟡 | 🔵 | ✅ |
| `ADMIN-04` | ~~**Dashboard grid (metro-style) con poll 3-5s** — Layout de cards con polling en cadena.~~ ✅ 2026-08-08 — commit `b62fff7c` (`MetricsGrid.tsx` metro 6 tiles, poll 4s, deltas + trend, responsive 3→1 col) | `desktop/src/pages/Dashboard.tsx`, `desktop/src/components/*` | 🔴 | 🔵 | ✅ |
| `ADMIN-05` | ~~**KPIs derivados** — a partir de snapshot.~~ ✅ 2026-08-08 — commit `4dcf268e` (`KpiCards.tsx` 5 KPIs con guard div-by-zero + sparklines CSS puro; bridge `vanta.ts` con interfaz `OperationalMetrics` única) | `desktop/src/components/KpiCard.tsx`, `desktop/src/utils/kpi.ts` (derivados) | 🟡 | 🔵 | ✅ |
| `ADMIN-06` | ~~**SOP panels (WAL replay / Reindex / Health) con semáforo** — Flujo con estado.~~ ✅ 2026-08-08 — commit `f20d67a4` (`SopPanel.tsx` 3 paneles: WAL Replay/Reindex muestran último valor del snapshot + Refresh, Health llama `vanta_health` en vivo; triggers de replay/rebuild no existen en core — documentado) | `desktop/src/components/SopPanel.tsx`, `desktop/src/hooks/useSop.ts` | 🟡 | 🔵 | ✅ |
| `ADMIN-07` | ~~**Data Explorer** — Tabla navegable de `memory` con paginación.~~ ✅ 2026-08-08 — commit `7a19a9f5` (`DataExplorer.tsx`: browse `vanta_list` + search `vanta_search` con score, "Load more" con limit creciente 50→100→200; core NO soporta offset/cursor — verificado, `ponytail:` documentado) | `desktop/src/pages/Explorer.tsx`, `src/commands/data.rs` | 🟡 | 🔵 | ✅ |
| `ADMIN-08` | ~~**Panel Procesos & Conexiones** — list_connections + panel de procesos.~~ ✅ 2026-08-08 — commit `f5c69788` (`ProcessPanel.tsx`: conexiones con shutdown por entrada; subprocesos placeholder — core sin `McpSpawnRegistry`, `McpSpawn` nunca instanciado, documentado) | `desktop/src/components/ProcessesPanel.tsx`, `src/commands/process.rs` | 🟡 | 🔵 | ✅ |
| `ADMIN-09` | ~~**Snapshot export + persistencia** — export JSON + history corto.~~ ✅ 2026-08-08 — commit `e0e8ff3a` (`ExportPanel.tsx`: blob download JSON con timestamp ISO + persistencia de último snapshot en localStorage) | `src-tauri/src/commands/metrics.rs`, `desktop/src/hooks/useMetrics.ts` | 🟢 | 🔵 | ✅ |

> **Secuencia:** ADMIN-01 → ADMIN-03 (base) → ADMIN-02/04/05 en paralelo → ADMIN-06 → ADMIN-07 → ADMIN-08 → ADMIN-09. Reutiliza `operational_metrics_snapshot()` — NO crear telemetría nueva de cero; el snapshot core es la fuente única de KPIs.

> **Total:** 35 tareas (DESKTOP-02..27 + ADMIN-01..09) — 15 🟢, 16 🟡, 2 🔴, 2 condicionales (Node/Python feature-gate). Secuencia: Fase 0 → 1 → (2/3/4 en paralelo) → 5 → 6 → 7.

---

## Phase 13: 🔎 AUDREP — Hallazgos Audit Report 2025-07-27 (verificados vigentes)

> **Origen:** `docs/audit-reports/archive/audit-full-2025-07-27.md` (auditoría estática multi-agente sobre `develop@63b0101d`).
> **Verificación:** 2026-08-05 contra HEAD `8fe574b5` — los 62 hallazgos siguen presentes en código (líneas actualizadas). 6 fueron corregidos y NO se incluyen: CRIT-01 (recover round-robin), CRIT-06 (flush_threshold=1), CRIT-09 (providers `[workspace]`), ALTO-01 (double antes de parse_i64), CRIT-10-prometheus (feature añadida), MED-15 (batch por shard). Además AUDREP-02/05/06 corregidos 2026-08-05 (tachados) y AUDREP-01/04 ya completados.
> **Ejecución:** priorizar por severidad. CRÍTICOS/ALTOS con categoría **Durabilidad** y **Panic** son los de mayor riesgo de producción.

### CRÍTICOS (8)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### ALTOS (14)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### MEDIOS (25)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### BAJOS (15)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### NV — Hallazgos nuevos verificados en código (2026-08-05, limpieza docs/audit-reports)

> **Origen:** revisión de `docs/audit-reports/` + `docs/reviews/` 2026-08-05 (vanta-lead con verificación manual en código). Candidatos que resultaron **ya tickereados** NO se duplican: `rayon` feature ausente = `AUDREP-07`, `next.config ignoreBuildErrors` = `AUDREP-19`.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| ~~`NV-02`~~ | ~~**🟡 MEDIO / Server-Robustez: `expect`/`unwrap` en `cli_server.rs`** — Líneas 142 y 176 usan `expect`/`unwrap` en paths de request HTTP; un fallo inesperado aborta el hilo del servidor en vez de devolver 500. **Recomendación:** propagar error con `?` → `IntoResponse`.~~ → ✅ 2026-08-08 (handlers ya robustos vía AUDREP-32; único expect restante en GovernorConfig build → degrada a sin rate-limit con log) | `src/cli_server.rs:142, 176` | 🟢 1h | 🟡 | ✅ Completado |
| ~~`NV-03`~~ | ~~**🟡 MEDIO / Packaging-Licencia: `vantadb-wasm` sin archivo LICENSE** — El crate publicable no incluye archivo `LICENSE` (el workspace es Apache-2.0); crates.io lo marca sin license → falla publicación/CI deny. **Recomendación:** añadir `LICENSE` (Apache-2.0) en `vantadb-wasm/`.~~ → ✅ 2026-08-08 (LICENSE Apache-2.0 copiado del raíz, hash idéntico) | `vantadb-wasm/` (Cargo.toml) | 🟢 5min | 🟡 | ✅ Completado |
| ~~`NV-05`~~ | ~~**🟢 BAJO / Config-Dependencias: divergencia `deny.toml` vs `.cargo/audit.toml`** — `deny.toml` no ignora `RUSTSEC-2024-0436` (paste) pero `.cargo/audit.toml:21` sí; herramientas reportan estados distintos del mismo advisory. **Recomendación:** unificar la política de ignorados en un solo lugar.~~ → ✅ 2026-08-08 (ignore RUSTSEC-2024-0436 añadido a deny.toml, `cargo deny check advisories` pasa) | `.cargo/audit.toml:21`, `deny.toml` | 🟢 30min | 🟢 | ✅ Completado |

> **Total:** 63 tareas (8 🔴 CRÍTICOS, 14 🟠 ALTOS, 26 🟡 MEDIOS, 15 🟢 BAJOS). Origen: `docs/audit-reports/archive/audit-full-2025-07-27.md` — 6 hallazgos ya corregidos excluidos (ver nota superior) + `DEPS-01` de la sección 7 del mismo reporte. **Todas resueltas 2026-08-05..08** (commits en `docs/avance/historial/snapshot-2026-08-07.md`); residuales re-capturados en P15/P16 (ERR-021, SEC-01, PERF-07/08/09, CI-01, REVIEW-05).

---

## P14 — REVIEW items (hallazgos de `docs/reviews/review-full-2026-07-27-0309.md`, validado 2026-08-05)

> **Origen:** findings de la unified-review full 2026-07-27, re-validados contra el código el 2026-08-05 (sub-agentes vanta-worker/vanta-docs). Los items marcados como corregidos en el report y confirmados se excluyen. `DEPS-01` ya cubre la duplicación de crates/lru; `AUDREP-41` cubre next-auth dead dep — no duplicados aquí.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `REVIEW-04` | **MEDIO / Refactor: 3 god modules** — `src/node.rs` (1554→1882L, creció), `src/config.rs` (1313L), `src/storage/vfile.rs` (1165L). **Recomendación:** partir en submódulos (ej. separar UnifiedNode de FieldValue; config per-feature). | `src/node.rs`, `src/config.rs`, `src/storage/vfile.rs` | 1-2 semanas | 📆 Backlog | 📝 Pendiente |
| `REVIEW-05` | **God files restantes (>1300L sin split)** — verificación 2026-08-09 (audit-reports 07-24/08-04): `src/index/serialize.rs` 1452L (serialization hot path), `src/index/distance.rs` 1591L, `src/physical_plan.rs` 1380L. tests.rs 4076L ya fue dividido en `src/storage/engine/tests/` ✅. Misma recomendación que REVIEW-04: submódulos por concern | `src/index/serialize.rs`, `src/index/distance.rs`, `src/physical_plan.rs` | 1 semana | 📆 Backlog | 📝 Pendiente |

---

## P15 — Hallazgos de Revisión Multi-Agente 2026-08-08 (errors-found.md)

> **Origen:** `docs/reviews/errors-found.md` — revisión por capas con 6 sub-agentes en paralelo (vanta-audit, vanta-arch, vanta-engine, vanta-worker, vanta-tuner, vanta-docs) + verificación manual del lead sobre `develop@7a19a9f5`. 51 hallazgos documentados; ERR-017 descartado al verificar (métrica euclidiana uniforme en `distance.rs:495/516/536`). Índices de esfuerzo: 🟢 < 1 día | 🟠 1-3 días | 🔴 > 3 días.

### CRÍTICOS (5) — ✅ todos ejecutados por plan `2026-08-09-backlog-pipeline.md` (ver progreso)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-010` | **🔴 Persistencia / Race checkpoint↔snapshot** — fix original `a5ca4389` (insert_lock across checkpoint/save); **RE-ABIERTO 2026-08-11**: 13 tests fallan con `TimeoutError: acquire insert_lock in flush (ERR-010)` (5s) — self-deadlock insert_lock en rebuild/reindex/compact (verificado en corrida aislada, no es contención). Deuda ya documentada en `docs/plans/archive/2026-08-09-backlog-pipeline.md:286`. | `src/storage/engine/` | 🔴 | 🔴 | 🔁 Reabierto |
| `ERR-021` | **🔴 MCP OOM** — *resuelto* (`b01e9ed6`: streaming restaurado con take(n) + límite). | `vantadb-mcp/src/lib.rs` | 🟠 | 🔴 | ✅ Completado |
| `ERR-022` | **🔴 top_k/k sin tope → alloc gigante** — *resuelto* (`3eeb86e1`: k.min(MAX_K) en bindings+MCP). | bindings + `src/index/search.rs` | 🟢 | 🔴 | ✅ Completado |
| `ERR-035` | **🔴 Read-lock global HNSW** — *resuelto* (`1b0016d5`: contención reader/writer mitigada). | `src/physical_plan.rs:211`, `src/storage/engine/ops.rs` | 🔴 | 🔴 | ✅ Completado |

### ALTOS (16 verificados) — ✅ 14 ejecutados por plan `2026-08-09` (ERR-001..025; ver progreso). Residuales:

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-037` | **🟠 `batch_insert` chequea existencia por nodo** — 10k batch = 10k read-paths completos + write-lock cache + clone de vector descartado. | `src/storage/engine/ops.rs:853-925` | 🟠 | 🟠 | 📝 Pendiente |

### MEDIOS (12) — ✅ 7 ejecutados por plan `2026-08-09` (ERR-005/014/027/028/029/030/050; ver progreso). Residuales:

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-026` | **🟡 parse_metadata descarta filtros no-escalables** — arrays/objetos/null ignorados → filtro silenciosamente no aplicado → resultados súper-conjunto. | `vantadb-mcp/src/lib.rs` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-042` | **🟡 `read_header` 2× por candidato** en hot loop (+ entry points) — trabajo duplicado constante. | `src/index/search.rs:275-280, 347-353` | 🟠 | 🟡 | 📝 Pendiente |
| `ERR-043` | **🟡 `shrink_neighbors` clona vector** del nodo solo para usarlo como query. | `src/index/graph.rs:920-926` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-044` | **🟡 `TextAnalyzer` reconstruido por llamada** — batch N paga N setups (stemmer/stopwords). | `src/tokenizer.rs:44-106` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-045` | **🟡 `get_neighbors` clona la lista por nodo** — O(N×M) allocs durante compactación BFS. | `src/index/neighbor_index.rs:66-68` | 🟢 | 🟡 | 📝 Pendiente |

### BAJOS (9) — ✅ 3 resueltos/verificados por plan `2026-08-09` (ERR-016 SKIP verificado, ERR-034 ⏫, ERR-051 ⛠; ver progreso). Residuales:

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-015` | **🔵 kill() siempre en `request_shutdown`** — sin señal graciosa SIGINT; metadata loss en Windows. | `desktop/src-tauri/src/connections/child_process.rs:170-189` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-031` | **🔵 `VecIndex::add` traga rechazos** (solo warn, sin Result) — futuros Arc<dyn> perderían inserts. | `src/index/search.rs:664-698` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-032` | **🔵 Test de `deserialize_node_payload` removido** — pérdida de cobertura del guard MAX_PERSISTED_NODE_BYTES. | `src/storage/ops.rs` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-033` | **🔵 `memory_list(limit=0)` → devuelve 1** — `max(1)` en core vs 0 pedido. | `vantadb-mcp/src/lib.rs:1139-1142` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-047` | **🔵 Copy inline en cada pop del hot loop** (`take_l + extend`). | `src/index/search.rs:225-238` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-048` | **🔵 2 hash lookups en `visited`** — `contains + insert` en vez de `insert` devuelve bool. | `src/index/search.rs:268-269` | 🟢 | 🔵 | 📝 Pendiente |

### INFO (5)

| COM | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-006` | `deny.toml` ignore RUSTSEC-2024-0436 stale ("advisory-not-detected") — limpiar o trackear. | `deny.toml` | 🟢 | ⚪ | 📝 Pendiente |
| `ERR-007` | `multiple-versions` warn activo — hashbrown ×3, rand, syn, thiserror, windows-sys. | Cargo.lock | 🟠 | ⚪ | 📝 Pendiente |
| `ERR-008` | `copy_unsafe` en vfile sin guard explícito de bounds (solo debug assert). | `src/storage/vfile.rs` | 🟢 | ⚪ | 📝 Pendiente |
| `ERR-009` | Correr `cargo miri test` (tree-borrows) sobre vfile/ops antes del próximo merge. | CI / tooling | 🟢 | ⚪ | 📝 Pendiente |
| `ERR-049` | Sin bench dedicado a `ivf.rs` ni `batch_insert` — hallazgos ERR-037/39-41 sin cuantificar. | `benches/` | 🟠 | ⚪ | 📝 Pendiente |

> **Descartado:** ERR-017 (métrica euclidiana consistente en `flat.rs:43` / `distance.rs:495/516/536` / `search.rs:176/327` — no se confirma la divergencia flat vs HNSW).

---

## Phase 16: 🧩 Completitud de Features & Docs (de investigación multi-agente 2026-08-09)

> Origen completo: `docs/Investigaciones/investigacion-equipo-2026-08-09.md`. **Ejecutada 2026-08-09 por plan `docs/plans/archive/2026-08-09-backlog-pipeline.md` — 19/19 tareas ✅** (FEAT-01..07, REVISAR-01, COV-001/003/004, PERF-01/04/06, DOC-02..08; RELEASE-01/02/03 + SEC-01 en Wave 0). Commit por feature en `docs/progreso/README.md`. Residuales viven en sus secciones: `PERF-07/08/09` (sección PERF), `CI-01` (hallazgos pendientes/CI), `REVIEW-05` (P14).

---

## Referencias Cruzadas

- **RC items:** `docs/reviews/PROJECT_FULL_REVIEW_2026-07-13.md` (generado por `vantadb-full-review` skill)
- **REV items:** `docs/reviews/2026-07-13-full-review.md`
- **DRV findings:** `docs/plans/2026-07-15-cross-ref-docs-vs-code.md` + `docs/audit-reports/cross-ref-wave3-final-report.md`
- **OLD items:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 — ~280 archivos VANTADB DOC OLD analizados
- **COMP items:** `docs/audit-reports/competitive-features-consolidated-report.md` + `docs/audit-reports/deep-analysis-{vector,graph,arch}.md` — 27 archivos, 172 features, top 30 priorizados

---



## Hallazgos pendientes de reportes

Hallazgos >= medium derivados de reportes de auditoría. Fuente: `docs/audit-reports/audit-full-20260808-002617.md` (2026-08-08).

| ID | Severidad | Hallazgo | Archivo:línea | Estado |
|----|-----------|----------|---------------|--------|
| AUD-016 | Media | RUSTSEC-2026-0002 (lru 0.12.5 unsound via ratatui) no mechanizado en deny.toml ignore (allow roto en práctica) | deny.toml | 🟡 pendiente |
| AUD-017 | Media | `remove_node` remueve inbound sin limpiar refs cruzadas -> desync INV-024 (dead code, pero contrato roto si se cablea) | src/index/neighbor_index.rs:167-176 | 🟡 pendiente |
| AUD-018 | Media | CI clippy excluye mcp/wasm/server (ci-rust-10.yml:86) -> 5 errores latentes pasan CI; extend gate o documentar deuda | .github/workflows/ci-rust-10.yml | 🟡 pendiente |
| AUD-019 | Media | `__array_interface__` expone puntero raw a Python sin // SAFETY: ni lifetime doc | vantadb-python/src/types.rs:365-380 | 🟡 pendiente |
| AUD-021 | Media | Rate limiter fall-open: si GovernorConfigBuilder::finish() falla, endpoint sirve sin límite | src/cli_server.rs:160-164 | 🟡 pendiente |

---

## P17 - Mejoras del task-system (de REPORTE-FINAL 2026-08-10 §3.3/§3.5)

> Items §3.3 (FALTA, cobertura 0-30%) y §3.5 (MAL/CONTRADICTORIO) de `docs/Investigaciones/2026-08-10-agent-engineering/REPORTE-FINAL.md` no asignados a los planes P0-P3 (Task 14 de `docs/plans/archive/2026-08-10-docs-task-system-consolidation.md`). **Todos los items TSYS-01..16 implementados 2026-08-11** (plan `docs/plans/archive/2026-08-11-residuo-consolidado.md`); los de runtime con datos dependen de Task 2 (poblar `verify-log.jsonl`).

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `TSYS-01` | **Observabilidad de decisión** - log estructurado de qué herramienta usó el agente y por qué cambió de estado (gap-01 §3.3-17). Runtime: instrumentar `campaign-server.mjs`. | `.opencode/task-system/` (`campaign-server.mjs`, `state-tools.mjs`) | 🟡 | 🟡 | ✅ Implementado (TSYS-09) — decision_reason/pattern + evento plan.adjust en campaign-server.mjs |
| `TSYS-02` | **Handoff con invariantes** - recitation debe exigir "invariantes + comandos de verificación + deuda", no solo lastAction/nextAction (gap-01 §3.3-18). | `.opencode/task-system/prompts/task.md`, `pipeline-full.md` | 🟢 | 🟢 | ✅ Implementado (T12) — invariantes+deuda en task.md/pipeline-full.md (commit 8f774c18) |
| `TSYS-03` | **ADR gate mecánico** - gate que falle si se toca API pública sin ADR en `docs/architecture/adr/` (gap-01 §3.3-20; Regla 5 hoy depende de memoria del agente). | `.github/workflows/`, `docs/architecture/adr/` | 🟡 | 🟡 | ✅ Implementado (T13) — job `adr-gate` en ci-rust-10.yml:120-181 (commit d9f2a4cb) |
| `TSYS-04` | **Estimar con appetite (Shape Up)** - "tiempo que VAMOS a invertir" como default en vez de effort vago (gap-01 §3.3-21). | `.opencode/task-system/prompts/plan.md` | 🟢 | 🟢 | ✅ Implementado (T14) — Appetite Shape Up en plan.md (commit 8f774c18) |
| `TSYS-05` | **SLA del pipeline** - SLI/SLO/error budget; hoy no se sabe si el pipeline falla mucho (gap-01 §3.3-23). Requiere datos de `verify-log.jsonl`. | `evals/`, `docs/reports/` | 🟡 | 🟡 | ✅ Implementado (ADR-017) |
| `TSYS-06` | **Chaos/resilience del task-system** - `vanta-chaos` fuzzza el código fuente, no a `campaign-server.mjs` ni a la máquina de estados (gap-01 §3.3-24). | `vanta-chaos`, `.opencode/task-system/config/state-tools.mjs` | 🔴 | 🟢 | 🟡 Diseño implementado (T19) — `task-system-chaos-resilience.md`; runner DEFER (sin verificación) |
| `TSYS-07` | **Recitation duplicado (3 definiciones)** - `pipeline-full.md` (RESULTADO), `task.md` (datos) y parámetro de `campaign_update_task_state`; unificar a 1 fuente (gap-01 §3.5-2). | `.opencode/task-system/prompts/` | 🟢 | 🟢 | ✅ Implementado (T15) — recitation unificado estructura §12 en pipeline-full.md/task.md (commit 8f774c18) |
| `TSYS-08` | **Triage "es ahora" (Shape Up)** - el triage clasifica DO/DEFER/SKIP/BLOQUEADO pero no pregunta "¿es el problema adecuado? ¿correcto el appetite? ¿es ahora?" (gap-01 §3.5-8). | `.opencode/task-system/prompts/plan.md` | 🟢 | 🟢 | ✅ Implementado (T16) — triage "es ahora" + Cynefin en plan.md (commit 8f774c18) |
| `TSYS-09` | **Tracing de decisiones** - instrumentar `decision_reason`/`pattern` en `campaign_emit_event` y evento `plan.adjust` al cambiar estado (por qué se reabrió/cerró, qué patrón) (agent-03 §5.2/§9; gap-01 §3.3-17, FALLA #6). | `.opencode/task-system/mcp/campaign-server.mjs` | 🟢 | 🟡 | ✅ Implementado (TSYS-09) |
| `TSYS-10` | **Human-in-the-loop: escalera a humano** - HITL checkpoint: tareas 🔴 o ambiguas requieren confirmación humana antes de arrancar, salvo familia de ejecución ya aprobada (agent-03-orchestration.md:262-265 §6.1.6; gap-01 §3.3-24 relacionado). | `.opencode/task-system/prompts/subagent-recovery.md` | 🟢 | 🟠 | ✅ Implementado (T10) — §5 HITL checkpoint en subagent-recovery.md (commit d9f2a4cb) |
| `TSYS-11` | **Límites de herramientas por rol** - hoy todos los `permission:` de `.opencode/agents/*.md` otorgan `allow` amplio y los sub-agentes escalan a tools del lead; definir tabla de permisos por rol (worker = solo tools de su dominio; RESEARCH/bash read-only; solo vanta-lead hace git push/commit/release) y alinear los archivos de agentes (agent-03-orchestration.md §9.2). Contrato de referencia documentado en `.opencode/AGENTS.md` → "Límites de herramientas por rol". | `.opencode/agents/*.md`, `.opencode/AGENTS.md` | 🟡 | 🟡 | ✅ Implementado (T11) — tabla permisos por rol en .opencode/AGENTS.md (commit d9f2a4cb) |
| `TSYS-12` | **Waves en paralelo + merge del lead** - `FAIL_MODE=parallel` es single-loop síncrono sin merge step estructural ni modelado de critical path; diseño: 3-5 sub-agentes por wave + contrato de merge (duplicados/huecos/conflictos). Opcional, NO gate-CI (REPORTE-FINAL §3.4-4; agent-03 §7.2-7.4). Diseño propuesto en `docs/architecture/task-system-waves-parallel.md`. | `docs/architecture/task-system-waves-parallel.md`, `.opencode/task-system/prompts/pipeline-run.md` | 🟡 | 🟢 | ✅ Diseño implementado (T12) — `task-system-waves-parallel.md` (merge/duplicados/huecos); runtime opcional NO gate-CI (commit d9f2a4cb) |
| `TSYS-13` | **Validación de citas rotas por crawler** - el doc asume que el modelo valida URLs citadas; sin check mecánico la evidencia con cita rota se acepta. Paso de pipeline que extrae URLs de la evidencia, las resuelve (webfetch/HEAD; fallback manual sin red) y marca inválida la evidencia cuya URL no resuelve (agent-02 §7.8 y §11.2). | `.opencode/task-system/prompts/pipeline-full.md` | 🟢 | 🟢 | ✅ Implementado (T13) — step validación de citas en task.md (commit d9f2a4cb) |
| `TSYS-16` | **Definir "qué es feature shippable" (trunk-based)** - hoy el criterio queda al juicio humano; umbral a formalizar en el DoD: (a) tests unit + integración, (b) docs API/uso actualizadas en el mismo PR, (c) monitoring/observabilidad que evidencie que funciona, (d) rollback viable (revert limpio o flag-off, sin migración irreversible), (e) sin caballos sueltos (deuda conocida documentada con ID, no silenciosa) (REPORTE-FINAL §3.4-11). | `.opencode/references/definition-of-done.md` | 🟢 | 🟡 | ✅ Implementado (T16) — umbral "feature shippable" en definition-of-done.md (commit 138d8735) |
| `TSYS-14` | **Checklist anti-hábitos tóxicos como contrato** - checklist conductual (agent-02 §12) sin home ni enforcement; referenciarlo desde `prompts/task.md` como guía de comportamiento obligatoria en la fase de revisión (agent-02-task-execution.md:398-409; gap-01 §3.3-23 relacionado). | `.opencode/task-system/prompts/task.md` | 🟢 | 🟢 | ✅ Implementado (TSYS-14) |
| `TSYS-15` | **Memoria con esquema fijo y retrieval por tema** - escritura sin esquema → dos memorias desincronizadas (`lessons.md`/`decisions.md`); documentar campos mínimos (tema, fecha, decisión/lección, ref archivo) y read por tema (REPORTE-FINAL §3.4-2, FALLA #11). Esquema: `- <fecha-auto> | <tema> | <decisión|lección> | ref: <ruta:línea>`; read por tema vía `rg -n <tema> .opencode/task-system/memory/*.md`. | `.opencode/task-system/prompts/iter-loop-tools.md`, `.opencode/skills/campaign-executor/RULES.md` | 🟡 | 🟡 | ✅ Implementado (TSYS-15) |
