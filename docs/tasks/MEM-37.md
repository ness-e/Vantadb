# MEM-37 — Integración offload↔recall (budget + cursor compartidos)

Plan: `docs/plans/2026-08-21-vanta-context-engine.md` Task 7 · Estado: ✅ COMPLETED

## Impacto mapeado (Regla 0)
- **Leídos completos:** `context_engine/engine.rs` (assemble/mild/aggressive), `mmd_injector.rs`, `compressor.rs`, `token_estimator.rs`, `types.rs`, `mod.rs`, `core/hooks/auto_recall.rs`, `offload/state_manager.rs`, `tests/context_engine.rs`, `tests/e2e_flow.rs`.
- **Referencias entrantes:** `inject_mmd` 5 callers, `assemble` 2 callers; struct literal de ChatMessage solo en `adapters/standalone/llm_runner.rs` (tipo local DISTINTO, no afectado).
- **Veredicto:** cambio aditivo — campo opcional `id` en `ChatMessage` (serde default) + fn coordinadora `assemble_with_recall` en engine.rs. NO reescribe assemble → stop condition no aplica.

## Steps
- ✅ S1: campo `id` en ChatMessage + `with_id` (`types.rs`)
- ✅ S2: `assemble_with_recall` (coordinator único budget compartido) + exports (`engine.rs`, `mod.rs`)
- ✅ S3: tests D19 (a)(b) en `tests/context_engine.rs`
- ✅ S4: test D19 (c) e2e en `tests/e2e_flow.rs`
- ✅ S5: verify mecánico completo — check ✅ · nextest 430/430 ✅ · fmt --check ✅ · clippy -D warnings ✅

## Decisiones / hallazgos
- Coordinator = ~60 líneas en `engine.rs`: assemble → inject_mmd → inject_recall_block ×2, UN `remaining` mutable; cada inyección whole-or-skip contra lo que resta ⇒ unión ≤ budget por construcción.
- Cursor MEM-20 → boundary = fin de la unidad atómica del tool call (call + ToolResults contiguos) → `protected_prefix.max(boundary)`. Bug detectado en propio primer draft: boundary `pos+1` dejaba los ToolResults fuera del prefijo.
- Hallazgo de motor (MEM-22, no tocado): con `protected_prefix > 0`, `aggressive_one_shot` no puede cortar (splice frontal, `eligible(0)` falso) → degrada a emergency, que sí respeta el prefijo. Documentado; posible mejora futura en MEM-22.
- Tests calibrados con unidades gruesas (~203 tokens): mild queda limitado por MIN_REPLACEMENTS_PER_PASS=10 → aggressive corre y deja headroom real para las 3 inyecciones.

## Context Save Point
Ninguno — tarea completa. Sin commit (regla de la invocación).
