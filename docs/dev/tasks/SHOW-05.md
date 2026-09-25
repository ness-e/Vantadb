# SHOW-05 (resto) — decisión `vantadb-ts/examples` + referencia en README/QUICKSTART

> **Plan:** `docs/dev/plans/2026-09-18-cierre-mvp.md` (Task 9, Wave2 última — ÚLTIMA del plan)
> **Campaign:** c578fd8c-1bee-45a8-b036-2f7b00262828 · **Branch:** develop · **Appetite:** max 1h
> **Estado:** 🟢 DISCOVERY completo → EJECUCIÓN (1 step tiny, docs-only)
> **Commit:** `docs: SHOW-05 — ...` (selectivo: README.md + docs/user/QUICKSTART.md + este file). NO PUSH.

## 1. TAREA — objetivo + contrato + AC

**Objetivo:** cerrar el resto pendiente de SHOW-05: decidir mover-vs-referenciar
`vantadb-ts/examples/` y dejar la decisión implementada + escrita.

**Resto pendiente verificado (del plan, confirmado en DISCOVERY):**
QUICKSTART §0 ✅ + requirements `vantadb-py>=0.5.0` ✅ + README §192 ✅ existen
(FIND-105, commit 5e428aea). Solo faltaba la decisión TS + su referencia.

**Contrato:** decisión escrita (mover o referenciar + porqué) + implementada
(línea o mudanza) + coverage 0 gaps.

**AC:**
1. Decisión mover-vs-referenciar ESCRITA con motivo (esta task file §5 + commit msg).
2. Decisión IMPLEMENTADA (línea(s) de referencia o mudanza con grep previo).
3. `scripts/validate-docs-coverage.ps1` → 0 gaps + `git diff --check` limpio.

## 2. ARCHIVOS

**Clave (con :línea):**
- `vantadb-ts/examples/` — existe; inventario verificado: 3 `.mjs`
  (`langchain-rag.mjs`, `llamaindex-rag.mjs`, `vercel-ai-memory.mjs`) + 3 dirs
  (`langchain/`, `llamaindex/`, `vercel-ai/`). Solo lectura (inventario).
- `README.md:65` — fila Quick Links `Run runnable examples`; se extiende con
  `· [TypeScript](vantadb-ts/examples/)` (1 línea, append-only).
- `docs/user/QUICKSTART.md:51-53` — blockquote `Note` del §0; se añade 1 línea TS
  con link relativo `../vantadb-ts/examples/` (append-only).

**Relacionados (lectura):**
- `examples/README.md:4,34-40` — YA referencia `../vantadb-ts/examples/`
  (base de la decisión; no se toca).
- `SKILLS-MANIFEST.md` — verificado: cero referencias a `vantadb-ts` (rg limpio);
  es catálogo de skills, fuera de scope añadir TS-examples ahí (ponytail: no tocar).
- `SPEC.md:97` — ya fija `referenciar ... (decide SHOW-05-resto)`; no se toca.
- `scripts/install.*` — lectura N/A (no aplica; no tocar por contrato).
- `.github/workflows/ci-examples-12.yml` — leído vía grep: NO referencia
  `vantadb-ts/examples` (solo `examples/**` + runs python). Precisión vs FIND-74
  (§5 hallazgo).

**Prohibidos (intocables):** `README_ES.md` (re-traducción ES prohibida) · resto de
`web/` · `vantadb-ts/src/` (código TS) · mudanza sin grep previo · `reparacion.bat` ·
`.opencode` · `Justfile` · `ocr-*` · `completions/*` · `desktop/src-tauri/Cargo.lock` ·
stash@{0} GOV-C4 · `docs/dev/Backlog.md` (lo cierra el orquestador con progreso) ·
plan file (solo lee; recitation la escribe el orquestador/lead) · `C:/Users/Eros/.vantadb*`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `examples/README.md` (50L entero),
  `README.md:40-99` + `:180-209`, `docs/user/QUICKSTART.md:1-60`,
  `.opencode/rules/js-ecosystem.md` (34L entero),
  `.opencode/references/definition-of-done.md` (144L entero),
  skill `documentation-and-adrs`, plan file Task 9 + recitations,
  `docs/dev/tasks/FIND-74.md` (vía grep, evidencia histórica).
