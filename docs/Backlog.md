---
title: "Active Backlog — VantaDB"
type: backlog-tracking
status: active
tags: [vantadb, backlog, engineering, phases, priorities]
last_reviewed: 2026-08-26
verified_by: "Historial de verificación: docs/avance/historial/backlog-history.md"
---

# Active Backlog — VantaDB

> **Purpose:** Single source of truth for all project tasks — organized by execution order.
> **Execution state lives in:** `docs/plans/YYYY-MM-DD-<campaign>.md` (plan file) + task files — per campaign-executor RULES.md §2. This file is the task catalog; the plan file is the execution state.
> **Completed tasks moved to:** `docs/avance/` (dominio) + `docs/avance/historial/backlog-history.md`
> **Historial de syncs y migraciones:** `docs/avance/historial/backlog-history.md` (último sweep mayor: 2026-08-26 — P37 DAUD-01..09 → historial vía DESKTOP-QW5; previo 2026-08-25 — limpieza P35/P38/P39 + auditoría docs/research)
> **Total open items:** 109 activas (reconteo DESKTOP-QW5 2026-08-26: P37 DAUD-01..09 9/9 → historial, -9; previo GOV-C7 2026-08-25: 118)
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
| **P24** 🧪 I+D futura (v3.0+) | 10 (FUT-02..11) | 📆 Futuro | 🗺️ Roadmap |
| **P25** 🔌 Exposición MCP/HTTP | 11 (MCP-16..26) | ~2-3 semanas | 🟡 Media |
| **P26** 🖥️ Vanta Studio (consola human-facing desktop) | Fase 0 ✅ (14/19) + Fase 1 ✅ (9/9) + Fase 2 ✅ (10/10: VS-CORE-04/05/06 + GRAFO-01..03 + ESPACIO-01..02 + OP-01..02) + Fase 3 ✅ (7/7: WEB-00..06) + **Fase 4 ✅ (18/18: DOC-01..04 + REST-01..06 + WASM-01..04 + FEAT-01..03 + VER-01)** | Planes archivados `docs/plans/archive/2026-08-18-vanta-studio-fase{1,2,3}.md` + `docs/plans/archive/2026-08-19-vanta-studio-fase4.md` | 🟢 Completada 2026-08-20 (ADR-027; E2E server + standalone WASM/OPFS PASS) |
| **P27** 🧠 Vanta Memory Engine (TDAM, orden F1–F7) | 38 (MEM-01..38) | ~8-12 semanas | 🔴 Alta (decisión de producto) |
| **GOV** 📋 Gobernanza Documental (post-auditoría 2026-08-21, plan `docs/plans/2026-08-22-doc-governance-plan.md`) | 30 tareas (T0: TIR ×3 · A: medición ×5 · B: Show-HN ×6 · C: maestros ×7 · D: taxonomía ×6 · E: limpieza ×1 · F: auditoría intocadas ×2) | ~6 días | 🔴 Alta (Wave B bloqueante Show HN; decisiones D1-D14 del owner en `docs/reviews/auditoria-documentacion-2026-08-21.md`) |
| **P36** 🔧 Auditoría AGENTS.md & sistema de agentes (2026-08-24) | 6 (AGT-01..06; 3 fixes ya aplicados en sesión) | ~1 día | 🟠 Media |
| **P37** 🎨 Auditoría diseño desktop post-fix (2026-08-24, orquestador + 5 sub-agentes) | 0 — ✅ 9/9 ejecutadas (DAUD-01..09 — commits `3c53d8b2`,`480935a7`,`b865c625`; DAUD-02 via DESKTOP-QW4 `ad0f34b1`) | — | ✅ Cerrada 2026-08-26 |
| **P38** 🔬 Research huérfanas → tarea (auditoría docs/research, 2026-08-25) | 17 (RES-01..15 + DEC-01/02; cada fila validada contra código con evidencia) | ~2-3 semanas (RES-01 es la más grande) | 🟡 Media (RES-01/RES-02 🔴 calidad/durabilidad) |

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
| `MKT-18h` | **Wheels ARM64 Linux + SHA reales Homebrew** — binarios incluyen `aarch64-unknown-linux-gnu` (`release-binaries-63.yml`) pero wheels NO (`release-wheels-60.yml` solo x86_64); `Formula/vantadb.rb` tiene SHA256 `0000...0` placeholders (inutilizable). | 🟡 1d | 🟠 | ❌ Pendiente |
| `MKT-18i` | **Docker Compose multi-servicio: Ollama + VantaDB + AnythingLLM** — `docker-compose.yml` existe pero solo servicio VantaDB. La guía migración LanceDB YA existe (`docs/tutorials/migration-from-lancedb.md`). | 🟢 2-4h | 🟡 | ❌ Pendiente |
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

## Phase 11: 🧠 Embeddings Local-First — `embeddings/` + 9 modelos + `embed-local` (2026-08-28)

> **Origen:** Investigación 2026-08-27 (`src/llm.rs:26`, `providers/*`, `vanta-memory`, MTEB 2026, fastembed) + decisiones owner 2026-08-28 (Opción B descarga lazy, default `multilingual-e5-small`, ONNX+HF, 3 EN / 3 ES / 3 Combined, excepción Qwen3 >3GB).
> **Plan:** `docs/plans/2026-08-28-embeddings-local.md` (9 tareas EMB-01..09, Fase1 4-6d, Fase2 2-3d, Fase3 1d, Fase4 4h).
> **Carpeta:** `embeddings/` no existe hoy — se crea en EMB-01 con `manifest.json` (9 modelos, rev pinned), `download.py`, `verify.py`, `README.md`, `models/` gitignored (`/embeddings/models/`).
> **Modelos:** 8 ≤3GB + 1 excepción `Qwen3-Embedding-8B` (16GB, MTEB 75.1). Todos ONNX+HF salvo Qwen3 (HF only). Balance 3 EN / 3 ES / 3 Combined (ver tabla en plan §4).

