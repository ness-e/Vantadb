---
title: Cargo Feature Registry — per-feature status
kind: concept
status: active
description: "Inventario honesto de las features declaradas en Cargo.toml [features]"
tags: [vantadb, architecture, features, cargo]
---

# Cargo Feature Registry

Inventario honesto de las features declaradas en `Cargo.toml` `[features]`
(del crate `vantadb`), qué gatea cada una y dónde se ejerce. Fuentes:

- `Cargo.toml`, `[features]` en `:155-199` (`default` en `:156`)
- `.github/workflows/ci-rust.yml` (qué features se compilan y se corren en CI)
- `rg 'feature = "…"' src/ tests/` (gates reales de código)

**Qué combinaciones se prometen y qué gate cubre cada una:
[`docs/feature-matrix.md`](../../feature-matrix.md).** Este fichero es el registro **por
feature** — qué gatea cada flag, con `file:line`; la matriz es la que responde por
**combinaciones**, cobertura y coste de runner. La tabla agregada de CI que vivía acá se
reemplazó por un puntero: era la copia que se desincronizó.

## Status legend

| Status | Significado |
| --- | --- |
| ✅ `default` | En la feature set por defecto; compilada en todo build |
| ✅ `ci` | Ejercida en runs de test de CI (no necesariamente default) |
| 🟡 `opt-in` | Código real detrás del flag; no default y ninguna lane de test lo corre (solo se compila vía `clippy --all-features` o un combo dedicado, ej. `Feature Combo (server-full)`) |
| ⚠️ `experimental` | Código real pero fuera de la superficie de producto estable (ver ADR-0014 / `docs/dev/operations/CI_POLICY.md`) |
| 💀 `no-op` | Sin gates de código; marcador vacío |

## Feature table

