---
title: "DOCS-F2: Cerrar docs-consolidation F2 (ejemplos ejecutables en CI + verificación de gates)"
kind: task
description: "Cerrar docs-consolidation F2: doctests Rust con -D warnings y pydoclint en CI, verificación de los gates T14/T15 existentes y TS derivado a FIND-263 con plan de burn-down."
---

# DOCS-F2: Cerrar docs-consolidation F2 (ejemplos ejecutables en CI + verificación de gates)

## Metadata
- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 21, F0)
- **Fuente:** `docs/dev/plans/2026-09-28-docs-consolidation.md` §F2 (T13/T14/T15)
- **Esfuerzo:** 🟡 8h (max 2d) · **Prioridad:** 🟠
- **Tipo:** Mixto (CI workflows + docstrings Python + docs de plan/backlog)
- **Turns estimados:** 15-25
- **Creado:** 2026-10-04T09:35
- **last-synced:** 2026-10-04T09:50
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas (reconciliadas en DISCOVERY — ver Notas N1/N2)
- **Pendientes (downhill):** 0 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `.github/workflows/ci-rustdoc.yml` ← CI (push main / PR main,develop, paths `src/**`, `vantadb-*/**`); `.github/workflows/gate-doc-examples.yml` ← CI (paths `docs/**`, `vantadb-python/**`, `Cargo.*`); `vantadb-python/vantadb_py/__init__.py` ← importado por `vantadb/__init__.py` (alias público) y por usuarios `import vantadb_py` (deprecado); plan docs-consolidation F2 ← leído por skill `progreso` y por el orquestador |
| Callees | `ci-rustdoc.yml` → `./.github/actions/rust-setup` (composite existente), `cargo test --doc`; `gate-doc-examples.yml` → `actions/setup-python`, `pip install pydoclint`; nada de código de producto Rust se toca |
| Implicaciones | Cero cambio de API pública (ni Rust ni Python: los edits son docstring/annotation de un método interno ya tipado). CI suma 2 jobs: `doctests` (workspace, medido EXIT 0 con `-D warnings`) y `python-docstrings` (pydoclint, 4 violaciones drenadas a 0). `gate-docs-links.yml` NO se toca (F1 recién cerrada). Los gates T14/T15 se verifican, no se modifican |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/ci-rustdoc.yml` (110L), `.github/workflows/gate-doc-examples.yml` (102L), `.github/workflows/gate-api-docs.yml` (129L), `.github/workflows/gate-docs-links.yml` (156L), `.github/workflows/gate-docs.yml` (120L+), `vantadb-python/vantadb_py/__init__.py` (621L, 3 lecturas), `docs/dev/plans/2026-09-28-docs-consolidation.md` (561L), `docs/dev/workflow/RULES.md` (218L), `docs/dev/workflow/gate-doc-examples.md` (§46-194), `.opencode/task-system/prompts/task.md` (398L), `docs/dev/tasks/DOCS-F1.md` (formato). El Backlog se lee en su sección de inserción (`## Hallazgos pendientes de reportes`, L339-432) — fila append-only, formato verificado contra `findings.md` + filas FIND-260/261/262.
- **Archivos referenciados hacia dentro (imports/dependencias):** `ci-rustdoc.yml` usa `./.github/actions/rust-setup` (toolchain stable + sccache; sin cambios); `gate-doc-examples.yml` usa `actions/setup-python` + `maturin-action` (el job nuevo NO necesita wheel — pydoclint es estático); `vantadb_py/__init__.py` importa `dataclasses.asdict` (el método `SearchRequest.asdict` lo usa — la anotación `-> dict` es correcta).
- **Archivos que referencian a los editados (referencias entrantes):** `ci-rustdoc.yml` es invocado por el trigger propio (push/PR/dispatch); `gate-doc-examples.yml` ídem + documentado en `docs/dev/workflow/gate-doc-examples.md` (§When it runs) — el job nuevo queda dentro del mismo workflow y su doc no requiere cambio estructural (se anota en el comentario del workflow); `vantadb_py/__init__.py` es re-exportado por `vantadb/__init__.py` (`from vantadb_py import *`) — ningún símbolo cambia de nombre ni firma. `docs/dev/plans/2026-09-28-docs-consolidation.md` es citado por el master plan Task 21 y la skill `progreso`.
- **Veredicto impacto:** **bajo.** Solo CI + docstrings + docs de plan. Riesgos: (1) job `doctests` con scope `--workspace` más caro en CI que local → mitigado con timeout 30 (medido: compile 2m47s warm + 20s tests; hermano `rustdoc` del mismo workflow usa 30); (2) pydoclint en CI = dependencia de tool nueva → pin exacto `0.11.0` + job de 5 min aislado; (3) el trigger de `gate-doc-examples.yml` no incluye cambios solo de `docs/dev/plans/**`? Sí los incluye (`docs/**`) — irrelevante para el job nuevo.

