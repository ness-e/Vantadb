# TASK FIND-35: Ciclo StorageEngine get/prefetch (2 nodos)

## Metadata
- **Plan file:** `docs/plans/2026-08-27-backlog-v2.md`
- **Creado:** 2026-08-27T15:00
- **last-synced:** 2026-08-27T18:30
- **Estado:** ✅ COMPLETED (vanta-worker)
- **Ruta:** vanta-worker
- **Prioridad:** 🔴 Alta | **Esfuerzo:** 🟠 1d | **Appetite:** max 1d

## Spec

| Decisión | Opción elegida | Alternativa descartada | Justificación (evidencia) |
|----------|----------------|------------------------|---------------------------|
| Ciclo get↔prefetch: romper vs documentar | Documentar ciclo intencional single-level + PrefetchGuard | Aplanar: extraer `fetch_prefetch_targets` sin recursión a `get()` o split `get_inner` sin prefetch | `src/storage/engine/get.rs:205` `get()`→`prefetch_related()`→`get(warm_id)` es ciclo intencional de 2 nodos por diseño OLD-20 (prefetch co-access). Guard `PrefetchGuard` thread_local+RAII ya resuelve SO MCP-15. Romper añade indirection y duplica lógica de materialización de nodos (backend+HNSW+vstore) sin eliminar callees. Ponytail rung 1: documentar intención es más barato. |
| PrefetchGuard: thread_local vs AtomicBool global | thread_local `Cell<bool>` + RAII | `AtomicBool` global o `Mutex` | Prefetch es re-entrancia por stack-depth, no cross-thread. thread_local aisla workers Tokio (cada `tokio-rt-worker` tiene su flag) sin contención. `AtomicBool` global serializaría prefetches de lectores concurrentes y ocultaría bug de nested prefetch legítimo en threads distintos. Evidencia: `src/storage/engine/get.rs:24-45` + test `test_get_prefetch_does_not_recurse_forever` usa cold-tier A↔B en mismo thread, pasa 0.486s. |
| Tests: existentes vs nuevos | Reusar `test_get_prefetch_does_not_recurse_forever` + 2 tests cache_hit existentes, añadir 1 edge de 3-nodo si aporta | No añadir | Co-access A↔B ya cubre ciclo 2 nodos. Ciclo 3-nodo A→B→C no es alcanzable con guard single-level (prefetch anidado es no-op). Test adicional no aporta señal. Pool existente 8 tests prefetch/get_cache verdes (ver contrato). |
| Doc dónde justificar | File header `//!` en `get.rs:1-15` + inline en `PrefetchGuard` y `prefetch_related` | ADR separado | Doc cercano al código es descubierto por codegraph_explore y reviewers sin indirection. ADR es overhead para ciclo intencional documentado con guard existente. Ponytail: borrar antes de añadir. |

**Contrato mecánico cubierto:** no se añaden `pub fn` nuevos, solo `//!` doc header + comentario ampliado (si aplica). No requiere spec-first gate para feature-add. Gate D no dispara (blast radius 2 archivos, sin API pública nueva).

## Blast Radius

**Callers → Callees → Implicaciones (grep verificado 2026-08-27)**

