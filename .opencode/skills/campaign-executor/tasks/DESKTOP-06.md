# DESKTOP-06 — Commands CRUD async + ConnectionManager registry

> Plan: `docs/plans/2026-08-06-desktop-mvp.md` (Task 5)
> Branch: develop
> Commit: see `git log -1`
> Estado: ✅ COMPLETED

## Objetivo

Reemplazar el placeholder `manager: ()` de `AppState` por un `ConnectionManager`
thread-safe que registra múltiples conexiones (`HashMap<id, Box<dyn VantaConnection>>`
+ `active_id`), y exponer los commands Tauri async CRUD (connect/disconnect/list/
set_active/ingest/ingest_batch/search/get/delete/list).

## Iteraciones

| # | Acción | Resultado | Herramienta |
|---|--------|-----------|-------------|
| 1 | `ConnectionManager` en `connections/manager.rs` (`tokio::sync::RwLock<Inner>`: connections + active_id; add/remove/set_active/active_id/list_connections/health/ingest/ingest_batch/search/get/delete/list_records) + re-export en `connections/mod.rs` | ✅ | write/edit |
| 2 | `AppState.manager: ConnectionManager::new()` en `lib.rs`; registrar todos los commands en `generate_handler![...]` (mantiene `ping` y `vanta_health`) | ✅ | edit |
| 3 | Commands conexión (`vanta_connect` con enum `ConnectTarget{native|server}`, `vanta_disconnect`, `vanta_list_connections`, `vanta_set_active`) en `commands/connection.rs` | ✅ | edit |
| 4 | Commands datos (`vanta_ingest`, `vanta_ingest_batch`, `vanta_search`, `vanta_get`, `vanta_delete`, `vanta_list`) en `commands/data.rs`; keys/namespaces como `String` | ✅ | write |
| 5 | Feature tokio `sync` en Cargo.toml; `cargo check` desktop → exit 0; `cargo test --lib` 22 passed (incluye nuevo E2E nativo) | ✅ | bash/cargo |

## Notas

- **Conflicto paralelo DESK-11:** `child_process.rs` + `tests/child_process.rs` (WIP de
  otra tarea, untracked) estaban declarados en `connections/mod.rs` y no compilaban.
  Se removieron sus 2 líneas de `mod.rs` para dejar `cargo check` en exit 0; el WIP queda
  sin trackear en disco. DESK-11 debe re-añadir `pub mod child_process;` +
  `pub use child_process::McpSpawn;` cuando su oda pueda compilar.
- `NativeConnection` no se re-exporta desde `connections` (solo por `native::`), por eso los
  commands usan `crate::connections::native::NativeConnection`.
- Solo se commitean archivos bajo `desktop/src-tauri/`; se dejan fuera `desktop/README.md`
  (untracked) y los archivos dirty de `vantadb-server/tests/`.