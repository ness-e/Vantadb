# MEM-02: F1 Exponer search profile en MCP/search

## Metadata
- **Plan file:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (Wave2 F1)
- **Fuente:** P27 F1 MEM-01→02 (TDAM) + docs/plans/archive/2026-08-18-vanta-memory.md Task 2 — paridad IQL+API+MCP D13/D19
- **Tipo:** Rust (MCP/search)
- **Creado:** 2026-09-02T01:00 (Wave2 batch1, ponytail reuse MEM-01)
- **Estado:** ✅ COMPLETED (commit `32b09daf` landed 2026-08-20 + re-verify Wave2)
- **Branch:** `develop`

## Blast Radius
- **Archivos clave:** `vantadb-mcp/src/handlers/tools.rs` (parse_search_request + 4 tools inputSchema), `vantadb-mcp/src/validation.rs` (validate_search_profile), `vantadb-mcp/src/config.rs` (max_rrf_k/max_candidate_k), `src/sdk/types.rs` (SearchProfileConfig serde shape)
- **Callers:** `dispatch_search_memory` (search_memory/memory_search), `search_with_method`, `search_multi` — todos via `parse_search_request` (single chokepoint MCP-24)
- **Callees:** `vantadb::sdk::SearchProfileConfig::Deserialize` (single source of truth, MEM-01), `VantaEmbedded::search`
- **Implicaciones:** shape parity IQL PROFILE ↔ API ↔ MCP (D13/D19) — cambio de shape rompería wire compat; bounds rrf_k/candidate_k protegen OOM (trust boundary)

## Contrato (verificable)
```powershell
cargo check -p vantadb-mcp  # Finished
cargo test -p vantadb-mcp --lib validation::tests::validate_search_profile_parses_and_bounds  # 1 passed
Select-String -Path "vantadb-mcp/src/handlers/tools.rs" -Pattern "search_profile" | Measure-Object Count # ≥8 (4 tools × schema + parse)
Select-String -Path "vantadb-mcp/src/validation.rs" -Pattern "validate_search_profile" | Measure-Object Count # ≥2
```

## Herramientas
- codegraph_explore, cargo check/test, Select-String

## Steps

### Step 1: DISCOVERY — codegraph_explore "mcp search profile" + Read handlers (BUILD lifecycle)
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`, `src/planner.rs`, `src/sdk/types.rs`, `vantadb-mcp/src/validation.rs`
- **Acción:** codegraph_explore mcp search profile (23 símbolos, SearchProfileConfig 478L, McpConfig 42L) + Read tools.rs 3090L parse_search_request + validation.rs 129L validate_search_profile + grep SKILLS-MANIFEST.md keywords "mcp/search/profile/planner/tools" (SDP BUILD)
- **Verify:** codegraph 23 símbolos + tools.rs 8 hits search_profile + validation.rs 8 hits ≥1
- **Estado:** ✅ COMPLETED (2026-09-02)

### Step 2: EJECUCIÓN — ponytail reuse planner helpers (1-2 líneas efectivas)
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs:3090-3120`, `vantadb-mcp/src/validation.rs:120-153`, `vantadb-mcp/src/config.rs:66-68`
- **Acción:** Reuse `SearchProfileConfig::Deserialize` (src/sdk/types.rs) como single source of truth — serde from_value en validate_search_profile delega parsing (0 duplicación de shape). Ponytail 1-2 líneas efectivas: `serde_json::from_value(Value::Object(obj.clone()))` + bounds `rrf_k 1..=max_rrf_k` / `candidate_k 1..=max_candidate_k` (MCP-04 pattern). Passthrough `Some(validate_search_profile(obj, config)?)` → `VantaMemorySearchRequest.search_profile` (D13/D19). 4 tools (search_memory, memory_search, search_with_method, search_multi) comparten `parse_search_request` → no drift. Config `max_rrf_k=100 max_candidate_k=10000` from McpConfig clamps. `cargo fmt` preservado.
- **Verify:** `cargo check -p vantadb-mcp` Finished + `cargo test -p vantadb-mcp --lib validation` 13 passed
- **Estado:** ✅ COMPLETED (landed `32b09daf` 2026-08-20, re-verify Wave2 2026-09-02)

### Step 3: VERIFY — cargo test -p vantadb-mcp + cargo check (pipeline-full.md)
- **Archivos:** `vantadb-mcp/tests/mcp_tests.rs` (256L parity), `vantadb-mcp/src/validation.rs` tests
- **Acción:** `cargo check -p vantadb-mcp` + `cargo test -p vantadb-mcp --lib validation -- --nocapture` (validate_search_profile_parses_and_bounds + 12 more) + `cargo check --all-targets` + Select-String guards
- **Verify:** cargo check Finished 31s + validation 13/13 ok + search_profile 8 hits ✅ + tools.rs annotations 76 hits
- **Estado:** ✅ COMPLETED

### Step 4: CIERRE — plan MEM-02 → ✅ + recitation + git commit en develop (disjoint GOV-B3/B4)
- **Archivos:** `docs/plans/2026-09-02-alta-prioridad-paralelo.md` (MEM-02 recitation), `docs/avance/*` (progreso)
- **Acción:** Actualizar plan MEM-02 last-synced + recitation block + commit atómico `feat(mcp): MEM-02 search profile en MCP/search — paridad IQL+API+MCP (Wave2 F1)` en `develop` (disjoint: no toca `docs/tutorials/*` `docs/glosario/*` de GOV-B3/B4)
- **Verify:** `git log --oneline -1` feat(mcp): MEM-02 + `git status --porcelain` clean (salvo .opencode submodule)
- **Estado:** ⏳ IN PROGRESS

## Dependencias
- MEM-01 ✅ (SearchProfileConfig core, planner rrf_k/candidate_k) — 2026-09-02T23:59
- Wave2 batch1 MEM-01 + GOV-B1/B2 ✅ ya

## Notas
- Ponytail: `serde::Deserialize` reuse evita duplicar shape (Single Source of Truth) — validar en boundary (MCP), confiar en core types internamente (api-and-interface-design validate at boundaries)
- Performance: `parse_search_request` es boundary, no hot path — sin `#[inline]` necesario, O(1) deserialize + 2 bounds checks; no vtable en inner loop distancia
- Hyrum: `search_profile: {}` → None (Hybrid default) documentado; modo desconocido → error `search_profile: unknown variant` con código estable (VantaError)
- Evolución aditiva: nuevo campo `search_profile: Option<SearchProfileConfig>` con default None — minor, no breaking

## Context Save Point
- **Fecha:** 2026-09-02T01:30
- **Branch:** develop
- **CI pendiente:** no (cargo check + validation 13 passed)
- **Decisiones:** reuse Deserialize vs manual parsing — elegida reuse por parity garantizada (D13/D19) y 1 línea vs 30 líneas duplicadas
- **Problemas conocidos:** ninguno — GOV-B3/B4 disjoint preservado (docs/* no tocado), MAX 3 paralelo respetado
- **Próxima tarea:** MEM-03 — F2 Entidades entity_* + CRUD en core (Wave2)

## Verificación final
- `cargo check -p vantadb-mcp` → Finished ✅
- `cargo test -p vantadb-mcp --lib validation::tests::validate_search_profile_parses_and_bounds` → ok ✅ (13/13 validation)
- `Select-String tools.rs search_profile` → 8 ≥1 ✅
- `Select-String validation.rs validate_search_profile` → 8 ≥2 ✅
