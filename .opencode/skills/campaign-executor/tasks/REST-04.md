# REST-04 — Cursor real en server (paginación list/search — gap VS-CORE-01)

## Metadata
- **Plan file:** `docs/plans/2026-08-19-vanta-studio-fase4.md`
- **Creado:** 2026-08-19
- **last-synced:** 2026-08-19
- **Estado:** ✅ COMPLETO (vanta-worker; verify lead: 1832 lib + 18/18 TS + curl sin duplicados)

## Contrato
- `GET /api/v2/list` y `POST /api/v2/search` devuelven `next_cursor` real (cursor del core, serialización string segura).
- Paginación verificable: 2 llamadas con limit N devuelven N y el resto (sin duplicados).
- Tests.

## Resultado
- `src/cli_server.rs`: `SearchPageRequest` (flatten + cursor/limit server-only, backward compatible) y `SearchPageV2` (`{records, next_cursor}`); `records_search` traduce cursor/limit → `top_k = cursor + limit + 1` y recorta. Test `v2_list_and_search_paginate`.
- `desktop/src/vanta-http-map.ts`: `vanta_search` unwrap `{records, next_cursor}` → `SearchResult[]` (firma `vanta.ts` intacta).
- `desktop/src/vanta-http-map.test.ts`: test search actualizado al nuevo wire page.
- `desktop/scripts/selfcheck-web-e2e.ts`: lectura de `.records`.
- Verify: `cargo test --features server --lib` 1832 passed / 0 failed; `node --test` 18/18; `tsc --noEmit` OK; curl list limit=2 → 2+2+1 keys sin dups; search limit=2 → 3×2 = 5 keys sin dups.