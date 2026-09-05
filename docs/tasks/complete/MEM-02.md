# MEM-02: Exponer search_profile en el tool MCP search_memory (passthrough a VantaMemorySearchRequest)

## Metadata
- **Plan file:** docs/plans/2026-08-18-vanta-memory.md
- **Creado:** 2026-08-20
- **last-synced:** 2026-08-20
- **Estado:** ✅ COMPLETED
- **Branch:** develop (HEAD b1130cae)

## Blast Radius
- `vantadb-mcp/src/handlers/tools.rs` — schema del tool `search_memory` (~93-108) + handler (~440-546). HOY hardcodea `search_profile: None` en línea 538. Cambio principal.
- `vantadb-mcp/src/validation.rs` — helpers `validate_identifier`/`validate_payload`/`validate_vector` (11/37/47). Agregar `validate_search_profile` (bounds rrf_k/candidate_k, mode por enum serde).
- `vantadb-mcp/src/config.rs` — `McpConfig` con knobs (max_top_k/max_namespace_length/etc, defaults 39-46). Agregar `max_rrf_k`/`max_candidate_k` siguiendo el patrón existente.
- `vantadb-mcp/tests/mcp_tests.rs` — tests MCP: `setup_storage()` (13), `default_config()` (9), llaman `handle_tools_call` directo. Aquí viven los tests de paridad.
- `src/cli_server.rs:1090-1100` — `SearchPageRequest` hace `#[serde(flatten)] request: VantaMemorySearchRequest` → la API nativa YA acepta `search_profile`. SIN cambios (solo se usa como referencia de paridad).
- `src/sdk/types.rs:437/452` — `SearchProfileMode` + `SearchProfileConfig` (serde lowercase, `#[serde(default)]`, ya tiene `Deserialize`) — forma de wire compartida (D13). Reutilizar con `serde_json::from_value` (cero parsing custom).
- `docs/api/MCP.md` — documentar `search_profile` en search_memory.

## Contrato
"`cargo check -p vantadb-mcp` pasa; test de paridad IQL/API/MCP (D19)"

## Pasos
### Step 1: Wire format + passthrough en handler - ✅
- DECISIÓN: `search_profile` = objeto JSON anidado `{"mode": "keyword|vector|hybrid", "rrf_k": int, "candidate_k": int}` — EXACTAMENTE la forma serde de `SearchProfileConfig` (types.rs:452). Parsear con `serde_json::from_value::<SearchProfileConfig>` (ya Deserialize + defaults) → garantiza paridad de forma con la API nativa y con la cláusula IQL `PROFILE`. `{}` o ausente → `None` (modo Hybrid + constantes core).
- tools.rs schema: agregar a `search_memory` inputSchema: `"search_profile": { "type": "object", "properties": { "mode": {"type":"string","enum":["keyword","vector","hybrid"]}, "rrf_k": {"type":"number"}, "candidate_k": {"type":"number"} }, "description": "SearchProfileConfig opcional (MEM-01): mode fuerza canal (keyword/vector/hybrid), rrf_k y candidate_k ajustan RRF" }`.
- tools.rs handler: tras parsear filters (o antes, da igual), `let search_profile = match args.get("search_profile") { Some(Value::Object(obj)) => Some(validate_search_profile(obj, config).map_err(|e| e.to_json())?), Some(_) => return Ok(error_content("search_profile must be an object ...")), None => None };` y reemplazar `search_profile: None,` por `search_profile,`.
- Detalle: `args.get("search_profile")` → None si ausente; valor no-objeto → error claro sin panic.

### Step 2: Validación en trust boundary - ✅
- validation.rs: `validate_search_profile(obj: &Map<String,Value>, config: &McpConfig) -> Result<SearchProfileConfig, McpError>`:
  - `serde_json::from_value::<SearchProfileConfig>(Value::Object(obj.clone()))` → mode inválido = error serde claro (McpError::invalid_params con prefijo "search_profile:").
  - Bounds: `rrf_k` en `1..=config.max_rrf_k`, `candidate_k` en `1..=config.max_candidate_k` (0 o > max → error). Justificación: candidate_k gigante puede inflar memoria (trust boundary, mismo patrón que MCP-04 dim check); rrf_k=0 = fusión RRF degenerada.
