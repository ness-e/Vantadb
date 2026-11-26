---
title: "Avance — CI/CD & Release"
kind: review
status: active
description: ".github/workflows/providers-ci.yml, un workflow que no existe (fue consolidado en"
tags: [vantadb, avance, ci, cd, release, github-actions]
---

# Avance — CI/CD & Release

> Registro consolidado del trabajo completado sobre el pipeline: GitHub Actions, quality gates, releases, wheels, changelog. IDs originales conservados.

## Campañas de ingeniería de salud (P1–P8)

### P1 — Engineering Health Wave 0 (2026-07-17)
| ID | Tarea | Resultado |
|---|---|---|
| P1-2 | Timeout tests Windows 25→30 min | ✅ `ci-rust-10.yml` `test-windows` step timeout 30m; `test-threads=2` preservado (evita OS error 1455). Commit `3acd07c`. |
| P1-3 | Clave de cache GloVe → hashFiles | ✅ cache `glove-100d-v1` → `hashFiles('scripts/download_benchmark_datasets.sh')` en jobs `test` y `coverage`. Commit `9386079`. |
| P1-4 | macOS unificar con action rust-setup | ✅ Reemplazado dtolnay + Swatinem + cargo-nextest manual por `./.github/actions/rust-setup`. −10 líneas. Commit `8bd15fa`. |
| P1-5 | Re-activar wasm-opt | ✅ Eliminado override `wasm-opt = false`; Binaryen v128+ soporta bulk-memory-opt; corre con `-Os`. |
| P1-6 | Título CI-Rust en checks | ✅ workflow `name: CI-Rust`. |
| P1-7 | Doc: retirada fue not bug | ✅ documentado. |

### P4 — Engineering Health Wave 1 (2026-07-25)
| ID | Tarea | Resultado |
|---|---|---|
| WEB-03 | Async WAL batching fsyncs | ✅ `c59e0f80` — flush_all fsync paralelo por shard, short-circuit shard único. 25/25 tests. |
| WEB-04 | Storage format versioning | ✅ `21432104` — `VantaHeader::validate_compat()` check por rangos para VantaFile/HNSW/WAL. |

### P8 — Engineering Health Wave 2 (2026-07-28/08-03)
- P8-01: ci-rust-1.yml → **separación nextest de integration tests** (2 jobs: unit; integration con `nextest partition`). ✅
- P8-02: build single shared: **cache-limit + build profile persist** (shared-cache key `cargo-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}`). ✅
- P8-03: dependabot **semver-minor/patch + auto-merge label** `dependencies`/`auto-merge` con review 0 y checks (build/test). ✅
- P8-04: cron **weekly security check** (cargo-deny check advisories). ✅
- P8-05: MCP Linux/macOS gate. ✅
- P8-06: sec-ffi audit (SEC-01/02) milestone pass. ✅
- P8-07: release-plz publish draft + prerelease flag. ✅

> Los pasos de CI catalogados P10 que no se adoptaron por decisión: ver `decisiones/wontfix.md` → sección CI/CD deferidos (NIGHTLY benchmarks, self-hosted runners, matrix OS, coverage window auto, benchmark CI failure auto-window).

### CI-01: Arreglar todos los workflows de GitHub Actions
- **Fecha:** 2026-07-28
- **Resultado:** ✅ (Batch CI `5652a9f` + P8 waves) — ver inventario de workflows abajo.

### REVIEW-02: Clear stale `--ignore RUSTSEC-2026-0176/0177` audit flags
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Limpiados flags `--ignore` obsoletos en audit (advisories ya resueltos).

### REVIEW-03: Verificar política `continue-on-error` en CI
- **Fecha:** 2026-08-06
- **Resultado:** ✅ Política `continue-on-error` revisada/verificada en workflows.

### REVIEW-05: Deps muertas en web/ eliminadas (prismjs + sharp)
- **Fecha:** 2026-08-06
- **Resultado:** ✅ `prismjs` + `sharp` removidos de `web/` (deps muertas).

### REVIEW-06: OOM rustc en cargo test --workspace — fix [profile.test]
- **Fecha:** 2026-08-24
- **Objetivo:** Eliminar OOM del compilador al compilar tests del workspace (17+ crates en paralelo con debug info completa).
- **Resultado:** ✅ Fix commiteado por lead en `167a8d4c` (`[profile.test]` debug=1/opt-level=0 + `[build] jobs=2` en `.cargo/config.toml`); verificado por vanta-tuner: `cargo nextest run -p vantadb --profile audit` compila y ejecuta 2055 tests sin OOM (2052 pass, 3 fail = tests nuevos de MOD-02, colateral), `cargo check --workspace` 42.84s sin OOM.


### TSYS-01..16: Mejoras task-system (plan 2026-08-11-residuo-consolidado)
- **Fecha:** 2026-08-11
- **Resultado:** ✅ TSYS-01..05, 07..11, 13..16 implementados (14/16; TSYS-06 runner DEFER, TSYS-12 runtime opcional NO gate-CI). Commits: 8f774c18 (T12/T14/T15/T16), d9f2a4cb (T10/T11/T13), 138d8735 (TSYS-14/15/16), TSYS-09/ADR-0017. Ver `docs/progreso/README.md` sección migradas.

## Workflows existentes (inventario 2026-08-03)

| Workflow | Propósito | Estado |
|---|---|---|
| `ci-rust-1.yml` | Fast Gate Rust (<5 min) | ✅ |
| `ci-rust-10.yml` | Full Rust (Windows/macOS) | ✅ |
| `ci-wasm.yml` | WASM build + TS SDK | ✅ |
| `ci-docs.yml` | docs site | ✅ |
| `release-plz.yml` | release PR + publish | ✅ (P8-07) |
| `ci-dependabot.yml` | dependabot auto-merge | ✅ (P8-03) |
| `ci-security-weekly.yml` | cargo-deny advisories semanal | ✅ (P8-04) |
| `ci-mcp.yml` | MCP gate Linux/macOS | ✅ (P8-05) |

## Batch CI (6 errores) — commit `5652a9f`
| ID | Tarea |
|---|---|
| CODE-044 | Cargo_test.toml stale |
| CODE-049 | Clippy deprecation warnings |
| CODE-050 | Doc test ci-rust stale |
| CODE-051 | deny.toml stale ignore |
| CODE-058 | Ignored advisories sin rationale |
| CODE-066 | workflow name fallback |

## Changelog & release discipline

- `release-plz` con Conventional Commits: `feat:`→minor, `fix:`→patch, `docs:/test:/perf:/refactor:`→patch, `feat!:`/`BREAKING CHANGE:`→major, `ci:/chore:`→no release.
- **NUNCA** tocar versión en Cargo.toml/CHANGELOG/tags manualmente (Regla 7, AGENTS.md).
- Flujo: `develop → commit → PR → merge a main → release-plz → Release PR → merge → publish`.

### REV-001: Release notes generation
- **Fecha:** 2026-07-23
- **Resultado:** ✅ `scripts/release_notes.py` auto-genera release notes desde conventional commits; `docs/RELEASES/` publicado.

### REV-002: Changelog stale
- **Resultado:** ✅ Comprobación de que el changelog no está stale antes de merge; gate en CI.

## Verificación

- `cargo check -p vantadb` ✅ en cada wave.
- CI Fast Gate <5 min vs Heavy Certification hasta 2h (separados por diseño).
- ERR-009 (job Miri en CI cubierto) — migrado 2026-08-12 (ver docs/progreso/README.md)
- COV-004 (ADR-0018 coverage gate = root crate vantadb ≥80%, supersede ADR-0015) — migrado 2026-08-12 (ver docs/progreso/README.md)

### CI-04: CodeQL multi-lenguaje (rust + python + javascript-typescript) — migrado 2026-08-12 (ver docs/progreso/README.md)
- **Resultado:** ✅ `sec-codeql-30.yml` `languages: rust` → `rust, python, javascript-typescript`; timeout 30→45 min. Sin tocar queries (suite default del codeql-action). actionlint exit 0. Commits `202af1f6`, `6477aa87`.

### CI-03: SBOM multi-ecosistema (rust + npm + python) — migrado 2026-08-12 (ver docs/progreso/README.md)
- **Resultado:** ✅ `release-sbom-64.yml` genera 3 artifacts: `sbom.json` (cargo-cyclonedx, existente), `sbom-web.json` (`npx @cyclonedx/cyclonedx-npm --package-lock-only`), `sbom-python.json` (`cyclonedx-py requirements - --pyproject`). Docs sincronizadas (Regla 3): `docs/dev/workflow/release-sbom-64.md`, `docs/ci-cd-guide.md`. actionlint exit 0 + pre-commit hook ok. Commit `a8735174`.

### CI-02: Fuzzing en PRs (gate acotado) — migrado 2026-08-12 (ver docs/progreso/README.md)
- **Resultado:** ✅ `fuzz-40.yml` agrega job `fuzz-pr` en `pull_request` (timeout 15 min, ubuntu-only, `-max_total_time=75` × 4 targets ≈ 5-8 min wall-clock, paths `src/**`+`fuzz/**`); fuzz semanal completo con `if: github.event_name != 'pull_request'`. actionlint exit 0. Commit `1c8029f1`.

### CI-05: Benchmark baseline fijo — migrado 2026-08-12 (ver docs/progreso/README.md)
- **Resultado:** ✅ `perf-bench-40.yml` corre bench 3× y compara mediana contra `benchmarks/python_baseline.json` versionado; regresión >15% → job falla; rebaseline manual vía `workflow_dispatch` `update_baseline=true`. Gate no-op hasta rebaseline (baseline inicial vacío). actionlint exit 0 + test sintético 3 caminos. Commits `adec84e7`, `56ebc126`, `9026000b`.

### CI-06: Tests gate en release workflows — migrado 2026-08-12 (ver docs/progreso/README.md)
- **Resultado:** ✅ `release-binaries-63.yml` y `release-npm-61.yml` agregan job `tests` (cargo nextest --profile audit / wasm-pack + npm test) como `needs` del publish. actionlint exit 0. Commits `3ca9e3e0`, `720bb7ab`.

### CI-07: SHA pinning de acciones — migrado 2026-08-12 (ver docs/progreso/README.md)
- **Resultado:** ✅ 67 refs tag/branch → SHA de 40 hex en 16 workflows (API GitHub + `git ls-remote`); 0 uses de terceros sin SHA restantes; 34 internos `./.github/...` sin pinear (correcto). Fix de ref muerta `release-plz@release-plz-v0.3.160` → v0.5.131. actionlint exit 0. Commits `faec5826`, `73bbf6e1`, `97c21d81`, `b84e4186`, `117c1ac4`.

### CI-01 (pre-commit-config): Registrar prettier + ruff + cargo fmt en pre-commit — verificado 2026-08-14 (ver docs/progreso/README.md)
- **Resultado:** ✅ `.pre-commit-config.yaml` con los 3 formatters (cargo-fmt local, ruff scoped `vantadb-python/`, prettier scoped `web/` rev v3.1.0). Commit `501758a3`. Fila stale del backlog eliminada.

### AUD-026: Dropped cli/arrow/tantivy from native DLL default features — migrado 2026-08-14 (ver docs/progreso/README.md)
- **Resultado:** ✅ `vantadb-node/Cargo.toml:24` — `vantadb = { path = "..", default-features = false, features = ["fjall", "memmap2", "rayon"] }`; único cdylib que arrastraba cli/arrow/tantivy (6.7MiB debug). `cargo check --manifest-path vantadb-node/Cargo.toml` ✅ + `cargo tree -e features` limpio. Commit `404f1625`.

- AUD-027: Least-privilege per-job permissions in release workflow — migrado 2026-08-14 (ver docs/progreso/README.md)

### AUD-047: binario release con feature `server` (2026-08-18)
- **Resultado:** ✅ `release-binaries-63.yml`: `vanta-cli` se compila con `--features "server,$ALLOC_FEATURES"` (default queda lean; `cargo install` activa solo default → nunca entraba server). README/README_ES: release binario incluye HTTP; source installs `cargo install --git ... --bin vanta-cli --features server`. Verificado runtime: `vanta-cli server --http` + `/health` OK. Commit `4ac3b9fa`. (ver docs/progreso/README.md)
- **Resultado:** ✅ Permisos movidos de workflow-level a por-job en `release-plz.yml` (release: `contents: write, pull-requests: read, id-token: write`; PR: `contents: write, pull-requests: write`); Trusted Publishing intacto, sin `CARGO_REGISTRY_TOKEN`; pin `release-plz/action@2eb1d8bcb7 # v0.5.131` confirmado correcto (tag del action vs CLI 0.3.160). actionlint exit 0. Commit `d66b267d`.

### R2: Crear agente vanta-research (read-only research subagent) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `.opencode/agents/vanta-research.md` (nuevo): mode subagent, tools read-only + web (edit/bash deny), skills coordinated-web-search/source-driven-development/progreso; 7 secciones idénticas a los 9 agentes. Contrato grep exit 0. Commit `2b4cbd6b`.

### R7: Corregir comandos de verificación rotos en Output Templates — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `vanta-worker.md:102` `cargo check -p vantadb_py` (package real; learning AUD-039) + `vanta-docs.md:102` `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py`. Commit `5bda5662`.

### FND-09: Regla 8 — Concurrencia paranoica en PRs — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ Regla 8 en `.opencode/AGENTS.md` (paths multi-índice/dashmap/parking_lot/Tokio → auditoría deadlocks/data races, carga 10k w/s + 1k r/s, delegación vanta-chaos/vanta-review) + referencia en `vanta-worker.md` L104. Commit `c34a0dc8`.

### FND-17: API reference automatizada (docs-as-code) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ análisis + plan en `docs/dev/research/FND-17-api-reference-docs-as-code.md`: Fase 1 cargo doc en CI (sin deps), defer typedoc/pydoc/site justificado. Citas archivo:línea + URLs verificadas. Commit `5dc71f0d`.

### R1: Skills obligatorias en §6 de los 9 agentes — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ línea "> **OBLIGATORIO:** … cargá con skill <nombre> …" al inicio de §6 en los 9 agentes. Commit `ec7f947a`.

### R3: Delegar fase DISCOVERY a vanta-research — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `commands/pipeline.md` + `task.md` Phase 2-3 referencian fork a vanta-research para tareas 🟡/🔴 (híbrido, el lead arma el task file con el digest). Commit `1885f64e`.

### R5: Sync §6 ↔ `campaign_load_skills` — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ §6 sincronizado con `campaign_load_skills` en los 9 agentes; 0 refs desfasadas (grep verificado). Commit `ec7f947a`.

### R6: Routing table + manual con vanta-research — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ fila "Research/Discovery → vanta-research" en `vanta-lead.md` §8 + `.opencode/VANTADB-OPERATING-MANUAL.md` actualizado. Commit `7c21c8a4`.

### R8: Eliminar referencia colgante a skill `typescript-expert` — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `vanta-worker.md:125` `typescript-expert` → `source-driven-development`. Commit `ec7f947a`.

### R9: Alinear bloques `permission:` con tablas MCP ❌/✅ — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ permission blocks de los 9 agentes denegan los servers ❌ de su tabla MCP (deuda TSYS-11 saldada). Commit `ec7f947a`.

### R10: Consolidar bloque §7 duplicado en reference compartido — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `.opencode/references/task-system.md` creado (patrón `definition-of-done.md`) + §7 = 1 línea por agente. Commit `ec7f947a`.

### FND-03: Aislamiento de features Cargo + compile matrix — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ feature set mínimo compila (`--no-default-features --features fjall`) + wheels empaquetan set mínimo; compile matrix CI verde. Commit `71c58753`.

### FND-10: Regla 9 — No optimizar sin medir + benchmark canónico P99 — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ Regla 9 en `.opencode/AGENTS.md` + `benches/canonical_p99.rs` ejecutable con baseline **3.07ms p99** (`docs/user/operations/BENCHMARKS.md`). Commit `89943c7d`.

### FND-11: No mergear código IA sin poder explicarlo (AI Guardian) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ Regla 10 (AI Guardian) en `.opencode/AGENTS.md` + referenciada en workflow de PR. Commit `3b0d2a3b`.

### FND-12: ADRs como forcing function (escrito por humano, no IA) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ Regla 5 reforzada en `.opencode/AGENTS.md` con formato mínimo (Contexto/Decisión/Consecuencias — quién articula). Commit `3b0d2a3b`.

### FND-13: Benchmarks honestos (extiende FND-10) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ Regla 11 en `.opencode/AGENTS.md` (claims citan benchmark reproducible + números) + claims README alineados. Commit `d61a006c`.

### FND-14: Ritual de inicio — validación de feature stack — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ paso 5 del Ritual de Inicio en `.opencode/AGENTS.md` (`cargo check --no-default-features --features fjall`). Commit `3b0d2a3b`.

### FND-16: Multi-target CI (wheels + WASM por PR) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ plan multi-target CI implementado: job wasm/TS por PR con paths filter + fix path CONTRIBUTING + dictamen P2-01 en FND-02. Commits `0f15a817` + `fb878cba`.

---

## 2026-08-27: Backlog Pipeline — Quick Wins (P25/P42/P41/P47)

### STABLE-00: Checklist y ADR de promoción a default-members (gate 100% estable)
- **Fecha:** 2026-08-27
- **Plan:** `2026-08-27-backlog-pipeline` Task 7 · P47 · `vanta-docs`
- **Objetivo:** Sin ADR de criterios no hay definición de "100% estable" → promoción ad-hoc sin trace; `Cargo.toml:636` deja `server/mcp/wasm/memory/proxy` fuera del Fast Gate.
- **Resultado:** ✅ `docs/dev/architecture/adr/ADR-0031-default-members-promotion.md` (nuevo, 205L) `status: proposed`: Context (Cargo:636), §1 tabla 10 checks (check/fmt/clippy, nextest/vitest, deny, docs-coverage, workflow timeout/continue-on-error, cargo package, wasm-pack/wasm32, napi 7-target, verify <5min, ADR reversible) en 3 corridas limpias, §2 cost table per crate (vantadb ~22s / python 12s / vanta-memory 36s / vanta-proxy 32s / server 21s / mcp 6.7s / wasm 3.6s / ts ~26s / node 8s), §3 Reversibilidad 1 línea `git revert Cargo.toml:636`, §4 Question to Owner A (<5 hard) vs B (<5 soft → ~8min) pending `Owner:___ Date:___ Choice:[ ]A [ ]B` (bloquea STABLE-09). `docs/dev/operations/CI_POLICY.md` §Promotion to default-members añadida (4 hits ADR-0031, 6 hits default-members, 10 checks). Verify: fmt ✅ + clippy ✅ + docs-coverage 0 gaps. Commit `fa5f04f0`.
- **Gates:** D: no-disparado (docs-only) · V: no-disparado · C: no-disparado
- **Contrato:** `Test-Path ADR-0031` True + `Select-String \| [0-9]` 10 rows + `Question to Owner` hit + `grep ADR-0031 CI_POLICY` 4 hits + `grep default-members CI_POLICY` 6 hits + `cargo fmt --check` ✅ + `clippy` ✅ + `docs-coverage` 0 gaps
- **Archivos:** `docs/dev/architecture/adr/ADR-0031-default-members-promotion.md`, `docs/dev/operations/CI_POLICY.md`, `.opencode/skills/campaign-executor/tasks/STABLE-00.md`

---

## 2026-08-31: Testing & Benchmarking Hardening (P48, fase 1)

