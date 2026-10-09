---
title: "`ci-rust.yml` — CI: Rust — Build & Lint + Tests"
kind: runbook
status: active
description: "Pipeline de CI del núcleo Rust: 21 jobs — fmt, clippy, tests en 3 SO, cobertura ≥80%, gates de API pública (semver/snapshot/ADR), seguridad (audit/OSV/deny), Miri, sanitizers y combo release"
tags: [vantadb, ci, ci-rust]
---

# `ci-rust.yml` — CI: Rust — Build & Lint + Tests

> **Keep in sync:** este runbook documenta [`.github/workflows/ci-rust.yml`](../../../.github/workflows/ci-rust.yml). Los conteos, timeouts y comandos de abajo se derivan de ese YAML — al cambiar el workflow, actualizar este doc en el mismo PR (gates: `node scripts/docs/check-links.mjs` + `node scripts/docs/check-docs.mjs`).

## ¿Qué hace?

Pipeline completo de integración continua para el núcleo Rust del proyecto. Ejecuta formateo, linting, pruebas unitarias/de integración en 3 SO, cobertura de código (gate ≥80%), gates de API pública (semver-checks, snapshot dorado, ADR), auditoría de seguridad (`cargo audit` + OSV-Scanner), dependencias sin usar (`cargo-machete`), detección de UB con Miri, análisis con sanitizadores (AddressSanitizer y ThreadSanitizer), verificación de MSRV, dependencias mínimas, políticas de dependencias, compilación del combo release y checks de crates experimentales y WASM.

## ¿Cómo lo hace?

21 jobs (conteo derivado del YAML; 8 declaran `needs: [fmt, clippy]`):

| Job | Qué ejecuta | Runner | Timeout |
|-----|------------|--------|---------|
| `fmt` | `cargo fmt --check` (nightly) | ubuntu | 10m |
| `clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` (gate extendido a wasm/server/mcp — AUD-018) | ubuntu | 15m |
| `semver-checks` | `cargo semver-checks -p vantadb` contra la última versión publicada en crates.io (solo push a `main` / PR contra `main`) | ubuntu | 30m |
| `public-api-snapshot` | Snapshot dorado del API público (`nextest -p vantadb --test public_api --run-ignored ignored-only`; mismo scope que `semver-checks`) | ubuntu | 45m |
| `adr-gate` | Falla si un PR toca API pública sin agregar un ADR (opt-out `[no-adr]`; solo PR) | ubuntu | 5m |
| `test` | `cargo nextest run --profile audit` con features `cli,arrow,tls,opentelemetry` | ubuntu | 30m |
| `test-windows` | `cargo check` + clippy (excluye `vantadb-wasm`, `vantadb-server`, `vantadb-mcp`) + `nextest --profile ci-windows` | windows | 60m |
| `test-macos` | `cargo check` + clippy (mismos excludes) + `nextest --profile audit` | macos | 30m |
| `msrv` | `cargo check` con toolchain 1.94.1 | ubuntu | 15m |
| `minimal-versions` | `cargo +nightly check -Zminimal-versions` (continue-on-error) | ubuntu | 30m |
| `coverage` | `cargo llvm-cov nextest` + enforce threshold ≥80% (ADR-0015) + subida de `lcov.info` | ubuntu | 30m |
| `wasm-test` | `wasm-pack test --chrome --headless` (continue-on-error; solo PR o push que toque `vantadb-wasm/`) | ubuntu | 10m |
| `experimental-check` | `cargo check` de `vantadb-server`/`vantadb-mcp`/`vantadb-wasm` + providers por `--manifest-path` | ubuntu | 15m |
| `release-combo` | `cargo check --release` del combo del release (`server,jemalloc` + `jemalloc`, `RUSTFLAGS=-D warnings`) — pre-gate de `release-binaries` | ubuntu | 15m |
| `audit` | `cargo audit` (seguridad en dependencias) | ubuntu | 5m |
| `osv` | OSV-Scanner v2.6.0 (pinneado) sobre `./Cargo.lock` | ubuntu | 10m |
| `machete` | `cargo machete` 0.9.2 (pinneado) — dependencias sin usar | ubuntu | 10m |
| `miri` | `cargo miri test` — UB con Stacked Borrows (gate); variante Tree Borrows best-effort | ubuntu | 60m |
| `deny` | `cargo deny check` (licencias, advisories, bans) | ubuntu | 5m |
| `sanitizer-asan` | `cargo +nightly test` con `-Zsanitizer=address` (continue-on-error; salta `test_benchmark_internal_10k` y `sift1m_competitive_benchmark` — guard release-only, FIND-236) | ubuntu | 45m |
| `sanitizer-tsan` | `cargo +nightly test` con `-Zsanitizer=thread` (continue-on-error; salta tests release-only/incompatibles) | ubuntu | 30m |

