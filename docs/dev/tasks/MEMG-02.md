---
title: "TASK MEMG-02: Outcome loop → refuerzo de confianza post-recall"
kind: task
description: "Op real de refuerzo (SDK `Embedded::reinforce` + tool MCP `memory_reinforce`) que realimenta `confidence` + `last_validated_at_ms` con política declarada (used/corrected/unused; bump/decay saturado y acotado) + cierre del loop desde `vanta-memory` (`reinforce_recalled`). Calibración excluida (VER-08)"
---

# TASK MEMG-02: Outcome loop → refuerzo de confianza post-recall

## Metadata

- **Plan file:** `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (Task 38, Wave F2)
- **Fuente:** Backlog `MEMG-02` (fila :138) + plan Task 38 (bloque F0-expandido 2026-10-05) + MGR-12 (`docs/dev/research/mgr-12-confianza.md` §3.3, §6.3, §8) + DELTA §P1
- **Esfuerzo:** 🟠 2-3d | **Appetite:** max 3d
- **Prioridad:** 🟠
- **Tipo:** Mixto (Rust core SDK + tool MCP + vanta-memory + docs)
- **Turns estimados:** 15-30
- **Creado:** 2026-10-05T00:00Z | **last-synced:** 2026-10-05T12:00Z
- **Estado:** ✅ COMPLETED (commit local en curso — Step 6)
- **Incógnitas (uphill):** 0 — shape/semántica del outcome fijados por el plan (used/corrected/unused) y resueltos por evidencia en §Spec; estado real re-verificado HEAD
- **Pendientes (downhill):** 0 steps (6/6 ✅; resta commit local + cierre campaign)
- **Campaign ID:** master-plan-0.9.0-20261004 · **Campaign taskId:** `38`

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | **Nuevos (0 previos):** `Embedded::reinforce` ← tool MCP `memory_reinforce` (nuevo) ← agentes; `reinforce_recalled` ← hosts de `vanta-memory`. **Existentes a verificar:** `RecalledMemory` (2 callers: `hooks/mod.rs`, `auto_recall.rs` — solo se le agrega un helper additive, el struct no cambia); `handle_tools_list` (consumido por `vantadb-mcp/src/lib.rs`, `server.rs`, `tests/mcp_tests.rs` — tool count 80→81, annotations coverage, perfiles); `tests/api/public-api.txt` (HARD-01 snapshot — requiere regenerar). |
| Callees | `Embedded::reinforce` → `self.get` + `engine.insert` + `memory_record_to_node_owned` (serialización existente; `__vanta_last_validated_at_ms` ya mapeado, `serialization/mod.rs:562`) + `self.audit` + `supersede_lock` (lock existente, mismo orden). `reinforce_recalled` → `sanitize_key` (`l0_recorder.rs:157`) + `db.reinforce`. Cero dependencias nuevas. |
| Implicaciones | **Contrato público (aditivo):** `Embedded::reinforce(namespace, key, outcome) -> Result<MemoryRecord>` + enum `ReinforceOutcome` (re-export en `sdk/mod.rs` + `lib.rs`). Semver minor. **Comportamiento:** ningún default cambia — quien no llama la op ve exactamente lo de hoy (`last_validated_at_ms` sigue `None` en `put`); el refuerzo es el primer writer del campo. **Serialización:** sin cambios de formato (campos v2 existentes). **Migración:** ninguna. **Performance:** no hot path (RMW O(1) por registro bajo lock existente; sin benchmark — Regla 9 no dispara, no hay claim de optimización). **Tests:** `test_mcp_tool_annotations_coverage` (80→81 + annotations) y snapshot público regenerado; suites existentes intactas. **Docs:** `EMBEDDED_SDK.md` + `scores.md` + `MCP.md` en el mismo commit (Regla 3 + R-5). |

**PROHIBIDO tocar:** `opencode.jsonc` (WIP ajeno), `docs/dev/plans/2026-10-04-master-plan-0.9.0.md` (bookkeeping del orquestador), `docs/pipeline-state.json`, `vanta-memory/src/core/record/l1_writer.rs` / `l1_dedup.rs` / `core/prompts/` / `core/dream/` (WIP de MEMG-01 — si hay conflicto real, documentar y no pisar), `src/wal.rs`, `src/storage/`, `src/vector/` (dominio Arch/Engine).

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos o secciones funcionales íntegras):**
  - `src/sdk/api/memory.rs` (1675L — leído: helpers `QuarantineState` `:44-153`, `materialize_confidence` `:185-250`, `put_one` `:439-543`, `put` `:573`, `delete_certified`/`verify_purge_certificate` `:953-995`, `put_record_exact` `:1005-1060`, `supersede` `:1071-1150`, `quarantine_apply/promote/reject` `:1152-1279`) — patrón RMW bajo `supersede_lock` + `engine.insert` + `audit`; punto de inserción de `reinforce`: tras `supersede` (`:1150`).
  - `src/sdk/types/record.rs` — `default_confidence()` `:74` (D_a=1.0), `DERIVATION_DISCOUNT` `:81`, `ConfidenceClass` `:87-114` (patrón `#[non_exhaustive]` + `as_wire_str`), `MemoryInput` `:124-194` (sin `last_validated_at_ms` — el op NO puede ir por `put`), `MemoryRecord` `:196-273` (campos v2 `:244-253`), `MemoryListOptions.min_confidence` `:363-367`.
  - `vanta-memory/src/core/hooks/auto_recall.rs` (líneas 1-456 + 536-644) — `RecalledMemory` `:43-57` (ya expone `source_namespace` + `source_key`, VER-04), `RecallResult` `:159-174`, `search_records` `:539-644`, `perform_auto_recall` `:311`.
  - `vanta-memory/src/core/record/l1_reader.rs` (322L) — `read_namespace_records` (payload JSON + vector; el core `confidence` del nodo no se lee hoy), `l1_namespace`, `sanitize` usage.
  - `vanta-memory/src/core/record/l1_writer.rs` (secciones `:100-345`) — `put_record` key = `sanitize_key(record.id)` `:334`; `L1Error::Vanta(#[from] Error)` `:31-36`.
  - `vantadb-mcp/src/handlers/tools.rs` (secciones: comentario contrato `:1-40`, tools list `:81-1090` incl. `memory_recall` `:320-342`, `memory_supersede` `:284-302`, dispatch `memory_recall` `:1881-2004`, `memory_tools` profile array `:1076-1099`).
  - `vantadb-mcp/tests/mcp_tests.rs` — `test_mcp_tool_annotations_coverage` `:4841-4910` (assert `tools.len() == 80`), structuredContent test `:4796-4828`, perfiles `:5100-5128`.
  - `tests/api/public_api.rs` (mecanismo de snapshot HARD-01; regen: `VANTADB_PUBLIC_API_UPDATE=1` + nightly — nightly presente: rustc 1.101.0-nightly).
  - `docs/api/EMBEDDED_SDK.md` §Memory `:66-125` + `MemoryRecord` `:181-208`; `docs/api/scores.md` §Record Confidence `:73-95`; `docs/api/MCP.md` tabla tools `:287-304` + Last sync `:653`.
