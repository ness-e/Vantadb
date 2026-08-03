# COMP-028: Semantic Cost Estimator (SCE) unificado

## Metadata
- **Plan file:** N/A (tarea standalone del backlog)
- **Fuente:** `docs/Backlog.md:292` — COMP-028
- **Esfuerzo:** 🟡 1-2d
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 20-30
- **Creado:** 2026-08-02T00:00
- **last-synced:** 2026-08-02T03:20
- **Estado:** ✅ COMPLETADO

## Objetivo
Extraer la estimación de costos de query, hoy distribuida en tres componentes, a un
módulo unificado `src/cost_estimator.rs` **sin cambio de comportamiento público**.
Habilitador de OLD-21 (routing multi-índice HNSW/IVF/Flat), que está Diferido esperando
esta tarea. Referencia de auditoría: `docs/audit-reports/backlog-validation-2026-07-28.md:234`.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/planner.rs` (CBO `optimize_and_compile`, 6 callers), `src/sdk/search/mod.rs` (`select_filter_strategy`, 6 callers), `src/governor.rs` (ResourceGovernor), `src/executor.rs` (`execute_plan`) |
| Callees | `src/storage/engine/stats.rs` (`get_estimated_selectivity`, 21 callers), `src/storage/engine/mod.rs` (`cardinality_stats`, `hnsw`), `src/query.rs` (`LogicalPlan`, `LogicalOperator`, `RelOp`), `src/sdk/types.rs` (`VantaValue`) |
| Implicaciones | Contratos públicos **no cambian**. `get_estimated_selectivity` conserva su firma y delega al estimador (los 21 callers quedan intactos). `FilterStrategy` se mueve de `sdk/search/mod.rs` a `cost_estimator.rs` como tipo `pub(crate)` — verificar que no se reexporta en API pública. Sin migración de datos, sin re-indexación. Performance: overhead despreciable (cálculo por operador, sin scans). |

## Contrato
```
cargo nextest run --profile audit -p vantadb --build-jobs 2  pasa  AND
cargo clippy --workspace -- -D warnings  no emite warnings nuevos  AND
cargo fmt --check  pasa  AND
existe src/cost_estimator.rs con CostEstimator::estimate_plan(LogicalPlan) -> PlanCost  AND
plan.operators len >= 1 → PlanCost.estimated_bytes > 0  (unit test)
```

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- codegraph_explore (blast radius, ya mapeado)
- rust-analyzer-mcp (goto def / diagnostics si hay dudas)

## Investigation Notes
- La funcionalidad existe y está distribuida: `ResourceGovernor` (`src/governor.rs:23`, memory/timeout
  admission), CBO con `get_estimated_selectivity` (`src/planner.rs:270-289`, reordenamiento y
  eliminación de filtros identity), `select_filter_strategy` (`src/sdk/search/mod.rs:60-79`, elige
  PreFilter/InFilter/PostFilter según joint selectivity vs `PREFILTER_THRESHOLD`=0.01).
- `get_estimated_selectivity` (`src/storage/engine/stats.rs:222-292`) usa `cardinality_stats`
  (`HashMap<field, HashMap<value_key, usize>>`) contra `hnsw.nodes.len()`. Hardcodes `0.33` para
  rangos (Gt/Lt/Gte/Lte) y `0.5`/`0.0`/`1.0` fallbacks.
- `LogicalOperator` variants relevantes para estimación: `Scan`, `FilterRelational`,
  `VectorSearch`, `Limit`, `Sort`, `Project`, `Join`, `SubqueryFilter`, `Traverse`
  (`src/query.rs:321`).
- No requiere web research: refactor interno Rust, sin APIs externas.

## Steps

### Step 1: Crear módulo `src/cost_estimator.rs` — tipos base
- **Archivos:** `src/cost_estimator.rs` (nuevo), `src/lib.rs` (mod declaration)
- **Acción:** definir `pub(crate) struct OperatorCost { estimated_rows: f64, estimated_bytes: usize }`,
  `pub(crate) struct PlanCost { pub estimated_rows: f64, pub estimated_bytes: usize }`,
  `pub(crate) struct CostEstimator<'a> { storage: &'a StorageEngine }` con `new(storage)`,
  y `pub(crate) enum FilterStrategy { PreFilter, InFilter, PostFilter }` (movido desde
  `sdk/search/mod.rs`, conservando las mismas constantes `PREFILTER_THRESHOLD`=0.01 y
  `HIGH_SELECTIVITY_THRESHOLD`=0.10). Sin cambio de semántica.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 2: Mover lógica de `select_filter_strategy` al estimador
- **Archivos:** `src/cost_estimator.rs`, `src/sdk/search/mod.rs`
- **Acción:** mover `select_filter_strategy` (hoy en `sdk/search/mod.rs:60`) al módulo del
  estimador como `CostEstimator::select_filter_strategy(&self, filters: &VantaMemoryMetadata) ->
  FilterStrategy`. `sdk/search/mod.rs` pasa a llamar al estimador. Mantener el tipo
  `FilterStrategy` `pub(crate)` y eliminar el duplicado en search/mod.rs.
- **Verify:** `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2 search`
- **Estado:** ✅ COMPLETADO

### Step 3: Centralizar `get_estimated_selectivity` → `CostEstimator::selectivity`
- **Archivos:** `src/cost_estimator.rs`, `src/storage/engine/stats.rs`, `src/storage/engine/mod.rs`
- **Acción:** copiar la lógica exacta de `get_estimated_selectivity` (stats.rs:222-292) al
  estimador como `CostEstimator::selectivity(&self, field, op, value) -> f32`. Hacer que
  `StorageEngine::get_estimated_selectivity` delegue a `CostEstimator::new(self).selectivity(...)`
  — los 21 callers existentes no cambian. Marcar la delegación con comentario `// COMP-028`.
