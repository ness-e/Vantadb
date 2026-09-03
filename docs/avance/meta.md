---
title: "Avance — Meta / Proceso"
type: meta
status: active
tags: [vantadb, avance, meta, proceso, housekeeping]
last_reviewed: 2026-08-22
aliases: []
---

# Avance — Meta / Proceso

> Cambios de proceso, housekeeping del backlog, decisiones de documentación y mejoras de pipeline. IDs originales conservados.

## Contrato del mirror `activo/`

Los archivos de `docs/avance/activo/` **se actualizan al cierre de cada campaña** (no daily); los dominios del mirror = **crates activos** del workspace. Un crate nuevo ⇒ archivo de dominio nuevo en el mismo cierre. Verificación: muestreo cruzado `git log --grep MEM-` ↔ archivos de dominio (GOV-D1, 2026-08-22).

## Backlog housekeeping

### 2026-07-26 — Backlog Cleanup P0–P4, P7, P9–P10 (53 items → progreso)
- **Objetivo:** Limpiar backlog verificando cada item ✅ contra código real.
- **Resumen:** P0 6 stale removidos + 1 WONTFIX (DEVOPS-15); P1 fase completa cerrada (9); P2 7 ✅ + 24 stale; P3 7 ✅ + 7 stale; P4 10 ✅; P7 2 ✅; P9 7 ✅; P10 12 ✅.
- **Impacto:** Backlog ~120 → ~65 items activos. 5 fases cerradas (P1–P4, P7).
- **Verificación:** cada item verificado contra código real antes de mover.

### 2026-07-07 — Reorganización Masiva del Backlog (24 eliminaciones, 21 adiciones, 11 prioridades)
- Fuente: `docs/research/VantaDB_ANALISIS_COMPLETO.md`.
- 24 items eliminados (Cloud entero, optimizaciones prematuras, SOC2/HIPAA, WAL shipping, PITR, Semantic Kernel, visual regression, duplicados).
- 11 re-priorizados; 21 nuevos agregados. Backlog 79 → **65 items activos**.
- Documentación: `docs/progreso/backlog-2026-07-07.md`.

### Housekeeping sin ID
- **Backlog audit:** 4 discrepancias corregidas (TSK-94/67/80/82) ✅
- **Clippy/fmt fixes:** 3 unused vars, 18 archivos formateados, conditional imports ✅
- **Fix `with_writer`:** MakeWriter closure en vez de `Box<dyn Write>` ✅
- **`vantadb-mcp` ttl_ms:** `planner.rs:369` `expires_at_ms: Some(0)` ✅

### P2 Backlog Housekeeping: DRV-041, VFY-006, VFY-007 (2026-07-26)
- Document-only: 3 tareas triageadas como ya corregidas (ver `historial/no-ops.md`). Backlog P2 counter 15→12.

### ECO-001: Eliminar hooks muertos de Claude Code
- **Fecha:** 2026-07-28
- **Resultado:** ✅ Hooks muertos de Claude Code eliminados. Detalle en snapshot-2026-08-07.

## Proceso / Pipeline

### 2026-08-23 — Task-System Hardening I + II (campañas cerradas y archivadas)
- **Hardening I** (H1-H9): hardening del campaign server (locks, verify) + question gates HITL con umbral único — 38/38 tests (`fcd7b243`, `26f68ff7`).
- **Hardening II** (R1-R7, S1, D1-D4): paralelismo real multi-instancia (recitation por-tarea, claim temprano anti doble-Discovery, locks session/snapshot, lock wait cap, guard >1 plan activo, rotación verify-log + eval_summary/lock_info, classify_workflow robusto), spec-first obligatorio (`prompts/spec-template.md`), routing de questions documentado, trim SKILL/SARL, manual→índice, statewright fuera + memoria versionada — 42/42 tests (`1a86bd2a`, `ed6c6ae7`, `b8a23939`, `3cc0aa50`).
- Fix adicional: persistencia de Campaign ID en `updateTaskStateCore`. **Nota operativa:** reiniciar OpenCode para que el server cargue el código R1-R7.
- Planes archivados: `docs/plans/archive/2026-08-23-task-system-hardening{,-2}.md`; retrospectivas en `docs/progreso/campanas/planes-archivados-punteros.md`.