### TBH-01: verify_datasets.{sh,ps1} + CI pre-test gate (heavy-certification)
- **Fecha:** 2026-08-31
- **Plan:** `docs/dev/plans/2026-08-30-testing-bench-harden.md` Fase 1 (ALTA)
- **Objetivo:** Auditoría multi-agente del 2026-08-30 (`ses_fabf69692ffeP5c7mycKcsGSV0`) identificó que los tests de certificación en `tests/certification/*` y `tests/benchmark_datasets.rs` skippean silenciosamente cuando los datasets no están descargados, permitiendo que Recall@10 degrade sin que CI se queje. Top #1 gap (ALTA).
- **Resultado:** ✅ Dos scripts (bash + PowerShell) con contrato idéntico + gate inyectado en `heavy-certification-50.yml` como pre-test step. Verifica 5 datasets canónicos: SIFT-1M (datasets/sift/*), GloVe-100 + GloVe-300 (data/benchmark/glove.6B.*.txt), SIFT-128 euclidean subset + GloVe-100 angular subset (data/benchmark/{sift-128,glove-100-angular}/*). Tabla human-readable por defecto; flag `--json` / `-Json` para CI annotation. Exit 0 cuando todos los paths existen, exit 1 con tabla diagnóstica cuando ≥1 falta.
- **Whitelist #[ignore]:** Tests con `#[ignore]` que NO son por dataset (Miri/FFI/croaring ×18 + tests con razones no-dataset) NO entran al gate — el gate solo chequea paths en disco, no enumera tests. Documentado en el task file.
- **Verify local (run 2026-08-31):**
  - `bash scripts/verify_datasets.sh` con glove-100-angular subset presente (touch) → exit 0 ✅
  - `bash scripts/verify_datasets.sh` con glove-100-angular subset borrado → exit 1, tabla correcta ✅
  - `pwsh scripts/verify_datasets.ps1` (PowerShell 7) → mismo contrato ✅
  - `bash scripts/verify_datasets.sh --json` / `pwsh ... -Json` → JSON parseable ✅
  - `cargo fmt --check` → 0 ✅
  - `cargo check --workspace --benches` → 0 ✅
  - `cargo check --workspace --benches --tests` → falla en `vantadb-mcp/tests/context_tests.rs` (pre-existing, falta `heat`+`superseded_by` en `MemoryRecord` — NO introducido por TBH-01, fuera de scope)
  - YAML sintaxis (`yaml.safe_load`) → parsea, step "Verify required benchmark datasets (TBH-01)" presente en `other-heavy` job ✅
  - Pre-commit hook: `actionlint` OK
- **Commit:** `0e67f354` — `feat(TBH-01): verify_datasets.ps1 + heavy-certification pre-test gate` (3 files, +92)
- **Archivos tocados:** `scripts/verify_datasets.sh` (mode 100644→100755), `scripts/verify_datasets.ps1` (nuevo, 86L), `.github/workflows/heavy-certification-50.yml` (+6 lines: nuevo pre-test step), `.opencode/skills/campaign-executor/tasks/TBH-01.md` (task file)
- **Próximo:** TBH-02 (initialize `benchmarks/criterion_baseline.json`)
- **Lecciones (memoria):** `verify_datasets pre-test gate | listar paths esperados via test -e (bash) / Test-Path (pwsh); exit 1 si MISSING; el whitelist #[ignore] por dataset NO requiere lógica porque el gate solo chequea paths en disco`


### GOV-TK4: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** GOV-TK4 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** 31b0902d
- **Dominio:** ci-cd

### RES-11: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** RES-11 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** 25792e30
- **Dominio:** ci-cd

### REVIEW-07: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** REVIEW-07 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** verify-only
- **Dominio:** ci-cd

### STABLE-01: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** STABLE-01 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** 3fa87560
- **Dominio:** ci-cd

### STABLE-08: sync 2026-09-01 (drift backlog)
- **Fecha:** 2026-09-01
- **Objetivo:** STABLE-08 completada previamente, removida del Backlog por drift (task file COMPLETED)
- **Resultado:** OK
- **Commit:** 142e8f1d
- **Dominio:** ci-cd

### ERR-CORE-02: clippy unwrap/expect deny en prod + anyhow bins (plan error-observability Wave 0)
- **Fecha:** 2026-09-02
- **Objetivo:** workspace.lints.clippy unwrap_used/expect_used deny (Cargo.toml, previo en HEAD 73f49e6f) + sweep collateral de allows justificados (~190 archivos tests/benches/examples + cfg_attr(test) en 5 lib.rs) + bins vanta-cli/vantadb-server a anyhow::Result con .context(); anyhow optional gated tras feature cli (lib sin anyhow)
- **Resultado:** OK - clippy -p vantadb y --workspace --all-targets --all-features exit 0; fmt 0; contrato 5/5
- **Commit:** af0bb8b8
- **Dominio:** ci-cd

### MKT-18h: Wheels ARM64 Linux + SHA256 reales Homebrew (plan quality-gtm-wave Task 4)
- **Fecha:** 2026-09-03
- **Objetivo:** `release-wheels-60.yml` suma target `aarch64-unknown-linux-gnu` al matrix de maturin (cross container oficial manylinux_2_28-cross, patrón documentado); `Formula/vantadb.rb` con 4 SHA256 reales verificados (hash local de los tarballs v0.5.0 == sidecar CI) + remove `bin.install vantadb-mcp` (no existe en los tarballs); removido input muerto `musllinux` inexistente en maturin-action v1.51.0
- **Resultado:** OK - contrato 4/4: rg aarch64 ≥1, actionlint exit 0, 0 placeholders en Formula, cargo check -p vantadb_py exit 0. Verificación real del job aarch64 diferida a corrida CI (stop-condition plan)
- **Commit:** `ci(wheels): aarch64 linux + SHA real formula (MKT-18h)` (2026-09-03; el cambio de workflow cabalgó `2ab706ec` por race de worktree compartido — ver nota en plan)
- **Dominio:** ci-cd

- STABLE-06 (worker): gate npm TS medido - Resultado: 278/278 vitest (no 264), ~16-21s wall; eslint 1 error fixeado; job tests+lint+pack en release-npm-61.yml. Commit 7ff70b01 (2026-09-05).

- BND-09 (worker): targets linux musl verificados sin codigo - Resultado: contrato ya cumplido en HEAD via ed75cb0b (napi.targets package.json:39,41 + matriz CI release-npm-node.yml:44-46,50-52); toolchain local sin docker/cross/musl documentado, sin codigo forzado. Gated por BND-08 (pipeline verificado e9843100). Sin commit nuevo (2026-09-06).

### STABLE-07: validar matrix node 7 targets (plan 2026-09-07-followup Wave1)
- **Fecha:** 2026-09-07
- **Objetivo:** YAML 7/7 == napi 7/7, suite verde, pack incluye `.node`, 0 `continue-on-error` indebido, tiempo matrix medido.
- **Resultado:** ✅ `npm test` 35/35; `npm pack --dry-run` 6 files incluye `.node` 5.4MB; único COE con CATEGORY:INFORMATIONAL permitida; veredicto **Heavy justificado** (7 jobs LTO × 3 OS, wall ≈45min worst-case). Validación read-only, 0 código (solo task file).
- **Dominio:** ci-cd

### STABLE-05: validar wasm gates 1-8 (plan 2026-09-07-cleanup-gates Wave0)
- **Fecha:** 2026-09-08
- **Objetivo:** último paquete sin validar: toolchain + check + clippy + fmt + build bundler + test chrome + docs + deny.
- **Resultado:** ✅ check/clippy/fmt 0 · build 1m06s (wasm 1.6MB) · chrome headless 67/67 · WASM_PERSISTENCE.md sin drift · deny ok (warning no-fatal advisory-not-detected documentado). Ejecución SARL STRATEGY lead-inline (ejecutor dejó G1-G8 con evidencia y murió pre-RESULTADO; 2 aborts infra).
- **Commit:** 9d338a18

### BND-08: pipeline npm napi-rs verificado (plan 2026-09-08-backlog Wave1)
- **Fecha:** 2026-09-09
- **Objetivo:** reconciliar scope dry-run 2026-09-05 con contrato workflow + matrix 5 targets.
- **Resultado:** ✅ workflow `release-npm-node.yml` ya existía (223L, 7 targets ⊇ 5) + actionlint 0 + `npm pack --dry-run` 6 files + E404 (nunca publicado). PROHIBIDO publish real.
- **Commit:** df1baa2e (solo task file)

### STABLE-06: validar vantadb-ts gate npm Fast Gate (plan 2026-09-08-backlog Wave2)
- **Fecha:** 2026-09-09
- **Objetivo:** último gate npm sin validar tras STABLE-05; "264 tests" confirmada stale.
- **Resultado:** ✅ 280/280 vitest (275+5 graph) + eslint 0 + engines + pack dry-run + porción TS ~30s <5min.
- **Commit:** ceb81d90 (solo task file)

### STABLE-09: promoción subset default-members (plan 2026-09-09 Wave0, Owner A)
- **Fecha:** 2026-09-09
- **Objetivo:** carryover plan 2026-09-08 — PR único subset que mantiene Fast Gate <5min.
- **Resultado:** ✅ `default-members` → `[., python, memory, server, mcp]`; proxy (Heavy) y wasm (toolchain Tier 3) fuera con justificación CI_POLICY; nextest warm ×2 296s 2831/0; rollback `git revert 546dabd1`.
- **Commit:** 546dabd1

### TS-12: prep publish vantadb-node en npm (plan 2026-09-09 Wave0)
- **Fecha:** 2026-09-09
- **Objetivo:** desbloqueada por BND-08 — prebuilds + README experimental + checklist publish humano.
- **Resultado:** ✅ pack dry-run 6 files + tags sin colisión + secrets 0; publish real PROHIBIDO (humano vía checklist 9 ítems).
- **Commit:** 88ef674b (RETRY tras abort sin task_id)

### AST-007: Release major + barrido final cero-remanentes (plan 2026-09-10 anti-stutter 7/7)
- **Fecha:** 2026-09-11
- **Objetivo:** gate pre-publish (semver major, deny, verify) + barrido cero-remanentes + rollback plan + retrospectiva + archivar plan. Sin publish (via release-plz al mergear a main).
- **Resultado:** ✅ semver-checks major 15/0 (FormatKind::VantaFile removida, QueryResult struct->enum + drift develop) + deny exit 0 (RUSTSEC-2023-0071 triaged HS256-only, stale RUSTSEC-2026-0253 removido) + just verify exit 0 (fmt + clippy -D warnings + nextest 3143 passed/1 skipped + deny); migracion 38 pares del mapa en 102 ficheros downstream (346 errores clippy -D warnings) + VantaFileMap->FileMap + WasmMemoryInput; superficie Python preservada (pyclass/m.add/repr).
- **Commit:** 8fb3dd77 (+59b7b46c archiva plan)

### C2M1: baseline metricas Ca/Ce/I/A/D + gate ADP informativo (Fase 2 Wave 0)
- **Fecha:** 2026-09-13
- **Objetivo:** termometros pineados (cargo-modules 0.27 + cargo-coupling 0.4.0 + rust-dsm@5950a18) + JSON archivado + workflow informativo no bloqueante + tabla baseline.
- **Resultado:** ✅ 352 modulos/6207 couplings/health C + 13 SCC + acyclic artefacto documentado.
- **Commit:** 90aeca88

### F3G: cierre gate Fase 2 a 4/4 (Fase 3 Wave 0)
- **Fecha:** 2026-09-13
- **Objetivo:** re-medicion M1 post (D crate 0.5238->0.5167, campos 26->20) + excepcion accumulator firmada BND-03X + firma A1.
- **Resultado:** 3 artefactos + firmas con fecha, cero src.
- **Commit:** dd892c4c
### F3B: job CI informativo canonical_p99 (Fase 3 Wave 2)
- **Fecha:** 2026-09-13
- **Objetivo:** toolchain-check release (tantivy NO reproduce: compila 8m18s) + workflow no bloqueante + baseline publicado.
- **Resultado:** YAML-OK + fmt; issues ante regresion, nunca verde falso.
- **Commit:** e33c307f

### FIND-64: `'vanta-memory/**'` en paths `ci-rust-10.yml` (campaña 2026-09-15; distinto del FIND-64 llamaindex 2026-09-07)
- **Fecha:** 2026-09-15
- **Objetivo:** cambios solo-`vanta-memory/` disparan CI (push+PR); fix 2 líneas + gemelo `ci-rustdoc.yml` ticketado como FIND-92.
- **Resultado:** ✅ YAML parse + actionlint 0 + diff-check limpio; review P2-01 approve.
- **Commit:** 9a419d65 (nota: Backlog citaba hash `0f16cd39` erróneo; real `9a419d65`)

### FIND-66: Formula sync (ARM64 + head + mcp)
- **Fecha:** 2026-09-15
- **Objetivo:** `Formula/README.md` veraz ×3 (fila mcp quitada, Linux x86_64-or-ARM64, sección `--head` sin stanza eliminada, macOS ARM64 ✅ + fila Linux ARM64); rb shas intactos.
- **Resultado:** ✅ diff +3/-11 solo README + hooks verdes; review P2-01 approve.
- **Commit:** f50da880

### FIND-70: bench `ingestion_concurrent` en nightly con feature
- **Fecha:** 2026-09-15
- **Objetivo:** nightly corre el bench con `--features async-ingestion` (antes citado pero jamás corrido); `Cargo.toml` intacto.
- **Resultado:** ✅ actionlint 0 + YAML 12 steps + diff-check limpio; monitorear 1ª corrida nightly.
- **Commit:** 34effd35

### FIND-92: `'vanta-memory/**'` en paths `ci-rustdoc.yml` (gemelo FIND-64)
- **Fecha:** 2026-09-15
- **Objetivo:** réplica del fix FIND-64 sobre `ci-rustdoc.yml` push+PR + HALLAZGO gemelo-del-gemelo (`rustdoc-70.yml` → FIND-95).
- **Resultado:** ✅ YAML + actionlint 0 + rg 4 hits/0 gaps; review P2-01 reconciliado.
- **Commit:** e395b563

### FIND-95: `'vanta-memory/**'` en paths `rustdoc-70.yml` (gemelo-del-gemelo FIND-92)
- **Fecha:** 2026-09-16
- **Objetivo:** `rustdoc-70.yml` (`cargo doc --workspace`) ciego a `vanta-memory/`; +2 líneas push+PR.
- **Resultado:** ✅ actionlint 0 + YAML parse + review P2-01 approve.
- **Commit:** 79a4942d

### FIND-104: instalador interactivo completo end-to-end
- **Fecha:** 2026-09-17
- **Objetivo:** wizard guiado cero-friccion (proxy default-on + opt-out, bloques MCP por cliente, regla agente, prueba viva final).
- **Resultado:** ✅ setup-embeddings.ps1 (DbPath + MCP + regla + proxy TOML + resumen + live test) + launcher passthrough -ProxyConfig + assets/install/ (6 plantillas); NonInteractive intacto, secrets 0 hits, idempotente, test-mcp 5/5; P2-01 approve + follow-ups (backup .bak en Install-AgentRule).
- **Commit:** a1bea54b + 25109073 (follow-ups P2-01)

### FIND-105: comando unico de instalacion
- **Fecha:** 2026-09-17
- **Objetivo:** one-liner por OS (sin clone ni rustup) que encadena instalador->wizard + docs; incluye resto SHOW-05.
- **Resultado:** ✅ install.sh/ps1 con dry-run/no-wizard, backup idempotente, trust header honesto (sha256 cuando hay asset), README + QUICKSTART con one-liner; P2-01 approve + follow-ups (header sha256 suavizado, nota skew fallback).
- **Commit:** 1349e63c + 25109073 (follow-ups P2-01)

### FIND-108: integracion completa open-code-review
- **Fecha:** 2026-09-17
- **Objetivo:** review-rules por path + job CI nocturno + viewer como evidencia; delegation default sin key.
- **Resultado:** ✅ .opencodereview/rule.json (13 entradas 1:1, pretty, 13/13 Custom) + ocr-nightly.yml (delegate siempre-verde + full con key via check-key env-only + artefactos 30d) + task file sync; actionlint 0 + wrapper exit 0; P2-01 changes-required levantado (paths reales, secret via env, glob docs/api/**).
- **Commit:** 2430385a + 25109073 (follow-ups P2-01)

### FIND-115: sync one-liner con FIND-105
- **Fecha:** 2026-09-18
- **Objetivo:** superficies con instrucciones viejas muestran el one-liner FIND-105 o referencian la fuente unica.
- **Resultado:** ✅ comandos bare byte-identicos en 4 superficies (delta = contexto Trust/wizard/dry-run); README_ES nota-referencia ES (sin re-traduccion) + docs-view parrafo Trust/wizard EN + referencia a fuente; path web confirmado existente (sin SKIP); coverage 0 gaps + diff-check 0 + tsc exit 0 + OCR advisory sin Critical/High.
- **Commit:** c1e72d88

### FIND-119: sync counts mirrors `.opencode` (solo strings)
- **Fecha:** 2026-09-18
- **Objetivo:** 6 x `86 tools` stale en mirrors submodule vs 87 fuente.
- **Resultado:** 12 strings (submodule + espejo parent por invariante FIND-83 hash-SAME); 0 hits 86 + coverage 0 gaps + 10 pares SAME; cero commit en submodule; breakdowns suman-86 tracked a configOpencode; P2-01 approve.
- **Commit:** 83ff5ce0 (docs-only padre)

### SHOW-05 (resto): decision `vantadb-ts/examples`
- **Fecha:** 2026-09-18
- **Objetivo:** resto pendiente: mover-vs-referenciar `vantadb-ts/examples/`.
- **Resultado:** decision REFERENCIAR (precedente FIND-74, sin evidencia nueva) + 2 lineas (README:65, QUICKSTART:53); coverage 0 gaps; P2-01 approve.
- **Commit:** 9398b2bc

### FIND-126: limpieza docs formato-only (frontmatter + 70 lints)
- **Fecha:** 2026-09-19
- **Objetivo:** Lint Markdown + Frontmatter rojos en PR #182.
- **Resultado:** 11 archivos formato-only + acote `docs/dev/research/archive/**` en config; 1423 files 0 issues + frontmatter 0 missing; P2-01 approve.
- **Commit:** aa07c887

### FIND-127: triage infra CI + Dependabot (PR #182)
- **Fecha:** 2026-09-19
- **Objetivo:** providers sin rustc, wheel aarch64 sin docker, CodeQL/Vercel, 20 vulns npm/web.
- **Resultado:** renombres providers a API actual + wheels portable + triage 12 fix-ya / 8 aceptar-riesgo; P2-01 approve + follow-ups (lru motivo).
- **Commit:** d3774c1a/909a119b (mismo contenido) + d755aeec (follow-ups P2-01)

### FIND-128: fix ADR-Gate multiline + matriz triggers 28 workflows
- **Fecha:** 2026-09-19
- **Objetivo:** ADR Gate rojo (`Invalid format ADR-0015` + `output` command) + duplicados push-vs-PR en PR #182.
- **Resultado:** 1 línea (`paste -sd`) + comentario; matriz 28 triggers (duplicados = push[develop]+PR[main] mismo SHA); dedup como propuesta escrita; P2-01 approve.
- **Commit:** 50c799d9 (rebase de ddd9d58c)

### FIND-129: triage 20 PRs dependabot + release-plz
- **Fecha:** 2026-09-19
- **Objetivo:** 20 PRs abiertos sin cerrar a ciegas.
- **Resultado:** Lote1a mergeado (#168/#165/#164/#163 pins CI); 9 rebases pateados; majors #174/#175 + #180 + #161 con veredicto; hallazgo: ruleset 11 checks vs triggers PR-a-main (ningún dependabot-PR lo satisface).
- **Commit:** remotos #168/#165/#164/#163 + 72b29b92 (task file)

### FIND-133: triage semver ~20 breakings intencional-0.6.0
- **Fecha:** 2026-09-19
- **Objetivo:** Semver Checks rojo (develop vs crates.io 0.5.0) — HACERLO PASAR con triage, no con revert ciego.
- **Resultado:** 21 cats/~100 ítems, 0 accidentales; ADR-0044 sin-revert; verde real solo con bump 0.6.0 vía release-plz en main (rojo-en-develop bendecido por scope main-only); P2-01 approve.
- **Commit:** 30b6a1f2/ab3eb373 (rebase de 3c4f146c/6afbe06e)

### C-06 fast/heavy + otel quartet
- **Fecha:** 2026-09-23
- **Objetivo:** sacar bench FULL del path de PR; resolver skew otel 0.33.
- **Resultado:** ✅ heavy-bench-nightly sin trigger PR; otel trio + tracing-opentelemetry 0.34 atomicos (rustdoc verde).
- **Commit:** e0f79a85 + 18b7352b

### FIND-150: Higiene dependencias + DoD v2 (llvm-cov/machete/OSV)
- **Fecha:** 2026-09-24
- **Objetivo:** Dar herramienta medible al DoD v2 (70% cobertura) + 0 deps sin justificar (Regla 6) + OSV-Scanner en CI junto a cargo audit.
- **Resultado:** ✅ machete 9 paths exit 0 (1 dev-dep `clap` removida + 7 ignored justificados); OSV 2.6.0 exit 0 (2 vulns reales lru/paste triageadas en osv-scanner.toml — audit.toml las silencia); jobs `osv`+`machete` en ci-rust.yml + REQUIRED en ci-gate; DoD v2 nombra las 3 herramientas reales.
- **Commit:** 3fef3cf9 (+9b75978 en configOpencode: DoD v2)

### EST-03: ci-gate mide `main` HEAD (fix del rojo eterno en PRs)
- **Fecha:** 2026-09-24
- **Objetivo:** El check `ci-gate / Main is green` (reusable; consumido por `fuzz`/`heavy-bench-nightly`/`heavy-certification` vía `needs: ci-gate`) fallaba en cada PR: usaba el head SHA del PR, corría a los 14s con checks pending → 13 `<not found>` → fail-closed eterno (y skipeaba los jobs de fuzz en PRs).
- **Resultado:** ✅ mide `main` HEAD vía API + conclusión más reciente por nombre (`sort_by(.started_at)`); `success|skipped|neutral` pass; `failure|timed_out|cancelled|action_required` fail; sin runs en main → WARN tolerado (OSV/machete aún no corren en main pre-merge). Decisión owner opción A. Verificación local 3/3 casos (REQUIRED real → FAILED=0; +`Lint Markdown` failure → FAILED=1; inexistente → WARN) + `ci-gate / Main is green` = **pass** en PR #222 (run `36084499760`). Nota: el check no está en los rulesets (11 contexts) — no bloqueaba merge, pero su rojo skipeaba fuzz.
- **Commit:** 0c27a960

### FIND-153: perf-bench regression gate activado (baseline compuesto)
- **Fecha:** 2026-09-25
- **Objetivo:** Activar el gate de regresión de perf-bench (detectado INERTE por el review P2-01 de EST-05: baseline vacío → compare warning + exit 0).
- **Resultado:** ✅ Rebaseline via `workflow_dispatch update_baseline=true` (run `36093538630`, median 3 runs, artifact `vanta-benchmark-results`) → `benchmarks/python_baseline.json` compuesto (5 secciones / 16 métricas, `updated: 2026-09-25`, metadata `perf-bench.yml` corregida — fold-in del review) + push. Run de verificación `36094025761`: compare REAL → `##[notice]No regression > 15.0% detected across 16 metrics.` (runtime `##[warning]No baseline stored` = 0). El job ya puede fallar por regresión >15%.
- **Commit:** 114f55f0

### C-07: curación de PRs/ramas (contrato: PRs abiertos = solo vivos)
- **Fecha:** 2026-09-25
- **Objetivo:** Cerrar la higiene de ramas: borrar stale sin PR; dejar solo PRs vivos.
- **Resultado:** ✅ Remoto: solo `develop`/`main` (8 ramas `release-plz` stale borradas tras capturar SHAs `ce164415…af105a7f`; 8/8 recuperables vía API); locales: 2 merged borradas (`ccc8ee4e`, `bd22f387`), 5 no-merged conservadas; ~35 refs dependabot stale limpiadas con `fetch --prune`; PR abierto único = #222 (intacto). Verify `campaign_verify_cmd` passed=true (`origin=['develop','main'] | open PRs=[222]`). Review P2-01 ✅ ronda 2 (ronda 1 cazó verify por cardinalidad → v2 con set de nombres).
- **Commit:** ac46911d (+02e7d32a docs)

### Release 0.7.0: merge #222 → publish completo (crates.io/npm/PyPI/wheels/SBOM)
- **Fecha:** 2026-09-25
- **Objetivo:** Cerrar la puerta de release (merge develop→main + tag + publish) tras GO owner.
- **Resultado:** ✅ Merge #222 (`58a41ad8`; 11/11 checks requeridos verde; ASan/TSan = ruido informativo) → release-plz publicó por el bump previo en develop: crates.io `0.7.0` + tag `v0.7.0` + GitHub Release `vantadb-v0.7.0`. Cascada: los tags de release-plz usan `GITHUB_TOKEN` → no disparan workflows; se completó con `workflow_dispatch --ref v0.7.0` (wheels `publish-pypi` evalúa `github.ref` tag) + aprobación deployments `pypi`/`npm`: npm `vantadb`/`vantadb-wasm` 0.7.0, PyPI `vantadb-py` 0.7.0, 4 wheels adjuntos al Release, SBOM ok. Release PR #223 (CHANGELOG) mergeado con bypass admin (0 checks por diseño, igual que #221).
- **Pendiente:** `vantadb-node` (nunca publicado; EST-11), binaries (nunca construidos; parity 0.6.1 = solo wheels), release PR #225 (v0.7.1 acumulando — no mergear sin decisión).
- **Commit:** `58a41ad8` + `2d4d24bf` (main)

### Fix mojibake CHANGELOG 0.7.0 (PR #226)
- **Fecha:** 2026-09-25
- **Objetivo:** Corregir `ΓåÆ`/`├│` en la entrada 0.7.0 de `docs/CHANGELOG.md` (UTF-8 leído como CP437/CP850).
- **Causa raíz:** el subject del squash de #222 se pasó por consola PowerShell (CP850) → commit `58a41ad8` con mojibake → release-plz lo copió al changelog.
- **Resultado:** ✅ PR #226 (worktree + bytes UTF-8 exactos) → `16d78dff`; verificado por API (0 marcadores) + scan repo-wide de secuencias CP437 = 0. Residual: mensaje del commit histórico `58a41ad8` (irreparable sin reescribir main). Prevención: `gh pr merge` sin `--subject` o `[Console]::OutputEncoding=UTF8` antes de capturar no-ASCII.
- **Commit:** `16d78dff` (main)

### EST-09: cierre stale configs CodeQL (post-merge #222)
- **Fecha:** 2026-09-25
- **Objetivo:** Confirmar que no reaparece la categoría stale tras el merge (contrato: últimas 20 analyses solo `sec-codeql.yml:analyze` + banner apagado).
- **Resultado:** ✅ `codeql.yml`/`sec-codeql-30.yml` ya no existen en `main`; sin analyses nuevas con categoría stale post-merge (las 3 históricas quedan ≤2026-09-24 18:27 y se deslizan del top-20 con los próximos runs). **Banner apagado: confirmación visual del owner pendiente (1 clic en Security → Code scanning).**
- **Commit:** (este commit)

### C-08: CodeQL setup post-release
- **Fecha:** 2026-09-25
- **Objetivo:** Confirmar el setup correcto y check verde tras el release (el check apuntaba a `codeql.yml` inexistente) + triage de alertas.
- **Resultado:** ✅ El check requerido `Analyze` lo provee `sec-codeql.yml` (job `Analyze`, línea 18) — verde en #222 (16m35s); `default-setup` = `not-configured` es el estado correcto (advanced setup activo; activarlo lo reemplazaría); 0 alertas abiertas. Contrato: check verde ✅.
- **Commit:** (este commit)

### FIND-154: perf-bench gate falso positivo por varianza de runner
- **Fecha:** 2026-09-25
- **Objetivo:** Explicar el rojo del gate de regresión en el push a main post-merge.
- **Resultado:** 🔴 Falso positivo: mismos commits verdes en develop (`36094025761`, dispatch 04:20Z) → rojo en main (`36101773914`, push 06:11Z): `query_hybrid.p50` 5.76→12.01ms (+108.5%), `p95` +88.1%, `p99` +18.8%, `query_text.p99` 0.01→0.04ms (µs = ruido). Baseline `benchmarks/python_baseline.json` calibrado en una máquina concreta → varianza entre runners. No es check requerido (no bloquea merges). **Acción propuesta:** tolerancia por métrica / banda de varianza multi-runner / re-baseline; fila `FIND-154` en Backlog.
- **Resolución (2026-09-27, HARD-06):** ✅ compare extraído a `benchmarks/compare_baseline.py` con bandas por familia — métricas estables bloquean >15%; familias ruidosas (`query_hybrid`, `query_text`) warn >15% y bloquean solo >300% (ceiling catastrófico, justificado en docstring); `--self-test` 2/2 (ruido +108.5% no bloquea; regresión +40% estable bloquea). `perf-bench.yml` invoca el script. FIND-153 sigue resuelto (baseline activo + run verde `36094025761`).
- **Commit:** (este commit)

### HIG-01: CHANGELOG dedup + release_always=false + release bodies reparados
- **Fecha:** 2026-09-25
- **Objetivo:** Cerrar la causa raíz de los GitHub Releases sin descripción (revisión solicitada por el owner).
- **Causa raíz:** `docs/CHANGELOG.md` tenía 2 documentos concatenados (2× `# Changelog`, 2× `## [Unreleased]`, frontmatter huérfano) → release-plz loguea "multiple release notes for 'Unreleased'. The git release body will be empty." → bodies vacíos en v0.6.0/v0.6.1/v0.7.0 (len=0).
- **Resultado:** ✅ PR #227 (main `6f2c1cfe`): dedup (1× H1 + 1× Unreleased + orden 0.7.0→0.4.0; 174,914 vs 175,339 bytes — 0 contenido perdido) + `release_always=false` (release solo al mergear el Release PR; elimina el race publish-antes-de-changelog). develop sincronizado (`54845169`). Bodies reparados: v0.7.0 (206 chars), v0.6.1 (33,808), v0.6.0 (119,109, truncado con link — límite GitHub 125k). #225 (release v0.7.1 docs-only) cerrado. **Pendiente de verificación:** el próximo Release PR no debe reintroducir el duplicado.
- **Commit:** `6f2c1cfe` (main) + `54845169` (develop)

### API-09: W8 cierre — VERSIONING 11 superficies + docs sync + gates (review P2-01 ✅ fresco)
- **Fecha:** 2026-09-26
- **Objetivo:** Cerrar la campaña "Estandarización 11 APIs": `docs/api/VERSIONING.md` con las 11 superficies (Gate P), sync de `docs/api/` (19 files), gates de cierre verdes.
- **Resultado:** ✅ Contrato 6/6 — coverage 0 gaps · `verify.ps1` ALL 11 PASS (fix tooling: `llvm-cov nextest run`→`nextest`; coverage real 81.63% ≥60) · MCP re-smoke 11/11 · OCR 0 Critical/High · plan 18/18 + campaña 9/9 · review P2-01 fresco ✅ APPROVE (3 nits Low aplicados) · FIND-161/162 registradas.
- **Commit:** 032cbd0f (local, sin push)

---

## Release 0.8.0 — cierre post-release (2026-10-02)

### Release 0.8.0: merge #233 → publish completo + política de merge commit
- **Fecha:** 2026-10-02
- **Objetivo:** Publicar 0.8.0 (schema v2 + estandarización API + fixes pre-release) con changelog curado y docs de API sincronizadas.
- **Resultado:** ✅ Merge develop→main #233 (`72353e7f`) + Release PR `cca43b9e` → **publicado y verificado: crates.io · PyPI · npm wasm+TS · binarios 5/5 · SBOM**. Docs API sincronizadas a 0.8.0 (`2f8528f1`, `f54e0b8c`) + CHANGELOG enriquecido curado de los 184 commits del ciclo (`ef30ab1f`, owner-approved) + snapshot public-api refrescado (`eb1d09b2`). Decisión owner: merges develop→main con merge commit (no squash) para changelog rico (`d5339480`).
- **Commit:** `72353e7f` (main) + `cca43b9e` (main)

### FIND-229: `release-binaries` falló en su PRIMERA ejecución (0.8.0) — combo release + Docker retirado + backfill
- **Fecha:** 2026-10-02
- **Objetivo:** Desbloquear el primer run de `release-binaries`: `-D warnings` (default de setup-rust-toolchain) + features `server,jemalloc` SIN `cli` moría por imports sin cfg-gate en `debug_ops.rs` + `fuse_rrf` dead-code (CI no cubre el combo); y `docker build .` sin Dockerfile en la raíz.
- **Resultado:** ✅ cfg-gate (`9004c43f`) + **Docker eliminado repo-wide por decisión del owner** (`fc50adb2`: job `docker-image` borrado, Dockerfiles/compose `git rm`, docs purgadas) + input `release_tag` para backfills + merge topológico a main (`c2afb9dd`) + dispatch backfill v0.8.0 → **binarios 5/5**. Historial Docker: WEB-02 (Dockerfile webapp → ghcr) + Docker build CI multi-arch (wontfix). Derivadas: FIND-230 (gate npm version) + FIND-231 (job CI del combo release).
- **Commit:** `9004c43f` + `fc50adb2` (+ `c2afb9dd` merge)

### npm TS backfill: `vantadb-ts` 0.7.0 → 0.8.0 + publish-ts no-skip
- **Fecha:** 2026-10-02
- **Objetivo:** El tren npm 0.8.0 saltó el publish de `vantadb-ts` en silencio (`package.json` quedó 0.7.0 y el check "already published" salió success sin publicar); el backfill `package=ts` destapó que `publish-ts` también se saltaba cuando `publish-wasm` se saltaba.
- **Resultado:** ✅ bump `package.json`/lock a 0.8.0 (`e62e0f62`) + republicación manual (dispatch `package=ts`) + fix del workflow `needs + always` (`70dd6eb3`: solo bloquea si wasm falla) + limpieza del escape literal (`723bc291`). Gate mecánico anti-repetición: FIND-230 en Backlog (Task 3 del plan post-release).
- **Commit:** `e62e0f62` + `70dd6eb3` + `723bc291`

### rustls ARM64: reqwest → rustls en `vanta-memory` + root (drop native-tls/openssl)
- **Fecha:** 2026-10-02
- **Objetivo:** El build release `aarch64-unknown-linux-gnu` moría cross-compilando `openssl-sys` (native-tls vía reqwest default; sin sysroot cross) → asset ARM64 bloqueado del backfill de binarios.
- **Resultado:** ✅ migración a `rustls-tls` en `vanta-memory` + root (alineado con mcp/server/proxy): openssl fuera de todo árbol Linux (-86 líneas de lock) + `deny.toml` allow `CDLA-Permissive-2.0` (`webpki-roots`) y drop del ignore stale RUSTSEC-2024-0429 (`cf86e49b`, `cargo deny check` OK). Parte del backfill de binarios v0.8.0 (FIND-229).
- **Commit:** `01d86ab4` + `cf86e49b`

### FIND-228: Dedupe de triggers CI — drop `develop` de `push.branches` (15 workflows)
- **Fecha:** 2026-10-03
- **Objetivo:** RULE 1 (RULES.md §1): los 15 workflows listaban `develop` en `push.branches` → duplicado push+PR por el mismo SHA en ventanas de release (~80 checks vs ~45 esperados).
- **Resultado:** ✅ `develop` removido de `push.branches` en 15 workflows (queda `[main]`; `pull_request` intacto) + TRIGGERS.md/RULES.md/FAQ.md actualizados + actionlint 0 + review P2-01 approve. Contrato verificado EN VIVO: el push de cierre disparó SOLO `PERF` (run 37102010279).
- **Commit:** `0e5c9e9d`

### FIND-230: Gate mecánico de versiones npm (anti skip-silencioso)
- **Fecha:** 2026-10-03
- **Objetivo:** `vantadb-ts` quedó 0.7.0 vs workspace 0.8.0 → el publish npm saltó silencioso en el release 0.8.0 (run 37045932895).
- **Resultado:** ✅ `scripts/docs/check-npm-versions.mjs` (self-test 10/10, fail-closed) + job `check-npm-versions` en gate-docs.yml + `::warning::` visible en los skips de release-npm-61/node + PUBLISH.md (orden del bump). Contrato FAIL/PASS verificado (0.7.0→exit 1 / 0.8.0→exit 0). Review P2-01 approve.
- **Commit:** `838ec8b6`

### FIND-231: Job `release-combo` — combo de release cubierto en CI
- **Fecha:** 2026-10-03
- **Objetivo:** El combo `server`+allocator con `-D warnings` (que rompió el primer run de release-binaries 0.8.0) no lo compilaba ningún job (los tests van en debug).
- **Resultado:** ✅ Job `release-combo` en ci-rust.yml (réplica de release-binaries.yml:109-122; `check --release` + `RUSTFLAGS=-D warnings`; 24-25s warm) + repro del rojo documentada (6 errores idénticos al run 37045939672 con el fix revertido). Review P2-01 approve (ronda 2).
- **Commit:** `4e1bb03a`

### FIND-232: perf-bench — perfil alineado + bandas cross-VM + re-baseline
- **Fecha:** 2026-10-03
- **Objetivo:** perf-bench rojo crónico (13 runs desde 2026-09-25): causa raíz doble — push corría perfil 1000/100 vs baseline 10000/1000 (p99 = máximo muestral) + varianza cross-VM medida 1.7-2.4x.
- **Resultado:** ✅ Perfil único 10000/1000 + guarda de mismatch fail-closed + bandas recalibradas (stable 25/200; noisy 300%+0.5ms; `insert.p99` ≥100ms absoluto) + re-baseline documentado + self-test 9/9. Post-push: run **37102010279 = success** ("No blocking regression detected across 16 metrics"). Derivada: FIND-233 (instrumento cross-VM).
- **Commit:** `04b3eaa0` + `0173b339`

### FIND-234: check-avance-coverage.ps1 leía docs/avance (inexistente) — reporte 0/237 engañoso
- **Fecha:** 2026-10-04
- **Objetivo:** El script de cobertura (referenciado por la skill progreso como check de cierre) apuntaba a `docs/avance` desde la ruptura 2026-09-23 (`b764d703`) → "0/237 (0.0%)" falso + errores de ruta.
- **Resultado:** ✅ Fix de 1 línea (L10 → `docs/dev/avance`): `1034/1034 (100.0%)` real, sin errores de ruta, exit 0. Review P2-01 APPROVE (before reproducido desde el blob HEAD~1; conteo independiente 237+907−1034=110 ✓).
- **Commit:** `35cbd2e1`

### BENCH-01: competitive_bench — región medida del Ingest aislada + fin del doble rebuild
- **Fecha:** 2026-10-04
- **Objetivo:** El timer de Ingest del harness competitivo envolvía setup (client init ~317 ms + prep de payloads) y el modo single-call duplicaba el rebuild HNSW (hidden rebuild dentro de Ingest + `rebuild_index()` en Index) → el número medido no era el que decía medir.
- **Resultado:** ✅ Región medida = `put_batch_raw` calls + `flush` (init/prep fuera) + `effective_chunk_size()` clampa todo chunk a <1000 por construcción (sin doble build posible; `--batch-size 0` legacy clampeado a 999) + `--self-test` 9/9 (fixture de regiones con stub engine: init excluido, calls <1000, exactamente 1 rebuild; RED→GREEN capturado) + docstring/README/BENCHMARKS §18/COMPETITIVE_SDK_BENCH/ANALYSIS actualizados con nota de comparabilidad (números publicados no regenerados). Review P2-01 delegado al orquestador (worker leaf) con evidencia mecánica completa.
- **Commit:** `860340b9` (+ `8cc49824` bookkeeping)

### DIST-05: Assets del release + verificación post-release real (fix del 404)
- **Fecha:** 2026-10-04
- **Objetivo:** el zip Windows de v0.7.0 daba 404; el flujo post-release debía verificar artefactos de verdad; `install.ps1` apuntaba a un asset inexistente.
- **Resultado:** ✅ `scripts/verify-release.ps1` (14 assets + 4 registries + smoke con sha256) + `.github/workflows/release-verify.yml` (semanal + dispatch — backstop que habría detectado el 404 en días) + `PUBLISH.md §Post-release verification` (gate con dueño). El verify encontró que `install.ps1` estaba roto **end-to-end** contra el asset real de v0.8.0 (zip flat vs `release\vanta-cli.exe`; PS 5.1 sin parsear; fallback v0.4.0 muerto) → corregido y verificado con installs reales en PS 5.1 y 7 (`vanta-cli 0.8.0`). v0.7.0: zip faltante por accidente estructural (cascade suprimido pre-`RELEASE_PLZ_TOKEN`), no por decisión; backfill no viable honestamente (documentado + procedimiento opcional). Review P2-01 APPROVE. OPTIONALs → FIND-261/262. Verificación en 0.9.0 diferida (checklist en PUBLISH.md).
- **Commit:** 2390344c (local, sin push)

### DIST-06: Estrategia de los 11 crates `publish = false` (PUBLISH.md §crates)
- **Fecha:** 2026-10-04
- **Objetivo:** decidir y documentar por crate qué se publica y por qué canal (la ambigüedad que costó FIND-230).
- **Resultado:** ✅ Nueva sección `PUBLISH.md §crates` (64 líneas): decisión por crate (10 con `publish=false` + `vanta-memory` remitiendo a DIST-01), canal real / producido-por / motivo; invariante release-plz con 2 comandos de re-verificación; política **fechada 2026-10-04 con 4 review triggers**; `release-plz.toml` intacto (0 churn). Verify: `rg '^\s*publish = false'` = 10 crates + overrides ⊆ tabla. Review P2-01 (ronda 1 → fix → delta APPROVE). Nota: durante el cierre, WIP de DOCS-F1 (check-links) hacía fallar gates globales — staging quirúrgico, ajeno a esta tarea.
- **Commit:** 56b0bc0a (local, sin push)

### DOCS-F1: Cerrar docs-consolidation F1 (triage de links + mojibake + markdownlint)
- **Fecha:** 2026-10-04
- **Objetivo:** cerrar F1 del plan docs-consolidation: triage P0-P4 de los 55 enlaces rotos vigentes, mojibake `[[bench]]`/`[[test]]` en prosa de `avance/`, y los 7 errores markdownlint introducidos por la migración (BASELINE 12→5).
- **Resultado:** ✅ Triage completo P0-P4 en `docs/dev/tasks/DOCS-F1.md` (P0-P2 drenados → **0 rotos fuera de frozen**; P3/P4 excluidos del gate con motivo en script `FROZEN_RE` + workflow). Scanner: links md ahora sobre `proseOf` (solo clicables gatean; 15 de 46 canónicos fully-in-code); budgets 58→0 y 40→20; líneas con offset de frontmatter corregido. T9: mojibake a 0 en prosa (2 a code span, `git log -S` sin variante anterior) + fórmula corrupta en `rrf.md` (`$[0, \infty)$`). T12: los 7 de BENCHMARKS/CONFIGURATION ya estaban drenados; MD052 del índice corregido en `gen-index.mjs` (escape de celdas); **ratchet 12→0 para la superficie controlada** (master plan excluido del conteo: recitations machine-appended del orquestador, FIND — su conteo creció 7→11 durante el cierre) con CI verde simulado (N=0). `postcard.md` nuevo + banner de deprecación en `master-index`. F1 ✅ en el plan (T8/T9/T12 COMPLETED). Verify: check-links 0/20 exit 0 · check-docs 0 · gen-index 0 · markdownlint 0 scoped · actionlint 0 · coverage 0 gaps. Review P2-01 ronda 1 degradado (subagente hoja) + OCR 0 Critical/High. **Ronda 2 (review adversarial fresco `vanta-review` `ses_ef8f093d1ffeuZhUBhU4WR3rW9`): 🔴 changes-required → fixes C-1 (links CI-only: HTTP_API `.opencode`→code span, discord `todo.md`→de-link), R-1 (`rrf.md` completo: 49 runs CP437→0), R-2/O-1 (FIND-264/265; 263 ya tomado por DOCS-F2), R-3 (triage reconciliado 55 local / 60 canónico), O-2/O-4.** Verificado con checkout limpio (`git archive HEAD`): check-links 0 · gen-index 0; local: check-links 0 · check-docs 0 · gen-index 0 · markdownlint 0 scoped.
- **Commit:** `6b07d1d9` + ronda 2 `a5909e38` (local, sin push)

### DOCS-F2: Cerrar docs-consolidation F2 (ejemplos ejecutables en CI + verificación de gates)
- **Fecha:** 2026-10-04
- **Objetivo:** cerrar F2 del plan docs-consolidation: T13 (ejemplos de docs ejecutables), T14 (gate API↔docs) y T15 (gate anti-fuga).
- **Resultado:** ✅ Reconciliación plan↔repo: T14/T15 **ya existían** desde 2026-09-29 (commit `71139665`: `gate-api-docs.yml` patrón DuckDB + `gate-docs-secrets.yml`) — verificados, no reimplementados (self-tests 17/17 y 29/29; 0 fugas). Hueco real de T13 cableado: **job `doctests`** en `ci-rustdoc.yml` (`RUSTDOCFLAGS="-D warnings" cargo test --doc --workspace`, medido EXIT 0: 13+1i vantadb, 1 vanta-memory, 1 mcp; `vantadb_py` cdylib skip) + **job `python-docstrings`** en `gate-doc-examples.yml` (pydoclint==0.11.0 `--style=numpy`; 4 violaciones DOC105/109/110/203 drenadas a 0). TS (typedoc + ejemplos como tests) → **FIND-263** con burn-down medido (typedoc 0 errores/39 warnings; sin infra CI de TS). F2 ✅ en el plan (T13/T14/T15 COMPLETED). Verify: verify.ps1 ALL 10 PASS · doctests EXIT 0 · pydoclint 0 · actionlint 0 · gates docs verdes · OCR 0 Critical/High. Review P2-01 degradado (subagente hoja) + escalado a vanta-review formal. Nota: el commit concurrente `a5909e38` (DOCS-F1 ronda 2) absorbió la regeneración de índices + la fila FIND-263 (staging quirúrgico, sin drift).
- **Commit:** `100e3fef` (local, sin push)

### PROV-12: Publicar wheels PyPI de providers (estrategia H-04) — lane de release + dry-run local
- **Fecha:** 2026-10-04
- **Objetivo:** dar camino de distribución PyPI a los 3 providers Rust (`providers/{openai,ollama,litellm}`): sin `pyproject.toml`/maturin, PyPI 404 (H-04 aprobada: publicar). Publish real = owner.
- **Resultado:** ✅ 3 `pyproject.toml` maturin (nombres canónicos `vantadb-openai`/`vantadb-ollama`/`vantadb-litellm`, `dynamic = ["version"]`, deps SDK declaradas) + `abi3-py311` ×3 → wheels `cp311-abi3`; **`.github/workflows/release-providers.yml`** (matriz 3 providers × 4 plataformas — linux x86_64 + aarch64 manylinux_2_28, macOS, Windows; maturin-action; OIDC TestPyPI dispatch / PyPI tag `providers-v*.*.*`; `skip-existing`; smoke por plataforma + verify installs; guard tag↔versión) + `.github/scripts/provider_wheel_smoke.py` (1 fuente para build/TestPyPI/PyPI). Verify: `maturin build` ×3 + `twine check` PASSED ×3 + smoke venv limpio ×3 + pytest 18/17/19 + actionlint 0 + `verify.ps1` ALL 10 + gates docs 0/0/0. Docs: `CI_POLICY` (canal), `PUBLISH.md` §Providers + checklist owner (durable), `TRIGGERS`/`README` (inventario 40), READMEs (Install PyPI con caveat). **Gate D owner:** colisión de nombres PyPI (`integrations/{openai,ollama}` reclamaban los mismos nombres/módulos) → providers canónicos; twins a renombrar/retirar en F6 (**FIND-273**); higiene de IDs de campañas archivadas → **FIND-274**. Review P2-01 APPROVE (`vanta-review` `ses_ef5dd5fa7ffejGRYYZxxlz3hA7`; M1 tag↔versión aplicado post-review). Publish = owner (checklist en `PUBLISH.md` §Providers).
- **Commit:** `04be0ec1` + `8ee81ef1` (local, sin push)

### PROV-13: Providers en Windows — job CI + fix real (verify_pyi.py Unicode)
- **Fecha:** 2026-10-05
- **Objetivo:** los 3 providers en Windows + job CI Windows (matriz provider).
- **Resultado:** ✅ DISCOVERY: la premisa "no compilan en Windows" estaba **stale** (resuelta por PROV-01/02/04; PROV-12 ya lo había anotado) — repro fresco: `cargo check`/`clippy -D warnings` ×3 ✅ + **54/54 tests** ✅. El bloqueante Windows REAL aislado: `.github/scripts/verify_pyi.py` crasheaba (`UnicodeEncodeError`, U+2713 vs cp1252) → fix raíz (`sys.stdout.reconfigure(utf-8)`), no parche en el workflow. Job CI Windows (matriz provider × os) + cache de target. Review P2-01 APPROVE (contexto fresco); `verify.ps1` ALL 10.
- **Commit:** b616e97c + 078e2fe2 (local, sin push)

### BENCH-02: BEIR/MTEB recall@k vs sqlite-vec — harness + número reproducible (BEIR SciFact test)
- **Fecha:** 2026-10-06
- **Objetivo:** medir calidad de recuperación (no solo velocidad) con el estándar BEIR/MTEB: recall@k contra qrels vs ≥1 competidor del segmento, reproducible con comando documentado (dataset/hardware/seed).
- **Resultado:** ✅ `benchmarks/beir_recall_bench.py` (recall@k MTEB/TREC + nDCG@10 + index-recall vs exact kNN; VantaDB vs sqlite-vec 0.1.9 sobre los MISMOS embeddings all-MiniLM-L6-v2 ONNX locales — sin torch; dataset `mteb/scifact` split test: 300 queries / 5.183 docs; seed 42; `--self-test` offline 16/16) + `BENCHMARKS.md` §21 (comando + entorno + notas de comparabilidad). **Número: recall@10 = 0.7833 / nDCG@10 = 0.6451** — nDCG@10 reproduce la referencia publicada MTEB Table 11 (MiniLM-L6 SciFact = 64.51). Ambos engines exactos a este tamaño (VantaDB rutea al flat exact scan: `flat_threshold` default 10.000 > 5.183 → `index-recall` 1.0 por construcción; `engine_config` en el JSON) — el número mide el stack embedding+ranking, no fidelidad ANN (declarado; FIND-315 para la medición HNSW real). pgvector no viable (sin Docker) → FIND-314. Review P2-01 `vanta-review` ronda 1 REQUEST CHANGES (R1 atribución HNSW + R2 FIND-312 duplicado con WIRE-12) → fixes → **ronda 2 APPROVE** (27/27 celdas §21 ↔ JSON, sha256 recomputados, IDs sin duplicados). Gates docs: check-links/check-docs/coverage 0; `gen-index --check` rojo por `WIRE-12.md` ajeno (staging quirúrgico, condición C1 del review — no se re-escribió).
- **Commit:** `21e9ff0f` (local, sin push)

### MKT-20 (F6/Task 71): Adapters PyPI — prep final con decisiones owner (retirar twins + bump pins)
- **Fecha:** 2026-10-06
- **Objetivo:** publicación de adapters (owner-assisted): dry-run + checklist; decisiones D1/D2 del owner.
- **Resultado:** ✅ **D1 (FIND-273):** twins `openai`/`ollama` RETIRADOS del release (matrices 9→7 ×2; dirs source-only con nota "superseded by providers"; FRAMEWORKS/PUBLISH/README re-apuntados) — decisión owner 2026-10-06. **D2:** pins `vantadb-py>=0.5.0,<0.7.0` → **`>=0.6.1,<0.9.0`** ×7 (piso real: 0.5.0 ✗ sin `Client`; 0.6.1/0.7.0/0.8.0 ✓). Re-verify: build ×7 + twine 14/14 + smoke ×7 (core 0.8.0 en UN paso) + `verify.ps1` ALL 10 + actionlint 0. Review 2 rondas APPROVE. **Checklist owner listo** (§F6-5): push → publishers OIDC ×7 → dry-run TestPyPI → tag `adapters-v0.5.0` → post-publish.
- **Commit:** 9f773e92 + 8d336134 + 01ef7a4c (local, sin push)
### Cierre CI PR #242 (2026-10-06): fixes para verde antes de main
- **Objetivo:** resolver los checks rojos del PR #242 (develop→main) antes del release 0.9.0.
- **Resultado:** ✅ 10 fallos clasificados y resueltos: **Tests (Windows)** race mismo-key (fix DUR-03 r3, 6f3990aa) · **ASan** FIND-244 (test stale vs `usable_vector`; expectativa corregida, 5ab753ce; verificado con `cargo test` 29/29) · **DIST-17 conformance** (producer Python desde root: dirs in-tree tapaban el wheel; a93152c6, validado local `1 passed`) · **markdownlint** (dedent recitations FIND-264 + pipe MEMG-11; d7053a74) · **gen-index/CodeQL** (escapes backslash ×4 + regen; 7625d655; 15 alertas test-only dismissadas, triaje vanta-audit) · **Desktop CI Linux** (timeout 30→45 cold-cache; 6e1b5d8c) · **Semver** (deltas vs 0.8.0 documentados en COMPATIBILITY.md, 0bc58339 — rojo en develop→main = señal de decisión; verde en release commit) · **TSan** (status quo FIND-227; no gatea) · **bench-informational** (crónico → FIND-317; no gatea).
- **Commits:** d7053a74 · 0bc58339 · 7625d655 · a93152c6 · 5ab753ce · 6e1b5d8c · 0602bcfb · 6f3990aa (+ este cierre; todos locales)
- **Snapshot (2026-10-08):** el ASan destapó `test_hardlink_snapshot_independence` (binario EXCLUIDO del default-filter de nextest → solo el job ASan lo ejecutaba): el snapshot hardlinkeaba archivos MUTABLES (WAL shards append+sync por put, `vector_index.bin`, journal fjall activo) → escrituras post-snapshot mutaban el snapshot (determinista en Linux; Windows copia → inmune). Fix worker `936868e5`: copiar appendables / hardlinkear solo inmutables (O(1) preservado, FIND-33 intacto) + test determinista RED→GREEN. Residuales → FIND-318 (vstore in-place: delete/compactación) + FIND-319 (keyspaces no recursado).
- **WAL (2026-10-08, capa 5 ASan):** `test_wal_selective_crc_corruption_recovery` rojo **determinista desde VER-01** (framing v3): el walk legacy leía los 32B génesis del `prev_hash` como 4 registros fantasma (len=0) → 5 vs 3 esperados. Invisible 9 días porque `wal_resilience` está EXCLUIDO del default-filter de nextest (línea 33) → solo lo corren el ASan (`cargo test` plano) y heavy cert (rojo desde Oct 4 sin triage; mismo `left: 5`). Fix worker `310aadda`: walk chain-aware + assert de consumo exacto del archivo (un framing futuro falla loud). Verificado 5/5 (+ storage 3/3, durability 7/7, path failpoints 5/5). Notas sistémicas: (1) el test corre en ~5s → candidato a gate más frecuente que weekly; (2) heavy cert arrastra 4 jobs rojos desde Oct 4 (other_heavy, memory_concurrency, mutation_test, failpoint_injections) — el failpoint_injections queda verde con este fix. (FIND pendiente de registrar cuando el Backlog esté libre — WIP ajeno.)

### CI-07: Pinning estricto de dependencias y versiones de actions
- **Fecha:** 2026-10-15
- **Objetivo:** Fijar versiones inmutables y hashes/tags fijos en GitHub Actions y dependencias para garantizar reproducibilidad en CI.
- **Resultado:** ✅ Verificado y completado dentro del marco del plan de pre-mercado y hardening de workflows.

### CI-08: Consolidación de workflows y eliminación de gates redundantes
- **Fecha:** 2026-10-15
- **Objetivo:** Consolidar los 40 workflows de GitHub Actions a 20 archivos `.yml` normativos, agrupando matrices y retirando duplicidades.
- **Resultado:** ✅ Consolidación completada a 20 workflows activos conformes a `docs/dev/workflow/RULES.md`.

### FIND-379: Automatización de sincronización multi-ecosistema en Release PR
- **Fecha:** 2026-10-18
- **Objetivo:** Automatizar la sincronización de manifiestos no-cargo (`vantadb-ts/package.json`, `server.json`, `docs/api/openapi.yaml`, `integrations/*/pyproject.toml`) en el Release PR de `release-plz` y desacoplar canales de publicación.
- **Resultado:** ✅ Implementado `scripts/docs/sync-workspace-version.mjs` (--check / --self-test), integrado en paso `release-plz-pr` de `.github/workflows/release.yml`, desacoplado `release-npm-node.yml` con input booleano `publish`, y `skip-existing: true` en `release-wheels.yml`.

### FIND-380: Hardening de seguridad y cumplimiento de RULES.md en workflows
- **Fecha:** 2026-10-18
- **Objetivo:** Auditar y blindar los 20 workflows de GitHub Actions según las reglas de seguridad de `RULES.md`.
- **Resultado:** ✅ Configurados `permissions: contents: read` a nivel global en `electron-node.yml`, pines de versión en `jetli/wasm-pack-action` y comandos `cargo install --version ... --locked`, eliminación de interpolación directa de `${{ inputs.* }}` en scripts inline de shell encapsulándolos en `env:`, y sincronizado el catálogo en `docs/dev/workflow/README.md`.
### FIND-178: `public_api_snapshot_matches_committed_file` fuera del fast gate local
- **Fecha:** 2026-10-02
- **Objetivo:** El test del snapshot de API p, build de rustdoc nightly) corr del fast gate local (`nextest -p vantadb`, profile audit, terminate-after 180s): con target fr el timeout de forma determinista (verify.ps1 2026-09-27: `TIMEOUT [180.364s]`, 2268/2269) y el snapshot whole-crate pod ya commiteado entre regeneraciones. Sacarlo del gate sin perder la se del surface.
- **Resultado:** ✅ Resuelto en `2510c16e`: el test es `#[ignore]` y qued; corre solo en el job dedicado (`--run-ignored ignored-only`) y los comandos de actualizaci (`VANTADB_PUBLIC_API_UPDATE=1`) quedaron documentados (header del test + COMPATIBILITY/VERSIONING). Residual relacionado (stack overflow cold-cache) en `FIND-191`. Origen: WIRE-04 cierre (2026-09-27).

### FIND-235: `docs/dev/workflow/ci-rust-10.md` stale — dice "13 jobs" (hoy 20) y coverage "59%" (hoy ~80%)
- **Fecha:** 2026-10-18
- **Objetivo:** Doc de CI alineada con el YAML real
- **Resultado:** ✅ 21 jobs y cobertura >=80% coinciden con `ci-rust.yml` (`docs/dev/workflow/ci-rust-10.md:19`).
- **Commit:** 357be38e

### FIND-236: Job ASan corre `sift1m_competitive_benchmark` (guard release-only) en debug → panic pre-existente mantiene el job roj...
- **Fecha:** 2026-10-18
- **Objetivo:** Test release-only sin coste en cada CI
- **Resultado:** ✅ Skip aplicado en `.github/workflows/ci-rust.yml:711-715,736`; task file completada y borrada.
- **Commit:** bec2dd1b

### FIND-321: Drift inventario workflows: `CI_POLICY.md` declara 28, el árbol tiene 40
- **Fecha:** 2026-10-18
- **Objetivo:** Gate mecanico del catalogo de workflows
- **Resultado:** ✅ Gate activo e inventario reconciliado 40->20 (`docs/dev/workflow/README.md:9-16`, `docs-gates.yml:581-585`).

### FIND-327: Parser del test WAL quedó stale tras VER-01 (framing v3); invisible 9 días por exclusión de nextest + heavy-cert sin ...
- **Fecha:** 2026-10-18
- **Objetivo:** Test de WAL parser alineado al framing v3
- **Resultado:** ✅ Parser chain-aware (5 vs 3 registros) y fixture en el gate nightly (`tests/storage/wal_resilience.rs:256-267`, `nightly.yml:108`).
- **Commit:** 310aadda

### MKT-20: Publicar 5 adapters en PyPI (re-escalado 2026-09-03)
- **Fecha:** 2026-10-18
- **Objetivo:** Publicar los 5 adapters en PyPI
- **Resultado:** ✅ 5/5 live en PyPI 0.5.0 (vantadb-langchain, llamaindex, mem0, crewai, dspy; 2026-10-07/08). Solo queda texto stale en READMEs (FIND-330).

### REL-03: Manifest único de versiones y superficies de distribución (generado)
- **Fecha:** 2026-10-18
- **Objetivo:** Manifest generado de versiones por superficie
- **Resultado:** ✅ Version/canal/tag por superficie generados (`SURFACE_MANIFEST.md:37-46`) con gates `version_coherence` y npm anti-skip.
- **Commit:** f84f7c70

### W5-12: la docstring de `verify_pyi.py` apuntaba a un workflow inexistente
- **Fecha:** 2026-10-20
- **Resultado:** ✅ `.github/scripts/verify_pyi.py` documentaba que su invocación de CI era
  `.github/workflows/providers-ci.yml`, un workflow que no existe (fue consolidado en
  `ci-fleet.yml`); el gate real corre en `.github/workflows/ci-fleet.yml:468` (job `check`).
  Docstring corregida. El gate se ejecutaba igual: solo la referencia estaba obsoleta.
  Hallazgo de la auditoría profunda v0.9.0 (P3, `FIND-461`).

### FIND-382 / FIND-383 / FIND-390 / FIND-471: cierres verificados

- **Fecha:** 2026-10-21
- **Resultado:** ✅ 4 hallazgos de la campana del 2026-10-20 (informe S1, area CI/release) ya
  estaban resueltos en el codigo pero sus filas seguian abiertas. Verificados uno a uno:

  - `FIND-382` (S1-08/manifest): la licencia real de `embeddinggemma-2` es **Apache-2.0**, no "Gemma" (verificado contra la API de Hugging Face el 2026-10-21: `cardData.license = apache-2.0`).
  - `FIND-383` (S1-08/downloader): el patron de descarga por modelo deja de arrastrar las 5 variantes ONNX: `onnx/*` son 30 ficheros (5.888 MB) cuando el encoder de texto solo necesita 2 (`onnx/model.onnx` + `onnx/model.onnx_data` = 1.034 MB).
  - `FIND-390` (Regla 2): los 11 `continue-on-error: true` llevan ya su `# CATEGORY:` y su justificacion (EXPERIMENTAL / BEST-EFFORT / NON-CRITICAL / INFORMATIONAL, segun `docs/dev/operations/CI_POLICY.md`).
  - `FIND-471` (S1-08/pins): los 4 pins de `actions/attest-build-provenance` apuntan a un SHA **inexistente** en upstream: re-pineados al SHA verificado `0f67c3f4...` (v4.1.1) con `gh api` 200. La atestacion de procedencia llevaba fallando en silencio en los 4 workflows de release.

---

## Campaña Pre-`v0.10.0`: Consolidación CI, Release-plz & Snapshots (2026-11-18)

### FIND-466: Blindar filtros `paths:` en workflows de CI para evitar checks colgados (absorbe FIND-582 M-13)
- **Fecha:** 2026-11-18
- **Objetivo:** Incluir `release-plz.toml`, `release-manifest.json`, `cliff.toml` y el propio workflow en `ci-rust.yml` y `docs-gates.yml`, más `integrations/**` en `ci-fleet.yml`, para que ningún PR de release o de adaptadores quede bloqueado esperando checks requeridos.
- **Resultado:** ✅ Filtros `paths:` (push y PR) blindados en `.github/workflows/ci-rust.yml`, `.github/workflows/docs-gates.yml` y `.github/workflows/ci-fleet.yml`; contrato PyYAML y auditoría `#24` PASS.
- **Commit:** `29a27e56f`

### FIND-392: Imponer `-p <crate> --locked` en los 5 pasos Cargo sin candado en `ci-rust.yml`
- **Fecha:** 2026-11-18
- **Objetivo:** Aplicar la Regla 1 (`-p <crate> --locked`) a los 5 pasos de `cargo check` y `cargo nextest run` de los jobs `test`, `test-windows` y `test-macos` en `.github/workflows/ci-rust.yml`.
- **Resultado:** ✅ Los 5 pasos explicitan `-p vantadb -p vantadb_py -p vanta-memory -p vantadb-server -p vantadb-mcp --locked`, con contrato endurecido que verifica la resolución de cada `-p` contra `Cargo.toml` del workspace; auditoría `#24` PASS.
- **Commit:** `934c55c97`, `430ab3174`

### FIND-468: Fallar cerrado (`exit 1`) en `sync-workspace-version.mjs` y cablear `--check` en `docs-gates.yml` (absorbe FIND-469)
- **Fecha:** 2026-11-18
- **Objetivo:** Hacer que `scripts/docs/sync-workspace-version.mjs --write` re-parsee tras transformar y falle cerrado (`exit 1`) ante cualquier regex miss o deriva, y cablear `--self-test` y `--check` en `.github/workflows/docs-gates.yml`.
- **Resultado:** ✅ 22 manifiestos sincronizados (`0.9.0`), `--self-test` 13/13 verdes, `--check` cableado en `docs-gates.yml`; auditoría `#24` PASS.
- **Commit:** `fa34006cf`

### FIND-467: Endurecer selector de rama `release-plz` y eliminar doble disparo TOCTOU en `release-wheels.yml` (absorbe FIND-472)
- **Fecha:** 2026-11-18
- **Objetivo:** Anclar el selector de rama de `release.yml` a `^release-plz-[0-9]{4}-` ordenando por `createdAt` descendente y eliminar el doble disparo `fanout` + `push.tags` sobre `release-wheels.yml`.
- **Resultado:** ✅ Selector estricto inmune a ramas manuales `release-plz-fix-*`, `push.tags` retirado de `release-wheels.yml` con `cancel-in-progress: false` conservando `workflow_dispatch`; auditoría `#24` PASS.
- **Commit:** `13e167d62`, `b18844ddc`

### FIND-473: Declarar `publish = false` en `vantadb-wasm/Cargo.toml`
- **Fecha:** 2026-11-18
- **Objetivo:** Impedir como defensa en profundidad que `cargo publish --workspace` o `release-plz` intenten publicar `vantadb-wasm` en crates.io.
- **Resultado:** ✅ `publish = false` declarado en `[package]` de `vantadb-wasm/Cargo.toml`; auditoría `#24` PASS.
- **Commit:** `99ffce9dc`

### FIND-455: Gate mecánico de ancestro (`origin/main` ancestro de `develop`) en PRs hacia `main`
- **Fecha:** 2026-11-18
- **Objetivo:** Verificar mecánicamente `git merge-base --is-ancestor origin/main HEAD` con `fetch-depth: 0` en PRs con base `main` (eximiendo ramas `release-plz-*`) para impedir regresiones de hotfixes.
- **Resultado:** ✅ Job `main-ancestry-gate` implementado con checkout de `head.sha` y `fetch-depth: 0`; auditoría `#24` PASS (movimiento a workflow sin filtro de rutas cubierto por `FIND-633`).
- **Commit:** `e87ede1f8`

### FIND-550: Re-inyectar `vantadb-node` en `optionalDependencies` en `release-npm-wasm.yml`
- **Fecha:** 2026-11-18
- **Objetivo:** Restaurar `optionalDependencies["vantadb-node"]` con la versión exacta del paquete después de `npm ci` y antes de `npm publish` en `.github/workflows/release-npm-wasm.yml`.
- **Resultado:** ✅ Step de restauración y aserción fail-fast verificado (`npm ci` < restauración < `npm publish`) y validado con `npm pack`; auditoría `#24` PASS.
- **Commit:** `6b7a37f4d`

### FIND-649: Reparar la matriz semanal de compatibilidad de adaptadores (`ci-fleet.yml` `compat-matrix`)
- **Fecha:** 2026-11-18
- **Objetivo:** Declarar `package:` explícito en las 18 filas de `compat-matrix` en `.github/workflows/ci-fleet.yml` alineado con las cotas inferiores de `integrations/*/pyproject.toml`, verificar la versión instalada fail-closed y elevar el piso de `langchain-core` a `>=0.3.81`.
- **Resultado:** ✅ Código, pisos (`langchain-core>=0.3.81`) y `actionlint` verificados en auditoría `#24` (PASS con nota: pendiente ejecución CI real dado que el job está gateado a `schedule`/`workflow_dispatch` y no corre en push a `develop`).
- **Commit:** `682db2472`, `a1a73bfed`

### FIND-319: Espejar recursivamente `keyspaces/` de Fjall en `create_snapshot` y restaurar `backend/` en `snapshot_restore` (absorbe FIND-287)
- **Fecha:** 2026-11-18
- **Objetivo:** Hacer que `StorageEngine::create_snapshot` espeje recursivamente `keyspaces/` de Fjall (copiando manifiestos/journals activos y hard-linkeando solo SSTs/segmentos sellados) y que `snapshot_restore` restaure atómicamente `data/` y `backend/` con staging dual y rollback E/S.
- **Resultado:** ✅ Espejo recursivo con allowlist estricta Fjall/RocksDB + `.vanta.schema`, validación `backend_currents_match`, staging dual `data.pre_restore_*` + `backend.pre_restore_*`, `.restore_pending` diferido para MCP, y descriptor `sparse_index` en RocksDB; certificación Linux 7/7 fases verde (run `44183458267`), review P2-01 `vanta-review e7c1b42e` APPROVE y auditoría `#20` PASS.
- **Commit:** `08b0cf03b`, `2e0e3016a`, `f91d44a73`

### FIND-318: Excluir archivos mutables (`vstore_L*.vanta`, `encryption.registry.bak`) del hard-link en `create_snapshot`
- **Fecha:** 2026-11-18
- **Objetivo:** Garantizar aislamiento físico de inodos entre la base viva y los snapshots copiando todos los artefactos mutables in-place (`vstore_L*.vanta`, `vector_store.vanta`, `encryption.registry.bak`) y limitando los hard-links a segmentos WAL rotados inmutables.
- **Resultado:** ✅ `mirror_data_dir` copia toda la familia mutable y test E2E `snapshot_certification.rs:1372-1460` verifica `nlink == 1` e `ino != live_ino`; review P2-01 delta APPROVE y auditoría `#20` PASS.
- **Commit:** `5dca4cc25`, `08b0cf03b`

### Cierre Auditoría Forense `release-plz` y SemVer `v0.10.0` (`v0.9.0..fcfc3f875`)
- **Fecha:** 2026-11-18
- **Objetivo:** Confirmar mecánicamente qué versión calculará `release-plz` al abrir el Release PR desde `0.9.0` y auditar los commits con `BREAKING CHANGE` y cambios de API pública Rust en `v0.9.0..HEAD` (419 commits).
- **Resultado:** ✅ Confirmado determinísticamente (`#17`):
  - `release-plz 0.3.160` calculará **`0.10.0`** (tag `v0.10.0`, nunca `1.0.0`): en serie `0.y.z`, `VersionIncrement::breaking(0.9.0)` produce siempre `VersionIncrement::Minor` (`0.10.0`) tanto por Conventional Commits (`fix!:`) como por `semver_check = true`.
  - De los 6 commits que contienen `"breaking change"` (case-insensitive), solo **2** son roturas reales con `!:` y footer (`c76dde1bf` `FIND-411` y `de45d5961` `FIND-413/414`); los otros **4** (`d628a2741`, `a3da7cd78`, `2d9d66a95`, `004556f7a`) son menciones en prosa sobre el envelope por página W7 diferido.
  - Identificados **3 grupos adicionales sin `!:`** que modifican la API pública Rust (`tests/api/public-api.txt`) y activan `SemverCheck::Incompatible`: `f17caf906` (`FIND-652` `chrono → jiff`, byte-idéntico en disco/wire), `928df0970` (remoción de `InMemoryEngine`/`columnar`/`tokenizer` de la superficie default) y `f8bfd0a2d`+`18d36b792`+`a51dfcc1f` (`FIND-655` fachada `IndexReader`). Deben listarse explícitamente bajo `### Breaking Changes` en el `CHANGELOG.md` de `v0.10.0`.
- **Commit:** N/A (auditoría forense read-only `#17`)


## Migración de backlog — pase 2 campaña `consolidation-2026-11-07` (2026-11-19)

### FIND-633: El gate de ancestro `main-ancestry-gate` vive en un workflow con filtro `paths:`: un PR a `main` que solo toque rutas fuera del filtro no lo ejecuta
- **Fecha:** 2026-11-19
- **Objetivo:** Mover el gate de ancestro `main-ancestry-gate` a un workflow propio sin filtro `paths:` para que corra en todo PR hacia `main`.
- **Resultado:** ✅
- **Commit:** `81749f751`

### FIND-638: Pasos de `ci-rust.yml` fuera del alcance de `FIND-392` siguen sin `--locked`
- **Fecha:** 2026-11-19
- **Objetivo:** Añadir `--locked` a los pasos Cargo restantes de `ci-rust.yml` fuera del alcance de FIND-392.
- **Resultado:** ✅
- **Commit:** `26223a498`, `3dd031b1e`

### FIND-639: `sync-workspace-version.mjs` solo valida la primera ocurrencia por patrón y degrada a `warn` los fallos de `gen-surface`/`gen-index`
- **Fecha:** 2026-11-19
- **Objetivo:** Validar todas las ocurrencias por patrón en `sync-workspace-version.mjs` y fallar cerrado ante un fallo de regeneración.
- **Resultado:** ✅
- **Commit:** `f47a5e1d2`

### FIND-650: El job TSan agota siempre su timeout de 85 min y `continue-on-error` lo reporta como `success`: nunca ha dado una señal verificable
- **Fecha:** 2026-11-19
- **Objetivo:** Sustituir los `--skip` inválidos del job TSan por filtro `nextest -E` para que deje de agotar el timeout enmascarado por `continue-on-error`.
- **Resultado:** ✅
- **Commit:** `80495db2f`, `3dd031b1e`

### PRD-05: Firma Automatizada `cosign`/`minisign` y Manifiestos `winget` + `brew` (`Q25 Opción B`)
- **Fecha:** 2026-11-07
- **Objetivo:** Firma automatizada `cosign`/`minisign` y manifiestos `winget` + `brew` generados (Q25 Opción B).
- **Resultado:** ✅
- **Commit:** `88022de96`


### FIND-185: `vantadb-node` y `providers/` NO eran workspace members
- **Fecha:** 2026-11-22
- **Objetivo:** Incorporar `vantadb-node` y `providers/` al workspace para que fmt/clippy/nextest los cubran.
- **Resultado:** ✅ — verificado en HEAD 2026-11-22: ambos en `members` (`Cargo.toml:31,58`); quedan fuera de `default-members` solo los experimentales declarados (vanta-proxy, vantadb-wasm) y `providers/*` individuales por crash MSVC (nota `Cargo.toml:802`).
- **Commit:** `dcf07a6d7`

### FIND-204: Sweep de refs stale `heavy-certification-50` fuera de BENCHMARKS
- **Fecha:** 2026-11-22
- **Objetivo:** Eliminar referencias stale al job `heavy-certification-50` en `CONTRIBUTING.md` y `.config/nextest.toml`.
- **Resultado:** ✅ — verificado en HEAD 2026-11-22: 0 ocurrencias de `heavy-certification-50`/`heavy_certification_50` en ambos archivos.
- **Commit:** `4c5c5bbd1`

### FIND-472: Eliminación de doble publicación concurrente tag+dispatch (`release-npm-wasm.yml` + `--locked` CI)
- **Fecha:** 2026-11-22
- **Objetivo:** Suprimir el trigger `push.tags: ["v*.*.*"]` en `.github/workflows/release-npm-wasm.yml` para evitar carrera TOCTOU con el fan-out de `release.yml`; completar `--locked` en `release-binaries.yml` y `ci-fleet.yml`; añadir exclusiones de artefactos temporales en `.gitignore`.
- **Resultado:** ✅ `RELEASE_PLZ_TOKEN` y retira de guards confirmados en `release.yml`, trigger duplicado retirado en `release-npm-wasm.yml`, y `--locked` aplicado en steps Cargo restantes. Adicionalmente verificado `FIND-489` (`test-embedding-features` activo en `ci-rust.yml:433`).
- **Commit:** `799f4a187`

### Cierre stamper de frontmatter BOM/CRLF (FIND-707)
- **Fecha:** 2026-11-24
- **Objetivo:** Que volver a correr `scripts/docs/stamp-frontmatter.mjs` nunca inyecte un segundo bloque de frontmatter, ni deje una linea en blanco antes del H1, ni derive el `title` del nombre de fichero. Causa raiz doble en el punto compartido: `parseFrontmatter` (`scripts/docs/lib.mjs`) no toleraba BOM (`hadFrontmatter:false` falso) ni un bloque cerrado al final sin salto, y el escritor recomponia con `/^\n+/` dejando el `\r` atras. Un solo fix en el parser cura a los 5 consumidores (`check-docs`, `check-links`, `gen-index`, `check-doc-examples`, `stamp-frontmatter`).
- **Resultado:** ✅ Verificado: `--self-test` 6/6 con 3 casos de dano; 3 mutantes (sin strip de BOM, `/^\n+/` restaurado, LF sobre CRLF) devuelven exit 1; idempotencia `--write` -> 777 unchanged con SHA256 identicos; `check-links`/`check-docs`/`check-secrets`/`check-doc-examples` en 0. Corpus reparado a mano (33 ficheros) sin reescritura masiva.
- **Commit:** `51c038efd`

### Cierre MSRV declarado (FIND-705, absorbido en FIND-701)
- **Fecha:** 2026-11-24
- **Objetivo:** El MSRV declarado (`rust-version = "1.94.1"`) era falso: `croaring 2.8.0` y `sysinfo 0.39.6` exigen `rustc 1.95.0`, y nadie lo notó mientras el gate era decorativo (el pin de `rust-toolchain.toml` coincidía con el MSRV real). Declarar el MSRV verdadero y renombrar el check requerido.
- **Resultado:** ✅ `Cargo.toml` declara `rust-version = "1.95.0"` y el job pasó a `MSRV Check (1.95.0)` (`ci-rust.yml:468`). Evidencia del fallo original: run `45253777361`, job `137970066687`. El fallout de `FIND-701` en los jobs fleet/health se cerró aparte. Queda pendiente **solo** la pasada verde de CI, que se verifica en `FIND-701`.
- **Commit:** `5a669a186`, `11abb6e54`

### Cierre matriz de binarios del release (FIND-709)
- **Fecha:** 2026-11-24
- **Objetivo:** La fórmula de Homebrew referencia 4 tarballs (`x86_64`/`aarch64` × `apple-darwin`/`linux-gnu`) mientras `release-binaries.yml` publicaba un solo target: el release saldría con la fórmula rota.
- **Resultado:** ✅ Verificado en `release-binaries.yml:68-77`: la matriz declara 5 targets (`x86_64`/`aarch64-unknown-linux-gnu`, `x86_64`/`aarch64-apple-darwin`, `x86_64-pc-windows-msvc`), de los cuales los 4 de la fórmula existen. La validación real llega con `scripts/verify-release.ps1 -Tag v<version> -Smoke` en el release.
- **Commit:** `9dab5093b`

### Cierre de los dos gates que mentían (FIND-701 + FIND-702)

- **Fecha:** 2026-11-24
- **Objetivo:** Cada uno de estos dos gates daba por buena una realidad que no era: el `toolchain:` del composite `rust-setup` era decorativo (`rust-toolchain.toml` ganaba al input, así que el job MSRV medía 1.95.0 mientras declaraba otra cosa) y ningún gate linteaba las composite actions (`actionlint` solo mira `.github/workflows`, de modo que un `run:` sin `shell:` — inválido para GitHub — pasaba verde). Ambos fixes estaban aplicados desde el 11-23 en estado 🟡 Parcial; solo faltaba la pasada real de CI, que es el único gate que ninguno de los dos puede simular en local: el bug de FIND-701 era precisamente que el entorno local y el de CI no coincidían.
- **Resultado:** ✅ Verificado sobre `8d355e2df` (run `45456649424`) y sobre el run de docs posterior:
  - **FIND-701** — `MSRV Check (1.95.0)` en **success** (job `138667791016`) y, lo que de verdad prueba el fix, el log del job: `rustc 1.95.0 (59807616e 2026-04-14)`, el step del composite ejecutó `echo "RUSTUP_TOOLCHAIN=1.95.0" >> "$GITHUB_ENV"`, todos los steps siguientes llevan `RUSTUP_TOOLCHAIN: 1.95.0` y `cargo check --locked` termina en 1m16s. El gate mide el toolchain que declara. Suma `Release Combo (server + allocator, -D warnings)` en success (job `138675656932`), que usa el default del composite (`1.95.0`) vía el mismo export.
  - **FIND-702** — `actionlint (workflows lint + inventory gate)` en **success** (run `45461424443` @ `ee41ec51b`), y `git merge-base --is-ancestor a78e36ed2 ee41ec51b` exit 0: el run incluye el guard nuevo de conteo `run:`/`shell:` (`docs-gates.yml:634-648`), que pasó sobre el árbol real.
  - **TSan (las 4 particiones en rojo del mismo run):** diagnosticado como **clase de infraestructura ya cerrada en FIND-227** (`std` sin instrumentar + TSan ciego a fences), **no una race del motor**. Los 4 logs comparten clase — `std::sync::mpmc::list::Channel<test::event::CompletedTest>` + `core::profiling::compiler_copy`, **cero frames de `vantadb`** — con el test en `test result: ok` y muerte del proceso por exit code 66 (el `exitcode` por defecto de TSan), sin aserción fallida que triagear. No es regresión del push ni del nightly flotante: **misma clase y los mismos cuatro tests** con nightly `137d1d494` (run `45109855351`) y `20fe935d5` (este run) — los conteos difieren (`59 vs 60` binarios, `2761 vs 2763` filtered-out) porque son commits distintos, y eso no afecta el veredicto. Ya estaba rojo en `44853213235`. El job es `continue-on-error` con `CATEGORY: NON-CRITICAL` (`ci-rust.yml:952`, catalogado en `CI_POLICY.md:65`), así que no bloquea el merge ni el release.
- **Hallazgo derivado (registrado como `FIND-714`, no arreglado):** la lane TSan es **ciega**, no solo ruidosa — y eso es lo que FIND-227 no cubre. La premisa de FIND-650 ("nextest elimina la clase `libtest`") es falsa para `--lib --tests --bins`: `libtest` sigue construyendo su canal `mpmc` en cada proceso, y con fail-fast la lane muere en **0,60–0,79 s** habiendo probado **4 de 821 tests** (`Starting 821 tests across 60 binaries` → `4/821 tests run: 0 passed, 4 failed`; los 4 son del binario `--lib`, los otros 59 nunca corren). Su rojo no dice "hay una race" y un eventual verde tampoco diría "el motor está limpio". En un motor con paths multi-índice eso es deuda de la Regla 8 por sí sola. Opciones con contrato en la fila: (a) aislar el ruido del harness **y** exigir *tests ejecutados* además del exit code (ojo: `--no-fail-fast` solo no sirve, convierte 4 rojos en ~205 rojos lentos); (b) retirar el job con decisión explícita en `CI_POLICY.md` + `# CATEGORY:`, nunca con `continue-on-error` (Regla 2). No se implementa acá: exige un round-trip de CI y el push no está autorizado para esta tarea.
- **Hallazgo derivado (registrado como `FIND-715`, 🔴 Crítica — bloquea el release):** los rulesets `develop` (`23692587`) y `main` (`16844590`), ambos `enforcement: active`, siguen exigiendo el status check **`MSRV Check (1.94.1)`**, nombre que ningún job reporta desde `5a669a186`. O sea: **el run es verde y el gate requerido sigue siendo insatisfacible**, así que el PR `release-plz-*` → `main` de F0.6 no puede cumplirlo. Que `8d355e2df` entrara no lo desmiente: un push directo no pasa por el gate de PR, y eso lo enmascaró. Es el mismo género que FIND-701 (el nombre decía una cosa y el sistema medía otra) en la capa que ningún workflow ve. Fix: PUT del ruleset con el context correcto en ambos entornos — acción del owner (token admin), fuera de las autorizaciones del plan.
- **Commit:** _(este commit)_ `docs(ci): close FIND-701/702 and register the TSan lane debt (F0.1)`
- **Nota operativa:** `Tests (Windows)` cerró **✅ success** (56 min de 90, job `138675656820`), con lo que el run completo quedó `completed / success`: 19 success, 4 skipped, 4 failure (TSan no bloqueante).



### Cierre recitations MD007 y exclusión del gate (`FIND-264`)
- **Fecha:** 2026-11-24
- **Objetivo:** El formato de recitation rompía MD007 y el plan maestro estaba excluido del conteo de `docs-lint-regression` con una excepción permanente. La premisa de la fila (template + plan maestro) resultó **stale**: el plan se archivó en `4e831d3e1` y el corpus ya estaba en 0 (`a5da35529`, MD007 50→0); el `  - claim:` lo producía el **emisor**, `updateRecitation` (`parsers.mjs:158`), pegando `Contrato: ${data.contract}` en crudo.
- **Resultado:** ✅ El bloque se emite fenceado (`~~~text` … `~~~`), así MD007 es inexpresable en ese punto y la evidencia sale de `proseOf()` (cierra OBS-1: links vivos en recitations). Los marcadores `=== RECITATION [ID] ===` no cambian, así que `parseRecitation` y el aislamiento por tarea siguen iguales. Retirada la exclusión (`grep -v 'master-plan-0.9.0.md'`, 2 sitios) y el comentario que la justificaba; el conteo `N` es ahora el árbol controlado completo, con `test -s /tmp/lint.txt` para que un log no escrito no reporte verde. Verificado: markdownlint `docs/**` 342 ficheros / 0 issues · `check-links` 0 rotos (3239 links) · `check-docs` all clear · plan sintético con el emisor → MD007 0 · harness 35/35 y 20/20 · `actionlint` 0.
- **Commit:** `1a6df66b6` (VantaDB) · emisor + test + template + lesson en `.opencode/` (repo separado, sin commitear)
- **Deuda:** `gen-index --check` estaba rojo en HEAD por índices generados desactualizados; se regenera en el cierre de wave.

### Cierre triage heavy cert (`FIND-328`)
- **Fecha:** 2026-11-24
- **Objetivo:** Dar veredicto por job a los rojos de la certificación pesada (`other_heavy`, `memory_concurrency`, `mutation_test`) que llevaban sin triage desde el 4-oct.
- **Resultado:** ✅ Tres veredictos con evidencia de 25 runs programadas (11-06 → 11-24):
  - `memory_concurrency`: **arreglado, no se toca** — `success` 25/25; el fix fue de código/tests, no de CI. Es el gate de concurrencia de la Regla 8 y no se retira. Atribución del commit exacta no aislable (el workflow viejo se borró en `a4c290914`): candidatos `6b8002c1c`/`23c594f4f`/`8aad0cac1`, confianza media declarada.
  - `other_heavy`: **defecto de CI** — el gate TBH-01 corría `verify_datasets.sh` (5 datasets) **antes** de la descarga y el job solo baja GloVe-100 → `5 missing / 5 expected`, exit 1 en 12/12 runs. Reacotado a GloVe-100, movido después del download, con la misma guarda de truncado.
  - `mutation_test`: **dos defectos** — (a) moría en "Set up job" por un pin inexistente de `actions/upload-artifact` (`4e7e4d1…` → 422); (b) aun con pin válido no medía nada: `--check` significa *"cargo check a cada mutante, sin correr los tests"* y `--output-json` no es un flag, con el exit≠0 tragado por `|| true` → verde sin artefacto. Quedó INFORMATIONAL con pin válido, inventario `--list --json`, `if-no-files-found: error` y `test -s`.
  - **Causa del rojo invisible:** la fila apuntaba a `heavy-certification.yml`, renombrado a `certification.yml` en `a4c290914`; el comando de triage devolvía 404 y nadie inspeccionaba el workflow real.
- **Commit:** `3f7fdd3fa` · filas derivadas: `FIND-716` (colisión de `concurrency` job-level en 4 jobs) y `FIND-717` (la cobertura de mutación no existe).
- **Deuda:** atribución exacta del fix de `memory_concurrency` (workflow borrado) y falta un run verde de `certification.yml` (semanal/dispatch: un dispatch corre ~21 jobs, 2 h+).

### Scope de la regla `-p` en las superficies normativas del harness (`FIND-164`)
- **Fecha:** 2026-11-24
- **Objetivo:** Las superficies que cada agente lee antes de correr tests tenían menciones bare de `cargo nextest`/`cargo test` sin `-p`, contra la Regla dura `-p` de `AGENTS.md` (incidente `f6c395ef`, hang en `:8080`). La lista del backlog era de 2026-09-27 y el harness había cambiado.
- **Resultado:** ✅ Re-medido con `rg`: 9/9 líneas del backlog seguían vigentes, 2 rutas stale y 17 líneas bare **no inventariadas** → 26 menciones corregidas en 12 archivos. Tres clases: **scoped** (`-p <crate>` en `vanta-worker:93/237/257/262`, `vanta-engine:318`, `vanta-lead:170/228`, `vanta-docs:64/73/336/411`, `RULES.md:123`, `iter-loop-tools:178/205/234/371`, `task.md:357/395/398`, `pre-call-checks:61/64/88`, `workflows/{feature-add,refactor}.json`) · **tier de cierre** (`--workspace` explícito en `vanta-lead:84/265`, `vanta-worker:367`, `RULES.md:154`) · **intactos** los `--workspace` ya correctos (CI, `verify` full, certificación). Contrato: `rg -n 'cargo (nextest|test)' .opencode/{agents,task-system} ~/.agents/skills/unified-review --glob '!**/task-system/memory/**' | rg -v -- '-p |--workspace|# tier|cierre'` → **0 líneas**.
- **Commit:** `513d055f7` + `cba6c2a0f` (VantaDB, solo el task file) · los 12 ficheros del harness quedan **sin commitear** en el repo `.opencode/`.
- **Deuda:** gate contra el próximo drift **propuesto y no implementado** (~15 líneas en `dev-tools/verify.ps1` condicionado a diff en `.opencode/`, o 1 línea en el checklist de Gate H; 30 min). Además: `.opencode/skills/unified-review/` **no existe** — el espejo de skills del harness está masivamente borrado y sin commitear; la superficie viva es `~/.agents/skills/unified-review`.

### Bench canónico `canonical_p99`: el workflow no existía y el bug era el timeout (`FIND-317`)
- **Fecha:** 2026-11-24
- **Objetivo:** Decidir qué hacer con un check rojo perpetuo del bench canónico, con evidencia de runs en vez de la descripción de la fila.
- **Resultado:** ✅ **La premisa de la fila estaba vencida en 4 de 5 puntos.** El workflow que nombraba (`bench-canonical-p99-informational.yml`) **no existe**: se borró en `2ba62d46f` (2026-10-15) y quedó absorbido por `nightly.yml` (CI-08); `gh run list --workflow …` devuelve **HTTP 404**, así que el comando de triage de la fila no podía encontrar nada (misma clase de fallo que `FIND-328`). En las últimas 100 corridas de `nightly.yml`: **0 `cancelled`**. El sucesor **completa**: `159m39s` (run `44228761011`) y `145m02s` (`44228771621`), ambos `success`. Hipótesis resuelta: **lentitud, no hang** — `insert = 698.16 s/iter × 10 muestras = 7030 s` estructural; el run histórico murió a `5298 s` con timeout de `5400 s`, o sea el bug era el **timeout de 90 min**, no el bench.
  - **Decisión (c):** `bench-informational` pasa a `schedule`/`dispatch` (`if: github.event_name != 'pull_request'`), con `concurrency` propio `cancel-in-progress: false` (forma de `failpoint-tests`, **no** el grupo del workflow — esa colisión es `FIND-716`), `timeout-minutes: 240` intacto (1.5× del medido), y se elimina el gate `perf-path` que quedaba inalcanzable y mentía sobre qué corre. `toolchain-check` (9 min, compile-gate) se queda en PR.
  - Categoría `INFORMATIONAL` en `CI_POLICY.md` §6 con el bloque de decisión, presupuesto y reglas Must/Must-not. **0 exenciones nuevas** (23/23 `continue-on-error` siguen con `# CATEGORY:`).
- **Commit:** `9ffdd97b9`
- **Deuda derivada (fila `FIND-720`):** el comparador publica `stable_count: 0` → **la detección de regresiones no puede disparar nunca**; y `origin/main` sirve el bench **1536d** mientras `develop` ya tiene GloVe-100d, así que las corridas medidas midieron el corpus que `BENCHMARKS.md` §11 declara superseded. Además `BENCHMARKS.md:454` sigue citando el archivo retirado.

### Lane TSan: suppression del harness y aserción de corpus (`FIND-714`)
- **Fecha:** 2026-11-24
- **Objetivo:** La lane TSan existía, estaba roja siempre y **no medía nada**: las 4 particiones morían en `Summary [0.788s] 4/821 tests run` por el canal interno de `libtest` (`Channel<CompletedTest>`, **0 frames de `vantadb`**; `Tests (Linux)` verde en el mismo sha). `continue-on-error: true` hacía que el run sonriera igual.
- **Resultado:** ✅ La lane vuelve a poder medir: `TSAN_OPTIONS=suppressions=<abs>:halt_on_error=1` con `dev-tools/tsan-suppressions.txt` (`race:std::sync::mpmc` + `race:CompletedTest`, **libtest-only**: 0 hits de ambos patrones en `src/`+`tests/`), `--no-fail-fast`, `tee` del log y un step nuevo de aserción que falla con `X/Y tests run` (abortado) o `0 tests` (selección vacía), con `N` medido por nextest y no hardcodeado. El archivo entra en los `paths:` de push y PR. Comentario de `FIND-650` corregido (su premisa "nextest elimina la clase" es falsa). `continue-on-error` **se mantiene**: retirarlo exige un run verde real, que es acción del owner.
  - Detalle de higiene que evitó un falso negativo: el suppression file se verificó sin BOM y con `git ls-files --eol` en `lf` — un `\r` embebido haría que la suppression nunca matchee.
  - El review cazó dos verdes falsos: un `pipefail` que mataba el diagnóstico y un `0 tests` que pasaba la aserción. Ambos corregidos antes del `approve`.
- **Verificación:** `actionlint` exit 0 (worktree y blob staged) · la aserción probada con el shell real de CI en **7/7 shapes** (abortado 4/821, completo con fallos, limpio con suppression matched, sin summary, sin log, selección 0, singular `1 test run`) · `check-links` 0 rotos · `markdownlint` 0 issues · auditoría de no-contaminación: las 11 líneas borradas del workflow son todas de `sanitizer-tsan`, **0 de WASM** (`FIND-725` intacto).
- **Commit:** `4ca972ddb`
- **Pendiente (owner):** el run verde de las 4 particiones. Si sale verde con `suppression hits: > 0` y `race reports: 0`, se retira `continue-on-error`; si quedan races no-harness, la salida es retirar el job con `CATEGORY` explícito en `CI_POLICY.md`.
- **Residuales detectados y no tocados:** `docs/dev/workflow/ci-rust-10.md:43` describe la lane como `cargo test` 30m (obsoleto) · `.lsan_suppressions` (ASan) tampoco está en `paths:`.

### Gate de WASM por rango de push (`FIND-725`)
- **Fecha:** 2026-11-24
- **Objetivo:** El job `WASM (wasm-pack browser tests)` decidía con `contains(github.event.head_commit.modified, 'vantadb-wasm/')`, y `head_commit` es **solo el commit final** del push. En un push multi-commit, los tests de WASM no corrían aunque un commit intermedio tocara `vantadb-wasm/`.
- **Resultado:** ✅ Job liviano `wasm-scope` que difea el rango `before..after` (`git fetch --depth 1` + `git diff --name-only --no-renames -z`) y publica `outputs.wasm`; `wasm-test` lo consume vía `needs` con `!cancelled() && actor != 'dependabot[bot]' && (needs.wasm-scope.outputs.wasm == 'true' || needs.wasm-scope.result != 'success')`. **Fail-open en 4 bordes**: primer push de rama (`before` = ceros), fetch fallido, `pull_request`, y path no-ASCII.
  - Bug reproducido con la historia real: `a46689786..ec1bf9d58` contiene wasm (nuevo=true / viejo=false) y `20c164970..ec1bf9d58` no — exactamente el caso que dejó el job `skipped` en el run `45510840907`.
- **Verificación:** `actionlint` exit 0 (y controles negativos que fallan) · **8/8 escenarios** del predicado probados sobre el bloque `run:` extraído del `.yml`, con clones shallow y remoto `file://` (wasm intermedio, sin wasm, primer push, tip, fetch-fail, pull_request, rename-out, path no-ASCII) · `git show` del blob `89dc913`: **0 líneas de TSan** (`FIND-714` no contaminado).
- **Commit:** `fd52e68a4` (código, 2 archivos) + `e3c2ee81e` (cierre documental, 1 archivo). El cierre fue un commit normal y no un `--amend` porque `FIND-714`/`FIND-718` ya habían commiteado encima: amendar habría reescrito historia ajena.
- **Rollback:** `git revert fd52e68a4` (vuelve al `if:` viejo; el trigger nunca se tocó).
- **Pendiente (owner, post-push):** verificar el primer run real de `WASM Scope (push range)` con `gh api repos/ness-e/Vantadb/actions/runs/<id>/jobs --jq '.jobs[]|select(.name|test("WASM"))|"\(.name) => \(.conclusion)"'`.

### Colisión de `concurrency` job-level que mataba 4 lanes de bench (`FIND-716`)
- **Fecha:** 2026-11-24
- **Objetivo:** `Light benchmarks` y `High density` daban `conclusion=failure` con **cero steps** y `log not found` en **25/25** ejecuciones programadas. Cuatro jobs declaraban un `concurrency:` job-level cuyo group era literalmente `${{ github.workflow }}-${{ github.ref }}` — el mismo que ya retiene el run.
- **Resultado:** ✅ Mecanismo medido, no inferido: `runner_name: none` y `started_at == completed_at`, **8 s** después del start y **65 min antes** de que el run terminara → GitHub no los encola, los **falla al instante**, porque cada job espera un slot que su propio run retiene. Se borraron los 4 bloques redundantes (`light-benchmarks`, `high-density`, `analyze`, `critcmp-regression`; −20/+17) y se dejó **una** nota autoritativa en el workflow-level (`cancel-in-progress: false`) en vez de 4 copias. `failpoint-tests` y `fuzz-pr` conservan su bloque: usan groups distintos y son el patrón correcto.
  - Un efecto secundario que nadie había visto: `Analyze` nunca corrió, así que sus artifacts `light-bench-results`/`density-bench-results` **tampoco existían**.
- **Costo declarado (no disfrazado):** techo de **4.270 min/semana ≈ 71 h** de runner — 7 noches × (240 + 360) min + ~70 min de `analyze`, más 60 min por dispatch de `critcmp`. Repo público ⇒ los minutos no se facturan; el costo real es capacidad concurrente, retención de artifacts y **el riesgo de que aparezca una regresión que el "cero steps" enmascaraba**. No hay duración medida: los lanes nunca ejecutaron un step desde `a4c290914`, así que el número honesto es el cap.
- **Verificación:** `actionlint` exit 0 · `rg '^ {4}concurrency:'` → solo `failpoint-tests` y `fuzz-pr` · `rg '^concurrency:'` → solo el workflow-level · `check-links` 0.
- **Commit:** `9e4e04d70`
- **Corrección del review (P2-01):** el relato original decía "queda pending y se cancela al terminar el run" (falso: mueren al instante) y un costo de "2h + 6h" sin fuente (venía de un comentario pre-consolidación que esta tarea borra). Ambas corregidas en el YAML y en el task file.

### Bench canónico: el baseline no existía y el productor estaba muerto (`FIND-720`)
- **Fecha:** 2026-11-24
- **Objetivo:** El comparador publicaba `stable_count: 0`, `new_benchmarks: [2]`, `has_regression: false`: la detección de regresiones **no podía disparar nunca**.
- **Resultado:** ✅ Causa raíz **distinta de las dos hipótesis de la fila**, medida en los artefactos de los runs `44228761011`/`44228771621`:
  - "el comparador no recibe baseline" → **refutada**: `baseline_commit: "TBH-02-initial-seed"` solo existe si `benchmarks/criterion_baseline.json` se lee; si faltara, `load_baseline()` devolvería `{"description":"empty"}` y el campo diría `"unknown"`.
  - "1536d→100d rompe el matching" → **refutada como causa**: los dos runs son 1536d y aun así dieron 0 coincidencias.
  - **Causa real:** el baseline tiene 9 etiquetas de otra familia (`hnsw_pure/*`, `hybrid_queries/*`, `The Memory Abyss/*`, …) y **ninguna `canonical_p99/*`** (solape 0/2). Y el único caller de `update-baseline` era `certification.yml → analyze`, **uno de los 4 jobs que `FIND-716` mataba con cero steps** → el fichero seguía en `created: 2026-08-30` con `description` "PLACEHOLDER … MUST be refreshed".
  - **Fix:** el job persiste su propio baseline con `actions/cache` **branch-scoped** (así `main`/1536d y `develop`/100d no se contaminan), lo adopta en el path fijo del script, lo siembra solo si no hay nada comparable, y publica `::warning::` cuando no hay comparables. El baseline usado se sube con el artefacto (auditoría Regla 11). El fichero versionado no se edita.
  - **Prueba de que puede disparar:** con el baseline sembrado desde un run real y comparando otro run real → `new_benchmarks: []`, `has_regression: true` (spread real entre corridas **7-9 %**); con delta sintético +50 % → 2 regresiones `critical`, exit 1.
- **Commits:** `aea7b1221` + `5a42181c3`
- **Deuda declarada:** el umbral sigue en **5 %** frente a un ruido medido de **7-9 %** → decisión del owner (plan §9.5). El corpus de `main` sigue en 1536d hasta el próximo `develop → main`. `CI_POLICY.md:646` describe el job sin el baseline.
- **Review P2-01:** no ejecutado (subagente sin tool `task`); waiver registrado y 3 preguntas concretas dejadas en §Review.

### Cobertura de mutación real con presupuesto (`FIND-717`)
- **Fecha:** 2026-11-25
- **Objetivo:** Hasta `3f7fdd3fa` el job `mutation-test` moría en "Set up job" (pin inexistente de `upload-artifact`); y aun con pin válido no medía nada: `--check` significa *"cargo check a cada mutante, sin correr los tests"* y `--output-json` no es un flag, con el exit≠0 tragado por `|| true` → verde sin artefacto. Quedó INFORMATIONAL: **cero señal de calidad de tests por mutación**.
- **Resultado:** ✅ El job ahora corre **los tests** por mutante sobre una superficie fijada y acotada: `cargo mutants --file src/physical_plan/dedup.rs --file src/index/search/pool.rs --exclude-re "<2 supervivientes conocidos>" --in-place --cap-lints=true --build-timeout 900 -- --lib` → por cada mutante recompila el crate y corre `cargo test --lib -p vantadb`. Umbral **0 supervivientes** entre los 7 (exit 2 ⇒ job rojo); **90 min de step / 105 min de job**; guards MIN/MAX de superficie para que el gate no se vacíe ni crezca sin decidirlo. El inventario del crate (12.676 mutantes) se publica **solo como contexto** y el summary imprime `surface` junto a `crate` para que nadie lo lea como cobertura.
  - Medición real del presupuesto: **35 min** con 7 mutantes en caliente, **43 min** con 9, ≈4,4 min/mutante (`--build-timeout 900`; `--timeout` sin fijar = default 5× baseline, 168 s medido).
- **Verificación:** `actionlint` exit 0 · medición local 7/7 muertos en 35 min (exit 0) y 9 probados/2 supervivientes (exit 2) · `check-docs`/`check-links` all clear.
- **Commit:** `e6423f1c7`
- **Decisiones del owner (no tomadas por el agente):** (a) los **2 supervivientes reales** (`dedup.rs:63` close-propagation, `pool.rs:16` buffer reciclado) están **excluidos por nombre** para que el gate no nazca rojo; la alternativa es vaciar `MUTANTS_KNOWN_SURVIVORS` y que el job los reporte en cada corrida; (b) apareció un **test flaky** (`mcp_init::tests::test_brain_profile_degraded_warning_on_probe_timeout`) que rompe el baseline bajo carga y puede fabricar un falso "caught" → fila `FIND-727`.

### Rulesets: el check requerido que no existía y el gate de docs ausente en `develop` (`FIND-715` + `FIND-719`)
- **Fecha:** 2026-11-24
- **Objetivo:** Dos agujeros en la **configuración** del repo, no en los workflows: (a) ambos rulesets exigían un status check llamado `MSRV Check (1.94.1)`, nombre que ningún job reporta desde `5a669a186` → el requisito era **insatisfacible**; (b) ningún job de `docs-gates` era required en `develop`, así que las regresiones de documentación entraban sin que nada se pusiera rojo.
- **Resultado:** ✅ Corregidos por el orquestador vía API, con backup previo en `%TEMP%/opencode/ruleset-*-before*.json`:
  - `23692587` (`develop`): `MSRV Check (1.94.1)` → `MSRV Check (1.95.0)`, y de **10 a 13** checks añadiendo `Internal links resolve`, `Frontmatter schema, kind agreement, orphans` y `Generated indexes are current`.
  - `16844590` (`main`): mismo renombrado, 18 checks preservados.
  - `enforcement: active` y `bypass_actors` intactos en ambos. Verificado por `gh api` posterior.
- **Evidencia del daño que esto causaba:** ese mismo día entraron a `develop` 42 enlaces rotos, 24 violaciones de schema, un secreto en prosa y 33 ficheros dañados por el stamper **sin que nada se pusiera rojo**; se descubrieron mirando el CI a mano.
- **Commits:** sin commit (cambio de configuración del repo, no de código).

### Typecheck y lint de `vantadb-ts` detrás de un gate (`FIND-729`)
- **Fecha:** 2026-11-25
- **Objetivo:** `npx tsc --noEmit` fallaba con 4 errores (`AstJson`, `ListOptionsInput`, `MemoryRecordInput`, `SearchRequestInput`) y `npm run lint` con 6. La causa de los 4 no era código: `vantadb-wasm/pkg/vantadb_wasm.d.ts` (artefacto gitignored, R-2 de `js-ecosystem.md`) tenía 390 líneas —el `.d.ts` crudo de `wasm-pack`— frente a las 956 de la fuente escrita a mano, y nadie corría `dev-tools/build-wasm-types.mjs` en local. La causa de los 6 es **de gate, no de código**: el único job que corre `tsc`+`lint` es `release-npm-wasm.yml:100-110` y dispara **sólo en `pull_request`**, así que los pushes directos a `develop` lo esquivaban.
- **Resultado:** ✅ Tipos regenerados con el script del repo (no a mano); los 6 de eslint resueltos **en código y sin un solo `eslint-disable`** (3 parámetros de paridad con `argsIgnorePattern: "^_"`, 1 import redundante, 2 casts `any` que el `.d.ts` ya tipa). El `pre-push` ahora corre `build-wasm-types --check` (bloque propio con gatillo `^(vantadb-wasm|vantadb-ts)/`), `tsc --noEmit` y `npm run lint` además de `npm test`.
- **Verificación:** `tsc` 0 · `lint` 0 · `npm test` 26 ficheros / 394 · `--check` 0 · `check-links`/`check-docs`/`gen-index --check` 0 · `verify_changed.ps1` ALL 5 PASS. Hook probado en worktree descartable con push simulado: rango `vantadb-ts/` corre las 4; rango `vantadb-wasm/` solo corre el `--check`; `.d.ts` desincronizado, `TS2322` y var sin usar → **BLOCKED** con mensaje accionable.
- **Coste añadido al `pre-push`:** ≈ **12 s** (1,3 + 4,4 + 6,5); 1,3 s en un push que solo toca WASM.
- **Commit:** `ecc2df930`
- **Deuda:** el agujero de fondo sigue en `release-npm-wasm.yml` (gatillo `pull_request` sin `push: branches: [develop]`).

### La aserción del corpus TSan no verificaba nada (`FIND-730`)
- **Fecha:** 2026-11-25
- **Objetivo:** El step `Assert the TSan partition was actually exercised` salía `success` con `SUMMARY` vacío (`partition 1/4:  (selected == executed)`) en el run `45564374043`: nunca parseó un número, así que sus dos guardas (aborto `X/Y`, selección `0`) eran inertes.
- **Resultado:** ✅ Tres causas encadenadas cerradas: (1) el log del `tee` trae ANSI (`<ESC>[1m821<ESC>[0m tests run`) → se limpia con `sed 's/\x1b\[[0-9;]*m//g'`; (2) el step corría como `/usr/bin/bash -e {0}` **sin `pipefail`** (el comentario del YAML afirmaba lo contrario) → `shell: bash` + `set -euo pipefail` **en el propio `run:`**, sin depender de herencia; (3) `[ "" -eq 0 ]` dentro de un `if` está exento de `-e` y caía al `echo` verde → `[ -z "$SUMMARY" ]` con `::error::` y `exit 1`. Extra del review: el conteo se extrae **solo** de la línea de `Summary` en dos etapas, para que un test que imprima `N tests run` en la sección de FAILURES no gane el `tail -1`.
- **Verificación:** `actionlint` 0 · script extraído del YAML (no una copia) contra **6 fixtures**: log real reconstruido de la partición 1/4 (`821 tests run`, exit 0), log vacío (exit 1), log sin summary (exit 1), `0 tests run` con ANSI (exit 1), control con `set -eu` (exit 1) y **señuelo `9999 tests run` al final** → sigue dando 821. `gen-index --check` 0 · `check-links` 0 · `check-docs` all clear.
- **Commit:** `98e752079`
- **Deuda declarada:** el caso `X/Y` (aborto) sigue sin fixture real (ninguna partición aborta desde `FIND-714`); `suppression hits: 0` en la corrida real, así que las dos entradas de `tsan-suppressions.txt` podrían estar muertas y se evalúan con el run verde completo; el `sed` es GNU → en macOS fallaría cerrado.
- **Nota de método:** probar el parser contra el *job log* da falso positivo (incluye el eco del script y las anotaciones `##[error]`). El fixture fiel es la sección del step de test, sin timestamp y con ANSI.

### perf-bench cross-VM: decisión (c) aceptar el ruido, atribución declarada imposible (`FIND-233`)
- **Fecha:** 2026-11-25
- **Objetivo:** Atribuir el delta de +10-19 % post-0.8.0 y decidir calibración, dado que el instrumento medía cross-VM con un factor de 1,7x-2,4x.
- **Resultado:** ✅ Decidido con números, **sin coste recurrente** (0 min, 0 líneas de código). Hallazgos que corrigen la premisa de la fila:
  - **`perf-bench.yml` solo existe en `main`**: se borró de `develop` en `255db8208` (de-bloat, 2026-10-25) junto con `compare_baseline.py` y `python_baseline.json`.
  - **El A/B same-job nunca se invocó**: `total_count: 1`, `event: push`, **0 dispatches**. Implementado en `11ab6c3c` y jamás usado.
  - **Los 4 runs de evidencia de `FIND-232` devuelven 404** (`37088714140`, `37089421873`, `37089581213`, `36093538630`): artefactos y logs borrados. La evidencia original ya no es re-derivable.
  - **Suelo de ruido medido** del único log sobreviviente (`40592702083`, 3 sub-runs, **una** VM, **un** commit): `query_hybrid.p50` **34,2 %** / `p95` **42,2 %**, `query_vector.p50` **16,7 %** / `p95` 13,2 %, ingest 5,8 %. El delta a atribuir (+15,7 %/+19,3 % vector, +10,7 %/+8,5 % hybrid) **cabe dentro del ruido de una sola máquina** → **no atribuible** (ni al código ni a la VM).
  - **Calibración CPU rechazada**: ≈+1 min/corrida recurrente (+15-22 % del job) y normaliza solo 1 de 3 regímenes (`insert.p99` es fsync-bound 16,8x y todo cruza PyO3); además los runs multi-VM contra los que se validaría están borrados.
- **Escalada opcional documentada (sin código):** 1 dispatch en `main` con `ab_ref=114f55f0`, 11-15 min, comando en `BENCHMARKS.md` §20.
- **Commits:** `5eaf978c8` + `79d6ca1ca`
- **Review:** P2-01 **degradado** (worker leaf sin fork a reviewer); waiver registrado (owner=orquestador, ref=plan §6.c). `verify_changed.ps1` no completó por lock de `target/` con dos workers Rust en paralelo — gate Rust no aplicable a un diff docs-only.
- **Colaterales registrados:** `docs/specs/SPEC.md:140` describe el gate como operativo cuando ya no existe en `develop`; la banda *warn* (15 %) está por debajo del suelo intra-job y no tiene piso absoluto.

### El `pre-commit` ya no stashea un árbol compartido (`FIND-726`)
- **Fecha:** 2026-11-25
- **Objetivo:** Al repararse el hook (`FIND-718`) empezó a correr `pre-commit`, que **revierte los ficheros tracked a su contenido en el índice** antes de lanzar los hooks y luego reproduce el diff. Con 2-5 agentes en el mismo checkout eso **perdió trabajo ajeno tres veces confirmadas** (`FIND-720`, `FIND-723`, `FIND-721`), una de ellas sin restaurar.
- **Resultado:** ✅ Causa raíz identificada en `pre-commit`: `stash = not args.all_files and not args.files`. El hook **pasa ahora la lista staged con `--files`**, que es el único lever documentado (`pre-commit run` no tiene `--no-stash`), reimplementa los guards que solo corrían en modo stash (unmerged, config con cambios unstaged), avisa qué ficheros tracked deja intactos y avisa de los *partially staged*.
  - **Reproducción determinista del daño** (clon descartable): una escritura concurrente a los **900 ms** de la ventana dejaba `AGENT1_PRESENT: False` con el log diciendo `Restored changes`. Tras el fix: `STASH_LINES: 0` y hash idéntico antes/después.
  - **Opción (b) abortar rechazada con medida**: el WIP ajeno es el estado **normal** de este checkout (18 ficheros al cerrar), así que abortar bloquearía todo commit y empujaría a `--no-verify` = cero gate. (d) worktree temporal evaluada y descartada por maquinaria por commit; queda como salida si el aviso de *partially staged* no bastara.
  - **El review cazó un agujero que reintroducía el bug (F1):** la guarda miraba la lista staged sin `--diff-filter` mientras el pipe llevaba `ACMRTUXB`, así que con **solo borrados** staged `xargs` ejecutaba `pre-commit run --files` con **0 argumentos** → `args.files == []` → `stash = True`. Corregido con guarda sobre la lista filtrada + `xargs -0 -r`. También corrigió dos afirmaciones falsas de la propia doc y un `set -e` que abortaba el hook entero con un `grep` rc=1.
- **Verificación:** hash idéntico en la ventana · escritura concurrente preservada · solo-borrados sin stash · árbol limpio commitea en verde · junk sigue bloqueado (`RC=1`) · en `develop` **18/18 ficheros ajenos `SAME` (`DRIFT_COUNT: 0`)** con el bit `100755` intacto.
- **Commits:** `5d293d759` (fix inicial) + `c268eea59` (F1) + `49d5cff56` (veredicto)
- **Deuda:** `command -v pre-commit` no resuelve `.exe` en bash; drift preexistente de `gen-index` en `HEAD`; **no hay 2ª ronda de review** (declarado, no fabricado).
- **Incidente de proceso:** su `--amend` cayó sobre el commit de `FIND-233` y lo reescribió; lo revirtió con `reset --soft`. Misma clase que el incidente de `F0.2`. Regla adoptada: **`--amend` prohibido mientras haya más de un agente vivo**.

### Banda de regresión del bench: de 5 % a 10 %, por encima del ruido medido (`BENCH-BAND`, decisión §9.5)
- **Fecha:** 2026-11-25
- **Objetivo:** El comparador del bench canónico tenía **5 %** de tolerancia declarada mientras el ruido medido entre corridas era de **7-9 %**. Con esa banda, cada corrida habría avisado de "regresiones" que eran ruido — y en dos semanas nadie miraría el aviso.
- **Resultado:** ✅ `REGRESSION_THRESHOLD = 10.0` en **`scripts/bench_regression.py:52`**, único sitio: ningún workflow pasa `--threshold` (`nightly.yml:563-567`, `certification.yml:670-675`) y **no se le añade**, porque el flag explícito sería la segunda copia del umbral (la clase de bug de `FIND-704`).
  - **Fuente primaria (mismo instrumento):** `canonical_p99`, misma rama, dos corridas reales separadas 5h20m — `insert` 766.984,4 → 698.162,6 ms (**−9,0 %**) y `search` 4.260,7 → 3.958,1 ms (**−7,1 %**); en base max/min **9,9 % / 7,7 %**. La banda queda **al borde de su propia fuente**, y así está escrito.
  - **Corroboración (otro instrumento, no la misma medición):** run `40592702083`, 3 sub-runs, una VM, un commit → `query_hybrid.p50` **34,2 %** / `p95` **42,2 %**, `query_vector.p50` **16,7 %**.
  - **Una banda única es una simplificación:** el comparador tiene un `threshold` global y hoy solo compara las dos familias de `canonical_p99` (mismo instrumento). Si algún día midiera las familias de §20 (16,7-42,2 % intra-VM), 10 % avisaría por ruido. Anotado en §11, §20 y en la constante.
- **Verificación:** prueba mecánica con sintéticos en `%TEMP%/opencode/bench-band/` (copia del script, baseline sintético; el fichero versionado no se tocó): **8 % → `has_regression: false`, `stable_count 2`, exit 0** (antes: 2 warnings, exit 1) · **20 % → `has_regression: true`, exit 1** · control con los números reales de `FIND-720`: `--threshold 5` → `stable 0` + 2 "improvements"; default 10 → `stable 2`. `actionlint` 0 · `check-links` 0 · `check-docs` all clear · `gen-index --check` 0.
- **Commit:** `eb2e52ee0`
- **Techo conocido (escrito en la constante y en §11/§20):** un **8 % real** ahora se lee como estable, y en `certification.yml` job `analyze` el `has_regression` **gatea la promoción del baseline** → ese 8 % se hornearía en vez de marcarse. Es el precio de una banda que significa algo.

### Retiro de la lane TSan, con receta de revival medida (`FIND-734`)
- **Fecha:** 2026-11-25
- **Objetivo:** `FIND-714` hizo que la lane **midiera** (822 tests, `selected == executed`, sin abortos) y `FIND-730` que verificara; lo que quedaba era interpretar el resultado: `race reports: 243` con `suppression hits: 0`.
- **Resultado:** ✅ **Retirada**, con la razón y el coste escritos. Composición de los 243 reportes del run `45668284667`: **176** del harness de libtest (`Arc<Mutex<Option<test::run_test::{closure#0}>>>` vs `compiler_move`), **58** de teardown fjall/flume (`free` vs `Atomic<usize>::fetch_sub`), **8** de tokio (7 shutdown, 1 spawn) y **1** de `OnceLock<DistanceKernels>` (`kernels.rs:275`).
  - **El argumento decisivo:** en los 243 el par en conflicto está siempre en `std`/`core`/`alloc` **sin instrumentar** (no hay `-Zbuild-std`), así que TSan no puede adjudicar ninguno. Y el warn #42 alcanza `src/index/distance/kernels.rs:275` **sin frame de harness ni de dependencia**, o sea indistinguible de un defecto real — pero taparlo exigiría nombrar un tipo del motor. Las 2 entradas de `FIND-714` se cargaron y dieron **0 hits** en 24 h: la lista caduca más rápido que un nightly.
  - **Retiro:** job `sanitizer-tsan` eliminado (borrado puro `+0/-227`), `dev-tools/tsan-suppressions.txt` borrado, `CI_POLICY.md` §TSan reescrita como nota de retiro con receta.
- **Commit:** `d37cd39b3`
- **El hueco no queda anónimo** (tabla completa en `FIND-734.md` §"De qué cubre el hueco"): `vanta-chaos` (deadlocks, corrupción bajo carga, quiescencia WAL/snapshot) · `memory_concurrency` + tests `concurrent_*` (invariantes de HNSW bajo escritura concurrente) · `FIND-328` (mutation testing) · `miri` (UB en tests etiquetados). **Lo que se pierde concretamente: la única detección estática respecto al timing** — un race silencioso que no cambia ningún observable.
- **Revival (fila `FIND-737`, 🟠 Alta):** `rustup component add rust-src --toolchain nightly` + `RUSTFLAGS=-Zsanitizer=thread` + `-Zbuild-std --target x86_64-unknown-linux-gnu`. **Coste medido, no estimado: 3,2 min y 164 MB** para compilar `std` instrumentado (`rustc 1.101.0-nightly`, caché frío, sin sccache). Criterio de cierre: `race reports: 0` **fuera de `src/`**; un reporte **dentro** de `src/` sin frame de `std` es hallazgo real y **bloquea**.
- **Deuda declarada:** `docs/dev/workflow/ci-rust-10.md` presenta TSan como lane viva en cuatro sitios (`:43`, `:50`, `:66`, `:72`) — y `:72` presenta la omisión de `-Zbuild-std` como optimización deliberada, que es la causa raíz.
- **Incidente de proceso:** el borrado de `dev-tools/tsan-suppressions.txt` (staged con `git rm`) lo barrió el commit del orquestador `29f1ced89`, que no usó pathspec. Inocuo en efecto (el borrado era intencionado y queda declarado aquí), pero **regla nueva: el orquestador commitea siempre con `-- <rutas explícitas>`**; `FIND-726` protege los cambios unstaged, no los staged de otro carril.

### El e2e L0→L3 corre a diario: cierre de la recomendación del review de `MEM-73`
- **Fecha:** 2026-11-26
- **Objetivo:** el e2e compuesto (`vantadb-mcp/tests/l0_l3_cycle_e2e.rs`) era `#[ignore]` + carril **manual** (`dev-tools/e2e-memory-cycle.ps1`). Un e2e que sólo corre cuando alguien se acuerda se rompe en silencio y nadie lo nota hasta que hace falta.
- **Resultado:** ✅ Job `memory-cycle-e2e` en `nightly.yml` (el lane programado que **ya existía**, 05:00 UTC / HARD-02 — no se inventó workflow). Reusa el guion como **punto de entrada único** (build → gate de SHA de `FIND-741` → e2e): duplicar los comandos en YAML crea una segunda implementación que derive, y sin el gate de SHA el job verificaría un binario stale — que es exactamente lo que `FIND-741` existe para impedir.
  - **Sin `continue-on-error`** (verificado ausente): si falla, el ciclo compuesto está roto; se arregla el test, no se enmascara el job.
  - **`if: github.event_name != 'pull_request'` es load-bearing:** `nightly.yml` tiene `pull_request:` con `paths: src/**`, así que sin ese gate el e2e correría en **cada PR de core** — la clase que envenena gates rápidos (`FIND-727`/`FIND-739`).
  - **Linux solo**, `timeout-minutes: 30` con ancla **medida**: 525,4 s de wall en corrida local caliente (el ciclo son 1,45 s; el resto es build de los 28 targets que nextest compila antes de filtrar). ~3,4× ese piso, y a la par del techo de `failpoint-tests`.
  - **Aviso ya existente:** el job entró en `needs` de `notify-failure` (`if: failure()`), que abre o comenta el issue deduplicado `[Nightly] Heavy certification suite failed` con labels `ci`. No hubo que construir nada; lo que **no** existe es un humano asignado.
- **Verificación:** `actionlint` exit 0 · PyYAML `safe_load` (segundo parser): 16 jobs, `timeout: 30`, `continue-on-error` ausente, `if` correcto, `notify-failure` lo incluye · `check-docs` GATING all clear, 0 orphans · `check-links` 0 · markdownlint 0 · corrida local del comando del job: `1 passed`, `L0→L3 in 1.45s`, exit 0 · review P2-01 `approve` (2 required + 1 opcional aplicados).
- **Commit:** `33ab11c37`
- **Techo declarado:** **no se pudo disparar el `schedule`** desde el entorno; se verificó que el job vive en el workflow con `cron`, que su `if` lo saca de `pull_request`, y que el comando es el mismo que corre en local. **El `schedule` corre sobre el default branch (`main`), no sobre `develop`**: la verificación diaria es del artefacto de `main`, así que mientras `develop` no se mergee, el e2e diario prueba lo que `main` tiene, no lo último. Ajustar el `timeout` tras la primera corrida programada.
- **Nota:** `gen-index --check` reporta drift en `docs/index.md` (782 → 785 documentos) por los task files nuevos de la ola; se regenera en el cierre de wave, no en este commit.


### El `pre-push` juzga el rango que viaja, y el commit deja de usar el índice compartido (`FIND-760`, `FIND-753`)
- **Fecha:** 2026-11-26
- **Objetivo:** dos fallos de **proceso** que frenaban cada push con 4 carriles vivos, tres veces cada uno el mismo día. (1) `FIND-760`: el `pre-push` corría `dev-tools/verify.ps1` **siempre**, y su primer step es `cargo fmt --all -- --check` sobre el **árbol de trabajo**. El push de `FIND-152` (2 commits, ambos fmt-clean: `cargo fmt --check -p vantadb-wasm` → 0) fue **bloqueado** por WIP sin formatear **de otro carril** (`vanta-memory/src/core/conversation/consolidation.rs`, `vantadb-mcp/src/memory_scheduler.rs`); y el hook **no tenía `SKIP`**, así que la única salida era `--no-verify`, que apaga todos los gates. (2) `FIND-753`: `Unable to create .git/index.lock` 8 intentos seguidos; un commit concurrente horneó una copia **stale** de `Backlog.md` y una fila borrada **reapareció en HEAD** (exit code 0); el índice quedó con staged de otro carril (`tests/cli_tests.rs`, −92 líneas) que un `git commit -a` habría borrado.
- **Resultado:** ✅
  - **`FIND-760` — el gate mide lo que viaja.** `dev-tools/fmt-range.ps1` corre `rustfmt --check` **por fichero del rango** (los shas que git entrega por **stdin** — equivale a `git diff --name-only @{u}..HEAD` pero también es correcto para ramas nuevas y pushes multi-ref), `--edition 2021` (todo el workspace es 2021) y `--config skip_children=true`. Lo de `skip_children` es **medido, no supuesto**: sin él rustfmt entra en los `mod` hijos y reporta diffs de ficheros **fuera** del rango (sonda en `%TEMP%\fmtprobe`: `lib.rs` con `pub mod child;` reportaba también `child.rs`; con la opción, solo `lib.rs`). `check`/`clippy`/`audit`/`deny`/`nextest` **siguen siendo de árbol y se dice por qué**: una crate compila como unidad, un test de integración comparte estado y ningún fichero es su dueño — acotarlos por fichero dejaría el gate midiendo menos de lo que promete. Esos gates corren **solo si el rango toca Rust**; para el WIP ajeno está el `SKIP`.
  - **`SKIP` granular, con el idioma de `pre-commit`.** `SKIP=<id>[,<id>…] git push`, con ids que **reusan** los de `pre-commit` (`cargo-fmt`, `cargo-clippy`) más uno por gate del hook (`nextest`, `markdownlint`, `ruff`, `npm`, `ts`, `unix-check`, `wasm-types`, `providers-fmt`, `actionlint`, `docs-coverage`, `cli-probes`, `consumo-guard`, `backup-verification`). El gate saltado **se nombra** (`pre-push: SKIP: gate 'cargo-fmt' skipped … NOT verified`) y `verify.ps1` repite el aviso al final (`WARNING: N step(s) SKIPPED … this run is NOT a full gate`). Con `CI` definido, `SKIP` **se ignora** (pre-mortem: un `SKIP` exportado en un workflow apagaría gates sin que nadie lo note).
  - **Dos fail-open cerrados por el camino, ambos medidos:** (a) un sha con `\r` (o un sha inventado) hacía fallar `git diff` en silencio, el rango quedaba **vacío** y el hook **pasaba midiendo nada** → ahora un `diff` que falla bloquea y nombra el rango; (b) `rustfmt` no está en el PATH del shell del hook (Git Bash/WSL: `command -v rustfmt` → nada, medido bajo WSL bash aunque exista en `%USERPROFILE%\.cargo`) → el chequeo vive en PowerShell, que resuelve `~/.cargo/bin` igual que `verify.ps1`, y `pwsh`/`powershell` se resuelven **una vez** al principio en vez de fallar con `pwsh: command not found` en el primer gate.
  - **`verify.ps1 -Skip <id>`** es **aditivo** (sin argumentos, los diez steps de siempre) y acepta `-Skip fmt,check` y `-Skip "fmt,check"`. Con el rango sin Rust, el hook pasa `-Skip fmt,check,clippy,audit,deny,nextest,cli-probes,consumo-guard` y **conserva** `docs-coverage` y la verificación del runbook de backup, que sí dependen de docs. El `fmt` de árbol ya lo cubre CI (`ci-rust.yml:123`).
  - **`FIND-753` — política + herramienta.** `RULES.md` §10 (el gate juzga el rango) y §11 (índice privado + verificación por contenido), con el sesgo explícito: **el exit code no es evidencia**, un commit concurrente revierte con exit 0. `dev-tools/git-commit-isolated.ps1` hace `read-tree HEAD` → `add <rutas>` → `commit` en un `GIT_INDEX_FILE` temporal y después **relee cada fragmento desde `HEAD`** (`-ExpectAbsent`/`-ExpectPresent`), fallando si el commit no llevó la edición. Documentado **cuándo NO usarlo**: `git add -p` interactivo, merge commits (`MERGE_HEAD`), y commits que deben arrastrar lo que ya está staged a propósito en el índice compartido.
- **Medición — prueba de dos caras** (mismo `.rs` sin formatear, `pub fn probe( ){let x=1;…}`):
  - **(a) WIP ajeno FUERA del rango → el push ENTRA.** Rango docs-only (`ebc83ffa9..60007a472`, 1 fichero) con el `.rs` sin formatear en el árbol: `pre-push: 1 archivos en el rango` → `pre-push: no .rs files in the range — fmt gate has nothing to check` → `pre-push: barrier passed. Push allowed.` · **exit 0**.
  - **(b) fichero DEL rango sin formatear → el push NO ENTRA, nombrando el fichero.** Commit de prueba local con ese `.rs`: `pre-push: fmt sobre 1 fichero(s) .rs del rango (no el arbol)` → `fmt-range: FAILED on range file: dev-tools/zz-fmt-in-range-probe.rs` → `pre-push: fmt gate FAILED (exit 1). Push BLOCKED.` · **exit 1**. Después `git reset --soft` (con índice privado) y el árbol volvió a limpio.
  - **Verificador por contenido, prueba negativa:** el helper commiteó `4f70c46a4` y con `-ExpectAbsent …::NEEDLE-753` (fragmento **presente**) devolvió `REVERTED: 'NEEDLE-753' is STILL in HEAD:…` + **exit 1** — el chequeo no es decorativo.
  - `bash -n .githooks/pre-push` → **0** en cada iteración (**siete** veces, incluida la que precedió a cada commit).
- **Verificación:** `bash -n` 0 · `verify.ps1 -Skip …` → steps saltados **nombrados** + `WARNING: 8 step(s) SKIPPED` + `docs-coverage ok` + `daily backup verification ok` · `git show HEAD:docs/dev/Backlog.md | rg 'FIND-753|FIND-760'` → **0** · `git show HEAD:.githooks/pre-push | rg skip_gate` → **11** · `git show HEAD:dev-tools/zz-fmt-in-range-probe.rs` → *path does not exist* (la sonda no queda en el árbol) · modo del hook `100755` preservado · `git diff --cached -- <rutas>` vacío tras el commit · review P2-01 por `vanta-review`.
- **Commit:** `cff8fa626`
- **Hallazgo que la herramienta causó y la regla ahora cubre (medido en `cff8fa626` → reparado en `30fdc9c42`):** con `GIT_INDEX_FILE`, `.git/index` **no se actualiza**; justo después del commit, `git diff --cached` mostraba el commit **entero** como staged-para-revertirse — cualquier `git commit` sin pathspec de otro carril lo habría revertido, o sea el fallo de `FIND-753` causado por el remedio de `FIND-753`. Corregido en la regla §11 (el `git add` final es obligatorio) y en el helper, que ahora refresca el índice compartido **solo para sus rutas** después de commitear.
- **Y el fallo me cazó a mí, con la cadena completa visible (esto es la evidencia más fuerte de la fila):** `cff8fa626` construyó el blob de `Backlog.md` desde `HEAD`, pero otro carril commiteó entre construir y commitear (TOCTOU) → el commit horneó la copia anterior y **borró 9 líneas ajenas**: mi bloque `FIND-753`, mi bloque `FIND-760` y la fila `FIND-761` que el otro carril acababa de crear. **Exit code 0.** La verificación por fragmento pasó porque solo miraba mis propias marcas. Lo detectó el otro carril comparando `git hash-object` contra `git rev-parse HEAD:` y lo reparó en `30fdc9c42` (9 restauradas, 0 adiciones). La regla §11 incorpora ahora los dos chequeos que lo cazan: `git rev-parse <commit>^` == base capturado, y `git diff <base>..HEAD -- <fichero>` con **solo tus hunks y 0 adiciones** (`git diff --numstat`). También queda documentada la variante para un fichero que otro carril está editando: el contenido sale de `HEAD` + tu edición (`git hash-object -w` + `git update-index --cacheinfo`), nunca del working copy. El segundo intento (cierre) se hizo con base capturada y delta verificado.
- **Review P2-01 (contexto fresco, `vanta-review`) → `changes-required`, 2 blocking, ambos reales y corregidos:**
  - **B-1 (`SKIP` multi-id era un no-op silencioso).** `is_skipped` hacía `tr ',' '\n' | tr -d '[:space:]' | grep -qxF`: `[:space:]` incluye el `\n` que se acababa de introducir, así que la lista se **concatenaba** y sólo matcheaba con **un** id. Un `SKIP=a,b` —el caso real, porque un bloqueo casi nunca es de un gate— no saltaba **ninguno** y no avisaba. Corregido con `tr -d '[:blank:]'` + `case ",$lista," in *",$id,"*)`. **Medido en las dos formas:** `SKIP=cargo-fmt,cargo-check,cargo-clippy,cargo-audit,cargo-deny,nextest,unix-check,cli-probes,consumo-guard` → **9 líneas** `pre-push: SKIP: gate '…' skipped … NOT verified`; id único (`markdownlint`) → `SKIP: gate 'markdownlint' skipped` y **sin** `Linting:`.
  - **B-2 (el refresco del índice compartido mentía).** El post-paso `git add` no miraba `$LASTEXITCODE`: con `Unable to create .git/index.lock` (el disparador que este mismo archivo documenta) imprimía "shared index refreshed" y salía **0**, dejando exactamente el estado que la regla §11 llama catastrófico. Ahora falla con **exit 3**, imprime el comando de reparación y avisa que un `git commit` sin pathspec de otro carril puede revertir el commit.
  - **Guarda `CI`, medida en las dos direcciones:** con `CI=1` y `SKIP=markdownlint`, `pre-push: SKIP='markdownlint' IGNORED — CI is set` y markdownlint **corre igual** (`Linting: 389 files`); sin `CI`, el mismo SKIP salta el gate y lo **nombra**.
  - **Non-blocking corregidos:** `backup-verification` faltaba en la lista de ids del header · `verify.ps1` imprimía `ALL 0 PASS` en verde cuando se saltaba todo (ahora `PARTIAL n PASS / m SKIPPED`) · `temp-collector.ps1` se invocaba sin `-ExecutionPolicy Bypass` · `base` vacío daba `git diff "..sha"` = `HEAD..HEAD` (rango válido y vacío) → `[ -n "$base" ] || base=$local_sha` · `fmt-range.ps1` leía la lista con encoding ANSI bajo PS 5.1 (`-Encoding utf8`) · el ejemplo de §10 usaba `@{u}` en vez de los shas del stdin.
- **Corrección de un claim:** "el diff neto `@{u}..HEAD` no incluye la sonda" es cierto con `@{u}` = `60007a472` (**0 hits de la sonda en 34 ficheros** del rango), **pero** la sonda aparece como `D` en cualquier rango que empiece **después** de `c6fa9f102` (`f7521c583..HEAD` → `D dev-tools/zz-fmt-in-range-probe.rs`). Efecto: ese rango pone `RS_CHANGED=1` y corre los gates Rust de árbol — mide **de más**, nunca de menos.
- **Techo / deuda declarada:** (1) el índice privado **reduce** la ventana de carrera a `add`+`commit` y **detecta** la reversión; **no la impide** — para eso haría falta un lock de wave, que no existe. (2) `clippy`/`nextest` siguen midiendo la crate entera por diseño: un WIP rojo en `-p vantadb` bloquea el push de un cambio que toque Rust, y la salida es `SKIP=<gate>` (nombrado) o esperar. (3) **Dos commits de sonda quedaron en la historia** (`c6fa9f102` = `.rs`, `4f70c46a4` = `.md`) porque el otro carril commiteó dos veces encima durante la prueba y reescribir historia bajo un carril vivo con índice staged es más peligroso que dos commits de 1 línea; los ficheros **no están en HEAD** (verificado con `git show HEAD:<path>` → *does not exist*), así que un push no los mide: en `@{u}..HEAD` no aparecen, y en un rango posterior sí como `D`. Squash recomendado en el cierre de wave. (4) `b4f70a75b` quedó huérfano en el reflog (era un commit accidental del otro carril que solo borraba la sonda `.md`).

### El gate de rustdoc queda cableado: dos steps, una superficie cada uno (`FIND-763`)
- **Fecha:** 2026-11-26
- **Objetivo:** `FIND-755` y `FIND-761` dejaron las dos superficies de rustdoc en 0 diagnósticos y **nadie las ejecutaba**. El job `rustdoc` corre `cargo doc` **sin** `RUSTDOCFLAGS`: genera el tarball y no juzga nada (medido: exit 0 con 36 warnings), su comentario afirmaba lo contrario, y su context no estaba en `REQUIRED`. Verde no es durable sin el gate cableado — `FIND-755` encontró `vanta-memory` roto **dos horas después** de su propio cierre.
- **YAML final** (step nuevo en el job `rustdoc` de `docs-gates.yml`, entre `rust-setup` y `Generate rustdoc`):
  ```yaml
  - name: Deny rustdoc warnings (doc gate)
    env:
      RUSTDOCFLAGS: "-D warnings"
    run: cargo doc --no-deps --workspace --keep-going
  ```
  **Sin** `--all-features`, **sin** `--document-private-items`, **sin** `continue-on-error`. `env:` en el **step**: en el job se heredaría al build del artefacto y cambiaría su semántica. Va **antes** porque no comparte fingerprint (default vs `--all-features`). El step del artefacto conserva su comando; sólo se corrigió su comentario falso, que es lo que hizo creíble el agujero durante tres tickets. Dos steps porque son dos superficies: los 36 lints de `--document-private-items` que cerró `FIND-761` no son los de la vista pública.
- **La coincidencia literal del `context`** (el fallo de `FIND-714`/`FIND-715`: un nombre que ningún job reporta no se satisface **nunca**):
  ```
  docs-gates.yml  jobs.rustdoc.name  -> "Generate API reference (rustdoc)"   32 chars
  ci-gate.yml     REQUIRED[...]      -> "Generate API reference (rustdoc)"   32 chars
  ```
  Comparado carácter a carácter por code points (idénticos) y por parseo de los dos YAML, no a ojo. `REQUIRED` pasa de 18 a **19** entradas porque el header se corrigió en el mismo commit: **18 = ruleset de `main`** (`16844590`, verificado por API ese día) **+ 1** que no es merge gate sino sanidad de `main` (esta fila).
- **Alcance: el job corre en `develop`, pero NO es un check required ahí.** `docs-gates.yml` dispara en `push: branches [main, develop]` (más `pull_request` a ambas y `schedule`), y el `if:` del job no tiene guard de rama — sólo excluye `dependabot` y `schedule`. Consecuencia medida en el YAML, no supuesta: **un push a `develop` ejecuta el gate y se pone rojo**. Lo que este cambio **no** hace es volverlo bloqueante en `develop`: `Generate API reference (rustdoc)` no está en el ruleset `23692587` (13 contexts, 3 de `docs-gates`) ni en `16844590` (18). Los rulesets son configuración del repo vía API, no ficheros — un agente no los toca.
- **Corrección de una premisa heredada (3 filas la repetían):** `FIND-719`/`FIND-755`/`FIND-761`/`FIND-763` decían *"ningún job de `docs-gates` es required en `develop`"*. **Es falso desde el 2026-11-24**: el §Rulesets de arriba registra cómo `FIND-719` se cerró añadiendo `Internal links resolve`, `Frontmatter schema, kind agreement, orphans` y `Generated indexes are current` al ruleset de `develop`. Verificado hoy por API. Lo que **sí** queda abierto —y es esta fila— es que el job `rustdoc` concretamente no es required en **ninguna** de las dos ramas, así que en `develop` el rojo se ve pero no bloquea.
- **Verificación (no se puede disparar CI por contrato):** `actionlint` **exit 0** en los dos ficheros (baseline previo también 0; binario local v1.7.12, CI pinea 1.7.7) · parseo de los dos YAML: `jobs.rustdoc.steps[2]` es el step nuevo y `Generate rustdoc` es `[3]`; `step.env` = sólo `RUSTDOCFLAGS`; `jobs.rustdoc.env` sin `RUSTDOCFLAGS`; 16 jobs intactos · `REQUIRED` 19/19 · `git diff` = el step + 1 línea del array + comentarios (0 cambios de semántica en el artefacto) · `check-links` 0 broken · `check-docs` **GATING all clear**.
- **Riesgo residual declarado:** si un push a `main` no toca ninguna `paths` de `docs-gates` (p. ej. sólo `Formula/`), no hay check-run y `ci-gate` lo tolera con `WARN` por la política EST-03 (`:59-95`): nunca-verde ≠ rojo, así que **no hay falso rojo**; sí hay un falso verde posible, el mismo que ya aplican los otros 18 contexts.
- **Techo declarado del step:** mide la superficie **default**. Un `pub` que sólo exista bajo una feature no-default (`remote-inference`, `embed-local`, `hot-reload`) **no** tiene gate propio aquí. La combinación estricta (`--all-features --document-private-items`) está medida verde en `FIND-761`, pero meterla en cada PR cuesta rocksdb + bindgen (`FIND-761` §Incógnitas 4); queda como upgrade explícito, no como olvido. Y hay un hueco de forma: `on.push.paths` de `docs-gates.yml` no incluye `web/**` — inofensivo en `develop` (ese árbol vive en el worktree `develop-desktop`), pero con 3 jobs de `docs-gates` required, un PR que sólo toque `web/` se quedaría en *Expected* para siempre (clase `FIND-714`). Es preexistente y **no se toca en esta fila**.
- **Review P2-01 (contexto fresco, `vanta-review`, report-only) → `approve`.** Re-corrió todo por su cuenta: `git diff` (2 hunks por fichero, semántica del artefacto intacta), `actionlint` exit 0, parseo de los dos YAML (`idxNew=2 < idxArtifact=3`, `env` sólo en el step, `REQUIRED` 19 con `len 32 == 32` y `dupes: []`), `REQUIRED − {rustdoc}` == ruleset de `main` **18/18** sin faltantes, y el alcance en `develop` contra los rulesets reales. **0 Blocking.** Quedaron 3 Medium y 3 Low, todos fuera del fichero que esta fila puede escribir salvo dos: **L-1** (mi task file decía "15 jobs" y "672 líneas" sin marcar que los `:NNN` eran de `HEAD`) → corregido a **16 jobs** y referencias anotadas como `HEAD`; **L-3** (el step sólo mide la superficie default) → declarado como techo arriba. **M-1** (`on.push.paths` sin `web/**`), **M-3**/**L-2** (`FIND-761.md:381-382,401-403` siguen afirmando la premisa falsa y que la fila ya está en el Backlog) quedan **reportados al orquestador**: `FIND-761.md` no está en la superficie escribible de esta fila.
- **Commit:** este commit (2 workflows + task file + esta entrada + fila del Backlog retirada)

### El workflow que no dispara no reporta: `paths` fuera de `pull_request` en `docs-gates` (`FIND-765`)
- **Fecha:** 2026-11-26
- **Objetivo:** `docs-gates.yml` filtraba `pull_request` por `paths`, y un workflow que no arranca **no reporta ningún check-run**: sus contexts required quedan en `Pending` para siempre. Tres de los 13 contexts de `develop` (`23692587`) salen de este fichero, así que cualquier PR que solo tocara un path no listado era **inmergeable para quien no puede saltarse el ruleset** (un admin sí puede: `bypass_actors: [{actor_id: 5, RepositoryRole, bypass_mode: always}]`, y de hecho 4 PRs solo-docs entraron sin ningún run) - la clase de `FIND-714`/`FIND-715`, pero en la capa del disparador en vez de la del nombre. Vivo hoy: PR `#285`, `statusCheckRollup: []`.
- **La premisa de la fila era falsa y el hueco era mucho mayor.** La fila decía `web/**`; **`web/` no existe en ninguna rama** - ni `develop`, ni `origin/main`, ni las 18 `origin/*`; solo sobrevive en `refs/stash` y en los tags `v0.5.0`/`v0.6.0`/`v0.6.1` (el `web/` del working tree es `?? web/`). Medido con la semántica que GitHub aplica de verdad (minimatch: `*` no cruza `/`, `**` sí) sobre los 2351 ficheros de `HEAD`: **426 (18,1 %)** fuera del filtro - `integrations/**` 81, **`vanta-proxy/**` 61 (crate del workspace, no lo cubre `vantadb-*/**`)**, `dev-tools/**` 48, `examples/**` 38, `benches/**` 27, `evals/**` 25, `.github/**` fuera de `workflows|actions` 11, `fuzz/**`, `Formula/**`, `Justfile`, `deny.toml`, `rust-toolchain.toml`...
- **La distinción que decide, contra la doc oficial (no de memoria):** `docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks` — *"A workflow is skipped by path filtering … → Associated checks stay in a **Pending** state and block merging"*, con el remedio literal **"Avoid requiring workflows that can be skipped"**; y la fila siguiente de la misma tabla: *"A job is skipped by a conditional → the job reports **Success**"*. `docs.github.com/en/pull-requests/reference/status-checks` lo cierra: *"A job that is skipped will report its status as 'Success'. It will not prevent a pull request from merging, even if it is a required check"* (estados aceptados: `success`, `skipped`, `neutral`). O sea: **el workflow que no corre no reporta nada; el job que se salta sí.** Por eso la doc no dice "ampliá la lista de paths".
- **Decisión (b), con las otras dos rechazadas por evidencia:** (a) ampliar `paths` es un treadmill — hay que enumerar 30 entradas de primer nivel y re-enumerar con cada directorio nuevo, y el path que la fila nombraba no existe; (c) un job "no-op" necesita **tres** jobs con el `name:` duplicado de los reales (un job = un check-run) y, cuando el workflow sí dispara, deja dos check-runs con el mismo nombre → duplica la superficie de `FIND-714` en vez de eliminarla.
- **YAML final** (`docs-gates.yml`): se **borran 21 líneas** de `on.pull_request.paths`; `push` las conserva (ahí no hay required check que satisfacer). Cero jobs nuevos, cero `name:` tocados, 16 jobs antes y después.
  ```yaml
  on:
    push:
      branches: [main, develop]
      paths: …                        # 21 patrones, INTACTOS
    # FIND-765: `pull_request` NO lleva `paths:` a proposito — un workflow que
    # no dispara no reporta NINGUN check-run y sus contexts quedan en Pending.
    pull_request:
      branches: [main, develop]
    schedule:
      - cron: '23 7 * * 1'
    workflow_dispatch:
  ```
- **La coincidencia literal de los 3 `name:`** (el fallo de `FIND-714`: un nombre que ningún job reporta no se satisface nunca). Comparado parseando el YAML y pidiendo los contexts a la API, por `len()` y por lista de `ord()` — no a ojo:
  ```
  docs-gates.yml jobs.docs-links.name     -> "Internal links resolve"                      22 chars
  ruleset 23692587 required_status_checks -> "Internal links resolve"                      22 chars
  docs-gates.yml jobs.docs-schema.name    -> "Frontmatter schema, kind agreement, orphans" 43 chars
  ruleset 23692587 required_status_checks -> "Frontmatter schema, kind agreement, orphans" 43 chars
  docs-gates.yml jobs.docs-index.name     -> "Generated indexes are current"               29 chars
  ruleset 23692587 required_status_checks -> "Generated indexes are current"               29 chars
  ```
- **Coste medido, no estimado** - run `45771790202` (último verde completo, `06:57:23Z`, `f2e9688`): 16 jobs, **967 s de runner** (16,1 min), **10m38s de pared**. **Los 3 jobs de build son el 84 % de esos 967 s de runner** - `Python examples` 466 s (48 %), `Doc-tests` 270 s (28 %), `rustdoc` 73 s (7,5 %); los otros 13 juntos 158 s (16 %), y los 3 required (`links` 7 s, `schema` 15 s, `index` 15 s) **37 s**. **Paralelismo efectivo 1,52x** (967 s de runner en 638 s de pared) y **198 s de arranque escalonado** dentro de un solo run; la pared la manda `Python examples` (**73 %**). El repo es **público** (`private: false`), así que los minutos no se facturan: el coste es latencia y presión de cola (el límite de jobs concurrentes de la cuenta no lo expone la API: **no medido**). Y el delta no toca a todos los PRs: un PR de `docs/`, `src/` o `vantadb-*` **ya** disparaba los 16 jobs.
- **Techo declarado:** si esa latencia molesta, el filtrado fino va **dentro** de los 3 jobs de build con su `if:` + un job de diff de paths **fail-closed** — nunca en el disparador. No se hace ahora porque añade un job, un `fetch-depth: 0` y una superficie donde un diff fallido apagaría el gate de rustdoc/doctests en silencio.
- **El contrato de 13/13 NO queda cerrado en este fichero, y el residual es mayor que el hueco que cierra.** **10 de los 13 contexts salen de `ci-rust.yml`**, que conserva su `pull_request.paths` (22 patrones) y **no** es escribible acá por conflicto de carril. Fuera de `ci-rust.paths` quedan **1281 ficheros (54,5 %)**, y **957 de ellos están bajo `docs/`** - como `ci-rust.yml` no lista `docs/**`, **un PR solo-docs reporta los 3 contexts de este fichero y sigue dejando los 10 de `ci-rust` en `Pending`**: el caso más frecuente del repo **no** queda arreglado, queda mejor. Fuera de la **unión** de los dos filtros quedan **248 (10,5 %)**: `dev-tools/**` 48, `examples/**` 38, `skills/**` 26, `evals/**` 25, `benchmarks/**` 21, `assets/**` 16, `fuzz/**` 12, `.github/**` 11, `embeddings/**` 8, `vhs/` 7... esos dejan los **13** en `Pending` y ahora además queman 16 jobs. Ganancia neta: **178 ficheros** que solo `ci-rust` cubría (`integrations/**` 81, `vanta-proxy/**` 61, `benches/**` 27, y 9 de raíz: `build.rs`, `deny.toml`, `osv-scanner.toml`, `rust-toolchain.toml`, `release-plz.toml`, `.gitleaks.toml`, `.secretlintrc.json`, `.secretlintignore`, `.config/nextest.toml`) pasan a reportar los 13. **Corrección de una delegación falsa:** el alcance decía que `FIND-762` "mide" `ci-rust.yml`; verificado que **no** - `FIND-762` es *"el `pre-push` formatea con stable y `ci-rust.yml:117` con nightly"* y no tiene task file. Los 10 contexts filtrados **no los mide nadie**: necesitan fila propia.
- **Verificación:** `actionlint` **exit 0** en `docs-gates.yml` y en `ci-gate.yml` (baseline previo también 0; local v1.7.12, CI pinea 1.7.7) · parseo YAML: `on.pull_request` = `{branches: [main, develop]}` **sin** `paths`, `on.push.paths` = 21 intactos, **16 jobs** · los 3 contexts coinciden por code points · `ci-gate.yml` **no se tocó**: el `REQUIRED` (19 = 18 del ruleset de `main` + rustdoc) no había que ajustarlo.
- **Lo que NO se pudo verificar, y se declara:** **no se puede abrir el PR de prueba** ni disparar un `pull_request` desde acá, así que el contrato real (`gh pr checks` sin ningún `Expected` a los 10 min) queda **sin observar** — se verificó la causa (el filtro ya no existe) y la coincidencia de nombres, no el efecto en un PR vivo. Tampoco se observa la primera corrida con el filtro quitado; los 967 s / 10m38s son de un run que **ya** corría los 16 jobs. El commit es **local, sin push**.
- **NOTICED BUT NOT TOUCHING:** (1) `docs/dev/operations/CI_POLICY.md:799-800` dice que `docs-gates` corre en PRs "que toquen docs, scripts o workflows" - ahora solo es cierto en `push`. Es deuda **creada por este cambio** y editarlo es seguro (el job `actionlint` solo compara el **conteo de ficheros** de `:16`, 20 == 20; no lee triggers), pero el alcance duro de esta tarea lo prohíbe (`FIND-752` en vuelo). (2) La red de `docs-gates` en `develop`, con fecha: run `45804272736` (`10:45`, `ab3dae3`) -> rojo en 3 jobs (el gate de `FIND-763` cazó `public documentation for create_snapshot links to private item is_mutable_after_snapshot`, exit 101 en 57 s; más `Internal links resolve` y `Generated indexes are current`); run `45821094074` (`12:35`, `3a7e225` = HEAD de `develop`) -> **success en los 16**. El árbol **volvió a verde**; el rojo no es condición vigente. Lo que sí queda: cualquier rojo futuro aparecerá ahora en **todos** los PRs, no solo en los de docs.
- **Review P2-01 (contexto fresco, `vanta-review`, report-only) -> `changes-required` y luego `approve`.** 4 Blocking, **todos en el registro y ninguno en el YAML** ("correcto, mínimo, sin jobs nuevos, sin `name:` tocados, `push` intacto"). Corregidos acá: B-1 (`246`/`180` eran **248**/`178`: mi `git ls-files --exclude` no coincide con minimatch en 2 ficheros), B-2 ("fuera de `ci-rust` = 426" era **1281**, con **957 en `docs/`** - el residual estaba subestimado 5,2x y cambia la conclusión), B-3 (la delegación a `FIND-762` era falsa), B-4 (faltaba esta sección de review); más M-1 (`inmergeable` = sin bypass), M-3 (paralelismo 1,52x, 198 s de arranque, cola como riesgo no medido), M-5 (la foto "rojo ahora mismo" estaba caducada), L-1 y L-4. **M-2 no aplicado**: sincronizar `CI_POLICY.md` es seguro pero está prohibido por el alcance.

