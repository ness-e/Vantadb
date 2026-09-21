# DOC-SYNC-01 — Sincronizar descripciones stales del Backlog

- **Plan:** `docs/plans/2026-09-07-cleanup-gates.md` Task 3
- **Estado:** ⏳ IN PROGRESS → ✅ al verificar
- **Branch:** develop (no commitear — cierre sin commit por orden del run)
- **Scope:** SOLO `docs/Backlog.md` (2 textos). NO tocar plan file, ni archivos de STABLE-05/GOV-TK2.

## Contrato (ley)

- Header con conteo real de filas activas (contado por comando, con fecha de medición)
- STABLE-06 dice 280 tests
- `git diff --stat` muestra SOLO `docs/Backlog.md` (en lo tocado por esta tarea)

## Discovery

- **Tipo detectado:** `docs` (campaign_detect_task_type: Documentation; skills writing-guidelines, writing-plans; check validate-docs-coverage.ps1)
- **SDP:** `campaign_discover_skills_v2` phase=BUILD keywords=[backlog, conteo filas activas, STABLE-06, tests, docs sync] → 8 candidatas (mayoría lifecycle genéricas no aplicables a doc-only). Cargadas útiles: `documentation-and-adrs`, `writing-guidelines` + base `campaign-executor`, `progreso` + `ponytail(full)` siempre. Resto (incremental, TDD, context-engineering, source-driven, doubt-driven, frontend-ui, api-design) descartadas por no-aplicar a 2 ediciones de texto.
- **Gate D (question-gates):** NO dispara — blast radius 1 archivo docs-only, sin hot path, sin API pública, sin símbolos nuevos, contrato inequívoco (2 textos). Sin `question` al usuario.
- **Gate spec-first:** N/A — docs-sync, no feature-add/lógica nueva, sin sección `## Spec` requerida.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `docs/Backlog.md:1-80` (header) + `docs/Backlog.md:600-629` (fila STABLE-06) + `docs/tasks/STABLE-06.md` (medición 278/278) + `docs/avance/activo/ci-cd.md:292` (278/278) + `docs/plans/2026-09-07-cleanup-gates.md` Task 3.
- **Referencias hacia dentro:** Backlog header referenciado por triages/planes; fila STABLE-06 referenciada por plan 2026-09-04 (Task 6) y task STABLE-06.md.
- **Referencias entrantes:** plan 2026-09-07 Task 3 apunta a estos 2 textos; ningún código importa estos textos.
- **Veredicto:** impacto nulo en código — 2 ediciones de texto en 1 archivo md. Reversible (revert de 2 líneas).

## Mediciones (comandos + fecha 2026-09-08)

- **Filas activas:** `Select-String -Path docs/Backlog.md -Pattern '^\| `[^`]+` \|' | Measure` = **67** (no-struck). Cross-check: pattern `Pendiente` = 67; total `^\| (`|~~`)` = 68 (1 struck `TBH-01` ✅). Header decía 99 → stale.
- **Suite TS:** `Select-String -Path vantadb-ts/src/__tests__/*.test.ts -Pattern '^\s*(test|it)\s*\('` = **275** (dx04 37, flat 7, hardening 72, integration 18, load 6, native-error 5, subclients 17, types 6, vanta 107) + `vantadb-ts/tests/graph.test.ts` = **5** → **280 total**. Vitest medido 278/278 el 2026-09-05 (commit 7ff70b01); +2 = vanta.test.ts 105→107 (MOD-24). Fila decía 264 → stale. Se escribe 280 con fecha.
- **Comando header a pegar:** `Select-String -Path docs/Backlog.md -Pattern '^\| `[^`]+` \|'`

## Steps

### Step 1: Editar header (99→67 con fecha+comando) ✅ COMPLETED 2026-09-08
- ACT: edit `docs/Backlog.md` old `**Total open items:** 99 activas` → new con 67 + fecha/comando.
- VERIFY: grep `Total open items` → 67 + fecha ✅.

### Step 2: Editar STABLE-06 (264→280 con fecha) ✅ COMPLETED 2026-09-08
- ACT: edit fila STABLE-06 old 264 → 280 + medición.
- VERIFY: grep STABLE-06 → 280 ✅; `git diff --stat -- docs/Backlog.md` = 1 file (mis 2 líneas) ✅. Global incluye dirty ajeno pre-existente (ORG-01..11, M .opencode, M opencode.jsonc) — no tocado.

## Notas Wave0

- Paralelo con STABLE-05/GOV-TK2, archivos disjuntos — no tocar sus archivos.
- Pre-existing dirty ajeno (verificado 2026-09-08): `M .opencode` (submodule), `M opencode.jsonc`, sección ORG-01..11 en Backlog (18 insertions otra sesión) + plan file untracked. NO tocarlos; el `diff --stat` global los incluye — deuda ajena, se reporta en RESULTADO.