### 2026-08-22 — GOV-D1: catch-up del mirror + dominios faltantes
- El mirror `activo/` estaba congelado al 20/08 y sin los crates creados después: `vanta-proxy`, context engine, y las campañas P29/P30/P31 sin registrar (MEM-43 `a0bcb112` / MEM-44 `785db22c` ausentes).
- **Fix:** 3 archivos de dominio nuevos por campaña (no commit-por-commit): `activo/vanta-memory.md` (P27 F1-F4 + P29 + P31), `activo/vanta-proxy.md` (P30 F6-F7: MEM-25..33), `activo/context-engine.md` (MEM-22/23/24/37 + wiring `a0bcb112`). Contrato del mirror actualizado (sección arriba).

### 2026-07-24 — auto-progreso + auto-commit en /pipeline task
- **Proceso:** `skill progreso` (Trigger 1) y el commit automático no se ejecutaban al final del pipeline MODO TAREA.
- **Fix:** `pipeline.md` pasos 6-7 después del Review: `skill progreso` + auto-commit. Aplica a MODO TAREA y MODO RUN. Decisión en campaign_memory como policy.

### 2026-08-07 — Migración de `docs/progreso/` → `docs/avance/`
- Reorganización del README único en árbol por dominio (este índice).
- 0 info perdida: snapshot completo 2026-08-03 en `historial/snapshot-2026-08-03.md`; fuentes originales conservadas hasta validar equivalencia de IDs.
- **Re-sync post-validación:** detectadas entradas del 04-08..07-08 ausentes en los archivos de dominio → `snapshot-2026-08-07.md` (copia íntegra del README actual) + `activo/desktop.md` (DESKTOP-01..11) + entradas añadidas: NUEVO-17, COMP-021, COMP-029, ENT-04 (core-engine/bindings), CI-01, REVIEW-02/03/05 (ci-cd), AUDIT-01/02, P13 (seguridad), ECO-001 (meta).

## Documentación

