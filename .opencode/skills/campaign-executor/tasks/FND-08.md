# FND-08 — Backend validado contra patrón de acceso real + auditoría de compactación

- **Plan:** docs/plans/2026-08-16-wave-p20-tsys.md (P20a)
- **Estado:** ✅ COMPLETED
- **Tipo:** audit + docs (config backend, ADR, regla)
- **Prio/Effort:** 🟡 / 🟡

## Objetivo

Verificar que la config de compactación del backend (Fjall default, RocksDB
opt-in) está tuneada para el patrón real de VantaDB — escrituras pequeñas
frecuentes + reads random — y no con defaults pensados para escritura
secuencial masiva (bulk-load). Complementa FND-02 (locks multi-índice ya
commiteado).

**DoD:** ADR-023 numerado tras ADR-022 existe (o diferimiento justificado en el
reporte); config justificada **o** regla agregada a `.opencode/rules/durability.md`;
bench de lectura random compila o referencia existente; `cargo check -p vantadb` pasa.

## Archivos clave

- `src/backends/fjall_backend.rs` (open:54-57 — defaults totales)
- `src/backends/rocksdb_backend.rs` (open:32-142 — bloom/cache/memtable; sin nivel)
- `src/storage/engine/init.rs` (dispatch backends:269-289)
- `benches/backend_compare.rs` (random_get:70-94, seed 42)
- `.opencode/rules/durability.md` (regla a agregar)
- `docs/architecture/adr/ADR-023-backend-compaction.md` (nuevo)
- `docs/Investigaciones/FND-08-backend-compaction.md` (nuevo)

## Impacto mapeado (Regla 0)

- `src/backends/fjall_backend.rs` — LEÍDO completo (codegraph, 457 líneas). 15 callers (tests propios + init.rs). NO se modifica.
- `src/backends/rocksdb_backend.rs` — LEÍDO completo (503 líneas). NO se modifica.
- `src/storage/engine/init.rs` — LEÍDO completo (587 líneas). NO se modifica.
- `benches/backend_compare.rs` — LEÍDO completo (170 líneas). Verificado que compila con `--features rocksdb` (cargo check 2026-08-16 ✅). Se referencia, no se crea bench nuevo (ponytail: existe > nuevo).
- `.opencode/rules/durability.md` — LEÍDO (12 líneas). Se agrega regla (edición aditiva, sin romper referencias).
- ADR nuevo: `ADR-023-backend-compaction.md` — no rompe nada; numeración verificada (022 es último).
- Reporte nuevo en `docs/Investigaciones/` — no rompe nada.
- Referencias entrantes a durability.md: lazy-load desde `.opencode/AGENTS.md` tabla → sin impacto de edición aditiva.
- Veredicto: ediciones SOLO aditivas (ADR + regla + reporte + task file). Cero cambios de código. Sin git (lead commitea).

## Steps

- [x] S1 DISCOVERY: codegraph config backends + pattern de acceso real (get.rs:79, write_batch, text_index postings) + benches existentes
- [x] S2 Validación web: docs.rs fjall DatabaseBuilder/KeyspaceCreateOptions + source fjall-3.1.8 (cache_size default=32MiB db_config.rs:90, memtable 64MiB, block 4KiB, compaction Leveled) + RocksDB tuning guide (dynamic_level_bytes, bloom)
- [x] S3 Análisis/clasificación: gap-real / marginal / ok (ver reporte §Análisis)
- [x] S4 Escribir task file + Impacto mapeado
- [x] S5 Escribir ADR-023 (Context/Decision/Consequences, evidencia archivo:línea)
- [x] S6 Agregar regla a `.opencode/rules/durability.md` (config backend = patrón de acceso documentado)
- [x] S7 Escribir reporte `docs/Investigaciones/FND-08-backend-compaction.md`
- [x] S8 Verify: `cargo check -p vantadb` (contrato) + bench referenciado compila (verificado S1)

## Context Save Point

- Commit: NINGUNO (el lead commitea — instrucción explícita de la tarea).
- Backlog/plan/AUD-024/verify-log/AGENTS.md: NO tocados.
- Deuda: cambio de config (cache_size Fjall, dynamic_level_bytes RocksDB) DIFERIDO — requiere bench con dataset > 32MiB para justificarlo (Regla 9). Señal de reapertura en ADR-023 §Consequences.