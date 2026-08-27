---
title: "Backlog History — Items Removed & Migrated"
type: tracking
status: active
tags: [vantadb, backlog, history]
last_reviewed: 2026-08-07
aliases: []
---

# Backlog History — Items Removed & Migrated

> Historial narrativo de los items que salieron del catálogo activo (`docs/Backlog.md`): completados, removidos por stale, resueltos o cerrados WONTFIX. Este archivo documenta el *por qué*; el catálogo solo lista lo que queda por hacer.

## Items removidos totales

**71+ items removidos:** ~25 originales + 6 P0 stale + 9 P1 resueltos + 24 P2 stale + 7 P3 stale + 10 P4 completados + 7 P9 completados + 11 P10 completados + 1 P7 completado + 24 crates de integración nunca implementados.

## Por fase

### P0 — Release Blockers (7 removidos + 1 WONTFIX)

- `DEVOPS-10` — deferido
- `DEVOPS-12` — PyPI signing
- `DEVOPS-14` — ✅
- `NUEVO-09` — ✅
- `NUEVO-10` — ✅
- `DEVOPS-15` — ❌ **WONTFIX** (remover `cli, memmap2, fs2, sysinfo` rompe UX "it just works"; las 7 features mantienen experiencia completa)
- `META-001` — queda como único P0 activo en su momento

### P1 — Security & Critical (9 resueltos)

Todos los items P1 originales resueltos/deferidos en campañas anteriores.

### P2 — Quick Wins Técnicos (31 removidos)

`DRV-014` ✅, `DRV-028` ✅, `DRV-041` ✅, `VFY-006` ✅, `VFY-007` ✅, `REV-012` ✅, `DRV-136` ✅ + 24 stale items de la auditoría original.

> ⚠️ **Nota DRV-014:** el fix fue revertido por `cae92db3` — ver `docs/architecture/adr/DRV-014-wal-batch-tradeoff.md`. Tradeoff de performance posterior, no deuda pendiente.

### P3 — Test Coverage (14 removidos)

`DRV-013` ✅, `DRV-017` ✅, `DRV-061` ✅, `DRV-067` ✅, `DRV-073` ✅, `TEST-11` ✅, `TEST-12` ✅ + 7 stale de auditoría original.

### P4 — Engineering Health (10 completados)

`WEB-03` ✅, `WEB-04` ✅, `VFY-004` ✅, `VFY-011` ✅, `DRV-121` ✅, `DRV-122` ✅, `DRV-123` ✅, `DRV-130` ✅, `DRV-131` ✅, `DOC-20` ✅ — movidos a `docs/progreso/README.md`.

### P7 — WASM & Performance (5 removidos)

`NUEVO-11`/`NUEVO-12` (WASM IndexedDB + multi-tab coordinación — ✅ implementados), `NUEVO-14` (bundle 394KB gzip < 500KB — ✅ en WASM-04), `NUEVO-19` (`SourceDesign/` no existe), `BENCH-01` (solo mención en backlog, sin script ni dataset).

### P8 — Post-Launch & Enterprise (1 removido)

`NUEVO-20` (Dockerfile ya existe en raíz del repo — multi-stage, Rust 1.94).

### P9 — Old Docs Rescue (8 removidos a progreso)

`OLD-04` (OpenTelemetry), `OLD-07` (AutoHot/Cold tiering), `OLD-13` (Explainable ranking), `OLD-15` (Euclidean SIMD), `OLD-16` (WAL rotation 256MB), `OLD-17` (Migration guides), `OLD-18` (TEMPERATURE param), `OLD-22` (Arrow columnar export).

### P10 — Competitive Features (12 removidos a progreso)

`COMP-001` (SQ8/PQ), `COMP-002` (HNSW persist), `COMP-003` (in-filter), `COMP-004` (bitset), `COMP-005` (params), `COMP-006` (Edge Label Interning), `COMP-007` (inline u128), `COMP-010` (auto-embedding), `COMP-011` (CRUD tombstones), `COMP-015` (hybrid pipeline), `COMP-018` (Double-linked chains), `COMP-020` (RRF fusion), `COMP-030` (survival mode).

### P16 — Security gaps (1 removido como STALE)