| Feature | Status | Qué gatea | Dónde se usa |
| --- | --- | --- | --- |
| `default` | ✅ default | Agregador de **cinco** flags: `cli`, `fjall`, `memmap2`, `fs2`, `roaring` | `Cargo.toml:156`. `arrow`, `advanced-tokenizer`, `sysinfo` y `rayon` **no** están en default; combinaciones en [feature-matrix.md](../../feature-matrix.md) §1-2 |
| `cli` | ✅ default · ✅ ci | `bin/vanta-cli`, `cli`, `cli_handlers`, `console`; 8 tests con `required-features = ["cli"]` (`cli_tests`, `mmap_hnsw`, `file_locking_stress`, …) | `src/lib.rs:68-70,80`, `src/bin/vanta-cli.rs`; CI: `--features "cli,arrow,tls,opentelemetry"` (Linux/Windows/macOS), Miri |
| `arrow` | 🟡 opt-in · ✅ ci | `columnar` (storage columnar tipado, JSON shredding) | `src/lib.rs:74`; test `columnar` (`required-features = ["arrow"]`); CI: lane de tests (`ci-rust.yml:469`) |
| `fjall` | ✅ default · ✅ ci | Backend de storage Fuji (LSM) | `src/backends/…` (19 gates); CI: default + Miri (`cli,fjall,memmap2,fs2`) |
| `roaring` | ✅ default · ✅ ci | `FilterBitset` vía `croaring` (C FFI) | `src/node.rs` (9 gates); CI default. ⚠️ Miri corre **sin** `roaring` a propósito (C FFI no ejecutable bajo Miri; fallback `Vec<u64>`) — `ci-rust.yml:1119-1146` |
| `advanced-tokenizer` | 🟡 opt-in | `tokenizer` (BM25 avanzado vía `tantivy`) | `src/lib.rs:127`, `src/config.rs`, `src/tokenizer.rs` (48 gates); CI: solo compilada en `Feature Combo (server-full)` (`ci-rust.yml:1008`), ninguna lane de test la corre |
| `memmap2` | ✅ default · ✅ ci | Index mmap (mmap_index / mmap_hnsw) | `src/storage/…` (9 gates); CI: default + Miri |
| `fs2` | ✅ default · ✅ ci | File locking multi-proceso | `src/storage/…` (5 gates); test `multi_process_lock`; CI: default + Miri |
| `sysinfo` | 🟡 opt-in | Telemetría de hardware (RAM/CPU del host) | `src/hardware/mod.rs` (28 gates), `src/config.rs`; ninguna feature string de CI la enciende (solo `clippy --all-features`, `ci-rust.yml:280`) |
| `rayon` | 🟡 opt-in · ✅ ci | Iteración paralela (bulk put/import) | `src/…` (5 gates); CI: entra por unión en las tres lanes de tests porque `vantadb-python/Cargo.toml:25` la enciende (ver [feature-matrix.md](../../feature-matrix.md) §4) |
| `tls` | 🟡 opt-in · ✅ ci | TLS para el server HTTP (`axum-server` + `rustls`); sin efecto si `server` no está activo | `src/cli_server.rs` (5 gates); CI: `--features "cli,arrow,tls,opentelemetry"` en las 3 OS + sanitizers |
| `opentelemetry` | 🟡 opt-in · ✅ ci | Tracing OTLP export + métricas OTel del server (`opentelemetry_sdk/rt-tokio`) | `src/cli_server.rs` (9 gates); CI: audit features en las 3 OS + coverage |
| `rocksdb` | 🟡 opt-in | Backend de storage RocksDB (opcional; no es miembro de default) | `src/backends/…` (16 gates). No está en features de test de CI (solo `clippy --all-features`); macOS instala `rocksdb` para smoke `cargo check` sin features extras |
| `remote-inference` | 🟡 opt-in · ✅ ci | `llm` + llamadas a proveedores externos desde el executor (vía `reqwest`) | `src/lib.rs:98`, `src/executor.rs` (13 gates), `src/physical_plan.rs`; CI: `Tests (embeddings features)` leg `remote-inference`, `--lib` (`ci-rust.yml:676-687`) |
| `failpoints` | 🟡 opt-in · ✅ ci | API de failpoints de chaos + `testing`; test `chaos_integrity` (`required-features = ["failpoints"]`) | `src/lib.rs:177-194`, `src/edge_index.rs`, `src/index/serialize.rs` (20 gates). CI: `chaos_integrity` corre en la certificación pesada (`certification.yml:197-198`, `nightly.yml:133-134`), **no** en el Fast Gate — el audit de PR no incluye `failpoints` |
| `encryption` | 🟡 opt-in | Cifrado at-rest AES-256-GCM **cableado a los 4 write paths** (WAL v4, VantaFile v3, backend-KV por valor, imagen HNSW v9); el modo se decide **por los bytes del disco** y una base nueva con clave se cifra desde el primer byte (una base plaintext existente **no** se migra: la clave se ignora con aviso). El indice de texto y los snapshots siguen por transito. Limites declarados en `SECURITY.md` |
| `server` | 🟡 opt-in | Server HTTP embebido: `circuit_breaker`, `cli_server`, `connection_pool` (requiere `cli`) | `src/lib.rs:66,72,78` (6 gates). No en features de test de CI; `vantadb-server` la activa (`vantadb-server/Cargo.toml:10`). Promovida de experimental a estable 2026-08-25: base del server embebido + REST `/api/v2/*` (ADR-0026) |
| `tui` | 🟡 opt-in | TUI interactiva del CLI (`ratatui`+`crossterm`) | `src/lib.rs:129`, `src/tui.rs`, `src/bin/vanta-cli.rs` (3 gates) |
| `prometheus` | 🟡 opt-in | Métricas Prometheus (endpoints + counters de governor/cache warmer) | 62 gates — la superficie opt-in más grande (`src/cache_warmer.rs`, `src/memory_governor.rs`); Candidata Pro |
| `python_sdk` | 🟡 opt-in | `python` — bindings directos `pyo3` desde el crate core | `src/lib.rs:112` (2 gates). Nota: `vantadb-python` **no** la activa y tampoco apaga los defaults — `vantadb-python/Cargo.toml:25` consume el core con `features = ["fjall","memmap2","rayon"]` y sin `default-features = false`, así que el wheel se construye con **`default` encendido + `rayon`** |
| `jemalloc` | 🟡 opt-in | Allocator jemalloc para el binario `vanta-cli` (gated a unix en Cargo.toml) | `src/bin/vanta-cli.rs` (4 gates) |
| `custom-allocator` | 🟡 opt-in | Allocator custom (mimalloc) para el binario `vanta-cli` cuando jemalloc no aplica | `src/bin/vanta-cli.rs` (1 gate) |
| `async-ingestion` | 🟡 opt-in | `ingestion` — pipeline async de inserción a worker pool | `src/lib.rs:146`, `src/ingestion.rs` (1 gate en lib) |
| `async-io` | 🟡 opt-in | `transcript` — I/O async de transcriptos | `src/lib.rs:149`, `src/transcript.rs` (9 gates) |
| `hot-reload` | 🟡 opt-in | Watcher de recarga de config (JSON/TTL) vía `notify` | `src/config.rs` (7 gates) |
| `wal-shipping` | 🟡 opt-in | Envío async de WAL a réplica remota (`reqwest`) | `src/lib.rs:138` (`wal_shipping`). Feature real, documentada en ADR-0014 como separada de PITR; Candidata Pro |
| `pitr` | 💀 removed | ~~`wal_archiver` — archivado WAL + point-in-time recovery~~ | **Removido 2026-08-25 (FIND-26):** dead code sin wiring desde el engine (RES-02); el código vive en git history. Ver ADR-0014 (superseded) y backlog CORE-02 si se reviviera PITR |
| `bayesian_decay` | 🟡 opt-in | Variantes de eviction con decay Bayesian Beta-Binomial | `src/eviction.rs` (17 gates) |
| `wasm` | 💀 no-op | Sin `#[cfg]` en código del crate core | 0 matches en `src/`/`tests/`; únicamente referenciada por la dependencia de `vantadb-wasm/Cargo.toml:21` (`default-features = false, features = ["wasm"]`). **Candidate for removal** (la dependencia funciona sin ella) — extracción trivial, fuera de alcance de este task |

