# FIND-20 — Persistencia estado ventana desktop

> **Plan:** `docs/dev/plans/2026-09-10-fixes.md` (Task 3) · **Campaign:** 1b2c3d4e-5f6a-7b8c-9d0e-1f2a3b4c5d01
> **Estado:** ⏳ IN PROGRESS · **Branch:** develop · **Appetite:** max 1d
> **Contrato:** `npm run build` desktop exit 0 + smoke arranque conserva posición/tamaño/maximizado + `npx tsc --noEmit` exit 0
> **SDP:** frontend-ui-engineering, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (+ campaign-executor/progreso base)

## Verificación real (DISCOVERY 2026-09-10, no re-derivar)

- `window-state|window_state|WindowState` = 0 matches en `desktop/src-tauri/src/` y `desktop/src/` → gap real ✅
- `desktop/src-tauri/Cargo.toml`: tauri 2 (lock: 2.11.5), features `[]`, sin plugins de ventana; `tauri.conf.json` sin label → default `"main"` (tauri-utils config.rs:2363)
- `Settings.tsx` existe (perfiles/idioma/embeddings) — no toca geometría de ventana, fuera de scope

## Decisión: persistencia manual, sin plugin (pre-mortem fallo 1)

`tauri-plugin-window-state` = nueva dependencia (red + riesgo compat v2) por ~100 líneas que el core ya cubre.
APIs verificadas en vendored `tauri-2.11.5/src/` (source-driven, offline):
`WindowEvent::{Moved,Resized,CloseRequested}` (app.rs:113-118) · `Window::on_window_event/inner_position/inner_size/is_maximized/set_size/set_position/maximize` (window/mod.rs) · `Manager::get_window` (manager/mod.rs:640) · `PathResolver::app_data_dir` (path/desktop.rs:247) · `tauri::{WindowEvent,PhysicalPosition,PhysicalSize,Position,Size}` (lib.rs:214-220) · label default `"main"` (tauri-utils config.rs:2363,3024).

## Impacto mapeado (Regla 0)

- **Archivos leídos completos:** `desktop/src-tauri/src/lib.rs` (218L: Builder/setup/invoke_handler/RunEvent), `desktop/src-tauri/Cargo.toml` (59L), `desktop/src-tauri/tauri.conf.json` (65L), `desktop/package.json` (scripts build/tsc), `desktop/src/pages/Settings.tsx` (vía codegraph, 224L)
- **Referencias hacia dentro (lo que el cambio usa):** `tauri::{Manager,Window,WindowEvent,Position,Size}` (core, ya importado `Manager` en lib.rs:20) · `serde/serde_json` (ya deps) · `app.path().app_data_dir()` · `app.get_window("main")`
- **Referencias entrantes (quién depende de lo tocado):** `main.rs → lib::run()` (único caller); ningún comando IPC depende de geometría; frontend no referencia estado de ventana (0 matches) → blast radius cerrado
- **Veredicto:** ADITIVO — 1 archivo nuevo (`window_state.rs`) + wiring en `lib.rs` (mod decl + 2 llamadas en setup + save en evento). Sin cambios a conexiones/comandos/IPC/frontend. Riesgo boot bloqueado → mitigado con default + try/catch (pre-mortem fallo 2). Gate D: no dispara (≤10 archivos, sin API pública nueva, contrato claro).

## Steps

- [x] **Step 0 — DISCOVERY:** gap real, APIs verificadas, decisión manual, task file creado
- [x] **Step 1 — `window_state.rs`:** struct + load/save JSON en app_data_dir + restore/attach (4 unit tests)
- [x] **Step 2 — wiring `lib.rs`:** `pub mod window_state` + restore en setup + save en Moved/Resized/CloseRequested
- [x] **Step 3 — VERIFY contrato:** check ✅ + test 4/4 ✅ + clippy ✅ + fmt ✅ + build ✅ + tsc ✅ + commit + sync plan

## Verify obtenido (2026-09-10)

- `cargo check -p vantadb-desktop` (en `desktop/src-tauri`) → ✅ Finished 8.12s
- `cargo test -p vantadb-desktop --lib window_state` → ✅ 4 passed / 0 failed
- `cargo clippy -p vantadb-desktop --all-targets -- -D warnings` → ✅ 0 warnings
- `rustfmt --check` (window_state.rs, lib.rs) → ✅ (1 auto-format aplicado)
- `npm run build` (en `desktop/`) → ✅ built in 19.89s
- `npx tsc --noEmit` (en `desktop/`) → ✅ 0 errores
- Deuda: smoke con app viva (WebView) pendiente de sesión manual — lógica cubierta por unit tests

## Implementación

### Step 1 — `desktop/src-tauri/src/window_state.rs` (nuevo)

```rust
//! FIND-20: persistencia manual de geometría de ventana (sin plugin).
//! Guarda `window-state.json` en app_data_dir; estado corrupto/ausente → default (try/catch).
```

- `WindowState { x, y, width, height, maximized }` + `load(app) -> WindowState` + `save(app, &Window)` + `restore(window, &WindowState)` + `attach(window, app_handle)` (on_window_event: Moved/Resized/CloseRequested → save best-effort)
- Unit tests: roundtrip serde + corrupto → default (Prove-It pre-mortem 2)

### Step 2 — `desktop/src-tauri/src/lib.rs`

- `pub mod window_state;`
- En `.setup()`: `if let Some(w) = app.get_window("main") { window_state::restore(&w, &window_state::load(app)) }` (best-effort, nunca falla boot)
- Tras setup / en restore: `window_state::attach(...)` con `AppHandle` clonado para saves async-safe (on_window_event es sync → save directo best-effort)

## Verify (contrato)

1. `cargo check` + `cargo test -p vantadb-desktop` en `desktop/src-tauri` (own workspace)
2. `npm run build` en `desktop/` exit 0
3. `npx tsc --noEmit` en `desktop/` exit 0
4. Smoke lógico: load corrupto → default; restore aplica posición/tamaño/maximize (unit tests, sin WebView en CI)

## Cierre

- Commit: `feat: FIND-20 — persistencia estado ventana desktop` (solo `docs/dev/tasks/FIND-20.md` + `desktop/src-tauri/src/window_state.rs` + `desktop/src-tauri/src/lib.rs` + sync plan)
- Recitation completed + skill progreso