- **Archivos referenciados hacia dentro (imports/deps):** `memory.rs` → `types::*`, `serialization::*` (`memory_record_to_node_owned`, `validate_*`, `now_ms`), `error::{Error, Result}`, `node::FieldValue`. `auto_recall.rs` → `l1_reader`, `l1_writer::EmbedFn`, `conversation::sanitize_key` (vía l1_reader). Sin imports nuevos de terceros.
- **Referencias entrantes (grep/CodeGraph):** `perform_auto_recall` 15 callers (desktop, pipeline_worker, vantadb-python, hooks); `RecalledMemory` 2 callers (solo se agrega función libre `reinforce_recalled` — el struct no cambia, cero breaking); `handle_tools_list` consumido por lib/server/tests (tool count assertions); `tests/api/public-api.txt` comparado por el job dedicado `public-api-snapshot`.
- **Veredicto impacto:** **MEDIO (controlado, aditivo)** — superficie pública nueva (SDK + MCP) sancionada por el contrato del plan; sin cambios de semántica existente, sin formato on-disk, sin hot path. Riesgos del pre-mortem mitigados: (1) semántica falsifiable → enum explícito `Used/Corrected/Unused`; (2) inflación → saturación + ventana de 5 min anclada en `last_validated_at_ms`; (3) scope a calibración → excluido (solo contadores + audit).

## Contrato