- **Referencias hacia dentro (lo que mis editados citan):** ninguna nueva lógica;
  los 2 links añadidos apuntan a `vantadb-ts/examples/` (existe, 6 entries).
- **Referencias entrantes (quién cita mis editados):** `validate-docs-coverage.ps1`
  (chequea links/gaps de docs); `examples/README.md:5` cita QUICKSTART (no roto,
  append-only); ningún código cita README/QUICKSTART (docs-only).
- **Veredicto:** impacto 🟢 nulo en código/CI. Blast radius = 2 archivos docs,
  2 líneas añadidas, 0 líneas movidas/borradas. Rollback = `git revert` trivial.

## 3. DEPENDENCIAS / WAVE

Wave2 última en secuencia (IMPL-112-S2 ✅, FIND-117 ✅ — recitations en plan file).
Sin bloqueantes. Stop del plan: si la decisión exigiera mudanza grande → solo
decisión + nota (tiny). No aplica: se decide referenciar. NextTask: ninguna —
ÚLTIMA del plan (cierre del orquestador: P2-01 batch + progreso + archive).

## 4. REFERENCIAS / RULES / COMMANDS

- **Rules:** ninguna de código (docs-only). `js-ecosystem.md` en lectura (área
  `vantadb-ts/`): R-2 (no referenciar artefactos `pkg/`/`dist/` — cumplido, solo se
  enlaza `examples/`) · R-1/R-3/R-4 N/A (sin WASM/persistencia/build/bindings).
- **Refs:** `definition-of-done.md` (DoD standing + v1 baseline adaptado a docs:
  Correctness=contrato, Quality=diff mínimo, Docs=esta decisión registrada,
  Ship=commit `docs:` revertible, sin deuda).
- **Commands:** `pipeline.md` (ejecución). **SPEC raíz:** `SPEC.md:97` (pre-fija
  referenciar salvo motivo escrito). Tabla Spec: N/A (docs/decisión).

## 5. DECISIÓN — REFERENCIAR (no mover) + motivo ponytail

**Decisión: REFERENCIAR `vantadb-ts/examples/` desde README + QUICKSTART. No mover.**

**Motivo escrito (5 puntos):**
1. **Precedente vigente:** FIND-74 (2026-09-15, commit `5e428aea`) ya decidió
   `referenciar (no mover)` con evidencia (conteo 0 `.ts` en `examples/` +
   `examples/README.md:4,34-40` ya referencia). Sin evidencia nueva, re-debatir
   es waste — la lección quedó en `lessons.md:653`.
2. **Costo:** referenciar = 2 líneas añadidas, 0 rotos. Mover = mudanza de
   6 entries + rewrites obligados en `examples/README.md:4,34`, `SPEC.md:97`,
   `docs/dev/tasks/FIND-74.md` (evidencia histórica), snapshots de avance y book
   HTML que citan el path actual.
3. **CI real:** `release-npm-61.yml:21,27` filtra por `vantadb-ts/**` — sacar los
   ejemplos de ese árbol cambia triggers de release npm (scope-creep a workflow).
   (Precisión vs FIND-74: `ci-examples-12.yml` NO cita el path TS — verificado por
   grep; el riesgo CI vive en release-npm, no en examples-12. Hallazgo menor,
   no bloquea.)
4. **Descubrimiento ya existe:** `examples/README.md` indexa el árbol TS
   (`:4` blockquote + `:34-40` tabla). Solo faltan los 2 puntos de entrada
   top-level (README Quick Links, QUICKSTART §0) — eso es este resto.
5. **Ponytail ladder:** 2 líneas < mudanza con grep de links. La opción más
   chica que cierra el contrato gana.

