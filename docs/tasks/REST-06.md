# REST-06 — ServerConnection.query (IQL en consola web)

## Metadata
- Plan: `docs/plans/2026-08-19-vanta-studio-fase4.md` (Task 10, Wave 1)
- Estado: ✅ COMPLETO
- Implementación: vanta-lead (STRATEGY tras 2 RETRYs vacíos de vanta-worker)
- Commit: (ver git log)

## Contrato
- `ServerConnection.query` implementado (HTTP `/api/v2/query`, que ya existía en `ServerClient`).
- `queryResultFromResponse` completo (Read/Write/StaleContext, sin truncar) — ya estaba completo en `vanta-http-map.ts` (intentos previos).
- Mapeo `vanta_query` + `vanta_iql_autocomplete` — ya presentes.
- Tests roundtrip IQL.

## Resultado
- `desktop/src-tauri/src/connections/server.rs`: `impl VantaConnection for ServerConnection::query` + `query_response_to_result()` (mapeo Read/Write/StaleContext desde `QueryResponse` wire) + 3 tests unitarios.
- Verify: 59/59 lib tests desktop (3 nuevos `connections::server::tests`), `cargo fmt` OK.
- Nota: el mapeo web no requería cambios — el único gap real era el default `Unsupported` del trait en ServerConnection.