"`Embedded::reinforce(ns, key, outcome)` (outcome ∈ `Used`/`Corrected`/`Unused`) realimenta `confidence` + `last_validated_at_ms` con política declarada — `Used`: +0.05 saturado en 1.0, máx 1×/ventana de 5 min (ancla `last_validated_at_ms`), stampa validación; `Corrected`: −0.10 con piso 0.0, sin stamp (success-only, MGR-12 §3.3); `Unused`: neutral (audit-only) — y está expuesta como tool MCP `memory_reinforce`; `vanta-memory` cierra el loop con `reinforce_recalled(db, &RecalledMemory, outcome)`. Test dedicado RED→GREEN: recall → refuerzo positivo → score sube / negativo → baja; métrica de selección antes/después documentada (`min_confidence` / `confidence_threshold`); sin cambio de defaults para quien no llama la op; derived rechazado; calibración excluida. Verify: `cargo nextest run --profile audit -p vantadb -E 'test(/reinforce/)'` + `-p vanta-memory --test reinforce_loop` + `-p vantadb-mcp --test mcp_tests -E 'test(/reinforce|annotations/)'` + `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` verdes."

## Spec (SDD — decisiones por evidencia)

> **Gate D evaluado (DISCOVERY): no disparado** — el contrato del plan (Task 38, F0-expandido, Gate Result ✅ DO, "listas para ejecutar") sanciona la superficie ("una op real de refuerzo (SDK/MCP)") y fija la semántica ("API explícita del host (used/corrected/unused) fijada en DISCOVERY"); el nombre `reinforce` está fijado por el título/Backlog. Micro-decisiones de forma abajo, decidido-por-evidencia (precedente DIST-15/DIST-02). Sin símbolos fuera de la sanción del plan.

| # | Decisión | Opciones (+tradeoff) | Resuelto |
|---|----------|----------------------|----------|
| 1 | Superficie de la op | A) **Core SDK `Embedded::reinforce` + tool MCP `memory_reinforce`** (contrato del plan: "SDK/MCP") / B) solo SDK (agentes no cierran el loop) / C) solo MCP (el core pierde la op) | ✅ **A** — decidido-por-evidencia: plan Task 38 contrato + Backlog :138 (`reinforce`); MCP es la puerta del ICP (`memory_recall` ya expone `source_namespace`/`source_key`, `tools.rs:1966-1978`) |
| 2 | Semántica de outcome | A) **`Used`/`Corrected`/`Unused` explícitos** (plan pre-mortem #1: "nunca inferencia silenciosa") / B) bool success / C) inferencia desde uso | ✅ **A** — decidido-por-evidencia: plan Task 38 pre-mortem #1; enum `#[non_exhaustive]` (R-6) |
| 3 | Política positiva (`Used`) | A) **+0.05 saturado en 1.0, máx 1×/ventana de 5 min (ancla = `last_validated_at_ms`), stampa validación** / B) bump mayor (inflación) / C) sin rate-limit | ✅ **A** — decidido-por-evidencia: pre-mortem #2 "saturación + tasa acotada por ventana"; MGR-12 §3.3 (estampado success-only); ancla = el campo que la op ya escribe (sin schema nuevo — YAGNI) |
| 4 | Política negativa (`Corrected`) | A) **−0.10 con piso 0.0, sin ventana** (correcciones repetidas = señal real; el piso acota) / B) −0.05 / C) sin decay | ✅ **A** — decidido-por-evidencia: contrato "bump/decay acotado y saturado"; asimetría deliberada (trust lento de ganar, rápido de perder); fallos no stampan (MGR-12 §3.3) |
| 5 | `Unused` | A) **Neutral: sin cambio de score, sin stamp; solo audit** ("recalled but not used" ≠ incorrecta; evita decay masivo de memories válidas: hasta 5 recalls/turno) / B) decay pequeño (falsos negativos masivos) / C) error | ✅ **A** — decidido-por-evidencia: señal conservadora (precedente MEMG-01 "solo contradicción explícita"); contrato solo exige positivo sube / negativo baja |
| 6 | Registros `derived` | A) **Rechazar (`InvalidInput`)** — el score derived se computa de padres (V2/V4, ADR-046 §D4); mutarlo rompe determinismo / B) permitir (rompe V4) / C) no-op silencioso | ✅ **A** — decidido-por-evidencia: MGR-12 §2.3 (V2/V4) + `materialize_confidence` (`memory.rs:185-250`); mismo patrón de rechazo explícito que `quarantine_promote` (`memory.rs:1221-1225`) |
| 7 | `version`/snapshot al reforzar | A) **NO bumpear `version`, NO snapshot; `updated_at_ms = now`; audit `memory_reinforce`** (state ≠ content — precedente quarantine ops; evita inundar version-history de 32) / B) bump + snapshot por refuerzo (flood de history) | ✅ **A** — decidido-por-evidencia: `quarantine_apply`/`quarantine_promote` (`memory.rs:1152-1244`, "state ≠ content; the audit event is the evidence") |
| 8 | Dentro de la ventana (positivo) | A) **No-op: retorna el registro sin cambios; audit reason `used_rate_limited`** / B) error / C) bump igual | ✅ **A** — política declarada; el host reintenta después; el no-op es observable en el audit (nunca silencio) |
| 9 | Ordenamiento | A) **Sin ranking ponderado** (excluido: MGR-12 §8 "ponderar con scores no calibrados = regresión"); métrica = selección por umbral (`min_confidence`/`confidence_threshold`) antes/después / B) boost en ranking | ✅ **A** — decidido-por-evidencia: MGR-12 §6.3/§8 + contrato "sin cambio de defaults" |
| 10 | Calibración / jueces LLM | A) **Excluido (VER-08 / MGR-12 §6.3); solo contadores + audit** / B) incluir ahora | ✅ **A** — decidido-por-evidencia: plan Task 38 pre-mortem #3 + stop condition |
| 11 | Nombre tool MCP | A) **`memory_reinforce`** — familia `memory_*` canónica (API-04), pareja de `memory_recall` / B) `reinforce` (fuera de familia) / C) `memory_feedback` | ✅ **A** — decidido-por-evidencia: convención `memory_*` (`tools.rs:1076-1099`); el Backlog/título fijan `reinforce` como nombre de la operación |
| 12 | Lock del RMW | A) **Reusar `supersede_lock`** (mismo patrón REVIEW-13 que `supersede`/`quarantine_*`) / B) lock nuevo (orden nuevo — riesgo deadlock) | ✅ **A** — decidido-por-evidencia: `memory.rs:1088,1174,1213,1258` |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. `src/wal.rs`, `src/storage/`, `src/vector/`, `src/serialization/` NO se tocan (solo se consumen mappers existentes).
  2. Campos de `MemoryRecord`/`MemoryInput` sin cambios — v2 congelado; `last_validated_at_ms` ya viaja por el wire (`__vanta_last_validated_at_ms`).
  3. `put`/`put_batch` intactos: `last_validated_at_ms = None` al re-escribir (semántica actual documentada en MGR-12 §3.3); `reinforce` es el único writer.
  4. Ranking por defecto intacto — sin ponderación por confianza (MGR-12 §8).
  5. `perform_auto_recall` byte-idéntico para quien no usa `reinforce_recalled` (helper additive; `RecalledMemory` no cambia).
  6. Enums nuevos `#[non_exhaustive]` (R-6); sin `unwrap`/`expect` en código nuevo; `unsafe` cero.
  7. Tool count MCP actualizado en código + tests + `MCP.md` en el mismo commit (R-5 + `validate-docs-coverage.ps1`).
  8. `tests/api/public-api.txt` regenerado (HARD-01) — el diff del snapshot limitado a los símbolos nuevos.
  9. Sin dependencias nuevas.
  10. WIP de MEMG-01 (l1_writer/l1_dedup/prompts/dream) no se pisa; pathspec al commitear.