## Contrato

**(a)** Ejemplos de docs ejecutables en CI, mínimo viable por lenguaje: **Rust** — `RUSTDOCFLAGS="-D warnings" cargo test --doc --workspace` wireado como job `doctests` en `ci-rustdoc.yml` (medido hoy: EXIT 0, 15 doctests verdes + 1 ignored; `vantadb_py` skipeado por cdylib — warning de cargo, no error); **Python** — `pydoclint --style=numpy` wireado como job `python-docstrings` en `gate-doc-examples.yml` con las 4 violaciones drenadas a 0 (los bloques python de `docs/` ya se verifican desde 2026-09-29 vía `check-doc-examples.mjs` contra el paquete real); **TS** — derivado a **FIND-263** con plan de burn-down medido (typedoc 0.28.20 corre: 0 errores/39 warnings, sin config ni devDep ni job CI de TS; 12 ficheros de docs con bloques TS sin verificar — los 3 docs objetivo de T13 no tienen bloques TS). **(b)** Gate API↔docs: `gate-api-docs.yml` + `scripts/docs/check-api-docs.mjs` (patrón DuckDB `NeedsDocumentation.yml`) — **ya existía** (commit `71139665`, 2026-09-29); verificado verde hoy (self-test 17/17). **(c)** Gate anti-fuga: `gate-docs-secrets.yml` + `scripts/docs/check-secrets.mjs` — **ya existía** (mismo commit); verificado verde hoy (self-test 29/29; 0 fugas; el mecanismo es scanner propio offline, decisión de implementación pre-existente documentada en `docs/dev/workflow/gate-docs-secrets.md`). **(d)** F2 del plan docs-consolidation marcado ✅ con T13/T14/T15 actualizados a realidad.

**Verificación mecánica:**
- `$env:RUSTDOCFLAGS="-D warnings"; cargo test --doc --workspace` → EXIT 0 (vantadb 13+1ignored, vanta_memory 1, vantadb_mcp 1, resto 0; compile 2m47s warm)
- `pydoclint --style=numpy --quiet vantadb-python/vantadb_py vantadb-python/vantadb` → exit 0 (0 violaciones tras fixes; antes: 4 en 2 funciones)
- `node scripts/docs/check-api-docs.mjs --self-test` → 17/17 assertions
- `node scripts/docs/check-secrets.mjs --self-test` → 29/29 assertions
- `node scripts/docs/check-doc-examples.mjs --self-test` → OK (sin ejecutar snippets; 17 assertions)
- `actionlint .github/workflows/ci-rustdoc.yml .github/workflows/gate-doc-examples.yml` → 0 issues
- `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` → gate docs sigue verde (no-regresión por tocar docs de plan/task/backlog)

