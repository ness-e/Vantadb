# FIND-142 — Quitar numeración de filenames de workflows

> **Plan:** docs/dev/plans/2026-09-21-workflows-repair.md (Wave 3, tras Waves 0-2 verdes)
> **Tipo:** refactor mecánico (rename) · **Estado:** 🟡 IN PROGRESS
> **SDP:** ci-cd-and-automation, git-workflow-and-versioning, shipping-and-launch, documentation-and-adrs (+ campaign_discover_skills_v2 BUILD base: campaign-executor, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, source-driven-development, api-and-interface-design — no aplican TDD/frontend; se usan las 4 de dominio)

## Objetivo

Quitar numeración (`ci-rust-10.yml`→`ci-rust.yml`, etc.) vía `git mv` + actualizar todas las referencias en el mismo commit. Contrato: actionlint 0 + checks requeridos resuelven con nuevos nombres + badges 200 + 0 refs rotas (grep en alcance).

## Restricción dura del lead (desvío justificado del plan, NO negociable)

NO renombrar estos 3 archivos — filenames pineados por configs externas de trusted publishing (OIDC): `release.yml` (crates.io Trusted Publisher exige ese filename), `release-npm-61.yml` y `release-npm-node.yml` (npm Trusted Publisher exige esos filenames). Excepción documentada en commit message + RESULTADO.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `.opencode/rules/release-ci.md` (42L), plan file Wave 3 (§65-67), `docs/workflow/{README,TRIGGERS,PUBLISH,RUNBOOK,FAQ}.md` (vía grep), listing 27 yml en `.github/workflows/`.
- **Tabla final de renames (14):** ci-rust-10→ci-rust, ci-examples-12→ci-examples, ci-web-11→ci-web, gate-docs-21→gate-docs, sec-codeql-30→sec-codeql, chaos-45→chaos, fuzz-40→fuzz, perf-bench-40→perf-bench, heavy-certification-50→heavy-certification, heavy-bench-nightly-51→heavy-bench-nightly, release-wheels-60→release-wheels, release-adapters-62→release-adapters, release-binaries-63→release-binaries, release-sbom-64→release-sbom. Evidencia de cita por nombre: TRIGGERS.md:23-59, README.md:27-83, FAQ.md:22-49, RUNBOOK.md:50-59, PUBLISH.md:27-74, CI_POLICY.md (15+ hits), README.md:8-10/137/217, README_ES.md:8-10/131/211.
- **Referencias entrantes (qué cita cada nombre):** badges README×2 (URLs actions/workflows/ + shields — SE ROMPEN con rename, fix obligatorio); `docs/workflow/*.md` (15 files con `related:` + cuerpo); CI_POLICY.md (mínimo: solo strings de filename); self-`paths:` dentro de 4 yml renombrados (chaos, ci-rust, ci-examples, release-wheels — string del propio filename, parte del rename, cero cambio semántico); `uses: ci-gate.yml` NO se toca (ci-gate sin número, no renombrado).
- **Referencias salientes:** ninguna nueva. Required checks de branch protection usan JOB names (no filenames) → rename no los rompe (evidencia: ruleset develop 11 checks, FIND-139).
- **Veredicto:** blast radius 14 yml + ~20 docs, pero cambio 100% mecánico (strings de filename), cero semántica YAML. Gate D: disparado por >10 archivos → GO por orden explícita del lead + plan Wave 3 + restricción de pins. Docs `docs/workflow/*.md` conservan sus filenames (solo contenido) para no romper links de master-index.md:277-290.

## Alcance (lead)

- SÍ: 14× `git mv`, self-paths en 4 yml, badges+links README.md + README_ES.md, `docs/workflow/*.md` (contenido), CI_POLICY.md (solo strings filename), task file.
- NO (prohibidos): contenido YAML más allá del rename, `src/`, `web/src/`, `desktop/`, `reparacion.bat`, `.opencode`, `completions/*`, `*.lock`, stash GOV-C4, `docs/dev/Backlog.md`, plan file (solo recitation), `docs/CHANGELOG.md`, `docs/api/openapi.yaml`, `docs/api/MCP.md`, `C:/Users/Eros/.vantadb*`, secretos. Histórico (archive/avance/historial/tasks viejas/ADRs/CHANGELOG) NO se reescribe. Fuera de alcance quedan stale en prosa (sin URLs rotas — verificado): TEST_MAP×2, ci-cd-guide×2, chaos-testing×2, BENCHMARKS, QUICKSTART, CONTRIBUTING, Formula, pyproject, web/guides, glosario, master-index (cita .md, no .yml) → deuda para orquestador.

## Steps

- [x] **S1 — git mv ×14:** renombrar los 14 yml. Verify: `git status --short` muestra 14 R + pins intactos. ✅
- [x] **S2 — self-paths ×4 + comentarios ×2:** `chaos.yml`, `ci-rust.yml`, `ci-examples.yml`, `release-wheels.yml`: `.github/workflows/<old>` → `<new>` en `paths:` (2 ocurrencias c/u); + `ci-rustdoc.yml:52`, `providers-ci.yml:53` (comentarios). Verify: grep 0 old-name en `.github/workflows/`. ✅ (más 3 citas cruzadas halladas en re-grep: bench-canonical:5, desktop:70, ocr-nightly:22 — también fijas)
- [x] **S3 — READMEs:** badges (3×2) + heavy-cert link + ci-examples mención en README.md + README_ES.md. Verify: grep 0 old-name en ambos. ✅
- [x] **S4 — docs/workflow ×19:** reemplazo mecánico de strings `<old>` → `<new>` (NO renombrar .md — master-index.md:277-290 cita los .md). Verify: grep 0 old-names en `docs/workflow/`. ✅
- [x] **S5 — CI_POLICY mínimo:** solo strings de filename viejo → nuevo (21/20 líneas). Verify: grep 0 old-names en el archivo. ✅
- [x] **S6 — verify + commit:** actionlint full exit 0 + `git diff --check` limpio + grep alcance 0 + commit `ci: FIND-142 — ...` selectivo, NO PUSH. ✅

## Contrato

- `actionlint` 0 en `.github/workflows/` (full, no solo tocados).
- Checks requeridos resuelven con nuevos nombres (job names intactos — cero cambio YAML semántico).
- Badges 200: NO verificable sin red (Internet N/A) → deuda declarada.
- 0 refs rotas (grep) en alcance: `.github/workflows/ + README.md + README_ES.md + docs/workflow/ + docs/user/operations/CI_POLICY.md`.

## Context Save Point

- Branch: develop (trunk-based, Regla 7). WIP ajeno: ninguno en `git status` (verificado S0 — status limpio salvo este task file untracked).
- Si se interrumpe: reanudar desde primer step ⬜; `git mv` es reversible pre-commit (`git reset --hard HEAD` solo si no hay WIP ajeno).
- NO PUSH (pushea solo vanta-lead).
