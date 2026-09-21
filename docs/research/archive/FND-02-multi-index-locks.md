# FND-02: Regla de coordinación multi-índice + auditoría de deadlocks y contención

**Estado:** ✅ Resuelto (fixes aplicados + tests + regla normativa)
**Fecha:** 2026-08-16
**Prioridad:** 🔴 (P20a)
**Fuente:** docs/Backlog.md:484
**Alcance:** paths de escritura multi-índice (vector + grafo + text) en `src/storage/engine/`
**Archivos tocados:** `maintenance.rs`, `insert.rs`, `get.rs`, `tests/ops.rs`, `.opencode/rules/concurrency-async.md`

---

## 1. Objetivo

Fijar el orden de locks de los paths de escritura multi-índice y eliminar los puntos de
deadlock/contención por reentrancia de locks no reentrantes (`parking_lot::FairMutex` de
`insert_lock` y `parking_lot::RwLock` de `volatile_cache`). Salida: regla normativa en
`.opencode/rules/concurrency-async.md` (Regla 8) + fixes + tests.

## 2. Mapa de locks por path (orden global)

Orden global (writers): `cardinality_stats → insert_lock → {wal, pending_hnsw_batch, hnsw (RCU), vstore, volatile_cache, backend}`.

| Path | Locks que toma (en orden) | Ubicación |
|---|---|---|
| `insert` → `apply_insert` | cardinality_stats, insert_lock, wal shard, pending_hnsw_batch, hnsw (ArcSwap), volatile_cache.write, backend.put | `insert.rs:289-328` |
| `batch_insert` | igual que insert | `insert.rs:820-854` |
| `delete` | insert_lock, hnsw.remove, scalar.clear_many, edge.remove_all_for_node, text, backend | `delete.rs` |
| `delete_batch` | insert_lock, por nodo: hnsw.remove + scalar/edge/text + backend | `delete.rs:193` |
| `commit_transaction` | cardinality_stats, insert_lock, wal shard, hnsw, vstore, volatile_cache, backend | `txn.rs:119` |
| `flush` | insert_lock (checkpoint ERR-010) | `engine/ops.rs` |
| `evict_cold_nodes_with_reason` | insert_lock (vía consolidate → refresh_index), volatile_cache.read (candidatos) + volatile_cache.write (remove) | `maintenance.rs:470` |
| `get` | volatile_cache.try_write (o read), hnsw (RCU), vstore.read | `get.rs:53-75` |
| `get_many` | volatile_cache.try_write (o read), hnsw (RCU), vstore.read | `get.rs:231` |
| text index (put_one) | SDK layer, SIN insert_lock | `sdk/api.rs:111` → `impl_index.rs:147` |

Read paths nunca toman `insert_lock` → sin inversión de orden entre threads. El text index se
actualiza en capa SDK sin retener `insert_lock` → sin inversión vector/text.

## 3. Hallazgos

### 🔴 H1 — Reentrancia de `insert_lock` en la evicción (deadlock-riesgo, timeout silencioso)

- **Síntoma:** `apply_insert` (`insert.rs:306` original) y `batch_insert` (`insert.rs:834`
  original) llamaban `evict_cold_nodes_with_reason` **mientras sostenían `insert_lock`**
  (parking_lot::FairMutex, NO reentrante). La evicción → `consolidate_node` →
  `refresh_index` re-adquiría `insert_lock` con `try_lock_for(insert_lock_timeout_ms=5000)`
  → timeout de 5s **por candidato**; al fallar, el `.is_ok()` del loop descartaba el
  candidato y la evicción fallaba **silenciosamente** (evicted=0) con ~5s×N de bloqueo.
- **Severidad:** 🔴 — degradación silenciosa + latencia punta de 5s×N bajo pressure de cache.

### 🔴 H2 — Deadlock real: write guard de `volatile_cache` retenido durante la evicción

- **Síntoma:** los mismos call sites llamaban a la evicción **dentro del scope del guard
  `volatile_cache.write()`** (`insert.rs:290`, `insert.rs:821` originales). La evicción
  toma `volatile_cache.read()` (candidatos, `maintenance.rs:373`) y `volatile_cache.write()`
  (remove, `maintenance.rs:344`) → parking_lot::RwLock no es reentrante → **deadlock del
  mismo thread** (peor que H1: bloquea indefinidamente, no hay timeout).
- **Por qué no se veía en producción:** la evicción solo se dispara con `cache.len() > max_nodes`
  (≈ RAM/4 / 1536B) — raro en workloads pequeños.
- **Severidad:** 🔴 — deadlock duro cuando se alcanza el watermark.

### 🟡 H3 — Contención en `get_many`: write lock bloqueante en path de lectura

- **Síntoma:** `get_many` (`get.rs:228` original) tomaba `volatile_cache.write()` bloqueante
  en el path de lectura masiva; un writer activo serializa TODOS los readers batch detrás de él.
