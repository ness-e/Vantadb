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
> **Total open items:** ~35 activas (DISC-01..03, LEG-01, BIZ-01b, OLD-01, DESKTOP-12..27, ADMIN-01..09, NV-02/03/05, REVIEW-04)
> **Origen docs-audit:** `docs/strategy/ROADMAP.md`, `docs/progreso/bitacora.md`, `docs/reviews/FULL_CODEBASE_AUDIT_2026-07-11.md`, `docs/reviews/analisis_proyecto.md`, `docs/operations/PERFORMANCE_TUNING.md`, `docs/operations/REPO_CHECKLIST.md`, `docs/architecture/STORAGE_VERSIONING.md`, `docs/plans/2026-07-13-workflow-repair-campaign.md`, `docs/Investigaciones/cargo-check-optimizacion.md`, `docs/discord/todo.md`

---

## Exec Summary

| Phase | Items | Est. Effort | Priority |
|-------|-------|-------------|----------|
| **P0** 🚀 Release Blockers | 0 | — | ✅ Cerrado |
| **P1** 🛡️ Security & Critical | 0 | — | ✅ Cerrado |
| **P2** ⚡ Quick Wins Técnicos | 0 | — | ✅ Cerrado |
| **P3** 🧪 Test Coverage (adapters) | 0 | — | ✅ Cerrado |
| **P4** 🔧 Engineering Health | 0 | — | ✅ Cerrado |
| **P5** 📖 Docs & Community | 3 (DISC-01..03) | ~1-2 semanas | 🟡 Media |
| **P6** 🚀 Launch Campaign | 1 (LEG-01) | ~1-2 semanas | 🟡 Media |
| **P7** 🌐 WASM & Performance | 0 | — | ✅ Cerrado |
| **P8** 🔮 Post-Launch & Enterprise | 1 (BIZ-01b) | ~3-5 semanas | 🔵 Futuro |
| **P9** 📚 Old Docs Rescue (reference) | 1 (OLD-01) | — | 📖 Referencia |
| **P10** 🏗️ Competitive Features (catalog) | 0 | — | 🗺️ Roadmap |
| **P11** 🐛 GitHub Issues | 0 | — | ✅ Cerrado |
| **P12** 🖥️ DESKTOP App (Tauri) + Consola Admin | 25 (DESKTOP-12..27 + ADMIN-01..09) | ~4-6 semanas | 🔵 Futuro |
| **P13** 🔎 AUDREP — Audit Report 2025 | 3 (NV-02, NV-03, NV-05) | ~2-3 semanas | 🔴 Mixta |
| **P14** 🔍 REVIEW items | 1 (REVIEW-04) | 📆 Backlog | 🟡 Media |
| **P15** 🔍 ERR items (revisión multi-agente 2026-08-08) | 50 (ERR-001..051, 1 descartado) | ~2-3 semanas | 🔴 Mixta (5 críticos) |

> **Historial de items removidos/completados:** ver `docs/progreso/BACKLOG_HISTORY.md`.
> **Nuevo 2026-08-04:** Fase 12 DESKTOP (26 tareas, app Tauri multi-connection sobre las 6 integraciones) + `DEBT-01` (gate docs-coverage roto, Fase 4) + `TECH-01..08` (hallazgos de investigación DESKTOP-01b: 2 bugs reales, 1 batch stale-docs, 1 ADR env-naming, 4 features/decisiones, todos en Phase 4).
> **Nuevo 2026-08-07:** Fase 7 ADMIN (ADMIN-01..09, consola administrativa centralizada: datos/métricas/KPIs/SOPs/telemetría/procesos/conexiones) sobre la infra DESKTOP-02..11 ya completada. Fuente de KPIs/SOPs: investigación de mercado (Grafana VectorDB Observability, Milvus, Qdrant, Weaviate, Zilliz/VectorDBBench) + snapshot de métricas ya existente en core (`src/metrics/core/snapshot.rs`, 112 líneas). Tareas DESKTOP-12..27 (drivers/MCP/empaquetado) se ejecutan después del core del dashboard.
> **Nuevo 2026-08-08:** P15 ERR — 50 hallazgos de la revisión multi-agente por capas (6 sub-agentes: vanta-audit/arch/engine/worker/tuner/docs + verificación manual). Origen completo en `docs/reviews/errors-found.md`. Top 3 a atacar primero: ERR-010 (raza checkpoint/persistencia), ERR-021 (OOM MCP), ERR-022 (top_k sin clamp → alloc gigante).

---

## ✅ Definition of Ready / Done

> **DoR + DoD del proyecto (VantaDB-specific) viven en:** `.opencode/references/definition-of-done.md` — secciones "VantaDB — Definition of Ready" y "VantaDB — Project-specific DoD commands". Referencia única; no duplicar aquí.

---