- `AUD-036` — **STALE / falso positivo** (2026-08-13). El finding apuntaba a `src/schema.rs:255,263` (`let _ = std::fs::create_dir_all/remove_dir_all`), pero ambas líneas están dentro de `#[cfg(test)] mod tests` (setup/cleanup de test — idiomático ignorar). El código de producción (`read_from`, `write_to`, `load_or_create_schema`, `check_schema_compatibility`) propaga TODOS los errores fs con `map_err` + `?`, y todos los callers (`src/storage/engine/init.rs:259,266`, `src/migration.rs:342`, `src/cli_handlers/migrate.rs:167,181,206`) propagan con `?`. Sin fix aplicable. Evidencia grep en task file `.opencode/skills/campaign-executor/tasks/AUD-036.md`.

## Historial de verificación del catálogo

- **2026-07-27:** vanta-lead ejecutó 8 tareas de P5/P6/P8.
- **2026-07-28:** 5 sub-agentes explore validaron 69 items contra código real — ver `docs/audit-reports/backlog-validation-2026-07-28.md`.
- **2026-07-29:** 19 items INVESTIGACION agregados (INV-001 a INV-017) tras verificación de consolidación de 4 sub-agentes vs código real.

## Completados de `docs/strategy/` (2026-08-05)

Verificación de los 5 documentos de `docs/strategy/` (ROADMAP, GO_TO_MARKET, SHOW_HN_PREP, BLOG_SERIES_PLAN, REDDIT_POSTS) contra backlog + `docs/progreso/README.md` + git history. Items que strategy lista como tarea y **ya están completados** — registrados aquí porque no tenían fila activa en el backlog:

### Release & packaging (Fase 0 roadmap)

- `REL-01` — bump a v0.2.0 → **superado: repo en v0.5.0** (`vantadb-ts/package.json`, SHOW_HN_PREP). Commit `0b3b8353` (fase 4 release engineering).
- `REL-02` — publicar `vantadb-ts` en npm → **✅ completado** — commit explícito `cb9589db release(REL-02): publicar vantadb-ts en npm`; `vantadb@0.5.0` en package.json.
- `DEVOPS-05` — pipeline CI a PyPI para adapters → **✅ completado** — task file borrado `DEVOPS-05.md` + commits `2ac6b033`, `1e986b68`.

### Integraciones (GTM Tier 1-2)

- `INT-01` — LangChain adapter → **✅ completado** — task file borrado `INT-01.md`; adapter en `integrations/langchain-vantadb`.
- `INT-02` — LlamaIndex adapter → **✅ completado** — task file borrado `INT-02.md`.
- `INT-03→09` — 7 adapters Python puros (Mem0, CrewAI, DSPy, Haystack, Letta, OpenAI, Ollama) → **✅ completados** — commit `60c7b3e7`.
- `TSK-90` (CrewAI), `TSK-91` (DSPy) → **✅ completados** — commit `23a40320` (7 framework integration adapter crates).

### Web / Marketing / Legal

- `WEB-02` — corregir claims falsos del landing (50x→40x, SQL, auto-embeddings, cloud) → **✅ completado** — commit `e84e3c40 fix(web): correct landing page claims`; `vanta-data.ts` hoy muestra 2.80x/2.18x/2.14x.
- `MKT-13` — WASM demo funcional → **✅ completado** — `/demo` existe (`web/src/app/demo`) + commit `ee310422 feat(WEB-001): run real WASM in playground`.
- `MKT-17` — página de comparación competitiva → **✅ completado** — commit `e898b47b` (Fase 1 cierres).
- `LEG-01` — trademark → **✅ cerrado** — commit `e898b47b` (cierre en backlog-validation).

### SDK / Platform

- `TSK-61` — feature gates + build profiles → **✅ completado** — `docs/progreso/README.md:123` (✅).
- `TSK-68` — Python SDK latency <20ms / zero-copy NumPy → **✅ completado** — commit `0c1962b2 feat(python): zero-copy NumPy FFI via buffer protocol (TSK-68)`.
- `TSK-100` — Homebrew formula macOS → **✅ completado** — `Formula/vantadb.rb` existe + task file `DEVOPS-HOMEBREW.md` + `docs/progreso/README.md:1409`.
- `TSK-101` — ARM64 Linux wheels → **✅ completado** — `docs/progreso/README.md:1407` + `release-binaries-63.yml` (aarch64-apple-darwin).