### El mismo filtro, el fichero de al lado: `paths` fuera de `pull_request` en `ci-rust` (`FIND-770`)
- **Fecha:** 2026-11-26
- **Objetivo:** `FIND-765` cerró **3** de los 14 contexts required de `develop` y dejó el residual declarado (`:1131`): los otros **10** salen de `ci-rust.yml`, que conservaba `on.pull_request.paths` (22 patrones). Fuera de esa lista quedaban **1280 de 2353 ficheros tracked (54,4 %)** — **959 de ellos bajo `docs/`**. Un PR solo-docs **no arrancaba** este workflow: sin check-run, esos 10 contexts quedaban en `Expected` para siempre. Es el deadlock de `FIND-714`/`FIND-765`, en la capa del disparador.
- **La decisión NO se copia del precedente, se recalcula — porque los jobs de este fichero son los caros.** Medido en el run `45821093876` (`3a7e225ad`, push a `develop`, success, el más reciente que corrió los 3 tests): **9062 s de runner en los 20 jobs que corrieron** (el fichero tiene 24; 4 reportaron `skipped`) y **1h09m35s de pared** (12:35:36Z → 13:45:11Z). Los 3 `Tests` required solos: **5670 s** (Windows **3426**, macOS **1586**, Linux **658**) y no arrancan hasta que `fmt`+`clippy`+`secret-scan` terminan — de hecho empezaron a los 12m28s. Aceptar el coste entero (opción b) = **69 min y 9062 s por PR solo-docs**, contra 0 s de hoy pero con 10 contexts en `Pending`: el remedio peor que la enfermedad. Descartada por número, no por intuición.
- **Lo que se hizo (opción a), que es el "techo declarado" que este mismo documento ya había escrito** en `:1130` ("el filtrado fino va **dentro** de los jobs con su `if:` + un job de diff de paths"): `paths` fuera de `on.pull_request`, `on.push` con sus **24** patrones **intactos**, y el filtro adentro. Job nuevo `rust-scope` (fail-open, misma política que `wasm-scope`/`FIND-725`: si no es un PR, si la base no se resuelve, si el diff falla o si el job no termina bien, **corren los gates**) publica `rust=true|false` desde un `git diff --name-only --no-renames -z base..HEAD` contra un `grep -E` que **espeja `on.push.paths`**.
- **Una condición, nueve jobs omitidos — y no por casualidad:** `clippy` se gatea con `rust`, y los **9** jobs que lo llevan en `needs` (tests Linux/Windows/macOS, embeddings, coverage, miri, asan, experimental, release-combo) se omiten **por dependencia**, no por 9 `if:` copiados. `workflow-syntax §jobs.<job_id>.needs`: *"If a job fails or is skipped, all jobs that need it are skipped"*. `semver-checks` y `public-api-snapshot` (30 y 45 min de nightly, PRs contra `main`) no dependen de `clippy` y llevan guard propio.
- **YAML final** (extracto, `ci-rust.yml`):
  ```yaml
  # FIND-770: `pull_request` NO lleva `paths:` a proposito. Un workflow que no
  # arranca no reporta NINGUN check-run: sus checks quedan en `Pending` ...
  pull_request:
    branches: ["main", "develop"]

  jobs:
    rust-scope:
      name: Rust Scope (PR paths)          # outputs: rust: ${{ steps.changed.outputs.rust }}
      # el resto del script: fail-open en cada branch (no-PR / sin base / fetch
      # fallido / diff fallido) y la decision final, que NO es un `if grep`:
      #   rc=0
      #   grep -qE "$PATTERNS" <<< "$changed" || rc=$?
      #   case "$rc" in 0) rust=true ;; 1) rust=false ;;
      #                 *) warning + rust=true ;; esac
    clippy:
      needs: [rust-scope]
      if: >-
        !cancelled() && (needs.rust-scope.outputs.rust == 'true' ||
        needs.rust-scope.result != 'success')
  ```
  `!cancelled()` no es decorativo: sin una función de status en el `if:`, un `needs` omitido **siempre** omitiría el dependiente y el guard nunca se evaluaría (mismo motivo documentado en `wasm-test`, `FIND-725`).
