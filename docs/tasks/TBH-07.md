# TBH-07: Configure `cargo-mutants` weekly job in heavy-certification-50.yml

## Metadata
- **Plan file:** `docs/plans/2026-08-30-testing-bench-harden.md` (TASK-07, mutation-testing, Phase 2)
- **Creado:** 2026-08-31T00:00
- **last-synced:** 2026-08-31T00:00
- **Estado:** ⬜ PENDING

## Blast Radius
- **Callees (entrantes):** none — workflow files are leaf nodes; no Rust code references workflows
- **Callers (salientes):** `.github/workflows/heavy-certification-50.yml` is consumed only by GitHub Actions scheduler and `workflow_dispatch` manual trigger; no project code imports it
- **Implicaciones:**
  - Mutation testing is **slow** (full workspace can take >2h) — contract says `-p vantadb` only and `--timeout 120`
  - Adds a new CI job that runs weekly + manual only (NOT on PR) — **no fast-gate impact**
  - Surface "mutation score" via `benchmarks/mutation_score.json` artifact (best-effort; non-blocking)

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:**
  - `.github/workflows/heavy-certification-50.yml` (281 líneas) — verificado
  - `.github/actions/rust-setup/action.yml` (100 líneas) — confirma `taiki-e/install-action@43aecc8d...` SHA-pin pattern ya en uso
  - `docs/plans/2026-08-30-testing-bench-harden.md` líneas 88-93 (TASK-07 spec)
- **Referencias hacia dentro (este archivo es consumido por):**
  - GitHub Actions scheduler (cron `0 3 * * 0` en workflow)
  - `workflow_dispatch` manual trigger
  - `grep -r "heavy-certification-50" .github/` — no other workflow references this file
- **Referencias hacia fuera (este archivo importa):**
  - `.github/workflows/ci-gate.yml` (existing reusable workflow — used by ci-gate job)
  - `.github/actions/rust-setup` (composite — used by all 12 existing jobs)
  - NINGUNA referencia nueva (no new imports; mutation-test job standalone)
- **Veredicto:** IMPACTO BAJO. Aditivo (1 job nuevo, 0 jobs modificados). Riesgo: nuevo cargo subcommand (cargo-mutants) → verificar instalación. Patrón `taiki-e/install-action` ya validado en 3 workflows existentes (ci-rust-10, fuzz-40, release-sbom-64). NO TOCAR los 11 jobs existentes.

## Contrato (verificable mecánicamente)
1. `heavy-certification-50.yml` tiene un nuevo job `mutation-test`
2. El job instala `cargo-mutants` via `taiki-e/install-action` con SHA-pin (no `@v2.83.x`)
3. El job corre `cargo mutants --check -p vantadb --timeout 120`
4. Trigger: solo `workflow_dispatch` + `schedule weekly` (NO `pull_request` — mutation testing es muy lento para PR gate)
5. NO TOCAR los 11 jobs existentes en heavy-certification-50.yml
6. Artifact `benchmarks/mutation_score.json` se sube como output (best-effort, no bloquea)
7. Comando verify: `python -c "import yaml; yaml.safe_load(open('.github/workflows/heavy-certification-50.yml')); print('OK')"` → OK
8. `Select-String -Path heavy-certification-50.yml -Pattern "mutation-test|cargo-mutants"` → matchea
9. `actionlint heavy-certification-50.yml` → 0 errors

## SDP (Skill Discovery Protocol)
- **SKILLS_CARGADAS:** `ci-cd-and-automation` (referencia cargada vía skill tool), `campaign-executor` (auto)
- **SDP keywords:** "workflow", "GitHub Actions", "cargo-mutants", "mutation testing", "weekly schedule"
- **SDP note:** base-only + ci-cd-and-automation (única skill de CI/CD aplicable al área; el resto son Rust/Frontend/Bindings irrelevantes)

## Herramientas
- Read, Edit, Write
- bash (PowerShell 7): python yaml parse, actionlint, Select-String
- git commit / status

## Steps

### Step 1: Crear task file
- **Archivos:** `.opencode/skills/campaign-executor/tasks/TBH-07.md`
- **Acción:** Crear este archivo (already done)
- **Verify:** Test-Path del archivo = True
- **Estado:** ✅ DONE

### Step 2: Agregar job `mutation-test` a heavy-certification-50.yml
- **Archivos:** `.github/workflows/heavy-certification-50.yml`
- **Acción:** Insertar un nuevo job al final del bloque `jobs:` (después de `other-heavy`, línea 280). El job NO toca los 11 existentes. SIN `pull_request` trigger. Trigger flow-level ya tiene `workflow_dispatch` + `schedule weekly` (líneas 4-15). El job individual puede omitir trigger si hereda del workflow.
- **Verify:** `python -c "import yaml; yaml.safe_load(open(...))"` → OK
- **Estado:** ⬜ PENDING

### Step 3: Verify (YAML parse + actionlint + grep)
- **Archivos:** workflow
- **Acción:** Correr 3 verifies
- **Verify:**
  - `python -c "import yaml; yaml.safe_load(open(...)); print('OK')"` → OK
  - `actionlint heavy-certification-50.yml` → exit 0
  - `Select-String` para `mutation-test|cargo-mutants` → match
- **Estado:** ⬜ PENDING

### Step 4: Commit + update plan
- **Archivos:** `.github/workflows/heavy-certification-50.yml` + task file
- **Acción:** `git add` + `git commit -m "ci(TBH-07): add cargo-mutants weekly mutation-test job to heavy-certification workflow"` + update plan file
- **Verify:** `git log --oneline -1` muestra commit nuevo
- **Estado:** ⬜ PENDING

## Dependencias
- TBH-01 ✅ (agregó pre-test step en `other-heavy` — pero `mutation-test` es nuevo job, no toca `other-heavy`)
- Ninguna otra

## Notas
- **Ponytail reflex:** 1 job nuevo. NO reformatear el resto. NO renombrar jobs. NO tocar el orden de los 11 existentes.
- **mutation score ≥ 70% goal** (plan file Phase 2 checkpoint) — gate semántico, no bloqueante en workflow
- **Timeout per mutant 120s** — viene del contrato; mutants individuales que cuelguen se abortan
- **Scope `-p vantadb`** — solo crate raíz (risk table: cargo-mutants es lento en workspace completo)
- **Artifact:** `benchmarks/mutation_score.json` — output nativo de cargo-mutants desde `--output-json` o `mutants.json` (default en workspace)

## Context Save Point
- **Fecha:** 2026-08-31T00:00
- **Branch:** develop
- **CI pendiente:** TBH-07 job will be visible only on workflow_dispatch + weekly schedule
- **Decisiones:**
  - SHA-pin: usar `taiki-e/install-action@43aecc8d72668fbcfe75c31400bc4f890f1c5853 # v2.83.2` (el mismo que ya usa rust-setup) — TBH-13 cubrió pinning pero este SHA ya estaba pinned en otras 3 workflows
  - Cargo crate name: `cargo-mutants` (binario se invoca como `cargo mutants`); `taiki-e/install-action tool:` acepta ambos formatos (taiki-e docs), pero el patrón seguro es `tool: cargo-mutants`
  - Trigger: omitido en job (heredado del workflow `on:`). TBH-04 agregó `develop` a 6 workflows pero este ya tenía `workflow_dispatch` + `schedule` (sin push, sin PR) → cumple contract
- **Problemas conocidos:** ninguno
- **Próxima tarea:** Wave 4 — handoff al orquestador