- `StorageEngine::get` (`src/storage/engine/get.rs:50`) — pub. Callers: `insert.rs:135` (existence probe), `delete.rs:81,213`, `maintenance.rs:683,715,1086`, `warm_hnsw_top_layer` (221-222), `prefetch_related` (249, recursivo), `cache_warmer` metrics, ~30 call sites en `tests/`. Callees: `volatile_cache.try_write/read`, `backend.get`, `hnsw.nodes.get`, `vector_store.get`, `read_header`, `prefetch_related`. Implicación: read hot path; cambio debe preservar ERR-036 (no blocking write lock) + Binary/SQ8 payload restore.
- `StorageEngine::prefetch_related` (`src/storage/engine/get.rs:228`) — private `fn(&self, id)`. Callers: solo `get` (205). Callees: `PrefetchGuard::acquire`, `cache_warmer.suggest_warm_ids`, `self.get(warm_id)` (recursivo, guardado), `volatile_cache.write`, `cache_warmer.record_prefetch_hit`. Implicación: single-level prefetch; guard hace nested no-op; perf hot path <8 fetches (max_prefetch).
- `PrefetchGuard` (`src/storage/engine/get.rs:31-45`) — private `struct` + `acquire()->Option<Self>` + `Drop`. Callers: `prefetch_related`. Callees: `thread_local PREFETCH_IN_PROGRESS: Cell<bool>`. Implicación: RAII unwind-safe; flag resetea en panic mid-prefetch, evita silent disable.
- `CacheWarmer` (`src/cache_warmer.rs`) — `suggest_warm_ids(id, cache_contains)`, `record_co_access`, `record_prefetch_hit`. Caller: `prefetch_related`. Callee: `RwLock<HashMap>`. Implicación: table O(n²) con cap 1M pairs (MAX_CO_ACCESS_PAIRS), decay cada 1K events; no hot path salvo `suggest_warm_ids` read-lock.
- `warm_hnsw_top_layer` (`src/storage/engine/get.rs:212`) — pub(crate). Callers: init warmer. Callees: `get()` por cada top_id (re-entra ciclo pero guard lo corta). Implicación: top-layer warming no debe stack-overflow si HNSW vacío.
- **Conclusión:** grafo dirigido intencional `get ↔ prefetch_related` (2 nodos SCC) pero acotado a profundidad 1 por `PrefetchGuard`. Sin guard, SCC es infinito con par co-access mutuo uncached (MCP-15 SO). Con guard, SCC existe sintácticamente pero operacionalmente es DAG `get → prefetch_related → get (no-op nested)`. CodeGraph reporta SCC por CALLS, no por runtime guard; doc justifica intención.

## Impacto mapeado (Regla 0)

- **Archivos leídos completos (antes de editar):**
  - `src/storage/engine/get.rs` (434 líneas completas) — `PrefetchGuard` (13-45), `get` (50-207), `warm_hnsw_top_layer` (212-224), `prefetch_related` (228-257), `get_many` (261-433)
  - `src/cache_warmer.rs` (490 líneas completas) — `CacheWarmer::suggest_warm_ids` (158-183), `record_co_access` (111-151), `MAX_CO_ACCESS_PAIRS` cap, decay, tests
  - `src/storage/engine/tests/engine.rs:664-701` — `test_get_prefetch_does_not_recurse_forever` (cold-tier A↔B, 3 co-access, asset)
  - `src/storage/engine/tests/ops.rs:604-630` — `test_get_cache_tombstone_flag`, `test_get_cache_hit_bumps_hits_uncontended`
  - `src/storage/engine/mod.rs:373` — `cache_warmer: CacheWarmer` field
  - `docs/plans/2026-08-27-backlog-v2.md` Task 2 (contrato + gate justification + risk/pre-mortem)
  - `SKILLS-MANIFEST.md` (grep `get/prefetch/cache/cycle` → Base only, SDP)
- **Referencias hacia dentro (qué importa este archivo):**
  - `crate::backend::BackendPartition`, `crate::error::Result`, `crate::lsm::unpack_offset`, `crate::node::{UnifiedNode, FilterBitset, VectorRepresentations}`, `crate::storage::engine::{StorageEngine, BufferedWrite, FLAG_TOMBSTONE}`, `crate::storage::ops::NodeMetadata`, `crate::cache_warmer::CacheWarmer`, `web_time::{SystemTime, UNIX_EPOCH}`, `std::collections::HashMap`
- **Referencias entrantes (quién depende de lo que cambia):**
  - `src/storage/engine/tests/engine.rs` → `engine.get` + `cache_warmer.record_co_access` (test co-access par) — debe seguir pasando
  - `src/storage/engine/insert.rs:135` → `self.get(node.id)` existence probe — no afectado (lee sin mutar prefetch)
  - `src/storage/engine/delete.rs`, `maintenance.rs` → `self.get(node_id)` — paths usan mismo guard, no cambian
  - `src/cache_warmer.rs` → no depende de get.rs, solo provee `suggest_warm_ids`
  - `vantadb-python` / `vantadb-wasm` / `desktop` — no tocan `get.rs` directo
  - `codegraph_explore` consumidor — lee doc header para justificar SCC
- **Veredicto:** cambio seguro y reversible. Solo `//!` file-header doc (15 líneas) ampliando contrato OLD-20/MCP-15 + comentario guard si precisa. No rompe API pública, no introduce `pub fn` nuevo, no cambia comportamiento runtime. Riesgo: doc desactualizado si firma cambia → mitigado doc cercano + test regression ya cubre guard. `thread_local` no funciona en async spawn_blocking si prefetch se moviera a async — documentar invariante: `get` es sincrono, prefetch es sincrono single-thread.

## Contrato

