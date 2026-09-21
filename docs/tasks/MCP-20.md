# Task MCP-20 — Recovery índices: rebuild_index + audit_text_index

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-20` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🟡 · **Esfuerzo:** 🟢

## Impacto mapeado (Regla 0)
- **Leídos completos:** `src/sdk/search/audit.rs` (impl VantaEmbedded: audit_text_index/deep/repair/generate_snippet), `src/sdk/api.rs:723` (rebuild_index → VantaIndexRebuildReport), `src/sdk/types.rs:279,326,644` (structs report, todos Serialize), `vantadb-mcp/src/handlers/tools.rs` (schemas L22-364, dispatch L367-1516, patrón purge_expired/compact_layout L1130-1164), `vantadb-mcp/tests/mcp_tests.rs` (setup_storage, maintenance_call, test_maintenance_tools_round_trip).
- **Referencias hacia dentro:** wrappers ya existen en core SDK vía Python/WASM bindings (rebuild_index, audit_text_index) — sin cambios en API pública del SDK.
- **Referencias entrantes:** `handle_tools_call` es el único dispatcher; tests usan `handle_tools_list`/`handle_tools_call` directo. Docs que cuentan tools: SKILL.md x2 + api-reference + mcp-protocol (contar contra source antes de actualizar números — learning MCP-18/19).
- **Veredicto:** cambio aditivo acotado a `vantadb-mcp/src/handlers/tools.rs` (+schemas, +dispatch arms), tests en `mcp_tests.rs`, docs sync. Cero riesgo sobre core SDK/WAL/vector (prohibidos).

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-20` completa en `docs/Backlog.md`.
- [x] `codegraph_explore` de los símbolos/archivos listados en la fila (blast radius).
- [x] Confirmar que ningún cambio requiere alterar API pública del SDK (wrappers only).

## Fase 2 — EJECUCIÓN
- [x] Implementar las acciones (1..n) definidas en la fila, en orden.
- [x] Tool(s) nueva(s) en `vantadb-mcp/src/handlers/tools.rs` + schema JSON-RPC.
- [x] Tests round-trip en `vantadb-mcp/tests/mcp_tests.rs` según criterios de la fila.

## Fase 3 — VERIFICACIÓN (contrato mecánico)
- [x] `cargo check -p vantadb-mcp` — ✅
- [x] `cargo clippy -p vantadb-mcp -- -D warnings` — ✅
- [x] `cargo test -p vantadb-mcp` — ✅ 93 tests (60 en mcp_tests, 3 nuevos)

## Fase 4 — CIERRE
- [x] Actualizar `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (hash SAME).
- [x] Marcar la fila como ✅ en `docs/Backlog.md` con nota breve del cambio.

## RESULTADO (obligatorio al final)

RESULTADO: ✅ COMPLETO
TASK_ID: MCP-20
STEPS_OK: 3/3 fases de ejecución + discovery + verificación + cierre
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead comitea)
ARCHIVOS: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, skills/vantadb-mcp/SKILL.md, skills/vantadb-mcp/references/api-reference.md, skills/vantadb-mcp/references/mcp-protocol.md (+ copias .opencode hash SAME), docs/Backlog.md, este task file
VERIFY_CONTRATO: pasa (check/clippy -D warnings/test/fmt -p vantadb-mcp ✅)
BLOQUEO: ninguno
