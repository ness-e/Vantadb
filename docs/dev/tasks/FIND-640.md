---
id: FIND-640
title: "Atomicidad batch-wide de put_batch entre chunks: transacción WAL implícita (Begin/Prepare/Commit)"
status: in_progress
type: feat
plan: docs/dev/plans/archive/2026-11-07-consolidacion-tecnica-pre-v0100.md (Task 54)
---

# FIND-640: Atomicidad batch-wide de `put_batch` entre chunks — Diseño de transacción WAL implícita

## Metadata
- **Plan file:** `docs/dev/plans/archive/2026-11-07-consolidacion-tecnica-pre-v0100.md` (Task 54)
- **Fuente:** `docs/dev/Backlog.md` (`FIND-640`; corrige la fila `FIND-221`)
- **Esfuerzo estimado (diseño):** 🟠 ~1 semana de implementación (dentro del appetite 2sem)
- **Prioridad:** 🟠
- **Cynefin:** 🟥 complejo → tras el análisis, 🟨 complicado: la infraestructura 2PC YA EXISTE; el trabajo es plomería, no protocolo nuevo
- **Creado:** 2026-11-24 (fase de diseño, vanta-arch)
- **Estado:** 📐 DISEÑO LISTO PARA IMPLEMENTACIÓN
- **ADRs:** ADR-0060 (compat on-disk, citado); se propone ADR-0061 para esta decisión

## Veredicto ejecutivo

**IMPLEMENTABLE en appetite 2 semanas. SIN cambio de formato on-disk. Compatible con ADR-0060.**

El descubrimiento central del análisis: **el protocolo de transacción WAL ya está implementado end-to-end** (WAL v2 / RES-01 / TXN-01): `WalRecord::{Begin, Prepare, Commit, Abort}` existen en el enum desde antes de `v0.9.0`, el recovery de `init.rs` ya descarta lotes sin `Commit` (skip_mask + open_txn, `init.rs:764-818`) y ya ejecuta un *undo pass* físico que purga KV/HNSW/vstore de transacciones no commiteadas (`init.rs:820-964`, TXN-01 Capa B). **No hay que diseñar ningún marcador nuevo ni tocar el recovery.** FIND-640 se reduce a enmarcar `batch_insert_with_opts` dentro de una transacción implícita con id propio y sostener `insert_lock` durante todo el lote.

## 1. Estado actual (verificado en código)

### 1.1 Flujo de `batch_insert_with_opts` hoy (`src/storage/engine/insert.rs:895-1043`)

```
validate_batch_elements(nodes)          // FASE 1: validación total (FIND-221)
for chunk in nodes.chunks(1000):        // FASE 2
    probe pre_existing (backend.get por nodo, para rollback)
    { acquire insert_lock }             // guard POR CHUNK
        serialize_batch_nodes()         // → vstore mmap append + stage KV ops + HNSW entries
        append_batch_wal()              // → WalRecord::Insert × chunk (sin marcador txn)
        write_batch_kv_or_tombstone()   // → backend.write_batch (KV commit del chunk)
        insert_hnsw_leveled()           // → HNSW bulk
        applied_records.push(...)       // bookkeeping para rollback in-process
on Err: rollback in-process (HNSW remove, vstore tombstone, KV restore/delete, cache evict)
post-commit: stats (rayon|serial), cache Hot, eviction, maybe_auto_flush
```

**Brecha:** cada chunk es una unidad durable independiente. Crash entre chunk *k* y *k+1* → los `Insert` WAL del prefijo se re-aplican en replay → prefijo visible tras reabrir. El rollback in-process no corre post-mortem (documentado en `memory.rs:1433-1441` como residual FIND-640).

### 1.2 Infraestructura 2PC existente (la pieza que falta usar)

| Pieza | Ubicación | Qué hace |
|-------|-----------|----------|
| `WalRecord::{Begin,Prepare,Commit,Abort}` | `wal.rs:144-187` | Marcadores de txn. **Idénticos en v0.9.0** (`git show v0.9.0:src/wal.rs:135-152`) — mismo orden de variantes, mismo postcard |
| `commit_transaction` (patrón de referencia) | `txn.rs:283-486` | `[Begin+ops+Prepare]` un `batch_append` → apply a stores → `Commit` (commit point de durabilidad) → `Abort` en error |
| Recovery: descarte de txn sin Commit | `init.rs:764-818` | `open_txn` tracking sobre global_seq ordenado; Begin nuevo cierra txn incompleta; EOF con txn abierto → descarta prefijo; Abort → descarta rango |
| Recovery: **undo pass físico** | `init.rs:820-964` | Para cada registro descartado con `txn_id != 0`: si `meta.created_by_txn == txn_id` → `backend.delete` + `hnsw.remove_node` + tombstone vstore (vía `storage_offset_of` o scan) |
| `TxnManager::next_txn_id` | `txn.rs:51,66-70` | AtomicU64 compartido — asignador de ids único del motor |
| `checkpoint_seq` | `maintenance.rs:83-97` | `wal.total_record_count()` — cuenta TODOS los registros incl. marcadores; flush toma `insert_lock` (ERR-010) |