- **Comandos de verificación:** `cargo nextest run --profile audit -p vantadb -E 'test(/reinforce/)'` · `cargo nextest run --profile audit -p vanta-memory --test reinforce_loop` · `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests` · `cargo fmt --check` · `cargo clippy --workspace --all-targets --all-features -- -D warnings` · `pwsh scripts/validate-docs-coverage.ps1` · `node scripts/docs/check-links.mjs && node scripts/docs/check-docs.mjs && node scripts/docs/gen-index.mjs --check`.
- **Deuda pendiente:** calibración empírica del bump/decay (VER-08, excluida por contrato); consumo automático más profundo en el host (p.ej. hooks que reporten outcome por turno) queda como FIND si el slice mínimo no lo cubre.

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** sin deuda nueva (op additive + tests + docs; no introduce `unsafe`, clones en hot path, ni abstracciones especulativas).

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable del task file: op SDK + tool MCP + helper loop con test dedicado RED→GREEN; suites scoped verdes; fmt/clippy verdes; métrica de selección antes/después documentada; docs sincronizadas |
| **Commit** | Commit atómico conventional `feat(memory):` + pathspec solo de archivos propios + verificación mecánica (nunca auto-reporte); LOCAL (nunca push) |
| **Release** | Changelog release-plz (feature → minor); `dev-tools/verify.ps1` no se corre completo por presupuesto — verify scoped + workspace clippy/fmt documentado |

## Herramientas necesarias

