--
status: completed
title: "AUDREP-08: WalArchiver::archive_segment â€” colisiÃ³n de timestamps + rename no atÃ³mico"
kind: task
description: cargo check -p vantadb pasa; cargo clippy -p vantadb -- -D warnings pasa; archivesegment genera nombres Ãºnicos en <1ms (test de colisiÃ³n); no hay removefile + rename no atÃ³mico para el destino; parsesegmenttimestamp sigue parseando los...
---

# AUDREP-08: WalArchiver::archive_segment â€” colisiÃ³n de timestamps + rename no atÃ³mico

## Metadata
- **Plan file:** docs/dev/plans/2026-08-05-backlog-validation-actions.md (Phase 13)
- **Fuente:** docs/dev/Backlog.md lÃ­nea 462
- **Esfuerzo:** ðŸŸ¡ 2-4h
- **Prioridad:** ðŸ”´
- **Tipo:** Rust (WAL)
- **Estado:** â¬œ PENDING

## Blast Radius
| DirecciÃ³n | MÃ³dulos |
|-----------|---------|
| Callers | `WalArchiver::archive_segment` (rotation path, archiving) |
| Callees | `std::fs::remove_file`, `std::fs::rename`, `web_time::SystemTime`, PITR recovery (`parse_segment_timestamp`) |
| Implicaciones | Dos segmentos archivados en <1ms colisionan en el nombre `{file}.{millis}` â†’ `remove_file` borra el archivo previo + rename no atÃ³mico â†’ pÃ©rdida de datos de WAL. Fix: nombre Ãºnico (nanos+nonce o UUID) + `tempfile`+rename atÃ³mico. Debe mantener compat con `parse_segment_timestamp` (wal_archiver.rs:283-293) para PITR. |

## Contrato
"`cargo check -p vantadb` pasa; `cargo clippy -p vantadb -- -D warnings` pasa; archive_segment genera nombres Ãºnicos en <1ms (test de colisiÃ³n); no hay `remove_file` + `rename` no atÃ³mico para el destino; `parse_segment_timestamp` sigue parseando los nombres archivados; tests WAL existentes pasan."

## Herramientas
- cargo-mcp (check, clippy, fmt, test), rust-analyzer-mcp, grep

## Steps
### Step 1: Investigar naming + parseo PITR
- **Archivos:** `src/wal_archiver.rs:76-88`, `src/wal_archiver.rs:283-293` (parse_segment_timestamp), callers
- **AcciÃ³n:** leer cÃ³mo se genera el nombre, cÃ³mo `parse_segment_timestamp` extrae el timestamp (formato esperado del sufijo), y quiÃ©n llama a `archive_segment` (Â¿puede ser concurrente?).
- **Verify:** comprensiÃ³n completa; sin cambios aÃºn.
- **Estado:** â¬œ PENDING

### Step 2: Aplicar fix de unicidad + atomicidad
- **Archivos:** `src/wal_archiver.rs`
- **AcciÃ³n:** mÃ­nimo que resuelve el hallazgo: (a) unicidad â€” usar `as_nanos()` + contador/nonce o UUID corto en el sufijo (manteniendo el formato parseable por `parse_segment_timestamp`; si cambia el formato, actualizar el parser), y (b) atomicidad â€” escribir el segmento a un path temporal en el mismo filesystem y `rename` atÃ³mico (no borrar destinos existentes para "crear espacio"). Usar la stdlib; no aÃ±adir dep nueva si no hace falta (ponytail).
- **Verify:** `cargo check -p vantadb`
- **Estado:** â¬œ PENDING

### Step 3: Test de colisiÃ³n + verificaciÃ³n
- **Archivos:** tests de wal_archiver existentes
- **AcciÃ³n:** test que archiva 2 segmentos en el mismo ms (o llama 2Ã— con nombre origen distinto y tiempo virtual igual) y confirma destinos Ãºnicos, ambos persistidos, y `parse_segment_timestamp` los lee.
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2 wal` + `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings`
- **Estado:** â¬œ PENDING

## Dependencias
- Ninguna.

## Notas
- Backlog: "RecomendaciÃ³n: usar nanosegundos + sufijo UUID, o `tempfile` + atomic rename".
- Commit selectivo: SOLO `src/wal_archiver.rs` + tests tocados.
