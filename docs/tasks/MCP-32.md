# Task: MCP-32 — Threads CRUD vía MCP

- **Plan:** docs/plans/2026-08-23-backlog-triage.md (NO editar)
- **Estado:** ✅ COMPLETED
- **Ejecutado por:** vanta-lead inline (SARL step 3 STRATEGY tras 3 delegaciones con canal vacío)
- **Archivos clave:** vantadb-mcp/src/threads.rs (nuevo), handlers/tools.rs, tests/thread_tests.rs (nuevo), SKILL.md ×2, api-reference ×2, docs/api/MCP.md

## Steps

1. ✅ DISCOVERY — API real en `src/agentic/thread.rs` + builder.rs (:161-206); patrón scenes.rs (`d03b6517`); `MessageThread` Serialize pero `u128` no serializa en serde_json → helper `thread_to_json` (ids como strings, convención MEM-32).
2. ✅ ACT — módulo `threads.rs`: 6 tools (`thread_create/send/get/list/delete/purge_expired`), schemas tools/list, dispatch en tools.rs (:409, :1644); trust boundary `validate_payload` (títulos/roles cap max_query_length, contenido 4×); errores de dominio como error_content, params inválidos -32602; get de thread ausente → "not found" (consistente con memory_get/MCP-11).
3. ✅ VERIFY — RED inicial: test list panic-eó por u128 en serde_json (bug real cazado por TDD) → GREEN con thread_to_json.

## Verificación mecánica

- Contrato: `cargo nextest run -p vantadb-mcp` = **58/58** ✅ (7 tests nuevos round-trip: create→send→get→list→delete→get-not-found, purge=0, malformed id → -32602, tools_list incluye las 6)
- fmt --check ✅ · clippy pendiente en commit final del orquestador
- Docs: SKILL.md ×2 hash SAME (True/True) · api-reference ×2 (66 tools, sección Threads API) · docs/api/MCP.md (66 in 7 families)

## Notas

- delete_thread es permanente (sin undo) — documentado en la tool description.
- Boundary RBAC D34: los threads viven en la DB embebida del server MCP; sin proxy de por medio.
- SKILLS_CARGADAS: source-driven-development (firmas reales builder.rs via codegraph), test-driven-development (RED u128-caught), security-and-hardening (validate_payload en boundary), code-review-and-quality (checklist pre-commit).
