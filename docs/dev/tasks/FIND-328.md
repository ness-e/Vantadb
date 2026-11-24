---
title: "FIND-328 - triage the three red heavy-certification jobs with evidence"
kind: task
status: completed
description: "other_heavy and mutation_test were broken by CI defects (unsatisfiable dataset gate, invalid action pin plus a command that never ran tests); memory_concurrency has been green on 25/25 scheduled runs. Two residual bench-family reds are filed with repro."
tags: [ci, certification, gates, triage]
---

# FIND-328: Heavy cert con 3 jobs rojos sin triage (other_heavy, memory_concurrency, mutation_test)

## Metadata
- **Plan file:** `docs/dev/plans/2026-11-23-release-v0.10.0-gates.md` (§5, Fase 1, wave 1.1)
- **Fuente:** `docs/dev/Backlog.md` fila `FIND-328` (origen: cierre CI 2026-10-08, tras FIND-327)
- **Esfuerzo:** 🟢 1h
- **Prioridad:** 🟡 P2
- **Tipo:** CI/CD (YAML + runbook) — sin código Rust
- **Turns estimados:** 8
- **Creado:** 2026-11-24T19:10
- **last-synced:** 2026-11-24T19:10
- **Estado:** ✅ COMPLETED
- **Incógnitas (uphill):** 0 abiertas
- **Pendientes (downhill):** 0

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | Ninguno en código: `certification.yml` es manual/scheduled (nunca Fast Gate, nunca required check de PR). |
| Callees | `scripts/verify_datasets.sh`, `scripts/download_benchmark_datasets.sh`, `actions/upload-artifact`, `taiki-e/install-action`, `./.github/actions/rust-setup`. |
| Implicaciones | No toca el merge de PRs. Sí afecta la certificación previa al release (tier Heavy): un job retirado o degradado sin razón pierde cobertura de concurrencia/mutación. `CI_POLICY.md` §2 lista los jobs por nombre → se sincroniza en el mismo commit. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `.github/workflows/certification.yml` (secciones `memory-concurrency` L245-286, `other-heavy` L287-354, `mutation-test` L355-387, `light-benchmarks` L388-463, `high-density` L464-491), `docs/dev/operations/CI_POLICY.md` (609 L), `scripts/verify_datasets.sh` (completo), `scripts/download_benchmark_datasets.sh` (completo).
- **Archivos referenciados hacia dentro:** `tests/benchmark_datasets.rs` (único consumidor de `data/benchmark/glove.6B.100d.txt` en `other_heavy`); `.github/actions/rust-setup` (composite).
- **Archivos que referencian a los editados:** `rg -n "verify_datasets"` → solo `scripts/verify_datasets.{sh,ps1}`, `data/README.md`, docs de revisión y CHANGELOG (histórico). `scripts/verify_datasets.*` **no se toca** (queda como herramienta local/opt-in; el contrato exit 0/1 se preserva).
- **Veredicto impacto:** bajo. Ningún path de código, ningún gate de PR, ningún contrato de API. Lo único con consecuencia real: el tier Heavy deja de producir 3 rojos permanentes y 1 job pasa de "score" mentiroso a inventario etiquetado.

## Contrato

"`gh run list --workflow certification.yml --limit 5 --json conclusion,headSha,createdAt` corrido; cada uno de `other_heavy`, `memory_concurrency` y `mutation_test` tiene veredicto escrito con evidencia de run; el pin inválido de `upload-artifact` ya no existe en el archivo; `actionlint .github/workflows/certification.yml` pasa; `scripts/verify_datasets.sh` conserva su contrato exit 0/1."

