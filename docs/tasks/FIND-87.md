# FIND-87: `./native` en exports + nota wiki (TS packaging)

## Metadata
- **Plan file:** docs/plans/2026-09-15-find-correcciones.md (Task 10, Wave3)
- **Fuente:** plan Task 10 + Backlog FIND-87
- **Esfuerzo:** 🟡 (4h appetite, ajustado de 🟢 por decisión packaging)
- **Prioridad:** 🟡
- **Tipo:** TypeScript SDK + Docs (Mixto TS/docs)
- **Turns estimados:** 8
- **Creado:** 2026-09-15
- **last-synced:** 2026-09-15
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-ts/src/__tests__/native-error.test.ts` (import `../native.js` directo, no vía subpath); `vantadb-node/bench/bench-abi.mjs`; consumidores externos `import { NativeVantaDB } from "vantadb"` (vía `.` re-export) |
| Callees | `vantadb-node` (napi-rs, dynamic `import("vantadb-node")` en `native.ts:165`); `dist/native.js` + `dist/native.d.ts` (artefactos tsc, ya emitidos — verificado en disco) |
| Implicaciones | contrato: nuevo subpath `./native` aditivo, `.` intacto; comportamiento público: `NativeVantaDB` ya expuesto vía `.` (`vantadb.ts:1487` `export * from "./native.js"`), subpath es alias de descubrimiento, no símbolo nuevo; performance: nula (solo package.json); serialización: nula; migración: nula; tests: `native-error.test.ts` no afectado (import relativo) |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-ts/package.json` (68 líneas), `docs/api/TS_SDK.md` (703 líneas), `vantadb-ts/src/native.ts` (1-160 + 160-361 vía codegraph), `vantadb-ts/src/vantadb.ts:1477-1487` (re-export), `vantadb-ts/src/__tests__/native-error.test.ts` (84 líneas), `vantadb-ts/README.md` (375 líneas, §vantadb vs vantadb-node :152-164 + SSR :129-131), `vantadb-ts/tsconfig.json` (15 líneas), `.opencode/rules/js-ecosystem.md`
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** `native.ts` → `./errors.js`, `./guards.js`, `./types.js`, `vantadb-node` (type-only + dynamic import); `vantadb.ts` → `vantadb-wasm`, `./errors.js`, `./guards.js`, `./metadata.js`, `./native.js` (export *); `package.json` files: `dist/`, README, LICENSE
- **Archivos que referencian a los editados (referencias entrantes):** `package.json` exports consumido por Node resolver + `npm pack`; `TS_SDK.md` referenciado por `related: [PYTHON_SDK.md, NODE_SDK.md]` y README cross-links; `native.ts` referenciado por `vantadb.ts:1487`, `native-error.test.ts:2`, `bench-abi.mjs`
- **Veredicto impacto:** BAJO — 2 archivos editados (package.json + TS_SDK.md), ambos aditivos; `dist/native.*` ya existe en disco (tsc lo emite, `include: src/**/*.ts` menos tests); `files: ["dist/"]` ya lo empaqueta; ningún test importa vía subpath hoy → 0 breakage posible. Riesgo empaquetado cubierto por `npm pack --dry-run`.

## Contrato

"`./native` en `vantadb-ts/package.json` exports (o decisión documentada de no exponer) + nota wiki/sección native explícita en `docs/api/TS_SDK.md` + `npm run build` + `npx vitest run` verdes en `vantadb-ts` + `npm pack --dry-run` con el export sin errores"