### 1.3 Por qué el recovery ya funciona para este caso — y su condición

La lógica `open_txn` (`init.rs:766-772`) asume que los registros de una txn ocupan **slots de global_seq contiguos** ("written by a single batch_append"). Con multi-shard WAL, `batch_append` reserva `len` slots contiguos atómicamente (`wal_sharded.rs:551`). La contigüidad multi-chunk se mantiene **si y solo si ningún otro writer intercala registros entre los `batch_append` de nuestros chunks** — y eso se garantiza sosteniendo `insert_lock` durante TODO el lote (toda mutación de WAL pasa por `insert_lock` desde FIND-651/ERR-010). Intercalar un `Insert` simple dentro del rango de la txn abierta lo atribuiría a nuestro `txn_id` en `record_txn_id` y lo descartaría junto al prefijo → pérdida de un registro ajenamente commiteado. **La decisión de diseño #1 (lock único) es por tanto un requisito de corrección, no una opción.**

## 2. Diseño: transacción implícita por batch

### 2.1 Contrato resultante

`batch_insert_with_opts` con WAL habilitado (`!opts.skip_wal && self.wal.is_some()`):

- **Commit point:** `WalRecord::Commit(batch_txn_id)` durable. Todo lo anterior es preparación; todo lo posterior es epílogo.
- **Crash antes del Commit durable** → recovery descarta `[Begin..]` + undo pass purga KV/HNSW/vstore aplicados → **0 registros del lote tras reabrir**.
- **Crash después del Commit durable** → replay re-aplica los `Insert` (idempotente, upsert por id) → lote completo tras reabrir.
- **Error in-process antes de Commit** → rollback existente + `Abort(batch_txn_id)` best-effort → `Err` veraz (nada commiteado, nada resucita).
- **`opts.skip_wal = true`** → comportamiento actual, sin frame de txn (la atomicidad no se puede garantizar sin WAL; queda documentado — `skip_wal` es opt-in explícito para bulk-load regenerable).
- **Lote ≤ 1000 (un chunk):** mismo frame; overhead = 3 registros extra (~40-120 bytes). Aceptable; no se bifurca el código.

### 2.2 Secuencia (ops-first, dos fases WAL/apply bajo UN solo `insert_lock`)

```
let batch_txn = self.txn.alloc_implicit_id();            // id único, NO registrado en active
let execution_result = (|| {
    let _guard = self.acquire_insert_lock("batch txn frame")?;   // UN guard para TODO el lote

    // ── Fase WAL (toda la verdad durable primero) ──
    self.wal_coordinator().append(&WalRecord::Begin(batch_txn))?;
    for chunk in nodes.chunks(BATCH_CHUNK_SIZE) {
        let records: Vec<WalRecord> = chunk.iter()
            .map(|n| WalRecord::Insert(stamp(n, now_ms))).collect();
        sharded.batch_append(records)?;                  // slots contiguos por lock+round-robin
    }
    self.wal_coordinator().append(&WalRecord::Prepare {
        txn_id: batch_txn, op_count: nodes.len() as u32 })?;

    // ── Fase Apply (mismo guard; per-chunk como hoy) ──
    for chunk in nodes.chunks(BATCH_CHUNK_SIZE) {
        probe pre_existing; serialize_batch_nodes();     // vstore + stage
        write_batch_kv_or_tombstone(kv_ops stamped batch_txn);
        insert_hnsw_leveled(); applied_records.push(...);
    }
    self.wal_coordinator().append(&WalRecord::Commit(batch_txn))?;  // COMMIT POINT
    Ok(())
})();
// guard liberado al salir del closure
if let Err(e) = execution_result {
    rollback in-process (existente, adquiere insert_lock propio);
    let _ = self.wal_coordinator().append(&WalRecord::Abort(batch_txn));  // best-effort
    return Err(e);
}
post-commit: stats / cache / eviction / auto-flush (sin cambios)
```

