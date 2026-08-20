# MEM-34: F1 Core Telemetría por capa (L1/L2/L3/recall/offload + persona) + audit memory

## Metadata
- **Plan file:** `docs/plans/2026-08-18-vanta-memory.md`
- **Creado:** 2026-08-20T12:30
- **last-synced:** 2026-08-20T12:30
- **Estado:** ⏳ IN PROGRESS

## Blast Radius
**Callers** (del código que voy a tocar):
- `operational_metrics_snapshot()` (src/metrics/core/mod.rs:545) — 23 callers: `src/tui/dashboard.rs`, tests en `mod.rs`; el valor retorna a `VantaEmbedded::operational_metrics()` (src/sdk/api.rs:1271) → `VantaOperationalMetrics` (DTO) → `/api/v2/metrics` (cli_server.rs:588) + desktop `vanta_metrics` (desktop/src/vanta.ts:496).
- `VantaOperationalMetrics` (src/sdk/types.rs:345) — 3 conversiones consumen campo a campo: `conversions.rs:23` (From), `vantadb-wasm/src/lib.rs:208` (From → JsOperationalMetrics, strings), `vantadb-python/src/convert.rs:531` (pydict). Agregar campos NO rompe estas (mapean los que leen).
- `OperationalMetricsSnapshot` (src/metrics/core/snapshot.rs:41) — usado en `conversions.rs` From + test literal `test_operational_metrics_conversion` (conversions.rs:179-232) que construye el struct completo → **se rompe al agregar campos, hay que actualizarlo**.
- Audit: `/api/v2/audit` YA EXISTE (cli_server.rs:227, handler 1219) dentro del router `protected` con `auth_middleware` (cli_server.rs:256). `src/audit.rs` (AuditEvent/AuditLogger JSONL) ya existe (WEB-01).

**Callees:** `memory_breakdown_snapshot()` (mod.rs:514).

**Implicaciones:**
- Retrocompat JSON: el desktop TS declara subset (`OperationalMetrics` vanta.ts:469); campos nuevos en el wire son additive — Studio gana latencias sin código nuevo (contrato del task).
- `VantaOperationalMetrics` derive `PartialEq, Eq, Serialize` — los campos nuevos u64 no rompen derives.
- Nombres de campos siguen TDAM `01-core-pipeline.md:100-104`: `l1_extraction_latency_ms`, `l1_dedup_latency_ms`, `l2_extraction_latency_ms`, `l2_llm_duration_ms`, `l3_generation_latency_ms`, `persona_length_before/after`, `persona_drift_ratio`, `recall_hit_count`, `recall_top_score`, `recall_latency_ms`, `recall_strategy` (codificado 0=skipped/1=keyword/2=embedding/3=hybrid). Offload: `offload_latency_ms`.
- Restricción core LLM-free/WASM-compatible: campos AtomicU64 error-silent inicializados en 0; hooks mínimos de incremento; el crate futuro `vanta-memory` (F4) los alimenta.

## Impacto mapeado (Regla 0)
- **Archivos leídos completos:** `src/metrics/core/mod.rs` (953L), `src/metrics/core/snapshot.rs` (112L), `src/metrics/core/registry.rs` (parcial 120L), `src/sdk/types.rs` (345-484), `src/sdk/serialization/conversions.rs` (1-135 + test 135+), `src/cli_server.rs` (196-300, 575-620, 1190-1309), `src/audit.rs` (96L), `vantadb-server/src/lib.rs` (9L), `vantadb-wasm/src/lib.rs` (200-244), `vantadb-python/src/convert.rs` (520-589), `desktop/src/vanta.ts` (465-504).
- **Referencias hacia dentro (de lo que edito):** snapshot.rs ← mod.rs (snapshot fns), conversions.rs (From), tests; types.rs VantaOperationalMetrics ← conversions.rs, api.rs:1271, wasm lib.rs:208, python convert.rs:531, cli_server.rs:588.
- **Referencias hacia fuera:** mod.rs → registry.rs (histogramas feature-gated); audit.rs ← builder.rs:114-119 (VantaEmbedded.audit()), lib.rs:58, cli_server.rs:10.
- **Veredicto:** extender campos es additive; ÚNICO literal completo a actualizar: `test_operational_metrics_conversion`. NO crear `vantadb-server/src/audit.rs` (duplica infra existente en core; server wrapper delega en core). Desktop nativo (`desktop/src-tauri/src/commands/audit.rs`) se mantiene intacto.

## Contrato
`cargo check -p vantadb` pasa; tests dedicados de snapshot metrics (D19) pasan (`cargo nextest run -p vantadb metrics`).

## Herramientas
- codegraph, cargo/terminal (Rust), campaign_verify_cmd