- **Contraste:** `get()` ya usaba el patrón ERR-036 (`try_write` → fallback `read`,
  `get.rs:53-75`).
- **Severidad:** 🟡 — contención, no deadlock.

### 🟡 H4 — DashMap full-shard sweep en delete paths (documentar, no fixear)

- `scalar_index.remove_node` usa `DashMap::iter_mut()` (lockea todos los shards) y
  `edge_index.remove_all_for_node` usa `DashSet::retain`. Costoso bajo delete masivo, pero
  funcionalmente correcto; requiere benchmark antes de rediseñar (Regla 9). **Pendiente de
  rediseño — documentado, no fixeado.**

### ✅ OK — verificado, sin problema

- `evict_cold_nodes` desde `stats.rs:144,166` (MemoryGovernor/watermark): standalone, sin
  guards → usan la variante pública correcta.
- `try_push_pending_hnsw`/`drain_hnsw_batch_locked` (`ops.rs:143,170`) y `insert_to_cf`
  (`ops.rs:224`, único caller standalone de `refresh_index`): correctos.
- Orden de locks sin inversión entre threads (read paths no toman insert_lock).

## 4. Fixes aplicados

### Fix 1 — `maintenance.rs`: variantes `*_locked` + `apply_index_entry_unlocked`

- `apply_index_entry_unlocked` (`maintenance.rs:250`): cuerpo del refresh SIN adquirir
  `insert_lock`. Preserva la conversión `MmapFull→Full` (owned) que es requisito previo al
  `release_mmap_vector` — el refresh NO es un no-op.
- `refresh_index` público (`maintenance.rs:276`): lock + `apply_index_entry_unlocked`.
  Semántica idéntica para callers standalone (`insert_to_cf`, tests).
- `consolidate_node_inner(node, lock_held)` (`maintenance.rs:298`): si `lock_held` →
  `apply_index_entry_unlocked`, si no → `refresh_index`.
- `consolidate_node` público (`maintenance.rs:368`, lock_held=false) + `pub(crate)`
  `consolidate_node_locked` (`maintenance.rs:376`, lock_held=true).
- `evict_cold_nodes_inner(ratio, reason, lock_held)` (`maintenance.rs:390`):
  consolida con la variante según `lock_held`.
- `evict_cold_nodes_with_reason` público (`maintenance.rs:470`) + `pub(crate)`
  `evict_cold_nodes_with_reason_locked` (`maintenance.rs:482`).
- **API pública sin cambios** — solo se añaden variantes `pub(crate)`.

### Fix 2 — `insert.rs`: soltar el guard de cache + usar `_locked` (H1 + H2)

- `apply_insert` (`insert.rs:295-327`): el guard `volatile_cache.write()` queda limitado al
  cálculo de `needs_eviction`; la evicción corre DESPUÉS de soltarlo y usa
  `evict_cold_nodes_with_reason_locked` (`insert.rs:317`).
- `batch_insert` (`insert.rs:829-856`): mismo patrón (`insert.rs:851`).

### Fix 3 — `get.rs`: `get_many` con try_write (H3, ERR-036)

- `get_many` (`get.rs:231`): `try_write()` → bump de hits/last_accessed cuando no hay
  contención; bajo contención fallback a `volatile_cache.read()` (stats no bumpadas).
  Mismo contrato que `get()`.

## 5. Tests agregados (`tests/ops.rs`)

| Test | Qué valida |
|---|---|
| `test_evict_cold_nodes_locked_no_reentrant_timeout` (ops.rs:1257) | Sostiene `insert_lock`, llama `evict_cold_nodes_with_reason_locked(1.0, Manual)` con 4 nodos Hot en cache: completa < 1s (antes: 5s×4 de timeout) y evicta > 0 (antes: 0). |
| `test_multi_index_write_paths_no_deadlock` (ops.rs:1279) | Stress concurrente: 4 writers (insert + get_many), 1 deleter (`delete_batch`), 1 evictor (standalone), con watchdog `recv_timeout(30s)` que FALLA el test (no cuelga CI) si hay deadlock. |

Resultado: 296 tests del módulo `storage::engine::tests` pasan (71 en ops), 2 de integración
`core_invariants` pasan.

## 6. Regla normativa

`.opencode/rules/concurrency-async.md` → **Regla 8 — Coordinación multi-índice: orden global
de locks y variantes `*_locked` (FND-02)**:
- orden global de locks para writers;
- prohibición de re-adquirir `insert_lock` y de llamar a métodos que toman
  `volatile_cache` mientras se retiene su write guard;
- `try_write` en read paths (ERR-036);
- obligación de variantes `*_locked` para paths dentro de secciones de insert;
- el refresh HNSW en consolidación no es un no-op.

## 7. Deuda pendiente / documentado

- **H4** (DashMap `iter_mut` / DashSet `retain` en delete): rediseño requiere benchmark
  (Regla 9). No es deuda nueva de este PR.
