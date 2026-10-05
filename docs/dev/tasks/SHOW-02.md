---
title: SHOW-02 — recetas clicables del playground (5-6)
kind: task
description: "Verificación E2E de las 6 recetas del playground (repo ness-e/Vantadb-web) contra WASM real; defecto de clicabilidad detectado y fixeado + guard E2E."
---

# SHOW-02 — recetas clicables del playground (5-6)

## Metadata

- **Plan file:** docs/dev/plans/2026-10-04-master-plan-0.9.0.md (Task 35; campaign taskId `35`)
- **Fuente:** Backlog SHOW-02 (prerrequisito del anuncio) + master plan F1 Task 35 (expandido a F0)
- **Esfuerzo:** 🟡 2-3d (contrato re-scopeado a verificación E2E; delta real = 1 fix puntual + guard)
- **Prioridad:** 🟠
- **Tipo:** Verificación de ejecución (showcase) + fix puntual web — sin lógica nueva
- **Turns estimados:** 6
- **Creado:** 2026-09-19 (ejecución previa, repo viejo) · **Re-scopeado:** 2026-10-05
- **last-synced:** 2026-10-05
- **Estado:** ✅ COMPLETO (2026-10-05 — review P2-01 approve; web `a482da4` + commit docs de cierre)
- **Incógnitas (uphill):** 0 — resueltas en DISCOVERY
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/app/playground/page.tsx:14` y `src/components/vanta/docs-view.tsx:450` renderizan `<CodePlayground />` (2 callers; `EXAMPLES` es const interna, no exportada — verificado rg) |
| Callees | `code-playground.tsx` → `playground-executor.tsx` (iframe), `public/playground-executor.html` (harness `new Function` + VantaDB WASM), `public/vanta-wasm/*` (bundle 1.2MB), `reveal.tsx` (stacking contexts), `toast.tsx`, `code-tokenizer.ts` |
| Implicaciones | Sin cambio de API pública; fix de stacking local (1 clase en `code-playground.tsx`); afecta playground en `/playground` y `/docs` (mismo componente); reversible (`git revert`); sin impacto perf/serialización |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `src/components/vanta/code-playground.tsx` (471L) · `src/components/vanta/playground-executor.tsx` (118L) · `public/playground-executor.html` (107L) · `public/vanta-wasm/vantadb_wasm.d.ts` (279L) · `src/components/vanta/reveal.tsx` (76L) · `src/app/playground/page.tsx` (17L) · `playwright.config.ts` (36L) · `e2e/flujo-critico.spec.ts` (32L) · `AGENTS.md` (102L) · bloque Task 35 del master plan · task file previo (2026-09-19).
- **Archivos referenciados hacia dentro:** `code-playground.tsx` → `playground-executor` (handle `execute`/`isReady`), `reveal`, `toast`, `code-tokenizer`, `utils`, `language-provider`, `lucide-react`; `playground-executor.tsx` → iframe `/playground-executor.html`; executor.html → `/vanta-wasm/vantadb_wasm.js` + `_bg.wasm`.
- **Archivos que referencian a los editados (referencias entrantes):** `app/playground/page.tsx` (import) y `docs-view.tsx:29,450` (import + render); ninguno depende de nombres internos (`EXAMPLES` no exportado).
- **Veredicto impacto:** BAJO — fix = 1 clase en 1 archivo (stacking) + 1 spec E2E nuevo (aditivo); nada se rompe si se revierte; sin símbolos públicos nuevos.

## Contrato

> "las 5-6 recetas (RAG, híbrido, grafo, TTL, batch, persistencia) funcionan en el playground con WASM real (verificación de ejecución, no solo existencia de código); sin código muerto; **o** cierre como ya-resuelto con evidencia de la verificación E2E + delta documentado si emerge." — master plan Task 35

**Resultado de la verificación:** las 6 recetas EXISTEN y EJECUTAN contra el WASM real (sin API drift), pero la verificación E2E detectó un **defecto de clicabilidad**: el dropdown del playground quedaba cubierto por el panel de output (stacking context de `Reveal`), dejando 5/6 recetas no clicables (verificado también en producción `vantadb.vercel.app`). Delta real = fix puntual (1 clase) + guard E2E de las 6 recetas.

## Spec (SDD — decisiones)

| # | Decisión | Evidencia / porqué |
|---|----------|--------------------|
| 1 | Verificar EJECUCIÓN E2E real (browser + WASM), no solo existencia de código | Plan Task 35 lo exige; la ejecución previa (2026-09-19) solo validó estáticamente (`node --check` + tsc) — la clicabilidad nunca se probó |
| 2 | Base de verificación = worktree `..\web-show02` desde `origin/main` (detached), sin tocar `design/v2` (WIP owner) | Playground idéntico entre `origin/main` y `design/v2` (`git diff` vacío en paths del playground); worktree aísla del WIP |
| 3 | Fix = elevar el stacking context del header Reveal (`relative z-10`) | Root cause: `Reveal` (`will-change-transform` + `translate-y-0`, `reveal.tsx:64`) crea stacking contexts persistentes; el header (DOM-early, z-auto) pinta debajo del grid (DOM-late) → el menú `z-50` queda confinado al contexto del header. `relative z-10` en el Reveal del header lo pinta sobre el grid. Precedente del patrón: navbar (`sticky z-50`) |
| 4 | Commit web: branch local `show02/recipes-clickable` con fix + spec E2E `e2e/playground-recipes.spec.ts` (regresión del defecto — TDD Prove-It) | Instrucción del orquestador; repo web = flujo propio (owner mergea); spec e2e local es precedente commiteado (`web09-screenshots.spec.ts`); E2E no corre en CI web aún → sin carga CI |
| 5 | No tocar `design/v2`, `ts10/*`, `ts13/*`; NUNCA push | Invariantes del orquestador |

## Invariantes de dominio (handoff — MUST)

- ⛔ NUNCA `git push` — ni repo principal ni web.
- ⛔ No tocar: `opencode.jsonc`, master plan 0.9.0, `docs/pipeline-state.json`, `docs/dev/Backlog.md`, `design/v2` (WIP owner), branches `ts10/*`/`ts13/*`, áreas PROV-13/DESKTOP-44.
- Repo principal: `git add` por pathspec explícito (WIP ajeno presente: master plan M, opencode.jsonc M).
- El trabajo del web vive en su repo (branch local); su merge es decisión del owner.
- Scope = recetas; no refactor del playground.

## Deuda técnica (Regla 6)

Sin deuda nueva: fix = 1 clase (reemplaza un bug, no agrega complejidad); spec E2E = cobertura nueva (+). Saldo neto ≤ 0. Observación sistémica (patrón `Reveal` + overlay fuera del playground) → `queda_pendiente` para routing FIND del orquestador (no se toca `Backlog.md` acá).

## Definition of Done

- **Task:** contrato verificado — 6 recetas ejecutan E2E (dropdown → run → output correcto, sin ✗) con evidencia por receta; defecto detectado y fixeado (clicabilidad 6/6).
- **Commit:** repo principal `docs: SHOW-02 — …` (task file + RESULTADO); web `fix(web): SHOW-02 — …` (branch local `show02/recipes-clickable`, NUNCA push).
- **Release:** n/a.

## Herramientas necesarias

- git (worktree/commit local web; status/diff/pathspec principal) · node/npm (tsc/lint/build/playwright en web) · playwright-cli (diagnóstico) · gates docs (`scripts/docs/*.mjs`) · `campaign_*` (state) · `pwsh dev-tools/ocr-review.ps1` · `vanta-review` (P2-01).

**Skills cargadas (SDP v3, phase=BUILD + mandato):** campaign-executor · progreso · security-and-hardening (pinned) · frontend-ui-engineering · test-driven-development · source-driven-development · documentation-skill · playwright-cli · incremental-implementation · context-engineering.

## Investigation Notes

- **(a) Estado del repo web (2026-10-05):** checkout `..\web` en `design/v2` con WIP del owner; worktrees `..\web-ts10` (branch `ts10/npm-install-card`) y `..\web-ts13` (`ts13/orama-column`) pendientes de merge. `origin/main` = `fd7b41b`. Playground idéntico entre `origin/main` y `design/v2` (`git diff` vacío en los 4 paths del playground + bundle).
- **(b) Las 6 recetas existen** en `EXAMPLES` (`code-playground.tsx:46-163`): RAG Mini :48 · Hybrid Search :65 · Graph BFS :84 · TTL Expiry :106 · Batch Insert :123 · Persistence :146. Dropdown `loadExample` :258. Ejecución real vía iframe sandbox (`playground-executor.html:53-82`: `new mod.VantaDB({storage_path:"playground_data"})` + `new Function("VantaDB","db","console", …)`).
- **(c) API del bundle servido (d.ts 279L) cubre todas las recetas:** `put` :164 · `put_batch` :168 · `search` :199 · `get` :96 · `add_edge` :15 · `graph_bfs` :104 · `purge_expired` :160 · `save` :191 · `load` :140 · `list_namespaces` :136 · `flush` :88 · `close` :44 — sin drift de API.
- **(d) DEFECTO DETECTADO (clicabilidad):** el menú del dropdown (`.absolute z-50`) queda **cubierto por el panel de output** — hit-test (`document.elementFromPoint`): "Persistence" → `scroll-manga h-80` (output panel); "RAG Mini" → el item mismo. 5/6 items no clicables (verificado local `localhost:3000` **y** producción `vantadb.vercel.app`). Root cause: stacking contexts de `Reveal` (`reveal.tsx:64` `will-change-transform` + `translate-y-0`); el header Reveal (sin z-index) pinta debajo del grid Reveal (posterior en DOM). Fix: `relative z-10` en el Reveal del header.
- **(e) Ejecución previa (2026-09-19, repo viejo — histórico):** las 6 recetas se crearon y validaron estáticamente (`node --check` 6/6, tsc, eslint); Playwright fue N/A-justificado → el defecto de clicabilidad no se detectó entonces. Esta iteración re-verifica en el repo nuevo con ejecución real.
- **(f) Worktree `..\web-show02`** (detached `origin/main`, `npm ci` 646 paquetes/36s); spec temporal `e2e/show02-recipes.spec.ts` + evidencia `show02-evidence.json` + screenshots `show02-shots/`.

## Steps atómicos

- [x] **Step 1 — DISCOVERY**: repo web + código + bundle + API + defecto reproducido (local + prod) — ✅ COMPLETED (2026-10-05)
- [x] **Step 2 — Fix puntual**: `relative z-10` en Reveal del header (`code-playground.tsx`) + branch `show02/recipes-clickable` — ✅ COMPLETED (hit-test post-fix: menú sobre el panel; screenshot after)
- [x] **Step 3 — Verificación E2E 6/6**: spec Playwright (dropdown → run → output) RED→GREEN + gates web (tsc/lint/build/playwright full 8/8) + commit web `a482da4` (LOCAL) — ✅ COMPLETED
- [x] **Step 4 — Cierre**: task file + OCR + DoD + review P2-01 + commit docs local + campaign taskId 35 + RESULTADO §7 — ✅ COMPLETED (review P2-01 APPROVE `ses_ef5869fbbffefxEijm7HLgO9zM`; commit docs pathspec explícito)

## Evidencia VERIFY

**E2E real (Playwright/Chromium, iframe WASM, worktree `..\web-show02` desde `origin/main`):**

| # | Receta | Output clave (evidencia por receta) | Líneas ✗ |
|---|--------|--------------------------------------|----------|
| 1 | RAG Mini | `retrieved 2 chunks:` · `[doc-auth] score=0.0164` · `answer (cited): … see doc-auth, doc-ttl` | 0 |
| 2 | Hybrid Search | `doc-2 score=0.0328` · `doc-1 score=0.0161` · `doc-0 score=0.0159` | 0 |
| 3 | Graph BFS | `bfs from alice (depth 2): 3 nodes` · `alice` · `bob` · `carol` | 0 |
| 4 | TTL Expiry | `purged 1 expired record(s)` · `session -> null` · `pinned  -> pinned - never expires` | 0 |
| 5 | Batch Insert | `inserted 100 records` · `found 10 results` | 0 |
| 6 | Persistence | `saved to OPFS` · `after load -> notita - survives save/load roundtrip` · `namespaces: agent/main` | 0 |

Todas incluyen `✓ VantaDB WASM engine loaded` + `◆ executed in Xms · wasm32 (sandboxed iframe)` (engine real, sin mocks). Artefactos: `..\web-show02\show02-evidence.json` (6/6 `ok:true`) · `show02-shots/` (screenshot por receta) · `show02-menu-open.png` (ANTES: solo "RAG MINI" visible, resto cubierto) · `show02-menu-open-after.png` (DESPUÉS: 6 items visibles).

**RED→GREEN (TDD Prove-It):** corrida 1 → 6/6 fallan (menú cubierto; hit-test "Persistence" → panel de output). Fix stacking → corrida 2 → 4/6 (Graph BFS: `✗ Node not found` — receta rota, API real exige `insert_node`). Fix receta → corrida 3 → **6/6 pass** (15.0s).

**Gates web (branch `show02/recipes-clickable`):** `npx tsc --noEmit` ✅ · `npm run lint` ✅ · `npm run build` ✅ (exit 0) · `npx playwright test` ✅ **8/8** (flujo-critico + web09 + playground-recipes 6/6). Commit web: `a482da4` (2 files, +116/−14) — LOCAL, nunca push.

**Docs gates (repo principal):** `check-links.mjs` ✅ · `check-docs.mjs` ✅ · `gen-index.mjs --check` ✅ · OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`): 11 archivos en workspace; reviewables (2) = `.github/scripts/verify_pyi.py` + `.github/workflows/providers-ci.yml` (WIP de PROV-13, fuera de scope); el diff de esta tarea (`docs/dev/tasks/SHOW-02.md`) queda excluido por `unsupported_ext` → N/A-justificado.

**Review P2-01:** ✅ APPROVE — ver sección Review.

## Review (GATE — P2-01) — ✅ APPROVE

- **Tier:** Fast (paths `docs/dev/**` + contenido web) → verify mecánico + spot-check; reviewer `vanta-review` fresh context (sesión `ses_ef5869fbbffefxEijm7HLgO9zM`).
- **Enfoque:** contrato Task 35 (ejecución E2E) · scope del commit web `a482da4` · causa-raíz del fix de stacking · API real de la receta Graph · evidencia 6/6 · invariantes (no push).
- **Cómo se probó:** `git show --stat a482da4` (solo 2 files) · **re-ejecución independiente del guard** `npx playwright test e2e/playground-recipes.spec.ts` → **6 passed (15.3s)** · `npx tsc --noEmit` exit 0 · gates docs re-verificados · OCR preview (exclusión `unsupported_ext` verificada) · `git log origin/main..show02/recipes-clickable` (sin push).
- **Veredicto:** ✅ APPROVE — 0 Critical/Required; Optional: commit docs con pathspec explícito para no arrastrar el master plan M (aplicado); Nit: selector por clase en el spec (pragmático, aceptado).

## RESULTADO §7

- **Estado:** ✅ COMPLETO
- **Steps:** 4/4
- **Commits:** web `a482da4` en branch `show02/recipes-clickable` (LOCAL — nunca push; merge = decisión del owner) + commit docs de cierre en repo principal (task file).
- **Verificación:** E2E 6/6 recetas (dropdown → run → output, WASM real; `show02-evidence.json` + screenshots before/after) · web: tsc ✅ lint ✅ build ✅ playwright 8/8 ✅ · docs: check-links ✅ check-docs ✅ gen-index ✅ · OCR N/A-justificado · review P2-01 ✅ APPROVE.
- **Contrato:** cumplido con delta — las 6 recetas EJECUTAN (verificación de ejecución real, no existencia); delta emergido y cerrado: defecto de clicabilidad (stacking) + receta Graph BFS rota contra el API real → fix puntual + guard E2E.
- **Handoff/deuda:** observación sistémica (patrón `Reveal` + overlay fuera del playground) → routing FIND del orquestador; `progreso`/avance + fila Backlog SHOW-02 → orquestador (Backlog.md bajo edición concurrente); merge del branch web = owner.

## Context Save Point

- **Última acción:** cierre — review P2-01 APPROVE + RESULTADO §7; commit docs local (task file).
- **Próximo paso:** ninguno (tarea cerrada). Orquestador: routing FIND de la observación sistémica (Reveal+overlay); `progreso`/avance + fila Backlog SHOW-02 (Backlog.md bajo edición concurrente); merge del branch web `show02/recipes-clickable` = owner.
- **Estado del repo:** principal `develop` con WIP ajeno (master plan M, opencode.jsonc M — no tocar); web en `design/v2` intacto + worktree `..\web-show02` (branch `show02/recipes-clickable`, commit `a482da4`; evidencia en `show02-*.json/png`, `show02-shots/`).
- **Evidencia clave:** `show02-evidence.json` (6/6 ok) · `show02-menu-open.png` / `show02-menu-open-after.png` (before/after del fix).