## Spec (SDD — decisiones resueltas por evidencia, no por feature-add)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | `other_heavy`: qué hacer con el gate TBH-01 de 5 datasets | (A) descargar los 5 en CI (GB + red externa en cada run) / (B) acotar el gate al dataset que el job consume, después de la descarga / (C) borrar el gate (vuelve el skip silencioso que TBH-01 mató) | (B) | ✅ decidido-por-evidencia: el log del run 45388973797 muestra las 5 MISSING **antes** de la descarga; `other_heavy` solo consume GloVe-100 (`tests/benchmark_datasets.rs:6,16`); los otros 4 son de los jobs opt-in `sift-validation`/`competitive-bench`. (B) conserva el fail-fast real y no agrega costo de red. |
| 2 | `mutation_test`: el comando | (A) quitar `--check` (corre la suite por mutante: horas × N mutantes) / (B) dejarlo con `--check` (solo compila mutantes: **no mide nada**, y `--output-json` no existe → verde-como-no-op) / (C) inventario honesto (`--list --json`) etiquetado INFORMATIONAL + fila para el gate real | (C) | ✅ decidido-por-evidencia: `mutants.rs/print.html` define `--check` como *"Run cargo check on all generated mutants … but don't actually run the tests"*; `--output-json` no aparece en el manual ni en el código (`options.rs` expone `check_only`/`emit_json`). |
| 3 | `memory_concurrency` | (A) retirar / (B) dejar como está | (B) | ✅ decidido-por-evidencia: 25/25 runs programadas con el job en `success` (ver §Evidencia). No hay nada que arreglar; retirarlo perdería el gate de concurrencia que exige la Regla 8. |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) `certification.yml` NUNCA entra al Fast Gate ni a `paths:` de un required check de PR; (2) `continue-on-error` sigue prohibido sin `# CATEGORY` (Regla 2) — este cambio **no agrega ninguno**; (3) `scripts/verify_datasets.sh` conserva exit 0/1 (herramienta local y de jobs opt-in); (4) ningún test se deshabilita ni se borra.
- **Comandos de verificación:** `actionlint .github/workflows/certification.yml` → 0 issues · `gh run list --workflow certification.yml --limit 5 --json conclusion,headSha,createdAt` → estado real del tier · `rg -n "4e7e4d1" .github/workflows/` → 0 hits.
- **Deuda pendiente:** el commit exacto que devolvió `memory_concurrency` a verde no es aislable (el workflow viejo `heavy-certification.yml` se borró en `a4c290914` y sus runs ya no se enumeran por `gh`). Documentado abajo con candidatos y confianza media.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | Encabezado `# FIND-328` |
| `lastAction` | Step 4 ✅ (commit local con triage + 2 fixes de CI) |
| `result` | `OK` ↔ ✅ COMPLETED |
| `nextAction` | ninguno (fila `FIND-716` propuesta para el bench family) |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | `FIND-264` (paralela, `vanta-docs`) / `F0.2` (paralela, `vanta-worker`) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda

> El cambio borra deuda (un pin inválido + un comando semánticamente falso + un gate imposible de satisfacer). No agrega deuda nueva; la cobertura de mutación *real* nace como fila `FIND-717` con evidencia, no como promesa.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | 3 veredictos con evidencia de run; YAML válido (`actionlint`); contrato de §Contrato en verde |
| **Commit** | 1 commit atómico `ci(heavy):` con los 3 archivos del scope; sin push |
| **Release** | N/A — no toca versionado, packaging ni API pública. Justificado en Notas |

## Herramientas necesarias
- `gh` (run list / run view --log-failed / api jobs) — estado real de CI
- `actionlint` — validación del workflow
- `rg` — pins y referencias cruzadas

**Skills cargadas (SDP):** `campaign-executor`, `progreso` (base auto-cargadas) · `ci-cd-and-automation` (pinned: CI/release — gates y feedback de fallos) · `git-workflow-and-versioning` (pinned: CI/release — conventional commit + atomicidad) · `systematic-debugging` (triage: log antes de fix, Hipótesis → 1 variable) · `doubt-driven-development` (review adversarial en contexto fresco).

## Investigation Notes

### Evidencia por job (runs 2026-11-12 → 2026-11-24, `certification.yml`)

```
gh run list --workflow heavy-certification.yml --limit 5 --json conclusion,headSha,createdAt
→ HTTP 404: workflow heavy-certification.yml not found on the default branch
```
El nombre del backlog está muerto: el archivo real es **`certification.yml`** (renombrado en
`a4c290914`, 2026-10-15, "consolidar workflows 40-18"). `CI_POLICY.md` §2 ya lo dice. El 404
es la razón por la que nadie miró: el comando de triage del backlog no apunta a ningún lado.

