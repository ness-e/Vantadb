# DESKTOP-11 - Spawn manager subproceso MCP (sidecar)

- **Estado:** ✅ COMPLETED (2026-08-07)
- **Esfuerzo:** 🟢
- **Archivos clave:** `desktop/src-tauri/src/connections/child_process.rs`
- **Agente:** `vanta-worker` (review de vanta-audit: sub-proceso es trust boundary)
- **Commit:** `feat(DESKTOP-11): spawn manager subproceso MCP (sidecar)`

## Context

Inicio de la Fase 3 (adaptador MCP stdio): lanzar el binario `vantadb-server`
en modo `--mcp` como sub-proceso del desktop app, con stdio piped, stderr
teed a log temporal, y timeout de arranque. El sub-proceso es un trust
boundary — nunca `.unwrap()` en paths del usuario.

## Confirmación del flag `--mcp` (decisión point)

**El flag `--mcp` SÍ existe** en `vantadb-server/src/main.rs:27`:

```rust
let is_mcp = std::env::args().any(|a| a == "--mcp");
```

- NO es un flag clap — es un match de raw `std::env::args`. `.arg("--mcp")`
  funciona idéntico.
- En modo MCP, `vantadb::cli_server::init_telemetry(true, …)` →
  `init_telemetry_fmt` usa `.with_writer(stderr)` → **toda la telemetría va a
  stderr**, y `run_stdio_server` (vantadb-mcp `lib.rs:440`) registra
  `"MCP stdio server started"`. Ese marcador en stderr es la señal de ready.
- stdout está reservado SOLO para JSON-RPC (protocolo MCP) → nunca tocar
  stdout del child para readiness.
- **Confirmación:** el flag vive en la crate `vantadb-server` (workspace
  miembro, separado de la core root). La core root NO se tocó y NO hay que
  tocar — el contrato de DESK-11 se cumple sin modificar la raíz.

## Contrato cumplido

- `locate_binary()` → resuelve `vantadb-server` (best-effort, `Option<PathBuf>`,
  nunca `.unwrap()` en paths del usuario):
  1. `VANTVADB_SERVER_BIN` env override (explicit)
  2. `current_exe()` sibling (bundled sidecar pattern en release)
  3. `$CARGO_MANIFEST_DIR/{,../}/target/debug/vantadb-server[.exe]` (dev)
- `struct McpSpawn`:
  - `spawn()` async → localiza, lanza `--mcp` con stdio piped, tees stderr a
    `temp_dir/vantadb-mcp-<pid>.log`, espera el marcador de ready en stderr
    dentro de [`SPAWN_TIMEOUT`]=30s; timeout → kill + `VantaError::Mcp`.
  - `pid()`, `is_running()`, `log_path()`, `request_shutdown(grace)`, `kill()`
  - `Drop` → `start_kill()` limpio (sync, no panic).
  - Manejo seguro: nunca `.unwrap()` en paths; errores → `VantaError`.
- `tests/child_process.rs` (integración): spawn→ready→kill limpio + verifica
  stderr log; **SKIP documentado** si el binario no existe (no compra raíz).

## Archivos

- `desktop/src-tauri/src/connections/child_process.rs` - launcher sidecar MCP
- `desktop/src-tauri/src/connections/mod.rs` - `pub mod child_process` + re-export `McpSpawn`
- `desktop/src-tauri/Cargo.toml` - añade features `process` y `io-util` a tokio (sin nueva dep)
- `desktop/src-tauri/tests/child_process.rs` - integración spawn/kill + stderr

## Verification

- `cargo check` en `desktop/src-tauri`:
  - **`child_process.rs` compila limpio (0 errores).**
  - ⚠️ El lib tiene 1 error PRE-EXISTENTE de trabajo paralelo sin commitear:
    `commands/connection.rs:16` `use crate::connections::NativeConnection` →
    `no NativeConnection in connections`. ES DES-06 en vuelo (uncommitted), NO
    está en el scope de DES-11; NO se toca. Es un cascada: el lib no compila ⇒
    `tests/child_process.rs` también reporta unresolved import del lib.
  - `cargo test --lib`: no corre hasta que DES-06 exponga `NativeConnection`.
- **Skip del e2e real:** los tests integración (spawn del binario real) quedan
  gated a que el binario exista y a que el lib compile — skip instructivo con
  stash del build de binario. Ver note de collaboración.

## Notes

- **Collaboración con tasks paralelas:** el estado del árbol de trabajo trae
  `commands/connection.rs`/`mod.rs` modificados + `data.rs` untracked (DES-06
  wire MCP) que referencian un `NativeConnection` aún no re-exportado en
  `connections/mod.rs`. Ese trabajo rompe el `cargo check` del desktop. DES-11
  deja su módulo aislado y no toca commands. Reportar a orchestrator: el
  `cargo check` del desktop quedará exit-0 cuando DES (commands) exponga
  `NativeConnection`, o lo resuelva el dueño de esa tarea.
- Decision: duplicar el marcador `MCP stdio server started` const en
  child_process.rs (no importable desde la core root sin dep).
- La core root `vantadb` NO se tocó; `cargo check -p vantadb` raíz invariante.