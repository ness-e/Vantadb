--
status: completed
title: "AUDREP-03: Errores de tombstone tragados â€” registros fantasma"
kind: task
description: "cargo check -p vantadb pasa; cargo clippy -p vantadb -- -D warnings pasa; los 3 sitios let = vstore.writeheader ya no tragan el error (propagan o loguean con tracing::error!); test unitario verifica que un fallo de writeheader produce..."
---

# AUDREP-03: Errores de tombstone tragados â€” registros fantasma

## Metadata
- **Plan file:** docs/dev/plans/2026-08-05-backlog-validation-actions.md (Phase 13)
- **Fuente:** docs/dev/Backlog.md lÃ­nea 457
- **Esfuerzo:** ðŸŸ¡ 2-4h
- **Prioridad:** ðŸ”´
- **Tipo:** Rust (storage/engine)
- **Estado:** â¬œ PENDING

## Blast Radius
| DirecciÃ³n | MÃ³dulos |
|-----------|---------|
| Callers | `StorageEngine::put/delete/update` (vstore write paths) |
| Callees | `vstore.read_header`, `vstore.write_header`, `BackendPartition::put`, `rebuild_hnsw_from_vstore` |
| Implicaciones | Si el tombstone falla silenciosamente, un registro zombie queda vivo y es reindexado como nodo vivo por `rebuild_hnsw_from_vstore`. Fix NO cambia API pÃºblica; agrega logging/retry. |

## Contrato
"`cargo check -p vantadb` pasa; `cargo clippy -p vantadb -- -D warnings` pasa; los 3 sitios `let _ = vstore.write_header` ya no tragan el error (propagan o loguean con `tracing::error!`); test unitario verifica que un fallo de write_header produce log/error y no zombie silencioso."

## Herramientas
- cargo-mcp (check, clippy, fmt), rust-analyzer-mcp, grep

## Steps
### Step 1: Investigar los 3 sitios + patrÃ³n de error
- **Archivos:** `src/storage/engine/ops.rs:424-427, 708-711, 1025-1028`
- **AcciÃ³n:** leer cada sitio y cÃ³mo `vstore.write_header` reporta errores (`Result`), cÃ³mo el caller trata el error de `backend.put`/`delete`, y si existe `tracing` ya importado.
- **Verify:** lectura completa; no hay cambios aÃºn.
- **Estado:** â¬œ PENDING

### Step 2: Aplicar fix en los 3 sitios
- **Archivos:** `src/storage/engine/ops.rs`
- **AcciÃ³n:** reemplazar `let _ = vstore.write_header(...)` por manejo que no trague el error: mÃ­nimo `tracing::error!` (preferido: si el path ya retorna `Result`, propagar con `.map_err(...)?` si el contexto lo permite; si es recovery path, log + `debug_assert`). NO cambiar el flujo de negocio.
- **Verify:** `cargo check -p vantadb`
- **Estado:** â¬œ PENDING

### Step 3: Test + verificaciÃ³n
- **Archivos:** tests existentes de storage/engine si aplica
- **AcciÃ³n:** agregar test (o ampliar existente) que fuerce fallo de write_header y confirme que no hay zombie silencioso (log o error). Si inyectar el fallo no es factible sin failpoints, documentar y cubrir con test del happy-path + grep del manejador.
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2 storage` + `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings`
- **Estado:** â¬œ PENDING

## Dependencias
- Ninguna.

## Notas
- Backlog: "RecomendaciÃ³n: `tracing::error!` + reintento o fallo explÃ­cito en los 3 sitios".
- Commit selectivo: SOLO `src/storage/engine/ops.rs` + tests tocados. No commitear otros archivos del Ã¡rbol sucio.
