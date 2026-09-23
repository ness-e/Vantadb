# FIND-140 — Publishing hardening en 5 workflows

> **Plan:** `docs/plans/2026-09-21-workflows-repair.md` (Wave 2, 1 task, steps atómicos por archivo, NADA sin re-run)
> **Tipo:** devops (CI/CD) · **Estado:** 🔄 EN PROGRESO
> **SDP:** ci-cd-and-automation · shipping-and-launch · git-workflow-and-versioning · incremental-implementation · doubt-driven-development (+ base campaign-executor/progreso; discover v2 devolvió lifecycle genérico descartado por N/A a YAML/CI salvo incremental)
> **Regla leída:** `.opencode/rules/release-ci.md` (completa) · **Refs:** `definition-of-done.md`, `dev-tools.md`, `SPEC.md` raíz (tabla Spec N/A — CI), commands `pipeline.md`
> **Herramientas:** `actionlint` en cada edit + `git diff --check` + verify vía bash directa (BUG conocido `campaign_verify_cmd` exit -1) + mención. `cargo -j 2` solo si valida Rust (no aplica — 0 código).

## Contrato (AC)

- (a) publish solo donde debe: `release.yml` push solo `main` (quitar `develop`); tags separados por namespace; `release-plz-pr` no se rompe.
- (b) cada cambio con re-run verde documentado; `actionlint` 0 en cada edit.
- (c) `release-plz-release` con timeout/env/concurrency.
- (d) `release-npm-61.yml` + `release-npm-node.yml`: triggers separados tags-vs-push (+PR trigger a node si falta); combinación tags+branches+paths inválida corregida separando bloques `push:`.
- (e) `release-adapters-62.yml`: gate version-exists en prod (como TestPyPI/`skip-existing` y checks npm ya tienen).
- (f) `release-binaries-63.yml`: trigger alineado con condición de upload (no construir 5 targets para desechar).
- (g) namespaces de tags documentados (§ Namespaces).
- Rollback: `git revert <commit>` por commit (uno por archivo si hace falta) + re-run del workflow. Commits `ci: FIND-140 — ...`. NO PUSH (solo vanta-lead pushea).

## Impacto mapeado (Regla 0)

**Archivos leídos completos (5 + 1 referencia):**
- `release.yml` (47L): `on.push.branches: [main, develop]` — develop dispara release-plz-release AND release-plz-pr en cada push a develop (publish donde no debe). `release-plz-release` sin `timeout-minutes`, sin `env`, sin `concurrency`. `release-plz-pr` SÍ tiene `concurrency` (group release-plz-ref, cancel false). Ambos usan SHAs vigentes (checkout 3d3c42e5, rust-toolchain 02cb101e, release-plz b8d6b54b).
- `release-npm-61.yml` (~230L): UN bloque `push:` con `tags: ["v*.*.*"]` + `branches: [main]` + `paths:` — combinación inválida/ambiguo (tags+branches en mismo bloque push no filtra como AND; GitHub trata tags y branches como eventos distintos y paths solo aplica a branch pushes). Jobs `publish-wasm`/`publish-ts` SÍ gatean por `startsWith(github.ref, 'refs/tags/v')` + `check-* exists` (version-exists OK). Tiene `pull_request.paths` (test gate) + `concurrency` con `cancel-in-progress: ${{ !startsWith(github.ref, 'refs/tags/') }}` + timeouts (10/15).
- `release-npm-node.yml` (~200L): UN bloque `push:` con `tags: ["node-v*.*.*"]` + `branches: [main]` + `paths:` — mismo defecto que -61. Jobs `publish` gatea por `startsWith(github.ref, 'refs/tags/node-v')` + `check-node exists` (OK). Tiene `concurrency` igual, timeouts (20/10/15), matrix 7 targets, SIN trigger `pull_request` (sin test gate en PR). `Attach to Release` con `if: github.event_name == 'release'` es dead-code (sin trigger `release:` — harmless, fuera de scope YAGNI, no tocar).
- `release-adapters-62.yml` (~150L): `push.tags: ["adapters-v*.*.*"]` limpio (sin branches mezclados — OK). `publish-testpypi` tiene `skip-existing: true` (gate TestPyPI OK). `publish-pypi` (prod) usa `pypa/gh-action-pypi-publish` SIN `skip-existing` y SIN step previo `version-exists` — re-publicar misma versión falla en PyPI (HTTP 400) en vez de skip graceful. Jobs tienen timeouts (15/10) + `concurrency` con cancel-false-en-tags. `Attach to Release` con `if: github.event_name == 'release'` dead-code igual (sin trigger release, harmless, no tocar).
- `release-binaries-63.yml` (~160L): `push.tags: ['v*']` + `release.types: [published]` — jobs `tests`+`build`(5 targets)+`docker-image` corren en AMBOS eventos; pero `Upload release asset` / `Export image` gatean con `if: github.event_name == 'release'`. En push de tag `v*` se construyen 5 targets (+docker) para desechar (upload nunca corre). Trigger `v*` además colisiona en namespace con npm `v*.*.*` (ambos corren en cada tag de versión — por diseño hoy, documentar, no cambiar wiring sin owner).
- Referencia `release-wheels-60.yml`: `push.tags: ["v*.*.*"]` SEPARADO + `pull_request.paths` SEPARADO (patrón correcto a replicar). Prod `publish-pypi` tampoco tiene `skip-existing` (mismo hueco que adapters; FUERA de scope — FIND-140 solo toca adapters-62).