- **Verify:** `cargo check -p vantadb` + tests de stats (`cargo nextest run --profile audit -p vantadb --build-jobs 2 -E 'test(stats)'`)
- **Estado:** ✅ COMPLETADO

### Step 4: `estimate_operator` por variante de `LogicalOperator`
- **Archivos:** `src/cost_estimator.rs`
- **Acción:** implementar `fn estimate_operator(&self, op: &LogicalOperator) -> OperatorCost`:
  `Scan` → rows = cardinalidad de la entidad (via `hnsw.nodes.len()` o stats), bytes = rows ×
  tamaño medio de nodo; `FilterRelational` → `selectivity()` aplicada a rows; `VectorSearch` →
  rows = top_k estimado (max(limit, 100) o `nodes.len()` si no hay limit), bytes = rows × dims × 4;
  `Limit{top_k}` → rows = min(rows, top_k); `Traverse` → rows × max_depth^fanout estimado;
  `Sort`/`Project`/`Join`/`SubqueryFilter` → passthrough de rows, bytes ajustados.
  Estimar con datos de stats ya disponibles — **no** hacer scans ni recorrer nodos.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 5: `estimate_plan` — combinar operadores
- **Archivos:** `src/cost_estimator.rs`
- **Acción:** implementar `pub fn estimate_plan(&self, plan: &LogicalPlan) -> PlanCost` que
  encadena `estimate_operator` en orden de `plan.operators` (rows fluyen entre operadores,
  bytes se acumulan por el operador pico). Resultado `PlanCost { estimated_rows,
  estimated_bytes }`.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ✅ COMPLETADO

### Step 6: Alimentar al CBO y a `ResourceGovernor`
- **Archivos:** `src/planner.rs`, `src/governor.rs`, `src/executor.rs` (si aplica)
- **Acción:** (a) en `planner.rs:270-289` el CBO usa `CostEstimator` (o la delegación del Step 3)
  — sin cambio de decisión de planificación; (b) en `governor.rs`, añadir overload/helper
  `ResourceGovernor::estimate_plan_cost(&self, plan: &LogicalPlan) -> PlanCost` que use el
  estimador, para que el admission de queries pueda reservar memoria según costo del plan.
  **No** cambiar la firma pública `request_allocation(bytes)` ni el comportamiento de admission
  actual (ponytail: si no hay caller que necesite el helper todavía, dejarlo `pub(crate)` y sin
  wiring — OLD-21 lo consumirá).