## Spec (SDD — export = nuevo subpath público, decisión por evidencia)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Exponer `./native` como subpath | A) Exponer `./native` → `dist/native.js`+`.d.ts` (pro: descubrimiento `import { NativeVantaDB } from "vantadb/native"`, tree-shaking dedicado, wiki enlazable; contra: +1 superficie que mantener, riesgo pack si dist no lo emite) / B) No exponer, documentar acceso vía `.` (pro: 0 superficie nueva — `vantadb.ts:1487` ya re-exporta `* from native.js`; contra: consumidores no descubren native, gap original persiste) | A | ✅ decidido-por-evidencia: `dist/native.js`+`native.d.ts` YA existen en disco (tsc `include src/**/*.ts` los emite; `files: dist/` los empaqueta); `native-error.test.ts` prueba el módulo; README `:129-131,:161-164` ya documenta `NativeVantaDB.connect()` como backend nativo — el subpath hace esa doc importable. Riesgo pack mitigado por dry-run antes/después |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** `.` export intacto (re-export `* from native.js` no se toca); `NativeVantaDB` API intacta (sin cambios en `native.ts`); `engines node>=22.19` intacto; `files: dist/` intacto; prohibidos intactos (`.opencode/`, `completions/`, `desktop/src-tauri/Cargo.lock`, providers/* FIND-73, embeddings/* FIND-71, `Cargo.toml`, otros workflows); WIP ajeno en `git status` NO se commitea
- **Comandos de verificación:** `npm run build` (vantadb-ts) + `npx vitest run` (vantadb-ts) + `npm pack --dry-run` + `git diff --check`
- **Deuda pendiente:** ninguna (pack + wiki cierran el gap; sin deuda neta Regla 6)

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | FIND-87 — `./native` en exports + nota wiki (TS packaging) |
| `lastAction` | DISCOVERY completo + task file creado (este archivo) |
| `result` | PARTIAL (discovery ✅, steps ejecución pendientes) |
| `nextAction` | Step 1: añadir `./native` a exports + build + pack dry-run |
| `contract` | ver ## Contrato + ## Invariantes de dominio |
| `nextTask` | FIND-73 / FIND-71 (paralelos Wave3, archivos disjuntos) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable se cumple + `npm run build` + `npx vitest run` + `npm pack --dry-run` verdes |
| **Commit** | Commit atómico (~100 líneas), `docs:`/`feat:` + FIND-87, solo 2 archivos propios, `git diff --check` limpio |
| **Release** | N/A (docs + packaging, sin cambio de versión; release-plz lo bumpa solo) |

## Herramientas necesarias

- `codegraph_explore` (blast radius) ✅ usado
- bash directa (`npm run build`, `npx vitest run`, `npm pack --dry-run`, `git diff --check`) — `campaign_verify_cmd` con bug exit -1 conocido → fallback documentado
- `git` (diff/add/commit solo archivos propios)

**Skills cargadas (SDP):** campaign-executor (base type TS SDK), progreso (base), source-driven-development (base type TS SDK), incremental-implementation (lifecycle BUILD slices delgados), test-driven-development (lifecycle BUILD), context-engineering (lifecycle BUILD sesión nueva), doubt-driven-development (lifecycle BUILD stakes packaging público), frontend-ui-engineering (lifecycle BUILD — N/A para este slice docs/packaging, cargada por SDP sin uso), api-and-interface-design (lifecycle BUILD interfaces públicas — subpath `./native`)
- **SDP:** `campaign_discover_skills_v2 archivosClave="vantadb-ts/package.json:11-22, docs/api/TS_SDK.md:25" phase="BUILD" contractKeywords=["ts-exports","npm-pack","native-binding","TS_SDK-docs"] maxSkills=8` → 8 skills (ver lista arriba)

## Investigation Notes

