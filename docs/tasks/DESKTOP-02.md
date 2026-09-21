# DESKTOP-02: Scaffold Tauri v2 + propio workspace

## Metadata
- **Plan file:** `docs/plans/2026-08-06-desktop-mvp.md`
- **Creado:** 2026-08-06T22:10
- **last-synced:** 2026-08-06T22:25
- **Estado:** ✅ COMPLETED
- **Agente:** vanta-worker

## Blast Radius
- **Callers:** `desktop/src-tauri/src/main.rs` → `vantadb_desktop_lib::run()`; frontend `src/App.tsx` → `invoke("ping")`
- **Callees nuevos:** `tauri::Builder`, `tauri::generate_handler![ping]`, `tauri::generate_context!`
- **Implicaciones:** root `cargo check -p vantadb` debe quedar INVARIANTE — `desktop/src-tauri/Cargo.toml` declara su propio `[workspace]` (1 miembro = `.`), fuera de los members raíz (vantadb, vantadb-python, vantadb-server, vantadb-mcp, vantadb-wasm).

## Zero-code plan (3 bullets)
1. **Desacople de workspace:** `desktop/src-tauri/Cargo.toml` con `[workspace] members=["."]` propio → `cargo` no engancha al workspace raíz; root lock intacto.
2. **Shell Tauri v2:** mantener `tauri`/`tauri-build` v2, `[lib] vantadb_desktop_lib` (staticlib/cdylib/rlib), `tauri.conf.json` (productName `vantadb-desktop`, id `com.vantadb.desktop`), `capabilities/default.json` con `core:default`, `ping` command → `"pong"`.
3. **Frontend Vite/React:** `src/App.tsx` llama `invoke<string>("ping")` y muestra respuesta; `npm run build` pasa.

## Contrato (verificado, NO asumido)
"`Test-Path desktop/` → True; `cargo check` en `desktop/src-tauri` → exit 0; `cargo check -p vantadb` (raíz) → exit 0 sin cambios; `npm run build` en `desktop` → exit 0."

## Herramientas
- create-tauri-app 4.6.2 (npm), cargo, node/npm, codegraph_explore, webfetch (docs oficiales Tauri).

## Steps
### Step 1: Scaffold Tauri v2 no interactivo
- **Acción:** `npm create tauri-app@latest desktop -- --template react-ts --manager npm --identifier com.vantadb.desktop --yes`
- **Verify:** `Test-Path desktop/src-tauri/tauri.conf.json` → True; schema `tauri.app/config/2` (v2, no v1).
- **Estado:** ✅

### Step 2: Desacoplar workspace + shell Tauri
- **Acción:** `src-tauri/Cargo.toml` con `[workspace] members=["."]` propio, `tauri`/`tauri-build` v2, `[lib] name=vantadb_desktop_lib`, capacibilades `core:default`, command `ping`.
- **Verify:** `cargo check` en `src-tauri` → exit 0.
- **Estado:** ✅

### Step 3: Frontend pide `ping`
- **Acción:** `src/App.tsx` llama `invoke<string>("ping")` y renderiza la respuesta.
- **Verify:** `npm run build` → exit 0.
- **Estado:** ✅

### Step 4: Invariante root + commit scoped
- **Acción:** `cargo check -p vantadb` (raíz) → exit 0 (sin cambios en Cargo.lock raíz). Commit `git add desktop/**`.
- **Verify:** `git show --stat` solo `desktop/`.
- **Estado:** ✅

## Dependencias
- Ninguna (Wave 0). DESK-03 (←02) dependerá de este scaffold.

## Notas (@DESK-02 relay en paralelo)
- **Colisión de run paralelo:** DESK-04 y DESK-08 escriben en el MISMO árbol `desktop/` (crearon `desktop/Cargo.toml` `members=["src-tauri"]`, `src/error.rs`, `src/connections/*`, `tests/server_client_mock.rs`) y dejaron `lib.rs` con `run()` no-op esperando a DESK-02 que cableé el runtime Tauri. **MERGE, NO REPLACE:** mantuve `pub mod connections; pub mod error;` + re-exports en `lib.rs` e injerté el runtime Tauri (`ping`) + `[lib] vantadb_desktop_lib` sobre su base.
- **Doble workspace:** quedó tanto `desktop/Cargo.toml` (DESK-04) como `[workspace]` dentro de `src-tauri/Cargo.toml`. Ambos aíslan del root; `cargo check` desde `desktop/src-tauri` usó el workspace propio y pasó. **Arbitrar en checkpoint lead** si `desktop/Cargo.toml` debe eliminarse (mi `src-tauri/Cargo.toml` ya es autocontenido).
- **`npm run tauri dev`** (DoD ventana manual): no se abrió en CI (entorno headless). Contrato CI-fuable `cargo check` + `npm run build` pasan.
- Scaffold trae warning benigno `unexpected_cfgs: cfg(mobile)` del template — no introducido por esta tarea.

## Context Save Point
- **Fecha:** 2026-08-06T22:25
- **Branch:** develop
- **CI pendiente:** sí — correr `dev-tools/verify.ps1` al cerrar el plan (Regla 1 pre-commit).
- **Decisiones:** (1) Desacoplar con `[workspace] members=["."]` en `src-tauri/Cargo.toml` (no vacío puro) para coexistir con `desktop/Cargo.toml` sibling. (2) No eliminar archivos de DESK-04/08. (3) Merge del comando `ping` + module expors en el `lib.rs` sibling.
- **Problemas conocidos:** doble definición de workspace (`desktop/Cargo.toml` vs `src-tauri/Cargo.toml`) → arbitrar; warning generics `mobile`.
- **Próxima tarea:** DESK-03 (integra crate `vantadb` + managed state + healthcheck vía NativeConnection).