## Phase 0: 🚀 Release Blockers

> Items que bloquean un release público seguro. Resolver antes de cualquier publicación.

| ID | Descripción | Archivos | Esfuerzo | Prio |

> **Items removidos (7) + WONTFIX:** ver `docs/progreso/BACKLOG_HISTORY.md` (P0). `META-001` era el único P0 activo.

---

## Phase 1: 🛡️ Security & Critical

> Investigaciones de seguridad y dependencias críticas.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|


> **Items previos resueltos (9):** ver `docs/progreso/BACKLOG_HISTORY.md` (P1).

---

> **Phase 2: ⚡ Quick Wins Técnicos** — **31 items removidos:** ver `docs/progreso/BACKLOG_HISTORY.md` (P2). No quedan items activos en P2.
> ⚠️ **Nota DRV-014 (tradeoff WAL batch):** ver ADR `docs/architecture/adr/DRV-014-wal-batch-tradeoff.md` — el fix original fue revertido deliberadamente por `cae92db3` (batch-append por shard, 3-5× speedup). Tradeoff de performance, no deuda pendiente.

---

> **Phase 3: 🧪 Test Coverage (Adapters & Engine)** — **14 items removidos:** ver `docs/progreso/BACKLOG_HISTORY.md` (P3). No quedan items activos en P3.

---

## Phase 4: 🔧 Engineering Health & Architecture

> Investigaciones de salud de ingeniería — rendimiento, concurrencia, arquitectura.

| ID | Descripción | Archivos | Esfuerzo | Prio |
|----|-------------|----------|----------|------|

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

> **Origen:** `docs/audit-reports/vantadb-audit-report.md` (auditoría estática multi-agente sobre `develop@63b0101d`).
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

> **Total:** 63 tareas (8 🔴 CRÍTICOS, 14 🟠 ALTOS, 26 🟡 MEDIOS, 15 🟢 BAJOS). Origen: `docs/audit-reports/vantadb-audit-report.md` — 6 hallazgos ya corregidos excluidos (ver nota superior) + `DEPS-01` de la sección 7 del mismo reporte. Críticos restantes activos: AUDREP-07 (03/08/13/02/05/06 tachados).

---

## P14 — REVIEW items (hallazgos de `docs/reviews/review-full-2026-07-27-0309.md`, validado 2026-08-05)

> **Origen:** findings de la unified-review full 2026-07-27, re-validados contra el código el 2026-08-05 (sub-agentes vanta-worker/vanta-docs). Los items marcados como corregidos en el report y confirmados se excluyen. `DEPS-01` ya cubre la duplicación de crates/lru; `AUDREP-41` cubre next-auth dead dep — no duplicados aquí.

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `REVIEW-04` | **MEDIO / Refactor: 3 god modules** — `src/node.rs` (1554→1882L, creció), `src/config.rs` (1313L), `src/storage/vfile.rs` (1165L). **Recomendación:** partir en submódulos (ej. separar UnifiedNode de FieldValue; config per-feature). | `src/node.rs`, `src/config.rs`, `src/storage/vfile.rs` | 1-2 semanas | 📆 Backlog | 📝 Pendiente |

---

## P15 — Hallazgos de Revisión Multi-Agente 2026-08-08 (errors-found.md)

> **Origen:** `docs/reviews/errors-found.md` — revisión por capas con 6 sub-agentes en paralelo (vanta-audit, vanta-arch, vanta-engine, vanta-worker, vanta-tuner, vanta-docs) + verificación manual del lead sobre `develop@7a19a9f5`. 51 hallazgos documentados; ERR-017 descartado al verificar (métrica euclidiana uniforme en `distance.rs:495/516/536`). Índices de esfuerzo: 🟢 < 1 día | 🟠 1-3 días | 🔴 > 3 días.