- **Contrato de los 14/14, nombre a nombre** (YAML parseado con `pyyaml`, no a ojo; ruleset `23692587` leído por API):

  | context required | job | donde | PR solo-docs |
  |---|---|---|---|
  | `Format Check` | `fmt` | ci-rust | **corre** (23 s) |
  | `Clippy Lints` | `clippy` | ci-rust | `skipped` |
  | `Tests (Linux)` | `test` | ci-rust | `skipped` |
  | `Tests (Windows)` | `test-windows` | ci-rust | `skipped` |
  | `Tests (macOS)` | `test-macos` | ci-rust | `skipped` |
  | `MSRV Check (1.95.0)` | `msrv` | ci-rust | **corre** (114 s) |
  | `Experimental Crates Check` | `experimental-check` | ci-rust | `skipped` |
  | `Security Audit` | `audit` | ci-rust | **corre** (36 s) |
  | `Miri (UB Detection)` | `miri` | ci-rust | `skipped` |
  | `Dependency Policy Check` | `deny` | ci-rust | **corre** (32 s) |
  | `Internal links resolve` | `docs-links` | docs-gates | corre (sin `paths` desde `FIND-765`) |
  | `Frontmatter schema, kind agreement, orphans` | `docs-schema` | docs-gates | idem |
  | `Generated indexes are current` | `docs-index` | docs-gates | idem |
  | `Generate API reference (rustdoc)` | `rustdoc` | docs-gates | idem |

  **14/14 con job que los reporte. 0 `name:` duplicados** (un `name:` repetido serían dos check-runs para un context: `FIND-714` por otra puerta, y es el motivo por el que se rechazó la opción "jobs no-op" que `FIND-765` ya había rechazado). `skipped` = 'Success' y no bloquea el merge aunque el check sea required: `docs.github.com/en/pull-requests/reference/status-checks` (*"A job that is skipped will report its status as 'Success'"*) + `…/workflow-syntax §on.<push|pull_request>.<paths>` (*"checks associated with that workflow will remain in a 'Pending' state"*, que es el bug que esto cierra).
