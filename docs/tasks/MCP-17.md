# Task MCP-17 — Backup/restore vía MCP: export + import JSONL

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-17` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🟠 · **Esfuerzo:** 🟠

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-17` completa en `docs/Backlog.md`.
- [x] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [x] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** `vantadb-mcp/src/handlers/tools.rs` (883L), `src/sdk/serialization/impl_export.rs` (export_namespace/export_all/import_records/import_file + write_export_file), `src/sdk/api.rs` (put_record_exact:532, bulk_import_stream/file, list_namespaces), `src/sdk/serialization/mod.rs` (export_line_from_record pub, record_from_export_line pub(crate)), `src/sdk/types.rs` (VantaExportReport/VantaImportReport/VantaMemoryExportLine), `vantadb-mcp/src/validation.rs` (error_content/text_content/serialize_content/for_each_record), `vantadb-mcp/tests/mcp_tests.rs` (helpers setup_storage/default_config/patrón handle_tools_call), `.opencode/rules/server-mcp.md`.

**Referencias hacia dentro (lo que toco):** `record_from_export_line` — 1 caller interno (import_file_inner). Re-export lists en `src/sdk/mod.rs` (pub use serialization/types). `tools.rs` base_tools + dispatch match.

**Referencias entrantes (qué depende):** tools/list y tools/call son API wire MCP — agregar tools es aditivo, no rompe clientes. `record_from_export_line` pub es aditivo (nada existente cambia). Docs que deben sync: `skills/vantadb-mcp/SKILL.md`, `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME), `skills/vantadb-mcp/references/api-reference.md` ("33 tools" → +2), `docs/api/EMBEDDED_SDK.md` § Export/Import.

**Hallazgo clave:** `put_record_exact` valida `node_id == memory_node_id(ns,key)` con hash pub(crate) → el import fiel NO puede reconstruir records desde MCP sin exponer `record_from_export_line`. Decisión: hacerla `pub` (diff 1 palabra) + re-export; el export usa API 100% pública (`list()` vía `for_each_record` + `export_line_from_record`). Límite stdio: const `MAX_TRANSFER_BYTES = 10 MB` documentado.

**Veredicto:** impacto bajo — todo aditivo, sin cambios de comportamiento existente.

## Fase 2 — EJECUCIÓN
- [x] Implementar las acciones (1..n) definidas en la fila, en orden.
- [x] Tool(s) nueva(s) en `vantadb-mcp/src/handlers/tools.rs` + schema JSON-RPC.
- [x] Tests round-trip en `vantadb-mcp/tests/mcp_tests.rs` según criterios de la fila.

## Fase 3 — VERIFICACIÓN (contrato mecánico)
- [x] `cargo check -p vantadb-mcp`
- [x] `cargo clippy -p vantadb-mcp -- -D warnings`
- [x] `cargo test -p vantadb-mcp`

## Fase 4 — CIERRE
- [x] Actualizar `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME).
- [x] Marcar la fila como ✅ en `docs/Backlog.md` con nota breve del cambio.

## RESULTADO (obligatorio al final)
RESULTADO: ✅ COMPLETO
STEPS_OK: 12/12 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead comitea)
ARCHIVOS: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, src/sdk/serialization/mod.rs, src/sdk/mod.rs, skills/vantadb-mcp/SKILL.md, .opencode/skills/vantadb-mcp/SKILL.md, skills/vantadb-mcp/references/api-reference.md, docs/api/EMBEDDED_SDK.md, docs/Backlog.md
VERIFY_CONTRATO: pasa
EVIDENCIA: cargo check ✅ / clippy -D warnings ✅ / cargo test 48 passed (incl. roundtrip export→import→get en DB fresca + multi-namespace + malformed-line counting) / SKILL.md hash SAME (EC94974E…)
