---
title: Cargo Feature Registry — per-feature status
kind: concept
status: active
description: "Inventario honesto de las features declaradas en Cargo.toml [features]"
tags: [vantadb, architecture, features, cargo]
---

# Cargo Feature Registry

Inventario honesto de las features declaradas en `Cargo.toml` `[features]` (del crate
`vantadb`), qué gatea cada una y dónde se ejerce. Combinaciones prometidas y cobertura de CI: `docs/feature-matrix.md`. Fuentes:

- `Cargo.toml`, `[features]` en `:155-199` (`default` en `:156`)
- `.github/workflows/ci-rust.yml` (qué features se compilan y se corren en CI)
- `rg 'feature = "…"' src/ tests/` (gates reales de código)

## Status legend

| Status | Significado |
| --- | --- |
| ✅ `default` | En la feature set por defecto (`Cargo.toml:156`); compilada en todo build |
| ✅ `ci` | Alguna lane de `.github/workflows/ci-rust.yml` compila o corre una combinación que **enciende** el flag, más allá de `clippy --all-features` (ese job compila la unión y por eso no cuenta como ejercicio). Los workflows de release (que construyen lo publicado) no cuentan como lane de `ci` |
| 🟡 `opt-in` | Código real detrás del flag; **no está en `Cargo.toml:156`**. Los dos ejes son independientes: `opt-in` dice "no es default", `ci` dice "alguien lo ejerce" |
| ⚠️ `experimental` | Código real pero fuera de la superficie de producto estable (ver ADR-0014 / `docs/dev/operations/CI_POLICY.md`) |
| 💀 `no-op` | Sin gates de código; marcador vacío |

## Feature table