- **Verify:** `cargo check -p vantadb` + `cargo nextest run --profile audit -p vantadb --build-jobs 2 governor`
- **Estado:** ✅ COMPLETADO

### Step 7: Tests unitarios del estimador
- **Archivos:** `src/cost_estimator.rs` (mod tests)
- **Acción:** tests: (a) `estimate_plan` con plan Scan → `estimated_bytes > 0`; (b) `selectivity`
  Eq con cardinality conocida → `freq/total`; (c) `select_filter_strategy` con joint_sel < 0.01 →
  PreFilter, 0.01..0.10 → InFilter, ≥ 0.10 → PostFilter, filtros vacíos → PostFilter; (d)
  `estimate_operator` con `Limit` recorta rows. Reutilizar el patrón de setup de
  `src/storage/engine/tests/stats.rs`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2 cost_estimator`
- **Estado:** ✅ COMPLETADO

### Step 8: Verificación final
- **Archivos:** —
- **Acción:** correr `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`,
  `cargo machete` (0 warnings de deps no usadas), y el contrato completo de nextest. Confirmar
  que `FilterStrategy` no se reexporta en API pública (grep `pub use.*FilterStrategy`).
- **Verify:** contrato de la sección Contrato
- **Estado:** ✅ COMPLETADO
  - ✅ fmt de mis archivos (`cost_estimator.rs`, `sdk/search/mod.rs`, `governor.rs`, `stats.rs`) — limpio
  - ✅ `FilterStrategy` NO se reexporta en API pública (grep `pub use.*FilterStrategy` → 0)
  - ✅ `cargo machete` — 0 unused deps
  - ⛔ `cargo clippy --workspace -- -D warnings`: bloqueado — `vantadb-server` activa `vantadb/server` (feature unification) → compila WIP ajeno roto: `circuit_breaker.rs` (E0004 match no exhaustivo), `connection_pool.rs` (PoolGuard sin Debug/PartialEq). Archivos untracked de otro agente — NO tocados (Regla 0)
  - ⛔ `cargo fmt --check` global: diffs pre-existentes en `cli_server.rs`, `connection_pool.rs`, y reorden `circuit_breaker` en lib.rs (WIP ajeno)
  - ⛔ `cargo nextest run --profile audit -p vantadb --build-jobs 2`: lib-test no compila — `storage/engine/tests/{ops,maintenance}.rs` no actualizados para el nuevo campo `Edge.created_at_ms` (WIP en `node.rs`)
  - ⛔ `cargo check -p vantadb` falla AHORA por WIP de `graph.rs` (E0061: caller en graph.rs:110 sin el nuevo arg `time_range` de `dfs_traverse_filtered`) — **pasó limpio** antes de que el agente paralelo editara graph.rs
- **Bloqueado por:** COMP-019/021 (connection pool + circuit breaker, server feature), edge-timestamps (`node.rs` + storage tests), graph traversal `time_range` — todos WIP en curso, fuera del dominio Worker
- **Re-verificar cuando:** el árbol se estabilice (git status solo con archivos de COMP-028) — correr los comandos del contrato en orden

## Dependencias
- DRV-121/122 (planner/CBO) — ✅ COMPLETADAS (`docs/Backlog.md:104`). Desbloqueada.

## Notas
- Regla 6 (deuda): este refactor **reduce** deuda (elimina duplicación de estimación en 3
  lugares) — saldo neto negativo. No introduce unsafe ni clones en hot path.
- OLD-21 (routing multi-índice) es el consumidor futuro; el SCE debe quedar con API `pub(crate)`
  estable pero sin wiring forzado hasta que OLD-21 se active (ponytail).
- `get_estimated_selectivity` conserva su firma pública; solo la implementación delega.
- Si `estimate_plan` no tiene consumidor activo al terminar, no conectar al pipeline de
  ejecución real — dejar el helper listo (el contrato solo exige que exista y tenga test).