- config.rs: `max_rrf_k: usize` default 100, `max_candidate_k: usize` default 10_000, siguiendo el patrón max_top_k.
- Test unitario `validate_search_profile_parses_and_bounds` (validation.rs tests): parse válido, empty→hybrid defaults, mode bogus, rrf_k=0, candidate_k>max.
- Nota de diseño: los bounds solo limitan los valores explícitos del tool; un profile ausente siempre pasa (None → constantes core).

### Step 3: Tests de paridad + validación - ✅
- mcp_tests.rs, 3 tests nuevos (patrón seed de `test_mcp_tool_search` línea 547):
  1. `test_search_profile_mcp_passthrough_parity_with_native` — seed 3 records vía memory_put MCP. (a) profile explícito {hybrid, rrf_k 30, candidate_k 64}: MCP vs nativo `VantaEmbedded::from_engine(storage.clone()).search(mismo request)` → MISMO set de keys en orden + scores (<1e-4). (b) sin profile en ambos → idénticos (defaults MEM-01).
  2. `test_search_profile_mode_force_channels` — keyword → solo "a" (excluye vector-only "b"); vector → ["b","a"] (orden puramente vectorial, texto ignorado) == control vector-only. Espeja las aserciones del test core MEM-01 (tests.rs:968/1041).
  3. `test_search_profile_validation_errors` — mode bogus, rrf_k=0, candidate_k>max → error claro sin panic; tipo no-objeto → error "must be an object".
- HALLAZGO: `StorageEngine::open` (setup_storage) deja el text_index_state ausente → `text_query` falla "text_index not found: bm25" (AUD-044). Fix en tests: `VantaEmbedded::from_engine(storage.clone()).rebuild_index()` tras el seed (misma advertencia que el mensaje de error: "reopen writable or run rebuild_index").
- PARIDAD IQL: no se aserta igualdad exacta MCP↔IQL porque los motores difieren por diseño: IQL texto = scan + `text_contains_query` (substring, filter.rs:140), SDK = BM25 postings (lexical.rs:23). La paridad D19 se cumple como: (a) passthrough MCP↔API exacto (mismo struct), (b) selección de canal por `mode` idéntica en los 3 (IQL vía planner MEM-01, API/MCP vía SDK). Documentado en comentario del test y en Notas.

### Step 4: Docs + verify full - ✅
- docs/api/MCP.md: agregado `search_profile` a la doc del tool search_memory (shape + bounds + defaults).
- Verify: `cargo fmt` aplicado; `cargo clippy -p vantadb-mcp --all-targets` limpio; `cargo clippy -p vantadb --all-targets` limpio; `cargo test -p vantadb-mcp` 44/44; `cargo test -p vantadb --lib search_profile` 11/11 (regresión MEM-01 OK).

### Step 5: Commit - ✅ (preparado; ejecutado en esta sesión)
- Conventional commit: `feat(mcp): search_profile passthrough en tool de búsqueda (MEM-02)` — archivos: config.rs, handlers/tools.rs, validation.rs, tests/mcp_tests.rs, docs/api/MCP.md, tasks/MEM-02.md (mover a complete/). NO incluye los task files pre-modificados (AUD-047..PERF-02 — ajenos).

## Dependencias
- MEM-01 (commits 6a50b8ee + b113ede0): `SearchProfileConfig`/`SearchProfileMode` + cláusula IQL `PROFILE` — ya entregado. Sin esto el passthrough no tiene forma de wire que compartir (D13).

## Notas
- MEM-01 solo tocó MCP con `+ search_profile: None` (tools.rs:538) — el passthrough real es TODO de MEM-02.
- La API nativa (`/api/v2/search` vía SearchPageRequest flatten) YA acepta search_profile — MEM-02 no la toca; el test la usa como referencia de paridad.
- `vantadb-python/src/lib.rs` (995/1826/1937) también hardcodea `search_profile: None` — FUERA de scope MEM-02 (task separada); anotar como follow-up.
- Al cerrar: actualizar estado MEM-02 en docs/plans/2026-08-18-vanta-memory.md (⏳ → ✅) + mover este archivo a tasks/complete/.