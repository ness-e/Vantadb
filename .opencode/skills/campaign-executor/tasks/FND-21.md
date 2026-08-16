# FND-21 — ADRs retroactivos de decisiones ya tomadas (Fjall vs RocksDB, zero-copy Arrow, WAL async/batch)

- **Plan:** docs/plans/2026-08-16-wave-r2-r7-fnd.md (Task 10, Wave 2)
- **Estado:** ✅ COMPLETED
- **Tipo:** documentation (ADRs retroactivos)
- **Prio/Effort:** 🟡 / 🟢

## Objetivo

Escribir ADRs retroactivos para: (a) Fjall vs RocksDB como backend por defecto,
(b) zero-copy Arrow en bindings, (c) WAL async/batch. Complementa FND-12 (método).
**DoD:** 3 ADRs en `docs/architecture/adr/` con Contexto/Decisión/Consecuencias.

## Archivos clave

- `docs/_templates/adr.md` (plantilla)
- `docs/architecture/adr/` — ADRs existentes: `004_storage_backend.md`,
  `DRV-014-wal-batch-tradeoff.md`, `DRV-015-wal-async-roadmap.md`,
  `019_sparse_vector_persisted_format.md` (ADR-019 tomado)
- `Cargo.toml` (features: fjall default, rocksdb opt-in)
- `src/config.rs` (backend_kind default), `src/storage/engine/init.rs` (dispatch)
- `src/columnar.rs` (Arrow RecordBatch), `vantadb-wasm/src/lib.rs` (PERF-08)
- `src/wal.rs`, `src/wal_sharded.rs` (batch-append por shard)

## Impacto mapeado (Regla 0)

- Archivos nuevos en `docs/architecture/adr/` — no rompen referencias.
- (a) y (c) ya cubiertos por ADR 004 y DRV-014/DRV-015 → los ADRs nuevos son de
  consolidación/retro-documentación con evidencia archivo:línea, **no duplicados**
  (referencian los existentes en frontmatter `related:` y en secciones).
- (b) no tenía ADR → ADR nuevo genuino.
- Números libres verificados: ADR-020/021/022 (ADR-019 ya ocupado).

## Steps

- [x] S1 DISCOVERY: numeración (grep dir), evidencia en código (Cargo.toml:97,
      config.rs:582-598, init.rs:269-289, columnar.rs:22, wasm lib.rs:1428-1447,
      wal.rs:297/340/342/358, wal_sharded.rs:9-14/191/198-218), estado bindings
      Python/Node (sin Arrow → FND-04 pendiente)
- [x] S2 Escribir task file + Impacto mapeado
- [x] S3 Escribir 3 ADRs (ADR-020 consolidación backend, ADR-021 Arrow nuevo,
      ADR-022 consolidación WAL) — inglés, plantilla, evidencia archivo:línea
- [x] S4 Verify contrato: 3 ADRs con Context/Decision/Consequences/Status +
      numeración sin colisión + evidencia citada

## Context Save Point

- Commit: NINGUNO (el lead commitea al cerrar la wave — Regla plan wave R2-R7).
- Backlog/plan: NO tocados.