Serie por run (per-job conclusions de `gh api .../runs/<id>/jobs`; 25/25 runs programadas 2026-11-06 -> 2026-11-24 verdes para `memory_concurrency`):

| Job | Estado | Evidencia |
|-----|--------|-----------|
| `other_heavy` | **failure en las 12 muestreadas** | run 45388973797: step *Verify required benchmark datasets (TBH-01)* → `=== verify_datasets: 5 missing / 5 expected ===` + `##[error]Process completed with exit code 1` (11:59:58). Falla **antes** de la descarga que provee GloVe-100. |
| `memory_concurrency` | **success 25/25** | 11:57:56 → 12:06:32 (8.6 min). Job idéntico byte a byte al de `heavy-certification.yml` previo (`git show a4c290914^:.github/workflows/heavy-certification.yml`); el fix fue en código/tests, no en CI. |
| `mutation_test` | **failure en las 12 muestreadas, 1 step** | `##[error]Unable to resolve action 'actions/upload-artifact@4e7e4d1bb8e5ae8b5b8e6dad68a47e8afbd4d76a', unable to find version '...'` → el job muere en "Set up job" (3 s), nunca ejecuta cargo-mutants. |

### Causa raíz de cada rojo

1. **`mutation_test` — 2 bugs superpuestos.** (a) El pin `4e7e4d1…` (comentado `# v4.5.0`) **no existe** en `actions/upload-artifact` (`gh api repos/actions/upload-artifact/commits/4e7e4d1…` → **HTTP 422 "No commit found for SHA"**; el SHA real de v4.5.0 es `6f51ac03…`). Todo el resto del repo usa `cf430e03… # v7.0.2`, que sí resuelve. (b) Aun con el pin arreglado, el comando no mide mutación: `--check` significa *"Run cargo check on all generated mutants … but don't actually run the tests"* (manual oficial) y `--output-json` **no es un flag** (no aparece en el manual ni en `src/options.rs`), así que el paso fallaría con exit 2 y el `|| true` lo convertiría en **verde sin artefacto** — el peor resultado posible.
2. **`other_heavy` — gate imposible de satisfacer.** El step TBH-01 corre `verify_datasets.sh` (5 datasets) **antes** del paso de descarga, y la descarga (`download_benchmark_datasets.sh`) solo trae GloVe-100. Los otros 4 (SIFT-1M, GloVe-300, subsets ground-truth) pertenecen a los jobs opt-in `sift-validation`/`competitive-bench`. Un gate que no se puede satisfacer no es un gate: es un rojo permanente con apariencia de protección.
3. **`memory_concurrency` — ya arreglado.** El job nunca falló desde que existe el archivo consolidado. Los cambios que caen en la ventana del rojo original (4-oct → 15-oct) son `6b8002c1c` (DUR-03, 10-04), `23c594f4f` (MEMG-11, 10-05) y `8aad0cac1` (10-06, `test(edge_cases): FIND-244 — all-zeros expectation matches usable_vector filter`), este último sobre `tests/edge_cases.rs`, que es exactamente el batch 1 del job. **Confianza media** en la atribución exacta (los runs del workflow borrado no son enumerables); **confianza alta** en que hoy está verde.

### Hallazgo colateral (fuera del scope de las 3, va como fila)

