---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-09-02
verified_by: "Historial de verificación: docs/avance/historial/backlog-history.md"
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks — organized by execution order.
> **Execution state lives in:** `docs/plans/YYYY-MM-DD-<campaign>.md` (plan file) + task files — per campaign-executor RULES.md §2. This file is the task catalog; the plan file is the execution state.
> **Completed tasks moved to:** `docs/avance/` (dominio) + `docs/avance/historial/backlog-history.md`
> **Historial de syncs y migraciones:** `docs/avance/historial/backlog-history.md` (último sweep mayor: 2026-08-26 — P37 DAUD-01..09 → historial vía DESKTOP-QW5; previo 2026-08-25 — limpieza P35/P38/P39 + auditoría docs/research)
> **Total open items:** 123 activas (post-paso0-plan 2026-09-03: -3 purgadas con evidencia - FIND-22 (cerrada 3b1b820b, fila stale), PY-02 (BENCHMARKS §2 SDK Python con comando reproducible ya existe), FIND-51 (premature: handlers.rs 1469L < umbral 2500L); ver plan 2026-09-03-quality-gtm-wave; previo 126)
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
| **P12** 🖥️ DESKTOP App (Tauri) + Consola Admin | 0 — ✅ **Cerrada 2026-08-24** (campaña DESKTOP-23..39: 17/17 ejecutadas, ver `docs/avance/activo/desktop.md`; ADMIN-01..09 + DESKTOP-20 ✅; resto archivadas por P26). Deuda menor: smoke-test instalador en VM limpia + validación manual dashboard proxy con upstream LLM | — | ✅ Cerrada |
| **P13** 🔎 AUDREP — Audit Report 2025 | 0 | — | ✅ Cerrado |
| **P14** 🔍 REVIEW items | 0 | — | ✅ Cerrada |
| **P15** 🔍 ERR items (revisión multi-agente 2026-08-08) | 0 — ✅ todos resueltos (36 por plan 2026-08-09 + 10 migradas 2026-08-12 + ERR-007 ban-skip 2026-08-12) | — | ✅ Cerrada |
| **P16** 🧩 Completitud de Features (investigación 2026-08-09) | 1 residual (CI-01) | 📆 Backlog | 🟢 Baja (19 ejecutadas 2026-08-09 + PERF-07/08/09 2026-08-12) |
| **P23** 🔒 VantaDB Pro (Open Core) | 6 (PRO-01..06) | ~8-12 semanas | 🔵 Futuro |
| **P24** 🧪 I+D futura (v3.0+) | 13 (FUT-02..14) | 📆 Futuro | 🗺️ Roadmap |
| **P25** 🔌 Exposición MCP/HTTP | 11 (MCP-16..26) | ~2-3 semanas | 🟡 Media |
| **P26** 🖥️ Vanta Studio (consola human-facing desktop) | Fase 0 ✅ (14/19) + Fase 1 ✅ (9/9) + Fase 2 ✅ (10/10: VS-CORE-04/05/06 + GRAFO-01..03 + ESPACIO-01..02 + OP-01..02) + Fase 3 ✅ (7/7: WEB-00..06) + **Fase 4 ✅ (18/18: DOC-01..04 + REST-01..06 + WASM-01..04 + FEAT-01..03 + VER-01)** | Planes archivados `docs/plans/archive/2026-08-18-vanta-studio-fase{1,2,3}.md` + `docs/plans/archive/2026-08-19-vanta-studio-fase4.md` | 🟢 Completada 2026-08-20 (ADR-027; E2E server + standalone WASM/OPFS PASS) |
| **P27** 🧠 Vanta Memory Engine (TDAM, orden F1–F7) | 38 (MEM-01..38) | ~8-12 semanas | 🔴 Alta (decisión de producto) |
| **GOV** 📋 Gobernanza Documental (post-auditoría 2026-08-21, plan `docs/plans/2026-08-22-doc-governance-plan.md`) | 30 tareas (T0: TIR ×3 · A: medición ×5 · B: Show-HN ×6 · C: maestros ×7 · D: taxonomía ×6 · E: limpieza ×1 · F: auditoría intocadas ×2) | ~6 días | 🔴 Alta (Wave B bloqueante Show HN; decisiones D1-D14 del owner en `docs/reviews/auditoria-documentacion-2026-08-21.md`) |
| **P36** 🔧 Auditoría AGENTS.md & sistema de agentes (2026-08-24) | 6 (AGT-01..06; 3 fixes ya aplicados en sesión) | ~1 día | 🟠 Media |
| **P37** 🎨 Auditoría diseño desktop post-fix (2026-08-24, orquestador + 5 sub-agentes) | 0 — ✅ 9/9 ejecutadas (DAUD-01..09 — commits `3c53d8b2`,`480935a7`,`b865c625`; DAUD-02 via DESKTOP-QW4 `ad0f34b1`) | — | ✅ Cerrada 2026-08-26 |
| **P38** 🔬 Research huérfanas → tarea (auditoría docs/research, 2026-08-25) | 16 (RES-01..15 + DEC-01/02; cada fila validada contra código con evidencia. RES-09 → FUT-12/13/14 en P24, 2026-09-03) | ~2-3 semanas (RES-01 es la más grande) | 🟡 Media (RES-01/RES-02 🔴 calidad/durabilidad) |
| **P48** 🧪 Testing & Benchmarking Hardening (auditoría multi-agente 2026-08-30, plan `docs/plans/archive/2026-08-30-testing-bench-harden.md`) | 0 (cerrada 2026-08-31: 22 ✅ + 1 🟡 INCOMPLETO `TBH-06`; resumen en sección "P48 — CIERRE") | ~2-3 semanas | 🔴 **Cerrada 2026-08-31** |

> **Historial de items removidos/completados:** ver `docs/progreso/BACKLOG_HISTORY.md`.
> **Nuevo 2026-08-04:** Fase 12 DESKTOP (26 tareas, app Tauri multi-connection sobre las 6 integraciones) + `DEBT-01` (gate docs-coverage roto, Fase 4) + `TECH-01..08` (hallazgos de investigación DESKTOP-01b: 2 bugs reales, 1 batch stale-docs, 1 ADR env-naming, 4 features/decisiones, todos en Phase 4).
> **Nuevo 2026-08-07:** Fase 7 ADMIN (ADMIN-01..09, consola administrativa centralizada: datos/métricas/KPIs/SOPs/telemetría/procesos/conexiones) sobre la infra DESKTOP-02..11 ya completada. Fuente de KPIs/SOPs: investigación de mercado (Grafana VectorDB Observability, Milvus, Qdrant, Weaviate, Zilliz/VectorDBBench) + snapshot de métricas ya existente en core (`src/metrics/core/snapshot.rs`, 112 líneas). Tareas DESKTOP-12..27 (drivers/MCP/empaquetado) se ejecutan después del core del dashboard.
> **Nuevo 2026-08-08:** P15 ERR — 50 hallazgos de la revisión multi-agente por capas (6 sub-agentes: vanta-audit/arch/engine/worker/tuner/docs + verificación manual). Origen completo en `docs/reviews/errors-found.md`. Top 3 a atacar primero: ERR-010 (raza checkpoint/persistencia), ERR-021 (OOM MCP), ERR-022 (top_k sin clamp → alloc gigante).
> **Nuevo 2026-08-09:** Fase 3 reabierta con COV-001..004 — medición de coverage multi-agente (3 sub-agentes 2026-08-09): Rust root 81.40% ✅ gate / TS 0% medible (bloqueado import-time por `vite-plugin-wasm`↔`vitest`) / Python wrapper 69% (core PyO3 invisible). Detalle en `docs/reviews/coverage-2026-08-09.md` (pendiente de generar). COV-004 es decisión de política ADR, no código.
> **Nuevo 2026-08-09:** P16 Completitud de Features + P0/P1 reabiertos — investigación multi-agente (5 sub-agentes: audit/worker/docs×2/tuner) que barrió features declaradas → parciales/huérfanas (PITR/WAL-shipping standalone-vacío, DiskANN "in-memory", Arrow 1-comp, IVF/SCANN sin SDK, integrations stubs), releases (semver-checks, publish 0.5.0, artifactos de ejecución), seguridad (UAF `__array_interface__`), rendimiento (benchmarks publicados no confiables) y docs (CHANGELOG muerto / llms.txt inventado / mojibake). Origen completo: `docs/research/investigacion-equipo-2026-08-09.md`. **Top 3 a atacar primero:** RELEASE-01 (semver-checks), SEC-01 (UAF), PERF-01 (sellar benchmarks).
> **Nuevo 2026-08-09 (audit-reports archive):** 4 reportes `audit-full-2026-07-*`/`08-04` archivados tras verificación contra código. Resueltos desde entonces: archive.rs fsync (AUDREP-04/35) ✅, deny.toml RUSTSEC-2024-0436 ✅, tests.rs god file dividido ✅, `.playwright-cli` → .gitignore ✅, JSON-LD en web ✅, prefers-reduced-motion ✅, metrics registry test ✅, pyi stubs ✅. Hallazgos pendientes incorporados como tareas nuevas: `PERF-07` (sparse JSON hot path), `PERF-08` (WASM serialización completa), `REVIEW-05` (god files restantes), `CI-01` (pre-commit-config). C1 UAF en `types.rs:365-380` (VantaSearchHit) sigue vigente → cubierto por `SEC-01`.
> **Nuevo 2026-08-09 (batch 2 audit-reports archive):** 4 reportes más archivados (audit-full-20260808, deps-01, inv-001, inv-024). Resueltos/verificados: AUD-012..015 (clippy 5 errores, tests INV-024, prune canonical, cap over-capacity) ✅ commit `9d3c05a2`; INV-024 H-1 (panic sq8 dims) ✅ NV-01 clamp; INV-024 M-1 (alineación `vector_offset`) ✅ `vfile.rs:739` central guard; inv-001 RUSTSEC sin acción ✅; deps-01 duplicación legítima trackeada en `ERR-007` ✅. Pendientes ya en backlog: AUD-016..021 (sección Hallazgos pendientes), `ERR-009` (Miri), `SEC-01`/`AUD-019` (__array_interface__). Único gap → `PERF-09` creado (cold-start "zero-copy" engañoso, `_force_copy` muerto).
> **Nuevo 2026-08-09 (batch 3 audit-reports archive):** `audit-full-2025-07-27.md` (vantadb-audit-report, auditoría multi-agente sobre `develop@63b0101d`) archivado → `docs/audit-reports/archive/`. Reporte íntegramente procesado: fue la fuente de **P13 AUDREP-01..62 + DEPS-01 + NV-01..05** (verificado 2026-08-05, todos resueltos 2026-08-05..08, commits en `docs/avance/historial/snapshot-2026-08-07.md`). 6 hallazgos fueron corregidos antes del ticketeo (CRIT-01, CRIT-06, CRIT-09, ALTO-01, CRIT-10-prometheus, MED-15) y los residuales vivos ya están recapturados como tareas activas (SEC-01, ERR-021, PERF-07/08/09, CI-01, REVIEW-05). **No se crean tareas nuevas — archivo cerrado.**
> **Nuevo 2026-08-24 (auditoría agentes):** **P36** creada (6 tareas AGT-01..06) desde revisión integral de AGENTS.md raíz + .opencode/AGENTS.md + global: fixes ya aplicados (root pointer-only, metadata stale, conteos skills), pendientes commit de diffs, verificación stats CodeGraph, refs file:line de deuda P2, limpieza opencode-loop corrupt/tmp, convención checkpoints paralelos, script anti-drift de refs.
> **Nuevo 2026-08-30 (auditoría testing & benchmarking):** **P48** creada (23 tareas TBH-01..23) desde auditoría multi-agente (5 sub-agentes: research externas + tests Rust + benchmarks + datasets + CI/CD) ejecutada 2026-08-30, sesión `ses_fabf69692ffeP5c7mycKcsGSV0`. Plan completo en `docs/plans/2026-08-30-testing-bench-harden.md`. Decisiones del owner (D1-D7): estrategia conservadora (no VIBE/PQ/head-to-head/divan), scope = TODOS los 23 fixes, ci-gate universal (eliminar `if: schedule`), TS SDK diferido a `ISSUE-TS-001`. Distribución: **5 ALTA** (rompen 'replicable y funcional' — `verify_datasets.{sh,ps1}`, init bench baseline, fix ci-gate.yml:24, branches develop×6+release, gitignore data_*_bench/) + **10 MEDIA** (insta snapshots, cargo-mutants, wal_throughput.rs, crash_recovery.rs, convert bench_concurrent, bench-nightly cobertura 8 benches, data/README.md + datasets/README.md, SHA-pins faltantes, cliff.toml conventional_commits, audit-tokens consolidación) + **8 BAJA** (divan/loom/dhat eval, markdownlint pre-commit, ci-examples matrix, coverage threshold policy, release-binaries-tags trigger, fmt scope unify). Top 3 a atacar primero: **TBH-01** (verify_datasets — unmask silent skips), **TBH-02** (bench baseline — activar regresión nocturna), **TBH-03** (ci-gate — main rojo corta PRs).

---

## ✅ Definition of Ready / Done

