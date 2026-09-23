# FIND-136 — Cobertura + caché en 4 workflows

> **Plan:** `docs/plans/2026-09-21-workflows-repair.md` (Wave 0, paralelo ×3, archivos disjuntos)
> **Tipo:** devops (CI/CD) · **Estado:** ✅ COMPLETO
> **SDP:** ci-cd-and-automation · git-workflow-and-versioning (+ base campaign-executor/progreso; discover v2 sin keyword-mapped extras, lifecycle genérico descartado por N/A a YAML/CI)
> **Regla leída:** `.opencode/rules/release-ci.md` (completa) · **Ref:** `definition-of-done.md`

## Contrato (AC)

- (a) cambio en `providers/` dispara clippy/tests (no solo rustdoc)
- (b) sin `|| true` que oculte fallos
- (c) pip/Rust con caché donde se instala/compila
- (d) `actionlint` exit 0

## Impacto mapeado (Regla 0)

**Archivos leídos completos (4, solo `.github/workflows/`):**
- `providers-ci.yml` (78L): push/PR solo `paths`, sin `branches` → corre hasta en tags; toolchain `02cb101e` = resto; pip `cache: pip` ✓; sin clippy; sin rust-cache (compila vía maturin)
- `chaos-45.yml` (46L): push `branches: [main, develop]` + paths sin `vantadb-*/**` ni `vanta-memory/**`; sin `concurrency`; usa rust-setup (caché ✓)
- `ci-examples-12.yml` (147L): `setup-python@5fda3b9` L99-102 sin `cache:`; usa rust-setup (caché ✓); `concurrency` ✓
- `adapters-compat.yml` (127L): pip `cache: pip` ✓ L104; toolchain directo sin rust-cache; fallback `|| true` L120 oculta fallo de pin de versión

**Referencias hacia dentro:** `Swatinem/rust-cache@7e35be21…# v2.9.1` (SHA vigente verificado en fuzz-40, ci-rust-10:238/482, release-wheels-60); `setup-python@5fda3b9…# v7.0.0` + `cache: pip` (patrón en providers-ci:39, adapters-compat:104); `branches: [main, develop]` push / `[main]` PR (convención chaos-45, ci-examples-12, ci-rust-10); `concurrency` + `cancel-in-progress: true` (convención repo).

**Referencias entrantes:** ninguna (workflows no son importados; triggers disjuntos por paths).

**Veredicto:** blast radius = 4 archivos YAML, 0 código, 0 API pública, 0 símbolos nuevos → Gate D NO dispara (sin question). Toolchain: `providers-ci:42` ya es `02cb101e` = resto; `fa04a145` vive solo dentro de `rust-setup/action.yml:62` (composite, fuera de scope) → **no-op verificado, sin cambio**.

**Hallazgo clave (a):** `ci-rust-10.yml:6-37` paths NO incluyen `providers/**` (PROHIBIDO tocarlo) → el clippy de providers se añade como step en `providers-ci.yml` (patrón espejo de `ci-rust-10:438-444` con `cargo clippy -- -D warnings`).

## Steps

- [x] S1 — DISCOVERY (blast radius, SHAs vigentes, convenciones) ✅
- [x] S2 — `providers-ci.yml`: `branches` push/PR + step clippy 3 providers + rust-cache ✅
- [x] S3 — `chaos-45.yml`: paths `+vantadb-*/**`, `+vanta-memory/**` (push+PR) + `concurrency` ✅
- [x] S4 — `ci-examples-12.yml`: `cache: pip` en setup-python L99-102 ✅
- [x] S5 — `adapters-compat.yml`: rust-cache tras toolchain + quitar `|| true` L120 ✅
- [x] S6 — VERIFY (actionlint exit 0 + `git diff --check` exit 0, vía bash directa por BUG campaign_verify_cmd) + commit selectivo + RESULTADO ✅

## Verify (evidencia)

- `git diff --check` → exit 0
- `actionlint` 4 archivos → exit 0
- secrets-grep en diff → 0 coincidencias

## Gate D

No disparado: ≤4 archivos YAML, sin hot path, sin API pública, contrato no ambiguo.

## Riesgo conocido (S5)

Tras quitar `|| true`, si ningún patrón (`<adapter>-core==ver`, `<adapter>==ver`, `<ver>`) resuelve en PyPI para algún adapter (p. ej. `llamaindex`), el step falla en vez de pasar en silencio — ese es el comportamiento buscado por (b); si ocurre, nace FIND de seguimiento, no se revierte el fix.

## Rollback

`git revert <commit>` por archivo (commits atómicos por workflow si hace falta) + re-run del workflow. Sin migración de datos.
