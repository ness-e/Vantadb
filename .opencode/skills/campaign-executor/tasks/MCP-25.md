# Task MCP-25 — Bulk import: bulk_import_file/stream

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-25` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🟡 · **Esfuerzo:** 🟢

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-25` completa en `docs/Backlog.md`.
- [x] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [x] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

## Impacto mapeado (Regla 0)

**Archivos leídos completos:** mismos que MCP-17 (tools.rs 883L completo, api.rs bulk_import_stream:1587/bulk_import_file:1676 con formato binario vdbdump, BulkImportReport:1572). `BulkImportReport` ya es público (`vantadb::BulkImportReport`, usado por wasm/python/cli).

**Hallazgo clave:** `bulk_import_stream` NO acepta NDJSON — espera binario `.vdbdump` (magic `VDBJSON\n` + versión + count LE + JSON array de `VantaMemoryInput`). El criterio del backlog ("NDJSON de N records") se implementa así: la tool `bulk_import_stream` MCP detecta el magic; si no está, parsea NDJSON línea a línea como `VantaMemoryInput` y sintetiza el header vdbdump alrededor del JSON array antes de delegar. `VantaMemoryInput` tiene campos Option implícitamente default en serde → NDJSON mínimo `{namespace,key,payload,metadata}` es válido.

**Veredicto:** impacto bajo — solo tools.rs + tests + docs; cero cambios core.

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
ARCHIVOS: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, skills/vantadb-mcp/SKILL.md, .opencode/skills/vantadb-mcp/SKILL.md, skills/vantadb-mcp/references/api-reference.md, docs/Backlog.md
VERIFY_CONTRATO: pasa (nota: clippy workspace intermitente por edición paralela de otra sesión en vanta-memory; crate propio limpio)
EVIDENCIA: cargo check ✅ / cargo test 49 passed incl. NDJSON 100 records → total_records=100 + malformed-line + missing-file error_content / SKILL.md hash SAME