- **Código (fuentes verificadas):**
  - `vantadb-ts/package.json:11-22` exports = `.` + `./types`, SIN `./native` (gap confirmado, coincide plan)
  - `docs/api/TS_SDK.md:25-26` solo nota Node 18+ ("Requires Node.js 18+... no native build step") — SIN sección native/wiki (gap confirmado); además `:26` dice "no native build step" que es cierto para WASM pero el backend native SÍ requiere `.node` — la nueva sección lo aclara sin tocar la nota WASM
  - `vantadb-ts/src/vantadb.ts:1487` `export * from "./native.js"` → `NativeVantaDB` YA accesible vía `.` (codegraph + lectura directa). El subpath es alias de descubrimiento, no símbolo nuevo → Gate D no dispara (blast radius 2 archivos, sin hot path, contrato no ambiguo)
  - `vantadb-ts/src/native.ts:130` `class NativeVantaDB` + `:163-174` `connect()` con dynamic `import("vantadb-node")` + `wrapNativeError` — lazy, browser-safe (browsers usan WASM wrapper)
  - `vantadb-ts/src/__tests__/native-error.test.ts` 5 tests TS-02/ERR-TS-01 (import relativo `../native.js`, no afectado por subpath)
  - `vantadb-ts/tsconfig.json` `include: src/**/*.ts`, `exclude: __tests__`, `outDir: dist`, `declaration: true` → `dist/native.js`+`native.d.ts` emitidos (verificado en disco: existen)
  - `vantadb-ts/README.md:152-164` tabla `vantadb` vs `vantadb-node` + `:129-131` SSR "prefer NativeVantaDB, lazy dynamic import" — precedente de doc native que la sección TS_SDK amplía (no duplica)
  - Node engines: `package.json:7` `node>=22.19` vs TS_SDK `:694` "Node.js 18+ ✅" y README `:359-360` "22.12+ ✅ / 18–22.11 ESM-only" — la sección native cita `>=22.19` (fuente package.json) sin reabrir FIND-78 (nota engines Node, otro task)
- **Riesgo empaquetado (pre-mortem plan):** `files: ["dist/"]` + dist ya contiene `native.*` → dry-run debe listar `dist/native.js`+`native.d.ts`; si falta → STOP a decisión documentada (no forzar)
- **Reglas:** `.opencode/rules/js-ecosystem.md` R-2 (`dist/` artefacto no commiteado — no citar como fuente), R-3 (vantadb-node standalone intencional), R-1 (persistencia solo vía backends explícitos — la sección native nombra `connect(path)` fjall/WAL como diferencial vs WASM CODE-089)
- **Internet:** sin ambigüedad en spec `exports` (Node docs subpath pattern estándar `./native` → `./dist/native.js`; verificado contra `package.json` existente `./types` que sigue el mismo patrón) → sin web research (plan: solo si ambigüedad; no la hay — el patrón `./types` es el precedente interno)

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: sin trust boundaries nuevos (sin input usuario, sin auth, sin deps nuevas/bump, sin storage nuevo, sin FFI nuevo — `native.ts` intacto, solo `package.json` exports + docs). Justificación: cambio packaging/docs aditivo.
- [x] **PERFORMANCE** — no aplica: sin hot path (no `vector/`, no `engine.rs`, no loops search/ingestión, no serialización). Justificación: 0 líneas de runtime.

## Steps

### Step 1: `./native` en exports + build + pack dry-run
- **Archivos:** `vantadb-ts/package.json`
- **Acción:** añadir subpath `./native` (`types: ./dist/native.d.ts`, `import`+`require`: `./dist/native.js`) siguiendo el patrón existente de `./types`; NO tocar `.`, `files`, `engines`, deps
- **Verify:** `npm run build` (tsc) + `npm pack --dry-run` lista `dist/native.js`+`native.d.ts` + `git diff --check`
- **Estado:** ✅ COMPLETED (build tsc ✅ sin errores; pack dry-run ✅ 20 files con dist/native.js 13.4kB + native.d.ts 5.1kB; exports JSON ✅ con ./native; diff-check ✅ exit 0)

### Step 2: Sección native/wiki en TS_SDK.md + vitest + cierre
- **Archivos:** `docs/api/TS_SDK.md`
- **Acción:** añadir sección `## Native backend (Node.js, `NativeVantaDB` via `vantadb/native`)` tras `## WASM vs Node.js Differences` (o junto a Runtimes): qué es, cuándo usarlo (persistencia fjall/WAL vs WASM in-mem CODE-089), import vía `.` y vía `./native`, ejemplo `connect()`, fallback platform-specific (browsers → WASM), engines `>=22.19` (fuente package.json), link a `vantadb-ts/src/native.ts` + README tabla vantadb vs vantadb-node; corregir NADA fuera de la sección (nota `:26` intacta)
- **Verify:** `npx vitest run` + `git diff --check` + `npm pack --dry-run` (re-confirma con docs, opcional)
- **Estado:** ✅ COMPLETED (vitest ✅ 12 files 311 passed 2.71s; dist/native.js import ✅ exports NativeVantaDB,wrapNativeError; diff-check ✅; sección en TS_SDK.md:690 entre WASM-vs-Node:681 y Runtimes:739)

