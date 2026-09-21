# MEM-41 — Generation-log provenance (L1/L2/L3 consultable)

Plan: `docs/plans/2026-08-21-vanta-context-engine.md` Task 3 · Ruta: vanta-worker
Contrato: `cargo check -p vanta-memory` pasa; tests D19: (a) cada generación L1/L2/L3 exitosa registra entry {layer, status, anchor_id, session, ts}; (b) fallo LLM registra status=failed BEST-EFFORT (nunca bloquea pipeline — Principio 4); (c) consulta por session/layer ordenada por ts.
Stop condition: wiring >5 archivos existentes → API standalone sin hooks automáticos.

## Impacto mapeado (Regla 0)

**Archivos leídos completos (vía codegraph_explore verbatim):**
- `vanta-memory/src/core/record/l1_writer.rs` (237L) — `write_memory`, `apply_dedup_batch`, `put_record`
- `vanta-memory/src/core/scene/scene_extractor.rs` — `extract_scenes_with_llm` (L353), `extract_scenes`, `apply_strategy`
- `vanta-memory/src/core/persona/persona_generator.rs` — `generate_persona` (L203), `write_persona`
- `vanta-memory/src/services/pipeline_worker.rs` — `run_l1/run_l2/run_l3` (L190-325)
- `vanta-memory/src/core/record/l1_reader.rs` (217L) — patrón namespace sanitizado + lectura paginada
- `vanta-memory/src/core/conversation/l0_recorder.rs` — `sanitize_component`/`sanitize_key` (L106-125)
- `vanta-memory/src/adapters/mock.rs` (96L) — MockLlmRunner para tests
- `vanta-memory/Cargo.toml` — deps: serde/serde_json/thiserror/tracing (todas presentes, sin deps nuevas)
- TDAM ref: `memory-generation-log/{types,store,best-effort}.ts` (277L) — fuente de diseño

**Referencias hacia dentro (entrantes):**
- `write_memory`: 5 callers (l1_writer interno, record/mod.rs; tests l1_dedup.rs)
- `extract_scenes_with_llm`: 5 callers (pipeline_worker run_l2, scene/mod.rs; tests scene_strategy.rs)
- `generate_persona`: 16 callers (pipeline_worker run_l3, persona/mod.rs; tests persona.rs)
- Ninguno de los 3 cambia firma → blast radius = cero en callers.

**Referencias hacia afuera (salientes del módulo nuevo):**
- `vantadb::sdk::{VantaEmbedded, VantaMemoryInput, VantaMemoryMetadata}` (put/get/list/delete)
- `crate::core::conversation::sanitize_component`

**Veredicto:** impacto BAJO. Módulo nuevo aislado (`core/memory_generation_log/`) + hooks aditivos
en 4 archivos existentes (l1_writer, scene_extractor, persona_generator, core/mod.rs decl) +
pipeline_worker run_l1 (log failed L1) = 5 archivos existentes ≤ stop condition. Sin cambios de
firmas públicas existentes. Namespace `genlog/<session>` (sanitizado, patrón P27).

## Diseño

- **Entry schema** (mínimo D19): `{layer: l1|l2|l3, status: succeeded|failed, anchor_id: Option<String>, session_key: String, ts_ms: u64, error: Option<String>}`
- **Persistencia:** ns `genlog/<session>` (sanitize_component 128), key `{ts_ms:013}_{seq}` (zero-pad → orden lexicográfico = cronológico), payload = JSON entry.
- **Best-effort:** `try_record() -> Result` + `record_best_effort()` que traga error con `tracing::warn!` (TDAM best-effort.ts). Nunca propaga → nunca bloquea pipeline (P4).
- **Cap:** MAX_ENTRIES_PER_SESSION = 100 keep-recent (borra oldest al exceder).
- **Query:** `query_session(db, session, layer: Option<Layer>) -> Result<Vec<Entry>>` ordenado por ts asc.
- **Hooks aditivos:**
  - L1: `write_memory` tras put exitoso → succeeded, anchor_id = record.id (Skip → no log).
  - L2: wrapper en `extract_scenes_with_llm` → success/failure según resultado (anchor = primera escena aplicada).
  - L3: wrapper en `generate_persona` → success/failure según resultado.
  - L1-failed: `run_l1` en pipeline_worker registra failed cuando la etapa falla (el writer no ve fallos LLM).

## Steps

### Step 1 — Módulo standalone + tests D19 (a)(c) + cap ✅ DONE
- `core/memory_generation_log/{mod,store}.rs` creados + declarados en `core/mod.rs`
- Verify: `cargo nextest run -p vanta-memory memory_generation_log` → 7/7 PASS

### Step 2 — Hooks aditivos L1/L2/L3 + test fallo LLM (b) ✅ DONE
- `l1_writer.rs`: log succeeded post-put (ambas ramas Store/Update-Merge, anchor = record.id)
- `scene_extractor.rs` / `persona_generator.rs`: wrapper inner+log (solo generaciones reales o fallos; skip/empty no loguea)
- `pipeline_worker.rs`: run_l1 wrapper → failed log
- Verify: `cargo nextest run -p vanta-memory --test generation_log` → 4/4 PASS

### Step 3 — Verify mecánico + cierre ✅ DONE
- `cargo check -p vanta-memory` → exit 0
- `cargo fmt --check -p vanta-memory` → exit 0
- `cargo clippy -p vanta-memory --all-targets` → exit 0 (0 warnings propios; 7 pre-existentes en vantadb core d5624082, fuera de blast radius)
- `cargo nextest run -p vanta-memory` → 389/389 PASS (378 previos + 11 nuevos)
- Sin commit (regla explícita de la invocación: NO commitear)

## Context Save Point
(ninguno aún)