## 6. SKILLS (SDP Paso 0b — real, `campaign_discover_skills_v2` phase=BUILD)

**SDP devuelto (8, type=TypeScript SDK):** campaign-executor · source-driven-development ·
incremental-implementation · test-driven-development · context-engineering ·
doubt-driven-development · frontend-ui-engineering · api-and-interface-design
(+ keyword-mapped no rankeadas en top-8: documentation-and-adrs, writing-guidelines,
constraint-driven-development; sugerida del plan: documentation-and-adrs).

**Cargadas (selectivo, justificado — anti-flooding §3a):**
- `documentation-and-adrs` — cuándo aplica: la entrega ES una decisión escrita
  (mover-vs-referenciar + motivo) + referencia en docs. Núcleo de esta tarea.
- Base auto vía MCP (no carga manual): campaign-executor, progreso, ponytail(full).

**No cargadas (motivo 1 línea c/u):**
- incremental-implementation: 1 slice tiny de 2 líneas, sin iteración multi-slice.
- test-driven-development: docs-only, cero lógica — el "test" es coverage 0 gaps.
- context-engineering: contexto ya empaquetado (plan verificado como dado).
- doubt-driven-development: sin stakes de producción/seguridad (2 líneas docs).
- frontend-ui-engineering / api-and-interface-design: sin UI ni API pública.
- source-driven-development: sin frameworks/APIs externas (links relativos internos).
- writing-guidelines / constraint-driven-development: estilo ya fijado por
  FIND-74/examples/README (consistencia por copia, no por guía nueva).

## 7. HERRAMIENTAS + MCP + VERIFY

- Grep referencias: `rg -n "vantadb-ts" README.md docs/user/QUICKSTART.md SKILLS-MANIFEST.md`
  (pre: vacío ✅) + `rg -n "vantadb-ts/examples" .github/ docs/ README.md` (pre: sin
  `.github`, con docs históricos ✅) — post: deben aparecer README.md + QUICKSTART.md.
- `git diff --check` (whitespace).
- `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps (contrato).
- `campaign_verify_cmd`: bug exit -1 conocido → bash directa + mención en RESULTADO.
- codegraph N/A (docs-only, sin símbolos). Cargo N/A. Internet N/A.
- OCR delegation: N/A-justificado — docs-only 2 líneas, sin trust boundary, sin
  dependencias, sin código (criterio punto 10 del encargo).
- SECURITY/PERFORMANCE phases: N/A (sin input usuario/auth/storage/FFI/red/hot paths).

## 8. STEPS ATÓMICOS

| # | Step | Estado |
|---|------|--------|
| 1 | DISCOVERY: inventario `vantadb-ts/examples/` + greps pre + lectura puntos de inserción + Gate D | ✅ hecho |
| 2 | EJECUCIÓN: 2 ediciones (README.md:65 + QUICKSTART §0 note) + greps post | ✅ hecho (post: README.md:65 + QUICKSTART.md:53; MANIFEST cero por diseño) |
| 3 | CIERRE: `git diff --check` + coverage 0 gaps + commit `docs:` selectivo + RESULTADO §7 | ✅ hecho (diff-check 0 · coverage 0 gaps · commit selectivo 3 paths) |

**Gate D (evaluado tras planning, antes de editar):** NO dispara — docs-only,
2 líneas append-only, sin símbolos públicos nuevos, sin hot path, blast radius
2 archivos lectura-mapeada. Motivo ≤6 palabras: *docs-only dos líneas sin API*.

## Context Save Point

- Todo el contexto está en este file + plan file Task 9. Reanudar = leer este
  file desde Step 2.
- WIP ajeno en `git status` (submodule `.opencode`/`skills`, `completions/*`,
  `reparacion.bat`, `docs/dev/Backlog.md`, plan file untracked): INTOCABLE —
  staging selectivo solo de los 3 paths propios.
- Cierre del plan (P2-01 batch + progreso + archive + recitation plan file):
  lo hace el ORQUESTADOR, no esta tarea.
