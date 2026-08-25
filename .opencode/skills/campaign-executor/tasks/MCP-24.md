# MCP-24: search_with_method + search_multi/all como tools MCP

## Metadata
- **Plan file:** docs/plans/2026-08-25-batch-core-server-mcp.md (Task 11)
- **Fuente:** docs/Backlog.md fase P25 → fila MCP-24
- **Esfuerzo:** 🟡 1d (real ~2h)
- **Prioridad:** 🟡
- **Tipo:** Rust (MCP)
- **Turns estimados:** 8
- **Creado:** 2026-08-25
- **Estado:** ⬜ PENDING
- **Incógnitas (uphill):** 0
- **Pendientes (downhill):** 6 steps

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/lib.rs:39,41` (re-export), `vantadb-mcp/src/server.rs:275,292` (dispatch) |
| Callees | `vantadb::sdk::VantaMemorySearchRequest`, `vantadb::VantaEmbedded::search/search_with_method/search_multi` (`src/sdk/search/mod.rs:69,84`, `src/sdk/search/multi.rs:20`) |
| Implicaciones | No cambia firma pública de `handle_tools_call/list` (solo agrega arms). Sin cambios SDK (wrappers only). Añade 2 tools al catálogo. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):**
  - `vantadb-mcp/src/handlers/tools.rs` (1772 líneas, completo)
  - `src/sdk/search/mod.rs` (380, completo — `search` :69, `search_with_method` :84)
  - `src/sdk/search/multi.rs` (89, completo — `search_multi` :20, `search_all` :76)
  - `vantadb-mcp/src/config.rs` (71 — `McpConfig`: max_top_k, max_vector_dim, max_namespace_length)
  - `vantadb-mcp/tests/mcp_tests.rs` (completo — patrón round-trip)
  - `skills/vantadb-mcp/SKILL.md` + `.opencode/skills/vantadb-mcp/SKILL.md` (527, hash SAME actual C287…)
  - `docs/api/MCP.md` (tabla tools)
  - `src/index/mod.rs:27` (`IndexType`: Hnsw/Ivf/Flat/DiskAnn/Scann, público)
- **Referenciados hacia dentro:** `use crate::validation::*` (text_content/error_content/serialize_content/validate_identifier/validate_vector/parse_filter_ops/parse_metadata/validate_search_profile), `crate::config::McpConfig`, `crate::error::McpError`, `vantadb::index::IndexType`, `vantadb::DistanceMetric`
- **Referencias entrantes (a los editados):** `handle_tools_call`/`handle_tools_list` re-exportados en `lib.rs` y despachados en `server.rs` — firmas intactas, no se rompe nada.
- **Veredicto impacto:** bajo — refactor interno de un arm + 2 arms nuevos + 2 helpers privados + tests + docs. Ninguna API pública del crate cambia de firma.

## Contrato
"`cargo test -p vantadb-mcp --test mcp_tests` pasa; tools `search_with_method` y `search_multi` round-trip (aparecen en tools/list, ejecutan búsqueda y devuelven hits, errores claros); docs skill ×2 hash SAME."

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** los canales de error de `search_memory` existentes se conservan exactamente (param errors → JSON-RPC Err; domain errors dim/filter-range → `Ok(error_content)`, requerido por `test_mcp_search_filters_accept_eq_and_reject_range`). Las 2 copias de SKILL.md quedan byte-idénticas (hash SAME). No se toca `src/sdk/search/*` (wrappers only).
- **Comandos de verificación:** `cargo check -p vantadb-mcp` ✅ · `cargo fmt --check` ✅ · `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅ · `cargo test -p vantadb-mcp --test mcp_tests` ✅ · `Get-FileHash skills/vantadb-mcp/SKILL.md` == `Get-FileHash .opencode/skills/vantadb-mcp/SKILL.md`
- **Deuda pendiente:** ninguna (salvo count de tools del SKILL.md que venía desfasado y se corrige).

## Deuda técnica (Regla 6)

**Sin deuda nueva.** Al contrario: corrige el conteo desfasado de tools del SKILL.md (57→68) que ya estaba mal — saldo neto ≤ 0.

## Definition of Done

| Nivel | Gate |
|-------|------|
| Task | Contrato del task file (mcp_tests pasa + round-trip + hash SAME) |
| Commit | convencional `feat(mcp):` — lo prepara el worker, lo ejecuta el lead (regla: sub-agentes NO commitean) |
| Release | N/A (no release; verify mecánico del lead) |

## Fases explícitas — SECURITY | PERFORMANCE

- [x] **SECURITY** — toca trust boundary (input de usuario MCP): validar top_k (clamp `max_top_k`), dims vectoriales contra índice (`index_vector_dim`), identifiers de namespace/namespaces (`validate_identifier`), método de backend (enum whitelist). Reutiliza el mismo patrón de validación de `search_memory`.
- [ ] **PERFORMANCE** — N/A (wrappers finos; no toca hot paths del core, sin loops nuevos en SDK). Los arrays de `namespaces` no se materializan más allá de lo necesario (Vec<&str>). Justificado en Notas.

## Steps

### Step 1: Task file + baseline verify
- **Archivos:** `.opencode/skills/campaign-executor/tasks/MCP-24.md`
- **Acción:** crear task file con Impacto mapeado (Regla 0) y correr baseline `cargo check -p vantadb-mcp` para confirmar estado limpio antes de editar.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ COMPLETED (baseline limpio)

### Step 2: Extraer parse_search_request compartido + helpers
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** extraer el parsing del arm `search_memory` a `fn parse_search_request(namespace, args, config, storage) -> Result<ParsedSearchRequest, Value>` (enum `Ready(Request)` / `Rejected(envelope)` para preservar canales); agregar `fn parse_search_method(val) -> Result<Option<IndexType>, Value>`.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ COMPLETED (check verde)

### Step 3: Refactor search_memory arm + 2 tools defs + 2 arms nuevos
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`
- **Acción:** refactorar arm `search_memory` a usar el parser; agregar defs de tool `search_with_method` y `search_multi` en `base_tools` (schema JSON-RPC) y arms que llaman `search_with_method`/`search_multi` del SDK.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ COMPLETED (check verde; defs en :183/:206, arms en :906/:928)

### Step 4: Tests round-trip
- **Archivos:** `vantadb-mcp/tests/mcp_tests.rs`
- **Acción:** agregar `test_mcp_tools_list_includes_advanced_search`, `test_mcp_search_with_method_round_trip` (hnsw=flat mismo top hit, método inválido→error), `test_mcp_search_multi_round_trip` (merge cross-namespaces, top_k cap, empty namespaces→-32602).
- **Verify:** `cargo test -p vantadb-mcp --test mcp_tests`
- **Estado:** ✅ COMPLETED (3 tests verdes en aislamiento y en suite completa)

### Step 5: Docs — SKILL.md x2 hash SAME + docs/api/MCP.md
- **Archivos:** `skills/vantadb-mcp/SKILL.md`, `.opencode/skills/vantadb-mcp/SKILL.md`, `docs/api/MCP.md`
- **Acción:** documentar `search_with_method` y `search_multi` en Search Operations del SKILL.md (mismo contenido en ambas copias), corregir conteo de tools (57→68, core 36→38, otros 24→30); agregar filas a tabla Search & Query de `docs/api/MCP.md` (3→5).
- **Verify:** hashes de ambas copias SKILL.md idénticos; `Get-FileHash` SAME
- **Estado:** ✅ COMPLETED (hash SAME D0C2…; docs/api/MCP.md actualizado)

### Step 6: Verify full + recitation
- **Archivos:** — (verify)
- **Acción:** `cargo check`, `cargo fmt --check`, `cargo clippy -p vantadb-mcp --all-targets -- -D warnings`, `cargo test -p vantadb-mcp --test mcp_tests`; verificar hash SAME.
- **Verify:** todos los comandos verdes + hash SAME
- **Estado:** ✅ COMPLETED (check ✅, fmt ✅, clippy ✅, mcp_tests 68/68 ✅, hash SAME ✅)

## Dependencias
- Ninguna (independiente; wave 2).

## Review (GATE — agente distinto, P2-01)
- **Revisor:** (lo delega el lead — `vanta-review`/`vanta-audit`)
- **Enfoque:** pendiente — el parser compartido con enum no debe alterar canales de error de `search_memory`.
- **Cómo se probó:** pendiente — evidencia mecánica (mcp_tests + check + clippy).
- **Veredicto:** ⬜ pendiente

## Notas
- `parse_search_request` se comparte entre los 3 tools (no duplicar parsing, requisito de la tarea). El enum `ParsedSearchRequest::Rejected` preserva el canal `Ok(error_content)` que exige el test de filtros `$gt` (MEM-32) y el dim-mismatch, mientras los param errors siguen siendo JSON-RPC Err.
- `search_multi` en el SDK recibe un solo request + array de namespaces (ignora `request.namespace`); el wire format es `{namespaces:[...], query_vector, ...}`. Resultado = array plano de hits fusionados.
- `search_with_method.method` acepta `hnsw|ivf|flat|diskann|scann` (enum `IndexType`); omisión = routing automático.
- FASE SECURITY: top_k clamp a `max_top_k`, dims validadas contra índice, identifiers validados, method whitelist, namespaces no vacíos.
- PERFORMANCE: N/A — wrappers; no hot paths.
- Cambio menor de canal: antes el dim-mismatch en `search_memory` iba por `Ok(error_content)`; ahora (refactor) se conserva IGUAL vía `Rejected`. Sin cambio neto.

## Recitation (canónico — estructura única)

- **activeGoal:** MCP-24: exponer search_with_method y search_multi como tools MCP
- **lastAction:** Steps 1-5 completos: parser compartido `parse_search_request` (enum `ParsedSearchRequest::Ready/Rejected`) + `parse_search_method`; arms search_memory/search_with_method/search_multi; defs de tools; 3 tests round-trip; docs SKILL.md x2 (hash SAME D0C2…) + docs/api/MCP.md. Step 6: verify full verde.
- **result:** OK
- **nextAction:** ninguno — tarea completa; el lead verifica mecánico y commitea
- **contract:**
  - verificacion: `cargo test -p vantadb-mcp --test mcp_tests` ✅ 68/68; `cargo check -p vantadb-mcp` ✅; `cargo fmt --check` ✅; `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` ✅; hash SAME SKILL.md ✅
  - evidencia:
    - claim: search_with_method (:84) y search_multi (multi.rs:20) existen en SDK sin tool MCP
      evidencia: src/sdk/search/mod.rs + multi.rs (leídos completos)
      confianza: alta
    - claim: los 3 tests MCP-24 round-trip pasan y aparecen en tools/list
      evidencia: cargo test -p vantadb-mcp --test mcp_tests (test_mcp_tools_list_includes_advanced_search, test_mcp_search_with_method_round_trip, test_mcp_search_multi_round_trip)
      confianza: alta
    - claim: docs skill x2 byte-idénticas
      evidencia: Get-FileHash SHA256 = D0C2DE2EFDFE410D5CF207AE96B592D749C7F8D8791CED81943B4E4F0B389C50 en ambas copias
      confianza: alta
  - artefactos: vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, skills/vantadb-mcp/SKILL.md, .opencode/skills/vantadb-mcp/SKILL.md, docs/api/MCP.md, task file
  - invariantes: canales de error de search_memory intactos (Rejected → Ok(error_content), param → Err JSON-RPC; verificado por test_search_profile_validation_errors + test_mcp_search_filters_accept_eq_and_reject_range que pasan); SKILL.md x2 hash SAME; NO se tocó src/sdk/search/*
  - deuda: ninguna propia. Colateral observado durante la ejecución: test_mcp_memory_supersede_round_trip (MOD-10, tool supersede en el MISMO tools.rs, sesión concurrente) fallaba temporalmente con `superseded_by: None` esperado Some("beta") — resuelto por MOD-10 durante la sesión (suite final 68/68). Otro agente (MOD-10) edita el mismo tools.rs/mcp_tests.rs en paralelo — el lead debe mergear.
  - queda_pendiente: el lead verifica mecánico y commitea (regla: sub-agentes NO commitean) — archivos: tools.rs, mcp_tests.rs, SKILL.md x2, docs/api/MCP.md, task file. Coordinar con el commit de MOD-10 (mismos archivos).
- **nextTask:** ninguna (única tarea de esta invocación)
