# FIND-90: Fallout F3X — `vantadb-mcp` handlers sobre `dyn IndexPort` (E0609)

## Metadata
- **Plan file:** (sin plan — creado desde validación proyecto 2026-09-15)
- **Fuente:** docs/dev/Backlog.md fila FIND-90 · validación proyecto 2026-09-15 · causa `13f0f729` (F3X trait-split)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🔴 Alta
- **Tipo:** Rust (vantadb-mcp handlers + trait `IndexPort`)
- **Turns estimados:** 10-15
- **Creado:** 2026-09-15
- **last-synced:** 2026-09-15
- **Estado:** ✅ COMPLETED (2026-09-15)
- **Incógnitas (uphill):** 0 (resuelta: `HnswConfig{..Default}` + 3 getters vivos — sin tocar trait, sin símbolos nuevos)
- **Pendientes (downhill):** 0 (3/3 steps ✅)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp` handlers (`resources.rs` `build_schema_resource`, `tools.rs` search + `index_vector_dim`), `vantadb-server` (depende de `vantadb-mcp`), `mcp_tests` |
| Callees | `src/index_port.rs` (trait `IndexPort`: `distance_metric()`, `index_kind()`, `node_count()`, `all_node_ids()`, `stored_vector()`, `contains_node()`), `StorageEngine::vec_index()` |
| Implicaciones | Sin cambio de comportamiento: solo cambiar la vía de acceso (campo concreto → getter del trait). Riesgo = `schema://` cambia de forma si se serializa distinto → mantener forma JSON actual. |

## Impacto mapeado (Regla 0)

- **Archivos leídos (completos):** `vantadb-mcp/src/handlers/resources.rs:120-151`, `vantadb-mcp/src/handlers/tools.rs:1810-1830`, `vantadb-mcp/src/handlers/tools.rs:2841-2853`, `src/index_port.rs` (trait completo)
- **Archivos referenciados hacia dentro:** `vantadb::StorageEngine`, `vantadb::DistanceMetric`, `vantadb::VECTOR_INDEX_VERSION`, `vantadb::TextIndexSpec`
- **Archivos que referencian a los editados:** `vantadb-server` (depende del crate), `vantadb-mcp/tests/mcp_tests.rs`, `docs/api/MCP.md` (documenta `distance` como "lower is more similar" — no tocar semántica)
- **Veredicto impacto:** MEDIO — 3 puntos de acceso en 2 archivos, mismo crate; sin migración de datos; `mcp_tests` debe seguir verde.

## Contrato

```
cargo check -p vantadb-mcp --tests pasa (hoy falla con 3× E0609)
cargo clippy -p vantadb-mcp --all-targets -- -D warnings pasa
cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests pasa
cargo check -p vantadb-server pasa (desbloqueado por el fix)
```

## Invariantes de dominio (handoff — MUST)

- **Invariantes a preservar:**
  1. Semántica de `distance` en MCP intacta (Cosine → `1 - similarity`, Euclidean → negado, SparseDot directo — `tools.rs:1825-1829` no se toca la lógica, solo la fuente de `metric`).
  2. Forma JSON de `schema://` intacta (si no hay serialización equivalente vía trait, construir el objeto desde los getters con las mismas keys).
  3. `index_vector_dim` mantiene semántica: `None` en índice vacío, dim del primer nodo con vector en caso contrario.
- **Comandos de verificación:** los 4 del Contrato.
- **Deuda pendiente:** ninguna.

## Recitation

```
=== RECITATION ===
Objetivo activo: FIND-90 — vantadb-mcp handlers sobre dyn IndexPort
Estado: PENDING
Última acción: task file creado, sin implementar
Resultado: ⬜
State: PENDING (desde: —)
Próxima acción: Step 1 — resolver serialización de config en resources.rs:134
Contrato: ver sección Contrato
Invariantes: semántica distance + forma schema:// + semántica index_vector_dim
Comandos de verificación: cargo check/clippy/nextest -p vantadb-mcp + check -p vantadb-server
Deuda: ninguna
Próxima tarea si completa: FIND-91 (misma causa raíz, distinto crate/archivo)
last-synced: 2026-09-15
=== END RECITATION ===
```

## Deuda técnica (Regla 6 — MUST)

**Saldo neto de deuda por PR:** Sin deuda — el fix elimina deuda (código roto por F3X).

## Definition of Done

| Nivel | Gate |
|-------|------|
| **Task** | Contrato verificable se cumple + `mcp_tests` verdes |
| **Commit** | Commit atómico, conventional commit (`fix:`), `git diff` limpio, verificación mecánica |
| **Release** | N/A (fix de compilación, sin cambio de API) |

## Herramientas necesarias

- cargo (check, clippy, nextest)
- codegraph_explore (verificar que no queden otros `.config`/`.nodes` sobre `dyn IndexPort` en el crate: `rg "\.config\b|\.nodes\b" vantadb-mcp/src/`)

**Skills cargadas (SDP):** campaign-executor, source-driven-development, incremental-implementation, test-driven-development, context-engineering, doubt-driven-development, api-and-interface-design (SDP v2 BUILD) + systematic-debugging, code-review-and-quality, security-and-hardening (sugeridas task). frontend-ui-engineering descartada (sin UI).

## Investigation Notes

