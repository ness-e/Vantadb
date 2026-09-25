# DESKTOP-09 - ServerConnection sobre cliente IQL + test server real

- **Estado:** ✅ COMPLETED (2026-08-07)
- **Esfuerzo:** 🔴
- **Archivos clave:** `desktop/src-tauri/src/connections/server.rs`
- **Agente:** `vanta-worker`

## Context

Adaptador `ServerConnection` que implementa la trait `VantaConnection` del
multi-connection contract (DESK-04) delegando al cliente IQL tipado
`ServerClient` (DESKTOP-08). Expone un runtime HTTP remoto sobre el binario
`vantadb-server` sin duplicar lógica: la lógica vive en el core, el binding es
thin wrapper.

El binario `vantadb-server` NO tiene flag `--require-auth` (solo `--mcp`);
singular auth es 100% config env: `VANTADB_API_KEY`, `VANTADB_REQUIRE_AUTH=true`,
`VANTADB_HOST`, `VANTADB_PORT`, `VANTADB_STORAGE_PATH` (validado en
`src/config.rs` + spawn real).

## Contrato cumplido

- `ServerConnection { client: ServerClient, cfg, connected, next_id_counter }`
  con `with(cfg)` y helper `timeout_ops()` → `VantaError::Timeout`
- Impl. de `VantaConnection`:
  - `info()` → `ConnectionInfo` (via `Capability::Http`, status)
  - `capabilities()` → `[Capability::Http]`
  - `connect()` → `client.health()`; success → connected, `success:false` →
    `VantaError::Http { kind: Domain, status: Some(200) }`
  - `disconnect()` → connected=false
  - `ingest()` → parsea id u128, arma `content` + metadata, `client.put`,
    devuelve id
  - `ingest_batch()` → loop por item
  - `search()` / `get()` (u128; missing → `Http { kind: NotFound }`) /
    `delete()` / `list()` (`limit` via `.take`)
  - `health()` → wire `HealthReport` → contract con `backend: "http"`
- `relational_str()` tolera valores relationales planos (mock) y en forma
  tipada del server real (`{"String": ...}` / `{"Number": ...}` /
  `{"Bool": ...}` / `{"List": [...]}`)

## Archivos

- `desktop/src-tauri/src/connections/server.rs` - adaptador `ServerConnection`
- `desktop/src-tauri/src/connections/mod.rs` - `pub mod server` + re-export
- `desktop/src-tauri/tests/server_connection_real.rs` - e2e contra server real

## Colaboración con tasks paralelas

- DES-05 (`NativeConnection`) llegó en vuelo y tocó `connections/mod.rs` con
  `pub mod native` (native.rs aún sin commitear). Para no cerrar un commit con
  un módulo inexistente, el commit de DESKTOP-09 incluye `mod.rs` SOLO con la
  línea `server`; la línea `native` se restaura al árbol de trabajo y la
  volvió a registrar DES-05 en su propio commit.

## Verification

- `cargo test` en `desktop/src-tauri`:
  - `cargo test --lib`: 21 pass (contract, wire, error, native)
  - `cargo test --test server_connection_real` con `VANTADB_TEST_SERVER=1`
    (spawn del binario real): 2 pass
    - `real_server_health_put_get_search_delete` (e2e real, 3.4s)
    - `dead_server_yields_http_error`
  - Gate: `VANTADB_TEST_SERVER=1` + binario disponible; se skipa sin env.
  - Fix clave: primer run falló `got ""` en `get()` porque el server real
    devuelve valores relational en forma tipada, no plana; corregido con
    `relational_str()`.

## Notes

- Los tests real-server spawnan `target/debug/vantadb-server.exe` con env de
  auth; `/health` público sin token (`wait_ready` usa token=None).
- IDs numéricos; `type=u128` con parse; no parse → `VantaError::Other`.
- Commit: `feat(DESKTOP-09): ServerConnection sobre cliente IQL`