## Steps
### Step 1: Campos nuevos en `OperationalMetricsSnapshot` + DTO
- **Archivos:** `src/metrics/core/snapshot.rs`, `src/sdk/types.rs`
- **Acción:** agregar 13 campos u64 documentados a `OperationalMetricsSnapshot` (l1_extraction_latency_ms, l1_dedup_latency_ms, l2_extraction_latency_ms, l2_llm_duration_ms, l3_generation_latency_ms, persona_length_before, persona_length_after, persona_drift_ratio [bps ×10000], recall_hit_count, recall_top_score [bps ×10000], recall_latency_ms, recall_strategy [0-3 TDAM], offload_latency_ms) y espejo en `VantaOperationalMetrics` (types.rs).
- **Verify:** `cargo check -p vantadb` (falla en conversión — esperado, se completa en Step 3)
- **Estado:** ✅ DONE

### Step 2: Statics AtomicU64 + record functions + snapshot wiring
- **Archivos:** `src/metrics/core/mod.rs`
- **Acción:** 13 statics AtomicU64 (patrón error-silent existente) + `pub enum RecallStrategy { Skipped=0, Keyword=1, Embedding=2, Hybrid=3 }` con `as_code()`; funciones `record_l1_extraction`, `record_l1_dedup`, `record_l2_extraction`, `record_l2_llm`, `record_l3_generation`, `record_persona(before, after, drift_ratio_bps)`, `record_recall(hit_count, top_score_bps, latency_ms, strategy)`, `record_offload`; poblar en `operational_metrics_snapshot()`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ DONE

### Step 3: Conversión From + fix test literal
- **Archivos:** `src/sdk/serialization/conversions.rs`
- **Acción:** mapear 13 campos nuevos en `From<OperationalMetricsSnapshot> for VantaOperationalMetrics`; actualizar `test_operational_metrics_conversion` con los campos nuevos.
- **Verify:** `cargo check -p vantadb` + `cargo nextest run -p vantadb --test metrics`
- **Estado:** ✅ DONE (también actualizados literales en types.rs tests y tests/sdk_serialization.rs)

### Step 4: Tests dedicados (D19)
- **Archivos:** `src/metrics/core/mod.rs` (mod tests)
- **Acción:** tests: defaults 0; record_l1/l2/l3/offload round-trip (fetch_max); record_persona (store semantics); record_recall acumula hit_count, strategy code correcto (Skipped→0, Keyword→1, Embedding→2, Hybrid→3), top_score store; snapshot incluye campos nuevos.
- **Verify:** `cargo nextest run -p vantadb --test metrics`
- **Estado:** ✅ DONE (7 tests nuevos, 7/7 PASS)

### Step 5: Helper audit para eventos memory (L1/L2/L3/offload)
- **Archivos:** `src/audit.rs` + test
- **Acción:** `AuditEvent::memory(layer: &str, namespace: &str, outcome: &str, reason: Option<String>)` → op `memory_{layer}` para eventos L1/L2/L3/offload; test JSONL round-trip (escribir con AuditLogger en tempfile + leer línea).
- **Verify:** `cargo nextest run -p vantadb audit`
- **Estado:** ✅ DONE (2 tests nuevos, 2/2 PASS)

### Step 6: Verify full + commit
- **Acción:** fmt/clippy/nextest/docs; commit conventional con MEM-34.
- **Verify:** `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo nextest run --profile audit --workspace --build-jobs 2`, `scripts/validate-docs-coverage.ps1`
- **Estado:** ✅ DONE (fmt 0, clippy 0, 2002/2002 nextest; docs coverage gaps `supersede` preexistentes, no de MEM-34)

## Dependencias
- MEM-01 ✅ (6a50b8ee — SearchProfileConfig), MEM-02 ✅ (32b09daf — MCP passthrough). Telemetría registra strategy usado por esos paths.

## Notas
- Nomenclatura TDAM: `docs/research/tdam/01-core-pipeline.md:100-104` (metric-tracking-{l1,l2,l3,recall}-latency). Recall strategy codificado 0..3.
- `persona_drift_ratio` y `recall_top_score` se guardan en basis points (×10000 del ratio 0..1) por compat con u64/AtomicU64/WASM — documentar en doc comments. El consumidor (Studio) divide por 10000.
- Audit server YA resuelto por WEB-01 (c81bc23a): `/api/v2/audit` + `src/audit.rs` JSONL + auth middleware. MEM-34 aporta helper `memory` para que F4 `vanta-memory` escriba eventos L1/L2/L3/offload en el mismo JSONL.
- Regla 6: no duplicar lógica — `vantadb-server/src/audit.rs` NO se crea.

## Context Save Point
- **Fecha:** 2026-08-20T12:30
- **Branch:** develop
- **CI pendiente:** sí (verify full en Step 6)
- **Decisiones:** campos u64 + basis points para ratios (AtomicU64/LLM-free/WASM-compat); audit memory helper en core (no duplicar en vantadb-server); nombres TDAM exactos para contrato wire.
- **Problemas conocidos:** `test_operational_metrics_conversion` (conversions.rs:179) se rompe al agregar campos — se actualiza en Step 3.
- **Próxima tarea:** Checkpoint F1 (tras F1+F2)