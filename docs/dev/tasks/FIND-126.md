# FIND-126 — limpieza docs (frontmatter + 70 lints)

> **Plan:** `docs/dev/plans/2026-09-19-ci-green.md` Task 4 (Wave1) · **Campaign:** 0ad2d7e2-94e3-4313-8f5c-e8d57c08a6af
> **Ruta:** vanta-worker · **Estado:** ⏳ IN PROGRESS · **Rama:** `develop`
> **SDP:** documentation-and-adrs, ci-cd-and-automation, incremental-implementation, context-engineering (+ base campaign-executor/progreso/ponytail)

## Objetivo

Checks del PR en rojo por higiene docs: o se limpia o se acota el scope del workflow (decidir en DISCOVERY con conteo, no con opinión).

## Contrato

Lint Markdown verde + Frontmatter verde (o scope del workflow acotado a archivos del PR con decisión escrita del owner) + `markdownlint-cli2 "docs/**/*.md"` 0 issues en el scope decidido.

## Acceptance criteria

- [ ] `npx markdownlint-cli2 "docs/**/*.md"` → 0 issues en el scope decidido
- [ ] Frontmatter loop del workflow (`gate-docs-21.yml` Check Frontmatter) → PASS
- [ ] Cero contenido tocado (solo formato); WIP ajeno intacto

## TAREA / ARCHIVOS / DEPENDENCIAS / REFERENCIAS / HERRAMIENTAS (ver prompt invocador §1-6 — no re-derivar)

- Clave: 12 archivos del log CI run 35413944160 + `docs/dev/tasks/AUD-033.md` (frontmatter) + `.markdownlint-cli2.yaml` + `.github/workflows/gate-docs-21.yml` (solo lectura).
- Prohibidos: contenido (solo formato), `reparacion.bat`, `.opencode`, `Justfile`, `ocr-*`, `completions/*`, `desktop/src-tauri/Cargo.lock`, stash@{0} GOV-C4, `C:/Users/Eros/.vantadb*`, `src/`, `web/`, `examples/`, `vantadb-ts/`, `desktop/`, plan file (solo recitation al cierre).
- Deps: Wave1 segunda (FIND-125 ✅ antes, archivos disjuntos). Sin bloqueantes. NextTask: FIND-127 (orquestador).
- Refs: `definition-of-done.md`, `pipeline.md`, SPEC.md raíz. Rules: ninguna de código (higiene docs).
- Herramientas: `npx markdownlint-cli2 "docs/**/*.md"` + loop frontmatter + `campaign_verify_cmd` (bug exit -1 → bash directa).

## DISCOVERY — conteo real (no opinión)

Repro local `npx markdownlint-cli2 "docs/**/*.md"`: **70 issues en 12 archivos** (igual que CI). `MCP.md:212` ya fixed `4f2d7d1c` — confirmado, no aparece.

| Archivo | Issues | Reglas | Fix formato-only |
|---|---|---|---|
| `docs/api/BINDINGS_NAMESPACES.md` | 2 (:36,:245) | MD028 | línea en blanco entre quotes → `>` |
| `docs/dev/Backlog.md` | 5 (:239 MD049 ×2, :373,:538,:540 MD028) | MD049/MD028 | `_x_` → `*x*`; blancos → `>` |
| `docs/benchmarks/ivf_bench.md` | 1 (:17) | MD028 | blanco → `>` |
| `docs/dev/research/archive/Investigacion-plan.md` | 4 (:562,:609,:636 MD003 + :4423 MD028) | MD003/MD028 | ⚠️ NO TOCAR (ver decisión) |
| `docs/strategy/VantaDB_Manual_Estrategico_Unificado.md` | 27 | MD010 tabs | `\t` → espacios |
| `docs/dev/tasks/complete/DRV-122.md` | 3 (:9-11) | MD007 | indent 2 → 0 |
| `docs/dev/tasks/complete/OLD-20.md` | 4 (:39-42) | MD007 | indent 2 → 0 |
| `docs/dev/tasks/CORE-01.md` | 9 (:119-120,:130-131 MD005+MD007, :122 MD023) | MD005/MD007/MD023 | indent 1 → 0; ` ###` → `###` |
| `docs/dev/tasks/GOV-A4.md` | 2 (:59,:106) | MD052 | `["record"]["key"]` → escapar |
| `docs/dev/tasks/MCP-41.md` | 3 (:7-9) | MD027 | `>   ` → `> ` |
| `docs/dev/tasks/PROV-03.md` | 2 (:30) | MD005+MD007 | indent 3 → 2 |
| `docs/dev/tasks/UX-19.md` | 8 (:23-29) | MD055 | pipes trailing/leading |
| Frontmatter (loop workflow) | **1** (`docs/dev/tasks/AUD-033.md` con `---` sin `title:`) | — | añadir `title:` |

**Hallazgo vs plan:** el plan decía "Frontmatter exige `title:` en los 1473 `docs/**/*.md` (falla estructural)". El workflow (`gate-docs-21.yml:44-53`) solo exige `title:` en archivos **que ya tienen frontmatter** (empiezan con `---`). Conteo real sobre 1626 md: **1 solo archivo** con frontmatter sin `title:` (`AUD-033.md`). No hay falla estructural masiva.

