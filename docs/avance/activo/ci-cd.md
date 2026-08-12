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
