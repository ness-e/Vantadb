# FIND-49 — Split src/sdk/types.rs 1699L por dominio

> **Plan:** docs/plans/2026-09-08-backlog.md (Task 2, Wave0)
> **Estado:** ✅ COMPLETED (commit e3711dea; re-verificado 2026-09-08: check 0 + sdk:: 412/0 + fmt 0 + API 42/42 idéntica)
> **Tipo:** refactor (pure move, sin cambio API pública)
> **Workflow:** refactor (audit → migrate → cleanup → verify → review → accept → close)
> **SDP:** campaign-executor, source-driven-development, doubt-driven-development, incremental-implementation, test-driven-development, context-engineering, api-and-interface-design (frontend-ui-engineering descartada: no toca web/)
> **Branch/Commit:** develop / `refactor: FIND-49 split sdk types por dominio`
> **Contrato:** `cargo check -p vantadb` 0 + `cargo test -p vantadb sdk::` 0 failed + `cargo fmt --check` 0 + `scripts/validate-docs-coverage.ps1` 0 gaps nuevos

## Gate Justificación (del plan)

tipos SDK monolíticos 63035 bytes mezclados record/search/graph; split sin cambio API pública.

## Pre-mortem (del plan)

1. re-exports rotos rompen bindings Python/WASM → mitigación: `crate::sdk::types::X` y `vantadb::sdk::X` preservados vía `pub use`; verify incluye `cargo check -p vantadb-python -p vantadb-wasm`.
2. tipo en módulo equivocado → mitigación: asignación por dominio verificada con codegraph + callers antes de mover.

## Gate D — evaluación (2026-09-08)

- Blast radius: 17 archivos usan `crate::sdk::types::` in-crate + externos vía `vantadb::sdk::{...}` (providers, proxy, mcp, python, wasm, node, tauri). TODOS preservados vía re-exports — pure move, 0 símbolos públicos nuevos/eliminados.
- Hot path: no (solo types serializables, sin lógica runtime).
- Contrato: mecánico y no ambiguo.
- **Veredicto: NO disparado** — refactor pre-aprobado por owner (plan Gate P ✅ DO). Sin question tool disponible en este runner; se procede con la opción conservadora (API idéntica).

## Impacto mapeado (Regla 0)

**Archivo a dividir:** `src/sdk/types.rs` — 63035 bytes, 1856 líneas (medido 2026-09-08; backlog decía 1699L/2026-09-02, drift +157L por tests nuevos).

**Estructura actual (items `pub`, con líneas rg):**
- L8-33 `pub(crate) mod u128_serde` — helper serde u128↔string. Usado por: types.rs:191,750; `serialization/graph_types.rs:11,58`, `vector_types.rs:63`; docs en mcp/tests/lib.rs. QUEDA en types.rs (primitiva compartida).
- Primitivas (QUEDAN en types.rs): `VantaRuntimeProfile` 37, `VantaStorageTier` 48, `VantaValue`+impl 57-99, `VantaFields` 102, `VantaMemoryMetadata` 105.
- Dominio RECORD → `types/record.rs`: `VantaFilterOp` 109, `VantaMemoryFilterItem` 120, `VantaMemoryFilter` 127, `VantaMemoryInput`+impl 131-173, `VantaMemoryRecord` 175, `VantaMemoryListOptions`+Default 214-247, `VantaMemoryListPage` 249, `DEFAULT_EXPIRING_SOON_WINDOW_MS` 257, `VantaNamespaceStats` 261, `VantaNamespaceStatsMap` 271, `VantaMemoryExportLine` 699, `VantaExportReport` 298, `VantaImportReport` 311.
- Dominio SEARCH → `types/search.rs`: `VantaMemorySearchDebugReport` 451 (cfg debug), `SearchProfileMode` 463, `SearchProfileConfig` 478, `VantaHybridFusionReport` 492, `VantaSearchExplanation` 505, `VantaSearchExplanationHit` 516, `VantaBm25TermContribution` 537, `VantaTextIndexAuditReport` 644, `TextIndexState`/`TextIndexCounts` 568/592 (pub(crate)), `VantaTextIndexRepairReport` 326, `VantaIndexRebuildReport` 279 + re-export `vector_types::{VantaMemorySearchHit, VantaMemorySearchRequest, VantaSearchHit}` (hoy L273-275).
- Dominio GRAPH → `types/graph.rs`: re-export `graph_types::{VantaEdgeRecord, VantaNodeInput, VantaNodeRecord}` (hoy L731; definiciones QUEDAN en `serialization/graph_types.rs` — evita ciclo con `storage/engine` que usa `graph_types::unified_to_record` + evita tocar `serialization/mod.rs`) + `VantaQueryResult` 735 (usa VantaNodeRecord + u128_serde).
- QUEDAN en types.rs: `VantaOperationalMetrics` 345 (+tests 1362,1421), `VantaCapabilities` 757 (+tests 1099,1551,1564), Skills 777-903 (`SkillRecord`, `SkillCreateInput/Update/Patch`, `SkillListOptions/Page`, `SkillWriteResult` — fuera de scope, NOTICED BUT NOT TOUCHING), + `mod record/search/graph` + `pub use` re-exports + `mod tests` podado.