**Evidencia MD003 Investigacion-plan.md (archive, 5115 líneas):** slice aislado líneas 536-650 → 0 issues; prefix 1-559 → 0 issues; archivo completo → MD003 en :562/:609/:636. El flag depende de parser-state global (las 3 líneas son `name: ...` dentro de fences ```` ``` ```` de templates que GitHub renderiza como código). "Arreglarlo" como setext→atx cambiaría contenido renderizado. Pre-mortem del plan lo anticipó (churn/contenido). **Decisión: NO tocar el archivo; acotar scope del workflow** añadiendo `docs/dev/research/archive/**` a `ignores` de `.markdownlint-cli2.yaml`, con el mismo precedente ya escrito en ese archivo (`docs/archive/**` "Frozen history is never linted"). Formato-cero-riesgo, 1 línea config.

**Decisión scope (conteo):** limpieza directa 11 archivos (66 issues formato-only seguros) + 1 frontmatter + acote mínimo 1 línea config para el archivo archive. Sin necesidad de decisión del owner más allá de la ya dada (Gate P aprueba arreglar; el acote sigue el precedente existente del repo).

## Gate D

Blast radius 13 paths (>10 dispara Gate D formal). Sin `question` tool en este runner (precedente `docs/dev/tasks/UX-19.md:31`). Scope pre-autorizado por plan + Gate P owner ("crear un plan con las FIND para arreglar todo"), formato-only reversible, cero comportamiento → se procede y se registra. Pregunta que se haría: "¿limpieza total (11 files + 1 config) o acotar más?" → respondida con conteo: el acote extra es innecesario (66/70 fixes seguros).

## Impacto mapeado (Regla 0)

- Leídos completos (zonas con issues): BINDINGS_NAMESPACES.md:25-54,235-259 · Backlog.md:232-246,366-380,531-545 · ivf_bench.md:1-30 · AUD-033.md:1-20 · Investigacion-plan.md:556-575,4418-4427 (más fence-parity vía scripts) · Manual:428-439,998-1013 · DRV-122.md:1-20 · OLD-20.md:32-46 · CORE-01.md:114-133 · UX-19.md:18-32 · MCP-41.md:1-20 · gate-docs-21.yml + .markdownlint-cli2.yaml completos.
- Pendientes de leer antes de editar: GOV-A4.md:55-65,100-110 · PROV-03.md:25-35 · Backlog.md:239 exacto (línea larga) · BINDINGS exactos.
- Referencias hacia dentro: `gate-docs-21.yml` consume `.markdownlint-cli2.yaml` + `docs/**/*.md`. Ningún código consume estos md (docs-only).
- Referencias entrantes: plan file Task 4, Backlog FIND-126 (orquestador).
- Veredicto: impacto docs-only, sin código, sin tests, sin runtime. Reversible (`git checkout -- docs/`).

## Steps atómicos

- [x] S1: acote config — `.markdownlint-cli2.yaml` += `docs/dev/research/archive/**` (1 línea + comment precedente). Verify: lint ya no lista Investigacion-plan.md. ✅ 2026-09-19 (lint Finding excluye el path; Summary 0 issues).
- [x] S2: fixes MD028/MD049/MD027/frontmatter (BINDINGS ×2, Backlog ×5, ivf_bench ×1, MCP-41 ×3, AUD-033 title). Verify: esos files limpios. ✅ 2026-09-19 (diffs solo-formato verificados hunk por hunk).
- [x] S3: fixes listas/tablas (DRV-122, OLD-20, CORE-01, PROV-03, UX-19, GOV-A4) + tabs Manual (script `\t`→espacios, diff revisado). Verify: `markdownlint` 0 issues. ✅ 2026-09-19 (Manual: tabs→espacios, mismo texto).
- [x] S4: verify contrato completo (lint 0 + frontmatter loop PASS) + `git add` selectivo + commit `docs:`/`ci:` + recitation + RESULTADO. ✅ 2026-09-19.

## Cierre S4 (verify contrato, 2026-09-19)

- `npx markdownlint-cli2 "docs/**/*.md"` → `Linting: 1423 files / Summary: 0 issues in 0 files` ✅
- Frontmatter loop (`gate-docs-21.yml:44-53`, equivalente PowerShell) → 0 archivos sin `title:` ✅ (`AUD-033.md` ya tiene `title:`)
- Diffs verificados solo-formato (13 paths, `git diff -- <cada archivo>`): BINDINGS/MD028 `→>`, Backlog MD049/MD028, ivf_bench MD028, Manual MD010 tabs→espacios, DRV-122/OLD-20/CORE-01/PROV-03 MD007/MD005, GOV-A4 MD052 code-span, MCP-41 MD027, UX-19 MD055 pipes, AUD-033 frontmatter `title:` ✅
- Backlog.md: 4 hunks, todos formato-only, sin WIP ajeno → incluido en commit ✅
- Commit atómico `docs:` (NO PUSH) · Plan-file recitation la actualiza el orquestador (este file no toca `docs/dev/plans/` para no mezclar su WIP).

## Context Save Point

DISCOVERY completo 2026-09-19. Reproducción local OK (70/12 + frontmatter 1). Decisión scope escrita arriba. Prohibidos vigentes (ver arriba). `git status` base: WIP ajeno en `.opencode`, `completions/*`, `desktop/src-tauri/Cargo.lock`, `docs/dev/Backlog.md` (¡MODIFICADO por orquestador!), `docs/dev/plans/*`, `providers/ollama/Cargo.lock`, `skills/*`, `?? reparacion.bat` — staging SELECTIVO solo mis paths; si `docs/dev/Backlog.md` trae WIP ajeno en el diff, commitear SOLO mis hunks de formato (o coordinar).

## Spec

N/A (higiene docs, sin lógica nueva — tabla Spec del plan confirma).
