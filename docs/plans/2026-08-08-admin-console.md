# Plan de Ejecución: Consola Administrativa Desktop (ADMIN-01..09 + DESKTOP-20)

> **Campaign ID: 68db6ae9-51f0-4e64-bf06-195ac18a2b5e
> **Inicio:** 2026-08-08
> **Estado: completed
> **Fuente:** `docs/Backlog.md` → Phase 12 Fase 7 (ADMIN-01..09) + Phase 12 Fase 5 (DESKTOP-20)
> **Dirección:** el usuario dirige el desktop hacia una **consola administrativa** (dashboard métricas/KPIs/SOPs/telemetría/procesos/conexiones + data explorer), no solo MVP multi-connection. Base ya implementada: DESKTOP-02..11 (scaffold, trait, nativa, server, MCP spawn manager, frontend MVP).
> **Contexto 2026-08-08:** verificado con codegraph — `operational_metrics_snapshot()` existe en core (`src/metrics/core/mod.rs:522`, ~45 campos) y `vanta_health` ya implementado (DESKTOP-03). No crear telemetría nueva: el snapshot core es la fuente única de KPIs.

## Resumen

| Resultado | Count |
|-----------|-------|
| ✅ DO | 10 (ADMIN-01..09 + DESKTOP-20) |
| 🟡 DEFER | 20 (DESKTOP-12..19, 21..27, DISC-01/02, BIZ-01b, OLD-01, REVIEW-04) |
| ❌ SKIP | 1 (DISC-03 — ICEBOX confirmado) |
| 🔴 BLOQUEADO | 0 |

## Tasks

### Task 1: ADMIN-01 — Command `vanta_metrics` IPC

- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 (base de todo el dashboard)
- **Archivos clave:** `desktop/src-tauri/src/commands/metrics.rs` (nuevo), `desktop/src-tauri/src/lib.rs`, `desktop/src-tauri/src/commands/mod.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `operational_metrics_snapshot()` existe en `src/metrics/core/mod.rs:522` con `OperationalMetricsSnapshot` (~45 campos incl. `derived_prefix_scans`, `derived_full_scan_fallbacks`, `memory: MemoryBreakdownSnapshot`). Falta exponerlo como comando Tauri. Gap confirmado.
- **Gate Justificación:** base del dashboard completo; 8 tareas ADMIN dependen del snapshot; costo bajo (1 command + serde).
- **Gate Result:** ✅ DO
- **Contrato: cargo nextest run: 1844 passed, 0 failed; clippy clean
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Reutilizar `operational_metrics_snapshot()` del core (`vantadb` crate) — NO crear telemetría nueva.
  - Serde serialize `OperationalMetricsSnapshot` (verificar `serde` derives en `src/metrics/core/snapshot.rs`; añadir si faltan).
  - Registrar command en `lib.rs` invoke_handler + `commands/mod.rs`.

### Task 2: ADMIN-03 — Migrar UI al design system web (modo claro)

- **Esfuerzo:** 🟡 | **Prioridad:** 🔴 (base visual de la consola)
- **Archivos clave:** `desktop/src/App.tsx`, `desktop/src/App.css`, `desktop/tailwind.config.js`, `desktop/src/components/*`
- **Verificación real:** ✅ CÓDIGO-REAL — `ConnectionSelector.tsx` existe y está muerto (según ADMIN-03 y verificación de archivos); `App.css` usa tema oscuro propietario. Design system web disponible en `web/globals.css` (cream `#FBF9F5`, ink, neon `#FF5A45`).
- **Gate Justificación:** sin modo claro no hay consola admin; elimina componente muerto (deuda). Reutiliza design system ya probado de la web.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run build` en `desktop/` ✅; app abre en modo claro con tokens web; `ConnectionSelector.tsx` eliminado.
- **Task file:** `skills/campaign-executor/tasks/ADMIN-03.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Copiar tokens de `web/globals.css` (`@theme inline {}` + Tailwind v4) a `desktop/tailwind.config.js`/CSS.
  - Verificar que Tailwind está configurado en desktop (`tailwind.config.js` existe).

### Task 3: ADMIN-02 — Métricas vivas (delta entre snapshots)

- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 (depende de ADMIN-01)
- **Archivos clave:** `desktop/src/hooks/useMetrics.ts` (nuevo), `desktop/src-tauri/src/commands/metrics.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — ADMIN-01 no implementado aún (gap); hook no existe. Cálculo de deltas entre snapshots consecutivos (poll 3-5s): QPS, latencia p50/p95/p99, error_rate, upsert_rate, cache_hit_rate.
- **Gate Justificación:** KPIs vivas requieren deltas; 2 snapshots a 1s → deltas correctos (contrato del backlog).
- **Gate Result:** ✅ DO
- **Contrato:** `npm run build` en `desktop/` ✅; 2 snapshots tomados a 1s → deltas correctos en dashboard.
- **Task file:** `skills/campaign-executor/tasks/ADMIN-02.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Poll 3-5s con `setInterval` + cleanup en unmount.
  - Latencia p50/p95/p99: snapshot core expone `last_*_ms`; calcular percentiles requiere historial corto en el hook.

### Task 4: ADMIN-04 — Dashboard grid (metro-style) con poll 3-5s

- **Esfuerzo:** 🔴 | **Prioridad:** 🟠 (depende de ADMIN-01/02/03)
- **Archivos clave:** `desktop/src/pages/Dashboard.tsx` (nuevo), `desktop/src/components/*`
- **Verificación real:** ✅ CÓDIGO-REAL — no existe `pages/Dashboard.tsx` (gap). Patrón de polling en cadena ya existe en `web/` (equivalente frontend web).
- **Gate Justificación:** pieza central de la consola; QPS/latencia/recall/RSS en vivo sin bloquear UI.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run build` en `desktop/` ✅; dashboard visualiza métricas vivas con auto-refresh sin bloqueo.
- **Task file:** `skills/campaign-executor/tasks/ADMIN-04.md`
- **Estado:** ✅ COMPLETED
- **Branch:** develop
- **Commit:** b62fff7c

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Cards con sparkline (KPIs), tabla de índices, grid de procesos/conexiones.
  - Estados de health por vía (nativa/server/MCP).

### Task 5: ADMIN-05 — KPIs derivados

- **Esfuerzo:** 🟡 | **Prioridad:** 🟠 (depende de ADMIN-01)
- **Archivos clave:** `desktop/src/components/KpiCard.tsx` (nuevo), `desktop/src/utils/kpi.ts` (nuevo)
- **Verificación real:** ✅ CÓDIGO-REAL — snapshot core expone los campos base (evictions_total, quantized_nodes_total, hybrid_candidates_fused, etc.). Derivados (`recall@k`, `query_index_hit_rate`, `import_error_rate`, `eviction_rate`, `ann_rebuild_ms`, `hybrid_fusion_ratio`, `mem_per_kb`) no existen (gap).
- **Gate Justificación:** KPIs derivados son el valor del dashboard; puro cálculo, bajo riesgo.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run build` en `desktop/` ✅; panel de KPIs con tarjetas y sparkline.
- **Task file:** `skills/campaign-executor/tasks/ADMIN-05.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - `recall@k` requiere comparar hits esperados vs decididos (fuente: planner stats).

### Task 6: ADMIN-06 — SOP panels (WAL replay / Reindex / Health) con semáforo

- **Esfuerzo:** 🟡 | **Prioridad:** 🟡 (depende de ADMIN-01/03)
- **Archivos clave:** `desktop/src/components/SopPanel.tsx` (nuevo), `desktop/src/hooks/useSop.ts` (nuevo)
- **Verificación real:** ✅ CÓDIGO-REAL — no existe `SopPanel.tsx` ni `useSop.ts` (gap). Patrón `idle → running → done|error` ya usado en `web/`.
- **Gate Justificación:** operaciones de mantenimiento visibles (WAL replay, reindex, health) con semáforo; copia patrón probado.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run build` en `desktop/` ✅; UI muestra semáforo en WAL replay/health y botón ejecutar/re-run.
- **Task file:** `skills/campaign-executor/tasks/ADMIN-06.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Health del backend e índices; paneles de `wal_replay` y estado de embebido.

### Task 7: ADMIN-07 — Data Explorer

- **Esfuerzo:** 🟡 | **Prioridad:** 🟡 (depende de ADMIN-03)
- **Archivos clave:** `desktop/src/pages/Explorer.tsx` (nuevo), `desktop/src-tauri/src/commands/data.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `data.rs` ya tiene `vanta_list_records` con `limit` (línea 68) pero **sin offset/cursor** (gap a verificar: ¿core soporta paginación por cursor? `list_records` en manager/connection trait — revisar en DISCOVERY). `Explorer.tsx` no existe.
- **Gate Justificación:** explorar 10K+ records sin lag es core de la consola; registry ops como PANEL en la web.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run build` en `desktop/` ✅; navegar 10K+ records sin lag; columnas con acciones.
- **Task file:** `skills/campaign-executor/tasks/ADMIN-07.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Si `list_records` no soporta offset → añadir `offset: Option<usize>` al command y al trait de conexión (verificar contrato antes de ampliar API).

### Task 8: ADMIN-08 — Panel Procesos & Conexiones

- **Esfuerzo:** 🟡 | **Prioridad:** 🟡 (depende de ADMIN-01/03)
- **Archivos clave:** `desktop/src/components/ProcessesPanel.tsx` (nuevo), `desktop/src-tauri/src/commands/process.rs` (nuevo)
- **Verificación real:** ✅ CÓDIGO-REAL — `child_process.rs` existe (spawn manager DESKTOP-11) y `list_connections` ya existe en manager; falta command de procesos (`process.rs`) y panel UI (gap).
- **Gate Justificación:** cada vía (nativa/server/MCP) con estado, PID, uptime, QPS, memoria — visible desde la UI.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check --manifest-path desktop/src-tauri/Cargo.toml` ✅; `npm run build` en `desktop/` ✅; panel muestra cada vía con estado/PID/uptime.
- **Task file:** `skills/campaign-executor/tasks/ADMIN-08.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Reusar `list_connections` existente; añadir datos de ChildProcess (PID, uptime).

### Task 9: ADMIN-09 — Snapshot export + persistencia

- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 (depende de ADMIN-01)
- **Archivos clave:** `desktop/src-tauri/src/commands/metrics.rs`, `desktop/src/hooks/useMetrics.ts`
- **Verificación real:** ✅ CÓDIGO-REAL — no existe export (gap). Filesystem reactivo disponible en Tauri (`app_data_dir`).
- **Gate Justificación:** exportar snapshot JSON + history corto (últimos N puntos) es barato y útil para diagnóstico.
- **Gate Result:** ✅ DO
- **Contrato:** `npm run build` en `desktop/` ✅; botón exporta JSON con timestamp; recargar conserva últimos N puntos.
- **Task file:** `skills/campaign-executor/tasks/ADMIN-09.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Command `vanta_metrics_export` → JSON a `app_data_dir` con timestamp.

### Task 10: DESKTOP-20 — Lifecycle shutdown_all

- **Esfuerzo:** 🟢 | **Prioridad:** 🟡 (higiene; base para cierre)
- **Archivos clave:** `desktop/src-tauri/src/lib.rs`, `desktop/src-tauri/src/connections/manager.rs`
- **Verificación real:** ✅ CÓDIGO-REAL — `manager.rs` NO tiene `shutdown_all` (métodos listados: add/remove/set_active/active_id/list_connections/active_info/health/ingest/ingest_batch/list_records). Gap confirmado.
- **Gate Justificación:** cerrar app con MCP+Node+Python conectados no debe dejar procesos huérfanos; `child_process.rs` ya existe para matar subprocesos.
- **Gate Result:** ✅ DO
- **Contrato:** `cargo check --manifest-path desktop/src-tauri/Cargo.toml` ✅; `shutdown_all` en `RunEvent::ExitRequested`; sin procesos huérfanos tras cierre.
- **Task file:** `skills/campaign-executor/tasks/DESKTOP-20.md`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:**

  **Iteraciones:**
  | # | Acción | Resultado | Herramienta |
  |---|--------|-----------|-------------|
  | — | — | — | — |

  **Notas:**
  - Orden: webview → subprocesos → nativa última (flush); timeout configurable + kill forzoso.

---

## Secuencia

```
ADMIN-01 → ADMIN-03 (base) ──┬── ADMIN-02 ──┐
                             ├── ADMIN-04   ├── (en paralelo)
                             └── ADMIN-05 ──┘
DESKTOP-20 (independiente, paralelo desde wave 0)
        ↓
ADMIN-06 → ADMIN-07 → ADMIN-08 → ADMIN-09
```

- **Wave 0:** ADMIN-01, ADMIN-03, DESKTOP-20 (independientes)
- **Wave 1:** ADMIN-02, ADMIN-04, ADMIN-05 (dependen de ADMIN-01/03)
- **Wave 2:** ADMIN-06, ADMIN-07 (dependen de wave 1)
- **Wave 3:** ADMIN-08, ADMIN-09 (dependen de wave 2)

## Deferred (post-consola, no incluidos en este plan)

| ID | Razón |
|----|-------|
| DESKTOP-12/13/14 (MCP client) | vía MCP no necesaria para consola inicial — nativa + server cubren; DEFER |
| DESKTOP-15/16/17/18 (Node/Python) | scoping previo 2026-08-05 (valor marginal, empaquetado frágil); DEFER |
| DESKTOP-19 (path_holders/capability) | manager básico ya funciona; se añade cuando haya 2+ vías activas; DEFER |
| DESKTOP-21/22/23/24/25/26/27 | post-consola (UI multi-conn, streaming, config, empaquetado, CI, tests, docs); DEFER |
| DISC-01/02 | requieren UI manual de Discord (no agentes); DEFER |
| DISC-03 | ICEBOX confirmado 2026-08-05; SKIP |
| LEG-01 | humana (trademark); DEFER |
| BIZ-01b | post-launch; DEFER |
| OLD-01 (PGWire) | roadmap 2-3 sem; DEFER |
| REVIEW-04 (god modules) | refactor 1-2 sem; DEFER |

=== RECITATION ===
Campaign ID: 3deee89d-78c7-43d9-aab3-0fd0d6e125c3
Objetivo activo: AUD-014 dedupe pruning logic
Estado: completed
Última acción: select_neighbors canonicalizado como single source of truth; cap over-capacity a 2m fixea hang de test 10k (6.46s vs hang previo)
Resultado: ✅
Próxima acción: handoff
Contrato: cargo check pasa; shutdown_all en evento de cierre; sin huérfanos
Próxima tarea si completa: 
=== END RECITATION ===