**Referencias hacia dentro (lo que types.rs consume):** `crate::node::SparseVector` (L4), `super::serialization::{vector_types, graph_types}` (L273,731), serde, BTreeMap/BTreeSet.

**Referencias entrantes (quién consume `sdk::types::`):** `src/skills.rs:28`, `src/skills/tests.rs:9`, `src/cost_estimator.rs:16,324`, `src/sdk/version_history.rs:19,287`, `src/sdk/serialization/{graph_types:3,vector_types:4-5,impl_index:287,impl_text_index:483,impl_rebuild:534}`, `src/sdk/search/{debug.rs:13,text_index.rs:10-11,tests.rs:5}`, `src/sdk/api/namespaces.rs`. Externos vía `vantadb::sdk::{...}`: providers/*, vanta-proxy, vantadb-mcp, vantadb-python, vantadb-wasm, vantadb-node, desktop tauri. Codegraph: `search` symbol 3 callers (mcp handlers, search/multi); `MemoryRecord` múltiples instantiations.

**Cobertura índice:** codebase-memory `check_index_coverage` src/sdk/types.rs + api.rs → `no_recorded_issue` (best-effort); CodeGraph 18 símbolos relevantes.

**Veredicto:** split seguro como pure move con `pub use` re-exports en types.rs. `mod.rs` (`pub use types::{...}` lista L22-33) NO cambia. Ningún consumer cambia imports. Riesgo residual: `TextIndexState/Counts` pub(crate) movidos a search.rs — accesibles vía `crate::sdk::types::{...}` igual que hoy.

## Context Save Point

- Codebase verificado: types.rs 1856L (rg max 1849), mod.rs 33L con re-export list, u128_serde solo pub(crate).
- Decisión dominio: Export/Import reports → record.rs (ciclo de vida del record); IndexRebuild/TextIndexRepair reports → search.rs (ops de índice); OperationalMetrics/Capabilities/Skills → types.rs (fuera de scope).
- Tests se mueven CON su dominio (risk register plan: "mover tests con su módulo").
- Técnica Rust: `src/sdk/types.rs` + dir `src/sdk/types/*.rs` coexisten (Rust 2018+); `crate::sdk::types::X` preservado.

## Steps atómicos

- [x] **Step 1 — types/record.rs:** creado (13 items + 14 tests); types.rs: `mod record;` + `pub use record::{...}`; `cargo check` verde tras fix graph path + DebugReport re-export.
- [x] **Step 2 — types/search.rs:** creado (20 items + re-export vector_types + 14 tests); `pub use` + `pub(crate) use` en types.rs; check verde.
- [x] **Step 3 — types/graph.rs:** creado (re-export graph_types + VantaQueryResult + 6 tests); check verde.
- [x] **Step 4 — Cierre:** `cargo test -p vantadb sdk::` 412 passed/0 failed; `cargo fmt --check` 0; `cargo check -p vantadb_py -p vantadb-wasm` verde; docs-coverage script roto pre-existente (ver Notas) + verificación manual 0 gaps (sin cambio API pública); commit.

## Notas de ejecución (2026-09-08)

- `campaign_verify_cmd` MCP con bug ("autoTransition is not defined") → verificación vía bash directo (mismo comandos).
- Script quirúrgico PowerShell colgado por `while ($lines[$e] -ne '};')` con índice fuera de rango (PowerShell devuelve `$null`, loop infinito) en re-export single-line de graph_types → fix con guard + rama single-line.
- `cargo test -p vantadb sdk::` falló 1 vez por OOM de rustc (E0463 en cascada, build concurrente con otros agentes Wave0) → retry con `CARGO_BUILD_JOBS=2` verde.
- `scripts/validate-docs-coverage.ps1` tiene error de parse PRE-EXISTENTE (commit 24d0b86d, scripts/ limpio en git) → contrato verificado manualmente: 0 items públicos nuevos/eliminados/renombrados (solo moves + re-exports), `docs/api` intacto.

## Contrato verificación

- `cargo check -p vantadb` → 0 warnings/errors
- `cargo test -p vantadb sdk::` → 0 failed
- `cargo fmt --check` → 0 diff
- `scripts/validate-docs-coverage.ps1` → 0 gaps nuevos
- Extra (pre-mortem): `cargo check -p vantadb-python -p vantadb-wasm` verde
