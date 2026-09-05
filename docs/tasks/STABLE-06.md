# STABLE-06 — gate npm TS como Fast Gate

## Metadata
- **Plan file:** `docs/plans/2026-09-04-durability-release-readiness.md` (Task 6, Wave 1)
- **Creado:** 2026-09-05 (DISCOVERY)
- **Estado:** ⏳ IN PROGRESS
- **Ruta:** vanta-worker
- **Branch:** develop
- **Commit:** `ci(ts): gate npm Fast Gate medido (STABLE-06)`

## Contrato
`npm ci && npm run build && npx vitest run` verde + `npx eslint .` 0 + `npm pack` incluye `engines` + tiempo medido en CI limpio (<5 min o justificado como Heavy con evidencia).

## Gate Justificación
Validación 🟠 para promoción default-members. DISCOVERY primero: verificar claim "264 tests / 26s" contra disco; si difiere, re-escalar con evidencia sin inflar.

## SDP
`campaign_discover_skills` phase=BUILD keywords=[npm, vitest, eslint, Fast Gate, release-npm] → campaign-executor, progreso, ponytail (base auto-vía-MCP), source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development. Tipo detectado: `typescript` (checks: npx tsc --noEmit, npm test). Esta tarea es validación/medición CI — no lógica nueva: TDD N/A (sin RED/GREEN, gate mecánico); incremental slice único (medir → gatear workflow → commit).

## Blast Radius
| Área | Archivos | Riesgo |
|------|----------|--------|
| TS SDK manifest | `vantadb-ts/package.json` (engines node>=22.19, files dist/, scripts build/test/lint) | Bajo — solo lectura salvo que falte engines (ya presente) |
| TS SDK src | `vantadb-ts/src/` (6 fuentes + 9 test files en `__tests__/` + `tests/graph.test.ts`) | Nulo — no se edita src en esta tarea |
| CI workflow | `.github/workflows/release-npm-61.yml` (job tests: wasm-pack build + npm ci + build + npm test; publish-ts ya tiene guard TS-05 engines + smoke TS-07) | Medio — único archivo editable candidato (añadir lint/pack-gate si falta) |
| Dependientes | `vantadb-wasm/pkg` (file: dep), `vantadb-node` (file: devDep), examples/ | Nulo — no se tocan |

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `vantadb-ts/package.json` (66L), `.github/workflows/release-npm-61.yml` (240L), `vantadb-ts/vitest.config.ts` (25L), `vantadb-ts/eslint.config.js` (24L), `vantadb-ts/tsconfig.json` (15L), `vantadb-ts/scripts/smoke-pack.mjs` (106L), `docs/plans/2026-09-04-durability-release-readiness.md` Task 6 (§76-84).
- **Referencias hacia dentro:** `vantadb-ts` → `vantadb-wasm/pkg` (file: dep, requiere wasm-pack build previo en CI), `vantadb-node` (file: devDep solo tests), `dist/` (tsc outDir, excluido de eslint, incluido en files).
- **Referencias entrantes:** workflow `release-npm-61.yml` job `tests` corre en PRs que tocan `vantadb-wasm/**`/`vantadb-ts/**`; jobs `publish-wasm`/`publish-ts` dependen de `tests` (needs). `desktop/`, examples consumen el SDK pero fuera de scope.
- **Veredicto:** cambio acotado a CI/manifest. Cero código productivo. Si el gate ya pasa, el único edit es el comentario de tiempo medido en el workflow (TS-06) + este task file. Si falta lint/pack en job tests, añadir steps (aditivo, reversible).