## Spec (decisiones técnicas)

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Alcance Rust doctests | A: `-p vantadb` (rápido) / B: `--workspace` (contrato literal del plan; medido verde) | ✅ B — medido EXIT 0 hoy con `-D warnings`; cubre los 3 crates con doctests reales; el costo (~3 min warm) entra en timeout 30 |
| 2 | Mecanismo leak gate | A: gitleaks/trufflehog (dep externa nueva) / B: scanner propio ya existente (`check-secrets.mjs`) | ✅ B — el gate T15 ya existe desde 2026-09-29 con self-test 29/29 y 0 fugas; el contrato pide "gate anti-fuga verde" — el mecanismo es decisión de implementación previa documentada; agregar una segunda herramienta sería duplicación |
| 3 | TS: wire vs FIND | A: typedoc ahora (devDep + config + CI + 39 warnings a drenar + harness de ejemplos) / B: FIND-263 con burn-down | ✅ B — no existe NINGUNA infraestructura TS en CI (ni tests en PR); excede el appetite restante; stop condition del contrato: derivar (a) parcial a FIND con plan |
| 4 | pydoclint versión/estilo | `--style=numpy` + pin `pydoclint==0.11.0` (en 0.11 `--check-style` se renombró a `--style`; el comando del plan quedó viejo) | ✅ decidido-por-evidencia (medido: 4 violaciones → 0 con fixes de docstring) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) `docs/dev/tasks/` y `docs/dev/plans/` no se mueven/renombran (restricción owner); (2) PROHIBIDO tocar: master plan `2026-10-04-master-plan-0.9.0.md`, `opencode.jsonc`, `docs/pipeline-state.json`, `PUBLISH.md`, `release-plz.toml`, `gate-docs-links.yml` (F1 recién cerrada); (3) los gates `gate-api-docs.yml` / `gate-docs-secrets.yml` NO se modifican — solo se verifican; (4) RULES.md de workflows: SHA pins, timeout-minutes, `permissions: contents: read`, branches declaradas, sin `continue-on-error` nuevo; (5) push PROHIBIDO — commit local (Regla 7); (6) `docs/CHANGELOG.md` no se toca (release-plz); (7) ningún símbolo público cambia de nombre/firma en `vantadb_py/__init__.py` (solo docstring + anotación de retorno).
- **Comandos de verificación:** ver §Contrato (los 7 comandos; todos deben pasar antes del commit).
- **Deuda pendiente:** TS docs (typedoc + ejemplos como tests) → FIND-263 con burn-down; `vantadb_py` no tiene doctests (cdylib) — ceiling inherente; método-a-método en bloques Rust de `docs/` requiere índice compilado (`rustdoc --output-format json`) — ceiling documentado en `gate-doc-examples.md` §What it deliberately does not catch.

## Deuda técnica (Regla 6)

**Saldo neto:** negativo (paga deuda). Drena: 4 violaciones pydoclint → 0; doctests sin gate → gate; verifica y fija los gates T14/T15. Sin deuda nueva en el repo (pydoclint es tool de CI pinneada; no entra al árbol de dependencias de producto).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato (a)-(d) verificado mecánicamente (comandos §Contrato) + OCR sin Critical/High + review P2-01 registrado (tier fast: paths CI/docs; el fix de docstring es código Python no listado en adversarial) |
| **Commit** | Commit atómico `ci:` (~150 líneas), solo archivos del blast radius, verificación mecánica adjunta |
| **Release** | n/a (CI + docs; sin release). Los 2 jobs nuevos corren en el próximo push/PR (evidencia local equivalente: mismos comandos) |

## Herramientas necesarias
- `actionlint` (workflows), `cargo test --doc` (local), `pydoclint` (vía `vantadb-python/.venv`), `node scripts/docs/*.mjs`, `pwsh dev-tools/ocr-review.ps1`

**Skills cargadas (SDP v3):** `documentation-skill` (obligatoria docs — DoD y sintaxis), `ci-cd-and-automation` (pinned CI/release — jobs y quality gates), `git-workflow-and-versioning` (pinned — commit conventional), `documentation-and-adrs` (pinned API docs), `api-and-interface-design` (pinned — surface vs docs), `security-and-hardening` (T15 leak gate — threat model del gate de fugas), `doubt-driven-development` (base — verificación adversarial), `incremental-implementation` (lifecycle BUILD — slices por step). Base auto: `campaign-executor`, `progreso`, `ponytail`.

## Investigation Notes