**Por qué ops-first y no interleaved `[ops_k, apply_k]`:** un crash durante la fase WAL deja cero trabajo aplicado en stores (undo pass trivialmente vacío); un crash durante apply encuentra TODAS las ops durables y el undo pass las purga por `created_by_txn`. Es el espejo exacto de `commit_transaction` fase-1/fase-2 (RES-01), maximizando el reuso del camino de recovery ya auditado.

**Por qué `Prepare` aunque el recovery no lo exija:** uniformidad con el invariante RES-01 ("toda txn multi-op emite Prepare"); coste = 1 registro; `op_count` queda como cross-check de integridad futuro. `Abort` en la rama de error es igualmente opcional pero barato y simétrico con `commit_transaction`.

### 2.3 Cambios de código requeridos

| Archivo | Sección | Cambio |
|---------|---------|--------|
| `src/storage/engine/txn.rs` | `impl TxnManager` (~:65) | **Nuevo método** `alloc_implicit_id() -> u64`: `next_txn_id.fetch_add(1, Relaxed)` SIN insertar en `active`. CRÍTICO: si se usara `begin()`, `insert()` vería `has_active()==true` y bufferizaría escrituras ajenas dentro del buffer de la txn implícita |
| `src/storage/engine/insert.rs` | `build_kv_put_op` (:430), `stage_one_node` (:448), `serialize_batch_nodes` (:463), `append_batch_wal` (:340) | Threading de `stamp_txn: u64` — el `created_by_txn` del `NodeMetadata` pasa de `txn.snapshot_id()` (id compartido, inseguro para undo) al `batch_txn` dedicado. Sin el sello dedicado el undo pass no puede distinguir entradas del lote de inserts ajenos |
| `src/storage/engine/insert.rs` | `batch_insert_with_opts` (:895-1043) | Reestructura §2.2: `alloc_implicit_id`, un solo `insert_lock`, fase WAL completa, fase apply por chunks, `Commit`; en error `Abort` tras el rollback existente. Mantener `applied_records`/`all_staged_offsets` y el rollback tal cual |
| `src/storage/engine/insert.rs` | dentro del loop de apply | **Nuevo failpoint** `put_batch_chunk_commit_fail` (feature `failpoints`) entre chunks para los tests del DoD (`wal_append_fail` existente no permite apuntar al 2º chunk limpiamente desde la API) |
| `src/sdk/api/memory.rs` | doc-comment de `put_batch` (:1410-1448) | Eliminar el residual "Crash window in Stage B"; documentar el nuevo contrato (Commit point, recovery discard, residual upsert §4.3/R3, exposición Stage C sin cambios) |
| `src/wal.rs` | — | **NINGÚN cambio.** Marcadores, framing, chain hash y versiones intactos |
| `src/storage/engine/init.rs` | — | **NINGÚN cambio.** Recovery ya implementa el descarte + undo |
| tests | `tests/storage/` (nuevo `batch_atomicity.rs`) o módulo de tests de `memory.rs` | tests del plan §3 |

### 2.4 `checkpoint_seq`, rotación y WAL sharded

- **`checkpoint_seq`:** cuenta `total_record_count()` incl. marcadores; recovery los salta en el match de replay (`init.rs:1053-1059`). Un batch aporta `N+3` registros (`+4` con Abort). **Ningún flush puede caer a mitad de lote**: `flush_locked` toma `insert_lock` (ERR-010) y el lote lo sostiene completo → checkpoint ve el lote entero o nada. Implicación: ninguna.
- **Auto-rotate mid-batch:** `try_auto_rotate` puede rotar un segmento entre chunks (Begin en rotado, Commit en activo). El recovery enumera segmentos rotados en orden cronológico y `open_txn` persiste a lo largo del stream ordenado → cubierto (CRASH-01, `init.rs:703-709`). Verificar con test §3.
- **Sharded WAL:** `batch_append` por chunk mantiene `verify_shard_counts` equilibrado (round-robin atómico); el `Commit`/`Begin`/`Prepare` por `append` toman un slot global cada uno. Contigüidad garantizada por `insert_lock`. Verificar con `wal_shards=4` en test.
- **WAL cifrado (v4):** los marcadores son registros ordinarios; el framing AEAD/chain es agnóstico al payload. Sin impacto.
- **SyncMode:** con `Never`, el `Commit` puede vivir solo en page cache → perderlo en crash = descarte del lote (dirección fail-safe correcta: nunca se resucita un lote sin Commit durable). Con `Always`/`Periodic` el `Commit` fsynca según política.

### 2.5 RAM (pre-mortem #2 — resuelto)

