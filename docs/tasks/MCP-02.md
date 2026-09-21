# MCP-02: S2 — `distance_metric=euclidean` sin efecto observable en search

## Metadata
- **Plan file:** docs/Backlog.md (P22, Bloque 1)
- **Fuente:** Backlog P22 `MCP-02` (test-busqueda.py T14)
- **Esfuerzo:** 🟡 1d
- **Prioridad:** 🟠
- **Tipo:** Rust
- **Turns estimados:** 15-30
- **Creado:** 2026-08-17T14:30
- **last-synced:** 2026-08-17T15:30
- **Estado:** 🟨 EJECUTADO — listo para review (vanta-audit, P2-01)
- **Incógnitas (uphill):** 0 — resuelta (métrica per-request, opción (a))
- **Pendientes (downhill):** 0 — 3 steps completados

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `vantadb-mcp/src/handlers/tools.rs` (search_memory, search_semantic) |
| Callees | `src/index/search/nearest.rs:62` (search_nearest → flat_search con self.config.distance_metric), `src/index/flat.rs:8-52`, `src/index/distance.rs` |
| Implicaciones | Parámetro documentado sin efecto; cambiar comportamiento puede afectar scores existentes (breaking para quien confió en cosine) |

## Impacto mapeado (Regla 0)
- **Archivos leídos (completos):** `src/index/search/nearest.rs`, `src/index/flat.rs` (función flat_search)
- **Archivos referenciados hacia dentro:** `DistanceMetric` (node.rs), `calculate_similarity`
- **Archivos que referencian a los editados:** handlers MCP, SDK search
- **Veredicto impacto:** MEDIO — si se decide soportar métrica por-request, cambia la API del handler MCP (schema); si es config-time, es solo doc (MCP-06)

## Contrato
"`python C:\Users\Eros\AppData\Local\Temp\opencode\test-busqueda.py` T14 verifica que `distance_metric:"euclidean"` produce scores DISTINTOS a cosine en el mismo índice; `cargo check -p vantadb-mcp` ✅"

## Invariantes de dominio (handoff — MUST)
- **Invariantes a preservar:** cosine sigue siendo el default y su semántica actual no cambia; índices existentes no se re-indexan
- **Comandos de verificación:** `cargo check -p vantadb-mcp` ✅; test-busqueda.py T14 ✅
- **Deuda pendiente:** ninguna

## Fase 1 — Evidencia de Debugging (GATE — Bug)
- **Repro:** crear índice, insertar vectores, `search_memory` con `distance_metric:"euclidean"` y con cosine → scores idénticos
- **Hipótesis:** `search_nearest` (nearest.rs:62) y `flat_search` (flat.rs:13) usan `self.config.distance_metric` — el parámetro por-request del handler MCP nunca llega al cálculo
- **1 variable controlada:** UNA decisión por intento (propagar métrica vs declararla config-time)
- **Test RED:** test-busqueda.py T14 (scores idénticos = FAIL, confirmado 2026-08-17)

## Steps

