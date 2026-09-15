# FIND-63 — remanente vitest localStorage desktop (suite verde por archivo)

> Plan: `docs/plans/2026-09-15-find-correcciones.md` Task 4, Wave1 · Ruta: vanta-worker · Branch: develop · Appetite 4h
> Tipo auto-detect: **desktop** (`campaign_detect_task_type`: Desktop Tauri) · Gate D: no dispara (sin símbolos públicos nuevos, contrato no ambiguo, blast radius ≤6 archivos)
> SDP: `campaign_discover_skills_v2 archivosClave="desktop/vitest.config.ts desktop/src/store/undo.test.ts" phase="BUILD" contractKeywords=["vitest","jsdom","localStorage","typescript"]` → campaign-executor, frontend-ui-engineering, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design + sugeridas systematic-debugging (keyword-mapped, cargada manual)
> SKILLS_CARGADAS: campaign-executor, brainstorming, writing-plans, planning-and-task-breakdown, progreso, ponytail(full), test-driven-development, systematic-debugging, incremental-implementation, frontend-ui-engineering, source-driven-development, context-engineering

## HALLAZGO — task file STALE reconciliado (2026-09-15)

El task file describía `SyncMode::Never en src/wal.rs` (3 steps DONE 2026-09-04, commit a7285969). Ese scope está CERRADO y verificado. El plan 2026-09-15 re-scopeó FIND-63 a **remanente vitest localStorage desktop**. NO se continúan los steps wal. Divergencias vs plan reportadas como HALLAZGO (no silenciadas):
- H1: plan cita `desktop/src/store/projection.worker.test.ts:1` — ruta real `desktop/src/components/space/projection.worker.test.ts:1` (único con `// @vitest-environment node`, correcto por worker UMAP).
- H2: plan cita 13 fails originales — reproducidos hoy 13/86 (11 undo + 1 MemoryLens + 1 ProxyDashboard), causa raíz `localStorage` undefined bajo Node v26.8.1 (warning experimental `--localstorage-file`), no config global.
- H3: `npx tsc --noEmit` reporta 4 errores extra en `transport.ts`/`vanta-wasm-map.ts` (`VantaDB` vs `Client` del pkg wasm-bindgen) — entra al contrato tsc 0 como Slice 2.
- H4: `.opencode/rules/frontend-web.md` cubre solo `web/`, no `desktop/` — sin regla aplicable al scope; se sigue patrón DESKTOP-23/26/30 existente.

## Objetivo (re-scopeado)

Suite desktop verde sin tocar config global: fijar remanente/ambiente **por archivo**. `desktop/vitest.config.ts:10` YA tiene `environment: jsdom` (solo lectura). Acceptance: `npx vitest run` 0 fails + `npx tsc --noEmit` 0 (conteo final el que reporte la suite).

## Contrato (ley)

`npx vitest run` en `desktop/` 0 failed + `npx tsc --noEmit` 0 errores. Commit `fix: FIND-63 — ...` solo tras verify. No tocar QUICKSTART (FIND-74 Wave8). Prohibidos: `src/wal.rs`, `.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, GOV-C4 stash.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/vitest.config.ts` (10: jsdom global, exclude e2e + 6 node:test files), `desktop/src/store/undo.test.ts` (54: `localStorage.clear()` asume jsdom compartido), `desktop/src/components/space/projection.worker.test.ts:1` (node explícito, NO tocar), `desktop/src/store/undo.ts` (21-27: `defaultStorage()` con guard `typeof localStorage`, singleton `undoStore:314`), `desktop/src/components/proxy/ProxyDashboard.tsx:17-33` + test `:23` (acceso directo sin try en test), `desktop/src/components/memory/MemoryLens.tsx:170,172` + test `:79` (setItem directo), `desktop/src/transport.ts:10,91,97`, `desktop/src/vanta-wasm-map.ts:14`, `vantadb-wasm/pkg/vantadb_wasm.{js,d.ts}` (exporta solo `Client`), `desktop/package.json` (vitest 4.1.11, jsdom 30, Node v26.8.1), `desktop/tsconfig.json` (exclude `*.test.ts`).
- **Referencias hacia dentro:** `undo.ts` → `../vanta` (mockeado en test); `proxyUrl/setProxyUrl` → `localStorage` directo con try en fuente, sin try en test; `MemoryLens PersonaPanel` → `localStorage` directo; `transport WasmBackend` → `mod.Client` (runtime) + tipo `Client`; `vanta-wasm-map` → solo tipo.
- **Referencias entrantes (codegraph_explore):** `undo` 2 callers en script impeccable (no runtime) + test propio; `transport` 38 callers en `vanta.ts`/ConsolidateLens + test `vanta.test.ts` (no toca WASM path por default Tauri/HTTP); `server_client.rs::transport` homónimo Rust sin relación. `useMetricsPoll.test.tsx` usa fake timers aislados (pasa, no tocar).
- **Veredicto:** editar SOLO 3 tests (stub localStorage por archivo) + 2 fuentes (`transport.ts`, `vanta-wasm-map.ts`) + 1 test (`vanta-wasm-map.test.ts` consistencia, fuera de tsc pero mismo drift). NO tocar `vitest.config.ts`, `projection.worker.test.ts`, `undo.ts`, `connections.ts`, stores con fakeStorage (patrón sano), ni scope wal viejo.

## Steps atómicos

- [x] Step 0 — DISCOVERY: tipo desktop, blast radius, RED reproducido (13/86 fails + tsc 4 errores), task file reescrito con HALLAZGOS H1-H4 y sección legacy archivada. Verify: este archivo existe con Impacto mapeado lleno (Gate Regla 0).
- [x] Step 1 — GREEN vitest: stub `localStorage` en memoria por archivo en `undo.test.ts`, `ProxyDashboard.test.tsx`, `MemoryLens.test.tsx` (guard `globalThis`, sin config global, sin fake timers). Verify: 3 archivos 22/22 ✅ + full `npx vitest run` 14 files 86/86 ✅.
- [x] Step 2 — GREEN tsc: `Client as VantaDB` alias + `mod.Client` runtime en `transport.ts`, alias en `vanta-wasm-map.ts` y `vanta-wasm-map.test.ts`. Verify: `npx tsc --noEmit` 0 errores + suite sigue verde + `git diff --check` limpio.
- [x] Step 3 — CIERRE: review vanta-review approve (2 nits no bloqueantes; H3-doc aplicado, H2-consistente con patrón repo), DoD, commit `fix:`, recitation, RESULTADO. Verify: contrato + Gates.

## Decisiones

- Opción A (elegida): stub por archivo (duplicado DAMP, ~10 líneas por test) — respeta Stop (no global), aisla contaminación (vitest isolate por archivo), preserva semántica compartida de undo (`clear()` entre tests).
- Opción B (descartada): mock/setup global en config — contamina `projection.worker.test.ts` (node) y viola Stop explícito.
- tsc: alias `Client as VantaDB` (1 línea por archivo) + fix runtime `mod.Client` — mínimo diff vs rename total; comentario documenta divergencia pkg.

## Scope previo wal-Never ya cerrado (legacy archivado, NO continuar)

`SyncMode::Never` en `src/wal.rs:376-389` — 3 steps DONE 2026-09-04, commit a7285969 (+62/-9 solo `src/wal.rs`), test `test_sync_mode_never_skips_auto_sync` RED→GREEN, suite wal 63/63, clippy/fmt ✅. Detalle original preservado en git history; este archivo ya no trackea ese scope.
