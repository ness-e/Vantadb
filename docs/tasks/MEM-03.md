---
id: MEM-03
campaign: 20260902-alta-prioridad-paralelo
title: "F1 IQL PROFILE clause (src/query.rs, parser) — reuse MEM-01 SearchProfileConfig"
status: completed
estimate: "1-2 turns"
owner: vanta-engine
type: rust
---

# MEM-03 — F1 IQL PROFILE clause (parser + query + planner)

## Contexto (del lead)

F1 search profile ya landed en MEM-01 (`SearchProfileConfig {mode, rrf_k, candidate_k}` en `src/sdk/types.rs`, propagado a `VantaMemorySearchRequest` y `LogicalPlan`). Faltaba exponer la cláusula IQL `PROFILE` que permite a Studio/IQL fijar el perfil por query sin bridge: `FROM Person WHERE bio ~ "rust" PROFILE keyword rrf_k 20 candidate_k 64`. Esta tarea cierra el gap IQL → planner reutilizando el tipo de MEM-01 (disjoint con MEM-02 MCP passthrough y GOV-B3 docs).

Wave2 F1 paralelo MAX 3: MEM-03 (parser/query/planner) || MEM-02 (vantadb-mcp) || GOV-B3 (docs/tutorials). Archivos clave disjuntos: `src/parser/mod.rs`, `src/query.rs`, `src/planner/optimize_and_compile`.

## Contrato (DoD)

- [x] `src/parser/mod.rs` parsea `PROFILE <mode> [rrf_k <n>] [candidate_k <n>]` al final de `parse_query` — mode keyword|vector|hybrid, opts None → delega constantes core.
- [x] `src/query.rs` `Query.search_profile: Option<SearchProfileConfig>` y `LogicalPlan.search_profile` propagado en `into_logical_plan`.
- [x] `src/planner/optimize_and_compile` respeta mode: Keyword descarta vector_search, Vector descarta text_matches, Hybrid natural; ponytail rrf_k/candidate_k propagados sin efecto CBO (documentado).
- [x] `cargo test --lib parser` ≥117 passed, `cargo check` Finished, `cargo test --lib` 1976+ sin regresión.
- [x] Sin colisión con `SearchProfile` (src/index/search/profile.rs profiler I/O) — D14.
- [x] Commit atómico `feat(iql): MEM-03 PROFILE clause — reuse SearchProfileConfig (Wave2 F1)` en develop, sin push.

## Steps

- [x] **S1 — DISCOVERY** `codegraph_explore "iql profile parser query planner SearchProfileConfig"` (22 símbolos, 7 files) + Read `src/query.rs` 803L + `src/parser/mod.rs` PROFILE handling (lines 237-274) + `src/planner.rs` optimize_and_compile mode handling + `src/sdk/types.rs` SearchProfileConfig. Verify: grep SKILLS-MANIFEST.md keywords "iql|profile|query|parser|planner" → 0 hits directos (skill selection por analogía) — base 4 + extras por BUILD lifecycle. ✅
- [x] **S2 — Task file** crear `MEM-03.md` (este archivo) con contrato y steps. ✅
- [x] **S3 — EJECUCIÓN** verificar implementación reuse MEM-01: `SearchProfileConfig` ya en `src/sdk/types.rs:478`, `Query.search_profile` en `src/query.rs:105`, `LogicalPlan.search_profile` en `src/query.rs:423`, parser PROFILE en `src/parser/mod.rs:237-262` (+ parse_profile_mode + RESERVED_KEYWORDS PROFILE), planner mode routing en `src/planner.rs:322-328`. `cargo test --lib parser` 117/117, `cargo check --all-targets` Finished (1m42s). Ponytail: 0 líneas nuevas — reuse 6a50b8ee; deuda `rrf_k/candidate_k sin efecto CBO` documentada. ✅
- [x] **S4 — VERIFY** `cargo test --lib parser -- --nocapture` 117 passed + `cargo check --all-targets` Finished + `cargo test --lib query` (via --lib) sin regresión. ✅
- [x] **S5 — CIERRE** sync plan `2026-09-02-alta-prioridad-paralelo.md` MEM-03 → ✅ + recitation + git commit `feat(iql): MEM-03 ...` en develop. ✅

