# MEM-69: Batch extracción costo-reducida

## Metadata
- **Plan file:** docs/dev/plans/2026-09-10-code.md
- **Fuente:** plan file Task 2 (Wave0 con PRX-02 y DESKTOP-40-slice2, disjuntos)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟢 Media
- **Tipo:** Rust (feature-add: símbolos públicos nuevos — ver Phase 1b)
- **Turns estimados:** 15-20
- **Creado:** 2026-09-10T00:00
- **last-synced:** 2026-09-10T00:00
- **Estado:** ⏳ IN PROGRESS
- **Incógnitas (uphill):** 0 (verificación real del plan: `batch_dedup` + `PendingMemory` + `ScriptedRunner` existen)
- **Pendientes (downhill):** 5 steps de ejecución restantes

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vanta-memory/src/services/pipeline_worker.rs:428-449` (`run_l1_inner`: extract → dedup, 2 llamadas LLM por flush); tests `vanta-memory/tests/l1_dedup.rs`, `tests/l1_extractor.rs`, `tests/conversation_hook.rs` |
| Callees | `core/prompts/l1_extraction.rs` (system+user prompt), `core/prompts/l1_dedup.rs` (`CandidateMatch`, pool+judge prompt), `core/record/l1_reader.rs` (`recall_candidates`), `offload/local_llm/parsers/l1_parser.rs` (`parse_l1_extraction`, `memory_from_value`, `normalize_type`), `offload/local_llm/parsers/json_utils.rs` (`extract_json`), `core/record/l1_writer.rs` (`generate_memory_id`), `core/abstractions` (`LlmRunner`, `ExtractedMemory`, `DedupDecision`) |
| Implicaciones | Contratos existentes intactos (solo aditivo: nuevo módulo + `pub(crate)` visibility ×2 + helper; `extract_l1_segments`/`batch_dedup` sin cambio de firma ni comportamiento). Sin cambio de comportamiento público (nuevo símbolo opt-in, pipeline_worker NO se toca en este slice). Sin impacto CPU/memoria (LLM-bound; el prompt combinado ≈ suma de los dos actuales menos 1 system prompt → −40/50% tokens por flush). Sin migración de datos. Tests existentes no afectados (verificado por suite verde al cierre). |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vanta-memory/src/core/record/l1_extractor.rs` (260L), `vanta-memory/src/core/record/l1_dedup.rs` (533L), `vanta-memory/src/core/record/mod.rs` (42L), `vanta-memory/src/core/prompts/l1_extraction.rs` (253L), `vanta-memory/src/core/prompts/l1_dedup.rs` (259L), `vanta-memory/src/services/pipeline_worker.rs` (líneas 380-479), `vanta-memory/src/core/abstractions/llm_runner.rs` (líneas 1-130), `vanta-memory/src/core/record/l1_writer.rs` (líneas 230-309), `vanta-memory/src/offload/local_llm/parsers/l1_parser.rs` (líneas 1-196), `vanta-memory/tests/l1_extractor.rs` (líneas 1-100, patrón `CapturingRunner`)
- **Archivos referenciados hacia dentro (imports/includes/dependencias):** l1_batch dependerá de: `core::abstractions::{DedupAction,DedupDecision,ExtractedMemory,L1ExtractionResult,LlmRunParams,LlmRunner,MemoryRecord,MemoryType}`, `core::conversation::L0Message`, `core::prompts::l1_extraction::{extract_memories_system_prompt,format_extraction_prompt,PromptMode}`, `core::prompts::l1_dedup::CandidateMatch`, `core::record::l1_extractor::{should_extract_l1,split_messages (nuevo helper),L1ExtractorConfig}`, `core::record::l1_dedup::{decision_from_value (→pub(crate)),L1DedupConfig}`, `core::record::l1_reader::recall_candidates`, `core::record::l1_writer::generate_memory_id`, `offload::local_llm::parsers::{json_utils::extract_json,l1_parser::memory_from_value (→pub(crate))}`
- **Archivos que referencian a los editados (referencias entrantes):** `l1_extractor.rs` ← `pipeline_worker.rs:428`, `tests/l1_extractor.rs`; `l1_dedup.rs` ← `pipeline_worker.rs:441`, `record/mod.rs:33`, `tests/l1_dedup.rs`, `tests/conversation_hook.rs`; `l1_parser.rs` ← `l1_extractor.rs:23`, `l1_dedup.rs:24`
- **Veredicto impacto:** BAJO — 1 archivo nuevo + 4 ediciones mínimas (helper aditivo, 2 visibility `fn`→`pub(crate)`, 2 líneas re-export). Ninguna firma existente cambia. Riesgo principal (pre-mortem): batch degrada calidad dedup → mitigado con test comparativo split-vs-batch con `ScriptedRunner` (conteo llamadas mock-dependiente → runner scriptado local con contador, patrón `CapturingRunner` de `tests/l1_extractor.rs`).

