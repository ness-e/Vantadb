# MEM-24 — MMD persistente (Task 6 P29)

## Objetivo
MMD (current-task memory) persistente sobre el store VantaDB + inyección en el
context engine. Formato decidido por el Lead (D23): contrato META reutilizado
(`SceneMeta`) — NO Mermaid.

## Impacto mapeado (Regla 0)
- **Leídos completos:** `context_engine/{mod,types,engine,token_estimator}.rs`,
  `core/abstractions/types.rs` (SceneMeta), `core/conversation/l0_recorder.rs`
  (sanitize_component), `offload/state_manager.rs` + `offload/storage.rs`
  (patrón put/get/list paginado), `utils/sanitize.rs`.
- **Referencias hacia dentro (nuevos archivos):** ninguno — módulos nuevos.
- **Referencias entrantes:** `context_engine/mod.rs` re-exporta; callers
  futuros (integración LLM) fuera de scope de esta tarea.
- **Veredicto:** aditivo puro. Solo variantes nuevas en `ContextError`
  (#[non_exhaustive], compatible). Sin deps nuevas.

## Steps
1. ✅ `mmd.rs`: TaskMemory{meta: SceneMeta, content}, save/load_active,
   push/list_history (ns `mmd/<session>/{active,history}`), dedup fingerprint
   `{len}:{primeros 64 chars}`, presupuesto 4000 chars char-boundary.
2. ✅ `mmd_injector.rs`: inject_mmd tras prefijo System, nunca entre
   tool_call/result (build_units), descuenta budget, dedup re-inyección.
3. ✅ Wiring mod.rs + tests D19: (a) post-aggressive, (b) fingerprint dedup,
   (c) pares intactos, (d) reopen (gated `#[cfg(feature = "fjall")]`).
4. ✅ Verify: check ✅ · nextest 427/427 default + 428/428 con fjall ✅ ·
   fmt --check ✅ · clippy -D warnings (default y --features fjall) ✅.
5. ✅ Cierre: recitation completed.

## Notas
- NO commit (instrucción explícita del orquestador).
- LLM para generar content: OUT OF SCOPE (caller pasa content).
- Test D19(d) requiere `--features fjall` (backend persistente); default suite
  queda verde sin él.
