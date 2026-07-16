# `ci-rust-10.yml` — CI: Rust — Build & Lint + Tests

## ¿Qué hace?

Pipeline completo de integración continua para el núcleo Rust del proyecto. Ejecuta formateo, linting, pruebas unitarias/de integración en 3 SO, cobertura de código, auditoría de seguridad, detección de UB con Miri, análisis con sanitizadores (AddressSanitizer y ThreadSanitizer), verificación de MSRV, dependencias mínimas y políticas de dependencias.

## ¿Cómo lo hace?

13 jobs que se ejecutan en paralelo (con algunas dependencias secuenciales):

| Job | Qué ejecuta | Runner | Timeout |
|-----|------------|--------|---------|
| `fmt` | `cargo fmt --check` (nightly) | ubuntu | 10m |
| `clippy` | `cargo clippy --workspace --all-targets --all-features --exclude vantadb-wasm -- -D warnings` | ubuntu | 15m |
| `test` | `cargo nextest run --profile audit` con features `cli,arrow,tls,opentelemetry` | ubuntu | 30m |
| `test-windows` | `cargo check` + `clippy` (excluye `vantadb-wasm`) + `nextest --profile ci-windows` | windows | 30m |
| `test-macos` | `cargo check` + `clippy` (excluye `vantadb-wasm`) + `nextest --profile audit` | macos | 30m |
| `msrv` | `cargo check` con toolchain 1.94.1 | ubuntu | 15m |
| `minimal-versions` | `cargo +nightly check -Zminimal-versions` (continue-on-error) | ubuntu | 15m |
| `coverage` | `cargo llvm-cov nextest` + enforce threshold >=59% + subida de `lcov.info` | ubuntu | 30m |
| `audit` | `cargo audit` (seguridad en dependencias, ignora RUSTSEC-2026-0176/0177) | ubuntu | 5m |
| `miri` | `cargo miri test` para detección de undefined behavior (continue-on-error) | ubuntu | 60m |
| `deny` | `cargo deny check` (licencias, advisories, bans) | ubuntu | 5m |
| `sanitizer-asan` | `cargo +nightly test --config target.x86_64-unknown-linux-gnu.rustflags = ["-Zsanitizer=address"]` (continue-on-error, solo instrumenta crates target, no proc-macros) | ubuntu | 45m |
| `sanitizer-tsan` | `cargo +nightly test --config target.x86_64-unknown-linux-gnu.rustflags = ["-Zsanitizer=thread"]` (continue-on-error, skips tests incompatibles, no depende de ASan) | ubuntu | 60m |

## ¿Qué tests usa?

- **Perfil `audit` de nextest**: subconjunto rápido de tests (~454 tests) que excluye los heavys y de certificación.
- **Perfil `ci-windows`**: subconjunto aún más reducido para Windows.
- **Tests con Miri**: solo los tests etiquetados con `miri` en el core `vantadb`.
- **Tests con sanitizers**: todos los tests del package `vantadb` con `--config target.x86_64-unknown-linux-gnu.rustflags = ["-Zsanitizer=address|thread"]`. Esto evita instrumentar crates proc-macro (como `tokio_macros`) que se compilan para el host. TSan salta `sift1m_competitive_benchmark`, `test_glove100_hnsw_basic`, `test_triple_backend_parity_validation`, `zero_dim_vector_search_empty` (tests que requieren `--release` o features no disponibles).

## ¿Qué verifica?

- Formateo correcto del código (`rustfmt`)
- Ausencia de warnings de Clippy
- Tests unitarios y de integración pasan en Linux, Windows y macOS
- Compila en la MSRV (Minimum Supported Rust Version: 1.94.1)
- Compila con versiones mínimas de dependencias (`-Zminimal-versions`)
- Cobertura de código >= 59% (reporte LCOV)
- Vulnerabilidades de seguridad en dependencias (`cargo audit`)
- Políticas de licencias y dependencias (`cargo deny`)
- Ausencia de undefined behavior (Miri, continue-on-error)
- Ausencia de memory leaks, use-after-return (ASan, continue-on-error — `--config` per-target evita el bug de proc-macros, pero nightly puede tener otros issues)
- Data races (TSan, continue-on-error — tests incompatibles skippeados)

## Funcionalidad final

Garantizar que cualquier cambio en el código Rust (`src/`, `tests/`, `benches/`, `Cargo.toml`, etc.) cumple con los estándares de calidad, seguridad y portabilidad del proyecto antes de llegar a `main`. ASan y TSan tienen `continue-on-error: true` por posibles issues del compilador nightly. TSan corre en paralelo a ASan (sin dependencia).

## ¿Cuándo se ejecuta?

- **Push** a `main` con cambios en: `src/`, `tests/`, `benches/`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `.config/nextest.toml`, `.github/workflows/ci-rust-10.yml`, `deny.toml`, `rust-toolchain.toml`, `vantadb-*/**`, `integrations/**` (excluye `web/**`)
- **Pull Request** a `main` con los mismos paths
- **Workflow dispatch** manual