- Causa raíz: F3X (`13f0f729`) cambió `StorageEngine::vec_index()` a `Guard<Arc<Box<dyn IndexPort>>>`; el trait expone `distance_metric()`, `index_kind()`, `node_count()`, `all_node_ids()`, `stored_vector()`, `contains_node()` — pero NO `config` ni `nodes`. Los 3 accesos son los únicos del crate (verificado en validación 2026-09-15).
- `tools.rs:1821` es trivial: `storage.vec_index().config.distance_metric` → `storage.vec_index().distance_metric()` (el trait lo tiene, `index_port.rs:200`).
- `tools.rs:2850` (`.nodes.iter().find_map(vector_slice)`) no tiene equivalente directo en el trait: camino probable = `all_node_ids()` + `stored_vector(id)` buscando el primero con vector no vacío. Verificar coste aceptable (es path de validación de dim, no hot loop).
- `resources.rs:134` (`index.config.clone()` → `to_value`) es la incógnita: el trait no expone la config serializable. Opciones a decidir en DISCOVERY: (a) construir el JSON desde getters (`index_kind`, `distance_metric`, `flat_threshold`, `node_count`…), (b) exponer un método serializable en el trait (toca `src/index_port.rs` + `port_impl.rs` — preferir evitar si (a) cubre el contrato de `schema://`). Revisar qué campos consume el lector de `schema://` antes de elegir.
- Verificado que son pre-existentes al fix FIND-89 (último toque ajeno a FIND-89) y que el verify de Fase 3 solo pedía `cargo check -p vantadb`, por eso pasaron.

## Incógnitas (uphill) vs Pendientes (downhill)

| Eje | Contador |
|-----|----------|
| Incógnitas abiertas (uphill) | 1 — serialización de config en `resources.rs:134` (opciones en Investigation Notes) |
| Pendientes de ejecución (downhill) | 3 steps |
| % completado | 0% |

## Fases explícitas — SECURITY | PERFORMANCE (P2-07)

- [x] **SECURITY** — no toca trust boundaries (solo lectura de getters para serialización/validación). Sin nuevos riesgos.
- [x] **PERFORMANCE** — `index_vector_dim` pasa de iteración directa a `all_node_ids` + `stored_vector` (clona vectores). Justificación: path frío de validación por request, no hot loop; si el review mide regresión, optimizar entonces (Regla 9).

## Steps

### Step 1: `resources.rs:134` — config HNSW sin `.config`
- **Archivos:** `vantadb-mcp/src/handlers/resources.rs:130-151`
- **Acción:** resolver la incógnita (ver Investigation Notes) y reemplazar `index.config.clone()` por la vía elegida manteniendo la forma JSON de `schema://`.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ DONE — vía (a): `HnswConfig{ distance_metric, flat_threshold, index_type: getters vivos, ..Default::default() }` + `to_value` (forma serde idéntica; engine solo crea defaults vía `port_impl`). Sin método nuevo en trait → sin Spec/Gate D.

### Step 2: `tools.rs:1821,2850` — metric + dim vía trait
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs:1821`, `vantadb-mcp/src/handlers/tools.rs:2847-2853`
- **Acción:** `metric` ← `storage.vec_index().distance_metric()`; `index_vector_dim` ← iteración vía trait (`all_node_ids` + `stored_vector` o equivalente) preservando semántica `None`-en-vacío.
- **Verify:** `cargo check -p vantadb-mcp`
- **Estado:** ✅ DONE — `metric ← distance_metric()`; `index_vector_dim ← all_node_ids + stored_vector → as_f32_slice().len()` (espeja `HnswNode::vector_slice`; path frío).

### Step 3: Verify completo + server desbloqueado
- **Archivos:** ninguno (verificación)
- **Acción:** `cargo clippy -p vantadb-mcp --all-targets -- -D warnings` + `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests` + `cargo check -p vantadb-server` + `rg "\.config\b|\.nodes\b" vantadb-mcp/src/` → 0 accesos directos restantes.
- **Verify:** los 4 comandos del Contrato en verde.
- **Estado:** ✅ DONE — check ✅ · clippy ✅ · mcp_tests 91/91 ✅ (vía `cargo test`; ver HALLAZGO nextest) · server ✅ · fmt ✅ · rg solo `StorageEngine.config`/graphrag/comentarios.

## Dependencias
- Ninguna (los getters del trait ya existen; no depende de FIND-91).

## Review (GATE — agente distinto, P2-01)

> Lo ejecuta un agente DISTINTO al implementador. Sin esto registrado, la tarea no está COMPLETED.

- **Revisor:** (pendiente — vanta-audit o vanta-review)
- **Enfoque:** ¿la vía elegida para `schema://` preserva la forma JSON? ¿la iteración de dim es aceptable en coste?
- **Cómo se probó:** check/clippy/nextest-vía-cargo-test/server/fmt en verde 2026-09-15 (ver Step 3); `test_mcp_resources_read_schema` + `test_mcp_search_semantic_distance_semantics` pasan.
- **Checklist anti-hábitos tóxicos:** (pendiente — 10 ítems)
- **Veredicto:** ⬜ pending review (implementador: worker; gate P2-01 abierto)

## Notas
- Lección de proceso (de la validación): en cambios de traits públicos el contrato debe exigir `cargo check --workspace` (o al menos `-p` de dependientes directos), no solo el crate tocado.
- **HALLAZGO (contrato vs repo):** `cargo nextest run --profile audit -p vantadb-mcp --test mcp_tests` devuelve "no tests to run" porque `.config/nextest.toml:62` excluye `package(vantadb-mcp)+binary(mcp_tests)` del default-filter (heredado por audit); el runner canónico de ese binario es `cargo test -p vantadb-mcp --test mcp_tests` (heavy-certification-50.yml:272-276). Verificado 91/91 por esa vía. Recomendación: enmendar el contrato del plan a `cargo test`.
- **HALLAZGO (path):** `vantadb::index::graph` es `pub(crate)`; `HnswConfig` se nombra como `vantadb::index::HnswConfig` (re-export `pub use graph::*`).
- WIP ajeno respetado: `tests/durability_recovery.rs` (FIND-91) modificado en worktree — NO tocado; `completions/`, `desktop lock`, `.opencode` — NO tocados.