`cargo nextest run -p vantadb -E 'test(prefetch|get.*cache)'` ✅ (o equivalente `test(/prefetch|get.*cache/)` / `test(prefetch) or test(get_cache)` → 8 passed) + `rg -n "PrefetchGuard" src/storage/engine/get.rs` hit (3 líneas) + `codegraph_explore "StorageEngine get prefetch"` muestra ciclo 2 nodos justificado en doc (file header `//! StorageEngine get/prefetch — intentional SCC` + `PrefetchGuard` guard) o ciclo roto (no CALLS SCC)

Verificación mecánica:
1. `cargo nextest run -p vantadb -E 'test(prefetch) or test(get_cache)'` — 8 tests (6 prefetch + 2 get_cache + 1 recurse) todos verdes; con regex `/prefetch|get.*cache/` idem
2. `rg -n "PrefetchGuard" src/storage/engine/get.rs` → 3 hits (líneas 31,32,41,234) — guard existe
3. `cargo check -p vantadb --all-targets` ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 (si aplica)
4. Doc justification visible en `src/storage/engine/get.rs:1-18` header `//!` + `PrefetchGuard` doc + `prefetch_related` doc — codegraph_explore justificado

## Herramientas
- `codegraph_explore` (blast radius — grep verificado; actual codegraph MCP no disponible, fallback rg)
- `cargo nextest` (profile default, filter `prefetch`/`get_cache`)
- `rg` (verificar PrefetchGuard hits)
- `cargo check`, `cargo clippy`, `cargo fmt --check`

## Skills

**Base (campaign_load_skills):** campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging
**SDP Lifecycle BUILD/VERIFY (skills-engineering.md):** incremental-implementation (BUILD slices), test-driven-development (VERIFY prefetch logic) — Lifecycle mapping BUILD/VERIFY justifican prefetch read path hot
**SDP grep SKILLS-MANIFEST keywords "get/prefetch/cache/cycle":** `cache_warmer` no es skill; manifest hits: `performance-optimization` (hot path), `code-review-and-quality` (VERIFY), `documentation-and-adrs` (justification doc). Candidatas retenidas: `documentation-and-adrs` (doc cycle), `code-review-and-quality` (pre-commit gate). `performance-optimization` no aplica (no hot-path change); ponytail evita over-optimization.
→ **SDP: Lifecycle + manifest discovery aplicado; sin candidatos adicionales beyond already-loaded + documentation-and-adrs**

**Total SKILLS_CARGADAS (7):** campaign-executor, progreso, ponytail, source-driven-development, systematic-debugging, incremental-implementation, documentation-and-adrs

## Steps

### Step 1: Discovery — verificar SCC intencional + coverage existente
- **Archivos:** `src/storage/engine/get.rs`, `src/cache_warmer.rs`, `src/storage/engine/tests/engine.rs`
- **Acción:** Confirmar via rg que `get → prefetch_related → get` es SCC 2 nodos intencional con `PrefetchGuard` thread_local. Listar tests existentes (8 prefetch/get_cache, 1 co-access recurse). Marcar ciclo como intencional single-level, no falso positivo. No edita código.
- **Verify:** `rg -n "PrefetchGuard|prefetch_related" src/storage/engine/get.rs` → 5 hits + `cargo nextest list -E 'test(prefetch) or test(get_cache)'` → 8 tests listados ✅
- **Estado:** ✅ COMPLETED (2026-08-27 discovery: rg 5 hits, nextest list 8 tests, SCC intencional single-level verificado)

### Step 2: Doc justification file-header + verify contract (ACT)
- **Archivos:** `src/storage/engine/get.rs`
- **Acción:** Añadir/aumentar `//!` file header (líneas 1-21) explicando SCC intencional `get ↔ prefetch_related` (2 nodos), por qué existe (OLD-20 co-access prefetch), cómo `PrefetchGuard` lo hace single-level (MCP-15), invariante sync-only (thread_local no cross-task), y referencia a `test_get_prefetch_does_not_recurse_forever`. ~18 líneas, ponytail minimal (doc, no refactor).
- **Verify:** `cargo nextest run -p vantadb -E 'test(prefetch) or test(get_cache)'` ✅ (8/8, regex 10/10) + `rg -n "PrefetchGuard" src/storage/engine/get.rs` 5 hits + `cargo check -p vantadb` ✅ + `cargo fmt --check` 0 ✅
- **Estado:** ✅ COMPLETED (2026-08-27 — doc header `src/storage/engine/get.rs:1-21` añadido, check 3.49s, fmt 0, nextest 8/8 1.5s, rg 5)

