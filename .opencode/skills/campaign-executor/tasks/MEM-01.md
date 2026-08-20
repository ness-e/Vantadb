---
id: MEM-01
campaign: vanta-memory
title: "SearchProfileConfig per namespace (core) + cláusula IQL profile"
status: pending
estimate: "15-30 turns"
owner: vanta-worker
type: rust
---

# MEM-01 — SearchProfileConfig per namespace (core SDK + IQL)

## Contexto (del lead)

Los usuarios necesitan ajustar la fusión híbrida por consulta/namespace: hoy `RRF_K=60` y `hybrid_candidate_budget` son constantes globales en `src/planner.rs`. El report ya expone `rrf_k` (D20) pero hardcodea la constante. Se agrega un perfil de búsqueda configurable por request:

- Nuevo tipo `SearchProfileConfig` (NO colisionar con `SearchProfile` del profiler I/O — D14): `{ mode: keyword|vector|hybrid, rrf_k: Option<usize>, candidate_k: Option<usize> }`, `None` = usar constantes core.
- Campo `#[serde(default)] search_profile: Option<SearchProfileConfig>` en `VantaMemorySearchRequest` (retrocompat JSON).
- Parametrizar `fuse_rrf`/`fuse_rrf_many`/`fuse_rrf_with_report`/`hybrid_candidate_budget` de planner.rs.
- Cláusula IQL opcional `PROFILE <mode> [rrf_k <n>] [candidate_k <n>]` para que Studio la use sin cambios de bridge (D13).

## Contrato (DoD)

- [ ] `cargo check -p vantadb` pasa.
- [ ] Tests dedicados de `SearchProfileConfig`: serde roundtrip, defaults, mode routeo (keyword/vector/hybrid), rrf_k/candidate_k efectivos (D19).
- [ ] Tests del parser IQL: cláusula `PROFILE` parsea, valida mode, rechaza mode inválido; sin cláusula → None (D19).
- [ ] Sin cambio de comportamiento con profile ausente (mode default Hybrid, valores default core).
- [ ] No colisiona con `SearchProfile` (src/index/search/profile.rs).
- [ ] Commit local convencional con `MEM-01`; **sin push** (vanta-lead lo ejecuta).

## Steps

- [x] **S1 — Tipos**: `SearchProfileConfig` + `SearchProfileMode` en `src/sdk/types.rs` (junto a `VantaHybridFusionReport`) con serde defaults (`mode=Hybrid`, `rrf_k=None`, `candidate_k=None`). Campo `search_profile` en `VantaMemorySearchRequest` (`src/sdk/serialization/vector_types.rs`, tras `exclude_superseded`) + actualizar `impl Default` (línea 37) + tests roundtrip/defaults en vector_types.rs. Verify: `cargo check -p vantadb`. ✅ (12/12 tests)
- [x] **S2 — Migrar struct literals completos** (~18; los que usan `..Default::default()`/spread NO se rompen): `src/sdk/serialization/vector_types.rs` (Default impl + tests 96/116), `src/cli_handlers/search.rs` (53/398/455), `tests/memory_api.rs` (378/401/474), `tests/sdk_serialization.rs` (84), `tests/proptest_serialization_roundtrip.rs` (279), `tests/prefetch_benchmark.rs` (30/49), `vantadb-mcp/src/handlers/tools.rs` (528), `vantadb-wasm/src/lib.rs` (986/1044), `vantadb-python/src/lib.rs` (985/1815). Agregar `search_profile: None,`. Verify: `cargo check --workspace --all-targets` (el compilador marca los que falten). ✅ (check workspace verde; prefetch corrigió CRLF)
- [x] **S3 — Parametrizar planner**: cambiar firmas a `fuse_rrf(lexical, vector, rrf_k: f32)`, `fuse_rrf_many(channels, rrf_k: f32)`, `fuse_rrf_with_report(lexical, vector, rrf_k: f32)` (report.rrf_k = valor efectivo), `hybrid_candidate_budget(top_k, candidate_k: Option<usize>)`, `apply_rrf_contributions(..., rrf_k)`. Helper `pub fn resolve_search_profile(req: &VantaMemorySearchRequest) -> (f32, Option<usize>)` en planner.rs (rrf_k efectivo + candidate_k). Actualizar callers en `src/sdk/search/{mod,hybrid,debug_ops,explain}.rs` (~15 sites) + tests de planner (líneas 443-620). Verify: `cargo check -p vantadb` + `cargo nextest run -p vantadb --lib`. ✅ (1814/1814; `hybrid_search` ganó params `rrf_k`/`candidate_k`; `SearchProfileConfig`/`SearchProfileMode` re-exportados en `sdk/mod.rs`)
- [x] **S4 — Mode routeo en search_impl** (`src/sdk/search/mod.rs` ~99): resolver mode antes del match `(text_query, has_vector, has_sparse)` — `Keyword` → fuerza lexical-only (has_vector=false, has_sparse=false), `Vector` → fuerza vector-only (text_query=None), `Hybrid` → natural. El report rrf_k ya sale del perfil vía fuse_rrf_with_report. Verify: tests dedicados. ✅ (aplicado también en `debug_ops.rs` y `explain.rs` para consistencia)
- [x] **S5 — Tests dedicados** (en `src/sdk/search/tests.rs`, infra VantaEmbedded existente): (a) serde roundtrip con profile; (b) request sin profile → constantes core (retrocompat); (c) mode routeo: put text+vector → `Keyword` devuelve solo BM25, `Vector` solo vector; (d) candidate_k alto cambia candidatos vs clamp estándar; (e) rrf_k custom cambia el orden vs RRF_K=60. Verify: `cargo nextest run -p vantadb --lib search`. ✅ (146/146; hallazgo: `explain_memory_search` nunca llenaba `fusion_report` — arreglado el branch text+vector para exponer el report con rrf_k del perfil, D20)
- [x] **S6 — Cláusula IQL** `PROFILE <mode> [rrf_k <n>] [candidate_k <n>]` al final del query (patrón `WITH TEMPERATURE`): `parse_profile_mode` (keyword|vector|hybrid) + `opt` de rrf_k/candidate_k en `src/parser/mod.rs` (`parse_query` ~210); agregar `"PROFILE"` a `RESERVED_KEYWORDS` (línea 40); campo `search_profile: Option<SearchProfileConfig>` en `Query` (src/query.rs) y en `LogicalPlan` (query.rs:409) + `into_logical_plan`; efecto mode en `optimize_and_compile` (planner.rs ~207): `Keyword` → descartar vector_search, `Vector` → descartar text_matches. `rrf_k`/`candidate_k` se propagan al plan pero **sin efecto en el path IQL** (el CBO no fusiona RRF — deuda documentada con `ponytail:`). Tests parser (D19): parse válido, mode-only defaults, ausente → None, mode inválido queda unconsumed. Verify: `cargo check -p vantadb` + tests parser. ✅ (117/117 parser; ajuste: `parse_query` tolera tokens extra por diseño — el test de mode inválido verifica remaining, no `is_err`)
- [x] **S7 — Cierre**: verify full: `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run --profile audit --workspace --build-jobs 2`, `cargo doc -p vantadb --no-deps`. Commit convencional (sin push). Actualizar plan file MEM-01 → completed. ✅ (fmt/clippy/doc verdes; audit 1992/1992; commit `6a50b8ee` en develop, sin push)