## Contrato

`cargo test -p vanta-memory` 0 failed + test batch agrupa split+dedup en 1 llamada ✅ + quality gate intacto + clippy 0

## Spec (SDD — Phase 1b detectó feature-add: `pub fn extract_dedup_batch` + `pub mod l1_batch` + consts nuevos)

| # | Decisión | Opciones (+tradeoff) | Default recomendado | Resuelto |
|---|----------|----------------------|--------------------|----------|
| 1 | Dónde vive el batch | A) nuevo `l1_batch.rs` (aislado, aditivo) / B) dentro de `l1_dedup.rs` (acopla extracción en dedup) | A | ✅ decidido-por-evidencia: `mod.rs` ya separa `l1_extractor`/`l1_dedup` por MEM (ref: `mod.rs:6-25`) |
| 2 | Forma del resultado | A) tupla `(L1ExtractionResult, Vec<PendingMemory>, Vec<DedupDecision>)` (espeja `extract_l1_segments`) / B) struct nuevo `L1BatchResult` (más superficie pública) | A | ✅ decidido-por-evidencia: `extract_l1_segments` devuelve tupla `(result, memories)` (ref: `l1_extractor.rs:72-77`) |
| 3 | Cómo viaja el juicio dedup en 1 llamada | A) objeto `dedup` inline por memoria en el JSON de escenas (parse 1 pasada, alineación por construcción) / B) dos arrays `{scenes, decisions}` (riesgo desalineación si el parser tolerante dropea ítems) | A | ✅ decidido-por-evidencia: `parse_l1_extraction` dropea ítems malformados vía `filter_map` (ref: `l1_parser.rs:46-53`) → zip por índice sería frágil |
| 4 | Pool de existentes en el prompt | A) recall LLM-free por mensaje nuevo (`recall_candidates`, unión por id, acotado) / B) todos los records de la sesión (no acotado) | A | ✅ decidido-por-evidencia: `recall_candidates` ya es el gate de fase 1 con top_k (ref: `l1_dedup.rs:127-141`) |
| 5 | `now_ms` para ids deterministas | A) parámetro (tests deterministas) / B) reloj interno como `run_l1_dedup` | A | ✅ decidido-por-evidencia: `prepare_pending(memories, now)` ya toma tiempo como parámetro (ref: `l1_dedup.rs:109`) |
| 6 | Alcance pipeline_worker | A) NO tocar (primitiva opt-in + follow-up wiring) / B) cablear con flag | A | ✅ Gate P/appetite: wiring cambia comportamiento del flush para todos; slice 2 separado (ver Deuda) |

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:** (1) Principio 4: fallo LLM → L0 intacto, degradación a `success:false`/store-all, nunca pérdida de datos. (2) Quality gate `should_extract_l1` filtra TODO lo que el extractor actual filtra (ruido framework, slash, símbolos). (3) 1 memoria de entrada → exactamente 1 decisión (alineación por construcción). (4) `extract_l1_segments`/`batch_dedup`/`run_l1_dedup` sin cambios de comportamiento. (5) `unwrap`/`expect` prohibido en código nuevo (solo en tests, con `ponytail: blanket allow` como `tests/l1_extractor.rs:1`).
- **Comandos de verificación:** `cargo test -p vanta-memory` (0 failed) · `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` (0) · `cargo fmt --check` (limpio)
- **Deuda pendiente:** wiring en `pipeline_worker.rs` (`run_l1_inner`) → slice 2 follow-up (requiere decisión de rollout + test de worker). Test comparativo usa runner scriptado (no LLM real): equivalencia con LLM real queda como validación manual futura.

## Recitation (canónico — estructura única)

| Campo recitation (MCP) | ← fuente en este task file |
|------------------------|----------------------------|
| `activeGoal` | MEM-69 — Batch extracción costo-reducida |
| `lastAction` | (se actualiza por step) |
| `result` | PARTIAL (steps pendientes) |
| `nextAction` | Step 1: RED — test `tests/l1_batch.rs` con `ScriptedRunner` contador |
| `contract` | `## Contrato` + `## Invariantes de dominio` |
| `nextTask` | MEM-70 (misma Wave1 no — Wave0: informar a orquestador) |

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda nueva (código aditivo + tests; `decision_from_value`/`memory_from_value` visibility `pub(crate)` no es deuda). Moneda de pago N/A.

## Definition of Done (contrato multi-nivel — P2-08)