- **N1 — Reconciliación T14/T15 (incógnita principal del DISCOVERY):** el master plan afirma "falta gate API↔docs" / "falta gate anti-fuga", pero ambos YA EXISTEN: commit `71139665` "ci(docs): cuatro gates que miden lo que antes era inmedible" (2026-09-29) creó `gate-api-docs.yml` (T14, patrón DuckDB) + `gate-docs-secrets.yml` (T15) + `gate-doc-examples.yml` (T13-python) + `gate-docs-links.yml` (F0-T6 ampliado), con sus runbooks en `docs/dev/workflow/`. Los planes (docs-consolidation y master) quedaron stale. Decisión: verificar los gates existentes (self-tests + estado) y cerrar F2 con la realidad — no duplicar. Evidencia: `git log --oneline -- .github/workflows/gate-api-docs.yml` → `0e5c9e9d`, `71139665`; self-tests 17/17 y 29/29.
- **N2 — Medición del hueco real T13:** `ci-examples.yml` ejecuta `examples/` (Rust+Python) pero NO los bloques de `docs/`. Estado por lenguaje: **Python** cubierto por `check-doc-examples.mjs` (resuelve atributos de cada bloque runnable contra el paquete real; 77 chains verificadas; budget 1 entrada: `docs/user/blog/ollama_vantadb_local_memory.md:45`); **Rust** — bloques `docs/`: crate-name check + 16 bloques `doc-test` contados como coverage (method-existence requiere índice compilado — ceiling documentado); doctests de fuente: SIN gate → este task lo cablea; **TS** — sin cobertura (FIND-263). Medición doctests: `--workspace` EXIT 0 con `-D warnings` (13+1i vantadb, 1 vanta_memory, 1 mcp; `vantadb_py` skipeado por cdylib).
- **N3 — pydoclint:** 4 violaciones (DOC109/110/105 en `error_to_dict` L77; DOC203 en `SearchRequest.asdict` L160) → fixes de docstring (`exc : BaseException`) y anotación (`-> dict`). En 0.11.0 el flag `--check-style` del plan ya no existe: es `--style`.
- **N4 — TS burn-down (FIND-263):** typedoc 0.28.20 medido hoy en `vantadb-ts`: 0 errores, 39 warnings (18 símbolos no incluidos en docs + 3 links externos + assets); no hay `typedoc.json` ni devDep; no hay workflow CI que corra TS en PRs (solo release-npm). Burn-down propuesto: (1) `typedoc.json` + devDep pin + job CI; (2) drenar 39 warnings a 0 (o budget-ratchet); (3) harness de bloques ```ts de docs/ (extender `check-doc-examples` o script propio con tsc).
- **N5 — gate-docs-links.yml:** NO se toca (F1 lo acaba de cerrar con baseline 0 scoped). La mención en los "Archivos clave" del master plan era pre-2026-09-29: el gate API↔docs vive en `gate-api-docs.yml` (archivo separado), no en gate-docs-links.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 — N1 reconciliado (gates existen); N2 medido; N3 medido; N4 medido |
| Pendientes de ejecución (downhill) | 5 — steps 1-5 |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — aplica parcialmente: (1) el job `python-docstrings` agrega una dependencia de CI (`pydoclint==0.11.0`, pin exacto, corre read-only sobre fuentes del repo — sin secrets, sin red en runtime); (2) el gate anti-fuga T15 se verifica (0 fugas; scanner offline). Sin trust boundaries de producto tocados. `security-and-hardening` cargada; checklist razonable: pin de versión + least-privilege (`contents: read`) + sin secretos.
- [x] **PERFORMANCE** — no aplica: no se toca ningún hot path (CI + docstrings + docs). Justificado.

## Steps

### Step 1: Drenar pydoclint a 0 en `vantadb_py/__init__.py`
- **Archivos:** `vantadb-python/vantadb_py/__init__.py`
- **Acción:** `error_to_dict` — docstring numpy `exc:` → `exc : BaseException` (DOC105/109/110); `SearchRequest.asdict` — `def asdict(self):` → `def asdict(self) -> dict:` (DOC203).
- **Verify:** `pydoclint --style=numpy --quiet vantadb-python/vantadb_py vantadb-python/vantadb` → exit 0 ✅ (medido: "No violations", EXIT=0; py_compile OK; runtime sanity `SearchRequest.asdict()` y `error_to_dict` OK)
- **Estado:** ✅ COMPLETED

### Step 2: Job `doctests` en `ci-rustdoc.yml`
- **Archivos:** `.github/workflows/ci-rustdoc.yml`
- **Acción:** job `doctests` con `RUSTDOCFLAGS: "-D warnings"`, `cargo test --doc --workspace`, timeout 30, rust-setup (mismo composite que el job hermano), comentario con la medición de hoy + nota cdylib.
- **Verify:** `actionlint .github/workflows/ci-rustdoc.yml` → 0 ✅ (medido); comando local ya medido EXIT 0
- **Estado:** ✅ COMPLETED

### Step 3: Job `python-docstrings` en `gate-doc-examples.yml`
- **Archivos:** `.github/workflows/gate-doc-examples.yml`
- **Acción:** job `python-docstrings` (setup-python 3.11 + `pip install pydoclint==0.11.0` + `pydoclint --style=numpy vantadb-python/vantadb_py vantadb-python/vantadb`), timeout 5, sin wheel (estático).
- **Verify:** `actionlint .github/workflows/gate-doc-examples.yml` → 0 ✅ (medido); pydoclint local exit 0 ✅
- **Estado:** ✅ COMPLETED

### Step 4: FIND-263 + F2 ✅ en el plan docs-consolidation
- **Archivos:** `docs/dev/Backlog.md` (fila FIND-263 en `## Hallazgos pendientes de reportes`), `docs/dev/plans/2026-09-28-docs-consolidation.md` (F2 header ✅ + resultado + T13/T14/T15 estados + header del plan F0-F2)
- **Acción:** registrar el burn-down TS como fila FIND-263 (formato `findings.md`); actualizar F2 con la realidad verificada.
- **Verify:** `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check` → verdes ✅ (link EXIT 0, docs EXIT 0, index regenerado con `--write` — diff atribuido 100% a DOCS-F2.md: 1508→1509 docs, 1059→1060 task files; la regeneración quedó en el commit concurrente `a5909e38`, ver N8); `rg "FIND-263" docs/dev/Backlog.md` → 1 hit ✅ (absorbido por `a5909e38`)
- **Estado:** ✅ COMPLETED