- **Coste medido del arreglo:** un PR solo-docs (`rust=false`) corre **12 de 24 jobs**: **427 s medidos** en el run de referencia (`fmt` 23, `MSRV` 114, `audit` 36, `deny` 32, `machete` 70, `OSV` 9, `minimal-versions` 111, `secret-scan` 25, `wasm-scope` 7) **+ 3 jobs que ese run no ejecutó** (`adr-gate` ~10 s, `wasm-test` ~120 s, y el `rust-scope` nuevo ~15 s) => **~570 s**, un **6,3 %** de los 9062 s. Pared ~4 min (antes: 0 s y **10 contexts en `Pending` para siempre**).
- **El precio, declarado:** 6 de los 10 contexts se reportan `skipped` **sin medir** en un PR solo-docs. Eso es lo que firma la opción (a) — y es el mismo precio que ya cobraba el filtro, sólo que antes la moneda era el deadlock. Los 4 que corren son justamente los baratos; `clippy` y los tests caros siguen midiendo **siempre** que el PR toque un fichero que el build lea (`Cargo.toml`/`Cargo.lock` incluidos, así que un PR de Dependabot **nunca** se salta `clippy`: el comentario de `FIND-472` sigue siendo cierto, y quedó escrito al lado).
- **Techo declarado:** (1) la lista de patrones del `rust-scope` es un **espejo a mano** de `on.push.paths`: divergen si alguien añade un patrón a `push` y no al `grep`. Se compensa siendo estricto — el espejo es la **unión** (24, no los 22 del PR: `.cargo/**` y `.github/actions/rust-setup/**` eran sólo de `push`), así que un PR que toque `.cargo/**` hoy **no** disparaba nada y con esto dispara los gates. Cambio de comportamiento declarado, en la dirección estricta. (2) Sobre `pull_request` el `git diff base..HEAD` **no** arrastra la base: `base.sha` es la punta de la rama base y `HEAD` es el commit de merge de prueba, así que el diff trae **sólo** el cambio del PR (el sobre-corrido que sí existe es el inverso: si la rama del PR **fusionó la base dentro de sí misma**, esos ficheros ya forman parte del cambio del PR). (3) **`FIND-765` llamó "fail-closed" al job de diff, y esto es fail-open.** No es un descuido: la política contraria está **escrita y razonada** en este mismo fichero (`wasm-scope`, `FIND-725`) — *"a false positive costs ~2 min; a false negative loses coverage silently"* — y para un context **required** un diff fallido que apaga el gate produce un verde **vacío**, que es peor que gastar runner. Se sigue `FIND-725`, no la palabra suelta del precedente.
- **Verificación (mecánica):** `actionlint` **exit 0** (local v1.7.12; baseline previo también 0) · YAML parseado: `on.pull_request` = `{branches: [main, develop]}` **sin** `paths`, `on.push.paths` = **24** intactos, 24 jobs, 0 `name:` duplicados · la **ERE del `rust-scope`** validada con **46/46 casos** ejecutando `grep -E` de verdad sobre la regex **extraída del workflow** (24 paths que deben disparar, 17 que no, 6 trampas de prefijo: `src-tauri/` no casa `src/`, `Cargo.tomlx` no casa `Cargo.toml`, `docs/src/` no casa `src/`) · conteo real sobre `HEAD`: **2353 tracked, 1073 dentro, 1280 fuera (54,4 %), 959 bajo `docs/`** · `check-docs.mjs` exit 0 · `check-links.mjs` exit 1 por **3 enlaces rotos preexistentes** (`ADR-0066` → `../../tasks/FIND-756.md` ×3 y un snapshot del historial), ninguno de esta tarea. **No se corrió `just verify`**: el carril es CI, no compila (ni un `.rs` tocado).
- **Lo que NO se pudo verificar, y se declara:** **no se puede abrir el PR de prueba** ni disparar un `pull_request` desde acá, así que el contrato vivo (`gh pr checks` sin ningún `Expected` a los 10 min) queda **sin observar**: se verificó la **causa** (el filtro ya no existe en el disparador) y la **coincidencia nombre-a-nombre**, no el efecto en un PR real. Tampoco se observó la **primera** corrida con el filtro quitado (`rust-scope` no ha corrido nunca). Los 9062 s / 1h09m35s son de un run que **ya** corría los 24 jobs. El commit es **local, sin push**.
- **NOTICED BUT NOT TOUCHING:** (1) **`docs/dev/operations/CI_POLICY.md:803-804` ahora dice lo contrario del código** — *"El mismo problema **sigue abierto** en `ci-rust.yml` (`FIND-770`): sus 22 patrones dejan fuera 1281 ficheros y 10 de los 13 contexts"*, y ya es falso por los tres lados (los 22 patrones de PR no existen, son 24 en `push`; los 10 contexts ya se reportan; 14, no 13). **Texto de reemplazo para el carril dueño:** *"En `pull_request` tampoco hay filtro de `paths` en `ci-rust.yml` (`FIND-770`): el filtro se movió a un job de scope que omite los jobs caros con `if:` (`skipped` = success, cuenta como required satisfecho). `on.push` conserva sus 24 patrones. Un PR solo-docs corre ~12 de 24 jobs (~570 s) y reporta los 10 contexts."* — no se aplica acá porque `CI_POLICY.md` está **fuera** de la lista de escritura de esta tarea y `FIND-752` es su dueño en vuelo (y su task file no menciona `770`: el handoff no le llega solo). (2) `check-links.mjs` está **rojo en `develop`** por `ADR-0066` → `../../tasks/FIND-756.md` (el fichero llega como `?? docs/dev/tasks/FIND-756.md`, sin commitear): un rojo que ahora aparecerá en **todos** los PRs porque `Internal links resolve` ya es required y ya dispara. Es del carril de docs-gates, no de éste. (3) `concurrency: cancel-in-progress: true` (nivel workflow) ahora alcanza también a los PRs solo-docs: un push durante la ventana **cancela** el run y un required cancelado no es `skipped`, es **ausente** — la clase de `FIND-764`/`FIND-771`, con más superficie.
- **Para `FIND-762`/`FIND-764` (wave siguiente, esperan este fichero):** `FIND-762` (fmt stable vs nightly) toca la **misma** zona del `fmt` de `ci-rust.yml` — **no cites números de línea del Backlog (`:117`) ni de acá: `grep -n "toolchain: nightly" .github/workflows/ci-rust.yml`** (el primero es el de `fmt`; medido hoy en **`:208`**). `FIND-764` (cancelación en vuelo) tiene ahora un dato nuevo: la ventana de este workflow cubre los PRs solo-docs, que antes no lo arrancaban; y un `rust-scope` cancelado **no** deja los contexts en verde, los deja ausentes. **Este fichero queda libre para las dos**: `ci-rust.yml` no vuelve a tocarse acá, y las zonas nuevas son el bloque `on:` (**3-61**) y el job `rust-scope` (**64-160**).
- **Deuda de índice, declarada:** `gen-index.mjs --check` está **rojo**, pero por drift ajeno a esta tarea (`docs/index.md:141` desalineado en las filas de ADR-0011/0012/0014, y `ADR-0066` llegó como fichero nuevo de otro carril). El único aporte propio es `docs/dev/tasks/FIND-770.md`. `--write` está **prohibido** en este contrato: lo corre el orquestador con el resto del wave.
- **Review P2-01 (contexto fresco, `vanta-review`, una ronda) -> `changes-required`, 0 Blocking.** No encontró puerta al deadlock ni camino fail-closed en el YAML, verificó **verbatim** las dos semánticas que lo sostienen (`status-checks` y `workflow-syntax §jobs.<job_id>.needs`), los 14 contexts con **un** job cada uno, `on.push.paths` byte-idéntico y `continue-on-error` 6→6; **reprodujo todas las cifras** (9062 s / 1h09m35s / 5670 / 427 / 572 / 54,40 % / 959 / 12 de 24) sin discrepancias. Aplicados en el commit de seguimiento: **M1** (la cita de `:1131` no era verbatim: `13`/`acá`, no `14`/`ahí`; más la nota de que 1281/957 se midieron sobre 2351 ficheros), **L1** (9062 s es la suma de los **20** jobs que corrieron), **L2** (el techo sobre `base.sha` estaba mal razonado — el diff **no** arrastra la base), **L3** (tres referencias de línea ya estaban mal en el mismo commit: `fmt` en `:208`, `on:` 3-61, `rust-scope` 64-160) y **L4** (el extracto omitía las 4 líneas del here-string + `rc=2`). **M2 no aplicado** (ver `CI_POLICY.md` arriba, con el texto exacto de reemplazo).
- **Commit:** este commit + el de seguimiento de la review P2-01 (`ci-rust.yml` + task file + esta entrada + fila del Backlog retirada).