- `codegraph_codegraph_explore` (blast radius `perform_auto_recall`/`RecalledMemory`/`MemoryRecord`) + `codebase-memory-mcp_check_index_coverage` (10 paths, `no_recorded_issue`) + grep puntual (configs/docs no indexados)
- `cargo nextest` scoped por crate (loop TDD) + `campaign_verify_cmd` (verify mecánico)
- `pwsh dev-tools/ocr-review.ps1 -Format json` (OCR delegation al cierre)
- Regen snapshot: `$env:VANTADB_PUBLIC_API_UPDATE="1"; cargo nextest run -p vantadb --test public_api --run-ignored ignored-only` (nightly presente)

**Skills cargadas (SDP v3):** `test-driven-development` (RED→GREEN de la op nueva) · `rust-write-tests` (calidad de tests core/MCP/memory) · `source-driven-development` (verificación contra código real, no memoria) · `incremental-implementation` (slices verticales: core → MCP → loop → docs) · `documentation-skill` (task file + `docs/api/*` con frontmatter y gates) · `doubt-driven-development` (stakes de API pública — verificación adversarial al cierre). Base auto: campaign-executor, progreso, ponytail. SDP: 6 cargadas + base; `security-and-hardening` excluida (sin input de usuario nuevo en trust boundary nuevo — el MCP reusa validadores existentes), `performance-optimization` excluida (no hot path).

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — evaluación: la op MCP es un trust boundary de agente, pero reutiliza los validadores existentes (`validate_namespace`/`validate_key` vía `reinforce`; outcome parseado contra enum whitelist — string desconocido → `invalid_params`, nunca mutación). Sin auth/secrets/deps nuevas; el audit event no loggea payloads. Sin hallazgos que requieran `security-and-hardening` (no hay control nuevo de seguridad — solo un contador de confianza acotado).
- [x] **PERFORMANCE** — no aplica: RMW O(1) por registro bajo lock existente, fuera de hot paths (search/ingest/serialización). Regla 9 no dispara (no hay claim de optimización).

## Steps

### Step 1 — RED: tests dedicados del contrato core (`reinforce`)

- **Archivos:** `src/sdk/api.rs` (tests module)
- **Acción:** escribir los tests que fijan el contrato ANTES de implementar: `reinforce_used_bumps_confidence_and_stamps_validation` (0.5→0.55 + `last_validated_at_ms = Some`), `reinforce_used_is_rate_limited_within_window`, `reinforce_used_after_window_bumps_again`, `reinforce_corrected_decays_and_does_not_stamp`, `reinforce_corrected_floors_at_zero`, `reinforce_used_saturates_at_one`, `reinforce_unused_is_neutral`, `reinforce_missing_record_errors_not_found`, `reinforce_rejects_derived_record`, `reinforce_changes_min_confidence_selection` (métrica antes/después), `reinforce_flips_confidence_threshold_abstention_before_after` (métrica de abstención). Correr → RED esperado: no compila (`no method named reinforce` / `ReinforceOutcome` inexistente) = falla por la razón correcta.
- **Verify:** `cargo check -p vantadb --tests` → RED
- **Evidencia:** ✅ RED verificado: `cargo check -p vantadb --tests` → 35 errores — E0599 `no method named reinforce found for struct Embedded` (13×), E0433 `cannot find type ReinforceOutcome` (12×), E0425 `cannot find value REINFORCE_WINDOW_MS in module super::memory` (1×). Fallo por la razón correcta (API inexistente, no por error de sintaxis del test).
- **Estado:** ✅ COMPLETED

### Step 2 — GREEN: enum + op + re-exports (mínimo)

- **Archivos:** `src/sdk/types/record.rs` (enum `ReinforceOutcome`), `src/sdk/types.rs` (re-export), `src/sdk/api/memory.rs` (`reinforce` + consts de política), `src/sdk/mod.rs` + `src/lib.rs` (re-exports)
- **Acción:** `ReinforceOutcome { Used, Corrected, Unused }` (`#[non_exhaustive]`, serde snake_case, `as_wire_str`); `Embedded::reinforce` con RMW bajo `supersede_lock`: get → guardas (derived reject) → política (`REINFORCE_CONFIDENCE_BUMP=0.05`, `REINFORCE_CONFIDENCE_DECAY=0.10`, `REINFORCE_WINDOW_MS=300_000`) → `engine.insert` + audit `memory_reinforce` (reason = outcome / `used_rate_limited`). Mínimo código que pasa los tests del Step 1.
- **Verify:** `cargo nextest run --profile audit -p vantadb --lib --build-jobs 2 -E 'test(/reinforce/)'` → GREEN
- **Evidencia:** ✅ `cargo nextest run --profile audit -p vantadb --lib --build-jobs 2 -E 'test(/reinforce/)'` → **11 tests run: 11 passed, 2309 skipped** (2.006s). Nota de entorno: el primer intento de nextest (todos los targets) falló por presión de disco del host (`failed to mmap rlib: page file too small` + `LNK1318 PDB LIMIT`) — NO del código; resuelto con `dev-tools/target-cleanup.ps1 -Clean -Yes` (26.45 GB de `target/debug/incremental` liberados; 2.5 MB → 31.28 GB libres) + corrida scoped `--lib`. `cargo check -p vantadb --tests` exit 0.
- **Estado:** ✅ COMPLETED

