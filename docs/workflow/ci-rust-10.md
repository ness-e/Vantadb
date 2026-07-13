# `ci-rust-10.yml` — CI: Rust — Build & Lint + Tests

## ¿Qué hace?

Pipeline completo de integración continua para el núcleo Rust del proyecto. Ejecuta formateo, linting, pruebas unitarias/de integración en 3 SO, cobertura de código, auditoría de seguridad, detección de UB con Miri, análisis con sanitarizadores (AddressSanitizer y ThreadSanitizer), verificación de MSRV, dependencias mínimas y políticas de dependencias.

## ¿Cómo lo hace?

19 jobs que se ejecutan en paralelo (con algunas dependencias secuenciales):

| Job | Qué ejecuta | Runner | Timeout |
|-----|------------|--------|---------|
| `fmt` | `cargo fmt --check` (nightly) | ubuntu | 10m |
| `clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ubuntu | 15m |
| `test` | `cargo nextest run --profile audit` con features `cli,arrow,tls,opentelemetry` | ubuntu | 30m |
| `test-windows` | `cargo check` + `clippy` + `nextest --profile ci-windows` | windows | 30m |
| `test-macos` | `cargo check` + `clippy` + `nextest --profile audit` | macos | 30m |
| `msrv` | `cargo check` con toolchain 1.94.1 | ubuntu | 15m |
| `minimal-versions` | `cargo +nightly check -Zminimal-versions` | ubuntu | 15m |
| `coverage` | `cargo llvm-cov nextest` + subida de `lcov.info` | ubuntu | 30m |
| `audit` | `cargo audit` (seguridad en dependencias) | ubuntu | 5m |
| `miri` | `cargo miri test` para detección de undefined behavior | ubuntu | 60m |
| `deny` | `cargo deny check` (licencias, advisories, bans) | ubuntu | 5m |
| `sanitizer-asan` | `cargo test` con AddressSanitizer (nightly) | ubuntu | 45m |
| `sanitizer-tsan` | `cargo test` con ThreadSanitizer (nightly, depende de ASan) | ubuntu | 60m |

## ¿Qué tests usa?

- **Perfil `audit` de nextest**: subconjunto rápido de tests (~560 tests) que excluye los heavys y de certificación.
- **Perfil `ci-windows`**: subconjunto aún más reducido para Windows.
- **Tests con Miri**: solo los tests etiquetados con `miri` en el core `vantadb`.
- **Tests con sanitizers**: todos los tests del package `vantadb` con flags `-Z sanitizer=address` y `-Z sanitizer=thread`.

## ¿Qué verifica?

- Formateo correcto del código (`rustfmt`)
- Ausencia de warnings de Clippy
- Tests unitarios y de integración pasan en Linux, Windows y macOS
- Compila en la MSRV (Minimum Supported Rust Version: 1.94.1)
- Compila con versiones mínimas de dependencias (`-Zminimal-versions`)
- Cobertura de código (reporte LCOV)
- Vulnerabilidades de seguridad en dependencias (`cargo audit`)
- Políticas de licencias y dependencias (`cargo deny`)
- Ausencia de undefined behavior (Miri)
- Ausencia de memory leaks, use-after-return, data races (ASan/TSan)

## Funcionalidad final

Garantizar que cualquier cambio en el código Rust (`src/`, `tests/`, `benches/`, `Cargo.toml`, etc.) cumple con los estándares de calidad, seguridad y portabilidad del proyecto antes de llegar a `main`.

## ¿Cuándo se ejecuta?

- **Push** a `main` con cambios en: `src/`, `tests/`, `benches/`, `Cargo.toml`, `Cargo.lock`, `build.rs`, `.config/nextest.toml`, `.github/workflows/ci-rust-10.yml`, `deny.toml`, `rust-toolchain.toml`, `vantadb-*/**`, `integrations/**` (excluye `web/**`)
- **Pull Request** a `main` con los mismos paths
- **Workflow dispatch** manual