| ID | Descripción (→ Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `EMB-01` | **Infra `embeddings/` + manifest + `download.py` + `verify.py` + `.gitignore`** — `manifest.json` 9 modelos rev pinned + `manifest.lock` vacío + `download.py` (huggingface_hub lazy, `--only`) + `verify.py` (ort+tokenizers dim+cosine, ONNX vs HF >0.98) + `README.md` + delta `.gitignore:72` (`/embeddings/models/`). Validado `python -m py_compile download.py` + `verify.py --check` sin red. | `embeddings/**` (nuevo), `.gitignore:72` | 🟢 4-6h | 🔴 Alta | ⬜ Pendiente |
| `EMB-02` | **Feature `embed-local` + `LocalOnnxProvider` (ort+tokenizers)** — `Cargo.toml:97` `embed-local = ["dep:ort","dep:tokenizers"]` (ort 2.0 `load-dynamic`), `src/llm.rs:132` `LocalOnnxProvider { session, tokenizer, dim }` impl `EmbeddingProvider` (tokenize → session.run → mean pooling + L2 normalize), `src/llm.rs:39` factory rama `"local"` → `LocalOnnxProvider::new("embeddings/models/multilingual-e5-small/onnx")`, env `VANTA_LOCAL_MODEL`. | `Cargo.toml:97`, `src/llm.rs:26,39,132`, `src/config.rs` | 🟠 3-5d | 🔴 Alta | ⬜ Pendiente |
| `EMB-03` | **Descarga + verificación 9 modelos** — `python embeddings/download.py --all` (~22GB: 8×~0.8GB ONNX+HF + Qwen3 16GB) + `python embeddings/verify.py --all` → 9× dim + multi cosine (`"hola mundo" vs "hello world"` >0.60 multi, <0.50 EN-only) + ONNX vs HF >0.98. Output `verify.log` + `manifest.lock`. Flag `--skip-exception` para CI sin Qwen3. | `embeddings/models/**` (gitignored), `embeddings/manifest.lock`, `embeddings/verify.log` | 🟡 1d wall | 🔴 Alta | ⬜ Pendiente |
| `EMB-04` | **Cablear `vanta-memory` L1 (fix punto 3)** — `vanta-memory/Cargo.toml:40` `embed-local = ["vantadb/embed-local"]`, `vanta-memory/src/core/record/l1_writer.rs` + `core/hooks/auto_recall.rs:69` corregir doc stale + `L1DedupConfig::with_local_provider(Arc<dyn EmbeddingProvider>)` usa `LocalOnnxProvider` en vez de hash dim=8 (`MEM-47`). `vanta-cli --features embed-local` auto-cablea. | `vanta-memory/Cargo.toml:40`, `vanta-memory/src/core/record/*`, `vanta-memory/src/core/hooks/auto_recall.rs:69` | 🟡 1-2d | 🔴 Alta | ⬜ Pendiente |
| `EMB-05` | **MCP tool `embed_texts` (fix punto 3)** — `vantadb-mcp/src/handlers/tools.rs:549` nuevo arm `embed_texts { texts: string[], model?: string } -> { embeddings: float[][] }` reusa `EmbeddingProvider::embed_batch` (nuevo método batch, default impl en trait), budgeting `MCP-39` (truncado + cursor si >25k). | `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/config.rs`, `src/llm.rs:26` | 🟢 4-6h | 🟡 Media | ⬜ Pendiente |
| `EMB-06` | **SQL vector auto-embed (fix punto 3)** — `src/physical_plan/vector.rs:51` `PhysicalVectorSearch::open()` y `:129` `Refine::open()` añadir `#[cfg(feature="embed-local")]` branch con `LocalOnnxProvider::embed(&query_vec_text)` además de `remote-inference`. `VECTOR_SEARCH('hola mundo')` offline sin `VANTA_LLM_URL`. | `src/physical_plan/vector.rs:51,129` | 🟢 2-4h | 🟡 Media | ⬜ Pendiente |
| `EMB-07` | **Bench comparativo 9 modelos** — `benchmarks/embed_bench.py` (ingest 1k EN+ES, QPS, recall@10, RSS, p50 embed) → `benchmarks/embed_bench_report.json` (gitignored) + `docs/operations/BENCHMARKS.md` sección EMB (comando reproducible `python benchmarks/embed_bench.py --models all --skip-exception` vs `--include-exception`). | `benchmarks/embed_bench.py`, `docs/operations/BENCHMARKS.md` | 🟡 1d | 🟡 Media | ⬜ Pendiente |
| `EMB-08` | **Docs + Quickstart multi (fix punto 1)** — `docs/QUICKSTART.md:182` quitar "not yet exposed", `docs/tutorials/05-embedding-integrations.md:11,126` añadir `embed-local` tabla 9 modelos + one-model-per-namespace, nuevo `docs/api/EMBEDDINGS.md` (API `EmbeddingProvider`, env vars), `README.md:232` mover `embed-local` a Optional. | `docs/QUICKSTART.md:182`, `docs/tutorials/05-embedding-integrations.md`, `docs/api/EMBEDDINGS.md` (nuevo), `docs/operations/EXPERIMENTAL_FEATURES.md:60` | 🟢 4h | 🟡 Media | ⬜ Pendiente |
| `EMB-09` | **Excepción Qwen3 >3GB — wiring + doc** — Solo HF (16GB, GPU), `download.py --include-exception` + `verify.py` HF-only (dim 4096, multi >0.70), `embeddings/README.md` sección "Excepción >3GB" (Matryoshka 4096→1024, `trust_remote_code`), bench flag `--include-exception`. | `embeddings/README.md`, `benchmarks/embed_bench.py` | 🟢 2h | 🟢 Baja | ⬜ Pendiente |

**Dependencias:** `EMB-01 → EMB-02 → EMB-03 → (EMB-04,05,06 en paralelo) → EMB-07 → EMB-08 → EMB-09`. `EMB-03` wall 1d por descarga 22GB.
**Tabla modelos (9):** 3 EN (`bge-small-en-v1.5` 384 253MB, `all-MiniLM-L6-v2` 384 170MB, `bge-base-en-v1.5` 768 878MB) + 3 ES (`jina-es-v2-base` 768 2.2GB, `paraphrase-multilingual-MiniLM-L12-v2` 384 941MB, `distiluse-multilingual` 512 1.08GB) + 3 Combined (`multilingual-e5-small` 384 691MB **DEFAULT**, `bge-m3` 1024 3.47GB int8, `qwen3-embedding-8b` 4096 16GB **EXCEPCIÓN >3GB MTEB 75.1**). Ver `docs/plans/2026-08-28-embeddings-local.md` §4 para matriz completa.

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
| REVIEW-07 | Media | `.config/nextest.toml` profile `audit`: filtro referencia binarios inexistentes → parse failure bloquea TODA invocación nextest. Podar stale verificado con `cargo nextest list` | .config/nextest.toml | 🟢 | 🟡 Media | 🔴 Abierta — derivada de review-full-20260822 H01-CODE-001 |
| AUD-043 | Alta | Fix clippy `unused variable: ns` en closure `options_for` (`src/cli_server.rs:1302`, commit `abc4ec10`) que rompe `just verify` / pre-push gate / CI Fast Gate. Fix: renombrar parámetro a `_ns` (~2 min). Origen: audit-full-20260825-010607 | `src/cli_server.rs:1302` | 🟢 | 🔴 Alta | Pendiente |
| AUD-044 | Alta | Shim `MmapMut` (no-memmap2) sin write-back: `flush()` no-op hace que `compact_layout` pierda datos silenciosamente en builds nativos sin memmap2. Fix: flush real vía seek+write_all o `compile_error!` guard + test round-trip compaction con `--no-default-features`. Origen: audit-full-20260825-031011 Phase 2 | `src/storage/vfile_mmap.rs:130-141`, `src/storage/archive.rs:95-137` | 🟡 | 🔴 Alta | ✅ Completada (2026-08-25, batch core-fixes-research) — shim flush write-back + 4 tests |
| AUD-045 | Media | Clones de vector completo per-candidate en hot path IVF search (`centroid.clone()` y `entry.vector.clone()` en loops internos). Acción: medir baseline vs `canonical_p99` (Regla 9), luego A/B variante borrowed/slice en `calculate_similarity`; aceptar solo si p99 mejora. Origen: audit-full-20260825-031011 Phase 3 | `src/index/ivf.rs:250,275` | 🟡 | 🟡 Media | Pendiente |
| AUD-047 | Baja | Duplicación ~50 líneas del bloque match métrico (Cosine/Euclidean/SparseDot) en layer.rs, anidado peor desde f2c2141e. Acción: extraer helper `score_f32_pair(...)` compartido por ambos call-sites. Origen: audit-full-20260825-031011 Phase 4 | `src/index/search/layer.rs` | 🟢 | 🟢 Baja | ✅ Completada (2026-08-25) — metric_score closure, -35 lineas |
| REVIEW-10 | Alta | God-file `cli_server.rs` ~3800-4141 líneas (routing + RBAC + TLS + OTEL + tests inline) — blast radius total del server en un archivo. Split por concern bajo `src/server/`; congelar features nuevas ahí | src/cli_server.rs | 🟠 | 🔴 Alta | 🟠 Abierta — derivada de review-full-20260822 H06-ARCH-001 |
| FIND-22 | Alta | Formalizar en docs/operations/CI_POLICY.md las 3 exclusiones de tests del fast gate en dev-tools/verify.ps1 (deserialize_absurd_node_count, test_search_with_bizarre_text_query, test_malformed_payload_extremely_large — CATEGORY: RESOURCE-GUARD, documentadas inline pero sin entrada en la taxonomía CI_POLICY ni Issue tag flaky). Origen: auditoría dev-tools 2026-08-25 | dev-tools/verify.ps1, docs/operations/CI_POLICY.md | 🟢 | 🟡 Media | Pendiente |
| FIND-23 | Media | `vanta-http-map.ts` manda `namespace: item.namespace ?? ""` en ingest/get con namespace omitido → el server embebido rechaza ("Validation error on namespace: namespace must not be empty"); el mapping WASM sí defaulta `DEFAULT_NS` (inconsistencia WEB-04). En el web build embedded, IngestForm con namespace vacío falla (y su pre-check `get()` rompe antes). Origen: plan 2026-08-25-batch-desktop-ux-core#E2E-VISUAL (hallazgo e2e) | desktop/src/vanta-http-map.ts:93 | 🟢 | 🟡 Media | ✅ Completada (2026-08-25) — DEFAULT_NS en http-map + test |
| BND-08 | 🔴 P0 | **Pipeline npm release napi-rs** (H-01): nunca publicado (E404); crear workflow CI create-npm-dirs/artifacts/prepublish modelo LanceDB/napi.rs, 5 targets + musl futuro. Origen: research-vantadb-node-20260825 | .github/workflows/, antadb-node/package.json | 🔴 | 🔴 Alta | Pendiente |
| BND-09 | 🟡 | **Target linux musl** (H-04): Docker/Alpine sin cobertura; agregar arch64/x86_64-unknown-linux-musl a napi.targets cuando exista pipeline. Origen: research-vantadb-node-20260825 | antadb-node/package.json, CI | 🟢 | 🟠 Media | Pendiente |
| BND-10 | 🔴 | **Paridad API node vs python/MCP** (H-08): exponer versions/supersede/vacuum/compact_wal/purge_expired/count/delete_by_filter/similar_to_key/search_with_method/search_multi. Fraccionable en fases (memoria-lifecycle → maintenance → search-advanced). Origen: research-vantadb-node-20260825 | antadb-node/src/lib.rs, index.d.ts | 🔴 | 🟠 Media | Pendiente |
| BND-11 | 🟡 | **Tipado fuerte index.d.ts** (H-05): eliminar any en put/search/list/filter — tipos MemoryRecord/SearchRequest/ListOptions manuales o ts-rs. Origen: research-vantadb-node-20260825 | antadb-node/index.d.ts | 🟡 | 🟠 Media | Pendiente |
| BND-12 | 🟡 | **Cobertura tests node** (H-06): 8→~20 tests; cubrir search/explain_search/put_batch/capabilities/close-drain. Origen: research-vantadb-node-20260825 | antadb-node/tests/ | 🟡 | 🟠 Media | Pendiente |
| BND-13 | 🟡 | **docs/api/NODE_SDK.md** (H-07): doc completa + ejemplos por runtime; README ya creado (quickstart + matriz native-vs-wasm). Origen: research-vantadb-node-20260825 | docs/api/NODE_SDK.md | 🟢 | 🟡 Media | Pendiente |
| PERF-BENCH-01 | 🟡 | **Benchmark A/B vantadb-node nativo vs vantadb-ts WASM** (H-09): insert/search p99 + tamaño binario; decide posicionamiento (decisión tomada: native primario en Node condicionado a números). Regla 9. Origen: research-vantadb-node-20260825 | benches nuevos | 🟡 | 🟠 Media | Pendiente |
| FIND-24 | Alta | `VantaEmbedded::list` con ventana grande es lento en debug: list de 10_000 records in-memory ≈ 60-70s (~6.7ms/nodo get_many+convert) — el fan-out HTTP all-namespaces (`/api/v2/list` sin namespace, NS_CAP=10_000/ns por request) excede `REQUEST_TIMEOUT=30s` con >10k records y devuelve 408. Costo escala O(ventana total) por request; el fan-out re-lista TODO en cada página. Fix correcto: cursor cross-namespace server-side (requiere soporte SDK) + perf de `indexed_ids_by_namespace`/`get_many`. Origen: AUD-046 (medición debug 2026-08-25, máquina cargada; validar en release antes de priorizar) | src/sdk/api.rs:601-684, src/cli_server.rs (records_list fan-out) | 🟠 | 🔴 Alta | Pendiente |
| MCP-34b | Media | Tool MCP `snapshot_restore(name)` — wrapper del restore físico cuando S2-S4 de RES-02 aterricen (validación identifier + confirmación destructiva explícita). Prerrequisito: S1 (quiesce+flush en create_snapshot) y S2-S4 (core restore + tests) | docs/research/res02-backup-restore.md §3 | 🟢 | 🟡 Media | ⬜ Pendiente (prerrequisito core nuevo) |
| FIND-33 | Media | Snapshot filesystem NO captura el backend KV: Fjall/RocksDB abre en `storage_path` raíz (`init.rs:287`, hermano de `data_dir=storage_path/data` en `init.rs:298`) y `create_snapshot` solo imagea `data_dir` → la consistencia del reopen depende 100% del replay de `vanta.wal`; un snapshot tomado tras `compact_wal()` (que archiva segmentos WAL) pierde irrecuperablemente los datos que viven solo en el backend (metadata relacional/edges/checkpoint_seq). Fix requiere rediseño del layout snapshot (copiar backend o moverlo bajo data_dir) — escalado desde FIND-25 según stop condition del plan 2026-08-25. Hallazgo durante ejecución FIND-25 | src/storage/engine/init.rs:159-309 · src/storage/engine/mod.rs (create_snapshot) · docs/research/res02-backup-restore.md §1 | 🟠 | 🔴 Alta | Pendiente |
| ~~FIND-26~~ | Baja | ✅ RESUELTA (remove, 2026-08-25): `src/wal_archiver.rs` eliminado + export/feature `pitr` removidos + docs actualizados (FEATURES.md, EXPERIMENTAL_FEATURES.md, ADR-014 superseded). Decisión del lead basada en RES-02 §2b: PITR necesita base snapshot + replay (prerrequisito grande sin consumer); código conservado en git history (`git log --follow src/wal_archiver.rs`) | research res02-backup-restore.md §2b · ADR-014 | 🟠 | 🟡 Media | Completada |
| REVIEW-12 | Media | `api.rs` ~2300-2500 líneas aproximándose a god-file — SDK surface concentrada dificulta evolución `#[non_exhaustive]`. Refactor aditivo por dominio (memory/search/namespaces/admin), re-exportado vía sdk::api, sin break público | src/sdk/api.rs | 🟡 | 🟡 Media | 🟡 Abierta — derivada de review-full-20260822 H06-ARCH-002 |

| FIND-38 | Media | Ciclo Serialization (5 nodos): `get_string_field`↔`get_u64_field`↔`memory_record_from_node*`↔`VantaEmbedded.get` — consolidar helpers duplicados. Origen: codegraph-20260827-143245 Fase 1 | src/sdk/serialization/mod.rs, src/sdk/api.rs | 🟡 | 🟡 Media | Pendiente |
| FIND-40 | Media | Drift docs/api vs firmas reales: 13 archivos en `docs/api/` (200KB+) — verificar `EMBEDDED_SDK.md`, `PYTHON_SDK.md`, `HTTP_API.md` contra código actual. Origen: codegraph-20260827-143245 Fase 11 | docs/api/*.md | 🟡 | 🟡 Media | Pendiente |
| FIND-41 | Media | 6 clusters `src` fragmentados (cohesión 0.59-0.71): Leiden IDs 15,33,49,74,58,17 vs skills/desktop 0.97 — consolidar o documentar fronteras. Origen: codegraph-20260827-143245 Fase 1 | src/ (clusters Leiden) | 🟡 | 🟡 Media | Pendiente |
| FIND-42 | Media | Boundary `src → skills` (173 llamadas): core llama a skills/agentes (impeccable, etc.) — inversión de dependencia semántica. Origen: codegraph-20260827-143245 Fase 1 | src/ → .agents/skills/ | 🟡 | 🟡 Media | Pendiente |
| FIND-43 | Media | Ciclo CacheWarmer (3 nodos): builder pattern recursivo `new`→`with_config`→`with_config_and_cap` — aplanar a builder no recursivo. Origen: codegraph-20260827-143245 Fase 1 | src/cache_warmer.rs | 🟢 | 🟡 Media | Pendiente |
| FIND-44 | Alta | Sin ADRs registrados: proyecto sin Architecture Decision Records — crear ADR inicial (PURPOSE, STACK, ARCHITECTURE, PATTERNS, TRADEOFFS, PHILOSOPHY). Origen: codegraph-20260827-143245 Fase 12 | Proyecto entero | 🟠 | 🔴 Alta | Pendiente |
| FIND-45 | Media | `src → skills` violation semántica: dependencia core→agents skills (impeccable) — determinar si tooling leakage o diseño intencional. Origen: codegraph-20260827-143245 Fase 1/2 | src/ → .agents/skills/impeccable/* | 🟡 | 🟡 Media | Pendiente |
| FIND-46 | Media | Doc drift puntual (gate semver-checks): derivar a `cargo semver-checks` + review manual antes de release. Origen: codegraph-20260827-143245 Fase 11 | docs/api/ vs src/, vantadb-python/, vantadb-ts/ | 🟢 | 🟡 Media | Pendiente |
| FIND-47 | Baja | `handle_tools_call` complejidad 295 (dispatcher MCP): match gigante 20+ brazos, 8 scans en loops batch — no hotspot algorítmico; si crece, extraer sub-dispatchers. Origen: codegraph-20260827-143245 Fase 5 | vantadb-mcp/src/handlers/tools.rs:549 | 🟢 | 🟢 Baja | Pendiente |


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

---

## P25 - Exposición MCP/HTTP — Gaps del SDK no expuestos (2026-08-18)

> **Origen:** gap analysis del lead (2026-08-18): comparación API pública core SDK (`src/sdk/api.rs`, `VantaEmbedded` — put:206, get:406, list:545) vs 15 tools MCP (`vantadb-mcp/src/handlers/tools.rs`) vs 3 endpoints HTTP (`src/cli_server.rs`: `/health`, `/api/v2/query`, `/metrics`). Conclusión: el SDK tiene ~35 métodos públicos; las herramientas MCP cubren CRUD+búsqueda+grafos-IQL, pero **faltan capacidades de ciclo de vida de datos** que el MCP no puede invocar: purge TTL (crítico: AUD-045 habilitó TTL pero no hay forma de limpiar expirados), backup/restore, mantenimiento WAL/layout, delete batch por filtro, batch put, rebuild/recovery de índices. El HTTP server es deliberadamente mínimo (producto embedded-first; "server as primary boundary" diferido en skill) — no es bug, es decisión; solo se trackea aquí el MCP.
> **Nota:** NINGUNA tool MCP de la lista requiere cambio de API pública del SDK — son wrappers `handle_tools_call` sobre métodos ya públicos. Riesgo bajo, sin semver implications.
> **Actualización 2026-08-19 (Fase 3, D11):** la decisión "server as primary boundary diferido" fue **re-considerada por el usuario (D11)** — el REST completo del SDK ya está implementado (`/api/v2/*` en `src/cli_server.rs`, ADR-026). El HTTP server ya NO es mínimo: ~27 endpoints v2 (health, records CRUD+batch+versions+delete_by_filter, list con cursor, search, autocomplete, query, audit, export/import, graph bfs/dfs/degree/pagerank/centrality, maintenance purge/compact/flush/rebuild-index, threads, snapshots) + dashboard embebido `/dashboard`. Los gaps MCP-16..26 de esta sección siguen válidos SOLO para el canal MCP (el REST no sustituye las tools del agente).

| ID | Descripción (Gap → Acciones → Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `MCP-35` | **Fallback HTTP automático: N instancias MCP sobre la misma BD** — hoy `vanta-cli server --mcp --db X` muere con exit 1 si otra instancia tiene el lock exclusivo del engine (single-writer por diseño; incidente 2026-08-25: 2 sesiones OpenCode, la segunda sin tools). Acciones: (1) la primera instancia escribe discovery file en el data dir (`.vanta.server.json` con `{pid, http_port}`) y abre listener HTTP local; (2) instancias subsecuentes detectan "Database busy" al abrir el engine, leen el discovery file, verifican que el PID vivo responda `/health`, y arrancan en **modo proxy**: exponen las MISMAS tools MCP pero cada call se resuelve vía HTTP (`/api/v2/*`, reusando ServerConnection/auth token) contra el server dueño del lock; (3) si el discovery file apunta a un PID muerto → limpiar lock y abrir embebido normal. Delegar diseño de discovery/lock a vanta-arch (DISCOVERY), implementación a vanta-worker. Criterios: 2+ sesiones OpenCode simultáneas comparten memoria; crash del server dueño no corrompe; tools parity 1:1 con modo embebido | `src/cli_server.rs` (modo mcp), `vantadb-mcp/src/handlers/`, `src/config.rs` | 🟡 2-4d | 🔴 Alta (bloquea multi-sesión real) | ⬜ Pendiente |

---

## P26 - Exposición capa cognitiva `vanta-memory` + agentic restante vía MCP (2026-08-23)

> **Origen:** auditoría de integración total OpenCode↔VantaDB (2026-08-23): el motor core quedó ~100% expuesto tras cerrar P25 (MCP-16..29), pero la capa cognitiva (`vanta-memory`: scenes, context engine, compresión MMD) y las APIs agénticas restantes (threads CRUD, axiom-write, snapshots create/restore) no tienen tool MCP. **Fuera de alcance por diseño (NO trackear aquí):** `vanta-proxy` (infraestructura proxy LLM), módulo LLM interno (`llm.rs`, feature-gated para ingesta), HTTP REST completo (embedded-first), desktop/web (UIs).
> **Nota:** los handlers del gateway ya existen como funciones puras sobre `&VantaEmbedded` (`vanta-memory/src/gateway/knowledge_handlers.rs`) — las filas MCP-30/31 son wrappers; MCP-32..34 exponen APIs SDK/core ya públicas.

| ID | Descripción (Gap → Acciones → Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `MCP-34` | **G19 🟡 Snapshots físicos: create/restore vía MCP** — Problema: `list_snapshots` lista snapshots Fjall pero no hay create/restore vía MCP (backup físico puntual). Acciones: (1) verificar APIs de snapshot en `StorageEngine` (`storage/engine/mod.rs:523` tiene list; buscar create/restore); (2) tools `snapshot_create`/`snapshot_restore` si existen como métodos públicos; (3) restore exige validación de path (trust boundary — sanitizar, solo dentro del data dir); (4) tests + docs. Resultado: backup físico puntual ejecutable desde el agente. **Re-scope 2026-08-25 (research mcp-research-20260825.md):** `snapshot_create` YA existe (`vantadb-mcp/src/handlers/tools.rs:466`) — falta SOLO `snapshot_restore` + anti path-traversal en `validate_identifier` (no bloquea `/ \ . ..`, lección 2026-08-25 snapshot tools). | `src/storage/engine/mod.rs`, `vantadb-mcp/src/handlers/tools.rs` | 🟢 | 🟡 | ❌ Pendiente |
| `MCP-37` | **🔴 P0 Perfiles de tool surface** — ~75 tools exceden el cap de ~40 tools de Cursor (forum.cursor.com/t/108637) y presionan contexto en todos los clientes. Acciones: (1) env `VANTADB_MCP_PROFILE=memory|dev|full` filtrando qué tool definitions registra `handle_tools_list`; (2) default sensato por caso de uso documentado; (3) tests por perfil. Resultado: cabe en clientes con caps sin recortar funcionalidad. Ref: docs/reviews/mcp-research-20260825.md §6 P0-B | `vantadb-mcp/src/handlers/tools.rs`, `vantadb-mcp/src/config.rs` | 🟡 | 🔴 | ❌ Pendiente |
| `MCP-39` | **🟡 Output budgeting** — Claude Code trunca a 25k tokens hard, OpenCode a 2000 líneas/50KB no configurable (#22565); `search_multi`/`memory_list` pueden excederlo silenciosamente. Acciones: (1) byte budget configurable por respuesta con truncado explícito + `next_cursor`; (2) docs del límite por cliente. Ref: docs/reviews/mcp-research-20260825.md §2,§6 P1-E | `vantadb-mcp/src/handlers/tools.rs` | 🟢 | 🟡 | ❌ Pendiente |
| `MCP-40` | **🟡 Registro en el ecosistema MCP** — no estamos en registry.modelcontextprotocol.io (catálogo canónico desde 2026), ni glama.ai/smithery. Acciones: (1) `server.json` manifest; (2) PR/submission al registry; (3) listings glama/smithery con annotations correctas (depende MCP-38). Ref: docs/reviews/mcp-research-20260825.md §6 P1-F | `docs/`, repo metadata | 🟢 | 🟡 | ❌ Pendiente |
| `MCP-41` | **🟡 Memoria conversacional auto-consolidada** — la brecha competitiva vs mem0/graphiti/cognee (todos extraen y consolidan memoria de conversaciones vía LLM); nuestro hueco natural: hacerlo feature-gated reusando scenes (MEM-13/14) sin LLM key obligatoria. Acciones: DISCOVERY primero (vanta-arch) sobre diseño extract→consolidate→recall local-first. Ref: docs/reviews/mcp-research-20260825.md §7 apuestas | `vanta-memory/src/core/scene/`, `vantadb-mcp/src/` | 🔴 1-2s | 🟡 | ⬜ Pendiente (requiere DISCOVERY) |
| `FIND-24b` | **Docs drift detectado por research MCP** — (a) `docs/api/MCP.md:12` enlaza `skills/vantadb-mcp/SKILL.md` relativo → `docs/skills/vantadb-mcp/SKILL.md` NO existe; (b) los dos SKILL.md reales (`.opencode/skills/vantadb/` hash 155E93…, `.opencode/skills/vantadb-mcp/` hash DF1A68…) difieren — drift entre skill canónica y variante MCP; (c) conteo de tools desactualizado (72 vs ~75 contados). Ref: docs/reviews/mcp-research-20260825.md §4.5 | `docs/api/MCP.md`, `.opencode/skills/vantadb*/SKILL.md` | 🟢 | 🟢 | ❌ Pendiente |
| `PY-01` | **Paridad `graph_bfs_filtered`** — node (`vantadb-node/src/lib.rs:326-343`) y ts exponen traversal con filtro de labels/time_range; el binding Python NO (`vantadb-python/src/lib.rs:314-325` GraphClient sin filtered). Agregar método flat + forward en GraphClient + stub .pyi + test. Contrato verificable: `db.graph_bfs_filtered(roots, max_depth, direction, filter={"labels":[...], "time_range":[f,t]})` paridad de resultado con node. Ref: docs/reviews/research-vantadb-python-20260825.md H-04 | `vantadb-python/src/lib.rs`, `vantadb_py/*.pyi`, `tests/` | 🟡 | 🟡 | ❌ Pendiente |
| `PY-02` | **Benchmarks reproducibles del SDK Python** — no hay números publicados del SDK (Regla 11); competidores publican los suyos. Acciones: (1) bench canónico python (insert/search P50/P95/P99, dataset determinístico seed 42, estilo canonical_p99); (2) sección en `docs/operations/BENCHMARKS.md` con comando exacto + entorno; (3) recién entonces claims comparativos vs chroma/lance. Contrato verificable: BENCHMARKS.md tiene sección Python SDK con comando reproducible citado. Ref: docs/reviews/research-vantadb-python-20260825.md H-06 | `benchmarks/`, `docs/operations/BENCHMARKS.md` | 🟡 | 🟡 | ❌ Pendiente |
| `PY-03` | **Consolidar identidad de import en `vantadb`** (decisión HITL INV-DECIDE Q7 sobre H-09): `vantadb` es el nombre documentado en README/ejemplos; mantener alias interno `vantadb_py` con `DeprecationWarning` por 1 minor y remover después. Contrato verificable: `import vantadb` es la única vía documentada en README/docs/api/PYTHON_SDK.md; alias emite warning y tiene timeline de remoción anotada. Ref: research-vantadb-python H-09 | `vantadb-python/vantadb_py/__init__.py`, `README`, `docs/api/PYTHON_SDK.md` | 🟢 | 🟢 Baja | ❌ Pendiente |
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
| `MEM-59` | **Recall MCP público** (gap #4): exponer el recall cognitivo como tools MCP `memory_recall(query, scope, top_k)` + `memory_search(filters)` — hoy el recall solo es automático vía proxy/IPC interno y un Claude Code/Cursor externo NO puede preguntar la memoria. Criterio: cliente MCP externo consulta y recibe hits relevantes con scopes session/agent/team | `vantadb-mcp/src/handlers/tools.rs`, `vanta-memory/src/core/hooks/auto_recall.rs` | 🟡 | 🔴 | ⬜ Pendiente |
| `MEM-60` | **Lifecycle heat+decay L1 + contradicciones** (gaps #1+#2, enfoque elegido): extender heat de escenas a records L1 — lo usado sube score, lo no usado decae y se poda tras umbral (patrón memify); en escritura, detectar contradicción → invalidar/sustituir la vieja conservando provenance (nunca borrado silencioso). Criterios: memoria sin uso N días con heat bajo deja de salir en recall; contradicción nueva desactiva la anterior rastreablemente | `vanta-memory/src/core/record/`, `core/scene/scene_index.rs` | 🔴 | 🔴 | ⬜ Pendiente |
| `MEM-61` | **Dreaming — consolidación idle** (gap #3): job en downtime (idle ≥X min o cierre de sesión) que consolida L0/L1 crudo → learned context: fusiona duplicados residuales, resuelve contradicciones pendientes, normaliza fechas relativas→absolutas. Modelo LLM configurable más potente para el job (sleep-time tiering, patrón Letta/OpenAI/Anthropic). El store original jamás se muta — store consolidado nuevo revisable/descartable | `vanta-memory/src/services/pipeline_worker.rs`, nuevo `core/dream/` | 🔴 | 🟠 | ⬜ Pendiente |
| `MEM-62` | **Export markdown git-friendly** (gap #7): `vanta-cli memory export --format md --scope agent|team` genera árbol versionable (archivo por escena/tema, frontmatter metadata) para memoria de equipo en git; reimport idempotente vía `vanta-seed`. Criterio: round-trip export→git clone→import sin pérdida ni duplicados | `src/cli.rs`, `vanta-memory/src/seed/` | 🟡 | 🟡 | ⬜ Pendiente |
| `MEM-63` | **Quick-win docs+embeddings**: corregir doc stale `auto_recall.rs:69-73` (dice que embeddings "degradan hasta wirear"; MEM-47 ya implementó el hook) + embeddings auto-on cuando hay provider configurado (chars-fallback solo sin provider) | `vanta-memory/src/core/hooks/auto_recall.rs`, Cargo features | 🟢 | 🟢 | ⬜ Pendiente |
| `MEM-64` | **Skills versionadas + CompactionReport**: usar `skill_versions` del core en el pipeline (hoy content-hash upsert sin historial) + persistir CompactionReport por sesión (hoy solo el IntegratedContext final en `__assembled`) | `vanta-memory/src/core/skill/conversation_add/`, `context_engine/` | 🟡 | 🟡 | ⬜ Pendiente |
| `MEM-65` | **Telemetría por capa + pLimit real**: instrumentar latencias L1/L2/L3/recall en PipelineWorker (MEM-34 cubrió solo métricas core) + hacer `global_llm_concurrency` real (pLimit; hoy es techo documental, `ingest/mod.rs:9-12`) | `vanta-memory/src/services/pipeline_worker.rs`, `src/ingest/mod.rs` | 🟡 | 🟡 | ⬜ Pendiente |
| `MEM-66` | **claimStaleTasks (recuperación multi-worker)**: port del TDAM original no porteado (`pipeline_worker.rs:12-13` lo documenta) — worker muerto a mitad de tarea no debe atascar la tarea hasta el TTL; otro worker la reclama | `vanta-memory/src/services/pipeline_worker.rs`, `utils/local_backend.rs` | 🟡 | 🟠 | ⬜ Pendiente |
| `MEM-67` | **TokenEstimator auto-detección**: usar tiktoken (precise-tokens) automáticamente si la dependencia está compilada; chars/3 como fallback — sin invertir el default manualmente (CJK/código subestimados hoy) | `vanta-memory/src/context_engine/token_estimator.rs` | 🟢 | 🟢 | ⬜ Pendiente |
| `MEM-68` | **Gate opcional de aprobación de capturas** (gap #6): config `capture_approval=off|on`; en `on`, las memorias extraídas van a cola pendiente y un comando/tool `memory_approve/reject` las publica o descarta (patrón Cursor). Default off (filosofía never-block intacta) | `vanta-memory/src/core/record/l1_writer.rs`, MCP tools | 🟡 | 🟠 | ⬜ Pendiente |
| `MEM-69` | **Batch extracción costo-reducida**: agrupar split+dedup en menos llamadas LLM por flush (patrón Memobase: batch fijo −40-50% tokens) sin perder quality gate | `vanta-memory/src/core/record/l1_extractor.rs` | 🟡 | 🟢 | ⬜ Pendiente |
| `MEM-70` | **Benchmarks públicos LongMemEval-S + LoCoMo**: harness de evaluación contra vanta-memory y publicación en `docs/operations/BENCHMARKS.md` (Regla 11: bench archivo + comando reproducible). Referencia mercado: SuperMemory 81.6%, Hindsight 94.6% self-report | nuevo `evals/memory_bench.py` o Rust harness, `docs/operations/BENCHMARKS.md` | 🟡 | 🟡 | ⬜ Pendiente |

---

> **Origen:** follow-up del fix `8c8eef23` (vectores Binary insertables/recuperables por `get()`). El fix resolvió el contrato insert→get en memoria; la persistencia on-disk de vectores no-F32 queda pendiente (formato).

| ID | Descripción (→ Resultado) | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `CORE-01` | **Persistencia on-disk de vectores Binary (y no-F32) en vstore** — `write_node_to_vstore` (`ops.rs:59`) escribe `vector_len=0` y NO persiste datos para `Binary`/`Turbo`; `get()` rescata el vector desde el HNSW (fix `8c8eef23`), pero tras reopen + `rebuild_hnsw_from_vstore` el Binary original se pierde (header `vector_len=0` → rebuild lo lee como sin vector). Definir codificación on-disk (flag de formato en `DiskNodeHeader` o convención sobre `vector_len`) + escritura/lectura en `write_node_to_vstore`/`get()`/`get_with_snapshot`/rebuild + migración/versionado. Gate: ADR de formato antes de implementar. | `src/storage/ops.rs:59`, `src/node/disk.rs`, `src/storage/archive.rs:359`, `src/storage/engine/get.rs`, `src/storage/engine/txn.rs` | 🟡 | 🟡 | ❌ Pendiente |
| `CORE-02` | **Integrar PITR al engine (salir de experimental)** ⚠️ BLOQUEADA por FIND-26 (2026-08-25): `wal_archiver.rs` fue REMOVIDO (dead code) — restaurar desde git history (`git log --follow src/wal_archiver.rs`) antes de cualquier wiring; re-evaluar si sigue teniendo sentido. Decisión original del owner 2026-08-25: integrar, no congelar. `wal_archiver.rs` (feature `pitr`, ADR-014) es API standalone self-tested pero NO está wired a StorageEngine/SDK. Acciones: (1) DISCOVERY (vanta-arch): decidir puntos de enganche - hook en recovery/open vs subcomando CLI (`vanta-cli pitr`) vs endpoints SDK/HTTP; (2) implementar wiring + tests de restore point-in-time; (3) promover flag de ⚠️ experimental a 🟡 opt-in en FEATURES.md y cerrar la decisión pendiente de decisions.md 2026-08-25. Criterio: restaurar a timestamp T en DB temporal y verificar records coherentes | `src/wal_archiver.rs` (en history), `src/storage/engine/mod.rs`, `src/cli.rs`, ADR-014 | 🔴 | 🔴 | ❌ Pendiente |




---

## GOV - Gobernanza Documental — corrección integral post-auditoría (2026-08-22)

> **Origen:** auditoría documental completa docs/reviews/auditoria-documentacion-2026-08-21.md (Volumen I+II+Addendum, salud 6.5/10) + **decisiones del owner D1-D14** registradas en su sección final. **Plan de ejecución:** docs/plans/2026-08-22-doc-governance-plan.md (formato campaign, triage 30 DO / 2 DEFER). Task files se crean bajo demanda vía /pipeline task GOV-XX. Wave B es bloqueante del Show HN.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
> **CAMPAÑA COMPLETADA 2026-08-22: 29 ✅ · 1 ⬛ (A1 stop condition, fallback aplicado) · 0 failed.** Registro completo: docs/progreso/campanas/doc-gobernanza-gov.md + plan file §Estado de ejecución. Tickets derivados vivos abajo.


## GOV-TK — Tickets derivados de la campaña GOV (2026-08-22)

| ID | Descripción | Prio | Fuente |
|----|-------------|------|--------|
| GOV-TK1 | **CLI backup verification**: subcomando `verify` o flag `--dry-run` en Restore + `doctor --fix` - el runbook DR nuevo depende conceptualmente | 🟢 | D4b/B2 |
| GOV-TK2 | **Release** para que el binario MCP exponga las 18 tools skill_*/code_*/wiki_* (skill ya documenta 33; binario publicado tiene 15) | 🔴 | B6 |
| GOV-TK3 | Drift yaml↔real ×3: gramática IQL case del yaml vs parser UPPERCASE · GraphTraversalBody (roots numéricos + max_depth requerido) · search en DB fresca requiere rebuild-index previo | 🟠 | B5 |
| GOV-TK4 | Re-medición coverage local: llvm-cov ICE rustc 0xc0000409 Windows (probar -j 2 limpio post-fingerprint-clean o CI artifact) | 🟠 | A1 |
| GOV-TK5 | Split Manual Estratégico según recomendación F2 (negocio→docs/business/ con banner snapshot; estado técnico fuera; archivar monolito) | 🟠 | F2/D-decisión |
| GOV-TK7 | put_batch metadatas solo-str: alinear doc-tutorial vs API o ampliar coercion | 🟡 | B3 |
| GOV-TK8 | Benchmarks: mejorar/probar/documentar (insumo: docs/benchmarks/_run_stdout.md se conserva como evidencia de corrida cruda) | 🟡 | owner E1 |
| GOV-TK9 | URL `vantadb-examples` repo distinto en pilot-onboarding-checklist:51 - verificar si existe | 🟢 | B3 |

> Ticketeados aparte con decisión previa: ACID 4a-4d (post-launch Fase A, D14) · release triage semver 0.6.0 (D5, diferido) · MKT-18h wheels ARM64 + MKT-18f adapters (confirmados live por GOV-A5).

| `BND-07` | **Discord invite inválido + vantadb.dev sin DNS** (GOV-F1 🔴×2) — requieren acción externa del owner: crear invite nuevo de Discord y configurar DNS de vantadb.dev; luego actualizar README/CONTRIBUTING/SECURITY con los valores reales. Registrado en auditoría raíz pública GOV-F1 (commit dc3775ef). | README.md, CONTRIBUTING.md, SECURITY.md (externo al repo) | 🟡 | 🟠 | ⏳ Externo owner |

---

## P32 — Reviews de Módulos (campaña 14 reportes, 2026-08-23)

> **Origen:** campaña de deep-review con 14 sub-agentes sobre los 12 módulos del repo + análisis transversal. Reportes completos en `docs/reviews/modulos/*.md` (core.md, vantadb-mcp.md, vantadb-server.md, vantadb-python.md, vantadb-ts.md, vantadb-wasm.md, vantadb-node.md, vanta-memory.md, vanta-proxy.md, providers.md, integrations.md, benches.md, benchmarks.md, cross-modulos.md). Scores: memory 8.5 · core 8.3 · mcp 8.3 · proxy 8.0 · server 7.5 · ts/wasm/benches 7.0 · integrations/benchmarks/cross 6.5 · python 6.5 · providers 5.0 · node 4.5.
> **Regla:** cada fila referencia su reporte fuente; los reportes llevan sección "Trazabilidad Backlog" con el MOD-ID por hallazgo. Duplicados ya trackeados NO recreados (CORE-01/02, REVIEW-06..20, MKT-18f, MCP-24/28/29, DESKTOP-28..39).

| ID | Módulo | Sev | Hallazgo → Acción | Referencia | Estado |
|----|--------|-----|-------------------|------------|--------|
| `MOD-05` | core | 🟢 | Deprecar `InMemoryEngine` hacia StorageEngine in-memory: elimina clase de bug MOD-01 y ~850 líneas | `engine.rs:72` · core.md R5 | ❌ Pendiente |
| `MOD-15` | server | 🟢 | Nits agrupados: middleware.rs re-export redundante, feature sysinfo vacía, main.rs abre engine raw sin comentario, falta constructor ServerState para tests | `middleware.rs:1`, `Cargo.toml:33` · server.md | ❌ Pendiente |
| `MOD-22` | vantadb-ts | 🔴 | Tipos grafo ficticios: `GraphBfsResult{visited,levels,path}` vs wire format real (`u128[]` plano); blind-cast `as GraphBfsResult` + test que afirme shape real | `vantadb-ts/src/types.ts:208-212`, `vantadb-ts/src/vantadb.ts:1094` · ts.md R4#1 | ❌ Pendiente |
| `MOD-23` | vantadb-ts | 🟠 | `NativeVantaDB._native` captura solo throws síncronos; rechazos async escapan sin envolver en `VantaError` | `vantadb-ts/src/native.ts:89-95` · ts.md R4#2 | ❌ Pendiente |
| `MOD-24` | vantadb-ts | 🟡 | Nits agrupados TS: semántica distance/score JSDoc, guard type-lie (`validateVector` asserts Float32Array), duplicación `_mapRecord`/`_buildSearchRequest`, ejemplos JSDoc que no compilan, dep `pkg/` commiteada | `vantadb-ts/src/guards.ts`, `vantadb-ts/src/{vantadb,native}.ts` · ts.md R4#3–#10 | ❌ Pendiente |

---

## P33 — Developer Experience de SDKs (evaluación usuario-facing, 2026-08-23)

> **Origen:** evaluación DX solicitada por owner sobre superficies que tocan usuarios (instalación, quickstart, SDKs Python/TS/WASM, docs/api). Hallazgos verificados contra disco hoy. Formato `prompts/findings.md`: fila FIND-* con ref de origen.
> **Regla:** duplicados parciales referencian su fila MOD existente; no se recrean.

| ID | Sev | Hallazgo → Acción | Referencia | Estado |
|----|-----|-------------------|------------|--------|
| `FIND-11` | 🟢 | Rutas alternativas sin pulir: `desktop/` sin README ni instalador público (no-programador sin ruta verificable); sin hooks/ejemplos React ni guía SSR; bundle .wasm 1.3MB sin doc de estrategia lazy-load; confusión potencial `vantadb-node` vs `vantadb` en npm | `desktop/` (glob *.md vacío), `vantadb-ts/dist/vantadb_wasm_bg.wasm` | ❌ Pendiente |
| `FIND-17` | 🟢 | Identidad de marca inconsistente: repo GitHub `ness-e/Vantadb` vs crate/npm `vantadb` vs PyPI `vantadb-py` vs dominio comprado sin DNS. Auditar consistencia de nombres en todos los artefactos públicos y decidir convención única pre-launch | URLs en pyproject.toml, package.json, README badges | ❌ Pendiente |
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
| `RES-01` | 🟡 | **ACID Phase 4a: WAL v2 con `WalRecord::Prepare`** + reordenar commit point (keystone de rollback multi-capa; habilita errores truthful y MVCC stamps que sobreviven restart). Diseño completo con acceptance criteria por fase (4a-4d) ya escrito. Delegar a vanta-arch | `src/wal.rs` (hoy `WAL_FORMAT_VERSION=1`, sin Prepare — verificado 2026-08-25) · Origen: `docs/research/ACID_ROLLBACK_DESIGN.md` | ⬜ Pendiente |
| `RES-02` | 🟡 | **Separar binario `chaos_failpoints`** (race global de failpoints, flaky local) **+ crear `crash_kill_recovery.rs`** (kill real a mitad de escritura, fsync falso, concurrencia+kill). Plan completo en FND-15 (items 01-05); los archivos no existen hoy. Delegar a vanta-chaos | `tests/storage/` · Origen: `docs/research/FND-15-crash-recovery-verificacion.md` | ⬜ Pendiente |

### 🟡 Media

| ID | Effort | Descripción | Archivos / Origen | Estado |
|----|--------|-------------|-------------------|--------|
| `RES-03` | 🟡 | **Canal multi-consumidor en ingestion pipeline**: reemplazar `Arc<Mutex<mpsc::Receiver>>` por async-channel/flume (contención serializada; única instancia sospechosa del inventario FND-19) | `src/ingestion.rs:72` (patrón intacto — verificado 2026-08-25) · Origen: `FND-19-arc-mutex-inventario.md` | ⬜ Pendiente |
| `RES-04` | 🟡 | **Phrase queries end-to-end**: condición `TextMatch` en parser IQL + tokenización literal de frases (sin stemming/stopwords) + highlight de frase completa en snippets. Enforcement base ya implementado en `lexical_search`; faltan estos 3 gaps | `src/iql/`, `src/sdk/search/` · Origen: `INV-009-phrase-queries-term-positions.md` | ⬜ Pendiente |
| `RES-05` | 🟢 | **PY-4: context manager síncrono** `__enter__/__exit__` en pyclass VantaDB (~10 líneas; hoy un `with` no hace flush del WAL — riesgo durabilidad). El wrapper async ya tiene `__aenter__`; falta el sync | `vantadb-python/src/lib.rs`, `vantadb_py/__init__.py` · Origen: `FND-05-sdk-idiomatico.md` | ⬜ Pendiente |
| `RES-06` | 🟡 | **Semántica de scores oficial**: documentar scoring (RRF/cosine/BM25) en `docs/api/` + resolver drift zero-norm cosine entre core Rust y `vantadb.ts`. Grep docs/api "score semantics/zero-norm" → 0 hits (2026-08-25) | `docs/api/`, `vantadb-ts/src/vantadb.ts` · Origen: `FND-06-core-bindings-boundaries.md` (H1+H3) | ⬜ Pendiente |
| `RES-07` | 🟡 | **Calibrar `rss_threshold`** (F2: recalibrar `DEFAULT_RSS_THRESHOLD=0.80` con medición real) + bench full-scale `[10k..100k]` (F3) | `src/config.rs:22`, benches memory-budget · Origen: `FND-01-memory-budget.md` (follow-ups F2/F3) | ⬜ Pendiente |
| `RES-08` | 🟢 | **Benchmark delete-masivo antes de rediseñar DashMap sweep** (H4): medir contención real del sweep en path de deletes; decidir rediseño solo si la medición lo justifica | `src/storage/engine/maintenance.rs` · Origen: `FND-02-multi-index-locks.md` (H4) | ⬜ Pendiente |
| `RES-09` | 🟡 | **Trackear roadmap post-launch huérfano** (investigado con archivo:línea, sin filas): WAL async ingest (10-100×), query planner con optimizaciones reales, DiskANN disk-I/O real. Agregar como filas a P24 o sub-fase propia | P24 / `docs/research/investigacion-equipo-2026-08-09.md` §roadmap | ⬜ Pendiente |

### 🟢 Baja / proceso

| ID | Effort | Descripción | Archivos / Origen | Estado |
|----|--------|-------------|-------------------|--------|
| `RES-11` | 🟢 | **Job rustdoc en CI**: `cargo doc --no-deps --workspace` + artifact. Grep `.github/` "cargo doc" → 0 matches (2026-08-25). API reference actualizada para adoptantes pre-docs.rs, costo mínimo | `.github/workflows/` · Origen: `FND-17-api-reference-docs-as-code.md` (Fase 1) | ⬜ Pendiente |
| `RES-12` | 🟢 | **Touch targets ≥44px restantes** (~20 componentes web: navbar `h-9`, close buttons `h-7`, footer text-only). Solo los 3 severos fueron corregidos con `size-11`. Delegar a vanta-worker (web) | `web/src/components/*` · Origen: `INV-015-touch-targets-44px.md` | ⬜ Pendiente |
| `RES-13` | 🟢 | **Activar pre-push hook git real** (template existe; `.git/hooks/pre-push` NO existe ni husky/lefthook — verificado 2026-08-25). Gate mecánico hoy manual/saltable | `.git/hooks/` o lefthook/husky · Origen: `gap-02-engineering.md` (P1-7) | ⬜ Pendiente |
| `RES-14` | 🟠 | **Review por segundo agente obligatorio para tareas 🔴** (process-change): diagnosticado como *la falla más grave* del sistema de agentes (P2-1/P2-3 gap-02); no hay fila que lo trackee. Wiring: exigir `task(vanta-review)` antes de COMPLETED en tareas rojas | `.opencode/task-system/prompts/task.md`, workflows · Origen: `gap-02-engineering.md` | ⬜ Pendiente |
| `RES-15` | 🟢 | **Institucionalizar meta-001 B/C**: micro-ADR obligatorio en cierres WONTFIX/DEFER + separar backlog negocio/técnico. Solo la recomendación A quedó implementada; B/C sin evidencia en `.opencode/rules/` (grep 0 hits 2026-08-25) | `.opencode/rules/`, `docs/Backlog.md` · Origen: `meta-001-root-cause-analysis.md` | ⬜ Pendiente |

### Decisiones de producto (antes de código)

| ID | Effort | Descripción | Origen | Estado |
|----|--------|-------------|--------|--------|
| `DEC-01` | 🟠 | **Session layer VantaDB MCP: go/no-go y alcance.** Roadmap de 4 fases propuesto (session cache, Claude Code plugin, sync/improve, lesson extraction) pero las 5 open questions siguen abiertas. Decidir vía ADR antes de escribir código | `COGNEE_EVALUATION.md`; research `docs/research/res03-session-layer-gonogo.md` | ✅ Resuelta por research (2026-08-25): **defer-as-scoped** — F1 no-go (threads/scenes/genlog ya lo cubren), F2 defer docs-only (guía conexión Claude Code), F3/F4 no-go (sync auto requiere benches Regla 9). Owner escribe ADR citando el doc |
| `DEC-02` | 🟠 | **Billing/quota CreditCalculator en server mode** (TDAM #9, diferido explícitamente fuera de F1-F7 y nunca trackeado). Decisión previa requerida: UNA calculadora (÷1000 vs ÷10000). Habilita multi-usuario/VantaDB Pro | `tdam/SYNTHESIS.md` §9, `tdam/09-deploy-usage.md` | ⬜ Pendiente |

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
| `SRV-02` | **Tracing-id por request** (`x-request-id`/`x-tracing-id`/`traceparent`, qdrant v1.18) correlacionado en audit log + logs de request middleware | `src/cli_server.rs:860-908` (middlewares), `src/audit.rs` | 🟢 | 🟢 Baja | ⬜ Pendiente |
| `SRV-03` | **Drift de distribución**: registro del módulo decía "crates.io" pero el crate es `publish = false` (celda ya corregida en el registro 2026-08-25); verificar que README/docs de instalación apunten a GitHub Release binaries | `docs/api/HTTP_API.md`, README instalación, `.opencode/references/research-modules.md:18` | 🟢 | 🟢 Baja | ⬜ Pendiente |
| `SRV-04` | **Multi API keys + rotación sin downtime** (patrón qdrant `alt_api_key` v1.17): el mapa token→role ya existe (`token_role_map`); falta soporte config para N keys simultáneas + ventana de rotación (key vieja+nueva activas) | `src/cli_server.rs:455-471` (`ServerState`), `src/config.rs` (`RbacConfig`) | 🟡 | 🟠 Media-Alta | ⬜ Pendiente |
| `SRV-05` | **RBAC scoping por namespace**: `Permission` hoy es solo por método HTTP (`cli_server.rs:718-740`); agregar read/write por namespace (patrón qdrant per-collection r/w v1.9, weaviate roles) | `src/rbac.rs`, `src/cli_server.rs` (`auth_middleware`), handlers | 🟡 | 🟠 Media-Alta | ⬜ Pendiente |
| `SRV-06` | **OIDC/JWT authentication** (estratégica enterprise): requisito de facto para "equipos" (weaviate OIDC nativo, qdrant JWT RBAC HS256 offline). **DISCOVERY primero (vanta-arch):** jsonwebtoken HS256 offline vs OIDC discovery, alcance mínimo viable sobre `auth_middleware` | `src/cli_server.rs:633-773`, `src/config.rs`; research §2 | 🔴 | 🟡 Media | ⬜ Pendiente (requiere DISCOVERY) |
| `SRV-07` | **Imagen Docker oficial + compose** — canal #1 de adopción self-hosted (los 4 competidores la tienen). Incluir variante unprivileged (patrón qdrant `-unprivileged`) y wiring al workflow RELEASE Binaries | nuevo `Dockerfile`, `docker-compose.yml`, `.github/workflows/` | 🟡 | 🟠 Media-Alta | ⬜ Pendiente |
| `SRV-08` | **Guía hardening + posicionamiento "local-first seguro por default"**: comparativa honesta vs qdrant/weaviate/milvus/marqo citando fuentes oficiales — somos únicos con rate-limit nativo fail-closed + refuse-to-start guard (FIND-07) en la tabla | `docs/operations/`, `docs/api/HTTP_API.md`; research §2 + H-11 | 🟢 | 🟡 Media | ⬜ Pendiente |

---

## P41 - Investigación INV-vantadb-ts (2026-08-25)

> **Origen:** `/research vantadb-ts` → `docs/reviews/research-vantadb-ts-20260825.md` (score 7.2/10, apéndice H-01..H-14). Decisiones HITL 2026-08-25: 13 filas nuevas TS, 0 wontfix. Duplicados ya trackeados NO recreados: H-01=`MOD-22`, H-02=`MOD-23` (filas restauradas en P32 el mismo día — la ausencia era el hallazgo H-05, aplicado inline), H-03 solapa=`RES-06`. Quick wins (TS-02/05/06/07/08) → plan `docs/plans/2026-08-25-research-vantadb-ts-quickwins.md`.

| ID | Descripción | Archivos clave | Esfuerzo | Prioridad | Estado |
|---|---|---|---|---|---|
| `TS-01` | **Corregir tipos de grafo ficticios**: alinear `GraphBfsResult`/`GraphDfsResult`/`GraphTopologicalSortResult` con el wire format real del binding WASM + test que afirme shape real (no `toBeDefined()`) (=MOD-22/H-01) | `vantadb-ts/src/types.ts:208-223`, `vantadb-ts/src/vantadb.ts:1094`, `vantadb-wasm/src/lib.rs:1353+` | 🟡 | 🔴 Alta | ⬜ Pendiente |
| `TS-02` | **Fix `_native` async**: convertir a `async` con `await` para envolver rechazos de promesas en `VantaError` (=MOD-23/H-02) — fix 3 líneas + test | `vantadb-ts/src/native.ts:89-95` | 🟢 | 🔴 Alta | ⬜ Pendiente |
| `TS-03` | **Verificar semántica score/distance contra core y unificar docs** una sola vez en core (¿`h.score` es distancia o similitud?) — resolver drift zero-norm cosine; solapa RES-06 (H-03) | `vantadb-ts/src/types.ts:71-77`, `vantadb-ts/src/native.ts:304`, `docs/api/`; origen FND-06-core-bindings-boundaries | 🟡 | 🟠 Media-Alta | ⬜ Pendiente |
| `TS-04` | **Cerrar huecos API vs core/Python**: exponer `remove_edge` (hay add sin remove), `count`, `versions/supersede`, pasar `sparse_vector` (hoy siempre None), `filter_ops` avanzados, batch search paridad con Python (H-09) | `vantadb-wasm/src/lib.rs:871-879`, `vantadb-ts/src/{vantadb,types}.ts` | 🟡 | 🟠 Media-Alta | ⬜ Pendiente |
| `TS-06` | **Gate CI para tests TS SDK**: 261 tests vitest sin gate ("no CI gate for TS SDK" TEST_MAP) — agregar job Fast Gate o Heavy según duración medida (H-07) | `.github/workflows/`, `vantadb-ts/package.json` scripts | 🟢 | 🟠 Media-Alta | ⬜ Pendiente |
| `TS-07` | **Smoke-test del tarball publicado**: paso automatizado `npm pack` + install en dir limpio + quickstart mínimo como parte del release npm (H-08) | pipeline npm release, script nuevo en `vantadb-ts/scripts/` | 🟢 | 🟡 Media | ⬜ Pendiente |
| `TS-08` | **Entrada CDN ESM documentada** para evaluación browser zero-install (patrón Orama jsdelivr `+esm`): verificar que funciona con wasm-bindgen glue o documentar por qué no (H-10) | `vantadb-ts/README.md:80-97`, demo WASM existente | 🟢 | 🟡 Media | ⬜ Pendiente |
| `TS-09` | **Bench reproducible JS/WASM** (insert+search p50/p95/p99, dataset determinístico) citable bajo Regla 11 — sin números no hay claims ni comparativas vs Orama/vectra posibles (H-11) | nuevo bench en `vantadb-ts/bench/`, resultados a `docs/operations/BENCHMARKS.md` | 🟡 | 🟡 Media | ⬜ Pendiente |
| `TS-10` | **Plan de distribución/adopción** (estratégica): playground browser interactivo + docs-site + comparativa honesta vs Orama/vectra/wa-sqlite/DuckDB-WASM usando la matriz del informe §3 — adopción actual 12 dl/semana vs 35K-465K competidores (H-06) | web/, docs/, estrategia SHOW_HN; research §3 | 🔴 | 🟠 Media-Alta | ⬜ Pendiente (requiere DISCOVERY) |
| `TS-11` | **Roadmap paridad sub-clientes**: planificar exposición vía WASM de wiki/conversation/skills cuando core lo permita — hoy `db.wiki` es `{}` documentado (H-12) | `vantadb-wasm/src/lib.rs`, `vantadb-ts/src/vantadb.ts:311-319` | 🔴 | 🟢 Baja | ⬜ Pendiente |
| `TS-12` | **Publicar `vantadb-node` en npm** (prebuilds multiplataforma napi-rs): desbloquea la ruta `NativeVantaDB` (persistencia real fjall/WAL en Node) rota para consumidores hoy (npm 404) (H-14); marcar experimental hasta prebuilds completos | `vantadb-node/` (CI prebuilds), README native path | 🟡 | 🟠 Media-Alta | ⬜ Pendiente |
| `TS-13` | **Posicionamiento vs Orama en web/** (decisión HITL: mover a módulo web): sección "Why VantaDB" honesta con matriz diferenciadores (durable WAL browser, híbrido RRF nativo, grafo+IQL, errores tipados) vs FTS-first de Orama (H-13) | `web/` (landing/docs); fuente: research-vantadb-ts §3 | 🟢 | 🟡 Media | ⬜ Pendiente |

## P42 - Investigación INV-vantadb-wasm (2026-08-25)

> **Origen:** `/research vantadb-wasm` → `docs/reviews/research-vantadb-wasm-20260825.md` (score 6.8/10, apéndice H-01..H-23). Decisiones HITL 2026-08-25: 14 filas nuevas WSM + plan quick wins (`docs/plans/2026-08-25-wasm-quickwins.md` con H-01/H-05/H-07/H-08/H-16) + 2 estrategias aprobadas (H-21 adopción npm, H-22 ADR browser-first) + 1 wontfix (H-23 static-hosting read-only). Duplicados ya trackeados NO recreados: H-19=`P2-8`. MOD-25..28 derivados del review 2026-08-22 nunca materializados (H-02): absorbidos por este batch (fallback→WSM-01, cuotas→WSM-02, nits→plan quick wins).

| ID | Descripción | Archivos clave | Esfuerzo | Prioridad | Estado |
|---|---|---|---|---|---|
| `WSM-02` | **Manejo de cuotas de storage browser**: chequear `navigator.storage.estimate()` pre-write grande, intentar `navigator.storage.persist()`, mapear `QuotaExceededError` a error accionable (no DOMException crudo). Patrón validado por DuckDB-WASM OPFS | `vantadb-wasm/src/opfs.rs`, `idb.rs` | 🟡 | 🟠 Alta | ⬜ Pendiente |
| `WSM-03` | **Auto-save**: hook `visibilitychange`/`pagehide` que dispare `save()`/`save_idb()` pendiente — hoy los writes desde el último save() explícito se pierden al cerrar la pestaña. Exponer opt-in/out en config WASM | `vantadb-wasm/src/lib.rs` (save/save_idb), glue JS | 🟡 | 🟠 Alta | ⬜ Pendiente |
| `WSM-04` | **Errores tipados `{code,message}`**: `to_js_err` aplana VantaError a string (`lib.rs:1518`); mapear discriminado a shape `{code,message}` consistente — habilita taxonomía compartida TS/node/Python (sinergia con MOD-20 Python) | `vantadb-wasm/src/lib.rs:1518`, validaciones `JsValue::from_str` | 🔴 | 🟡 Media-Alta | ⬜ Pendiente |
| `WSM-05` | **`.d.ts` hand-written para pkg standalone**: el generado es casi todo `any` (limitación wasm-bindgen); escribir tipos reales encima del glue (patrón DuckDB-WASM/sqlite-wasm-http). Paquete npm usable sin wrapper TS | `vantadb-wasm/pkg/vantadb_wasm.d.ts` (generado), nuevo `.d.ts` fuente | 🟡 | 🟡 Media-Alta | ⬜ Pendiente |
| `WSM-06` | **Batch paridad API vs core**: exponer parámetros perdidos (`filter_ops`, `exclude_superseded`, `sparse_vector`, `search_profile`) y métodos ausentes priorizados (`remove_edge`, `count`, `namespace_stats`, `similar_to_key`, `supersede`) según tabla faltantes de BINDINGS_NAMESPACES.md. **DISCOVERY primero:** recortar a lo que el nicho browser usa (H-22) | `vantadb-wasm/src/lib.rs`, `docs/api/BINDINGS_NAMESPACES.md` | 🔴 | 🟡 Media | ⬜ Pendiente (requiere DISCOVERY) |
| `WSM-07` | **DX connect_worker**: exponer helper `spawnOpfsWorker` desde el glue pkg (hoy exige inyección manual de `globalThis.spawnOpfsWorker`) | `vantadb-wasm/src/opfs_bridge.js`, `worker.rs`, `lib.rs:334` | 🟢 | 🟢 Baja | ⬜ Pendiente |
| `WSM-08` | **Corregir docs TS contradictorias**: `vantadb.ts:105` afirma "WASM backend always uses in-memory engine" — falso desde connect_persistent/connect_idb/graph_state (CORE-02 resuelto). Actualizar comentario + `docs/api/TS_SDK.md` §WASM note | `vantadb-ts/src/vantadb.ts:105`, `docs/api/TS_SDK.md` | 🟢 | 🟡 Media | ⬜ Pendiente |
| `WSM-09` | **Unificar límites FFI entre transports en core**: MAX_F32_VEC_LEN=10M (wasm) vs MAX_VEC_DIM=10k (node); MAX_K=1k vs top_k≤10k — misma operación acepta límites distintos según transporte. Mover constantes al core y derivar | `vantadb-wasm/src/lib.rs:38-43`, `vantadb-node/src/lib.rs`, core | 🟡 | 🟢 Baja-Media | ⬜ Pendiente |
| `WSM-10` | **Semántica score/distance consistente**: wasm emite campo `score` que TS documenta como distance y node describe como similitud — unificar criterio y documentar en los 3 transports | bindings wasm/ts/node + docs api | 🟡 | 🟡 Media | ⬜ Pendiente |
| `WSM-11` | **Señalizar metadata descartada**: `memory_record_to_js` ignora error de serialización de metadata (record devuelto sin metadata sin señal); propagar error o contador | `vantadb-wasm/src/lib.rs:1582 aprox` | 🟢 | 🟢 Baja | ⬜ Pendiente |
| `WSM-12` | **Contador de sanitizaciones NaN/Inf→0.0** en operational_metrics — la coerción silenciosa altera datos; hacerla observable | `vantadb-wasm/src/lib.rs` (coerción vectors), metrics | 🟢 | 🟢 Baja | ⬜ Pendiente |
| `WSM-13` | **Estrategia de bundle documentada**: 1.3 MB wasm sin doc de lazy-load/code-split ni comparativa honesta vs motores JS puros (~50 KB). Doc en README/demo + evaluar feature flags de build si aplica (FIND-11 parcial) | `vantadb-wasm/pkg/`, README npm, `demo/README.md` | 🟡 | 🟢 Baja-Media | ⬜ Pendiente |
| `WSM-14` | **Plan adopción npm** (estrategia H-21 aprobada): README npm con posicionamiento del nicho "browser AI agent memory", demo Transformers.js enlazada, keywords/comparativa honesta vs Orama (5.44M desc/mes vs 187). Sin claims de performance sin benchmark (Regla 11) | `vantadb-wasm/pkg/README.md` template, landing docs | 🟡 | 🟠 Alta | ⬜ Pendiente |

## P43 — Research web 2026-08-25 (INV-web-01, docs/reviews/research-web-prod-20260825.md)

| `WEB-01` | **Explorar pricing mid-tier**: hoy $0/Custom únicamente; los 5 competidores publican free tier + tier intermedio. Decisión HITL: explorar opciones (no implementar sin decisión de negocio). Origen: INV-web-01 H-08 | `web/src/app/pricing/page.tsx` | 🟢 | 🟡 Media | ⬜ Pendiente |
| `WEB-02` | **Publicar benchmarks propios en /benchmarks**: convertir datos de `docs/operations/BENCHMARKS.md` en tablas p50/p99 públicas citando fuente reproducible (estilo Chroma home) — social proof honesto sin claims de adopción inexistentes (Regla 11). Verificar que la ruta existente muestre datos actuales. Origen: INV-web-01 H-10 | `web/src/app/benchmarks/page.tsx`, `docs/operations/BENCHMARKS.md` | 🟡 | 🟡 Media | ⬜ Pendiente |
| `WEB-03` | **Restaurar/eliminar assets gato faltantes**: 4 refs a `mascota_gato.png`/`avatar_gato.png` con fallbacks silenciosos — restaurar en `public/assets/` o quitar refs muertas. Origen: INV-web-01 H-03 | `web/src/components/vanta/easter-egg.tsx:78`, `web/src/app/opengraph-image.tsx:15`, `web/src/components/vanta/vanta-data.ts:1062,1069` | 🟢 | 🟢 Baja | ⬜ Pendiente |
| `WEB-05` | **Re-medir Lighthouse post-WDA-05** (−7.615 líneas, lazy command-palette): actualizar claim perf del registro y AGENTS.md web con medición fresca (Regla 11). Si EPERM ambiental persiste, documentar workaround. Origen: INV-web-01 H-06 | `web/` (Lighthouse), `.opencode/references/research-modules.md`, `web/AGENTS.md` | 🟢 | 🟢 Baja | ⬜ Pendiente |

| `WEB-08` | **E2E web como guard de regresión**: specs Playwright del flujo crítico landing→docs→playground (hoy 0 tests E2E en web/; desktop tiene 2). Origen: INV-web-01 H-05 | `web/e2e/` (nuevo), CI | 🟡 | 🟡 Media | ⬜ Pendiente |
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
| `PROV-01` | **Fix compile vantadb-openai**: `list()` construye `VantaMemoryListOptions` sin campo `exclude_superseded` (E0063 contra core actual); anadir `exclude_superseded: false` o `..Default::default()`. Verificar con `cargo check --manifest-path providers/openai/Cargo.toml`. Origen: INV-providers-01 H-01 | `providers/openai/src/python.rs:296-302`, `src/sdk/types.rs:214-232` | ?? | ?? Alta | ? Pendiente |
| `PROV-02` | **Reparar tests rotos de los 3 crates**: litellm+openai llaman `search(emb, top_k)` sin `namespace` obligatorio; fixture ollama usa `create_namespace()` inexistente en vantadb_py. Actualizar a firma actual. Origen: INV-providers-01 H-02 (vigente desde review 2026-08-23 P1/P2) | `providers/*/tests/test_*.py` | ?? | ?? Alta | ? Pendiente |
| `PROV-03` | **Sincronizar stubs .pyi**: firman API vieja sin namespace/text_query/filters/distance_metric/top_k default; omiten get/list/delete/list_namespaces y params model/timeout/base_url. Regenerar desde firmas reales. Origen: INV-providers-01 H-03 | `providers/*/vantdab_*.pyi` | ?? | ?? Media-Alta | ? Pendiente |
| `PROV-04` | **Unificar contrato de salida entre crates**: records devuelven `payload`(litellm) vs `text`(ollama/openai); cursor `next_cursor` vs `cursor`; limit usize vs i32; node_id solo en litellm. Decidir contrato canonico y aplicarlo (antes de publicar, es API publica potencial). Origen: INV-providers-01 H-05 | los 3 `providers/*/src/python.rs` | ?? | ?? Media-Alta | ? Pendiente |
| `PROV-05` | **Extraer helpers compartidos** (~500 lineas duplicadas): `record_to_pydict`, `err_to_py`, extraccion metadata, loop store. Elimina la causa raiz de drifts tipo H-01. Evaluar crate interno vs macro vs modulo compartido por build script. Origen: INV-providers-01 H-06 | nuevo `providers/common` o equivalente | ?? | ?? Media | ? Pendiente |
| `PROV-06` | **Pasar timeout en litellm**: parametro aceptado pero muerto (`#[allow(dead_code)]`, python.rs:73-74); LiteLLM soporta `timeout` por llamada (doc oficial docs.litellm.ai). Origen: INV-providers-01 H-07 | `providers/litellm/src/python.rs:124-134` | ?? | ?? Baja | ? Pendiente |
| `PROV-07` | **Validacion explicita de inputs**: distance_metric invalido cae silencioso a cosine (debe ValueError); metadata con tipos no soportados se descarta sin warning. Origen: INV-providers-01 H-09 | los 3 `python.rs` (search/store) | ?? | ?? Baja-Media | ? Pendiente |
| `PROV-08` | **READMEs completos**: hoy 5 lineas ("Methods: embed, search, store" cuando hay 7); agregar quickstart completo + requisito `pip install openai/ollama/litellm` + tabla de metodos. Origen: INV-providers-01 H-10 | los 3 `providers/*/README.md` | ?? | ?? Baja | ? Pendiente |
| `PROV-09` | **Tests robustos + CI Python de providers**: pytest.importorskip para SDKs, test embed() mockeado (hoy ninguno llama embed), job CI que corra los tests (hoy solo cargo check en ci-rust-10.yml:431-433). Origen: INV-providers-01 H-11 | tests, `.github/workflows/ci-rust-10.yml` o nuevo workflow | ?? | ?? Media | ? Pendiente |
| `PROV-10` | **store() con custom key/upsert determinista**: keys hoy autogeneradas por nanosegundo; estandar del ecosistema es ID del usuario (chroma add(ids=)). Parametro opcional key, sin romper firma actual. Origen: INV-providers-01 H-08 | los 3 `store()` | ?? | ?? Media | ? Pendiente |
| `PROV-11` | **Embed batching/async** (baja prioridad): aprovechar AsyncClient ollama / aembedding litellm cuando haya volumenes grandes. Origen: INV-providers-01 H-12 | los 3 `embed()` | ?? | ?? Baja | ? Pendiente |
| `PROV-12` | **Publicar wheels PyPI** (estrategia H-04 aprobada): pyproject.toml + maturin, CI release multiplataforma (macos/windows/linux x86_64+aarch64), secrets PyPI. Desbloquea el diferenciador real (storage embebido local acoplado al embed). Pre-requisitos: PROV-01/02/04. Origen: INV-providers-01 H-04 | providers/*/, nuevo workflow release, CI_POLICY | ?? | ?? Alta | ? Pendiente |

## P47 — Promoción a `default-members`: criterios 100% estables (server/mcp/memory/proxy + ts/node)

> **Origen:** discusión 2026-08-27 sobre `Cargo.toml:636-642` — `default-members = [".","vantadb-python"]` deja `server/mcp/wasm/memory/proxy` fuera del Fast Gate a propósito (`CATEGORY: EXPERIMENTAL`). `ts`/`node` son paquetes npm (no crates) y nunca pueden entrar en `default-members`, pero necesitan gate npm equivalente. Esta fase define **qué validar antes de decir “100% estable y puede pasar a default”** y deja la promoción como cambio reversible en 1 línea. **No promocionar sin que los 10 checks pasen.** Ver `docs/operations/CI_POLICY.md` y `.opencode/references/definition-of-done.md`.

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
| `STABLE-01` | **Validar `vanta-memory`** — `publish=false`, depende de `vantadb` sin `server`. Gates 1-6: `cargo check -p vanta-memory --all-targets` + `clippy -D warnings` + `nextest -p vanta-memory` + `validate-docs-coverage` + `deny check` + `cargo package -p vanta-memory --dry-run`. Medir tiempo y registrar en ADR. | `vanta-memory/Cargo.toml`, `vanta-memory/src/`, `src/sdk/` | 🟡 1d | 🟠 Media | ⬜ Pendiente |
| `STABLE-02` | **Validar `vanta-proxy`** — gateway `axum+tokio+reqwest`, heavy. Gates 1-6 + e2e `vanta-proxy` con upstream mock (LLM) + `rate-limit`/`session`/`memory_tools` sin dead-lock. Si `cargo check` >60s, documentar como Heavy. | `vanta-proxy/Cargo.toml`, `vanta-proxy/src/` | 🟡 1d | 🟠 Media | ⬜ Pendiente |
| `STABLE-03` | **Validar `vantadb-server`** — ya pulido (SRV-01/02/06, 42 tests). Gates 1-6 + `cargo test -p vantadb-server --all-targets` (42 tests) + audit rotación + `x-request-id` + `ServerState::new` helper. Medir `clippy` con `server` feature. | `vantadb-server/src/`, `src/cli_server.rs`, `src/audit.rs` | 🟡 1d | 🔴 Alta | ⬜ Pendiente |
| `STABLE-04` | **Validar `vantadb-mcp`** — 72 `mcp_tests`, protocolVersion 2024-11-05, `serve_lines` con `OpGate`. Gates 1-6 + `cargo test -p vantadb-mcp` + `test-mcp.py` 37 checks vs skill. | `vantadb-mcp/src/`, `vantadb-mcp/tests/` | 🟡 1d | 🔴 Alta | ⬜ Pendiente |
| `STABLE-05` | **Validar `vantadb-wasm`** — requiere `wasm32` + `wasm-pack`. Gates 1-8: `rustup target add wasm32-unknown-unknown` + `wasm-pack build --target bundler` + `wasm-pack test --node` si hay test wasm + `cargo check -p vantadb-wasm` + docs de `WASM_PERSISTENCE.md`. | `vantadb-wasm/Cargo.toml`, `vantadb-wasm/src/`, `vantadb-wasm/pkg/` | 🟡 1d | 🟠 Media | ⬜ Pendiente |
| `STABLE-06` | **Validar `vantadb-ts` (npm)** — no entra en `default-members`, pero gate npm debe ser Fast Gate. Gates 5+2: `release-npm-61.yml:tests` ya mide 26s (<5 min) — re-medir en CI limpio + `npm ci && npm run build && npx vitest run` 264 tests + `npx eslint .` + `npm pack` incluye `engines`. | `vantadb-ts/package.json`, `vantadb-ts/src/`, `.github/workflows/release-npm-61.yml` | 🟢 4h | 🟠 Media | ⬜ Pendiente |
| `STABLE-07` | **Validar `vantadb-node` (npm, napi-rs)** — matrix 7 targets. Gates 5+8: `release-npm-node.yml` builds `x86_64-pc-windows-msvc` + Linux gnu/musl + macOS x64/arm64 + `npm test` 25 tests + `npm pack` incluye `*.node`. Medir tiempo matrix y asegurar sin `continue-on-error`. | `vantadb-node/package.json`, `vantadb-node/src/lib.rs`, `.github/workflows/release-npm-node.yml` | 🟡 1d | 🟠 Media | ⬜ Pendiente |
| `STABLE-08` | **Medición Fast Gate con `default` ampliado** — Simular `default-members = [".","vantadb-python","vanta-memory","vanta-proxy","vantadb-server","vantadb-mcp","vantadb-wasm"]` en rama `test/default-all` + `just verify` / `dev-tools/verify.ps1` + `dev-tools/verify_changed.ps1` con cache fría. Registrar wall time por job y decidir si Fast Gate sigue `<5 min` o se re-etiqueta como Heavy (requiere justificación en CI_POLICY). | `Cargo.toml:636`, `dev-tools/verify.ps1`, `.github/workflows/ci-rust-10.yml` | 🟡 1d | 🔴 Alta | ⬜ Pendiente |
| `STABLE-09` | **Promoción atómica + rollback plan** — Si `STABLE-00..08` verde en 3 corridas, PR único: `Cargo.toml:636 default-members` ampliado + `docs/operations/CI_POLICY.md` §default-members + `.opencode/rules/release-ci.md` + `dev-tools/verify.ps1` comentando que `wasm` requiere `rustup target add`; `git tag` no, `cargo publish` no afectado (`publish=false` sigue). Incluir rollback de 1 línea (revert `default-members`) en descripción del PR. | `Cargo.toml`, `docs/operations/CI_POLICY.md`, `dev-tools/verify.ps1` | 🟢 4h | 🔴 Alta | ⬜ Pendiente |
