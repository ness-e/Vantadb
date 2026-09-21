# Task MCP-26 — Menores: capabilities + generate_snippet + list_snapshots

**Fuente de verdad:** `docs/Backlog.md` → fase **P25 - Exposición MCP/HTTP** → fila `MCP-26` (leer ANTES de ejecutar: problema, acciones numeradas, archivos exactos).
**Prioridad:** 🟢 · **Esfuerzo:** 🟢

## Impacto mapeado (Regla 0)
- **Leídos completos:** `src/sdk/api.rs:1160` (capabilities() → VantaCapabilities, Serialize), `src/sdk/search/audit.rs:43` (generate_snippet → Option<String>, función libre sin estado), `src/storage/engine/mod.rs:523` (list_snapshots() → Result<Vec<String>>, wrapper trivial <10 líneas), `vantadb-mcp/src/handlers/tools.rs` (patrón schema+dispatch MEM-32), `vantadb-mcp/tests/mcp_tests.rs` (patrones de test).
- **Referencias entrantes:** mismas superficies de docs que MCP-20 (contar tools contra source antes de actualizar números).
- **Veredicto:** aditivo, mismo archivo que MCP-20. `list_snapshots` ES trivial (llamada directa a `storage.list_snapshots()`) → se incluye. Sin cambios en API pública del SDK.

## Fase 1 — DISCOVERY
- [x] Leer la fila `MCP-26` completa en `docs/Backlog.md`.
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
TASK_ID: MCP-26
STEPS_OK: 3/3 fases de ejecución + discovery + verificación + cierre
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead comitea)
ARCHIVOS: mismos que MCP-20 (mismo commit/superficie): tools.rs, mcp_tests.rs, SKILL.md x2, api-reference.md x2, mcp-protocol.md x2, docs/Backlog.md, este task file. list_snapshots INCLUIDA (wrapper trivial <10 líneas).
VERIFY_CONTRATO: pasa (check/clippy -D warnings/test/fmt -p vantadb-mcp ✅)
BLOQUEO: ninguno
