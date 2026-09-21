# SHOW-02 — recetas clicables del playground (5-6)

> **Plan:** `docs/plans/2026-09-19-publicacion.md` (Wave0, segunda en secuencia) · **Ruta:** vanta-worker
> **Estado:** ✅ COMPLETO · **Appetite:** 2d · **Esfuerzo:** 🟡 · **Branch:** develop · **Commit:** `feat: SHOW-02 — ...`
> **SDP:** campaign-executor, frontend-ui-engineering, design-taste-frontend, incremental-implementation, test-driven-development, context-engineering, source-driven-development, doubt-driven-development + manual: systematic-debugging, documentation-and-adrs

## 1. TAREA

**Objetivo:** showcase que prueba valor público — 5-6 recetas clicables corriendo contra el playground WASM existente (RAG, híbrido, grafo, TTL, batch, persistencia). Prerrequisito del anuncio.

**Contrato exacto (del plan):**
- (a) 5-6 recetas clicables corriendo contra el playground (cada una: setup + run + resultado visible).
- (b) 1 línea `docs/api/MCP.md`: ops dependientes en invocaciones secuenciales (nota smoke Fase 2: batches multi-call se reordenan).
- (c) coverage 0 gaps si toca docs (`scripts/validate-docs-coverage.ps1`).

**Acceptance criteria:**
- [ ] Dropdown del playground lista 6 recetas: RAG, Híbrido, Grafo, TTL, Batch, Persistencia.
- [ ] Cada receta: setup (puts) + run (query/op) + resultado visible vía `console.log` (panel output existente).
- [ ] Solo se usan APIs verificadas en `vantadb-wasm/src/vantadb_wasm.d.ts` (ver §7).
- [ ] 1 línea en MCP.md sobre invocaciones secuenciales.
- [ ] `npx tsc --noEmit` (web) + `node --check` de recetas + `validate-docs-coverage.ps1` verdes.
- [ ] Commit `feat: SHOW-02 — ...` con staging SELECTIVO (solo paths de esta tarea). NO PUSH.

**Reutilización (NO reconstruir):** `CodePlayground` (`web/src/components/vanta/code-playground.tsx:116`) + iframe sandbox `PlaygroundExecutor` (`playground-executor.tsx:28`, `sandbox="allow-scripts allow-same-origin"`) + `public/playground-executor.html` + bundle servido `public/vanta-wasm/vantadb_wasm.{js,wasm}` (1.2MB, presente).

**Stop del plan:** playground base no corre → DEFER con diagnóstico (no rediseñar `web/`). **Veredicto DISCOVERY: base SANA → se ejecuta** (evidencia §8).

## 2. ARCHIVOS

**Clave (con :línea):**
- `web/src/components/vanta/code-playground.tsx:24-114` — `STARTER_CODE` + `EXAMPLES` (4 actuales → 6 recetas)
- `web/src/app/playground/page.tsx:1-17` — ruta `/playground` (lectura, no se toca salvo necesidad)
- `docs/api/MCP.md:196-211` — ancla: blockquote annotations tras tabla Tool Families (1 línea nueva)

**Relacionados (callers/callees codegraph + trace):**
- `web/src/components/vanta/playground-executor.tsx:1-118` — iframe executor (postMessage execute/result, ping/retry ready); callers: `code-playground.tsx:287`; sin tests (conocido)
- `web/src/components/vanta/docs-view.tsx` — segundo caller de `CodePlayground` (verificar que el cambio de EXAMPLES no rompe su uso)
- `web/public/playground-executor.html:53-82` — `executeSnippet`: `new mod.VantaDB({storage_path:"playground_data"})`, `new Function("VantaDB","db","console",...)` async → recetas pueden usar `await` + `db` instancia `Client`
- `vantadb-wasm/src/vantadb_wasm.d.ts` — superficie verificada: `put:558`, `put_batch:563`, `get:566`, `search:583` (+`text_query:227`), `save:515`/`load:527` async, `purge_expired:702`, `add_edge:737`, `graph_bfs:750` (`TraversalDirectionStr="Forward"|"Reverse"|"Both":419`), `MemoryRecord.node_id:148`, `ttl_ms:178` (relativo)
- `src/sdk/types/record.rs:51-53` — `ttl_ms` relativo: `expires_at_ms = now_ms() + ttl_ms` (put lo computa server-side)
- `examples/` — patrones de recetas (lectura referencia); SHOW-05 referenció `vantadb-ts/examples/` (solo lectura)
- E2E guard WEB-08: `web/e2e/flujo-critico.spec.ts` (landing→docs→playground) — no se toca, es evidencia de base sana

**Prohibidos (WIP ajeno / fuera de scope — NO tocar):**
`web/` fuera del playground · `reparacion.bat` · `.opencode/` · `Justfile` · `ocr-*` · `completions/*` · `desktop/src-tauri/Cargo.lock` · stash@{0} GOV-C4 · `docs/Backlog.md` · plan file (solo recitation al cierre) · `C:/Users/Eros/.vantadb*` (datos vivos) · `src/` Rust core (cero cambios) · `vantadb-mcp/src/` (no tocar) · `target/` (builds).