### Enterprise / Governance (Q1-Q2 2027 GTM)

- `TSK-72` — AES-256 at-rest encryption → **✅ completado** — commit `b78a9b5a feat: Phase 5 complete — governance, encryption, WAL shipping, PITR`.
- `TSK-107b` — audit logging → **✅ completado** — task file archivado `tasks/complete/TSK-107b.md` + commit `cc095774`.
- `BIZ-02` — async WAL shipping → **✅ completado** — commit `b78a9b5a` (Phase 5, WAL shipping + PITR).
- `BIZ-03` — pricing page → **✅ completado** — commit `c73e8a4a docs: move BIZ-03, DOC-11 and DOC-12 to progress log`.

### Seguridad

- `SEC-13` — CSP + HSTS + nonce → **✅ completado** — task file borrado `SEC-13.md`; ARCHIVO_HISTORICO P1 lo lista cerrado.
- `SEC-14` — cargo-deny / licencias → **✅ RESUELTO** — `docs/progreso/README.md:2763` (`cargo deny check` pasa en CI).

### Engine / Engineering Health (roadmap Sem 5)

- `DRV-001` — split `search.rs` god file → **✅ completado** — task file archivado `tasks/complete/DRV-001.md`.
- `DRV-002` — `put_batch` duplica `put()` DRY → **✅ completado** — task file archivado `tasks/complete/DRV-002.md`.
- `DRV-003` — `purge_expired` O(n) index rebuilds → **✅ completado** — commit `d9e1caf9 perf(DRV-003): replace O(n) index rebuild with selective removal`.

### Nota — pendientes de strategy SIN evidencia de completado (no registrar como ✅)

`CLD-01/02/04` (cloud beta, pitch deck, case study), `OLD-001`, `VFY-008` (WAL fsync batching), `DRV-115` (MSVC linker), `DRV-117` (advisory ignores), `DRV-119` (ACID 0) — aparecen solo como **menciones** en ROADMAP/GO_TO_MARKET/backlog-guide; no hay task file, commit de fix, ni fila de progreso que demuestre completado. Siguen pendientes o sin trackear.

## Limpieza masiva 2026-08-07 (225 filas / 221 IDs eliminados de docs/Backlog.md)

Accion: se eliminaron del catalogo activo todas las filas completadas (✅) — 225 filas, 221 IDs unicos — aplicando la politica nueva (completadas se eliminan del backlog, no se tachan). El registro de completado vive en docs/progreso/README.md; el por que de cada cierre quedo documentado en la propia fila del backlog antes de eliminarla (commits, fechas, ADRs).

IDs eliminados por area:

