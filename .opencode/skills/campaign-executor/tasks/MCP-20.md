# Task MCP-20 — Recovery índices: rebuild_index + audit_text_index

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-20` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🟡 · **Esfuerzo:** 🟢

## Fase 1 — DISCOVERY
- [ ] Leer la fila `MCP-20` completa en `docs/Backlog.md`.
- [ ] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [ ] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

## Fase 2 — EJECUCIÓN
- [ ] Implementar las acciones (1..n) definidas en la fila, en orden.
- [ ] Tool(s) nueva(s) en `vantadb-mcp/src/handlers/tools.rs` + schema JSON-RPC.
- [ ] Tests round-trip en `vantadb-mcp/tests/mcp_tests.rs` según criterios de la fila.

## Fase 3 — VERIFICACIÓN (contrato mecánico)
- [ ] `cargo check -p vantadb-mcp`
- [ ] `cargo clippy -p vantadb-mcp -- -D warnings`
- [ ] `cargo test -p vantadb-mcp`

## Fase 4 — CIERRE
- [ ] Actualizar `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME).
- [ ] Marcar la fila como ✅ en `docs/Backlog.md` con nota breve del cambio.

## RESULTADO (obligatorio al final)
Bloque `RESULTADO`: `✅ COMPLETO` | `🟡 INCOMPLETO` | `❌ FALLIDO` + evidencia (comandos corridos, tests pasando, archivos modificados).