## 3. DEPENDENCIAS

- **Wave:** Wave0 segunda en secuencia (FIND-98-retry ✅ STOP contractual antes; archivos disjuntos: bins fuera-del-repo vs web+docs).
- **Bloqueantes:** ninguno.
- **Previa:** FIND-98-retry (✅ completada, commit `8d7d1bbf`).
- **NextTask tras cierre:** SHOW-03 (la ejecuta el orquestador, no yo).

## 4. REFERENCIAS

- **Rules (leída completa antes de actuar — toca `web/`):** `.opencode/rules/frontend-web.md` — aplican R-FE-4 (light-only, recetas no tocan estilos), R-FE-5 (dropdown/buttons existentes ya ≥44px, no se agregan targets nuevos), R-FE-6 (sin motion nueva). R-FE-1/2/3/7 N/A (sin errores/dep/json-ld/utilities).
- **Refs:** `.opencode/references/definition-of-done.md` (DoD 3 niveles al cierre) · `dev-tools.md` + `test-suite.md` (verify) · `ocr-review.md` (gate VERIFY, docs-only→N/A-justificado) · `SPEC.md` raíz Success Criteria (sin cambios).
- **Commands:** `pipeline.md` (ejecución) · `audit.md` (verify post-tarea).
- **Tabla Spec:** N/A — showcase/docs, sin símbolos públicos nuevos (`EXAMPLES` es const interna del módulo; línea MCP.md es prosa). Sin endpoints/métodos/tool nuevos → sin decisión que documentar.

## 5. SKILLS (SDP Paso 0b — `campaign_discover_skills_v2` phase=BUILD, keywords [playground, recetas, wasm, clickable, demo, e2e])

| Skill | Cuándo aplica (1 línea) |
|-------|------------------------|
| campaign-executor | Base: state machine PLAN→ACT→VERIFY + recitation/handoff de la tarea |
| frontend-ui-engineering | Dropdown/editor/output accesibles; no introducir AI-slop visual en recetas |
| design-taste-frontend | Recetas heredan estética manga/linocut existente sin rediseñar |
| incremental-implementation | Slice 1 (EXAMPLES) + Slice 2 (MCP.md), ~100 líneas/slice, repo compilable |
| test-driven-development | Receta = test e2e por construcción: `node --check` sintaxis + tsc + click→Run existente |
| context-engineering | Context pack del slice: rules → plan → source + d.ts verificado, sin inventar APIs |
| source-driven-development | APIs validadas contra `vantadb_wasm.d.ts` + `record.rs`, no contra memoria del modelo |
| doubt-driven-development | Cambio visible público (showcase): verificación adversarial de cada snippet |
| systematic-debugging (manual) | Si una receta falla verify: root-cause antes de fix (trazar d.ts, no adivinar) |
| documentation-and-adrs (manual) | 1 línea MCP.md precisa + honesta (Regla 11: sin claims sin fuente) |

## 6. HERRAMIENTAS + MCP

- `codegraph_explore` — ya usado en DISCOVERY (blast radius §7); solo re-usar si un edit toca `engine.rs`/`node.rs` (no es el caso).
- `campaign_verify_cmd` — BUG exit -1 conocido → **bash directa + mención en RESULTADO**.
- Verificación mecánica por slice: `cd web && npx tsc --noEmit` · `node --check` sobre recetas extraídas · `pwsh scripts/validate-docs-coverage.ps1` (tras MCP.md).
- Playwright MCP: N/A-justificado — E2E guard WEB-08 ya cubre landing→playground WASM run (3.1s verde documentado); click-through de 6 recetas = scope creep fuera de appetite (pre-mortem: recetas solo-local con nota).
- Cargo: N/A (cero cambios Rust; evitar builds pesados sin motivo). Internet: N/A (todo local).

## 7. INVESTIGACIÓN CÓDIGO (blast radius — generado en DISCOVERY)

- `CodePlayground` (`code-playground.tsx:116`): 2 callers (`app/playground/page.tsx`, `components/vanta/docs-view.tsx`); sin tests. Cambio: solo el const `EXAMPLES` (líneas 46-114) — dropdown (`238-264`), `loadExample` (`212-217`), `reset` (`206-210`) y `run`/output (`157-204`, `354-410`) funcionan por construcción para cualquier snippet (mismo mecanismo que los 4 ejemplos actuales).
- `PlaygroundExecutor` (`playground-executor.tsx:28`): 1 caller; postMessage `execute {code, requestId}` → `result {output, error}`; timeout 30s; ping/retry ready. Sin cambios.
- `playground-executor.html:53-82`: cada Run = `new mod.VantaDB({storage_path:"playground_data"})` fresco + `new Function` async con `(VantaDB, db, console)` → recetas usan `db` (instancia con todos los métodos `Client`) y `await` libremente. `db.close()` automático post-run.
- Recetas existentes vs faltantes: ✅ Híbrido (mejorar con `text_query`) · ✅ Batch · ❌ RAG · ❌ Grafo · ❌ TTL · ❌ Persistencia → Slice 1 crea las 6 finales (RAG, Híbrido, Grafo, TTL, Batch, Persistencia).
- `docs-view.tsx` usa `CodePlayground` — verificar tras el edit que no referencia `EXAMPLES` por nombre (grep en VERIFY).
- `docs/api/MCP.md` (523 líneas): ancla § Tool Families, tras blockquote annotations (~línea 211). `validate-docs-coverage.ps1` valida cobertura de tools — 1 línea prosa no la afecta (verificar en VERIFY).

