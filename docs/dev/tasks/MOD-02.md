# MOD-02 — Transacciones no crash-atómicas (H-2)

## Impacto mapeado (Regla 0)

**Archivos leídos completos:**
- `src/storage/engine/txn.rs` (399L) — commit_transaction escribe batch `[Begin, ops..., Commit]` vía `batch_append`; fallback phase-1 escribe Commit suelto.
- `src/wal_sharded.rs` (848L) — `batch_append` agrupa por shard y sincroniza shard-por-shard (crash entre shards = batch parcial durable). El `fetch_add` atómico asigna rangos contiguos de slots ⇒ batches de txns NUNCA se intercalan en orden global-seq.
- `src/storage/engine/init.rs:384-586` — `recover_state`: replay real del StorageEngine. YA tiene skip-mask (desde `610b5d0a`): descarta txns sin Commit.
- `src/engine.rs:140-193` — replay de InMemoryEngine ignora markers, pero InMemoryEngine nunca escribe Begin/Commit (solo Insert/Delete/Update planos) ⇒ sin bug real allí.

**Hallazgo clave:** el review H-2 citaba `engine.rs:162-165`, pero ese es el path InMemory que jamás recibe markers. El path real (`init.rs::recover_state`) ya descarta txns incompletas. **Hueco residual real:** `skip_mask[start..]` al finalizar salta hasta EOF — si tras el crash quedaron durables registros de OTRO writer con slot posterior al batch incompleto, se pierden silenciosamente.

**Referencias entrantes:** `init.rs` llamado desde `StorageEngine::open*`; nadie más consume los markers.
**Referencias salientes:** `WalRecord::{Begin,Commit,Abort}` (definidos en `wal.rs`), `verify_shard_counts`.

**Veredicto:** fix acotado a `init.rs` (bound del skip al extent de la txn) + tests de chaos/regression en `tests/`. Sin cambios de API pública. ERR-010 (checkpoint↔snapshot) no se toca.

## Spec

Replay debe garantizar atomicidad crash: ops de una txn se aplican solo si existe `Commit(txn_id)` posterior en orden global-seq; txns parciales/abortadas se descartan SIN perder registros durables ajenos.

## Steps

- ✅ S1: Fix en `init.rs` — skip-mask bound al extent de la txn. Reemplazó el `txn_end` no-op (equivalente a `[start..]`) por tracking per-id `open_txn: Option<(u64, usize)>`: el Commit/Abort cierra solo el txn que matchea por id, y un Begin posterior acota el extent de una txn incompleta previa (`skip_mask[start..i]`); txn abierta al EOF → skip fail-safe `[start..]`. Fix real vs. el aplicado por el run anterior (que era funcionalmente idéntico al original).
- ✅ S2: Tests chaos/regression en `src/storage/engine/tests/init.rs`:
  - kill entre shards: `[Begin, ops...]` durable sin Commit → tras reopen NO están (8, 9) + nodo pre-crash (7) sobrevive vía backend.
  - control positivo: `[Begin, ops, Commit]` completo → tras reopen ESTÁN (10, 11).
  - colateral: txn parcial `[Begin(300), op, op]` seguida de batch completo ajeno `[Begin(301), op, Commit(301)]` → ops de la parcial descartadas (12, 13), registro ajeno posterior sobrevive (14).
  - Crash-sim via el handle WAL del propio engine (`engine.wal`) — el intento previo escribía en `dir.path()/vanta.wal` pero el engine usa `data_dir/vanta.wal` (subdir `data/`), por eso el replay leía shards vacíos.
  - Tests con backend real (default Fjall), gate `#[cfg(any(feature = "fjall", feature = "rocksdb"))]` — InMemory jamás llama recover_state.
- ✅ S3: Verify: `cargo nextest run -p vantadb init:: + txn` (45/45), recovery/durability (22/22), chaos failpoints (1/1), crash_injection (1/1, 218s) + durability_recovery (7/7), `cargo check -p vantadb`, fmt + clippy limpios.

## Context Save Point

Tarea COMPLETA (2026-08-24). Cambios en worktree sin commitear (lead commitea): `src/storage/engine/init.rs` (fix recover_state) + `src/storage/engine/tests/init.rs` (3 tests MOD-02). Sin archivos fuera del blast radius declarado. Contrato cumplido: "tests txn + chaos pasan; replay respeta Commit marker". No se tocaron wal.rs, vector/, storage/ (excepto init.rs + tests). Fix verificado: per-id matching, no-op `txn_end` eliminado. Nota: `test_records_after_partial_txn_survive_recovery` reescrito — el registro ajeno posterior debe ir en un batch completo propio (Begin/Commit) para ser distinguible; un registro plano sin markers inmediatamente después de una txn parcial es indistinguible y se descarta (fail-safe, ceiling documentado).