- Fase WAL: `Vec<WalRecord>` de **un chunk** (≤1000 clones de `UnifiedNode`) por `batch_append`, liberado por iteración.
- Fase apply: `BatchStageBufs` por chunk (como hoy), `applied_records`/`all_staged_offsets` del lote completo (como hoy — sin incremento).
- `nodes: &[UnifiedNode]` ya está materializado por `put_batch_inner` (sin incremento).
- **Neto:** el diseño NO retiene el lote entero en estructuras adicionales; el pico es idéntico al actual ±1 chunk de clones WAL.

## 3. Plan de tests

Contrato del plan: `cargo nextest run --profile audit -p vantadb --features failpoints --locked --build-jobs 2 -E "test(put_batch_multi_chunk_atomic)"`

| Test | Mecánica | Aserción |
|------|----------|----------|
| `put_batch_multi_chunk_atomic_commit_visible` | 2500 records (3 chunks), `put_batch` OK, close+reopen | 2500 registros; WAL contiene exactamente 1×Begin/Prepare/Commit con el mismo `txn_id` |
| `put_batch_multi_chunk_atomic_fail_second_chunk` | failpoint `put_batch_chunk_commit_fail` (nuevo) armado `1*return` en el apply del 2º chunk | `Err` del `put_batch`; WAL contiene `Abort(batch_txn)`; reopen → 0 registros del lote |
| `put_batch_multi_chunk_atomic_crash_no_commit` | `wal_append_fail` armado para fallar en el `append` del `Commit` (o failpoint dedicado `put_batch_commit_marker_fail`) | `Err`; reopen → 0 registros del lote (skip_mask descarta `[Begin..Prepare]` + undo purga chunks aplicados) |
| `put_batch_multi_chunk_atomic_panic_mid_apply` | failpoint `put_batch_chunk_commit_fail` con acción `panic` en el 2º chunk; `catch_unwind` en el test; drop engine; reopen | 0 registros del lote; recovery sin panic; `wal_records_replayed` excluye el lote |
| `put_batch_multi_chunk_atomic_crash_before_apply` | failpoint que falla el `batch_append` del chunk 2 de la fase WAL | `Err`; reopen → 0 registros; undo pass sin trabajo (nada aplicado) |
| `put_batch_multi_chunk_atomic_sharded` | misma mecánica del test crash con `Config { wal_shards: 4 }` | 0 registros tras reopen; `verify_shard_counts` OK |
| `put_batch_multi_chunk_atomic_rotation` | forzar `try_auto_rotate` (límite de segmento pequeño) entre chunk 1 y 2; crash-sim post-apply parcial | recovery recorre segmentos rotados; 0 registros |
| `put_batch_multi_chunk_atomic_checkpoint_boundary` | `flush()` entre dos `put_batch` | `checkpoint_seq` nunca parte un frame de txn; replay coherente |
| `put_batch_multi_chunk_atomic_upsert_residual` | upsert de clave pre-existente + crash pre-Commit | registro AUSENTE tras reopen (documenta R3 — no restaura la versión previa) |
| `put_batch_multi_chunk_atomic_concurrent_insert` | `insert()` concurrente durante `put_batch` (threads) | el insert ajeno nunca queda dentro del frame de la txn ni se pierde tras crash+reopen |

**Benchmark:** correr benches de batch existentes (`benches/`) antes/después; gate <10% de regresión. Coste esperado: 3 frames pequeños + `stamp_txn` threading + lock sostenido (sin efecto en bench single-threaded).

## 4. Riesgos y mitigaciones