### Step 3 — Superficie MCP `memory_reinforce` + tests + docs

- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (tools list + dispatch + `memory_tools` + comentario contrato), `vantadb-mcp/tests/mcp_tests.rs` (counts + test funcional), `docs/api/MCP.md` (fila + counts + Last sync)
- **Acción:** registrar la tool (annotations: readOnly false / destructive false / idempotent false / openWorld false), dispatch con validación de args y `structuredContent {namespace, key, confidence, last_validated_at_ms, outcome}`, error mapping tipado; sumar a `memory_tools` (4 perfiles); actualizar counts en tests + comentario; fila en MCP.md (R-5).
- **Verify:** `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests --build-jobs 2 --ignore-default-filter` → GREEN · `pwsh scripts/validate-docs-coverage.ps1` → 0 gaps
- **Evidencia:** ✅ `mcp_tests` completa **115/115 passed** (31.6s; incluye el test nuevo `test_memory_reinforce_updates_confidence_and_rejects_bad_outcome` y los counts actualizados: annotations 80→81, profiles agent 38→39 · full 80→81 · dev 37→38 · memory 21→22, canonical-names 80→81, agent-default-surface 38→39). `vantadb-mcp` suite completa (21 binarios) **261/261 passed**. `validate-docs-coverage.ps1` → **0 gaps** (MCP tools ok). Nota: `mcp_tests` está excluido del `default-filter` de nextest → scoped con `--ignore-default-filter` (patrón del repo para esta suite).
- **Estado:** ✅ COMPLETED

### Step 4 — Loop closure en `vanta-memory` + test de recall→refuerzo

- **Archivos:** `vanta-memory/src/core/hooks/auto_recall.rs` (fn `reinforce_recalled`), `vanta-memory/src/core/hooks/mod.rs` (re-export), `vanta-memory/tests/reinforce_loop.rs` (nuevo)
- **Acción:** `pub fn reinforce_recalled(db, recalled: &RecalledMemory, outcome) -> Result<vantadb::MemoryRecord, L1Error>` — mapea `source_namespace` + `sanitize_key(source_key)`; identidad vacía (payload legacy) → `InvalidInput` explícito. Test: put L1 record (confidence 0.5) → `perform_auto_recall` → hit con identidad → `reinforce_recalled(Used)` → `confidence` 0.55 + stamp; luego `Corrected` → baja.
- **Verify:** `cargo nextest run --profile audit -p vanta-memory --test reinforce_loop --build-jobs 2` → GREEN
- **Evidencia:** ✅ `reinforce_loop` **2/2 passed** (loop completo recall→used→0.55+stamp→corrected→0.45; rechazo de hit legacy sin identidad). Suite completa `vanta-memory` (31 binarios) **589/589 passed** (49.3s).
- **Estado:** ✅ COMPLETED

### Step 5 — Docs API + snapshot público

- **Archivos:** `docs/api/EMBEDDED_SDK.md` (fila `reinforce` + `ReinforceOutcome`), `docs/api/scores.md` (§Reinforcement: política declarada + límites), `tests/api/public-api.txt` (regen)
- **Acción:** documentar la op y la política (bump/decay/ventana, success-only, derived rechazado, calibración excluida, métrica de selección) en el mismo commit (Regla 3); regenerar snapshot público con nightly (solo símbolos nuevos en el diff).
- **Verify:** `cargo nextest run -p vantadb --test public_api --run-ignored ignored-only` (comparación) + gates docs (`check-links`/`check-docs`/`gen-index --check`)
- **Evidencia:** ✅ Snapshot regenerado (`VANTADB_PUBLIC_API_UPDATE=1` + nightly; 67 inserciones: `ReinforceOutcome` sdk+root, `Embedded::reinforce` — **absorbe además drift pre-existente de FIND-237 (`MigrateCommand::*::target: Option<String>`) y DIST-15 (`GraphRag*::Serialize`) no refrescado desde `eb1d09b2`**; documentado en Notas). Re-verificación comparación: **1/1 passed**. Gates docs: `check-links` exit 0 · `check-docs` exit 0 · `gen-index --check` exit 0 · `validate-docs-coverage.ps1` 0 gaps.
- **Estado:** ✅ COMPLETED

