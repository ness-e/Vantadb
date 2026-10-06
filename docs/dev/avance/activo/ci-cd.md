---
title: "Avance — CI/CD & Release"
kind: review
status: active
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
- **Plan:** `docs/dev/plans/2026-08-27-backlog-pipeline.md` Task 7 · P47 · `vanta-docs`
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
