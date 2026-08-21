# MEM-23: Token estimator + emergency truncate + report types

## Metadata
- **Plan file:** `docs/plans/2026-08-21-vanta-context-engine.md` (Task 1)
- **Fuente:** plan file Task 1 (MEM-23)
- **Esfuerzo:** 🟢 | **Prioridad:** 🔴 (fundación de MEM-22)
- **Tipo:** Rust (crate `vanta-memory`)
- **Creado:** 2026-08-21
- **Estado:** ✅ COMPLETED

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** pipeline-full.md (247); plan file Tasks 1+5 (contrato CompactionReport: modo/msgs conservados/tokens antes-después); task file `MEM-21.md` (76, plantilla); crate: `lib.rs` (45 — wiring `pub mod X` aditivo), `core/conversation/l0_recorder.rs` (L0Role solo User/Assistant + patrón thiserror `L0Error`, tests inline), `core/hooks/auto_capture.rs` (RawMessage, sanitize pattern, tests inline), `core/abstractions/types.rs` (MemoryType/ExtractedMemory — serde snake_case, structs plain sin non_exhaustive), `core/record/l1_dedup.rs` (patrón config struct + tests inline); TDAM: `offload-client/token-estimator.ts` (extractLlmVisibleText role+"\n"+content, calibración 0.5–3.0 drift>15% — NO portar tiktoken), `offload/hooks/llm-input-l3.ts` (emergencyCompress: drop-from-front por ratio, MIN_KEEP, tail-delete, truncate oversized ~600-token guard), `offload/mmd-injector.ts:200-280` (adjustForToolCallPair: nunca insertar/partir entre assistant-tool_use y sus tool_results)
- **Referencias hacia dentro:** módulo nuevo consume SOLO std + serde + thiserror (ya deps del crate). Nada del core `vantadb`.
- **Referencias entrantes:** ninguna hoy — módulo nuevo. Única edición a archivo existente: `lib.rs` (agregar `pub mod context_engine;`). MEM-22 (Task 5) consumirá `types::{ChatMessage, CompactionReport, CompactionMode}` y `token_estimator::{TokenEstimator, emergency_truncate}`.
- **Veredicto impacto:** bajo — 3 archivos nuevos en `context_engine/` + 1 línea de wiring en `lib.rs`; cero callers rotos; NO se toca el core `vantadb`.

## Contrato

"`cargo check -p vanta-memory` pasa; tests D19: estimate_tokens determinista (casos conocido vacío/ascii/unicode), truncado respeta pares tool_call/tool_result (nunca los parte), CompactionReport serde roundtrip."

## Diseño (puente TDAM → Rust, decisiones)

| Pieza TDAM | Acción MEM-23 |
|---|---|
| Representación de mensajes (`any[]` con role string) | `types.rs`: `ChatRole` enum `{System, User, Assistant, ToolCall, ToolResult}` (serde snake_case) + `ChatMessage {role, content}`. DECISIÓN: tipo propio tipado, NO serde Value (el guard necesita discriminar roles sin parsear JSON) y NO L0Message (L0Role sin roles de tool). Host-neutral wire propio que MEM-22 consume |
| `fast-token-estimate.ts` / D21 | `TokenEstimator { chars_per_token: usize (default 3) }`; `estimate_tokens(&str) -> u64` = `chars().count() / cpt` (determinista, unicode-safe por chars no bytes); `estimate_message` = role + "\n" + content (paridad extractLlmVisibleText). Techo documentado: subestima CJK/código — aceptado por D21, calibración diferida post-MEM-22 |
| `emergencyCompress` (llm-input-l3.ts:755+) | `emergency_truncate(msgs, budget_tokens, estimator, min_keep)`: drop-from-front de UNIDADES ATÓMICAS — unidad = mensaje suelto o grupo `[ToolCall, ToolResult...]` contiguo (nunca se parte un par). Respeta `min_keep` mensajes finales. Si aún excede tras drops, trunca contenido del mensaje más grande restante vía `truncate_content` (char-boundary safe, marcador `…[truncated]`) |
| `adjustForToolCallPair` (mmd-injector.ts:231) | El guard vive en la construcción de unidades: escaneo lineal agrupa cada ToolCall con los ToolResult contiguos siguientes → imposible partir un par por construcción |
| `compaction-handler.ts` report `{messages, report}` | `CompactionReport {mode, msgs_conserved, msgs_before, tokens_before, tokens_after}` + `CompactionMode {None, Mild, Aggressive, Emergency}` (snake_case) — contrato esperado por Task 5 (MEM-22) |

