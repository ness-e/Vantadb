# MEM-07: F3 MCP tools skill_* — 6 tools del review agent sobre SkillStore

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md`
- **Creado:** 2026-08-20T16:30
- **last-synced:** 2026-08-20T16:30
- **Estado:** ✅ COMPLETED (commit <hash>, verify 13/13 nextest + 44/44 mcp_tests + clippy -D warnings)
- **Workflow:** feature-add (spec → implement → verify → review → accept → close)

## Blast Radius

**Callers (aguas arriba — dependen de esto):**
- MEM-35 (F3, ⬜ PENDING): `GET /skill/listing` — data plane REST; NO depende de estos tools MCP (canal separado), pero comparte `SkillStore::list`. Sin conflicto.
- Clientes MCP (agentes/LLM): los 6 tools `skill_*` son API pública del server MCP → documentar en `docs/api/MCP.md` (Regla 3).
- `vantadb-mcp` lib: `McpConfig` gana 2 knobs (`max_skill_resource_bytes`, `max_skill_total_bytes`) — struct pública; callers usan `Default`/`from_storage`, sin literales externos → sin break.

**Callees (aguas abajo — de lo que depende):**
- `src/skills.rs` (MEM-06, commit `92cf709f`): `SkillStore::new(&StorageEngine)`, `get_head/get_version/list/create/update/patch/delete`. **Sin cambios al core — MEM-07 solo consume.**
- `src/sdk/types.rs` (MEM-06): `SkillCreateInput`, `SkillUpdateInput`, `SkillPatchInput`, `SkillListOptions`, `SkillRecord` (re-exportados en `vantadb::sdk` y raíz `vantadb::`).
- `vantadb-mcp/src/handlers/tools.rs` (MEM-02 patrón, commit `32b09daf`): `handle_tools_list` (array de defs) + `handle_tools_call` (match por name → `text_content(serialize_content(...))` / `error_content(...)`); `McpError::invalid_params(...).to_json()` para params faltantes.
- `vantadb-mcp/src/validation.rs`: `validate_identifier`, `validate_payload`, `serialize_content`, `text_content`, `error_content`.
- `vantadb-mcp/src/config.rs`: `McpConfig` (bounds pattern — `max_*`).

**Implicaciones:**
- NO tocar el core (`src/skills.rs`, `src/sdk/`): D13 = los tools consumen `SkillStore`/tipos directamente, lógica en core, MCP = thin wrapper.
- NO deps nuevas: `vantadb` ya es dep de `vantadb-mcp` (feature `cli`); serde_json ya está. WASM-safe (sin red, sin std::time nuevo — solo comparación de tamaños).
- Owner check: MCP embebido (stdio) NO tiene capa auth HTTP (MEM-05 es server HTTP). La identidad del caller es el parámetro `owner_agent` que el agente declara — misnmo contrato que TDAM (`user_id/team_id/agent_id` vienen del router). Mismatch owner → **404 idéntico al not-found** (sin filtrar existencia — port `assertTeamMatch` de `skill-permission.ts`).
- Límites: 5MB/recurso + 50MB/skill = constantes TDAM (`DEFAULT_MAX_RESOURCE_SIZE_BYTES=5_000_000`, `DEFAULT_MAX_SKILL_TOTAL_BYTES=50_000_000`) como defaults de knobs McpConfig.
- `skill_patch` en TDAM es substring replace (old_string/new_string/replace_all) — el core `SkillPatchInput` toma content completo → el MCP traduce (read head → replace → patch). No es duplicar lógica, es el wire contract del tool.
- `skill_files_write` almacena recursos como entries `file:{path}` en `metadata` (BTreeMap<String,String>) del skill: valor = JSON `{content, encoding, mime_type, is_executable, size_bytes}`. Sin store de recursos nuevo en core (MEM-06 modelo = content + metadata).

## Contrato
`cargo check -p vantadb-mcp` pasa; tests dedicados de skills tools (D19):
- `cargo nextest run -p vantadb-mcp -- skills` ✅ (CRUD vía MCP, optimistic lock, owner check 404 sin filtrar existencia, límites 5MB/50MB, path traversal, paridad con API nativa)

## Herramientas
- bash: `cargo check -p vantadb-mcp`, `cargo nextest run -p vantadb-mcp -- skills`, `cargo fmt --check`, `cargo clippy -p vantadb-mcp -- -D warnings`
- codegraph (intel), skill source-driven-development (referencia TDAM), doubt-driven-development (decisiones no triviales), security-and-hardening (FASE SECURITY: trust boundary MCP)

## Impacto mapeado (Regla 0) — GATE ANTES DE EDITAR

**Archivos leídos completos (2026-08-20):**
- `vantadb-mcp/src/handlers/tools.rs` — leído completo (884 líneas): patrón list+call; helpers `text_content`/`error_content`/`serialize_content`; dispatch match; `McpError::invalid_params` para params.
- `vantadb-mcp/src/server.rs` — leído completo (283 líneas): `handle_tools_call` vía spawn_blocking con `Executor::new(&storage_ctx)` + config clonado; sin auth — identity es param del tool.
- `vantadb-mcp/src/validation.rs` — leído parcial (1-120, 330-389): `validate_identifier`, `validate_payload`, `validate_vector`, `validate_search_profile`, `serialize_content` (339), `text_content` (346), `error_content` (363), `for_each_record`.
- `vantadb-mcp/src/config.rs` — leído completo (65 líneas): `McpConfig` knobs `max_*`, `Default` explícito, `from_storage`.
- `vantadb-mcp/src/lib.rs` — leído completo (36 líneas): `mod` list + re-exports `pub use handlers::tools::*`.
- `vantadb-mcp/Cargo.toml` — leído completo: deps `vantadb` (features=["cli"]), tokio, serde, serde_json, tracing. Sin deps nuevas necesarias.
- `src/skills.rs` — leído completo vía codegraph + líneas 590-692: `SkillStore` API pública (`new/get_head/get_version/list/list_versions/create/update/patch/delete`), validadores privados (`validate_skill_id` rechaza `#~{}:`; `validate_owner`/`validate_name` rechazan `#{}:`).
- `src/sdk/types.rs` — leído líneas 770-902: `SkillRecord` (content: String, metadata: BTreeMap<String,String>), `SkillCreateInput`, `SkillUpdateInput` (description: String obligatorio), `SkillPatchInput` (Options), `SkillListOptions` (owner_agent/name_prefix/limit/offset), `SkillListPage`, `SkillWriteResult { record, idempotent }`.
- `src/lib.rs` — leído líneas 110-189: `pub mod skills;` + re-exports raíz y `sdk::`.
- TDAM clon @ `97f9465`: `skill-tools.ts` (218 líneas, completo) — 6 tools + shapes; `skill-permission.ts` (68, completo) — assertOwner/assertTeamMatch(404)/assertVersionFresh; `skill-resource-store.ts` (120/253) — 5MB/50MB + `assertPath` (no vacío, no absoluto, no NUL, no `..`); `skill-handlers.ts` (grep) — error codes 40301/40401/40901.
- `vantadb-mcp/tests/mcp_tests.rs` — leído 1-80 + 615-744: patrón test (`setup_storage()` tempdir, `handle_tools_call(&params, &executor, &storage, &cfg)`, `v["content"][0]["text"]`).
- `docs/api/MCP.md` — grep Tools section (líneas 18-50): formato de documentación de tools.