### CRÍTICOS (5)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-010` | **🔴 Persistencia / Race checkpoint↔snapshot** — `fluir()` escribe `checkpoint_seq` en backend (línea ~74) ANTES de serializar el índice (`save_vector_index()` línea ~86), sin `insert_lock`. Insert concurrente → en reopen el record se reaplica (duplicación) o queda invisible para siempre. **Fix:** fijar `checkpoint_seq` después de `save_vector_index` con la seq del snapshot bajo el mismo lock; failpoint + test de interleave. | `src/storage/engine/maintenance.rs` | 🔴 | 🔴 | 📝 Pendiente |
| `ERR-016` | **🔴 Parser descarta WHERE/RANK silenciosamente** — sin alias explícito, `opt(ident)` consume `WHERE`/`RANK` como alias y el filtro se pierde (data loss silencioso en queries). Test existente línea 1044 lo asume. | `src/parser/mod.rs:1044, 174-175` | 🟢 | 🔴 | 📝 Pendiente |
| `ERR-021` | **🔴 MCP OOM — `collection_stats/list/delete` materializan namespace completo** vía `collect_all_records`; el streaming `collect_stats` (AUDREP-21) se eliminó. Namespace >100k vectores → OOM por llamada. **Fix: restaurar streaming con `take(n)` + límite.** | `vantadb-mcp/src/lib.rs:333-365, 1401, 1430, 1499` | 🟠 | 🔴 | 📝 Pendiente |
| `ERR-022` | **🔴 `top_k`/`k` sin tope → alloc gigante → abort** — bindings pasan `top_k` sin clamp; core aloca `HashSet::with_capacity(ef*3)` con `k=10⁹` → abort del proceso. **Fix: `k.min(MAX_K)` en bindings y MCP.** | `vantadb-mcp/src/lib.rs:1301`, `vantadb-python/src/lib.rs`, `vantadb-wasm/src/lib.rs`, `src/index/search.rs` | 🟢 | 🔴 | 📝 Pendiente |
| `ERR-035` | **🔴 Read-lock global bloquea inserts** — `vector_store[0].read()` retenido durante TODO el HNSW `search_nearest`; el `insert`/`batch_insert` necesita `write()` y queda congelado por cada query (viceversa). Contención global writer↔reader. | `src/physical_plan.rs:211`, `src/storage/engine/ops.rs` | 🔴 | 🔴 | 📝 Pendiente |