## Cobertura en CI

El mapeo job → combinación → minutos medidos vive en
[`docs/feature-matrix.md`](../../feature-matrix.md) §2 ("Gate that covers it today"). No se
repite acá: la tabla anterior de este fichero era la copia que envejeció.

Lo que la columna `Status` de arriba afirma se sostiene en tres lanes de `ci-rust.yml`,
leídas contra el árbol:

- **`Tests (Linux/Windows/macOS)`** — `cargo nextest run --features
  "cli,arrow,tls,opentelemetry"` (`:469`, `:512`, `:562`). Los defaults van encendidos, así
  que la unión ejercida en runtime es `cli, fjall, memmap2, fs2, roaring, arrow, tls,
  opentelemetry`.
- **`Clippy Lints`** — `--all-features` (`:280`): compila la **unión**, una sola combinación.
  No prueba un `#[cfg(not(feature = …))]` que se haya podrido (las features son aditivas).
- **`Miri (UB Detection)`** — `--no-default-features --features
  "cli,fjall,memmap2,fs2,encryption"` (`:1128-1129`): la única lane que compila **sin**
  `roaring`.

Los flags que ninguna de esas lanes enciende solo se compilan vía `clippy --all-features`
(`:280`) o un combo dedicado (`Feature Combo (server-full)`, `:1008`); `failpoints` corre
además en la certificación pesada (`certification.yml`, `nightly.yml`). Son `opt-in` por
diseño, no regresiones de cobertura.

## Candidates for removal (no ejecutados en este task)

- `wasm` (💀 no-op): ver tabla. Borrar implica tocar `vantadb-wasm/Cargo.toml`.

## Cross-References

- `docs/dev/architecture/adr/ADR-0014-pitr.md` — decisión `pitr` (experimental standalone API, integración diferida)
- `docs/user/operations/EXPERIMENTAL_FEATURES.md` — boundary de producto v0.1.x (vista por superficie, no por feature Cargo)
- `docs/dev/operations/CI_POLICY.md` — política de crates experimentales (server/mcp/wasm fuera de default-members)
- `docs/dev/strategy/VANTADB-PRO-FEATURES.md` — mapa de candidatas Pro (`pitr`, `wal-shipping`, `encryption`, `server`, `tls`, `prometheus`, …)
