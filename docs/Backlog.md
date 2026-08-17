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
> **Total open items:** ~24 activas — previas (DISC-01..03, LEG-01, BIZ-01b, OLD-01, DESKTOP-15..27, [ADMIN-XX pending W4]) + P15 residuales (ERR-006/007/008/009/015/026/031/032/033/036/037/042/043/044/045/047/048/049) + P16 residuales (PERF-07/08/09, CI-01). P15/P16 principales ejecutadas por plan `docs/plans/archive/2026-08-09-backlog-pipeline.md` (49/49 ✅). P19 CI batch (CI-02..07) ejecutado y migrado 2026-08-12 — plan archivado `docs/plans/archive/2026-08-12-ci-deuda.md`. Origen: investigación multi-agente 2026-08-09 → `docs/Investigaciones/investigacion-equipo-2026-08-09.md`
> **Sync 2026-08-09:** plan `docs/plans/archive/2026-08-09-backlog-pipeline.md` archivado — 49/49 tareas delegables completadas (Wave 0-3: RELEASE-01/02/03, SEC-01, 24 ERR, 7 FEAT, REVISAR-01, COV-001/003/004, PERF-01/04/06, DOC-02..08). RELEASE-02 verificado live: 0.5.0 publicado (crates.io/PyPI/npm/GitHub 2026-08-01). Filas completadas eliminadas de P15/P16; residuales siguen activas. Task 50 COM-02/03 (humana) queda en la tabla.
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`
> **Sync 2026-08-14:** 9 agentes de `.opencode/agents/` revisados (lead/worker/arch/review/audit/docs/engine/tuner/chaos) → 10 recomendaciones R1-R10 agregadas como **P19** (mejoras del sistema de agentes: Output Templates, skills §6, permissions, §7, DISCOVERY).
> **Sync 2026-08-17:** batería de pruebas exhaustiva del MCP server (`vanta-cli 0.5.0`, 4 sub-agentes en paralelo, JSON-RPC stdio, DBs temporales aisladas) contra `.opencode/skills/vantadb-mcp/` → **P22** creada (14 tareas en 5 bloques: bugs server MCP, discrepancias skill↔realidad, deficiencias de documentación, referencias muertas, inconsistencias internas). Resultados: memoria 17/17, collections 12/12, grafo 25/25 PASS; **búsqueda 12/20 FAIL** (4 bugs reales del server, root cause trazado al código). Evidencia completa en los scripts `test-memoria.py`/`test-busqueda.py`/`test-grafo.py`/`test-collections.py` (temp) y en el historial de sesión.
> **Sync 2026-08-17 (verificación strategy multi-agente):** 6 sub-agentes verificaron los ítems propuestos de `docs/strategy/` + `docs/backlog-futuro.md` contra el código → **P23** creada (6 features VantaDB Pro sin código ni tracking), **P24** creada (I+D futura re-verificada: FUT-01 implementado, FUT-07/09 redefinidos), **P6** ampliada (9 tareas: MKT-04 publicación Reddit, MKT-18f/g/h/i, CLD-01/02/04, BLOG-CTA). Verificaciones en sesión: adapters `integrations/` NUNCA publicados en PyPI; SHOW_HN_PREP.md tenía 2 claims falsos (corregidos); ROADMAP.md stale (0.5.0/24 items reales vs v0.2.0/165 — banner añadido).

---

## Exec Summary

| Phase | Items | Est. Effort | Priority |
|-------|-------|-------------|----------|
| **P0** 🚀 Release Blockers | 0 — ✅ 3/3 ejecutadas (plan 2026-08-09: RELEASE-01 semver-checks, RELEASE-02 publish 0.5.0 verificado live, RELEASE-03 artefactos) | — | ✅ Cerrada |
| **P1** 🛡️ Security & Critical | 0 — ✅ 1/1 ejecutada (SEC-01 UAF `__array_interface__` fix) | — | ✅ Cerrada |
| **P2** ⚡ Quick Wins Técnicos | 0 | — | ✅ Cerrado |
| **P3** 🧪 Test Coverage (core SDKs) | 0 — ✅ 4/4 ejecutadas (COV-001..004 completadas 2026-08-12) | — | ✅ Cerrada |
| **P4** 🔧 Engineering Health | 0 — PERF-01..09 migradas a progreso 2026-08-12 | — | ✅ Cerrado |
| **P5** 📖 Docs & Community | 3 (DISC-01..03) | ~1-2 semanas | 🟡 Media |
| **P6** 🚀 Launch Campaign | 11 (LEG-01, MKT-04, MKT-18f/g/h/i, CLD-01/02/04, BLOG-CTA) | ~2-3 semanas | 🟡 Media |
| **P7** 🌐 WASM & Performance | 0 | — | ✅ Cerrado |
| **P8** 🔮 Post-Launch & Enterprise | 1 (BIZ-01b) | ~3-5 semanas | 🔵 Futuro |
| **P9** 📚 Old Docs Rescue (reference) | 1 (OLD-01) | — | 📖 Referencia |
| **P10** 🏗️ Competitive Features (catalog) | 0 | — | 🗺️ Roadmap |
| **P11** 🐛 GitHub Issues | 0 | — | ✅ Cerrado |
| **P12** 🖥️ DESKTOP App (Tauri) + Consola Admin | 25 (DESKTOP-12..27 + ADMIN-01..09) | ~4-6 semanas | 🔵 Futuro |
| **P13** 🔎 AUDREP — Audit Report 2025 | 0 | — | ✅ Cerrado |
| **P14** 🔍 REVIEW items | 0 | — | ✅ Cerrada |
| **P15** 🔍 ERR items (revisión multi-agente 2026-08-08) | 0 — ✅ todos resueltos (36 por plan 2026-08-09 + 10 migradas 2026-08-12 + ERR-007 ban-skip 2026-08-12) | — | ✅ Cerrada |
| **P16** 🧩 Completitud de Features (investigación 2026-08-09) | 1 residual (CI-01) | 📆 Backlog | 🟢 Baja (19 ejecutadas 2026-08-09 + PERF-07/08/09 2026-08-12) |
| **P23** 🔒 VantaDB Pro (Open Core) | 6 (PRO-01..06) | ~8-12 semanas | 🔵 Futuro |
| **P24** 🧪 I+D futura (v3.0+) | 10 (FUT-02..11) | 📆 Futuro | 🗺️ Roadmap |

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

## Phase 4: 🔧 Engineering Health & Architecture

> Investigaciones de salud de ingeniería — rendimiento, concurrencia, arquitectura.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|

> **Ejecutadas 2026-08-12 — tanda 1 (commits en develop, migradas a progreso):** PERF-01 (`30e90cd9` claims revalidados), PERF-04 (`152ddd26` prefetch flag default off), PERF-06 (`914514bb`+`d9378656` KB/MB/GB), PERF-07 (`88b0f875` sparse parse explícito), PERF-09 (`0be56cac` cold-start log honesto).
> **Ejecutadas 2026-08-12 — tanda 2 (commits `32462de6`/`437a1125`/`9eef37c5`/`5105f22d`, migradas a progreso):** PERF-02 (baseline criterion determinista + critcmp), PERF-03 (bench competitivo honesto SDKs), PERF-05 (ADR DRV-015 WAL async roadmap), PERF-08 (WASM Float32Array zero-copy). P4 completo.
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
| `MKT-04` | **Publicar 3 drafts de Reddit (r/rust, r/MachineLearning, r/LocalLLaMA)** — drafts listos en `docs/strategy/REDDIT_POSTS.md` (status: draft), NUNCA publicados. Verificado 2026-08-17. ⚠️ Corregir claims primero: "recall>0.998 SIFT1M"/"zero deps" no verificados (ver SHOW_HN_PREP.md nota). | 🟢 2-4h | 🟠 | ❌ Pendiente |
| `MKT-18f` | **Publicar 5 adapters en PyPI + PRs upstream** — langchain, llama-index, mem0, crewai, dspy. Verificado 2026-08-17: código existe en `integrations/` pero **404 en PyPI** (progreso "inflado"). Desbloquea GTM checkboxes langchain/llama-index/Mem0/TSK-90/91. | 🟡 1-2d | 🔴 | ❌ Pendiente |
| `MKT-18g` | **Corregir claims falsos en SHOW_HN_PREP.md + REDDIT_POSTS.md** — cpufeatures NO es dep (usa `std::is_x86_feature_detected!`, `src/hardware/mod.rs:236-245`); "zero-dependency" falso (croaring→cc compila C/C++); recall ">0.998 SIFT1M" y "sub-ms" sin verificar a esa escala (medido 0.9975 @ef_400 SIFT 10K, ~1.2ms @ef_200). Notas ya aplicadas 2026-08-17; queda verificar consistencia total del post + benchmark SIFT1M si se quiere el claim. | 🟢 2-4h | 🔴 | 🟡 Notas aplicadas, verificación pendiente |
| `MKT-18h` | **Wheels ARM64 Linux + SHA reales Homebrew** — binarios incluyen `aarch64-unknown-linux-gnu` (`release-binaries-63.yml`) pero wheels NO (`release-wheels-60.yml` solo x86_64); `Formula/vantadb.rb` tiene SHA256 `0000...0` placeholders (inutilizable). | 🟡 1d | 🟠 | ❌ Pendiente |
| `MKT-18i` | **Docker Compose multi-servicio: Ollama + VantaDB + AnythingLLM** — `docker-compose.yml` existe pero solo servicio VantaDB. La guía migración LanceDB YA existe (`docs/tutorials/migration-from-lancedb.md`). | 🟢 2-4h | 🟡 | ❌ Pendiente |
| `CLD-01` | **VantaDB Cloud beta on Fly.io** — checkbox vacío en `GO_TO_MARKET.md:420`; cero archivos de infra. Verificado 2026-08-17: no existe nada. | 🟠 1-2 sem | 🔵 | ❌ Pendiente |
| `CLD-02` | **Pitch deck + one-pager** — checkbox vacío en `GO_TO_MARKET.md:408`; cero archivos `*pitch*`/`*deck*`. | 🟡 3-5d | 🔵 | ❌ Pendiente |
| `CLD-04` | **Case study #1 (enterprise pilot)** — checkbox vacío en `GO_TO_MARKET.md:409`; cero archivos. Depende de pilot real. | 🟠 1 sem | 🔵 | ❌ Pendiente |
| `BLOG-CTA` | **CTAs + metadata de la serie de blogs + posts 6-7** — M3 🟡 date drift (2026-06-06 vs web 2025), M4 🟡 title drift, CTA débil en 2 posts (`how_hybrid_search_works.md`, `sqlite_for_ai_agents.md`), posts 6-7 no redactados (Ollama+VantaDB, Claude Code MCP). M1/M2/M5/M6 ya resueltos. | 🟡 3-5d | 🟠 | ❌ Pendiente |

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

> **Origen 2026-08-12:** batch CI-01 de `docs/archive/EXTRACCION-DOC-OLD-2026-08-05.md` §4, verificado contra código real (Paso 0). ROOT1-007 (release catch-22) **resuelto** — `release-plz.toml` tiene `git_release_enable=true` (fa4c6849) y GitHub Release v0.5.0 publicado 2026-08-01. WF1-001 (RUSTSEC) **resuelto** — audit.toml removió 0176/0177. PLAN2-038 **resuelto** — publish-ts ya declara `needs: [publish-wasm]`. WF1-002/003/004 **categorizados** — sanitizers con `# CATEGORY: BEST-EFFORT` (conforme Regla 2).

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|
> **Sync 2026-08-12:** batch CI-02..07 ejecutado y migrado a `docs/progreso/README.md` (plan `docs/plans/archive/2026-08-12-ci-deuda.md`).

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
> **Total:** 21 items, **1 activo** (OLD-01; 8 ✅ removidos a progreso, 12 migrados/cerrados). **Estado:** OLD-01 ❌ No implementado.
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
> **Total:** 30 items, **0 activos** (tablas detalladas vacías; 12 ✅ removidos a progreso, 18 migrados/cerrados — ver `docs/progreso/BACKLOG_HISTORY.md` P10).
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
> **Total:** 14 tareas (issues #119–#144), **0 activas en backlog** (tabla vacía — cerradas/migradas). Todos son `good first issue`.
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

---

## P15 — Hallazgos de Revisión Multi-Agente 2026-08-08 (errors-found.md)

> **Origen:** `docs/reviews/errors-found.md` — revisión por capas con 6 sub-agentes en paralelo (vanta-audit, vanta-arch, vanta-engine, vanta-worker, vanta-tuner, vanta-docs) + verificación manual del lead sobre `develop@7a19a9f5`. 51 hallazgos documentados; ERR-017 descartado al verificar (métrica euclidiana uniforme en `distance.rs:495/516/536`). Índices de esfuerzo: 🟢 < 1 día | 🟠 1-3 días | 🔴 > 3 días.

### CRÍTICOS (5) — ✅ todos ejecutados por plan `2026-08-09-backlog-pipeline.md` (ver progreso)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-010` | **🔴 Persistencia / Race checkpoint↔snapshot** — *resuelto*. Fix original `a5ca4389` (insert_lock across checkpoint/save) + deadlock real reparado en `c9188639` (flush antes de insert_lock). **Reapertura 2026-08-11 cerrada**: 13 tests fallaban por contratos desactualizados + 2 bugs reales del core — `9019342d` (scan excluía nodo id 0 por `cursor.parse().unwrap_or(0)`), `164108e1` (snapshot copiaba flat en snap_dir/ pero `init.rs:291` espera `snap_dir/data/` → reapertura vacía), `0fd1d24a` (13 tests alineados: 5 zero-norm, 3 stemming Tantivy, 4 snapshot-misc, 1 deadlock propio del test). Verify: nextest audit 1902 passed, bins pesados 15/15 + 4/4 + 18/18, fmt+clippy limpios. Deuda original en `docs/plans/archive/2026-08-09-backlog-pipeline.md:286`. | `src/storage/engine/` | 🔴 | 🔴 | ✅ Completado |
| `ERR-021` | **🔴 MCP OOM** — *resuelto* (`b01e9ed6`: streaming restaurado con take(n) + límite). | `vantadb-mcp/src/lib.rs` | 🟠 | 🔴 | ✅ Completado |
| `ERR-022` | **🔴 top_k/k sin tope → alloc gigante** — *resuelto* (`3eeb86e1`: k.min(MAX_K) en bindings+MCP). | bindings + `src/index/search.rs` | 🟢 | 🔴 | ✅ Completado |
| `ERR-035` | **🔴 Read-lock global HNSW** — *resuelto* (`1b0016d5`: contención reader/writer mitigada). | `src/physical_plan.rs:211`, `src/storage/engine/ops.rs` | 🔴 | 🔴 | ✅ Completado |

### ALTOS (16 verificados) — ✅ 14 ejecutados por plan `2026-08-09` (ERR-001..025; ver progreso). Residuales:

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-037` | **🟠 `batch_insert` chequea existencia por nodo** — *resuelto* (`b97c0ccd` probe + follow-up `ExistingMeta` chunked): overwrite path −30.3% (97.1→67.7ms @10k), probe cache-hit 1-3ms. Bench en `benches/batch_existing_check.rs`. | `src/storage/engine/ops.rs` | 🟠 | 🟠 | ✅ Completado |

### MEDIOS (12) — ✅ 7 ejecutados por plan `2026-08-09` (ERR-005/014/027/028/029/030/050) + ERR-026 resuelto (`ce265569`/`aa1754d2`) y migrado a progreso 2026-08-12. Sin residuales.

### BAJOS (9) — ✅ 3 resueltos/verificados por plan `2026-08-09` (ERR-016 SKIP verificado, ERR-034 ⏫, ERR-051 ⛠) + ERR-015/032/033/047/048 resueltos y migrados a progreso 2026-08-12. Sin residuales.

### INFO (5) — ✅ 5/5 resueltos/obsoletos (ERR-006 deny.toml limpio, ERR-008 `copy_unsafe` ya no existe, ERR-009 job Miri en CI `ci-rust-10.yml:457`, ERR-049 `ivf_bench.rs` registrado `Cargo.toml:247`, ERR-007 multiple-versions documentado en deny.toml `[bans] skip` 2026-08-12). Sin residuales — sección cerrada.

> **Descartado:** ERR-017 (métrica euclidiana consistente en `flat.rs:43` / `distance.rs:495/516/536` / `search.rs:176/327` — no se confirma la divergencia flat vs HNSW).

---

## Phase 16: 🧩 Completitud de Features & Docs (de investigación multi-agente 2026-08-09)

> Origen completo: `docs/Investigaciones/investigacion-equipo-2026-08-09.md`. **Ejecutada 2026-08-09 por plan `docs/plans/archive/2026-08-09-backlog-pipeline.md` — 19/19 tareas ✅** (FEAT-01..07, REVISAR-01, COV-001/003/004, PERF-01/04/06, DOC-02..08; RELEASE-01/02/03 + SEC-01 en Wave 0). Commit por feature en `docs/progreso/README.md`. Residuales viven en sus secciones: `CI-01` (hallazgos pendientes/CI), `REVIEW-05` (P14).

---

## Referencias Cruzadas

- **RC items:** `docs/reviews/PROJECT_FULL_REVIEW_2026-07-13.md` (generado por `vantadb-full-review` skill)
- **REV items:** `docs/reviews/2026-07-13-full-review.md`
- **DRV findings:** `docs/plans/2026-07-15-cross-ref-docs-vs-code.md` + `docs/audit-reports/cross-ref-wave3-final-report.md`
- **OLD items:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 — ~280 archivos VANTADB DOC OLD analizados
- **COMP items:** `docs/audit-reports/competitive-features-consolidated-report.md` + `docs/audit-reports/deep-analysis-{vector,graph,arch}.md` — 27 archivos, 172 features, top 30 priorizados

---



## Hallazgos pendientes de reportes

Hallazgos >= medium derivados de reportes de auditoría. Fuente: `docs/reviews/audit-full-20260812-231204.md` (2026-08-12).

| ID | Severidad | Hallazgo | Archivo:línea | Estado |
|----|-----------|----------|---------------|--------|
| AUD-042 | Media | Upgrade tantivy ≥0.18 — elimina allowlist RUSTSEC-2026-0253 + desbloquea lru 0.18 (security debt, rec#6) | Cargo.toml (tantivy), deny.toml | 🔴 BLOQUEADO upstream — verificado 2026-08-13: tantivy 0.26.1 (última publicada) fija `lru ^0.16.3`; el fix (`lru = "0.18.2"`) está en tantivy main (0.27.0) pero NO publicada en crates.io (404). Re-evaluar cuando tantivy ≥0.27.0 publique: bump tantivy + lru directo a 0.18.2 y remover allowlist. Comentario deny.toml actualizado con el estado. |


---

## P17 - Mejoras del task-system (de REPORTE-FINAL 2026-08-10 §3.3/§3.5)

> Items §3.3 (FALTA, cobertura 0-30%) y §3.5 (MAL/CONTRADICTORIO) de `docs/Investigaciones/2026-08-10-agent-engineering/REPORTE-FINAL.md` no asignados a los planes P0-P3 (Task 14 de `docs/plans/archive/2026-08-10-docs-task-system-consolidation.md`). **TSYS-01..05, 07..11, 13..16 implementados 2026-08-11 → migrados a progreso** (plan `docs/plans/archive/2026-08-11-residuo-consolidado.md`). Pendientes: TSYS-06 (runner DEFER) y TSYS-12 (runtime opcional, NO gate-CI). Los de runtime con datos dependen de Task 2 (poblar `verify-log.jsonl`).

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `TSYS-12` | **Waves en paralelo + merge del lead** - `FAIL_MODE=parallel` es single-loop síncrono sin merge step estructural ni modelado de critical path; diseño: 3-5 sub-agentes por wave + contrato de merge (duplicados/huecos/conflictos). Opcional, NO gate-CI (REPORTE-FINAL §3.4-4; agent-03 §7.2-7.4). Diseño propuesto en `docs/architecture/task-system-waves-parallel.md`. | `docs/architecture/task-system-waves-parallel.md`, `.opencode/task-system/prompts/pipeline-run.md` | 🟡 | 🟢 | ✅ Diseño implementado (T12) — `task-system-waves-parallel.md` (merge/duplicados/huecos); runtime opcional NO gate-CI (commit d9f2a4cb) |

---

## P18 - Gaps residuales del task-system (re-verificados 2026-08-12, para investigación y decisión)

> Re-verificación post-auditoría de `docs/Investigaciones/2026-08-10-agent-engineering/` (10 sub-agentes, reporte consolidado en `docs/reviews/audit-agent-engineering-2026-08-11.md`). Los 48 ❌ brutos se redujeron a **8 gaps reales** tras descartar falsos negativos (ADR gate, DMAIC, 2-sombreros, merge-a-main-DoD), DEFER con fecha (P3-2 calibración, P3-3/P3-9 mutation) y cosméticos. Los 7 gaps TIR (investigados 2026-08-17, docs en `docs/Investigaciones/TIR-*.md`) tienen decisión registrada en la columna Estado; NO son tareas de implementación directa. **Follow-ups pendientes de las decisiones:** (a) TIR-02 — implementar recovery time en `evals/dora.mjs` (~30 líneas); (b) TIR-04 — formalizar contenedor de tareas fallidas (`tasks/closed/` + regla re-procesamiento `pending` + índice `rg "❌ FAILED"`); (c) TIR-08 — criterios 1-2 en `research-agent.md` (~6 líneas).

| ID | Descripción | Origen (doc:línea) | Afecta | Esfuerzo | Prio | Estado |
|----|-------------|--------------------|--------|----------|------|--------|
| `TIR-01` | **Compaction de contexto runtime** - no existe mecanismo de compactación de contexto para tareas largas; el harness depende de note-taking manual (`Context Save Point`) y la escalera de retry solo ofrece "contexto fresco + resumen ~200 tokens". Investigar: ¿resumen incremental por fase? ¿qué se conserva? Comparar con el claim original (multi-turn compaction). | agent-01-fundaments.md #21/#40/#59; iter-loop-tools.md:178 | Tareas de ejecución larga / multi-turn en el loop | 🟡 | 🟠 | ✅ Investigada — decisión: micro-cambio de prompt (Context Save Point por fase + SARL lee task file como digest); aplicar a criterio del lead. Doc: `docs/Investigaciones/TIR-01-contexto-compaction.md` |
| `TIR-02` | **DORA recovery time + rework rate** - `docs/reports/dora.md` mide lead/cycle/CFR/throughput pero no recovery time (tiempo en volver a DE del pipeline tras fallo) ni rework rate (tareas reabiertas/total). Requiere datos de `verify-log.jsonl` (Task 2 histórico). Investigar viabilidad de métricas con la telemetría actual antes de implementar. | eng-03-project.md §8.3 (DORA); docs/reports/dora.md:183-197 | Observabilidad del pipeline | 🟡 | 🟠 | ✅ Investigada — decisión: **IMPLEMENTAR recovery time** (verify-log.jsonl existe y se puebla; extender `evals/dora.mjs` ~30 líneas); **DEFERIR rework rate** hasta TSYS-09 completo. Doc: `docs/Investigaciones/TIR-02-dora-recovery-rework.md` |
| `TIR-04` | **Dead-letter queue** - tareas que agotan retries solo escalan a humano (SARL ESCALATE); no hay cola de mensajes muertos que preserve el estado/traceId de la tarea fallida para revisión posterior ni re-procesamiento. Investigar si basta un contenedor de tareas fallidas citado desde el plan vs infraestructura nueva. | agent-02-task-execution.md §8.2; pipeline-run.md (SARL) | Retry/recuperación de sub-agentes | 🟢 | 🟡 | ✅ Investigada — decisión: **IMPLEMENTAR contenedor de tareas fallidas citado desde el plan** (formalizar `tasks/closed/` + regla re-procesamiento `pending` + índice `rg "❌ FAILED"`); **WONTFIT** DLQ infraestructura nueva. Doc: `docs/Investigaciones/TIR-04-dead-letter-queue.md` |
| `TIR-05` | **LLM-as-judge (0.0-1.0)** - los evals son mecánicos (compare/verify primitivo); no hay judge de fabricación para salidas sin ground-truth determinista. Investigar si aplica a salidas sintéticas del task-system (resúmenes, categorizaciones) vs costo por llamada. | agent-03-orchestration.md #35; evals/ | Calidad de verificación de salidas no-deterministas | 🟡 | 🟢 | ✅ Investigada — decisión: **DEFERIR** (falta rúbrica calibrada + volumen bajo; evals hoy 100% mecánicos cubren el caso). Triggers de reapertura en doc. Doc: `docs/Investigaciones/TIR-05-llm-as-judge.md` |
| `TIR-06` | **Post-release / monitoring en el loop** - el pipeline cierra en CLOSE (commit) sin verificación post-merge; no hay paso de monitoreo de lo shippeado ni retroalimentación al pipeline. Relacionado con DoD (d) monitoring. Investigar: ¿un paso de "verificación post-release" opcional vs delegación a `progreso`/registro? | REPORTE-FINAL §3.3-27; definition-of-done.md:104 | Pipeline (cierre de campaña) | 🟢 | 🟢 | ✅ Investigada — decisión: **DEFERIR** (duplicaría release-plz/CI; gap real ya priorizado como P0-1 evals del pipeline en REPORTE-FINAL §3.7). Cierre opcional: 1 línea en pipeline-run.md CLOSE. Doc: `docs/Investigaciones/TIR-06-post-release-monitoring.md` |
| `TIR-07` | **Chaos runner del task-system (TSYS-06 runtime)** - existe el diseño (`docs/architecture/task-system-chaos-resilience.md`, T19) pero no runner que fuzzee `campaign-server.mjs`/máquina de estados. Marca "DEFER (sin verificación)". Investigar si construir el runner vale frente a tests de inyección de fallos puntuales. | TSYS-06; task-system-chaos-resilience.md; gap-01 §3.3-24 | Robustez del MCP server | 🔴 | 🟢 | ✅ Investigada — decisión: **DEFERIR runner** (pre-condiciones T19 ya implementadas; ~9/12 escenarios ya cubiertos); cerrar gaps C8/C9/concurrencia con tests puntuales. Doc: `docs/Investigaciones/TIR-07-chaos-runner.md` |
| `TIR-08` | **Saturación <20% + Broadening/Narrowing + jitter en retry** - criterios de investigación (stop si saturación <20%) y de re-enfoque (broadening/narrowing) están en la investigación pero no en prompts; retry sin jitter (exponential backoff determinista en RULES.md:453). Investigar si formalizarlos en `iter-loop-tools.md`/`task.md` o si son guías tácitas. | agent-02-task-execution.md §7.2/§7.6/§8.1; RULES.md:453 | Prompts de iteración/retry | 🟢 | 🟢 | ✅ Investigada — decisión: **IMPLEMENTAR parcial** (criterios 1 y 2 en `research-agent.md` ~6 líneas); **WONTFIT** jitter (backoff determinista correcto, thundering-herd no aplica). Doc: `docs/Investigaciones/TIR-08-saturacion-broadening-jitter.md` |

---

## P19 - Mejoras del sistema de agentes (recomendaciones multi-agente 2026-08-14)

> Revisión de los 9 agentes en `.opencode/agents/` (2026-08-14): Output Templates, skills §6, bloques `permission:`, bloque §7 duplicado y delegación DISCOVERY. Evidencia con `file:línea` en cada fila. R1-R3/R7/R9 son los de mayor impacto; R4 es decisión registrada (NO hacer).

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|


| `R4` | **NO fragmentar por step (decisión)** — cada paso como sub-agente dedicado = overhead de sesión por ~100 líneas + riesgo de regresión; SARL RESUME (`subagent-recovery.md:29-34`) ya conserva el contexto del sub-agente ejecutor con task_id; `/pipeline run` → `pipeline-full.md` es la respuesta correcta. Registrado para no re-proponer. | — | — | — | ✅ Decidido (no hacer) |






---

## P20 - Fundamentos CS & Stack Engineering (roadmap.sh 2026-08-16)

> **Origen:** análisis de roadmap.sh (Backend / Software Architect / Computer Science / DevOps / Product Management) aplicado a VantaDB + ajuste pre-launch (auditoría dirigida del código antes del Show HN Sept 2026). **Proceso obligatorio por tarea:** Investigación a profundidad → Análisis a profundidad → Plan de implementación detallado (si procede) → DoD verificado. Cada tarea integra su fase de verificación/análisis; no separar regla y auditoría en tareas distintas. **Poda 2026-08-16:** eliminadas FND-08/19/20/29 (métodos de aprendizaje del fundador y nota de decisión, sin valor al producto) y unificadas 8 duplicaciones regla↔auditoría (FND-10/11/13/14/24/25/26/27 absorbidas). Fuente del roadmap: `docs/Investigaciones/` (generar `2026-08-16-roadmap-stack.md` al ejecutar la primera tarea).

### P20a - Reglas de ingeniería (regla + verificación integrada)

| ID | Descripción (Investigación → Análisis → Implementación) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|









### P20b - Instrucciones para AGENTS.md (prácticas del agente)

> **Eliminadas 2026-08-16:** métodos de aprendizaje del fundador (Socratic FND-08, teach-back FND-20, plan 90/10 FND-29) y nota de decisión System Design (FND-19) — sin valor al desarrollo del producto.

| ID | Descripción (Investigación → Análisis → Implementación) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|






### P20c - Tareas de verificación y análisis (sin regla asociada)

| ID | Descripción (Investigación → Análisis → Plan si procede) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|


### P20d - Fase 0 pre-launch y decisiones de producto (2026-08-16)

> **Origen:** textos 3 y 4 (cierre de ciclo roadmap.sh). Acciones concretas de la Fase 0 (pre-Show HN) y decisiones de producto post-launch NO cubiertas por FND-01..17. Prioridad: FND-18/19 son la "acción física de hoy" pre-launch; FND-22/23/24 son post-launch (se ejecutan tras el Show HN).

| ID | Descripción (Investigación → Análisis → Implementación) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

## P21 - Skills de VantaDB (`skills/`) — ✅ Cerrada (2026-08-17)

> **CERRADA 2026-08-17:** 4/4 tareas ejecutadas. SKL-01/02/03 implementadas (docs + scripts), SKL-04 gate P2-01 emitido (CHANGES-REQUIRED con 1 falla de docs — "4 resources" vs 2 reales — fix aplicado por SKL-02 y re-verificado 4/4 exit 0). Verificado por el lead: `test-mcp.py` 4/4 exit 0 contra `vanta-cli.exe` v0.5.0 (15 tools, 2 resources, 4 prompts). Plan archivado: `docs/plans/archive/2026-08-17-skills-vantadb.md`. Task files: `.opencode/skills/campaign-executor/tasks/SKL-0{1,2,3,4}.md` (✅). Detalle en `docs/progreso/README.md`.

## P22 - Certificación del MCP server vs skill `vantadb-mcp` (2026-08-17)

> **Origen:** batería de pruebas exhaustiva (4 sub-agentes vanta-worker en paralelo, JSON-RPC 2.0 stdio contra `vanta-cli server --mcp --db <temp>`, binario 0.5.0 en PATH, DBs temporales aisladas, asserts por afirmación documentada de la skill). Resultados por área: **Memoria CRUD 17/17 ✅ · Collections 12/12 ✅ · Grafo/IQL 25/25 ✅ · Búsqueda 12/20 ❌ (8 FAIL = 4 bugs reales + discrepancias)**. Las skills son copia exacta (hash SAME) de las versionadas en `skills/vantadb-mcp/` (wave SKL). Los 4 bugs del server están trazados al código fuente (root causes abajo). Scripts de prueba (re-ejecutables, autocontenidos): `C:\Users\Eros\AppData\Local\Temp\opencode\test-{memoria,busqueda,grafo,collections}.py`. **Orden de ejecución:** Bloque 1 (server) primero → Bloque 2 (doc ya sincronizada a la realidad) → Bloques 3-5 en paralelo (vanta-docs).
> **⚠️ Regla de edición de skills (aplica a Bloques 2-5):** la fuente de verdad **versionada** es `skills/vantadb-mcp/` (commit `61381d29`, wave SKL) — editar SIEMPRE ahí. `.opencode/skills/vantadb-mcp/` es la **copia runtime sin versionar** que OpenCode usa para resolver `skill <nombre>` (lookup: `.opencode/skills/` primero) y las rutas relativas de SKILL.md (`../../docs/api/MCP.md` etc.) solo resuelven desde `skills/`. Después de editar, **copiar los archivos tocados a `.opencode/skills/vantadb-mcp/`** para que ambos lados queden idénticos (hash SAME) antes de cerrar la tarea.

### Bloque 1 — Bugs del server MCP (código, vanta-arch/vanta-worker) — BLOQUEANTES

> Problema actual: la skill declara disponibles features de búsqueda lexical/híbrida que **fallan siempre** vía MCP en DB fresca, y 3 discrepancias de comportamiento más. El MCP server (`vanta-cli server --mcp`) es la vía canónica para agentes (OpenCode/Claude/Cursor) — estos bugs rompen el contrato documentado. Delegar diseño del fix S1 a vanta-arch (dónde llamar `ensure_indexes_current`) y S2-S4 a vanta-worker.

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `MCP-01` | **S1 🔴 text_query/hybrid/filters-text ROTOS vía MCP** — Problema: `search_memory` con `text_query` (solo o híbrido) y `filters` en path textual fallan siempre con `Search Error: text_index not found: bm25` en DB fresca (tests T09/T11/T13). Root cause trazado: `vantadb-server/src/main.rs` abre `StorageEngine` directo y **nunca ejecuta `ensure_indexes_current()`** — esa construcción del text index solo ocurre en `VantaEmbedded::open_with_config` (`builder.rs:105`) o vía `rebuild_index()` (`api.rs:660`), ninguna tool MCP la expone. Los puts sí escriben postings (`text_postings_written: 26` en metrics) pero el estado del índice nunca se crea → `ensure_text_index_query_ready` (`text_index.rs:18`) falla siempre. Acciones: (1) decidir el punto de construcción del text index en el path MCP (llamar `ensure_indexes_current` en el arranque del server, o exponer `rebuild_index` como tool MCP, o lazy-build en primer put); (2) implementar; (3) re-ejecutar `test-busqueda.py` — T09/T11/T13 deben pasar. Resultado: búsqueda lexical/híbrida/filters-text funcional vía MCP en DB fresca, sin pasos manuales. | `vantadb-server/src/main.rs`, `vantadb-mcp/src/handlers/tools.rs`, `src/sdk/builder.rs:105`, `src/sdk/api.rs:660`, `src/text_index.rs:18` | 🔴 | 🔴 | ✅ Fix: ensure_indexes_current pub + llamado en arranque run_stdio_server; T09/T11/T13 ✅; 32/32 tests |
| `MCP-02` | **S2 🟠 `distance_metric=euclidean` sin efecto observable** — Problema: `search_memory` con `distance_metric: "euclidean"` produce scores IDENTICOS a cosine (T14: `[1.0, 0.9701, 0.0...]`); el parámetro por-request no se aplica. Root cause: `flat_search` usa `self.config.distance_metric` (`src/index/flat.rs`), ignora la métrica del request. Acciones: (1) propagar `distance_metric` del request al cálculo de score en flat/HNSW (o documentar que es config-time si es decisión); (2) implementar; (3) verificar con T14 que cosine ≠ euclidean. Resultado: el parámetro documentado tiene efecto real. | `src/index/flat.rs`, `src/index/search.rs`, `vantadb-mcp/src/handlers/tools.rs` | 🟡 | 🟠 | ✅ Fix: métrica per-request propagada (nearest.rs/search/mod.rs/alternate.rs); T14 euclidean≠cosine ✅ |
| `MCP-03` | **S3 🟠 `search_semantic.distance` = similaridad coseno mal etiquetada** — Problema: `search_semantic` devuelve `distance=1.0` para vector idéntico y `0.0` para ortogonal (T17/T18) — es similaridad coseno con el nombre "distance" (la skill y `api-reference.md` documentan "distance: lower is more similar"). Ranking correcto, valor invertido. Acciones: (1) corregir la semántica (devolver distancia real `1 - cosine_sim`, o renombrar el campo a `similarity` — decisión de API pública, requiere `feat!`/semver según Regla 7); (2) implementar en el handler y/o la fuente del valor; (3) verificar T17/T18: idéntico→0.0, orden ascendente. Resultado: el campo `distance` significa lo que documenta. | `vantadb-mcp/src/handlers/tools.rs`, `src/index/search.rs` | 🟡 | 🟠 | ✅ Fix: handler convierte score→distancia real (Cosine→1−sim); 33/33 tests; doc api-reference actualizada |
| `MCP-04` | **S4 🟡 Sin validación de dimensionalidad en `search_semantic`** — Problema: query vector 3-dim contra índice 4-dim aceptada con éxito y distancias 0.0 (T19) — scoring silenciosamente basura en vez del documentado `VantaError::DimensionMismatch { expected, got }` (`api-reference.md`, `src/error.rs`). Acciones: (1) validar dims del query contra la dim del índice en el handler `search_semantic` (y `search_memory` vectorial); (2) devolver error claro; (3) verificar T19: error DimensionMismatch con expected/got correctos. Resultado: queries mal dimensionadas fallan con error explícito, no con basura. | `vantadb-mcp/src/handlers/tools.rs`, `src/error.rs` (DimensionMismatch) | 🟢 | 🟡 | ✅ Verificado aislado (T19) — batería completa bloqueada por MCP-15 |
| `MCP-15` | **S5 🔴 Stack overflow del child `vantadb-server` durante `search_semantic`/`search_memory`** — Problema: tras la secuencia T09-T16, `search_semantic` con dim VÁLIDA crashea el child con `thread 'tokio-rt-worker' (18480) has overflowed its stack` → pipe roto (`[Errno 22]`) en T17/T18/T19 (test-busqueda.py 16/20). Detectado independientemente por MCP-03 y MCP-04. Bump `thread_stack_size(8 MiB)` probado → no resuelve (recursión real, probablemente serialización del node payload en el worker tokio del child). Acciones: (1) repro directo del child (repro-full.py) y trazar la recursión; (2) romper la recursión (iterativo/boxed/refcount) o el worker que la dispara; (3) re-ejecutar test-busqueda.py completo — 20/20. Resultado: la batería completa pasa sin crash del child. | `vantadb-server` (child tokio-rt-worker), serialización node payload | 🔴 | 🔴 | ❌ Activa |
| `T15` | **S6 🟡 `explain=true` no devuelve el contrato documentado** — Problema: T15 falla por discrepancia de shape: el hit no trae `explanation` ni `route`/`fusion_report`; trae `rrf_text_rank`/`rrf_vector_rank`. Acciones: (1) verificar contra el código del handler qué shape real devuelve explain; (2) alinear docs (`api-reference.md` / skill) o el handler según el contrato deseado; (3) T15 verde. Resultado: explain documentado == explain real. | handler `search_memory` explain, `references/api-reference.md` | 🟢 | 🟡 | ❌ Activa |

### Bloque 2 — Discrepancias skill ↔ realidad (docs, vanta-docs — DESPUÉS del Bloque 1)

> Problema actual: la skill `skills/vantadb-mcp/SKILL.md` documenta features como disponibles que el server no cumple (o al revés). Dependencia: ejecutar DESPUÉS de los fixes del Bloque 1 para documentar la realidad ya corregida; si un fix del Bloque 1 se difiere, marcar la feature como limitada en la skill (nunca dejar el claim falso).

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `MCP-05` | **D1 Sección "Hybrid Search" + `search_memory` declaran búsqueda textual/híbrida** — Problema: SKILL.md:200-206 y tool `search_memory` (L118) presentan `text_query`/hybrid como disponibles; en la realidad actual fallan siempre (MCP-01). Acciones: tras fix MCP-01, verificar y dejar como está; si el fix se difiere, añadir nota "text search requiere text index construido; ver MCP-01" + ejemplo mínimo de uso. Resultado: la skill no promete features rotas. | `skills/vantadb-mcp/SKILL.md` (L117-123, L200-206) | 🟢 | 🟠 | ✅ Verificado post-fix MCP-01 + ejemplo mínimo de hybrid query |
| `MCP-06` | **D2 `distance_metric` documentado `cosine \| euclidean`** — Problema: SKILL.md:118 y api-reference declaran la métrica configurable; el parámetro no tiene efecto observable (MCP-02). Acciones: tras fix MCP-02 verificar; si se difiere, documentar "métrica del índice (config-time)" en vez de parámetro por-request. Resultado: doc alineada con el comportamiento real. | `skills/vantadb-mcp/SKILL.md` (L118), `references/api-reference.md` | 🟢 | 🟡 | ✅ Documentado per-request con efecto observable (cosine≠euclidean) |
| `MCP-07` | **D3 `search_semantic` "Nearest neighbors with distances"** — Problema: los "distances" reales son similaridad invertida (MCP-03); la skill y `api-reference.md:299-306` documentan "lower is more similar". Acciones: tras fix MCP-03, verificar y dejar; documentar el significado exacto del campo (distancia o similaridad) y su rango. Resultado: doc del campo correcta. | `skills/vantadb-mcp/SKILL.md` (L121-123), `references/api-reference.md` | 🟢 | 🟡 | ✅ Documentado distancia real (1−cosine, orden ascendente) + nota SDK |
| `MCP-08` | **D4 `VantaError::DimensionMismatch` documentado pero no ocurre vía MCP** — Problema: `api-reference.md:272` lista la variante; el handler no valida dims (MCP-04). Acciones: tras fix MCP-04 verificar el error llega al cliente MCP (isError content); documentar en la skill qué error esperar. Resultado: doc del error correcta. | `skills/vantadb-mcp/SKILL.md` (search_semantic), `references/api-reference.md` (Error Handling) | 🟢 | 🟢 | ✅ Documentado isError content `"Vector dimension mismatch: expected X, got Y"` |
| `MCP-09` | **D5 Schema real vs doc: `search_semantic.k` required** — Problema: el `inputSchema` real marca `k` como required (verificado en tools/list) pero la skill dice "k (default: 5)" (SKILL.md:122); el handler tolera omisión (default aplicado), así que el schema es incorrecto/confuso. Acciones: (1) decidir si el schema debe marcar `k` opcional (fix server en tools.rs) o la skill debe decir "k requerido en schema, opcional en runtime (default 5)"; (2) alinear ambos. Resultado: schema y doc consistentes. | `vantadb-mcp/src/handlers/tools.rs` (schema search_semantic), `skills/vantadb-mcp/SKILL.md` (L122) | 🟢 | 🟢 | ✅ Doc-only: "k required in schema, optional at runtime (default 5)" |

### Bloque 3 — Deficiencias de la skill (cosas que no explica — vanta-docs)

> Problema actual: un usuario que integre contra el MCP server no puede descubrir la sintaxis IQL real, el envelope de respuestas, ni los comportamientos de borde — la skill documenta nombres/params/shapes pero no cómo usar el protocolo en la práctica. Estos fueron los mayores fricciones en las 4 baterías de prueba.

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `MCP-10` | **F1 Sintaxis IQL real NO documentada (la más grave)** — Problema: la skill dice "reading structures and inserting/mutating Nodes" (L127-129) sin UN solo ejemplo; la sintaxis real (verificada en `vantadb-mcp/tests/mcp_tests.rs:462` + parser) es: `INSERT NODE#id TYPE X { campo: "val" }` (+ opcional `VECTOR [..]`), `UPDATE NODE#id SET campo = "val"`, `DELETE NODE#id`, `FROM NODE#id`, y **`RELATE NODE#a--"label"-->NODE#b [WEIGHT w]`** para crear edges — indescubrible sin el código. Acciones: añadir sección "IQL Syntax" en SKILL.md con la gramática exacta verificada y 1 ejemplo por statement (INSERT/FROM/UPDATE/DELETE/RELATE) + nota "LISP no soportado". Resultado: un usuario puede escribir queries IQL correctas sin leer el código fuente. | `skills/vantadb-mcp/SKILL.md` (Graph Operations), `references/api-reference.md` | 🟢 | 🔴 | ✅ Sección "IQL Syntax" (7 statements verificados + nota LISP) |
| `MCP-11` | **F2/F3/F6 Envelope MCP + wire-format + canales de error** — Problema: (F2) todas las respuestas reales son `{"content":[{"type":"text","text":"<json>"}]}` con `isError`, no el JSON directo que asumen los "Returns:" de la skill — parsear requiere adivinarlo (los 4 scripts de prueba tropezaron con esto); (F3) metadata: entrada JSON plano `{"priority":2}`, salida serde-tagged `{"priority":{"Int":2}}` — confunde al leer/escribir programáticamente; (F6) errores por canales mixtos: `rehydrate` con id inválido → JSON-RPC `-32602`, `get_node_neighbors` con nodo inexistente → `isError` content. Acciones: (1) documentar el envelope MCP y cómo extraer payload/isError (con ejemplo); (2) documentar el wire-format de `VantaValue` (entrada plana, salida tagged) en api-reference; (3) documentar ambos canales de error y cuándo ocurre cada uno. Resultado: integrar contra el MCP sin adivinar el protocolo. | `skills/vantadb-mcp/SKILL.md` (Returns de cada tool), `references/api-reference.md` (VantaValue), `references/mcp-protocol.md` | 🟡 | 🟠 | ✅ "Response Envelope" + "Error Channels" + wire-format VantaValue |
| `MCP-12` | **F4/F5/F7/F8/F9/F10/F11 Comportamientos de borde no documentados** — Problema: la skill no documenta (F4) `memory_get` not-found → `isError` content `"Record not found"` (no JSON-RPC error); (F5) cursor de `memory_list` es offset numérico (usar `next_cursor` como `cursor`); (F7) advertencias del parser IQL: `LINK` no existe (falla silenciosa — inserta sin edge), trailing garbage aceptado sin error, `FROM NODE#a,b` multi-id devuelve solo el primero, `FROM` de nodo borrado → `[]`; (F8) `get_node_neighbors` solo muestra edges salientes hacia nodos vivos (dangling omitidos en silencio); (F9) `read_axioms` devuelve 4 objetos `{id, name, description}` (shape exacto); (F10) `rehydrate` requiere nodos archivados por un summary previo — no alcanzable con tools MCP solas (una pasada simple da `recovered_count: 0`); (F11) `memory_put` devuelve campos extra `version`/`node_id`/`expires_at_ms` (solo en api-reference, no en SKILL.md). Acciones: añadir sección "Behavior Notes" con cada punto + 1 línea de ejemplo/evidencia. Resultado: la skill describe el comportamiento real de borde, no solo los casos felices. | `skills/vantadb-mcp/SKILL.md` (sección nueva), `references/api-reference.md` | 🟡 | 🟡 | ✅ Sección "Behavior Notes" (7 puntos con evidencia) |

### Bloque 4 — Referencias muertas / rotas en la skill (vanta-docs)

> Problema actual: la wave SKL eliminó `config-template.json` y `create-namespace.py` (tool inexistente) pero SKILL.md aún los referencia; `test-mcp.py` cambió su interfaz (binario por argv/`VANTADB_MCP_BIN`) sin documentarlo. Un lector sigue instrucciones que fallan.

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `MCP-13` | **R1/R2/R3 Enlaces y comandos muertos** — Problema: (R1) SKILL.md:56 referencia `assets/config-template.json` que ya NO existe (eliminado en wave SKL); (R2) SKILL.md:87-88 sección "Namespace Management" instruye `python scripts/create-namespace.py create|list` — script eliminado (la tool no existe; la realidad es usar `memory_put` para crear el namespace implícitamente o `collection_list` para verlos); (R3) SKILL.md:79 `python scripts/test-mcp.py` — el script ahora requiere binario vía argv[1] o env `VANTADB_MCP_BIN` (funciona sin args solo si `vanta-cli` está en PATH). Acciones: (1) borrar L56 y la sección "Namespace Management" (o reemplazarla con el equivalente real); (2) documentar la interfaz actual de `test-mcp.py`. Resultado: toda instrucción de la skill ejecuta sin archivos inexistentes. | `skills/vantadb-mcp/SKILL.md` (L53-56, L74-89) | 🟢 | 🟠 | ✅ Referencias muertas eliminadas + Namespace real + interfaz test-mcp.py |

### Bloque 5 — Inconsistencias internas de la skill (vanta-docs)

> Problema actual: la skill se contradice a sí misma (SKILL.md vs `references/configuration.md`) y usa un ejemplo de config OpenCode que no funciona en la práctica (OpenCode no expande `~` en spawn directo — verificado en sesión 2026-08-17: `opencode.jsonc` usa ruta absoluta `C:/Users/Eros/.vantadb`).

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `MCP-14` | **I1/I2/I3/I4 Contradicciones internas** — Problema: (I1) Quick Start L20 dice que setup "creates default configuration" pero NO existe `config.json` ni config file (`configuration.md:3` — "there is no config.json"); (I2) Performance Optimization L236 sugiere "Adjust HNSW parameters" pero HNSW no se expone vía env vars (`configuration.md:155` — solo programático); (I3) Security L242 sugiere "read-only mode" pero NO existe `VANTADB_READ_ONLY` (`configuration.md:167` — solo programático vía SDK); (I4) ejemplo OpenCode L67 usa `~/.vantadb` pero OpenCode NO expande `~` en spawn directo (usar ruta absoluta o relativa al cwd). Acciones: corregir cada punto (borrar claim de config file, aclarar que HNSW/read-only son programáticos, usar ruta absoluta en el ejemplo OpenCode con nota). Resultado: la skill es internamente consistente y sus ejemplos funcionan. | `skills/vantadb-mcp/SKILL.md` (L20, L60-72, L236, L242), `references/configuration.md` | 🟢 | 🟡 | ✅ I1-I4 corregidos (config/HNSW/read-only programáticos, ruta absoluta OpenCode) |

---

## P23 - VantaDB Pro — Backlog de features (Open Core, 2026-08-17)

> **Origen:** `docs/strategy/VANTADB-PRO-FEATURES.md` § "Backlog Pro" + verificación multi-agente 2026-08-17. **Meta-modelo implementado** (repo privado `vantadb-pro` existe con `license.rs::verify_string` + `generate-license.ps1` + 4 tests; workspace aislado `Cargo.toml:620-626`; core sin refs Pro — D4 respetado). **Las 6 features NO tienen código** (solo `lib.rs`+`license.rs`) **ni tracking** — esta sección las trackea. Nacen en `vantadb-pro` (repo privado, fuera del workspace). Proceso: D5 entrega manual Enterprise hasta entidad.

| ID | Descripción (Feature Pro sugerida → qué clava) | Código actual | Esfuerzo | Prio | Estado |
|----|-------------|----------|------|--------|--------|
| `PRO-01` | **Multi-tenancy / RBAC** — aislamiento cifras org | `vantadb-pro`: solo `lib.rs`+`license.rs` | 🔴 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-02` | **Replicación multi-copy / Sync** — DR | ídem | 🔴 3-4 sem | 🔵 | ❌ Sin código |
| `PRO-03` | **WAL shipping + PITR (gates ya existen en core)** — failover | gates `wal-shipping`/`pitr` en core (`src/lib.rs:138,142`) | 🟠 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-04` | **TTL / retention policies** — compliance | ídem | 🟡 1-2 sem | 🔵 | ❌ Sin código |
| `PRO-05` | **Admin server + dashboard** — UX enterprise | ídem | 🟠 2-3 sem | 🔵 | ❌ Sin código |
| `PRO-06` | **Audit trail / compliance** — ídem | ídem | 🟡 1-2 sem | 🔵 | ❌ Sin código |

---

## P24 - I+D futura (v3.0+, re-verificada 2026-08-17)

> **Origen:** `docs/backlog-futuro.md` — catálogo de I+D diferido (freeze R5). Re-verificación multi-agente 2026-08-17: **FUT-01 ✅ IMPLEMENTADO (sacado)** — RaBitQ 1-bit + Hamming ya existe (`quantization.rs:16,33-46`, `Binary(Box<[u64]>)`); **FUT-07/FUT-09 redefinidos** (bloques base existen, falta cableado); **FUT-02/03/04/05/06/08/10/11 siguen sin implementar**. Documento fuente es la versión canónica de cada fila.

| ID | Descripción | Esfuerzo | Prio | Estado |
|----|-------------|----------|------|--------|
| `FUT-02` | **Embeddings Matryoshka** — truncamiento dinámico de dimensionalidad (1536→256) | 🔴 | 🗺️ | ❌ Sin implementar |
| `FUT-03` | **Community detection Leiden/Louvain nativa** — `docs/graphrag/README.md:302` sigue vigente | 🔴 | 🗺️ | ❌ Sin implementar |
| `FUT-04` | **Índices aprendidos (RMI)** para metadatos escalares | 🔴 | 🗺️ | ❌ Sin implementar |
| `FUT-05` | **Corrección de residuos QJL (Fase 3 TurboQuant/RaBitQ)** — `turbo_quant_*` es PolarQuant 4-bit puro | 🟠 | 🗺️ | ❌ Sin implementar |
| `FUT-06` | **Aceleración LUT / Bit-Slicing** para distancias cuantizadas | 🟠 | 🗺️ | ❌ Sin implementar |
| `FUT-07` | **Selector adaptativo de precisión por tier** — bloques existen (`VectorRepresentations` + tiers), falta selector automático; `consolidate_node_inner` no cambia representación | 🟠 | 🗺️ | 🟡 Redefinido (cableado) |
| `FUT-08` | **Go SDK vía C-ABI + cbindgen + cgo** — no existe capa C-ABI pública (solo `sigbus_handler` en `vfile_mmap.rs:206`) | 🔴 | 🗺️ | ❌ Sin implementar |
| `FUT-09` | **Curación AUDN en ingesta** — `DuplicatePreventionFilter` (Bloom) existe SIN callers en write path; bucle semántico AUDN ausente | 🟠 | 🗺️ | 🟡 Redefinido (cablear primitiva) |
| `FUT-10` | **Fuerza de retención Ebbinghaus / repetición espaciada** — solo `BayesianDecay` de eviction | 🟠 | 🗺️ | ❌ Sin implementar |
| `FUT-11` | **Export bidireccional a Markdown legible** — hoy JSONL machine-readable; bajo valor | 🟢 | 🗺️ | ❌ Sin implementar |