| Feature | Status | Qué gatea | Dónde se usa |
| --- | --- | --- | --- |
| `default` | ✅ default | Agregador de **cinco** flags: `cli`, `fjall`, `memmap2`, `fs2`, `roaring` | `Cargo.toml:156`. `arrow`, `advanced-tokenizer`, `sysinfo` y `rayon` **no** están en default; combinaciones en [feature-matrix.md](../../feature-matrix.md) §1-2 |
| `cli` | ✅ default · ✅ ci | `bin/vanta-cli`, `cli`, `cli_handlers`, `console`; 8 tests con `required-features = ["cli"]` (`cli_tests`, `mmap_hnsw`, `file_locking_stress`, …) | `src/lib.rs:68-70,80`, `src/bin/vanta-cli.rs`; CI: `--features "cli,arrow,tls,opentelemetry"` (Linux/Windows/macOS), Miri |
| `arrow` | 🟡 opt-in · ✅ ci | `columnar` (storage columnar tipado, JSON shredding) | `src/lib.rs:74`; test `columnar` (`required-features = ["arrow"]`); CI: en la string de `--features` de las tres lanes de tests |
| `fjall` | ✅ default · ✅ ci | Backend de storage Fuji (LSM) | `src/backends/…` (19 gates); CI: default + Miri (`cli,fjall,memmap2,fs2`) |
| `roaring` | ✅ default · ✅ ci | `FilterBitset` vía `croaring` (C FFI) | `src/node.rs` (9 gates); CI default. ⚠️ Miri corre **sin** `roaring` a propósito (C FFI no ejecutable bajo Miri; fallback `Vec<u64>`) — step `Miri Test (core - unsafe patterns)` y su comentario |
| `advanced-tokenizer` | 🟡 opt-in · ✅ ci | `tokenizer` (BM25 avanzado vía `tantivy`) | `src/lib.rs:127`, `src/config.rs`, `src/tokenizer.rs` (48 gates); CI: las tres lanes de tests la encienden **por unificación** (`vantadb-server/Cargo.toml:21-30` y `vantadb-mcp/Cargo.toml:14-23` la piden) y `Feature Combo (server-full)` la compila explícita |
| `memmap2` | ✅ default · ✅ ci | Index mmap (mmap_index / mmap_hnsw) | `src/storage/…` (9 gates); CI: default + Miri |
| `fs2` | ✅ default · ✅ ci | File locking multi-proceso | `src/storage/…` (5 gates); test `multi_process_lock`; CI: default + Miri |
| `sysinfo` | 🟡 opt-in · ✅ ci | Telemetría de hardware (RAM/CPU del host) | `src/hardware/mod.rs` (28 gates), `src/config.rs`; CI: por unificación en las tres lanes de tests — la piden las mismas listas de `vantadb-server`/`vantadb-mcp` que `advanced-tokenizer` |
| `rayon` | 🟡 opt-in · ✅ ci | Iteración paralela (bulk put/import) | `src/…` (5 gates); CI: por unificación en las tres lanes de tests — la piden `vantadb-python/Cargo.toml:25` y las listas de `vantadb-server`/`vantadb-mcp` (ver [feature-matrix.md](../../feature-matrix.md) §4) |
| `tls` | 🟡 opt-in · ✅ ci | TLS para el server HTTP (`axum-server` + `rustls`); sin efecto si `server` no está activo | `src/cli_server.rs` (5 gates); CI: `--features "cli,arrow,tls,opentelemetry"` en las 3 OS + sanitizers |
| `opentelemetry` | 🟡 opt-in · ✅ ci | Tracing OTLP export + métricas OTel del server (`opentelemetry_sdk/rt-tokio`) | `src/cli_server.rs` (9 gates); CI: audit features en las 3 OS + coverage |
| `rocksdb` | 🟡 opt-in | Backend de storage RocksDB (opcional; no es miembro de default) | `src/backends/…` (16 gates). No está en features de test de CI (solo `clippy --all-features`); macOS instala `rocksdb` para smoke `cargo check` sin features extras |
| `remote-inference` | 🟡 opt-in · ✅ ci | `llm` + llamadas a proveedores externos desde el executor (vía `reqwest`) | `src/lib.rs:98`, `src/executor.rs` (13 gates), `src/physical_plan.rs`; CI: job `Tests (embeddings features)`, leg `remote-inference`, `--lib`; también dev-dep de `vanta-memory/Cargo.toml:59` |
| `failpoints` | 🟡 opt-in · ✅ ci | API de failpoints de chaos + `testing`; test `chaos_integrity` (`required-features = ["failpoints"]`) | `src/lib.rs:177-194`, `src/edge_index.rs`, `src/index/serialize.rs` (20 gates). CI: `chaos_integrity` corre en la certificación pesada (job `Chaos Tests (failpoints)` en `certification.yml`, y `nightly.yml`), **no** en el Fast Gate |
| `encryption` | 🟡 opt-in · ✅ ci | Cifrado at-rest AES-256-GCM **cableado a los 4 write paths** (WAL v4, VantaFile v3, backend-KV por valor, imagen HNSW v9); el modo se decide **por los bytes del disco** y una base nueva con clave se cifra desde el primer byte (una base plaintext existente **no** se migra: la clave se ignora con aviso). El indice de texto y los snapshots siguen por transito. Limites declarados en `SECURITY.md`; CI: step `Miri Test (core - unsafe patterns)` la corre |
| `server` | 🟡 opt-in · ✅ ci | Server HTTP embebido: `circuit_breaker`, `cli_server`, `connection_pool` (requiere `cli`) | `src/lib.rs:66,72,78` (6 gates). No está en la string de `--features` de ninguna lane, pero las tres de tests la encienden **por unificación** (`vantadb-server/Cargo.toml:21-30`, `vantadb-mcp/Cargo.toml:14-23`). Promovida de experimental a estable 2026-08-25: base del server embebido + REST `/api/v2/*` (ADR-0026) |
| `tui` | 🟡 opt-in | TUI interactiva del CLI (`ratatui`+`crossterm`) | `src/lib.rs:129`, `src/tui.rs`, `src/bin/vanta-cli.rs` (3 gates) |
| `prometheus` | 🟡 opt-in | Métricas Prometheus (endpoints + counters de governor/cache warmer) | 62 gates — la superficie opt-in más grande (`src/cache_warmer.rs`, `src/memory_governor.rs`); Candidata Pro |
| `python_sdk` | 🟡 opt-in | `python` — bindings directos `pyo3` desde el crate core | `src/lib.rs:112` (2 gates). Nota: `vantadb-python` **no** la activa y tampoco apaga los defaults — `vantadb-python/Cargo.toml:25` consume el core con `features = ["fjall","memmap2","rayon"]` y sin `default-features = false`, así que el wheel se construye con **`default` encendido + `rayon`** |
| `jemalloc` | 🟡 opt-in · ✅ ci | Allocator jemalloc para el binario `vanta-cli` (gated a unix en Cargo.toml) | `src/bin/vanta-cli.rs` (4 gates); CI: job `release-combo` hace `cargo check --release --features "server,jemalloc"` sobre el binario |
| `custom-allocator` | 🟡 opt-in | Allocator custom (mimalloc) para el binario `vanta-cli` cuando jemalloc no aplica | `src/bin/vanta-cli.rs` (1 gate) |
| `async-ingestion` | 🟡 opt-in | `ingestion` — pipeline async de inserción a worker pool | `src/lib.rs:146`, `src/ingestion.rs` (1 gate en lib) |
| `async-io` | 🟡 opt-in | `transcript` — I/O async de transcriptos | `src/lib.rs:149`, `src/transcript.rs` (9 gates) |
| `hot-reload` | 🟡 opt-in | Watcher de recarga de config (JSON/TTL) vía `notify` | `src/config.rs` (7 gates) |
| `wal-shipping` | 🟡 opt-in | Envío async de WAL a réplica remota (`reqwest`) | `src/lib.rs:138` (`wal_shipping`). Feature real, documentada en ADR-0014 como separada de PITR; Candidata Pro |
| `pitr` | 💀 removed | ~~`wal_archiver` — archivado WAL + point-in-time recovery~~ | **Removido 2026-08-25 (FIND-26):** dead code sin wiring desde el engine (RES-02); el código vive en git history. Ver ADR-0014 (superseded) y backlog CORE-02 si se reviviera PITR |
| `bayesian_decay` | 🟡 opt-in | Variantes de eviction con decay Bayesian Beta-Binomial | `src/eviction.rs` (17 gates) |
| `wasm` | 💀 no-op | Sin `#[cfg]` en código del crate core | 0 matches en `src/`/`tests/`; únicamente referenciada por la dependencia de `vantadb-wasm/Cargo.toml:21` (`default-features = false, features = ["wasm"]`). **Candidate for removal** (la dependencia funciona sin ella) — extracción trivial, fuera de alcance de este task |
| `embed-local` | 🟡 opt-in · ✅ ci | Embedder local in-process (`ort` + `tokenizers`): inferencia sin proveedor externo | `Cargo.toml:166`, `src/lib.rs:120-121` (`cfg(any(remote-inference, embed-local))`); CI: job `Tests (embeddings features)`, leg `embed-local`, `--lib`. *Fila agregada en `FIND-808` (ronda 2): era la única feature declarada en `Cargo.toml` sin fila* |