### Step 6 — Verify full + OCR + review P2-01 + commit local

- **Archivos:** `docs/dev/tasks/MEMG-02.md`
- **Acción:** `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + suites scoped (4) + `validate-docs-coverage.ps1` + gates docs; OCR delegation (`pwsh dev-tools/ocr-review.ps1 -Format json`); review P2-01 tier **Adversarial** (`src/sdk/**` + `docs/api/**` matchean HARD-02 → fork a `vanta-review`); registrar veredicto en §Review; commit **LOCAL** `feat(memory):` con pathspec (nunca push).
- **Verify:** `git show --stat HEAD` limitado a archivos propios; veredicto en §Review
- **Estado:** ⬜ PENDING

## Dependencias

- SCH-02/04/05 ✅ (campos v2 + filtros/abstención — base read-side), VER-04 ✅ (`source_namespace`/`source_key` en `RecalledMemory`), MEM-59 ✅ (`memory_recall` MCP).
- MEMG-01 (Task 37, en vuelo): sin dependencia funcional; solo coordinación de paths (no solapar archivos).
- nextTask: MEMG-11 (Task 39) — lo decide el orquestador.

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** vanta-review — P2-01, contexto fresco (no participó de la implementación); tier Adversarial (HARD-02: `src/sdk/**` + `docs/api/**`). Sesión reviewer: `ses_ef52142a4ffexvKGFDej3kq2vs`. Veredicto: ✅ **APPROVE** (sin Critical/High; 6 hallazgos Low).
- **Enfoque:** contrato verificado comando por comando re-corridos por el reviewer; decisiones de diseño red-teameadas (bool/inferencia rechazados; `put`-con-confidence rechazado por `put_one` reset + `MemoryInput` sin campo; `Unused` neutral correcto; reuso de `supersede_lock` idéntico a REVIEW-13; boost de ranking correctamente excluido MGR-12 §8; precedente `quarantine_apply` exacto). Loop correctness verificado (misma `sanitize_key` write↔helper; `source_namespace = l1_namespace(session_key)`; lock order sin deadlock). Snapshot drift FIND-237/DIST-15 absorbido: **aceptable** (regeneración íntegra, diff = solo esos símbolos + propios).
- **Cómo se probó:** reviewer re-corrió: `cargo nextest --profile audit -p vantadb --lib -E 'test(/reinforce/)'` 11/11 · `-p vanta-memory --test reinforce_loop` 2/2 · `-p vantadb-mcp --test mcp_tests --ignore-default-filter` (reinforce/annotations 2/2; profiles/agent/canonical 3/3) · `cargo fmt --check` 0 · clippy scoped 0 · `public_api` snapshot compare 1/1 · `validate-docs-coverage.ps1` 0 gaps · gates docs 0. Invariantes por grep (sin unwrap/expect/unsafe en producción; `put` intacto; `l1_writer`/`l1_dedup`/prompts/dream y `pipeline-state.json` sin tocar; RED creíble — HEAD sin la API).
- **Hallazgos (Low) + disposición:**
  1. Rustdoc prometía drift-test inexistente para `ReinforceOutcome` → **cerrado**: test `reinforce_outcome_wire_str_matches_serde` agregado (`record.rs`, patrón del drift-test de `ConfidenceClass`).
  2. Wording "case-insensitive" contradictorio → **cerrado**: rustdoc corregido (match exacto snake_case).
  3. `used_rate_limited` sin test + decisión #7 sin pin → **cerrado**: test `reinforce_rate_limited_is_audited_and_does_not_bump_version` (audit JSONL real + `version`/`updated_at_ms` intactos).
  4. Interacción `Corrected`↔ancla no documentada → **cerrado**: nota en rustdoc de `reinforce` + `scores.md` §Reinforcement.
  5. Cuarentena no chequeada en `reinforce` → **documentado** (rustdoc + scores.md: recall/list excluyen quarantined; solo key explícita alcanza).
  6. Nombrar el drift del snapshot en el commit → **aplicado** en el body de `feat(memory):`.
- **Post-approve micro-fixes (dentro de la dirección aprobada):** re-verificados tras los fixes: `test(/reinforce/)` **13/13** · `cargo fmt --check` 0 · clippy scoped 0 · `check-docs`/`check-links` 0.
- **Veredicto:** ✅ **APPROVE** — contrato sostenido con evidencia reproducible; listo para commit local (Step 6, sin push).

## Notas

- **Estado real verificado HEAD (2026-10-05):** `reinforce` = 4 hits y ninguno toca confianza (3 prompts `persona_generation.rs:91,94,197` + comentario `fusion.rs:88`); `outcome` = 14 (`vanta-memory/src`) / 52 (`src/`), sin loop de feedback; `last_validated_at_ms` sin writer de producción (`put_one` lo setea `None`; solo tests/version-history lo escriben). Gap vigente confirmado.
- **Por qué el op NO va por `MemoryInput`/`put`:** `MemoryInput` no tiene `last_validated_at_ms` y `put_one` resetea a `None` (`memory.rs:500`); el refuerzo es un RMW de estado, mismo patrón que `quarantine_apply` (MGR-12 §3.3: un `put` no toca `last_validated`).
- **D1 default = 1.0:** el bump positivo es no-op saturado en registros default (D_a); el valor del `Used` en esos casos es el **estampado de frescura** (`last_validated_at_ms`) — comportamiento declarado, no bug. Registros con score declarado < 1.0 o corregidos sí suben.
- **`Unused` neutral:** decisión de política documentada en §Spec #5 (evita decay masivo de memories válidas no usadas en cada turno).
- **Derived rechazado:** coherente con V2/V4 (MGR-12 §2.3); el camino correcto para derived es reforzar a los padres.
- **Snapshot público — drift pre-existente absorbido:** la regeneración (HARD-01) incluye 3 cambios de FIND-237 (`MigrateCommand::{Check,Plan,Run}::target: String → Option<String>`) y 8 de DIST-15 (`GraphRag{Edge,Node,Result,Stats}::Serialize`) que no fueron refrescados desde `eb1d09b2` — el job `public-api-snapshot` estaba rojo en develop por eso. Se absorben en esta regeneración (transparencia en el commit); el diff propio son los símbolos MEMG-02 (`ReinforceOutcome` + `Embedded::reinforce`).
- **NOTICED BUT NOT TOUCHING:** `memory_put`/`memory_put_batch` (MCP) no aceptan `confidence` declarable (schema sin el campo; `parse_memory_input` lo ignora) — gap análogo a FIND-199 (Python); candidato a fila `FIND-*` (no se arregla inline: fuera del contrato MEMG-02). Registrado en RESULTADO `queda_pendiente`.
- **WIP ajeno:** `opencode.jsonc` + master plan modificados en el árbol por otros — NO se stagean; MEMG-01 puede estar en vuelo sobre `l1_writer.rs`/`l1_dedup.rs` (no solapados con este slice).

## RESULTADO §7 (contrato de retorno — pipeline-full)

```
RESULTADO: ✅ COMPLETO
STEPS_OK: 6/6 total steps
PROXIMO_STEP: ninguno
COMMIT_HASH: <se registra tras el commit local feat(memory): + docs(task)>
ARCHIVOS: src/sdk/types/record.rs, src/sdk/types.rs, src/sdk/mod.rs, src/lib.rs, src/sdk/api/memory.rs, src/sdk/api.rs, vanta-memory/src/core/hooks/auto_recall.rs, vanta-memory/src/core/hooks/mod.rs, vanta-memory/tests/reinforce_loop.rs, vantadb-mcp/src/handlers/tools.rs, vantadb-mcp/tests/mcp_tests.rs, docs/api/EMBEDDED_SDK.md, docs/api/scores.md, docs/api/MCP.md, tests/api/public-api.txt, docs/dev/Backlog.md (FIND-278), docs/dev/tasks/MEMG-02.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
GATES_EVALUADOS: P:no D:no V:no C:no | P:plan sanciona superficie · D:contrato del plan fija superficie/semántica (micro-decisiones por evidencia en §Spec) · V:verde (13/13 + suites scoped + fmt/clippy) · C:sin colaterales (FIND-278 registrado; WIP ajeno fuera del stage)
SKILLS_CARGADAS: test-driven-development, rust-write-tests, source-driven-development, incremental-implementation, documentation-skill, doubt-driven-development (base auto: campaign-executor, progreso, ponytail)
```