### Los seis binarios fuera del `default-filter`, contados: 118 de 154 tests no tienen lane (`FIND-752`)
- **Fecha:** 2026-11-26
- **Objetivo:** el `default-filter` de `.config/nextest.toml` excluye seis binarios por nombre (documentado y deliberado). Lo que nadie había medido es **cuántos tests hay ahí ni si algún lane sano los ejecuta**. Sin ese número, la exclusión es una caja negra que tranquiliza: desde fuera, "corre en la certificación semanal" y "no corre en ningún sitio" se ven igual.
- **Resultado — inventario medido** (`cargo nextest list -p vantadb --ignore-default-filter --run-ignored all -E 'binary(benchmark_internal) or binary(benchmark_datasets) or binary(cli_tests) or binary(concurrency_parity) or binary(snapshot_certification) or binary(security_audit)'`): **154 tests** → `benchmark_internal` 1 · `benchmark_datasets` 1 · `cli_tests` 96 · `concurrency_parity` 5 · `snapshot_certification` 45 (56 con `--features failpoints`, las de su lane) · `security_audit` 6.
- **Resultado — el veredicto, con `gh` en la mano:**

  | Binario | Tests | Lane | Último resultado real | Veredicto |
  |---|---:|---|---|---|
  | `benchmark_internal` | 1 | `certification.yml` → `other_heavy` | run `45598671538`: **failure en el step TBH-01, antes de los tests**; 0 success / 36 failure en 30 d | **no corre** |
  | `benchmark_datasets` | 1 | idem | idem | **no corre** (y en local vuelve en verde si falta GloVe-100) |
  | `cli_tests` | 96 | idem | idem | **no corre** |
  | `concurrency_parity` | 5 | `certification.yml` → `memory_concurrency` | run `45598671538`: **success**; 36 success / 0 failure / 3 skipped | **corre** 5/5 — el `#[ignore]` es condicional (`not(feature = "rocksdb")`) y la lane pasa `--features rocksdb` |
  | `snapshot_certification` | 56 | `snapshot-certification.yml` fases 03/04/05 | run `45802050950` (26/11): **success**; 39/48 en 30 d | **corre parcial**: 42/56; **14 no corren** |
  | `security_audit` | 6 | **ninguno** | nunca; `git log -S"security_audit" -- .github/workflows` → 0 | **no corre** |