**Referencias hacia dentro (grep):** `skills.rs` (nuevo en vantadb-mcp) — sin referencias aún; `McpConfig` usado por lib/server/handlers (Default/from_storage — sin literales). `handle_tools_call`/`handle_tools_list` re-exportados en lib.rs.

**Referencias salientes del cambio:**
- `vantadb-mcp/src/skills.rs` (nuevo) ← consume `vantadb::skills::SkillStore` (pub), `vantadb::sdk::*` tipos (pub), `McpConfig` (pub fields), validation helpers (pub(crate)), `McpError` (pub). Sin cambios a esos archivos.
- `vantadb-mcp/src/handlers/tools.rs` ← agrega 6 entries al array de list + 6 arms al match que delegan a `crate::skills::handle_skill_tool`. Sin cambios a arms existentes.
- `vantadb-mcp/src/config.rs` ← agrega 2 campos `pub` con defaults en `Default`. Callers existentes intactos.
- `vantadb-mcp/src/lib.rs` ← agrega `mod skills;`. Sin re-export necesario (tests usan `handle_tools_call` público).
- `vantadb-mcp/tests/skills_tests.rs` (nuevo) ← test integration D19.
- `docs/api/MCP.md` ← agrega sección de los 6 tools.

**Veredicto de impacto:** ✅ bajo — 3 archivos modificados por append (tools.rs, config.rs, lib.rs) + 1 nuevo (skills.rs) + 1 nuevo tests + 1 doc. Ningún archivo existente cambia comportamiento. Sin deps nuevas. WASM-safe.

