# Task DEBT-01 — Reparar gate docs-coverage + 13 gaps reales

**Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 19 (Fase 3, Engineering Health)
**Estado:** ✅ COMPLETED
**Fecha:** 2026-08-05
**Ejecutor:** vanta-docs

## Contexto

`scripts/validate-docs-coverage.ps1` línea 64 apuntaba a `src\sdk\search.rs` (directorio, no archivo) → el error de `Select-String` mataba la validación SDK (reportaba "0 items"). Además existían gaps reales de documentación.

## Parte A — Fix de ruta (script)

- `scripts/validate-docs-coverage.ps1:64`: `src\sdk\search.rs` → `src\sdk\search\mod.rs` (módulo real del search SDK).
- Excludes correctos:
  - SDK públicos: `test_empty` (constructor `#[doc(hidden)]` solo para tests, builder.rs:127).
  - Python `pyInternals`: `try_enter`, `drain`, `drop` (plomería interna `OpGate`/`OpGuard`, lib.rs:85-156, NO son `#[pymethods]` — el bloque pymethods empieza en lib.rs:158).

Al arreglar la ruta quedaron visibles 28 gaps SDK reales (la sección estaba muerta). Documentados los 27 públicos + `test_empty` excluido.

## Parte B — Gaps documentados

| Gap | Ubicación código | Doc |
|-----|------------------|-----|
| `bulk_commit_interval` | config.rs:304 | CONFIGURATION.md §1 tabla VantaConfig |
| `segment_optimizer` | config.rs:366 | CONFIGURATION.md §1 tabla VantaConfig |
| `NoVectorForKey` | error.rs:265 | EMBEDDED_SDK.md Error Handling |
| `create` (snapshot) | cli.rs:326 | CONFIGURATION.md §4 Commands (`snapshot create`) |
| `search-all` / `search-multi` | cli.rs:271,252 | CONFIGURATION.md §4 Commands |
| `similar-to-key` | cli.rs:220 | CONFIGURATION.md §4 Commands (reemplaza row stale `search-similar`) |
| `vacuum` (wal) | cli.rs:369 | CONFIGURATION.md §4 Commands (`wal vacuum`) |
| `bulk_import` / `bulk_import_bytes` | vantadb-python/src/lib.rs:1130,1140 | PYTHON_SDK.md Advanced Operations |
| `graph_page_rank` / `graph_degree_centrality` | lib.rs:1531,1558 + gds.rs | PYTHON_SDK.md Node/Graph |
| `recover_archived_nodes` | lib.rs:1593 + builder.rs:207 | PYTHON_SDK.md Advanced + EMBEDDED_SDK.md |
| 27 métodos SDK (search_multi, search_all, similar_to_key, remove_edge, filtered traversals, accumulators, threads, snapshots, graphrag_search, vacuum, pipeline, optimizer_config, set_optimizer_config, reindex_hnsw_from_text, bulk_import_file, bulk_import_stream, recover_archived_nodes) | src/sdk/* | EMBEDDED_SDK.md |

## Contrato

- `pwsh scripts/validate-docs-coverage.ps1` → exit 0 ✅
- Sección SDK reporta items reales: 63 items ok en EMBEDDED_SDK.md ✅ (antes "0 items")

## Commit

`docs(DEBT-01): reparar gate docs-coverage y documentar 13 gaps`

## Notas

- NO se tocó `docs/api/MCP.md` (otra tarea).
- El número "13" del plan cuenta config(2)+error(1)+cli(5)+python(5); la sección SDK estaba muerta antes del fix de ruta y destapó 27 gaps SDK reales adicionales, documentados para cumplir exit 0.