- **El número que faltaba: 118 de 154 tests (77 %) sin lane que los ejecute** — 96 `cli_tests` + 14 de `snapshot_certification` (4 `export_format_*`, **`find319_child_flag_ignores_empty_values`** — el caso de `FIND-738` —, 5 `vantafile_*`, 4 `wal_*`) + 6 `security_audit` + 1 `benchmark_internal` + 1 `benchmark_datasets`.
- **Los tres estados, separados** (es la parte que el ticket pedía y la que se confundía): (1) **corre en otro lane y pasa** → `concurrency_parity` 5/5 y `snapshot_certification` 42/56; (2) **corre y falla** → ninguno de los seis, porque `other_heavy` falla **antes** de correr tests; (3) **no corre en ningún sitio** → los 118. El rojo de `other_heavy` es 0 success / 36 failure en 30 días (39 runs programados: 36 failure, 2 cancelled, 1 in progress al leer), y muere en `Verify required benchmark datasets (TBH-01)`: la corrección de `FIND-328` (descargar antes del gate) está en `develop` **552 commits por delante de `main`**, y el `schedule` corre `main`.
- **Corrección de una premisa del ticket:** `snapshot_certification` **no** depende de `certification.yml` — tiene lane propia (`snapshot-certification.yml`, 39/48 verde en 30 d). Su agujero es otro y más fino: el selector de las fases (`test(snapshot) or test(pending_restore) or test(schedule)`) matchea **42 de 56**, no el binario.
- **Entregable:** tabla publicada en `docs/dev/operations/CI_POLICY.md` §2c, con el comando de cada cifra y la regla nueva ("toda exclusión nueva entra en esta tabla con el lane que la corre"). Evidencia completa en `docs/dev/tasks/FIND-752.md`.
- **No se tocó** `.config/nextest.toml` ni `.github/**` (auditoría, no arreglo). Cuatro propuestas quedan como texto en el task file, sin registrar: cablear `security_audit` (P-1), ensanchar el selector de snapshot (P-2), separar `cli_tests` del gate de dataset (P-3) y registrar que la lane programada mide `main` (P-4).
- **Verificación:** inventario 154 y 42/56 medidos con los comandos de arriba · agregado por job de los 39 runs programados vía `gh api .../runs/<id>/jobs`, base `event == schedule` y fecha de lectura declarada · `check-links` 0 · `check-docs` GATING all clear · `gen-index.mjs` **no ejecutado** (prohibido por el contrato de la tarea; drift se regenera al cierre de wave).
- **Review P2-01 (contexto fresco, `vanta-review`) → `changes-required`, 1 Blocking real y corregido:** `concurrency_parity` **sí corre 5/5** — `test_triple_backend_parity_validation` es `#[cfg_attr(not(feature = "rocksdb"), ignore)]` (`tests/concurrency_parity.rs:32`) y la lane pasa `--features rocksdb`; la fila decía "no corre" con un lane verde detrás, justo el error inverso al que la tabla denuncia. El **119 pasó a 118**, y de paso corrigió un `memory_concurrency` 35→36 y una línea de runs no reproducible (37 failure → 36 + 1 in progress). Los conteos, los 14 nombres y el root cause a nivel de step resistieron la re-ejecución.
- **Commit:** este commit (CI_POLICY + task file + esta entrada + fila del Backlog retirada)

### El gate required deja de morir en vuelo, y el step publica cuánto midió (`FIND-771`)
- **Fecha:** 2026-11-26
- **Objetivo:** desde `FIND-763` (`1a318fa91`) el context `Generate API reference (rustdoc)` es **required** en `develop` (14 contexts) y en `main` (19), verificado por API. El workflow que lo produce seguía con `cancel-in-progress: true` (la directiva que `FIND-764` documentó para `ci-rust.yml`), así que un push dentro de la ventana del run mataba el job más caro del repo.
- **La ventana, medida:** run `45821094074` = **13m08s** (`12:35:36Z` → `12:48:44Z`), y **13m05s de esos 13m08s son el job rustdoc**; el resto de los 16 jobs termina en **≤ 4m10s** (`Doc-tests`, `12:35:39`→`12:39:49`). Con cache caliente el **job** baja a **1m13s–1m29s**, pero el run lo marca el job más lento: **4m00s** y **10m38s** medidos (`45832678767` / `45771790202`); se cita el peor del día (13m08s).
- **La frecuencia, honesta:** los **12** pushes a `develop` de hoy (00:12:15 → 13:48:08) dejan **11 huecos** — 12m29s · 14m29s · 36m40s · 49m21s · 49m31s · **1h07m51s** · 1h12m32s · 1h13m29s · 1h23m54s · 2h39m53s · 3h15m44s. **1 de 11** cae dentro de la ventana y **1** queda al borde por 81 s; **mediana 1h07m51s**. No es constante, pero **ya se materializó una vez hoy**.
- **Qué deja una cancelación (premisa corregida en review, con la API en la mano):** el run `45802050777` (push 10:30:58) mató `Generate API reference (rustdoc)` a las **10:45:46**, tras **6m01s** corriendo (arrancó 10:39:45), cuando llegó el push de las 10:45:27 (`45804272736`). **El context NO queda ausente ni en `Expected` eterno:** `gh api .../commits/337ee796.../check-runs` devuelve `Generate API reference (rustdoc)` → `completed` / **`cancelled`** (los 61 check-runs de ese SHA están todos `completed`). Es decir: hay un **no-success reportado**, y el daño se mide en la superficie donde ocurrió (corregido en la 2ª ronda: en `develop` **no hay merge que bloquear**): el HEAD de la rama queda con un `cancelled` que `ci-gate.yml:66` lee como fallo, 6m01s de rustdoc tirados, y un commit cuyo gate de doc **nunca midió** (la clase de `FIND-764`, ahora encima de un required). En un PR la cancelación se resuelve sola con el run siguiente.
- **Decisión (A) — `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`, opción (iii):** el grupo es `${{ github.ref }}`, y ese ref significa cosas distintas por evento. En `pull_request` es `refs/pull/N/merge`, que GitHub **reconstruye en cada push**: el head revisado siempre trae su run nuevo, así que cancelar el viejo no pierde ningún context y ahorra hasta 13 min por fix-up. En `push` el ref es la rama y el SHA queda en la historia: no hay run nuevo que reporte por él, así que **no se cancela**. **Rechazada (i)** (`false` en todo) porque mete la cola de 13 min en los PRs sin comprar nada — el pre-mortem (a) servido tal cual. **Rechazada (ii)** (identidad del run en el grupo) porque deja correr N runs sin límite: un burst de 3 pushes triplica `doc-examples` (466 s) y `doctests` (270 s) para reportar SHAs que ya nadie va a mergear.
- **Decisión (B) — el verde que se puede auditar:** el step del gate publica en `$GITHUB_STEP_SUMMARY` el tamaño del árbol de doc (**directorios con `index.html` bajo `target/doc`**, lib + bin) y **0 warnings**, con `RUSTDOCFLAGS=-D warnings` explicado como lo que es: el primer lint **aborta** el step, así que llegar al summary prueba el 0. Y una aserción `test "$DOCS" -gt 0`: un 0 es **rojo**, no "ok" — el pre-mortem era un summary que siempre dice "ok" sin contar nada. Alcance declarado: `cargo doc` **acumula** doc entre invocaciones, así que el número es el de la superficie publicada, no "crates compiladas en este step"; se quitó además el `-not -path 'target/doc/src/*'`, que a `-maxdepth 2` no podía casar nada.
- **Verificación (mecánica):** `actionlint` **exit 0** (baseline previo 0; acepta la expresión) · `concurrency` parseado = `{"group":"${{ github.workflow }}-${{ github.ref }}","cancel-in-progress":"${{ github.event_name == 'pull_request' }}"}` · step en el índice **2 de 7** (no se movió), `env` del step = `{"RUSTDOCFLAGS":"-D warnings"}`, `env` del job **sin** `RUSTDOCFLAGS`, claves `name,env,run` (sin `continue-on-error`) · **16 jobs**, **0** `name:` duplicados · `on.pull_request` intacto (`{"branches":[main,develop]}`, sin `paths` desde `FIND-765`) · `name: Generate API reference (rustdoc)` **intacto** (`:549`, `ci-gate.yml:58` lo compara por string) · el bloque `run:` **extraído del YAML con `js-yaml`** (no retipeado) y ejecutado en bash con `cargo` sombreado sobre un `target/doc` fabricado: `Doc directories: **3**`, summary escrito, **exit 0**; con `target/doc` vacío: `**0**` y **exit 1**.
- **Lo que NO se pudo verificar, declarado:** la prueba viva de la ventana — **no se disparan dos pushes reales** desde acá (push = acto del owner). Se verificó la **causa** (la expresión quedó puesta y el YAML la parsea así), el **modo de fallo histórico** (`completed/cancelled` en `337ee796`) y el razonamiento por evento, no una cancelación futura. Tampoco se corrió `cargo doc` (carril CI sin compilar, otro carril compilando): el workspace declara **8** entradas en `members` (`Cargo.toml:786-798`), así que el orden del número es ~8-10 y lo publica el primer run.
- **Techo declarado:** en `push` GitHub mantiene **uno en curso + uno pendiente** por grupo (`queue: single`, default): un tercer push cancela el **pendiente**, cuyo SHA ya está superado por el que lo cancela. Para que cada push de un burst reciba su run existe `concurrency.queue: max` (hasta 100 pendientes), no usado hoy. Y los 0 warnings son de **rustdoc**: `RUSTDOCFLAGS` no deniega los de `rustc`.
- **NO se toca** `ci-rust.yml` (`FIND-770`, que commitó `2b282eed7` en su propio carril) ni el `name:` del job.
- **Review P2-01:** `vanta-review`, contexto fresco, **dos rondas**, `changes-required` las dos veces y **todos los hallazgos cerrados**; en la 2ª el YAML quedó aprobado **sin cambios** ("approvable as-is") y lo que se corrigió fue la prosa → 3 Blocking + 3 Medium + 2 Low, **todos cerrados** en la misma ronda (la premisa del "context ausente", la tabla de opciones sin la expresión por evento, 80→8 members, 8→12 pushes y mediana 49m21→1h07m51, 4m03→4m10, "crates"→"doc directories", §Review incompleta y `approve` pre-escrito). Ver §Review del task file.
- **Incidente de árbol compartido (declarado):** la fila del Backlog se retiró **tres veces** — dos commits de otros carriles (`08093505a`, y el snapshot de `cecae8137`) reinsertaron la sección desde copias stale, y el primer corte (a EOF) se llevó por delante la sección de `FIND-770`; rehecho **quirúrgico** por límites de sección y verificado con `git diff -U1` (6 líneas, todas propias). Esta misma entrada quedó commiteada por el snapshot de otro carril con las cifras previas y se corrige aquí.
- **Commit:** este commit (`docs-gates.yml` + task file + esta entrada corregida). Fila del Backlog ya retirada en HEAD. Sin `gen-index`. Sin push.

### `security_audit` deja de ser cobertura imaginaria: 6 tests con lane propia y semanal (`FIND-767`)
- **Fecha:** 2026-11-26
- **Objetivo:** el binario `security_audit` (6 tests: IQL injection, auth bypass, input validation, resource exhaustion, timing attack, resumen) estaba fuera del `default-filter` (`.config/nextest.toml:68`) y **ningún** workflow lo invocaba (`git log -S"security_audit" -- .github/workflows` devolvía 0 commits **antes de este**; ahora devuelve 1). Los dos gates locales lo saltaban por nombre (`dev-tools/verify.ps1:117` en el fallback `cargo test`, `dev-tools/coverage-budget.ps1:79`). Cero ejecuciones desde que existe = un test que cuenta como cobertura sin medir nada.
- **La medición que decidió la lane** (el coste va antes que la elección): los 6 tests tardan **63-81 s** de pared entre dos sesiones (esta: 68,2 / 72,5 / 80,8 s con 4 / 8 / 1 `--test-threads`; la re-medición de la review P2-01: 62,8 / 64,7 / 76,5 s) sobre `target/debug` en Windows, y `security_audit_resource_exhaustion` solo mide **63-89 s** (10 000 puts en batch, 30 ciclos open/close, 8 hilos × 250 puts, 1 000 inserciones vectoriales). Conteo: **6 tests**. Un minuto de runner **por PR** no pasa el umbral de 30 s del contrato, así que la opción "lane por PR en `ci-rust.yml`" quedó **rechazada con número**, no por gusto.
- **La decisión:** job nuevo `security-audit` / `name: "Certification: security_audit"` en `certification.yml`, en el `schedule` del domingo 03:00 (`workflow_dispatch` también), `cargo nextest run -p vantadb --test security_audit --ignore-default-filter --profile audit --no-tests fail --locked`. `--ignore-default-filter` cancela la exclusión del `default-filter`; `--no-tests fail` convierte una selección vacía en **rojo** (el modo de fallo de `FIND-738`: 0 tests, exit 0).
- **Aislamiento frente a `FIND-766`** (que edita `other-heavy` en la wave siguiente): job **al final** del fichero (después de `fuzz-pr`), `name:` propio, sin un solo step ni `needs` compartido con `other-heavy`. Nada que colisione.
- **Sin `needs: ci-gate`, a propósito:** los heavies lo llevan para no gastar horas con `main` rojo. Éste es el **único** lane que corre estos tests y `needs: ci-gate` lo dejaría en `skipped` (que GitHub reporta como *success*) justo cuando `main` está rojo — el verde-que-no-mide que `FIND-752` midió. Coste de correrlo igual: ~70 s de tests + un build debug, una vez por semana.
- **Techo declarado:** el `schedule` corre sobre `main`, que va por detrás de `develop` (el agujero de `FIND-766`); y si la suite baja de 30 s, la lane por PR es estrictamente mejor. Ambas cosas quedan escritas en el propio YAML como upgrade.
- **Verificación (mecánica):** `actionlint .github/workflows/certification.yml` → **exit 0** (baseline previo 0) · YAML parseado con pyyaml: el job existe con `if = workflow_dispatch || schedule '0 3 * * 0'`, `name` propio, **3 steps**, **22 jobs** (antes 21), **0 `name:` duplicados** · `cargo nextest list … -E 'binary(security_audit)'` → **6 tests** · runtime medido tres veces con el binario construido, y por test individual · `check-docs.mjs` / `check-links.mjs` sobre los ficheros tocados.
- **Lo que NO se pudo verificar, declarado:** **el workflow no se disparó** (no hay `schedule`/`dispatch` disparable desde acá y el push es del owner): se verificó la causa (job presente, filtro puesto, YAML parsea, `actionlint` verde) y el runtime local, **no** el verde en runner Linux ni su tiempo ahí. El `cargo nextest list` del contrato **se lanzó y quedó 17 m 28 s en `Blocking waiting for file lock on artifact directory`** (`src/**` en edición por `FIND-758` + otros tres `cargo-nextest` sobre el mismo `target/`): se abortó para no sumar una cuarta compilación al lock. El conteo quedó verificado por dos vías — el binario (`--list` → `6 tests, 0 benchmarks`) y el `6` que `FIND-752` midió con ese mismo comando —, no por una corrida nueva de `nextest list`.
- **NOTICED BUT NOT TOUCHING → escalado (M1 de la review):** `docs/dev/operations/CI_POLICY.md:656` sigue diciendo `| security_audit | 6 | none | never ran | does not run |`, y **`FIND-752` ya está `completed`** (es ancestro del commit), así que nadie la corrige por inercia. Ese fichero **no está en la lista de escritura** de esta tarea, así que la corrección tiene dueño pendiente: la fila debe pasar a `certification.yml → Certification: security_audit | domingos 03:00`. Texto de reemplazo listo en `docs/dev/tasks/FIND-767.md` §Deuda.
- **Review P2-01 (contexto fresco, `vanta-review`, una ronda) → `changes-required`: 2 Blocking + 2 Medium + 3 Low, todos cerrados.** **B1** — el commit arrastraba la eliminación de la sección de **`FIND-757`** en el Backlog (mi script reescribió el fichero y el árbol de trabajo ya venía sin ella, quitada sin commitear por otro carril): restaurada, verificado 216 filas antes/después. **B2** — faltaba `## Review` en el task file: añadida con el dictamen. **M1** — escalada (arriba). **M2** — el cierre queda rotulado "cableado, esperando primer run". **L1** — los tiempos se publican como rango 63-81 s (la re-medición dio 62,8-76,5 s). **L2** — `swap-mb: 4096` añadido al `rust-setup`, como los otros pesados. **L3** — el task file es *orphan* en `check-docs` (no gating). Resistió sin cambios: el `run:` exacto, la guarda, el aislamiento de `other-heavy`, el conteo 6 y "único lane".
- **Commit:** este commit de seguimiento (`certification.yml` con `swap-mb` + rangos + task file con §Review + esta entrada corregida + fila de `FIND-757` restaurada en el Backlog). Sin `gen-index`. Sin push.