## Steps

### Step 1: Config knobs + registro de módulo
- **Archivos:** `vantadb-mcp/src/config.rs`, `vantadb-mcp/src/lib.rs`
- **Acción:** Agregar a `McpConfig`: `max_skill_resource_bytes` (default 5_000_000), `max_skill_total_bytes` (default 50_000_000) — constantes TDAM como bounds configurables (patrón MEM-02). Registrar `mod skills;` en lib.rs.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ DONE (config.rs + lib.rs)

### Step 2: `skills.rs` — definiciones de los 6 tools + dispatch + validación base
- **Archivos:** `vantadb-mcp/src/skills.rs` (nuevo)
- **Acción:** Módulo con: constantes `FILE_META_PREFIX = "file:"`; `skill_tool_definitions() -> Vec<Value>` (6 defs JSON Schema fieles a TDAM + param `owner_agent`); `handle_skill_tool(name, args, storage, config) -> Result<Value, Value>` (match interno); helpers: `require_str(args, key)`, `require_u64(args, key)`, `require_bool`, `opt_str`, `skill_not_found()` (error_content idéntico para no-existe y owner mismatch), `require_owned(head, owner)`, `validate_skill_size`.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ DONE (skills.rs — defs + dispatch + helpers + 6 handlers)

### Step 3: Reads — `skill_list` + `skill_view`
- **Archivos:** `vantadb-mcp/src/skills.rs`
- **Acción:** `skill_list(owner_agent, name_prefix?, limit?, offset?)` → `SkillStore::list` con `SkillListOptions` (owner requerido = scope); response `[{skill_id, name, description, version}]` + `total`. `skill_view(skill_id, owner_agent, version?)` → `get_head`/`get_version`; **owner check → 404**; response `{skill_id, version, name, description, content, files:[{path, content, encoding, mime_type, is_executable, size_bytes}]}` (files desde metadata `file:`).
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ DONE (incluido en Step 2 module)

### Step 4: Writes — `skill_create` + `skill_update`
- **Archivos:** `vantadb-mcp/src/skills.rs`
- **Acción:** `skill_create(name, owner_agent, content, description?, metadata?, ttl_secs?)` → `SkillCreateInput`; valida tamaño total ≤ max_skill_total_bytes; response `{ok, skill_id, version, idempotent}`. `skill_update(skill_id, owner_agent, expected_version, content, description?)` → owner check 404 → `SkillUpdateInput` (description = arg o head) → `store.update` (expected_version = optimistic lock → `ExecutionConflict` → error_content 409-style); response `{ok, version, idempotent}`.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ DONE (incluido en Step 2 module)

### Step 5: `skill_patch` (substring) + `skill_files_write` (límites + path)
- **Archivos:** `vantadb-mcp/src/skills.rs`
- **Acción:** `skill_patch(skill_id, owner_agent, expected_version, old_string, new_string, replace_all?)`: old_string vacío → error; count==0 → error; count>1 && !replace_all → error; replace/replacen sobre content del head → `store.patch(content: Some)`. `skill_files_write(skill_id, owner_agent, expected_version, path, content, encoding?, mime_type?, is_executable?)`: `assert_path` (no vacío/absoluto/NUL/`..`); decoded size ≤ max_skill_resource_bytes; owner check 404; agregado (content + Σ files - reemplazado + nuevo) ≤ max_skill_total_bytes; metadata[`file:{path}`] = JSON record; `store.patch(metadata: Some)`.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ DONE (incluido en Step 2 module)

### Step 6: Registrar en `tools.rs` (list + dispatch)
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** En `handle_tools_list` agregar los 6 tool objects de `skills::skill_tool_definitions()`; en `handle_tools_call` agregar arm `"skill_list" | "skill_view" | "skill_create" | "skill_update" | "skill_patch" | "skill_files_write" => crate::skills::handle_skill_tool(name, args, storage, config)`.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ DONE (base_tools vec + extend + dispatch arm)

