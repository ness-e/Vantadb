# FND-23-F1: Instrumentar vanta_graph_ops_total (deuda ADR-024)

## Metadata
- **Plan:** docs/plans/2026-08-16-wave-followups.md (W3)
- **Fuente:** deuda documentada en ADR-024 (`vanta_graph_ops_total` pendiente de instrumentar, vanta-tuner post-launch)
- **Estado:** ✅ COMPLETO (2026-08-17) · **Sub-agente:** vanta-tuner
- **Prioridad:** 🟡

## Objetivo
Instrumentar el contador `vanta_graph_ops_total` (operaciones de grafo: `add_edge`, `traverse` BFS en `src/engine.rs:349`) en `src/metrics/` — la métrica nombrada como pendiente en ADR-024. NO cambia el default-on (decisión ADR-024 intacta); solo telemetría.

## Archivos clave
- `src/metrics/core/registry.rs` (1168L — donde vive METRICS_REGISTRY), `src/metrics/core/mod.rs` (record_* helpers), `src/engine.rs:349` (traverse BFS), `src/edge_index.rs` (add_edge), `docs/architecture/adr/ADR-024-graph-engine-default-telemetry.md` (nota: señal ahora instrumentada)

## Steps
1. DISCOVERY: leer registry.rs (patrón de contadores existentes, p.ej. vanta_planner_*), engine.rs traverse, edge_index add_edge; decidir dónde incrementar (mínimo: 1 contador total de ops de grafo; opcional: label por tipo op si ya hay patrón)
2. Implementar: registrar `vanta_graph_ops_total` (u8/atomic counter siguiendo el patrón del registry) + incrementar en los call sites de ops de grafo
3. Test: test que incremente el contador y lo verifique en `export_metrics_text`/snapshot (patrón de tests del registry)
4. Verificar: `cargo check -p vantadb --features prometheus` + test del contador pasa + grep de la métrica en registry
5. Actualizar ADR-024 (nota: instrumentado, fecha) + task file + RESULTADO

## Contrato (verify mecánico)
- `vanta_graph_ops_total` registrado en registry.rs y exportado en /metrics (grep)
- Incrementado en los call sites reales de ops de grafo (traverse/add_edge)
- Test del contador pasa
- `cargo check -p vantadb --features prometheus` pasa (y default sin prometheus: no-op como el resto)

## Invariantes (handoff)
- NO tocar: docs/Backlog.md, AUD-024.md, verify-log.jsonl, _vanta-cli.ps1, plan file, .opencode/AGENTS.md, .opencode/agents/*
- NO git add/commit; NO campaign_update_task_state
- NO cambiar default-on de grafos (ADR-024: decisión solo cambia con telemetría + umbral — esta tarea ES la telemetría)
- Sin feature gate nuevo; misma técnica cfg-guard que las métricas existentes

## Fases
- SECURITY: n/a
- PERFORMANCE: costo del contador atómico en hot path de grafo — despreciable, seguir patrón existente

## Resultado
```
RESULTADO: ✅ COMPLETO
STEPS_OK: 5/5
PROXIMO_STEP: ninguno
COMMIT_HASH: ninguno (lead commitea)
ARCHIVOS: src/metrics/core/registry.rs, src/metrics/core/mod.rs, src/engine.rs, src/sdk/api.rs, src/sdk/graph.rs, docs/architecture/adr/ADR-024-graph-engine-default-telemetry.md, .opencode/skills/campaign-executor/tasks/FND-23-F1.md
VERIFY_CONTRATO: pasa
BLOQUEO: ninguno
```

## Ejecución (2026-08-17, vanta-tuner)

- **Steps 1-5:** ✅ completados. DISCOVERY leyó registry.rs (patrón IntCounterVec de
  HTTP_REQUESTS_TOTAL), engine.rs:351 traverse, api.rs add_edge/remove_edge,
  sdk/graph.rs (6 métodos edge_query), ADR-024. Implementación: GRAPH_OPS_TOTAL
  (IntCounterVec label `op`) en registry.rs; `record_graph_op(op)` cfg-dual en
  mod.rs; increments en engine.rs:358 (traverse), api.rs:1051 (add_edge),
  api.rs:1087 (remove_edge), sdk/graph.rs ×6 (edge_query). Test
  `test_graph_ops_counter_init` (incremento por label + verificación en
  export_metrics_text) + handle en test_all_counter_handles_some. ADR-024
  actualizado (nota instrumentado 2026-08-17, metric row, reopen signal ✅ DONE).
- **Verificación:** `cargo check -p vantadb --features prometheus` ✅ ·
  `cargo check -p vantadb` (default, no-op) ✅ · `cargo test --features
  prometheus --lib metrics::` 93 passed ✅ · `cargo fmt --check` ✅ ·
  `cargo clippy --features prometheus --lib` ✅ · grep métrica en registry ✅
- **Invariantes:** NO git add/commit; NO campaign_update_task_state; NO tocar
  Backlog/AUD-024/verify-log/plan/AGENTS/agents/mcp/SKL-* — respetado.
  Default-on de grafos intacto (ADR-024). Sin feature gate nuevo.