### ALTOS (16 verificados)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-001` | **🟠 UB 32-bit (wasm32)** — `view_start + vector_len*4` en `usize` con wrap; `slice::from_raw_parts` OOB. Fix: `checked_mul/checked_add`. | `src/storage/engine/ops.rs:518-521 (+1266,1451,1851)`, `src/index/search.rs:541` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-002` | **🟠 SIGBUS handler → infinite loop** — handler devuelve sin resolver el fault; flags nunca consumidas; fallo se re-ejecuta → hang. | `src/storage/vfile.rs:211-223` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-003` | **🟠 Panic por header corrupto** — `vector_store[seg_id as usize]` en 4 puntos; `apply_delete` ya usa `.get()`. | `src/storage/engine/ops.rs:507, 1311, 1397, 1820` | 🟢 | 🟠 | 📝 Pendiente |
| `ERR-004` | **🟠 `lru 0.12.5` RUSTSEC-2026-0002** vía `ratatui 0.28.1` — advisory de Stacked Borrows violation. | `deny.toml`, Cargo.lock | 🟢 | 🟠 | 📝 Pendiente |
| `ERR-011` | **🟠 WAL replay pierde silenciosamente** — `recover_state` confía en `local_pos` round-robin; shard truncado → record marcado ya-checkpointed sin serlo. | `src/wal_sharded.rs`, `src/storage/engine/init.rs:454-480` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-012` | **🟠 Contadores `inbound` stale en delete** — `apply_delete` no decrementa inbounds; `shrink_neighbors` decide evictions con conteo corrupt — fuga de memoria del índice. | `src/index/neighbor_index.rs`, `src/index/graph.rs` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-013` | **🟠 Stats antes de txn abort** — cardinalidad/edges se actualizan antes del `Abort`; txn abortada deja inventario inflado. | `src/storage/engine/ops.rs` (insert paths) | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-018` | **🟠 `random_layer` capado en level 2 con `ml` default** — grafos sin profundidad, recall degrada. | `src/index/graph.rs:441-444` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-019` | **🟠 Bench mide brute-force, no HNSW** — `flat_threshold: Some(10000)` con count=10000. Falso resultado de rendimiento. | `benches/hnsw_pure.rs:33,63` | 🟢 | 🟠 | 📝 Pendiente |
| `ERR-020` | **🟠 ACORN second-hop con neighbor lists stale** tras `repair_orphan_links` — arcos muertos/omitidos. | `src/index/search.rs` (ACORN), `src/index/graph.rs` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-023` | **🟠 Python node IDs u64 truncado** — core es u128; `search` devuelve `u64` cortado, `delete` recibe OverflowError en IDs ≥ 2⁶⁴. | `vantadb-python/src/lib.rs` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-024` | **🟠 WASM u64 vs core u128** — `insert/get/delete_node` toman u64 mientras search devuelve string u128; nodos >2⁶⁴ inalcanzables. | `vantadb-wasm/src/lib.rs:1011, 1039, 1047` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-025` | **🟠 MCP `get_node_neighbors` lee id `as_u64`** — JSON pierde precisión ≥2⁵³; nodos u128 grandes inaccesibles. | `vantadb-mcp/src/lib.rs:1330-1340` | 🟢 | 🟠 | 📝 Pendiente |
| `ERR-036` | **🟠 Write-lock en hot path de `get()`** — `volatile_cache.write()` en cada read solo para `hits+=1`; lectores calientes serializados. | `src/storage/engine/ops.rs:1204-1214` | 🟠 | 🟠 | 📝 Pendiente |
| `ERR-037` | **🟠 `batch_insert` chequea existencia por nodo** — 10k batch = 10k read-paths completos + write-lock cache + clone de vector descartado. | `src/storage/engine/ops.rs:853-925` | 🟠 | 🟠 | 📝 Pendiente |

### MEDIOS (12)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-005` | **🟡 Test AUDREP-45 eliminado** del diff; perdimos cobertura del guard length-prefix oversized. | `src/storage/ops.rs` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-014` | **🟡 Staleness insert→get** — insert escribe WAL antes del `pending_hnsw` drain; `get()` concurrente puede dar `None` para nodo ya committed. | `src/storage/engine/ops.rs` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-026` | **🟡 parse_metadata descarta filtros no-escalables** — arrays/objetos/null ignorados → filtro silenciosamente no aplicado → resultados súper-conjunto. | `vantadb-mcp/src/lib.rs` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-027` | **🟡 HTTP 200 con `success:false`** en `execute_query` — proxies y monitoreo no distinguen errores de query. | `src/cli_server.rs:607-627` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-028` | **🟡 Query con vector norma 0 → `[]` sin error** — bindings muestran "sin resultados" falso. | `src/index/search.rs:129-148` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-029` | **🟡 `edge_count = u16` overflow** — nodo >65.535 aristas corrompe silenciosamente al persistir. | `src/storage/ops.rs:85` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-030` | **🟡 `put_batch` cross-namespace** — path legacy permite namespace por entrada; path keyword mezcla datos. | `vantadb-python/src/lib.rs:311-350` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-042` | **🟡 `read_header` 2× por candidato** en hot loop (+ entry points) — trabajo duplicado constante. | `src/index/search.rs:275-280, 347-353` | 🟠 | 🟡 | 📝 Pendiente |
| `ERR-043` | **🟡 `shrink_neighbors` clona vector** del nodo solo para usarlo como query. | `src/index/graph.rs:920-926` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-044` | **🟡 `TextAnalyzer` reconstruido por llamada** — batch N paga N setups (stemmer/stopwords). | `src/tokenizer.rs:44-106` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-045` | **🟡 `get_neighbors` clona la lista por nodo** — O(N×M) allocs durante compactación BFS. | `src/index/neighbor_index.rs:66-68` | 🟢 | 🟡 | 📝 Pendiente |
| `ERR-050` | **🟡 CHANGELOG desactualizado** — [0.5.0] es la última; 25+ commits (serie ADMIN/AUDREP/DESKTOP) sin doc. Falta [Unreleased]. | `docs/CHANGELOG.md` | 🟢 | 🛎 | 📝 Pendiente |

### BAJOS (9)

| ID | Descripción | Archivos | Esfuerzo | Prio | Estado |
|----|-------------|----------|----------|------|--------|
| `ERR-015` | **🔵 kill() siempre en `request_shutdown`** — sin señal graciosa SIGINT; metadata loss en Windows. | `desktop/src-tauri/src/connections/child_process.rs:170-189` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-031` | **🔵 `VecIndex::add` traga rechazos** (solo warn, sin Result) — futuros Arc<dyn> perderían inserts. | `src/index/search.rs:664-698` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-032` | **🔵 Test de `deserialize_node_payload` removido** — pérdida de cobertura del guard MAX_PERSISTED_NODE_BYTES. | `src/storage/ops.rs` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-033` | **🔵 `memory_list(limit=0)` → devuelve 1** — `max(1)` en core vs 0 pedido. | `vantadb-mcp/src/lib.rs:1139-1142` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-034` | 🔵 Verificado sin hallazgo: `/metrics` protegido, `/health` público OK. | `src/cli_server.rs` | — | — | ⏫ Verificado |
| `ERR-047` | **🔵 Copy inline en cada pop del hot loop** (`take_l + extend`). | `src/index/search.rs:225-238` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-048` | **🔵 2 hash lookups en `visited`** — `contains + insert` en vez de `insert` devuelve bool. | `src/index/search.rs:268-269` | 🟢 | 🔵 | 📝 Pendiente |
| `ERR-051` | 🔵 Verificado / sync CLI OK — subcomandos clap con handlers. | `src/cli.rs`, `src/bin/vanta-cli.rs` | — | — | ⛠ Verificado |

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
| AUD-020 | Media | vantadb-server sin tests de integración HTTP (auth/RBAC/rate-limit) — superficie de ataque pública | vantadb-server/ | 🟡 pendiente |
| AUD-021 | Media | Rate limiter fall-open: si GovernorConfigBuilder::finish() falla, endpoint sirve sin límite | src/cli_server.rs:160-164 | 🟡 pendiente |
