# NUEVO-13: HNSW ef_search auto-tuning (heuristic doubling)

## Metadata
- **Plan file:** `docs/Backlog.md` (Phase 7)
- **Fuente:** `docs/Backlog.md:183`
- **Esfuerzo:** 🟡 3-5d
- **Prioridad:** 🟡
- **Tipo:** Rust
- **Turns estimados:** 15-30
- **Creado:** 2026-07-26
- **Estado:** ⬜ PENDING

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| **Callers** | `src/index/search.rs:478` (search_nearest consume `current_ef()`), `src/sdk/search/mod.rs:349,377` (vector_memory_search llama report_brute_fallback/report_success) |
| **Callees** | `std::sync::atomic::{AtomicUsize, Ordering}` |
| **Implicaciones** | ✅ Contrato público no cambia. `AutoTune` es pub-only interna. `search_nearest` ya usa `current_ef()` como upper bound. Performance: cambios en ef_search afectan recall vs latency. |

## Estado actual (pre-implementación)

Lo que ya existe y funciona:
- `AutoTune` struct con `ef_search` + `hit_streak` atómicos
- `report_brute_fallback()`: dobla `ef_search` hasta `MAX_EF` (2000), resetea hit_streak
- `report_success()`: cada 10 éxitos, divide `ef_search` a la mitad hasta `MIN_EF` (10)
- 3 tests unitarios: doubling, halving, bounds
- Cableado en `search_nearest()`: `ef_search = max(static_ef, tuned_ef, top_k)`
- Señalización desde `vector_memory_search()` en `sdk/search/mod.rs`

Lo que **no** existe y debe agregarse:
1. **Persistencia** — `ef_search` se pierde al reiniciar el proceso (siempre arranca en 50). Guardar en metadata de engine y restaurar en startup.
2. **Amortiguación** — doubling es agresivo. Usar factor 1.5 en lugar de 2.0, o dampening basado en frecuencia de fallbacks.
3. **Observabilidad** — métrica `vantadb_auto_tune_ef` gauge expuesta vía `crate::metrics`.
4. **Test de integración** — test que inserte datos, haga queries y verifique que `ef_search` se ajusta.

## Contrato
"`cargo nextest run --profile audit --workspace --build-jobs 2` pasa. Nuevos tests (unit + integration) pasan. Gauge métrica `vantadb_auto_tune_ef` reporta valor actual."

## Herramientas necesarias
- cargo-mcp (check, test)
- codegraph_explore (blast radius confirm)
- rust-analyzer-mcp (diagnostics)

## Steps

### Step 1: Persistencia — guardar/restaurar ef_search en engine
- **Archivos:** `src/index/auto_tune.rs`, `src/engine.rs`
- **Acción:** Agregar `AutoTune::set_ef(v)` y `AutoTune::snapshot()`. En `engine.rs`, guardar ef_search en metadata al shutdown, restaurar al connect().
- **Verify:** `cargo check -p vantadb`

### Step 2: Amortiguación — factor 1.5 en lugar de doubling
- **Archivos:** `src/index/auto_tune.rs`
- **Acción:** Cambiar factor de 2.0 a 1.5 en `report_brute_fallback()`. Mantener división por 2 en `report_success()`.
- **Verify:** `cargo nextest run --profile audit -p vantadb --test auto_tune` — tests existentes se actualizan con nuevos valores esperados.

### Step 3: Observabilidad — gauge métrica
- **Archivos:** `src/index/auto_tune.rs`, `src/metrics.rs`
- **Acción:** Agregar gauge `vantadb_auto_tune_ef` que refleje `current_ef()`.
- **Verify:** `cargo check -p vantadb` + inspección visual de métricas.

### Step 4: Test de integración — auto-tuning end-to-end
- **Archivos:** Tests existentes en `src/index/auto_tune.rs` + nuevo test en `tests/core/` o `tests/certification/`
- **Acción:** Test que inserte N vectores, ejecute queries forzando fallbacks, verifique que ef_search sube.
- **Verify:** `cargo nextest run --profile audit --workspace --build-jobs 2`

## Dependencias
- Ninguna

## Notas
- **No implementar PID loop** — por diseño (ver descripción del backlog). Solo heuristic doubling mejorado.
- `report_brute_fallback()` se dispara cuando HNSW da 0 hits y se cae a flat search. Esto es un evento raro — no necesita optimización de throughput.
- `ponytail:` Si persistencia es overkill para 0.2.0, skip Step 1 y solo mejorar algoritmo + test. Add when: usuarios reportan búsquedas lentas en warmup.

## Recitation
- **Objetivo activo:** Mejorar auto-tuning HNSW ef_search con persistencia, amortiguación, métricas y test de integración
- **Estado actual:** ⬜ PENDING — esperando delegación a vanta-worker
- **Próximo paso:** Delegar implementación a vanta-worker con entry point `src/index/auto_tune.rs`
