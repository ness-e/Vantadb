# REST-03 — Endpoint `graph_v2` con DTO desktop (u128-safe)

## Metadata
- **Plan file:** `docs/plans/2026-08-19-vanta-studio-fase4.md`
- **Creado:** 2026-08-19
- **last-synced:** 2026-08-19
- **Estado:** ✅ COMPLETO (vanta-worker; verify lead: 28/28 lib + 18/18 TS + smoke curl u128)

## Contrato
- Endpoint(s) `/api/v2/graph/v2/*` que serialice DTO de grafo con u128 seguro (string en wire, patrón `thread_id` de Fase 3).
- Mapeo `vanta_graph_bfs/dfs/degree` en `vanta-http-map.ts` → dejan de ser rechazos.
- Test roundtrip con IDs u128 grandes (> u64::MAX).

## Resultado
- `src/cli_server.rs`: handlers `graph_v2_bfs/dfs/degree` + rutas `/api/v2/graph/v2/*` (legacy `/api/v2/graph/*` intacto).
- `desktop/src/vanta-http-map.ts`: mapeos `vanta_graph_bfs/dfs/degree` → `/api/v2/graph/v2/*`, args camelCase → wire (roots string, direction lowercase), response passthrough (IDs ya string).
- `desktop/src/vanta-http-map.test.ts`: tests bfs/dfs/degree + roundtrip u128 (id `18446744073709551616` sobrevive como string).
- Verify: `cargo test --features server --lib cli_server` 28/28; `node --test src/vanta-http-map.test.ts` 18/18; smoke curl `/api/v2/graph/v2/bfs` con root > u64::MAX → 200 `{nodes,edges}`. Rechazos vanta-http-map: 8 → 4.