### Week 2026-07-01 — Documentation overhaul & Code Hardening
- Re-creado Obsidian graph color groups; plugins (Dataview, Linter, Calendar).
- 58 wikilinks rotos reparados (10 archivos).
- Fix syntax error `cli_server.rs` (//! + duplicate use).
- Clippy `if_same_then_else` en `src/sdk/search.rs:307`.
- `cargo fmt` en 22 archivos (1349 líneas).
- Windows pagefile os error 1455 → compilación lib tests individual. 440/440 tests pasan.

### Week 2026-06-19 — Comprehensive Audit (AUD-01→44)
- 44 hallazgos resueltos en un día con agentes paralelos (3 por batch, 15 batches).
- 7 críticos, 14 medios, 23 bajos. ~45 archivos modificados.
- CVEs resueltos: RUSTSEC-2025-0141 (bincode), RUSTSEC-2026-0176/0177 (pyo3).
- PHASE 3 exit criteria actualizados: todos AUDs resueltos ✅.

### 2026-08-07 — Auditoría y reclasificación de docs (C-*)
- **ECO-002:** Contradicción `--no-verify` en AGENTS.md (Regla 1 vs Regla 7) → Regla B eliminada; queda solo prohibición en línea 791. `.antigravity/AGENTS.md` idéntico.
- Reclasificación de archivos: `vectara-competitive-research` y `meta-001-root-cause-analysis` → `docs/research/`; `backlog-validation`, `progreso-readme-part1/2/3`, `progreso-sistema` → `docs/reviews/archive/`.

## Skills ecosystem

### S1: Consolidar skills duplicadas (~40% waste, ~80 a remover de ~190)
- Duplicados identificados: `minimalist-skill`=`minimalist-ui`, `redesign-skill`=`redesign-existing-projects`, `stitch-skill`=`stitch-design-taste`, `soft-skill`=`high-end-visual-design`, `threejs` local=`threejs-*` global, `prisma` basic=`prisma-expert`, `browser-use`=`agent-browser`+Playwright MCP, `gpt-taste`=`impeccable`+`design-taste-frontend`.
- Eliminar: Venice.ai suite (5 stubs), Fal.ai stub suite (10 de 14), `imagen` (5th image gen), `design-taste-frontend-v1` (migrar a v2).
- Referencia: `docs/reviews/FINAL-REVIEW.md` (Core 50).

### S2: Empty skill directories
- 9 dirs en `.claude/skills/` sin `SKILL.md`: cargo-nextest, github-repo-management, m10-performance, markdown-documentation, python-packaging, rust-ffi, rust-write-tests, test-reporting, vector-database-engineer. Poblar o limpiar.

## Fuentes
- `docs/progreso/ARCHIVO_HISTORICO.md` §Meta/Proceso
- `docs/progreso/README.md` §Housekeeping y C-*
- `docs/progreso/bitacora.md` §SKILLS ECOSYSTEM
## Retrospectiva — Batch REVIEW/MOD/FIND (plan 2026-08-24-batch-review-mod-find, archivado 2026-08-25)
- **Cierre:** 10/10 tareas completadas (8 commits + 1 fix pre-existente verificado), 0 failed, 0 stalled. Waves: W0 {REVIEW-06, MOD-02, FIND-27} · W1 {FIND-28, MOD-19, MOD-08+09} · W2 {UX-01+05, FIND-04}.
- **Start (seguir haciendo):** waves paralelas con MAX_CONCURRENT=3; sub-agentes NO commitean y el lead verifica+commitea por tarea (aislamiento de commits, sin race del index); worktree durable: 2 tareas (MOD-02, UX-01+05) se retomaron del estado parcial del run pausado sin perder trabajo.
- **Stop (dejar de hacer):** correr waves sobre un árbol sucio (H1 del plan: ~35 archivos desktop sin commit al iniciar). Probar `git status` limpio antes de lanzar cualquier wave paralela.
- **Continue:** contrato verificable por tarea; SARL para resultados no-DONE (UX-01+05 fallo por error transitorio de provider → RETRY fresco resolvio sin rehacer); verify mecanico del lead antes de commitear.
- **Accion medible:** reducir reintentos SARL por tarea de 2 a 1 (metric: retries/tarea; baseline esta campaña = 2 tasks con 1 retry cada una). North Star: >90% first-try — esta campaña 8/10 first-try (80%), 2 requirieron retry por causa infra, no de codigo.

## Retrospectiva — Batch Core/Server/MCP/Python/TS (plan 2026-08-25-batch-core-server-mcp)
- **Cierre:** 14/15 tareas completadas (12 commits), 1 DEFER (MCP-34), 0 FAILED. Waves: W0 {REVIEW-13, FIND-29, MOD-14} · W1 {MOD-04, REVIEW-17, FIND-18} · W2 {MOD-10, MCP-24, MOD-13} · W3 {MOD-18, FIND-10} · W4 {MOD-20, MCP-33, FIND-06} · W5 {MCP-34 → DEFER}.
- **Start:** waves paralelas MAX_CONCURRENT=3; sub-agentes NO commitean, lead verifica+commitea por tarea (aislamiento); verify mecanico del lead antes de cada commit (reveló que FIND-29/MOD-14 dependian del clippy de REVIEW-13 → commitear en orden de dependencia).
- **Stop:** lanzar 2 sub-agentes que editan los MISMOS archivos (MOD-10 y MCP-24 compartieron tools.rs/mcp_tests.rs → diff combinado, commit conjunto; MOD-18/MOD-20 comparten vantadb-python → secuenciados). Regla: NO paralelizar tareas del mismo directorio/archivo.
- **Continue:** contrato verificable por tarea; STOP CONDITION de MCP-34 respetada (snapshot_restore no existe en core → DEFER, no scope-creep); hallazgos colaterales ruteados a Backlog (FIND-30/31/32); hash SAME de skills MCP verificado.
- **Accion medible:** reducir superposicion de archivos en waves a 0 (metrica: tasks por wave tocando el mismo archivo; baseline esta campana = 1 colision MOD-10/MCP-24). North Star: 14/15 first-try completado (93%), 0 falsos positivos.
- **Deuda:** MOD-34 DEFER (snapshot_restore = feature core nueva, candidato MCP-34a wrapper snapshot_create); FIND-30/31/32 abiertos (colaterales pre-existentes).

## Retrospectiva — Batch Colaterales + Deuda + Desktop (plan 2026-08-25-batch-colaterales-deuda-desktop)
- **Cierre:** 14/14 tareas (12 commits + 2 verificadas como ya resueltas: FIND-30 absorbido por MOD-13, MEM-51 por batch Última Milla). 0 failed. Waves: W0 {FIND-30, UX-16, FIND-32} · W1 {FIND-31, MCP-34a, MOD-06} · W2 {MOD-11, MOD-21, BND-05} · W3 {AGT-02, AGT-04} · W4 {AGT-03, AGT-06, MEM-51}.
- **Start:** verificar con git log -S + rg antes de editar un fix reportado (FIND-30/MEM-51 ya resueltos — patrón FIND-30); waves con archivos disjuntos; lead verifica+commitea por tarea.
- **Stop:** confiar en hipótesis del backlog sin diagnóstico empírico (FIND-31: la hipótesis "text index no se reconstruye" era incorrecta — la causa real era lazy TTL eviction en memory_record_from_node). Lanzar sub-agentes que editan el mismo archivo en paralelo (AGT-02/AGT-03/AGT-06 comparten AGENTS.md — se secuenciaron).
- **Continue:** regla de sesiones paralelas ya no aplica (eliminada); desktop incluido (UX-16 lucide-react); STOP CONDITIONS respetadas (MCP-34a sin snapshot_restore, MEM-51 sin refactor grande).
- **Accion medible:** tasa de "ya resuelto" detectado en DISCOVERY = 2/14 (14%) — el Paso 0 con verificación de código real ahorra reimplementación. North Star: 14/14 first-try, 0 falsos positivos.

## Retrospectiva — Batch Desktop UX/DAUD + Core menor (plan 2026-08-25-batch-desktop-ux-core)
- **Cierre:** 8/8 tareas agrupadas (cubren ~20 filas backlog: UX-02..17, DAUD-01..08, MOD-15, FIND-11/17, TIR-08), 6 commits + 1 verificado ya-resuelto (TIR-08 en 1c7660dc). 0 failed. Waves: W0 {UX-A11Y, MOD-15, FIND-17} · W1 {UX-POLISH, FIND-11, TIR-08} · W2a {DAUD-LIMPI} · W2b {E2E-VISUAL}.
- **Start:** agrupar tareas desktop por área en 1 sub-agente (lección previa: NO paralelizar el mismo dir); verificar con git log -S + rg antes de editar fixes reportados (TIR-08 ya resuelto); CodeGraph auto-sync deshabilitado → leer archivos directos.
- **Stop:** confiar en verificación stale de stash (DAUD-08: la verificación 2026-08-24 decía "0 difiere" pero el diff real = 242 archivos → NO dropeado, reportado al usuario).
- **Continue:** STOP CONDITIONS respetadas (DAUD-08 no dropear con contenido real; FIND-17 sin renames); hallazgos colaterales ruteados a Backlog (FIND-23 namespace vacío en vanta-http-map).
- **Accion medible:** 3/8 tareas del batch requirieron verificación "ya-resuelto" o stop-condition (TIR-08, DAUD-08, FIND-17 parcial) — la verificación de código real antes de editar ahorra trabajo. North Star: 8/8 first-try, 0 falsos positivos, 0 regresiones.
- **Deuda:** DAUD-08 stash@{0} (41 archivos, 1500+ líneas WIP P34) pendiente de decisión del usuario; FIND-23 (namespace vacío HTTP) abierto; window.confirm persiste en ImportPaste/ImportDrop.

## Retrospectiva — Batch Core Fixes + Research P38 (plan 2026-08-25-batch-core-fixes-research)
- **Cierre:** 9/9 tareas (5 commits de código + 3 docs research + 1 docs CI). Pausa intermedia por usuario tras Wave 0 (3/9), reanudada después. 0 failed. Waves: W0 {FIND-23, AUD-044, AUD-047} · W1 {AUD-045, AUD-046, FIND-22} · W2 {RES-01/02/03}.
- **Start:** bench Regla 9 rindió (AUD-045: -59% IVF reutilizando helper existente f32_slice_similarity — cero código nuevo); research con vanta-research leaf sin write → contenido inline persistido por el lead (funciona, pero añade un paso manual).
- **Stop:** confiar en verificación previa de hallazgos del backlog (AUD-043 ya resuelto por FIND-30 — el backlog acumula filas resueltas sin sync). Verificar "ya-resuelto" ANTES de ticketear.
- **Continue:** verify mecánico del lead antes de cada commit; hallazgos colaterales ruteados a Backlog en el momento (FIND-24, MCP-34b, FIND-25, FIND-26); STOP CONDITIONS respetadas.
- **Accion medible:** 1/6 hallazgos verificados estaba ya resuelto (AUD-043) — métrica: tasa de stale-detection al triagear. North Star: 9/9 first-try, 0 falsos positivos.
- **Outputs de research:** RES-01 GO condicional (WAL v2 Prepare tras flag+bench) · RES-02 restore físico S1-S5 recomendado (+MCP-34b/FIND-25/FIND-26) · RES-03 session layer defer-as-scoped (DEC-01 resuelta).

## Retrospectiva — Backup/Restore Chain (plan 2026-08-25-batch-backup-restore-chain)
- **Cierre:** 3/3 tareas secuenciales (FIND-25 → MCP-34b → FIND-26), 3 commits. 0 failed. La cadena completa backup/restore física quedó operativa: create_snapshot consistente (quiesce+mirror recursivo) → snapshot_restore (core+SDK+MCP con confirm destructiva) → PITR dead code removida (ADR-014 superseded).
- **Start:** research previa (RES-02) con diseño file:line verificado hizo la ejecución directa (0 incógnitas); plan secuencial por dependencias evitó colisiones; hallazgo colateral ruteado en el momento (FIND-33: snapshot tras compact_wal pierde datos — backend KV fuera de data_dir).
- **Stop:** cargo clean -p vantadb durante compilación de la otra sesión rompió el target dir compartido (48GB, STATUS_STACK_BUFFER_OVERRUN). NUNCA limpiar cache compartido con otra sesión compilando — esperar o verificar con --target aislado.
- **Continue:** verify mecánico del lead antes de cada commit; Regla 0 antes de eliminar (FIND-26: grep exhaustivo confirmó solo export+tests propios).
- **Accion medible:** cadena ejecutada 3/3 first-try con diseño previo de research vs batches sin diseño (~1 retry promedio). North Star cumplida: 0 falsos positivos, 0 regresiones.
- **Deuda:** FIND-33 abierto (snapshot tras compact_wal — rediseño >100 líneas); stash@{1..9} viejos sin revisar.

## Regla de proceso — Derivación atómica de hallazgos (INV-DECIDE, 2026-08-26)
- **Contexto:** 4 pérdidas de trazabilidad en el programa INV-* 2026-08-25 (MOD-22..24, MOD-25..28, MOD-41..45, MOD-46..50): hallazgos "derivados al Backlog" en la prosa de reports pero sin fila creada — violación repetida del invariant progreso (nada se elimina del Backlog sin completar o archivar).
- **Regla dura:** derivar un hallazgo = crear la fila en `docs/Backlog.md` EN EL MISMO commit donde se registra la derivación. Prohibido dejar la derivación solo en prosa de reports/informes.
- **Chequeo mecánico:** Trigger 4 de `progreso` incluye grep de IDs citados en `docs/reviews/*.md` (apéndices H-NN, MOD-\*, derivaciones) vs filas reales en Backlog/historial; IDs huérfanos = hallazgo REC-\* inmediato.
- **Origen:** decisión HITL `/research synthesis` Q8 (sesión 2026-08-26); ref: `docs/reviews/research-bindings-synthesis-20260825.md` §3.

## Cierre ejecución INV-DECIDE — waves quick-wins completas (2026-08-26)
- **Ejecución:** 9 planes de los aprobados, 2 waves paralelas por directorios disjuntos, 9 sub-agentes vanta-worker. Commits: `2754c783` providers · `4ffb833b` python (+PY-03) · `a7ed0d22` desktop (CSP+sparse_vector) · `c141c1ce` ts (gate CI+smoke) · `f72a0cc0` server (SRV-01/02) · `53f080e5` wasm (QW-1..5) · `96e143ec` integrations (QW-1..8) · `a86c7e4e` node (BND-11/12/13+bench) · `9d6758f6` web (WEB-03..09). Lead verificó: cargo check ×9 crates + workspace completo ✅, build web ✅, suites vitest/wasm-pack/pytest/npm test según agente.
- **Bloqueados (con destino):** PROV-09 CI job + QW-7 PyPI publish (tokens owner) · BND-08/BND-09 npm/musl (tokens owner) · PY-QW2/P2-5 (llamaindex tuplas legacy primero — ver lessons) · DESKTOP-45 creado (H-11/H-15/H-07 restos) · WEB-05 Lighthouse EPERM ambiental · SRV-04..08 en Backlog P40.
- **Incidentes:** 2 interferencias entre agentes paralelos (revert percibido providers, commit externo pisó ediciones ts) — auto-reparados; 1 stash ajeno aplicado y restaurado por el agente py (lección: nada de stash con agentes paralelos); web agent abortó tras implementar → lead verificó build y commiteó.
- **Pendiente bookkeeping:** migración filas Backlog→avance de las completadas por estos commits (pasada progreso Trigger 1 dedicada); verify.ps1 completo antes de push (pre-push hook lo enforced).

## Retrospectiva — Python SDK Quick Wins (plan 2026-08-25-py-quickwins)
- **Cierre:** 5/5 tareas completadas (1 commit código + 4 verificación/docs). 0 failed, 0 stalled. Wave 1: {PY-QW1, PY-QW2, PY-QW3} · Wave 2: {PY-QW4, PY-QW5}. Todas ✅ first-try.
- **Start (seguir haciendo):** verificación empírica ANTES de editar (PY-QW1/PY-QW3 ya resueltas, PY-QW4 solo .gitignore nuevo, PY-QW5 ya presente) — evita reimplementar; contracts verificables simples (rg, classifier check, git status).
- **Stop (dejar de hacer):** confiar en hipótesis del backlog sin diagnóstico (PY-QW2: la dual API real era 53 líneas, no 100+; PY-QW3 ya tenía 3.14).
- **Continue:** contracts mínimos por tarea; lead verifica+commitea; plan file archivado tras cierre.
- **Accion medible:** 3/5 tareas del plan ya estaban resueltas al triagear (PY-QW1, PY-QW3, PY-QW5) — métrica: tasa de stale-detection al inicio = 60%. North Star: 5/5 first-try (100%), 0 falsos positivos, 0 regresiones.

## Retrospectiva — Integrations Quick Wins (plan 2026-08-25-integrations-research-wins)
- **Cierre:** 9/9 tareas completadas (1 commit código + test fixes + workflow + docs). 0 failed, 0 stalled. Waves: W1 {QW-1, QW-2, QW-3} · W2 {QW-4, QW-5, QW-6} · W3 {QW-7, QW-8} · W4 {QW-9}. Todas ✅ first-try.
- **Start (seguir haciendo):** verificación empírica ANTES de editar (QW-1/2/3/4/5/6 ya resueltas en código); contracts verificables simples (rg, pytest, workflow lint); test fixes mínimos (backend 'flat'→'memory' en 2 tests).
- **Stop (dejar de hacer):** confiar en hipótesis del backlog sin diagnóstico (QW-7: publicación PyPI es manual/token-gated, no código; QW-9: workflow ya existía en borrador).
- **Continue:** contracts mínimos por tarea; lead verifica+commitea; plan file archivado tras cierre; test fixes documentados.
- **Accion medible:** 6/9 tareas del plan ya estaban resueltas al triagear (QW-1, QW-2, QW-3, QW-4, QW-5, QW-6) — métrica: tasa de stale-detection al inicio = 67%. North Star: 9/9 first-try (100%), 0 falsos positivos, 0 regresiones. Total tests pasando: 150 (9 adapters).

## Retrospectiva — Providers Quick Wins (plan 2026-08-25-research-providers-quickwins)
- **Cierre:** 7/7 tareas completadas (1 commit CI + verificación/docs). 0 failed, 0 stalled. Waves: W1 {PROV-01, PROV-06, PROV-03, PROV-07, PROV-08, PROV-02} · W2 {PROV-09}. Todas ✅ first-try.
- **Start (seguir haciendo):** verificación empírica ANTES de editar (PROV-01/06/03/07/08/02 ya resueltas en código); contracts verificables simples (cargo check, rg, pytest structure); CI workflow reutiliza patrón adapters-compat.yml.
- **Stop (dejar de hacer):** confiar en hipótesis del backlog sin diagnóstico (PROV-01/06/03/07/08 ya implementados; PROV-09 workflow reutiliza patrón existente).
- **Continue:** contracts mínimos por tarea; lead verifica+commitea; plan file archivado tras cierre.
- **Accion medible:** 6/7 tareas del plan ya estaban resueltas al triagear (PROV-01, PROV-06, PROV-03, PROV-07, PROV-08, PROV-02) — métrica: tasa de stale-detection al inicio = 86%. North Star: 7/7 first-try (100%), 0 falsos positivos, 0 regresiones. 3 crates compilan, .pyi verificado, CI workflow creado.

## Retrospectiva — Desktop Quick Wins (plan 2026-08-25-research-desktop-quickwins)
- **Cierre:** 10/10 tareas completadas (5 commits wave1-2 audit-only + 1 CSP edit + 1 versión exclude + 1 BENCHMARKS §Desktop + 1 E2E 12/12). 0 failed al cierre (1 retry CSP por rate limit + 2 retries QW10 por disco lleno). Waves: W1 {QW1 palette, QW2 F1/F2, QW3 report ES, QW4 FILTROS, QW5 DAUD} · W2 {QW6 CSP, QW7 sparse_vector} · W3 {QW8 release-plz, QW9 BENCHMARKS, QW10 E2E}.
- **Start (seguir haciendo):** verify-only primero con `codegraph_explore` + `rg` (QW1/QW3/QW4/QW7 audit-only ahorró ~200 líneas edición innecesaria); CSP mínima 2 líneas ponytail (localhost:* + https://*); BENCHMARKS con fuente reproducible (command+env+date).
- **Stop (dejar de hacer):** confiar en backlog sin verificar código (6/10 ya resueltos desde a7ed0d22); asumir disco infinito (StorageFull 112 bloqueó e2e hasta liberar 120GB).
- **Continue:** contracts `npm --prefix desktop run build` + `npm test 69/69` + `cargo check` por wave; E2E con mock `page.route` + far-future TTL evita drift.
- **Accion medible:** 6/10 audit-only al triagear (60% stale-detection) vs 14% en batches colaterales — verificación previa ahorra código. North Star: 10/10 first-try post-retry (100%), 0 falsos positivos. Coste: disk full lesson — limpiar temp antes de e2e webServer.

## Archivo 2026-09-02: campaña error-observability (20260902-error-observability)

Plan docs/plans/archive/2026-09-02-error-observability-excellence.md archivado con retrospectiva: 9/9 ✅ (4 waves, MAX_CONCURRENT=3, 89% first-try — único retry por fallo de infra uv_spawn, no de diseño). Contrato unificado VANTADB_* (10 códigos) core→WASM/Node/TS→Python/providers→MCP(-320xx)→Desktop→Web. Follow-ups: FIND-52/53/54 + sanitización 500.


## Archivo 2026-09-03: campana alta-prioridad-paralelo (20260902-alta-prioridad-paralelo)

Plan archivado con auditoria de cierre: 82/88 verificados con evidencia + 6 reaperturas (RES-07/08/09/12/15 + DEC-02, premisa-falsa sin ejecucion - stamps masivos T00:00). GOV-T01 corregido a completed (stale). Backlog: -5 filas (MCP-35, RES-04/06/13/14) +2 restauradas (RES-02/03, colision ID con docs de research en sync 09-01). Leccion: cotejar syncs por contenido/evidencia, nunca por ID.


## Veredicto alta-prioridad 2026-09-03

-2 obsoletas (RES-02/08, evidencia en backlog-history) +3 re-escaladas (RES-09/12/15) + DEC-02 a ICEBOX. Backlog: 126 activas.

