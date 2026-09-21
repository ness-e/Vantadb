# OLD-21: CP-Index formal (query routing inteligente)

## Metadata
- **Plan file:** docs/Backlog.md (línea 251) — sin plan file activo; tarea standalone
- **Fuente:** Backlog línea 251 — ⏳ Deferido hasta COMP-028 ✅ (COMPLETADA)
- **Esfuerzo:** 🟡 1d-1sem (15-30 turns)
- **Prioridad:** 🟡
- **Tipo:** Rust core
- **Turns estimados:** 15-30
- **Creado:** 2026-08-03
- **last-synced:** 2026-08-03
- **Estado:** ✅ COMPLETED (2026-08-03)

## Contexto (estado actual — verificado por codegraph 2026-08-03)

- **COMP-028 ✅ COMPLETADA** (commit `f7cb46e4`): `src/cost_estimator.rs` con
  `CostEstimator::selectivity` / `estimate_operator` / `estimate_plan` /
  `select_filter_strategy`. `PlanCost` tiene `#[allow(dead_code)]` (sin reader).
- **`ResourceGovernor::estimate_plan_cost`** (`src/governor.rs:94`) — `#[allow(dead_code)]`,
  comentario explícito: *"COMP-028: consumed by OLD-21; not wired until it lands."*
- **`CPIndex::search_nearest`** (`src/index/search.rs:482`) — routing interno estático:
  1. `if config.index_type == IndexType::Ivf` → IVF lazy-build + search
  2. `if use_flat_search()` (`nodes.len() <= flat_threshold`) → flat scan
  3. else → HNSW layers
- **`create_index()`** (`src/index/mod.rs:96`) — `VecIndex` trait con 5 backends:
  HNSW/IVF/Flat/DiskAnn/Scann. `IndexType` enum existe.
- **`StorageEngine::vec_index()`** (`src/storage/engine/mod.rs:439`) — devuelve `Arc<CPIndex>`
  (tipo concreto, no `dyn VecIndex`).
- **`vector_memory_search`** (`src/sdk/search/mod.rs:568`) — usa `engine.vec_index()`
  + `select_filter_strategy` (PreFilter/InFilter/PostFilter — ya movido desde COMP-028).
- **Routing texto/vector/híbrido YA EXISTE** en `planner.rs` (`classify()` + CBO) —
  NO se toca en esta tarea.

**Lo que falta (scope OLD-21):**
1. Selección de índice (HNSW vs IVF vs Flat) **basada en costo estimado**, no solo
   config estática + `flat_threshold` fijo.
2. Conectar `ResourceGovernor::estimate_plan_cost` → admission control (request/free
   allocation) en el path de search — remover `#[allow(dead_code)]` del consumidor.
3. Métricas/EXPLAIN de la decisión de routing.

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/sdk/search/mod.rs` (`vector_memory_search`, `search`), `src/graphrag/seed.rs`, `vantadb-mcp/src/lib.rs` |
| Callees | `src/cost_estimator.rs`, `src/governor.rs`, `src/index/mod.rs`, `src/index/search.rs`, `src/index/ivf.rs`, `src/index/flat.rs` |
| Implicaciones | `vec_index()` devuelve `Arc<CPIndex>` concreto → si se generaliza a `Arc<dyn VecIndex>` hay que revisar `search_hnsw` y storage. `PlanCost` pierde dead_code. Contrato público de `search()` NO cambia (internal routing). |
| Riesgo | medio — toca hot path de búsqueda; cualquier regresión afecta recall/latencia |

## Contrato
"cargo nextest run --profile audit --workspace --build-jobs 2 pasa; `select_index_strategy` elige Flat para datasets < flat_threshold, HNSW para default, IVF para datasets grandes; `estimate_plan_cost` conectado en admission (request/free allocation); 0 warnings clippy"

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt, test)
- rust-analyzer-mcp (diagnostics)
- codegraph_explore (blast radius — ya hecho)

## Investigation Notes
- Routing texto/vector/híbrido ya existe en `planner.rs` (classify + CBO) — NO duplicar.
- COMP-028 dejó `ResourceGovernor::estimate_plan_cost` con `#[allow(dead_code)]`
  explícitamente "consumed by OLD-21". El comentario en `src/cost_estimator.rs:43-45`
  y `src/governor.rs:92-93` confirma que este es el consumidor esperado.
- `search_nearest` ya tiene branching interno IVF/flat/HNSW — la tarea formaliza la
  selección dinámica y la conecta al estimador de costos.

## Steps