`Light benchmarks` y `High density` fallan en **las 12 muestreadas** runs con **0 steps** (el log dice
`log not found`): nunca arrancan, ni siquiera cuando su `if:` las deja fuera de la ventana por
cron. **Cuatro** jobs del archivo declaran un `concurrency:` a nivel de job que reusa **el mismo
group que el `concurrency:` a nivel de workflow** (`${{ github.workflow }}-${{ github.ref }}`):
`light-benchmarks` (L416), `high-density` (L489), `analyze` (L524) y `critcmp-regression` (L697).
Según la doc oficial de GitHub, "there can be at most one running job or workflow in a concurrency
group at any time… a queued job will be `pending`" y con `queue: single` un segundo pendiente
cancela al primero: el job espera el slot que el propio run retiene hasta terminar, así que muere
sin ejecutar nada. Los dos que se ven rojos en cada run son los que el cron habilita; los otros dos
solo exponen el mismo muro cuando su `if:` se cumple (`analyze` en dispatch, `critcmp-regression`
con su input). Los otros dos bloques `concurrency:` job-level del archivo están bien
(`failpoint-tests` usa `failpoints-${{ github.ref }}`, `fuzz-pr` añade `${{ github.job }}`). El fix
es borrar el bloque en los **cuatro**, no en dos.

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 (la atribución exacta del fix de `memory_concurrency` quedó documentada como confianza media, no como incógnita bloqueante) |
| Pendientes de ejecución (downhill) | 0 |
| % completado | 100% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no aplica: sin trust boundaries, sin input de usuario, sin dependencias nuevas (se *corrige* un pin de acción a un SHA verificado). Justificado.
- [x] **PERFORMANCE** — no aplica a hot paths del motor. Efecto lateral medible: `mutation_test` pasaría de "falla en 3 s" a "inventario en segundos"; `other_heavy` deja de abortar en el minuto 2.

## Steps

### Step 1: Reproducir el estado real de CI (matar la premisa del nombre)
- **Archivos:** —
- **Acción:** `gh run list --workflow heavy-certification.yml` → 404; localizar `certification.yml`; bajar el log fallido del último run programado.
- **Verify:** `gh run view 45388973797 --log-failed` muestra los 3 errores distintos.
- **Estado:** ✅ COMPLETED

### Step 2: Veredicto `memory_concurrency`
- **Archivos:** —
- **Acción:** serie de 25 runs programadas + comparación byte del job viejo vs nuevo.
- **Verify:** 25/25 `success` (muestra: 11:57:56 → 12:06:32).
- **Estado:** ✅ COMPLETED

### Step 3: Fix `other_heavy` (gate TBH-01 acotado y después de la descarga)
- **Archivos:** `.github/workflows/certification.yml`
- **Acción:** mover el gate detrás del download/cache y acotarlo al dataset que el job consume (GloVe-100), dejando `verify_datasets.sh` intacto para uso local/opt-in.
- **Verify:** `actionlint .github/workflows/certification.yml` → 0 issues.
- **Estado:** ✅ COMPLETED

### Step 4: Fix `mutation_test` (pin inválido + comando honesto)
- **Archivos:** `.github/workflows/certification.yml`, `docs/dev/operations/CI_POLICY.md`
- **Acción:** pin → `cf430e03… # v7.0.2`; comando → inventario `--list --json` (lo que el job sí puede hacer en presupuesto), `if-no-files-found: error` para que un artefacto vacío vuelva a ser rojo; categoría `INFORMATIONAL` declarada en `CI_POLICY.md`.
- **Verify:** `rg -n "4e7e4d1" .github/workflows/` → 0 hits · `actionlint` → 0 issues.
- **Estado:** ✅ COMPLETED

### Step 5: Registrar el veredicto y los hallazgos colaterales
- **Archivos:** `docs/dev/operations/CI_POLICY.md`, este task file
- **Acción:** sección de triage FIND-328 con los 3 veredictos + tabla de jobs sincronizada + fila propuesta `FIND-716` (bench family) en el reporte.
- **Verify:** `pwsh dev-tools/ocr-review.ps1 -Format json` sin Critical/High sobre los paths tocados.
- **Estado:** ✅ COMPLETED

## Dependencias
- Ninguna. Wave 1.1 en paralelo con `FIND-264` (`docs-gates.yml`, otro archivo) y `F0.2` (`vanta-worker`, builds locales).

## Review (GATE — agente distinto, P2-01)