## Impacto mapeado (Regla 0)

### Leído (verificado)
- `src/planner.rs`: `RRF_K` (:25, 5 callers), `hybrid_candidate_budget` (:94, 10 callers), `fuse_rrf` (:120), `fuse_rrf_many` (:144), `fuse_rrf_with_report` (:153, `report.rrf_k = RRF_K as usize` :164), `classify`/`SearchRoute` (:49/:78), CBO `optimize_and_compile` (:207, operadores físicos: Scan/Filter/TextFilter/VectorSearch/Sort/Project/Limit/SubqueryFilter), tests (:443-620).
- `src/sdk/search/mod.rs`: `search_impl` (:99) — validaciones, match `(text_query, has_vector, has_sparse)` (~:134-268), routeo lexical/vector/sparse/hybrid/empty.
- `src/sdk/search/hybrid.rs` (:7): budget → lexical+vector+sparse → fuse.
- `src/sdk/search/{debug_ops,explain}.rs`: callers de budget/fuse.
- `src/sdk/types.rs`: `VantaHybridFusionReport` (:436, ya tiene `rrf_k`), `VantaMemorySearchDebugReport` (:425).
- `src/sdk/serialization/vector_types.rs`: `VantaMemorySearchRequest` (:11), `impl Default` (:37), tests (:96-177).
- `src/query.rs`: `LogicalPlan` (:409), `into_logical_plan` (:420).
- `src/parser/mod.rs`: `RESERVED_KEYWORDS` (:40-71), `non_keyword_ident` (:75), `parse_query` (:210).
- `src/executor.rs` `execute_hybrid` (:152) y `src/cli_server.rs` `execute_query` (:722): path IQL.
- `desktop/src-tauri/src/connections/server_client.rs:206`: sintaxis IQL de Studio (`FROM {kind} WHERE {field} ~ "{text}" min = {n}`).
- `src/index/search/profile.rs`: `SearchProfile` (profiler I/O) — NO tocar (D14).

### Hallazgos clave
1. **Dos paths separados**: path SDK usa `fuse_rrf`/`hybrid_candidate_budget` (aquí el profile tiene efecto total). Path IQL/executor NO fusiona RRF (CBO directo sin RRF) → `rrf_k`/`candidate_k` en IQL se propagan sin efecto; `mode` SÍ tiene efecto vía descarte de operadores en el CBO.
2. **~100 struct literals** de `VantaMemorySearchRequest` en 40 archivos; ~82 usan `..Default::default()` o spread (NO se rompen); ~18 completos a migrar (lista en S2).
3. `VantaHybridFusionReport.rrf_k` ya existe — solo alimentarlo con el valor del perfil (D20).

### Deuda documentada
- `ponytail: IQL rrf_k/candidate_k propagados sin efecto hasta que el CBO fusione RRF (path SDK ya lo usa).`
- `ponytail: mode Keyword ignora sparse en path SDK (busqueda lexical pura); revisar si conviene incluir sparse lexical.`

## Herramientas

- `cargo check -p vantadb`, `cargo check --workspace --all-targets`
- `cargo nextest run -p vantadb --lib` / `--profile audit --workspace --build-jobs 2`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo fmt --check`
- `rg`/`codegraph_explore` para confirmar callers
- Git: commit local `feat(planner): SearchProfileConfig per request + cláusula IQL PROFILE (MEM-01)` — sin push

## Context Save Point

- Branch: `develop`. Worktree con cambios de sesión previa (task files stale + plan file) — respetar, no pisar.
- Plan file: `docs/plans/2026-08-18-vanta-memory.md`; MEM-01 ⏳ EN PROGRESO.
- Detección: `rust` (checks arriba). Workflow: `feature-add` (spec → implement → verify → review → accept).
- Resultado a devolver: bloque RESULTADO (✅/🟡/❌/⚠️ + STEPS_OK + PROXIMO_STEP + COMMIT_HASH + VERIFY_CONTRATO + BLOQUEO).