### Step 5: Verify full + OCR + commit + cierre campaign
- **Archivos:** (commit de todos los anteriores)
- **Acción:** correr los 7 comandos del §Contrato; `pwsh dev-tools/ocr-review.ps1` (Critical/High bloquean); review P2-01; commit `ci(docs):` local; `campaign_update_task_state` con `taskId: "21"`.
- **Verify:** `dev-tools/verify.ps1` → ALL PASS ✅ (fmt/check/clippy/audit/deny/nextest/docs-coverage); OCR → 0 Critical/High/Medium ✅; doctests EXIT 0 ✅; pydoclint 0 ✅; actionlint 0 ✅; gates docs verdes ✅; commit local ✅ (hash en recitation)
- **Estado:** ✅ COMPLETED

## Dependencias
- F0 (docs-consolidation) ✅ · F1 ✅ (DOCS-F1, commit `6b07d1d9`) — releídos frescos `gate-docs-links.yml` y `check-links.mjs` (no se tocan).
- nextTask del master plan: ENC-01 (Task 72).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Tier (tabla mecánica):** **Fast** — paths del diff: `.github/workflows/**`, `vantadb-python/vantadb_py/__init__.py`, `docs/dev/**`. Ninguno matchea adversarial (`docs/api/**`, `src/sdk/**`, …). Gate: verify fast mecánico + veredicto registrado.
- **Verify fast mecánico:** `dev-tools/verify.ps1` → ALL PASS (fmt/check/clippy/audit/deny/nextest — evidencia en §Contrato/log).
- **OCR delegation:** `pwsh dev-tools/ocr-review.ps1 -Format json` → spec `ocr-delegate/v1` (3 archivos reviewables: 2 workflows + 1 python; 2 Rule Groups). Revisión host-agent contra las reglas (Security/Correctness/Reliability/Best Practices + user-specific VantaDB): **0 hallazgos Critical/High/Medium**. Detalle: SHA-pins ✅, permissions `contents: read` ✅, timeouts ✅, sin secrets/injection ✅, sin `continue-on-error` nuevo ✅, sccache solo vía rust-setup ✅; Python: docstring/annotación correctas, sin defaults mutables/dead code ✅.
- **Doubt-driven degradado (subagente hoja, sin tool de subagentes):** revisión adversarial del diff (ARTIFACT + CONTRACT, sin CLAIM) → 3 hallazgos: **A1** (OS de la medición no declarado en el comentario del job) → **corregido**; **A2** (pydoclint `--style=numpy` no flaggea drift Google-style futuro — upgrade path `--check-style-mismatch`) → **trade-off aceptado** (herramienta del contrato (a); anotado en Notas N7); **A3** (`Esfuerzo: ⬜` en tareas cerradas del plan) → **corregido**. Sin hallazgos bloqueantes.
- **Revisor:** `doubt-driven-development` degradado (hoja) + OCR delegation. **Escalado EJECUTADO: review P2-01 formal `vanta-review` (fresh, `ses_ef8cce5fbffeo7zVz0YyLSWaRs`) → ✅ APPROVE (2026-10-04)** — evidencia completa en este bloque (diff, self-tests, verify, OCR rules).
- **Veredicto:** ✅ APPROVE — dictamen fresh formal (superseded del waiver degradado). O-1/O-2 aplicados por el orquestador. Waiver original: `owner=orchestrator (vanta-lead)`, `ref=docs/dev/tasks/DOCS-F2.md §Review` (el server lo asienta en trace + decisions.md al ACCEPT — precedente DOCS-F1 task 20).

