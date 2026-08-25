---
title: Cargo Feature Registry — per-feature status
type: architecture
status: active
tags: [vantadb, architecture, features, cargo]
last_reviewed: 2026-08-09
aliases: []
---

# Cargo Feature Registry

Inventario honesto de las features declaradas en `Cargo.toml` `[features]`
(del crate `vantadb`), qué gatea cada una y dónde se ejerce. Fuentes:

- `Cargo.toml` (`[features]`, líneas 96-139)
- `.github/workflows/ci-rust-10.yml` (qué features se prueban en CI)
- `rg 'feature = "…"' src/ tests/` (gates reales de código)

## Status legend

| Status | Significado |
| --- | --- |
| ✅ `default` | En la feature set por defecto; compilada en todo build |
| ✅ `ci` | Ejercida en runs de test de CI (no necesariamente default) |
| 🟡 `opt-in` | Código real detrás del flag; no default, no en features de test de CI (solo compila vía `clippy --all-features`) |
| ⚠️ `experimental` | Código real pero fuera de la superficie de producto estable (ver ADR-014 / `docs/operations/CI_POLICY.md`) |
| 💀 `no-op` | Sin gates de código; marcador vacío |

## Feature table

| Feature | Status | Qué gatea | Dónde se usa |
| --- | --- | --- | --- |
| `default` | ✅ default | Agregador: `cli, arrow, fjall, roaring, advanced-tokenizer, memmap2, fs2, sysinfo, rayon` | `Cargo.toml:97`; CI correr con default features |
| `cli` | ✅ default · ✅ ci | `bin/vanta-cli`, `cli`, `cli_handlers`, `console`; 8 tests con `required-features = ["cli"]` (`cli_tests`, `mmap_hnsw`, `file_locking_stress`, …) | `src/lib.rs:68-70,80`, `src/bin/vanta-cli.rs`; CI: `--features "cli,arrow,tls,opentelemetry"` (Linux/Windows/macOS), Miri |
| `arrow` | ✅ default · ✅ ci | `columnar` (storage columnar tipado, JSON shredding) | `src/lib.rs:74`; test `columnar` (`required-features = ["arrow"]`); CI audit features |
| `fjall` | ✅ default · ✅ ci | Backend de storage Fuji (LSM) | `src/backends/…` (19 gates); CI: default + Miri (`cli,fjall,memmap2,fs2`) |
| `roaring` | ✅ default · ✅ ci | `FilterBitset` vía `croaring` (C FFI) | `src/node.rs` (9 gates); CI default. ⚠️ Miri corre **sin** `roaring` a propósito (C FFI no ejecutable bajo Miri; fallback `Vec<u64>`) — `ci-rust-10.yml:422-426` |
| `advanced-tokenizer` | ✅ default · ✅ ci | `tokenizer` (BM25 avanzado vía `tantivy`) | `src/lib.rs:127`, `src/config.rs`, `src/tokenizer.rs` (48 gates); CI default |
| `memmap2` | ✅ default · ✅ ci | Index mmap (mmap_index / mmap_hnsw) | `src/storage/…` (9 gates); CI: default + Miri |
| `fs2` | ✅ default · ✅ ci | File locking multi-proceso | `src/storage/…` (5 gates); test `multi_process_lock`; CI: default + Miri |
| `sysinfo` | ✅ default · ✅ ci | Telemetría de hardware (RAM/CPU del host) | `src/hardware/mod.rs` (28 gates), `src/config.rs`; CI default |
| `rayon` | ✅ default · ✅ ci | Iteración paralela (bulk put/import) | `src/…` (5 gates); CI default |
| `tls` | 🟡 opt-in · ✅ ci | TLS para el server HTTP (`axum-server` + `rustls`); sin efecto si `server` no está activo | `src/cli_server.rs` (5 gates); CI: `--features "cli,arrow,tls,opentelemetry"` en las 3 OS + sanitizers |
| `opentelemetry` | 🟡 opt-in · ✅ ci | Tracing OTLP export + métricas OTel del server (`opentelemetry_sdk/rt-tokio`) | `src/cli_server.rs` (9 gates); CI: audit features en las 3 OS + coverage |
| `rocksdb` | 🟡 opt-in | Backend de storage RocksDB (opcional; no es miembro de default) | `src/backends/…` (16 gates). No está en features de test de CI (solo `clippy --all-features`); macOS instala `rocksdb` para smoke `cargo check` sin features extras |
| `remote-inference` | 🟡 opt-in | `llm` + llamadas a proveedores externos desde el executor (vía `reqwest`) | `src/lib.rs:98`, `src/executor.rs` (13 gates), `src/physical_plan.rs`; no en features de test de CI |
| `failpoints` | 🟡 opt-in | API de failpoints de chaos + `testing`; test `chaos_integrity` (`required-features = ["failpoints"]`) | `src/lib.rs:177-194`, `src/edge_index.rs`, `src/index/serialize.rs` (20 gates). ⚠️ `chaos_integrity` **no corre** en CI: audit features no incluyen `failpoints` |
| `encryption` | 🟡 opt-in | Cifrado en reposo AES-256-GCM de archivos de storage | `src/lib.rs:53` (`crypto`), 9 gates. Candidata Pro (ADR-013) |
| `server` | 🟡 opt-in | Server HTTP embebido: `circuit_breaker`, `cli_server`, `connection_pool` (requiere `cli`) | `src/lib.rs:66,72,78` (6 gates). No en features de test de CI; `vantadb-server` la activa (`vantadb-server/Cargo.toml:10`). Promovida de experimental a estable 2026-08-25: base del deploy Docker + REST `/api/v2/*` (ADR-026) |
| `tui` | 🟡 opt-in | TUI interactiva del CLI (`ratatui`+`crossterm`) | `src/lib.rs:129`, `src/tui.rs`, `src/bin/vanta-cli.rs` (3 gates) |
| `prometheus` | 🟡 opt-in | Métricas Prometheus (endpoints + counters de governor/cache warmer) | 62 gates — la superficie opt-in más grande (`src/cache_warmer.rs`, `src/memory_governor.rs`); Candidata Pro |
| `python_sdk` | 🟡 opt-in | `python` — bindings directos `pyo3` desde el crate core | `src/lib.rs:112` (2 gates). Nota: `vantadb-python` no la activa (usa `default-features=false, features=["fjall","memmap2","rayon"]`) |
| `jemalloc` | 🟡 opt-in | Allocator jemalloc para el binario `vanta-cli` (gated a unix en Cargo.toml) | `src/bin/vanta-cli.rs` (4 gates) |
| `custom-allocator` | 🟡 opt-in | Allocator custom (mimalloc) para el binario `vanta-cli` cuando jemalloc no aplica | `src/bin/vanta-cli.rs` (1 gate) |
| `async-ingestion` | 🟡 opt-in | `ingestion` — pipeline async de inserción a worker pool | `src/lib.rs:146`, `src/ingestion.rs` (1 gate en lib) |
| `async-io` | 🟡 opt-in | `transcript` — I/O async de transcriptos | `src/lib.rs:149`, `src/transcript.rs` (9 gates) |
| `hot-reload` | 🟡 opt-in | Watcher de recarga de config (JSON/TTL) vía `notify` | `src/config.rs` (7 gates) |
| `wal-shipping` | 🟡 opt-in | Envío async de WAL a réplica remota (`reqwest`) | `src/lib.rs:138` (`wal_shipping`). Feature real, documentada en ADR-014 como separada de PITR; Candidata Pro |
| `pitr` | ⚠️ experimental | `wal_archiver` — archivado WAL + point-in-time recovery | `src/lib.rs:142`. API standalone funcional y self-tested, **no integrada** en StorageEngine/SDK; ver **ADR-014** (`docs/architecture/adr/ADR-014-pitr.md`); Candidata Pro |
| `bayesian_decay` | 🟡 opt-in | Variantes de eviction con decay Bayesian Beta-Binomial | `src/eviction.rs` (17 gates) |
| `wasm` | 💀 no-op | Sin `#[cfg]` en código del crate core | 0 matches en `src/`/`tests/`; únicamente referenciada por la dependencia de `vantadb-wasm/Cargo.toml:21` (`default-features = false, features = ["wasm"]`). **Candidate for removal** (la dependencia funciona sin ella) — extracción trivial, fuera de alcance de este task |