- **Revisor:** `vanta-review` (agente distinto, contexto fresco — P2-01 fresh, no degradado). Revisó `1bf4c2d1a`; emitió **changes-required** con 4 hallazgos, todos aplicados antes del commit final `51de511c1`+amend (ver §Notas).
- **Enfoque:** ¿acotar el gate de datasets preserva el propósito de TBH-01? ¿Convertir `mutation_test` en inventario es "arreglar retirando"? ¿El veredicto de `memory_concurrency` se apoya en evidencia o en ausencia de datos? ¿La atribución del 0-step red se sostiene contra el archivo y la doc?
- **Cómo se probó:** logs crudos de `gh run view --log-failed` (no auto-reporte), series de 25 runs con conclusión por job, `gh api` sobre los SHAs de las actions, manual oficial de cargo-mutants, código de cargo-mutants (`src/main.rs`, `src/list.rs`), doc oficial de concurrency de GitHub y `actionlint`.
- **Hallazgos del revisor y disposición:**
  - HIGH — "los únicos 2 jobs con `concurrency:` job-level" era **falso**: son 6 bloques, 4 reusan el group del workflow (`light-benchmarks`, `high-density`, `analyze`, `critcmp-regression`). → corregido en `CI_POLICY.md` y en este task file; la fila `FIND-716` ahora manda borrar los **cuatro**, no dos. El diagnóstico sobrevive.
  - MEDIUM — la cita del pin inexistente es **HTTP 422 "No commit found for SHA"** (404 solo aplica a `/git/commits/`). → corregido.
  - LOW — comentario obsoleto del job `mutation-test` ("`--check`: only run mutants; `--timeout 120`: per-mutant cap") contradecía el bloque nuevo 8 líneas abajo. → eliminado.
  - LOW — la tabla de jobs de `CI_POLICY.md` omitía los 4 jobs de bench que la misma sección comenta. → añadidos.
- **Checklist anti-hábitos tóxicos:**
  - [x] No inventar salidas de comandos (todo error citado viene de un log descargado).
  - [x] No saltarse la clarificación (no hubo decisión de producto; las 3 se resolvieron por evidencia).
  - [x] No declarar done sin verificar contra los acceptance criteria.
  - [x] No ignorar fallos (los rojos extra se reportan con repro, no se esconden).
  - [x] No un solo intento de búsqueda (404 del nombre viejo → archivo real → log → API de jobs → API de SHAs → código de cargo-mutants).
  - [x] No copiar sin citar (`mutants.rs/print.html`, `src/main.rs`, doc de concurrency, `gh api` SHAs).
  - [x] No reintentar en bucle sin diagnóstico.
  - [x] No huérfanos: cada step conectado a un veredicto.
  - [x] No degradar chequeos: `if-no-files-found` sube de `warn` a `error`.
  - [x] Parada explícita: los rojos de bench quedan como fila, no se tocan (appetite + decisión de costo del owner).
  - [x] Cobertura SDP: `ci-cd-and-automation` + `git-workflow-and-versioning` (pinned) + `systematic-debugging` cubren el dominio.
- **Veredicto:** ✅ approve tras aplicar los 4 hallazgos (la confianza media en el commit exacto de `memory_concurrency` queda declarada como deuda, no como hecho).

## Notas
- **`memory_concurrency` no se toca** a propósito: retirarlo habría sido el pre-mortem (a) — "arreglar" retirando. Está verde y es gate de concurrencia (Regla 8).
- **No se agregó `continue-on-error`** en ningún lado (Regla 2). El `|| true` del paso de mutants **se elimina** en este cambio: era el silenciador que habría convertido un comando inválido en verde.
- **No se re-habilitó** `Light benchmarks`/`High density`: el fix es borrar el bloque `concurrency:` job-level en los **cuatro** jobs que reusan el group del workflow (redundante con el workflow-level `cancel-in-progress: false`), pero eso reactiva dos lanes de bench de 2 h y 6 h — decisión de costo del owner, va como fila `FIND-716` con repro y fix exacto.
- El nombre `heavy-certification.yml` del backlog está muerto desde `a4c290914`; el comando de triage que traía la fila devolvía 404. Eso explica 9 días de rojo invisible tanto como los bugs.
