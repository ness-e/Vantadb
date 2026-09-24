# FIND-78 — link roto + nota engines Node (`vantadb-node/README.md`)

> Campaign: `6ab26f3f-cf16-4416-9255-c18cca0bcaf0` · Plan: `docs/dev/plans/2026-09-15-find-correcciones.md`
> Estado: ⬜ PENDING → IN PROGRESS (Wave2, disjunto de FIND-79 TS tests y FIND-70 bench-nightly)
> Appetite: 2h · Esfuerzo: 🟢 · Prioridad: 🟢 · Ruta: vanta-docs
> Branch: `develop` · Commit previsto: `docs: FIND-78 — ...` (solo `vantadb-node/README.md`)
> nextTask: FIND-70
> Sin símbolos → sin Spec (fix mecánico docs, 0 código).

## SDP

`campaign_discover_skills_v2 archivosClave="vantadb-node/README.md" phase="BUILD" contractKeywords=["docs","readme","links","node"]` →
base: campaign-executor, source-driven-development, progreso +
lifecycle: incremental-implementation, test-driven-development, context-engineering,
doubt-driven-development, frontend-ui-engineering, api-and-interface-design +
keywordMapped: documentation-and-adrs, writing-guidelines.
Cargadas: documentation-and-adrs, writing-guidelines.
SKILLS_CARGADAS base sesión: campaign-executor, brainstorming, writing-plans,
planning-and-task-breakdown, progreso, ponytail(full).
`SDP: documentation-and-adrs + writing-guidelines (keyword-mapped; resto lifecycle descartado por docs-only sin lógica)`

## Gate D (question-gates.md)

Blast radius = 1 README (edición) + 1 package.json (solo lectura) + `docs/dev/reviews/` (solo búsqueda).
Sin símbolos públicos nuevos, sin hot path, sin API pública, contrato no ambiguo
(0 links rotos muestreo total + nota compat explícita + no inventar rutas),
fix docs (no feature-add) → Gate D **no disparado**, sin `question`.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `vantadb-node/README.md` (79 líneas, entero),
  `vantadb-ts/package.json:1-9` (engines `node>=22.19`), `vantadb-node/package.json:46-48`
  (engines `node>=18`); `.opencode/references/definition-of-done.md` (sección docs);
  `docs/dev/reviews/README.md:40-60` (índice, solo lectura); `docs/dev/tasks/TS-12.md:1-15` (existe);
  `docs/api/NODE_SDK.md:1-10` (existe); `docs/dev/reviews/archive/research-vantadb-node-20260825.md:1-15` (existe, contenido correcto).
- **Referencias hacia dentro (lo que el README cita):** `https://napi.rs` (externo),
  `../vantadb-ts` (dir existe), `../docs/api/NODE_SDK.md` → `docs/api/NODE_SDK.md` (True),
  ``docs/dev/tasks/TS-12.md`` (bare, resuelve a raíz `docs/dev/tasks/TS-12.md` True; relativo a `vantadb-node/` False — code span, no link MD),
  ``docs/dev/reviews/research-vantadb-node-20260825.md`` (bare, raíz False → **roto**; reubicado a `docs/dev/reviews/archive/` True).
- **Referencias entrantes al README:** `rg "vantadb-node/README"` → solo `vantadb-node/package.json:files` (empaquetado) y el propio plan FIND-78; ningún código importa el README.
- **Veredicto:** impacto = 1 archivo MD. `vantadb-ts/package.json` **solo lectura, prohibido editar**.
  Prohibidos intactos: código TS, `.opencode/`, completions, desktop lock. Regla 11: cada claim con fuente file:línea.

## Investigación problema

- Review **SÍ existe en otra ruta**: `docs/dev/reviews/archive/research-vantadb-node-20260825.md` (True;
  frontmatter `title: Research: vantadb-node...`, `status: archived`) → pre-mortem confirmado:
  fue archivado, no borrado. Fix = **corregir link a la ruta archive**, no quitar la referencia.
  `docs/dev/reviews/README.md:46` también lo indexa (sin prefijo `archive/` — colateral fuera de scope, no se toca).
- Engines: `vantadb-node/package.json:47` `node>=18` vs `vantadb-ts/package.json:7` `node>=22.19` →
  README `:27` "Node ≥ 18" es cierto para este paquete pero ambiguo junto a `vantadb-ts`.
  Fix = mantener `Node ≥ 18` + nota compat explícita citando ambas fuentes. No inventar rutas.
- Muestreo total links README: `napi.rs` (externo, no verificado red — fuera de contrato mecánico),
  `../vantadb-ts` True, `../docs/api/NODE_SDK.md` True, `docs/dev/tasks/TS-12.md` True (raíz),
  review roto → archive True. Tras fix: 0 rotos locales.

## Steps

- [x] **Step 1 (único, ~6 líneas):** PLAN→ACT→VERIFY — `:16` ruta → `docs/dev/reviews/archive/research-vantadb-node-20260825.md`;
  `:27` + nota compat `node18 (este paquete) vs ts22.19 (vantadb-ts/package.json:7)`;
  `Test-Path` 0 rotos + `rg engines` + `git diff --check` + `campaign_verify_cmd`. ✅
  - `Test-Path docs/dev/reviews/archive/research-vantadb-node-20260825.md` → True ✅
    (vieja ruta `docs/dev/reviews/research-vantadb-node-20260825.md` → False, ya sin referencias en el README ✅)
  - `Test-Path docs/dev/tasks/TS-12.md` → True ✅; `docs/api/NODE_SDK.md` → True ✅; `vantadb-ts` → True ✅
  - `rg engines` → `vantadb-node/package.json:47 >=18` vs `vantadb-ts/package.json:7 >=22.19` + nota `:27` ✅
  - `git diff --check` → limpio (solo warning CRLF pre-existente en `completions/`, WIP ajeno) ✅
  - Self-review (correctness/scoped/Regla 11): diff = 2 líneas README, 0 código, fuentes file:línea ✅ → **Approve**
  - DoD docs: contrato cumplido; sin deuda neta; WIP ajeno intacto (`.opencode/`, completions, desktop lock no tocados).
  - Backlog NO tocado (race Wave2 paralela — precedente FIND-88; migración vía progreso por orquestador).

## Contrato

- 0 links rotos locales en `vantadb-node/README.md` (muestreo total: archive/TS-12/NODE_SDK/vantadb-ts True).
- Nota compat explícita node18 (este paquete) vs ts22.19 (`vantadb-ts/package.json:engines`) en `:27`.
- 0 rutas inventadas (toda ruta citada con `Test-Path True`).
- Diff = solo `vantadb-node/README.md` (+ este task file).