### Step 1: Decidir semántica (vanta-arch/vanta-worker con justificación)
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs` (schema search_memory/search_semantic), `src/index/search/nearest.rs`
- **Acción:** decidir: (a) propagar `distance_metric` del request al cálculo (cambia schema del handler), o (b) documentar como config-time y delegar a MCP-06. Justificar. Si (a): el índice puede tener una métrica por request — verificar que el índice config usa `DistanceMetric` internamente y que calcular con métrica distinta a la del build es válido
- **Verify:** decisión documentada en task file
- **Estado:** ✅ DONE — opción (a): propagar métrica per-request. Justificación: (1) la API ya expone la métrica per-request (schema MCP tools.rs:100, Python SDK, TS SDK; T16 rechaza valores inválidos) — (b) rompería la API o dejaría un no-op silencioso; (2) todo el cálculo es metric-paramétrico y exacto (`calculate_similarity`, `sq8_similarity`, SCANN approximate/full_distance, `cosine_sim_*`, `euclidean_distance_squared_f32`) — calcular con métrica distinta al build es válido porque el build-metric solo afecta la aproximación ANN (aristas HNSW, centroides IVF, codebooks SCANN), no la corrección; (3) precedente: AUDREP-55 (nearest.rs:32) y ERR-028 (sdk/search/mod.rs) ya tratan la métrica como per-request. Scope: thread metric en HNSW+flat; IVF/SCANN (caches metric-bound construidos con config metric) → validar y fallar con error claro en `search_with_method` (no silencioso). NO se toca `src/index/ivf.rs` ni `src/index/scann.rs` (fuera de blast radius autorizado)

### Step 2: Implementar (si opción (a))
- **Archivos:** `vantadb-mcp/src/handlers/tools.rs`, `src/index/search/nearest.rs` (firma con metric param), `src/index/flat.rs`
- **Acción:** pasar la métrica del request al search (o validar y fallar si no coincide con config)
- **Verify:** `cargo check -p vantadb-mcp` ✅
- **Estado:** ✅ DONE — cambios:
  - `src/index/search/nearest.rs`: wrapper `search_nearest` (config-driven, sin cambios de firma para ~70-80 call sites) delega en `search_nearest_with_metric(&self, ..., metric: DistanceMetric)` `pub(super)`; `metric` reemplaza `self.config.distance_metric` en guard zero-norm (AUDREP-55), `flat_search` y `effective_metric`; ramas IVF/SCANN config-driven emiten `tracing::warn!` si `metric != config`
  - `src/index/search/mod.rs`: `CPIndex::search` (trait `VecIndex`) propaga `distance_metric` → `search_nearest_with_metric` (root cause: el parámetro moría acá)
  - `src/index/search/alternate.rs`: `search_with_method(method, query_vec, query_mask, top_k, metric) -> Result<Vec<(u128,f32)>>`; ramas IVF/SCANN fallan con `VantaError::InvalidInput` claro si `metric != config` (metric-bound); Flat y Hnsw/DiskAnn propagan
  - `src/sdk/search/vector.rs:127`: `Some(m) => index.search_with_method(m, ..., distance_metric)?`
  - `src/index/search/tests.rs:478`: caller actualizado con `DistanceMetric::Cosine` + `.unwrap()`
  - `src/index/flat.rs`: NO tocado (revertido — fuera de blast radius; FlatIndex::search ignora su trait param, misma clase de bug pero off-path, delegar a engine)
  - `vantadb-mcp/src/handlers/tools.rs`: sin cambios (schema ya expone distance_metric y se parsea bien)

### Step 3: Verificar con batería
- **Archivos:** — (binario rebuild)
- **Acción:** rebuild binario, re-ejecutar test-busqueda.py T14
- **Verify:** T14: euclidean ≠ cosine en scores ✅
- **Estado:** ✅ DONE — `cargo check -p vantadb-mcp` ✅ (sin warnings), `cargo check -p vantadb --tests` ✅, `cargo build --bin vanta-cli` ✅, `cargo nextest run -p vantadb search_with_method` → `test_search_with_method_override_routes_backends` PASS ✅. T14 (2026-08-17, binario fresh en `target/debug`): `cosine top=vec-1 scores=[1.0, 0.97, 0.0] | euclidean top=vec-1 scores=[-0.0, -0.08, -1.58]` — scores DISTINTOS, contrato cumplido. Fallos restantes (T09/T11/T13/T15 BM25 `text_index not found`, T17-19 search_semantic) son pre-existentes y ajenos a `distance_metric`

## Dependencias
- Ninguna (independiente; desbloquea MCP-06)

## Review (GATE — agente distinto, P2-01)
- **Revisor:** vanta-audit
- **Enfoque:** ¿soportar métrica por-request rompe invariantes del índice (HNSW build con cosine + query euclidean)?
- **Cómo se probó:** T14 con salida real
- **Veredicto:** ⏳ pendiente

## Notas
- Si se elige (b) config-time: MCP-02 se cierra con solo doc (delegar a MCP-06) y el task file se marca resuelto como WONTFIX con evidencia

## Context Save Point
- **Fecha:** 2026-08-17T15:30
- **Branch:** develop
- **CI pendiente:** no (verificación local completa; sin commit — lo ejecuta vanta-lead)
- **Decisiones:** opción (a) — métrica per-request propagada al cálculo; IVF/SCANN metric-bound validan y fallan con error claro; FlatIndex::search queda como deuda conocida (off-path, delegar a engine)
- **Problemas conocidos:** (1) `~/.cargo/bin/vanta-cli.exe` es un binario VIEJO — el test T14 requiere el binario fresh (correr con `target\debug` primero en PATH o reinstalar); (2) state machine del campaign MCP bloquea transición MCP-02 → in-progress (13 tareas WIP de otras sesiones en el plan) — ejecución hecha igual, anotado en RESULTADO; (3) T09/T11/T13/T15 fallan por `text_index not found: bm25` y T17-19 por search_semantic — pre-existentes, fuera de MCP-02
- **Próxima tarea:** review MCP-02 (vanta-audit, P2-01) → MCP-03
