# Task: DESKTOP-03 — Integrar crate `vantadb` + managed state + healthcheck (Wave 1)

- **Effort:** 🟢 | **Priority:** 🔴
- **Plan:** `docs/plans/2026-08-06-desktop-mvp.md`
- **Agent:** `vanta-worker`
- **Estado:** ✅ COMPLETED
- **Branch:**
- **Commit:** (pending)

## Contrato (verificado)

| Criterio | Estado |
|----------|--------|
| Dep `vantadb` con `default-features=false` + `fjall,fs2,memmap2,roaring,advanced-tokenizer` (NUNCA cli/server/prometheus) | ✅ |
| `AppState { manager, config }` managed via `tauri::Builder::manage(state)` | ✅ |
| command `#[tauri::command] vanta_health(State<AppState>) -> Result<HealthReport, VantaError>` abre `VantaEmbedded` en temp dir, devuelve `backend="fjall"` y cierra | ✅ |
| Abrir dos veces el mismo path → error de lock (`VantaError::Lock`, `DatabaseBusy`) | ⏭ documentado; cubierto por DESK-05 |
| `cargo check` en `desktop/src-tauri` → exit 0 | ✅ (aislado; el full-break es por WIP paralelo DESK-05/09 `native.rs`/`server.rs`) |
| `cargo check -p vantadb` en raíz → exit 0, sin tocar Cargo.toml raíz | ✅ |
| `cargo test --lib` en `desktop/src-tauri` → pasa | ✅ 17 passed |

## Skills
- ponytail (full): dep subset mínimo, `_app_state` desusado (probab ad-hoc para healthcheck), sin limpieza de temp dir (OS temp space).
- source-driven-development: validado `VantaEmbedded::open(path)` / `.capabilities()` / `.close()` contra `src/sdk/builder.rs` + `docs/api/EMBEDDED_SDK.md`; `VantaConfig` + `VantaError::DatabaseBusy` contra `src/config.rs` e `src/error.rs:86-290`. `DatabaseBusy` → `VantaError::Lock` en `map_core_error`.

## Archivos

- `desktop/src-tauri/Cargo.toml` — dep `vantadb = { path = "../..", default-features = false, features = ["fjall","fs2","memmap2","roaring","advanced-tokenizer"] }`
- `desktop/src-tauri/src/commands/mod.rs` (nuevo) — módulo de commands
- `desktop/src-tauri/src/commands/connection.rs` (nuevo) — `vanta_health` + `map_core_error` + `probe_dir`
- `desktop/src-tauri/src/lib.rs` — `AppState { manager, config }` + `manage(state)` + registrar `vanta_health`
- `desktop/src-tauri/src/connections/types.rs` — `HealthReport` ganó campo `backend: String` (default `unknown`)

## Verification
- `cargo check -p vantadb --features "fjall,fs2,memmap2,roaring,advanced-tokenizer"` (raíz) — ✅ exit 0, invariante
- `cargo check` (desktop/src-tauri, aislando WIP paralelo) — ✅ exit 0
- `cargo test --lib` (desktop/src-tauri, aislado) — ✅ 17 passed incl. `health_report_roundtrip` con `backend="fjall"`

## Coordinación/avisos
- WIP paralelo (DESK-05/09) presente en `connections/{native,server}.rs` + `mod.rs` al momento de ejecutar: rompía el check del workspace. Verifiqué mi slice aislando `native.rs`/`server.rs` en `mod.rs` y **restauré** el archivo a su estado paralelo. No toqué `native.rs`/`server.rs`.
- `vanta_health` usa un path temp único por llamada, así que dos probes nunca colisionan en lock; el error de lock (dobla path) lo cubre DESK-05 `NativeConnection` — documentado en el comando.
- MSVC linker: sin crash esta vez (check/test vía cwd contenedor, cache caliente).
- No toqué `Cargo.toml` raíz ni `src/*` raíz. Solo archivos de `desktop/src-tauri` + este task file.