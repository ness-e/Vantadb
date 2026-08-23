# Task MCP-19 — Tool memory_put_batch (ingestión masiva)

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-19` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🟡 · **Esfuerzo:** 🟢

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-19` completa en `docs/Backlog.md`.
- [x] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [x] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

### Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `vantadb-mcp/src/handlers/tools.rs`, `src/sdk/api.rs` (put_batch :237 + put_batch_inner :253), `vantadb-mcp/src/validation.rs`.
- **Referencias entrantes:** idem MCP-18 — solo arms/schemas nuevos, nada modificado.
- **Referencias salientes:** helper nuevo `parse_memory_input` reutiliza validate_identifier/payload/vector, parse_sparse_vector, parse_metadata.
- **Veredicto:** aditivo puro. SDK put_batch valida todos los inputs upfront → all-or-nothing; duplicados = upsert con version bump (no errores parciales). Wire documentado acorde.

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
Bloque `RESULTADO`: `✅ COMPLETO` | `🟡 INCOMPLETO` | `❌ FALLIDO` + evidencia (comandos corridos, tests pasando, archivos modificados).
