# Task File: RES-01 — ACID Phase 4a: WAL v2 con WalRecord::Prepare (research)

**Plan:** docs/plans/2026-08-25-batch-core-fixes-research.md (Task 7)
**Estado:** 🔄 IN PROGRESS — research completo, doc persistido; falta routing Backlog + commit (lead)
**Tipo:** research (⬆️ uphill, Cynefin 🟧 complejo) — output DOC, no código
**Contrato:** doc de investigación generado en docs/research/ con análisis + plan; hallazgos ruteados a backlog si hay decisión

## Impacto mapeado (Regla 0)

- Leídos completos: `src/wal.rs` (1159L), `src/wal_sharded.rs` (855L), `src/storage/engine/txn.rs` (399L), `src/storage/engine/init.rs:400-618` (replay/MOD-02), `docs/architecture/adr/DRV-014-wal-batch-tradeoff.md`
- Referencias entrantes: replay usa ShardedWal layout (`init.rs:418-452`); commit usa batch_append (`txn.rs:158-160`)
- Referencias salientes: postcard, crc32c, binary_header::VantaHeader
- Veredicto: READ-ONLY sobre código core — único output es doc + task file. Cero archivos core tocados.

## Steps

1. ✅ DISCOVERY: código actual leído (WalRecord wal.rs:40-73, ShardedWal, commit_transaction txn.rs:119-191, replay MOD-02 init.rs:505-551)
2. ✅ DRV-014 revisado (clon por shard-grouping es tradeoff intencional cae92db3 — NO revertir)
3. ✅ Análisis commit point 1→2 fases (ganancias/costos)
4. ✅ Interacción MOD-02 evaluada: complementa; Prepare/Commit no-contiguos → recovery por outcome-map reemplaza skip-mask de slices
5. ✅ Doc redactado y persistido: `docs/research/res01-acid-wal-v2-prepare.md` (EN, 10 secciones)
6. ⬜ Lead: rutear a docs/Backlog.md → fila FIND (truthful-error gap: apply failure post-Commit resucita ops en restart, txn.rs:133-190) + fila ADR-humano Regla 5 (tradeoff +1 fsync/commit vs ACID semantics)
7. ⬜ Lead: verificar doc (secciones presentes + refs file:line spot-check) y commitear

## Hallazgos clave (digest)

- Commit point actual = durabilidad del batch `[Begin+ops+Commit]` vía un solo batch_append (txn.rs:146-160); MOD-02 da crash-atomicidad per-id en replay (init.rs:513-551).
- Gap real encontrado: si apply falla DESPUÉS del WAL-commit durable, el caller recibe Err pero el replay resucita los ops al reiniciar → errores no truthful.
- Prepare (redo-log prepare, sin locks ni coordinador): habilita errores truthful (Abort post-prepare), rollback multi-capa, watermark MVCC (`max_committed_txn`). Costo: ~2x syncs por commit (hoy ya sincroniza cada append, wal.rs:340-355), recovery dos pasadas, formato v2 (downgrade requiere dump/restore).
- Recomendación: GO condicional — S1-S2 detrás de flag `wal_prepare`, S5 bench canonical_p99 (Regla 9), ADR humano decide default-on.

## Context Save Point

Todo el trabajo está en `docs/research/res01-acid-wal-v2-prepare.md`. Si se interrumpe: solo quedan steps 6-7 (lead). No hay estado en memoria volátil.