## ¿Qué tests usa?

- **Perfil `audit` de nextest**: subconjunto rápido de tests (~454 tests) que excluye los heavys y de certificación.
- **Perfil `ci-windows`**: subconjunto aún más reducido para Windows.
- **Tests con Miri**: solo los tests etiquetados con `miri` en el core `vantadb` (sin feature `roaring` — croaring es C FFI y Miri no lo puede ejecutar).
- **Tests con sanitizers**: todos los tests del package `vantadb` con `--config target.x86_64-unknown-linux-gnu.rustflags = ["-Zsanitizer=address|thread", "-Cunsafe-allow-abi-mismatch=sanitizer"]`. El `--config` per-target evita instrumentar crates proc-macro (como `tokio_macros`). `-Cunsafe-allow-abi-mismatch=sanitizer` suprime el error de ABI mismatch entre crates target (con sanitizer) y sysroot (sin sanitizer). ASan salta `test_benchmark_internal_10k`; TSan salta `sift1m_competitive_benchmark`, `test_glove100_hnsw_basic`, `test_triple_backend_parity_validation`, `zero_dim_vector_search_empty` (tests que requieren `--release` o features no disponibles).

## ¿Qué verifica?

- Formateo correcto del código (`rustfmt`)
- Ausencia de warnings de Clippy (workspace completo, incluye crates experimentales)
- Tests unitarios y de integración pasan en Linux, Windows y macOS
- Compila en la MSRV (Minimum Supported Rust Version: 1.94.1)
- Compila con versiones mínimas de dependencias (`-Zminimal-versions`, continue-on-error)
- Cobertura de código >= 80% (gate, ADR-0015; reporte LCOV)
- API pública: sin breaking changes (`semver-checks`), snapshot dorado al día (`public-api-snapshot`) y ADR obligatorio si el PR toca la superficie (`adr-gate`)
- Vulnerabilidades de seguridad en dependencias (`cargo audit` + OSV-Scanner)
- Políticas de licencias y dependencias (`cargo deny`)
- Sin dependencias sin usar (`cargo-machete`)
- Ausencia de undefined behavior (Miri — Stacked Borrows gatea; Tree Borrows best-effort)
- Ausencia de memory leaks, use-after-return (ASan, continue-on-error — `--config` per-target evita el bug de proc-macros, pero nightly puede tener otros issues)
- Data races (TSan, continue-on-error — tests incompatibles skippeados)
- El combo release compila bajo `-D warnings` (`release-combo` — pre-gate del lane de release)
- Los crates experimentales (server/mcp/wasm) y providers compilan (`experimental-check`); WASM corre browser tests best-effort (`wasm-test`)

## Funcionalidad final

Garantizar que cualquier cambio en el código Rust (`src/`, `tests/`, `benches/`, `Cargo.toml`, etc.) cumple con los estándares de calidad, seguridad y portabilidad del proyecto antes de llegar a `main`. Los jobs best-effort (ASan, TSan, Miri Tree Borrows, WASM) tienen `continue-on-error: true` con `# CATEGORY:` explícita; ASan y TSan corren en paralelo (sin dependencia entre sí). Ambos suprimen el error de ABI mismatch con `-Cunsafe-allow-abi-mismatch=sanitizer` para evitar el overhead de `-Z build-std`.

## ¿Cuándo se ejecuta?

- **Push** a `main` con cambios en: `src/`, `tests/`, `benches/`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `.config/nextest.toml`, `.github/workflows/ci-rust.yml`, `deny.toml`, `osv-scanner.toml`, `rust-toolchain.toml`, `vantadb-*/**`, `vanta-memory/**`, `integrations/**`, `providers/**` (excluye `web/**`)
- **Pull Request** a `main` con los mismos paths
- **Workflow dispatch** manual