## Dependencias
- Previa: FIND-70 ✅ (sin archivos compartidos)
- Paralelas Wave3: FIND-73 (providers/*), FIND-71 (embeddings/*) — archivos disjuntos, no colisionan
- Next: Wave4 (FIND-77 tras Wave0 por mismo archivo tools.rs — no afecta a este task)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** vanta-review (sub-agente distinto, ses ses_f5a578f83ffe4SxVn93OHVE6kB)
- **Enfoque:** exponer subpath correcto (alias descubrimiento, no símbolo nuevo; patrón espejo de `./types`; alternativa no-exponer deja gap). Wiki precisa sin contradecir README/TS_SDK (cita engines package.json:7, async vs sync, fallback WASM, CODE-089).
- **Cómo se probó:** review reprodujo package.json:22-26, node import dist/native.js → NativeVantaDB,wrapNativeError, pack dry-run con native.* 20 files, TS_SDK.md:690 ubicación, git diff --check exit 0; build/vitest aceptados sin re-ejecución (0 líneas runtime).
- **Checklist anti-hábitos tóxicos:**
  - [ ] No inventar salidas de comandos/herramientas que no se ejecutaron.
  - [ ] No saltarse la clarificación por "ya sé qué quiere".
  - [ ] No declarar done sin verificar contra los acceptance criteria.
  - [ ] No ignorar fallos ni reportar "todo OK" cuando hubo fallo parcial.
  - [ ] No hacer un solo intento de búsqueda y darlo por saturado.
  - [ ] No copiar sin citar ni presentar supuestos propios como evidencia.
  - [ ] No reintentar en bucle sin diagnóstico.
  - [ ] No dejar huérfanos los pasos: cada paso conectado al objetivo.
  - [ ] No degradar el chequeo de errores en paths de dinero/seguridad.
  - [ ] No gastar presupuesto infinito; paradas explícitas.
- **Veredicto:** ✅ approve (sin nits bloqueantes; sugerencia futura exports["./package.json"] en task separado)

## Notas
- Ponytail full: escalera — existe (`./types` precedente) > stdlib (Node subpath exports) > 0 deps nuevas > mínimo diff (5 líneas package.json + 1 sección docs). Sin abstracciones.
- Regla 11: 0 claims sin fuente — cada número/fuente citada (package.json:7, TS_SDK:25-26, vantadb.ts:1487, native.ts:130/163, README:129/152, tsconfig).
- WIP ajeno en `git status` (providers, embeddings, completions, tauri lock, pipeline-state) NO se toca ni commitea.

## Context Save Point
- **Fecha:** 2026-09-15
- **Branch:** develop
- **CI pendiente:** no (verificación local: build ✅ / vitest 311 ✅ / pack ✅ / diff-check ✅)
- **Decisiones:** exponer `./native` (Spec #1, evidencia dist ya emitido + re-export existente + README precedente); sección wiki en TS_SDK.md:690 (no tocar nota :26)
- **Problemas conocidos:** `campaign_verify_cmd` corre en repo root sin workdir → `npm run build` falla ENOENT package.json (no es el bug exit -1 del plan, es limitación cwd); fallback bash directa con workdir vantadb-ts documentado. WIP ajeno en git status (providers FIND-73, embeddings FIND-71, completions, tauri lock, pipeline-state) NO commiteado
- **Próxima tarea:** FIND-73 / FIND-71 (Wave3 paralelos, archivos disjuntos)
