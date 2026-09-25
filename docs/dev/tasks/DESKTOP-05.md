# DESKTOP-05 - Adapter `NativeConnection` sobre el core `VantaEmbedded` con lock de path

- **Estado:** ✅ COMPLETED (2026-08-06)
- **Esfuerzo:** 🟡
- **Commit:** `5cebcc29` (native.rs) — `mod.rs` wiring re-emerges in the parallel DES-09 merge
- **Archivos clave:** `desktop/src-tauri/src/connections/native.rs`, `desktop/src-tauri/src/connections/mod.rs`, `desktop/src-tauri/Cargo.toml`
- **Agente:** `vanta-worker`

## Context

El workspace desktop (`desktop/src-tauri`, workspace propio aislado del ws raíz) define el
contrato multi-connection de DESKTOP-04. DESK-03 ya agregó el core `vantadb` como dep
(`default-features = false`, features `fjall/fs2/memmap2/roaring/advanced-tokenizer` — nunca
`cli`/`server`/`prometheus`). Este task implementa el primer adapter real: **NativeConnection**
que abre `VantaEmbedded` en un path y expone `trait VantaConnection`.

## Contrato cumplido

- `native.rs`: `NativeConnection::open(path)` crea `VantaEmbedded::open` y mantiene la instancia.
  - **Lock de path:** `StorageEngine::init` adquiere el fs2 lock exclusivo al abrir; un segundo
    open sobre el mismo path devuelve `VantaError::DatabaseBusy` del core, que se mapea a
    `VantaError::Lock` del crate desktop (test `second_open_same_path_locks` lo verifica, y la
    reapertura tras `disconnect()` vuelve a funcionar).
  - **Concurrencia**: todo op va por `tokio::task::spawn_blocking` (el core `VantaEmbedded` es
    síncrono) → el trait async se mantiene responsive.
  - `health()` → `HealthReport { status: Healthy, backend: "fjall", latency_ms, checked_at_ms }`.
  - `info()`/`capabilities()` → `Capability::Native`.
- `mod.rs`: `pub mod native;` + re-export `NativeConnection`.
- Tests (4, en `native.rs`): `crud_roundtrip_via_trait_object` (put/search/ge/delete por el
  trait), `second_open_same_path_locks`, `health_reports_fjall_backend`, `capabilities_and_info`.

## Verificación

- `cargo check` (desktop/src-tauri) → `exit 0`.
- `cargo test --lib` → ✅ `21 passed` (17 previos + 4 new de `native`).

## Decisiones / notas

- `HealReport` ahora lleva campo `backend` (contrato del shared `types.rs`); native reporta
  `"fjall"`, server reporta `"http"` (server.rs es WIP paralelo de DESK-09).
- El feature `fs2` es **requerido**: sin él el lock del core es un no-op y el open no falla.
- Reconciliación con WIP paralelo: `mod.rs` estaba TEMP-ISOLATE por DESK-09 (comentario);
  se restauró `pub mod native` porque ambos adapters compilan juntos.
- Fuera de alcance: WAL/vector/storage del core, `información` extra del engine, plan file.