| Nivel | Gate |
|-------|------|
| **Task** | Contrato ✅ + `cargo test -p vanta-memory` 0 failed + clippy 0 + fmt limpio |
| **Commit** | Commit atómico, `feat: MEM-69 — ...`, SOLO archivos propios en develop, verificación mecánica previa |
| **Release** | N/A (no release; justificado: tarea de código en develop, release vía release-plz a main) |

## Herramientas necesarias
- cargo check/clippy/fmt/nextest (terminal)
- codegraph_explore (blast radius — ya ejecutado en DISCOVERY)

**Skills cargadas (SDP):** test-driven-development (lógica nueva — RED→GREEN→REFACTOR, patrón CapturingRunner/ScriptedRunner) · incremental-implementation (1 slice vertical: batch + tests + commit) · context-engineering (jerarquía Rules→Plan→Source→Error) · api-and-interface-design (nuevo símbolo público `extract_dedup_batch` — boundary del módulo) · doubt-driven-development (base tipo Rust core) · source-driven-development (base tipo Rust core). Descartadas de SDP: frontend-ui-engineering (no hay UI/web en este slice — score lifecycle genérico, no aplica).
**SDP:** test-driven-development, incremental-implementation, context-engineering, api-and-interface-design, doubt-driven-development, source-driven-development

## Investigation Notes
- Sin web research (cero ambigüedad de APIs externas; todo el material está en el código: patrones `CapturingRunner` en `tests/l1_extractor.rs`, `ScriptedRunner` en `l1_dedup.rs:364-374`, parse tolerante en `l1_parser.rs`/`json_utils.rs`).
- Gate D evaluado: símbolos públicos nuevos SON el deliverable explícito del contrato (plan owner-aprobado vía Gate P 2026-09-10); blast radius 5 archivos/1 módulo, aditivo, LLM-bound (no hot path CPU) → Gate D considerado, NO disparado (sin `question`; reportado en RESULTADO).

## Incógnitas (uphill) vs Pendientes (downhill) — P2-03

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 0 |
| Pendientes de ejecución (downhill) | 5 steps |
| % completado | 10% (DISCOVERY ✅) |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — NO aplica: no hay trust boundary nuevo (parse JSON tolerante ya existente reutilizado; sin input de usuario directo, sin auth, sin dependencias nuevas, sin FFI). Justificado.
- [x] **PERFORMANCE** — NO aplica bench Regla 9 (no es hot path CPU: llamadas LLM por red; la métrica es conteo de llamadas, verificada por test con runner contador: 2→1 por flush). Prompt combinado ≈ suma de ambos menos 1 system prompt. Justificado.

## Steps

### Step 1: RED — test batch 1 llamada + quality gate + comparativo (tests/l1_batch.rs nuevo)
- **Archivos:** `vanta-memory/tests/l1_batch.rs` (nuevo, ~150L, patrón `CapturingRunner` de `tests/l1_extractor.rs`)
- **Acción:** escribir tests que FALLAN (símbolo `extract_dedup_batch` aún no existe): (a) batch con mensajes + existentes → exactamente 1 llamada; (b) solo ruido → 0 llamadas + success; (c) comparativo split-vs-batch: mismas memorias + mismas acciones con runner scriptado por `task_id`; (d) fallo runner → `success:false`; (e) `dedup` ausente/malformado → `store` fallback
- **Verify:** `cargo test -p vanta-memory --test l1_batch` falla por símbolo inexistente (RED válido)
- **Estado:** ✅ DONE (RED confirmado 2026-09-10: `E0432 unresolved imports extract_dedup_batch/EXTRACT_DEDUP_TASK_ID`; test file 379L heredado del intento previo + 1 ajuste metadata `{}` en cierre)

### Step 2: GREEN — split helper + visibility (l1_extractor.rs, l1_parser.rs, l1_dedup.rs)
- **Archivos:** `vanta-memory/src/core/record/l1_extractor.rs` (+`pub(crate) fn split_messages`, refactor mínimo de `extract_l1_segments`), `vanta-memory/src/offload/local_llm/parsers/l1_parser.rs` (`memory_from_value` → `pub(crate)`), `vanta-memory/src/core/record/l1_dedup.rs` (`decision_from_value` → `pub(crate)`)
- **Acción:** cambios mínimos aditivos; refactor usa el helper (mismo comportamiento)
- **Verify:** `cargo check -p vanta-memory` + `cargo test -p vanta-memory --test l1_extractor --test l1_dedup` verdes (sin regresión)
- **Estado:** ✅ DONE (heredado del intento previo: `split_messages` pub(crate) + refactor idéntico + 2 visibility pub(crate); verificado sin regresión en suite final 0 failed)

