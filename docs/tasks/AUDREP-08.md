# AUDREP-08: WalArchiver::archive_segment — colisión de timestamps + rename no atómico

## Metadata
- **Plan file:** docs/plans/2026-08-05-backlog-validation-actions.md (Phase 13)
- **Fuente:** docs/Backlog.md línea 462
- **Esfuerzo:** 🟡 2-4h
- **Prioridad:** 🔴
- **Tipo:** Rust (WAL)
- **Estado:** ⬜ PENDING

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `WalArchiver::archive_segment` (rotation path, archiving) |
| Callees | `std::fs::remove_file`, `std::fs::rename`, `web_time::SystemTime`, PITR recovery (`parse_segment_timestamp`) |
| Implicaciones | Dos segmentos archivados en <1ms colisionan en el nombre `{file}.{millis}` → `remove_file` borra el archivo previo + rename no atómico → pérdida de datos de WAL. Fix: nombre único (nanos+nonce o UUID) + `tempfile`+rename atómico. Debe mantener compat con `parse_segment_timestamp` (wal_archiver.rs:283-293) para PITR. |

## Contrato
"`cargo check -p vantadb` pasa; `cargo clippy -p vantadb -- -D warnings` pasa; archive_segment genera nombres únicos en <1ms (test de colisión); no hay `remove_file` + `rename` no atómico para el destino; `parse_segment_timestamp` sigue parseando los nombres archivados; tests WAL existentes pasan."

## Herramientas
- cargo-mcp (check, clippy, fmt, test), rust-analyzer-mcp, grep

## Steps
### Step 1: Investigar naming + parseo PITR
- **Archivos:** `src/wal_archiver.rs:76-88`, `src/wal_archiver.rs:283-293` (parse_segment_timestamp), callers
- **Acción:** leer cómo se genera el nombre, cómo `parse_segment_timestamp` extrae el timestamp (formato esperado del sufijo), y quién llama a `archive_segment` (¿puede ser concurrente?).
- **Verify:** comprensión completa; sin cambios aún.
- **Estado:** ⬜ PENDING

### Step 2: Aplicar fix de unicidad + atomicidad
- **Archivos:** `src/wal_archiver.rs`
- **Acción:** mínimo que resuelve el hallazgo: (a) unicidad — usar `as_nanos()` + contador/nonce o UUID corto en el sufijo (manteniendo el formato parseable por `parse_segment_timestamp`; si cambia el formato, actualizar el parser), y (b) atomicidad — escribir el segmento a un path temporal en el mismo filesystem y `rename` atómico (no borrar destinos existentes para "crear espacio"). Usar la stdlib; no añadir dep nueva si no hace falta (ponytail).
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Test de colisión + verificación
- **Archivos:** tests de wal_archiver existentes
- **Acción:** test que archiva 2 segmentos en el mismo ms (o llama 2× con nombre origen distinto y tiempo virtual igual) y confirma destinos únicos, ambos persistidos, y `parse_segment_timestamp` los lee.
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2 wal` + `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings`
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna.

## Notas
- Backlog: "Recomendación: usar nanosegundos + sufijo UUID, o `tempfile` + atomic rename".
- Commit selectivo: SOLO `src/wal_archiver.rs` + tests tocados.
