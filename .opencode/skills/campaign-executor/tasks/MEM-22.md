# MEM-22: Context Engine — assemble + cascada mild/aggressive

## Metadata
- **Plan file:** `docs/plans/2026-08-21-vanta-context-engine.md` (Task 5)
- **Fuente:** plan file Task 5 (MEM-22)
- **Esfuerzo:** 🔴 | **Prioridad:** 🔴 (killer feature F5)
- **Tipo:** Rust (crate `vanta-memory`)
- **Creado:** 2026-08-21
- **Estado:** ✅ COMPLETADA
- **Appetite:** max 3d — stop condition: excedido → mild-only

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** pipeline-full.md (247); plan file Task 5 (contrato/pre-mortem/risk register); task files `MEM-23.md` (83, plantilla + invariantes), crate: `lib.rs` (51 — `pub mod context_engine` ya existe), `context_engine/mod.rs` (11), `context_engine/types.rs` (103 — ChatRole/ChatMessage/CompactionMode/CompactionReport/ContextError), `context_engine/token_estimator.rs` (268 — TokenEstimator chars/3, emergency_truncate con build_units pair-guard, truncate_content); `offload/state_manager.rs` (grep cursor: `last_offloaded_tool_call_id`/`set_last_offloaded_tool_call_id`, persistido en PluginState); TDAM clone @97f9465: `offload/hooks/llm-input-l3.ts` (:113-117 consts MIN=10/INITIAL=7/FLOOR=1/MIN_KEEP=2; :402-576 compressByScoreCascade; :530-538 guard summary>original revert; :633-653 computeAggressiveDeleteCount con regla mínima 20%; :655-660 adjustDeleteCountForToolPairing; :667-751 aggressiveCompressUntilBelowThreshold one-shot; :588-618 capDeleteCountForUserMessage), `offload/index.ts` (:115-129 simpleHash + _msgFingerprint role+200chars; :1481-1523 FP-BOUNDARY-DELETE re-aplicación), `offload/state-manager.ts` (:96-101 `_lastAggressiveBoundary{originalIndex,fingerprint,keptMsgCount,remainingTokens}`), `offload-client/context-engine.ts` (:445-482 assemble ratio<compactionRatio skip), `offload/types.ts:28` (score = replaceability 0-10, mayor = summary puede reemplazar mejor)
- **Referencias hacia dentro:** módulo consume SOLO `context_engine::{types, token_estimator}` (MEM-23) + std. Sin deps nuevas. Sin tocar core `vantadb`.
- **Referencias entrantes:** ninguna hoy (módulo nuevo). Ediciones a existentes: `mod.rs` (+2 líneas re-export). El cursor MEM-20 (`OffloadStateManager`) NO se importa: el motor queda puro — `assemble` recibe `protected_prefix: usize` derivado del cursor por el caller (mensajes ≤ cursor ya offloaded, jamás se recomprimen). Mapeo documentado en docs del módulo.
- **Veredicto impacto:** bajo — 2 archivos nuevos (`engine.rs`, `compressor.rs`) + 1 test file nuevo + re-exports en `mod.rs`. Cero callers rotos.

## Contrato

"`cargo check -p vanta-memory` pasa; tests D19: (a) assemble ratio<0.5 → skip sin tocar mensajes; (b) mild cascade conserva los top-score hasta bajar del presupuesto, nunca parte pares tool_call; (c) summary más largo que original se revierte (guard TDAM); (d) aggressive one-shot baja bajo umbral y el fingerprint del boundary hace idempotente la re-aplicación; (e) report expone modo/msgs conservados/tokens antes-después; (f) 100% LLM-free (sin runner)"

## Diseño (puente TDAM → Rust, decisiones)

| Pieza TDAM | Acción MEM-22 |
|---|---|
| `assemble` (context-engine.ts:445) | `engine::assemble(msgs, token_budget, estimator, protected_prefix, cfg) -> Result<AssembleOutput, ContextError>`; ratio = tokens/budget; `ratio < cfg.compaction_ratio (0.5)` → mode None, mensajes intactos |
| score = replaceability (types.ts:28) | `compressor::score_message`: base por rol (ToolResult=6 > ToolCall=5 > Assistant=4 > User=2) + bonus antigüedad 0..4 (más viejo = más alto); System excluido. Determinista, LLM-free (sustituye al score L1 del LLM) |
| `compressByScoreCascade` (:402) | `engine::mild_cascade`: unidades atómicas (reusa semántica build_units), sort desc por score, umbral 7→FLOOR 1: reemplaza contenido de cada mensaje de la unidad con stub `[compacted N chars]`; para cuando baja del presupuesto o agota umbrales; cap MIN_COUNT=10 reemplazos/pasada |
| Guard :530-538 | si `stub.len() >= original.len()` → revertir (dejar original intacto, no contar como reemplazo) |
| `aggressiveCompressUntilBelowThreshold` (:667) | `engine::aggressive_one_shot`: delete-count acumulativo desde head (respeta protected_prefix y min_keep=2), regla mínima 20% (:648-651), extiende corte pasado tool_results huérfanos (:655), cap en último User (:611), splice único |
| `_msgFingerprint` (:121) + `_lastAggressiveBoundary` (state-manager.ts:96) | `compressor::msg_fingerprint` = simpleHash i32 de `"role:{primeros 200 chars}"`; `AggressiveBoundary{original_index, fingerprint, kept_msg_count}` devuelto en AssembleOutput; `compressor::apply_boundary` re-aplica el head-delete verificando fingerprint (None si mismatch → caller limpia) |
| Cursor MEM-20 (pre-mortem 3) | `protected_prefix` en assemble: región compactable = `[prefix .. len-min_keep)`; aggressive nunca borra dentro del prefijo |