### Step 3: GREEN — módulo l1_batch.rs (prompt + 1 llamada + parse)
- **Archivos:** `vanta-memory/src/core/record/l1_batch.rs` (nuevo, ~200L), `vanta-memory/src/core/record/mod.rs` (2 líneas re-export)
- **Acción:** `EXTRACT_DEDUP_TASK_ID`, `extract_dedup_system_prompt`, `format_extract_dedup_prompt`, `parse_batch_response`, `extract_dedup_batch`
- **Verify:** `cargo test -p vanta-memory --test l1_batch` verde
- **Estado:** ✅ DONE (2026-09-10: `l1_batch.rs` ~260L + re-export 4L en `mod.rs`; 7/7 tests verdes; fixes clippy `manual_contains`+`clone_on_copy`+fmt)

### Step 4: VERIFY full — suite + clippy + fmt
- **Archivos:** (ninguno — verificación)
- **Acción:** `cargo test -p vanta-memory` 0 failed + `cargo clippy -p vanta-memory --all-targets --all-features -- -D warnings` 0 + `cargo fmt --check` limpio (bash directa si `campaign_verify_cmd` falla con exit -1, precedente del plan)
- **Verify:** los 3 comandos verdes
- **Estado:** ✅ DONE (2026-09-10: suite `cargo test -p vanta-memory --tests -j 2` 0 failed en todos los targets incl. lib 328 + l1_batch 7; clippy `--all-targets --all-features -D warnings` 0; `cargo fmt --check` limpio. Nota: `cargo test` a jobs default crashea rustc 1.95 con STATUS_STACK_BUFFER_OVERRUN en targets ajenos — flake de toolchain, con `-j 2` todo verde)

### Step 5: CLOSE — commit + sync plan + progreso
- **Archivos:** SOLO `docs/dev/tasks/MEM-69.md`, `vanta-memory/tests/l1_batch.rs`, `vanta-memory/src/core/record/l1_batch.rs`, `l1_extractor.rs`, `l1_dedup.rs`, `l1_parser.rs`, `record/mod.rs` (NO stagear: `opencode.jsonc`, `.opencode`, `desktop/src-tauri/Cargo.lock`, `Investigacion-plan.md`, `docs/dev/plans/2026-09-10-code.md` es del plan — actualizar estado Task 2 inline SÍ está permitido por pipeline-full)
- **Acción:** `git add` selectivo + `git commit -m "feat: MEM-69 — batch extracción costo-reducida (split+dedup en 1 llamada)"` + actualizar plan file Task 2 → ✅ + `skill progreso`
- **Verify:** `git log --oneline -1` + `git status --short` sin restos propios
- **Estado:** ✅ DONE (2026-09-10: commit `d5ad0e45` con 7 archivos propios; hooks pre-commit fmt+clippy verdes; resto ajeno PRX-02/desktop/opencode intacto en worktree)

## Dependencias
- Wave0 paralelo MAX 3: PRX-02 + DESKTOP-40-slice2 (archivos disjuntos — sin coordinación necesaria)

## Review (GATE — agente distinto, P2-01)

- **Revisor:** self-review adversarial (doubt-driven degradado: sin sub-agente disponible en este runner; cross-model skip — contexto no-interactivo, anunciado)
- **Enfoque:** alineación 1:1 memoria→decisión; cero cambio de comportamiento en paths existentes; quality gate idéntico
- **Cómo se probó:** `cargo test -p vanta-memory --tests -j 2` 0 failed + clippy 0 + fmt limpio; test comparativo split-vs-batch con runner scriptado (batch≡split en contenido/tipo/prioridad/acción, 2 llamadas→1)
- **Checklist anti-hábitos tóxicos:** sin `unwrap/expect` en código nuevo ✅; sin duplicación de parsers (reusa tolerancias exactas) ✅; sin scope creep (pipeline_worker intacto, wiring en Deuda) ✅; sin WIP ajeno stageado ✅
- **Veredicto:** PASS (con nota: equivalencia con LLM real queda como validación manual futura — ver Deuda)

## Notas
- Patrón Memobase citado en el plan (Gate Justificación): −40/50% tokens por flush al fusionar extracción+dedup en 1 llamada.
- Stop condition del plan: quality baja → DEFER con números. El test comparativo del Step 1c es el instrumento: si batch diverge del split en acciones con el mismo script, se mide y se decide DEFER.
- `campaign_verify_cmd` bug exit -1 (precedente del plan) → usar bash directa para verify.
- Deuda/WIP ajeno en worktree (NO tocar ni stagear): `M opencode.jsonc`, `M .opencode`, `M desktop/src-tauri/Cargo.lock`, `?? Investigacion-plan.md`.
