# Task MCP-28 — Bulk import no escribe __vanta_namespace/__vanta_key

**Fuente de verdad:** `docs/Backlog.md` → fase **P25** → fila `MCP-28` (leer ANTES de ejecutar).
**Prioridad:** 🟠 · **Esfuerzo:** 🟢

## Fase 1 — DISCOVERY
- [x] Leer fila `MCP-28` completa en Backlog.
- [x] codegraph_explore: `bulk_import_stream`/`bulk_import_file` (`src/sdk/api.rs:1389,1478`) vs write path de `memory_record_to_node` — dónde se setean los campos `__vanta_*`.
  - **Hallazgo:** `bulk_import_stream` (api.rs:1638-1665) construye `UnifiedNode` a mano y solo setea `"payload"` (key literal SIN prefijo), vector, metadata y TTL. El read path (`memory_record_from_node`, serialization/mod.rs:299) exige `FIELD_NAMESPACE="__vanta_namespace"`, `FIELD_KEY="__vanta_key"`, `FIELD_PAYLOAD="__vanta_payload"`, `FIELD_CREATED_AT_MS/UPDATED_AT_MS/VERSION` → records bulk ilegibles vía memory_get/list/delete. Bug doble: faltan TODOS los campos reservados Y el payload usa key equivocada.
  - Write path correcto: `put_one` → `memory_record_to_node_owned` (serialization/mod.rs:404) setea todos los campos.
- [x] Verificar si `import_file` JSONL tiene el mismo problema.
  - **NO comparte el bug:** `import_file_inner` → `import_records` → `put_record_exact` (api.rs:532) → `memory_record_to_node_owned` → setea todos los campos `__vanta_*`. Ya cubierto por `test_import_file_happy_path` (impl_export.rs:572) que hace round-trip con `db.get("file","k1").is_some()`.

## Impacto mapeado (Regla 0)
- **Archivos leídos:** `src/sdk/api.rs` (put_one 112-187, put_batch_inner, get:428, put_record_exact:532, bulk_import_stream:1587-1680, tests 2270-2344), `src/sdk/serialization/mod.rs` (memory_record_from_node_inner:299-402, memory_record_to_node_owned:404-468, consts FIELD_*:14-24), `src/sdk/serialization/impl_export.rs` (import_records/import_file:253-347, tests 572-602).
- **Referencias hacia dentro:** bulk_import_stream usa `memory_node_id`, `UnifiedNode::new/set_field`, `FieldValue`, `now_ms`, `FIELD_EXPIRES_AT_MS`; constantes `FIELD_*` son `pub` en serialization/mod.rs.
- **Referencias entrantes:** `bulk_import_stream` ← 7 callers (wasm/python/mcp/api); `bulk_import_file` ← 4 callers; 0 tests que cubran bulk hoy. Cambio SOLO comportamiento interno de construcción del nodo — ninguna firma pública cambia (api-contract R-8 OK: lógica vive en core).
- **Veredicto:** impacto contenido en `bulk_import_stream` + tests nuevos al final de api.rs. Sin cambios de API pública ni de formato binario (.vdbdump).

## Fase 2 — EJECUCIÓN
- [x] Replicar seteo de campos reservados `__vanta_namespace/__vanta_key` en el bulk path del SDK.
  - `src/sdk/api.rs` `bulk_import_stream`: ahora setea `FIELD_NAMESPACE`, `FIELD_KEY`, `FIELD_PAYLOAD`, `FIELD_CREATED_AT_MS`, `FIELD_UPDATED_AT_MS`, `FIELD_VERSION` (timestamp único de import, version=1) — espejo de `memory_record_to_node_owned`. Fix adicional: el payload se escribía con key literal `"payload"` en vez de `"__vanta_payload"`. Sin cambios de firma pública (api-contract R-8 OK). Sparse vectors y metadata DateTime/List siguen sin soportarse en bulk (limitación preexistente del formato, documentada).
- [x] Test round-trip: bulk_import → memory_get por key direccionable.
  - `test_bulk_import_roundtrip_addressable_via_memory_get` (src/sdk/api.rs): bulk_import_stream → `db.get(ns, k1/k2)` encuentra ambos records + payload + version. ✅ pasa.
- [x] Test: import_file JSONL → mismo criterio (o documentar que ya funciona).
  - **Ya funciona** — no requiere fix: `import_file` → `import_records` → `put_record_exact` → `memory_record_to_node_owned` (setea todos los campos). Cobertura existente: `test_import_file_happy_path` ya hace round-trip con `db.get("file","k1").is_some()` — re-ejecutado ✅.

## Fase 3 — VERIFICACIÓN
- [x] `cargo check -p vantadb` — ✅ Finished dev profile
- [x] `cargo test -p vantadb-mcp` — ✅ 7 passed; 0 failed
- [x] `cargo fmt -p vantadb -p vantadb-mcp` — ✅ aplicado

## Fase 4 — CIERRE
- [x] Actualizar nota de deuda en SKILL.md ×2 (`skills/vantadb-mcp/SKILL.md:322` + `.opencode/skills/vantadb-mcp/SKILL.md:322`: caveat "NOT addressable" → "ARE addressable") + fila Backlog ✅ (`docs/Backlog.md` P25 → "✅ Resuelta (MCP-28)").

## RESULTADO (obligatorio)
✅ COMPLETO. Evidencia: test round-trip nuevo pasa (`4 passed` en módulo bulk_import); import_file tests 4/4 pasan; cargo check/fmt OK. Sin commit (el lead comitea).
