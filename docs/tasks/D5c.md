# D5c — Clampar limit/top_k a MAX_K en handlers HTTP

## 1. Descubrimiento (auto-detect tipo → codegraph blast radius → web si ambigüedad → baseline `/cleanCA <scope>`)
- Tipo: lógica/frontera (worker, TDD). Sin discovery MCP (adaptador §10: plan no está en campaign).
- `codegraph_explore "records_list records_search handlers.rs server"`: `records_list` (handlers.rs:390, callers routing.rs/router.rs), `records_search` (handlers.rs:519, mismos callers). Sin tests que los cubran. Blast radius chico: 2 handlers + 2 rutas, sin callers de lógica (son endpoints).
- Patrón a seguir: `clamp_top_k` Python (vantadb-python/src/lib.rs:44-49): `if requested > MAX_K { warn } ; requested.min(MAX_K)`, con `MAX_K` unificado de core (`vantadb::config::MAX_K` = 10_000, src/config.rs:50).
- Estado actual server: `records_list:419` usa `params.limit.unwrap_or(100)` sin cota; `records_search:532-534` deriva `page_size` y `request.top_k = cursor+page+1` sin cota (DoS: top_k gigante → materializa ranking gigante).
- Baseline `/cleanCA src/server/handlers.rs`: A2 aplica a B1/D1c, no a este clamp; no se corre baseline (tarea D5c = 1 línea de defensa por handler).

## 2. Contrato (qué cambia / qué NO cambia / archivos exactos / comandos de verify)
- Cambia: `src/server/handlers.rs` — (a) `use crate::config::MAX_K;` + `fn clamp_limit`; (b) `records_list`: `let limit = clamp_limit(params.limit.unwrap_or(100));`; (c) `records_search`: `page_size` clampado + `request.top_k` final clampado a `MAX_K`; (d) `#[cfg(test)] mod tests` con tests del clamp.
- NO cambia: wire (`next_cursor` idéntico para valores en rango — el clamp solo recorta >10_000); `conversation_add` (es B1, prohibido tocar); rutas en router.rs/routing.rs; validación de `filter_ops`/versiones/delete.
- Archivos exactos: `src/server/handlers.rs` (único archivo prod). Task file: `docs/tasks/D5c.md` (este).
- Verify: `cargo fmt --check` + `cargo clippy -p vantadb --all-targets --deny warnings` + `cargo nextest run --profile audit -p vantadb server` + `rg "MAX_K|clamp_limit" src/server/handlers.rs` (cota visible en ambos handlers).

## 3. Steps atómicos (☐ uno por slice: implementar → test → verificar → commit; si falla: `git reset --hard HEAD` del slice)
- [x] Slice 1: `use crate::config::MAX_K` + `fn clamp_limit` (patrón ERR-022 con `tracing::warn`) + aplicar en `records_list:419` + test `clamps_limit_to_max_k` (RED→GREEN). Verify: `cargo check -p vantadb` + nextest filtro.
- [x] Slice 2: aplicar `clamp_limit` en `records_search` (`page_size` y `top_k` final) + test `clamps_search_page_to_max_k`. Verify: mismo gate + suite `server` completa.
- [x] Cierre: `cargo fmt --check`, clippy deny warnings, `cargo nextest run --profile audit -p vantadb server`, `rg` contrato. Sin commit (lo ejecuta el lead). RESULTADO + recitation.

## 4. Cierre (RESULTADO + `/cleanCA <scope>` PASS + recitation)
- RESULTADO exigido por despacho (RESULTADO, STEPS_OK, PROXIMO_STEP, COMMIT_HASH, ARCHIVOS, VERIFY_CONTRATO, BLOQUEO, GATES_EVALUADOS, SKILLS_CARGADAS).
- `/cleanCA` mental regla A2: sin `top_k` ilimitado en frontera HTTP tras el clamp.