### Step 1: Research — confirmar estado actual
- **Archivos:** `src/cost_estimator.rs`, `src/governor.rs`, `src/index/search.rs`, `src/sdk/search/mod.rs`, `src/storage/engine/mod.rs`
- **Acción:** Leer los 5 archivos (source ya verificado por codegraph). Confirmar: cómo
  se fija `config.index_type` hoy, dónde se decide flat_threshold, qué expone `CostEstimator`,
  cómo `vector_memory_search` obtiene el índice. Documentar hallazgos en Notas.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 2: Diseñar `select_index_strategy` en CostEstimator
- **Archivos:** `src/cost_estimator.rs`
- **Acción:** Agregar método `pub(crate) fn select_index_strategy(&self, top_k: usize) -> IndexType`
  (o equivalente) con heurística mínima (ponytail):
  - `nodes.len() <= flat_threshold` → `IndexType::Flat`
  - `nodes.len() >= IVF_THRESHOLD` (ej. 10_000, constante) → `IndexType::Ivf`
  - else → `IndexType::Hnsw`
  Mantener la decisión consistente con el `flat_threshold` que ya usa `CPIndex::use_flat_search`.
  Si existe `config.index_type` explícito distinto de Hnsw → respetarlo (no overrider).
- **Verify:** `cargo check -p vantadb` + 1 test unitario de cada rama
- **Estado:** ⬜ PENDING

### Step 3: Conectar selección de índice en `vector_memory_search`
- **Archivos:** `src/sdk/search/mod.rs`
- **Acción:** En `vector_memory_search`, obtener la estrategia de `CostEstimator` y
  rutear: Flat → `flat_search` directo; IVF → path IVF (lazy-build); HNSW → `engine.vec_index()`
  (comportamiento actual = default). No cambiar la firma pública de `search()`.
  Respetar config explícita: si el engine tiene `IndexType` configurado ≠ Hnsw, usar ese.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test search` + tests existentes pasan
- **Estado:** ⬜ PENDING

### Step 4: Conectar `ResourceGovernor::estimate_plan_cost` → admission control
- **Archivos:** `src/governor.rs`, `src/sdk/search/mod.rs`
- **Acción:** En el path de búsqueda (donde hoy se ejecuta el plan), usar
  `ResourceGovernor::estimate_plan_cost` para estimar bytes del plan y llamar
  `request_allocation(bytes)` antes / `free_allocation(bytes)` después (guard). Remover
  `#[allow(dead_code)]` de `estimate_plan_cost`. Si el governor no está configurado en
  el engine, skip silencioso (sin cambio de comportamiento).
- **Verify:** `cargo check -p vantadb` + tests governor existentes pasan
- **Estado:** ⬜ PENDING

### Step 5: Métricas de routing
- **Archivos:** `src/metrics.rs` (si existe patrón) o `src/sdk/search/mod.rs`
- **Acción:** Registrar la decisión (Flat/IVF/HNSW) en métricas existentes si hay un
  patrón `record_planner_*` (ya visto en search). Si no hay patrón de métrica de índice,
  agregar `record_vector_index_routing(IndexType)` siguiendo el estilo existente.
  NO crear infraestructura de métricas nueva.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 6: Tests de routing + integración
- **Archivos:** `src/cost_estimator.rs` (tests), `tests/` existentes
- **Acción:** Tests: (a) dataset chico → Flat; (b) dataset grande → IVF; (c) default → HNSW;
  (d) config explícita respetada; (e) admission guard libera bytes en error path.
  Correr suite completa.
- **Verify:** `cargo nextest run --profile audit --workspace --build-jobs 2`
- **Estado:** ⬜ PENDING

### Step 7: Verify completo
- **Archivos:** —
- **Acción:** `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo nextest run --profile audit --workspace --build-jobs 2`. 0 warnings.
- **Verify:** todos pasan
- **Estado:** ⬜ PENDING

## Dependencias
- COMP-028 ✅ COMPLETADA (commit `f7cb46e4`) — `CostEstimator` ya existe

## Notas
- **NO tocar** `planner.rs` classify/CBO — routing texto/vector/híbrido ya está.
- **NO cambiar** la firma pública de `search()` / `VantaMemorySearchRequest`.
- Ponytail: heurística simple de umbrales. No construir un optimizer de índices completo.
- El `flat_threshold` ya existe en `CPIndex::use_flat_search()` — reusar, no duplicar.

## Context Save Point
- **Fecha:** 2026-08-03
- **Branch:** develop
- **CI pendiente:** sí — full verify pre-push
- **Decisiones:** routing multi-índice basado en umbrales (Flat/IVF/HNSW) usando CostEstimator; admission vía ResourceGovernor::estimate_plan_cost
- **Problemas conocidos:** `vec_index()` devuelve `Arc<CPIndex>` concreto — si se necesita `dyn VecIndex` para IVF path externo, evaluar en Step 3
- **Próxima tarea:** — (standalone; siguiente backlog item tras OLD-21)