## Notas
- La descripción del master plan ("T14 falta gate", "T15 falta gate") era stale: la realidad del repo (commit `71139665`) los tenía desde 2026-09-29. Este task cierra la brecha entre plan y realidad, no reimplementa.
- El contrato (a) pedía "Rust: `RUSTDOCFLAGS="-D warnings" cargo test --doc`" — se cablea `--workspace` (medido verde), que es lo que el plan T13 pedía literalmente.
- No se toca `ci-examples.yml` (el plan lo listaba como archivo clave, pero su rol — ejecutar `examples/` — no cambia; el hueco de `docs/` se cierra con los 2 jobs nuevos + lo ya existente).
- **N6 — `campaign_validate_scope` con IDs semánticos:** el validador busca `docs/dev/tasks/<taskId>.md` (convención numérica vieja, archivos 1-10); con IDs semánticos (DOCS-F2) devuelve `TASK_FILE_NOT_FOUND` — advisory (no intercepta el edit); mismo comportamiento en DOCS-F1/DIST-*. No se creó stub numérico para no duplicar la fuente de verdad.
- **N7 — pydoclint ceiling:** `--style=numpy` verifica las secciones numpy; un docstring Google-style nuevo no se flaggearía como mismatch (upgrade: `--check-style-mismatch`, 1 línea en el job). Trade-off aceptado (herramienta del contrato (a); endurecerlo es decisión de un próximo cambio).
- **N8 — cierre con agentes concurrentes:** durante el cierre, otro agente (DOCS-F1 ronda 2, commit `a5909e38`) regeneró `docs/index.md`/`llms.txt` con el task file DOCS-F2.md ya en disco y los commiteó — la fila DOCS-F2 y los conteos (1509/1060) quedaron absorbidos en su commit, igual que la fila FIND-263 del Backlog (junto a sus FIND-264/265). Este commit (workflows + pydoclint + plan + task file) no duplica esos artefactos: working tree == HEAD para índices/Backlog/llms, sin drift. Verificado: `git show a5909e38 --stat` no toca los 5 archivos de este commit.