- Los `Mutex` de índices no-Arc inventariados por FND-19 (`flat.rs:64`, `scann.rs:51-59`,
  `diskann.rs:48-51`) quedan serializados por `insert_lock` — sin ratio leído/escrito que
  justifique RwLock sin medir (documentado, no cambiado).
- **Saldo de deuda:** negativo — los fixes eliminan deuda (timeout 5s×N y deadlock de RwLock)
  sin introducir nueva.

## 8. Verificación

```bash
cargo check -p vantadb                        # ✅ pasa
cargo test -p vantadb --lib storage::engine::tests  # ✅ 296 ok
cargo test -p vantadb --test core_invariants  # ✅ 2 ok
```

Pendiente para el gate de cierre (lead): `cargo fmt --check`, `cargo clippy`, y la auditoría
concurrente obligatoria (Regla 8 AGENTS.md → vanta-chaos/vanta-review, P2-01 — el
implementador no se auto-audita).

---

## 9. Cierre follow-up FND-02-M3 — race delete ↔ consolidate_node

**Estado:** ✅ Resuelto
**Fecha:** 2026-08-16
**Fuente:** `.opencode/skills/campaign-executor/tasks/FND-02-M3.md` (apertura P2-01 del audit FND-02)

### Race mapeado

`delete()` (delete.rs) corre su sección crítica completa bajo `insert_lock`: WAL +
`apply_delete_inner` (remove HNSW) + `backend.delete` + remove de `volatile_cache`.
`consolidate_node_inner(lock_held=false)` (maintenance.rs, path público de eviction)
hacía `backend.put` FUERA del lock y re-aplicaba la entrada HNSW recién en
`refresh_index` (adquiría `insert_lock` solo en ese punto). Interleavings:

- **A (zombie):** delete corre entre `backend.put` y el refresh HNSW → nodo en HNSW sin
  metadata en backend (ghost index entry, rompe invariante `in_index ⇒ in_backend`).
- **B (resurrección):** delete completa antes de consolidate → el nodo eliminado vuelve a
  persistirse (backend.put) y re-indexarse (refresh HNSW).

El path `lock_held=true` (eviction desde insert) ya estaba serializado: el caller retiene
`insert_lock` durante la evicción (insert.rs:317, batch_insert.rs:851).

### Fix aplicado (maintenance.rs `consolidate_node_inner`)

1. **Lock completo:** cuando `lock_held=false`, adquiere `insert_lock` al inicio de la
   función (patrón `try_lock_for` + `VantaError::Timeout`, igual que delete.rs) y lo
   retiene durante TODA la sección crítica (backend.put + HNSW + release mmap + cache
   remove). Cierra el interleaving A: delete y consolidate se serializan.
2. **Version check bajo lock:** si el nodo ya no está en `volatile_cache` (delete() lo
   remueve bajo el mismo insert_lock) → `return Ok(())` sin persistir ni re-indexar.
   Cierra el interleaving B: consolidación de un nodo ya eliminado es no-op.
   - Check contra `volatile_cache`, NO contra HNSW: `batch_insert` con
     `InsertMode::Rebuild` (o `Auto` ≥ threshold) mete nodos Hot en el cache sin
     indexarlos (insert.rs:834-837) — un check HNSW los dejaría atrapados en cache Hot
     sin consolidar (memory leak). `delete()` remueve de ambos bajo el mismo lock, así
     que la presencia en cache es la señal de liveness correcta para ambos paths.
3. **`apply_index_entry_unlocked` en ambas ramas:** el lock está retenido en los dos
   paths (caller o nuestro), nunca se re-adquiere el no-reentrante `insert_lock`.
   `refresh_index` público queda intacto (callers standalone: `insert_to_cf`, tests).

### Tests agregados (tests/ops.rs)

| Test | Qué valida |
|---|---|
| `test_delete_vs_consolidate_no_resurrection` | Determinista (interleaving B): insert Hot → snapshot candidato → `delete(id)` → `consolidate_node(&stale_candidate)` → no metadata en backend, no entrada HNSW, `get(id)` None. Fallaría antes del fix (resurrección). |
| `test_delete_vs_evict_concurrent_no_zombie` | Stress (A+B): deleter + 2 evictors públicos sobre 32 nodos Hot, watchdog `recv_timeout(30s)`; invariante final: ningún nodo eliminado queda en HNSW ni en backend. |

### Verificación

```bash
cargo check -p vantadb   # ✅
cargo test -p vantadb --lib test_delete_vs_consolidate_no_resurrection   # ✅
cargo test -p vantadb --lib test_delete_vs_evict_concurrent_no_zombie    # ✅
cargo test -p vantadb --lib test_evict_cold_nodes_locked_no_reentrant_timeout  # ✅ (regresión)
cargo test -p vantadb --lib test_multi_index_write_paths_no_deadlock     # ✅ (regresión)
```

API pública sin cambios; invariante mmap intacto (la entrada HNSW se re-aplica antes del
release, `apply_index_entry_unlocked` preserva la conversión `MmapFull→Full`).