### Step 7: Tests D19 — `vantadb-mcp/tests/skills_tests.rs`
- **Archivos:** `vantadb-mcp/tests/skills_tests.rs` (nuevo)
- **Acción:** Tests con `setup_storage()` + `handle_tools_call` (patrón mcp_tests.rs):
  1. tools/list incluye los 6 skill tools.
  2. create → view roundtrip (owner match).
  3. list scope por owner (no muestra skills de otro owner).
  4. update con expected_version → v2; stale → conflict; idempotente (mismo content) → sin bump.
  5. patch substring: replace único; count>1 sin replace_all → error; con replace_all → ok.
  6. files_write: write+view (file en manifest); 5MB limit; 50MB total limit; path traversal `../` / absoluto / NUL → error.
  7. **Owner check → 404 sin filtrar existencia:** view/update de skill de owner A con owner B → error idéntico al de skill inexistente (mismo texto).
  8. Paridad API nativa (D13): create vía MCP → `SkillStore::get_head` core devuelve el mismo record; update vía core → view vía MCP muestra head nuevo.
  9. create idempotente (mismo content+name) → mismo skill_id, idempotent=true, sin v2.
- **Verify:** `cargo nextest run -p vantadb-mcp -- skills` → 10/10 ✅
- **Estado:** ✅ DONE

### Step 8: Verify full + docs + commit
- **Acción:** Verify full: `cargo fmt --check` + `cargo clippy -p vantadb-mcp -- -D warnings` + `cargo nextest run -p vantadb-mcp` (13/13) + `cargo test -p vantadb-mcp --test mcp_tests` (44/44, binary excluido de nextest) + `scripts/validate-docs-coverage.ps1` (vantadb-mcp 15/15 ok). Docs: `docs/api/MCP.md` — sección Skill Operations (Regla 3). Commit conventional con task ID: `feat(mcp): tools skill_* sobre SkillStore con owner check (MEM-07)`.
- **Estado:** ✅ DONE (commit <hash>)

## Dependencias
- MEM-06 (SkillStore + tipos) ✅ commit `92cf709f`
- MEM-02 (patrón tools MCP) ✅ commit `32b09daf`
- MEM-05 (auth 3 capas — contexto; MCP embebido sin HTTP auth, identity = param owner_agent) ✅ commit `01a5de66`
- TDAM fuente: `skill-tools.ts`, `skill-permission.ts`, `skill-resource-store.ts` @ `97f9465`

## Notas
- TDAM separa SKILL.md (DB) de resources (files/) — **NO portar el split**: VantaDB MEM-06 modela skill = content + metadata; los recursos viven como entries `file:{path}` en metadata (JSON record). Misma semántica observable (manifest en view, límites 5MB/50MB, optimistic lock), cero lógica nueva en core.
- TDAM `skill_patch` opera substring sobre SKILL.md — port fiel: mismo wire, traducción a `SkillPatchInput.content` en el MCP.
- Owner check unificado en 404 (TDAM `assertTeamMatch`): el caller nunca distingue "no existe" de "no es tuyo" — previene side-channel de existencia (SKILL_TEAM_MISMATCH → NOT_FOUND en TDAM §3.6).
- Error mapping: params inválidos → `Err(McpError::invalid_params(...).to_json())` (JSON-RPC error, patrón existente); errores de dominio (not found/conflict/limits/path) → `Ok(error_content(...))` para que el LLM self-correct (patrón TDAM jsonError).
- `expected_version` (u64) requerido en update/patch/files_write — optimistic lock del core (`ExecutionConflict`).
- FASE SECURITY: input del LLM es trust boundary (MCP) → validación en boundary (longitudes, tamaños, path traversal, base64 decode), owner check en cada read/write, sin secrets, sin logs de contenido.

## Context Save Point
- **Fecha:** 2026-08-20T16:30
- **Branch:** develop
- **CI pendiente:** no
- **Decisiones:**
  - Identity del MCP embebido = param `owner_agent` (requerido en los 6 tools) — el server stdio no tiene HTTP auth; contrato igual a TDAM router.
  - Recursos en metadata `file:{path}` (JSON record con size_bytes explícito) — sin store nuevo en core; accounting por size_bytes sin re-decodificar.
  - Límites como knobs McpConfig (defaults TDAM 5MB/50MB), no hardcodeados en skills.rs.
  - patch substring traducido a content completo — el core no conoce old_string.
- **Problemas conocidos:** ninguno.
- **Próxima tarea:** MEM-35