> **DoR + DoD del proyecto (VantaDB-specific) viven en:** `.opencode/references/definition-of-done.md` — secciones "VantaDB — Definition of Ready" y "VantaDB — Project-specific DoD commands". Referencia única; no duplicar aquí.

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
| `MKT-18i` | **AnythingLLM ↔ VantaDB (re-escalado 2026-09-03)** — compose demo Ollama+VantaDB shipped (`abb6594c`, ver `docs/avance/activo/operaciones.md`). Restante: AnythingLLM no soporta VantaDB como vector backend (evidencia: `server/.env.example` master de Mintplex-Labs/anything-llm — `VECTOR_DB` acepta lancedb/chroma/pgvector/qdrant/pinecone/astra/weaviate/milvus/zilliz/chromacloud). Requiere feature-request upstream (acción humana), no glue local. | ⚪ upstream | 🟡 | 🔴 BLOQUEADO |
| `CLD-01` | **VantaDB Cloud beta on Fly.io** — checkbox vacío en `GO_TO_MARKET.md:420`; cero archivos de infra. Verificado 2026-08-17: no existe nada. | 🟠 1-2 sem | 🔵 | ❌ Pendiente |
| `CLD-02` | **Pitch deck + one-pager** — checkbox vacío en `GO_TO_MARKET.md:408`; cero archivos `*pitch*`/`*deck*`. | 🟡 3-5d | 🔵 | ❌ Pendiente |
| `CLD-04` | **Case study #1 (enterprise pilot)** — checkbox vacío en `GO_TO_MARKET.md:409`; cero archivos. Depende de pilot real. | 🟠 1 sem | 🔵 | ❌ Pendiente |
| `BLOG-CTA` | **CTAs + metadata de la serie de blogs + posts 6-7** — M3 🟡 date drift (2026-06-06 vs web 2025), M4 🟡 title drift, CTA débil en 2 posts (`how_hybrid_search_works.md`, `sqlite_for_ai_agents.md`), posts 6-7 no redactados (Ollama+VantaDB, Claude Code MCP). M1/M2/M5/M6 ya resueltos. | 🟡 3-5d | 🟠 | ❌ Pendiente |

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

## P14 — REVIEW items (hallazgos de `docs/reviews/review-full-2026-07-27-0309.md`, validado 2026-08-05)

> **Origen:** findings de la unified-review full 2026-07-27, re-validados contra el código el 2026-08-05 (sub-agentes vanta-worker/vanta-docs). Los items marcados como corregidos en el report y confirmados se excluyen. `DEPS-01` ya cubre la duplicación de crates/lru; `AUDREP-41` cubre next-auth dead dep — no duplicados aquí.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

---

## Phase 16: 🧩 Completitud de Features & Docs (de investigación multi-agente 2026-08-09)

> Origen completo: `docs/research/investigacion-equipo-2026-08-09.md`. **Ejecutada 2026-08-09 por plan `docs/plans/archive/2026-08-09-backlog-pipeline.md` — 19/19 tareas ✅** (FEAT-01..07, REVISAR-01, COV-001/003/004, PERF-01/04/06, DOC-02..08; RELEASE-01/02/03 + SEC-01 en Wave 0). Commit por feature en `docs/progreso/README.md`. Residuales viven en sus secciones: `CI-01` (hallazgos pendientes/CI), `REVIEW-05` (P14).

---

## Phase 11: 🧠 Embeddings Local-First — 0 — ✅ 9/9 COMPLETADA 2026-08-28

> **Cerrada 2026-08-28 — 9/9 EMB-01..09** (commits `2c185021`→`d24eeb1c`, plan `docs/plans/2026-08-28-embeddings-local.md`): `embeddings/` + `embed-local` (ort+tokenizers) + `vanta-memory` L1 + MCP `embed_texts` + SQL auto-embed + bench + docs + Qwen3 excepción. Registro en `docs/avance/activo/core-engine.md` §Phase11 + `backlog-history.md`. Plan archivado `docs/plans/archive/2026-08-28-embeddings-local.md`.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| — | — | *Sin tareas activas — 9/9 EMB archivadas (ver historial)* | — | ✅ |

---

## Referencias Cruzadas

