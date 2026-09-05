# Task MCP-16 — Tools mantenimiento TTL/WAL: purge_expired + compact_wal

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-16` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🔴 · **Esfuerzo:** 🟡

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-16` completa en `docs/Backlog.md`.
- [x] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [x] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

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
