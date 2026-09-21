# DESKTOP-08 — Cliente IQL tipado + tests mock (Wave 0, paralelo)

- **Estado:** ✅ COMPLETED (2026-08-06)
- **Esfuerzo:** 🟡
- **Archivos clave:** `desktop/src-tauri/src/connections/server_client.rs`
- **Agente:** `vanta-worker`

## Context

La API real del servidor VantaDB tiene 3 endpoints (`/health`, `/metrics`,
`/api/v2/query` IQL). put/get/delete/list/search van como statements IQL por
`/api/v2/query` (NO es un REST por-endpoint). Validado contra
`docs/api/HTTP_API.md` + `src/cli_server.rs`:

- Auth: `Authorization: Bearer <token>` (constant-time compare en
  `auth_middleware`; `/health` es pública)
- Body query: `{"query": "<IQL>"}` → response envelope
  `{success, data, node_id?, nodes?}`
- Fallos de dominio: HTTP 200 con `success:false` → error de dominio, NO
  transporte

## Contrato cumplido

- `ServerClientConfig { url, port, token, timeout }` (en `wire_types.rs`)
- Statements IQL mapeados y autenticados:
  - `health()` → GET /health (sin auth)
  - `metrics()` → GET /metrics (Bearer, texto Prometheus)
  - `query(stmt)` → POST /api/v2/query (Bearer, `{"query": ...}`)
  - `put(id, kind, fields)` → `INSERT NODE#<id> TYPE <kind> {k: "v", ...}`
  - `get(id)` → `MATCH NODE#<id>`
  - `delete(id)` → `DELETE NODE#<id>`
  - `list(kind)` → `FROM <kind>`
  - `search(kind, field, text, min)` → `FROM <kind> WHERE <field> ~ "text" min = <min>`
- `success:false` (HTTP 200) → `VantaError::Http { kind: Domain, status: 200 }`
- Status no-2xx → `VantaError::Http { kind: Unauthorized/NotFound/..., status }`

## Archivos

- `desktop/src-tauri/src/connections/server_client.rs` — wrapper reqwest tipado
- `desktop/src-tauri/src/connections/wire_types.rs` — DTOs wire
  (`ServerClientConfig`, `QueryRequest`, `QueryResponse`, `NodeDTO`,
  `HealthReport`) — creados por DESK-08, reubicados por DESK-04 a `wire_types`
- `desktop/src-tauri/src/error.rs` — `VantaError` + `HttpErrorKind`
- `desktop/src-tauri/tests/server_client_mock.rs` — mock axum (dev-dep)

## Colaboración con tasks paralelas

- DESK-04 (contract) llegó mientras tanto: reescribió `types.rs` con los DTOs
  del trait `VantaConnection` y movió los DTOs wire de DESK-08 a
  `wire_types.rs` verbatim. Se actualizó el import del cliente a
  `crate::connections::wire_types` y se registró `pub mod server_client` +
  re-exports en `connections/mod.rs`.
- DES-02 (scaffold Tauri) modificó `Cargo.toml`/`lib.rs` con deps Tauri —
  fuera de scope; no se tocó.

## Verification

- `cargo test` en workspace aislado (sin deps Tauri, mismo código):
  28 tests pass (17 unit + 11 integración mock server)
- Cobertura integración: health, metrics bearer, put (INSERT + escaping de
  comillas), get (MATCH), delete (DELETE), list (FROM), search (~ min),
  success:false → Domain error, wrong token → Unauthorized, timeout → Http

## Notes

- Escaping: `{:?}` de Rust produce literales con comillas escapadas — evita
  doble escape (bug corregido en iteración 2).
- El check completo `cargo test` en `desktop/src-tauri` NO puede correr hoy:
  el manifest compartido tiene deps Tauri en vuelo (DES-02) que crashean el
  linker MSVC (STATUS_STACK_BUFFER_OVERRUN en `windows` crate). Verificación
  aislada usada como gate; re-checkear al mergear.
- Commit: `feat(DESKTOP-08): cliente IQL tipado + tests mock server`