## Impacto mapeado (Regla 0)

- **Leído completo:** `src/parser/mod.rs` (1700L, RESERVED_KEYWORDS PROFILE, parse_query PROFILE opt, parse_profile_mode), `src/query.rs` (803L, Query.search_profile, LogicalPlan.search_profile, into_logical_plan), `src/planner.rs` (optimize_and_compile mode routing 322-328, ponytail comment), `src/sdk/types.rs` (SearchProfileConfig 478), `.opencode/skills/campaign-executor/tasks/MEM-01.md` (reuse).
- **Referencias entrantes:** `src/executor.rs` (execute_plan → optimize_and_compile), `src/cli_server.rs` (execute_query → parser), `tests/query_result_advanced.rs` (search_profile usage).
- **Referencias salientes:** `crate::sdk::{SearchProfileConfig, SearchProfileMode}`, `crate::query::LogicalPlan`, `crate::storage::StorageEngine`.
- **Disjoint verificado:** MEM-02 toca `vantadb-mcp/src/handlers/tools.rs` + validation; GOV-B3 toca `docs/tutorials/03-migrating-from-chromadb.md` + `docs/glosario/hnsw.md`; MEM-03 toca `src/parser`, `src/query`, `src/planner` — 0 overlap.
- **Veredicto:** bajo — aditivo, reuse tipo, sin breaking, sin nuevos deps, WASM-compatible.

## Herramientas

- `codegraph_explore "iql profile parser"` + `Read src/query.rs` / `src/parser/mod.rs`
- `cargo test --lib parser` + `cargo check --all-targets` + `cargo test --lib`
- `git add` + `git commit` (conventional feat(iql))

## SKILLS_CARGADAS (SDP BUILD ≤8)

Lifecycle BUILD + grep SKILLS-MANIFEST.md keywords "iql|profile|query|parser|planner" → 0 hits directos (no hay skills IQL-specific), selección por analogía + base canónica §2:

1. `campaign-executor` — task system PLAN/ACT/VERIFY, budget, SARL (base canónica)
2. `planning-and-task-breakdown` — slice vertical F1 PROFILE (base canónica)
3. `writing-plans` — plan file + task file spec (base canónica)
4. `ponytail(full)` — reuse SearchProfileConfig, 0 líneas nuevas (base canónica)
5. `api-and-interface-design` — Contract First para SearchProfileConfig trait/config, validate at boundaries (query.rs dim/NaN/mode)
6. `performance-optimization` — workflow MEASURE→VERIFY, budgets recall/QPS, avoid vtable en inner loop distance (upstream core)
7. `systematic-debugging` — root-cause parser/profile mode handling, grep callers antes de fix
8. `test-driven-development` — benchmarks como tests de regresión (parser 117, lib 1976)

Total 8/8 justificadas — base 6 + 2 extras por keywords query/parser/planner.

## Deuda

- `ponytail: IQL rrf_k/candidate_k propagados sin efecto hasta que CBO fusione RRF (path SDK ya lo usa)` — en `src/planner.rs:320-321`
- `ponytail: Keyword mode ignora sparse en path SDK` — documentado MEM-01, no bloquea F1

## Context Save Point

- Branch: develop (worktree limpio, 86 tasks plan alta-prioridad-paralelo)
- Plan: `docs/plans/2026-09-02-alta-prioridad-paralelo.md` Wave2 F1 MEM-01→02→34 → MEM-03→04→05
- CI: `cargo test --lib parser` 117/117, `cargo check` Finished, `cargo test --lib` 1976+ audit
- Próxima: MEM-04 checker allow-only (depende MEM-03 entity en plan original, pero Wave2 PROFILE no bloquea — disjoint)