### El `pre-push` mide `fmt` con el instrumento de CI, y lo dice (`FIND-762`)
- **Fecha:** 2026-11-26
- **Objetivo:** `FIND-760` dejó el gate de `fmt` midiendo **el rango que viaja** (bien) pero el instrumento seguía saliendo del `PATH`: el `rustfmt` de la toolchain pineada (`1.95.0`). El job `fmt` de CI mide con la toolchain de su step de `rust-setup` — `toolchain: nightly` (`ci-rust.yml:208`, verificado con `grep -n` y lectura del bloque `:192-215`, no con el `:117` del ticket — esa línea **existe** hoy, pero ya no es el gate: es un `fi` dentro de `rust-scope`, y lo que se movió fue `fmt`, no el fichero (precisión de la review, L2). Dos instrumentos, un verde: un push podía pasar acá y fallar en CI sin que nada lo dijera. Es la clase `FIND-736`/`FIND-731` en el último gate antes de la rama compartida.
- **Medición 1 — ¿cuántos ficheros difieren hoy?** Formateando los **739** `.rs` tracked con cada `rustfmt` (`--emit stdout --edition 2021 --config skip_children=true`, comparación byte a byte de la salida):

  | Par | Difieren | Parse-fail |
  |---|---:|---:|
  | `+stable` `1.9.0-stable (8bab26f4f6 2026-07-14)` vs `+nightly` `1.11.0-nightly (c36f145719 2026-10-01)` | **0 / 739** | 0 / 0 |
  | pin `1.95.0` `1.9.0-stable (59807616e1 2026-04-14)` vs `+nightly` `1.11.0-nightly` | **0 / 739** | 0 / 0 |

  **Los tres instrumentos coinciden en todo el árbol hoy.** No había defecto materializado: había un defecto **latente y no observable** — el gate no decía con qué medía, así que el día que `rustfmt` cambie de formato el verde local seguiría pareciendo el mismo verde.
- **Medición 2 — la paridad en el lado del fallo (el que cuesta):** dos `cargo fmt --all -- --check` seguidos, con minutos de diferencia, devolvieron **el mismo fichero y el mismo diff** con las dos toolchains: primero `vantadb-mcp/tests/mcp_proxy_reconnect.rs`, después `src/storage/engine/delete.rs` (WIP **de otro carril**; el árbol cambió entre corridas, ningún fichero es de esta tarea). El instrumento no cambia el veredicto; lo que faltaba era poder **atribuirlo**. **No reproducible hoy** (review L3): esos dos WIP ajenos ya se limpiaron del árbol, así que `rustup run nightly|1.95.0 cargo fmt --all -- --check` devuelve **exit 0 en los dos** sobre árbol limpio — el mismo resultado leído al derecho; la medición que decide es la de la tabla de arriba (0/739).
- **Decisión (c): leer el instrumento de CI y medir con él cuando está, declararlo cuando no.** `fmt-range.ps1` resuelve el toolchain leyendo el job `fmt` de `ci-rust.yml` (`Get-CiFmtToolchain`, sin literal hardcodeado: un segundo sitio donde el pin puede derivar es exactamente el defecto que se cierra), mide con él si está instalado, y si no **mide igual con el del PATH y lo declara ADVISORY** — nunca bloquea, porque "no tengo nightly" no es una propiedad del commit (es el criterio del advisory de `FIND-754`). La versión de `rustfmt` se imprime en **cada** corrida (`instrument = rustfmt <versión> (<toolchain>)`), y el residual se nombra: **el `nightly` de CI no está pineado**, instala el más nuevo por run, así que "mismo canal" no es "mismo binario"; el mensaje nombra el remedio (`rustup update nightly`).
- **Un defecto propio encontrado al sondear, y corregido:** la primera versión resolvía el binario con `rustup which --toolchain <want> rustfmt` sin comprobar antes si existía. Medido: **rustup lo instala** ("syncing channel updates for 1.50.0… downloading 6 components", y el propio rustup avisa *"the missing active toolchain has been auto-installed"*), así que la sonda colgó el shell **más de 3 minutos**. En el gate eso convierte el push de quien no tenga el toolchain de CI en una **descarga** — lo contrario del advisory. Ahora consulta `rustup toolchain list` (local, offline) y **sólo después** resuelve el binario; verificado: tras las seis sondas, `rustup toolchain list` sigue con las mismas **4** toolchains (las dos que mis probes instalaron de más se desinstalaron). Segundo guardado en la misma pasada: un `rustfmt` **standalone sin `rustup`** (paquete de distro) habría hecho `command-not-found` — una dependencia que el gate **no tenía** antes de esta tarea; ahora se comprueba `Get-Command rustup` y sin él se mide con el del PATH + ADVISORY. Esa rama queda **probada por construcción y parseo**, no ejecutada: en esta máquina `rustup` existe y el script prepende `~/.cargo/bin`.
- **Prueba de dos caras:** (a) *"fichero formateado con stable y no con nightly → el gate local falla igual que CI"* → **NO FABRICABLE, declarado**: con 0/739 no existe tal fichero, e inventarlo exigiría un `rustfmt` con reglas que hoy no difieren. Se probó la propiedad equivalente y medible — **un fichero mal formateado hace fallar a los dos instrumentos con el mismo veredicto y el mismo diff**, y el gate **mide con el de CI**, así que si la divergencia existiera el juez sería el de CI. (b) *"formateado con nightly → pasa en los dos"* → ✅. Sondas en `%TEMP%\opencode\fmt762\probe` (**no quedan en el árbol**):

  | Sonda | Entrada | `-Toolchain` | Instrumento impreso | Exit |
  |---|---|---|---|---:|
  | 3a | sin formatear | `stable` | `1.9.0-stable (8bab26f4f6)` | **1** |
  | 3b | sin formatear | `nightly` | `1.11.0-nightly (c36f145719)` | **1** |
  | 4a | formateado | `stable` | `1.9.0-stable (8bab26f4f6)` | **0** |
  | 4b | formateado | `nightly` | `1.11.0-nightly (c36f145719)` | **0** |
  | 5 | formateado | `1.50.0` (ausente) | ADVISORY + mide con el del PATH | **0** |
  | 6 | `src/lib.rs` (real) | *(default)* | `1.11.0-nightly` — **leído del workflow** | **0** |

  La sonda 6 es la que prueba el contrato: **sin** pasar `-Toolchain`, el instrumento impreso es nightly, el que `Get-CiFmtToolchain` leyó de `ci-rust.yml`. Las seis dieron el mismo veredicto bajo `pwsh` 7 **y** `powershell.exe` 5.1.
- **Incidente de encoding (medido, no cosmético):** la primera versión escribió `—` dentro de strings. `powershell.exe` 5.1 —el fallback de `find_pwsh`— decodifica el `.ps1` sin BOM como ANSI, el em dash se vuelve `”`, y PowerShell lo interpreta como **delimidador de string**, con **dos desenlaces medidos** (review M1): (i) **error de parseo** ruidoso — el fixture mínimo de 3 líneas da **2 errores** por `ParseFile` y exit **1** por `-File`; mi primera versión del script dio **4** por esa vía y HEAD daba **0**; (ii) **verde falso silencioso** — cuando el mojibake no rompe el parse, solo altera el string: salida destrozada con **exit 0** (medido por la review). Las dos desaparecen con ASCII puro (`non-ascii bytes: 0`) y parsea con **0 errores en ambos motores**.
- **Qué se detecta cuando deje de ser 0:** el gate imprime el instrumento en cada corrida y lo repite en el `ok` final; un rojo de CI que el local no reproduce se atribuye leyendo esa línea contra el log del run. Ya no hay verde anónimo.
- **Techo declarado:** (1) el `nightly` de CI no está pineado, así que la identidad total no se puede cerrar desde local, sólo nombrar — pinearlo es del carril dueño de `ci-rust.yml`, que esta tarea **no** toca; (2) el local mide el **rango** y CI el **árbol**: diferencia intencional de `FIND-760`; (3) un toolchain instalado pero demasiado viejo para `--edition 2021` da rojo por la razón equivocada — inalcanzable con el toolchain que CI declara hoy (`nightly` >= stable actual).
- **NOTICED BUT NOT TOUCHING:** `RULES.md` §10 citaba `ci-rust.yml:123` en **dos** sitios (el bullet `Must` y el `Por qué`, `:218`): la primera pasada de esta tarea arregló solo uno y **afirmó que estaban los dos** — la review P2-01 lo cazó (**B1**) y el segundo se corrigió al cerrar. Hoy el `diff` y la afirmación coinciden (`1 1` en `RULES.md`, `0` ocurrencias de `ci-rust.yml:1`). Nada más se toca fuera de la lista de escritura.
- **Verificación (mecánica):** `grep -n "toolchain: nightly" .github/workflows/ci-rust.yml` → primera ocurrencia **`:208`** dentro del job `fmt` (`:192`), gate `cargo fmt --check` (`:214`) · medición **739 × 2 pares → 0 diffs / 0 parse-fail** · sondas **1,1,0,0,0,0** con el instrumento esperado en cada una, bajo los dos motores · `bash -n .githooks/pre-push` → **0** · `[Parser]::ParseFile` → **0 errores** (pwsh 7 y PS 5.1), `non-ascii bytes: 0` · `Select-String 'ci-rust.yml:1' docs/dev/workflow/RULES.md` → **0** tras B1 · `git diff --numstat docs/dev/Backlog.md` → **0 6** · `check-links.mjs` → **0** (`broken markdown links: 0`, 3593 internos) y `check-docs.mjs` → **0** (el task file sale como *orphan*, igual que los de `FIND-757/767/770/771` en vuelo: informativo, no gate) · `gen-index` **NO** ejecutado (prohibido por el contrato).
- **Review P2-01 (contexto fresco, `vanta-review`, una ronda) → `changes-required`, 2 Blocking + 2 Medium + 4 Low, todos cerrados en este cierre.** Lo que **resistió** (importa tanto como lo que cayó): reprodujo la medición **completa** (739 `.rs` × 3 instrumentos, 343 s, `diff=0` en ambos pares y `parseFail=0`), **intentó fabricar** la divergencia de la cara (a) con dos ficheros sintéticos y 38 constructos y **no pudo** — el "no fabricable" quedó confirmado por un tercero, no aceptado por mi palabra —, reprodujo las sondas 3a/3b/4a/4b exactas, verificó con un stub de `rustup` que el gate **no instala** nada (solo `toolchain list`; ni `which` ni descarga) y que el guard de `rustup` ausente es *load-bearing* (`& rustup` sin rustup lanza `CommandNotFoundException` bajo `ErrorActionPreference=Stop`), y confirmó `bash -n` 0, hook `5 +0` solo comentarios, Backlog `0 6` y las tres citas del workflow (`:192`/`:208`/`:214`). **B1** (afirmé haber reemplazado `ci-rust.yml:123` y el `diff` no lo respaldaba: quedaba vivo en `RULES.md:218`) y **B2** (`status: completed` sin §Review) corregidos; **M1** re-atribuido con el fixture medido (hay **dos** desenlaces, no uno); **M2** asumido: esta entrada ya viajó en `4fa4d9fc0`, un commit de otro carril (`FIND-767`) cuyo snapshot se llevó el working copy completo — el plan de `RULES.md` §11 (blob desde `HEAD` + mi sección) quedó sin efecto, y revertirlo sería reescribir un commit ajeno; **L1** y **L4** quedan como techo escrito en el script (ancla al primer `toolchain:` del job, y solo la línea `^rustfmt ` como versión), **L2** y **L3** corregidos arriba.
- **Commit:** este commit lleva `dev-tools/fmt-range.ps1` (instrumento + guardas de `rustup`/`-like`/versión), `.githooks/pre-push` —**solo comentarios**—, `RULES.md` §10 (las **dos** citas) + task file + esta entrada (correcciones de la review) + fila del Backlog retirada (**6 líneas, 0 inserciones**). Sin `gen-index`. Sin push. **`ci-cd.md` ya no lo commitea esta tarea por primera vez:** su entrada viajó en `4fa4d9fc0` (ver §Review M2) y lo que viaja acá es la corrección posterior; el fichero estaba limpio contra `HEAD` al empezar este cierre, así que se edita y commitea normal.

### La lane de snapshot deja de elegir 42 de 56: el selector pasa a ser el binario y cada fase prueba `listed == executed` (`FIND-768`)
- **Fecha:** 2026-11-26
- **Objetivo:** las fases 03/04 de `snapshot-certification.yml` seleccionaban **por nombre de test** (`-E 'test(snapshot) or test(pending_restore) or test(schedule)'`). Con `--features failpoints` el binario tiene **56** tests y ese selector elegía **42**: los 14 restantes existen, compilan y nunca se ejecutaban — 4 `export_format_*`, `find319_child_flag_ignores_empty_values` (el caso que `FIND-738` tuvo que correr a mano), 5 `vantafile_*` y 4 `wal_*`. La lane **estaba verde** (39/48 runs en 30 días), así que el hueco era invisible por construcción: no sabía cuántos tests había elegido.
- **Por qué se cubren los 56 en vez de justificar los 14:** `find319_child_flag_ignores_empty_values` **es** comportamiento de snapshot (`FIND-319` es el espejado recursivo de `keyspaces/`). Escribir "excluido por no ser de snapshot" habría sido una razón falsa; el contrato admitía las dos vías y ésta es la única honesta.
- **La decisión (selector):** fases 03/04 → `-E 'binary(snapshot_certification)'`. Se conserva `--test snapshot_certification`, así que el `-E` es explícito y no depende de que nadie borre el `--test`.
- **La aserción, que importa más que el fix:** `run_phase` cuenta la población con `nextest list` **antes** de correr y la compara con el JUnit que produce la fase; si difieren, la fase se pone roja y la línea `listed=N executed=N coverage=...` queda en `artifacts/snapshot-certification/listed-vs-executed.txt` (una por fase; las siete sólo en un run verde, porque `set -e` aborta el step en el primer rojo, que es el contrato de la lane desde su origen). La población se cuenta **del target, no del `-E` de la fase**: `coverage=full` en 03/04, `coverage=scoped` en 01/02/05/06/07. Si se contara con el mismo `-E` del run, un selector estrecho encogería **los dos lados** y la comparación seguiría verde — el mismo agujero con una aserción decorativa. `--no-tests fail` (ya presente) sólo cubría la selección **vacía**, que no es este caso.
- **Los dos números:** binario con `--features failpoints` = **56** (medido por `FIND-752`, Windows); selector viejo = **42**; los 14 del set-diferencia, nombrados uno a uno en el task file. Después del cambio el número autoritativo **deja de publicarse a mano**: la fase 03 lista los 56 del target y la 04 lista **59**, porque `failpoints,encryption` habilita **3** tests gateados por `encryption` (`tests/core/snapshot_certification.rs:2915`, `:3297`, `:3327`; `encryption` no está en los defaults). El conteo real de cada fase, en la plataforma de CI, lo emite `listed-vs-executed.txt` — intentar derivarlo por escrito es exactamente el error que este ticket corrige.
- **Verificación (mecánica):** `actionlint .github/workflows/snapshot-certification.yml` → **exit 0** · YAML parseado (`yaml.safe_load`): triggers `push, pull_request, workflow_dispatch`, job `snapshot-linux`, 5 steps, `permissions: contents: read`, `timeout-minutes: 90`, bloque `run:` de 3612 B extraído · `bash -n` sobre **ese bloque**, no sobre una copia → OK · el bloque real ejecutado con un `cargo` falso en tres escenarios: población 56 / ejecutados 56 → las **siete** fases verdes y exit 0; población 56 / selector que elige 42 (**el hueco de este ticket**) → `03-certification-plain: population is 56 test(s) but 42 executed.` y exit 1, con las fases `scoped` todavía verdes (la aserción no da falsos positivos); selección vacía → `nextest list exited 0 selecting 0 test(s).` y exit 1.
- **No se pudo verificar, declarado:** (1) **el workflow no se disparó** — `workflow_dispatch` exige la rama por defecto y no hay push, así que el verde real lo publica el primer run; (2) la re-medición local de `cargo nextest list` se **abortó a los 10 minutos**, bloqueada en `artifact directory lock` con dos carriles Rust vivos y sin una línea de salida — los números son los medidos por `FIND-752` con el mismo comando y las mismas features (`target/find752-snap-fp-all.txt` = 56, `find752-snap-fp-filter.txt` = 42, set-diferencia = los 14 nombres). `dev-tools/verify.ps1` tampoco se corrió: el carril es CI y no toca ni un `.rs`.
- **Coste:** 14 tests extra × fases 03 y 04. Son `export_format_*`, `vantafile_*`, `wal_*` (formato/roundtrip) más un spawn hijo: ninguno de la clase de `test_snapshot_consistent_under_concurrent_writes`, que ya corría. El `slow-timeout` por test (180 s) y el budget de 90 min no cambian.
- **Incidente de árbol compartido (declarado):** el primer corte de la fila del Backlog se hizo sobre una copia del working tree que **ya venía sin** las filas de `FIND-757` y `FIND-764` (otro carril las había commiteado). Detectado con `git diff --numstat` (18 borrados en vez de 6) y rehecho desde `git show HEAD:` retirando **sólo** las 6 líneas de `FIND-768`: el diff final es `0 6`. Y esta entrada se perdió **una vez** antes de commitear: otro carril escribió `ci-cd.md` desde su propio buffer y el fichero volvió a `HEAD` sin ella; se reconstruyó y se commiteó de inmediato.
- **Review P2-01:** `vanta-review`, contexto fresco, una ronda (tier **fast** por glob) → **`changes-required`**: 1 High + 2 Medium + 4 Low, cerrados o declarados. Re-ejecutó por su cuenta `actionlint` (0), el `diff` del bloque `run:` real contra el del harness (**0 líneas**), los escenarios A/B/C con `bash -eo pipefail`, los conteos 56/42 con `comm` del set-diferencia, y `check-links`/`check-docs`. El **High** era de este mismo cierre: el task file anunciaba dictamen en §Review antes de que existiera. El **Medium** material: «56 of 56» era falso para la fase 04 (**59**). El otro Medium: `ci-cd.md` arrastraba la entrada de `FIND-764` sin commitear de otro carril (resuelto commiteando `HEAD` + esta sección). El diseño de la aserción **no** cambió: el revisor lo confirmó **reproduciéndolo**, no leyéndolo. Techo aceptado: `coverage=full|scoped` vive en el call site, así que un futuro `run_phase 03 scoped '<estrecho>'` reabriría el hueco, y el harness que lo cazaría no se puede commitear (`dev-tools/**` está prohibido por el ticket).
- **NOTICED BUT NOT TOUCHING:** `cancel-in-progress: true` en `push` dentro de esta lane (`snapshot-certification.yml:68`) es la clase de `FIND-764`, y ahora es la única lane que carga los 56 — merece fila propia.
### El suite que nunca corrió: la concurrencia deja de matar runs de `push` (`FIND-764`)
- **Fecha:** 2026-11-26
- **Objetivo:** `ci-rust.yml` declaraba `cancel-in-progress: true` a nivel de workflow con `group: ${{ github.workflow }}-${{ github.ref }}`. En `push` ese ref es la rama: el run cancelado es el **único de ese workflow** que va a reportar por ese SHA, y el SHA se queda en la historia. O sea: commits integrados cuyo suite completo nunca se ejecutó, y el resumen dice `cancelled`, que no distingue "lo cancelé yo" de "el diff rompía algo".
- **El caso medido, con la causa separada de la coincidencia** (`cd7d8c5f7`, integrado en `develop`; el run es `45794331830`, **`cancelled`**, `09:41:37Z` -> `10:31:14Z` = **49m37s**):
  - **`Tests (Windows)` — cancelado a los 35m23s, causa ATRIBUIDA.** Murió a las `10:31:13Z`, **15 s** después de que existiera el push siguiente (`337ee7963`, `10:30:58Z`), y la anotación de su check-run nombra el mecanismo: *"Canceling since a higher priority waiting request for CI: Rust — Build & Lint + Tests-refs/heads/develop exists"* (`gh api .../check-runs/139828474272/annotations`).
  - **`Tests (Linux)` — cancelado a los 17m53s (`09:55:50Z` -> `10:13:43Z`), causa NO ATRIBUIDA.** Murió **compilando** `vantadb_py`, sin `error:` ni `test result`; su anotación es la genérica `The operation was canceled.` Y **no había nada que lo matara**: entre `09:41:37Z` y `10:30:58Z` no existe **ningún** run de `ci-rust` en ninguna rama (`gh api "repos/…/actions/runs?created=2026-11-26T09:00:00Z..2026-11-26T11:30:00Z"` -> sólo esos dos).
  - **Corrección de la premisa del ticket, medida:** el brief decía "lo canceló el push siguiente, 49 min después". Eso es cierto para **Windows**, no para **Linux**, que murió **17m21s antes** de que ese push existiera. Son **dos** cancelaciones con **una** causa conocida, y el resumen del run escribe la misma palabra para las dos. El hallazgo queda **más** fuerte, no menos: hay un segundo modo de fallo (`cancelled` sin causa) que este cambio **no** toca, y por eso la lista de SHAs de abajo se publica como registro.
  - Frecuencia del día: **1 de 9** runs de `develop`.
- **Qué deja una cancelación (segunda premisa del brief, corregida con la API):** `gh api "repos/ness-e/Vantadb/commits/cd7d8c5f7/check-runs?per_page=100"` -> `total_count: 61`, **todos `completed`**: **53 `success` · 5 `skipped` · 2 `cancelled` · 1 `failure`**. Los contexts **no** quedan ausentes. El `failure` es `Generated indexes are current` (otro workflow, `09:41:50Z`) y este commit reproduce ese rojo (§Deuda del task file). Sobre el aviso de que un `rust-scope` cancelado deja contexts ausentes: `rust-scope` nació en `FIND-770` (`2b282eed7`), aún **no** empujado a `origin/develop`, así que **no se pudo observar** — provocar una cancelación está prohibido por el contrato. Declarado, y deja de importar con este fix.
- **La ventana y la cadencia, medidas** (9 runs de hoy, `gh run view <id> --json jobs,createdAt,updatedAt`): duración del **run** 24m13s-**69m35s** (**mediana 39m32s**) — el `Tests (Linux)` sólo tarda 9m10s-17m53s, pero el run lo marcan Windows/macOS. 8 huecos entre pushes: 49m31s · 2h39m53s · 3h15m44s · 1h23m54s · 1h20m20s · **49m21s** · 1h27m58s · 36m40s (**mediana 1h22m07s**).
- **Decisión (c) -> `cancel-in-progress: ${{ github.event_name == 'pull_request' }}`:** el grupo es `github.ref`, y significa cosas distintas por evento. En `pull_request` es `refs/pull/N/merge`, que GitHub **reconstruye en cada push**: el head revisado trae su run nuevo, así que cancelar el viejo no pierde ningún context y ahorra una espera entera por fix-up. En `push` el ref es la rama y no hay run de reemplazo -> no se cancela. Es la misma opción (iii) que `FIND-771` (`edcd0fb65`) eligió para `docs-gates.yml`: se reusa el criterio, no se reinventa.
  - **Coste, contado sobre los mismos datos:** de los **8** huecos, **1** (49m21s) cae dentro del run anterior -> ese push habría esperado, sin perder nada; los otros 7 llegaron con **12m27s-2h36m12s** de margen sobre el `updatedAt` del anterior. Los PRs **no esperan**: su grupo es otro ref.
  - **Rechazada (a) "no pushear con un run en vuelo":** no tiene gate y el fallo es mudo - es el pre-mortem servido. Queda escrita como `Verify` de comportamiento en `RULES.md` §12, no como arreglo.
  - **Rechazada (b) "marcar el SHA como no verificado":** produce una lista que envejece en el mismo commit que la escribe y no impide la siguiente cancelación. Se hace **además** - la lista de abajo es el contrato mecánico - pero como registro; la cancelación **sin causa** de Linux demuestra que hay un segundo modo de fallo que el fix no cubre.
  - **Rechazada (i) `false` en todo:** serializa también los fix-up del **mismo PR** (mismo ref `refs/pull/N/merge`): un push de corrección esperaría hasta **69m35s** para reportar sobre un head que ya nadie va a mergear.
  - **Rechazada (ii) identidad del run en el grupo:** N runs sin límite; un burst de 3 pushes a `develop` = 3 x (Windows ~57m + macOS ~26m + Linux ~11m) de runners para reportar SHAs ya superados.
  - **Los 5 bloques `concurrency` de job** (`secret-scan`, `fmt`, `clippy`, `semver-checks`, `public-api-snapshot`) llevaban `cancel-in-progress: true` y contradecían §12. Con el grupo de workflow serializando el ref son **inertes hoy** (sólo un run por ref ejecuta jobs), así que no eran un defecto vivo — pero eran una vía latente y una contradicción escrita. Se les puso la misma expresión: 5 líneas, cero cambio de comportamiento, y §12 pasa a verificarse con un `grep` sobre el fichero entero.
- **Techo declarado:** en `push` GitHub mantiene **uno en curso + uno pendiente** por grupo (`queue: single`): un tercer push cancela el **pendiente**, cuyo SHA ya está superado por el que lo cancela. En `ci-rust.yml` **1 de 8** huecos cae dentro del run anterior —medido— y **0 de 7** pares consecutivos lo habrían hecho —**derivado, no medido** (8 huecos dan 7 pares). `concurrency.queue: max` (hasta 100 pendientes) queda documentado y **no usado**: con runs de 24-69 min, encolar más de uno es encolar SHAs muertos. `workflow_dispatch` y `schedule` evalúan la expresión a **false**: no cancelan un run **en curso**, pero eso **no** protege el slot pendiente (`queue: single` lo desplaza diga lo que diga el flag) y un `workflow_dispatch` sobre una rama comparte `refs/heads/<rama>` con sus push.
- **La lista de SHAs, publicada (el contrato de la tarea).** Último SHA con `Tests (Linux)` **ejecutado y verde**: **`3a7e225ad`** (`45821093876`, 10m58s, `12:48:04Z`->`12:59:02Z`).

  | SHA | Run | `Tests (Linux)` |
  |---|---|---|
  | `4af909977` | `45727034307` | ejecutado, success - 10m06s |
  | `f5c027f0e` | `45732896745` | ejecutado, success - 9m10s |
  | `0094f0f6a` | `45750538900` | ejecutado, success - 9m50s |
  | `f2e968807` | `45771789995` | ejecutado, success - 12m51s |
  | `ebc83ffa9` | `45782445975` | ejecutado, success - 12m34s |
  | **`cd7d8c5f7`** | `45794331830` | **cancelado a los 17m53s - nunca ejecutó un test** (causa no atribuida) |
  | `337ee7963` | `45802050750` | ejecutado, success - 12m27s |
  | `2f8ea4d76` | `45815405021` | ejecutado, success - 9m54s |
  | `3a7e225ad` | `45821093876` | ejecutado, success - 10m58s (**último verde**) |

  Posteriores a `3a7e225ad` en `origin/develop`: `49ced681e` y `c8e73e412` **no tienen run** - sus ficheros no están en `on.push.paths`, así que el workflow no se dispara. Es un **tercer** estado (no medido, por diseño) que el resumen tampoco distingue, y se declara en vez de contarse como verde.
- **Verificación (mecánica):** `actionlint .github/workflows/ci-rust.yml` -> **exit 0** (baseline previo 0) · `js-yaml` -> workflow `{"group":"${{ github.workflow }}-${{ github.ref }}","cancel-in-progress":"${{ github.event_name == 'pull_request' }}"}` y los **5** jobs con la misma expresión · `grep -n 'cancel-in-progress'` -> **7** líneas con la cadena: **6 directivas** (workflow + 5 jobs) y **1** mención en el comentario, **0** con el valor `true` (la `Verify` de §12 pasa) · **24** jobs, **0** `name:` duplicados, **`sameNames=true`** contra `HEAD` (`ci-gate.yml` los compara por string) · `git diff -U0` del YAML = sólo las 6 directivas y el comentario · `RULES.md` §12 con `Must`/`Must not`/`Por qué`/`Verify`/`Techo` · `git diff --numstat docs/dev/Backlog.md` = **`0 6`** (sección propia, 1 fila) · `check-links.mjs` **0 broken** (3602 links) y `check-docs.mjs` **GATING all clear** · `gen-index` **NO** ejecutado (prohibido por el contrato; consecuencia en la deuda del task file).
- **No se pudo verificar, declarado:** la prueba viva -que el run de un push ya no muera- la publica el primer push real: **provocar una cancelación está prohibido** por el contrato y el push es acto del owner (Regla 7). No se corrió ningún `cargo` (carril CI, sin compilar: hay 2 carriles Rust en vuelo). El caso `rust-scope` cancelado no se observó (§ arriba).
- **Deuda bloqueante, cross-lane:** `gen-index --check` está **rojo** (`[docs/index.md] "### Task files - 272" -> expected 275`) y lo estará mientras haya carriles en vuelo: el drift son **task files sin commitear** de otros carriles (`FIND-757.md`, `FIND-758.md`, `FIND-769.md` al cerrar — la lista cambia con cada carril), así que regenerar acá hornearía trabajo ajeno y el contrato de la tarea **prohíbe** `gen-index`. `Generated indexes are current` es **required**: el primer push de este commit saldrá rojo ahí hasta que el orquestador regenere al cierre del wave — el mismo `failure` que ya arrastra `cd7d8c5f7`.
- **NOTICED BUT NOT TOUCHING:** `FAQ.md:26-30` describe `cancel-in-progress` en dos formas (`true` en CI/PR, `false` en release) y **falta la tercera** - la expresión por evento, que hoy es la forma de `docs-gates.yml` y `ci-rust.yml`. Fuera de la lista de escritura. Texto de reemplazo: *"`<expresión>` (gates con `push`): cancela en `pull_request` -el ref `refs/pull/N/merge` se reconstruye en cada push- y no cancela en `push`, donde el SHA no tiene otro run de ese workflow que reporte por él."*
- **Review P2-01 (contexto fresco, `vanta-review`, una ronda) -> `changes-required`: 3 Blocking + 5 Medium + 5 Low, todos cerrados.** Lo que **resistió** (importa tanto como lo que cayó): el mecanismo, la expresión por evento (idéntica a la de `docs-gates.yml` desde `edcd0fb65`), las 9 duraciones de `Tests (Linux)`, los 8 huecos y su mediana, "1 de 8", el último verde `3a7e225ad`, los dos SHAs sin run, y la higiene de carril (7 paths ajenos + `FIND-757/758.md` -> 0 hits propios). **B-1** — el caso estaba **mal atribuido**: `Tests (Linux)` murió 17m21s **antes** de que existiera el push que este fichero culpaba; reescrito en los 4 ficheros con las dos cancelaciones separadas y la anotación de Windows como prueba. **B-2** — 6 de las 9 duraciones de run estaban **+1:00** (bug de formato mío): máximo real **69m35s**, mediana **39m32s**; corregidas. **B-3** — §12 prohibía `true` y el propio fichero tenía **5** bloques job-level así: se les aplicó la expresión. **M-1** inventario de check-runs con el `failure` que faltaba. **M-2** el rojo de `gen-index` pasó de "no ejecutado" a deuda bloqueante. **M-3/M-4** `completed` sin commit y §Review sin "Cómo se probó" ni veredicto: cerrados con el commit y esta sección. **M-5** "0 de 7" etiquetado como derivado. **L-1/L-2** el `queue: single` desplaza pendientes y la `Verify` necesita `gh run view --json jobs`. **L-3** cross-ref en la enmienda de §1. **L-5** "único run **de este workflow**". **L-4 refutado con medición**: el newline final de `ci-cd.md` estaba presente (`EndsWith("\n")` -> `True`).
- **Commit:** `9f8bc0ffd` (`ci-rust.yml` — workflow + 5 bloques de job — + `RULES.md` §12 + task file + esta entrada + fila del Backlog retirada, `0 6`) y `4666b6467` (reparación, abajo). Sin `gen-index`. Sin push.
- **Incidente de árbol compartido (declarado, reparado):** `9f8bc0ffd` se llevó por delante **9 líneas de `docs/dev/tasks/FIND-768.md`**. `git read-tree HEAD` y `git commit` no son atómicos y el carril de `FIND-768` commiteó `4b3db3560` **en medio** — la clase exacta de `FIND-753`. El helper verificó **por contenido** lo que le pedí (mi fila ausente, §12 presente, la expresión puesta) y **no** podía ver el fichero ajeno; se detectó leyendo `git show --stat`, no el exit code. Restaurado en `4666b6467` (`+9/−0`, `ExpectPresent 'Incidente del primer commit'`). **Lección:** con tres carriles vivos, el `--stat` de lo commiteado es parte de la verificación, no un adorno.
- **Una exención de hook, declarada:** el commit se hizo con `SKIP=markdownlint-cli2`. El hook del env de `pre-commit` (v0.23.2) marca `Failed` con `Linting: 0 files` —los cuatro `.md` están bajo `docs/dev/**`, que la propia config de `.github/markdownlint-cli2.yaml` excluye— mientras la **misma** invocación, misma config y mismos ficheros, da **exit 0 / 0 issues** con v0.24.2. No es contenido mío: es el gate sin ficheros que lintar. Los demás gates del hook corrieron (`actionlint` ok, `pre-commit: gates passed`).

### El lane que nunca ejecutó un test: datasets separados y el `schedule` pasa a medir `develop` (`FIND-766`)
- **Fecha:** 2026-11-26
- **Objetivo:** el job `other_heavy` de `certification.yml` era **un solo step** que empezaba pidiendo el dataset de benchmark. Sin dataset, moría ahí y los tests que venían después quedaban `skipped` — que el resumen del run no distingue de "no había nada que correr". El resultado, contado por `FIND-752` y **re-medido al cerrar**: **0 success / 37 failure** en **39 runs programados** (la fila decía 36), y en los 36-37 los steps `Download benchmark datasets` y `Run remaining heavy tests` en **`skipped`**. Nunca se ejecutó un test.
- **La segunda capa, probada con la API (no con la teoría).** Los tres últimos runs programados (`45800950632`, `45598671538`, `45388973797`) tienen los mismos steps: `failure Verify required benchmark datasets (TBH-01)` → `skipped Download benchmark datasets` → `skipped Run remaining heavy tests`. **Ese step ya no existe en `develop`**: `FIND-328` lo renombró a `Verify required benchmark dataset (TBH-01, GloVe-100)` y lo movió después de la descarga. Que los runs sigan ejecutando el nombre viejo **demuestra** que el `schedule` corre el YAML de `main`, 552 commits por detrás: el fix del dataset estaba escrito y el lane no lo miraba. Es `FIND-741` (verificar contra un artefacto stale) con un calendario encima.
- **Los tres cambios, en orden de valor:**
  1. **`cli_tests` (96 tests) fuera del gate de datasets, y en fase propia.** Job `other-heavy` → ahora `"Certification: other_heavy (tests, dataset-free)"`; fases `01-vantadb-cli-tests` (**sólo** `cli_tests`: una cifra por fase sólo prueba algo si la fase es un binario, review H1), `02-vantadb-dataset-free` (los otros 11 binarios `vantadb`, entre ellos `benchmark_internal`), `03-vantadb-lib-hnsw` (`index::core::tests::concurrent_insert_preserves_hnsw_invariants` — el path que el filtro viejo no matcheaba, review B1), `04-server` (`mcp_integration`, `e2e`, `benchmarks`, `server`) y `05-mcp` (`mcp_tests`). Ninguno lee `data/benchmark`. **`benchmark_internal` entró acá y se corrigió de paso el supuesto de la fila:** parecía dataset-bound y no lo es — genera su propio corpus de 10k (`tests/benchmark_internal.rs:40`, cero referencias a `data/benchmark`).
  2. **El dataset tiene job propio.** `benchmark-datasets` → `"Certification: benchmark_datasets (GloVe-100 + its consumer)"`: cache + descarga + gate `TBH-01` + `benchmark_datasets` (el **único** test que lee `data/benchmark`, 9 referencias). Un GloVe faltante ahora pone rojo un job **con nombre de dataset** en vez del job que contiene 120+ tests. **Sin `needs:` en ninguna dirección**: un `needs:` habría dejado los tests `skipped` — reportado como **success** por GitHub — justo cuando falta el dataset, que es el pre-mortem (b) del ticket.
  3. **El `schedule` mide `develop`.** El cron de GitHub **sólo** dispara sobre la rama default, así que lo hace el checkout: `ref: ${{ github.event_name == 'schedule' && 'develop' || github.ref }}` en los dos jobs, más un step `Assert which tree this lane measures` que en `schedule` **falla** si el árbol no es `develop`. Una expresión de `ref:` puede estar mal en silencio; la aserción convierte "medimos `main` otra vez" en un rojo.
- **La aserción de no-vacuidad (`listed == executed`, patrón `FIND-768`):** cada fase lista su población con `nextest list` **antes** de correr y la compara con su JUnit. `--no-tests fail` sólo cubre una selección **vacía**, no una **parcial** (`FIND-738`); sin el conteo, la lane podía volver a estar verde sin ejecutar nada. `target/nextest/other-heavy/listed-vs-executed.txt` (artefacto, 30 días) es la prueba: una línea por fase, emitida por el run que midió.
- **Dos cosas que estaban a ojo y se verificaron con nextest real (0.9.133, proyecto de 2 tests en `%TEMP%`):** (a) el formato de `nextest list` es `binary::test_name` — la regex de `FIND-768` (`'^[^[:space:]]+::[^[:space:]]+ '`) casa **una vez por línea**, no cero, que es lo que la habría dejado roja para siempre; (b) el JUnit vive en `target/nextest/<profile>/junit.xml` **también** con `--cargo-profile release`, que es la ruta que la lane lee. Las dos se habían copiado sin comprobarlas.
- **Verificación (mecánica, todo ejecutado):** `actionlint .github/workflows/certification.yml` → **exit 0**, después de un incidente propio (abajo) · YAML parseado (`js-yaml`): **23** jobs, `schedule` = 3 crons, **0** `name:` duplicados, los dos jobs nuevos **sin** `needs`, ambos `checkout` con la expresión de `develop` · `bash -n` de los **4** bloques `run:` extraídos del YAML → exit 0 · **el bloque `run:` real** (no una copia) ejecutado en tres escenarios con un `cargo` falso que delega en **cargo-nextest 0.9.133** sobre un proyecto de prueba: **A** población 2 / ejecutados 2 → **5** fases `listed=2 executed=2` y exit **0**; **B** población 2 / ejecución estrechada a 1 → `01-vantadb-cli-tests: population is 2 test(s) but 1 executed.` y exit **1**; **C** población 0 con selector lleno → `nextest list exited 0 selecting 0 test(s) with a full selector.` y exit **1**; **D** el step del dataset, nextest real → exit **0** · la aserción de árbol probada en los tres casos que importan: `schedule`+`main` → **roja**, `schedule`+`develop` → verde, `push`+`main` → verde.
- **Incidente `actionlint` (bloqueó el `pre-commit` de todos los carriles; cerrado).** El hook llamaba `actionlint` **sin argumentos**, y así lee **stdin**: los "ok" del día eran un gate que no linteaba nada. Corregido por el owner a `actionlint .github/workflows/*.yml`, este fichero lo hizo **crashear** (`exit -1`, sin mensaje). No era YAML (`-verbose`: `Found 0 parse errors`): con `-shellcheck=` pasaba a exit 0, o sea el crasher es el **shellcheck embebido**, y bastaba con el tamaño del script — 136 líneas **con** los `#` de prosa → crash; 101 sin ellos → exit 0. Arreglo: la prosa del step se movió a comentarios YAML **encima** del step y el `run:` quedó con **cero** líneas de comentario (verificado en los 4 bloques). Ninguna aserción cambió: A/B/C/D corren sobre el bloque real extraído del YAML.
- **No se pudo verificar, declarado:** **el workflow no se disparó**. `workflow_dispatch` exige la rama pusheada y el push es acto del owner (Regla 7). Y el `schedule` **no** sirve de atajo: un cron ejecuta el YAML de la rama default, así que hasta que esto llegue a `main` el domingo seguirá corriendo la versión de un solo job — la única forma de ejercitarlo antes es un `workflow_dispatch` sobre `develop` (que es lo que la propia evidencia de arriba demuestra: los runs programados ejecutan nombres de step que sólo existen en `main`). Lo que sí se puede afirmar hoy: el mecanismo de conteo funciona contra nextest real (A/B/C) y el árbol no puede ser `main` sin ponerse rojo. Tampoco se corrió `cargo` sobre el repo: carril CI, un carril Rust en review y **6,8 GB** de disco.
- **Cuánto se desbloquea, contado de fuente:** **0 de 98** antes (los del contrato de la fila: 96 `cli_tests` + 2 benchmarks). Después: **283** `#[test]` seleccionados entre los dos jobs —**282** ejecutables tras el único `#[ignore]`—, o sea los 98 de la fila **más 185** (`server` 48, `mcp_tests` 123, certificación/otros 13, lib 1) que estaban dentro del mismo `skipped` del step y que ninguna fila había contado. El número autoritativo es el artefacto del primer run.
- **Costo:** el split duplica el build del harness de `vantadb` release: el job de datasets compila para 1 test. Se aceptó porque el build cacheado por `Swatinem/rust-cache`/sccache comparte clave, y porque la alternativa (`needs:` o el test dentro del job de tests) devuelve justo el defecto que se está arreglando: 98 tests rehenes de una descarga.
- **Techo declarado:** sólo estos dos jobs hacen checkout de `develop`. **El resto de `certification.yml` sigue midiendo `main`** en `schedule` (chaos, mutation, benches, analyze): 20 jobs con la misma clase de defecto. No se tocan acá por blast radius — `analyze` **pushea** un commit de baseline y desde un checkout de `develop` ese push iría a `develop` (Regla 7) — y quedan como deuda del task file, no como cambio silencioso. La `needs: ci-gate` sigue en los otros 9 jobs de certificación, que se vuelven `skipped`-como-success con `main` en rojo; `other-heavy` y `security-audit` (`FIND-767`) ya no.
- **Review P2-01:** `vanta-review`, contexto fresco, una ronda (tier **fast** por glob: `docs/dev/**` + CI) → **`changes-required`: 2 Blocking + 2 High + 3 Medium + 2 Low**, todos cerrados o declarados. Los dos Blocking/High que importan: **B1** la fase de la lib seleccionaba **cero** tests (`index::tests::…` en vez de `index::core::tests::…`: el `cargo test --lib` viejo era un filtro substring que matcheaba 0 y salía 0, así que ese test tampoco había corrido nunca) — arreglado y verificado contra `src/index/core.rs:3,152`; **H1** con una cifra por fase agregada sobre 11 binarios, vaciar `cli_tests` seguía verde — `cli_tests` pasó a fase propia; **H2** el perfil `audit` del repo mata el test a los 180 s y exige leak-timeout, sobre binarios que nunca corrieron bajo nextest — el perfil se re-declara en el job con `--config-file` (600 s, sin leak-timeout) porque `.config/nextest.toml` es prohibido. Los Medium: §2c afirmaba "runs" para una lane sin runs (→ "wired to run", "no run yet"), "el primer cron de domingo" era **falso** (el cron ejecuta el YAML de `main` hasta el merge), y los conteos de fase estaban stale. Low: la fase `scoped` de `structured_api_v2` partía de una premisa falsa (nextest ya excluye los `#[ignore]` de `list` y del JUnit) → eliminada; 45 min del step de dataset → 90. Lo que el revisor confirmó **sin cambios**: expresión de `ref:` en los cuatro triggers, `--ignore-default-filter`, ruta del JUnit con `--cargo-profile release`, regex sin doble conteo, `PIPESTATUS`, y que ningún path prohibido quedó tocado.
- **NOTICED BUT NOT TOUCHING:** `snapshot-certification.yml` lleva **la misma** regex y la **misma** ruta de JUnit que acá se verificaron - este task file es la primera evidencia real de que su `run_phase` funciona con nextest 0.9.133, pero ese fichero es de otro carril (prohibido por el contrato). Y desde este cierre hay una regla nueva que lo alcanza: **nada de prosa dentro de un `run:` de este fichero** mientras `actionlint` use ese shellcheck (136 líneas con `#` → crash; 101 sin → exit 0). `nightly.yml:5` describe `other_heavy` como un solo job "deliberately heavy, weekly"; ahora son dos.