- **P0/P1 (Security & Critical):** DEVOPS-15, META-001, INV-001, INV-024, AUDIT-01..08.
- **P4 Engineering:** DEBT-01, TECH-01..08, INV-002..005, AUDIT-05..08.
- **AUDREP (P13):** AUDREP-01..62 (todos resueltos 2026-08-05..07, commits en las filas; incl. AUDREP-04), DEPS-01, NV-01, NV-04.
- **P10 (Competitive catalog):** COMP-006/008/009/010/012..019/021..029 — catalogado, decisiones registradas.
- **P9/P11 (Docs/GitHub):** OLD-02/03/08..12/14/16/19..21, GH-119/122/123/124/128/129/131/132/139..144.
- **Investigaciones:** INV-001..025 (reportes en docs/research/), REC-*.
- **MKT/DISC/NUEVO/WEB/TSK:** campanas ya cerradas (MKT-03/04/05/10/14..17, NUEVO-*, TSK-103/104/106/107, WEB-001/18).
- **P12 DESKTOP base:** DESKTOP-02..11 (scaffold Tauri, conexiones, commands, frontend MVP, IQL, server wire, MCP spawn — ✅ 2026-08-06).
- **P14 REVIEW:** REVIEW-01/02/03/05 (cerrados).
- **Otros sin ubicacion por fase:** `GFI-01` (18 Good first issues creados en GitHub #118-#145), `SDK-02` (`similar_to_key()` ✅ 2026-07-31), `SDK-04` (`search_multi`/`search_all` ✅ 2026-07-31).

Nota de integridad: filas eliminadas sin entrada en docs/progreso/README.md (fases cerradas por blockquote P2/P3/P7, items reference-only) pasan al historico aqui; fuente canonica para re-auditar: docs/audit-reports/* + secciones por fase de este archivo.

## Archivado DESKTOP 2026-08-20 (10 tareas obsoletas por dirección P26 Vanta Studio)

Acción: se eliminaron del catálogo activo (docs/Backlog.md Phase 12) las tareas del modelo "app multi-connection con 6 vías" que P26 Vanta Studio (Fases 0-4 ✅, completada 2026-08-20) reemplazó por un modelo de **transporte pluggable** (nativa embebida / HTTP `/api/v2/*` / WASM-OPFS standalone). `ConnectionSelector.tsx` ya había sido eliminado deliberadamente en ADMIN-03 (commit `847ab080`). El header del Phase 12 y las filas re-scopeadas (DESKTOP-23/26/27) + priorizadas (24/25) documentan el estado nuevo.

IDs archivados y por qué:

- **DESKTOP-12/13/14 (cliente rmcp / McpConnection / UI MCP)** — obsoletas: el motor está embebido (`vantadb` path `../..` en `desktop/src-tauri/Cargo.toml`); conectar la UI vía MCP stdio duplicaría la misma DB (regla 1-escritor por path → `VantaError::Lock`). `McpSpawn` (DESKTOP-11) ya existe solo como sidecar del server, no como vía UI.
- **DESKTOP-15/16/17/18 (drivers/connections Node + Python)** — obsoletas: ya deferidas en scoping 2026-08-05 (valor marginal, empaquetado frágil); F4 WASM/OPFS las supera como vía alternativa; Tauri no puede `require()` napi.
- **DESKTOP-19 (ConnectionManager completo: path_holders, capability gate, routing por id)** — obsoleta parcial: `ConnectionManager` ya existe (DESKTOP-06: registry HashMap + active_id + 14 métodos, commit `9d2d5319`); el lock de path ya lo da `NativeConnection` (DESKTOP-05 → `VantaError::Lock`); el resto era dead weight sin UI multi-connection.
- **DESKTOP-21 (UI multi-connection)** — obsoleta: contradice ADMIN-03 (`ConnectionSelector.tsx` eliminado, commit `847ab080`); el Studio es single-connection con transporte pluggable.
- **DESKTOP-22 (Eventos Tauri streaming)** — obsoleta: progreso de import ya cubierto por F2 (ImportDrop); SSE quedó "sin asignar" en la DEFER table del plan F4 (`docs/plans/archive/2026-08-19-vanta-studio-fase4.md`).

Re-scopeadas (siguen en catálogo, alcance ajustado al modelo Studio):

- **DESKTOP-23** → persistencia de preferencias UI (tema/layout/lentes/filtros), no "vías guardadas".
- **DESKTOP-26** → tests frontend del Studio (vitest, hoy no configurado); Rust ya tiene tests.
- **DESKTOP-27** → docs + ADR del modelo real (transporte pluggable; ADR-026/027/028 ya existen), no multi-connection 6 vías.

Priorizadas:

- **DESKTOP-24** (empaquetado NSIS/MSI) y **DESKTOP-25** (CI GitHub Actions desktop) → quedan como pendientes 🟡 del desktop, para ejecutar cuando se abra el plan de packaging.

Sin eliminar del catálogo: DESKTOP-20 ✅ (shutdown_all, `45f8bed8`), ADMIN-01..09 ✅ (consola admin), DESKTOP-02..11 ✅ (migrados a docs/progreso/README.md en la limpieza 2026-08-07).

---

## Gran limpieza 2026-08-25 (auditoría backlog + research huérfanas)

> **Origen:** auditoría completa del backlog (sub-agente vanta-research + verificación lead) tras crear P38.
> **Resultado:** backlog de 999 → 546 líneas. 84 tareas realmente activas (reconteo GOV-C7).
> **Nota:** sesión paralela del mismo día (batches 2026-08-25) ya había removido 31 filas y cerrado MCP-24; esta pasada fue complementaria.

### Completadas removidas (96 IDs, filas fuente sincronizadas con los DONE de la sesión P35)

- **P0/P1:** RELEASE-01..03, SEC-01
- **P12/ADMIN:** DESKTOP-20, ADMIN-01..09
- **P15:** ERR-010, ERR-021, ERR-022, ERR-035, ERR-037
- **Hallazgos:** REVIEW-07 (dup de BND-06), REVIEW-08 (`ff9b2933`), REVIEW-11 (`bf474822`), REVIEW-15 (`57090e0e`), REVIEW-16 (SKIP), REVIEW-19, REVIEW-20 (SKIP)
- **P17/P18/P19:** TSYS-12, TIR-01/02/04/05/06/07/08 (decisiones registradas; follow-ups vivos → RES-10 en P38), R4 (decidido no hacer)
- **P22/P25/P26 MCP:** MCP-01..15, T15, MCP-16..23, MCP-25/26/28/29, MCP-31/32
- **P26 Vanta Studio:** VS-00..11, VS-CORE-01..07 (sección completa ejecutada Fases 0-4)
- **GOV/P32/P33/P34/P36:** BND-01/02/06, GOV-TK6, MOD-02 (`db8b26b7`), MOD-06, MOD-08+09 (`5aa42007`), MOD-16, MOD-19 (`dc65c242`), FIND-04 (`9de39702`), FIND-06, FIND-27 (`447a07d7`), FIND-28 (`2d9fa75f`), UX-01+UX-05 (`6260938e`), UX-18 (Wontfix), AGT-05

### Secciones no-tarea colapsadas/migradas

- Header: blockquotes históricos de syncs comprimidos a puntero al historial
- Fases cerradas colapsadas a tumbas: Phase 0/1/4/7/10/11/12/13, P15, P20, P29-P31
- P35 (registro de sesión batch 2026-08-24): eliminada del catálogo — el contenido vivo (REVIEW-10/12, hallazgos) vive en "Hallazgos pendientes de reportes"; el log de sesión vive aquí y en `docs/plans/archive/2026-08-24-batch-review-mod-find.md`
- Bloque `=== RECITATION DESKTOP-24 ===` eliminado (estado de sesión, no tarea)
- GOV-TK1/TK9: filas partidas por escape corrupto reparadas
- Duplicado AUD-043 ≡ FIND-30: resuelto por sesión paralela (FIND-30 removida como done; AUD-043 re-derivada pendiente desde audit-full-20260825)

### Convención

Filas completadas NO viven más en el Backlog: se eliminan al completar y su registro va aquí (o a docs/avance/<dominio>.md). Ver progreso SKILL.md Trigger 1 y AGENTS.md "Progreso Skill".

## Limpieza DAUD 2026-08-26 (DESKTOP-QW5, H-13)

> **Origen:** `docs/plans/2026-08-25-research-desktop-quickwins.md` Wave1 Task5 (DESKTOP-QW5). **Motivo:** filas DAUD-01..09 en P37 marcadas ✅ Hecho/Cerrada pero aún presentes en `docs/Backlog.md` como stale — fixes ya aplicados en commits `3c53d8b2`, `480935a7`, `b865c625`; DAUD-02 resuelta por `ad0f34b1` (DESKTOP-QW4); DAUD-08 stash `06aa1a86` consumida por `b865c625` (actual stash `2fc26b26`).
> **Acción:** eliminadas 9 filas `| \`DAUD-01..09\`` de `docs/Backlog.md` P37 + colapso P37 a `0 — ✅ 9/9 ejecutadas` (Exec Summary 118→109 activas, `last_reviewed` 2026-08-26). Registro de dominio en `docs/avance/activo/desktop.md` §P37 + este historial. Plan quickwins Wave1 5/5.

| ID | Destino | Commit/evidencia |
|---|---|---|
| `DAUD-01` | `desktop/e2e/daud01-temas.spec.ts` + `flujo-critico.spec.ts` guard E2E-VISUAL | `480935a7` |
| `DAUD-02` | `desktop/src/components/layout/WorkspaceShell.tsx:295` `filterActive` | `ad0f34b1` (QW4) |
| `DAUD-03` | `desktop/src/App.css:53-59` press-effect scopeado | `3c53d8b2` |
| `DAUD-04` | `desktop/src/index.css` body consolidado | `3c53d8b2` |
| `DAUD-05` | `desktop/src/index.css` dead utilities borradas | `3c53d8b2` |
| `DAUD-06` | `desktop/src/components/layout/WorkspaceShell.tsx` Pencil + `Mark.tsx` neon | `b865c625` |
| `DAUD-07` | `desktop/DESIGN_DECISIONS.md` §5 convención iconos | `3c53d8b2` |
| `DAUD-08` | `git stash` `06aa1a86` → consumido por `b865c625` (no dropeo manual) | `b865c625` |
| `DAUD-09` | D1-D11 commit agrupado | `3c53d8b2` + `b865c625` |

### MOD-41..45 (derivados del review providers 2026-08-23 - nunca ejecutados)

> **Fecha de archivo:** 2026-08-25 · **Origen:** INV-providers-01 H-13 (perdida de trazabilidad detectada).
> Las filas MOD-41..45 derivadas de `docs/reviews/modulos/providers.md` (fase P32) desaparecieron
> del Backlog sin registro aqui ni en avance/ (grep 0 resultados en ambos arboles al 2026-08-25).
> Estado: **SUPERADAS** por la investigacion INV-providers-01 (`docs/reviews/research-providers-20260825.md`),
> que re-cubro su contenido con evidencia fresca:
>
> | Vieja | Contenido | Nueva fila |
> |---|---|---|
> | MOD-41 | Tests rotos (P1/P2) | PROV-02 (Backlog P45) |
> | MOD-42 | Sin distribucion PyPI (P4) | PROV-12 (decision HITL 2026-08-25: publicar wheels) |
> | MOD-43 | Duplicacion ~85% (P6) | PROV-05 |
> | MOD-44 | Stubs .pyi stale (P3) | PROV-03 |
> | MOD-45 | Nits unwrap/README/importorskip (P7-P9) | PROV-06/07/08/09 |

### P43 Research web quickwins 2026-08-27 (INV-web-01)

> **Origen:** `docs/plans/2026-08-25-research-web-quickwins.md` Wave1 Task2 (WEB-04).

| ID | Destino | Commit/evidencia |
|---|---|---|
| `WEB-04` | `docs/avance/activo/web-frontend.md` §WEB-04 + task file `.opencode/skills/campaign-executor/tasks/WEB-04.md` | Build exit 0 verificado + 5 layouts español consistente (grep manual) |

### FIND-37 — dispatcher híbrido sparse unwrap (2026-08-27)

> **Origen:** `codegraph-20260827-143245` Fase 9, plan `docs/plans/2026-08-27-backlog-pipeline.md` Task 1 (Wave 0). Gap verificado: 6 unwraps en `src/sdk/search/mod.rs:207,240,265,315,346,369` + 3 en `debug_ops.rs:288,335,374` — `has_sparse` bool garantizaba Some en teoría pero violaba `clippy::unwrap_used` y era panickable en prod si request sin sparse llegaba a hybrid.

| ID | Destino | Commit/evidencia |
|---|---|---|
| `FIND-37` | `docs/avance/activo/core-engine.md` §FIND-37 + task file `.opencode/skills/campaign-executor/tasks/FIND-37.md` | Commit `bd7c2691` `fix(search): FIND-37 eliminate query_sparse.unwrap panics` — 2 files `src/sdk/search/mod.rs` + `debug_ops.rs` (32+32 lines), `rg query_sparse.*unwrap` → 0, `cargo check` ✅, `nextest -E 'test(search)'` 157 passed, fmt/clippy hook ✅ |
| `FIND-34` | `docs/avance/activo/core-engine.md` §FIND-34 + task file `.opencode/skills/campaign-executor/tasks/FIND-34.md` | `fix: FIND-34 — WAL writer DAG justification + recovery/quarantine edge tests` — `src/wal.rs` doc DAG 15L + 2 tests (mid-file scan-forward + quarantine rotation), `cargo nextest -E test(wal)` 62/62 ✅, `rg` 1 def c/u, `codegraph_explore` justificado como falso positivo Leiden (no SCC) |