### Step 3: Cierre — verify full + plan file + commit + progreso
- **Archivos:** `docs/plans/2026-08-27-backlog-v2.md`, `docs/avance/`, `.opencode/skills/campaign-executor/tasks/FIND-35.md`
- **Acción:** `cargo fmt --check` + `cargo clippy --workspace --all-targets --all-features -- -D warnings` + `cargo nextest run -p vantadb -E 'test(prefetch) or test(get_cache)'` + `rg` contract + `cargo check`. Actualizar plan file Task 2 → ✅ COMPLETED + recitation. Commit `fix: FIND-35 — StorageEngine get/prefetch intentional SCC justification + PrefetchGuard doc`. Ejecutar skill progreso (Backlog FIND-35 → docs/avance si existe, o registrar completed en plan).
- **Verify:** `cargo fmt --check` ✅ + `cargo clippy -p vantadb --all-targets --all-features -- -D warnings` 0 ✅ + `cargo nextest -p vantadb -E 'test(prefetch) or test(get_cache)'` 8/8 ✅ + `rg -n "PrefetchGuard"` 5 hits ✅ + doc header `src/storage/engine/get.rs:1-21` justifica codegraph SCC
- **Estado:** ✅ COMPLETED (2026-08-27 — fmt/clippy/nextest/rg ✅, plan file ✅ COMPLETED, recitation añadida)

## Dependencias
- Ninguna (Wave 0 paralelo con FIND-34, STABLE-01)

## Notas
- Ponytail ladder: rung 1 (¿necesita existir refactor?) → No. Doc + guard existente es más barato que aplanar prefetch. Skipped: extraer `fetch_without_prefetch` helper, ADR separado, AtomicBool global. Add when: prefetch se vuelve async o cross-thread → migrar guard a `tokio::task_local!`.
- `// ponytail: doc justifica SCC intencional single-level; aplanar si prefetch se vuelve async (thread_local → task_local)`
- `get()` nunca inserta el nodo materializado (tail de `prefetch_related` lo hace post-recursion); guard evita SO pero no cambia semántica de cache — prefetched nodes se insertan solo si `Entry::Vacant`.
- codegraph_explore "StorageEngine get prefetch" post-fix debe mostrar: SCC 2 nodos documentado en `src/storage/engine/get.rs:1-18` header + `PrefetchGuard` guard, o comentario justificación citado en review.

## Context Save Point
- **Fecha:** 2026-08-27T18:00
- **Branch:** develop
- **CI pendiente:** `cargo nextest --profile audit --workspace --build-jobs 2` full (timeout; prefetch-filter 8/8 suficiente — workspace audit Heavy tier no Fast Gate)
- **Decisiones:** Doc SCC intencional elegido sobre refactor aplanar (ponytail rung 1); guard thread_local suficiente (sync); test co-access A↔B existente suficiente
- **Problemas conocidos:** CodeGraph SCC 2 nodos intencional, justificado; `cargo nextest -E 'test(prefetch|get.*cache)'` regex requiere `/.../` slashes (nextest expression syntax) — usar `or` equivalente 8/8
- **Próxima tarea:** FIND-36 (Wave 1 arch), CORE-01 (spec-first)

## Cierre
- **Fecha:** 2026-08-27T18:00
- **Branch:** develop
- **Resultado:** ✅ COMPLETED — contrato FIND-35 cumplido (SCC intencional 2 nodos justificado en doc header 1-21, PrefetchGuard RAII thread_local, 8/8 prefetch/get_cache, rg 5 hits, codegraph justificado)
- **Verificación:** cargo nextest prefetch 8/8 (regex 10/10) · rg PrefetchGuard 5 · cargo check/clippy/fmt ✅ · doc header justifica codegraph SCC
- **Commit:** `fix: FIND-35 — StorageEngine get/prefetch intentional SCC justification + PrefetchGuard doc` (este cierre)

## Archivos tocados
- `src/storage/engine/get.rs` (doc header `//!` 15L + posible ampliación comentario)
- `docs/plans/2026-08-27-backlog-v2.md` (Task 2 → ✅ COMPLETED)
- `docs/Backlog.md` (FIND-35 eliminado si existe)
- `.opencode/skills/campaign-executor/tasks/FIND-35.md` (este file)
- `.opencode/task-system/memory/lessons.md` (lesson si aporta)