## Invariantes de dominio (handoff - MUST)

1. Sin deps nuevas; sin unwrap/expect en producción; errores tipados `#[non_exhaustive]` (thiserror).
2. `estimate_tokens` determinista y total (unicode por chars, nunca bytes).
3. Un par tool_call/tool_result JAMÁS queda partido: o ambos viven o ambos mueren.
4. LLM-free 100% (Principio 4).
5. NO tocar el core `vantadb`; wiring aditivo en `lib.rs` únicamente.

## Steps

### Step 1 — Discovery + task file
- [x] Leer TDAM refs + APIs del crate (codegraph + reads)
- [x] Crear task file (este) con Impacto mapeado Regla 0
- **Gate:** ✅ registro antes de tocar código

### Step 2 — context_engine/{types,token_estimator,mod}.rs + wiring
- [x] types.rs: ChatRole/ChatMessage/CompactionMode/CompactionReport/ContextError + tests serde roundtrip
- [x] token_estimator.rs: TokenEstimator + emergency_truncate + truncate_content + tests D19 (vacío/ascii/unicode/determinismo/pares/min_keep/truncate_content)
- [x] mod.rs re-exports + `pub mod context_engine;` en lib.rs
- **Gate:** ✅ `cargo check -p vanta-memory` exit 0

### Step 3 — Verify completo + cierre
- [x] cargo check + nextest -p vanta-memory + fmt --check + clippy -p vanta-memory --all-targets --no-deps -- -D warnings — todos exit 0 (373/373 tests)
- [x] CIERRE: campaign_update_task_state taskId=1 completed + recitation §3; bloque RESULTADO §7
- **Gate:** ✅ verify todo exit 0

## Deuda técnica (Regla 6)

Sin deuda nueva neta. Techo documentado (D21): chars/3 subestima CJK y código (~drift hasta medirse); upgrade path = factor configurable ya expuesto + calibración linear TDAM (0.5–3.0) si benchmarks post-MEM-22 lo justifican. `truncate_content` corta por chars (no por tokens) — suficiente para emergency, refinable en MEM-22.

## Recitation (canónico)

=== RECITATION ===
Objetivo activo: Task 1 — MEM-23 (token estimator + emergency truncate + report types)
Estado: completed
Última acción: implementado context_engine/{types,token_estimator,mod}.rs + wiring en lib.rs; 9 tests D19 inline
Resultado: OK
State: CLOSE (desde: ACT→VERIFY)
Próxima acción: ninguna (tarea cerrada); MEM-22 (Task 5) consume types::{ChatMessage, CompactionReport, CompactionMode} y token_estimator::{TokenEstimator, emergency_truncate}
Contrato:
  verificacion: `cargo check -p vanta-memory` ✅ · `cargo nextest run -p vanta-memory` ✅ 373/373 · `cargo fmt --check` ✅ · `cargo clippy -p vanta-memory --all-targets --no-deps -- -D warnings` ✅
  evidencia:
    - claim: estimate_tokens determinista unicode-safe → evidencia: vanta-memory/src/context_engine/token_estimator.rs::estimate_tokens + test estimate_tokens_empty_ascii_unicode_deterministic — confianza: alta
    - claim: par tool_call/tool_result nunca partido → evidencia: build_units agrupa por construcción + test tool_call_pair_never_split_by_truncation — confianza: alta
    - claim: CompactionReport serde roundtrip snake_case → evidencia: test compaction_report_serde_roundtrip en types.rs — confianza: alta
  artefactos: vanta-memory/src/context_engine/{mod,types,token_estimator}.rs; vanta-memory/src/lib.rs (+2 líneas wiring)
  invariantes: NO tocar core `vantadb`; sin deps nuevas; sin unwrap/expect en producción (solo tests); errores #[non_exhaustive] thiserror; LLM-free
  deuda: D21 techo documentado — chars/3 subestima CJK/código; factor configurable ya expuesto, calibración diferida post-MEM-22. truncate_content corta por chars no tokens.
  queda_pendiente: commit lo ejecuta el lead (worker no commitea); skill progreso al cierre de campaña
Próxima tarea si completa: Task 5 — MEM-22 (context compaction)
last-synced: 2026-08-21T00:00
=== END RECITATION ===