## Cobertura en CI (`ci-rust-10.yml`)

| Job | Features |
| --- | --- |
| `fmt` / `clippy` | `cargo clippy --workspace --all-targets --all-features` → **todas** las features compilan como mínimo |
| `test` (Linux, audit) | `--features "cli,arrow,tls,opentelemetry"` **+ default set** |
| `test-windows` / `test-macos` | idem (Windows con `ci-windows`) |
| `semver-checks` | `cargo semver-checks -p vantadb` (baseline publicado; `--exclude` crates experimentales) |
| `miri` | `--no-default-features --features "cli,fjall,memmap2,fs2"` (sin `roaring`: C FFI) |
| `coverage` / `sanitizer-asan` / `sanitizer-tsan` | `--features "cli,arrow,tls,opentelemetry"` |
| `experimental-check` | `cargo check -p vantadb-server -p vantadb-mcp -p vantadb-wasm` + providers (no features de test) |

**Resultado:** 11 features están ejercidas en runtime en CI (`default` + `tls` + `opentelemetry`).
El resto (16) solo se comprueban vía `clippy --all-features` / `experimental-check` — son
`opt-in` o `experimental` por diseño, no regresiones de cobertura.

## Candidates for removal (no ejecutados en este task)

- `wasm` (💀 no-op): ver tabla. Borrar implica tocar `vantadb-wasm/Cargo.toml`.

## Cross-References

- `docs/architecture/adr/ADR-014-pitr.md` — decisión `pitr` (experimental standalone API, integración diferida)
- `docs/operations/EXPERIMENTAL_FEATURES.md` — boundary de producto v0.1.x (vista por superficie, no por feature Cargo)
- `docs/operations/CI_POLICY.md` — política de crates experimentales (server/mcp/wasm fuera de default-members)
- `docs/strategy/VANTADB-PRO-FEATURES.md` — mapa de candidatas Pro (`pitr`, `wal-shipping`, `encryption`, `server`, `tls`, `prometheus`, …)