- **RC items:** PROJECT_FULL_REVIEW_2026-07-13 (ARCHIVADO - generado por vantadb-full-review; ver docs/reviews/)
> **Nota GOV-C3 2026-08-22:** los reportes citados en esta seccion (audit-reports/*, REPORTE_EVALUACION_COMPLETO.md, FULL_CODEBASE_AUDIT_2026-07-11, PROJECT_FULL_REVIEW_2026-07-13) fueron disueltos/archivados; contenido procesado en docs/reviews/ y BACKLOG_HISTORY.md. Rutas = referencia historica.
- **REV items:** `docs/reviews/2026-07-13-full-review.md`
- **DRV findings:** `docs/plans/2026-07-15-cross-ref-docs-vs-code.md` + `docs/audit-reports/cross-ref-wave3-final-report.md`
- **OLD items:** `docs/REPORTE_EVALUACION_COMPLETO.md` secciones 6 y 7 — ~280 archivos VANTADB DOC OLD analizados
- **COMP items:** `docs/audit-reports/competitive-features-consolidated-report.md` + `docs/audit-reports/deep-analysis-{vector,graph,arch}.md` — 27 archivos, 172 features, top 30 priorizados

---



## Hallazgos pendientes de reportes

Hallazgos >= medium derivados de reportes de auditoría. Fuente: `docs/reviews/audit-full-20260812-231204.md` (2026-08-12).

| ID | Severidad | Hallazgo | Archivo:línea | Esfuerzo | Prioridad | Estado |
|----|-----------|----------|---------------|----------|-----------|--------|
| AUD-042 | Media | Upgrade tantivy ≥0.18 — elimina allowlist RUSTSEC-2026-0253 + desbloquea lru 0.18 (security debt, rec#6) | Cargo.toml (tantivy), deny.toml | 🟡 | 🟡 Media | 🔴 BLOQUEADO upstream — verificado 2026-08-13: tantivy 0.26.1 (última publicada) fija `lru ^0.16.3`; el fix (`lru = "0.18.2"`) está en tantivy main (0.27.0) pero NO publicada en crates.io (404). Re-evaluar cuando tantivy ≥0.27.0 publique: bump tantivy + lru directo a 0.18.2 y remover allowlist. Comentario deny.toml actualizado con el estado. |
| AUD-045 | Media | Clones de vector completo per-candidate en hot path IVF search (`centroid.clone()` y `entry.vector.clone()` en loops internos). Acción: medir baseline vs `canonical_p99` (Regla 9), luego A/B variante borrowed/slice en `calculate_similarity`; aceptar solo si p99 mejora. Origen: audit-full-20260825-031011 Phase 3 | `src/index/ivf.rs:250,275` | 🟡 | 🟡 Media | Pendiente |
| REVIEW-10 | Alta | God-file `cli_server.rs` ~3800-4141 líneas (routing + RBAC + TLS + OTEL + tests inline) — blast radius total del server en un archivo. Split por concern bajo `src/server/`; congelar features nuevas ahí | src/cli_server.rs | 🟠 | 🔴 Alta | 🟠 Abierta — derivada de review-full-20260822 H06-ARCH-001 |
| BND-08 | 🔴 P0 | **Pipeline npm release napi-rs** (H-01): nunca publicado (E404); crear workflow CI create-npm-dirs/artifacts/prepublish modelo LanceDB/napi.rs, 5 targets + musl futuro. Origen: research-vantadb-node-20260825 | .github/workflows/, antadb-node/package.json | 🔴 | 🔴 Alta | Pendiente |
| BND-09 | 🟡 | **Target linux musl** (H-04): Docker/Alpine sin cobertura; agregar aarch64/x86_64-unknown-linux-musl a napi.targets cuando exista pipeline. Origen: research-vantadb-node-20260825 | antadb-node/package.json, CI | 🟢 | 🟠 Media | Pendiente |
| BND-12 | 🟡 | **Cobertura tests node** (H-06): 8→~20 tests; cubrir search/explain_search/put_batch/capabilities/close-drain. Origen: research-vantadb-node-20260825 | antadb-node/tests/ | 🟡 | 🟠 Media | Pendiente |
| BND-13 | 🟡 | **docs/api/NODE_SDK.md** (H-07): doc completa + ejemplos por runtime; README ya creado (quickstart + matriz native-vs-wasm). Origen: research-vantadb-node-20260825 | docs/api/NODE_SDK.md | 🟢 | 🟡 Media | Pendiente |
| PERF-BENCH-01 | 🟡 | **Benchmark A/B vantadb-node nativo vs vantadb-ts WASM** (H-09): insert/search p99 + tamaño binario; decide posicionamiento (decisión tomada: native primario en Node condicionado a números). Regla 9. Origen: research-vantadb-node-20260825 | benches nuevos | 🟡 | 🟠 Media | Pendiente |
| MCP-34b | Media | Tool MCP `snapshot_restore(name)` — wrapper del restore físico cuando S2-S4 de RES-02 aterricen (validación identifier + confirmación destructiva explícita). Prerrequisito: S1 (quiesce+flush en create_snapshot) y S2-S4 (core restore + tests) | docs/research/res02-backup-restore.md §3 | 🟢 | 🟡 Media | ⬜ Pendiente (prerrequisito core nuevo) |
| ~~FIND-26~~ | Baja | ✅ RESUELTA (remove, 2026-08-25): `src/wal_archiver.rs` eliminado + export/feature `pitr` removidos + docs actualizados (FEATURES.md, EXPERIMENTAL_FEATURES.md, ADR-014 superseded). Decisión del lead basada en RES-02 §2b: PITR necesita base snapshot + replay (prerrequisito grande sin consumer); código conservado en git history (`git log --follow src/wal_archiver.rs`) | research res02-backup-restore.md §2b · ADR-014 | 🟠 | 🟡 Media | Completada |

| FIND-47 | Baja | `handle_tools_call` complejidad 295 (dispatcher MCP): match gigante 20+ brazos, 8 scans en loops batch — no hotspot algorítmico; si crece, extraer sub-dispatchers. Origen: codegraph-20260827-143245 Fase 5 | vantadb-mcp/src/handlers/tools.rs:549 | 🟢 | 🟢 Baja | Pendiente |
| FIND-48 | Alta | Split `src/index/graph.rs` 1846L — god-file HNSW: extraer `graph/hnsw.rs`, `graph/search.rs`, `graph/serialize.rs` por concern (verificado 2026-09-02: 75372 bytes, mayor archivo src) | `src/index/graph.rs` | 🟠 2-3d | 🟡 Media | ⬜ Pendiente |
| FIND-49 | Media | Split `src/sdk/types.rs` 1699L — tipos SDK monolíticos: extraer `types/record.rs`, `types/search.rs`, `types/graph.rs` (verificado 2026-09-02: 63035 bytes) | `src/sdk/types.rs` | 🟡 1-2d | 🟡 Media | ⬜ Pendiente |
| FIND-50 | Media | Split `src/parser/mod.rs` 1682L — parser IQL monolítico: extraer `parser/grammar.rs`, `parser/lexer.rs` por concern (verificado 2026-09-02: 64546 bytes) | `src/parser/mod.rs` | 🟡 1-2d | 🟡 Media | ⬜ Pendiente |
| FIND-53 | Baja | Doc drift: tool MCP `embed_texts` (EMB-05, commit `241b0868`) sin sección en `docs/api/MCP.md` — `scripts/validate-docs-coverage.ps1` reporta 1/49 gaps (2026-09-03, hallazgo colateral GOV-TK9) | `docs/api/MCP.md` | 🟢 | 🟢 Baja | ⬜ Pendiente |
| FIND-56 | Alta | **``vantadb-server/Dockerfile`` roto: COPY de crate inexistente (origen: verificación SRV-07 2026-09-03):** el builder hace ``COPY vantadb/Cargo.toml vantadb/`` pero el crate raíz vive en ``.`` del workspace — no existe ``vantadb/``, el build muere ahí. Duro: ``docs/operations/hardening.md:199-212`` lo documenta como FUNCIONAL (incluido un ``--target unprivileged`` inalcanzable). Fix: reescribir builder sobre el layout real (patrón SRV-07: ``COPY . .`` + cache mounts, o splicing correcto), o deprecable: fusionar con el Dockerfile raíz (que ya build-ea con USER no-root + ARG VANTA_RUNAS_UID) y apuntar hardening.md a él. Contrato sugerido: docker build de ambas variantes exit 0, o archivo eliminado con docs actualizados | ``vantadb-server/Dockerfile``, ``vantadb-server/docker-compose.yml``, ``docs/operations/hardening.md`` | 🟢 1-2h | 🔴 Alta (doc miente sobre artefacto de release) | ⬜ Pendiente |


---

## P17 - Mejoras del task-system (de REPORTE-FINAL 2026-08-10 §3.3/§3.5)

> Items §3.3 (FALTA, cobertura 0-30%) y §3.5 (MAL/CONTRADICTORIO) de `docs/research/2026-08-10-agent-engineering/REPORTE-FINAL.md` no asignados a los planes P0-P3 (Task 14 de `docs/plans/archive/2026-08-10-docs-task-system-consolidation.md`). **TSYS-01..05, 07..11, 13..16 implementados 2026-08-11 → migrados a progreso** (plan `docs/plans/archive/2026-08-11-residuo-consolidado.md`). Pendientes: TSYS-06 (runner DEFER) y TSYS-12 (runtime opcional, NO gate-CI). Los de runtime con datos dependen de Task 2 (poblar `verify-log.jsonl`).

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

---

## P18 - Gaps residuales del task-system (re-verificados 2026-08-12, para investigación y decisión)

> Re-verificación post-auditoría de `docs/research/2026-08-10-agent-engineering/` (10 sub-agentes, reporte consolidado en `docs/reviews/audit-agent-engineering-2026-08-11.md`). Los 48 ❌ brutos se redujeron a **8 gaps reales** tras descartar falsos negativos (ADR gate, DMAIC, 2-sombreros, merge-a-main-DoD), DEFER con fecha (P3-2 calibración, P3-3/P3-9 mutation) y cosméticos. Los 7 gaps TIR (investigados 2026-08-17, docs en `docs/research/TIR-*.md`) tienen decisión registrada en la columna Estado; NO son tareas de implementación directa. **Follow-ups pendientes de las decisiones:** (a) TIR-02 — implementar recovery time en `evals/dora.mjs` (~30 líneas); (b) TIR-04 — formalizar contenedor de tareas fallidas (`tasks/closed/` + regla re-procesamiento `pending` + índice `rg "❌ FAILED"`); (c) TIR-08 — criterios 1-2 en `research-agent.md` (~6 líneas).

| ID | Descripción | Origen (doc:línea) | Afecta | Esfuerzo | Prio | Estado |
|----|-------------|--------------------|--------|----------|------|--------|

---

## P19 - Mejoras del sistema de agentes (recomendaciones multi-agente 2026-08-14)

> Revisión de los 9 agentes en `.opencode/agents/` (2026-08-14): Output Templates, skills §6, bloques `permission:`, bloque §7 duplicado y delegación DISCOVERY. Evidencia con `file:línea` en cada fila. R1-R3/R7/R9 son los de mayor impacto; R4 es decisión registrada (NO hacer).

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|








---

## P21 - Skills de VantaDB (`skills/`) — ✅ Cerrada (2026-08-17)

> **CERRADA 2026-08-17:** 4/4 tareas ejecutadas. SKL-01/02/03 implementadas (docs + scripts), SKL-04 gate P2-01 emitido (CHANGES-REQUIRED con 1 falla de docs — "4 resources" vs 2 reales — fix aplicado por SKL-02 y re-verificado 4/4 exit 0). Verificado por el lead: `test-mcp.py` 4/4 exit 0 contra `vanta-cli.exe` v0.5.0 (15 tools, 2 resources, 4 prompts). Plan archivado: `docs/plans/archive/2026-08-17-skills-vantadb.md`. Task files: `.opencode/skills/campaign-executor/tasks/SKL-0{1,2,3,4}.md` (✅). Detalle en `docs/progreso/README.md`.

## P22 - Certificación del MCP server vs skill `vantadb-mcp` (2026-08-17)

> **Origen:** batería de pruebas exhaustiva (4 sub-agentes vanta-worker en paralelo, JSON-RPC 2.0 stdio contra `vanta-cli server --mcp --db <temp>`, binario 0.5.0 en PATH, DBs temporales aisladas, asserts por afirmación documentada de la skill). Resultados por área: **Memoria CRUD 17/17 ✅ · Collections 12/12 ✅ · Grafo/IQL 25/25 ✅ · Búsqueda 12/20 ❌ (8 FAIL = 4 bugs reales + discrepancias)**. Las skills son copia exacta (hash SAME) de las versionadas en `skills/vantadb-mcp/` (wave SKL). Los 4 bugs del server están trazados al código fuente (root causes abajo). Scripts de prueba (re-ejecutables, autocontenidos): `C:\Users\Eros\AppData\Local\Temp\opencode\test-{memoria,busqueda,grafo,collections}.py`. **Orden de ejecución:** Bloque 1 (server) primero → Bloque 2 (doc ya sincronizada a la realidad) → Bloques 3-5 en paralelo (vanta-docs).
> **⚠️ Regla de edición de skills (aplica a Bloques 2-5):** la fuente de verdad **versionada** es `skills/vantadb-mcp/` (commit `61381d29`, wave SKL) — editar SIEMPRE ahí. `.opencode/skills/vantadb-mcp/` es la **copia runtime sin versionar** que OpenCode usa para resolver `skill <nombre>` (lookup: `.opencode/skills/` primero) y las rutas relativas de SKILL.md (`../../docs/api/MCP.md` etc.) solo resuelven desde `skills/`. Después de editar, **copiar los archivos tocados a `.opencode/skills/vantadb-mcp/`** para que ambos lados queden idénticos (hash SAME) antes de cerrar la tarea.

### Bloque 1 — Bugs del server MCP (código, vanta-arch/vanta-worker) — BLOQUEANTES

> Problema actual: la skill declara disponibles features de búsqueda lexical/híbrida que **fallan siempre** vía MCP en DB fresca, y 3 discrepancias de comportamiento más. El MCP server (`vanta-cli server --mcp`) es la vía canónica para agentes (OpenCode/Claude/Cursor) — estos bugs rompen el contrato documentado. Delegar diseño del fix S1 a vanta-arch (dónde llamar `ensure_indexes_current`) y S2-S4 a vanta-worker.

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### Bloque 2 — Discrepancias skill ↔ realidad (docs, vanta-docs — DESPUÉS del Bloque 1)

> Problema actual: la skill `skills/vantadb-mcp/SKILL.md` documenta features como disponibles que el server no cumple (o al revés). Dependencia: ejecutar DESPUÉS de los fixes del Bloque 1 para documentar la realidad ya corregida; si un fix del Bloque 1 se difiere, marcar la feature como limitada en la skill (nunca dejar el claim falso).

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### Bloque 3 — Deficiencias de la skill (cosas que no explica — vanta-docs)

> Problema actual: un usuario que integre contra el MCP server no puede descubrir la sintaxis IQL real, el envelope de respuestas, ni los comportamientos de borde — la skill documenta nombres/params/shapes pero no cómo usar el protocolo en la práctica. Estos fueron los mayores fricciones en las 4 baterías de prueba.

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### Bloque 4 — Referencias muertas / rotas en la skill (vanta-docs)

> Problema actual: la wave SKL eliminó `config-template.json` y `create-namespace.py` (tool inexistente) pero SKILL.md aún los referencia; `test-mcp.py` cambió su interfaz (binario por argv/`VANTADB_MCP_BIN`) sin documentarlo. Un lector sigue instrucciones que fallan.

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

### Bloque 5 — Inconsistencias internas de la skill (vanta-docs)

> Problema actual: la skill se contradice a sí misma (SKILL.md vs `references/configuration.md`) y usa un ejemplo de config OpenCode que no funciona en la práctica (OpenCode no expande `~` en spawn directo — verificado en sesión 2026-08-17: `opencode.jsonc` usa ruta absoluta `C:/Users/Eros/.vantadb`).

| ID | Descripción (Problema actual → Acciones → Resultado a obtener) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

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

> **Origen:** `docs/backlog-futuro.md` — catálogo de I+D diferido (freeze R5). Re-verificación multi-agente 2026-08-17: **FUT-01 ✅ IMPLEMENTADO (sacado)** — RaBitQ 1-bit + Hamming ya existe (`quantization.rs:16,33-46`, `Binary(Box<[u64]>)`); **FUT-07/FUT-09 redefinidos** (bloques base existen, falta cableado); **FUT-02/03/04/05/06/08/10/11 siguen sin implementar**. Documento fuente es la versión canónica de cada fila. **Actualización 2026-09-03 (RES-09):** +FUT-12/13/14 del roadmap huérfano → `docs/research/archive/investigacion-equipo-2026-08-09.md` §roadmap.

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
| `FUT-12` | **WAL fsync-batching / flush asíncrono** — hoy `src/wal.rs`: Periodic con threshold default 1 = sync por escritura (`wal.rs:372`; default Periodic `config.rs:180`, plumbing `SyncMode` existe sin group-commit). NO es «async ingest» genérico: `src/ingestion.rs` ya da pipeline async de nodos. Ganancia esperada 10-100× en ingesta batch (§roadmap:184). Riesgo: durabilidad — requiere decisión explícita de política de sync (ventana de pérdida vs throughput) antes de implementar. Trackeado desde RES-09 (investigación 2026-08-09 §roadmap) | 🔴 | 🗺️ | ❌ Sin implementar |
| `FUT-13` | **Query planner con optimizaciones reales** — hoy router + heurística: clasifica la query Hybrid/TextOnly/VectorOnly (`src/planner.rs`; §roadmap:186 «hoy router + heurística»). Gap: optimizaciones más allá de la clasificación por tipo (estimación/costo para elegir camino de índices). Requiere ADR + benchmark Regla 9 antes de tocar. Trackeado desde RES-09 (investigación 2026-08-09 §roadmap) | 🔴 | 🗺️ | ❌ Sin implementar |
| `FUT-14` | **DiskANN con disk-I/O real** — nombre engañoso: `src/index/diskann.rs:7,13` explícito «purely in-memory, **not disk-backed**» (Vamana en RAM; inv. l.93). Gap: page layout SSD (beam/sector reads) para datasets > RAM; mientras tanto exponerlo por SDK como in-memory (inv. l.72: IVF/SCANN/DiskANN sin exposición SDK). Trackeado desde RES-09 (investigación 2026-08-09 §roadmap) | 🟠 | 🗺️ | ❌ Sin implementar |

---

## P25 - Exposición MCP/HTTP — Gaps del SDK no expuestos (2026-08-18)

> **Origen:** gap analysis del lead (2026-08-18): comparación API pública core SDK (`src/sdk/api.rs`, `VantaEmbedded` — put:206, get:406, list:545) vs 15 tools MCP (`vantadb-mcp/src/handlers/tools.rs`) vs 3 endpoints HTTP (`src/cli_server.rs`: `/health`, `/api/v2/query`, `/metrics`). Conclusión: el SDK tiene ~35 métodos públicos; las herramientas MCP cubren CRUD+búsqueda+grafos-IQL, pero **faltan capacidades de ciclo de vida de datos** que el MCP no puede invocar: purge TTL (crítico: AUD-045 habilitó TTL pero no hay forma de limpiar expirados), backup/restore, mantenimiento WAL/layout, delete batch por filtro, batch put, rebuild/recovery de índices. El HTTP server es deliberadamente mínimo (producto embedded-first; "server as primary boundary" diferido en skill) — no es bug, es decisión; solo se trackea aquí el MCP.
> **Nota:** NINGUNA tool MCP de la lista requiere cambio de API pública del SDK — son wrappers `handle_tools_call` sobre métodos ya públicos. Riesgo bajo, sin semver implications.
> **Actualización 2026-08-19 (Fase 3, D11):** la decisión "server as primary boundary diferido" fue **re-considerada por el usuario (D11)** — el REST completo del SDK ya está implementado (`/api/v2/*` en `src/cli_server.rs`, ADR-026). El HTTP server ya NO es mínimo: ~27 endpoints v2 (health, records CRUD+batch+versions+delete_by_filter, list con cursor, search, autocomplete, query, audit, export/import, graph bfs/dfs/degree/pagerank/centrality, maintenance purge/compact/flush/rebuild-index, threads, snapshots) + dashboard embebido `/dashboard`. Los gaps MCP-16..26 de esta sección siguen válidos SOLO para el canal MCP (el REST no sustituye las tools del agente).

| ID | Descripción (Gap → Acciones → Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|

---

## P26 - Exposición capa cognitiva `vanta-memory` + agentic restante vía MCP (2026-08-23)

> **Origen:** auditoría de integración total OpenCode↔VantaDB (2026-08-23): el motor core quedó ~100% expuesto tras cerrar P25 (MCP-16..29), pero la capa cognitiva (`vanta-memory`: scenes, context engine, compresión MMD) y las APIs agénticas restantes (threads CRUD, axiom-write, snapshots create/restore) no tienen tool MCP. **Fuera de alcance por diseño (NO trackear aquí):** `vanta-proxy` (infraestructura proxy LLM), módulo LLM interno (`llm.rs`, feature-gated para ingesta), HTTP REST completo (embedded-first), desktop/web (UIs).
> **Nota:** los handlers del gateway ya existen como funciones puras sobre `&VantaEmbedded` (`vanta-memory/src/gateway/knowledge_handlers.rs`) — las filas MCP-30/31 son wrappers; MCP-32..34 exponen APIs SDK/core ya públicas.

| ID | Descripción (Gap → Acciones → Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `MCP-41` | **🟡 Memoria conversacional auto-consolidada** — la brecha competitiva vs mem0/graphiti/cognee (todos extraen y consolidan memoria de conversaciones vía LLM); nuestro hueco natural: hacerlo feature-gated reusando scenes (MEM-13/14) sin LLM key obligatoria. Acciones: DISCOVERY primero (vanta-arch) sobre diseño extract→consolidate→recall local-first. Ref: docs/reviews/mcp-research-20260825.md §7 apuestas | `vanta-memory/src/core/scene/`, `vantadb-mcp/src/` | 🔴 1-2s | 🟡 | ⬜ Pendiente (requiere DISCOVERY) |

> **No trackeado aquí** (deliberadamente): (a) threads CRUD directos (`get/list/delete_thread`, `purge_expired_threads` — `builder.rs:168-195`, `src/agentic/thread.rs`) — `inject_context` cubre el caso agente; (b) HTTP REST completo — diferido por diseño (embedded-first, `cli_server.rs` solo `/health` + `/api/v2/query` + `/metrics`); (c) `add_edge`/`get_node`/`delete_node` directos — alcanzables vía IQL (`RELATE`/`FROM`/`DELETE`).

---

## P26 - Vanta Studio — Consola human-facing desktop (Fase 0, 2026-08-18)

> **Origen:** investigación `docs/research/human-facing-db-ui/` (01-05) + corrección cognitiva (07) + síntesis `06-synthesis/SYNTHESIS.md` (concepto "Vanta Studio": workspace de memoria para ver/crear/editar/eliminar cualquier registro sin código). **Decisiones del usuario 2026-08-18** (ver plan `docs/plans/2026-08-18-vanta-studio-fase0.md`): Fase 0 completa; solo desktop; dirección visual Manga Tradicional & Grabado Linocut (tokens de `web/`: cream `#FBF9F5`, ink `#000`, neon `#FF5500`, paper `#F2EDE2`); Tailwind v4 + tokens web; tema ambos con default claro; grafo = renderer three.js propio (R3F) en Fase 2; prototipo Fase 0 core en HTML; mascota = **MARK** (variante desktop del personaje SVG del hero). **D2:** Historial+Diff en espera hasta que `VantaMemoryRecord` retenga versiones (VS-CORE-07). **Auditoría multi-agente 2026-08-18** (4 sub-agentes: vanta-research/review/arch/docs) → correcciones aplicadas: **VS-10 + VS-11 nuevos (críticos: bridge put + DTO enriquecido)**, dark palette diseñada propia (la web no tiene tokens dark), VS-06 + payload/CodeMirror, VS-04 actividad = updated recientes, VS-09 sin IQL, VS-CORE-01/03/06 re-scopeados (ya existen en core/bindings), `ink-corner` eliminado, DEFERs ampliados (favoritos, Copy-as, a11y chips, import CSV/JSON). **Se ejecuta por plan** `docs/plans/2026-08-18-vanta-studio-fase0.md`. **Relación con P27:** integración por contratos (no ejecución) — tabla "Relación con P27" en el plan; no bloqueante. **Fases 0-3 completadas 2026-08-18/19:** F0 14/19 (VS-00..11 + VS-CORE-01/02; 5 VS-CORE diferidas a F1/F2) · F1 9/9 (VS-CORE-03/07 + VS-12..18) · F2 10/10 (VS-CORE-04/05/06 + GRAFO-01..03 + ESPACIO-01..02 + OP-01..02) · F3 7/7 (WEB-00..06 — transporte pluggable, REST completo `/api/v2/*` + dashboard `/dashboard` servido por `vanta-cli server --dashboard-dir`; ADR-026). Planes en `docs/plans/archive/2026-08-18-vanta-studio-fase{1,2,3}.md`; registro en `docs/progreso/README.md` §Vanta Studio. **Fase 4 en ejecución 2026-08-19** (plan `docs/plans/2026-08-19-vanta-studio-fase4.md`): reconciliación documental + cierre deuda REST + WASM/OPFS + diferenciadores research.

| ID | Descripción (→ Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|


---

## P27 - Vanta Memory Engine — port de TDAM (orden F1–F7, 2026-08-18)

> **Origen:** investigación completa `docs/research/tdam/` (PLAN + 01..09 verificados + SYNTHESIS) — TencentDB-Agent-Memory (clon completo `97f9465`, v2.0.0-beta.1). **Decisión arquitectónica:** core VantaDB puro (sin LLM, WASM-compatible); features LLM-driven en crate nuevo `vanta-memory`; se extiende `vantadb-mcp`/`vantadb-server`; binario opcional `vanta-proxy`. **NO copiar** split 4 servicios, Redis, SQLite/dual-write, store Mongo, `@colbymchenry/codegraph`, agent-adapters, 3 imágenes Docker, prompts chinos Kenty. Análisis multi-agente (3× vanta-research 2026-08-18) → 38 tareas consolidadas (2 propuestas duplicadas fusionadas: telemetría). **Se ejecuta por plan** `docs/plans/2026-08-18-vanta-memory.md`. **Decisiones resueltas 2026-08-18 (D1-D12):** LLMRunner ambos sync+async · nodo escena entra · tiktoken o200k_base (validar WASM) · entity_* en InternalMetadata · vanta-proxy crate aparte · checker 7 eslabones · skills síncrona · MMD Mermaid literal · callback hook local + estado en store · **F6/F7 segunda iteración** · vanta-memory interno del workspace. **Relación con P26:** integración por contratos (no ejecución) — tabla "Relación con P26" en el plan; no bloqueante.

| ID | Descripción (→ Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|


## P28 - Deuda técnica core — follow-ups (2026-08-18)

### Wave 2 — Memoria agéntica competitiva (investigación 3 frentes, 2026-08-25)

> **Origen:** investigación profunda vanta-memory vs estado del arte (Mem0/Letta/Zep/Cognee/Memobase/OpenAI Dreaming/Anthropic Auto Dream) vs necesidades de usuarios de coding agents (Claude Code/OpenCode/Codex/Cursor/Antigravity/Windsurf). Diferenciadores ya propios: context engine integrado + wiki-ingest + proxy interceptor + skills. Decisiones del owner: gaps estratégicos todos agendados; lifecycle = heat+decay L1; quick-wins todos; optimizables todos; gate de captura opcional por config; benchmarks agendados.

| ID | Descripción | Archivos | Effort | Prio | Estado |
|----|-------------|----------|--------|------|--------|
| `MEM-63` | **Quick-win docs+embeddings**: corregir doc stale `auto_recall.rs:69-73` (dice que embeddings "degradan hasta wirear"; MEM-47 ya implementó el hook) + embeddings auto-on cuando hay provider configurado (chars-fallback solo sin provider) | `vanta-memory/src/core/hooks/auto_recall.rs`, Cargo features | 🟢 | 🟢 | ⬜ Pendiente |
| `MEM-66` | **claimStaleTasks (recuperación multi-worker)**: port del TDAM original no porteado (`pipeline_worker.rs:12-13` lo documenta) — worker muerto a mitad de tarea no debe atascar la tarea hasta el TTL; otro worker la reclama | `vanta-memory/src/services/pipeline_worker.rs`, `utils/local_backend.rs` | 🟡 | 🟠 | ⬜ Pendiente |
| `MEM-68` | **Gate opcional de aprobación de capturas** (gap #6): config `capture_approval=off|on`; en `on`, las memorias extraídas van a cola pendiente y un comando/tool `memory_approve/reject` las publica o descarta (patrón Cursor). Default off (filosofía never-block intacta) | `vanta-memory/src/core/record/l1_writer.rs`, MCP tools | 🟡 | 🟠 | ⬜ Pendiente |
| `MEM-69` | **Batch extracción costo-reducida**: agrupar split+dedup en menos llamadas LLM por flush (patrón Memobase: batch fijo −40-50% tokens) sin perder quality gate | `vanta-memory/src/core/record/l1_extractor.rs` | 🟡 | 🟢 | ⬜ Pendiente |
| `MEM-70` | **Benchmarks públicos LongMemEval-S + LoCoMo**: harness de evaluación contra vanta-memory y publicación en `docs/operations/BENCHMARKS.md` (Regla 11: bench archivo + comando reproducible). Referencia mercado: SuperMemory 81.6%, Hindsight 94.6% self-report | nuevo `evals/memory_bench.py` o Rust harness, `docs/operations/BENCHMARKS.md` | 🟡 | 🟡 | ⬜ Pendiente |

---

> **Origen:** follow-up del fix `8c8eef23` (vectores Binary insertables/recuperables por `get()`). El fix resolvió el contrato insert→get en memoria; la persistencia on-disk de vectores no-F32 queda pendiente (formato).

| ID | Descripción (→ Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|




---

## GOV - Gobernanza Documental — corrección integral post-auditoría (2026-08-22)

> **Origen:** auditoría documental completa docs/reviews/auditoria-documentacion-2026-08-21.md (Volumen I+II+Addendum, salud 6.5/10) + **decisiones del owner D1-D14** registradas en su sección final. **Plan de ejecución:** docs/plans/2026-08-22-doc-governance-plan.md (formato campaign, triage 30 DO / 2 DEFER). Task files se crean bajo demanda vía /pipeline task GOV-XX. Wave B es bloqueante del Show HN.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
> **CAMPAÑA COMPLETADA 2026-08-22: 29 ✅ · 1 ⬛ (A1 stop condition, fallback aplicado) · 0 failed.** Registro completo: docs/progreso/campanas/doc-gobernanza-gov.md + plan file §Estado de ejecución. Tickets derivados vivos abajo.


## GOV-TK — Tickets derivados de la campaña GOV (2026-08-22)

| ID | Descripción | Prio | Fuente |
|----|-------------|------|--------|
| GOV-TK1 | **CLI backup verification (re-escalada 2026-09-03: la mitad doctor --fix landed con dry-run seguro en `3a27c5f4`+hoy; queda el verificador de restore)**: subcomando `verify` o flag `--dry-run` en Restore + paridad runbook | 🟢 | D4b/B2 |
| GOV-TK2 | **Release** para que el binario MCP exponga las 18 tools skill_*/code_*/wiki_* (skill ya documenta 33; binario publicado tiene 15) | 🔴 | B6 |
| GOV-TK3 | Drift yaml↔real ×3: gramática IQL case del yaml vs parser UPPERCASE · GraphTraversalBody (roots numéricos + max_depth requerido) · search en DB fresca requiere rebuild-index previo | 🟠 | B5 |
| GOV-TK5 | Split Manual Estratégico según recomendación F2 (negocio→docs/business/ con banner snapshot; estado técnico fuera; archivar monolito) | 🟠 | F2/D-decisión |
| GOV-TK7 | put_batch metadatas solo-str: alinear doc-tutorial vs API o ampliar coercion | 🟡 | B3 |
| GOV-TK8 | Benchmarks: mejorar/probar/documentar (insumo: docs/benchmarks/_run_stdout.md se conserva como evidencia de corrida cruda) | 🟡 | owner E1 |

> Ticketeados aparte con decisión previa: ACID 4a-4d (post-launch Fase A, D14) · release triage semver 0.6.0 (D5, diferido) · MKT-18h wheels ARM64 + MKT-18f adapters (confirmados live por GOV-A5).

| `BND-07` | **Discord invite inválido + vantadb.dev sin DNS** (GOV-F1 🔴×2) — requieren acción externa del owner: crear invite nuevo de Discord y configurar DNS de vantadb.dev; luego actualizar README/CONTRIBUTING/SECURITY con los valores reales. Registrado en auditoría raíz pública GOV-F1 (commit dc3775ef). | README.md, CONTRIBUTING.md, SECURITY.md (externo al repo) | 🟡 | 🟠 | ⏳ Externo owner |

---

## P32 — Reviews de Módulos (campaña 14 reportes, 2026-08-23)

> **Origen:** campaña de deep-review con 14 sub-agentes sobre los 12 módulos del repo + análisis transversal. Reportes completos en `docs/reviews/modulos/*.md` (core.md, vantadb-mcp.md, vantadb-server.md, vantadb-python.md, vantadb-ts.md, vantadb-wasm.md, vantadb-node.md, vanta-memory.md, vanta-proxy.md, providers.md, integrations.md, benches.md, benchmarks.md, cross-modulos.md). Scores: memory 8.5 · core 8.3 · mcp 8.3 · proxy 8.0 · server 7.5 · ts/wasm/benches 7.0 · integrations/benchmarks/cross 6.5 · python 6.5 · providers 5.0 · node 4.5.
> **Regla:** cada fila referencia su reporte fuente; los reportes llevan sección "Trazabilidad Backlog" con el MOD-ID por hallazgo. Duplicados ya trackeados NO recreados (CORE-01/02, REVIEW-06..20, MKT-18f, MCP-24/28/29, DESKTOP-28..39).

| ID | Módulo | Sev | Hallazgo → Acción | Referencia | Estado |
|----|--------|-----|-------------------|------------|--------|
| `MOD-05` | core | 🟢 | Deprecar `InMemoryEngine` hacia StorageEngine in-memory: elimina clase de bug MOD-01 y ~850 líneas | `engine.rs:72` · core.md R5 | ❌ Pendiente |
| `MOD-22` | vantadb-ts | 🔴 | Tipos grafo ficticios: `GraphBfsResult{visited,levels,path}` vs wire format real (`u128[]` plano); blind-cast `as GraphBfsResult` + test que afirme shape real | `vantadb-ts/src/types.ts:208-212`, `vantadb-ts/src/vantadb.ts:1094` · ts.md R4#1 | ❌ Pendiente |
| `MOD-23` | vantadb-ts | 🟠 | `NativeVantaDB._native` captura solo throws síncronos; rechazos async escapan sin envolver en `VantaError` | `vantadb-ts/src/native.ts:89-95` · ts.md R4#2 | ❌ Pendiente |
| `MOD-24` | vantadb-ts | 🟡 | Nits agrupados TS: semántica distance/score JSDoc, guard type-lie (`validateVector` asserts Float32Array), duplicación `_mapRecord`/`_buildSearchRequest`, ejemplos JSDoc que no compilan, dep `pkg/` commiteada | `vantadb-ts/src/guards.ts`, `vantadb-ts/src/{vantadb,native}.ts` · ts.md R4#3–#10 | ❌ Pendiente |

---

## P33 — Developer Experience de SDKs (evaluación usuario-facing, 2026-08-23)

> **Origen:** evaluación DX solicitada por owner sobre superficies que tocan usuarios (instalación, quickstart, SDKs Python/TS/WASM, docs/api). Hallazgos verificados contra disco hoy. Formato `prompts/findings.md`: fila FIND-* con ref de origen.
> **Regla:** duplicados parciales referencian su fila MOD existente; no se recrean.

| ID | Sev | Hallazgo → Acción | Referencia | Estado |
|----|-----|-------------------|------------|--------|
| `FIND-20` | 🟡 | Sin persistencia nativa de estado de ventana (posición/tamaño/maximizado): cada arranque abre default. Tauri window state plugin o persistencia manual en app config nativa | `desktop/src-tauri/` · research Studio 2026-08-23 | ❌ Pendiente |
| `FIND-21` | 🟡 | Sin menú contextual nativo ni atajos globales: right-click usa menú del WebView; evaluar tauri menu API + global shortcut plugin + atajos in-app documentados en la guía de uso | `desktop/src/` (0 matches contextmenu) · research Studio 2026-08-23 | ❌ Pendiente |

## P34 - Revisión diseño/UX Vanta Studio (auditoría 3 sub-agentes, 2026-08-24)

> Contexto: revisión diseño/estructura/UX con Playwright visible + 3 sub-agentes (shell/resumen, MEMORIAS/inspector, lenses). 38 hallazgos; los P1 estructurales ya fixeados en commits `abc4ec10`/`c9ccfce3`/`5977a71b` (list/search multi-namespace, TitleBar web crash, dark mode grid, shadow-ink token, errores destructive, hints F1/F2 falsos). Quedan los siguientes.
>
> **Adenda 2026-08-24 (smoke E2E):**
> - **Smoke test E2E PASS** (Playwright visible, server real :8090, 7+1 registros): ingest con labels → grid → Inspector por TECLADO (Enter) → borrado 2 pasos → papelera → RESTORE → paleta Ctrl+K → AJUSTES. 0 errores de consola tras reload. Evidencia: screenshots `smoke-0*.png` en temp de sesión.

| ID | Effort | Descripción | Archivos | Estado |
|----|--------|-------------|----------|--------|
| `UX-02` | 🟡 | **Filas del grid no navegables por teclado**: `<tr onClick>` sin tabIndex/onKeyDown/aria-selected (igual en ResultsList `<li>`). Teclado/SR no puede abrir el Inspector desde el grid. | `DataExplorer.tsx:859-871`, `ResultsList.tsx:22-27` | ⬜ Pendiente |
| `UX-03` | 🟡 | **Focus trap en modales**: ImportPaste/ImportDrop tienen `role="dialog" aria-modal` pero Tab escapa al shell y no restauran foco al cerrar; ImportDrop sin autofocus. *Hook `useModalFocus.ts` ya commiteado (huérfano) — falta conectar en ambos modales y borrar los useEffect de Escape duplicados* | `ImportPaste.tsx:107-115`, `ImportDrop.tsx:124-133`, `ingest/useModalFocus.ts` | ⬜ Pendiente |
| `UX-04` | 🟡 | **Labels visibles en IngestForm**: solo-placeholder (aria-label no cuenta, WCAG 3.3.2); errores van a toast global sin anclaje inline; `window.confirm` nativo inconsistente con el patrón inline de dos pasos. *Lote aplicado (labels + error inline) y sobrescrito — re-aplicar* | `IngestForm.tsx:44-74` | ⬜ Pendiente |
| `UX-06` | 🟡 | **Contraste neón como texto**: labels 10-11px en `text-neon` (#FF5500) sobre crema ≈3.0:1 fallan AA 4.5:1. Definir `--color-accent-text` más oscuro (~#C24000) solo para texto pequeño o migrar a foreground | `HomeOverview.tsx:248,347`, `WorkspaceShell.tsx:767,814`, `MetricsGrid.tsx:126` | ⬜ Pendiente |
| `UX-07` | 🟡 | **Tabs ARIA incompletos** (2 superficies): sin `role="tablist"`, paneles sin `role="tabpanel"`, sin navegación por flechas | `Inspector.tsx:206-220`, `MemoryLens.tsx:437-451` | ⬜ Pendiente |
| `UX-08` | 🟡 | **Canvas 3D/scatter sin fallback accesible**: GraphLens/SpaceLens sin `role="img"` + aria-label descriptivo ni lista alternativa de nodos para teclado/SR | `GraphLens.tsx:111`, `SpaceLens.tsx:274` | ⬜ Pendiente |
| `UX-09` | 🟡 | **3 lenguajes de confirmación destructiva**: SpaceLens usa `window.confirm` nativo, ConsolidateLens `ConfirmDiscard`, TrashLens confirm inline de dos pasos — unificar | `SpaceLens.tsx:192-196` | ⬜ Pendiente |
| `UX-10` | 🟢 | **Densidad del grid MEMORIAS**: Payload max-w fijo + TTL w-24 + acciones ~90px → desborde horizontal a 1440px con Inspector abierto; sin toggle de visibilidad de columnas | `DataExplorer.tsx:265,169,620-641` | ⬜ Pendiente |
| `UX-11` | 🟢 | **Estados vacíos sin salida**: "Sin registros."/"No matches." sin acción siguiente (botón contextual "Ir a Ingestar" / "Limpiar búsqueda") | `DataExplorer.tsx:797-798`, `ResultsList.tsx:13-17` | ⬜ Pendiente |
| `UX-12` | 🟢 | **RESUMEN sin énfasis primario**: 7 cards + Métricas + KPIs + SOP + Export compiten con el mismo peso (border 3-4 + sombra); título "VISTA GENERAL" se renderiza 2 veces con layouts distintos (card de carga vs h1) → salto visual | `HomeOverview.tsx:228-249`, `WorkspaceShell.tsx:835-873` | ⬜ Pendiente |
| `UX-13` | 🟢 | **Banner audit filtra internos**: mensaje con `Unsupported("audit log no configurado")`, `NativeConnection::open` — redactar para usuario + `<details>` técnico | `ActivityPanel.tsx:149-158` | ⬜ Pendiente |
| `UX-14` | 🟢 | **PersonaPanel traga errores**: `catch(() => {})` muestra error real como "sin snapshot" — propagar con `onError(vantaErrorMessage(err))` | `MemoryLens.tsx:172` | ⬜ Pendiente |
| `UX-15` | 🟢 | **Misc menor**: badge `err` = `warn` en MetricsGrid (usar `text-destructive`); 2 formateadores de bytes (decimal vs binario) — extraer `fmtBytes` compartido; splash no saltable por teclado; notice bar sin botón ✕ enfocable; microcopy ES/EN mezclado ("waiting…"/"check"); botones sin clase `press` en ActivityPanel; skeleton en IndicesLens mientras llega el snapshot; párrafo de jerga en header Retrieval + magic number `h-[calc(100dvh-112px)]` duplicado | `MetricsGrid.tsx:120-136,7-12`, `KpiCards.tsx:16-18`, `SplashScreen.tsx:38-44`, `WorkspaceShell.tsx:752-760`, `ActivityPanel.tsx:111-134`, `IndicesLens.tsx:171-200`, `RetrievalLens.tsx:234-238` | ⬜ Pendiente |
| `UX-17` | 🟢 | **Grid no se refresca tras ingest manual**: IngestForm hace `onDone` (notice) pero no remonta el DataExplorer (`gridKey`) — el registro nuevo no aparece hasta pulsar "Traer". Pasar `onRefresh` al IngestForm como ya hace el batch delete | `WorkspaceShell.tsx` (superficie MEMORIAS), `IngestForm.tsx` | ⬜ Pendiente |
| `UX-19` | 🟢 | **Smoke E2E como guard de regresión**: el recorrido ingest→teclado→borrar→papelera→restore→paleta pasó verde con datos reales — convertirlo en test Playwright permanente (`desktop/e2e/`) para que el flujo crítico no dependa de QA manual | `desktop/` (nuevo e2e), CI | ⬜ Pendiente |

## P36 - Auditoría AGENTS.md & sistema de agentes (2026-08-24)

> Contexto: revisión integral de `AGENTS.md` (raíz) + `.opencode/AGENTS.md` + global `~/.config/opencode/AGENTS.md`. Verificadas las 17 rutas referenciadas (todas existen ✅), conteo de skills (193 = 162+31 ✅ contra audit del manifest), tablas de reglas/comandos/agents (1:1 con disco ✅). **Ya resuelto en sesión** (no requiere tarea): root `AGENTS.md` reescrito pointer-only (eliminaba duplicado de Regla 7 que divergía; commit via checkpoint paralelo `89ab5e2c`); metadata stale del manual corregida en `.opencode/AGENTS.md:9`; conteos globales actualizados (154→162). Queda lo siguiente.

| ID | Effort | Descripción | Archivos | Estado |
|----|--------|-------------|----------|--------|

## P37 - Auditoría diseño desktop post-fix (auditoría orquestador + 5 sub-agentes, 2026-08-24)

> **✅ Cerrada 2026-08-26 — DESKTOP-QW5 (H-13):** 9/9 DAUD verificadas y archivadas. Fixes D1-D11 ya commitados: E2E-VISUAL `480935a7` (DAUD-01), DAUD-LIMPI `3c53d8b2` (DAUD-03/04/05/07) + `b865c625` (DAUD-06/11), filterActive `ad0f34b1` (DAUD-02/QW4). Stash `06aa1a86` consumido por `b865c625` (DAUD-08). Registro en `docs/avance/activo/desktop.md` §P37 + `backlog-history.md` §Limpieza DAUD 2026-08-26. Plan: `docs/plans/2026-08-25-research-desktop-quickwins.md` Wave1 Task5.
>
> Contexto histórico: segunda auditoría de diseño de `desktop/` (misma jornada que P34, sesión paralela independiente). Detectó D1-D11; D1-D11 fueron corregidos el mismo día por 5 sub-agentes en paralelo (archivos disjuntos, cero colisiones con P34). Verificación integrada original: tsc ✅ · vitest 68/68 ✅ · vite build exit 0 ✅ · grep emoji 0 ✅. Resumen fixes: D1 App.css tokens theme-flipping (marco crema en dark eliminado) · D2 fuente base única (Geist gobierna) · D3 glifos emoji-prone → Lucide sw2.5 (~20 archivos, geométricos monocromos conservados como identidad linocut) · D4 FILTROS activo = active-state del sistema (INGEST único neón) · D5 hit targets ≥32px · D6 Mark.tsx hex → var(--color-neon) · D7 splash duplicado eliminado · D8 stagger extendido a 12 hijos · D9 ~14 utilidades CSS muertas borradas · D10 grid-tech/speed-lines overrides dark · D11 ventana Tauri 1280×800 min 1024×640 center. Dep nueva: `lucide-react`.

| ID | Effort | Descripción | Archivos | Estado |
|----|--------|-------------|----------|--------|
| — | — | *Sin tareas activas — 9/9 DAUD archivadas (ver historial)* | — | ✅ |
---

---

## P38 - Investigaciones huérfanas convertidas en tarea (auditoría docs/research, 2026-08-25)

> Contexto: auditoría completa de `docs/research/` (~80 docs, 3 sub-agentes vanta-research + verificación lead vía codegraph/grep). Detectó investigaciones con acciones recomendadas que **nunca se convirtieron en tarea** ni se ejecutaron. Cada fila fue validada contra el código actual el 2026-08-25 (evidencia citada). IDs nuevos: `RES-*` (research→tarea) y `DEC-*` (decisión de producto).

### Candidatos DESCARTADOS en validación (no re-proponer)

- **CRIT-01..09** del informe externo `VantaDB-28-07-2026.md` → **todos resueltos**: `archive.rs` valida longitudes + `sync_all` + flush con map_err (:95/:109/:135/:137), `wal_sharded.rs recover()` usa checkpoint_seq global con tests (:243-274, :641/:663), `wal.rs:338` default Periodic = sync cada write, Dockerfile `RUST_VERSION=1.94.1`, providers excluidos del workspace con NOTE documentada (`Cargo.toml:638`). El informe queda archivado como referencia histórica.
- **Guías Vectara/Chroma** (`vectara-competitive-research`) → ya existen `docs/migrate-from-vectara.md` y `03-migrating-from-chromadb.md` + `benchmarks_vs_lancedb_chroma.md`.
- **Purga PERFORMANCE_TUNING.md** (residual FND-13) → el archivo ya no existe en `docs/` (solo snapshots históricos).
- **INV-008 search_batch_requests** → YA implementado (`vantadb-python/src/lib.rs:1688`, wrapper async `__init__.py:342`). Archivar INV-008.

### 🔴 Alta

| ID | Effort | Descripción | Archivos / Origen | Estado |
|----|--------|-------------|-------------------|--------|

### 🟡 Media

| ID | Effort | Descripción | Archivos / Origen | Estado |
|----|--------|-------------|-------------------|--------|
| `RES-03` | 🟡 | **Canal multi-consumidor en ingestion pipeline**: reemplazar `Arc<Mutex<mpsc::Receiver>>` por async-channel/flume (contención serializada; única instancia sospechosa del inventario FND-19). `src/ingestion.rs:72` intacto verificado 2026-09-03 | `src/ingestion.rs:72` · Origen: `FND-19-arc-mutex-inventario.md` | ⬜ Pendiente (fila RESTAURADA 2026-09-03: borrada por error en sync 2026-09-01 — colisión ID con doc de research, implementación no realizada) |
| `RES-07` | 🟡 | **Calibrar `rss_threshold`** (F2: recalibrar `DEFAULT_RSS_THRESHOLD=0.80` con medición real) + bench full-scale `[10k..100k]` (F3) | `src/config.rs:22`, benches memory-budget · Origen: `FND-01-memory-budget.md` (follow-ups F2/F3) | ⬜ Pendiente |

### 🟢 Baja / proceso

| ID | Effort | Descripción | Archivos / Origen | Estado |
|----|--------|-------------|-------------------|--------|
| `RES-12` | 🟢 | **Touch targets ≥44px — re-escalado 2026-09-03: no son ~20 componentes; grep de `<button>` con `h-7/h-9` arroja 4-5 archivos**: `docs-view.tsx` (copiar-código c:576 + otro), `shortcut-overlay.tsx`, `site-navbar.tsx`, `tutorial-modal.tsx`, `command-palette.tsx:232` (cerrar búsqueda). El resto de hits son decorativos (barras benchmark-race, spans de iconos) — fuera de WCAG 2.5.5. Solo los 3 severos fueron corregidos antes con `size-11` | `web/src/components/vanta/{docs-view,shortcut-overlay,site-navbar,tutorial-modal,command-palette}.tsx` · Origen: `INV-015-touch-targets-44px.md` | ⬜ Pendiente (alcance 4-5 archivos) |
| `RES-15` | 🟢 | **Institucionalizar meta-001 C** — re-escalado 2026-09-03: B YA está institucionalizada (`research-decide.md:81` dirige WONTFIX/DESCARTES a `docs/avance/decisiones/wontfix.md` con motivo; wontfix.md con entries = micro-ADR vigente). Queda solo C: separar backlog negocio/técnico. Cruzar con GOV-TK5 (split Manual Estratégico) para no duplicar | `.opencode/rules/`, `docs/Backlog.md` · Origen: `meta-001-root-cause-analysis.md` | ⬜ Pendiente (solo C) |

### Decisiones de producto (antes de código)

| ID | Effort | Descripción | Origen | Estado |
|----|--------|-------------|--------|--------|
| `DEC-01` | 🟠 | **Session layer VantaDB MCP: go/no-go y alcance.** Roadmap de 4 fases propuesto (session cache, Claude Code plugin, sync/improve, lesson extraction) pero las 5 open questions siguen abiertas. Decidir vía ADR antes de escribir código | `COGNEE_EVALUATION.md`; research `docs/research/res03-session-layer-gonogo.md` | ✅ Resuelta por research (2026-08-25): **defer-as-scoped** — F1 no-go (threads/scenes/genlog ya lo cubren), F2 defer docs-only (guía conexión Claude Code), F3/F4 no-go (sync auto requiere benches Regla 9). Owner escribe ADR citando el doc |
| `DEC-02` | 🟠 | **Billing/quota CreditCalculator en server mode** (TDAM #9). Decisión previa requerida: UNA calculadora (÷1000 vs ÷10000). Habilita multi-usuario/VantaDB Pro | `tdam/SYNTHESIS.md` §9, `tdam/09-deploy-usage.md` | ⏸️ **ICEBOX 2026-09-03** — verificado: 0 implementaciones cost/credit en vanta-proxy, PRX-03 (virtual keys) también pendiente; gate de decisión de producto, no deuda activa. Reabrir cuando Pro/billing entre al horizonte |

---

## P39 - vanta-proxy — Gateway agéntico completo (investigación 3 frentes, 2026-08-25)

> **Origen:** investigación profunda vanta-proxy vs estado del arte (LiteLLM/OpenRouter/Portkey/Helicone/Cloudflare AIG/claude-code-router/Bifrost/TensorZero) vs necesidades de usuarios de coding agents. **Identidad decidida por el owner: gateway completo** (no especialista). Diferenciador a preservar: memoria en tránsito + interceptor de tools server-side (único en el mercado). Deudas internas: state machine `advance()` sin wiring, clasificador Claude Code huérfano, mem-commands stub, fail-open sin trigger (`server.rs:177`, `claude_code.rs:57`, `mem_command.rs:102`).

| ID | Effort | Descripción | Archivos | Estado |
|----|--------|-------------|----------|--------|
| `PRX-01` | 🟡 | **Wiring completo de código ya construido**: (1) conectar `SessionStore::advance()` al pipeline (disparo por header/ruta — hoy solo tests); (2) consumidor para `classify_cc_request` (routing Main/Fork/Sidequery); (3) mem-commands reales: `mem:sync`/`create-skill` ejecutando el pipeline en vez de responder stub; (4) trigger real de `set_degraded(true)` (upstream 429/fallos consecutivos → fail-open del limiter) | `server.rs`, `session.rs`, `session/claude_code.rs`, `mem_command.rs`, `rate_limit.rs` | ⬜ Pendiente |
| `PRX-02` | 🔴 | **Fallback multi-upstream + retries**: config `[upstreams]` array con prioridad; retries backoff exponencial ante 429/5xx; health pasivo (errores consecutivos degradan un upstream); el caso #1 de la comunidad: quota agotada → seguir sin cortar la sesión | `config.rs`, `forward.rs` | ⬜ Pendiente |
| `PRX-03` | 🔴 | **Cost tracking + virtual keys**: contabilidad tokens/costo por key/sesión/modelo (tabla de precios configurable), budgets con enforcement (429 propio al agotar), `/snapshot` ampliado a dashboard de consumo. Base para equipos | nuevo `cost.rs`, `report.rs` | ⬜ Pendiente |
| `PRX-04` | 🔴 | **Cache-preserving injection** (prioritario): la inyección actual cambia el prefijo del system prompt y invalida los hits de prompt caching de Anthropic/OpenAI. Inyectar después del bloque cacheable o gestionar markers `cache_control`; test de prefijo estable byte-a-byte | `inject.rs`, nuevo test | ⬜ Pendiente |
| `PRX-05` | 🟡 | **Model discovery + endpoints auxiliares**: `GET /v1/models` (puebla el picker /model de Claude Code con `CLAUDE_CODE_ENABLE_GATEWAY_MODEL_DISCOVERY=1`), `count_tokens`, manejo de headers `anthropic-beta` — hoy 404s rompen clientes (litellm#13252) | `handlers/`, `forward.rs` | ⬜ Pendiente |
| `PRX-06` | 🟠 | **Task-aware routing por tier**: slots haiku/sonnet/opus mapeables a modelos/upstreams distintos (patrón claude-code-router: −90% costo documentado) usando el clasificador CC ya existente; incluir `/v1/responses` en el tool-loop y con spaceId propio | `server.rs`, `config.rs` | ⬜ Pendiente |
| `PRX-07` | 🟡 | **PII/secret redaction en egress**: patrones configurables (AWS keys, tokens, emails, custom regex) aplicados antes del forward; modo block/mask/log (estilo AegisGate/Kong Ent.) | nuevo `redact.rs` en pipeline pre-forward | ⬜ Pendiente |
| `PRX-08` | 🟢 | **Higiene y ceilings documentados**: auth O(1) con índice HashMap (hoy scan lineal 10k users/request); upstream default no autorreferencial; LRU evict para sesiones; evict de buckets de rate-limit; writeback incremental (hoy full-file rewrite); tools mixtas cliente+nuestras sin dejar tool_calls colgados | `auth.rs`, `config.rs`, `session.rs`, `rate_limit.rs`, `writeback.rs`, `memory_tools.rs` | ⬜ Pendiente |
| `PRX-09` | 🟠 | **Semantic caching** (gateway completo): cache exact primero (barato), luego semántico opcional con embeddings del provider configurado — cuidado con invalidación por inyección de memoria (coordinar con PRX-04) | nuevo `cache.rs` | ⬜ Pendiente |
| `PRX-10` | 🟢 | **Guardrails y MCP governance (fase posterior)**: moderación input/output conectable, allowlists MCP por virtual key (patrón Bifrost/Portkey). Requiere PRX-03 (keys) — no empezar antes | diseño previo requerido | ⬜ Pendiente |
| `PRX-11` | 🔴 | **Traducción Anthropic↔OpenAI bidireccional fiel** (necesidad #4, pains #1/#4/#5/#6): hoy el proxy es verbatim passthrough — solo sirve upstreams del mismo protocolo. Gateway completo exige: `/v1/messages` → backend OpenAI-only (GLM/DeepSeek/Ollama) y viceversa, con streaming SSE + tool_use/tool_result incrementales + thinking/reasoning blocks bidireccionales + sanitización de campos Anthropic-only (`cache_control`, `thinking`) + guard de `max_tokens` para no truncar JSON de tool calls largos + manejo de beta headers (`anthropic-beta`) que Bedrock/upstreams rechazan. Es donde se rompen TODOS los proxies competidores — hacerlo bien es ventaja directa | nuevo `translate.rs`, `handlers/` | ⬜ Pendiente |
| `PRX-12` | 🟡 | **Compat suite contra releases de coding agents** (pain estructural #3: "actualizaciones de Claude Code rompen proxies sin aviso", litellm#11358): fixtures de requests/responses REALES de Claude Code/Codex/OpenCode (incl. params nuevos `output_config`, beta headers, thinking variants) como tests de regresión que corren en cada PR; actualizar fixtures cuando salga release nueva de los agentes | `vanta-proxy/tests/fixtures/`, CI | ⬜ Pendiente |
| `PRX-13` | 🟢 | **Optimización de contexto en tránsito** (necesidad #12, patrón shift proxy): resize/recompresión de imágenes en requests, modos performance/balanced/economy configurables por key — reduce tokens antes del forward | pipeline pre-forward, `config.rs` | ⬜ Pendiente |

---

## P40 - Investigación INV-vantadb-server (2026-08-25)

> **Origen:** `/research vantadb-server` → `docs/reviews/research-vantadb-server-20260825.md` (score 8.0/10, apéndice H-01..H-14). Decisiones HITL 2026-08-25: 8 filas nuevas SRV, 2 wontfix (H-12 gRPC, H-13 SSE). Duplicados ya trackeados NO recreados: H-01=`AUD-043`, H-02=`MOD-15`, H-03=`REVIEW-10`, H-10=`FIND-24` (confirmada prioridad 🔴 Alta por decisión del owner).

| ID | Descripción | Archivos clave | Esfuerzo | Prioridad | Estado |
|---|---|---|---|---|---|
| `SRV-01` | **Rotación/retención del audit log JSONL** — hoy crece indefinido (`append-only` sin límite); qdrant v1.17 rota daily + `max_log_files`. Rotación por tamaño/día + retención configurable + test | `src/audit.rs`, `src/cli_server.rs` (`audit_events`, `read_audit_page`) | 🟢 | 🟡 Media | ⬜ Pendiente |
| `SRV-06` | **OIDC/JWT authentication** (estratégica enterprise): requisito de facto para "equipos" (weaviate OIDC nativo, qdrant JWT RBAC HS256 offline). **DISCOVERY primero (vanta-arch):** jsonwebtoken HS256 offline vs OIDC discovery, alcance mínimo viable sobre `auth_middleware` | `src/cli_server.rs:633-773`, `src/config.rs`; research §2 | 🔴 | 🟡 Media | ⬜ Pendiente (requiere DISCOVERY) |

---

## P41 - Investigación INV-vantadb-ts (2026-08-25)

> **Origen:** `/research vantadb-ts` → `docs/reviews/research-vantadb-ts-20260825.md` (score 7.2/10, apéndice H-01..H-14). Decisiones HITL 2026-08-25: 13 filas nuevas TS, 0 wontfix. Duplicados ya trackeados NO recreados: H-01=`MOD-22`, H-02=`MOD-23` (filas restauradas en P32 el mismo día — la ausencia era el hallazgo H-05, aplicado inline), H-03 solapa=`RES-06`. Quick wins (TS-02/05/06/07/08) → plan `docs/plans/2026-08-25-research-vantadb-ts-quickwins.md`.

| ID | Descripción | Archivos clave | Esfuerzo | Prioridad | Estado |
|---|---|---|---|---|---|
| `TS-09` | **Bench reproducible JS/WASM** (insert+search p50/p95/p99, dataset determinístico) citable bajo Regla 11 — sin números no hay claims ni comparativas vs Orama/vectra posibles (H-11) | nuevo bench en `vantadb-ts/bench/`, resultados a `docs/operations/BENCHMARKS.md` | 🟡 | 🟡 Media | ⬜ Pendiente |
| `TS-10` | **Plan de distribución/adopción** (estratégica): playground browser interactivo + docs-site + comparativa honesta vs Orama/vectra/wa-sqlite/DuckDB-WASM usando la matriz del informe §3 — adopción actual 12 dl/semana vs 35K-465K competidores (H-06) | web/, docs/, estrategia SHOW_HN; research §3 | 🔴 | 🟠 Media-Alta | ⬜ Pendiente (requiere DISCOVERY) |
| `TS-11` | **Roadmap paridad sub-clientes**: planificar exposición vía WASM de wiki/conversation/skills cuando core lo permita — hoy `db.wiki` es `{}` documentado (H-12) | `vantadb-wasm/src/lib.rs`, `vantadb-ts/src/vantadb.ts:311-319` | 🔴 | 🟢 Baja | ⬜ Pendiente |
| `TS-12` | **Publicar `vantadb-node` en npm** (prebuilds multiplataforma napi-rs): desbloquea la ruta `NativeVantaDB` (persistencia real fjall/WAL en Node) rota para consumidores hoy (npm 404) (H-14); marcar experimental hasta prebuilds completos | `vantadb-node/` (CI prebuilds), README native path | 🟡 | 🟠 Media-Alta | ⬜ Pendiente |
| `TS-13` | **Posicionamiento vs Orama en web/** (decisión HITL: mover a módulo web): sección "Why VantaDB" honesta con matriz diferenciadores (durable WAL browser, híbrido RRF nativo, grafo+IQL, errores tipados) vs FTS-first de Orama (H-13) | `web/` (landing/docs); fuente: research-vantadb-ts §3 | 🟢 | 🟡 Media | ⬜ Pendiente |

## P42 - Investigación INV-vantadb-wasm (2026-08-25)

> **Origen:** `/research vantadb-wasm` → `docs/reviews/research-vantadb-wasm-20260825.md` (score 6.8/10, apéndice H-01..H-23). Decisiones HITL 2026-08-25: 14 filas nuevas WSM + plan quick wins (`docs/plans/2026-08-25-wasm-quickwins.md` con H-01/H-05/H-07/H-08/H-16) + 2 estrategias aprobadas (H-21 adopción npm, H-22 ADR browser-first) + 1 wontfix (H-23 static-hosting read-only). Duplicados ya trackeados NO recreados: H-19=`P2-8`. MOD-25..28 derivados del review 2026-08-22 nunca materializados (H-02): absorbidos por este batch (fallback→WSM-01, cuotas→WSM-02, nits→plan quick wins).

| ID | Descripción | Archivos clave | Esfuerzo | Prioridad | Estado |
|---|---|---|---|---|---|
| `WSM-11` | **Señalizar metadata descartada**: `memory_record_to_js` ignora error de serialización de metadata (record devuelto sin metadata sin señal); propagar error o contador | `vantadb-wasm/src/lib.rs:1582 aprox` | 🟢 | 🟢 Baja | ⬜ Pendiente |
| `WSM-14` | **Plan adopción npm** (estrategia H-21 aprobada): README npm con posicionamiento del nicho "browser AI agent memory", demo Transformers.js enlazada, keywords/comparativa honesta vs Orama (5.44M desc/mes vs 187). Sin claims de performance sin benchmark (Regla 11) | `vantadb-wasm/pkg/README.md` template, landing docs | 🟡 | 🟠 Alta | ⬜ Pendiente |

## P43 — Research web 2026-08-25 (INV-web-01, docs/reviews/research-web-prod-20260825.md)

| `WEB-02` | **Publicar benchmarks propios en /benchmarks**: convertir datos de `docs/operations/BENCHMARKS.md` en tablas p50/p99 públicas citando fuente reproducible (estilo Chroma home) — social proof honesto sin claims de adopción inexistentes (Regla 11). Verificar que la ruta existente muestre datos actuales. Origen: INV-web-01 H-10 | `web/src/app/benchmarks/page.tsx`, `docs/operations/BENCHMARKS.md` | 🟡 | 🟡 Media | ⬜ Pendiente |
| `WEB-03` | **Restaurar/eliminar assets gato faltantes**: 4 refs a `mascota_gato.png`/`avatar_gato.png` con fallbacks silenciosos — restaurar en `public/assets/` o quitar refs muertas. Origen: INV-web-01 H-03 | `web/src/components/vanta/easter-egg.tsx:78`, `web/src/app/opengraph-image.tsx:15`, `web/src/components/vanta/vanta-data.ts:1062,1069` | 🟢 | 🟢 Baja | ⬜ Pendiente |

| `WEB-09` | **Densidad efectos decorativos home**: 73 usos (trust-bar ×11, hero 5 capas) — requiere criterio visual del owner; puede quedar diferida como decisión de diseño fino. Origen: INV-web-01 H-07 | `web/src/app/page.tsx` + componentes mark/trust-bar | 🟡 | 🟡 Media-Baja | ⬜ Pendiente (requiere input visual owner) |

## P44 — Research integrations 2026-08-25 (INV-integrations-01, docs/reviews/research-integrations-20260825.md)

> **Origen:** `/research integrations` → `docs/reviews/research-integrations-20260825.md` (score 6.3/10, apéndice H-01..H-11). Decisiones HITL 2026-08-25: 9 quick wins APLICAR → plan `docs/plans/2026-08-25-integrations-research-wins.md` (QW-1..9; absorbe MOD-46..50 huérfanas: 46→QW-1, 47→QW-2, 48→QW-3, 49→QW-4, 50→QW-5; H-01=QW-7 cubre MKT-18f ampliada a 9 paquetes) + 2 estrategias aprobadas acá. Ningún wontfix.

| ID | Descripción | Archivos clave | Esfuerzo | Prioridad | Estado |
|---|---|---|---|---|---|
| `INTG-01` | **Adapter LangGraph** (H-06, estrategia): checkpointer para memoria corto-plazo + `BaseStore` KV con namespaces jerárquicos — la persistencia moderna del ecosistema langchain vive en LangGraph, hoy solo cubrimos `VectorStore`. Fuente: reference.langchain.com/python/langgraph.store.base/BaseStore · docs.langchain.com/oss/python/langgraph/persistence | `integrations/langchain/` (nuevo paquete o submódulo), CI | 🔴 | 🟠 Alta (P1) | ⬜ Pendiente |
| `INTG-02` | **Backend Memory unificada CrewAI** (H-07, estrategia): migrar de `BaseTool` a storage backend de la clase `Memory` unificada (v1.x reemplazó short/long/entity/external). Fuente: docs.crewai.com/concepts/memory | `integrations/crewai/vantadb_crewai/vectorstore.py` | 🟡 | 🟡 Media (P2) | ⬜ Pendiente |

## P46 - Research desktop 2026-08-25 (INV-desktop-prod, docs/reviews/research-desktop-prod-20260825.md)

> Score 7.4/10. Quick wins aprobados (H-01..H-05, H-07, H-11, H-13, H-14, H-15) -> plan `docs/plans/2026-08-25-research-desktop-quickwins.md` (listo para `/pipeline run`). Estrategia -> filas abajo. Firma de instaladores diferida -> wontfix (ver DEVOPS-10).

| ID | Effort | Descripción | Archivos | Estado |
|----|--------|-------------|----------|--------|
| `DESKTOP-40` | 🔴 | **i18n real ES/EN**: selector idioma en AJUSTES sin backing (DESKTOP-31 deuda); toda la UI ES hardcodeada mientras web usa tt(). Reutilizar patrón tt() de web con catálogos ES/EN; wiring Settings->shell. Origen: INV-desktop H-06 | `desktop/src/pages/Settings.tsx`, `WorkspaceShell.tsx` + catálogo nuevo | ⬜ Pendiente |
| `DESKTOP-41` | 🟢 | **Smoke-test instalador en VM Windows limpia** (Step 3 DESKTOP-24 pendiente): instalar NSIS+MSI, verificar arranque, sidecar server, deep link vanta://, WebView2 bootstrapper | instaladores `desktop/src-tauri/target/release/bundle/*`, VM limpia | ⬜ Pendiente |
| `DESKTOP-42` | 🔴 | **Bundles macOS/Linux** (prioridad baja): icns ya existe; falta target dmg/app + AppImage/deb, CI matrix y testing por SO. Toda la competencia GUI es cross-platform. Origen: INV-desktop H-09 | `tauri.conf.json:37` targets, `.github/workflows/desktop.yml` | ⬜ Pendiente |
| `DESKTOP-43` | 🟡 | **Auto-update vía tauri-plugin-updater**, bloqueado por firma (wontfix DEVOPS-10) y endpoint de manifests; desbloquear tras decisión de distribución pública. Origen: INV-desktop H-10 | `tauri.conf.json` plugins, CI release | ⬜ Pendiente |
| `DESKTOP-44` | 🟡 | **Validación manual Proxy Dashboard con upstream LLM vivo** (TurnReports/sesiones/write-back/rate-limit end-to-end, deuda DESKTOP-38) — sesión guiada owner+agente, no tarea autónoma. Origen: INV-desktop H-12 | `desktop/src/components/proxy/ProxyDashboard.tsx`, vanta-proxy | ⬜ Pendiente |
| `DESKTOP-45` | 🟢 | **Restos scope-blocked del plan desktop-quickwins** (wave 1 INV-DECIDE): H-11 versión single-source vía release-plz (`[[package]]` + `release=false`, toca raíz del workspace), H-15 baseline perf del app (startup/RAM idle → BENCHMARKS.md, exige app corriendo — Regla 11), H-07 specs E2E nuevos (proxy dashboard/graph/space lenses — requieren sesión Playwright dedicada) | `release-plz.toml`, `desktop/package.json`, `docs/operations/BENCHMARKS.md`, `desktop/e2e/` | 🟡 mixto | ⬜ Pendiente |

## P45 - Research providers 2026-08-25 (INV-providers-01, docs/reviews/research-providers-20260825.md)

> Score 4.0/10 (regresion vs review 2026-08-23 que dio 5.0: openai ya no compila). Quick wins aprobados (PROV-01/03/06/07/08) -> plan `docs/plans/2026-08-25-research-providers-quickwins.md` listo para `/pipeline run`. Trazabilidad MOD-41..45 archivada como superada (ver historial). Estrategia H-14 -> ADR pendiente (decision en memory).

| ID | Descripcion | Archivos clave | Esfuerzo | Prioridad | Estado |
|---|---|---|---|---|---|
| `PROV-11` | **Embed batching/async** (baja prioridad): aprovechar AsyncClient ollama / aembedding litellm cuando haya volumenes grandes. Origen: INV-providers-01 H-12 | los 3 `embed()` | ?? | ?? Baja | ? Pendiente |
| `PROV-12` | **Publicar wheels PyPI** (estrategia H-04 aprobada): pyproject.toml + maturin, CI release multiplataforma (macos/windows/linux x86_64+aarch64), secrets PyPI. Desbloquea el diferenciador real (storage embebido local acoplado al embed). Pre-requisitos: PROV-01/02/04. Origen: INV-providers-01 H-04 | providers/*/, nuevo workflow release, CI_POLICY | ?? | ?? Alta | ? Pendiente |

## P47 — Promoción a `default-members`: criterios 100% estables (server/mcp/memory/proxy + ts/node)

> **Origen:** discusión 2026-08-27 sobre `Cargo.toml:636-642` — `default-members = [".","vantadb-python"]` deja `server/mcp/wasm/memory/proxy` fuera del Fast Gate a propósito (`CATEGORY: EXPERIMENTAL`). `ts`/`node` son paquetes npm (no crates) y nunca pueden entrar en `default-members`, pero necesitan gate npm equivalente. Esta fase define **qué validar antes de decir “100% estable y puede pasar a default”** y deja la promoción como cambio reversible en 1 línea. **No promocionar sin que los 10 checks pasen.** Ver `docs/operations/CI_POLICY.md` y `.opencode/references/definition-of-done.md`.
>
> **Contrato de promoción (Definition of Done para default):** cada crate/paquete a promover debe pasar **todos** los gates siguientes en 3 corridas consecutivas sin flaky, en runner limpio (`cargo clean` / `npm ci`):
> 1. `cargo check -p <crate> --all-targets --all-features` + `cargo fmt --check` + `cargo clippy -p <crate> --all-targets --all-features -- -D warnings` = 0 warnings
> 2. `cargo nextest run -p <crate> --profile audit -j 2` (o `npm test` para ts/node) = 0 failed, 0 ignored flaky, sin `#[ignore]`
> 3. `cargo deny check` (licenses MIT/Apache-2.0, advisories, bans) = 0
> 4. `scripts/validate-docs-coverage.ps1` = 0 gaps para APIs públicas del crate
> 5. Workflow CI con `paths:` filter, sin `continue-on-error: true`, bloqueante en PR, `timeout-minutes` real <5 min medido en CI (Fast Gate) o documentado como Heavy
> 6. `publish = false` ok — pero `cargo package --dry-run` no debe fallar por metadata
> 7. Para `wasm`: `wasm-pack build --target bundler` + `wasm32-unknown-unknown` instalado en CI, `cargo test --target wasm32-unknown-unknown` si aplica
> 8. Para `node`: `napi build --platform` matrix 7 targets via `release-npm-node.yml` con artifact, `npm pack` incluye `*.node`
> 9. **Medición Fast Gate con todos incluidos:** `just verify` / `dev-tools/verify.ps1` con `default-members` ampliado debe seguir `<5 min` en `ubuntu-latest` (o se re-etiqueta como Heavy y se justifica)
> 10. ADR breve en `docs/architecture/adr/` registrando la promoción y el coste (tiempo CI + toolchain extra) — reversible

| ID | Descripción (→ Resultado) | Archivos clave | Esfuerzo | Prioridad | Estado |
|---|---|---|---|---|---|
| `STABLE-04` | **Validar `vantadb-mcp`** — 72 `mcp_tests`, protocolVersion 2024-11-05, `serve_lines` con `OpGate`. Gates 1-6 + `cargo test -p vantadb-mcp` + `test-mcp.py` 37 checks vs skill. | `vantadb-mcp/src/`, `vantadb-mcp/tests/` | 🟡 1d | 🔴 Alta | ⬜ Pendiente |
| `STABLE-05` | **Validar `vantadb-wasm`** — requiere `wasm32` + `wasm-pack`. Gates 1-8: `rustup target add wasm32-unknown-unknown` + `wasm-pack build --target bundler` + `wasm-pack test --node` si hay test wasm + `cargo check -p vantadb-wasm` + docs de `WASM_PERSISTENCE.md`. | `vantadb-wasm/Cargo.toml`, `vantadb-wasm/src/`, `vantadb-wasm/pkg/` | 🟡 1d | 🟠 Media | ⬜ Pendiente |
| `STABLE-06` | **Validar `vantadb-ts` (npm)** — no entra en `default-members`, pero gate npm debe ser Fast Gate. Gates 5+2: `release-npm-61.yml:tests` ya mide 26s (<5 min) — re-medir en CI limpio + `npm ci && npm run build && npx vitest run` 264 tests + `npx eslint .` + `npm pack` incluye `engines`. | `vantadb-ts/package.json`, `vantadb-ts/src/`, `.github/workflows/release-npm-61.yml` | 🟢 4h | 🟠 Media | ⬜ Pendiente |
| `STABLE-07` | **Validar `vantadb-node` (npm, napi-rs)** — matrix 7 targets. Gates 5+8: `release-npm-node.yml` builds `x86_64-pc-windows-msvc` + Linux gnu/musl + macOS x64/arm64 + `npm test` 25 tests + `npm pack` incluye `*.node`. Medir tiempo matrix y asegurar sin `continue-on-error`. | `vantadb-node/package.json`, `vantadb-node/src/lib.rs`, `.github/workflows/release-npm-node.yml` | 🟡 1d | 🟠 Media | ⬜ Pendiente |
| `STABLE-09` | **Promoción atómica + rollback plan** — Si `STABLE-00..08` verde en 3 corridas, PR único: `Cargo.toml:636 default-members` ampliado + `docs/operations/CI_POLICY.md` §default-members + `.opencode/rules/release-ci.md` + `dev-tools/verify.ps1` comentando que `wasm` requiere `rustup target add`; `git tag` no, `cargo publish` no afectado (`publish=false` sigue). Incluir rollback de 1 línea (revert `default-members`) en descripción del PR. | `Cargo.toml`, `docs/operations/CI_POLICY.md`, `dev-tools/verify.ps1` | 🟢 4h | 🔴 Alta | ⬜ Pendiente |
---

## Phase 48: 🧪 Testing & Benchmarking Hardening (auditoría multi-agente 2026-08-30)

> **Origen:** Auditoría multi-agente ejecutada 2026-08-30 (sesión `ses_fabf69692ffeP5c7mycKcsGSV0`) — 5 sub-agentes investigaron en paralelo: (1) prácticas externas, (2) tests Rust actuales, (3) benchmarks, (4) datasets/data, (5) CI/CD + scripts. Plan completo: `docs/plans/2026-08-30-testing-bench-harden.md`. Decisiones D1-D7 registradas en el plan.
>
> **Estado del proyecto al inicio:** 2034 tests pasando / 1 skipped (TEST_MAP.md canónico), 19 benches criterion 0.8, 4 fuzz targets, Miri+ASan+TSan, llvm-cov 81.40% root (ADR-018 gate ≥80%), 17 GH Actions workflows con SHA-pins generalizados, release-plz con OIDC.
>
> **Top 3 a atacar primero:** **TBH-01 ✅** (verify_datasets — completado 2026-08-31, commit `0e67f354`), **TBH-02** (init bench baseline — `benchmarks/criterion_baseline.json` está vacío, primera ejecución nightly lo poblará), **TBH-03** (ci-gate — main rojo actualmente NO corta PRs por `if: schedule`).

### 🔴 Prioridad ALTA — Rompen la promesa "replicable y funcional siempre que se pruebe"

| ID | Descripción | Archivos clave | Esfuerzo | Prio | Estado |
|----|-------------|----------------|----------|------|--------|
| ~~`TBH-01`~~ | ~~verify_datasets.{sh,ps1} + CI gate~~ — ✅ Completado 2026-08-31 (commit `0e67f354`). | `scripts/verify_datasets.sh`, `scripts/verify_datasets.ps1`, `.github/workflows/heavy-certification-50.yml` | — | 🔴 | ✅ |

### 🟡 Prioridad MEDIA — Mejoras de cobertura y calidad

| ID | Descripción | Archivos clave | Esfuerzo | Prio | Estado |
|----|-------------|----------------|----------|------|--------|

### 🟢 Prioridad BAJA — Nice to have (no bloquea)

| ID | Descripción | Archivos clave | Esfuerzo | Prio | Estado |
|----|-------------|----------------|----------|------|--------|

### 📋 Tarea Diferida (out of scope de P48)

| ID | Descripción | Por qué fuera |
|----|-------------|---------------|
| `ISSUE-TS-001` | **Fix TS SDK roto (80/219 tests con `unreachable!()` WASM)** | Pre-existente (snapshot-2026-08-07). Pertenece a capa de bindings (`vanta-worker`). D4: diferido a issue separado. |

### 📌 Notas & Referencias

- **Investigación origen:** sesión `ses_fabf69692ffeP5c7mycKcsGSV0` (5 sub-agentes en paralelo)
- **Plan completo:** `docs/plans/2026-08-30-testing-bench-harden.md` (7 decisiones, 3 fases, riesgos, verificación)
- **DoR / DoD:** `.opencode/references/definition-of-done.md` — aplicar a cada TASK-01..23
- **Conventional Commits:** cada cierre usa `fix:`, `feat:`, `chore:`, `ci:` o `refactor:`
- **Política `#[ignore]`:** según `.opencode/AGENTS.md:466` — no agregar ignores sin Issue `flaky`
- **Estrategia datasets/benchmarks:** conservadora (no migrar a VIBE, no añadir PQ/i8, no comparativas head-to-head vs sqlite-vss/duckdb-vss) — ver D1 en plan
- **Ejecución sugerida:** Phase 1 (ALTA) primero como PR único (gate `just verify` verde), luego Phase 2 (MED) en PRs paralelos por grupo (CI / benches / tests), finalmente Phase 3 (BAJA) en lote único

---

## FIND-* — Hallazgos recientes que requieren triage (2026-08-30)

| ID | Effort | Descripción | Archivos clave | Estado |
|----|--------|-------------|----------------|--------|

---

## P48 — Testing & Benchmarking Hardening (CIERRE) — 2026-08-31

**Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md` (archivado a `docs/plans/archive/` 2026-08-31)

### Resumen
- **23 tareas TBH-01..23** ejecutadas en modo `/pipeline run` paralelo (waves de 3)
- **22 ✅ + 1 🟡 INCOMPLETO** (TBH-06 parcial — 3/5 tests migrados; los 2 query_result no existían como archivos)
- **27 commits** en `develop` desde `72b98dc6` (TBH-03) hasta `fa87bbe2` (TBH-18)
- **Hallazgo pre-existente** `FIND-MCP-001` (test roto en `vantadb-mcp/tests/context_tests.rs:70`) detectado durante TBH-01 y registrado para triage fuera de P48

### Tareas completadas (dominios)

**CI/CD (`docs/avance/activo/ci-cd.md`):**
- `TBH-01` — `scripts/verify_datasets.{sh,ps1}` + pre-test step en `heavy-certification-50.yml` — commits `0e67f354`+`1c392eff`
- `TBH-02` — `benchmarks/criterion_baseline.json` inicializado con 9 entries placeholder — commit `450910ec`
- `TBH-03` — `ci-gate.yml` universal gate (removed `if: schedule`) — commit `72b98dc6`
- `TBH-04` — `develop` agregado a 6 workflows + `release.yml` (D6 Dependabot alignment) — commit `abcce28f`
- `TBH-05` — `benchmarks/data_comp_bench/`, `benchmarks/data_bench_db/` gitignored + untracked — commit `b4eada51`
- `TBH-13` — SHA-pins 6 third-party actions en `desktop.yml` + `opencode.yml` — commits `42141706`+`8944b6e5`
- `TBH-14` — `cliff.toml:15 conventional_commits = true` — commit `95b2394b`
- `TBH-19` — `markdownlint-cli2` pre-commit hook — commit `7f78efb3`
- `TBH-22` — `release-binaries-63.yml` `push.tags: ['v*']` trigger — commit `c727380d`
- `TBH-23` — `cargo fmt --all` unificado (Justfile + audit-all.ps1) — commit `c83aa43d`

**Benchmarks (`docs/avance/activo/ci-cd.md` o `core-engine.md`):**
- `TBH-08` — `benches/wal_throughput.rs` (12 samples, 3 SyncMode × 4 batch sizes) — commit `84dcea9f`
- `TBH-09` — `benches/crash_recovery.rs` (3 corpus sizes, open + WAL replay) — commit `74c15fb6`
- `TBH-10` — `bench_concurrent.rs` convertido a `criterion_main!` (genera `estimates.json`) — commit `e1b64580`
- `TBH-11` — `heavy-bench-nightly-51.yml` extendido de 5 → 8 benches — commit `1ad3dad8`

**Tests (`docs/avance/activo/ci-cd.md`):**
- `TBH-06` — `insta 1.48` snapshot testing (3 parser tests migrados; 2 query_result 🟡 INCOMPLETO) — commit `2aab9288`
- `TBH-07` — `cargo-mutants` weekly job en `heavy-certification-50.yml` — commit `adc4f931`

**Datasets/docs (`docs/avance/activo/ci-cd.md` o `operaciones.md`):**
- `TBH-12` — `data/README.md` + `datasets/README.md` consolidados — commit `3c477f2e`
- `TBH-15` — `audit-tokens.sh` deleted (consolidado a `.ps1` only) — commit `b5a57b0b`
- `TBH-20` — `ci-examples-12.yml` matrix 3-OS — commit `00a78dce`
- `TBH-21` — `CoverageThreshold=60` review cadence en `CI_POLICY.md` — commit `e6e73e2b`

**Decisiones (NO implementación, doc-only):**
- `TBH-16` — `divan` NO introducido (D1+D5) — `docs/research/bench-framework-evaluation-2026-08-30.md` — commit `42cce02c`
- `TBH-17` — `loom` NO introducido — `docs/research/concurrency-testing-2026-08-30.md` — commit `ef4652bf`
- `TBH-18` — `dhat` NO introducido (YAGNI) — `docs/research/dhat-evaluation-2026-08-30.md` — commit `fa87bbe2`

### Retrospectiva (Start / Stop / Continue + 1 acción medible)

**Start (continuar haciendo):**
- Modo FAIL_MODE=parallel con waves de 3: 3-4x speedup vs secuencial, sin conflicto de archivos
- Sub-agentes con `Ruta` explícita en el plan (vanta-lead, vanta-worker, vanta-docs) — routing determinístico
- `Ponytail option (b)` (placeholders + primer nightly refresca) para evitar `cargo bench --workspace` 30+ min
- `git stash` selectivo cuando hay unstaged de otras waves (preserva blast-radius mínimo)
- TBH-XX todos los sub-agentes respetaron `cargo check -p vantadb` (NO `--workspace` por FIND-MCP-001)

**Stop (dejar de hacer):**
- Delegar tareas que tocan `Cargo.toml` (TBH-06, TBH-08, TBH-09) **en paralelo** — riesgo de merge conflict; hacerlo secuencial (Wave 5)
- `cargo check --workspace` cuando FIND-MCP-001 está abierto — `-p vantadb` es el scope correcto

**Continue (seguir haciendo igual):**
- Single-file / single-config cambios como default
- Conventional Commits con `task ID` en el mensaje
- Verificación mecánica pre-commit (actionlint, cargo fmt, cargo clippy)
- Memory persistence via `campaign_memory_write`

**Acción medible propuesta:**
> **Reducir ratio `insta` snapshot churn** — la primera corrida de cada nuevo `insta::assert_snapshot!` genera un `.snap.new` que requiere review manual. Tracking: contar PRs que tocan `tests/**/snapshots/*.snap` sin cambio de test asociado. Baseline actual: N/A (TBH-06 fue el primer snapshot). Métrica: si >30% de PRs que tocan `.snap` no cambian el test source, ajustar a `--require-pristine` en CI.

### Hallazgo pre-existente (no resuelto en P48)
- `FIND-MCP-001` — `vantadb-mcp/tests/context_tests.rs:70` no compila. Pre-existente al inicio de P48. Owner: bindings layer (`vanta-worker`). Bloquea `cargo check --workspace --tests`. NO resuelto en este sprint — fuera de scope de P48 (que se limitó a testing/benchmarking hardening, no fix de bindings).