| # | Riesgo | Prob×Imp | Mitigación |
|---|--------|----------|------------|
| R1 | `insert_lock` sostenido durante todo el lote (hoy: por chunk) — writes concurrentes bloqueados más tiempo | 🟡×🟠 | `put_batch` es API bulk ya serializada de facto; el timeout `insert_lock_timeout_ms` da error tipado retriable. Documentar. Si la contención se mide, follow-up: recovery multi-txn abierta (HashMap<txn_id, rangos>) — NO necesario ahora |
| R2 | Deadlock por lock-ordering | 🟢×🔴 | Jerarquía preservada: dentro del guard solo WAL/vstore/KV/HNSW (igual que hoy por chunk); `apply_batch_stats_*` sigue post-commit fuera del guard (Rule 8, mismo orden que `commit_transaction` FIND-345) |
| R3 | **Residual upsert:** crash pre-Commit sobre registro pre-existente → undo pass hace `backend.delete` → se pierde la versión ANTERIOR (el WAL no lleva el valor viejo). Mismo comportamiento que una txn explícita abortada hoy (TXN-01) | 🟡×🟠 | Documentar como residual en doc-comment + ADR-0061. DoD usa registros nuevos. Fix real = WAL log del valor previo → backlog |
| R4 | **Compound downgrade:** crash mid-batch en v0.10 + open con v0.9.0. v0.9.0 tiene skip_mask (descarta WAL) pero NO tiene undo pass (ausente en `git show v0.9.0:init.rs`) → prefijo aplicado queda en KV y `get()` no chequea MVCC → prefijo visible | 🟢×🟠 | Evento compuesto (crash Y downgrade). Documentar en adenda de ADR-0060 §D5 como salvedad de rollback. v0.9.0 no es parchable retroactivamente |
| R5 | Intercalado de writers dentro del frame (bypass futuro de `insert_lock`) | 🟢×🔴 | Invariante FIND-651: TODA mutación WAL bajo `insert_lock`. Comentario-assert en `batch_insert_with_opts` |
| R6 | `Abort` append falla tras rollback | 🟢×🟢 | Fail-safe: prefijo sin Commit se descarta igual por regla EOF del recovery |
| R7 | Regresión perf >10% | 🟡×🟠 | Benchmark gate del plan; lock sostenido no afecta bench single-writer; 3 frames extra ~0.1% bytes |

## 5. Compatibilidad con ADR-0060 — veredicto explícito

- **¿Cambio de formato? NO.** Cero variantes nuevas en `WalRecord`, cero cambio de framing, `WAL_FORMAT_VERSION=3` / `WAL_POSTCARD_VERSION=1` intactos, chain hash v3 intacto.
- **¿Retrocompatible con WALs v0.9.0? SÍ, en ambas direcciones:**
  - v0.10 abriendo WAL v0.9.0: recovery idéntico (los marcadores ya existían y ya se manejaban).
  - v0.9.0 abriendo WAL v0.10 con frames de batch: el enum `WalRecord` de v0.9.0 es **byte-idéntico** (verificado `git show v0.9.0:src/wal.rs:135-152`) y su recovery ya tiene skip_mask/open_txn (v0.9.0 `init.rs:511-556`). Excepción documentada: R4 (sin undo pass en v0.9.0).
- **¿Migración? NO.** Los WALs nuevos solo *usan* más los marcadores existentes.
- **Stop condition del plan NO se dispara.** La fila de inventario de ADR-0060 para `WAL_FORMAT_VERSION` queda igual.
- **Acción documental:** ADR-0061 nuevo (transacción implícita por batch, ops-first, sello `created_by_txn`, residuales R3/R4) + adenda de 1 párrafo en ADR-0060 §D5 registrando R4.

## 6. Estimación por hito

| Hito | Contenido | Esfuerzo |
|------|-----------|----------|
| M1 — Engine plumbing | `alloc_implicit_id`, threading `stamp_txn`, reestructura `batch_insert_with_opts`, Abort path, failpoint nuevo | 2-3 días |
| M2 — Tests | 10 tests §3 (failpoints, reopen, sharded, rotation, concurrent) | 2-3 días |
| M3 — Bench + docs | benchmark gate 10%, doc-comments, ADR-0061 + adenda ADR-0060 | 1-2 días |
| **Total** | | **~1 semana** (holgado dentro de 2sem) |

## 7. Checklist de verificación

- [x] Contrato tipado: sin cambios de firma pública (`batch_insert_with_opts`, `BatchInsertOptions` intactos); threading interno de `stamp_txn`
- [x] `#[non_exhaustive]` / enums públicos: sin nuevos
- [x] Sin `panic`/`unwrap` en hot path: Abort best-effort, undo pass existente ya es best-effort
- [x] Ciclos de módulos: ninguno nuevo (insert.rs ya usa `wal::WalRecord`, `txn.rs` mismo crate)
- [x] ADR-0060: cumplido — sin cambio de formato (§5)
- [ ] Pendiente en implementación: verificar visibilidad de entradas `created_by_txn` post-Commit en `get()`/`search` (equivalente a commit_transaction hoy — Commit es la frontera, no MVCC)
- [ ] ADR-0061 a escribir en M3

## Definition of Done (heredado del plan Task 54)

| Nivel | Gate |
|-------|------|
| Task | `put_batch_multi_chunk_atomic_*` PASS (2500 con failpoint en 2º chunk → Err + 0 registros tras reabrir; panic mid-commit → recovery sin registros) + benchmark sin regresión >10% + diseño en ADR |
| Commit | `feat(storage): batch-wide atomic put_batch across chunks` |
| Release | `put_batch` all-or-nothing real en v0.10.0; `FIND-221` pasa a ✅ |