## DISCOVERY — verificación claim "264 tests / 26s" vs disco (2026-09-05)
- **Conteo estático (rg `^\s*(test|it)\s*\(`):** 278 tests en 10 archivos (dx04 37, flat-metadata 7, hardening 72, integration 18, load 6, native-error 5, subclients 17, types 6, vanta 105, graph 5). Describes: 53. `test.skip/todo/each`, `it.skip`, `describe.skip`: 0. Sin skips.
- **Claim plan "264":** DIFIERE por +14 (+5.3%). Re-escalado: fuente real en disco = 278 casos estáticos; número vitest final manda (lo confirma `npx vitest run` en EJECUCIÓN). Sin inflar: se reporta 278 estático + N vitest medido.
- **Claim "26s":** pendiente de medición en EJECUCIÓN (build + vitest + eslint con `Measure-Command`). Workflow dice "~18s vitest + build/install" (comentario TS-06, línea 45-46). Se mide limpio local como proxy de CI; CI limpio (<5 min) se justifica con números.
- **Estado disco:** node v26.8.1, npm 11.6.0, `node_modules/` presente, `dist/` presente. `engines.node >=22.19` presente en package.json. `files` incluye dist/. Scripts: build=tsc, test=vitest run, lint=eslint .
- **Git:** branch develop, `M .opencode` (submodule pointer +7e05f8a, ajeno — NO TOCAR, no stagear). Sin otros cambios.
- **Gap preliminar workflow:** job `tests` NO corre `eslint` ni verifica `engines` en pack (eso solo existe en `publish-ts`: guard TS-05 + smoke TS-07). Contrato STABLE-06 exige ambos → candidato a añadir 2 steps al job tests (lint + pack-engines-check) si la medición local pasa.

## Steps
### Step 1: Medir gate local (build + vitest + eslint + pack-engines) ✅ COMPLETED
- **Medido 2026-09-05 (node v26.8.1, npm 11.6.0, `npm ci` limpio 10.84s):** build (tsc) ✅ 2.33s · vitest ✅ 278/278 (10 files, Duration 15.92–16.43s, wall ~21s) · eslint ❌→✅ (1 error `no-empty-object-type` en `WikiClient{}` → fix 1 línea disable con justificación D43) · `npm pack --dry-run` ✅ + engines `>=22.19` ✅. Total porción TS ~58s wall <5min → Fast Gate.
- **Re-escalado claim:** plan decía "264 tests / 26s" → real 278 tests (+14) / ~16s Duration (~21s wall). Sin inflar.
- **Estado:** ✅ COMPLETED

### Step 2: Gatear workflow como Fast Gate (solo si Step 1 verde) ✅ COMPLETED
- **Archivos:** `.github/workflows/release-npm-61.yml` (job tests)
- **Acción:** añadidos steps `Lint TypeScript` + `Verify packed manifest keeps engines` (TS-05 mirror); comentario TS-06 actualizado con números medidos (278 tests, tiempos, nota wasm-pack no medido local).
- **Verify:** `python yaml.safe_load` OK + pre-commit hook `actionlint ok`.
- **Estado:** ✅ COMPLETED

### Step 3: Verify contrato + commit atómico ✅ COMPLETED
- **Acción:** re-verificado post-`npm ci` (build+vitest 278/278+eslint 0) + `git add` SOLO 2 archivos + commit `7ff70b01` + plan file Task 6→COMPLETO (sin stagear) + ajenos intactos.
- **Verify:** `git log --oneline -1` = 7ff70b01.
- **Estado:** ✅ COMPLETED

## Notas
- NUNCA stagear `.opencode` (submodule dirty de otra sesión).
- `npm ci` borra node_modules: medirlo aparte (una sola vez) para no invalidar tiempos incrementales; el contrato pide `npm ci` pero la medición CI-limpio se aproxima con install limpio cronometrado.
- Si vitest <278 por bailout/fallo → debug con systematic-debugging, no re-intentar a ciegas. 2 fallas mismo-error → Gate V.
- Ponytail: un slice, mínimo diff (workflow + comentario medido). Sin refactors.

## Context Save Point
- **Fecha:** 2026-09-05 DISCOVERY
- **Branch:** develop
- **Próximo:** Step 1 (medir gate local)
