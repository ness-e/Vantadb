---
title: "Avance — CI/CD & Release"
type: domain-log
status: active
tags: [vantadb, avance, ci, cd, release, github-actions, docker]
last_reviewed: 2026-08-07
aliases: []
---

# Avance — CI/CD & Release

> Registro consolidado del trabajo completado sobre el pipeline: GitHub Actions, quality gates, releases, docker, wheels, changelog. IDs originales conservados.

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

### TSYS-01..16: Mejoras task-system (plan 2026-08-11-residuo-consolidado)
- **Fecha:** 2026-08-11
- **Resultado:** ✅ TSYS-01..05, 07..11, 13..16 implementados (14/16; TSYS-06 runner DEFER, TSYS-12 runtime opcional NO gate-CI). Commits: 8f774c18 (T12/T14/T15/T16), d9f2a4cb (T10/T11/T13), 138d8735 (TSYS-14/15/16), TSYS-09/ADR-017. Ver `docs/progreso/README.md` sección migradas.

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

## Docker & packaging

### WEB-02 (Docker)
- **Resultado:** ✅ Dockerfile multi-stage para la webapp (build→nginx) publicado en ghcr.

### Docker build CI (P10 competitive)
- **Estado:** Pendiente según P10 catalog (ver decision/wontfix: docker multi-arch diferido).

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
### ERR-009 (job Miri en CI cubierto) — migrado 2026-08-12 (ver docs/progreso/README.md)
### COV-004 (ADR-018 coverage gate = root crate vantadb ≥80%, supersede ADR-015) — migrado 2026-08-12 (ver docs/progreso/README.md)

### CI-04: CodeQL multi-lenguaje (rust + python + javascript-typescript) — migrado 2026-08-12 (ver docs/progreso/README.md)
- **Resultado:** ✅ `sec-codeql-30.yml` `languages: rust` → `rust, python, javascript-typescript`; timeout 30→45 min. Sin tocar queries (suite default del codeql-action). actionlint exit 0. Commits `202af1f6`, `6477aa87`.

### CI-03: SBOM multi-ecosistema (rust + npm + python) — migrado 2026-08-12 (ver docs/progreso/README.md)
- **Resultado:** ✅ `release-sbom-64.yml` genera 3 artifacts: `sbom.json` (cargo-cyclonedx, existente), `sbom-web.json` (`npx @cyclonedx/cyclonedx-npm --package-lock-only`), `sbom-python.json` (`cyclonedx-py requirements - --pyproject`). Docs sincronizadas (Regla 3): `docs/workflow/release-sbom-64.md`, `docs/ci-cd-guide.md`. actionlint exit 0 + pre-commit hook ok. Commit `a8735174`.

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

### AUD-027: Least-privilege per-job permissions in release workflow — migrado 2026-08-14 (ver docs/progreso/README.md)
- **Resultado:** ✅ Permisos movidos de workflow-level a por-job en `release-plz.yml` (release: `contents: write, pull-requests: read, id-token: write`; PR: `contents: write, pull-requests: write`); Trusted Publishing intacto, sin `CARGO_REGISTRY_TOKEN`; pin `release-plz/action@2eb1d8bcb7 # v0.5.131` confirmado correcto (tag del action vs CLI 0.3.160). actionlint exit 0. Commit `d66b267d`.

### R2: Crear agente vanta-research (read-only research subagent) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `.opencode/agents/vanta-research.md` (nuevo): mode subagent, tools read-only + web (edit/bash deny), skills coordinated-web-search/source-driven-development/progreso; 7 secciones idénticas a los 9 agentes. Contrato grep exit 0. Commit `2b4cbd6b`.

### R7: Corregir comandos de verificación rotos en Output Templates — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ `vanta-worker.md:102` `cargo check -p vantadb_py` (package real; learning AUD-039) + `vanta-docs.md:102` `target/audit-venv/Scripts/python -m pytest vantadb-python/tests/test_sdk.py`. Commit `5bda5662`.

### FND-09: Regla 8 — Concurrencia paranoica en PRs — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ Regla 8 en `.opencode/AGENTS.md` (paths multi-índice/dashmap/parking_lot/Tokio → auditoría deadlocks/data races, carga 10k w/s + 1k r/s, delegación vanta-chaos/vanta-review) + referencia en `vanta-worker.md` L104. Commit `c34a0dc8`.

### FND-17: API reference automatizada (docs-as-code) — migrado 2026-08-16 (ver docs/progreso/README.md)
- **Resultado:** ✅ análisis + plan en `docs/Investigaciones/FND-17-api-reference-docs-as-code.md`: Fase 1 cargo doc en CI (sin deps), defer typedoc/pydoc/site justificado. Citas archivo:línea + URLs verificadas. Commit `5dc71f0d`.

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
- **Resultado:** ✅ Regla 9 en `.opencode/AGENTS.md` + `benches/canonical_p99.rs` ejecutable con baseline **3.07ms p99** (`docs/operations/BENCHMARKS.md`). Commit `89943c7d`.

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
