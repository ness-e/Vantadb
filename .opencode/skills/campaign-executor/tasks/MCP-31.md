# MCP-31 — Context engine vía MCP: tool `context_assemble`

**Estado:** ⏳ IN PROGRESS · **Wave:** 3 (MCP, serial) · **Appetite:** max 1d · **Esfuerzo:** 🟠

## Objetivo

Exponer el context engine de vanta-memory vía MCP: tool `context_assemble(session_key, token_budget, query?, messages?)`
wrapper de `assemble_with_recall` + `perform_auto_recall`. Hoy solo alimenta desktop por IPC (MEM-58).

## Decisiones de diseño (DISCOVERY)

1. **¿Expongo el compresor MMD? NO en v1.** Solo assemble (compaction + recall injection).
   El compresor es internal detail (`AggressiveBoundary`, `MemoryScoreMap` no cruzan la frontera);
   el report de compactación viaja como metadata estable dentro del output serde existente.
2. **Pre-condición BND-04 VERIFICADA ✅:** `vantadb-mcp/Cargo.toml:14` ya declara
   `vanta-memory = { path = "../vanta-memory" }` (MEM-52). vanta-memory YA linkea en el binario
   del server MCP — no se agregan dependencias → no stop condition. No hay threads/wasm issues.
3. **Shape determinista:** se serializa `IntegratedContext` directamente (ya es `serde::Serialize`
   con snake_case estable): `{messages:[{role,content,id?}], report:{mode,msgs_conserved,msgs_before,tokens_before,tokens_after}, mmd_injected, recall_injected}`. Cero wire types nuevos.
   Sesión inexistente → `perform_auto_recall` devuelve `Ok(None)` sin error; assemble sigue sobre
   la historia provista. Errores de dominio → `error_content` (patrón MEM-32 learning: nunca `?`).
4. **Semántica de recall:** corre si `session_key` está presente (`query?` opcional — con query vacía
   perform_auto_recall igual inyecta persona+scene-navigation, contrato documentado del hook).
   Desktop exige query no-vacía; acá relajamos al contrato propio de vanta-memory (más útil para agentes externos).
5. **Parámetros:** `session_key` (req, validado con `validate_identifier`/max_namespace_length),
   `token_budget` (req u64 >0), `query?`, `messages?` (historia a compactar, default [] = solo recall blocks).
   Cada mensaje valida role ∈ {system,user,assistant,tool_call,tool_result} + content ≤ max_payload_length.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `vantadb-mcp/src/handlers/tools.rs` (1-529, 1630-1759 patrón dispatch wiki),
  `vantadb-mcp/src/wiki.rs` (50-310 plantilla MEM-32/MEM-33), `vantadb-mcp/src/lib.rs`,
  `vantadb-mcp/src/config.rs`, `vantadb-mcp/src/validation.rs` (helpers), `vantadb-mcp/Cargo.toml`,
  `vanta-memory/src/context_engine/engine.rs` (assemble_with_recall), `types.rs`, `mod.rs`,
  `vanta-memory/src/core/hooks/auto_recall.rs` (perform_auto_recall), `core/hooks/mod.rs`,
  `vanta-memory/src/core/record/l1_reader.rs`, `vanta-memory/src/seed/{mod,input}.rs`,
  `desktop/src-tauri/src/commands/memory.rs` (run_assemble/run_recall referencia).
- **Referencias hacia dentro (entrantes):** `handle_tools_call` despacha por nombre (arm nuevo);
  `handle_tools_list` extiende arrays de módulos; tests `tests/mcp_tests.rs::test_mcp_tools_list`
  solo aserta presencia (no cuenta exacto) — verificar.
- **Referencias salientes:** `crate::context::*` nuevo módulo; usa helpers pub(crate) de validation.rs;
  APIs públicas de vanta-memory (estables, ya usadas por desktop/proxy/pipeline_worker).
- **Veredicto:** blast radius contenido a `vantadb-mcp` (2 archivos tocados + 1 nuevo) + docs ×3.
  Sin cambios en vanta-memory ni core. Riesgo bajo — wrapper thin.

## Spec

### Tool: `context_assemble`

```jsonc
// tools/list entry
{
  "name": "context_assemble",
  "description": "Assembles an agent context window under a token budget using the memory OS context engine: compacts the provided chat history and injects session recall (L1 memories for the query, persona, scene navigation) when session_key is present. Read-only.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "session_key": { "type": "string", "description": "Session whose memories/persona/scenes feed the recall blocks" },
      "token_budget": { "type": "number", "description": "Token budget for the assembled context (must be > 0)" },
      "query": { "type": "string", "description": "Optional user text that drives L1 memory search" },
      "messages": { "type": "array", "description": "Optional chat history to compact; each item is {role: system|user|assistant|tool_call|tool_result, content: string, id?: string}", "items": { "type": "object" } }
    },
    "required": ["session_key", "token_budget"]
  }
}
```

### Handler (función pura sobre storage)

```
db = VantaEmbedded::from_engine(storage.clone())
(prepend, append) = if recall hits: (r.prepend_context, r.append_system_context) else (None, None)
   via perform_auto_recall(db, AutoRecallParams{user_text: query.unwrap_or(""), session_key, isolation: None, config: default}, None)
ctx = assemble_with_recall(messages, budget, &TokenEstimator::default(), 0, &AssembleConfig::default(),
                           None /* active_mmd */, prepend.as_deref(), append.as_deref(), None, None)
→ text_content(serialize_content(&ctx))
```

### Contrato mecánico

- [ ] tool `context_assemble` en tools/list con schema JSON-RPC válido (patrón MEM-32)
- [ ] test round-trip sesión seedada (persona vía `import_seed`) → contexto ≤ token_budget
- [ ] SKILL.md sincronizado AMBOS lados hash SAME (`skills/vantadb-mcp/` fuente + copia `.opencode/skills/vantadb-mcp/`)
- [ ] docs/api/MCP.md actualizado (+ references/api-reference.md ambas copias)
- [ ] Verify full: fmt + clippy -D warnings + nextest -p vantadb-mcp

### Steps

- ✅ Step 1 RED: `tests/context_tests.rs` — 7 tests escritos, 6 fallan con "Tool not found" (RED confirmado)
- ✅ Step 2 GREEN: `src/context.rs` + wire lib.rs/tools.rs → 7/7 verdes, suite completa 44/44
- ✅ Step 3 DOCS: SKILL.md ×2 hash SAME + api-reference ×2 + mcp-protocol ×2 + docs/api/MCP.md
  (+ fix inline CONFIGURATION.md `allow_insecure` desbloqueando validate-docs-coverage 0 gaps)
- ⬜ Step 4 VERIFY+CLOSURE: fmt ✅ · clippy ✅ · nextest 44/44 ✅ · docs-coverage ✅ · commit pending

## Context Save Point

- Tests RED iniciales: expectativa "≤ budget SIEMPRE" era incorrecta vs engine (min_keep=2 protege
  tail; engine devuelve over-budget deliberadamente si el tail protegido excede — engine.rs:150).
  Test ajustado al contrato real: tail chico + assert ≤ budget + tokens_after ≤ tokens_before.
- Assert de modo exacto (`emergency`) removido: testear detalle de implementación; aggressive
  alcanza primero con historias normales.
- Colateral en worktree ajeno a esta tarea: docs/api/MCP.md contiene sección "Getting Started"
  de otra tarea en vuelo — incluida en este commit (mismo dominio, inseparable), documentado en body.

## Context Save Point

(nada aún — tarea nueva)
