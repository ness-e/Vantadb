# MOD-11 — Nits agrupados MCP server (mcp.md H4-H8 de P32)

- **Plan:** `docs/plans/2026-08-25-batch-colaterales-deuda-desktop.md` Task 6
- **Estado:** ⬜ PENDING
- **Esfuerzo:** 🟢 | **Prioridad:** 🟢 | **Appetite:** 2h
- **Contrato:** `cargo test -p vantadb-mcp --test mcp_tests` pasa; k clamp aplicado; docs ×2 hash SAME
- **Cynefin:** 🟦 obvio
- **Fuente:** revisión P32 `docs/reviews/modulos/vantadb-mcp.md` (H4-H8)

## Spec (nits a resolver)

| Nit | Hallazgo P32 | Decisión DISCOVERY | Acción |
|-----|--------------|--------------------|--------|
| H4 | `search_semantic` no acota `k` (`args["k"].as_u64().unwrap_or(5) as usize`, tools.rs:1002) | **PENDIENTE** — `search_memory` sí clampa (`min(config.max_top_k)` en parse_search_request tools.rs:2073) | Clamp `k` contra `config.max_top_k` + test observable |
| H5 | Timeout no cancela spawn_blocking (server.rs:287) — tokio no cancela blocking tasks; retiene permit | **DOCUMENTAR** — cancelación correcta (abort/cooperative) es invasiva (requiere CancellationToken en todos los handlers); P32 dice "mitigación aceptable para v1, conviene documentarlo" | Comment en server.rs + nota SKILL.md § Security |
| H6 | `total_bytes` de collection_stats usa `format!("{:?}", v).len()` para metadata (tools.rs:1208) — Debug len ≠ bytes reales | **DOCUMENTAR** — aproximación deliberada (payload bytes + metadata debug len) | Comment en tools.rs + nota SKILL.md (collection_stats) |
| H7 | `namespace://{ns}` hardcodea `limit: 100` sin paginación expuesta (resources.rs:99) | **ALINEAR + DOCUMENTAR** — usar `config.default_list_limit`; el recurso devuelve primera página con `next_cursor`, paginar vía `memory_list` | Cambiar a config-driven + nota SKILL.md § Available MCP Resources |
| H8 | Superficie LLM06: `bulk_import_file(path)` (tools.rs:1613) y `wiki_ingest(root)` (wiki.rs:223) aceptan rutas arbitrarias del host | **DOCUMENTAR** — stdio local single-user = riesgo aceptado; falta nota threat model | Nota threat model LLM06 en SKILL.md § Security |

## Steps

1. ✅ **H4**: clamp k en search_semantic (tools.rs:1002) + test `test_mcp_search_semantic_clamps_k` (72 passed)
2. ✅ **H5**: comment limitación timeout en server.rs (tools/call + resources/read)
3. ✅ **H6**: comment aproximación total_bytes en tools.rs (collection_stats)
4. ✅ **H7**: resources.rs usa `config.default_list_limit` (remove hardcode 100)
5. ✅ **H8**: nota threat model LLM06 en SKILL.md § Security (bulk_import_file/wiki_ingest + destructive tools + k clamp + timeout)
6. ✅ **Docs**: AMBOS SKILL.md editados idénticos → hash SAME DF1A68FA...
7. ✅ **Verify**: cargo test 72 passed; fmt 0; clippy -D warnings 0; check 0; hash SAME

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `vantadb-mcp/src/handlers/tools.rs` (2175L) — handler dispatch, parse_search_request (clamp existente), collection_stats, bulk_import_file
- `vantadb-mcp/src/server.rs` (499L) — dispatch_request, timeout + spawn_blocking, serve_lines
- `vantadb-mcp/src/config.rs` (71L) — McpConfig: max_top_k=1000, default_list_limit=100, max_namespace_length=256
- `vantadb-mcp/src/validation.rs` (537L) — validate_identifier (256 namespace), for_each_record (streaming)
- `vantadb-mcp/src/handlers/resources.rs` (148L) — namespace:// handler hardcode limit:100
- `vantadb-mcp/src/wiki.rs` (613L, secciones) — wiki_ingest handler (root path)
- `vantadb-mcp/tests/mcp_tests.rs` (4212L, secciones) — search_semantic tests, collection_stats tests
- `.opencode/skills/vantadb-mcp/SKILL.md` + `skills/vantadb-mcp/SKILL.md` (570L, hash SAME 54346F37...)
- `docs/reviews/modulos/vantadb-mcp.md` — fuente P32 H4-H8
- `.opencode/rules/server-mcp.md` — R-1 (serverInfo/docs sincronizados), R-2 (semáforo+spawn_blocking)

**Referencias hacia dentro (dependen de lo que cambio):**
- `handle_tools_call` — llamado por server.rs:292 (dispatch_request). Sin cambio de firma.
- `handle_resources_read` — llamado por server.rs:315. Sin cambio de firma.
- `search_semantic` handler — tests mcp_tests.rs:787,1087 (distance semantics). Clamp no rompe (k=2 < 1000).
- `collection_stats` — tests mcp_tests.rs:1283,1390. total_bytes sigue >0.
- `namespace://` resource — sin tests dedicados directos; no rompe (default_list_limit=100 = mismo valor).
- SKILL.md ×2 — docs contract hash SAME; editar ambos idénticos.

**Referencias hacia afuera (de qué dependo):**
- `config.max_top_k` (1000) — ya usado por parse_search_request (tools.rs:2073) y threads.rs:206/scenes.rs:150.
- `config.default_list_limit` (100) — ya usado por memory_list (tools.rs:777).
- `embedded.search_vector(&vector, k)` — firma SDK estable; k es usize.

**Veredicto de impacto:** cambios acotados a 3 archivos Rust + 2 SKILL.md + 1 test. Sin cambio de firma pública, sin cambio de comportamiento (clamp solo acota inputs extremos; namespace:// mismo valor 100 vía config). No toca wal/vector/storage (propiedad Arch/Engine). No introduce concurrencia nueva → Regla 8 no aplica. H5 solo documenta → sin riesgo de regresión.

## Invariantes
- No cambiar firma pública de handlers.
- No cambiar semántica de search_vector/search_memory.
- SKILL.md ×2 deben quedar con hash SAME (editar ambos con contenido idéntico).
- No tocar wal/vector/storage (Arch/Engine).
- H5: NO intentar abort de spawn_blocking (invasivo, riesgo regresión) — solo documentar.

## Context Save Point
<!-- rellenar al devolver INCOMPLETO -->