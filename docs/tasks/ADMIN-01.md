# ADMIN-01: Exponer snapshot de métricas operativas como comando Tauri `vanta_metrics`

## Metadata
- **Plan file:** docs/plans/2026-08-08-admin-console.md
- **Creado:** 2026-08-08
- **last-synced:** 2026-08-08
- **Estado:** ✅ COMPLETED

## Blast Radius
- `desktop/src-tauri/src/commands/metrics.rs` — NUEVO comando `#[tauri::command] vanta_metrics` (thin wrapper sobre `VantaEmbedded::operational_metrics()`).
- `desktop/src-tauri/src/commands/mod.rs` — `pub mod metrics;` declarado.
- `desktop/src-tauri/src/lib.rs` — `commands::metrics::vanta_metrics` agregado al `invoke_handler`.
- `src/metrics/core/snapshot.rs` — NO tocado (sin derive agregado; se usó la vía SDK ya serializable).

## Contrato
"`cargo check --manifest-path desktop/src-tauri/Cargo.toml` pasa; el snapshot incluye `derived_prefix_scans`."

## Pasos
### Step 1: Investigar path serializable — ✅
- `OperationalMetricsSnapshot` (`src/metrics/core/snapshot.rs`) NO deriva `Serialize` (solo `Debug, Clone, Copy, Default, PartialEq, Eq`).
- El módulo `vantadb::metrics` es `pub(crate)` (re-export `pub use core::*` limitado por `pub(crate) mod core`) → inaccesible desde el crate desktop.
- Path público y serializable: `VantaEmbedded::operational_metrics() -> VantaOperationalMetrics` (exportado en `vantadb::VantaOperationalMetrics`, lib.rs:166). Ya deriva `Serialize, Deserialize` e incluye `derived_prefix_scans` (sdk/types.rs:360). CERO cambios al core.

### Step 2: Escribir `metrics.rs` — ✅
- `#[tauri::command] pub fn vanta_metrics(_app_state: State<AppState>) -> Result<VantaOperationalMetrics, VantaError>`.
- Usa `VantaEmbedded::test_empty(VantaConfig::default())` — handle vacío sin abrir engine: `operational_metrics()` lee atomics process-global y devuelve el snapshot igual con engine cerrado (engine_handle falla → skips memory record → retorna snapshot). Sin I/O por poll (ponytail: no abrir DB throwaway como vanta_health).

### Step 3: Registrar en mod.rs + lib.rs — ✅
- `mod.rs`: `pub mod metrics;`
- `lib.rs` invoke_handler: `commands::metrics::vanta_metrics,`

### Step 4: Verify — ✅
- `cargo check --manifest-path desktop/src-tauri/Cargo.toml` → Finished dev, 5.21s. Único warning: `cfg(mobile)` en lib.rs:45 — PRE-EXISTENTE, no del cambio.

### Step 5: Commit — ✅
- `git add` SOLO los 3 archivos del desktop (`metrics.rs`, `mod.rs`, `lib.rs`). `src/metrics/core/snapshot.rs` NO se tocó.
- Pre-commit hook: clippy falló por 5 warnings PRE-EXISTENTES (native.rs:265 ok_or_else closure, error.rs:187 io_other_error, lib.rs:45 mobile cfg, connection.rs future-let, ConnectionManager Default) — ninguno en metrics.rs. Se usó el escape documentado `SKIP_CLIPPY=1` (fmt ✅, actionlint ✅). Commit: `d77559f3`.

## Dependencias
- `vantadb` crate vía path (`desktop/src-tauri/Cargo.toml:37`). Sin features nuevas requeridas — el módulo metrics no es feature-gated.

## Notas
- **Desviación del plan:** el paso 1 del contrato asumía que `operational_metrics_snapshot()` era accesible y que habría que definir struct local serde. Resultó innecesario: `VantaOperationalMetrics` (SDK, ya `Serialize`) es el camino mínimo — 0 toques al core.
- `test_empty` es `#[doc(hidden)]` pero es el único constructor sin I/O y `operational_metrics()` lo maneja con gracia (no da NotInitialized). Documentado con comentario ponytail.
- Campos expuestos: los 40 de `VantaOperationalMetrics` (ver "Campos del snapshot" abajo).

## Context Save Point
- **Fecha:** 2026-08-08
- **Branch:** develop
- **Commit:** d77559f3b5c909744787bdce14ffcf0309e11401 — `feat(ADMIN-01): expose operational metrics snapshot as vanta_metrics Tauri command`
- **CI pendiente:** no (cargo check local pasado)
- **Decisiones:** Usar la vía SDK `VantaEmbedded::operational_metrics()` → `VantaOperationalMetrics` (ya serializable) en lugar de tocar `src/metrics/core/snapshot.rs` con derive — menos superficie de cambio y no requiere `cargo check -p vantadb` en el root. Handle vacío (`test_empty`) para no abrir DB throwaway por poll.
- **Problemas conocidos:** Pre-commit clippy bloqueado por deuda pre-existente en desktop (native.rs/error.rs/lib.rs/connection.rs/manager.rs) — no relacionada con ADMIN-01; se saltó con `SKIP_CLIPPY=1` documentado en el hook. Working tree tenía cambios pre-existentes de otras líneas (src/, vantadb-mcp/, docs/, completions/) — NO se tocaron ni commitearon (commit tiene exactamente 3 archivos, 35 insertions).
- **Próxima tarea:** — (task única ejecutada)
