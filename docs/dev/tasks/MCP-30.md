# MCP-30 — Scenes API vía MCP: `scene_read`/`scene_list`/`scene_query`

**Estado:** ✅ COMPLETED · **Wave:** 3 (MCP, serial) · **Appetite:** max 4h · **Esfuerzo:** 🟢

## Objetivo

Exponer la navegación de escenas vía MCP: tools `scene_read(session_key, scene_name)`,
`scene_list(session_key)`, `scene_query(session_key, keyword, top_k?)` — wrappers directos de los
handlers puros de `vanta-memory::gateway` (`knowledge_handlers.rs`, funciones puras sobre
`&VantaEmbedded`). Hoy un agente MCP no puede navegar su memoria estructurada por escenas.
Patrón de referencia: commit `4d752f14` (MCP-31, `context_assemble`).

## Decisiones de diseño (DISCOVERY)

1. **Módulo dedicado `vantadb-mcp/src/scenes.rs`** (patrón MEM-33, igual que context.rs):
   `scene_tool_definitions()` + `handle_scene_tool()` dispatch + 3 fns privadas.
2. **Pre-condición verificada ✅:** `vantadb-mcp/Cargo.toml:14` ya declara `vanta-memory`
   (MEM-52). Cero dependencias nuevas. El docstring de `gateway/mod.rs` dice literalmente
   *"Typed request/response layer a future MCP server wraps"* — consumidor previsto.
3. **Shape determinista:** se serializan directamente los tipos serde existentes
   (`SceneReadResponse`/`SceneListResponse`/`SceneQueryResponse`, snake_case estable). Cero wire types nuevos.
4. **Trust boundary:** `session_key`/`scene_name` → `validate_identifier`; `keyword` →
   `validate_payload(keyword, config.max_query_length)`; `top_k?` u64 >0, cap `config.max_top_k`.
5. **Error contract MEM-32:** errores de dominio (`KnowledgeError::{Invalid,NotFound}`) como
   `error_content` autocorregible; params faltantes/inválidos como JSON-RPC invalid_params. Nunca `?`.
6. **embed=None en scene_query:** sin hook de embeddings en MCP v1 — ranking keyword-only,
   mismo modo degradado D38 documentado en context.rs. Upgrade futuro: hook opcional.
7. **Seed del test SIN pipeline L0 (pre-mortem resuelto):** `upsert_scene(db, session, name,
   summary, content)` es API pública de vanta-memory (`core/scene/mod.rs:29`) y así siembran
   los tests propios del gateway (`knowledge_handlers.rs:289`) — no requiere LLM runner.

## Impacto mapeado (Regla 0)

- **Leídos completos:** `vantadb-mcp/src/context.rs` (plantilla completa), 
  `vantadb-mcp/src/handlers/tools.rs` (dispatch 413+, tools_list ~397-409, tail 1638),
  `vantadb-mcp/src/lib.rs`, `vantadb-mcp/tests/context_tests.rs` (setup/call/msg helpers),
  `vanta-memory/src/gateway/knowledge_handlers.rs` (handlers + tests seed), 
  `vanta-memory/src/gateway/mod.rs`, `vanta-memory/src/core/scene/scene_index.rs:48`
  (upsert_scene), `vantadb-mcp/Cargo.toml`.
- **Referencias entrantes:** `handle_tools_call` (arm nuevo), `handle_tools_list` (extend array).
  Tests `tests/mcp_tests.rs` asertan presencia (no conteo exacto) — verificar al correr suite.
- **Referencias salientes:** `vanta_memory::gateway::{scene_read, scene_list, scene_query}` +
  request/response types; helpers `pub(crate)` de validation.rs; `VantaEmbedded::from_engine`.
- **Veredicto:** blast radius contenido a `vantadb-mcp` (2 archivos tocados + 1 nuevo módulo +
  1 test nuevo) + docs ×3 (SKILL.md ×2 hash SAME, api-reference ×2, docs/api/MCP.md).
  Sin cambios en vanta-memory ni core. Riesgo bajo — wrapper thin read-only.

## Spec

### Tools (tools/list)

```jsonc
{"name": "scene_read", "description": "Reads one live memory scene block by name from a session's structured scene store. Returns the SceneBlock {scene_name, meta{created,updated,summary,heat}, content}. Missing or soft-deleted scenes return a not-found error message (indistinguishable by design). Read-only.",
 "inputSchema": {"type": "object", "properties": {
   "session_key": {"type": "string"}, "scene_name": {"type": "string"}},
   "required": ["session_key", "scene_name"]}}
{"name": "scene_list", "description": "Lists the scene index of a session (heat descending, soft-deleted excluded). Returns {scenes:[{scene_name,created,updated,summary,heat}]}. Use scene_read to load a block. Read-only.",
 "inputSchema": {"type": "object", "properties": {"session_key": {"type": "string"}}, "required": ["session_key"]}}
{"name": "scene_query", "description": "Keyword search over the live scene blocks of a session (term overlap between keyword and summary+content; ranked overlap desc then heat desc). Returns {hits:[{scene_name,summary,heat,updated,score}]}. Load hits via scene_read. Read-only.",
 "inputSchema": {"type": "object", "properties": {
   "session_key": {"type": "string"}, "keyword": {"type": "string"},
   "top_k": {"type": "number", "description": "Optional max hits (default 5)"}},
   "required": ["session_key", "keyword"]}}
```

### Round-trip contract (mecánico)

Test round-trip con sesión seedada vía `upsert_scene`: `tools/list` registra las 3 →
`scene_list` devuelve >0 escenas → `scene_read` por id/nombre devuelve contenido →
`scene_query` con keyword matchea → casos error (params inválidos, sesión vacía, NotFound).

## Steps

- ✅ S1: RED — `tests/scene_tests.rs` 7 round-trips (6 fallaron con "Tool not found" ✅ RED real)
- ✅ S2: GREEN — `src/scenes.rs` (164L) + `lib.rs mod scenes` + 4 líneas en `handlers/tools.rs`
- ✅ S3: VERIFY — fmt exit 0 · clippy -D warnings exit 0 · nextest -p vantadb-mcp **51/51** (44 previos + 7 nuevos)
- ✅ S4: Docs — SKILL.md ×2 hash SAME, api-reference ×2, mcp-protocol ×2, MCP.md (60 tools/6 familias, parity script 0 gaps)
- ✅ S5: Commit `d03b6517` (pre-commit hooks ok)

## Context Save Point

Tarea completa. Hallazgos de implementación: (1) el id de navegación en `scene_list` es
`filename` (= scene_name sin extensión), no un campo `scene_name` — doc y tests alineados;
(2) seed vía `upsert_scene()` pública, sin L0/LLM (pre-mortem confirmado); (3) `handle_tools_list()`
toma 0 args. Colateral del worktree NO incluido en commit: cambios ajenos preexistentes
(completions/, avance/, lessons.md, task files MCP-31/REVIEW-15/MOD-03). Próxima tarea wave W3
serial: MCP-32 (threads CRUD).