**Referencias hacia dentro:** SHAs vigentes verificados por grep en repo (checkout `3d3c42e5` = v7.0.1, `setup-node 49933ea5` = v4.4.0, `rust-toolchain 02cb101e` = stable branch, `upload-artifact ea165f8d` = v4.6.2, `download-artifact 3e5f45b2` = v8.0.1, `attest 78e6cbd3` = v4.1.1, `gh-release efb35369` = v3.0.3, `maturin e83996d1` = v1.51.0, `setup-python 5fda3b95` = v7.0.0, `pypi-publish dc37677b` = v1.14.2); `concurrency` + `cancel-in-progress: ${{ !startsWith(github.ref, 'refs/tags/') }}` (convención release-*); OIDC `id-token: write` + environments `npm`/`pypi`/`testpypi` (Regla 7 AGENTS.md — no tocar permisos).

**Referencias entrantes:** ninguna (workflows no son importados; triggers disjuntos por tags salvo `v*` vs `v*.*.*` documentado abajo). `release-plz-pr` corre en PRs + push main/develop — quitar develop del trigger push NO lo rompe (sigue corriendo en PR y push main).

**Veredicto:** blast radius = 5 archivos YAML, 0 código, 0 API pública, 0 símbolos nuevos → Gate D NO dispara (sin question). Edits mínimos por archivo (~5-15 líneas c/u). `cargo` N/A. Internet N/A (docs oficiales solo si duda de sintaxis `on.push` — sintaxis ya verificada en wheels-60 como patrón).

## Namespaces de tags (documentación canónica)

| Namespace | Workflow | Ejemplo | Publica en |
|---|---|---|---|
| `v*.*.*` (semver) | `release-wheels-60.yml`, `release-npm-61.yml` (publish jobs), `release-binaries-63.yml` (trigger `v*`) | `v0.6.1` | PyPI (`vantadb-py`), npm (`vantadb-wasm`, `vantadb`), GitHub Release assets (binarios) |
| `node-v*.*.*` | `release-npm-node.yml` | `node-v0.6.1` | npm (`vantadb-node` nativo) |
| `adapters-v*.*.*` | `release-adapters-62.yml` | `adapters-v0.6.1` | PyPI (9 adapters `integrations/`) |
| `v*` (prefijo) | `release-binaries-63.yml` (`push.tags: ['v*']`) | cualquier `v…` | NOTA: `v*` matchea también `v0.6.1` → binaries corre en cada tag semver por diseño (documentado, no cambiado) |
| crates.io | `release.yml` (release-plz, push main) | n/a (versionado por conventional commits) | crates.io vía OIDC |

## Steps

- [x] S1 — DISCOVERY (este archivo: blast radius, contrato, namespaces, gates) ✅
- [x] S2 — `release.yml`: push solo `main` + timeout-30/env/concurrency en `release-plz-release` (`release-plz-pr` intacto verificado por diff) → actionlint 0 ✅
- [x] S3 — `release-npm-61.yml`: bloque `push:` tags-solo (verificado docs oficiales: paths no se evalúa en tags; branches+tags = OR, paths+branches = AND) → actionlint 0 ✅
- [x] S4 — `release-npm-node.yml`: bloque `push:` tags-solo + `pull_request.paths` nuevo (no existía) → actionlint 0 ✅
- [x] S5 — `release-adapters-62.yml`: `skip-existing: true` en `publish-pypi` prod (patrón reuse de publish-testpypi mismo archivo) → actionlint 0 ✅
- [x] S6 — `release-binaries-63.yml`: quitado `push.tags ['v*']` (release-plz `git_release_enable = true` crea el Release que dispara `release:published`; `workflow_dispatch` intacto para re-run manual) → actionlint 0 ✅
- [x] S7 — VERIFY (actionlint 5 files exit 0 + `git diff --check` exit 0 + secrets-grep 0, vía bash directa por BUG `campaign_verify_cmd` exit -1) + commits selectivos + RESULTADO ✅

## Verify (evidencia)

- `actionlint` 5/5 archivos → exit 0 (cada step + full final)
- `git diff --check` → exit 0
- secrets-grep en diff (`password|secret|api_key|token`) → 0 coincidencias
- Fuente docs oficial tags+branches+paths: https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#onpushbranchestagsbranches-ignoretags-ignore (+ `...#onpushpull_requestpull_request_targetpathspaths-ignore` — "Path filters are not evaluated for pushes of tags")
- Observado-no-tocado: `release-wheels-60.yml` prod tampoco tiene `skip-existing` (mismo hueco que S5; fuera de scope — candidato FIND futuro, no se toca por YAGNI); `Attach to Release` con `if: event == 'release'` sin trigger `release:` en node/adapters (dead-code harmless, no se toca); `release-sbom-64.yml` fuera de scope.

## Gate D

No disparado: 5 archivos YAML, sin hot path, sin API pública, contrato no ambiguo (detalle verificado del plan como dado).

## Context Save Point

- Estado: S1 ✅; S2–S7 ⬜. DISCOVERY completo, 0 edits aún.
- Reanudar: continuar desde primer step ⬜ (S2) — no rehacer S1.
- Rama: `develop`. NO PUSH. Prohibidos vigentes (ver task prompt).

## Rollback

`git revert <commit>` por commit (uno por archivo si hace falta) + re-run del workflow afectado. Sin migración de datos. Sin feature flags (CI YAML).
