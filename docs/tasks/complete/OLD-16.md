# OLD-16: WAL Rotation at 256MB

## Metadata
- **Plan file:** N/A (direct backlog task)
- **Fuente:** `docs/Backlog.md` Phase 9 línea 180
- **Esfuerzo:** 🟢 1d
- **Prioridad:** 🟢
- **Tipo:** Rust
- **Turns estimados:** 5-10
- **Creado:** 2026-07-26T16:30
- **last-synced:** 2026-07-26T16:30
- **Estado:** ✅ COMPLETED (2026-07-26, verificado batch 6: `try_auto_rotate()` en `src/wal.rs:393`, default 256MB :255, 3 tests)

## Blast Radius

| Dirección | Módulos |
|-----------|---------|
| Callers | `src/engine.rs` (ShardedWal usage), `src/storage/wal.rs` (init_wal), tests |
| Callees | `src/wal.rs` (WalWriter, WalHeader), `src/wal_sharded.rs` (ShardedWal), `src/wal_archiver.rs` (optional archive) |
| Implicaciones | Escritores de WAL existentes reciben auto-rotación sin cambios de API (additive). `bytes_written` ya trackeado. No rompe contratos. |

## Contrato
"`cargo nextest run --profile audit -p vantadb` pasa (tests WAL existentes + 3 nuevos), `cargo clippy -p vantadb -- -D warnings` pasa"

## Herramientas necesarias
- cargo-mcp (check, clippy, fmt)
- codegraph_explore (blast radius)

## Investigation Notes

### Código existente
- `WalWriter` (`src/wal.rs:177`): tiene `bytes_written: u64`, `rotate()` (consume self), `record_count`, `sync_mode`.
- `WalWriter::rotate()` (L352): sync → drop → rename `vanta.wal` → `vanta.wal.<timestamp>` → open fresh.
- `ShardedWal` (`src/wal_sharded.rs:9`): tiene `append()`, `batch_append()`, `rotate_all()`, `flush_all()`.
- `WalArchiver` (`src/wal_archiver.rs`): `archive_segment()`, `list_archived_segments()`, `WalArchiveConfig`.

### Lo que falta
- Nadie chequea `bytes_written` contra un límite. El WAL crece hasta que alguien llama a `rotate_all()` explícitamente.
- `ShardedWal::rotate_all()` se llama desde el engine... pero con qué frecuencia? No hay schedule.

### Approach (ponytail)
1. Agregar `max_segment_size: u64` a `WalWriter` (default: 256MB = 268435456)
2. Agregar `try_auto_rotate(&mut self) -> Result<bool>` — si `bytes_written >= max_segment_size`, rota inline (no consume self)
3. Llamar `try_auto_rotate()` al final de `append()` y `batch_append()`
4. Agregar `max_segment_size` a `ShardedWal::append()`/`batch_append()` via delegación
5. Exponer config en `VantaConfig` (opcional)

## Steps

### Step 1: Add `max_segment_size` to WalWriter + `try_auto_rotate()`
- **Archivos:** `src/wal.rs`
- **Acción:** Agregar campo `max_segment_size: u64` (default 256MB). Implementar `try_auto_rotate(&mut self) -> Result<bool>` que verifica bytes_written y rota inline si excede.
- **Verify:** `cargo check -p vantadb`

### Step 2: Call `try_auto_rotate()` in append/batch_append
- **Archivos:** `src/wal.rs`
- **Acción:** Al final de `append()` y `batch_append()`, llamar a `self.try_auto_rotate()?`.
- **Verify:** `cargo check -p vantadb`

### Step 3: Thread max_segment_size through ShardedWal
- **Archivos:** `src/wal_sharded.rs`
- **Acción:** Nada — `ShardedWal::append()`/`batch_append()` ya delegan a `self.shards[idx].lock().append(record)`. Si WalWriter tiene auto-rotate, ShardedWal lo hereda gratis.
- **Verify:** `cargo check -p vantadb`

### Step 4: Write tests
- **Archivos:** `src/wal.rs` (agregar tests en `#[cfg(test)]`)
- **Acción:** 3 tests: `test_auto_rotate_triggers_at_limit`, `test_auto_rotate_not_before_limit`, `test_auto_rotate_preserves_data`. Usar un `max_segment_size` pequeño (como 1KB) en tests.
- **Verify:** `cargo nextest run --profile audit -p vantadb -- wal`

### Step 5: fmt + clippy + verify
- **Acción:** `cargo fmt && cargo clippy -p vantadb -- -D warnings && cargo nextest run --profile audit --workspace --build-jobs 2`
- **Verify:** All pass.

## Dependencias
- Ninguna

## Notas
- Ponytail: no agregar config externa ni feature flag. Default 256MB es razonable. Si alguien quiere cambiarlo, se agrega después.
- `try_auto_rotate()` no consume self — inlinea la lógica de rotate pero preserva la referencia.
- Asegurar que el archivo rotado tenga el header escrito correctamente.