## Cobertura en CI

El mapeo job → combinación → minutos medidos vive en
[`docs/feature-matrix.md`](../../feature-matrix.md) §2 ("Gate that covers it today"). No se
repite acá: la tabla anterior de este fichero era la copia que envejeció.

Lo que la columna `Status` afirma se sostiene en tres lanes de `ci-rust.yml`, citadas por
**nombre de step** —los números de línea de ese workflow rotan con cada edición, y fueron
justamente el mecanismo que puso aquí una cita a un workflow inexistente—:

- **Jobs `Tests (Linux)`, `Tests (Windows)`, `Tests (macOS)`** — `cargo nextest run
  --features "cli,arrow,tls,opentelemetry"` sobre cinco `-p`: `vantadb`, `vantadb_py`,
  `vanta-memory`, `vantadb-server`, `vantadb-mcp`. Linux y macOS corren el perfil `audit`;
  Windows corre `ci-windows` (`--profile ci-windows`, step `Run quick integrity tests
  (Windows)` — no es el mismo step ni el mismo perfil que las otras dos).
- **`Clippy Lints`** — `--all-features`: compila la **unión**, una sola combinación. No
  prueba un `#[cfg(not(feature = …))]` que se haya podrido (las features son aditivas), así
  que por sí solo no cuenta como que un flag esté ejercitado.
- **`Miri Test (core - unsafe patterns)`** — `--no-default-features --features
  "cli,fjall,memmap2,fs2,encryption"`: la única lane que compila **sin** `roaring` (C FFI, no
  ejecutable bajo Miri).

⚠️ **La string de `--features` es un piso, no la lista de la unión.** Las features son
aditivas y una invocación con varios `-p` compila `vantadb` **una vez** con lo que pide cada
paquete seleccionado. Las tres lanes de tests encienden además `server`,
`advanced-tokenizer`, `sysinfo` y `rayon` (`vantadb-server/Cargo.toml:21-30` y
`vantadb-mcp/Cargo.toml:14-23` piden la misma lista de ocho) y `remote-inference`
(`vanta-memory/Cargo.toml:59`, dev-dep). Por eso este registro **no** publica la lista
cerrada de esa unión: la resuelve Cargo, y toda copia derivada a mano se desincroniza igual
que la tabla que se borró de aquí. `feature-matrix.md` §4 lo documenta como advertencia
medida.

Los flags que ninguna de esas lanes enciende solo se compilan vía `clippy --all-features` o
un combo dedicado (`Feature Combo (server-full)`); `failpoints` corre además en la
certificación pesada (`certification.yml`, `nightly.yml`). Son `opt-in` por diseño, no
regresiones de cobertura.

## Candidates for removal (no ejecutados en este task)

- `wasm` (💀 no-op): ver tabla. Borrar implica tocar `vantadb-wasm/Cargo.toml`.

## Cross-References

- `docs/dev/architecture/adr/ADR-0014-pitr.md` — decisión `pitr` (experimental standalone API, integración diferida)
- [`docs/feature-matrix.md`](../../feature-matrix.md) — combinaciones prometidas, gate que cubre cada una y coste de runner; no repite el registro por feature de esta página
- `docs/user/operations/EXPERIMENTAL_FEATURES.md` — boundary de producto v0.1.x (vista por superficie, no por feature Cargo)
- `docs/dev/operations/CI_POLICY.md` — política de crates experimentales (server/mcp/wasm fuera de default-members)
- `docs/dev/strategy/VANTADB-PRO-FEATURES.md` — mapa de candidatas Pro (`pitr`, `wal-shipping`, `encryption`, `server`, `tls`, `prometheus`, …)
