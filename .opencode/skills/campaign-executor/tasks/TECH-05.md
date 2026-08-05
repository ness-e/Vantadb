# TECH-05 — Implementar resource MCP `schema://`

**Plan:** `docs/plans/2026-08-05-backlog-validation-actions.md` → Task 21
**Estado:** ✅ COMPLETED 2026-08-05
**Commit:** `feat(TECH-05): resource MCP schema://`

## Premisa (corregida)

`docs/api/MCP.md:79-81` documenta `schema://` (config HNSW + text index version) pero el handler solo servía `metrics://`, `memory://`, `namespace://`. Contrato roto.

## Shape JSON definido

```json
{
  "vector_index": {
    "type": "HNSW",
    "format_version": 8,
    "config": { "m": 32, "m_max0": 64, "ef_construction": 100, "ef_search": 100, "ml": ..., "distance_metric": "Cosine", "flat_threshold": 10000, "index_type": "Hnsw", "auto_tune": false }
  },
  "text_index": {
    "schema_version": 4,
    "tokenizer": { "name": "tantivy-multilingual", "version": 1 },
    "key_format": "namespace\\0token\\0key"
  }
}
```

- `vector_index.config` = configuración HNSW **activa** del engine (`storage.vec_index().config`, `HnswConfig` serializable).
- `vector_index.format_version` = `vantadb::VECTOR_INDEX_VERSION` (u16, hoy 8).
- `text_index` = `TextIndexSpec::default()` (schema_version 3/4 según feature `advanced-tokenizer`, tokenizer, key_format).

## Implementación

- **`src/text_index.rs`:** `TextTokenizerSpec` / `TextIndexSpec` `pub(crate)` → `pub` (solo visibilidad; ya tenían `Default` que resuelve el cfg del tokenizer).
- **`src/lib.rs`:** re-export `pub use text_index::{TextIndexSpec, TextTokenizerSpec};`.
- **`vantadb-mcp/src/lib.rs`:** `schema://` añadido a `handle_resources_list()`; rama `uri == "schema://"` en `handle_resources_read()` → `build_schema_resource(storage)`.
- **`vantadb-mcp/tests/mcp_tests.rs`:** `test_mcp_resources_read_schema` (valida shape completo) + assert de `schema://` en `test_mcp_resources_list`.

## Verificación

- `cargo test -p vantadb-mcp` — ✅ 26 passed (incluye `test_mcp_resources_read_schema`)
- `cargo check -p vantadb` — ✅
- `cargo clippy -p vantadb-mcp -- -D warnings` — ✅

## Notas

- Sin solape con MCP-02..05 (no existen en backlog; limpiado en F0).
- `docs/api/MCP.md:79-81` ya describía `schema://` correctamente — sin cambio de doc necesario.
- La API del core no cambia de comportamiento; solo se expone el spec del text index (era `pub(crate)`).
