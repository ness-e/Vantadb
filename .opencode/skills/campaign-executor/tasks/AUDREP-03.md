# AUDREP-03: Errores de tombstone tragados — registros fantasma

## Metadata
- **Plan file:** docs/plans/2026-08-05-backlog-validation-actions.md (Phase 13)
- **Fuente:** docs/Backlog.md línea 457
- **Esfuerzo:** 🟡 2-4h
- **Prioridad:** 🔴
- **Tipo:** Rust (storage/engine)
- **Estado:** ⬜ PENDING

## Blast Radius
| Dirección | Módulos |
|-----------|---------|
| Callers | `StorageEngine::put/delete/update` (vstore write paths) |
| Callees | `vstore.read_header`, `vstore.write_header`, `BackendPartition::put`, `rebuild_hnsw_from_vstore` |
| Implicaciones | Si el tombstone falla silenciosamente, un registro zombie queda vivo y es reindexado como nodo vivo por `rebuild_hnsw_from_vstore`. Fix NO cambia API pública; agrega logging/retry. |

## Contrato
"`cargo check -p vantadb` pasa; `cargo clippy -p vantadb -- -D warnings` pasa; los 3 sitios `let _ = vstore.write_header` ya no tragan el error (propagan o loguean con `tracing::error!`); test unitario verifica que un fallo de write_header produce log/error y no zombie silencioso."

## Herramientas
- cargo-mcp (check, clippy, fmt), rust-analyzer-mcp, grep

## Steps
### Step 1: Investigar los 3 sitios + patrón de error
- **Archivos:** `src/storage/engine/ops.rs:424-427, 708-711, 1025-1028`
- **Acción:** leer cada sitio y cómo `vstore.write_header` reporta errores (`Result`), cómo el caller trata el error de `backend.put`/`delete`, y si existe `tracing` ya importado.
- **Verify:** lectura completa; no hay cambios aún.
- **Estado:** ⬜ PENDING

### Step 2: Aplicar fix en los 3 sitios
- **Archivos:** `src/storage/engine/ops.rs`
- **Acción:** reemplazar `let _ = vstore.write_header(...)` por manejo que no trague el error: mínimo `tracing::error!` (preferido: si el path ya retorna `Result`, propagar con `.map_err(...)?` si el contexto lo permite; si es recovery path, log + `debug_assert`). NO cambiar el flujo de negocio.
- **Verify:** `cargo check -p vantadb`
- **Estado:** ⬜ PENDING

### Step 3: Test + verificación
- **Archivos:** tests existentes de storage/engine si aplica
- **Acción:** agregar test (o ampliar existente) que fuerce fallo de write_header y confirme que no hay zombie silencioso (log o error). Si inyectar el fallo no es factible sin failpoints, documentar y cubrir con test del happy-path + grep del manejador.
- **Verify:** `cargo nextest run --profile audit -p vantadb --build-jobs 2 storage` + `cargo fmt --check` + `cargo clippy -p vantadb -- -D warnings`
- **Estado:** ⬜ PENDING

## Dependencias
- Ninguna.

## Notas
- Backlog: "Recomendación: `tracing::error!` + reintento o fallo explícito en los 3 sitios".
- Commit selectivo: SOLO `src/storage/engine/ops.rs` + tests tocados. No commitear otros archivos del árbol sucio.