## Invariantes de dominio (handoff - MUST)

1. Sin deps nuevas; sin unwrap/expect en producción; errores tipados `#[non_exhaustive]`.
2. Un par tool_call/tool_result JAMÁS partido (unidades atómicas + extensión de corte).
3. LLM-free 100% (Principio 4) — sin `LlmRunner`.
4. NO tocar core `vantadb`; wiring aditivo solo en `context_engine/mod.rs`.
5. Mensajes del prefijo protegido (cursor) jamás modificados ni eliminados.
6. Último User siempre conservado; min_keep=2 mensajes finales intactos.

## Steps

### Step 1 — Discovery + task file
- [x] Leer TDAM refs + APIs del crate (reads + grep cursor)
- [x] Crear task file (este) con Impacto mapeado Regla 0
- **Gate:** ✅ registro antes de tocar código

### Step 2 — compressor.rs (fingerprint + scoring + boundary)
- [x] `msg_fingerprint` (simpleHash TDAM) + `AggressiveBoundary` + `apply_boundary`
- [x] `score_message` heurística replaceability + consts MIN/INITIAL/FLOOR
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 3 — engine.rs (assemble + mild cascade + aggressive one-shot)
- [x] `assemble` con skip ratio<0.5, mild_cascade con guard (c), aggressive_one_shot, fallback emergency_truncate (prefix-aware: opera solo sobre la región compactable)
- [x] AssembleOutput {messages, report, boundary}
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 4 — tests D19 (tests/context_engine.rs)
- [x] (a)-(f) contrato completo + test cursor protected_prefix
- **Gate:** ✅ `cargo nextest run -p vanta-memory` 408/408 verde

### Step 5 — Verify completo + cierre
- [x] check + nextest -p vanta-memory + fmt --check + clippy -p vanta-memory --all-targets --no-deps -- -D warnings → todo exit 0
- [x] CIERRE: campaign_update_task_state taskId=5 completed + recitation §3; bloque RESULTADO §7
- **Gate:** ✅ verify todo exit 0 (commit omitido por instrucción del orquestador: NO commitees)

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Techos documentados: scoring heurístico sustituye al score LLM L1 (upgrade path post-MEM-24: consumir scores reales de offload entries); stub `[compacted N chars]` es placeholder sin semántica (refs re-leen a demanda — trade-off 05 §7 documentado en docs del módulo).

## Recitation (canónico)

=== RECITATION ===
- **activeGoal:** MEM-22 — Context Engine assemble + cascada mild/aggressive (Task 5, plan 2026-08-21-vanta-context-engine)
- **lastAction:** Implementados `compressor.rs` (msg_fingerprint role+200chars simpleHash i32, score_message heurístico ToolResult6>ToolCall5>Assistant4>User2 + bonus edad 0..4, AggressiveBoundary + apply_boundary), `engine.rs` (assemble con ratio-gate <0.5 → mild_cascade umbrales 7→1 cap 10 reemplazos → aggressive_one_shot head-delete ≥20% → emergency prefix-aware), wiring en mod.rs, tests D19 (a)-(f) + protected_prefix
- **result:** OK
- **nextAction:** Ninguna para MEM-22. Próxima tarea del plan (Task 6) vía orquestador
- **contract:**
  - verificacion: `cargo check -p vanta-memory` ✅ · `cargo nextest run -p vanta-memory` ✅ 408/408 · `cargo fmt --check` ✅ · `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` ✅
  - evidencia:
    - claim: contrato D19 (a)-(f) cubierto | evidencia: vanta-memory/tests/context_engine.rs (7 tests) | confianza: alta
    - claim: pares tool_call jamás partidos | evidencia: unidades atómicas build_units (pub(crate)) + assert_no_orphan_results en tests | confianza: alta
    - claim: prefijo protegido (cursor MEM-20) jamás tocado por ninguna pasada | evidencia: test protected_prefix_never_touched; emergency fallback parte en protected_prefix | confianza: alta
    - claim: fingerprint idempotente | evidencia: test d_aggressive_one_shot_boundary_idempotent (re-aplicación = mismo output; sobre historia compactada → None) | confianza: alta
  - artefactos: vanta-memory/src/context_engine/{compressor.rs, engine.rs, mod.rs, token_estimator.rs (build_units pub(crate))}, vanta-memory/tests/context_engine.rs
  - invariantes: sin deps nuevas · sin unwrap/expect en código nuevo · LLM-free 100% · core `vantadb` intacto · último User + min_keep=2 intactos
  - deuda: scoring heurístico sustituye score LLM L1 (upgrade post-MEM-24); stub `[compacted N chars]` sin semántica; si el prefijo protegido solo excede el budget, assemble devuelve over-budget en vez de violar el cursor
  - queda_pendiente: commit (orquestador ordenó NO commitear); lead decide commit + siguiente tarea
- **nextTask:** Task 6 del plan
=== END RECITATION ===