## 8. INVESTIGACIÓN PROBLEMA — ¿el playground base corre hoy?

**Veredicto: SÍ (evidencia estática + guard existente, sin opinión).**
- `web/src/components/vanta/code-playground.tsx` (425 líneas) + `playground-executor.tsx` (118) + `web/src/app/playground/page.tsx` existen y codegraph los resuelve con callers sanos.
- `web/public/playground-executor.html` (107 líneas) + bundle servido `web/public/vanta-wasm/vantadb_wasm.js` (56KB) + `vantadb_wasm_bg.wasm` (1.2MB) presentes → el iframe puede `fetch /vanta-wasm/*` mismo-origen.
- E2E guard WEB-08 (`web/e2e/flujo-critico.spec.ts`): landing→`/docs#quickstart`→`/playground` con WASM run, documentado verde local (~3.1s) en `web/AGENTS.md`.
- Deuda honesta: no se re-ejecutó el browser E2E en esta sesión (appetite/budget; pre-mortem lo permite: recetas solo-local). `npx tsc --noEmit` en VERIFY confirma compilación. → **NO DEFER.**

## 9. INVESTIGACIÓN INTERNET

N/A — todo local (playground + WASM + docs). Sin red usada, nada que marcar (TSYS-13 sin citas).

## 10. VALIDACIÓN + CIERRE

- Verify contrato: tsc + node --check + coverage ps1 (bash directa por bug exit -1, con mención).
- OCR delegation (`pwsh dev-tools/ocr-review.ps1`): cambio docs+showcase de 2 archivos, sin trust boundaries nuevos (iframe sandbox intacto, sin `allow-*` nuevos, sin input de red) → si el diff es solo EXAMPLES+prosa, N/A-justificado con evidencia del diff; si el CLI marca Critical/High → bloquea.
- DoD 3 niveles (definition-of-done.md) + P2-01 lo hace el orquestador (no yo) + Gates D/V/C vía `question` (D: evaluado no-dispara §7; V/C al cierre si aplica).
- RESULTADO §7 obligatorio al final. Commit `feat: SHOW-02 — ...` selectivo, NO PUSH.

## Steps atómicos

- [x] **Step 1 — EXAMPLES 6 recetas** (`code-playground.tsx:46-114`): RAG (put 3 docs + search vector+text_query + síntesis con citas) · Híbrido (puts + search `text_query`, log scores) · Grafo (3 puts + `add_edge` + `graph_bfs(roots,2,"Forward")`) · TTL (`ttl_ms:1` + `purge_expired` + get→null) · Batch (existente, conservar) · Persistencia (`put` + `await db.save()` + nota OPFS). Verify: `node --check` + `tsc --noEmit` + grep `docs-view.tsx` no rompe.
- [x] **Step 2 — 1 línea MCP.md** (tras blockquote annotations § Tool Families): ops dependientes en invocaciones secuenciales, no en un batch multi-call (nota smoke Fase 2: batches multi-call se reordenan). Verify: `validate-docs-coverage.ps1` 0 gaps.
- [x] **Step 3 — Cierre**: verify full → OCR justificado → commit selectivo `feat:` → recitation completed + RESULTADO §7.

## Evidencia VERIFY (2026-09-19)

- `node --check` 6/6 recetas (extracción EXAMPLES→temp, `recipe_*.js`): ok=6 fail=0.
- `cd web && npx tsc --noEmit`: EXIT 0.
- `cd web && npx eslint src/components/vanta/code-playground.tsx`: EXIT 0.
- `pwsh scripts/validate-docs-coverage.ps1`: EXIT 0 — "0 gaps" (MCP.md 49 tools ok).
- OCR: preview/rules OK (advisory sin API key); `code-playground.tsx` cae en rule group `web/**`; self-review sin Critical/High (solo strings estáticos, sin lógica/hooks/eval/secrets; sandbox iframe intacto) → N/A-justificado.
- `docs-view.tsx` solo renderiza `<CodePlayground />` (líneas 29,450) — sin dependencia de nombres EXAMPLES.
- `campaign_verify_cmd` no usado directo (bug exit -1 conocido) → bash directa + mención (esta sección).

## Context Save Point

DISCOVERY completo 2026-09-19. Si se reanuda: Steps 1-2 ⬜ pendientes (ver § Steps). Archivos: `web/src/components/vanta/code-playground.tsx`, `docs/api/MCP.md`. No hay trabajo parcial en worktree (solo este task file + recitation in-progress). NextTask: SHOW-03 (orquestador).
