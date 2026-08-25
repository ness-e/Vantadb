# MOD-04 — purge_expired O(N) full-scan → índice TTL selectivo

> **Plan:** 2026-08-25-batch-core-server-mcp.md · **Estado:** ✅ COMPLETO · **Cynefin:** 🟨 complicado

## Contrato
- bench before/after `purge_expired` con N records expirados documentado (comando exacto)
- tests existentes TTL pasan
- `cargo nextest run -p vantadb` verde · `cargo check` · fmt · clippy archivos tocados

## Archivos clave
- `src/sdk/api.rs:898-963` (`purge_expired`)
- `src/scalar_index.rs`
- `src/node/*`

## Spec

### Problema
`purge_expired()` hace `engine.scan_nodes()` (O(N)): lee metadata + clona el f32 vector de
**todos** los nodos (incluso no-TTL / no-expirados) para luego filtrar `FIELD_EXPIRES_AT_MS`.

### Hallazgo DISCOVERY (verificación real)
- `StorageEngine` persistente mantiene `scalar_index` en writes: `insert.rs:177-218`,
  `delete.rs:113-115`, `maintenance.rs:807` (vacuum). Indexa **todos** los campos
  relacionales, incluido `__vanta_expires_at_ms` cuando el record tiene TTL
  (`memory_record_to_node_owned` → `set_field(FIELD_EXPIRES_AT_MS, Int)`).
- **Gaps:** (a) `ScalarIndex.lookup` solo igualdad, no range `<= now`;
  (b) `scalar_index` se crea VACÍO en `init.rs:141` y **no se reconstruye** en
  `recover_state` (escribe directo vía `replay_write_node`) → tras reopen está vacío.

### Diseño (ponytail — reusar ScalarIndex, no duplicar índice)
1. `ScalarIndex::lookup_int_le(field, max)` → node_ids con `Int(v) <= max` (range sobre índice existente).
2. `StorageEngine::scalar_lookup_int_le(field, max)` accessor.
3. `StorageEngine::rebuild_scalar_index()` → repuebla desde backend Default (relational, sin clonar vector).
   Llamado en `open_with_config` (init.rs) tras construir el engine.
4. `rebuild_index` (api.rs:723) → añade `rebuild_scalar_index()`.
5. `purge_expired` reescrito: candidatos = `scalar_lookup_int_le(FIELD_EXPIRES_AT_MS, now)`,
   luego leer SOLO el metadata del candidato (`get_from_partition` + `NodeMetadata`, sin vector/cache)
   y construir record (misma extracción de campos). NOTA: el intento con `engine.get()` por candidato
   REGRESABA (overhead fijo cache/governor/vector) — el metadata-only es el que gana.

## Pasos (steps)
1. ✅ **PLAN** — Bench `benches/purge_expired.rs` (N total, E expirados) + entry Cargo.toml. Medir **ANTES**.
2. ✅ **ACT** — `ScalarIndex::lookup_int_le` + 2 tests unitarios.
3. ✅ **ACT** — `StorageEngine::scalar_lookup_int_le` + `rebuild_scalar_index`; llamar en init.rs open y en rebuild_index.
4. ✅ **ACT** — Reescribir `purge_expired` (api.rs) usando el índice (metadata-only).
5. ✅ **VERIFY** — Medir **DESPUÉS**: −23.9% (100 exp) / −9.1% (1000 exp), p<0.05. Tests TTL + nextest 2060/2060 crate + 2763/2763 workspace + check + fmt + clippy.
6. ✅ **CIERRE** — Task file + recitation + RESULTADO.

## Resultado (Regla 9 — before/after documentado)

**Comando exacto del bench:**
```
cargo bench --bench purge_expired -- --warm-up-time 1 --measurement-time 3 --sample-size 12
```
Bench file: `benches/purge_expired.rs` · entrada `[[bench]] name = "purge_expired"` en Cargo.toml.
Dataset: 4_000 records con vector 128d, E expirados (ttl=1ms ya lapsed), backend InMemory, put individual.

| Shape | ANTES (full scan) | DESPUÉS (índice + metadata-only) | Δ |
|-------|-------------------|----------------------------------|---|
| total_4000_expired_100 | 137.20 ms | 117.22 ms | **−23.9%** (p<0.05) |
| total_4000_expired_1000 | 1.2341 s | 1.0726 s | **−9.1%** (p<0.05) |

Intento intermedio con `engine.get()` por candidato REGRESABA (+11%/+24%, overhead fijo de
get: cache, quantization governor, vector clone) → reemplazado por lectura metadata-only del
backend (`get_from_partition` + `deserialize_node_payload<NodeMetadata>`), que es estrictamente
menos trabajo que `scan_nodes()` y nunca más lento.

**Entorno:** Windows, CPU AVX2, 12 cores, 31GB RAM, `target\release` bench profile.

## Hallazgo colateral (FIND-31, pre-existente)
`purge_expired` tras reopen falla con "text index df would go negative" si un expirado tiene
payload indexado — reproducido con código ORIGINAL (git stash) y con mi cambio. NO causado por
MOD-04; registrado en Backlog como FIND-31. Fix: reconstruir text index en reopen o guard de
deltas en purge.

## Impacto mapeado (Regla 0)
- **Leídos completos:** api.rs (purge_expired 906-1164, put/put_one/put_batch 112-187/217/237, delete 503, rebuild_index 723, namespace_stats 1557, insert_node 40, put_record_exact 532), scalar_index.rs (completo), storage/engine/{insert,delete,maintenance,init,partition,ops,get,mod}.rs, engine.rs (InMemory), sdk/serialization/mod.rs, impl_rebuild.rs, node/unified.rs, version_history.rs, types.rs.
- **Referencias hacia dentro (scalar_index):** insert.rs:177,214,602,622,664,687 · delete.rs:113,230 · maintenance.rs:807 · init.rs:141 · mod.rs:361 · tests/init.rs:441, engine.rs:633.
- **Referencias entrantes (purge_expired):** api.rs (def), version_history.rs:446 (test), builder.rs:215 (purge_expired_threads usa store, NO este).
- **Referencias hacia fuera de ScalarIndex:** solo `filter_field` (InMemoryEngine engine.rs:411) — el StorageEngine persistente NO lee scalar_index hoy (write-only).
- **Veredicto:** cambio interno (additivo). `lookup_int_le` nuevo método público del módulo `pub(crate)`; `rebuild_scalar_index` método nuevo de StorageEngine; `purge_expired` mantiene misma semántica (mismo filtro `now > expires`, misma construcción de record). No cambia API pública de bindings.

## Context Save Point
- Todos los steps ✅. Verify completo: nextest -p vantadb 2060/2060, nextest --workspace 2763/2763, fmt ok, clippy -p vantadb y --workspace ok, docs-coverage 0 gaps. NO commit (regla: el lead verifica y commitea).
- Archivos tocados (solo los míos): `src/scalar_index.rs`, `src/sdk/api.rs`, `src/storage/engine/init.rs`, `src/storage/engine/mod.rs`, `src/storage/engine/tests/init.rs`, `benches/purge_expired.rs`, `Cargo.toml` (bench entry), `tests/fuzz_proptest.rs` (revertido a original — el test de reopen se quitó por FIND-31), `docs/Backlog.md` (FIND-31), task file.
- Hallazgo FIND-31 queda en Backlog para el lead (bug text index tras reopen, pre-existente).
